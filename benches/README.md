# Benchmarks

This folder contains top-level benchmark assets for SFA v1.

## Goals

- Compare `sfa pack` and `sfa unpack` against `tar + same codec`.
- Keep the benchmark matrix stable for longitudinal regression checks.
- Store machine-readable outputs under `benches/results/`.
- Keep the default datasets runnable from a clean checkout without any download step.
- Keep enough observability in the committed results to explain performance regressions, not just detect them.

## Entry Points

- Rust runner dry-run: `cargo run --manifest-path crates/sfa-bench/Cargo.toml --bin tar_vs_sfa -- --dry-run --output benches/results/latest.json`
- Shell helper dry-run: `./benches/scripts/run_tar_vs_sfa.sh`
- Shell helper execute: `./benches/scripts/run_tar_vs_sfa.sh --execute --output benches/results/baseline-v0.1.0.json`

## Dataset Matrix

- `small-text`
- `small-binary`
- `large-control`

Each dataset is committed under `tests/fixtures/datasets/<name>/input`, and each dataset directory includes a README with its purpose and stable size summary.

## Current Baseline

- Committed result asset: `benches/results/baseline-v0.1.0.json`
- Recorded on: macOS `aarch64`
- Toolchain captured in the JSON metadata: `target/release/sfa-cli`, `/usr/bin/tar` (`bsdtar 3.5.3`), Homebrew `lz4` `1.10.0`, and Homebrew `zstd` `1.5.7`
- Report fields include command wall-time, SFA internal phase breakdowns, and `wait4/getrusage` CPU / RSS observations for each executed record

Refresh the committed baseline whenever the benchmark runner, default dataset corpus, planner/pipeline behavior, codec integration, or release benchmark environment changes in a way that could invalidate longitudinal comparison.
