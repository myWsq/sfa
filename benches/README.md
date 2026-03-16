# Benchmarks

This folder contains top-level benchmark assets for SFA v1.

## Goals

- Compare `sfa pack` and `sfa unpack` against `tar + same codec`.
- Keep the benchmark matrix stable for longitudinal regression checks.
- Store machine-readable outputs under `benches/results/`.

## Entry Points

- Rust runner: `cargo run --manifest-path crates/sfa-bench/Cargo.toml --bin tar_vs_sfa -- --dry-run`
- Shell helper: `./benches/scripts/run_tar_vs_sfa.sh`

## Dataset Matrix

- `small-text`
- `small-binary`
- `large-control`

Each dataset is expected under `tests/fixtures/datasets/<name>/input`.
