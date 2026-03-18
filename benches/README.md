# Benchmarks

This folder contains top-level benchmark assets for SFA v1.

## Goals

- Compare the default `sfa pack` and `sfa unpack` user path against a canonical `tar | zstd --fast=3` baseline.
- Keep the supported-host baseline stable enough for longitudinal regression checks without treating codec variants as the primary matrix.
- Store machine-readable outputs under `benches/results/`.
- Keep the default workload runnable from a clean checkout without any download step.
- Keep enough observability in the committed results to explain performance regressions, not just detect them.

## Entry Points

- Rust runner dry-run: `cargo run --manifest-path crates/sfa-bench/Cargo.toml --bin tar_vs_sfa -- --dry-run --output benches/results/latest.json`
- Workload materialization helper: `cargo run --manifest-path crates/sfa-bench/Cargo.toml --bin materialize_workload -- --output /tmp/node-modules-100k --summary-json /tmp/node-modules-100k-summary.json`
- Shell helper dry-run: `./benches/scripts/run_tar_vs_sfa.sh`
- Shell helper execute: `./benches/scripts/run_tar_vs_sfa.sh --execute --output benches/results/baseline-v0.1.0.json`

## Default Workload

- Workload asset: `benches/workloads/node-modules-100k/`
- Source of truth: committed `recipe.json`, `templates/`, and workload README under that directory
- Shape: deterministic `node_modules`-style nested package tree with `105,601` regular files, `10,560` generated packages, depth `5`, and dominant `.js`, `.json`, `.d.ts`, `README.md`, and `LICENSE` files
- The generated tree is not committed as a fixture; the runner materializes it into temporary space before execution

The canonical TAR baseline is `tar -cf - <input> | zstd --fast=3 > <archive>` for pack and `zstd -d -c <archive> | tar -xf - -C <output>` for unpack. This matches the effective SFA default compression profile of `zstd -3` without reintroducing a codec matrix into the default benchmark path.

## Current Baseline

- Committed result asset: `benches/results/baseline-v0.1.0.json`
- Recorded on: macOS `aarch64`
- Command profile captured in the JSON metadata: default `sfa pack` / `sfa unpack` vs canonical `tar | zstd --fast=3`
- Report fields include workload recipe metadata, command wall-time, files/s, MiB/s, archive or restored-byte size, unpack additive `wall_breakdown` buckets, unpack diagnostic `phase_breakdown` windows, and `wait4/getrusage` CPU / RSS observations for each executed record
- Unpack `wall_breakdown` records `setup`, `pipeline`, and `finalize` as a wall-time accounting view and should sum to the command `duration_ms`
- Unpack `phase_breakdown` continues to expose `header`, `manifest`, `frame_read`, `decode`, `scatter`, and `restore_finalize`; these phase windows are diagnostic and are not expected to sum to total wall-time

Repeated runs on the generated workload are cache-sensitive. For comparisons between thread counts or repeated baseline refreshes, keep cache warming consistent across runs or alternate run order so setup-side directory warming is not misread as a product change.

Refresh the committed baseline whenever the benchmark workload recipe, the default SFA command profile, the canonical TAR baseline, runner behavior, planner/pipeline behavior, result schema, observability fields, or supported benchmark environment changes in a way that could invalidate longitudinal comparison.
