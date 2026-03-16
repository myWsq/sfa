## 1. Extend Stats And CLI Contracts

- [x] 1.1 Add phase-breakdown data structures to `sfa-core` pack/unpack stats with explicit unavailable semantics for dry-run or unsupported observations.
- [x] 1.2 Add a machine-readable stats output mode to `sfa-cli` pack/unpack commands while keeping the current human-readable summary as the default behavior.

## 2. Instrument Stable Pack / Unpack Phases

- [x] 2.1 Instrument `sfa-unixfs` pack execution to record stable `scan`, `plan`, `encode`, and `write` timings and return them through the shared stats model.
- [x] 2.2 Instrument `sfa-unixfs` unpack execution to record stable `header`, `manifest`, `decode_and_scatter`, and `restore_finalize` timings and return them through the shared stats model.

## 3. Enrich Benchmark Records And Resource Sampling

- [x] 3.1 Extend `sfa-bench` report and parsing logic so SFA benchmark records include the structured CLI phase breakdown in non-dry-run execution.
- [x] 3.2 Implement benchmark runner resource sampling, supported-environment detection, and explicit unavailable-field handling for hosts that cannot provide CPU / RSS observations.
- [x] 3.3 Add or update tests that validate the benchmark report schema, committed baseline readability, and the new observability-field expectations.

## 4. Refresh Baseline And Documentation

- [x] 4.1 Regenerate the committed benchmark baseline asset with the new observability fields on a supported environment and ensure the repository verification steps still pass.
- [x] 4.2 Update benchmark, roadmap, README, and release documentation to explain the new stage/resource metrics, supported environments, unavailable-field semantics, and baseline refresh rules.
