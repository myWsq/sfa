#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT_DIR"

echo "[bench] running tar-vs-sfa benchmark harness"
echo "[bench] note: default script uses --dry-run to keep CI/runtime cost low"

cargo run \
  --manifest-path crates/sfa-bench/Cargo.toml \
  --bin tar_vs_sfa \
  -- \
  --dry-run \
  --output benches/results/latest.json

echo "[bench] done, report: benches/results/latest.json"
