## MODIFIED Requirements

### Requirement: Root README provides actionable installation guidance that matches release reality
The repository root README SHALL provide an installation section that matches the repository's actual release state. When a published release includes supported managed installation channels, the README MUST recommend those managed channels first, MUST identify the current supported primary channels, and MUST keep direct archive download plus build-from-source as explicit fallback paths. When the repository is still pre-release or managed channels are not yet available, the README MUST say that clearly and MUST present build-from-source as the active installation path instead of implying that published binaries already exist.

#### Scenario: Repository has published managed installation channels
- **WHEN** a user opens the installation section after a release is available with supported managed install paths
- **THEN** the README points the user to the supported Homebrew and install-script flows first and still documents archive download and source build as fallback options

#### Scenario: Repository is still preparing a release
- **WHEN** a user opens the installation section before published managed install channels are available
- **THEN** the README explicitly states that managed release installation is not yet the active path and directs the user to the supported build-from-source workflow
