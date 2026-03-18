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
The project SHALL include benchmark tooling that runs pack and unpack measurements for the default SFA user path against a canonical TAR baseline that uses `tar` piped through `zstd -3`. The default benchmark SHALL execute `sfa pack <input> <archive>` and `sfa unpack <archive> -C <output>` without overriding codec, compression level, thread count, bundle-target, small-file-threshold, or integrity parameters. Benchmark output MUST record wall time, files per second, MiB per second, output size, CPU usage, and RSS.

#### Scenario: Benchmark suite executes the default-path comparison
- **WHEN** the benchmark harness is run for the default benchmark workload
- **THEN** it executes the default SFA pack and unpack commands together with the canonical `tar | zstd -3` baseline on the same workload and stores comparable metrics for each phase

### Requirement: Test suites cover protocol, streaming, corruption, and safety baselines
Before the first v1 release, the project SHALL include automated tests for roundtrip correctness, fragmented sequential input, corruption rejection, path safety, golden archive compatibility, metadata roundtrip for supported Unix entries, and CLI behavior regressions for documented defaults, supported input-mode combinations, and expected usage failures. This verification coverage MUST include checks that supported restores preserve `mode` and `mtime` for regular files and directories, that the default unpack path leaves owner restoration disabled, and that link and safety scenarios remain represented in committed fixtures or tests.

#### Scenario: CI runs repository verification coverage
- **WHEN** repository tests are executed in CI
- **THEN** the suite includes protocol, streaming, corruption, security, golden-archive, metadata-restore, and CLI regression cases for the v1 feature set

### Requirement: Benchmark datasets are committed and documented for the default matrix
The repository SHALL provide a committed workload recipe for the default benchmark under repository-controlled benchmark assets rather than relying on a checked-in `100k+` input tree. The default benchmark workload MUST generate a deterministic `node_modules`-style deep directory tree without network access, MUST produce at least `100,000` regular files, and MUST document its generation inputs, directory-depth expectations, dominant file types, and stable size summary.

#### Scenario: Maintainer inspects the default benchmark workload definition
- **WHEN** a maintainer reviews the benchmark workload assets in a clean checkout
- **THEN** they can find the committed recipe, templates, and documentation needed to generate the default benchmark input tree offline

#### Scenario: Generated workload matches the target shape
- **WHEN** a maintainer generates the default benchmark workload from the committed recipe
- **THEN** the resulting input tree contains at least `100,000` regular files arranged in `node_modules`-style nested package subtrees rather than only flat repeated copies of a seed directory

### Requirement: Benchmark runner validates prerequisites before executing the default matrix
When the default benchmark is run in non-dry-run mode, the benchmark tooling MUST validate the required execution prerequisites before starting measurements. This validation MUST cover the requested `sfa` binary path, the presence and integrity of the committed workload recipe inputs, support for the canonical `tar + zstd -3` workflow on the current host, availability of any required temporary workspace for generating the workload, and creation of the archive and unpack output directories needed by each job. If any prerequisite is not met, the runner MUST fail with an actionable error before recording a partial comparison result set.

#### Scenario: Host cannot satisfy the default tar baseline prerequisites
- **WHEN** a maintainer runs the benchmark suite on a host where `tar` or `zstd` cannot execute the canonical `zstd -3` workflow
- **THEN** the runner reports which prerequisite is unmet and stops before executing the benchmark

#### Scenario: Workload recipe cannot be materialized
- **WHEN** a maintainer runs the benchmark suite and the committed workload recipe is missing, invalid, or cannot generate the required temporary input tree
- **THEN** the runner fails before benchmark execution with an actionable error that identifies the missing or invalid workload asset

### Requirement: Benchmark baseline results are recorded as repository assets
The repository SHALL include at least one committed, machine-readable benchmark result set for the default benchmark under `benches/results/`. Benchmark documentation and release guidance MUST identify the command used to generate the baseline, the workload recipe identity or generation parameters used for that run, the supported host environment for interpreting it, and the situations that require refreshing the committed result set.

