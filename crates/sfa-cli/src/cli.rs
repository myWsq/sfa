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
    #[arg(long, value_enum, default_value_t = DataCodec::Lz4)]
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
    /// Preserve uid/gid ownership metadata.
    #[arg(long, default_value_t = false)]
    pub preserve_owner: bool,
    /// Parse and summarize only; do not execute archival backend.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Args)]
pub struct UnpackArgs {
    /// Input archive path.
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
    /// Owner restore policy.
    #[arg(long, value_enum, default_value_t = RestoreOwnerPolicy::Auto)]
    pub restore_owner: RestoreOwnerPolicy,
    /// Parse and summarize only; do not execute unpack backend.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum DataCodec {
    Lz4,
    Zstd,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum IntegrityMode {
    Off,
    Fast,
    Strong,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum RestoreOwnerPolicy {
    Auto,
    Preserve,
    Never,
}

fn default_threads() -> usize {
    std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
}
