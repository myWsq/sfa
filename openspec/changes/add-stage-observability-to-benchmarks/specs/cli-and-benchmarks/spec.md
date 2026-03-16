## ADDED Requirements

### Requirement: SFA commands expose structured phase breakdown for benchmark consumers
When `sfa pack` or `sfa unpack` is invoked in a machine-readable stats mode, the command SHALL emit structured execution statistics that include the existing total counters plus a stable phase breakdown for the real execution path. Pack statistics MUST include `scan`, `plan`, `encode`, and `write` durations. Unpack statistics MUST include `header`, `manifest`, `decode_and_scatter`, and `restore_finalize` durations. Dry-run execution MUST NOT fabricate measured phase durations.

#### Scenario: Pack execution emits phase timing breakdown
- **WHEN** a maintainer runs `sfa pack` in machine-readable stats mode without `--dry-run`
- **THEN** the command output includes total execution statistics and measured durations for `scan`, `plan`, `encode`, and `write`

#### Scenario: Dry-run does not pretend to measure phases
- **WHEN** a maintainer runs `sfa pack` or `sfa unpack` in machine-readable stats mode with `--dry-run`
- **THEN** the command output leaves phase durations unavailable or explicitly marks them as unavailable instead of emitting synthetic measured values

### Requirement: Benchmark runner records structured observability for executed commands
The benchmark runner SHALL continue to record command wall-time for every benchmark record and SHALL additionally persist structured observability fields when they are available. For SFA commands, the runner MUST embed the structured phase breakdown emitted by the CLI. For both SFA and TAR commands executed on a supported host, the runner MUST record `user_cpu_ms`, `system_cpu_ms`, and `max_rss_kib` together with the identity of the sampler used to collect them. When resource observation is unavailable, the runner MUST preserve explicit unavailable semantics and a note explaining why rather than substituting zero values.

#### Scenario: Supported host captures resource metrics
- **WHEN** a maintainer executes the benchmark matrix on a host with a supported resource sampler
- **THEN** each executed benchmark record contains command wall-time, sampler identity, `user_cpu_ms`, `system_cpu_ms`, and `max_rss_kib`

#### Scenario: SFA records retain internal phase breakdown
- **WHEN** the benchmark runner executes an SFA pack or unpack command in non-dry-run mode
- **THEN** the resulting benchmark record contains the structured phase breakdown emitted by that command

#### Scenario: Unsupported resource sampling remains explicit
- **WHEN** the benchmark runner executes on a host where resource sampling is not supported
- **THEN** the benchmark record still contains wall-time results and marks resource fields unavailable with an explanatory note instead of recording zero values

### Requirement: Committed baseline assets preserve observability coverage guidance
The repository SHALL keep the committed benchmark baseline readable after observability fields are added, and the benchmark-facing documentation MUST identify which environments are expected to populate resource metrics, how unavailable metrics are represented, and when the committed baseline must be refreshed after observability-related runner or schema changes.

#### Scenario: Reviewer audits the current observability baseline
- **WHEN** a reviewer inspects the committed benchmark baseline and benchmark documentation
- **THEN** they can determine which records are expected to contain phase and resource observations, how missing values are represented, and when the baseline requires regeneration