#### Scenario: Release reviewer audits the default benchmark baseline
- **WHEN** a reviewer prepares an SFA release without rerunning the full benchmark suite locally
- **THEN** the repository contains a committed benchmark result asset and documentation that explain which default-path workload was used, how it was generated, how the commands were run, and when the result must be refreshed

### Requirement: SFA commands expose structured phase breakdown for benchmark consumers
When `sfa pack` or `sfa unpack` is invoked in a machine-readable stats mode, the command SHALL emit structured execution statistics with the existing total counters and the existing stable phase breakdown schema. Unpack statistics MUST continue to include diagnostic `phase_breakdown` fields `header`, `manifest`, `frame_read`, `decode`, `scatter`, and `restore_finalize`, and these fields SHALL remain valid even when they overlap in a parallel pipeline. Unpack statistics MUST additionally expose an additive `wall_breakdown` that identifies stable contiguous wall-time buckets for `setup`, `pipeline`, and `finalize`. The serialized `wall_breakdown` values MUST sum exactly to the reported unpack `duration_ms`. Dry-run execution MUST NOT fabricate measured phase or wall-breakdown durations.

#### Scenario: Reader-based unpack emits both additive and diagnostic breakdowns
- **WHEN** a maintainer runs `sfa unpack` in machine-readable stats mode using stdin or a local file without `--dry-run`
- **THEN** the command output contains the same total counters, measured diagnostic `header`, `manifest`, `frame_read`, `decode`, `scatter`, and `restore_finalize` durations, and a measured additive `wall_breakdown` for `setup`, `pipeline`, and `finalize`

#### Scenario: Serialized wall buckets reconcile to total duration
- **WHEN** a maintainer inspects the machine-readable output of a successful non-dry-run `sfa unpack`
- **THEN** the reported `wall_breakdown.setup_ms`, `wall_breakdown.pipeline_ms`, and `wall_breakdown.finalize_ms` sum exactly to the same `duration_ms` reported for the command

### Requirement: Benchmark runner records structured observability for executed commands
The benchmark runner SHALL continue to record command wall-time for every benchmark record and SHALL additionally persist structured observability fields when they are available. For SFA commands, the runner MUST embed the structured phase breakdown emitted by the CLI. For unpack records, the runner MUST preserve both the additive `wall_breakdown` and the overlapping diagnostic `phase_breakdown` emitted by the CLI so consumers can distinguish wall-time accounting from pipeline hotspot analysis. For both SFA and TAR commands executed on a supported host, the runner MUST record `user_cpu_ms`, `system_cpu_ms`, and `max_rss_kib` together with the identity of the sampler used to collect them. Benchmark and verification workflows for unpack MUST preserve the effective thread count and both classes of unpack timing needed to explain scaling and regressions. When resource observation is unavailable, the runner MUST preserve explicit unavailable semantics and a note explaining why rather than substituting zero values.

#### Scenario: Supported host captures resource metrics
- **WHEN** a maintainer executes the benchmark matrix on a host with a supported resource sampler
- **THEN** each executed benchmark record contains command wall-time, sampler identity, `user_cpu_ms`, `system_cpu_ms`, and `max_rss_kib`

#### Scenario: SFA unpack records additive and diagnostic timings
- **WHEN** the benchmark runner executes an SFA unpack command in non-dry-run mode
- **THEN** the resulting benchmark record contains the structured additive `wall_breakdown` and diagnostic `phase_breakdown` timings emitted by that command

#### Scenario: Unpack thread sweep remains auditable
- **WHEN** a maintainer runs a diagnostic unpack benchmark or thread sweep on a multi-bundle workload
- **THEN** the resulting records preserve the effective thread count, additive unpack wall buckets, and diagnostic unpack phase timings needed to compare one run against another

#### Scenario: Unsupported resource sampling remains explicit
- **WHEN** the benchmark runner executes on a host where resource sampling is not supported
- **THEN** the benchmark record still contains wall-time results and marks resource fields unavailable with an explanatory note instead of recording zero values

