#!/bin/sh

sfa_binary_name() {
  printf '%s\n' 'sfa'
}

sfa_default_repo() {
  printf '%s\n' "${SFA_INSTALL_REPO:-myWsq/sfa}"
}

sfa_default_tap_repo() {
  repo="$(sfa_default_repo)"
  owner="${repo%%/*}"
  printf '%s/homebrew-sfa\n' "$owner"
}

sfa_default_bin_dir() {
  if [ -n "${SFA_INSTALL_BIN_DIR:-}" ]; then
    printf '%s\n' "${SFA_INSTALL_BIN_DIR}"
  elif [ -n "${HOME:-}" ]; then
    printf '%s\n' "${HOME}/.local/bin"
  else
    printf '%s\n' '/usr/local/bin'
  fi
}

sfa_fail() {
  printf 'sfa installer: %s\n' "$1" >&2
  exit 1
}

sfa_trim_trailing_slash() {
  value="$1"
  case "$value" in
    */) printf '%s\n' "${value%/}" ;;
    *) printf '%s\n' "$value" ;;
  esac
}

sfa_normalize_release_tag() {
  version="${1:-latest}"
  case "$version" in
    ''|latest)
      printf '%s\n' 'latest'
      ;;
    v*)
      printf '%s\n' "$version"
      ;;
    *)
      printf 'v%s\n' "$version"
      ;;
  esac
}

sfa_version_without_v() {
  version="$1"
  case "$version" in
    v*)
      printf '%s\n' "${version#v}"
      ;;
    *)
      printf '%s\n' "$version"
      ;;
  esac
}

sfa_supported_targets() {
  cat <<'EOF'
Darwin	x86_64	x86_64-apple-darwin
Darwin	arm64	aarch64-apple-darwin
Darwin	aarch64	aarch64-apple-darwin
Linux	x86_64	x86_64-unknown-linux-gnu
Linux	amd64	x86_64-unknown-linux-gnu
EOF
}

sfa_release_targets() {
  cat <<'EOF'
x86_64-unknown-linux-gnu
x86_64-apple-darwin
aarch64-apple-darwin
EOF
}

sfa_resolve_host_target() {
  os_name="$1"
  arch_name="$2"

  case "${os_name}:${arch_name}" in
    Darwin:x86_64)
      printf '%s\n' 'x86_64-apple-darwin'
      ;;
    Darwin:arm64|Darwin:aarch64)
      printf '%s\n' 'aarch64-apple-darwin'
      ;;
    Linux:x86_64|Linux:amd64)
      printf '%s\n' 'x86_64-unknown-linux-gnu'
      ;;
    *)
      return 1
      ;;
  esac
}

sfa_detect_host_target() {
  os_name="${SFA_INSTALL_UNAME_S:-$(uname -s 2>/dev/null || printf '')}"
  arch_name="${SFA_INSTALL_UNAME_M:-$(uname -m 2>/dev/null || printf '')}"

  target="$(sfa_resolve_host_target "$os_name" "$arch_name")" || return 1
  printf '%s\n' "$target"
}

sfa_archive_name() {
  release_tag="$(sfa_normalize_release_tag "$1")"
  target="$2"

  if [ "$release_tag" = "latest" ]; then
    sfa_fail 'asset names require an explicit release tag'
  fi

  printf 'sfa-%s-%s.tar.gz\n' "$release_tag" "$target"
}

sfa_checksum_name() {
  release_tag="$1"
  target="$2"
  printf '%s.sha256\n' "$(sfa_archive_name "$release_tag" "$target")"
}

sfa_latest_release_api_url() {
  if [ -n "${SFA_INSTALL_RELEASES_API:-}" ]; then
    sfa_trim_trailing_slash "${SFA_INSTALL_RELEASES_API}"
    return 0
  fi

  repo="$(sfa_default_repo)"
  printf 'https://api.github.com/repos/%s/releases/latest\n' "$repo"
}

sfa_release_download_base_for_tag() {
  release_tag="$1"

  if [ -n "${SFA_INSTALL_BASE_URL:-}" ]; then
    base_url="$(sfa_trim_trailing_slash "${SFA_INSTALL_BASE_URL}")"
    printf '%s/download/%s\n' "$base_url" "$release_tag"
    return 0
  fi

  repo="$(sfa_default_repo)"
  printf 'https://github.com/%s/releases/download/%s\n' "$repo" "$release_tag"
}

