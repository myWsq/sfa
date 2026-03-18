use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::Instant;

use tempfile::TempDir;
use walkdir::WalkDir;

use crate::harness::{
    BenchmarkJob, BenchmarkPaths, CommandSpec, archive_path, build_pack_command,
    build_unpack_command, dry_run_paths, unpack_dir,
};
use crate::report::{
    BenchmarkEnvironment, BenchmarkRecord, BenchmarkSuiteReport, ResourceObservation,
    ResourceSamplerMetadata, SfaCommandStats, ToolMetadata,
};
use crate::workload::{BenchmarkWorkload, WorkloadSummary};
use sfa_core::{PackStats, UnpackStats};

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
    zstd_bin: PathBuf,
    environment: BenchmarkEnvironment,
    resource_sampler: ResourceSampler,
}

#[derive(Debug, Clone)]
enum ResourceSampler {
    Wait4Getrusage,
    #[cfg(not(unix))]
    Unsupported {
        note: String,
    },
}

impl ResourceSampler {
    fn detect() -> Self {
        #[cfg(unix)]
        {
            Self::Wait4Getrusage
        }
        #[cfg(not(unix))]
        {
            Self::Unsupported {
                note: "resource sampling requires a Unix host with wait4/getrusage".to_string(),
            }
        }
    }

    fn metadata(&self) -> ResourceSamplerMetadata {
        match self {
            Self::Wait4Getrusage => ResourceSamplerMetadata {
                name: "wait4/getrusage".to_string(),
                supported: true,
                note: Some(
                    "max_rss_kib is normalized from host-specific ru_maxrss units".to_string(),
                ),
            },
            #[cfg(not(unix))]
            Self::Unsupported { note } => ResourceSamplerMetadata {
                name: "wait4/getrusage".to_string(),
                supported: false,
                note: Some(note.clone()),
            },
        }
    }

    fn unavailable_observation(&self) -> ResourceObservation {
        match self {
            Self::Wait4Getrusage => ResourceObservation::unavailable(
                "wait4/getrusage",
                "resource observation was not collected",
            ),
            #[cfg(not(unix))]
            Self::Unsupported { note } => {
                ResourceObservation::unavailable("wait4/getrusage", note.clone())
            }
        }
    }
}

struct CommandExecution {
    output: Output,
    resource_observation: Option<ResourceObservation>,
}

pub fn run_jobs(
    jobs: &[BenchmarkJob],
    cfg: &RunnerConfig,
) -> Result<BenchmarkSuiteReport, Box<dyn std::error::Error + Send + Sync>> {
    let workload = BenchmarkWorkload::load_default()?;
    let planned_workload = workload.planned_summary()?;
    workload.ensure_summary_matches_contract(&planned_workload)?;
    let tools = prepare_tools(cfg)?;
    let (_workspace, paths, workload_summary) =
        materialize_workload(&workload, &planned_workload, cfg.dry_run)?;

    let mut report = BenchmarkSuiteReport::new(
        cfg.invocation.clone(),
        cfg.dry_run,
        tools.environment.clone(),
        workload_summary.clone(),
    )
    .stamp();

    for job in jobs {
        prepare_job_workspace(job, &paths)?;

        let pack = build_pack_command(
            job,
            &paths,
            &tools.resolved_sfa_bin,
            &tools.tar_bin,
            &tools.zstd_bin,
        );
        let unpack = build_unpack_command(
            job,
            &paths,
            &tools.resolved_sfa_bin,
            &tools.tar_bin,
            &tools.zstd_bin,
        );

        report.records.push(run_record(
            job,
            "pack",
            &pack,
            &paths,
            &workload_summary,
            cfg.dry_run,
            &tools.resource_sampler,
        )?);
        report.records.push(run_record(
            job,
            "unpack",
            &unpack,
            &paths,
            &workload_summary,
            cfg.dry_run,
            &tools.resource_sampler,
        )?);
        cleanup_job_workspace(job, &paths)?;
    }

    Ok(report)
}

