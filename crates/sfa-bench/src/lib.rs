pub mod harness;
pub mod report;
pub mod runner;
pub mod workload;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::harness::{Baseline, default_jobs};
    use crate::report::{BenchmarkSuiteReport, SfaCommandStats};

    #[test]
    fn default_jobs_include_sfa_and_tar() {
        let jobs = default_jobs();
        assert_eq!(jobs.len(), 2);
        assert!(jobs.iter().any(|job| job.baseline == Baseline::Sfa));
        assert!(jobs.iter().any(|job| job.baseline == Baseline::Tar));
    }

    #[test]
    fn committed_baseline_report_matches_current_default_jobs() {
        let report_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../benches/results/baseline-v0.1.0.json");
        let report: BenchmarkSuiteReport = serde_json::from_slice(
            &std::fs::read(&report_path).expect("committed benchmark baseline should exist"),
        )
        .expect("committed benchmark baseline should parse");
        let workload_name = report.workload.name.clone();

        let expected: std::collections::BTreeSet<_> = default_jobs()
            .into_iter()
            .flat_map(|job| {
                let workload_name = workload_name.clone();
                ["pack", "unpack"]
                    .into_iter()
                    .map(move |phase| (workload_name.clone(), job.baseline, phase.to_string()))
            })
            .collect();

        let actual: std::collections::BTreeSet<_> = report
            .records
            .iter()
            .map(|record| (record.workload.clone(), record.baseline, record.phase.clone()))
            .collect();

        assert_eq!(actual, expected);
        assert!(report.environment.tar.path.is_some());
        assert_eq!(report.workload.name, "node-modules-100k");
        assert_eq!(
            report.workload.recipe_path,
            "benches/workloads/node-modules-100k/recipe.json"
        );
        assert!(report.workload.regular_file_count >= 100_000);
        assert!(report.environment.resource_sampler.supported);
        assert!(
            report
                .records
                .iter()
                .all(|record| record.resource_observation.is_some())
        );
        assert!(report.records.iter().all(|record| record.files_per_sec.is_some()));
        assert!(report.records.iter().all(|record| record.mib_per_sec.is_some()));
        assert!(
            report
                .records
                .iter()
                .all(|record| record.output_size_bytes.is_some())
        );
        assert!(report.records.iter().all(|record| match record.baseline {
            Baseline::Sfa => matches!(
                record.sfa_stats,
                Some(SfaCommandStats::Pack(_)) | Some(SfaCommandStats::Unpack(_))
            ),
            Baseline::Tar => record.sfa_stats.is_none(),
        }));
        assert!(report.records.iter().all(|record| match &record.sfa_stats {
            Some(SfaCommandStats::Unpack(stats)) => {
                stats.threads > 0
                    && stats.wall_breakdown.setup_ms.value.is_some()
                    && stats.wall_breakdown.pipeline_ms.value.is_some()
                    && stats.wall_breakdown.finalize_ms.value.is_some()
                    && stats.wall_breakdown.setup_ms.value.unwrap_or_default()
                        + stats.wall_breakdown.pipeline_ms.value.unwrap_or_default()
                        + stats.wall_breakdown.finalize_ms.value.unwrap_or_default()
                        == stats.duration_ms
                    && stats.phase_breakdown.frame_read_ms.value.is_some()
                    && stats.phase_breakdown.decode_ms.value.is_some()
                    && stats.phase_breakdown.scatter_ms.value.is_some()
            }
            _ => true,
        }));
    }
}
