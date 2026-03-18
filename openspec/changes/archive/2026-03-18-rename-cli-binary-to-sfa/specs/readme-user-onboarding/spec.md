## MODIFIED Requirements

### Requirement: Root README provides actionable installation guidance that matches release reality
The repository root README SHALL provide an installation section that matches
the repository's actual release state. When prebuilt release assets are
available, the README MUST describe or link the real platform archives and
checksum flow, MUST describe the managed installation paths using the public
`sfa` command name, and MUST distinguish any required tap step from the short
`brew install sfa` command. When the repository is still pre-release, the
README MUST say that clearly and MUST present build-from-source as the active
installation path instead of implying that published binaries already exist.

#### Scenario: Repository has published release archives
- **WHEN** a user opens the installation section after a release is available
- **THEN** the README names or links the real release archive path, identifies
  the supported target platforms, and explains how to install or verify the
  `sfa` binary through the documented managed or manual paths

#### Scenario: Repository is still preparing a release
- **WHEN** a user opens the installation section before prebuilt release assets
  are published
- **THEN** the README explicitly states that release archives are not yet the
  active path and directs the user to the supported build-from-source workflow

### Requirement: Root README quick start is runnable for installed-binary users
The repository root README SHALL provide a quick-start flow that assumes the
CLI is already installed and SHALL show at least one pack example and one
unpack example using the real installed command name `sfa`. If stdin unpack
and machine-readable stats are part of the documented CLI surface, the README
MUST expose those examples without requiring users to infer them from
source-build paths.

#### Scenario: User installs the CLI and wants the first successful run
- **WHEN** a user follows the quick-start section after installation
- **THEN** they can create an archive, unpack an archive, and discover the
  documented stats or stdin workflow using `sfa` without first reading
  build-system instructions
