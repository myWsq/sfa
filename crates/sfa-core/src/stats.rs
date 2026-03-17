use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ObservationStatus {
    Measured,
    #[default]
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ObservedMetric {
    pub status: ObservationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl ObservedMetric {
    pub fn measured(value: u64) -> Self {
        Self {
            status: ObservationStatus::Measured,
            value: Some(value),
            note: None,
        }
    }

    pub fn unavailable(note: impl Into<String>) -> Self {
        Self {
            status: ObservationStatus::Unavailable,
            value: None,
            note: Some(note.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PackPhaseBreakdown {
    pub scan_ms: ObservedMetric,
    pub plan_ms: ObservedMetric,
    pub encode_ms: ObservedMetric,
    pub write_ms: ObservedMetric,
}

impl PackPhaseBreakdown {
    pub fn unavailable(note: impl Into<String>) -> Self {
        let note = note.into();
        Self {
            scan_ms: ObservedMetric::unavailable(note.clone()),
            plan_ms: ObservedMetric::unavailable(note.clone()),
            encode_ms: ObservedMetric::unavailable(note.clone()),
            write_ms: ObservedMetric::unavailable(note),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct UnpackPhaseBreakdown {
    pub header_ms: ObservedMetric,
    pub manifest_ms: ObservedMetric,
    pub frame_read_ms: ObservedMetric,
    pub decode_ms: ObservedMetric,
    pub scatter_ms: ObservedMetric,
    pub restore_finalize_ms: ObservedMetric,
}

impl UnpackPhaseBreakdown {
    pub fn unavailable(note: impl Into<String>) -> Self {
        let note = note.into();
        Self {
            header_ms: ObservedMetric::unavailable(note.clone()),
            manifest_ms: ObservedMetric::unavailable(note.clone()),
            frame_read_ms: ObservedMetric::unavailable(note.clone()),
            decode_ms: ObservedMetric::unavailable(note.clone()),
            scatter_ms: ObservedMetric::unavailable(note.clone()),
            restore_finalize_ms: ObservedMetric::unavailable(note),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct UnpackWallBreakdown {
    pub setup_ms: ObservedMetric,
    pub pipeline_ms: ObservedMetric,
    pub finalize_ms: ObservedMetric,
}

impl UnpackWallBreakdown {
    pub fn unavailable(note: impl Into<String>) -> Self {
        let note = note.into();
        Self {
            setup_ms: ObservedMetric::unavailable(note.clone()),
            pipeline_ms: ObservedMetric::unavailable(note.clone()),
            finalize_ms: ObservedMetric::unavailable(note),
        }
    }

    pub fn from_total_duration(
        total_duration: Duration,
        setup_duration: Duration,
        pipeline_duration: Duration,
    ) -> Self {
        let total_ms = duration_millis(total_duration);
        let setup_ms = duration_millis(setup_duration).min(total_ms);
        let remaining_after_setup = total_ms.saturating_sub(setup_ms);
        let pipeline_ms = duration_millis(pipeline_duration).min(remaining_after_setup);
        let finalize_ms = total_ms
            .saturating_sub(setup_ms)
            .saturating_sub(pipeline_ms);

        Self {
            setup_ms: ObservedMetric::measured(setup_ms),
            pipeline_ms: ObservedMetric::measured(pipeline_ms),
            finalize_ms: ObservedMetric::measured(finalize_ms),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackStats {
    pub codec: String,
    pub threads: usize,
    pub bundle_target_bytes: u32,
    pub small_file_threshold: u32,
    pub entry_count: u64,
    pub bundle_count: u64,
    pub raw_bytes: u64,
    pub encoded_bytes: u64,
    pub duration_ms: u64,
    #[serde(default)]
    pub phase_breakdown: PackPhaseBreakdown,
}

impl PackStats {
    pub fn from_duration(duration: Duration, mut stats: Self) -> Self {
        stats.duration_ms = duration_millis(duration);
        stats
    }

    pub fn files_per_second(&self) -> f64 {
        if self.duration_ms == 0 {
            return 0.0;
        }
        (self.entry_count as f64) / (self.duration_ms as f64 / 1000.0)
    }

    pub fn mib_per_second(&self) -> f64 {
        if self.duration_ms == 0 {
            return 0.0;
        }
        ((self.raw_bytes as f64) / (1024.0 * 1024.0)) / (self.duration_ms as f64 / 1000.0)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnpackStats {
    pub codec: String,
    pub threads: usize,
    pub entry_count: u64,
    pub bundle_count: u64,
    pub raw_bytes: u64,
    pub encoded_bytes: u64,
    pub duration_ms: u64,
    #[serde(default)]
    pub wall_breakdown: UnpackWallBreakdown,
    #[serde(default)]
    pub phase_breakdown: UnpackPhaseBreakdown,
}

impl UnpackStats {
    pub fn from_duration(duration: Duration, mut stats: Self) -> Self {
        stats.duration_ms = duration_millis(duration);
        stats
    }

    pub fn files_per_second(&self) -> f64 {
        if self.duration_ms == 0 {
            return 0.0;
        }
        (self.entry_count as f64) / (self.duration_ms as f64 / 1000.0)
    }

    pub fn mib_per_second(&self) -> f64 {
        if self.duration_ms == 0 {
            return 0.0;
        }
        ((self.raw_bytes as f64) / (1024.0 * 1024.0)) / (self.duration_ms as f64 / 1000.0)
    }
}

fn duration_millis(duration: Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{ObservationStatus, PackPhaseBreakdown, UnpackPhaseBreakdown, UnpackWallBreakdown};

    #[test]
    fn unavailable_phase_breakdowns_mark_all_phases_unavailable() {
        let pack = PackPhaseBreakdown::unavailable("dry-run");
        let unpack = UnpackPhaseBreakdown::unavailable("dry-run");
        let unpack_wall = UnpackWallBreakdown::unavailable("dry-run");

        assert_eq!(pack.scan_ms.status, ObservationStatus::Unavailable);
        assert_eq!(pack.write_ms.note.as_deref(), Some("dry-run"));
        assert_eq!(unpack.header_ms.status, ObservationStatus::Unavailable);
        assert_eq!(unpack.frame_read_ms.status, ObservationStatus::Unavailable);
        assert_eq!(unpack.decode_ms.status, ObservationStatus::Unavailable);
        assert_eq!(unpack.scatter_ms.status, ObservationStatus::Unavailable);
        assert_eq!(unpack.restore_finalize_ms.note.as_deref(), Some("dry-run"));
        assert_eq!(unpack_wall.setup_ms.status, ObservationStatus::Unavailable);
        assert_eq!(unpack_wall.finalize_ms.note.as_deref(), Some("dry-run"));
    }

    #[test]
    fn wall_breakdown_reconciles_to_total_duration() {
        let wall = UnpackWallBreakdown::from_total_duration(
            Duration::from_millis(17),
            Duration::from_millis(5),
            Duration::from_millis(8),
        );

        assert_eq!(wall.setup_ms.value, Some(5));
        assert_eq!(wall.pipeline_ms.value, Some(8));
        assert_eq!(wall.finalize_ms.value, Some(4));
    }
}
