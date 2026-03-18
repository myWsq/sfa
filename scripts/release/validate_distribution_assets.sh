#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
SFA_INSTALLER_SOURCE_ONLY=1 . "$ROOT_DIR/install.sh"

usage() {
  cat <<'EOF'
Usage: bash scripts/release/validate_distribution_assets.sh \
  --release-tag vX.Y.Z \
  --assets-json /path/to/release-assets.json
EOF
}

release_tag=""
assets_json=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --release-tag)
      release_tag="${2:-}"
      shift 2
      ;;
    --assets-json)
      assets_json="${2:-}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      printf 'validate_distribution_assets: unknown argument: %s\n' "$1" >&2
      exit 1
      ;;
  esac
done

[[ -n "$release_tag" ]] || { usage >&2; exit 1; }
[[ -n "$assets_json" ]] || { usage >&2; exit 1; }
[[ -f "$assets_json" ]] || { printf 'missing assets JSON: %s\n' "$assets_json" >&2; exit 1; }

release_tag="$(sfa_normalize_release_tag "$release_tag")"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

asset_rows="$(python3 - "$assets_json" "$release_tag" <<'PY'
import json
import sys

assets_path, release_tag = sys.argv[1], sys.argv[2]
targets = [
    "x86_64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
]

with open(assets_path, "r", encoding="utf-8") as fh:
    payload = json.load(fh)

assets = payload.get("assets", [])
by_name = {entry["name"]: entry for entry in assets}

for target in targets:
    archive_name = f"sfa-{release_tag}-{target}.tar.gz"
    checksum_name = f"{archive_name}.sha256"

    archive = by_name.get(archive_name)
    checksum = by_name.get(checksum_name)

    if archive is None:
        raise SystemExit(f"missing release asset: {archive_name}")
    if checksum is None:
        raise SystemExit(f"missing release asset: {checksum_name}")

    digest = archive.get("digest", "")
    if not digest.startswith("sha256:"):
        raise SystemExit(f"missing sha256 digest for {archive_name}")

    print(
        "\t".join(
            [
                target,
                archive_name,
                archive["url"],
                digest.split(":", 1)[1],
                checksum_name,
                checksum["url"],
            ]
        )
    )
PY
)"

while IFS=$'\t' read -r target archive_name archive_url archive_sha checksum_name checksum_url; do
  expected_archive_url="$(sfa_archive_url_for_tag "$release_tag" "$target")"
  expected_checksum_url="$(sfa_checksum_url_for_tag "$release_tag" "$target")"

  [[ "$archive_url" == "$expected_archive_url" ]] || {
    printf 'archive URL mismatch for %s\nexpected: %s\nactual:   %s\n' "$target" "$expected_archive_url" "$archive_url" >&2
    exit 1
  }

  [[ "$checksum_url" == "$expected_checksum_url" ]] || {
    printf 'checksum URL mismatch for %s\nexpected: %s\nactual:   %s\n' "$target" "$expected_checksum_url" "$checksum_url" >&2
    exit 1
  }

  checksum_path="${tmp_dir}/${checksum_name}"
  sfa_download_to_file "$checksum_url" "$checksum_path"

  checksum_line="$(tr -d '\r' < "$checksum_path" | sed -n '1p')"
  checksum_sha="$(printf '%s\n' "$checksum_line" | awk '{print $1}')"
  checksum_target="$(printf '%s\n' "$checksum_line" | awk '{print $2}')"

  [[ -n "$checksum_sha" ]] || {
    printf 'checksum asset %s did not contain a SHA-256 line\n' "$checksum_name" >&2
    exit 1
  }

  [[ "$checksum_sha" == "$archive_sha" ]] || {
    printf 'checksum mismatch for %s\nexpected: %s\nactual:   %s\n' "$archive_name" "$archive_sha" "$checksum_sha" >&2
    exit 1
  }

  [[ "$checksum_target" == "$archive_name" ]] || {
    printf 'checksum file %s references %s instead of %s\n' "$checksum_name" "$checksum_target" "$archive_name" >&2
    exit 1
  }
done <<<"$asset_rows"

printf 'Validated distribution metadata for %s\n' "$release_tag"
