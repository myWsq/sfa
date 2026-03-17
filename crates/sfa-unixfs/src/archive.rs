use std::collections::{BTreeSet, HashMap, HashSet};
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crossbeam_channel::{SendError, bounded};
use rayon::prelude::*;
use sfa_core::archive::{ArchiveReader, prepare_archive, write_archive};
use sfa_core::codec::{decode_data, encode_data};
use sfa_core::config::{FrameHashAlgo, PackConfig, RestoreOwnerPolicy, UnpackConfig};
use sfa_core::format::{FeatureFlags, TrailerV1};
use sfa_core::integrity::{frame_hash, trailer_hash};
use sfa_core::model::{EntryKind as CoreEntryKind, PlannerInputEntry};
use sfa_core::{
    EncodedFrame, FrameHeaderV1, ObservedMetric, PackPhaseBreakdown, PackStats,
    UnpackPhaseBreakdown, UnpackStats, plan_archive,
};

use crate::diagnostics::{UnpackDiagnostics, UnpackDiagnosticsCollector};
use crate::error::UnixFsError;
use crate::path::ensure_safe_relative_path;
use crate::restore::{
    ConcurrentFileWriter, EntryMetadata, LocalRestorer, PreparedRegularFile, RestorePolicy,
    RestoreTarget, Restorer, prepare_regular_descriptor,
};
use crate::scan::{EntryKind, ScannedEntry, scan_tree};

const UNTRUSTED_MARKER_NAME: &str = ".sfa-untrusted";

#[derive(Debug, Clone)]
struct BundleExtent {
    entry_id: u32,
    file_offset: u64,
    raw_offset_in_bundle: u32,
    raw_len: u32,
}

type BundleExtents = Arc<[BundleExtent]>;

#[derive(Debug)]
struct FrameWorkItem {
    header: FrameHeaderV1,
    payload: Vec<u8>,
    extents: BundleExtents,
}

#[derive(Debug, Default, Clone, Copy)]
struct DecodePhaseTotals {
    decode_ms: u64,
}

#[derive(Debug)]
struct DecodedBundleTask {
    bundle_id: u64,
    raw: Vec<u8>,
    extents: BundleExtents,
}

#[derive(Debug, Default, Clone, Copy)]
struct ScatterPhaseTotals {
    scatter_ms: u64,
}

#[derive(Debug, Default)]
struct UnpackPipelineOutcome {
    total_raw_bytes: u64,
    total_encoded_bytes: u64,
    trailer_input: Vec<u8>,
    frame_read_ms: u64,
    decode_ms: u64,
    scatter_ms: u64,
}

#[cfg(test)]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct UnpackProbeSnapshot {
    effective_worker_count: usize,
    worker_threads_started: usize,
    decode_calls: usize,
}

#[derive(Debug, Default)]
struct UnpackProbe {
    effective_worker_count: AtomicUsize,
    worker_thread_ids: Mutex<BTreeSet<String>>,
    decode_calls: AtomicUsize,
}

impl UnpackProbe {
    fn record_worker_count(&self, worker_count: usize) {
        self.effective_worker_count
            .store(worker_count, Ordering::Relaxed);
    }

    fn record_worker_started(&self) {
        self.worker_thread_ids
            .lock()
            .expect("probe lock")
            .insert(format!("{:?}", std::thread::current().id()));
    }

    fn record_decode_call(&self) {
        self.decode_calls.fetch_add(1, Ordering::Relaxed);
    }

    #[cfg(test)]
    fn snapshot(&self) -> UnpackProbeSnapshot {
        UnpackProbeSnapshot {
            effective_worker_count: self.effective_worker_count.load(Ordering::Relaxed),
            worker_threads_started: self.worker_thread_ids.lock().expect("probe lock").len(),
            decode_calls: self.decode_calls.load(Ordering::Relaxed),
        }
    }
}

pub fn pack_directory(
    input_dir: &Path,
    output_archive: &Path,
    config: &PackConfig,
) -> Result<PackStats, UnixFsError> {
    let started = Instant::now();
    let scan_started = Instant::now();
    let scan = scan_tree(input_dir)?;
    let scan_ms = elapsed_ms(scan_started.elapsed());

    let plan_started = Instant::now();
    let entries = scan
        .entries
        .iter()
        .map(|entry| into_planner_entry(input_dir, entry))
        .collect::<Vec<_>>();
    let planned = plan_archive(
        &entries,
        config.bundle_target_bytes,
        config.small_file_threshold,
    )?;
    let prepared = prepare_archive(planned.manifest.clone(), config)?;
    let plan_ms = elapsed_ms(plan_started.elapsed());

    let encode_started = Instant::now();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(config.threads.max(1))
        .build()
        .map_err(|_| UnixFsError::InvalidState("failed to build rayon thread pool"))?;
    let frames = pool.install(|| {
        planned
            .bundles
            .par_iter()
            .map(|bundle| encode_bundle(bundle, config))
            .collect::<Result<Vec<_>, _>>()
    })?;
    let encode_ms = elapsed_ms(encode_started.elapsed());

    let write_started = Instant::now();
    if let Some(parent) = output_archive.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut writer = File::create(output_archive)?;
    let trailer = write_archive(&mut writer, &prepared, frames.clone(), config.integrity)?;
    let write_ms = elapsed_ms(write_started.elapsed());

    let raw_bytes = planned
        .bundles
        .iter()
        .map(|bundle| u64::from(bundle.raw_len))
        .sum();
    let encoded_frame_bytes = frames
        .iter()
        .map(|frame| u64::from(frame.header.encoded_len))
        .sum::<u64>();
    let archive_overhead = sfa_core::HEADER_LEN as u64
        + prepared.manifest_bytes.len() as u64
        + (frames.len() as u64 * sfa_core::FRAME_HEADER_LEN as u64)
        + trailer
            .as_ref()
            .map(|_| sfa_core::TRAILER_LEN as u64)
            .unwrap_or(0);

    Ok(PackStats::from_duration(
        started.elapsed(),
        PackStats {
            codec: format!("{:?}", config.codec).to_lowercase(),
            threads: config.threads,
            bundle_target_bytes: config.bundle_target_bytes,
            small_file_threshold: config.small_file_threshold,
            entry_count: scan.entries.len() as u64,
            bundle_count: planned.bundles.len() as u64,
            raw_bytes,
            encoded_bytes: encoded_frame_bytes + archive_overhead,
            duration_ms: 0,
            phase_breakdown: PackPhaseBreakdown {
                scan_ms: ObservedMetric::measured(scan_ms),
                plan_ms: ObservedMetric::measured(plan_ms),
                encode_ms: ObservedMetric::measured(encode_ms),
                write_ms: ObservedMetric::measured(write_ms),
            },
        },
    ))
}

