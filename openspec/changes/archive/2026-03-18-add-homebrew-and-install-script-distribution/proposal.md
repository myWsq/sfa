## Why

SFA currently asks most users to either build from source or manually download a platform archive from GitHub Releases. That works for maintainers, but it is a weak user distribution story for a CLI that wants broader trial and repeat installation on macOS and Linux.

## What Changes

- Add a project-supported managed installation path for released SFA binaries centered on two user-facing channels: a project-owned Homebrew tap and a versioned shell install script.
- Treat the existing GitHub Release archives and checksum files as the single source of truth for shipped binaries, with both managed channels deriving from those release assets rather than introducing a second packaging source.
- Define the supported install-script behavior for platform detection, archive and checksum retrieval, checksum validation, installation target selection, and actionable failure modes on unsupported hosts.
- Define the supported Homebrew tap behavior so released versions can be installed and upgraded with `brew install` and `brew upgrade` on the repository's current macOS and Linux target matrix.
- Update release-facing documentation and onboarding guidance so managed install channels are the default recommendation, while direct archive download and source build remain documented fallback paths.
- Keep Windows package-manager distribution, crates.io publishing, and macOS signing or notarization out of scope for this change.

## Capabilities

### New Capabilities

- `cli-distribution`: defines the managed distribution contract for release archives, the install script, and the Homebrew tap workflow.

### Modified Capabilities

- `readme-user-onboarding`: change installation guidance so the README recommends managed install channels first when releases are available, while keeping archive download and source build as explicit fallbacks.

## Impact

- Release automation under `.github/workflows/` and any supporting scripts that derive install metadata from tagged release assets
- New installer-facing assets such as a public shell install script and Homebrew formula generation inputs
- `README.md`, `RELEASING.md`, and release notes that describe supported installation channels and fallback paths
- Repository secrets or release-time credentials needed to update a project-owned Homebrew tap repository
