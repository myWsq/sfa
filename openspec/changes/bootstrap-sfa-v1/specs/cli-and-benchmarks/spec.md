## ADDED Requirements

### Requirement: CLI exposes pack and unpack workflows
The `sfa` CLI SHALL provide `pack` and `unpack` subcommands for directory-to-archive and archive-to-directory workflows. `pack` MUST accept the input directory, output archive path, codec, threads, bundle planning parameters, integrity mode, and metadata policy flags. `unpack` MUST accept archive input, output root, optional thread override, overwrite policy, integrity policy, and owner restore policy.

#### Scenario: User runs pack with explicit parameters
- **WHEN** a user runs `sfa pack ./assets ./assets.sfa --codec lz4 --threads 8`
- **THEN** the CLI validates the arguments, creates a `.sfa` archive, and exits successfully if packing completes

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
