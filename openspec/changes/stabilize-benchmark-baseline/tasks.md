## 1. Commit Real Benchmark Datasets

- [x] 1.1 Replace the placeholder inputs under `tests/fixtures/datasets/small-text/input`, `small-binary/input`, and `large-control/input` with committed benchmark-ready files.
- [x] 1.2 Add dataset documentation that explains each dataset's purpose, construction or provenance, and stable size summary for reviewers.

## 2. Harden Benchmark Execution

- [x] 2.1 Add benchmark preflight checks for dataset presence, `sfa` binary availability, and `tar + same codec` support before non-dry-run execution.
- [x] 2.2 Update the runner to create and reset archive/unpack working directories per job and return actionable errors when prerequisites are not met.
- [x] 2.3 Extend the benchmark report and helper scripts with the command and environment metadata needed to interpret committed baseline results.

## 3. Record The First Baseline

- [x] 3.1 Run the default benchmark matrix on a supported environment and commit the first machine-readable result set under `benches/results/`.
- [x] 3.2 Add a repository-level verification step or documented command that confirms the committed benchmark result asset remains readable and aligned with the current runner.

## 4. Document Benchmark And Release Expectations

- [x] 4.1 Update benchmark-facing docs to describe the committed datasets, supported execution environment, and baseline regeneration workflow.
- [x] 4.2 Update release and roadmap docs so they state when the committed benchmark baseline must be refreshed and how it participates in the release gate.
