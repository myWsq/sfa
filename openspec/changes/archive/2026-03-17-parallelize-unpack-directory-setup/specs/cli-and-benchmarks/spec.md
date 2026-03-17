## MODIFIED Requirements

### Requirement: Benchmark and verification artifacts stay auditable after pipeline realignment
The benchmark runner SHALL continue to record command wall-time for every benchmark record and SHALL additionally persist the structured unpack observability emitted by the CLI. Verification and thread-sweep documentation for unpack SHALL describe the three-stage reader/decode/scatter execution model, the difference between additive wall buckets and overlapping diagnostic phase windows, the representative multi-bundle small-file workload used to diagnose setup bottlenecks, how to control for cache warming when comparing repeated unpack runs, the effective thread count used for diagnostics, and the `.sfa-untrusted` behavior on strong trailer failures.

#### Scenario: Diagnostic unpack sweep remains comparable after setup optimization
- **WHEN** a maintainer runs a diagnostic unpack benchmark or thread sweep on a representative multi-bundle small-file workload after setup optimization work lands
- **THEN** the resulting artifacts preserve the thread-count and unpack timing fields needed to compare setup bottlenecks against prior runs

#### Scenario: Verification docs explain cache-sensitive setup analysis
- **WHEN** a maintainer updates benchmark or verification documentation for unpack
- **THEN** the docs identify the representative setup-focused workload, explain how repeated runs can warm caches during setup comparisons, and mention that strong trailer failures leave `.sfa-untrusted` in the output root even though the command returns an integrity error
