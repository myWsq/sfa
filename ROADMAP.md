# SFA v1 路线图

本文件用于对外说明 SFA v1 的研发阶段、当前状态与近期优先级。除特别说明外，本文件不构成发布日期承诺。

最近更新：2026-03-16

## 当前状态

SFA v1 当前处于 `开发中` 状态，仓库已经具备可运行的 MVP，并已完成协议冻结。

当前已经可用的能力包括：

- 基于 Rust workspace 的实现，包含 `sfa-core`、`sfa-unixfs`、`sfa-cli`、`sfa-bench`
- 可运行的 `sfa pack` / `sfa unpack` 端到端链路
- manifest-first 的 `.sfa` 结构，已实现 header、manifest、frame、optional trailer
- `lz4`、`zstd` 编解码支持
- 确定性目录扫描、稳定 bundle 规划与顺序读写
- regular file、directory、symlink、hardlink 的打包与恢复
- roundtrip、streaming、corruption、safety 测试框架和 benchmark harness

当前尚未完成的关键收口项包括：

- 用真实数据替换 benchmark placeholder datasets
- 形成可追溯的 benchmark 基线结果记录
- 将 golden fixtures 扩展到更大的协议覆盖面

## v1 目标

SFA v1 的目标是提供一个面向本地归档场景的、可顺序读取的 `.sfa` 格式与对应工具链，重点覆盖以下能力：

- 稳定、可验证的归档格式定义
- 面向 Unix 文件树的可靠打包与恢复
- 支持主流压缩算法的可比较基线
- 可复现的验证、回归与性能测试流程

当前版本不承诺一次性覆盖全部 Unix 扩展语义，相关增强按里程碑继续推进。

## 里程碑概览

| 里程碑 | 名称 | 状态 | 目标说明 |
|---|---|---|---|
| M0 | 协议冻结 | 已完成 | 冻结 v1 协议文本，提交首批 golden fixtures，并完成评审留痕 |
| M1 | 最小可用链路 | 进行中 | 将当前 MVP 收口为可稳定回归、可进入 CI 的最小可用版本 |
| M2 | 性能主链路 | 进行中 | 补齐真实 benchmark 数据与结果，建立 `tar + same codec` 对照基线 |
| M3 | Unix 语义增强 | 未开始 | 在 v1 主链路稳定后，补充更完整的 Unix 元数据与边界能力 |

状态定义：

- `未开始`：尚未进入实际实现或资产落地阶段
- `进行中`：已有实现，但尚未满足关闭条件
- `已完成`：已满足关闭条件，可正式关闭

## 里程碑详情

### M0：协议冻结

状态：`已完成`

已完成：

- `spec/format-v1.md` 已成为权威协议定义
- 第一批 canonical golden fixture 已提交到 `tests/fixtures/golden/`
- `spec/format-v1-freeze-review.md` 已记录冻结输入、结论与延后事项
- protocol smoke 已消费 golden fixture 元数据

关闭结果：

- v1 协议兼容性边界已固定
- golden archive、manifest dump 与统计摘要已提交
- 协议评审结果可在仓库中追溯

### M1：最小可用链路

状态：`进行中`

已完成：

- `pack` / `unpack` MVP 已可端到端运行
- regular file、directory、symlink、hardlink 已支持
- 顺序读取解包已实现，不依赖 seek
- CLI 已接入真实实现

待完成：

- 补齐 CLI 行为测试、默认值和异常路径覆盖
- 将 golden fixtures 纳入更完整的 CI 回归

关闭条件：

- 典型目录树 roundtrip 稳定
- CLI 可支撑常规本地使用场景
- golden fixtures 成为 CI 基线的一部分

### M2：性能主链路

状态：`进行中`

已完成：

- 稳定线性 bundle planner 已实现
- ordered writer 与多线程 pack pipeline 已实现
- benchmark harness 已具备

待完成：

- 将 placeholder benchmark datasets 替换为真实数据集
- 生成并保存第一轮 `tar + same codec` 对比结果
- 增加阶段级耗时与性能观测信息

关闭条件：

- benchmark 数据集真实、稳定、可复用
- tar baseline 可重复运行
- 性能结果在仓库内有明确记录

### M3：Unix 语义增强

状态：`未开始`

计划范围：

- 更强的元数据恢复验证
- 如仍在范围内，评估 xattrs / ACL 的后续能力
- 更完整的 Unix 边界样例与异常路径覆盖

关闭条件：

- 相关增强能力有明确 spec、实现与测试资产

## 近期优先级

当前最优先的下一轮工作建议为：

`stabilize-benchmark-baseline`

建议范围：

- 补齐真实 benchmark 数据集
- 修复 `tar + same codec` baseline 的环境依赖与输出目录问题
- 记录第一轮 benchmark 结果
- 在不改变冻结协议的前提下扩展更多 golden / corruption / streaming / safety 资产

## 文档边界

本文件用于维护仓库级路线图与项目状态，不替代以下文档：

- `openspec/changes/...`：单次 change 的提案、设计与任务拆解
- `sfa-tech-solution/`：较完整的技术方案背景
- `spec/`：冻结后的协议与验证规范
