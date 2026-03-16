use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
            line.push_str(arg);
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

pub fn build_pack_command(job: &BenchmarkJob, sfa_bin: &Path) -> CommandSpec {
    let archive = job
        .case
        .output_dir
        .join(archive_name(&job.case.name, job.baseline, job.codec));
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
        Baseline::Tar => {
            let codec_flag = match job.codec {
                Codec::Lz4 => "--lz4",
                Codec::Zstd => "--zstd",
            };
            CommandSpec {
                program: "tar".to_string(),
                args: vec![
                    "-cf".to_string(),
                    archive.display().to_string(),
                    codec_flag.to_string(),
                    "-C".to_string(),
                    job.case.input_dir.display().to_string(),
                    ".".to_string(),
                ],
            }
        }
    }
}

pub fn build_unpack_command(job: &BenchmarkJob, sfa_bin: &Path) -> CommandSpec {
    let archive = job
        .case
        .output_dir
        .join(archive_name(&job.case.name, job.baseline, job.codec));
    let unpack_to = job.case.output_dir.join(format!(
        "unpack-{}-{}-{}",
        job.case.name,
        match job.baseline {
            Baseline::Sfa => "sfa",
            Baseline::Tar => "tar",
        },
        job.codec.as_str()
    ));
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
        Baseline::Tar => CommandSpec {
            program: "tar".to_string(),
            args: vec![
                "-xf".to_string(),
                archive.display().to_string(),
                "-C".to_string(),
                unpack_to.display().to_string(),
            ],
        },
    }
}
