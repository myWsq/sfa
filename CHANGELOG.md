# Changelog

This file records repository-level changes for SFA. The repository is preparing its first stable SFA v1 release, with `format-v1` frozen and any compatibility-sensitive follow-up managed through OpenSpec changes.

## [Unreleased]

## [1.0.0] - 2026-03-17

### Added

- Add the first stable `v1.0.0` release notes draft and extend repository release-readiness requirements to cover M3 closeout and first-stable-release preparation

### Changed

- Promote the current frozen `format-v1` surface to the first stable release target `1.0.0`
- Close the current M3 Unix metadata hardening slice in roadmap, README, and technical-solution docs while deferring xattrs, ACLs, special-file restore, and broader Unix extensions to post-v1 work
- Parallelize unpack directory setup before worker execution on the selected stable-release candidate revision
- Update release guidance so the selected `v1.0.0` candidate includes the post-`v0.3.0` unpack setup optimization and requires a refreshed committed benchmark baseline

### Fixed

- Restore repository formatting compliance so the authoritative release checklist is green on the `v1.0.0` candidate revision

## [0.3.0] - 2026-03-17

### Added

- Add additive unpack `wall_breakdown` metrics alongside diagnostic `phase_breakdown` stats in machine-readable CLI and benchmark outputs

### Changed

- Change the default `sfa pack` data codec to `zstd` at level `-3` when `--codec` is not provided
- Reduce small-file unpack setup overhead by tightening restore-path setup before worker scatter begins
- Refresh the committed benchmark baseline, benchmark documentation, and release guidance to describe the split unpack timing model

### Fixed

- Use the correct macOS Intel runner label in the GitHub release workflow so tagged release builds can complete on all intended platforms

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
