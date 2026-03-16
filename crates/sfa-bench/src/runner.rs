use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use walkdir::WalkDir;

use crate::harness::{
    BenchmarkJob, Codec, CommandSpec, archive_path, build_pack_command, build_unpack_command,
    unpack_dir,
};
use crate::report::{
    BenchmarkEnvironment, BenchmarkRecord, BenchmarkSuiteReport, CodecToolMetadata, DatasetSummary,
    ToolMetadata,
};

#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub sfa_bin: PathBuf,
    pub dry_run: bool,
    pub invocation: String,
}

impl RunnerConfig {
    pub fn new(sfa_bin: PathBuf, dry_run: bool, invocation: String) -> Self {
        Self {
            sfa_bin,
            dry_run,
            invocation,
        }
    }
}

#[derive(Debug, Clone)]
struct PreparedTools {
    resolved_sfa_bin: PathBuf,
    tar_bin: PathBuf,
    codec_bins: BTreeMap<Codec, PathBuf>,
    environment: BenchmarkEnvironment,
}

pub fn run_jobs(
    jobs: &[BenchmarkJob],
    cfg: &RunnerConfig,
) -> Result<BenchmarkSuiteReport, Box<dyn std::error::Error + Send + Sync>> {
    let datasets = summarize_datasets(jobs)?;
    let tools = prepare_tools(jobs, cfg)?;
    let mut report = BenchmarkSuiteReport::new(
        cfg.invocation.clone(),
        cfg.dry_run,
        tools.environment.clone(),
        datasets,
    )
    .stamp();

    for job in jobs {
        prepare_job_workspace(job)?;

        let codec_bin = tools
            .codec_bins
            .get(&job.codec)
            .ok_or_else(|| format!("missing codec tool for {}", job.codec.as_str()))?;
        let pack = build_pack_command(job, &tools.resolved_sfa_bin, &tools.tar_bin, codec_bin);
        let unpack = build_unpack_command(job, &tools.resolved_sfa_bin, &tools.tar_bin, codec_bin);
        report
            .records
            .push(run_record(job, "pack", &pack, cfg.dry_run)?);
        report
            .records
            .push(run_record(job, "unpack", &unpack, cfg.dry_run)?);
        cleanup_job_workspace(job)?;
    }
    Ok(report)
}

fn prepare_job_workspace(
    job: &BenchmarkJob,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    std::fs::create_dir_all(&job.case.output_dir).map_err(|e| {
        format!(
            "failed to create output dir {}: {e}",
            job.case.output_dir.display()
        )
    })?;
    let archive = archive_path(job);
    let unpack = unpack_dir(job);
    remove_path_if_exists(&archive)?;
    remove_path_if_exists(&unpack)?;
    std::fs::create_dir_all(&unpack)
        .map_err(|e| format!("failed to create unpack dir {}: {e}", unpack.display()))?;
    Ok(())
}

fn cleanup_job_workspace(
    job: &BenchmarkJob,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    remove_path_if_exists(&archive_path(job))?;
    remove_path_if_exists(&unpack_dir(job))?;
    Ok(())
}

fn run_record(
    job: &BenchmarkJob,
    phase: &'static str,
    cmd: &CommandSpec,
    dry_run: bool,
) -> Result<BenchmarkRecord, Box<dyn std::error::Error + Send + Sync>> {
    if dry_run {
        return Ok(BenchmarkRecord::from_dry_run(
            job,
            phase,
            cmd.to_shell_line(),
        ));
    }

    let started = Instant::now();
    let output =
        command_output(cmd).map_err(|e| format!("command failed to spawn: {cmd:?}: {e}"))?;
    let elapsed_ms = started.elapsed().as_millis() as u64;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let status = output.status.code().unwrap_or(1);
    if status != 0 {
        return Err(format!(
            "benchmark command failed for {} {} {} {}: exit {status}\ncommand: {}\nstdout:\n{}\nstderr:\n{}",
            job.case.name,
            match job.baseline {
                crate::harness::Baseline::Sfa => "sfa",
                crate::harness::Baseline::Tar => "tar",
            },
            job.codec.as_str(),
            phase,
            cmd.to_shell_line(),
            stdout,
            stderr,
        )
        .into());
    }

    Ok(BenchmarkRecord::from_execution(
        job,
        phase,
        cmd.to_shell_line(),
        elapsed_ms,
        status,
        stdout,
        stderr,
    ))
}

