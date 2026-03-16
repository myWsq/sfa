# small-tree-lz4-strong

This is the first canonical protocol fixture for the frozen SFA v1 format.

It freezes:

- manifest-first archive ordering
- a single aggregate data bundle for small regular files
- `lz4` data frames with the default `zstd` manifest codec
- `strong` integrity mode with a committed trailer
- directory, symlink, and explicit empty-file entry encoding

Generation command:

```bash
bash tests/scripts/generate_golden_fixture.sh \
  tests/fixtures/golden/small-tree-lz4-strong/input \
  tests/fixtures/golden/small-tree-lz4-strong/archive.sfa \
  tests/fixtures/golden/small-tree-lz4-strong/manifest.json \
  tests/fixtures/golden/small-tree-lz4-strong/stats.json \
  --codec lz4 \
  --threads 1 \
  --bundle-target-bytes 4194304 \
  --small-file-threshold 262144 \
  --integrity strong
```

The input tree is committed so reviewers can inspect exactly which filesystem shape produced the frozen archive.

The fixture generator normalizes `uid`, `gid`, and `mtime` to fixed values before packing so `manifest.json` and `stats.json` remain reproducible across development machines.
