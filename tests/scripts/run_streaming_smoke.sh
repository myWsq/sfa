#!/usr/bin/env bash
set -euo pipefail

CARGO_HOME="${CARGO_HOME:-/tmp/cargo-home}" cargo test -p sfa-unixfs archive::tests::fragmented_reader_header_roundtrip
echo "[test:streaming] ok"
