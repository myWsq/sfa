use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use crate::harness::{BenchmarkJob, CommandSpec, build_pack_command, build_unpack_command};
use crate::report::{BenchmarkRecord, BenchmarkSuiteReport};

#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub sfa_bin: PathBuf,
    pub dry_run: bool,
}

impl RunnerConfig {
    pub fn new(sfa_bin: PathBuf, dry_run: bool) -> Self {
        Self { sfa_bin, dry_run }
    }
}

pub fn run_jobs(
    jobs: &[BenchmarkJob],
    cfg: &RunnerConfig,
) -> Result<BenchmarkSuiteReport, Box<dyn std::error::Error + Send + Sync>> {
    let mut report = BenchmarkSuiteReport::default();
    for job in jobs {
        ensure_case_dirs(job)?;

        let pack = build_pack_command(job, &cfg.sfa_bin);
        let unpack = build_unpack_command(job, &cfg.sfa_bin);
        report
            .records
            .push(run_record(job, "pack", &pack, cfg.dry_run)?);
        report
            .records
            .push(run_record(job, "unpack", &unpack, cfg.dry_run)?);
    }
    Ok(report)
}

fn ensure_case_dirs(job: &BenchmarkJob) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    std::fs::create_dir_all(&job.case.output_dir).map_err(|e| {
        format!(
            "failed to create output dir {}: {e}",
            job.case.output_dir.display()
        )
    })?;
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
