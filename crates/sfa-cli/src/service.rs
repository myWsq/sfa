use std::fs;
use std::fs::File;
use std::path::Path;
use std::time::{Duration, Instant};

use sfa_core::format::read_header;
use sfa_core::{
    DataCodec as CoreDataCodec, IntegrityMode as CoreIntegrityMode,
    OverwritePolicy as CoreOverwritePolicy, PackConfig,
    RestoreOwnerPolicy as CoreRestoreOwnerPolicy, UnpackConfig,
};
use sfa_unixfs::UnixFsError;
use walkdir::WalkDir;

use crate::cli::{DataCodec, IntegrityMode, RestoreOwnerPolicy};
use crate::error::CliError;

#[derive(Debug, Clone)]
pub struct PackRequest {
    pub input_dir: std::path::PathBuf,
    pub output_archive: std::path::PathBuf,
    pub codec: DataCodec,
    pub threads: usize,
    pub bundle_target_bytes: u32,
    pub small_file_threshold: u32,
    pub integrity: IntegrityMode,
    pub preserve_owner: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct UnpackRequest {
    pub input_archive: std::path::PathBuf,
    pub output_dir: std::path::PathBuf,
    pub threads: Option<usize>,
    pub overwrite: bool,
    pub integrity: IntegrityMode,
    pub restore_owner: RestoreOwnerPolicy,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct RunStats {
    pub codec: DataCodec,
    pub threads: usize,
    pub bundle_target_bytes: u32,
    pub small_file_threshold: u32,
    pub entry_count: u64,
    pub bundle_count: u64,
    pub raw_bytes: u64,
    pub encoded_bytes: u64,
    pub duration: Duration,
}

pub trait ArchiveService {
    fn pack(&self, req: PackRequest) -> Result<RunStats, CliError>;
    fn unpack(&self, req: UnpackRequest) -> Result<RunStats, CliError>;
}

#[derive(Default)]
pub struct RealArchiveService;

impl ArchiveService for RealArchiveService {
    fn pack(&self, req: PackRequest) -> Result<RunStats, CliError> {
        if !req.input_dir.is_dir() {
            return Err(CliError::io(format!(
                "input directory does not exist: {}",
                req.input_dir.display()
            )));
        }
        if !req.dry_run {
            let config = PackConfig {
                codec: map_codec(req.codec),
                compression_level: None,
                threads: req.threads,
                bundle_target_bytes: req.bundle_target_bytes,
                small_file_threshold: req.small_file_threshold,
                integrity: map_integrity(req.integrity),
                preserve_owner: req.preserve_owner,
                ..PackConfig::default()
            };
            let stats = sfa_unixfs::pack_directory(&req.input_dir, &req.output_archive, &config)
                .map_err(map_unixfs_error)?;
            return Ok(RunStats {
                codec: req.codec,
                threads: stats.threads,
                bundle_target_bytes: stats.bundle_target_bytes,
                small_file_threshold: stats.small_file_threshold,
                entry_count: stats.entry_count,
                bundle_count: stats.bundle_count,
                raw_bytes: stats.raw_bytes,
                encoded_bytes: stats.encoded_bytes,
                duration: Duration::from_millis(stats.duration_ms as u64),
            });
        }
        let start = Instant::now();
        let fs_stats = collect_dir_stats(&req.input_dir)?;
        Ok(RunStats {
            codec: req.codec,
            threads: req.threads,
            bundle_target_bytes: req.bundle_target_bytes,
            small_file_threshold: req.small_file_threshold,
            entry_count: fs_stats.entry_count,
            bundle_count: estimate_bundle_count(fs_stats.raw_bytes, req.bundle_target_bytes),
            raw_bytes: fs_stats.raw_bytes,
            encoded_bytes: estimate_encoded_size(fs_stats.raw_bytes, req.codec),
            duration: start.elapsed(),
        })
    }

    fn unpack(&self, req: UnpackRequest) -> Result<RunStats, CliError> {
        if !req.input_archive.is_file() {
            return Err(CliError::io(format!(
                "input archive does not exist: {}",
                req.input_archive.display()
            )));
        }
        if !req.dry_run {
            let config = UnpackConfig {
                threads: req.threads,
                overwrite: if req.overwrite {
                    CoreOverwritePolicy::Replace
                } else {
                    CoreOverwritePolicy::Error
                },
                restore_owner: match req.restore_owner {
                    RestoreOwnerPolicy::Auto | RestoreOwnerPolicy::Never => {
                        CoreRestoreOwnerPolicy::Skip
                    }
                    RestoreOwnerPolicy::Preserve => CoreRestoreOwnerPolicy::Preserve,
                },
                integrity: map_integrity(req.integrity),
            };
            let stats = sfa_unixfs::unpack_archive(&req.input_archive, &req.output_dir, &config)
                .map_err(map_unixfs_error)?;
            return Ok(RunStats {
                codec: parse_codec_name(&stats.codec)?,
                threads: stats.threads,
                bundle_target_bytes: 4 * 1024 * 1024,
                small_file_threshold: 256 * 1024,
                entry_count: stats.entry_count,
                bundle_count: stats.bundle_count,
                raw_bytes: stats.raw_bytes,
                encoded_bytes: stats.encoded_bytes,
                duration: Duration::from_millis(stats.duration_ms as u64),
            });
        }
        let start = Instant::now();
        let archive_size = fs::metadata(&req.input_archive)
            .map_err(|e| CliError::io(format!("failed to read archive metadata: {e}")))?
            .len();
        let threads = req.threads.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(usize::from)
                .unwrap_or(1)
        });
        let codec = infer_archive_codec(&req.input_archive).unwrap_or(DataCodec::Lz4);
        Ok(RunStats {
            codec,
            threads,
            bundle_target_bytes: 4 * 1024 * 1024,
            small_file_threshold: 256 * 1024,
            entry_count: 0,
            bundle_count: estimate_bundle_count(archive_size, 4 * 1024 * 1024),
            raw_bytes: archive_size,
            encoded_bytes: archive_size,
            duration: start.elapsed(),
        })
    }
}

