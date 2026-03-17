#!/usr/bin/env bash
set -euo pipefail

CARGO_HOME="${CARGO_HOME:-/tmp/cargo-home}" cargo test -p sfa-unixfs archive::tests::roundtrip_pack_and_unpack
CARGO_HOME="${CARGO_HOME:-/tmp/cargo-home}" cargo test -p sfa-unixfs archive::tests::roundtrip_restores_mode_and_mtime_for_files_and_directories
echo "[test:roundtrip] ok"
