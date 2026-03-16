use std::time::Duration;

use serde::{Deserialize, Serialize};

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
    pub duration_ms: u128,
}

impl PackStats {
    pub fn from_duration(duration: Duration, mut stats: Self) -> Self {
        stats.duration_ms = duration.as_millis();
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
    pub duration_ms: u128,
}

impl UnpackStats {
    pub fn from_duration(duration: Duration, mut stats: Self) -> Self {
        stats.duration_ms = duration.as_millis();
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