pub fn unpack_archive(
    input_archive: &Path,
    output_dir: &Path,
    config: &UnpackConfig,
) -> Result<UnpackStats, UnixFsError> {
    unpack_archive_internal(input_archive, output_dir, config, None, None)
}

pub fn unpack_archive_with_diagnostics(
    input_archive: &Path,
    output_dir: &Path,
    config: &UnpackConfig,
) -> Result<(UnpackStats, UnpackDiagnostics), UnixFsError> {
    let diagnostics = Arc::new(UnpackDiagnosticsCollector::default());
    let stats = unpack_archive_internal(
        input_archive,
        output_dir,
        config,
        None,
        Some(diagnostics.clone()),
    )?;
    Ok((stats, diagnostics.snapshot()))
}

pub fn unpack_reader_to_dir<R: Read>(
    reader: R,
    output_dir: &Path,
    config: &UnpackConfig,
) -> Result<UnpackStats, UnixFsError> {
    unpack_reader_to_dir_internal(reader, output_dir, config, None, None)
}

pub fn unpack_reader_to_dir_with_diagnostics<R: Read>(
    reader: R,
    output_dir: &Path,
    config: &UnpackConfig,
) -> Result<(UnpackStats, UnpackDiagnostics), UnixFsError> {
    let diagnostics = Arc::new(UnpackDiagnosticsCollector::default());
    let stats =
        unpack_reader_to_dir_internal(reader, output_dir, config, None, Some(diagnostics.clone()))?;
    Ok((stats, diagnostics.snapshot()))
}

fn unpack_archive_internal(
    input_archive: &Path,
    output_dir: &Path,
    config: &UnpackConfig,
    probe: Option<Arc<UnpackProbe>>,
    diagnostics: Option<Arc<UnpackDiagnosticsCollector>>,
) -> Result<UnpackStats, UnixFsError> {
    let reader = File::open(input_archive)?;
    unpack_reader_to_dir_internal(reader, output_dir, config, probe, diagnostics)
}

