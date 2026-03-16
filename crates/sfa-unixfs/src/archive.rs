use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::{self, File};
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};
use std::time::Instant;

use filetime::{FileTime, set_file_mtime};
use rayon::prelude::*;
use sfa_core::archive::{ArchiveReader, prepare_archive, write_archive};
use sfa_core::codec::{decode_data, encode_data};
use sfa_core::config::{FrameHashAlgo, PackConfig, RestoreOwnerPolicy, UnpackConfig};
use sfa_core::format::{FeatureFlags, TrailerV1};
use sfa_core::integrity::{frame_hash, trailer_hash};
use sfa_core::model::{EntryKind as CoreEntryKind, PlannerInputEntry};
use sfa_core::{EncodedFrame, FrameHeaderV1, PackStats, UnpackStats, plan_archive};

use crate::error::UnixFsError;
use crate::path::ensure_safe_relative_path;
use crate::restore::{EntryMetadata, LocalRestorer, RestorePolicy, RestoreTarget, Restorer};
use crate::scan::{EntryKind, ScannedEntry, scan_tree};

pub fn pack_directory(
    input_dir: &Path,
    output_archive: &Path,
    config: &PackConfig,
) -> Result<PackStats, UnixFsError> {
    let started = Instant::now();
    let scan = scan_tree(input_dir)?;
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

    if let Some(parent) = output_archive.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut writer = File::create(output_archive)?;
    let trailer = write_archive(&mut writer, &prepared, frames.clone(), config.integrity)?;

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
        },
    ))
}

pub fn unpack_archive(
    input_archive: &Path,
    output_dir: &Path,
    config: &UnpackConfig,
) -> Result<UnpackStats, UnixFsError> {
    let started = Instant::now();
    fs::create_dir_all(output_dir)?;
    let archive_len = fs::metadata(input_archive)?.len();
    let reader = File::open(input_archive)?;
    let mut archive = ArchiveReader::new(reader);
    let header = archive.read_header()?;
    let manifest = archive.read_manifest()?;
    let threads = config
        .threads
        .unwrap_or_else(|| usize::from(header.suggested_parallelism.max(1)));

    let restore_targets = build_restore_targets(&manifest)?;
    let mut restorer = LocalRestorer::new(
        output_dir.to_path_buf(),
        RestorePolicy {
            overwrite: match config.overwrite {
                sfa_core::config::OverwritePolicy::Error => crate::restore::OverwritePolicy::Error,
                sfa_core::config::OverwritePolicy::Replace => {
                    crate::restore::OverwritePolicy::Replace
                }
            },
            restore_owner: matches!(config.restore_owner, RestoreOwnerPolicy::Preserve),
            ..RestorePolicy::default()
        },
    );

    for (entry_id, entry) in manifest.entries.iter().enumerate() {
        if matches!(entry.kind, CoreEntryKind::Directory) {
            restorer.create_dir(&restore_targets[entry_id])?;
        }
    }

    for (entry_id, entry) in manifest.entries.iter().enumerate() {
        if matches!(entry.kind, CoreEntryKind::Regular) && entry.size == 0 {
            restorer.ensure_file(&restore_targets[entry_id])?;
            restorer.finalize_entry(&restore_targets[entry_id])?;
            apply_file_times(&restore_targets[entry_id], output_dir)?;
        }
    }

    let bundle_extents = group_extents_by_bundle(&manifest);
    let mut total_raw_bytes = 0u64;
    let mut total_encoded_bytes = 0u64;
    let mut trailer_input = Vec::new();

    while let Some(frame) = archive.next_frame()? {
        let raw = decode_data(
            header.data_codec,
            &frame.payload,
            frame.header.raw_len as usize,
        )?;
        total_raw_bytes += u64::from(frame.header.raw_len);
        total_encoded_bytes += u64::from(frame.header.encoded_len);
        trailer_input.extend_from_slice(&frame.header.frame_hash.to_le_bytes());

        if let Some(extents) = bundle_extents.get(&frame.header.bundle_id) {
            for extent in extents {
                let start = extent.raw_offset_in_bundle as usize;
                let end = start + extent.raw_len as usize;
                let target = &restore_targets[extent.entry_id as usize];
                restorer.write_extent(target, extent.file_offset, &raw[start..end])?;
            }
        }
    }

    for (entry_id, entry) in manifest.entries.iter().enumerate() {
        if matches!(entry.kind, CoreEntryKind::Regular) && entry.size > 0 {
            restorer.finalize_entry(&restore_targets[entry_id])?;
            apply_file_times(&restore_targets[entry_id], output_dir)?;
        }
    }

    for (entry_id, entry) in manifest.entries.iter().enumerate() {
        match entry.kind {
            CoreEntryKind::Symlink => {
                let link_target =
                    extract_slice(&manifest.name_arena, entry.link_off, entry.link_len);
                restorer.create_symlink(&restore_targets[entry_id], link_target)?;
            }
            CoreEntryKind::Hardlink => {
                let master = entry.hardlink_master_entry_id as usize;
                restorer.create_hardlink(&restore_targets[entry_id], &restore_targets[master])?;
            }
            _ => {}
        }
    }

    restorer.finalize_dirs()?;
    apply_directory_times(&manifest, &restore_targets, output_dir)?;

    if header.feature_flags.contains(FeatureFlags::HAS_TRAILER) {
        let trailer = archive
            .read_trailer()?
            .ok_or(UnixFsError::InvalidState("expected trailer"))?;
        verify_trailer(
            &trailer,
            header.bundle_count,
            total_raw_bytes,
            total_encoded_bytes,
            &trailer_input,
        )?;
    }

    Ok(UnpackStats::from_duration(
        started.elapsed(),
        UnpackStats {
            codec: format!("{:?}", header.data_codec).to_lowercase(),
            threads,
            entry_count: manifest.entries.len() as u64,
            bundle_count: header.bundle_count,
            raw_bytes: total_raw_bytes,
            encoded_bytes: archive_len,
            duration_ms: 0,
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

fn group_extents_by_bundle(
    manifest: &sfa_core::Manifest,
) -> HashMap<u64, Vec<&sfa_core::ExtentRecord>> {
    let mut grouped: HashMap<u64, Vec<&sfa_core::ExtentRecord>> = HashMap::new();
    for extent in &manifest.extents {
        grouped.entry(extent.bundle_id).or_default().push(extent);
    }
    grouped
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

fn apply_file_times(target: &RestoreTarget, output_dir: &Path) -> Result<(), UnixFsError> {
    if target.relative_path.as_os_str().is_empty() {
        return Ok(());
    }
    let path = output_dir.join(&target.relative_path);
    let time = FileTime::from_unix_time(target.metadata.mtime_sec, target.metadata.mtime_nsec);
    set_file_mtime(path, time)?;
    Ok(())
}

fn apply_directory_times(
    manifest: &sfa_core::Manifest,
    targets: &[RestoreTarget],
    output_dir: &Path,
) -> Result<(), UnixFsError> {
    for (entry_id, entry) in manifest.entries.iter().enumerate().rev() {
        if matches!(entry.kind, CoreEntryKind::Directory) {
            apply_file_times(&targets[entry_id], output_dir)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Cursor;
    use std::os::unix::fs::symlink;
    use std::path::PathBuf;

    use tempfile::TempDir;

    use sfa_core::{PackConfig, UnpackConfig};

    use crate::UnixFsError;

    use super::{pack_directory, unpack_archive};

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
}
