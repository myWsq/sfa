# SFA v1 Roadmap

This document describes the current development stages, project status, and near-term priorities for SFA v1. Unless explicitly stated otherwise, it does not represent a release-date commitment.

Last updated: 2026-03-17

## Current Status

SFA v1 is in stable-release preparation. The repository already contains a runnable minimal usable chain, a frozen protocol definition, completed M0 through M3 milestones for the current stable-v1 scope, and a candidate `main` revision that extends the `v0.3.0` line with additional unpack setup optimization work.

The repository currently provides:

- A Rust workspace with `sfa-core`, `sfa-unixfs`, `sfa-cli`, and `sfa-bench`
- Runnable end-to-end `sfa pack` and `sfa unpack` workflows
- A manifest-first `.sfa` layout with header, manifest, frames, and an optional trailer
- `lz4` and `zstd` data codec support
- Deterministic directory scanning, stable bundle planning, and sequential read/write paths
- Pack and restore support for regular files, directories, symlinks, and hardlinks
- Default restore of `mode` and `mtime` for regular files and directories, with archive-side recording of `uid` and `gid`
- Explicit opt-in owner restore behavior, still constrained by effective root privileges
- Roundtrip, streaming, corruption, and safety test scaffolding together with a benchmark harness
- Phase-level benchmark timing and, in supported environments, CPU / RSS resource observations
- Machine-readable `sfa pack` / `sfa unpack` stats for benchmarks and automation
- Real bundle-level unpack worker scheduling with separate `frame_read`, `decode`, and `scatter` observations
- `sfa unpack -` support for reading from `stdin`, plus a library entry point for unpacking from a synchronous `Read`
- A restore path built around `dirfd` / `openat`-style safe I/O and `.sfa-untrusted` emission on `strong` trailer verification failure
- Expanded canonical golden fixtures and CLI regression coverage as repository-default verification baselines

The current release train is focused on:

- Preparing the first stable `v1.0.0` release from the current `main` revision
- Refreshing the committed benchmark baseline and re-running the authoritative release checklist because the candidate includes benchmark-affecting unpack setup changes beyond `v0.3.0`
- Keeping xattrs, ACLs, special files, and broader Unix extensions explicitly deferred to post-v1 work

## v1 Goals

SFA v1 aims to provide a local archiving format and toolchain with sequential-read semantics for `.sfa` archives, with emphasis on:

- A stable and verifiable archive format definition
- Reliable pack and restore behavior for Unix file trees
- Comparable baselines across mainstream codecs
- Reproducible verification, regression, and performance workflows

The current version does not attempt to cover all Unix extended semantics in one step. Those areas continue to be staged by milestone.

## Milestone Overview

| Milestone | Name | Status | Goal |
|---|---|---|---|
| M0 | Protocol freeze | Complete | Freeze the v1 protocol text, commit the first golden fixtures, and record the review outcome |
| M1 | Minimal usable chain | Complete | Close the MVP into a stable, regression-friendly, CI-ready minimal release candidate |
| M2 | Performance mainline | Complete | Establish real benchmark datasets and `tar + same codec` baselines with phase-level and resource-level observations |
| M3 | Unix semantics hardening | Complete | Stabilize the current metadata contract for the stable v1 boundary and move broader Unix extensions into post-v1 follow-up |
| M4 | Post-v1 Unix extensions | Not started | Evaluate xattrs, ACLs, special files, and other broader Unix extensions without changing the frozen v1 contract |

Status meanings:

- `Not started`: implementation and repository assets have not meaningfully begun
- `In progress`: implementation exists, but closure criteria are not yet satisfied
- `Complete`: the milestone closure criteria have been met

## Milestone Details

### M0: Protocol Freeze

Status: `Complete`

Delivered:

- `spec/format-v1.md` is the authoritative protocol definition
- The first canonical golden fixtures were committed under `tests/fixtures/golden/`
- `spec/format-v1-freeze-review.md` records inputs, conclusions, and deferred items from the freeze review
- Protocol smoke checks consume golden fixture metadata

Closure result:

- The v1 protocol compatibility boundary is fixed
- Golden archives, manifest dumps, and stats summaries are committed
- Protocol review outcomes are repository-traceable

### M1: Minimal Usable Chain

Status: `Complete`

Delivered:

