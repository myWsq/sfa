## MODIFIED Requirements

### Requirement: CLI exposes pack and unpack workflows
The `sfa` CLI SHALL provide `pack` and `unpack` subcommands for directory-to-archive and archive-to-directory workflows. `pack` MUST accept the input directory, output archive path, codec, threads, bundle planning parameters, integrity mode, and metadata policy flags. `unpack` MUST accept archive input, output root, optional thread override, overwrite policy, integrity policy, and owner restore policy, and MUST apply an explicit thread override to the effective unpack worker count used by the restore pipeline.

#### Scenario: User runs pack with explicit parameters
- **WHEN** a user runs `sfa pack ./assets ./assets.sfa --codec lz4 --threads 8`
- **THEN** the CLI validates the arguments, creates a `.sfa` archive, and exits successfully if packing completes

#### Scenario: User runs unpack with explicit thread override
- **WHEN** a user runs `sfa unpack ./assets.sfa -C ./out --threads 4`
- **THEN** the unpack command executes with effective worker count `4` and successful stats output reports `threads: 4`

### Requirement: SFA commands expose structured phase breakdown for benchmark consumers
When `sfa pack` or `sfa unpack` is invoked in a machine-readable stats mode, the command SHALL emit structured execution statistics that include the existing total counters plus a stable phase breakdown for the real execution path. Pack statistics MUST include `scan`, `plan`, `encode`, and `write` durations. Unpack statistics MUST include `header`, `manifest`, `frame_read`, `decode`, `scatter`, and `restore_finalize` durations. Dry-run execution MUST NOT fabricate measured phase durations.

#### Scenario: Pack execution emits phase timing breakdown
- **WHEN** a maintainer runs `sfa pack` in machine-readable stats mode without `--dry-run`
- **THEN** the command output includes total execution statistics and measured durations for `scan`, `plan`, `encode`, and `write`

#### Scenario: Unpack execution emits split restore timing breakdown
- **WHEN** a maintainer runs `sfa unpack` in machine-readable stats mode without `--dry-run`
- **THEN** the command output includes total execution statistics and measured durations for `header`, `manifest`, `frame_read`, `decode`, `scatter`, and `restore_finalize`

#### Scenario: Dry-run does not pretend to measure phases
- **WHEN** a maintainer runs `sfa pack` or `sfa unpack` in machine-readable stats mode with `--dry-run`
- **THEN** the command output leaves phase durations unavailable or explicitly marks them as unavailable instead of emitting synthetic measured values

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
