use std::path::{Path, PathBuf};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Codec {
    Lz4,
    Zstd,
}

impl Codec {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lz4 => "lz4",
            Self::Zstd => "zstd",
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Baseline {
    Sfa,
    Tar,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DatasetCase {
    pub name: String,
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkJob {
    pub baseline: Baseline,
    pub codec: Codec,
    pub case: DatasetCase,
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

pub fn default_cases() -> Vec<DatasetCase> {
    vec![
        DatasetCase {
            name: "small-text".to_string(),
            input_dir: PathBuf::from("tests/fixtures/datasets/small-text/input"),
            output_dir: PathBuf::from("tests/fixtures/datasets/small-text/output"),
        },
        DatasetCase {
            name: "small-binary".to_string(),
            input_dir: PathBuf::from("tests/fixtures/datasets/small-binary/input"),
            output_dir: PathBuf::from("tests/fixtures/datasets/small-binary/output"),
        },
        DatasetCase {
            name: "large-control".to_string(),
            input_dir: PathBuf::from("tests/fixtures/datasets/large-control/input"),
            output_dir: PathBuf::from("tests/fixtures/datasets/large-control/output"),
        },
    ]
}

pub fn default_matrix() -> Vec<BenchmarkJob> {
    let mut jobs = Vec::new();
    for case in default_cases() {
        for baseline in [Baseline::Sfa, Baseline::Tar] {
            for codec in [Codec::Lz4, Codec::Zstd] {
                jobs.push(BenchmarkJob {
                    baseline,
                    codec,
                    case: case.clone(),
                });
            }
        }
    }
    jobs
}

pub fn archive_name(case_name: &str, baseline: Baseline, codec: Codec) -> String {
    match baseline {
        Baseline::Sfa => format!("{case_name}.{}.sfa", codec.as_str()),
        Baseline::Tar => format!("{case_name}.tar.{}", codec.as_str()),
    }
}

pub fn archive_path(job: &BenchmarkJob) -> PathBuf {
    job.case
        .output_dir
        .join(archive_name(&job.case.name, job.baseline, job.codec))
}

pub fn unpack_dir(job: &BenchmarkJob) -> PathBuf {
    job.case.output_dir.join(format!(
        "unpack-{}-{}-{}",
        job.case.name,
        match job.baseline {
            Baseline::Sfa => "sfa",
            Baseline::Tar => "tar",
        },
        job.codec.as_str()
    ))
}

pub fn build_pack_command(
    job: &BenchmarkJob,
    sfa_bin: &Path,
    tar_bin: &Path,
    codec_bin: &Path,
) -> CommandSpec {
    let archive = archive_path(job);
    match job.baseline {
        Baseline::Sfa => CommandSpec {
            program: sfa_bin.display().to_string(),
            args: vec![
                "pack".to_string(),
                job.case.input_dir.display().to_string(),
                archive.display().to_string(),
                "--codec".to_string(),
                job.codec.as_str().to_string(),
            ],
        },
        Baseline::Tar => build_tar_pack_command(job, tar_bin, codec_bin, &archive),
    }
}

pub fn build_unpack_command(
    job: &BenchmarkJob,
    sfa_bin: &Path,
    tar_bin: &Path,
    codec_bin: &Path,
) -> CommandSpec {
    let archive = archive_path(job);
    let unpack_to = unpack_dir(job);
    match job.baseline {
        Baseline::Sfa => CommandSpec {
            program: sfa_bin.display().to_string(),
            args: vec![
                "unpack".to_string(),
                archive.display().to_string(),
                "-C".to_string(),
                unpack_to.display().to_string(),
            ],
        },
        Baseline::Tar => build_tar_unpack_command(job, tar_bin, codec_bin, &archive, &unpack_to),
    }
}

fn build_tar_pack_command(
    job: &BenchmarkJob,
    tar_bin: &Path,
    codec_bin: &Path,
    archive: &Path,
) -> CommandSpec {
    let codec_script = match job.codec {
        Codec::Lz4 => "\"$3\" -q -f - \"$4\"",
        Codec::Zstd => "\"$3\" -q -f -o \"$4\"",
    };
    CommandSpec {
        program: "sh".to_string(),
        args: vec![
            "-c".to_string(),
            format!("set -eu; \"$1\" -cf - -C \"$2\" . | {codec_script}"),
            "sh".to_string(),
            tar_bin.display().to_string(),
            job.case.input_dir.display().to_string(),
            codec_bin.display().to_string(),
            archive.display().to_string(),
        ],
    }
}

fn build_tar_unpack_command(
    job: &BenchmarkJob,
    tar_bin: &Path,
    codec_bin: &Path,
    archive: &Path,
    unpack_to: &Path,
) -> CommandSpec {
    let codec_script = match job.codec {
        Codec::Lz4 => "\"$1\" -q -d -c \"$2\"",
        Codec::Zstd => "\"$1\" -q -d -c \"$2\"",
    };
    let _ = job;
    CommandSpec {
        program: "sh".to_string(),
        args: vec![
            "-c".to_string(),
            format!("set -eu; {codec_script} | \"$3\" -xf - -C \"$4\""),
            "sh".to_string(),
            codec_bin.display().to_string(),
            archive.display().to_string(),
            tar_bin.display().to_string(),
            unpack_to.display().to_string(),
        ],
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
