## MODIFIED Requirements

### Requirement: Benchmark tooling compares SFA against tar with the same codec
The project SHALL include benchmark tooling that runs pack and unpack measurements for the default SFA user path against a canonical TAR baseline that uses `tar` piped through `zstd -3`. The default benchmark SHALL execute `sfa pack <input> <archive>` and `sfa unpack <archive> -C <output>` without overriding codec, compression level, thread count, bundle-target, small-file-threshold, or integrity parameters. Benchmark output MUST record wall time, files per second, MiB per second, output size, CPU usage, and RSS.

#### Scenario: Benchmark suite executes the default-path comparison
- **WHEN** the benchmark harness is run for the default benchmark workload
- **THEN** it executes the default SFA pack and unpack commands together with the canonical `tar | zstd -3` baseline on the same workload and stores comparable metrics for each phase

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

### Requirement: Benchmark and verification artifacts stay auditable after pipeline realignment
The benchmark runner SHALL continue to record command wall-time for every benchmark record and SHALL additionally persist the structured unpack observability emitted by the CLI. Verification and benchmark documentation for unpack SHALL describe the `node_modules`-style deep-directory workload used for default-path evidence, the difference between additive wall buckets and overlapping diagnostic phase windows, how repeated runs can warm caches on that workload, the effective thread count used by default or by explicit override, and the `.sfa-untrusted` behavior on strong trailer failures.

#### Scenario: Diagnostic unpack sweep remains comparable after benchmark repositioning
- **WHEN** a maintainer runs a diagnostic unpack benchmark or thread sweep on the representative `node_modules`-style workload after this benchmark redesign lands
- **THEN** the resulting artifacts preserve the thread-count and unpack timing fields needed to compare setup and scatter bottlenecks against prior runs

#### Scenario: Verification docs explain cache-sensitive default workload analysis
- **WHEN** a maintainer updates benchmark or verification documentation for unpack
- **THEN** the docs identify the representative `node_modules`-style workload, explain how to control cache warming when comparing repeated runs on that workload, and mention that strong trailer failures leave `.sfa-untrusted` in the output root even though the command returns an integrity error

### Requirement: Benchmark release guidance distinguishes mandatory dry-run from conditional baseline refresh
Release guidance for benchmark verification SHALL require the benchmark dry-run command on every release-preparation pass and MUST describe committed baseline refresh as conditional on benchmark-affecting changes such as default workload recipe changes, default command-profile changes, runner logic changes, benchmark result schema changes, unpack observability changes, or supported benchmark host/toolchain changes.

#### Scenario: Release prep without default-benchmark changes
- **WHEN** a maintainer prepares a release whose changes do not alter the default benchmark workload, command profile, runner behavior, or result schema
- **THEN** benchmark dry-run remains part of the mandatory checklist and a fresh committed baseline refresh is not required

#### Scenario: Default benchmark contract changes require baseline refresh
- **WHEN** a maintainer prepares a release that changes the default benchmark workload recipe, the canonical `tar + zstd -3` baseline, default SFA command profile, runner behavior, or benchmark result schema
- **THEN** the release guidance requires refreshing the committed baseline asset in addition to the mandatory dry-run verification
