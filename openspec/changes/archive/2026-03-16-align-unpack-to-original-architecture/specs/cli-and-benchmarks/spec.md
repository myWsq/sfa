## MODIFIED Requirements

### Requirement: CLI exposes pack and unpack workflows
The `sfa` CLI SHALL provide `pack` and `unpack` subcommands for directory-to-archive and archive-to-directory workflows. `unpack` MUST accept a filesystem path or `-` as archive input, MUST support sync-stream unpack through stdin when `-` is used, MUST apply an explicit thread override to the effective unpack worker count, and MUST reject `stdin` dry-run requests rather than fabricating stream-replay behavior.

#### Scenario: User unpacks from stdin
- **WHEN** a user runs `cat ./assets.sfa | sfa unpack - -C ./out`
- **THEN** the CLI reads archive bytes from stdin, executes the normal unpack pipeline, and exits successfully if restoration completes

#### Scenario: User requests dry-run from stdin
- **WHEN** a user runs `cat ./assets.sfa | sfa unpack - -C ./out --dry-run`
- **THEN** the CLI fails with a usage error explaining that dry-run is not supported for stdin input

### Requirement: SFA commands expose structured phase breakdown for benchmark consumers
When `sfa pack` or `sfa unpack` is invoked in a machine-readable stats mode, the command SHALL emit structured execution statistics with the existing total counters and the existing stable phase breakdown schema. Unpack statistics MUST continue to include `header`, `manifest`, `frame_read`, `decode`, `scatter`, and `restore_finalize`. Dry-run execution MUST NOT fabricate measured phase durations.

#### Scenario: Reader-based unpack emits the same phase schema
- **WHEN** a maintainer runs `sfa unpack` in machine-readable stats mode using stdin or a local file without `--dry-run`
- **THEN** the command output contains the same total counters and measured `header`, `manifest`, `frame_read`, `decode`, `scatter`, and `restore_finalize` durations

### Requirement: Benchmark and verification artifacts stay auditable after pipeline realignment
The benchmark runner SHALL continue to record command wall-time for every benchmark record and SHALL additionally persist the structured unpack phase breakdown emitted by the CLI. Verification and thread-sweep documentation for unpack SHALL describe the three-stage reader/decode/scatter execution model, the effective thread count used for diagnostics, and the `.sfa-untrusted` behavior on strong trailer failures.

#### Scenario: Diagnostic unpack sweep remains comparable after pipeline split
- **WHEN** a maintainer runs a diagnostic unpack benchmark or thread sweep on a multi-bundle workload after the pipeline is realigned
- **THEN** the resulting records preserve the same thread-count and phase-breakdown fields so the new results can be compared against prior baselines

#### Scenario: Verification docs describe strong failure marker
- **WHEN** a maintainer updates benchmark or verification documentation for unpack
- **THEN** the docs mention that strong trailer failures leave `.sfa-untrusted` in the output root even though the command returns an integrity error
