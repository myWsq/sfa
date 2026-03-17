## Context

The repository already captures and restores the core Unix metadata path used by the current v1 implementation: scan records `mode`, `uid`, `gid`, and `mtime`; unpack restores `mode` and `mtime`; and owner restoration is gated behind an explicit preserve-owner branch plus an effective-root check. The roadmap now points at M3, but the technical boundary is still blurry because older solution docs still treat some already-delivered behavior as future work, while repository-level verification does not yet describe the metadata contract as a first-class requirement.

This change is meant to harden the contract before any post-v1 metadata expansion. The main constraint is that v1 wire-format behavior is already frozen, so the work here must clarify and verify current semantics rather than redesign the archive layout.

## Goals / Non-Goals

**Goals:**

- Define the first M3 slice as contract hardening for the current v1 Unix metadata behavior.
- Clarify pack and unpack requirements for `mode`, `mtime`, stored owner fields, and owner-restore policy behavior.
- Expand repository verification expectations so metadata roundtrip behavior is traceable at the spec level and not only in crate-local tests.
- Keep roadmap and solution docs aligned with the narrowed M3 scope.

**Non-Goals:**

- Adding xattrs, ACLs, special-file restore, or cross-platform parity guarantees.
- Changing the frozen v1 archive layout, manifest structure, or feature-bit meanings.
- Redesigning the CLI/API owner policy model beyond documenting the behavior the repository already implements.
- Requiring privileged CI or release automation that assumes root access.

## Decisions

### 1. Reuse existing OpenSpec capabilities instead of introducing a new metadata capability

This change will modify `archive-pack`, `archive-unpack`, and `cli-and-benchmarks` rather than creating a separate `unix-metadata-contract` capability. The behavior already lives inside those interfaces, so the spec deltas should stay attached to the pack path, unpack path, and repository verification contract that own them today.

Alternative considered:

- Add a new metadata-focused capability and leave existing pack/unpack specs untouched. Rejected because it would split one behavior surface across multiple capabilities and make archive-time semantics harder to audit.

### 2. Treat stored owner fields and owner-restore intent as distinct parts of the contract

The pack path already records `uid` and `gid` in manifest entries while separately surfacing owner-preservation intent through configuration and header feature flags. This change will document that split explicitly: recorded owner fields are part of archive metadata, but applying them during restore remains opt-in and root-gated.

Alternative considered:

- Redefine unpack to infer owner restoration from archive metadata or caller identity. Rejected because that would change current behavior and broaden M3 from contract hardening into feature redesign.

### 3. Make repository verification cover the stable non-privileged metadata contract

Repository-level verification will be specified around behavior that can be exercised reliably on a normal Unix development host: metadata roundtrip for `mode` and `mtime`, default no-owner restore behavior, link semantics, and restore safety. Root-only owner application remains part of the spec contract, but the standard verification path should not depend on privileged CI.

Alternative considered:

- Add root-required smoke checks to the mandatory verification matrix. Rejected because they are hard to run portably and would make the default release checklist less reproducible.

### 4. Keep xattrs and ACLs explicitly deferred

This change will describe xattrs and ACLs as still deferred rather than partially reserving new runtime behavior. That keeps M3 focused on stabilizing the semantics already present in the codebase and avoids mixing contract cleanup with new metadata-surface delivery.

Alternative considered:

- Start consuming `MetaBlob` for xattrs or ACLs in the same change. Rejected because it increases scope and risks conflating v1 contract clarification with new format-facing behavior.

## Risks / Trade-offs

- [Current CLI owner-policy naming remains imperfect] -> Document the implemented behavior precisely now and leave any future `Auto` / `Never` cleanup to a separate change.
- [Standard verification will still not exercise root-only owner application end to end] -> Keep that branch in the normative spec while focusing repository-default verification on non-privileged semantics.
- [Documentation can drift again after the spec lands] -> Include roadmap and solution-doc synchronization in the implementation tasks, not as an optional follow-up.

## Migration Plan

No archive migration is required. Existing v1 archives remain valid because this change clarifies and verifies current behavior instead of changing wire compatibility.

Rollout order:

1. Update specs to define the metadata contract and deferred scope.
2. Align tests and smoke/integration coverage with the updated requirements.
3. Sync repository-facing docs so M3 reflects the new scope boundary.

Rollback is low risk because the change is documentation-and-verification centric. If a requirement proves too strict, the affected spec delta can be revised in a follow-up OpenSpec change without invalidating previously written archives.

## Open Questions

- Should repository docs keep exposing `restore_owner=auto` as a distinct concept, or simply describe it as part of the non-preserving path until the CLI surface is redesigned?
- Should metadata-focused verification be folded into the existing roundtrip smoke path, or called out as a separate integration/test asset for clearer auditing?
