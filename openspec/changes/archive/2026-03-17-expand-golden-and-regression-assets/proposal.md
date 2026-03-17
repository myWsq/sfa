## Why

SFA v1 has already frozen the wire format and landed the main pack/unpack and benchmark paths, but M1 is still open because the regression assets are too thin for release-grade confidence. The repository currently relies on a narrow golden fixture set and limited CLI behavior coverage, which leaves default values, usage errors, stdin combinations, and supported Unix entry semantics under-specified in automated checks.

## What Changes

- Expand the canonical golden fixture corpus under `tests/fixtures/golden/` to cover the frozen v1 codec and integrity combinations, multi-bundle layout, and supported Unix entry semantics already in scope for v1.
- Add CLI regression coverage for common defaults and supported behavior combinations, including missing-path failures, usage errors, `stdin` and `--dry-run` interactions, JSON stats output, and overwrite-related restore behavior.
- Tighten regression guidance so protocol smoke, CLI tests, and fixture documentation describe the required committed assets and when they must be refreshed.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `archive-format-v1`: strengthen the committed golden fixture requirements from a minimal freeze corpus to a representative canonical corpus that exercises the frozen v1 protocol surface.
- `cli-and-benchmarks`: require broader automated CLI behavior regressions and fixture-backed verification for common defaults, failure modes, and supported `stdin` combinations before the first v1 release.

## Impact

- OpenSpec capability deltas for `archive-format-v1` and `cli-and-benchmarks`
- Golden fixture assets and regeneration/documentation under `tests/fixtures/golden/`
- CLI regression tests under `crates/sfa-cli/tests/` and related verification entrypoints
- Repository verification guidance in test and release-facing documentation