fn map_codec(codec: DataCodec) -> CoreDataCodec {
    match codec {
        DataCodec::Lz4 => CoreDataCodec::Lz4,
        DataCodec::Zstd => CoreDataCodec::Zstd,
    }
}

fn parse_codec_name(value: &str) -> Result<DataCodec, CliError> {
    match value {
        "lz4" => Ok(DataCodec::Lz4),
        "zstd" => Ok(DataCodec::Zstd),
        other => Err(CliError::internal(format!(
            "unknown codec in stats: {other}"
        ))),
    }
}

fn infer_archive_codec(path: &Path) -> Option<DataCodec> {
    let mut file = File::open(path).ok()?;
    let header = read_header(&mut file).ok()?;
    Some(match header.data_codec {
        CoreDataCodec::Lz4 => DataCodec::Lz4,
        CoreDataCodec::Zstd => DataCodec::Zstd,
        CoreDataCodec::None => DataCodec::Lz4,
    })
}

fn map_integrity(mode: IntegrityMode) -> CoreIntegrityMode {
    match mode {
        IntegrityMode::Off => CoreIntegrityMode::Off,
        IntegrityMode::Fast => CoreIntegrityMode::Fast,
        IntegrityMode::Strong => CoreIntegrityMode::Strong,
    }
}

fn map_unixfs_error(error: UnixFsError) -> CliError {
    match error {
        UnixFsError::Io(err) => CliError::io(err.to_string()),
        UnixFsError::PathValidation(err) => CliError::safety(err.to_string()),
        UnixFsError::UnsupportedEntryKind(path) => {
            CliError::safety(format!("unsupported entry kind: {}", path.display()))
        }
        UnixFsError::MissingParent(path) => {
            CliError::safety(format!("missing parent: {}", path.display()))
        }
        UnixFsError::InvalidState(msg) => CliError::internal(msg),
        UnixFsError::Core(err) => match err {
            sfa_core::Error::ManifestHashMismatch
            | sfa_core::Error::FrameHashMismatch { .. }
            | sfa_core::Error::TrailerHashMismatch => CliError::integrity(err.to_string()),
            sfa_core::Error::InvalidHeader(_)
            | sfa_core::Error::InvalidFrame(_)
            | sfa_core::Error::InvalidManifest(_)
            | sfa_core::Error::UnexpectedEof
            | sfa_core::Error::UnsupportedDataCodec(_)
            | sfa_core::Error::UnsupportedManifestCodec(_)
            | sfa_core::Error::UnsupportedIntegrityMode(_)
            | sfa_core::Error::UnsupportedFrameHashAlgo(_)
            | sfa_core::Error::UnsupportedManifestHashAlgo(_)
            | sfa_core::Error::UnsupportedEntryKind(_) => CliError::parse(err.to_string()),
            sfa_core::Error::InvalidPath(_) => CliError::safety(err.to_string()),
            sfa_core::Error::Io { source, .. } => CliError::io(source.to_string()),
            _ => CliError::internal(err.to_string()),
        },
    }
}

struct DirStats {
    entry_count: u64,
    raw_bytes: u64,
}

fn collect_dir_stats(root: &Path) -> Result<DirStats, CliError> {
    let mut entry_count = 0_u64;
    let mut raw_bytes = 0_u64;
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(|e| CliError::io(format!("walkdir error: {e}")))?;
        entry_count += 1;
        if entry.file_type().is_file() {
            let meta = entry
                .metadata()
                .map_err(|e| CliError::io(format!("metadata error: {e}")))?;
            raw_bytes = raw_bytes.saturating_add(meta.len());
        }
    }
    Ok(DirStats {
        entry_count,
        raw_bytes,
    })
}

fn estimate_bundle_count(raw_bytes: u64, bundle_target_bytes: u32) -> u64 {
    if raw_bytes == 0 {
        return 0;
    }
    let per_bundle = u64::from(bundle_target_bytes.max(1));
    raw_bytes.div_ceil(per_bundle)
}

fn estimate_encoded_size(raw_bytes: u64, codec: DataCodec) -> u64 {
    match codec {
        DataCodec::Lz4 => (raw_bytes as f64 * 0.72) as u64,
        DataCodec::Zstd => (raw_bytes as f64 * 0.58) as u64,
    }
}
