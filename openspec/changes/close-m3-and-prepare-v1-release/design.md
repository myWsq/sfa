## Context

SFA has already reached the point where the frozen wire format, baseline verification assets, and current Unix metadata contract are all repository-traceable, but the repository still describes itself as if the current M3 slice were open-ended. At the same time, `main` has moved one commit beyond `v0.3.0` with additional unpack setup optimization work, so the next release candidate is no longer just a documentation exercise: it must decide whether the stable-release scope is based on `v0.3.0` or the current head, and then make every release-facing artifact tell the same story.

This is a cross-cutting repository-state change. It touches roadmap status, version selection, changelog and release notes, release guidance, benchmark evidence, and the documented quality gates used before tagging. The change must keep deferred post-v1 work visible while avoiding any accidental expansion of the stable v1 scope.

## Goals / Non-Goals

**Goals:**

- Close the current M3 Unix metadata hardening slice as the completed v1 metadata-contract boundary.
- Establish an auditable repository path to the first stable `v1.0.0` release.
- Synchronize versioned release materials, roadmap state, and release guidance around one candidate revision.
- Restore and re-run the authoritative release checklist on that candidate revision before calling it stable-release ready.

**Non-Goals:**

- Adding xattrs, ACLs, special-file restore, or any broader Unix metadata capability.
- Changing the frozen `format-v1` wire format or redefining current pack/unpack semantics.
- Replacing the existing release workflow, smoke scripts, or benchmark harness.
- Designing a post-v1 feature roadmap in full detail beyond keeping deferred work visible.

## Decisions

### 1. Treat this as a repository-release change, not a runtime feature proposal

The change will primarily operate on repository-facing state: milestone closure, stable-release framing, versioned release materials, and release-gate evidence. Runtime code changes are allowed only when they are necessary to restore the documented release checklist on the selected candidate revision.

Alternatives considered:

- Fold this work into a broader runtime feature change: rejected because it would blur the line between stable-release readiness and new capability delivery.
- Skip OpenSpec and patch docs directly: rejected because the repository already treats milestone closeout and release preparation as traceable spec-governed work.

### 2. Use the current `main` revision as the first stable-release candidate baseline

The repository should prepare the first stable release from the current head rather than rewinding to `v0.3.0`. The post-`v0.3.0` unpack directory setup optimization is already merged, accompanied by OpenSpec artifacts, and should either be shipped intentionally or reverted explicitly. This design chooses to ship it intentionally and make the release materials describe it.

Alternatives considered:

- Cut the first stable release from `v0.3.0` plus doc-only updates: rejected because it ignores already-merged mainline behavior and would force maintainers to explain why the stable release does not match `main`.
- Keep the scope undecided until tagging: rejected because roadmap, benchmark, and release-note preparation all depend on a concrete candidate revision.

### 3. Close the current M3 slice and move broader Unix extensions out of the stable-release path

The roadmap should treat the current metadata-contract hardening slice as complete once repository-facing status, technical-solution documents, and release materials all align with the shipped behavior. xattrs, ACLs, special files, and any broader Unix-surface expansion should remain explicitly deferred to a later post-v1 milestone instead of being implied as release blockers for `v1.0.0`.

Alternatives considered:

- Keep M3 marked in progress until all Unix extensions are decided: rejected because it leaves the stable v1 boundary ambiguous and contradicts the existing scoped metadata-contract work.
- Mark all Unix semantics work complete with no deferred follow-up: rejected because it would overstate the current product surface.

### 4. Make stable-release preparation auditable through one synchronized artifact set

The candidate revision should be considered stable-release ready only when `ROADMAP.md`, `README.md`, `CHANGELOG.md`, `RELEASING.md`, workspace version metadata, and the in-repo `release-notes/v1.0.0.md` all describe the same target release and deferred scope. If the candidate includes benchmark-affecting changes beyond `v0.3.0`, the committed benchmark baseline must be refreshed and validated as part of that artifact set.

Alternatives considered:

- Treat release notes as external-only and keep repository materials loosely aligned: rejected because repository-only review is a stated requirement.
- Treat benchmark refresh as optional for the stable release candidate even when unpack pipeline behavior changed: rejected because the existing release process already distinguishes mandatory dry-run from required baseline refresh for benchmark-affecting changes.

## Risks / Trade-offs

- [Stable version selection proves premature] -> Keep the design scoped to the already-frozen protocol and already-implemented v1 behavior, and keep deferred post-v1 work visible in roadmap and release notes.
- [Release-prep work expands into open-ended documentation cleanup] -> Limit file updates to milestone state, release materials, verification guidance, and directly related spec wording.
- [Benchmark baseline refresh is noisy or environment-sensitive] -> Use the documented refresh command, validate the committed asset with `cargo test -p sfa-bench`, and describe the environment assumptions in the release notes.
- [A small runtime regression blocks release prep late] -> Allow narrowly scoped fixes that restore the documented checklist, but avoid bundling unrelated improvements.

## Migration Plan

1. Update the release-readiness spec delta so repository requirements cover M3 closeout and first-stable-release preparation.
2. Synchronize roadmap and status documents to show the completed current M3 slice, the chosen stable release target, and the explicitly deferred post-v1 work.
3. Update versioned release materials for the chosen candidate revision, including workspace version, changelog, release guidance, and `release-notes/v1.0.0.md`.
4. Restore release-gate compliance on the candidate revision, run the authoritative checklist, and refresh the benchmark baseline if the selected scope includes benchmark-affecting changes beyond `v0.3.0`.
5. Review the final diff and release evidence, then hand off a clean candidate for tagging.

Rollback is straightforward: revert the repository-state updates and any narrowly scoped checklist-fix changes, then return the roadmap and release train to the previous pre-stable state.

## Open Questions

- Should the release notes frame `v1.0.0` as the first stable release of the existing CLI only, or explicitly as the first recommended public adoption point for the frozen `format-v1` protocol?
- Is any additional benchmark narrative needed beyond the refreshed committed baseline to explain the inclusion of the post-`v0.3.0` unpack directory setup optimization?
