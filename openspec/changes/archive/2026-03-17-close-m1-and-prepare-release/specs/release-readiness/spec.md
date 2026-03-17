## ADDED Requirements

### Requirement: Milestone closeout synchronizes release-facing status documents
When the repository closes a roadmap milestone or declares a release-ready state for that milestone, it SHALL update the release-facing status documents to describe the same completion state and deferred scope. This synchronization MUST cover `ROADMAP.md` and any higher-level status summary that describes the current project state, and it MUST make later-mileestone work visible rather than silently dropping it.

#### Scenario: M1 closeout is reflected across repository status docs
- **WHEN** maintainers conclude that the M1 minimal-usable chain has satisfied its closeout conditions
- **THEN** `ROADMAP.md` and the repository's top-level status summary both describe M1 as complete and identify the next milestone focus

#### Scenario: Deferred post-M1 work stays visible
- **WHEN** release-facing documents are updated for milestone closeout
- **THEN** work intentionally deferred to later milestones such as Unix metadata expansion remains documented instead of being implied as already delivered

### Requirement: Release artifacts are synchronized before tagging
Before an SFA version tag is created, the repository SHALL synchronize the versioned release artifacts that describe what is being shipped. This synchronization MUST cover the workspace version, `CHANGELOG.md`, and release guidance inputs that identify the required verification checklist and compatibility framing for the release.

#### Scenario: Maintainer prepares a version tag
- **WHEN** a maintainer updates the repository for the next SFA version
- **THEN** the workspace version, changelog entry, and release guidance inputs all describe the same release target before the tag is created

#### Scenario: Reviewer audits release inputs from the repository alone
- **WHEN** a reviewer inspects the repository before a version tag is pushed
- **THEN** they can determine the intended release version, the included change summary, and the required verification steps without relying on out-of-band notes

### Requirement: Repository-level release-prep changes are OpenSpec traceable
Repository-wide release-preparation or milestone-closeout work that changes quality gates, release guidance, or milestone status SHALL be captured through an OpenSpec change with reviewable proposal, design, and task artifacts.

#### Scenario: Maintainer audits why a milestone was closed
- **WHEN** a maintainer needs to understand why the repository treated a milestone as complete or release-ready
- **THEN** the relevant OpenSpec change records the motivation, design decisions, and implementation tasks for that closeout work
