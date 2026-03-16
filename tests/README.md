# Test Suites

This tree defines the verification skeleton for SFA v1.

## Planned suites

- `tests/golden`: protocol compatibility fixtures and canonical dumps
- `tests/corruption`: invalid header/manifest/frame/trailer behavior
- `tests/streaming`: fragmented sequential-read decode scenarios
- `tests/safety`: output-root path and node safety rules
- `tests/integration`: roundtrip semantics for supported Unix entries

The implementation crates can progressively migrate these docs into executable tests.
