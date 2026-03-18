#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT_DIR"

CARGO_HOME="${CARGO_HOME:-/tmp/cargo-home}"
MODE="dry-run"
OUTPUT="benches/results/latest.json"
SFA_BIN="target/release/sfa"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --execute)
      MODE="execute"
      shift
      ;;
    --dry-run)
      MODE="dry-run"
      shift
      ;;
    --output)
      OUTPUT="$2"
      shift 2
      ;;
    --sfa-bin)
      SFA_BIN="$2"
      shift 2
      ;;
    *)
      echo "[bench] unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

echo "[bench] running default-path node_modules-100k benchmark harness"
echo "[bench] mode: ${MODE}"
echo "[bench] baseline: sfa defaults vs tar | zstd --fast=3"

RUN_ARGS=(--output "$OUTPUT")
if [[ "$MODE" == "dry-run" ]]; then
  echo "[bench] note: dry-run keeps CI/runtime cost low and does not require a built CLI binary"
  RUN_ARGS=(--dry-run "${RUN_ARGS[@]}")
else
  echo "[bench] building release CLI binary for benchmark execution"
  CARGO_HOME="$CARGO_HOME" cargo build --release -p sfa-cli
  RUN_ARGS=(--sfa-bin "$SFA_BIN" "${RUN_ARGS[@]}")
fi

CARGO_HOME="$CARGO_HOME" cargo run \
  --manifest-path crates/sfa-bench/Cargo.toml \
  --bin tar_vs_sfa \
  -- \
  "${RUN_ARGS[@]}"

echo "[bench] done, report: $OUTPUT"
