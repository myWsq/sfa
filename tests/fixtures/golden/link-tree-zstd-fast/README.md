# link-tree-zstd-fast

This canonical protocol fixture expands the frozen SFA v1 corpus with link-oriented Unix semantics under `zstd` data frames and `fast` integrity.

It freezes:

- `zstd` data frames with the default `zstd` manifest codec
- `fast` integrity mode without a strong trailer hash
- directory, symlink, and hardlink entry encoding in one committed tree
- a single small aggregate bundle for link-heavy metadata coverage

Generation command:

```bash
bash tests/scripts/generate_golden_fixture.sh \
  tests/fixtures/golden/link-tree-zstd-fast/input \
  tests/fixtures/golden/link-tree-zstd-fast/archive.sfa \
  tests/fixtures/golden/link-tree-zstd-fast/manifest.json \
  tests/fixtures/golden/link-tree-zstd-fast/stats.json \
  --codec zstd \
  --threads 1 \
  --bundle-target-bytes 4194304 \
  --small-file-threshold 262144 \
  --integrity fast
```

Coverage notes:

- The `docs/master.txt` and `docs/master-peer.txt` paths exercise hardlink-master and hardlink-peer encoding.
- The `alias-to-master` symlink keeps symlink metadata coverage in the canonical corpus even when the existing `lz4 + strong` fixture changes later.
- The committed `input/` tree remains reviewable and small enough to audit alongside the decoded manifest snapshot.