fn unpack_reader_to_dir_internal<R: Read>(
    reader: R,
    output_dir: &Path,
    config: &UnpackConfig,
    probe: Option<Arc<UnpackProbe>>,
    diagnostics: Option<Arc<UnpackDiagnosticsCollector>>,
) -> Result<UnpackStats, UnixFsError> {
    let started = Instant::now();
    let header_started = Instant::now();
    fs::create_dir_all(output_dir)?;
    clear_untrusted_marker(output_dir)?;
    let mut archive = ArchiveReader::new(reader);
    let header = archive.read_header()?;
    let header_ms = elapsed_ms(header_started.elapsed());

    let manifest_started = Instant::now();
    let manifest = archive.read_manifest()?;
    let threads = config
        .threads
        .unwrap_or_else(|| usize::from(header.suggested_parallelism.max(1)));
    let restore_policy = RestorePolicy {
        overwrite: match config.overwrite {
            sfa_core::config::OverwritePolicy::Error => crate::restore::OverwritePolicy::Error,
            sfa_core::config::OverwritePolicy::Replace => crate::restore::OverwritePolicy::Replace,
        },
        restore_owner: matches!(config.restore_owner, RestoreOwnerPolicy::Preserve),
        ..RestorePolicy::default()
    };
    let restore_targets = build_restore_targets(&manifest)?;
    let mut restorer = LocalRestorer::new(output_dir.to_path_buf(), restore_policy);
    let manifest_ms = elapsed_ms(manifest_started.elapsed());

    for (entry_id, entry) in manifest.entries.iter().enumerate() {
        if matches!(entry.kind, CoreEntryKind::Directory) {
            restorer.create_dir(&restore_targets[entry_id])?;
        }
    }

    let regular_file_paths = prepare_regular_files(
        &manifest,
        &restore_targets,
        &mut restorer,
        output_dir,
        diagnostics.as_deref(),
    )?;
    let single_extent_regular_entries = regular_single_extent_entries(&manifest);
    let bundle_extents = group_extents_by_bundle(&manifest);
    let file_writer = ConcurrentFileWriter::new(
        regular_file_paths,
        restore_policy.overwrite,
        restore_policy.restore_owner,
        restore_policy.max_open_files,
        threads,
        diagnostics.clone(),
    );
    let pipeline = run_unpack_pipeline(
        &mut archive,
        &header,
        &bundle_extents,
        &file_writer,
        &restore_targets,
        &single_extent_regular_entries,
        threads,
        probe,
        diagnostics.clone(),
    )?;

    let restore_started = Instant::now();
    let directory_count = manifest
        .entries
        .iter()
        .filter(|entry| matches!(entry.kind, CoreEntryKind::Directory))
        .count() as u64;
    for (entry_id, entry) in manifest.entries.iter().enumerate() {
        if matches!(entry.kind, CoreEntryKind::Regular) && entry.size > 0 && entry.extent_count > 1
        {
            let finalize_started = Instant::now();
            let file = file_writer.take_or_open_entry(entry_id as u32)?;
            restorer.finalize_regular_data_file(&restore_targets[entry_id], file.as_ref())?;
            if let Some(collector) = diagnostics.as_ref() {
                collector.record_regular_finalize(finalize_started.elapsed());
            }
        }
    }
    file_writer.close_all()?;

    for (entry_id, entry) in manifest.entries.iter().enumerate() {
        match entry.kind {
            CoreEntryKind::Symlink => {
                let started = Instant::now();
                let link_target =
                    extract_slice(&manifest.name_arena, entry.link_off, entry.link_len);
                restorer.create_symlink(&restore_targets[entry_id], link_target)?;
                if let Some(collector) = diagnostics.as_ref() {
                    collector.record_symlink_create(started.elapsed());
                }
            }
            CoreEntryKind::Hardlink => {
                let started = Instant::now();
                let master = entry.hardlink_master_entry_id as usize;
                restorer.create_hardlink(&restore_targets[entry_id], &restore_targets[master])?;
                if let Some(collector) = diagnostics.as_ref() {
                    collector.record_hardlink_create(started.elapsed());
                }
            }
            _ => {}
        }
    }

    let dirs_started = Instant::now();
    restorer.finalize_dirs()?;
    if let Some(collector) = diagnostics.as_ref() {
        collector.record_dir_finalize(dirs_started.elapsed(), directory_count);
    }

    if header.feature_flags.contains(FeatureFlags::HAS_TRAILER) {
        let trailer_started = Instant::now();
        let trailer = archive
            .read_trailer()?
            .ok_or(UnixFsError::InvalidState("expected trailer"))?;
        let trailer_result = verify_trailer(
            &trailer,
            header.bundle_count,
            pipeline.total_raw_bytes,
            pipeline.total_encoded_bytes,
            &pipeline.trailer_input,
        );
        if let Some(collector) = diagnostics.as_ref() {
            collector.record_trailer_verify(trailer_started.elapsed());
        }
        if let Err(err) = trailer_result {
            write_untrusted_marker(output_dir)?;
            return Err(err);
        }
    }
    let restore_finalize_ms = elapsed_ms(restore_started.elapsed());
    let encoded_bytes = sfa_core::HEADER_LEN as u64
        + header.manifest_encoded_len
        + pipeline.total_encoded_bytes
        + header.bundle_count * sfa_core::FRAME_HEADER_LEN as u64
        + if header.feature_flags.contains(FeatureFlags::HAS_TRAILER) {
            sfa_core::TRAILER_LEN as u64
        } else {
            0
        };

    Ok(UnpackStats::from_duration(
        started.elapsed(),
        UnpackStats {
            codec: format!("{:?}", header.data_codec).to_lowercase(),
            threads,
            entry_count: manifest.entries.len() as u64,
            bundle_count: header.bundle_count,
            raw_bytes: pipeline.total_raw_bytes,
            encoded_bytes,
            duration_ms: 0,
            phase_breakdown: UnpackPhaseBreakdown {
                header_ms: ObservedMetric::measured(header_ms),
                manifest_ms: ObservedMetric::measured(manifest_ms),
                frame_read_ms: ObservedMetric::measured(pipeline.frame_read_ms),
                decode_ms: ObservedMetric::measured(pipeline.decode_ms),
                scatter_ms: ObservedMetric::measured(pipeline.scatter_ms),
                restore_finalize_ms: ObservedMetric::measured(restore_finalize_ms),
            },
        },
    ))
}

fn into_planner_entry(root: &Path, entry: &ScannedEntry) -> PlannerInputEntry {
    PlannerInputEntry {
        entry_id: entry.entry_id,
        parent_id: entry.parent_id.unwrap_or(u32::MAX),
        kind: match entry.kind {
            EntryKind::Root => CoreEntryKind::Root,
            EntryKind::Directory => CoreEntryKind::Directory,
            EntryKind::Regular => CoreEntryKind::Regular,
            EntryKind::Symlink => CoreEntryKind::Symlink,
            EntryKind::Hardlink => CoreEntryKind::Hardlink,
        },
        mode: entry.mode,
        uid: entry.uid,
        gid: entry.gid,
        mtime_sec: entry.mtime_sec,
        mtime_nsec: entry.mtime_nsec,
        size: entry.size,
        name: entry.name.clone(),
        link_target: entry.symlink_target.clone(),
        source_path: matches!(entry.kind, EntryKind::Regular)
            .then(|| root.join(&entry.relative_path)),
        hardlink_master_entry_id: entry.hardlink_master,
        dev_major: 0,
        dev_minor: 0,
        metadata: Vec::new(),
    }
}

fn encode_bundle(
    bundle: &sfa_core::BundleInput,
    config: &PackConfig,
) -> Result<EncodedFrame, UnixFsError> {
    let mut raw = vec![0u8; bundle.raw_len as usize];
    for part in &bundle.parts {
        let mut slice = vec![0u8; part.raw_len as usize];
        let file = File::open(&part.source_path)?;
        read_exact_at(&file, &mut slice, part.file_offset)?;
        let start = part.raw_offset_in_bundle as usize;
        let end = start + part.raw_len as usize;
        raw[start..end].copy_from_slice(&slice);
    }
    let payload = encode_data(config.codec, &raw, config.compression_level)?;
    let frame_hash = frame_hash(FrameHashAlgo::Xxh3_64, &raw);
    Ok(EncodedFrame {
        header: FrameHeaderV1 {
            bundle_id: bundle.bundle_id,
            raw_len: bundle.raw_len,
            encoded_len: payload.len() as u32,
            frame_hash,
            flags: 0,
        },
        payload,
    })
}

fn read_exact_at(file: &File, mut buf: &mut [u8], mut offset: u64) -> Result<(), UnixFsError> {
    while !buf.is_empty() {
        let read = file.read_at(buf, offset)?;
        if read == 0 {
            return Err(UnixFsError::InvalidState(
                "unexpected eof while reading bundle part",
            ));
        }
        offset += read as u64;
        buf = &mut buf[read..];
    }
    Ok(())
}