fn command_output(
    cmd: &CommandSpec,
) -> Result<std::process::Output, Box<dyn std::error::Error + Send + Sync>> {
    Command::new(&cmd.program)
        .args(&cmd.args)
        .env("LC_ALL", "C")
        .output()
        .map_err(|e| format!("unable to execute {}: {e}", cmd.to_shell_line()).into())
}

pub fn write_report(
    report: &BenchmarkSuiteReport,
    out: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(report)?;
    std::fs::write(out, json)?;
    Ok(())
}

fn prepare_tools(
    jobs: &[BenchmarkJob],
    cfg: &RunnerConfig,
) -> Result<PreparedTools, Box<dyn std::error::Error + Send + Sync>> {
    let resolved_sfa_bin = resolve_sfa_bin(&cfg.sfa_bin);
    let tar = resolve_optional_command("tar")?;
    let lz4 = resolve_optional_command("lz4")?;
    let zstd = resolve_optional_command("zstd")?;

    if !cfg.dry_run {
        ensure_input_dirs_populated(jobs)?;
        let sfa_bin = resolved_sfa_bin
            .clone()
            .ok_or_else(|| missing_sfa_bin_message(&cfg.sfa_bin))?;
        if !sfa_bin.is_file() {
            return Err(format!("SFA binary is not executable: {}", sfa_bin.display()).into());
        }
        let tar_bin = tar
            .clone()
            .ok_or_else(|| "required tool `tar` was not found on PATH".to_string())?;
        let lz4_bin = lz4
            .clone()
            .ok_or_else(|| "required tool `lz4` was not found on PATH".to_string())?;
        let zstd_bin = zstd
            .clone()
            .ok_or_else(|| "required tool `zstd` was not found on PATH".to_string())?;

        Ok(PreparedTools {
            resolved_sfa_bin: sfa_bin,
            tar_bin,
            codec_bins: BTreeMap::from([(Codec::Lz4, lz4_bin), (Codec::Zstd, zstd_bin)]),
            environment: build_environment(resolved_sfa_bin, tar, lz4, zstd),
        })
    } else {
        Ok(PreparedTools {
            resolved_sfa_bin: resolved_sfa_bin
                .clone()
                .unwrap_or_else(|| cfg.sfa_bin.clone()),
            tar_bin: tar.clone().unwrap_or_else(|| PathBuf::from("tar")),
            codec_bins: BTreeMap::from([
                (
                    Codec::Lz4,
                    lz4.clone().unwrap_or_else(|| PathBuf::from("lz4")),
                ),
                (
                    Codec::Zstd,
                    zstd.clone().unwrap_or_else(|| PathBuf::from("zstd")),
                ),
            ]),
            environment: build_environment(resolved_sfa_bin, tar, lz4, zstd),
        })
    }
}

fn ensure_input_dirs_populated(
    jobs: &[BenchmarkJob],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for summary in summarize_datasets(jobs)? {
        if summary.file_count == 0 || summary.total_bytes == 0 {
            return Err(format!(
                "benchmark dataset `{}` does not contain committed input files under {}",
                summary.dataset, summary.input_dir
            )
            .into());
        }
    }
    Ok(())
}

fn summarize_datasets(
    jobs: &[BenchmarkJob],
) -> Result<Vec<DatasetSummary>, Box<dyn std::error::Error + Send + Sync>> {
    let mut unique_cases = BTreeMap::new();
    for job in jobs {
        unique_cases
            .entry(job.case.name.clone())
            .or_insert_with(|| job.case.clone());
    }

    unique_cases
        .into_values()
        .map(|case| summarize_dataset(&case))
        .collect()
}

