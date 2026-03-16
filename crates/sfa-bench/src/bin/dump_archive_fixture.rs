use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Serialize;
use sfa_core::codec::decode_data;
use sfa_core::format::FeatureFlags;
use sfa_core::integrity::frame_hash;
use sfa_core::{ArchiveReader, Manifest};

#[derive(Debug, Serialize)]
struct FixtureDump {
    archive_bytes: u64,
    header: HeaderDump,
    manifest: ManifestDump,
    frames: Vec<FrameDump>,
    trailer: Option<TrailerDump>,
}

#[derive(Debug, Serialize)]
struct HeaderDump {
    version_major: u16,
    version_minor: u16,
    writer_version_major: u16,
    writer_version_minor: u16,
    data_codec: String,
    manifest_codec: String,
    integrity_mode: String,
    frame_hash_algo: String,
    manifest_hash_algo: String,
    suggested_parallelism: u16,
    bundle_target_bytes: u32,
    small_file_threshold: u32,
    entry_count: u64,
    extent_count: u64,
    bundle_count: u64,
    manifest_raw_len: u64,
    manifest_encoded_len: u64,
    feature_flags_bits: u64,
    feature_flags: Vec<String>,
    manifest_hash_hex: String,
}

#[derive(Debug, Serialize)]
struct ManifestDump {
    entry_count: usize,
    extent_count: usize,
    bundle_count: usize,
    name_arena_bytes: usize,
    meta_blob_bytes: usize,
    entries: Vec<EntryDump>,
    extents: Vec<ExtentDump>,
    bundles: Vec<BundleDump>,
}

#[derive(Debug, Serialize)]
struct EntryDump {
    entry_id: u32,
    parent_id: Option<u32>,
    kind: String,
    flags_bits: u8,
    flags: Vec<String>,
    path: String,
    name_utf8: String,
    name_hex: String,
    mode_octal: String,
    uid: u32,
    gid: u32,
    mtime_sec: i64,
    mtime_nsec: u32,
    size: u64,
    link_target_utf8: Option<String>,
    link_target_hex: Option<String>,
    first_extent: u64,
    extent_count: u32,
    hardlink_master_entry_id: Option<u32>,
    meta_len: u32,
}

#[derive(Debug, Serialize)]
struct ExtentDump {
    entry_id: u32,
    entry_path: String,
    bundle_id: u64,
    file_offset: u64,
    raw_offset_in_bundle: u32,
    raw_len: u32,
    flags_bits: u32,
    flags: Vec<String>,
}

#[derive(Debug, Serialize)]
struct BundleDump {
    bundle_id: u64,
    kind: String,
    raw_len: u32,
    file_count: u32,
    extent_count: u32,
    flags: u8,
}

#[derive(Debug, Serialize)]
struct FrameDump {
    bundle_id: u64,
    raw_len: u32,
    encoded_len: u32,
    flags: u16,
    frame_hash_hex: String,
}

#[derive(Debug, Serialize)]
struct TrailerDump {
    bundle_count: u64,
    total_raw_bytes: u64,
    total_encoded_bytes: u64,
    archive_hash_hex: String,
}