fn build_restore_targets(manifest: &sfa_core::Manifest) -> Result<Vec<RestoreTarget>, UnixFsError> {
    let mut paths = vec![PathBuf::new(); manifest.entries.len()];
    let mut targets = Vec::with_capacity(manifest.entries.len());

    for (entry_id, entry) in manifest.entries.iter().enumerate() {
        let relative_path = if entry_id == 0 {
            PathBuf::new()
        } else {
            let parent = if entry.parent_id == u32::MAX {
                PathBuf::new()
            } else {
                paths[entry.parent_id as usize].clone()
            };
            let name = extract_slice(&manifest.name_arena, entry.name_off, entry.name_len).to_vec();
            let mut path = parent;
            path.push(OsString::from_vec(name));
            ensure_safe_relative_path(&path)?;
            path
        };
        paths[entry_id] = relative_path.clone();
        targets.push(RestoreTarget {
            entry_id: entry_id as u32,
            relative_path,
            metadata: EntryMetadata {
                mode: entry.mode,
                uid: entry.uid,
                gid: entry.gid,
                mtime_sec: entry.mtime_sec,
                mtime_nsec: entry.mtime_nsec,
            },
        });
    }

    Ok(targets)
}

fn group_extents_by_bundle(manifest: &sfa_core::Manifest) -> HashMap<u64, BundleExtents> {
    let mut grouped: HashMap<u64, Vec<BundleExtent>> = HashMap::new();
    for extent in &manifest.extents {
        grouped
            .entry(extent.bundle_id)
            .or_default()
            .push(BundleExtent {
                entry_id: extent.entry_id,
                file_offset: extent.file_offset,
                raw_offset_in_bundle: extent.raw_offset_in_bundle,
                raw_len: extent.raw_len,
            });
    }
    grouped
        .into_iter()
        .map(|(bundle_id, extents)| (bundle_id, Arc::<[BundleExtent]>::from(extents)))
        .collect()
}

fn prepare_regular_files(
    manifest: &sfa_core::Manifest,
    restore_targets: &[RestoreTarget],
    restorer: &mut LocalRestorer,
    output_dir: &Path,
    diagnostics: Option<&UnpackDiagnosticsCollector>,
) -> Result<HashMap<u32, PreparedRegularFile>, UnixFsError> {
    let mut regular_paths = HashMap::new();
    let mut dir_cache = HashMap::new();

    for (entry_id, entry) in manifest.entries.iter().enumerate() {
        if !matches!(entry.kind, CoreEntryKind::Regular) {
            continue;
        }
        let target = &restore_targets[entry_id];
        if entry.size == 0 {
            let path = restorer.prepare_regular_file(target)?;
            let _ = path;
            restorer.finalize_entry(target)?;
        } else {
            let path = restorer.prepare_regular_path(target)?;
            let descriptor =
                prepare_regular_descriptor(output_dir, &path, &mut dir_cache, diagnostics)?;
            regular_paths.insert(target.entry_id, descriptor);
        }
    }

    Ok(regular_paths)
}

fn regular_single_extent_entries(manifest: &sfa_core::Manifest) -> Vec<bool> {
    manifest
        .entries
        .iter()
        .map(|entry| {
            matches!(entry.kind, CoreEntryKind::Regular)
                && entry.size > 0
                && entry.extent_count == 1
        })
        .collect()
}

