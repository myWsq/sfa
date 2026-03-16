#!/usr/bin/env bash
set -euo pipefail

CARGO_HOME="${CARGO_HOME:-/tmp/cargo-home}" cargo test -p sfa-unixfs archive::tests::roundtrip_pack_and_unpack
echo "[test:roundtrip] ok"
