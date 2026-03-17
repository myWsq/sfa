## MODIFIED Requirements

### Requirement: Test suites cover protocol, streaming, corruption, and safety baselines
Before the first v1 release, the project SHALL include automated tests for roundtrip correctness, fragmented sequential input, corruption rejection, path safety, golden archive compatibility, and CLI behavior regressions for documented defaults, supported input-mode combinations, and expected usage failures.

#### Scenario: CI runs repository verification coverage
- **WHEN** repository tests are executed in CI
- **THEN** the suite includes protocol, streaming, corruption, security, golden-archive, and CLI regression cases for the v1 feature set

## ADDED Requirements

### Requirement: CLI regression suite pins default and error-path behavior
The repository SHALL include automated CLI tests that exercise the documented defaults and common supported combinations for `sfa pack` and `sfa unpack`. These tests MUST cover successful machine-readable stats output under default pack options, missing-input failures, usage-error exit codes, supported `stdin` interactions, and overwrite-related restore behavior that differs from the default safe path.

#### Scenario: Pack dry-run exposes default stats without extra flags
- **WHEN** a user runs `sfa pack <input-dir> <archive-path> --dry-run --stats-format json`
- **THEN** the command succeeds and emits machine-readable stats that include the effective default codec, integrity mode, thread count, and bundle-planning fields

#### Scenario: Missing archive path fails before unpack work begins
- **WHEN** a user runs `sfa unpack ./missing.sfa -C ./out`
- **THEN** the CLI reports an actionable input-archive failure and exits non-zero instead of fabricating unpack stats

#### Scenario: Existing output requires explicit overwrite intent
- **WHEN** a user runs `sfa unpack <archive-path> -C <existing-output-root>` and restoration would replace an existing file
- **THEN** the default command fails non-zero, and the corresponding overwrite-enabled path is covered by automated CLI regression checks
