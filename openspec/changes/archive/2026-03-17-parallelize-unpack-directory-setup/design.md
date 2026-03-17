## Context

The previous unpack optimization work removed eager non-empty regular-file descriptor preparation from setup, but representative small-file diagnostics still show setup as the largest additive wall bucket. On a warm-cache multi-bundle workload built from many `small-text` copies, unpack improved from roughly `921ms` at one thread to `700ms` at eight threads, yet `setup_ms` remained flat at about `370ms` while only `pipeline_ms` shrank. That leaves the remaining pre-pipeline directory pass as the clearest limit on further scaling.

Today `unpack_reader_to_dir_internal()` still walks every directory entry before `run_unpack_pipeline()` and calls `LocalRestorer::create_dir()` serially. `create_directory_with_cache()` preserves the current dirfd/openat safety model and prewarms the writer's directory cache, but it also means setup performs all directory creation and handle materialization on one thread before any decode or scatter work starts.

This change needs to reduce that setup cost without changing archive format, restore ordering guarantees, overwrite semantics, metadata behavior, or the bounded reader/decode/scatter/finalize architecture.

## Goals / Non-Goals

**Goals:**
- Reduce the serial portion of unpack setup attributable to directory materialization on representative small-file workloads.
- Preserve the current property that directories needed for regular-file restore exist before the decode/scatter pipeline starts.
- Keep dirfd-style path validation, overwrite handling, and directory metadata finalization semantics intact.
- Preserve setup-side directory prewarming so later restore work does not pay the same parent-directory discovery cost again.
- Keep representative setup-focused diagnostics and documentation auditable across thread sweeps and repeated runs.

**Non-Goals:**
- Do not redesign the decode/scatter pipeline, worker-count heuristics, or the current regular-file restore paths in this change.
- Do not switch to fully lazy directory creation during scatter or finalize.
- Do not change `.sfa` wire format, CLI flags, or stable machine-readable stats schema.
- Do not add brittle CI timing thresholds or promise a fixed speedup ratio.

## Decisions

### 1. Split directory setup planning from directory materialization

Setup will stop treating directory preparation as an incidental side effect of iterating manifest entries. Instead, unpack will build an explicit directory-setup plan from the restore targets, while preserving manifest-order `dir_finalize_order` separately for later metadata finalization.

This separation makes the expensive part of setup easier to optimize without disturbing the later finalize ordering contract. It also avoids coupling performance work to the current `LocalRestorer::create_dir()` call pattern.

Alternatives considered:
- Keep the current serial `create_dir()` loop and only micro-optimize helper functions: too little headroom given the measured flat `setup_ms`.
- Move directory creation fully into writer-side lazy restore: larger semantic change, more risk around restore ordering, and unnecessary for the current bottleneck.

### 2. Materialize directories in depth-ordered parallel frontiers

Directory setup will execute in ascending path-depth frontiers. Parents at depth `N` must be materialized before children at depth `N+1`, but directories within the same frontier can be created concurrently because their required parents already exist. The setup worker count will be bounded by the effective unpack thread count and the frontier width.

Each worker will create only the leaf directory for its assigned target using an already prepared parent handle from the previous frontier, rather than re-walking the full ancestor chain from the root for every directory. This keeps the existing openat-based safety model while reducing repeated traversal work and allowing parallelism when the archive exposes multiple sibling directories.

Alternatives considered:
- Partition only by top-level subtree: simpler, but it leaves too much performance on the table for archives concentrated under one root subtree.
- Use a shared global work queue with fully dynamic parent readiness tracking: more flexible, but significantly more coordination complexity than the current problem justifies.

### 3. Preserve prepared-directory prewarming as a first-class output of setup

The setup phase will continue to hand `ConcurrentFileWriter` a prepared directory-handle map containing the root and every created directory. Parallel directory workers may use local temporary state, but their results will merge back into one prepared-directory cache before the pipeline begins.

This preserves the current useful property that scatter-side directory lookup can stay at cache-hit paths for directories already prepared in setup. It avoids fixing `setup_ms` only by pushing the same parent-directory discovery work back into scatter.

Alternatives considered:
- Stop prewarming directory handles and let the writer rediscover parents lazily: simpler setup, but likely shifts cost from setup to scatter instead of reducing it.
- Share one mutable cache directly across setup workers: viable, but introduces contention into the new setup path without a clear need.

### 4. Keep setup optimization evidence diagnostic, and document cache-sensitive comparisons

Verification coverage and benchmark guidance will continue to use a representative temporary multi-bundle small-file corpus rather than the tiny committed default fixtures for this optimization. Documentation will explicitly call out that repeated unpack runs can warm filesystem caches and that thread comparisons should either use warmed runs consistently or alternate run order to avoid attributing cache effects to setup parallelism.

This keeps the change auditable without turning environment-sensitive timing into a correctness gate.

Alternatives considered:
- Depend on committed default benchmark fixtures alone: too small to make the setup bottleneck legible.
- Add hard pass/fail speed assertions in CI: too noisy and host-dependent.

## Risks / Trade-offs

- [Depth-frontier barriers may still leave some archives partially serial] → Accept that narrow single-chain trees will not parallelize much, but keep the implementation simple and preserve correctness on all shapes.
- [Parallel setup could accidentally violate overwrite or parent-before-child behavior] → Build one directory plan first, deduplicate targets, and only schedule a frontier after every parent frontier completes.
- [Prepared directory handles could increase FD pressure] → Reuse the existing prepared-directory cache lifecycle and continue clearing handles after unpack completes.
- [Cache-warmed comparisons can mislead performance conclusions] → Keep representative diagnostics in documentation and explicitly describe how to compare repeated runs.

## Migration Plan

1. Introduce a directory-setup planning step that separates directory materialization from finalize ordering.
2. Refactor directory creation helpers so a worker can create one directory from a known prepared parent handle without re-walking the full path.
3. Execute directory frontiers in bounded parallel setup workers, merge prepared handles into the writer cache, and keep the rest of unpack startup unchanged.
4. Add or refresh representative setup-focused tests and benchmark documentation covering thread sweeps and repeated-run cache effects.

## Open Questions

- Do we need dedicated setup diagnostics counters beyond `wall_breakdown.setup_ms`, or is the existing timing view plus representative workload guidance sufficient?
- Should the frontier scheduler always use the effective unpack thread count, or should very small frontiers cap themselves more aggressively to reduce setup overhead?
