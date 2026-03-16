use crate::service::RunStats;

pub fn render_pack_summary(stats: &RunStats) -> String {
    format_summary("Pack Summary", stats)
}

pub fn render_unpack_summary(stats: &RunStats) -> String {
    format_summary("Unpack Summary", stats)
}

fn format_summary(title: &str, stats: &RunStats) -> String {
    let secs = stats.duration.as_secs_f64().max(0.000_001);
    let files_per_sec = stats.entry_count as f64 / secs;
    let mib_per_sec = (stats.raw_bytes as f64 / 1024.0 / 1024.0) / secs;
    format!(
        "{title}\n\
         codec: {:?}\n\
         threads: {}\n\
         bundle_target_bytes: {}\n\
         small_file_threshold: {}\n\
         entry_count: {}\n\
         bundle_count: {}\n\
         raw_bytes: {}\n\
         encoded_bytes: {}\n\
         duration_seconds: {:.4}\n\
         files_per_second: {:.2}\n\
         mib_per_second: {:.2}",
        stats.codec,
        stats.threads,
        stats.bundle_target_bytes,
        stats.small_file_threshold,
        stats.entry_count,
        stats.bundle_count,
        stats.raw_bytes,
        stats.encoded_bytes,
        secs,
        files_per_sec,
        mib_per_sec,
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::render_pack_summary;
    use crate::cli::DataCodec;
    use crate::service::RunStats;

    #[test]
    fn summary_contains_throughput_keys() {
        let stats = RunStats {
            codec: DataCodec::Lz4,
            threads: 8,
            bundle_target_bytes: 4 * 1024 * 1024,
            small_file_threshold: 256 * 1024,
            entry_count: 100,
            bundle_count: 5,
            raw_bytes: 1_048_576,
            encoded_bytes: 700_000,
            duration: Duration::from_secs(2),
        };
        let rendered = render_pack_summary(&stats);
        assert!(rendered.contains("files_per_second"));
        assert!(rendered.contains("mib_per_second"));
        assert!(rendered.contains("bundle_count"));
    }
}
