use sfa_core::{PackStats, UnpackStats};

pub fn render_pack_summary(stats: &PackStats) -> String {
    format_summary(
        "Pack Summary",
        &stats.codec,
        stats.threads,
        stats.bundle_target_bytes,
        stats.small_file_threshold,
        stats.entry_count,
        stats.bundle_count,
        stats.raw_bytes,
        stats.encoded_bytes,
        stats.duration_ms,
        stats.files_per_second(),
        stats.mib_per_second(),
    )
}

pub fn render_unpack_summary(stats: &UnpackStats) -> String {
    format_summary(
        "Unpack Summary",
        &stats.codec,
        stats.threads,
        4 * 1024 * 1024,
        256 * 1024,
        stats.entry_count,
        stats.bundle_count,
        stats.raw_bytes,
        stats.encoded_bytes,
        stats.duration_ms,
        stats.files_per_second(),
        stats.mib_per_second(),
    )
}

fn format_summary(
    title: &str,
    codec: &str,
    threads: usize,
    bundle_target_bytes: u32,
    small_file_threshold: u32,
    entry_count: u64,
    bundle_count: u64,
    raw_bytes: u64,
    encoded_bytes: u64,
    duration_ms: u64,
    files_per_second: f64,
    mib_per_second: f64,
) -> String {
    let secs = (duration_ms as f64 / 1000.0).max(0.000_001);
    format!(
        "{title}\n\
         codec: {codec}\n\
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
        threads,
        bundle_target_bytes,
        small_file_threshold,
        entry_count,
        bundle_count,
        raw_bytes,
        encoded_bytes,
        secs,
        files_per_second,
        mib_per_second,
    )
}

#[cfg(test)]
mod tests {
    use super::render_pack_summary;
    use sfa_core::{PackPhaseBreakdown, PackStats};

    #[test]
    fn summary_contains_throughput_keys() {
        let stats = PackStats {
            codec: "lz4".to_string(),
            threads: 8,
            bundle_target_bytes: 4 * 1024 * 1024,
            small_file_threshold: 256 * 1024,
            entry_count: 100,
            bundle_count: 5,
            raw_bytes: 1_048_576,
            encoded_bytes: 700_000,
            duration_ms: 2_000,
            phase_breakdown: PackPhaseBreakdown::default(),
        };
        let rendered = render_pack_summary(&stats);
        assert!(rendered.contains("files_per_second"));
        assert!(rendered.contains("mib_per_second"));
        assert!(rendered.contains("bundle_count"));
    }
}
