#!/usr/bin/env bash
set -euo pipefail

CARGO_HOME="${CARGO_HOME:-/tmp/cargo-home}" cargo test -p sfa-unixfs restore::tests::restorer_blocks_symlink_traversal
echo "[test:safety] ok"
