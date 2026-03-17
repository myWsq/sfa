## ADDED Requirements

### Requirement: Release verification checklist remains executable
The repository SHALL define a release-verification checklist for SFA that includes `cargo fmt --all --check`, `cargo test --workspace`, `bash tests/scripts/run_protocol_smoke.sh`, `bash tests/scripts/run_streaming_smoke.sh`, `bash tests/scripts/run_safety_smoke.sh`, `bash tests/scripts/run_roundtrip_smoke.sh`, and `cargo run -p sfa-bench --bin tar_vs_sfa -- --dry-run --output benches/results/latest.json`. The commands named in release guidance MUST succeed from a clean workspace on a supported development host before a version tag is created.

#### Scenario: Maintainer runs the documented release checklist
- **WHEN** a maintainer prepares an SFA release candidate from a clean checkout
- **THEN** the release-facing documentation names one authoritative checklist and each listed verification command completes successfully

#### Scenario: Formatting regression blocks release readiness
- **WHEN** a repository change causes `cargo fmt --all --check` to fail while the release checklist is being executed
- **THEN** the release-preparation workflow fails before a version tag is created and the formatting drift is treated as a blocker rather than an informational warning

### Requirement: Benchmark release guidance distinguishes mandatory dry-run from conditional baseline refresh
Release guidance for benchmark verification SHALL require the benchmark dry-run command on every release-preparation pass and MUST describe committed baseline refresh as conditional on benchmark-affecting changes such as runner logic, dataset contents, codec integration, planner semantics, or observability schema updates.

#### Scenario: Release prep without benchmark-affecting changes
- **WHEN** a maintainer prepares a release whose changes do not alter benchmark behavior or committed datasets
- **THEN** benchmark dry-run remains part of the mandatory checklist and a fresh committed baseline refresh is not required

#### Scenario: Benchmark-affecting change requires baseline refresh
- **WHEN** a maintainer prepares a release that changes benchmark logic, committed datasets, codec support, planner behavior, or benchmark result schema
- **THEN** the release guidance requires refreshing the committed baseline asset in addition to the mandatory dry-run verification
