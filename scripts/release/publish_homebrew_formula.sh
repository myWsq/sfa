#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
SFA_INSTALLER_SOURCE_ONLY=1 . "$ROOT_DIR/install.sh"

usage() {
  cat <<'EOF'
Usage: HOMEBREW_TAP_GITHUB_TOKEN=... bash scripts/release/publish_homebrew_formula.sh \
  --formula /path/to/sfa.rb \
  --release-tag vX.Y.Z \
  [--tap-repo owner/homebrew-sfa]
EOF
}

formula_path=""
release_tag=""
tap_repo="${HOMEBREW_TAP_REPOSITORY:-$(sfa_default_tap_repo)}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --formula)
      formula_path="${2:-}"
      shift 2
      ;;
    --release-tag)
      release_tag="${2:-}"
      shift 2
      ;;
    --tap-repo)
      tap_repo="${2:-}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      printf 'publish_homebrew_formula: unknown argument: %s\n' "$1" >&2
      exit 1
      ;;
  esac
done

[[ -n "$formula_path" ]] || { usage >&2; exit 1; }
[[ -n "$release_tag" ]] || { usage >&2; exit 1; }
[[ -f "$formula_path" ]] || { printf 'missing formula file: %s\n' "$formula_path" >&2; exit 1; }
[[ -n "${HOMEBREW_TAP_GITHUB_TOKEN:-}" ]] || {
  printf 'publish_homebrew_formula: HOMEBREW_TAP_GITHUB_TOKEN is required\n' >&2
  exit 1
}

release_tag="$(sfa_normalize_release_tag "$release_tag")"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

clone_url="https://x-access-token:${HOMEBREW_TAP_GITHUB_TOKEN}@github.com/${tap_repo}.git"
tap_checkout="${tmp_dir}/tap"

git clone "$clone_url" "$tap_checkout"
mkdir -p "$tap_checkout/Formula"
cp "$formula_path" "$tap_checkout/Formula/sfa.rb"
rm -f "$tap_checkout/Formula/sfa-cli.rb"

pushd "$tap_checkout" >/dev/null
git config user.name "${GIT_AUTHOR_NAME:-github-actions[bot]}"
git config user.email "${GIT_AUTHOR_EMAIL:-41898282+github-actions[bot]@users.noreply.github.com}"

git add -A -- Formula

if git diff --cached --quiet -- Formula/sfa.rb Formula/sfa-cli.rb; then
  printf 'Homebrew tap %s already matches %s\n' "$tap_repo" "$release_tag"
  popd >/dev/null
  exit 0
fi

git commit -m "Update sfa formula for ${release_tag}"
git push origin HEAD
popd >/dev/null

printf 'Published Homebrew formula for %s to %s\n' "$release_tag" "$tap_repo"
