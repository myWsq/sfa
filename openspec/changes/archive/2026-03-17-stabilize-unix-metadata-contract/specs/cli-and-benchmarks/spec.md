## MODIFIED Requirements

### Requirement: Test suites cover protocol, streaming, corruption, and safety baselines
Before the first v1 release, the project SHALL include automated tests for roundtrip correctness, fragmented sequential input, corruption rejection, path safety, golden archive compatibility, metadata roundtrip for supported Unix entries, and CLI behavior regressions for documented defaults, supported input-mode combinations, and expected usage failures. This verification coverage MUST include checks that supported restores preserve `mode` and `mtime` for regular files and directories, that the default unpack path leaves owner restoration disabled, and that link and safety scenarios remain represented in committed fixtures or tests.

#### Scenario: CI runs repository verification coverage
- **WHEN** repository tests are executed in CI
- **THEN** the suite includes protocol, streaming, corruption, security, golden-archive, metadata-restore, and CLI regression cases for the v1 feature set
