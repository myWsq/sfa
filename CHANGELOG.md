# Changelog

This file records repository-level changes for SFA. The project is still in the SFA v1 development cycle, with `format-v1` frozen and any compatibility-sensitive follow-up managed through OpenSpec changes.

## [Unreleased]

## [0.2.0] - 2026-03-17

### Added

- Frozen `format-v1` protocol text together with canonical golden fixtures, a protocol freeze review record, and protocol smoke baselines
- Committed benchmark datasets, a machine-readable `tar + same codec` baseline, and SFA phase-level / resource-level benchmark observations
- Synchronous `Read` and `stdin` unpack entry points, real bundle-level unpack worker scheduling, a `dirfd` / `openat`-style restore path, and `.sfa-untrusted` marking on `strong` trailer verification failure
- CLI regression coverage for defaults, usage errors, `stdin` / `--dry-run`, and overwrite semantics
- A `release-readiness` OpenSpec capability together with a release candidate notes draft

### Changed

- Repository status to reflect that M1 is complete, M2 is complete, and the next focus area is M3 Unix semantics hardening
- The release checklist to explicitly treat `cargo fmt --all --check`, workspace tests, smoke checks, and the benchmark dry run as the authoritative pre-release gates
- `README.md`, `ROADMAP.md`, `RELEASING.md`, and version metadata to keep release documentation, milestone state, and repository behavior aligned

## [0.1.0] - 2026-03-16

### Added

- The initial Rust workspace with `sfa-core`, `sfa-unixfs`, `sfa-cli`, and `sfa-bench`
- The `sfa pack` / `sfa unpack` MVP mainline, including `lz4` / `zstd` codec support and `fast` / `strong` integrity modes
- A manifest-first `.sfa` layout with header, manifest, frames, and an optional trailer
- Scan, pack, and restore support for regular files, directories, symlinks, and hardlinks
- Workspace tests, protocol / streaming / safety / roundtrip smoke checks, and benchmark dry-run CI coverage

### Changed

- `README.md`, `RELEASING.md`, and `ROADMAP.md` to define project status, release flow, and milestone structure
- Benchmark harness, fixture layout, and verification baselines as the foundation for protocol freeze and later baseline hardening
