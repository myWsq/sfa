## 1. Unpack Contract And Observability

- [x] 1.1 Update the unpack stats model, CLI JSON output, and benchmark report schema to replace `decode_and_scatter` with `frame_read`, `decode`, and `scatter` while preserving explicit unavailable semantics for dry-run.
- [x] 1.2 Update the unpack-facing spec, benchmark, and release documentation to describe the new phase semantics, their non-additive interpretation, and the requirement that `unpack --threads` controls effective worker count.

## 2. Reader And Pipeline Refactor

- [x] 2.1 Refactor `ArchiveReader` so frame iteration performs sequential framing and payload reads without forcing a full codec decode inside `next_frame()`.
- [x] 2.2 Rework `unpack_archive()` into a bounded parallel pipeline that reads frames sequentially, decodes and verifies bundles once, and schedules multi-bundle restore work according to the effective thread count.
- [x] 2.3 Ensure unpack integrity behavior remains correct after the refactor, including frame-hash rejection and strong trailer verification on the new pipeline.

## 3. Concurrent Restore Path

- [x] 3.1 Split the restore implementation so regular-file data writes can run safely from multiple workers while symlink, hardlink, metadata, and directory finalize steps remain correct.
- [x] 3.2 Replace the current single-threaded file-handle cache assumptions with a bounded strategy that supports concurrent `write_at` usage without violating path-safety guarantees.

## 4. Regression Coverage And Baseline Refresh

- [x] 4.1 Add deterministic regression tests or probes that prove multi-bundle unpack uses the configured worker count and does not decode each frame more than once on the normal success path.
- [x] 4.2 Update benchmark and smoke-level validation to persist the new unpack phase fields and to keep explicit thread-count observability for unpack diagnostic runs.
- [x] 4.3 Re-run the relevant test suites, unpack thread-sweep checks, and committed benchmark/report generation steps needed to document the repaired unpack pipeline behavior.