fn summarize_dataset(
    case: &crate::harness::DatasetCase,
) -> Result<DatasetSummary, Box<dyn std::error::Error + Send + Sync>> {
    if !case.input_dir.is_dir() {
        return Err(format!(
            "benchmark input dir is missing: {}",
            case.input_dir.display()
        )
        .into());
    }

    let mut file_count = 0u64;
    let mut directory_count = 0u64;
    let mut symlink_count = 0u64;
    let mut total_bytes = 0u64;

    for entry in WalkDir::new(&case.input_dir).follow_links(false) {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            file_count += 1;
            total_bytes += metadata.len();
        } else if metadata.is_dir() {
            directory_count += 1;
        } else if metadata.file_type().is_symlink() {
            symlink_count += 1;
        }
    }

    Ok(DatasetSummary {
        dataset: case.name.clone(),
        input_dir: case.input_dir.display().to_string(),
        file_count,
        directory_count,
        symlink_count,
        total_bytes,
    })
}

fn build_environment(
    sfa_bin: Option<PathBuf>,
    tar_bin: Option<PathBuf>,
    lz4_bin: Option<PathBuf>,
    zstd_bin: Option<PathBuf>,
) -> BenchmarkEnvironment {
    BenchmarkEnvironment {
        host_os: std::env::consts::OS.to_string(),
        host_arch: std::env::consts::ARCH.to_string(),
        tar: tool_metadata("tar", tar_bin.as_deref()),
        sfa: tool_metadata("sfa", sfa_bin.as_deref()),
        codecs: vec![
            CodecToolMetadata {
                codec: Codec::Lz4,
                tool: tool_metadata("lz4", lz4_bin.as_deref()),
            },
            CodecToolMetadata {
                codec: Codec::Zstd,
                tool: tool_metadata("zstd", zstd_bin.as_deref()),
            },
        ],
    }
}

fn tool_metadata(name: &str, path: Option<&Path>) -> ToolMetadata {
    ToolMetadata {
        name: name.to_string(),
        path: path.map(|path| path.display().to_string()),
        version: path.and_then(|path| capture_version(path)),
    }
}

fn capture_version(path: &Path) -> Option<String> {
    let output = Command::new(path)
        .arg("--version")
        .env("LC_ALL", "C")
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    stdout
        .lines()
        .chain(stderr.lines())
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

fn resolve_sfa_bin(requested: &Path) -> Option<PathBuf> {
    if requested.components().count() > 1 || requested.is_absolute() {
        return requested.exists().then(|| requested.to_path_buf());
    }

    [
        PathBuf::from("target/release/sfa"),
        PathBuf::from("target/release/sfa-cli"),
        PathBuf::from("target/debug/sfa"),
        PathBuf::from("target/debug/sfa-cli"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
    .or_else(|| {
        resolve_optional_command(requested.to_str().unwrap_or("sfa"))
            .ok()
            .flatten()
    })
    .or_else(|| resolve_optional_command("sfa-cli").ok().flatten())
}

fn resolve_optional_command(
    name: &str,
) -> Result<Option<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    if name.contains(std::path::MAIN_SEPARATOR) {
        let path = PathBuf::from(name);
        return Ok(path.exists().then_some(path));
    }

    let path_var = match std::env::var_os("PATH") {
        Some(path_var) => path_var,
        None => return Ok(None),
    };

    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.exists() {
            return Ok(Some(candidate));
        }
    }

    Ok(None)
}

fn missing_sfa_bin_message(requested: &Path) -> String {
    format!(
        "could not find the SFA CLI binary for benchmark execution. Checked `{}` and the local build outputs `target/release/sfa`, `target/release/sfa-cli`, `target/debug/sfa`, and `target/debug/sfa-cli`. Build it first with `cargo build --release -p sfa-cli` or pass `--sfa-bin <path>`.",
        requested.display()
    )
}

fn remove_path_if_exists(path: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        std::fs::remove_dir_all(path)?;
    } else {
        std::fs::remove_file(path)?;
    }
    Ok(())
}
