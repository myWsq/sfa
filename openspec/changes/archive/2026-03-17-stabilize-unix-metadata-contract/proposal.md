## Why

SFA has already implemented the core Unix metadata path for `mode`, `uid`, `gid`, and `mtime`, plus owner-restore policy switches and safe restore ordering. However, the repository still treats M3 "Unix semantics" as an open-ended bucket, so the current contract is not clearly stated at the spec level and is not fully enforced by repository-level verification.

## What Changes

- Define the first M3 slice around the existing v1 Unix metadata contract instead of jumping directly to xattrs or ACLs.
- Clarify which Unix metadata semantics are part of the supported v1 behavior for pack and unpack, including `mode`, `mtime`, stored owner fields, root-gated owner restore, and current non-goals.
- Add repository-verifiable coverage for metadata-focused roundtrip and policy behavior so these semantics are not left as implementation details.
- Align roadmap-facing and technical-solution documentation with the new M3 boundary and keep xattrs / ACL explicitly deferred.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `archive-pack`: clarify the v1 metadata fields and owner-preservation semantics that pack records for supported Unix entries.
- `archive-unpack`: define the stable restore contract for `mode`, `mtime`, and owner behavior, including supported policy branches and explicit deferred semantics.
- `cli-and-benchmarks`: require repository verification coverage for metadata-focused roundtrip and restore-policy behavior.

## Impact

- OpenSpec specs for pack, unpack, and repository verification
- `crates/sfa-core` manifest/config semantics related to preserved owner metadata
- `crates/sfa-unixfs` scan, restore, and roundtrip verification coverage
- `crates/sfa-cli` owner-policy behavior and CLI-facing validation/docs
- Repository docs such as `ROADMAP.md`, `README.md`, and `sfa-tech-solution/`
