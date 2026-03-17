## Why

SFA has already frozen `format-v1`, completed M0 through M2, and finished the current M3 metadata-contract hardening slice, but the repository still does not present a clean path from the current `0.x` line to a first stable `v1.0.0` release. Roadmap-facing status, versioned release materials, and the current post-`v0.3.0` release scope are not yet synchronized well enough for an auditable stable-release review.

## What Changes

- Close the current M3 Unix metadata hardening slice in repository-facing roadmap and status documents, while keeping xattrs, ACLs, and broader Unix extensions explicitly deferred past the first stable release.
- Define the repository-level release-prep work needed for the first stable `v1.0.0` release, including version-target selection, release-notes preparation, changelog synchronization, and clear treatment of the current post-`v0.3.0` candidate scope.
- Restore release-gate compliance on the candidate revision and re-run the authoritative verification checklist before treating the repository as stable-release ready.
- Make the first stable release review auditable from the repository alone by aligning roadmap state, release guidance, version metadata, and in-repo release notes around the same release target.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `release-readiness`: extend milestone-closeout and release-artifact synchronization requirements to cover M3 closeout and first-stable-release preparation, not only the earlier M1 closeout path.

## Impact

- Release-facing repository documents such as `ROADMAP.md`, `README.md`, `CHANGELOG.md`, and `RELEASING.md`
- Version metadata and the in-repo `release-notes/` set for the first stable release target
- The authoritative release checklist result for the candidate revision, including any required benchmark-baseline refresh for benchmark-affecting changes
- OpenSpec release-readiness requirements that govern milestone closure and stable-release preparation
