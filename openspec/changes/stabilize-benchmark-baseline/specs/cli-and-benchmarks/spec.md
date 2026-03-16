## ADDED Requirements

### Requirement: Benchmark datasets are committed and documented for the default matrix
The repository SHALL provide committed input trees for the default benchmark matrix under `tests/fixtures/datasets/small-text/input`, `tests/fixtures/datasets/small-binary/input`, and `tests/fixtures/datasets/large-control/input`. Each dataset MUST contain real benchmark content rather than placeholders, and each dataset directory MUST include accompanying documentation that identifies the dataset purpose, construction or provenance, and a stable summary of its scale.

#### Scenario: Maintainer inspects the default datasets
- **WHEN** a maintainer reviews the benchmark fixture directories in a clean checkout
- **THEN** each default dataset contains committed input files and dataset documentation without requiring an external download step

### Requirement: Benchmark runner validates prerequisites before executing the default matrix
When the default benchmark matrix is run in non-dry-run mode, the benchmark tooling MUST validate the required execution prerequisites before starting measurements. This validation MUST cover the requested `sfa` binary path, the presence of committed dataset inputs, support for the requested `tar + same codec` workflow on the current host, and creation of the archive and unpack output directories needed by each job. If any prerequisite is not met, the runner MUST fail with an actionable error before recording a partial comparison result set.

#### Scenario: Host cannot satisfy the tar codec prerequisites
- **WHEN** a maintainer runs the benchmark suite on a host whose `tar` implementation does not support one of the requested codecs
- **THEN** the runner reports which prerequisite is unmet and stops before executing the affected benchmark matrix

### Requirement: Benchmark baseline results are recorded as repository assets
The repository SHALL include at least one committed, machine-readable benchmark result set for the default matrix under `benches/results/`. Benchmark documentation and release guidance MUST identify the command used to generate the baseline, the environment constraints for interpreting it, and the situations that require refreshing the committed result set.

#### Scenario: Release reviewer audits the performance baseline
- **WHEN** a reviewer prepares an SFA release without rerunning the benchmark suite locally
- **THEN** the repository contains a committed benchmark result asset and documentation that explain how it was generated and when it must be refreshed
