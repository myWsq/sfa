## 1. Reduce serial unpack setup

- [x] 1.1 Replace eager non-empty regular-file descriptor preparation with a lightweight restore-plan representation produced during setup.
- [x] 1.2 Update unpack setup so it preserves manifest parsing, restore-target planning, and directory pre-creation while removing per-file parent-directory handle opening before `run_unpack_pipeline()`.

## 2. Rework lazy regular-file restore paths

- [x] 2.1 Refactor `ConcurrentFileWriter` to resolve parent directories and regular-file handles lazily, with shard-aware caching that preserves current dirfd-style safety checks.
- [x] 2.2 Update the single-extent scatter path to create, write, and finalize regular files on demand without relying on an earlier serial prepare pass.
- [x] 2.3 Update the multi-extent regular-file path to lazily acquire reusable file handles during scatter and keep post-pipeline metadata finalize semantics intact.

## 3. Preserve observability and verify the new bottleneck shape

- [x] 3.1 Extend or adjust unpack diagnostics and targeted tests so they continue to explain setup, directory-open, file-open, and scatter behavior after the lazy-prepare refactor.
- [x] 3.2 Add representative small-file regression or benchmark coverage that keeps setup-vs-scatter bottlenecks auditable under explicit unpack thread overrides.
- [x] 3.3 Re-run the relevant unpack tests and benchmark/diagnostic flows, then update any affected verification or benchmark documentation for the new representative small-file workload.
