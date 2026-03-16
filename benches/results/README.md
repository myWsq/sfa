# Benchmark Results

This directory stores committed machine-readable benchmark results for the default SFA v1 matrix.
The committed JSON schema includes command wall-time for every record, SFA phase breakdowns for `sfa` commands, and `wait4/getrusage`-based CPU / RSS observations on supported Unix hosts.
Unpack phase breakdowns are recorded as `header`, `manifest`, `frame_read`, `decode`, `scatter`, and `restore_finalize`; these are pipelined diagnostic windows rather than a simple additive timing ledger.

Current committed baseline:

- `baseline-v0.1.0.json`

Generation command used for the current baseline:

```bash
./benches/scripts/run_tar_vs_sfa.sh \
  --execute \
  --sfa-bin target/release/sfa-cli \
  --output benches/results/baseline-v0.1.0.json
```

Supported environment for the committed result:

- macOS `aarch64`
- `/usr/bin/tar` (`bsdtar 3.5.3`)
- `lz4` `1.10.0`
- `zstd` `1.5.7`
- `target/release/sfa-cli` built from this repository
- `wait4/getrusage` available so `user_cpu_ms`, `system_cpu_ms`, and `max_rss_kib` are populated in the result asset

Refresh the committed baseline when:

- the default benchmark datasets change
- benchmark runner behavior or command generation changes
- benchmark observability fields or report schema change
- planner, pipeline, or codec behavior changes in a way that affects performance comparison
- the repository adopts a different supported benchmark host/toolchain for release evidence
