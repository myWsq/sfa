# Golden Fixtures

Protocol fixture assets live under [tests/fixtures/golden](/Users/bytedance/github/sfa/tests/fixtures/golden).

Current canonical fixture set:

- `small-tree-lz4-strong`: a small aggregated tree with nested directories, an empty file, and a symlink, packed with `lz4` data frames and `strong` integrity.

These fixtures are consumed by `tests/scripts/run_protocol_smoke.sh` and anchor the frozen SFA v1 wire format.
