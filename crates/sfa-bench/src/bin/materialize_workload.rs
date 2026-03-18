use std::path::PathBuf;

use sfa_bench::workload::BenchmarkWorkload;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().collect();
    let mut output = None::<PathBuf>;
    let mut summary_json = None::<PathBuf>;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" => {
                i += 1;
                let value = args.get(i).ok_or("--output needs a value")?;
                output = Some(PathBuf::from(value));
            }
            "--summary-json" => {
                i += 1;
                let value = args.get(i).ok_or("--summary-json needs a value")?;
                summary_json = Some(PathBuf::from(value));
            }
            "--help" | "-h" => {
                println!(
                    "Usage: materialize_workload --output <dir> [--summary-json <path>]"
                );
                return Ok(());
            }
            unknown => {
                return Err(format!("unknown argument: {unknown}").into());
            }
        }
        i += 1;
    }

    let output = output.ok_or("--output is required")?;
    let workload = BenchmarkWorkload::load_default()?;
    let summary = workload.materialize(&output)?;
    let json = serde_json::to_string_pretty(&summary)?;

    if let Some(summary_json) = summary_json {
        if let Some(parent) = summary_json.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(summary_json, &json)?;
    }

    println!("{json}");
    Ok(())
}
