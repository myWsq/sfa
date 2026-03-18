# Benchmark Results

This directory stores committed machine-readable benchmark results for the default SFA v1 benchmark path.
The committed JSON schema includes workload recipe metadata, command wall-time for every record, files/s, MiB/s, output size, SFA timing breakdowns for `sfa` commands, and `wait4/getrusage`-based CPU / RSS observations on supported Unix hosts.
Unpack timing is stored in two views:

- `wall_breakdown`: additive `setup`, `pipeline`, and `finalize` buckets that should sum to the unpack command `duration_ms`
- `phase_breakdown`: `header`, `manifest`, `frame_read`, `decode`, `scatter`, and `restore_finalize` diagnostic windows for the pipelined restore path

Current committed baseline:

- `baseline-v0.1.0.json`

Recommended command to reproduce the current baseline contract:

```bash
./benches/scripts/run_tar_vs_sfa.sh \
  --execute \
  --sfa-bin target/release/sfa \
  --output benches/results/baseline-v0.1.0.json
```

Optional helper for diagnostic sweeps on the same workload:

```bash
cargo run --manifest-path crates/sfa-bench/Cargo.toml \
  --bin materialize_workload \
  -- \
  --output /tmp/node-modules-100k \
  --summary-json /tmp/node-modules-100k-summary.json
```

Supported environment for the committed result:

- macOS `aarch64`
- `/usr/bin/tar` (`bsdtar 3.5.3`)
- `zstd` `1.5.7`
- `target/release/sfa` built from this repository
- `wait4/getrusage` available so `user_cpu_ms`, `system_cpu_ms`, and `max_rss_kib` are populated in the result asset

Refresh the committed baseline when:

- the default benchmark workload recipe or materialization contract changes
- benchmark runner behavior or command generation changes
- benchmark observability fields or report schema change
- the default SFA command profile or canonical `tar | zstd --fast=3` baseline changes
- planner or pipeline behavior changes in a way that affects performance comparison
- the repository adopts a different supported benchmark host/toolchain for release evidence
