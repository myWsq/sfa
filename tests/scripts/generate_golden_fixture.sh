#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 4 ]; then
  echo "Usage: $0 <input-dir> <archive.sfa> <manifest.json> <stats.json> [pack args...]" >&2
  exit 2
fi

INPUT_DIR="$1"
ARCHIVE_PATH="$2"
MANIFEST_PATH="$3"
STATS_PATH="$4"
shift 4

CARGO_HOME="${CARGO_HOME:-/tmp/cargo-home}" cargo run -p sfa-bench --bin generate_golden_archive -- \
  "$INPUT_DIR" "$ARCHIVE_PATH" "$@"

CARGO_HOME="${CARGO_HOME:-/tmp/cargo-home}" cargo run -p sfa-bench --bin dump_archive_fixture -- \
  "$ARCHIVE_PATH" "$MANIFEST_PATH" --summary-out "$STATS_PATH"
