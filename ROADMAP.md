# SFA v1 Roadmap

This document describes the current development stages, project status, and near-term priorities for SFA v1. Unless explicitly stated otherwise, it does not represent a release-date commitment.

Last updated: 2026-03-17

## Current Status

SFA v1 is under active development. The repository already contains a runnable minimal usable chain, a frozen protocol definition, completed M1 and M2 milestones, and the first phase of M3 work focused on tightening the Unix metadata contract and its verification assets.

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

The next work cycle is currently focused on:

- Advancing the first phase of M3 by stabilizing the current Unix metadata contract and verification assets
- Keeping versioning, release checklist documentation, and release notes aligned with the repository state

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
| M3 | Unix semantics hardening | In progress | Stabilize the current metadata contract first, then decide the scope of further Unix metadata extensions |

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

Status: `In progress`

Current scope:

- Stabilize the current v1 Unix metadata contract, especially the commitment boundary for `mode`, `mtime`, and owner policy
- Add repository-level verification for metadata roundtrips, owner policy behavior, and existing link / safety scenarios
- Keep xattrs and ACLs deferred while aligning roadmap, README, and technical design documents with shipped behavior

Follow-up candidates:

- Evaluate xattrs and ACLs if they remain in scope for v1
- Expand Unix boundary cases and exceptional-path coverage

Closure criteria:

- The current metadata contract has aligned specs, implementation behavior, and verification assets
- Repository status documents and technical design documents no longer mark delivered behavior as future work
- Any broader metadata extension is split into a dedicated OpenSpec change

## Near-Term Priorities

The current recommended next work item is:

`M3: stabilize the Unix metadata contract`

Suggested scope:

- Deepen verification for owner and metadata restore behavior
- Keep xattrs and ACLs deferred instead of mixing them into the current change
- Make metadata roundtrip, owner policy, and existing link / safety scenarios part of auditable repository-level verification
- Keep the release checklist and benchmark dry run as stable pre-release gates

## Document Boundaries

This file tracks repository-level roadmap status. It does not replace:

- `openspec/changes/...`: per-change proposals, designs, and task breakdowns
- `sfa-tech-solution/`: broader technical design background
- `spec/`: frozen protocol and verification specifications