#[derive(Debug, Serialize)]
struct FixtureSummary {
    archive_bytes: u64,
    version_major: u16,
    version_minor: u16,
    data_codec: String,
    manifest_codec: String,
    integrity_mode: String,
    frame_hash_algo: String,
    manifest_hash_algo: String,
    feature_flags: Vec<String>,
    entry_count: u64,
    extent_count: u64,
    bundle_count: u64,
    frame_count: usize,
    manifest_raw_len: u64,
    manifest_encoded_len: u64,
    total_frame_raw_bytes: u64,
    total_frame_encoded_bytes: u64,
    has_trailer: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "Usage: dump_archive_fixture <archive.sfa> <manifest.json> [--summary-out <stats.json>]"
        );
        std::process::exit(2);
    }

    let archive = PathBuf::from(&args[1]);
    let manifest_output = PathBuf::from(&args[2]);
    let mut summary_output = None;

    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--summary-out" => {
                index += 1;
                let value = args.get(index).ok_or("--summary-out needs a value")?;
                summary_output = Some(PathBuf::from(value));
            }
            other => {
                return Err(format!("unknown argument: {other}").into());
            }
        }
        index += 1;
    }

    let archive_bytes = std::fs::metadata(&archive)?.len();
    let reader = File::open(&archive)?;
    let mut archive_reader = ArchiveReader::new(reader);
    let header = archive_reader.read_header()?;
    let manifest = archive_reader.read_manifest()?;
    let manifest_dump = dump_manifest(&manifest);

    let frames = collect_frames_verified(
        &mut archive_reader,
        header.data_codec,
        header.frame_hash_algo,
    )?;
    let trailer = archive_reader.read_trailer()?.map(|trailer| TrailerDump {
        bundle_count: trailer.bundle_count,
        total_raw_bytes: trailer.total_raw_bytes,
        total_encoded_bytes: trailer.total_encoded_bytes,
        archive_hash_hex: hex_bytes(&trailer.archive_hash),
    });

    let feature_flags = feature_flag_names(header.feature_flags);
    let dump = FixtureDump {
        archive_bytes,
        header: HeaderDump {
            version_major: header.version_major,
            version_minor: header.version_minor,
            writer_version_major: header.writer_version_major,
            writer_version_minor: header.writer_version_minor,
            data_codec: format!("{:?}", header.data_codec).to_lowercase(),
            manifest_codec: format!("{:?}", header.manifest_codec).to_lowercase(),
            integrity_mode: format!("{:?}", header.integrity_mode).to_lowercase(),
            frame_hash_algo: format!("{:?}", header.frame_hash_algo).to_lowercase(),
            manifest_hash_algo: format!("{:?}", header.manifest_hash_algo).to_lowercase(),
            suggested_parallelism: header.suggested_parallelism,
            bundle_target_bytes: header.bundle_target_bytes,
            small_file_threshold: header.small_file_threshold,
            entry_count: header.entry_count,
            extent_count: header.extent_count,
            bundle_count: header.bundle_count,
            manifest_raw_len: header.manifest_raw_len,
            manifest_encoded_len: header.manifest_encoded_len,
            feature_flags_bits: header.feature_flags.bits(),
            feature_flags: feature_flags.clone(),
            manifest_hash_hex: hex_bytes(&header.manifest_hash),
        },
        manifest: manifest_dump,
        frames,
        trailer,
    };

    write_json(&manifest_output, &dump)?;

    if let Some(summary_output) = summary_output {
        let summary = FixtureSummary {
            archive_bytes: dump.archive_bytes,
            version_major: dump.header.version_major,
            version_minor: dump.header.version_minor,
            data_codec: dump.header.data_codec.clone(),
            manifest_codec: dump.header.manifest_codec.clone(),
            integrity_mode: dump.header.integrity_mode.clone(),
            frame_hash_algo: dump.header.frame_hash_algo.clone(),
            manifest_hash_algo: dump.header.manifest_hash_algo.clone(),
            feature_flags,
            entry_count: dump.header.entry_count,
            extent_count: dump.header.extent_count,
            bundle_count: dump.header.bundle_count,
            frame_count: dump.frames.len(),
            manifest_raw_len: dump.header.manifest_raw_len,
            manifest_encoded_len: dump.header.manifest_encoded_len,
            total_frame_raw_bytes: dump
                .frames
                .iter()
                .map(|frame| u64::from(frame.raw_len))
                .sum(),
            total_frame_encoded_bytes: dump
                .frames
                .iter()
                .map(|frame| u64::from(frame.encoded_len))
                .sum(),
            has_trailer: dump.trailer.is_some(),
        };
        write_json(&summary_output, &summary)?;
    }

    Ok(())
}

