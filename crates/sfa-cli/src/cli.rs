use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Parser)]
#[command(
    name = "sfa",
    about = "Small File Archive CLI",
    version,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Pack(PackArgs),
    Unpack(UnpackArgs),
}

#[derive(Debug, Clone, Args)]
pub struct PackArgs {
    /// Input directory to archive.
    pub input_dir: PathBuf,
    /// Output archive path (.sfa).
    pub output_archive: PathBuf,
    /// Compression codec.
    #[arg(long, value_enum, default_value_t = DataCodec::Zstd)]
    pub codec: DataCodec,
    /// Worker thread count.
    #[arg(long, default_value_t = default_threads())]
    pub threads: usize,
    /// Target raw bundle size in bytes.
    #[arg(long, default_value_t = 4 * 1024 * 1024)]
    pub bundle_target_bytes: u32,
    /// Threshold under which regular files are aggregated.
    #[arg(long, default_value_t = 256 * 1024)]
    pub small_file_threshold: u32,
    /// Integrity mode.
    #[arg(long, value_enum, default_value_t = IntegrityMode::Fast)]
    pub integrity: IntegrityMode,
    /// Mark archive owner-preservation intent for later opt-in uid/gid restore.
    #[arg(long, default_value_t = false)]
    pub preserve_owner: bool,
    /// Parse and summarize only; do not execute archival backend.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
    /// Output format for command statistics.
    #[arg(long, value_enum, default_value_t = StatsFormat::Human)]
    pub stats_format: StatsFormat,
}

#[derive(Debug, Clone, Args)]
pub struct UnpackArgs {
    /// Input archive path, or `-` to read from stdin.
    pub input_archive: PathBuf,
    /// Output directory root.
    #[arg(short = 'C', long = "directory", default_value = ".")]
    pub output_dir: PathBuf,
    /// Optional worker thread override.
    #[arg(long)]
    pub threads: Option<usize>,
    /// Overwrite existing output files.
    #[arg(long, default_value_t = false)]
    pub overwrite: bool,
    /// Integrity policy used during decode.
    #[arg(long, value_enum, default_value_t = IntegrityMode::Fast)]
    pub integrity: IntegrityMode,
    /// Owner restore policy. `auto` and `never` keep owner restore disabled;
    /// `preserve` attempts uid/gid restore when unpack runs as root.
    #[arg(long, value_enum, default_value_t = RestoreOwnerPolicy::Auto)]
    pub restore_owner: RestoreOwnerPolicy,
    /// Parse and summarize only; do not execute unpack backend.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
    /// Output format for command statistics.
    #[arg(long, value_enum, default_value_t = StatsFormat::Human)]
    pub stats_format: StatsFormat,
}

#[derive(Debug, Clone, Copy, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataCodec {
    Lz4,
    Zstd,
}

impl DataCodec {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lz4 => "lz4",
            Self::Zstd => "zstd",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrityMode {
    Off,
    Fast,
    Strong,
}

#[derive(Debug, Clone, Copy, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RestoreOwnerPolicy {
    /// Default path: keep restored ownership on the current process identity.
    Auto,
    /// Attempt to apply stored uid/gid metadata when unpack runs as root.
    Preserve,
    /// Explicitly disable owner restoration even when archives carry owner metadata.
    Never,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum StatsFormat {
    Human,
    Json,
}

fn default_threads() -> usize {
    std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
}
