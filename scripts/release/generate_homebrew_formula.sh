#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
SFA_INSTALLER_SOURCE_ONLY=1 . "$ROOT_DIR/install.sh"

usage() {
  cat <<'EOF'
Usage: bash scripts/release/generate_homebrew_formula.sh \
  --release-tag vX.Y.Z \
  --assets-json /path/to/release-assets.json \
  --output /path/to/sfa-cli.rb
EOF
}

release_tag=""
assets_json=""
output_path=""

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
    --output)
      output_path="${2:-}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      printf 'generate_homebrew_formula: unknown argument: %s\n' "$1" >&2
      exit 1
      ;;
  esac
done

[[ -n "$release_tag" ]] || { usage >&2; exit 1; }
[[ -n "$assets_json" ]] || { usage >&2; exit 1; }
[[ -n "$output_path" ]] || { usage >&2; exit 1; }
[[ -f "$assets_json" ]] || { printf 'missing assets JSON: %s\n' "$assets_json" >&2; exit 1; }

release_tag="$(sfa_normalize_release_tag "$release_tag")"
version="${release_tag#v}"
repo_slug="${SFA_INSTALL_REPO:-$(sfa_default_repo)}"

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
    archive = by_name.get(archive_name)
    if archive is None:
        raise SystemExit(f"missing release asset: {archive_name}")
    digest = archive.get("digest", "")
    if not digest.startswith("sha256:"):
        raise SystemExit(f"missing sha256 digest for {archive_name}")
    sha_value = digest.split(":", 1)[1]
    print("\t".join([target, archive["url"], sha_value]))
PY
)"

linux_url=""
linux_sha=""
mac_intel_url=""
mac_intel_sha=""
mac_arm_url=""
mac_arm_sha=""

while IFS=$'\t' read -r target asset_url sha_value; do
  case "$target" in
    x86_64-unknown-linux-gnu)
      linux_url="$asset_url"
      linux_sha="$sha_value"
      ;;
    x86_64-apple-darwin)
      mac_intel_url="$asset_url"
      mac_intel_sha="$sha_value"
      ;;
    aarch64-apple-darwin)
      mac_arm_url="$asset_url"
      mac_arm_sha="$sha_value"
      ;;
    *)
      printf 'unexpected target in release asset metadata: %s\n' "$target" >&2
      exit 1
      ;;
  esac
done <<<"$asset_rows"

mkdir -p "$(dirname "$output_path")"

cat >"$output_path" <<EOF
class SfaCli < Formula
  desc "Small File Archive CLI for Unix directory trees with many small files"
  homepage "https://github.com/${repo_slug}"
  version "${version}"
  license "MIT"

  on_macos do
    on_arm do
      url "${mac_arm_url}"
      sha256 "${mac_arm_sha}"
    end

    on_intel do
      url "${mac_intel_url}"
      sha256 "${mac_intel_sha}"
    end
  end

  on_linux do
    depends_on arch: :x86_64
    url "${linux_url}"
    sha256 "${linux_sha}"
  end

  def install
    bin.install "sfa-cli"
    prefix.install "README.md", "LICENSE"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/sfa-cli --version")
  end
end
EOF

printf 'Generated Homebrew formula at %s\n' "$output_path"