fn materialize_workload(
    workload: &BenchmarkWorkload,
    planned_workload: &WorkloadSummary,
    dry_run: bool,
) -> Result<(Option<TempDir>, BenchmarkPaths, WorkloadSummary), Box<dyn std::error::Error + Send + Sync>>
{
    if dry_run {
        let paths = dry_run_paths(workload);
        return Ok((None, paths, planned_workload.clone()));
    }

    let workspace = tempfile::tempdir().map_err(|e| format!("failed to create temp workspace: {e}"))?;
    let paths = BenchmarkPaths {
        workload_name: workload.name().to_string(),
        input_dir: workspace.path().join("input"),
        workspace_dir: workspace.path().to_path_buf(),
    };
    let summary = workload.materialize(&paths.input_dir)?;
    Ok((Some(workspace), paths, summary))
}

fn prepare_job_workspace(
    job: &BenchmarkJob,
    paths: &BenchmarkPaths,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    std::fs::create_dir_all(paths.workspace_dir.join("artifacts")).map_err(|e| {
        format!(
            "failed to create benchmark artifact dir {}: {e}",
            paths.workspace_dir.join("artifacts").display()
        )
    })?;
    let archive = archive_path(job, paths);
    let unpack = unpack_dir(job, paths);
    remove_path_if_exists(&archive)?;
    remove_path_if_exists(&unpack)?;
    std::fs::create_dir_all(&unpack)
        .map_err(|e| format!("failed to create unpack dir {}: {e}", unpack.display()))?;
    Ok(())
}

fn cleanup_job_workspace(
    job: &BenchmarkJob,
    paths: &BenchmarkPaths,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    remove_path_if_exists(&archive_path(job, paths))?;
    remove_path_if_exists(&unpack_dir(job, paths))?;
    Ok(())
}

fn run_record(
    job: &BenchmarkJob,
    phase: &'static str,
    cmd: &CommandSpec,
    paths: &BenchmarkPaths,
    workload: &WorkloadSummary,
    dry_run: bool,
    resource_sampler: &ResourceSampler,
) -> Result<BenchmarkRecord, Box<dyn std::error::Error + Send + Sync>> {
    if dry_run {
        return Ok(BenchmarkRecord::from_dry_run(
            job,
            workload,
            phase,
            cmd.to_shell_line(),
        ));
    }

    let started = Instant::now();
    let execution = command_output(cmd, resource_sampler)
        .map_err(|e| format!("command failed to spawn: {cmd:?}: {e}"))?;
    let elapsed_ms = started.elapsed().as_millis() as u64;

    let stderr = String::from_utf8_lossy(&execution.output.stderr).to_string();
    let mut stdout = String::from_utf8_lossy(&execution.output.stdout).to_string();
    let status = execution.output.status.code().unwrap_or(1);
    if status != 0 {
        return Err(format!(
            "benchmark command failed for {} {} {}: exit {status}\ncommand: {}\nstdout:\n{}\nstderr:\n{}",
            paths.workload_name,
            match job.baseline {
                crate::harness::Baseline::Sfa => "sfa",
                crate::harness::Baseline::Tar => "tar",
            },
            phase,
            cmd.to_shell_line(),
            stdout,
            stderr,
        )
        .into());
    }

    let output_size_bytes = match phase {
        "pack" => std::fs::metadata(archive_path(job, paths)).ok().map(|metadata| metadata.len()),
        "unpack" => Some(total_file_bytes(&unpack_dir(job, paths))?),
        other => return Err(format!("unsupported benchmark phase: {other}").into()),
    };

    let sfa_stats = match job.baseline {
        crate::harness::Baseline::Sfa => Some(parse_sfa_stats(phase, &stdout)?),
        crate::harness::Baseline::Tar => None,
    };
    if matches!(job.baseline, crate::harness::Baseline::Sfa) {
        stdout.clear();
    }

    Ok(BenchmarkRecord::from_execution(
        job,
        workload,
        phase,
        cmd.to_shell_line(),
        elapsed_ms,
        output_size_bytes,
        status,
        stdout,
        stderr,
        sfa_stats,
        execution
            .resource_observation
            .or_else(|| Some(resource_sampler.unavailable_observation())),
    ))
}

fn total_file_bytes(path: &Path) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let mut total = 0u64;
    for entry in WalkDir::new(path).follow_links(false) {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            total = total.saturating_add(metadata.len());
        }
    }
    Ok(total)
}

