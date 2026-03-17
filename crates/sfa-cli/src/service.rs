use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::Instant;

use sfa_core::format::read_header;
use sfa_core::{
    DataCodec as CoreDataCodec, IntegrityMode as CoreIntegrityMode,
    OverwritePolicy as CoreOverwritePolicy, PackConfig, PackPhaseBreakdown, PackStats,
    RestoreOwnerPolicy as CoreRestoreOwnerPolicy, UnpackConfig, UnpackPhaseBreakdown, UnpackStats,
    UnpackWallBreakdown,
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

pub trait ArchiveService {
    fn pack(&self, req: PackRequest) -> Result<PackStats, CliError>;
    fn unpack(&self, req: UnpackRequest) -> Result<UnpackStats, CliError>;
}

#[derive(Default)]
pub struct RealArchiveService;

const STDIN_ARCHIVE_PATH: &str = "-";
const UNPACK_DIAGNOSTICS_JSON_ENV: &str = "SFA_UNPACK_DIAGNOSTICS_JSON";

impl ArchiveService for RealArchiveService {
    fn pack(&self, req: PackRequest) -> Result<PackStats, CliError> {
        if !req.input_dir.is_dir() {
            return Err(CliError::io(format!(
                "input directory does not exist: {}",
                req.input_dir.display()
            )));
        }
        if !req.dry_run {
            let config = PackConfig {
                codec: map_codec(req.codec),
                threads: req.threads,
                bundle_target_bytes: req.bundle_target_bytes,
                small_file_threshold: req.small_file_threshold,
                integrity: map_integrity(req.integrity),
                preserve_owner: req.preserve_owner,
                ..PackConfig::default()
            };
            let stats = sfa_unixfs::pack_directory(&req.input_dir, &req.output_archive, &config)
                .map_err(map_unixfs_error)?;
            return Ok(stats);
        }
        let start = Instant::now();
        let fs_stats = collect_dir_stats(&req.input_dir)?;
        Ok(PackStats {
            codec: req.codec.as_str().to_string(),
            threads: req.threads,
            bundle_target_bytes: req.bundle_target_bytes,
            small_file_threshold: req.small_file_threshold,
            entry_count: fs_stats.entry_count,
            bundle_count: estimate_bundle_count(fs_stats.raw_bytes, req.bundle_target_bytes),
            raw_bytes: fs_stats.raw_bytes,
            encoded_bytes: estimate_encoded_size(fs_stats.raw_bytes, req.codec),
            duration_ms: elapsed_ms(start.elapsed()),
            phase_breakdown: PackPhaseBreakdown::unavailable(
                "dry-run does not measure execution phases",
            ),
        })
    }

    fn unpack(&self, req: UnpackRequest) -> Result<UnpackStats, CliError> {
        let reads_stdin = reads_stdin_archive(&req.input_archive);
        if req.dry_run && reads_stdin {
            return Err(CliError::usage(
                "dry-run is not supported when reading an archive from stdin",
            ));
        }
        if !reads_stdin && !req.input_archive.is_file() {
            return Err(CliError::io(format!(
                "input archive does not exist: {}",
                req.input_archive.display()
            )));
        }
        if !req.dry_run {
            let config = build_unpack_config(&req);
            let diagnostics_path = std::env::var_os(UNPACK_DIAGNOSTICS_JSON_ENV)
                .filter(|value| !value.is_empty())
                .map(std::path::PathBuf::from);
            let stats = if let Some(path) = diagnostics_path.as_ref() {
                let (stats, diagnostics) = if reads_stdin {
                    let stdin = std::io::stdin();
                    unpack_from_reader_with_diagnostics(stdin.lock(), &req.output_dir, &config)?
                } else {
                    sfa_unixfs::unpack_archive_with_diagnostics(
                        &req.input_archive,
                        &req.output_dir,
                        &config,
                    )
                    .map_err(map_unixfs_error)?
                };
                write_unpack_diagnostics(path, &stats, &diagnostics)?;
                stats
            } else if reads_stdin {
                let stdin = std::io::stdin();
                unpack_from_reader(stdin.lock(), &req.output_dir, &config)?
            } else {
                sfa_unixfs::unpack_archive(&req.input_archive, &req.output_dir, &config)
                    .map_err(map_unixfs_error)?
            };
            return Ok(stats);
        }
        let start = Instant::now();
        let archive_size = fs::metadata(&req.input_archive)
            .map_err(|e| CliError::io(format!("failed to read archive metadata: {e}")))?
            .len();
        let header = read_archive_header(&req.input_archive);
        let threads = req.threads.unwrap_or_else(|| {
            header
                .as_ref()
                .map(|header| usize::from(header.suggested_parallelism.max(1)))
                .unwrap_or_else(|| {
                    std::thread::available_parallelism()
                        .map(usize::from)
                        .unwrap_or(1)
                })
        });
        let codec = header
            .as_ref()
            .map(map_archive_codec)
            .unwrap_or(DataCodec::Zstd);
        let unavailable_note = "dry-run does not measure execution phases";
        Ok(UnpackStats {
            codec: codec.as_str().to_string(),
            threads,
            entry_count: 0,
            bundle_count: estimate_bundle_count(archive_size, 4 * 1024 * 1024),
            raw_bytes: archive_size,
            encoded_bytes: archive_size,
            duration_ms: elapsed_ms(start.elapsed()),
            wall_breakdown: UnpackWallBreakdown::unavailable(unavailable_note),
            phase_breakdown: UnpackPhaseBreakdown::unavailable(unavailable_note),
        })
    }
}

fn build_unpack_config(req: &UnpackRequest) -> UnpackConfig {
    UnpackConfig {
        threads: req.threads,
        overwrite: if req.overwrite {
            CoreOverwritePolicy::Replace
        } else {
            CoreOverwritePolicy::Error
        },
        restore_owner: map_restore_owner_policy(req.restore_owner),
        integrity: map_integrity(req.integrity),
    }
}

fn map_restore_owner_policy(policy: RestoreOwnerPolicy) -> CoreRestoreOwnerPolicy {
    match policy {
        // The CLI keeps owner restoration opt-in. `auto` and `never` both stay on the
        // default non-restoring path until a future surface redesign differentiates them.
        RestoreOwnerPolicy::Auto | RestoreOwnerPolicy::Never => CoreRestoreOwnerPolicy::Skip,
        RestoreOwnerPolicy::Preserve => CoreRestoreOwnerPolicy::Preserve,
    }
}

fn unpack_from_reader<R: Read>(
    reader: R,
    output_dir: &Path,
    config: &UnpackConfig,
) -> Result<UnpackStats, CliError> {
    sfa_unixfs::unpack_reader_to_dir(reader, output_dir, config).map_err(map_unixfs_error)
}

fn unpack_from_reader_with_diagnostics<R: Read>(
    reader: R,
    output_dir: &Path,
    config: &UnpackConfig,
) -> Result<(UnpackStats, sfa_unixfs::UnpackDiagnostics), CliError> {
    sfa_unixfs::unpack_reader_to_dir_with_diagnostics(reader, output_dir, config)
        .map_err(map_unixfs_error)
}

fn reads_stdin_archive(path: &Path) -> bool {
    path == Path::new(STDIN_ARCHIVE_PATH)
}

fn write_unpack_diagnostics(
    path: &Path,
    stats: &UnpackStats,
    diagnostics: &sfa_unixfs::UnpackDiagnostics,
) -> Result<(), CliError> {
    #[derive(serde::Serialize)]
    struct DiagnosticsReport<'a> {
        stats: &'a UnpackStats,
        diagnostics: &'a sfa_unixfs::UnpackDiagnostics,
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| CliError::io(format!("failed to create diagnostics directory: {e}")))?;
    }
    let json = serde_json::to_vec_pretty(&DiagnosticsReport { stats, diagnostics })
        .map_err(|e| CliError::internal(format!("failed to encode unpack diagnostics: {e}")))?;
    fs::write(path, json)
        .map_err(|e| CliError::io(format!("failed to write unpack diagnostics: {e}")))?;
    Ok(())
}

