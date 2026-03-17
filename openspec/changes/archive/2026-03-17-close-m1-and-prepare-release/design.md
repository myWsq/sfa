## Context

SFA v1 has already landed its frozen format, benchmark baseline, golden fixtures, and CLI regression coverage, but the repository still presents that work as partially unfinished. `ROADMAP.md` still leaves M1 open, `CHANGELOG.md` does not yet summarize the post-`0.1.0` closeout work, and the documented release gate is not yet enforced by a clean formatting pass. This change is cross-cutting because it touches verification commands, release-facing docs, version bookkeeping, and OpenSpec capability definitions rather than one runtime module.

The main constraint is that this change must not silently expand the product scope. It should only formalize and verify the release-ready state of the existing v1 surface, while leaving post-M1 work such as broader Unix metadata support in later milestones.

## Goals / Non-Goals

**Goals:**

- Make the repository-level release checklist executable and aligned with the commands already documented for release preparation.
- Close the gap between actual implementation state and release-facing status documents.
- Define a traceable OpenSpec contract for milestone closeout and release artifact synchronization.
- Keep benchmark dry-run and conditional baseline refresh guidance explicit so release preparation stays deterministic.

**Non-Goals:**

- Changing the `.sfa` wire format, archive semantics, or benchmark schema
- Adding new Unix metadata features such as xattrs or ACL handling
- Replacing the existing smoke scripts or benchmark harness with a new verification framework
- Publishing a release automatically or defining distribution packaging beyond the current manual process

## Decisions

### 1. Treat M1 closeout as a repository-readiness change, not a product-feature change

This change will focus on repository state: formatting compliance, release checklist execution, milestone status, changelog coverage, and release guidance. It will not be used to bundle additional pack/unpack behavior or protocol changes.

Alternatives considered:

- Fold M1 closeout into the next feature change: rejected because it obscures whether the current v1 surface is independently releasable.
- Skip OpenSpec and just patch docs: rejected because milestone closeout and release readiness should be review-traceable.

### 2. Add a dedicated `release-readiness` capability instead of overloading runtime specs

The current OpenSpec capability set covers format, pack, unpack, and CLI/benchmark behavior, but it does not define repository-level requirements for version sync, milestone closeout, or release document alignment. A dedicated capability makes those expectations testable and archiveable.

Alternatives considered:

- Put all release-prep rules into `cli-and-benchmarks`: rejected because version and roadmap synchronization are broader than CLI verification.
- Keep release process only in `RELEASING.md`: rejected because it leaves change tracking and review history outside the spec system.

### 3. Make the documented release checklist authoritative and executable

The release gate will be defined around the commands already named in `RELEASING.md`: formatting check, full workspace tests, protocol/streaming/safety/roundtrip smoke checks, and benchmark dry-run. The implementation work for this change should make those commands pass in a clean workspace rather than inventing a second checklist.

Alternatives considered:

- Reduce the required checklist to tests only: rejected because the repository already documents formatting and benchmark dry-run as release gates.
- Require a fresh benchmark baseline on every release-prep change: rejected because the benchmark docs already distinguish between deterministic dry-run validation and environment-sensitive baseline refresh.

### 4. Keep benchmark baseline refresh conditional on benchmark-affecting changes

This change will preserve the current split: benchmark dry-run is mandatory for release readiness, while a fresh committed baseline refresh is only required when benchmark logic, datasets, codecs, planner settings, or observability schema change.

Alternatives considered:

- Always refresh the committed baseline: rejected because it makes routine release-prep work depend on host-specific tooling and environment conditions.

## Risks / Trade-offs

- [Repository docs drift again after this change] → Encode the closeout expectations in spec deltas and update the roadmap/changelog/release docs in the same implementation slice.
- [Release-readiness spec becomes too process-heavy] → Keep requirements narrowly scoped to repository-traceable artifacts and executable verification commands.
- [M1 is marked complete without enough evidence] → Require the documented release checklist to pass from a clean workspace before milestone closeout is treated as complete.
- [Version choice becomes a hidden decision] → Keep version selection explicit in tasks and release notes instead of burying it in unrelated edits.

## Migration Plan

1. Land the OpenSpec proposal, design, and spec deltas for release readiness and verification checklist requirements.
2. Update the affected repository files and tests so the documented release checklist passes cleanly.
3. Synchronize roadmap, changelog, and release guidance with the actual M1 state and the next release target.
4. Run the release checklist, review the resulting repository state, and only then decide whether to tag immediately or treat the branch as release-ready pending final version approval.

Rollback is a normal revert of the repository-state updates and spec deltas. No runtime data migration is involved.

## Open Questions

- Should M1 be marked complete as soon as the repository reaches release-ready state, or only once the next tag is actually created?
- What version number should carry the M1 closeout changes: `0.1.1` as a stabilization release or `0.2.0` as the first post-freeze milestone closeout?
