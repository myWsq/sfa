use std::path::{Path, PathBuf};

use crate::workload::BenchmarkWorkload;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Baseline {
    Sfa,
    Tar,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkJob {
    pub baseline: Baseline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkPaths {
    pub workload_name: String,
    pub input_dir: PathBuf,
    pub workspace_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

impl CommandSpec {
    pub fn to_shell_line(&self) -> String {
        let mut line = self.program.clone();
        for arg in &self.args {
            line.push(' ');
            line.push_str(&shell_escape(arg));
        }
        line
    }
}

pub fn default_jobs() -> Vec<BenchmarkJob> {
    vec![
        BenchmarkJob {
            baseline: Baseline::Sfa,
        },
        BenchmarkJob {
            baseline: Baseline::Tar,
        },
    ]
}

pub fn dry_run_paths(workload: &BenchmarkWorkload) -> BenchmarkPaths {
    let workspace_dir = std::env::temp_dir().join(format!("sfa-bench-dry-run-{}", workload.name()));
    BenchmarkPaths {
        workload_name: workload.name().to_string(),
        input_dir: workspace_dir.join("input"),
        workspace_dir,
    }
}

pub fn archive_name(workload_name: &str, baseline: Baseline) -> String {
    match baseline {
        Baseline::Sfa => format!("{workload_name}.sfa"),
        Baseline::Tar => format!("{workload_name}.tar.zst"),
    }
}

pub fn archive_path(job: &BenchmarkJob, paths: &BenchmarkPaths) -> PathBuf {
    paths
        .workspace_dir
        .join("artifacts")
        .join(archive_name(&paths.workload_name, job.baseline))
}

pub fn unpack_dir(job: &BenchmarkJob, paths: &BenchmarkPaths) -> PathBuf {
    paths.workspace_dir.join(format!(
        "unpack-{}-{}",
        paths.workload_name,
        match job.baseline {
            Baseline::Sfa => "sfa",
            Baseline::Tar => "tar",
        },
    ))
}

pub fn build_pack_command(
    job: &BenchmarkJob,
    paths: &BenchmarkPaths,
    sfa_bin: &Path,
    tar_bin: &Path,
    zstd_bin: &Path,
) -> CommandSpec {
    let archive = archive_path(job, paths);
    match job.baseline {
        Baseline::Sfa => CommandSpec {
            program: sfa_bin.display().to_string(),
            args: vec![
                "pack".to_string(),
                paths.input_dir.display().to_string(),
                archive.display().to_string(),
                "--stats-format".to_string(),
                "json".to_string(),
            ],
        },
        Baseline::Tar => CommandSpec {
            program: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "set -eu; \"$1\" -cf - -C \"$2\" . | \"$3\" -q --fast=3 -f -o \"$4\"".to_string(),
                "sh".to_string(),
                tar_bin.display().to_string(),
                paths.input_dir.display().to_string(),
                zstd_bin.display().to_string(),
                archive.display().to_string(),
            ],
        },
    }
}

pub fn build_unpack_command(
    job: &BenchmarkJob,
    paths: &BenchmarkPaths,
    sfa_bin: &Path,
    tar_bin: &Path,
    zstd_bin: &Path,
) -> CommandSpec {
    let archive = archive_path(job, paths);
    let unpack_to = unpack_dir(job, paths);
    match job.baseline {
        Baseline::Sfa => CommandSpec {
            program: sfa_bin.display().to_string(),
            args: vec![
                "unpack".to_string(),
                archive.display().to_string(),
                "-C".to_string(),
                unpack_to.display().to_string(),
                "--stats-format".to_string(),
                "json".to_string(),
            ],
        },
        Baseline::Tar => CommandSpec {
            program: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "set -eu; \"$1\" -q -d -c \"$2\" | \"$3\" -xf - -C \"$4\"".to_string(),
                "sh".to_string(),
                zstd_bin.display().to_string(),
                archive.display().to_string(),
                tar_bin.display().to_string(),
                unpack_to.display().to_string(),
            ],
        },
    }
}

fn shell_escape(arg: &str) -> String {
    if arg.is_empty() {
        return "''".to_string();
    }
    if arg.bytes().all(
        |b| matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'/' | b':'),
    ) {
        return arg.to_string();
    }
    format!("'{}'", arg.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::workload::BenchmarkWorkload;

    use super::{
        Baseline, BenchmarkJob, build_pack_command, build_unpack_command, default_jobs,
        dry_run_paths,
    };

    #[test]
    fn default_jobs_only_include_sfa_and_tar() {
        let jobs = default_jobs();
        assert_eq!(jobs.len(), 2);
        assert!(jobs.iter().any(|job| job.baseline == Baseline::Sfa));
        assert!(jobs.iter().any(|job| job.baseline == Baseline::Tar));
    }

    #[test]
    fn sfa_commands_request_json_stats() {
        let workload = BenchmarkWorkload::load_default().expect("workload");
        let paths = dry_run_paths(&workload);
        let job = BenchmarkJob {
            baseline: Baseline::Sfa,
        };
        let pack = build_pack_command(
            &job,
            &paths,
            Path::new("sfa"),
            Path::new("tar"),
            Path::new("zstd"),
        );
        let unpack = build_unpack_command(
            &job,
            &paths,
            Path::new("sfa"),
            Path::new("tar"),
            Path::new("zstd"),
        );

        assert!(
            pack.args
                .windows(2)
                .any(|pair| pair == ["--stats-format", "json"])
        );
        assert!(
            unpack
                .args
                .windows(2)
                .any(|pair| pair == ["--stats-format", "json"])
        );
    }

    #[test]
    fn tar_commands_use_canonical_zstd_fast_3_pipeline() {
        let workload = BenchmarkWorkload::load_default().expect("workload");
        let paths = dry_run_paths(&workload);
        let job = BenchmarkJob {
            baseline: Baseline::Tar,
        };
        let pack = build_pack_command(
            &job,
            &paths,
            Path::new("sfa"),
            Path::new("tar"),
            Path::new("zstd"),
        );
        let unpack = build_unpack_command(
            &job,
            &paths,
            Path::new("sfa"),
            Path::new("tar"),
            Path::new("zstd"),
        );

        assert_eq!(pack.program, "sh");
        assert!(pack.args.iter().any(|arg| arg.contains("--fast=3")));
        assert!(unpack.args.iter().any(|arg| arg.contains("-d -c")));
        assert!(pack.to_shell_line().contains("zstd"));
        assert!(!pack.to_shell_line().contains("lz4"));
        assert!(!unpack.to_shell_line().contains("lz4"));
    }
}
