use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[derive(Debug, Clone, Default, Serialize)]
pub struct UnpackDiagnostics {
    pub config: UnpackDiagnosticsConfig,
    pub pipeline: PipelineDiagnostics,
    pub scatter: ScatterDiagnostics,
    pub finalize: FinalizeDiagnostics,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UnpackDiagnosticsConfig {
    pub worker_count: u64,
    pub decode_worker_count: u64,
    pub queue_depth: u64,
    pub writer_shard_count: u64,
    pub max_open_files: u64,
    pub max_open_files_per_shard: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct PipelineDiagnostics {
    pub decode_dispatch_wait_ns: u64,
    pub decode_dispatch_wait_count: u64,
    pub scatter_dispatch_wait_ns: u64,
    pub scatter_dispatch_wait_count: u64,
    pub bundles_observed: u64,
    pub total_bundle_extents: u64,
    pub total_bundle_unique_entries: u64,
    pub total_bundle_entry_switches: u64,
    pub max_bundle_extent_count: u64,
    pub max_bundle_unique_entries: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ScatterDiagnostics {
    pub writer_lock_wait_ns: u64,
    pub dir_cache_hits: u64,
    pub dir_cache_misses: u64,
    pub file_cache_hits: u64,
    pub file_cache_misses: u64,
    pub file_create_count: u64,
    pub file_reopen_count: u64,
    pub handle_eviction_count: u64,
    pub directory_open_ns: u64,
    pub file_open_ns: u64,
    pub write_ns: u64,
    pub extents_written: u64,
    pub bytes_written: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct FinalizeDiagnostics {
    pub regular_finalize_ns: u64,
    pub regular_finalize_count: u64,
    pub dir_finalize_ns: u64,
    pub dir_finalize_count: u64,
    pub symlink_create_ns: u64,
    pub symlink_create_count: u64,
    pub hardlink_create_ns: u64,
    pub hardlink_create_count: u64,
    pub trailer_verify_ns: u64,
    pub trailer_verify_count: u64,
}

#[derive(Debug, Default)]
pub(crate) struct UnpackDiagnosticsCollector {
    worker_count: AtomicU64,
    decode_worker_count: AtomicU64,
    queue_depth: AtomicU64,
    writer_shard_count: AtomicU64,
    max_open_files: AtomicU64,
    max_open_files_per_shard: AtomicU64,
    decode_dispatch_wait_ns: AtomicU64,
    decode_dispatch_wait_count: AtomicU64,
    scatter_dispatch_wait_ns: AtomicU64,
    scatter_dispatch_wait_count: AtomicU64,
    bundles_observed: AtomicU64,
    total_bundle_extents: AtomicU64,
    total_bundle_unique_entries: AtomicU64,
    total_bundle_entry_switches: AtomicU64,
    max_bundle_extent_count: AtomicU64,
    max_bundle_unique_entries: AtomicU64,
    writer_lock_wait_ns: AtomicU64,
    dir_cache_hits: AtomicU64,
    dir_cache_misses: AtomicU64,
    file_cache_hits: AtomicU64,
    file_cache_misses: AtomicU64,
    file_create_count: AtomicU64,
    file_reopen_count: AtomicU64,
    handle_eviction_count: AtomicU64,
    directory_open_ns: AtomicU64,
    file_open_ns: AtomicU64,
    write_ns: AtomicU64,
    extents_written: AtomicU64,
    bytes_written: AtomicU64,
    regular_finalize_ns: AtomicU64,
    regular_finalize_count: AtomicU64,
    dir_finalize_ns: AtomicU64,
    dir_finalize_count: AtomicU64,
    symlink_create_ns: AtomicU64,
    symlink_create_count: AtomicU64,
    hardlink_create_ns: AtomicU64,
    hardlink_create_count: AtomicU64,
    trailer_verify_ns: AtomicU64,
    trailer_verify_count: AtomicU64,
}

impl UnpackDiagnosticsCollector {
    pub(crate) fn record_pipeline_config(
        &self,
        worker_count: usize,
        decode_worker_count: usize,
        queue_depth: usize,
    ) {
        self.worker_count.store(
            worker_count.min(u64::MAX as usize) as u64,
            Ordering::Relaxed,
        );
        self.decode_worker_count.store(
            decode_worker_count.min(u64::MAX as usize) as u64,
            Ordering::Relaxed,
        );
        self.queue_depth
            .store(queue_depth.min(u64::MAX as usize) as u64, Ordering::Relaxed);
    }

    pub(crate) fn record_writer_config(
        &self,
        max_open_files: usize,
        shard_count: usize,
        max_open_files_per_shard: usize,
    ) {
        self.max_open_files.store(
            max_open_files.min(u64::MAX as usize) as u64,
            Ordering::Relaxed,
        );
        self.writer_shard_count
            .store(shard_count.min(u64::MAX as usize) as u64, Ordering::Relaxed);
        self.max_open_files_per_shard.store(
            max_open_files_per_shard.min(u64::MAX as usize) as u64,
            Ordering::Relaxed,
        );
    }

    pub(crate) fn record_decode_dispatch_wait(&self, duration: Duration, count: u64) {
        saturating_add(&self.decode_dispatch_wait_ns, elapsed_ns(duration));
        saturating_add(&self.decode_dispatch_wait_count, count);
    }

    pub(crate) fn record_scatter_dispatch_wait(&self, duration: Duration, count: u64) {
        saturating_add(&self.scatter_dispatch_wait_ns, elapsed_ns(duration));
        saturating_add(&self.scatter_dispatch_wait_count, count);
    }

    pub(crate) fn record_bundle_shape(
        &self,
        extent_count: usize,
        unique_entry_count: usize,
        entry_switch_count: usize,
    ) {
        saturating_add(&self.bundles_observed, 1);
        saturating_add(&self.total_bundle_extents, extent_count as u64);
        saturating_add(&self.total_bundle_unique_entries, unique_entry_count as u64);
        saturating_add(&self.total_bundle_entry_switches, entry_switch_count as u64);
        self.max_bundle_extent_count
            .fetch_max(extent_count as u64, Ordering::Relaxed);
        self.max_bundle_unique_entries
            .fetch_max(unique_entry_count as u64, Ordering::Relaxed);
    }

    pub(crate) fn record_writer_lock_wait(&self, duration: Duration) {
        saturating_add(&self.writer_lock_wait_ns, elapsed_ns(duration));
    }

    pub(crate) fn record_dir_cache_hit(&self) {
        saturating_add(&self.dir_cache_hits, 1);
    }

    pub(crate) fn record_dir_cache_miss(&self) {
        saturating_add(&self.dir_cache_misses, 1);
    }

    pub(crate) fn record_file_cache_hit(&self) {
        saturating_add(&self.file_cache_hits, 1);
    }

    pub(crate) fn record_file_cache_miss(&self) {
        saturating_add(&self.file_cache_misses, 1);
    }

    pub(crate) fn record_file_create(&self, duration: Duration) {
        saturating_add(&self.file_create_count, 1);
        saturating_add(&self.file_open_ns, elapsed_ns(duration));
    }

    pub(crate) fn record_file_reopen(&self, duration: Duration) {
        saturating_add(&self.file_reopen_count, 1);
        saturating_add(&self.file_open_ns, elapsed_ns(duration));
    }

    pub(crate) fn record_handle_evictions(&self, evicted: u64) {
        saturating_add(&self.handle_eviction_count, evicted);
    }

    pub(crate) fn record_directory_open(&self, duration: Duration) {
        saturating_add(&self.directory_open_ns, elapsed_ns(duration));
    }

    pub(crate) fn record_write(&self, duration: Duration, bytes: usize) {
        saturating_add(&self.write_ns, elapsed_ns(duration));
        saturating_add(&self.extents_written, 1);
        saturating_add(&self.bytes_written, bytes as u64);
    }

    pub(crate) fn record_regular_finalize(&self, duration: Duration) {
        saturating_add(&self.regular_finalize_count, 1);
        saturating_add(&self.regular_finalize_ns, elapsed_ns(duration));
    }

    pub(crate) fn record_dir_finalize(&self, duration: Duration, count: u64) {
        saturating_add(&self.dir_finalize_count, count);
        saturating_add(&self.dir_finalize_ns, elapsed_ns(duration));
    }

    pub(crate) fn record_symlink_create(&self, duration: Duration) {
        saturating_add(&self.symlink_create_count, 1);
        saturating_add(&self.symlink_create_ns, elapsed_ns(duration));
    }

    pub(crate) fn record_hardlink_create(&self, duration: Duration) {
        saturating_add(&self.hardlink_create_count, 1);
        saturating_add(&self.hardlink_create_ns, elapsed_ns(duration));
    }

    pub(crate) fn record_trailer_verify(&self, duration: Duration) {
        saturating_add(&self.trailer_verify_count, 1);
        saturating_add(&self.trailer_verify_ns, elapsed_ns(duration));
    }

    pub(crate) fn snapshot(&self) -> UnpackDiagnostics {
        UnpackDiagnostics {
            config: UnpackDiagnosticsConfig {
                worker_count: self.worker_count.load(Ordering::Relaxed),
                decode_worker_count: self.decode_worker_count.load(Ordering::Relaxed),
                queue_depth: self.queue_depth.load(Ordering::Relaxed),
                writer_shard_count: self.writer_shard_count.load(Ordering::Relaxed),
                max_open_files: self.max_open_files.load(Ordering::Relaxed),
                max_open_files_per_shard: self.max_open_files_per_shard.load(Ordering::Relaxed),
            },
            pipeline: PipelineDiagnostics {
                decode_dispatch_wait_ns: self.decode_dispatch_wait_ns.load(Ordering::Relaxed),
                decode_dispatch_wait_count: self.decode_dispatch_wait_count.load(Ordering::Relaxed),
                scatter_dispatch_wait_ns: self.scatter_dispatch_wait_ns.load(Ordering::Relaxed),
                scatter_dispatch_wait_count: self
                    .scatter_dispatch_wait_count
                    .load(Ordering::Relaxed),
                bundles_observed: self.bundles_observed.load(Ordering::Relaxed),
                total_bundle_extents: self.total_bundle_extents.load(Ordering::Relaxed),
                total_bundle_unique_entries: self
                    .total_bundle_unique_entries
                    .load(Ordering::Relaxed),
                total_bundle_entry_switches: self
                    .total_bundle_entry_switches
                    .load(Ordering::Relaxed),
                max_bundle_extent_count: self.max_bundle_extent_count.load(Ordering::Relaxed),
                max_bundle_unique_entries: self.max_bundle_unique_entries.load(Ordering::Relaxed),
            },
            scatter: ScatterDiagnostics {
                writer_lock_wait_ns: self.writer_lock_wait_ns.load(Ordering::Relaxed),
                dir_cache_hits: self.dir_cache_hits.load(Ordering::Relaxed),
                dir_cache_misses: self.dir_cache_misses.load(Ordering::Relaxed),
                file_cache_hits: self.file_cache_hits.load(Ordering::Relaxed),
                file_cache_misses: self.file_cache_misses.load(Ordering::Relaxed),
                file_create_count: self.file_create_count.load(Ordering::Relaxed),
                file_reopen_count: self.file_reopen_count.load(Ordering::Relaxed),
                handle_eviction_count: self.handle_eviction_count.load(Ordering::Relaxed),
                directory_open_ns: self.directory_open_ns.load(Ordering::Relaxed),
                file_open_ns: self.file_open_ns.load(Ordering::Relaxed),
                write_ns: self.write_ns.load(Ordering::Relaxed),
                extents_written: self.extents_written.load(Ordering::Relaxed),
                bytes_written: self.bytes_written.load(Ordering::Relaxed),
            },
            finalize: FinalizeDiagnostics {
                regular_finalize_ns: self.regular_finalize_ns.load(Ordering::Relaxed),
                regular_finalize_count: self.regular_finalize_count.load(Ordering::Relaxed),
                dir_finalize_ns: self.dir_finalize_ns.load(Ordering::Relaxed),
                dir_finalize_count: self.dir_finalize_count.load(Ordering::Relaxed),
                symlink_create_ns: self.symlink_create_ns.load(Ordering::Relaxed),
                symlink_create_count: self.symlink_create_count.load(Ordering::Relaxed),
                hardlink_create_ns: self.hardlink_create_ns.load(Ordering::Relaxed),
                hardlink_create_count: self.hardlink_create_count.load(Ordering::Relaxed),
                trailer_verify_ns: self.trailer_verify_ns.load(Ordering::Relaxed),
                trailer_verify_count: self.trailer_verify_count.load(Ordering::Relaxed),
            },
        }
    }
}

fn elapsed_ns(duration: Duration) -> u64 {
    duration.as_nanos().min(u128::from(u64::MAX)) as u64
}

fn saturating_add(slot: &AtomicU64, value: u64) {
    let _ = slot.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
        Some(current.saturating_add(value))
    });
}
