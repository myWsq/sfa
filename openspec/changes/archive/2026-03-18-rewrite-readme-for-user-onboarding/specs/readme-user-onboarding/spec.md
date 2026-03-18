## ADDED Requirements

### Requirement: Root README leads with the user-facing value proposition
The repository root README SHALL open with a concise description of SFA's target use case, SHALL explain that the tool is intended for Unix directory trees where small-file behavior and ordered restore matter, and SHALL position SFA against `tar` in terms users can evaluate quickly.

#### Scenario: New visitor lands on the repository
- **WHEN** a user reads the first section of the root README
- **THEN** they can identify what SFA is for, why it exists, and that it is not presented as a general drop-in replacement for every `tar` workflow

### Requirement: Root README provides actionable installation guidance that matches release reality
The repository root README SHALL provide an installation section that matches the repository's actual release state. When prebuilt release assets are available, the README MUST describe or link the real platform archives and checksum flow. When the repository is still pre-release, the README MUST say that clearly and MUST present build-from-source as the active installation path instead of implying that published binaries already exist.

#### Scenario: Repository has published release archives
- **WHEN** a user opens the installation section after a release is available
- **THEN** the README names or links the real release archive path, identifies the supported target platforms, and explains how to verify or use the downloaded asset

#### Scenario: Repository is still preparing a release
- **WHEN** a user opens the installation section before prebuilt release assets are published
- **THEN** the README explicitly states that release archives are not yet the active path and directs the user to the supported build-from-source workflow

### Requirement: Root README quick start is runnable for installed-binary users
The repository root README SHALL provide a quick-start flow that assumes the CLI is already installed and SHALL show at least one pack example and one unpack example using the real command-line surface. If stdin unpack and machine-readable stats are part of the documented CLI surface, the README MUST expose those examples without requiring users to infer them from source-build paths.

#### Scenario: User installs the CLI and wants the first successful run
- **WHEN** a user follows the quick-start section after installation
- **THEN** they can create an archive, unpack an archive, and discover the documented stats or stdin workflow without first reading build-system instructions

### Requirement: Root README surfaces benchmark evidence with scope and traceability
The repository root README SHALL include a short benchmark snapshot drawn from the committed comparison baseline, SHALL state that the comparison uses `tar` with the same codec, and SHALL link readers to the benchmark methodology or result asset needed to interpret the claim correctly.

#### Scenario: User evaluates the performance claim
- **WHEN** a user reads the benchmark section of the README
- **THEN** they see concrete comparison evidence and can trace that evidence back to the committed baseline or methodology instead of relying on an unsupported marketing claim

### Requirement: Root README separates onboarding from maintainer detail
The repository root README SHALL place installation, quick start, benchmark evidence, and scope framing before repository-internal sections such as verification checklists, repository layout, and contribution guidance.

#### Scenario: User only wants to decide whether to try SFA
- **WHEN** a user scans the README from top to bottom
- **THEN** they reach the adoption path before maintainer-oriented repository detail