fn collect_frames_verified<R: Read>(
    archive_reader: &mut ArchiveReader<R>,
    codec: sfa_core::DataCodec,
    hash_algo: sfa_core::FrameHashAlgo,
) -> Result<Vec<FrameDump>, sfa_core::Error> {
    let mut frames = Vec::new();
    while let Some(frame) = archive_reader.next_frame()? {
        let raw = decode_data(codec, &frame.payload, frame.header.raw_len as usize)?;
        if frame_hash(hash_algo, &raw) != frame.header.frame_hash {
            return Err(sfa_core::Error::FrameHashMismatch {
                bundle_id: frame.header.bundle_id,
            });
        }
        frames.push(FrameDump {
            bundle_id: frame.header.bundle_id,
            raw_len: frame.header.raw_len,
            encoded_len: frame.header.encoded_len,
            flags: frame.header.flags,
            frame_hash_hex: hex_u64(frame.header.frame_hash),
        });
    }
    Ok(frames)
}

fn dump_manifest(manifest: &Manifest) -> ManifestDump {
    let paths = build_entry_paths(manifest);
    let entries = manifest
        .entries
        .iter()
        .enumerate()
        .map(|(entry_id, entry)| {
            let name = arena_slice(&manifest.name_arena, entry.name_off, entry.name_len);
            let link_target = (entry.link_len > 0)
                .then(|| arena_slice(&manifest.name_arena, entry.link_off, entry.link_len));
            EntryDump {
                entry_id: entry_id as u32,
                parent_id: (entry.parent_id != u32::MAX).then_some(entry.parent_id),
                kind: format!("{:?}", entry.kind).to_lowercase(),
                flags_bits: entry.flags,
                flags: entry_flag_names(entry.flags),
                path: paths[entry_id].clone(),
                name_utf8: String::from_utf8_lossy(name).into_owned(),
                name_hex: hex_bytes(name),
                mode_octal: format!("{:#o}", entry.mode),
                uid: entry.uid,
                gid: entry.gid,
                mtime_sec: entry.mtime_sec,
                mtime_nsec: entry.mtime_nsec,
                size: entry.size,
                link_target_utf8: link_target
                    .as_ref()
                    .map(|value| String::from_utf8_lossy(value).into_owned()),
                link_target_hex: link_target.as_ref().map(|value| hex_bytes(value)),
                first_extent: entry.first_extent,
                extent_count: entry.extent_count,
                hardlink_master_entry_id: (entry.hardlink_master_entry_id != u32::MAX)
                    .then_some(entry.hardlink_master_entry_id),
                meta_len: entry.meta_len,
            }
        })
        .collect();
    let extents = manifest
        .extents
        .iter()
        .map(|extent| ExtentDump {
            entry_id: extent.entry_id,
            entry_path: paths[extent.entry_id as usize].clone(),
            bundle_id: extent.bundle_id,
            file_offset: extent.file_offset,
            raw_offset_in_bundle: extent.raw_offset_in_bundle,
            raw_len: extent.raw_len,
            flags_bits: extent.flags,
            flags: extent_flag_names(extent.flags),
        })
        .collect();
    let bundles = manifest
        .bundles
        .iter()
        .map(|bundle| BundleDump {
            bundle_id: bundle.bundle_id,
            kind: format!("{:?}", bundle.kind).to_lowercase(),
            raw_len: bundle.raw_len,
            file_count: bundle.file_count,
            extent_count: bundle.extent_count,
            flags: bundle.flags,
        })
        .collect();

    ManifestDump {
        entry_count: manifest.entries.len(),
        extent_count: manifest.extents.len(),
        bundle_count: manifest.bundles.len(),
        name_arena_bytes: manifest.name_arena.len(),
        meta_blob_bytes: manifest.meta_blob.len(),
        entries,
        extents,
        bundles,
    }
}