### Requirement: Benchmark and verification artifacts stay auditable after pipeline realignment
The benchmark runner SHALL continue to record command wall-time for every benchmark record and SHALL additionally persist the structured unpack observability emitted by the CLI. Verification and benchmark documentation for unpack SHALL describe the `node_modules`-style deep-directory workload used for default-path evidence, the difference between additive wall buckets and overlapping diagnostic phase windows, how repeated runs can warm caches on that workload, the effective thread count used by default or by explicit override, and the `.sfa-untrusted` behavior on strong trailer failures.

#### Scenario: Diagnostic unpack sweep remains comparable after benchmark repositioning
- **WHEN** a maintainer runs a diagnostic unpack benchmark or thread sweep on the representative `node_modules`-style workload after this benchmark redesign lands
- **THEN** the resulting artifacts preserve the thread-count and unpack timing fields needed to compare setup and scatter bottlenecks against prior runs

#### Scenario: Verification docs explain cache-sensitive default workload analysis
- **WHEN** a maintainer updates benchmark or verification documentation for unpack
- **THEN** the docs identify the representative `node_modules`-style workload, explain how to control cache warming when comparing repeated runs on that workload, and mention that strong trailer failures leave `.sfa-untrusted` in the output root even though the command returns an integrity error

### Requirement: Committed baseline assets preserve observability coverage guidance
The repository SHALL keep the committed benchmark baseline readable after observability fields are added, and the benchmark-facing documentation MUST identify which environments are expected to populate resource metrics, how unavailable metrics are represented, and when the committed baseline must be refreshed after observability-related runner or schema changes.

#### Scenario: Reviewer audits the current observability baseline
- **WHEN** a reviewer inspects the committed benchmark baseline and benchmark documentation
- **THEN** they can determine which records are expected to contain phase and resource observations, how missing values are represented, and when the baseline requires regeneration

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

### Requirement: Release verification checklist remains executable
The repository SHALL define a release-verification checklist for SFA that includes `cargo fmt --all --check`, `cargo test --workspace`, `bash tests/scripts/run_protocol_smoke.sh`, `bash tests/scripts/run_streaming_smoke.sh`, `bash tests/scripts/run_safety_smoke.sh`, `bash tests/scripts/run_roundtrip_smoke.sh`, and `cargo run -p sfa-bench --bin tar_vs_sfa -- --dry-run --output benches/results/latest.json`. The commands named in release guidance MUST succeed from a clean workspace on a supported development host before a version tag is created.

#### Scenario: Maintainer runs the documented release checklist
- **WHEN** a maintainer prepares an SFA release candidate from a clean checkout
- **THEN** the release-facing documentation names one authoritative checklist and each listed verification command completes successfully

#### Scenario: Formatting regression blocks release readiness
- **WHEN** a repository change causes `cargo fmt --all --check` to fail while the release checklist is being executed
- **THEN** the release-preparation workflow fails before a version tag is created and the formatting drift is treated as a blocker rather than an informational warning

### Requirement: Benchmark release guidance distinguishes mandatory dry-run from conditional baseline refresh
Release guidance for benchmark verification SHALL require the benchmark dry-run command on every release-preparation pass and MUST describe committed baseline refresh as conditional on benchmark-affecting changes such as default workload recipe changes, default command-profile changes, runner logic changes, benchmark result schema changes, unpack observability changes, or supported benchmark host/toolchain changes.

#### Scenario: Release prep without default-benchmark changes
- **WHEN** a maintainer prepares a release whose changes do not alter the default benchmark workload, command profile, runner behavior, or result schema
- **THEN** benchmark dry-run remains part of the mandatory checklist and a fresh committed baseline refresh is not required

#### Scenario: Default benchmark contract changes require baseline refresh
- **WHEN** a maintainer prepares a release that changes the default benchmark workload recipe, the canonical `tar + zstd -3` baseline, default SFA command profile, runner behavior, or benchmark result schema
- **THEN** the release guidance requires refreshing the committed baseline asset in addition to the mandatory dry-run verification

