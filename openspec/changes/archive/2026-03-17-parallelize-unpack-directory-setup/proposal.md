## Why

Representative small-file unpack diagnostics show that the next meaningful bottleneck is now pre-pipeline setup rather than decode or scatter. On a warm-cache multi-bundle workload built from many `small-text` copies, raising unpack threads from 1 to 8 reduced total wall-time only from about `921ms` to `700ms` because `pipeline` shrank while `setup_ms` stayed roughly flat at `370ms`, making setup the largest remaining additive wall bucket.

## What Changes

- Define a focused unpack setup performance change aimed at reducing serial directory materialization and other pre-pipeline setup work on representative small-file workloads.
- Rework unpack so directory preparation no longer depends on one fully serial manifest-order pass that opens and prepares every directory before `run_unpack_pipeline()` begins, while preserving current safety checks and restore ordering guarantees.
- Keep the existing lazy regular-file preparation work intact and explicitly avoid changing wire format, CLI behavior, overwrite semantics, or metadata restore guarantees as part of this change.
- Add verification and benchmark guidance that keeps setup-side improvements auditable on representative multi-bundle small-file workloads instead of inferring them from the tiny committed default fixtures.

## Capabilities

### New Capabilities
- `unpack-setup-performance`: performance-oriented unpack requirements covering bounded, less-serial directory preparation before worker execution begins.

### Modified Capabilities
- `cli-and-benchmarks`: benchmark and verification guidance will expand so representative unpack setup measurements remain auditable across thread sweeps and repeated runs.

## Impact

- Affected code: `crates/sfa-unixfs/src/archive.rs`, `crates/sfa-unixfs/src/restore.rs`, unpack diagnostics, and related tests/benchmarks.
- Affected systems: setup-side directory creation and caching, unpack worker startup, representative performance verification, and benchmark documentation.
- No archive format, compatibility boundary, or user-facing CLI surface changes are intended.
