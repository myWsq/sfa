# cli-and-benchmarks Specification

## Purpose
TBD - created by archiving change bootstrap-sfa-v1. Update Purpose after archive.
## Requirements
### Requirement: CLI exposes pack and unpack workflows
The `sfa` CLI SHALL provide `pack` and `unpack` subcommands for directory-to-archive and archive-to-directory workflows. `unpack` MUST accept a filesystem path or `-` as archive input, MUST support sync-stream unpack through stdin when `-` is used, MUST apply an explicit thread override to the effective unpack worker count, and MUST reject `stdin` dry-run requests rather than fabricating stream-replay behavior.

#### Scenario: User unpacks from stdin
- **WHEN** a user runs `cat ./assets.sfa | sfa unpack - -C ./out`
- **THEN** the CLI reads archive bytes from stdin, executes the normal unpack pipeline, and exits successfully if restoration completes

#### Scenario: User requests dry-run from stdin
- **WHEN** a user runs `cat ./assets.sfa | sfa unpack - -C ./out --dry-run`
- **THEN** the CLI fails with a usage error explaining that dry-run is not supported for stdin input

### Requirement: CLI surfaces actionable failures
The CLI MUST print human-readable error messages and exit non-zero for invalid paths, unsupported archives, read or write failures, safety violations, and integrity mismatches.

#### Scenario: User unpacks a corrupted archive
- **WHEN** a user runs `sfa unpack ./broken.sfa -C ./out`
- **THEN** the CLI reports an integrity or parse failure and returns a non-zero exit status

### Requirement: CLI reports throughput-oriented execution statistics
On successful pack and unpack operations, the CLI SHALL report codec, thread count, bundle planning parameters, entry count, bundle count, raw bytes, encoded bytes, duration, files per second, and MiB per second in a stable summary format.

#### Scenario: Pack completes successfully
- **WHEN** a pack command finishes without error
- **THEN** the CLI prints a summary that includes throughput and archive-structure statistics needed for tuning and comparison

### Requirement: Benchmark tooling compares SFA against tar with the same codec
The project SHALL include benchmark tooling that runs pack and unpack measurements against `tar + same codec` for at least a small-text dataset, a small-binary mixed dataset, and a large-file control dataset. Benchmark output MUST record wall time, files per second, MiB per second, output size, CPU usage, and RSS.

#### Scenario: Benchmark suite executes the comparison set
- **WHEN** the benchmark harness is run for the default dataset set
- **THEN** it executes both SFA and `tar + same codec` workflows and stores comparable metrics for each dataset and codec combination

### Requirement: Test suites cover protocol, streaming, corruption, and safety baselines
Before the first v1 release, the project SHALL include automated tests for roundtrip correctness, fragmented sequential input, corruption rejection, path safety, and golden archive compatibility.

#### Scenario: CI runs protocol regression coverage
- **WHEN** repository tests are executed in CI
- **THEN** the suite includes protocol, streaming, corruption, security, and golden-archive cases for the v1 feature set

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

### Requirement: SFA commands expose structured phase breakdown for benchmark consumers
When `sfa pack` or `sfa unpack` is invoked in a machine-readable stats mode, the command SHALL emit structured execution statistics with the existing total counters and the existing stable phase breakdown schema. Unpack statistics MUST continue to include `header`, `manifest`, `frame_read`, `decode`, `scatter`, and `restore_finalize`. Dry-run execution MUST NOT fabricate measured phase durations.

#### Scenario: Reader-based unpack emits the same phase schema
- **WHEN** a maintainer runs `sfa unpack` in machine-readable stats mode using stdin or a local file without `--dry-run`
- **THEN** the command output contains the same total counters and measured `header`, `manifest`, `frame_read`, `decode`, `scatter`, and `restore_finalize` durations

### Requirement: Benchmark runner records structured observability for executed commands
The benchmark runner SHALL continue to record command wall-time for every benchmark record and SHALL additionally persist structured observability fields when they are available. For SFA commands, the runner MUST embed the structured phase breakdown emitted by the CLI. For both SFA and TAR commands executed on a supported host, the runner MUST record `user_cpu_ms`, `system_cpu_ms`, and `max_rss_kib` together with the identity of the sampler used to collect them. Benchmark and verification workflows for unpack MUST preserve the effective thread count and split unpack phase timings needed to explain scaling and regressions. When resource observation is unavailable, the runner MUST preserve explicit unavailable semantics and a note explaining why rather than substituting zero values.

#### Scenario: Supported host captures resource metrics
- **WHEN** a maintainer executes the benchmark matrix on a host with a supported resource sampler
- **THEN** each executed benchmark record contains command wall-time, sampler identity, `user_cpu_ms`, `system_cpu_ms`, and `max_rss_kib`

#### Scenario: SFA unpack records split restore phases
- **WHEN** the benchmark runner executes an SFA unpack command in non-dry-run mode
- **THEN** the resulting benchmark record contains the structured `header`, `manifest`, `frame_read`, `decode`, `scatter`, and `restore_finalize` timings emitted by that command

#### Scenario: Unpack thread sweep remains auditable
- **WHEN** a maintainer runs a diagnostic unpack benchmark or thread sweep on a multi-bundle workload
- **THEN** the resulting records preserve the effective thread count and split unpack phase timings needed to compare one run against another

#### Scenario: Unsupported resource sampling remains explicit
- **WHEN** the benchmark runner executes on a host where resource sampling is not supported
- **THEN** the benchmark record still contains wall-time results and marks resource fields unavailable with an explanatory note instead of recording zero values

### Requirement: Benchmark and verification artifacts stay auditable after pipeline realignment
The benchmark runner SHALL continue to record command wall-time for every benchmark record and SHALL additionally persist the structured unpack phase breakdown emitted by the CLI. Verification and thread-sweep documentation for unpack SHALL describe the three-stage reader/decode/scatter execution model, the effective thread count used for diagnostics, and the `.sfa-untrusted` behavior on strong trailer failures.

#### Scenario: Diagnostic unpack sweep remains comparable after pipeline split
- **WHEN** a maintainer runs a diagnostic unpack benchmark or thread sweep on a multi-bundle workload after the pipeline is realigned
- **THEN** the resulting records preserve the same thread-count and phase-breakdown fields so the new results can be compared against prior baselines

#### Scenario: Verification docs describe strong failure marker
- **WHEN** a maintainer updates benchmark or verification documentation for unpack
- **THEN** the docs mention that strong trailer failures leave `.sfa-untrusted` in the output root even though the command returns an integrity error

### Requirement: Committed baseline assets preserve observability coverage guidance
The repository SHALL keep the committed benchmark baseline readable after observability fields are added, and the benchmark-facing documentation MUST identify which environments are expected to populate resource metrics, how unavailable metrics are represented, and when the committed baseline must be refreshed after observability-related runner or schema changes.

#### Scenario: Reviewer audits the current observability baseline
- **WHEN** a reviewer inspects the committed benchmark baseline and benchmark documentation
- **THEN** they can determine which records are expected to contain phase and resource observations, how missing values are represented, and when the baseline requires regeneration

