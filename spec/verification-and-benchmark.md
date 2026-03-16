# Verification And Benchmark Baseline

## Verification baseline

- Roundtrip correctness for supported Unix entry types
- Sequential stream decoding with fragmented input
- Corruption handling for header/manifest/frame/trailer
- Path safety enforcement for restore roots

## Benchmark baseline

- Compare SFA and TAR using the same codec (`lz4`, `zstd`)
- Run on `small-text`, `small-binary`, and `large-control` datasets
- Use committed input datasets under `tests/fixtures/datasets/<name>/input`
- Record machine-readable execution metadata under `benches/results/`, including the runner invocation, host/tool versions, dataset summaries, and per-command wall time
- Treat `benches/results/baseline-v0.1.0.json` as the current repository baseline for SFA v1

## Runner

`crates/sfa-bench/src/bin/tar_vs_sfa.rs` is the current benchmark runner entrypoint.

Recommended commands:

```bash
./benches/scripts/run_tar_vs_sfa.sh
./benches/scripts/run_tar_vs_sfa.sh --execute --sfa-bin target/release/sfa-cli --output benches/results/baseline-v0.1.0.json
```

Refresh the committed benchmark baseline when the benchmark runner, default datasets, planner/pipeline behavior, codec integration, or supported benchmark environment changes materially.
