#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

CARGO_HOME="${CARGO_HOME:-/tmp/cargo-home}" cargo test -p sfa-core

while IFS= read -r archive_path; do
  fixture_dir="$(dirname "$archive_path")"
  fixture_name="$(basename "$fixture_dir")"
  manifest_path="$fixture_dir/manifest.json"
  stats_path="$fixture_dir/stats.json"
  readme_path="$fixture_dir/README.md"
  input_dir="$fixture_dir/input"
  actual_manifest="$TMP_DIR/$fixture_name.manifest.json"
  actual_stats="$TMP_DIR/$fixture_name.stats.json"

  for required in "$archive_path" "$manifest_path" "$stats_path" "$readme_path"; do
    if [ ! -e "$required" ]; then
      echo "[test:protocol] missing fixture asset: $required" >&2
      exit 1
    fi
  done

  if [ ! -d "$input_dir" ]; then
    echo "[test:protocol] missing fixture input tree: $input_dir" >&2
    exit 1
  fi

  CARGO_HOME="${CARGO_HOME:-/tmp/cargo-home}" cargo run -p sfa-bench --bin dump_archive_fixture -- \
    "$archive_path" "$actual_manifest" --summary-out "$actual_stats"

  if ! cmp -s "$manifest_path" "$actual_manifest"; then
    echo "[test:protocol] manifest drift detected for fixture: $fixture_name" >&2
    git diff --no-index --no-ext-diff -- "$manifest_path" "$actual_manifest" || true
    exit 1
  fi

  if ! cmp -s "$stats_path" "$actual_stats"; then
    echo "[test:protocol] summary drift detected for fixture: $fixture_name" >&2
    git diff --no-index --no-ext-diff -- "$stats_path" "$actual_stats" || true
    exit 1
  fi
done < <(find "$ROOT_DIR/tests/fixtures/golden" -mindepth 2 -maxdepth 2 -name archive.sfa | sort)

echo "[test:protocol] ok"
