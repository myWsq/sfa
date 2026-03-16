## Why

SFA v1 的协议冻结和 MVP 主链路已经完成，但 benchmark 仍停留在 placeholder dataset 和 dry-run 级别，无法作为发布前的真实性能基线。现在需要把 benchmark 资产、执行环境和结果留痕收口下来，让 M2 的性能主链路具备可重复、可比较、可审计的基线。

## What Changes

- 用真实、稳定且可复用的内容替换 `tests/fixtures/datasets/` 下的 placeholder benchmark datasets，并补充每个数据集的来源说明与规模摘要。
- 收口 `tar + same codec` benchmark runner 的执行前提、输出目录处理和失败可诊断性，使默认矩阵能够在受支持环境中稳定重跑。
- 为默认 benchmark 矩阵生成并提交第一轮机器可读结果，形成仓库内可追溯的 baseline 记录。
- 同步更新 benchmark 相关文档和发版检查项，明确何时需要刷新结果、保留哪些产物、以及如何解释环境限制。

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `cli-and-benchmarks`: 将 benchmark 要求从“具备比较 harness”收口为“具备真实数据集、可重复执行约束、结果留痕和环境说明”的发布前性能基线。

## Impact

- 影响 `crates/sfa-bench` 的 benchmark matrix、runner、report 输出和错误处理。
- 影响 `tests/fixtures/datasets/`、`benches/results/`、`benches/scripts/` 等 benchmark 资产目录。
- 影响 `README.md`、`ROADMAP.md`、`RELEASING.md`、`spec/verification-and-benchmark.md` 等与 benchmark 基线和发版闸口相关的文档。
- 不改变冻结的 `.sfa` v1 wire format，也不重新定义 pack/unpack 的协议兼容性边界。
