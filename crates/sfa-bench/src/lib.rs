pub mod harness;
pub mod report;
pub mod runner;

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use crate::harness::{Codec, default_cases, default_matrix};
    use crate::report::{BenchmarkSuiteReport, SfaCommandStats};

    #[test]
    fn matrix_contains_tar_and_sfa_for_each_dataset() {
        let cases = default_cases();
        let matrix = default_matrix();
        assert_eq!(matrix.len(), cases.len() * 2 * 2);
        assert!(matrix.iter().any(|job| job.codec == Codec::Lz4));
        assert!(matrix.iter().any(|job| job.codec == Codec::Zstd));
    }

    #[test]
    fn committed_baseline_report_matches_current_matrix() {
        let report_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../benches/results/baseline-v0.1.0.json");
        let report: BenchmarkSuiteReport = serde_json::from_slice(
            &std::fs::read(&report_path).expect("committed benchmark baseline should exist"),
        )
        .expect("committed benchmark baseline should parse");

        let expected: BTreeSet<_> = default_matrix()
            .into_iter()
            .flat_map(|job| {
                ["pack", "unpack"].into_iter().map(move |phase| {
                    (
                        job.case.name.clone(),
                        job.baseline,
                        job.codec,
                        phase.to_string(),
                    )
                })
            })
            .collect();

        let actual: BTreeSet<_> = report
            .records
            .iter()
            .map(|record| {
                (
                    record.dataset.clone(),
                    record.baseline,
                    record.codec,
                    record.phase.clone(),
                )
            })
            .collect();

        assert_eq!(actual, expected);
        assert!(report.environment.tar.path.is_some());
        assert_eq!(report.datasets.len(), default_cases().len());
        assert!(report.environment.resource_sampler.supported);
        assert!(
            report
                .records
                .iter()
                .all(|record| record.resource_observation.is_some())
        );
        assert!(report.records.iter().all(|record| match record.baseline {
            crate::harness::Baseline::Sfa => matches!(
                record.sfa_stats,
                Some(SfaCommandStats::Pack(_)) | Some(SfaCommandStats::Unpack(_))
            ),
            crate::harness::Baseline::Tar => record.sfa_stats.is_none(),
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
