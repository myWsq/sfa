## Why

SFA v1 的实现主链路已经存在，但协议文本、golden fixtures 和协议评审记录还没有收敛成可冻结、可回归、可追溯的发布资产。现在需要先把兼容性边界固定下来，避免后续 benchmark 和实现优化继续牵动 wire format 本身。

## What Changes

- 冻结 `spec/format-v1.md`，把当前 v1 wire format、字段语义、顺序读取约束和完整性语义收敛为权威协议文本。
- 提交第一批 canonical golden fixtures，包括 archive、manifest dump 和统计摘要，用于后续协议回归。
- 为协议冻结补充可追溯的评审记录，明确冻结输入、结论和未决的后续工作边界。
- 将 golden fixtures 接入现有 protocol/smoke 校验入口，确保后续改动不会无声漂移协议。
- 明确本 change 不包含真实 benchmark 数据集和 `tar + same codec` 正式性能基线，这些工作留给后续 change。

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `archive-format-v1`: 将 v1 协议定义提升为冻结后的权威 spec，并要求配套 canonical golden fixtures 与可追溯的协议评审记录。

## Impact

- 影响协议文档与冻结后的规范入口，主要是 `spec/format-v1.md` 和相关说明文档。
- 影响 golden fixture 资产与生成/校验脚本，主要位于 `tests/fixtures/golden/` 和 `tests/scripts/`。
- 影响协议回归与发布收口流程，需要在 smoke checks、README/ROADMAP 或相邻文档中明确冻结状态和资产位置。
- 不改变 `pack` / `unpack` 的目标范围，但可能暴露实现与冻结协议之间的偏差并要求后续修正。
