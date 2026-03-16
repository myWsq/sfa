use std::time::{SystemTime, UNIX_EPOCH};

use crate::harness::{Baseline, BenchmarkJob, Codec};
use sfa_core::{PackStats, UnpackStats};

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
    #[serde(default)]
    pub resource_sampler: ResourceSamplerMetadata,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ResourceSamplerMetadata {
    pub name: String,
    pub supported: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SfaCommandStats {
    Pack(PackStats),
    Unpack(UnpackStats),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceObservation {
    pub sampler: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_cpu_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_cpu_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_rss_kib: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl ResourceObservation {
    pub fn unavailable(sampler: impl Into<String>, note: impl Into<String>) -> Self {
        Self {
            sampler: sampler.into(),
            user_cpu_ms: None,
            system_cpu_ms: None,
            max_rss_kib: None,
            note: Some(note.into()),
        }
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sfa_stats: Option<SfaCommandStats>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_observation: Option<ResourceObservation>,
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
            sfa_stats: None,
            resource_observation: None,
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
        sfa_stats: Option<SfaCommandStats>,
        resource_observation: Option<ResourceObservation>,
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
            sfa_stats,
            resource_observation,
        }
    }
}
