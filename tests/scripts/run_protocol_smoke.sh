#!/usr/bin/env bash
set -euo pipefail

CARGO_HOME="${CARGO_HOME:-/tmp/cargo-home}" cargo test -p sfa-core
echo "[test:protocol] ok"