fn build_entry_paths(manifest: &Manifest) -> Vec<String> {
    let mut paths = vec![String::new(); manifest.entries.len()];
    for (entry_id, entry) in manifest.entries.iter().enumerate() {
        if entry_id == 0 {
            paths[entry_id] = ".".to_string();
            continue;
        }

        let parent = if entry.parent_id == u32::MAX {
            ".".to_string()
        } else {
            paths[entry.parent_id as usize].clone()
        };
        let name = String::from_utf8_lossy(arena_slice(
            &manifest.name_arena,
            entry.name_off,
            entry.name_len,
        ))
        .into_owned();
        paths[entry_id] = if parent == "." {
            name
        } else {
            format!("{parent}/{name}")
        };
    }
    paths
}

fn arena_slice(arena: &[u8], off: u32, len: u32) -> &[u8] {
    let start = off as usize;
    let end = start + len as usize;
    &arena[start..end]
}

fn feature_flag_names(flags: FeatureFlags) -> Vec<String> {
    let mut names = Vec::new();
    if flags.contains(FeatureFlags::HAS_SYMLINK) {
        names.push("has_symlink".to_string());
    }
    if flags.contains(FeatureFlags::HAS_HARDLINK) {
        names.push("has_hardlink".to_string());
    }
    if flags.contains(FeatureFlags::HAS_SPECIAL_FILE) {
        names.push("has_special_file".to_string());
    }
    if flags.contains(FeatureFlags::HAS_METADATA) {
        names.push("has_metadata".to_string());
    }
    if flags.contains(FeatureFlags::HAS_TRAILER) {
        names.push("has_trailer".to_string());
    }
    if flags.contains(FeatureFlags::PRESERVE_OWNER) {
        names.push("preserve_owner".to_string());
    }
    names
}

fn entry_flag_names(flags: u8) -> Vec<String> {
    let mut names = Vec::new();
    if flags & (1 << 0) != 0 {
        names.push("empty_file".to_string());
    }
    if flags & (1 << 1) != 0 {
        names.push("path_validated".to_string());
    }
    if flags & (1 << 2) != 0 {
        names.push("has_metadata".to_string());
    }
    names
}

fn extent_flag_names(flags: u32) -> Vec<String> {
    let mut names = Vec::new();
    if flags & (1 << 0) != 0 {
        names.push("last_extent".to_string());
    }
    if flags & (1 << 1) != 0 {
        names.push("aggregate_bundle".to_string());
    }
    if flags & (1 << 2) != 0 {
        names.push("single_file_bundle".to_string());
    }
    names
}

fn write_json<T: Serialize>(
    path: &Path,
    value: &T,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(value)?)?;
    Ok(())
}

fn hex_u64(value: u64) -> String {
    format!("{value:016x}")
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(nibble(byte >> 4));
        out.push(nibble(byte & 0x0f));
    }
    out
}

fn nibble(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + (value - 10)) as char,
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use sfa_core::PackConfig;
    use sfa_unixfs::pack_directory;

    use super::*;

    #[test]
    fn collect_frames_verified_rejects_corrupted_payload() {
        let temp = TempDir::new().expect("temp");
        let archive = temp.path().join("sample.sfa");
        fs::write(temp.path().join("one.txt"), b"one").expect("write input");
        pack_directory(temp.path(), &archive, &PackConfig::default()).expect("pack");

        let mut bytes = fs::read(&archive).expect("read archive");
        let idx = bytes.len() - 1;
        bytes[idx] ^= 0x55;
        fs::write(&archive, bytes).expect("rewrite archive");

        let reader = File::open(&archive).expect("open archive");
        let mut archive_reader = ArchiveReader::new(reader);
        let header = archive_reader.read_header().expect("header");
        archive_reader.read_manifest().expect("manifest");

        let err = collect_frames_verified(
            &mut archive_reader,
            header.data_codec,
            header.frame_hash_algo,
        )
        .expect_err("corrupted frame payload must fail");
        assert!(matches!(err, sfa_core::Error::FrameHashMismatch { .. }));
    }
}
