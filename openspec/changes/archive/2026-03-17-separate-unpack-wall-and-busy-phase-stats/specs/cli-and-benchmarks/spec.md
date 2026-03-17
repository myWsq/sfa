## MODIFIED Requirements

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
The benchmark runner SHALL continue to record command wall-time for every benchmark record and SHALL additionally persist the structured unpack observability emitted by the CLI. Verification and thread-sweep documentation for unpack SHALL describe the three-stage reader/decode/scatter execution model, the difference between additive wall buckets and overlapping diagnostic phase windows, the effective thread count used for diagnostics, and the `.sfa-untrusted` behavior on strong trailer failures.

#### Scenario: Diagnostic unpack sweep remains comparable after schema split
- **WHEN** a maintainer runs a diagnostic unpack benchmark or thread sweep on a multi-bundle workload after additive wall buckets are introduced
- **THEN** the resulting records preserve the same thread-count and diagnostic phase fields as before while additionally recording additive wall buckets that explain total unpack wall-time

#### Scenario: Verification docs describe additive and diagnostic semantics
- **WHEN** a maintainer updates benchmark or verification documentation for unpack
- **THEN** the docs explain that `wall_breakdown` is the additive wall-time accounting view, `phase_breakdown` is the overlapping diagnostic view, and strong trailer failures leave `.sfa-untrusted` in the output root even though the command returns an integrity error