fn command_output(
    cmd: &CommandSpec,
    resource_sampler: &ResourceSampler,
) -> Result<CommandExecution, Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(unix)]
    {
        let _ = resource_sampler;
        command_output_unix(cmd)
    }
    #[cfg(not(unix))]
    {
        let output = Command::new(&cmd.program)
            .args(&cmd.args)
            .env("LC_ALL", "C")
            .output()
            .map_err(|e| format!("unable to execute {}: {e}", cmd.to_shell_line()))?;
        Ok(CommandExecution {
            output,
            resource_observation: Some(resource_sampler.unavailable_observation()),
        })
    }
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
    cfg: &RunnerConfig,
) -> Result<PreparedTools, Box<dyn std::error::Error + Send + Sync>> {
    let resolved_sfa_bin = resolve_sfa_bin(&cfg.sfa_bin);
    let tar = resolve_optional_command("tar")?;
    let zstd = resolve_optional_command("zstd")?;
    let resource_sampler = ResourceSampler::detect();

    if !cfg.dry_run {
        let sfa_bin = resolved_sfa_bin
            .clone()
            .ok_or_else(|| missing_sfa_bin_message(&cfg.sfa_bin))?;
        if !sfa_bin.is_file() {
            return Err(format!("SFA binary is not executable: {}", sfa_bin.display()).into());
        }
        let tar_bin = tar
            .clone()
            .ok_or_else(|| "required tool `tar` was not found on PATH".to_string())?;
        let zstd_bin = zstd
            .clone()
            .ok_or_else(|| "required tool `zstd` was not found on PATH".to_string())?;
        ensure_tar_zstd_workflow_supported(&tar_bin, &zstd_bin)?;

        Ok(PreparedTools {
            resolved_sfa_bin: sfa_bin,
            tar_bin,
            zstd_bin,
            environment: build_environment(resolved_sfa_bin, tar, zstd, resource_sampler.metadata()),
            resource_sampler,
        })
    } else {
        Ok(PreparedTools {
            resolved_sfa_bin: resolved_sfa_bin.clone().unwrap_or_else(|| cfg.sfa_bin.clone()),
            tar_bin: tar.clone().unwrap_or_else(|| PathBuf::from("tar")),
            zstd_bin: zstd.clone().unwrap_or_else(|| PathBuf::from("zstd")),
            environment: build_environment(resolved_sfa_bin, tar, zstd, resource_sampler.metadata()),
            resource_sampler,
        })
    }
}

fn ensure_tar_zstd_workflow_supported(
    tar_bin: &Path,
    zstd_bin: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let probe_dir = tempfile::tempdir().map_err(|e| format!("failed to create probe dir: {e}"))?;
    let input_dir = probe_dir.path().join("input");
    let archive = probe_dir.path().join("probe.tar.zst");
    let output_dir = probe_dir.path().join("out");
    std::fs::create_dir_all(&input_dir)?;
    std::fs::write(input_dir.join("probe.txt"), "benchmark probe\n")?;
    std::fs::create_dir_all(&output_dir)?;

    let pack = Command::new("sh")
        .arg("-c")
        .arg("set -eu; \"$1\" -cf - -C \"$2\" . | \"$3\" -q --fast=3 -f -o \"$4\"")
        .arg("sh")
        .arg(tar_bin)
        .arg(&input_dir)
        .arg(zstd_bin)
        .arg(&archive)
        .env("LC_ALL", "C")
        .output()?;
    if !pack.status.success() {
        return Err(format!(
            "canonical tar + zstd --fast=3 benchmark workflow is unavailable\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&pack.stdout),
            String::from_utf8_lossy(&pack.stderr)
        )
        .into());
    }

    let unpack = Command::new("sh")
        .arg("-c")
        .arg("set -eu; \"$1\" -q -d -c \"$2\" | \"$3\" -xf - -C \"$4\"")
        .arg("sh")
        .arg(zstd_bin)
        .arg(&archive)
        .arg(tar_bin)
        .arg(&output_dir)
        .env("LC_ALL", "C")
        .output()?;
    if !unpack.status.success() {
        return Err(format!(
            "canonical tar + zstd --fast=3 unpack workflow is unavailable\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&unpack.stdout),
            String::from_utf8_lossy(&unpack.stderr)
        )
        .into());
    }
    Ok(())
}

