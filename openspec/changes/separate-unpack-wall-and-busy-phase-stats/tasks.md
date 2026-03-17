## 1. Stats schema and unpack sampling

- [x] 1.1 Extend `crates/sfa-core` unpack stats types with an additive `wall_breakdown` schema for `setup`, `pipeline`, and `finalize` while preserving the existing diagnostic `phase_breakdown`.
- [x] 1.2 Update `crates/sfa-unixfs` unpack timing collection to measure contiguous wall-time buckets with high-precision `Duration` values and serialize them so the reported wall buckets sum exactly to `duration_ms`.
- [x] 1.3 Keep the existing diagnostic `phase_breakdown` semantics intact, but ensure decode/scatter/frame-read accumulation only truncates at final output time rather than per subtask.

## 2. CLI and benchmark integration

- [x] 2.1 Update `crates/sfa-cli` machine-readable unpack output and diagnostics report wiring so successful non-dry-run unpacks emit both additive `wall_breakdown` and diagnostic `phase_breakdown`.
- [x] 2.2 Update `crates/sfa-bench` parsing, report schema, and validations so benchmark records preserve both unpack timing views together with effective thread count.
- [x] 2.3 Refresh any benchmark-facing JSON fixtures or committed baseline assets that need schema updates after the new unpack observability fields land.

## 3. Verification and documentation

- [x] 3.1 Add regression tests that prove serialized `wall_breakdown` sums to `duration_ms` and that diagnostic unpack phase timings remain populated for representative multi-bundle workloads.
- [x] 3.2 Add CLI/bench regression coverage for the new unpack JSON schema, including stdin/file entry paths and dry-run unavailable semantics.
- [x] 3.3 Update benchmark and verification documentation to explain the difference between additive wall buckets, overlapping diagnostic phase windows, and deep-dive diagnostics fields.