- The `pack` / `unpack` MVP runs end to end
- Regular files, directories, symlinks, and hardlinks are supported
- Sequential unpack works without seek-dependent restore behavior
- The CLI is wired to the real implementation
- `stdin` and synchronous `Read` unpack entry points are available
- `.sfa-untrusted` is emitted when `strong` trailer verification fails
- CLI behavior tests cover defaults, usage errors, `stdin` / `--dry-run`, and overwrite semantics
- The canonical golden corpus is expanded and enforced through protocol smoke checks and CI
- The release checklist includes `cargo fmt --all --check`, workspace tests, smoke checks, and the benchmark dry run

Closure criteria:

- Roundtrip behavior is stable for representative directory trees
- The CLI supports routine local usage scenarios
- Golden fixtures are part of the CI baseline

Closure result:

- M1 moved from “features exist” to “repository-level verification is executable”
- The main follow-up focus shifted to Unix semantics hardening rather than extending the minimal chain

### M2: Performance Mainline

Status: `Complete`

Delivered:

- A stable linear bundle planner is implemented
- The ordered writer and multithreaded pack pipeline are implemented
- The benchmark harness is available
- Committed benchmark datasets replaced placeholder corpora
- The first `tar + same codec` machine-readable baseline is recorded in `benches/results/baseline-v0.1.0.json`
- Benchmark reports record SFA phase-level wall-time breakdowns
- The benchmark runner records CPU / RSS resource observations in supported environments

Closure criteria:

- Benchmark datasets are real, stable, and reusable
- `tar` baselines are repeatable
- Performance results are clearly recorded in the repository
- Performance results contain enough observations to support regression analysis

### M3: Unix Semantics Hardening

Status: `Complete`

Delivered:

- Stabilize the current v1 Unix metadata contract, especially the commitment boundary for `mode`, `mtime`, and owner policy
- Add repository-level verification for metadata roundtrips, owner policy behavior, and existing link / safety scenarios
- Keep xattrs and ACLs deferred while aligning roadmap, README, and technical design documents with shipped behavior

Closure criteria:

- The current metadata contract has aligned specs, implementation behavior, and verification assets
- Repository status documents and technical design documents no longer mark delivered behavior as future work
- Any broader metadata extension is split into a dedicated OpenSpec change

Closure result:

- The stable v1 metadata boundary is explicit for `mode`, `mtime`, and owner-policy behavior
- Repository-facing status documents can now treat first-stable-release preparation as the next step instead of continuing to describe M3 as open-ended
- xattrs, ACLs, special files, and broader Unix extensions remain deferred to post-v1 work

### M4: Post-v1 Unix Extensions

Status: `Not started`

Initial scope:

- Evaluate xattrs and ACLs as dedicated post-v1 work items rather than implicit stable-release blockers
- Expand Unix boundary cases such as special files and broader metadata coverage only through dedicated OpenSpec changes
- Decide whether later restore-path and benchmark automation enhancements belong in the same post-v1 milestone or should be split further

Closure criteria:

- Any expanded Unix metadata or special-file surface has dedicated specs, verification assets, and release notes
- Post-v1 extensions do not redefine the frozen `format-v1` compatibility contract for the first stable release line

## Stable Release Train

Current target: `v1.0.0`

Selected candidate revision:

- The current `main` branch after `v0.3.0`
- Includes the post-`v0.3.0` unpack directory setup optimization work before the worker pipeline begins

Current release-train blockers:

- Restore release-gate compliance on the selected candidate revision
- Refresh and validate the committed benchmark baseline because the selected candidate changes benchmark-facing unpack behavior
- Synchronize version metadata, roadmap state, changelog, release notes, and release guidance around the same `v1.0.0` target

## Near-Term Priorities

The current recommended next work item is:

`v1.0.0: prepare the current main revision as the first stable release candidate`

Suggested scope:

- Keep xattrs, ACLs, special files, and broader Unix extensions deferred instead of treating them as stable-release blockers
- Re-run the authoritative release checklist on the selected candidate revision
- Refresh `benches/results/baseline-v0.1.0.json` because the candidate includes benchmark-affecting unpack setup changes beyond `v0.3.0`
- Prepare `v1.0.0` version metadata, changelog, and in-repo release notes for tagging

## Document Boundaries

This file tracks repository-level roadmap status. It does not replace:

- `openspec/changes/...`: per-change proposals, designs, and task breakdowns
- `sfa-tech-solution/`: broader technical design background
- `spec/`: frozen protocol and verification specifications
