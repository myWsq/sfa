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
- Record machine-readable execution metadata under `benches/results/`, including the runner invocation, host/tool versions, dataset summaries, per-command wall time, and runner-level resource sampling metadata
- For SFA commands, record machine-readable pack / unpack stats including phase breakdowns for `scan`, `plan`, `encode`, `write`, `header`, `manifest`, `decode_and_scatter`, and `restore_finalize`
- On supported Unix benchmark hosts, record command-level `user_cpu_ms`, `system_cpu_ms`, and `max_rss_kib` observations derived from `wait4/getrusage`
- Treat `benches/results/baseline-v0.1.0.json` as the current repository baseline for SFA v1

## Runner

`crates/sfa-bench/src/bin/tar_vs_sfa.rs` is the current benchmark runner entrypoint.
The runner requests `--stats-format json` from `sfa-cli` so it can persist structured SFA phase timing alongside command wall-time.

Recommended commands:

```bash
./benches/scripts/run_tar_vs_sfa.sh
./benches/scripts/run_tar_vs_sfa.sh --execute --sfa-bin target/release/sfa-cli --output benches/results/baseline-v0.1.0.json
```

Refresh the committed benchmark baseline when the benchmark runner, default datasets, planner/pipeline behavior, codec integration, or supported benchmark environment changes materially.
Also refresh it when the benchmark report schema or observability fields change in a way that affects interpretation of committed results.
