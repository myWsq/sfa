## Why

Current `sfa unpack` diagnostics show that multithreaded decode/scatter wiring is working, but total wall-time on multi-bundle small-file workloads is still dominated by work that happens outside or underneath that pipeline. In particular, eager regular-file preparation keeps `setup` mostly serial, and the small-file restore path still spends most of its time on per-file open, metadata, and directory traversal syscalls rather than codec work.

## What Changes

- Define a focused unpack-performance change that targets the two observed bottlenecks: serial pre-pipeline setup and syscall-heavy small-file scatter.
- Rework unpack so non-empty regular-file preparation no longer requires eager descriptor/path setup for every file before worker execution begins.
- Introduce a more syscall-efficient small-file restore path that preserves current safety and metadata guarantees while reducing per-file open/finalize overhead.
- Add verification and benchmark coverage that keeps the bottleneck visible on multi-bundle small-file workloads instead of relying only on tiny baseline fixtures.

## Capabilities

### New Capabilities
- `unpack-performance`: performance-oriented unpack requirements covering deferred regular-file preparation, bounded setup work, and syscall-efficient restore behavior for small-file workloads.

### Modified Capabilities
- `cli-and-benchmarks`: benchmark and verification guidance will expand to keep the new unpack-performance evidence auditable on representative small-file workloads.

## Impact

- Affected code: `crates/sfa-unixfs/src/archive.rs`, `crates/sfa-unixfs/src/restore.rs`, unpack diagnostics, and related tests/benchmarks.
- Affected systems: unpack worker scheduling, restore-path descriptor handling, benchmark fixtures or synthetic workload generation, and performance-facing documentation.
- No wire-format, archive compatibility, or CLI surface changes are intended.
