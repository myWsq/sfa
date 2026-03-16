use std::fs::File;
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};

use sfa_core::archive::{prepare_archive, write_archive};
use sfa_core::codec::encode_data;
use sfa_core::config::{DataCodec, IntegrityMode, ManifestCodec, PackConfig};
use sfa_core::integrity::frame_hash;
use sfa_core::model::{BundleInput, PlannerInputEntry};
use sfa_core::{EncodedFrame, FrameHashAlgo, FrameHeaderV1, plan_archive};
use sfa_unixfs::scan::{EntryKind, ScannedEntry};
use sfa_unixfs::scan_tree;

const DEFAULT_MTIME_SEC: i64 = 1_704_067_200;
const DEFAULT_UID: u32 = 0;
const DEFAULT_GID: u32 = 0;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "Usage: generate_golden_archive <input-dir> <archive.sfa> [--codec <lz4|zstd>] [--threads <n>] [--bundle-target-bytes <n>] [--small-file-threshold <n>] [--integrity <off|fast|strong>] [--mtime-sec <sec>] [--uid <uid>] [--gid <gid>]"
        );
        std::process::exit(2);
    }

    let input_dir = PathBuf::from(&args[1]);
    let output_archive = PathBuf::from(&args[2]);
    let mut config = PackConfig {
        manifest_codec: ManifestCodec::Zstd,
        ..PackConfig::default()
    };
    let mut normalized_mtime_sec = DEFAULT_MTIME_SEC;
    let mut normalized_uid = DEFAULT_UID;
    let mut normalized_gid = DEFAULT_GID;

    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--codec" => {
                index += 1;
                config.codec = parse_codec(args.get(index).ok_or("--codec needs a value")?)?;
            }
            "--threads" => {
                index += 1;
                config.threads = args.get(index).ok_or("--threads needs a value")?.parse()?;
            }
            "--bundle-target-bytes" => {
                index += 1;
                config.bundle_target_bytes = args
                    .get(index)
                    .ok_or("--bundle-target-bytes needs a value")?
                    .parse()?;
            }
            "--small-file-threshold" => {
                index += 1;
                config.small_file_threshold = args
                    .get(index)
                    .ok_or("--small-file-threshold needs a value")?
                    .parse()?;
            }
            "--integrity" => {
                index += 1;
                config.integrity =
                    parse_integrity(args.get(index).ok_or("--integrity needs a value")?)?;
            }
            "--mtime-sec" => {
                index += 1;
                normalized_mtime_sec = args
                    .get(index)
                    .ok_or("--mtime-sec needs a value")?
                    .parse()?;
            }
            "--uid" => {
                index += 1;
                normalized_uid = args.get(index).ok_or("--uid needs a value")?.parse()?;
            }
            "--gid" => {
                index += 1;
                normalized_gid = args.get(index).ok_or("--gid needs a value")?.parse()?;
            }
            other => {
                return Err(format!("unknown argument: {other}").into());
            }
        }
        index += 1;
    }

    let scan = scan_tree(&input_dir)?;
    let entries = scan
        .entries
        .iter()
        .map(|entry| {
            into_planner_entry(
                &input_dir,
                entry,
                normalized_mtime_sec,
                normalized_uid,
                normalized_gid,
            )
        })
        .collect::<Vec<_>>();
    let planned = plan_archive(
        &entries,
        config.bundle_target_bytes,
        config.small_file_threshold,
    )?;
    let prepared = prepare_archive(planned.manifest, &config)?;
    let frames = planned
        .bundles
        .iter()
        .map(|bundle| encode_bundle(bundle, &config))
        .collect::<Result<Vec<_>, _>>()?;

    if let Some(parent) = output_archive.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut writer = File::create(output_archive)?;
    write_archive(&mut writer, &prepared, frames, config.integrity)?;

    Ok(())
}

fn into_planner_entry(
    root: &Path,
    entry: &ScannedEntry,
    normalized_mtime_sec: i64,
    normalized_uid: u32,
    normalized_gid: u32,
) -> PlannerInputEntry {
    PlannerInputEntry {
        entry_id: entry.entry_id,
        parent_id: entry.parent_id.unwrap_or(u32::MAX),
        kind: match entry.kind {
            EntryKind::Root => sfa_core::EntryKind::Root,
            EntryKind::Directory => sfa_core::EntryKind::Directory,
            EntryKind::Regular => sfa_core::EntryKind::Regular,
            EntryKind::Symlink => sfa_core::EntryKind::Symlink,
            EntryKind::Hardlink => sfa_core::EntryKind::Hardlink,
        },
        mode: entry.mode,
        uid: normalized_uid,
        gid: normalized_gid,
        mtime_sec: normalized_mtime_sec,
        mtime_nsec: 0,
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
    bundle: &BundleInput,
    config: &PackConfig,
) -> Result<EncodedFrame, Box<dyn std::error::Error + Send + Sync>> {
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
    Ok(EncodedFrame {
        header: FrameHeaderV1 {
            bundle_id: bundle.bundle_id,
            raw_len: bundle.raw_len,
            encoded_len: payload.len() as u32,
            frame_hash: frame_hash(FrameHashAlgo::Xxh3_64, &raw),
            flags: 0,
        },
        payload,
    })
}

fn read_exact_at(
    file: &File,
    mut buf: &mut [u8],
    mut offset: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    while !buf.is_empty() {
        let read = file.read_at(buf, offset)?;
        if read == 0 {
            return Err("unexpected eof while reading bundle part".into());
        }
        offset += read as u64;
        buf = &mut buf[read..];
    }
    Ok(())
}

fn parse_codec(value: &str) -> Result<DataCodec, Box<dyn std::error::Error + Send + Sync>> {
    match value {
        "lz4" => Ok(DataCodec::Lz4),
        "zstd" => Ok(DataCodec::Zstd),
        other => Err(format!("unsupported codec: {other}").into()),
    }
}

fn parse_integrity(value: &str) -> Result<IntegrityMode, Box<dyn std::error::Error + Send + Sync>> {
    match value {
        "off" => Ok(IntegrityMode::Off),
        "fast" => Ok(IntegrityMode::Fast),
        "strong" => Ok(IntegrityMode::Strong),
        other => Err(format!("unsupported integrity mode: {other}").into()),
    }
}
