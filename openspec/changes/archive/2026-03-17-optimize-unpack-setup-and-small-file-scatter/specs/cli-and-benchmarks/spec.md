## MODIFIED Requirements

### Requirement: Benchmark and verification artifacts stay auditable after pipeline realignment
The benchmark runner SHALL continue to record command wall-time for every benchmark record and SHALL additionally persist the structured unpack phase breakdown emitted by the CLI. Verification and thread-sweep documentation for unpack SHALL describe the three-stage reader/decode/scatter execution model, the effective thread count used for diagnostics, a representative multi-bundle small-file workload used to reason about setup-vs-scatter bottlenecks, and the `.sfa-untrusted` behavior on strong trailer failures.

#### Scenario: Diagnostic unpack sweep remains comparable after small-file optimization
- **WHEN** a maintainer runs a diagnostic unpack benchmark or thread sweep on a representative multi-bundle small-file workload after this optimization work lands
- **THEN** the resulting artifacts preserve the same thread-count and phase-breakdown fields needed to compare setup and scatter bottlenecks against prior runs

#### Scenario: Verification docs explain representative small-file hotspot analysis
- **WHEN** a maintainer updates benchmark or verification documentation for unpack
- **THEN** the docs identify the representative small-file workload used for diagnosis, explain how to interpret setup-versus-scatter observations from that workload, and mention that strong trailer failures leave `.sfa-untrusted` in the output root even though the command returns an integrity error