fn run_unpack_pipeline<R: std::io::Read>(
    archive: &mut ArchiveReader<R>,
    header: &sfa_core::format::HeaderV1,
    bundle_extents: &HashMap<u64, BundleExtents>,
    file_writer: &ConcurrentFileWriter,
    restore_targets: &[RestoreTarget],
    single_extent_regular_entries: &[bool],
    threads: usize,
    probe: Option<Arc<UnpackProbe>>,
    diagnostics: Option<Arc<UnpackDiagnosticsCollector>>,
) -> Result<UnpackPipelineOutcome, UnixFsError> {
    let scatter_worker_count = threads.max(1);
    let decode_worker_count = recommended_decode_workers(scatter_worker_count);
    let queue_depth = scatter_worker_count.saturating_mul(2).max(1);
    let (decode_tx, decode_rx) = bounded::<FrameWorkItem>(queue_depth);
    let (scatter_tx, scatter_rx) = bounded::<DecodedBundleTask>(queue_depth);
    let failed = AtomicBool::new(false);
    let first_error = std::sync::Mutex::new(None::<UnixFsError>);
    let mut outcome = UnpackPipelineOutcome::default();

    if let Some(probe) = probe.as_ref() {
        probe.record_worker_count(scatter_worker_count);
    }
    if let Some(collector) = diagnostics.as_ref() {
        collector.record_pipeline_config(scatter_worker_count, decode_worker_count, queue_depth);
    }

    std::thread::scope(|scope| -> Result<(), UnixFsError> {
        let mut scatter_handles = Vec::with_capacity(scatter_worker_count);
        for _ in 0..scatter_worker_count {
            let worker_rx = scatter_rx.clone();
            let writer = file_writer;
            let restore_targets = restore_targets;
            let single_extent_regular_entries = single_extent_regular_entries;
            let failed = &failed;
            let first_error = &first_error;
            let diagnostics = diagnostics.clone();
            let worker_probe = probe.clone();
            scatter_handles.push(scope.spawn(move || {
                if let Some(probe) = worker_probe.as_ref() {
                    probe.record_worker_started();
                }
                let mut totals = ScatterPhaseTotals::default();
                while let Ok(task) = worker_rx.recv() {
                    if failed.load(Ordering::Relaxed) {
                        break;
                    }
                    match process_scatter_task(
                        task,
                        writer,
                        restore_targets,
                        single_extent_regular_entries,
                        diagnostics.as_deref(),
                    ) {
                        Ok(scatter_ms) => {
                            totals.scatter_ms = totals.scatter_ms.saturating_add(scatter_ms);
                        }
                        Err(err) => {
                            failed.store(true, Ordering::Relaxed);
                            let mut slot = first_error.lock().expect("pipeline error lock");
                            if slot.is_none() {
                                *slot = Some(err);
                            }
                            break;
                        }
                    }
                }
                totals
            }));
        }
        drop(scatter_rx);

        let mut decode_handles = Vec::with_capacity(decode_worker_count);
        for _ in 0..decode_worker_count {
            let worker_rx = decode_rx.clone();
            let worker_tx = scatter_tx.clone();
            let codec = header.data_codec;
            let hash_algo = header.frame_hash_algo;
            let failed = &failed;
            let first_error = &first_error;
            let worker_probe = probe.clone();
            let diagnostics = diagnostics.clone();
            decode_handles.push(scope.spawn(move || {
                let mut totals = DecodePhaseTotals::default();
                while let Ok(task) = worker_rx.recv() {
                    if failed.load(Ordering::Relaxed) {
                        break;
                    }
                    match process_decode_task(task, codec, hash_algo, worker_probe.as_deref()) {
                        Ok((decoded_task, decode_ms)) => {
                            totals.decode_ms = totals.decode_ms.saturating_add(decode_ms);
                            let wait_started = Instant::now();
                            let send_result = worker_tx.send(decoded_task);
                            let wait_duration = wait_started.elapsed();
                            if let Some(collector) = diagnostics.as_ref() {
                                collector.record_scatter_dispatch_wait(wait_duration, 1);
                            }
                            if let Err(SendError(_)) = send_result {
                                failed.store(true, Ordering::Relaxed);
                                let mut slot = first_error.lock().expect("pipeline error lock");
                                if slot.is_none() {
                                    *slot = Some(UnixFsError::InvalidState(
                                        "scatter workers terminated before all bundles were dispatched",
                                    ));
                                }
                            }
                            if failed.load(Ordering::Relaxed) {
                                break;
                            }
                        }
                        Err(err) => {
                            failed.store(true, Ordering::Relaxed);
                            let mut slot = first_error.lock().expect("pipeline error lock");
                            if slot.is_none() {
                                *slot = Some(err);
                            }
                            break;
                        }
                    }
                }
                totals
            }));
        }
        drop(decode_rx);
        drop(scatter_tx);

        loop {
            if failed.load(Ordering::Relaxed) {
                break;
            }
            let frame_started = Instant::now();
            let frame = archive.next_frame()?;
            outcome.frame_read_ms = outcome
                .frame_read_ms
                .saturating_add(elapsed_ms(frame_started.elapsed()));
            let Some(frame) = frame else {
                break;
            };

            outcome.total_raw_bytes = outcome
                .total_raw_bytes
                .saturating_add(u64::from(frame.header.raw_len));
            outcome.total_encoded_bytes = outcome
                .total_encoded_bytes
                .saturating_add(u64::from(frame.header.encoded_len));
            outcome
                .trailer_input
                .extend_from_slice(&frame.header.frame_hash.to_le_bytes());

            let extents = bundle_extents
                .get(&frame.header.bundle_id)
                .cloned()
                .unwrap_or_else(|| Arc::<[BundleExtent]>::from(Vec::<BundleExtent>::new()));
            if let Some(collector) = diagnostics.as_ref() {
                let (unique_entries, entry_switches) = summarize_bundle_extents(extents.as_ref());
                collector.record_bundle_shape(extents.len(), unique_entries, entry_switches);
            }
            let mut task = Some(FrameWorkItem {
                extents,
                header: frame.header,
                payload: frame.payload,
            });
            while let Some(pending) = task.take() {
                let wait_started = Instant::now();
                let send_result = decode_tx.send(pending);
                let wait_duration = wait_started.elapsed();
                if let Some(collector) = diagnostics.as_ref() {
                    collector.record_decode_dispatch_wait(wait_duration, 1);
                }
                if let Err(SendError(_)) = send_result {
                    failed.store(true, Ordering::Relaxed);
                    let mut slot = first_error.lock().expect("pipeline error lock");
                    if slot.is_none() {
                        *slot = Some(UnixFsError::InvalidState(
                            "decode workers terminated before all frames were dispatched",
                        ));
                    }
                    break;
                }
            }
            if failed.load(Ordering::Relaxed) {
                break;
            }
        }

        drop(decode_tx);
        for handle in decode_handles {
            let totals = handle
                .join()
                .map_err(|_| UnixFsError::InvalidState("decode worker panicked"))?;
            outcome.decode_ms = outcome.decode_ms.saturating_add(totals.decode_ms);
        }
        for handle in scatter_handles {
            let totals = handle
                .join()
                .map_err(|_| UnixFsError::InvalidState("scatter worker panicked"))?;
            outcome.scatter_ms = outcome.scatter_ms.saturating_add(totals.scatter_ms);
        }
        Ok(())
    })?;

    match first_error.into_inner() {
        Ok(Some(err)) => Err(err),
        Ok(None) => Ok(outcome),
        Err(_) => Err(UnixFsError::InvalidState("pipeline error lock poisoned")),
    }
}

fn process_decode_task(
    task: FrameWorkItem,
    codec: sfa_core::config::DataCodec,
    hash_algo: FrameHashAlgo,
    probe: Option<&UnpackProbe>,
) -> Result<(DecodedBundleTask, u64), UnixFsError> {
    let decode_started = Instant::now();
    let raw = decode_data(codec, &task.payload, task.header.raw_len as usize)?;
    let expected_hash = frame_hash(hash_algo, &raw);
    if expected_hash != task.header.frame_hash {
        return Err(sfa_core::Error::FrameHashMismatch {
            bundle_id: task.header.bundle_id,
        }
        .into());
    }
    let decode_ms = elapsed_ms(decode_started.elapsed());
    if let Some(probe) = probe {
        probe.record_decode_call();
    }

    Ok((
        DecodedBundleTask {
            bundle_id: task.header.bundle_id,
            raw,
            extents: task.extents,
        },
        decode_ms,
    ))
}

