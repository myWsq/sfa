use std::time::{SystemTime, UNIX_EPOCH};

use crate::harness::{Baseline, BenchmarkJob, Codec};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct BenchmarkSuiteReport {
    pub generated_at_unix_s: u64,
    pub invocation: String,
    pub dry_run: bool,
    pub environment: BenchmarkEnvironment,
    pub datasets: Vec<DatasetSummary>,
    pub records: Vec<BenchmarkRecord>,
}

impl BenchmarkSuiteReport {
    pub fn new(
        invocation: String,
        dry_run: bool,
        environment: BenchmarkEnvironment,
        datasets: Vec<DatasetSummary>,
    ) -> Self {
        Self {
            generated_at_unix_s: 0,
            invocation,
            dry_run,
            environment,
            datasets,
            records: Vec::new(),
        }
    }

    pub fn stamp(mut self) -> Self {
        self.generated_at_unix_s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct BenchmarkEnvironment {
    pub host_os: String,
    pub host_arch: String,
    pub tar: ToolMetadata,
    pub sfa: ToolMetadata,
    pub codecs: Vec<CodecToolMetadata>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ToolMetadata {
    pub name: String,
    pub path: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodecToolMetadata {
    pub codec: Codec,
    pub tool: ToolMetadata,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatasetSummary {
    pub dataset: String,
    pub input_dir: String,
    pub file_count: u64,
    pub directory_count: u64,
    pub symlink_count: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkRecord {
    pub dataset: String,
    pub baseline: Baseline,
    pub codec: Codec,
    pub phase: String,
    pub command: String,
    pub elapsed_ms: Option<u64>,
    pub exit_status: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub notes: Option<String>,
}

impl BenchmarkRecord {
    pub fn from_dry_run(job: &BenchmarkJob, phase: &str, command: String) -> Self {
        Self {
            dataset: job.case.name.clone(),
            baseline: job.baseline,
            codec: job.codec,
            phase: phase.to_string(),
            command,
            elapsed_ms: None,
            exit_status: None,
            stdout: None,
            stderr: None,
            notes: Some("dry-run only".to_string()),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_execution(
        job: &BenchmarkJob,
        phase: &str,
        command: String,
        elapsed_ms: u64,
        exit_status: i32,
        stdout: String,
        stderr: String,
    ) -> Self {
        Self {
            dataset: job.case.name.clone(),
            baseline: job.baseline,
            codec: job.codec,
            phase: phase.to_string(),
            command,
            elapsed_ms: Some(elapsed_ms),
            exit_status: Some(exit_status),
            stdout: if stdout.is_empty() {
                None
            } else {
                Some(stdout)
            },
            stderr: if stderr.is_empty() {
                None
            } else {
                Some(stderr)
            },
            notes: None,
        }
    }
}
