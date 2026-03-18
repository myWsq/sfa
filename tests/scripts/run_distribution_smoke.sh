#!/usr/bin/env bash
set -euo pipefail

export LC_ALL=C
export LANG=C

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
TMP_DIR="$(mktemp -d)"

server_pid=""
cleanup() {
  if [[ -n "$server_pid" ]] && kill -0 "$server_pid" >/dev/null 2>&1; then
    kill "$server_pid" >/dev/null 2>&1 || true
    wait "$server_pid" >/dev/null 2>&1 || true
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

compute_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

write_asset() {
  local target="$1"
  local version="$2"
  local release_dir="$3"
  local archive_name="sfa-${version}-${target}.tar.gz"
  local package_dir="$TMP_DIR/${archive_name%.tar.gz}"
  local archive_path="$release_dir/$archive_name"
  local checksum_path="${archive_path}.sha256"
  local digest

  mkdir -p "$package_dir"
  cat >"$package_dir/sfa-cli" <<EOF
#!/usr/bin/env sh
printf 'sfa-cli %s\n' "${version#v}"
EOF
  chmod +x "$package_dir/sfa-cli"
  printf 'readme\n' >"$package_dir/README.md"
  printf 'license\n' >"$package_dir/LICENSE"

  tar -C "$TMP_DIR" -czf "$archive_path" "$(basename "$package_dir")"
  digest="$(compute_sha256 "$archive_path")"
  printf '%s  %s\n' "$digest" "$archive_name" >"$checksum_path"
  printf '%s\n' "$digest"
}

RELEASE_TAG="v9.9.9"
RELEASE_DIR="$TMP_DIR/releases/download/$RELEASE_TAG"
API_DIR="$TMP_DIR/api/repos/myWsq/sfa/releases"
mkdir -p "$RELEASE_DIR" "$API_DIR"

linux_sha="$(write_asset "x86_64-unknown-linux-gnu" "$RELEASE_TAG" "$RELEASE_DIR")"
mac_intel_sha="$(write_asset "x86_64-apple-darwin" "$RELEASE_TAG" "$RELEASE_DIR")"
mac_arm_sha="$(write_asset "aarch64-apple-darwin" "$RELEASE_TAG" "$RELEASE_DIR")"

cat >"$API_DIR/latest" <<EOF
{"tag_name":"$RELEASE_TAG"}
EOF

python3 - "$TMP_DIR" "$TMP_DIR/http.port" >"$TMP_DIR/http.log" 2>&1 <<'PY' &
import functools
import http.server
import sys

directory, port_file = sys.argv[1], sys.argv[2]
handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=directory)
server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)

with open(port_file, "w", encoding="utf-8") as fh:
    fh.write(str(server.server_address[1]))

print(f"serving {server.server_address[1]}", flush=True)
server.serve_forever()
PY
server_pid="$!"

for _ in $(seq 1 50); do
  if [[ -s "$TMP_DIR/http.port" ]]; then
    break
  fi
  sleep 0.1
done

port="$(cat "$TMP_DIR/http.port")"
base_url="http://127.0.0.1:${port}"
release_base_url="${base_url}/releases"
releases_api_url="${base_url}/api/repos/myWsq/sfa/releases/latest"

resolved_target="$(sh "$ROOT_DIR/install.sh" --resolve-target Darwin arm64)"
[[ "$resolved_target" == "aarch64-apple-darwin" ]] || {
  printf 'expected Darwin arm64 to resolve to aarch64-apple-darwin, got %s\n' "$resolved_target" >&2
  exit 1
}

resolved_release_tag="$(SFA_INSTALL_RELEASES_API="$releases_api_url" sh "$ROOT_DIR/install.sh" --resolve-release-tag latest)"
[[ "$resolved_release_tag" == "$RELEASE_TAG" ]] || {
  printf 'expected latest release tag to resolve to %s, got %s\n' "$RELEASE_TAG" "$resolved_release_tag" >&2
  exit 1
}

archive_url="$(SFA_INSTALL_BASE_URL="$release_base_url" SFA_INSTALL_RELEASES_API="$releases_api_url" sh "$ROOT_DIR/install.sh" --print-archive-url "$RELEASE_TAG" x86_64-unknown-linux-gnu)"
[[ "$archive_url" == "${release_base_url}/download/${RELEASE_TAG}/sfa-${RELEASE_TAG}-x86_64-unknown-linux-gnu.tar.gz" ]] || {
  printf 'unexpected archive URL: %s\n' "$archive_url" >&2
  exit 1
}

INSTALL_BIN_DIR="$TMP_DIR/bin"
SFA_INSTALL_BASE_URL="$release_base_url" \
SFA_INSTALL_RELEASES_API="$releases_api_url" \
SFA_INSTALL_UNAME_S="Linux" \
SFA_INSTALL_UNAME_M="x86_64" \
sh "$ROOT_DIR/install.sh" --version "$RELEASE_TAG" --bin-dir "$INSTALL_BIN_DIR"

installed_version="$("$INSTALL_BIN_DIR/sfa-cli")"
[[ "$installed_version" == "sfa-cli 9.9.9" ]] || {
  printf 'unexpected installed version output: %s\n' "$installed_version" >&2
  exit 1
}