fn build_environment(
    sfa_bin: Option<PathBuf>,
    tar_bin: Option<PathBuf>,
    zstd_bin: Option<PathBuf>,
    resource_sampler: ResourceSamplerMetadata,
) -> BenchmarkEnvironment {
    BenchmarkEnvironment {
        host_os: std::env::consts::OS.to_string(),
        host_arch: std::env::consts::ARCH.to_string(),
        tar: tool_metadata("tar", tar_bin.as_deref()),
        sfa: tool_metadata("sfa", sfa_bin.as_deref()),
        zstd: tool_metadata("zstd", zstd_bin.as_deref()),
        resource_sampler,
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

    for candidate in [
        PathBuf::from("/opt/homebrew/bin").join(name),
        PathBuf::from("/usr/local/bin").join(name),
    ] {
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

fn parse_sfa_stats(
    phase: &str,
    stdout: &str,
) -> Result<SfaCommandStats, Box<dyn std::error::Error + Send + Sync>> {
    match phase {
        "pack" => Ok(SfaCommandStats::Pack(serde_json::from_str::<PackStats>(
            stdout.trim(),
        )?)),
        "unpack" => Ok(SfaCommandStats::Unpack(
            serde_json::from_str::<UnpackStats>(stdout.trim())?,
        )),
        other => Err(format!("unsupported SFA benchmark phase: {other}").into()),
    }
}

#[cfg(unix)]
fn command_output_unix(
    cmd: &CommandSpec,
) -> Result<CommandExecution, Box<dyn std::error::Error + Send + Sync>> {
    use std::mem::MaybeUninit;
    use std::os::unix::process::ExitStatusExt;

    let mut child = Command::new(&cmd.program)
        .args(&cmd.args)
        .env("LC_ALL", "C")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("unable to execute {}: {e}", cmd.to_shell_line()))?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| format!("failed to capture stdout for {}", cmd.to_shell_line()))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| format!("failed to capture stderr for {}", cmd.to_shell_line()))?;

    let pid = child.id() as libc::pid_t;
    let mut status = 0;
    let mut usage = MaybeUninit::<libc::rusage>::uninit();
    let waited = unsafe { libc::wait4(pid, &mut status, 0, usage.as_mut_ptr()) };
    if waited < 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    let mut stdout_buf = Vec::new();
    stdout.read_to_end(&mut stdout_buf)?;
    let mut stderr_buf = Vec::new();
    stderr.read_to_end(&mut stderr_buf)?;

    let usage = unsafe { usage.assume_init() };
    Ok(CommandExecution {
        output: Output {
            status: std::process::ExitStatus::from_raw(status),
            stdout: stdout_buf,
            stderr: stderr_buf,
        },
        resource_observation: Some(resource_observation_from_rusage(&usage)),
    })
}

#[cfg(unix)]
fn resource_observation_from_rusage(usage: &libc::rusage) -> ResourceObservation {
    ResourceObservation {
        sampler: "wait4/getrusage".to_string(),
        user_cpu_ms: Some(timeval_to_ms(usage.ru_utime)),
        system_cpu_ms: Some(timeval_to_ms(usage.ru_stime)),
        max_rss_kib: Some(normalize_max_rss_kib(usage.ru_maxrss)),
        note: None,
    }
}

#[cfg(unix)]
fn timeval_to_ms(timeval: libc::timeval) -> u64 {
    let secs = timeval.tv_sec.max(0) as u64;
    let micros = timeval.tv_usec.max(0) as u64;
    secs.saturating_mul(1_000).saturating_add(micros / 1_000)
}

#[cfg(all(unix, any(target_os = "macos", target_os = "ios")))]
fn normalize_max_rss_kib(value: libc::c_long) -> u64 {
    value.max(0) as u64 / 1024
}

#[cfg(all(unix, not(any(target_os = "macos", target_os = "ios"))))]
fn normalize_max_rss_kib(value: libc::c_long) -> u64 {
    value.max(0) as u64
}