fn map_codec(codec: DataCodec) -> CoreDataCodec {
    match codec {
        DataCodec::Lz4 => CoreDataCodec::Lz4,
        DataCodec::Zstd => CoreDataCodec::Zstd,
    }
}

fn read_archive_header(path: &Path) -> Option<sfa_core::HeaderV1> {
    let mut file = File::open(path).ok()?;
    read_header(&mut file).ok()
}

fn map_archive_codec(header: &sfa_core::HeaderV1) -> DataCodec {
    match header.data_codec {
        CoreDataCodec::Lz4 => DataCodec::Lz4,
        CoreDataCodec::Zstd => DataCodec::Zstd,
        CoreDataCodec::None => DataCodec::Lz4,
    }
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

fn elapsed_ms(duration: std::time::Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}

#[cfg(test)]
mod tests {
    use crate::cli::RestoreOwnerPolicy;

    use super::map_restore_owner_policy;

    #[test]
    fn auto_and_never_owner_policies_map_to_skip() {
        assert_eq!(
            map_restore_owner_policy(RestoreOwnerPolicy::Auto),
            sfa_core::RestoreOwnerPolicy::Skip
        );
        assert_eq!(
            map_restore_owner_policy(RestoreOwnerPolicy::Never),
            sfa_core::RestoreOwnerPolicy::Skip
        );
    }

    #[test]
    fn preserve_owner_policy_maps_to_core_preserve() {
        assert_eq!(
            map_restore_owner_policy(RestoreOwnerPolicy::Preserve),
            sfa_core::RestoreOwnerPolicy::Preserve
        );
    }
}