fn process_scatter_task(
    task: DecodedBundleTask,
    file_writer: &ConcurrentFileWriter,
    restore_targets: &[RestoreTarget],
    single_extent_regular_entries: &[bool],
    _diagnostics: Option<&UnpackDiagnosticsCollector>,
) -> Result<u64, UnixFsError> {
    let _bundle_id = task.bundle_id;
    let scatter_started = Instant::now();
    for extent in task.extents.iter() {
        let start = extent.raw_offset_in_bundle as usize;
        let end = start + extent.raw_len as usize;
        if single_extent_regular_entries
            .get(extent.entry_id as usize)
            .copied()
            .unwrap_or(false)
        {
            file_writer.write_extent_once(
                &restore_targets[extent.entry_id as usize],
                extent.file_offset,
                &task.raw[start..end],
            )?;
        } else {
            file_writer.write_extent(extent.entry_id, extent.file_offset, &task.raw[start..end])?;
        }
    }
    Ok(elapsed_ms(scatter_started.elapsed()))
}

fn extract_slice(arena: &[u8], off: u32, len: u32) -> &[u8] {
    &arena[off as usize..off as usize + len as usize]
}

fn verify_trailer(
    trailer: &TrailerV1,
    bundle_count: u64,
    total_raw_bytes: u64,
    total_encoded_bytes: u64,
    trailer_input: &[u8],
) -> Result<(), UnixFsError> {
    if trailer.bundle_count != bundle_count
        || trailer.total_raw_bytes != total_raw_bytes
        || trailer.total_encoded_bytes != total_encoded_bytes
        || trailer.archive_hash != trailer_hash(trailer_input)
    {
        return Err(sfa_core::Error::TrailerHashMismatch.into());
    }
    Ok(())
}

fn elapsed_ms(duration: std::time::Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}

fn summarize_bundle_extents(extents: &[BundleExtent]) -> (usize, usize) {
    let mut unique_entries = HashSet::with_capacity(extents.len());
    let mut entry_switches = 0_usize;
    let mut previous = None;
    for extent in extents {
        unique_entries.insert(extent.entry_id);
        if previous.is_some_and(|entry_id| entry_id != extent.entry_id) {
            entry_switches = entry_switches.saturating_add(1);
        }
        previous = Some(extent.entry_id);
    }
    (unique_entries.len(), entry_switches)
}

fn recommended_decode_workers(scatter_worker_count: usize) -> usize {
    if scatter_worker_count <= 4 { 1 } else { 2 }
}

fn clear_untrusted_marker(output_dir: &Path) -> Result<(), UnixFsError> {
    match fs::remove_file(output_dir.join(UNTRUSTED_MARKER_NAME)) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err.into()),
    }
}

