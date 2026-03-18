## ADDED Requirements

### Requirement: Tagged release assets remain canonical for managed installation
For every supported SFA release version, the repository SHALL publish
platform-specific GitHub Release archives together with matching checksum files.
Any supported managed installation channel MUST derive its artifact URLs and
integrity data from those tagged release assets rather than building, mutating,
or hosting a separate binary payload.

#### Scenario: Managed install metadata references the released artifacts
- **WHEN** a maintainer publishes a supported SFA release tag
- **THEN** the resulting managed installation metadata references the exact
  archive and checksum assets attached to that GitHub Release for the supported
  target matrix

### Requirement: Public install script installs the `sfa` binary with checksum verification
The repository SHALL provide a public shell install script for supported
released versions. The install script MUST detect the current host OS and
architecture, MUST resolve the matching release archive, MUST verify the
downloaded archive against the published checksum, MUST install the `sfa`
binary into a user-selectable destination directory, and MUST fail with
actionable errors when the host is unsupported, a required download or checksum
tool is unavailable, the release asset is missing, or verification fails.

#### Scenario: Supported host installs a released version with the script
- **WHEN** a user runs the supported install script on a supported macOS or
  Linux host for an available SFA release
- **THEN** the script downloads the matching release archive, verifies it
  against the published checksum, installs `sfa` into the requested
  destination, and reports the installed version

#### Scenario: Unsupported host runs the install script
- **WHEN** a user runs the install script on an unsupported operating system or
  architecture
- **THEN** the script exits non-zero before installation and reports that the
  host is unsupported for published SFA binaries

### Requirement: Homebrew tap publishes the `sfa` formula
The project SHALL maintain a project-owned Homebrew tap for the supported SFA
release matrix. For each supported released version, the published Homebrew
formula MUST be named `sfa`, MUST reference the canonical GitHub Release
archive URLs and checksums for that version, and MUST install the packaged
`sfa` binary from those release assets as part of the release publication flow
rather than as a detached manual step.

#### Scenario: Release publication refreshes the `sfa` formula
- **WHEN** a maintainer publishes a supported SFA release
- **THEN** the project-owned Homebrew tap contains `Formula/sfa.rb` for that
  release and its URLs and SHA-256 values match the corresponding GitHub
  Release assets

#### Scenario: User installs SFA from the tapped project tap
- **WHEN** a user adds the project tap and runs the documented `brew install sfa`
  command on a supported host
- **THEN** Homebrew retrieves the platform-appropriate GitHub Release archive
  for that version and installs the packaged `sfa` binary successfully
