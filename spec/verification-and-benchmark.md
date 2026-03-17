# Verification And Benchmark Baseline

## Verification baseline

- Roundtrip correctness for supported Unix entry types
- Sequential stream decoding with fragmented input
- `sync Read` / CLI stdin unpack roundtrip for the same archive content
- Corruption handling for header/manifest/frame/trailer
- Path safety enforcement for restore roots
- Reject unpack when a pre-existing symlink inside the output root would escape the restore tree
- When `strong` trailer verification fails after restore work has started, leave `.sfa-untrusted` in the output root before returning an error

## Benchmark baseline

- Compare SFA and TAR using the same codec (`lz4`, `zstd`)
- Run on `small-text`, `small-binary`, and `large-control` datasets
- Use committed input datasets under `tests/fixtures/datasets/<name>/input`
- Record machine-readable execution metadata under `benches/results/`, including the runner invocation, host/tool versions, dataset summaries, per-command wall time, and runner-level resource sampling metadata
- For SFA commands, record machine-readable pack / unpack stats including pack phase breakdowns for `scan`, `plan`, `encode`, and `write`, plus unpack additive `wall_breakdown` buckets for `setup`, `pipeline`, and `finalize` and unpack diagnostic phase breakdowns for `header`, `manifest`, `frame_read`, `decode`, `scatter`, and `restore_finalize`
- On supported Unix benchmark hosts, record command-level `user_cpu_ms`, `system_cpu_ms`, and `max_rss_kib` observations derived from `wait4/getrusage`
- Treat `benches/results/baseline-v0.1.0.json` as the current repository baseline for SFA v1

## Runner

`crates/sfa-bench/src/bin/tar_vs_sfa.rs` is the current benchmark runner entrypoint.
The runner requests `--stats-format json` from `sfa-cli` so it can persist structured SFA timing alongside command wall-time.
For unpack, `wall_breakdown` is the additive wall-time accounting view and its serialized buckets should sum to the total command `duration_ms`.
For unpack, the split `phase_breakdown` fields remain diagnostic windows for a pipelined restore path; they are not required to sum to the total command wall-time.
When `sfa unpack` is run with an explicit `--threads` override, the resulting stats and benchmark records should preserve that effective worker count for later comparison.
Real-world thread sweeps on large small-file corpora remain diagnostic evidence, not correctness proofs; when they regress against the previous known baseline, keep the results for analysis and do not silently treat the change as performance-accepted.

For setup-vs-scatter diagnosis on small-file workloads, use a representative temporary corpus rather than relying only on the tiny committed fixtures. One reproducible pattern is to copy `tests/fixtures/datasets/small-text/input` many times into a temporary tree, pack it with a small `--bundle-target-bytes` value so multiple bundles are produced, and then compare `sfa unpack --threads 1` against higher thread counts with `SFA_UNPACK_DIAGNOSTICS_JSON` enabled. On that workload:
- `wall_breakdown.setup` explains how much time remains serial before workers start.
- `phase_breakdown.scatter` explains worker busy time, which can overlap across threads.
- diagnostics fields such as `directory_open_ns`, `file_open_ns`, and `write_ns` explain whether the hotspot is dominated by filesystem syscall work rather than decode.
- after setup-side directory prewarming, `directory_open_ns` may legitimately fall to zero; use `dir_cache_hits`/`dir_cache_misses` together with `file_open_ns` to understand whether scatter is still paying for parent-directory discovery.

Recommended commands:

```bash
./benches/scripts/run_tar_vs_sfa.sh
./benches/scripts/run_tar_vs_sfa.sh --execute --sfa-bin target/release/sfa-cli --output benches/results/baseline-v0.1.0.json

tmp_root="$(mktemp -d)"
for i in $(seq -w 1 200); do cp -R tests/fixtures/datasets/small-text/input "$tmp_root/case-$i"; done
cargo run -p sfa-cli -- pack "$tmp_root" /tmp/sfa-smallfiles.zstd.sfa --codec zstd --bundle-target-bytes 65536 --stats-format json
SFA_UNPACK_DIAGNOSTICS_JSON=/tmp/diag-t1.json cargo run -p sfa-cli -- unpack /tmp/sfa-smallfiles.zstd.sfa -C /tmp/out-t1 --threads 1 --stats-format json
SFA_UNPACK_DIAGNOSTICS_JSON=/tmp/diag-t8.json cargo run -p sfa-cli -- unpack /tmp/sfa-smallfiles.zstd.sfa -C /tmp/out-t8 --threads 8 --stats-format json
```

Refresh the committed benchmark baseline when the benchmark runner, default datasets, planner/pipeline behavior, codec integration, or supported benchmark environment changes materially.
Also refresh it when the benchmark report schema or observability fields change in a way that affects interpretation of committed results.
