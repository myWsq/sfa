## 1. Restructure unpack setup around an explicit directory plan

- [x] 1.1 Extract directory setup planning from the current manifest-order `create_dir()` loop while preserving manifest-order directory finalize semantics.
- [x] 1.2 Refactor directory creation helpers so setup code can create one directory from a known prepared parent handle and return reusable prepared-directory entries for later restore work.

## 2. Parallelize directory materialization without changing restore semantics

- [x] 2.1 Implement depth-ordered setup frontiers that materialize independent directories with bounded worker parallelism before `run_unpack_pipeline()` begins.
- [x] 2.2 Merge setup-worker results back into the prepared-directory cache consumed by `ConcurrentFileWriter` and keep later directory lookup behavior unchanged.
- [x] 2.3 Add coverage for parent-before-child behavior, overwrite handling, and narrow-tree fallback under the new setup path.

## 3. Keep setup-focused verification and guidance auditable

- [x] 3.1 Add representative small-file diagnostics or regression coverage that shows setup remains measurable separately from pipeline work after directory setup parallelism lands.
- [x] 3.2 Update benchmark or verification documentation to explain the representative setup-focused workload and how repeated unpack runs can warm caches during comparisons.
- [x] 3.3 Re-run the relevant unpack tests and representative diagnostic flows, then refresh any affected documentation or guidance assets.
