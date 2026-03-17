## Context

`sfa unpack` now has real decode/scatter worker wiring and split observability, so the remaining bottlenecks are easier to see. Investigation on a representative multi-bundle small-file workload showed that raising unpack threads from 1 to 8 reduced total wall-time only from roughly 2.1s to 1.8s because the largest bucket remained serial `setup`, while pipeline time was dominated by filesystem syscall work rather than codec decode.

The current implementation pays that cost in two places:

- `unpack_reader_to_dir_internal()` performs eager directory creation and regular-file descriptor preparation for every non-empty regular file before `run_unpack_pipeline()` starts.
- The small-file scatter path restores each single-extent file through its own create/write/metadata sequence, so worker time is spent mostly on `openat`, `write_at`, `fchmod`, and `futimens` rather than on decode.

This change needs to reduce those costs without changing the archive wire format, path-safety guarantees, metadata semantics, or the bounded reader/decode/scatter/finalize model.

## Goals / Non-Goals

**Goals:**
- Reduce serial unpack setup by deferring non-empty regular-file preparation until worker-owned restore execution actually needs it.
- Make the small-file scatter path cheaper in syscall terms while preserving current overwrite, ownership, mode, and mtime behavior.
- Keep dirfd-style safety checks, bounded descriptor usage, and current restore ordering guarantees intact.
- Add regression and benchmark evidence that keeps the setup-vs-scatter bottleneck visible on representative small-file workloads.

**Non-Goals:**
- Do not change `.sfa` format, manifest semantics, CLI flags, or structured stats schema as part of this optimization.
- Do not redesign decode worker count heuristics or the overall reader/decode/scatter/finalize topology.
- Do not weaken path validation, symlink protections, or metadata restore guarantees to buy speed.
- Do not treat fragile wall-time ratios as CI gates.

## Decisions

### 1. Replace eager regular-file descriptor preparation with lazy restore plans

Setup will continue to parse the manifest, build restore targets, and pre-create directories, but it will stop opening parent directory handles and preparing per-file descriptors for every non-empty regular file before the pipeline begins. Instead, setup will build lightweight regular-file restore plans that contain only the data needed for safe later access, and `ConcurrentFileWriter` will resolve parent directory handles and file descriptors on demand.

This directly attacks the measured `setup` wall-time hotspot without changing restore semantics. It also avoids doing expensive work for files whose data path is already owned by scatter workers.

Alternatives considered:
- Parallelize the existing eager preparation loop: this would move some work off the main thread but would still front-load the same directory traversal and descriptor setup before useful restore work begins.
- Leave eager preparation in place and only tune queue depth or worker counts: measurements show decode is not the bottleneck, so this would not materially reduce the serial wall-time bucket.

### 2. Keep two regular-file restore paths, but make both lazy

Single-extent regular files will keep a dedicated fast path because they can still be restored in one worker-owned create/write/finalize sequence. The change is that this path will no longer depend on an earlier serial preparation pass. Multi-extent regular files will continue to use reusable file handles and a bounded cache, but those handles will be acquired lazily on first write rather than during setup, with metadata finalization staying in the current post-pipeline stage.

This preserves the current correctness boundary between data restore and finalize while removing redundant front-loaded work from both small-file and multi-extent paths.

Alternatives considered:
- Collapse everything into the multi-extent shared-handle path: simpler, but it throws away the current single-extent fast path and would likely increase small-file overhead.
- Finalize all regular files only after the pipeline: safer to reason about, but it would miss the current one-shot small-file optimization and keep unnecessary bookkeeping around.

### 3. Move parent-directory handle caching into the writer side

The current `prepare_regular_descriptor()` path uses a serial `dir_cache` during setup. After this change, safe parent-directory discovery and `openat(..., O_NOFOLLOW)` checks will move closer to the writer path, most likely as shard-local caches owned by `ConcurrentFileWriter`. This keeps the dirfd-style safety model intact while avoiding a global pre-open pass over every regular-file parent path.

Shard-local caches are preferred over one shared global cache because they align with the existing writer shard model and avoid turning the new lazy path into a high-contention mutex bottleneck.

Alternatives considered:
- Use path-string-based open APIs from `std::fs`: rejected because it weakens the current symlink-traversal protections.
- Use one global concurrent directory cache: viable, but likely to introduce coordination cost in the same place where we want the small-file path to stay cheap.

### 4. Add representative unpack-performance regression coverage

The existing committed benchmark datasets are valuable, but the smallest ones are too short to keep this bottleneck legible. This change will add focused verification coverage for a representative multi-bundle small-file workload, either via a generated temporary corpus in tests or a benchmark-side diagnostic path, so maintainers can tell whether a future regression moved time back into eager setup or per-file scatter syscalls.

The goal is diagnostic comparability, not a brittle performance gate. We want a reproducible workload that keeps the hotspot explainable, not a hard-coded speedup ratio.

Alternatives considered:
- Depend only on the committed tiny datasets: they are useful correctness fixtures but too noisy for this particular bottleneck.
- Add CI assertions on absolute timings or scaling ratios: too environment-sensitive to be reliable.

## Risks / Trade-offs

- [Lazy preparation could accidentally change restore ordering] → Keep directory creation and link/finalize order unchanged, and defer only non-empty regular-file descriptor work.
- [Writer-side directory caching could increase FD pressure] → Preserve bounded file-handle budgets, keep caches shard-local, and extend diagnostics where needed to expose directory-open churn.
- [Separate single-extent and multi-extent paths can drift apart] → Share restore-plan and metadata helpers so both paths still use one safety and metadata contract.
- [Representative small-file coverage can become noisy] → Keep it diagnostic and structural, not a release-blocking timing threshold.

## Migration Plan

1. Introduce a lightweight regular-file restore-plan representation and switch setup to produce plans rather than prepared descriptors.
2. Update `ConcurrentFileWriter` and the small-file scatter path to resolve parent directories and file handles lazily.
3. Preserve current multi-extent finalize behavior while moving its first-handle acquisition out of setup.
4. Add regression and benchmark coverage for representative small-file unpack behavior, then refresh any affected performance documentation.

## Open Questions

- Should parent-directory handles live in shard-local caches, worker-local caches, or a shared concurrent cache with sharding?
- Should the current `PreparedRegularFile` type be replaced outright with a lighter restore-plan type, or evolved incrementally to minimize refactor risk?
- Is the current equal split of `max_open_files` across shards still the right policy once parent-directory lookups also move into the writer side?