sfa_archive_url_for_tag() {
  release_tag="$1"
  target="$2"
  base_url="$(sfa_release_download_base_for_tag "$release_tag")"
  printf '%s/%s\n' "$base_url" "$(sfa_archive_name "$release_tag" "$target")"
}

sfa_checksum_url_for_tag() {
  release_tag="$1"
  target="$2"
  base_url="$(sfa_release_download_base_for_tag "$release_tag")"
  printf '%s/%s\n' "$base_url" "$(sfa_checksum_name "$release_tag" "$target")"
}

sfa_has_command() {
  command -v "$1" >/dev/null 2>&1
}

sfa_download_to_stdout() {
  url="$1"

  if sfa_has_command curl; then
    curl -fsSL "$url"
    return 0
  fi

  if sfa_has_command wget; then
    wget -qO- "$url"
    return 0
  fi

  sfa_fail 'missing required download tool: install curl or wget'
}

sfa_download_to_file() {
  url="$1"
  destination="$2"

  if sfa_has_command curl; then
    curl -fsSL "$url" -o "$destination"
    return 0
  fi

  if sfa_has_command wget; then
    wget -qO "$destination" "$url"
    return 0
  fi

  sfa_fail 'missing required download tool: install curl or wget'
}

sfa_extract_latest_release_tag_from_json() {
  tr -d '\n' | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p'
}

sfa_resolve_release_tag() {
  version="$(sfa_normalize_release_tag "${1:-latest}")"

  if [ "$version" != "latest" ]; then
    printf '%s\n' "$version"
    return 0
  fi

  latest_api_url="$(sfa_latest_release_api_url)"
  release_tag="$(sfa_download_to_stdout "$latest_api_url" | sfa_extract_latest_release_tag_from_json)"

  if [ -z "$release_tag" ]; then
    sfa_fail 'failed to resolve the latest release tag; retry with --version vX.Y.Z'
  fi

  printf '%s\n' "$release_tag"
}

sfa_sha_tool() {
  if sfa_has_command sha256sum; then
    printf '%s\n' 'sha256sum'
    return 0
  fi

  if sfa_has_command shasum; then
    printf '%s\n' 'shasum'
    return 0
  fi

  return 1
}

sfa_verify_archive_checksum() {
  archive_path="$1"
  checksum_path="$2"
  checksum_tool="$(sfa_sha_tool)" || sfa_fail 'missing required checksum tool: install sha256sum or shasum'

  checksum_dir="$(dirname "$checksum_path")"
  checksum_file="$(basename "$checksum_path")"

  (
    cd "$checksum_dir"
    case "$checksum_tool" in
      sha256sum)
        sha256sum -c "$checksum_file" >/dev/null
        ;;
      shasum)
        shasum -a 256 -c "$checksum_file" >/dev/null
        ;;
      *)
        exit 1
        ;;
    esac
  ) || sfa_fail "checksum verification failed for $(basename "$archive_path")"
}

sfa_installer_usage() {
  cat <<'EOF'
Usage: sh install.sh [--version vX.Y.Z] [--bin-dir DIR]

Installs sfa from the published GitHub Release archives into a local bin
directory. The default install location is $HOME/.local/bin when HOME is set,
otherwise /usr/local/bin.

Options:
  --version VERSION   Install an explicit release such as v1.0.0 (default: latest)
  --bin-dir DIR       Destination directory for sfa
  --repo OWNER/REPO   Override the GitHub repository slug used for downloads
  --help              Show this message
EOF
}

