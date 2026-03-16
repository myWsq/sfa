#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 3 ]; then
  echo "Usage: $0 <input-dir> <archive.sfa> <dump.json>" >&2
  exit 2
fi

INPUT_DIR="$1"
ARCHIVE_PATH="$2"
DUMP_PATH="$3"

CARGO_HOME="${CARGO_HOME:-/tmp/cargo-home}" cargo run -p sfa-cli -- pack "$INPUT_DIR" "$ARCHIVE_PATH"
CARGO_HOME="${CARGO_HOME:-/tmp/cargo-home}" cargo run -p sfa-bench --bin dump_archive_fixture -- "$ARCHIVE_PATH" "$DUMP_PATH"
