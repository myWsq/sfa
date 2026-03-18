## 1. Define the default `node_modules` benchmark workload

- [x] 1.1 Add committed recipe, templates, and documentation for a deterministic `node_modules`-style deep-directory benchmark workload that generates at least `100,000` regular files without network access
- [x] 1.2 Add validation or tests that the generated workload meets the documented file-count, directory-shape, and dominant file-type constraints

## 2. Rework benchmark execution around the default user path

- [x] 2.1 Replace the current codec matrix in `crates/sfa-bench` with default SFA pack/unpack commands and a canonical `tar | zstd -3` baseline
- [x] 2.2 Update benchmark preflight and workspace management so the runner can materialize the generated workload in temporary space, validate `tar`/`zstd`/`sfa` prerequisites, and fail before partial execution when setup is invalid
- [x] 2.3 Remove `lz4`-benchmark-specific command generation, script assumptions, and tests from the default benchmark path

## 3. Refresh the benchmark report and committed baseline

- [x] 3.1 Update the benchmark report schema and helper scripts so results capture workload recipe metadata, default-path throughput and size metrics, and the existing SFA observability fields in a form consumers can read without a codec matrix
- [x] 3.2 Run the redesigned benchmark on a supported host and commit the refreshed machine-readable result asset under `benches/results/`
- [x] 3.3 Add or update verification that the committed benchmark asset remains readable and aligned with the runner, workload recipe, and result schema

## 4. Align benchmark-facing and release-facing documentation

- [x] 4.1 Update `benches/README.md` and `spec/verification-and-benchmark.md` to describe the default-path benchmark contract, the `node_modules`-style workload, cache-warming caveats, and baseline regeneration workflow
- [x] 4.2 Update `README.md` so its benchmark snapshot cites the default-parameter `node_modules` workload and removes the current codec-matrix-focused headline claims
- [x] 4.3 Update `RELEASING.md` and related release guidance so dry-run remains mandatory while baseline refresh is tied to workload, default-command-profile, runner, and result-schema changes