if SFA_INSTALL_BASE_URL="$release_base_url" \
  SFA_INSTALL_RELEASES_API="$releases_api_url" \
  SFA_INSTALL_UNAME_S="Linux" \
  SFA_INSTALL_UNAME_M="aarch64" \
  sh "$ROOT_DIR/install.sh" --version "$RELEASE_TAG" --bin-dir "$TMP_DIR/unsupported" >"$TMP_DIR/unsupported.out" 2>"$TMP_DIR/unsupported.err"; then
  printf 'unsupported-host installation unexpectedly succeeded\n' >&2
  exit 1
fi

grep -q "unsupported host" "$TMP_DIR/unsupported.err" || {
  printf 'unsupported-host failure did not mention unsupported host\n' >&2
  exit 1
}

printf 'bad checksum  sfa-%s-x86_64-unknown-linux-gnu.tar.gz\n' "$RELEASE_TAG" >"$RELEASE_DIR/sfa-${RELEASE_TAG}-x86_64-unknown-linux-gnu.tar.gz.sha256"
if SFA_INSTALL_BASE_URL="$release_base_url" \
  SFA_INSTALL_RELEASES_API="$releases_api_url" \
  SFA_INSTALL_UNAME_S="Linux" \
  SFA_INSTALL_UNAME_M="x86_64" \
  sh "$ROOT_DIR/install.sh" --version "$RELEASE_TAG" --bin-dir "$TMP_DIR/bad-checksum" >"$TMP_DIR/checksum.out" 2>"$TMP_DIR/checksum.err"; then
  printf 'checksum-mismatch installation unexpectedly succeeded\n' >&2
  exit 1
fi

grep -q "checksum verification failed" "$TMP_DIR/checksum.err" || {
  printf 'checksum failure did not mention checksum verification\n' >&2
  exit 1
}

cat >"$TMP_DIR/release-assets.json" <<EOF
{
  "assets": [
    {
      "name": "sfa-${RELEASE_TAG}-aarch64-apple-darwin.tar.gz",
      "url": "${release_base_url}/download/${RELEASE_TAG}/sfa-${RELEASE_TAG}-aarch64-apple-darwin.tar.gz",
      "digest": "sha256:${mac_arm_sha}"
    },
    {
      "name": "sfa-${RELEASE_TAG}-aarch64-apple-darwin.tar.gz.sha256",
      "url": "${release_base_url}/download/${RELEASE_TAG}/sfa-${RELEASE_TAG}-aarch64-apple-darwin.tar.gz.sha256",
      "digest": "sha256:unused"
    },
    {
      "name": "sfa-${RELEASE_TAG}-x86_64-apple-darwin.tar.gz",
      "url": "${release_base_url}/download/${RELEASE_TAG}/sfa-${RELEASE_TAG}-x86_64-apple-darwin.tar.gz",
      "digest": "sha256:${mac_intel_sha}"
    },
    {
      "name": "sfa-${RELEASE_TAG}-x86_64-apple-darwin.tar.gz.sha256",
      "url": "${release_base_url}/download/${RELEASE_TAG}/sfa-${RELEASE_TAG}-x86_64-apple-darwin.tar.gz.sha256",
      "digest": "sha256:unused"
    },
    {
      "name": "sfa-${RELEASE_TAG}-x86_64-unknown-linux-gnu.tar.gz",
      "url": "${release_base_url}/download/${RELEASE_TAG}/sfa-${RELEASE_TAG}-x86_64-unknown-linux-gnu.tar.gz",
      "digest": "sha256:${linux_sha}"
    },
    {
      "name": "sfa-${RELEASE_TAG}-x86_64-unknown-linux-gnu.tar.gz.sha256",
      "url": "${release_base_url}/download/${RELEASE_TAG}/sfa-${RELEASE_TAG}-x86_64-unknown-linux-gnu.tar.gz.sha256",
      "digest": "sha256:unused"
    }
  ]
}
EOF

printf '%s  sfa-%s-x86_64-unknown-linux-gnu.tar.gz\n' "$linux_sha" "$RELEASE_TAG" >"$RELEASE_DIR/sfa-${RELEASE_TAG}-x86_64-unknown-linux-gnu.tar.gz.sha256"

SFA_INSTALL_BASE_URL="$release_base_url" \
  bash "$ROOT_DIR/scripts/release/validate_distribution_assets.sh" \
  --release-tag "$RELEASE_TAG" \
  --assets-json "$TMP_DIR/release-assets.json"

FORMULA_PATH="$TMP_DIR/sfa-cli.rb"
bash "$ROOT_DIR/scripts/release/generate_homebrew_formula.sh" \
  --release-tag "$RELEASE_TAG" \
  --assets-json "$TMP_DIR/release-assets.json" \
  --output "$FORMULA_PATH"

grep -q 'class SfaCli < Formula' "$FORMULA_PATH" || {
  printf 'formula generation did not emit the expected class name\n' >&2
  exit 1
}

grep -q "$linux_sha" "$FORMULA_PATH" || {
  printf 'formula generation did not include the Linux SHA-256 digest\n' >&2
  exit 1
}

grep -q "${release_base_url}/download/${RELEASE_TAG}/sfa-${RELEASE_TAG}-aarch64-apple-darwin.tar.gz" "$FORMULA_PATH" || {
  printf 'formula generation did not include the macOS arm64 asset URL\n' >&2
  exit 1
}

printf '[test:distribution] ok\n'
