# SFA Format v1 Freeze Review

Freeze date: 2026-03-16

## Decision

SFA v1.0 的协议冻结于 2026-03-16 生效。自该日期起，[format-v1.md](/Users/bytedance/github/sfa/spec/format-v1.md) 是仓库内唯一权威的协议定义，任何影响 wire format、顺序读取语义、完整性字段或 canonical fixture 解释方式的修改，都需要通过新的 OpenSpec change 进入评审。

## Frozen Inputs

The freeze decision is based on these committed repository assets:

- [spec/format-v1.md](/Users/bytedance/github/sfa/spec/format-v1.md)
- [tests/fixtures/golden/small-tree-lz4-strong/archive.sfa](/Users/bytedance/github/sfa/tests/fixtures/golden/small-tree-lz4-strong/archive.sfa)
- [tests/fixtures/golden/small-tree-lz4-strong/manifest.json](/Users/bytedance/github/sfa/tests/fixtures/golden/small-tree-lz4-strong/manifest.json)
- [tests/fixtures/golden/small-tree-lz4-strong/stats.json](/Users/bytedance/github/sfa/tests/fixtures/golden/small-tree-lz4-strong/stats.json)
- [tests/scripts/run_protocol_smoke.sh](/Users/bytedance/github/sfa/tests/scripts/run_protocol_smoke.sh)

## What Is Frozen

The repository now treats the following as fixed for SFA v1.0:

- archive ordering: `HeaderV1 + ManifestSection + DataFrame* + TrailerV1?`
- little-endian field encoding and fixed struct sizes
- header, manifest, frame, and trailer byte layouts documented in `spec/format-v1.md`
- manifest-first restore planning without compressed-offset backfilling
- per-frame raw-bundle hashing and strong trailer verification semantics
- the canonical interpretation of a small aggregate bundle fixture with directories, symlink, and explicit empty-file entry encoding

## Deferred Follow-Up

The following work is explicitly outside this freeze gate and remains open:

- populating real benchmark datasets under `tests/fixtures/datasets/`
- making the `tar + same codec` baseline reproducible across environments
- recording the first release-grade benchmark result set under `benches/results/`
- adding more golden fixtures for large-file chunking, hardlinks, and broader corruption coverage
- evaluating post-v1.0 metadata extensions such as xattrs and ACLs

## Reopen Conditions

Start a new change before altering the frozen protocol if any of the following becomes necessary:

- changing field sizes, offsets, or enum values in the v1 archive layout
- changing which bytes are covered by manifest, frame, or trailer integrity checks
- changing the meaning of committed golden fixture metadata
- redefining compatibility expectations for sequential reads or restore-path safety

## Notes

Benchmark effectiveness is intentionally not part of this freeze record. Performance remains a release concern, but protocol stability is gated first so later benchmark work can measure against a stable compatibility target.
