## Why

SFA v1 now has the main implementation, regression corpus, and verification entrypoints needed for a minimal usable release, but the repository still does not present that state cleanly. M1 remains marked in progress, release-facing documents are out of sync with the codebase, and the documented release gate is not yet being enforced end to end.

## What Changes

- Close out the remaining M1 release-readiness work by making the repository-level quality gates, milestone state, and release-facing documentation consistent with the current implementation.
- Fix the current formatting gate failure in the CLI regression tests and treat the documented release verification commands as the authoritative pre-release checklist.
- Update roadmap, changelog, and release guidance so the repository records that the expanded golden corpus and CLI regression coverage are now part of the M1 baseline.
- Define the repository requirements for release readiness and milestone closeout so future release-prep work is traceable in OpenSpec rather than living only in ad hoc docs.

## Capabilities

### New Capabilities

- `release-readiness`: defines the repository-level requirements for quality gates, milestone closeout, and release artifact synchronization before an SFA version is tagged.

### Modified Capabilities

- `cli-and-benchmarks`: tighten the verification contract so the documented release gate covers formatting, full workspace tests, smoke entrypoints, and benchmark dry-run in one auditable checklist.

## Impact

- Release-facing repository docs including `ROADMAP.md`, `CHANGELOG.md`, `README.md`, and `RELEASING.md`
- CLI regression tests and repository formatting compliance
- OpenSpec capability deltas for `cli-and-benchmarks` and the new `release-readiness` capability
- Versioning and milestone bookkeeping associated with the next release candidate
