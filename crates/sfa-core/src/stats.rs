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
pub struct UnpackPhaseBreakdown {
    pub header_ms: ObservedMetric,
    pub manifest_ms: ObservedMetric,
    pub decode_and_scatter_ms: ObservedMetric,
    pub restore_finalize_ms: ObservedMetric,
}

impl UnpackPhaseBreakdown {
    pub fn unavailable(note: impl Into<String>) -> Self {
        let note = note.into();
        Self {
            header_ms: ObservedMetric::unavailable(note.clone()),
            manifest_ms: ObservedMetric::unavailable(note.clone()),
            decode_and_scatter_ms: ObservedMetric::unavailable(note.clone()),
            restore_finalize_ms: ObservedMetric::unavailable(note),
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
    use super::{ObservationStatus, PackPhaseBreakdown, UnpackPhaseBreakdown};

    #[test]
    fn unavailable_phase_breakdowns_mark_all_phases_unavailable() {
        let pack = PackPhaseBreakdown::unavailable("dry-run");
        let unpack = UnpackPhaseBreakdown::unavailable("dry-run");

        assert_eq!(pack.scan_ms.status, ObservationStatus::Unavailable);
        assert_eq!(pack.write_ms.note.as_deref(), Some("dry-run"));
        assert_eq!(unpack.header_ms.status, ObservationStatus::Unavailable);
        assert_eq!(unpack.restore_finalize_ms.note.as_deref(), Some("dry-run"));
    }
}
