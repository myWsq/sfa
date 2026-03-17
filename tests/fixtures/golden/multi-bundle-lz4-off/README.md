# multi-bundle-lz4-off

This canonical protocol fixture expands the frozen SFA v1 corpus with an `integrity=off` archive whose small files are forced across multiple aggregate bundles.

It freezes:

- `lz4` data frames with integrity disabled
- absence of the optional trailer under `off` integrity
- more than one aggregate bundle / `DataFrame` in the committed fixture corpus
- deterministic multi-file ordering across several small regular files

Generation command:

```bash
bash tests/scripts/generate_golden_fixture.sh \
  tests/fixtures/golden/multi-bundle-lz4-off/input \
  tests/fixtures/golden/multi-bundle-lz4-off/archive.sfa \
  tests/fixtures/golden/multi-bundle-lz4-off/manifest.json \
  tests/fixtures/golden/multi-bundle-lz4-off/stats.json \
  --codec lz4 \
  --threads 1 \
  --bundle-target-bytes 64 \
  --small-file-threshold 1024 \
  --integrity off
```

Coverage notes:

- The four `chunks/part-*.txt` files are intentionally larger than the tiny bundle target, so the aggregate planner emits multiple bundles in a stable order.
- This fixture exists to keep the `off` integrity mode and multi-bundle manifest planning represented in the canonical protocol corpus.
