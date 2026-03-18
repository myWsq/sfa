## MODIFIED Requirements

### Requirement: Milestone closeout synchronizes release-facing status documents
When the repository closes a roadmap milestone or prepares a first stable v1 release from that milestone state, it SHALL update the release-facing status documents to describe the same completion state, immediate release train, and deferred scope. This synchronization MUST cover `ROADMAP.md` and any higher-level status summary that describes the current project state, and it MUST keep later milestone or post-v1 work visible rather than silently dropping it.

#### Scenario: M3 closeout is reflected across repository status docs
- **WHEN** maintainers conclude that the current M3 metadata-contract hardening slice has satisfied its closeout conditions for the stable v1 scope
- **THEN** `ROADMAP.md` and the repository's top-level status summary both describe that slice as complete and identify stable-release preparation as the next repository step

#### Scenario: Deferred post-v1 Unix work stays visible
- **WHEN** release-facing documents are updated for M3 closeout and first-stable-release preparation
- **THEN** xattrs, ACLs, and broader Unix metadata extensions remain documented as deferred follow-up work instead of being implied as already shipped in `v1.0.0`

### Requirement: Release artifacts are synchronized before tagging
Before an SFA version tag is created, the repository SHALL synchronize the versioned release artifacts that describe what is being shipped. This synchronization MUST cover the workspace version, `CHANGELOG.md`, the in-repository release notes file for the target version, and release guidance inputs that identify the required verification checklist, candidate release scope, and compatibility framing for that release.

#### Scenario: Maintainer prepares the first stable version tag
- **WHEN** a maintainer updates the repository for the first stable SFA version tag from a selected candidate revision
- **THEN** the workspace version, changelog entry, in-repo release notes, and release guidance inputs all describe the same stable release target and included change scope before the tag is created

#### Scenario: Reviewer audits first stable release inputs from the repository alone
- **WHEN** a reviewer inspects the repository before the first stable version tag is pushed
- **THEN** they can determine the intended release version, the included candidate scope, the deferred follow-up work, and the required verification steps without relying on out-of-band notes

## ADDED Requirements

### Requirement: Stable release candidates are verified on the selected candidate revision
Before the repository is treated as ready for a first stable SFA version tag, the authoritative release checklist SHALL succeed on the selected candidate revision. If that candidate revision includes benchmark-affecting changes since the latest tagged release, the repository SHALL also refresh and validate the committed benchmark baseline asset before tagging.

#### Scenario: Candidate includes benchmark-affecting unpack changes after the latest tag
- **WHEN** the selected first-stable-release candidate includes unpack setup, pipeline, benchmark-schema, or other benchmark-affecting changes beyond the latest tagged release
- **THEN** release preparation re-runs the authoritative checklist on that candidate revision and refreshes the committed benchmark baseline before the version tag is created

#### Scenario: Candidate revision fails a mandatory checklist gate
- **WHEN** the selected first-stable-release candidate fails `cargo fmt --all --check` or any other mandatory release-checklist command
- **THEN** the repository is not treated as stable-release ready until that blocker is fixed and the checklist is re-run successfully on the candidate revision
