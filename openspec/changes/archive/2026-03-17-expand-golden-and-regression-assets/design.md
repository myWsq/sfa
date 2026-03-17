## Context

SFA v1 has already frozen the wire format, landed a working pack/unpack CLI, and committed a first benchmark baseline. The remaining M1 closeout work is regression depth rather than core feature delivery: the golden fixture corpus currently contains a single small archive, the protocol smoke path validates only that narrow corpus, and CLI integration coverage is limited to stdin unpack and diagnostics output.

This change must improve release confidence without changing the frozen `.sfa` protocol or introducing flaky, environment-sensitive verification. The main constraints are:

- `spec/format-v1.md` is already frozen, so this change can only strengthen regression assets around the existing protocol surface.
- CI must stay deterministic and repository-local; required checks cannot depend on host-specific tar codec support or ad hoc fixture generation.
- Golden assets must remain reviewable, with each committed fixture documenting what protocol surface it covers and how it was generated.

## Goals / Non-Goals

**Goals:**

- Expand the committed golden corpus so the frozen v1 protocol is represented by a small but meaningful set of fixtures rather than a single sample archive.
- Add focused CLI regression coverage for defaults, supported option combinations, and common failure paths that matter to normal local use.
- Keep protocol-fixture checks, CLI behavior checks, and documentation aligned so contributors can tell when a fixture, test, or note must be updated together.

**Non-Goals:**

- Changing the `.sfa` wire format, supported entry semantics, or benchmark result schema
- Turning benchmark refresh into a mandatory blocker for this change
- Adding post-v1 Unix metadata features such as xattrs or ACLs
- Replacing the existing smoke entrypoints with a new verification framework

## Decisions

### 1. Expand coverage with several narrow canonical fixtures instead of one “kitchen sink” archive

The golden corpus will grow as a curated set of small fixtures, where each fixture makes one or two protocol dimensions obvious: codec and integrity choice, multi-bundle layout, and supported Unix entry semantics. This keeps fixture diffs reviewable and makes smoke failures easy to localize.

Alternatives considered:

- One large fixture covering everything: rejected because it hides which protocol dimension drifted and makes fixture review noisy.
- Generating fixtures on demand in CI: rejected because the committed golden set is supposed to anchor the frozen protocol, not merely reproduce it.

### 2. Split regression responsibilities between fixture-based protocol checks and CLI integration tests

Golden fixtures will remain the source of truth for wire-format compatibility: committed archive bytes, decoded manifest dumps, and stable summary snapshots. CLI regression tests will cover user-facing behavior that is awkward to represent as static fixture assets, such as defaults, usage errors, JSON stats output, stdin restrictions, and overwrite interactions.

Alternatives considered:

- Encode all regression behavior in shell smoke scripts: rejected because shell-only checks become brittle and make failures harder to attribute.
- Push more behavior checks down into library-only tests: rejected because several gaps are specifically CLI contract gaps, not library contract gaps.

### 3. Keep smoke scripts as thin entrypoints over committed assets and focused tests

The existing smoke scripts already provide stable CI entrypoints. This change should preserve that shape: protocol smoke discovers every committed golden fixture and validates its required assets, while Rust tests carry most of the option-level CLI behavior coverage. The scripts should orchestrate and fail fast, not become the primary place where behavior logic lives.

Alternatives considered:

- Replace the scripts with one monolithic test binary: rejected because the current scripts are already wired into CI and release guidance.

### 4. Treat fresh `tar + same codec` reruns as supporting evidence, not a prerequisite for apply-ready artifacts

The roadmap mentions a fresh benchmark rerun as useful corroboration, but it is environment-dependent and does not define the main release blocker for M1. This change will document it as an optional follow-up rather than coupling regression asset work to host-specific benchmark availability.

Alternatives considered:

- Make benchmark refresh part of the blocking task set: rejected because it would mix deterministic repository work with environment-specific execution evidence.

## Risks / Trade-offs

- [Golden corpus grows without a clear boundary] → Keep the fixture set representative rather than exhaustive, and require each fixture README to state the protocol dimensions it covers.
- [CLI regression tests become platform-fragile] → Limit coverage to Unix-like behavior already in scope for SFA v1 and use temporary workspaces with generated archives instead of host-dependent external assets.
- [Fixture and spec updates drift apart] → Update capability deltas, fixture docs, and smoke enumeration in the same change so protocol-significant asset changes remain reviewable.
- [Coverage duplicates existing library tests without adding confidence] → Target CLI-only behaviors such as argument defaults, exit semantics, output format, and supported stdin combinations instead of re-testing every backend detail through the CLI.

## Migration Plan

1. Land spec deltas that define the stronger expectations for the golden corpus and CLI regression coverage.
2. Add the new committed golden fixtures and their documentation, using the existing generator flow where appropriate.
3. Add or extend CLI integration tests to cover the agreed behavior matrix.
4. Update smoke or release-facing documentation where the expected regression assets and refresh triggers need to be explicit.

No runtime or data migration is required. Rollback is a normal code-and-assets revert of the added fixtures, tests, and documentation.

## Open Questions

There are no blocking open questions for proposal readiness. This design assumes:

- the canonical corpus will cover all three currently exposed integrity modes (`off`, `fast`, `strong`), and
- overwrite behavior will be validated through focused CLI tests rather than a new protocol fixture dimension.