fn write_untrusted_marker(output_dir: &Path) -> Result<(), UnixFsError> {
    fs::write(
        output_dir.join(UNTRUSTED_MARKER_NAME),
        b"strong trailer verification failed\n",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::Permissions;
    use std::io::Cursor;
    use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use nix::sys::stat::{UtimensatFlags, utimensat};
    use nix::sys::time::TimeSpec;
    use tempfile::TempDir;

    use sfa_core::{
        ArchiveReader, FeatureFlags, PackConfig, UnpackConfig, prepare_archive, write_archive,
    };

    use crate::UnixFsError;

    use super::{
        UNTRUSTED_MARKER_NAME, UnpackProbe, pack_directory, unpack_archive,
        unpack_archive_internal, unpack_reader_to_dir,
    };

    struct FragmentedReader {
        inner: Cursor<Vec<u8>>,
        chunk_len: usize,
    }

    impl FragmentedReader {
        fn new(bytes: Vec<u8>, chunk_len: usize) -> Self {
            Self {
                inner: Cursor::new(bytes),
                chunk_len,
            }
        }
    }

    impl std::io::Read for FragmentedReader {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let max_len = buf.len().min(self.chunk_len.max(1));
            self.inner.read(&mut buf[..max_len])
        }
    }

    fn set_path_mtime(path: &Path, sec: i64, nsec: u32) {
        let parent = fs::File::open(path.parent().expect("mtime parent")).expect("open parent");
        let leaf = path.file_name().expect("mtime leaf");
        utimensat(
            &parent,
            leaf,
            &TimeSpec::UTIME_OMIT,
            &TimeSpec::new(sec, i64::from(nsec)),
            UtimensatFlags::FollowSymlink,
        )
        .expect("set mtime");
    }

    fn alternate_raw_id(current: u32) -> u32 {
        if current == 0 { 1 } else { 0 }
    }

    fn rewrite_archive_owner_ids(archive: &Path, uid: u32, gid: u32) {
        let bytes = fs::read(archive).expect("read archive");
        let mut reader = ArchiveReader::new(Cursor::new(bytes));
        let header = reader.read_header().expect("read header");
        let mut manifest = reader.read_manifest().expect("read manifest");
        for entry in manifest.entries.iter_mut().filter(|entry| {
            matches!(
                entry.kind,
                sfa_core::EntryKind::Directory | sfa_core::EntryKind::Regular
            )
        }) {
            entry.uid = uid;
            entry.gid = gid;
        }

        let mut frames = Vec::new();
        while let Some(frame) = reader.next_frame().expect("read frame") {
            frames.push(frame);
        }
        let _ = reader.read_trailer().expect("read trailer");

        let config = PackConfig {
            codec: header.data_codec,
            manifest_codec: header.manifest_codec,
            compression_level: None,
            threads: usize::from(header.suggested_parallelism.max(1)),
            bundle_target_bytes: header.bundle_target_bytes,
            small_file_threshold: header.small_file_threshold,
            integrity: header.integrity_mode,
            preserve_owner: header.feature_flags.contains(FeatureFlags::PRESERVE_OWNER),
        };
        let prepared = prepare_archive(manifest, &config).expect("prepare archive");
        let mut writer = fs::File::create(archive).expect("rewrite archive");
        write_archive(&mut writer, &prepared, frames, config.integrity).expect("write archive");
    }

    #[test]
    fn roundtrip_pack_and_unpack() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        let archive = src.path().join("sample.sfa");
        fs::create_dir(src.path().join("dir")).unwrap();
        fs::write(src.path().join("dir/hello.txt"), b"hello world").unwrap();
        fs::write(src.path().join("empty.txt"), b"").unwrap();
        fs::write(src.path().join("master.txt"), b"same").unwrap();
        fs::hard_link(src.path().join("master.txt"), src.path().join("peer.txt")).unwrap();
        symlink("dir/hello.txt", src.path().join("hello-link")).unwrap();

        let pack_stats = pack_directory(src.path(), &archive, &PackConfig::default()).unwrap();
        assert!(pack_stats.bundle_count >= 1);

        let out = dst.path().join("out");
        let unpack_stats = unpack_archive(&archive, &out, &UnpackConfig::default()).unwrap();
        assert!(unpack_stats.entry_count >= 5);
        assert_eq!(fs::read(out.join("dir/hello.txt")).unwrap(), b"hello world");
        assert_eq!(
            fs::read_link(out.join("hello-link")).unwrap(),
            PathBuf::from("dir/hello.txt")
        );
        assert_eq!(fs::read(out.join("master.txt")).unwrap(), b"same");
        assert_eq!(fs::read(out.join("peer.txt")).unwrap(), b"same");
    }

    #[test]
    fn roundtrip_restores_mode_and_mtime_for_files_and_directories() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        let archive = src.path().join("sample.sfa");
        let nested = src.path().join("nested");
        let file = nested.join("payload.txt");

        fs::create_dir(&nested).unwrap();
        fs::write(&file, b"payload").unwrap();
        fs::set_permissions(&nested, Permissions::from_mode(0o751)).unwrap();
        fs::set_permissions(&file, Permissions::from_mode(0o640)).unwrap();
        set_path_mtime(&file, 9_876, 123);
        set_path_mtime(&nested, 8_765, 432);

        pack_directory(src.path(), &archive, &PackConfig::default()).unwrap();

        let out = dst.path().join("out");
        unpack_archive(&archive, &out, &UnpackConfig::default()).unwrap();

        let restored_dir = fs::metadata(out.join("nested")).unwrap();
        assert_eq!(restored_dir.permissions().mode() & 0o777, 0o751);
        assert_eq!(restored_dir.mtime(), 8_765);
        assert_eq!(restored_dir.mtime_nsec(), 432);

        let restored_file = fs::metadata(out.join("nested/payload.txt")).unwrap();
        assert_eq!(restored_file.permissions().mode() & 0o777, 0o640);
        assert_eq!(restored_file.mtime(), 9_876);
        assert_eq!(restored_file.mtime_nsec(), 123);
    }

    #[test]
    fn unpack_rejects_truncated_archive() {
        let temp = TempDir::new().unwrap();
        let archive = temp.path().join("bad.sfa");
        fs::write(&archive, b"SFA").unwrap();
        let out = temp.path().join("out");
        let err = unpack_archive(&archive, &out, &UnpackConfig::default()).unwrap_err();
        match err {
            UnixFsError::Core(_) | UnixFsError::Io(_) => {}
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn fragmented_reader_header_roundtrip() {
        let src = TempDir::new().unwrap();
        let archive = src.path().join("sample.sfa");
        fs::write(src.path().join("one.txt"), b"one").unwrap();
        pack_directory(src.path(), &archive, &PackConfig::default()).unwrap();
        let bytes = fs::read(&archive).unwrap();
        let mut cursor = Cursor::new(bytes);
        let mut archive_reader = sfa_core::ArchiveReader::new(&mut cursor);
        let header = archive_reader.read_header().unwrap();
        assert!(header.bundle_count >= 1);
    }

    #[test]
    fn unpack_reader_roundtrip_with_fragmented_input() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        let archive = src.path().join("sample.sfa");
        fs::write(src.path().join("one.txt"), b"one").unwrap();
        fs::write(src.path().join("two.txt"), b"two").unwrap();
        pack_directory(src.path(), &archive, &PackConfig::default()).unwrap();

        let reader = FragmentedReader::new(fs::read(&archive).unwrap(), 7);
        let out = dst.path().join("out");
        let stats = unpack_reader_to_dir(reader, &out, &UnpackConfig::default()).unwrap();

        assert!(stats.bundle_count >= 1);
        assert_eq!(fs::read(out.join("one.txt")).unwrap(), b"one");
        assert_eq!(fs::read(out.join("two.txt")).unwrap(), b"two");
    }

    #[test]
    fn unpack_rejects_preexisting_symlink_escape() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        let archive = src.path().join("sample.sfa");
        fs::create_dir_all(src.path().join("safe/escape")).unwrap();
        fs::write(src.path().join("safe/escape/payload.txt"), b"payload").unwrap();
        pack_directory(src.path(), &archive, &PackConfig::default()).unwrap();

        let out = dst.path().join("out");
        fs::create_dir_all(out.join("safe")).unwrap();
        symlink("/tmp", out.join("safe/escape")).unwrap();

        let err = unpack_archive(&archive, &out, &UnpackConfig::default()).unwrap_err();
        match err {
            UnixFsError::PathValidation(crate::PathValidationError::SymlinkTraversal(_)) => {}
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn unpack_rejects_corrupted_manifest() {
        let src = TempDir::new().unwrap();
        let archive = src.path().join("sample.sfa");
        fs::write(src.path().join("one.txt"), b"one").unwrap();
        let config = PackConfig {
            manifest_codec: sfa_core::ManifestCodec::None,
            ..PackConfig::default()
        };
        pack_directory(src.path(), &archive, &config).unwrap();

        let mut bytes = fs::read(&archive).unwrap();
        bytes[sfa_core::HEADER_LEN + 8] ^= 0xFF;
        fs::write(&archive, bytes).unwrap();

        let out = src.path().join("out");
        let err = unpack_archive(&archive, &out, &UnpackConfig::default()).unwrap_err();
        match err {
            UnixFsError::Core(sfa_core::Error::ManifestHashMismatch) => {}
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn unpack_rejects_corrupted_frame_payload() {
        let src = TempDir::new().unwrap();
        let archive = src.path().join("sample.sfa");
        fs::write(src.path().join("one.txt"), b"one").unwrap();
        pack_directory(src.path(), &archive, &PackConfig::default()).unwrap();

        let mut bytes = fs::read(&archive).unwrap();
        let idx = bytes.len() - 1;
        bytes[idx] ^= 0x55;
        fs::write(&archive, bytes).unwrap();

        let out = src.path().join("out");
        let err = unpack_archive(&archive, &out, &UnpackConfig::default()).unwrap_err();
        match err {
            UnixFsError::Core(sfa_core::Error::FrameHashMismatch { .. }) => {}
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn unpack_thread_override_drives_worker_pipeline_and_single_decode_per_bundle() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        let archive = src.path().join("sample.sfa");

        for idx in 0..8 {
            let path = src.path().join(format!("file-{idx}.txt"));
            fs::write(path, vec![b'a' + idx as u8; 96]).unwrap();
        }

        let pack_config = PackConfig {
            bundle_target_bytes: 128,
            small_file_threshold: 1024,
            ..PackConfig::default()
        };
        let pack_stats = pack_directory(src.path(), &archive, &pack_config).unwrap();
        assert!(pack_stats.bundle_count > 1);

        let unpack_config = UnpackConfig {
            threads: Some(3),
            ..UnpackConfig::default()
        };
        let out = dst.path().join("out");
        let probe = Arc::new(UnpackProbe::default());
        let unpack_stats =
            unpack_archive_internal(&archive, &out, &unpack_config, Some(probe.clone()), None)
                .unwrap();
        let probe = probe.snapshot();

        assert_eq!(unpack_stats.threads, 3);
        assert_eq!(probe.effective_worker_count, 3);
        assert_eq!(probe.worker_threads_started, 3);
        assert_eq!(probe.decode_calls as u64, unpack_stats.bundle_count);
        assert_eq!(fs::read(out.join("file-0.txt")).unwrap(), vec![b'a'; 96]);
    }

    #[test]
    fn strong_trailer_failure_marks_output_untrusted() {
        let src = TempDir::new().unwrap();
        let archive = src.path().join("sample.sfa");
        fs::write(src.path().join("one.txt"), b"one").unwrap();
        let pack_config = PackConfig {
            integrity: sfa_core::IntegrityMode::Strong,
            ..PackConfig::default()
        };
        pack_directory(src.path(), &archive, &pack_config).unwrap();

        let mut bytes = fs::read(&archive).unwrap();
        let idx = bytes.len() - 1;
        bytes[idx] ^= 0x55;
        fs::write(&archive, bytes).unwrap();

        let out = src.path().join("out");
        let err = unpack_archive(&archive, &out, &UnpackConfig::default()).unwrap_err();
        match err {
            UnixFsError::Core(sfa_core::Error::TrailerHashMismatch) => {}
            other => panic!("unexpected error: {other}"),
        }
        assert_eq!(
            fs::read_to_string(out.join(UNTRUSTED_MARKER_NAME)).unwrap(),
            "strong trailer verification failed\n"
        );
    }

    #[test]
    fn successful_unpack_clears_stale_untrusted_marker() {
        let src = TempDir::new().unwrap();
        let archive = src.path().join("sample.sfa");
        let out = src.path().join("out");
        fs::write(src.path().join("one.txt"), b"one").unwrap();
        pack_directory(src.path(), &archive, &PackConfig::default()).unwrap();

        fs::create_dir_all(&out).unwrap();
        fs::write(
            out.join(UNTRUSTED_MARKER_NAME),
            b"strong trailer verification failed\n",
        )
        .unwrap();

        unpack_archive(&archive, &out, &UnpackConfig::default()).unwrap();
        assert!(!out.join(UNTRUSTED_MARKER_NAME).exists());
    }

    #[test]
    fn unpack_default_policy_skips_stored_owner_metadata() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        let archive = src.path().join("sample.sfa");
        let nested = src.path().join("owned-dir");
        let file = nested.join("owned.txt");

        fs::create_dir(&nested).unwrap();
        fs::write(&file, b"owner metadata").unwrap();

        let source_metadata = fs::metadata(&file).unwrap();
        let current_uid = source_metadata.uid();
        let current_gid = source_metadata.gid();
        let archive_uid = alternate_raw_id(current_uid);
        let archive_gid = alternate_raw_id(current_gid);

        let pack_config = PackConfig {
            preserve_owner: true,
            ..PackConfig::default()
        };
        pack_directory(src.path(), &archive, &pack_config).unwrap();
        rewrite_archive_owner_ids(&archive, archive_uid, archive_gid);

        let out = dst.path().join("out");
        unpack_archive(&archive, &out, &UnpackConfig::default()).unwrap();

        let restored_dir = fs::metadata(out.join("owned-dir")).unwrap();
        assert_eq!(restored_dir.uid(), current_uid);
        assert_eq!(restored_dir.gid(), current_gid);
        assert_ne!(restored_dir.uid(), archive_uid);
        assert_ne!(restored_dir.gid(), archive_gid);

        let restored_file = fs::metadata(out.join("owned-dir/owned.txt")).unwrap();
        assert_eq!(restored_file.uid(), current_uid);
        assert_eq!(restored_file.gid(), current_gid);
        assert_ne!(restored_file.uid(), archive_uid);
        assert_ne!(restored_file.gid(), archive_gid);
    }
}