sfa_install_release() {
  requested_version="$1"
  bin_dir="$2"

  release_tag="$(sfa_resolve_release_tag "$requested_version")"
  target="$(sfa_detect_host_target)" || sfa_fail 'unsupported host; published binaries cover Linux x86_64 plus macOS x86_64 and arm64'
  archive_name="$(sfa_archive_name "$release_tag" "$target")"
  checksum_name="$(sfa_checksum_name "$release_tag" "$target")"
  archive_url="$(sfa_archive_url_for_tag "$release_tag" "$target")"
  checksum_url="$(sfa_checksum_url_for_tag "$release_tag" "$target")"
  tmp_dir="$(mktemp -d 2>/dev/null || mktemp -d -t sfa-install)"
  archive_path="${tmp_dir}/${archive_name}"
  checksum_path="${tmp_dir}/${checksum_name}"
  extract_dir="${tmp_dir}/extract"
  installed_path="${bin_dir}/$(sfa_binary_name)"

  trap 'rm -rf "$tmp_dir"' 0 INT HUP TERM

  mkdir -p "$extract_dir"

  sfa_download_to_file "$archive_url" "$archive_path" || sfa_fail "failed to download ${archive_name}"
  sfa_download_to_file "$checksum_url" "$checksum_path" || sfa_fail "failed to download ${checksum_name}"
  sfa_verify_archive_checksum "$archive_path" "$checksum_path"

  tar -xzf "$archive_path" -C "$extract_dir" || sfa_fail "failed to extract ${archive_name}"

  unpacked_binary="$(find "$extract_dir" -type f -name "$(sfa_binary_name)" 2>/dev/null | sed -n '1p')"
  if [ -z "$unpacked_binary" ]; then
    sfa_fail "downloaded archive did not contain $(sfa_binary_name)"
  fi

  mkdir -p "$bin_dir" || sfa_fail "failed to create install directory ${bin_dir}"
  cp "$unpacked_binary" "$installed_path" || sfa_fail "failed to copy $(sfa_binary_name) into ${bin_dir}"
  chmod 755 "$installed_path" || sfa_fail "failed to mark ${installed_path} as executable"

  reported_version="$("$installed_path" --version 2>/dev/null | sed -n '1p')"
  if [ -z "$reported_version" ]; then
    reported_version="$(sfa_binary_name) ${release_tag}"
  fi

  printf 'Installed %s to %s\n' "$reported_version" "$installed_path"

  case ":${PATH:-}:" in
    *:"$bin_dir":*)
      ;;
    *)
      printf 'Add %s to PATH if it is not already exported in your shell profile.\n' "$bin_dir"
      ;;
  esac
}

sfa_main() {
  requested_version='latest'
  bin_dir="$(sfa_default_bin_dir)"

  while [ "$#" -gt 0 ]; do
    case "$1" in
      --version)
        [ "$#" -ge 2 ] || sfa_fail 'missing value for --version'
        requested_version="$2"
        shift 2
        ;;
      --bin-dir)
        [ "$#" -ge 2 ] || sfa_fail 'missing value for --bin-dir'
        bin_dir="$2"
        shift 2
        ;;
      --repo)
        [ "$#" -ge 2 ] || sfa_fail 'missing value for --repo'
        SFA_INSTALL_REPO="$2"
        export SFA_INSTALL_REPO
        shift 2
        ;;
      --print-target-matrix)
        sfa_supported_targets
        exit 0
        ;;
      --print-release-targets)
        sfa_release_targets
        exit 0
        ;;
      --resolve-target)
        [ "$#" -ge 3 ] || sfa_fail 'usage: --resolve-target <os> <arch>'
        target="$(sfa_resolve_host_target "$2" "$3")" || sfa_fail 'unsupported host for release archives'
        printf '%s\n' "$target"
        exit 0
        ;;
      --resolve-release-tag)
        [ "$#" -ge 2 ] || sfa_fail 'usage: --resolve-release-tag <version>'
        sfa_resolve_release_tag "$2"
        exit 0
        ;;
      --print-archive-name)
        [ "$#" -ge 3 ] || sfa_fail 'usage: --print-archive-name <release-tag> <target>'
        printf '%s\n' "$(sfa_archive_name "$2" "$3")"
        exit 0
        ;;
      --print-checksum-name)
        [ "$#" -ge 3 ] || sfa_fail 'usage: --print-checksum-name <release-tag> <target>'
        printf '%s\n' "$(sfa_checksum_name "$2" "$3")"
        exit 0
        ;;
      --print-archive-url)
        [ "$#" -ge 3 ] || sfa_fail 'usage: --print-archive-url <version> <target>'
        resolved_tag="$(sfa_resolve_release_tag "$2")"
        printf '%s\n' "$(sfa_archive_url_for_tag "$resolved_tag" "$3")"
        exit 0
        ;;
      --print-checksum-url)
        [ "$#" -ge 3 ] || sfa_fail 'usage: --print-checksum-url <version> <target>'
        resolved_tag="$(sfa_resolve_release_tag "$2")"
        printf '%s\n' "$(sfa_checksum_url_for_tag "$resolved_tag" "$3")"
        exit 0
        ;;
      --print-binary-name)
        sfa_binary_name
        exit 0
        ;;
      --print-tap-repository)
        sfa_default_tap_repo
        exit 0
        ;;
      --help|-h)
        sfa_installer_usage
        exit 0
        ;;
      *)
        sfa_fail "unknown argument: $1"
        ;;
    esac
  done

  sfa_install_release "$requested_version" "$bin_dir"
}

if [ "${SFA_INSTALLER_SOURCE_ONLY:-0}" != '1' ]; then
  sfa_main "$@"
fi
