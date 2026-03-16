use std::path::PathBuf;

use sfa_bench::harness::default_matrix;
use sfa_bench::runner::{RunnerConfig, run_jobs, write_report};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().collect();
    let mut sfa_bin = PathBuf::from("sfa");
    let mut out = PathBuf::from("benches/results/latest.json");
    let mut dry_run = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--sfa-bin" => {
                i += 1;
                let value = args.get(i).ok_or("--sfa-bin needs a value")?.clone();
                sfa_bin = PathBuf::from(value);
            }
            "--output" => {
                i += 1;
                let value = args.get(i).ok_or("--output needs a value")?.clone();
                out = PathBuf::from(value);
            }
            "--dry-run" => {
                dry_run = true;
            }
            "--help" | "-h" => {
                println!(
                    "Usage: tar_vs_sfa [--sfa-bin <path>] [--output <report.json>] [--dry-run]"
                );
                return Ok(());
            }
            unknown => {
                return Err(format!("unknown argument: {unknown}").into());
            }
        }
        i += 1;
    }

    let cfg = RunnerConfig::new(sfa_bin, dry_run, args.join(" "));
    let jobs = default_matrix();
    let report = run_jobs(&jobs, &cfg)?;
    write_report(&report, &out)?;

    println!("Wrote benchmark report to {}", out.display());
    Ok(())
}
