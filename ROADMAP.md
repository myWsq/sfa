# SFA v1 路线图

本文件用于对外说明 SFA v1 的研发阶段、当前状态与近期优先级。除特别说明外，本文件不构成发布日期承诺。

最近更新：2026-03-17

## 当前状态

SFA v1 当前处于 `开发中` 状态，仓库已经具备可运行的最小可用链路，并已完成协议冻结、M1 收口与 M2 性能主链路建设。当前工作已进入 M3 的第一阶段：收口现有 Unix metadata contract 与验证资产。

当前已经可用的能力包括：

- 基于 Rust workspace 的实现，包含 `sfa-core`、`sfa-unixfs`、`sfa-cli`、`sfa-bench`
- 可运行的 `sfa pack` / `sfa unpack` 端到端链路
- manifest-first 的 `.sfa` 结构，已实现 header、manifest、frame、optional trailer
- `lz4`、`zstd` 编解码支持
- 确定性目录扫描、稳定 bundle 规划与顺序读写
- regular file、directory、symlink、hardlink 的打包与恢复
- regular file 与 directory 默认恢复 `mode` / `mtime`，archive manifest 持续记录 `uid` / `gid`
- owner restore 为显式 opt-in 路径，且仍受 effective root 约束
- roundtrip、streaming、corruption、safety 测试框架和 benchmark harness
- benchmark report 中的阶段级 wall-time breakdown 与支持环境下的资源观测信息
- `sfa pack` / `sfa unpack` 的 machine-readable stats 输出，可供 benchmark runner 和自动化脚本消费
- unpack worker 线程覆盖已接入真实 bundle 级 restore pipeline，阶段观测可区分 `frame_read` / `decode` / `scatter`
- `sfa unpack -` 已支持从 `stdin` 解包，库侧也已提供 `sync Read` 解包入口
- restore 主路径已切换到 `dirfd/openat` 风格安全 IO；`strong` trailer 失败会留下 `.sfa-untrusted`
- expanded canonical golden corpus 与 CLI 行为回归已经成为仓库默认验证基线

当前下一轮工作的重点包括：

- 推进 M3 第一阶段：收口当前 Unix metadata contract 与验证资产
- 维持当前手动发版节奏下的版本、验证清单与 release notes 同步

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
| M1 | 最小可用链路 | 已完成 | 将当前 MVP 收口为可稳定回归、可进入 CI 的最小可用版本 |
| M2 | 性能主链路 | 已完成 | 补齐真实 benchmark 数据与结果，建立 `tar + same codec` 对照基线，并记录阶段级/资源级观测 |
| M3 | Unix 语义增强 | 进行中 | 在 v1 主链路稳定后，先收口当前 metadata contract 与验证资产，再决定后续扩展元数据范围 |

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

状态：`已完成`

已完成：

- `pack` / `unpack` MVP 已可端到端运行
- regular file、directory、symlink、hardlink 已支持
- 顺序读取解包已实现，不依赖 seek
- CLI 已接入真实实现
- `stdin` / `sync Read` 解包入口已接通
- `strong` trailer 失败时会产生 `.sfa-untrusted`
- CLI 行为测试已覆盖默认值、usage error、`stdin` / `--dry-run` 组合与 overwrite 语义
- canonical golden corpus 已扩展并纳入协议 smoke 与 CI 验证路径
- release checklist 已包含 `cargo fmt --all --check`、workspace tests、smoke checks 与 benchmark dry-run

关闭条件：

- 典型目录树 roundtrip 稳定
- CLI 可支撑常规本地使用场景
- golden fixtures 成为 CI 基线的一部分

关闭结果：

- M1 已从“功能存在”收口到“仓库级验证可执行”
- 当前最主要的后续工作已转向 M3 Unix 语义增强，而不是继续补最小可用链路

### M2：性能主链路

状态：`已完成`

已完成：

- 稳定线性 bundle planner 已实现
- ordered writer 与多线程 pack pipeline 已实现
- benchmark harness 已具备
- committed benchmark datasets 已替换 placeholder corpus
- 第一轮 `tar + same codec` machine-readable baseline 已记录到 `benches/results/baseline-v0.1.0.json`
- benchmark report 已记录 SFA 阶段级 wall-time breakdown
- benchmark runner 已在支持环境中记录 CPU / RSS 资源观测信息

关闭条件：

- benchmark 数据集真实、稳定、可复用
- tar baseline 可重复运行
- 性能结果在仓库内有明确记录
- 性能结果具备足够的观测信息以支持回归解释

### M3：Unix 语义增强

状态：`进行中`

当前范围：

- 收口当前 v1 Unix metadata contract，明确 `mode` / `mtime` / owner policy 的承诺边界
- 补齐 metadata roundtrip、owner-policy 与现有 link / safety 场景的仓库级验证
- 同步路线图、README 与技术方案文档，使 xattrs / ACL 继续保持 deferred

后续候选：

- 如仍在范围内，评估 xattrs / ACL 的后续能力
- 更完整的 Unix 边界样例与异常路径覆盖

关闭条件：

- 当前 metadata contract 有明确 spec、实现与测试资产
- 仓库级状态文档与技术方案文档不再把已交付能力误标为未来里程碑
- 若进入扩展元数据能力，使用新的 OpenSpec change 明确拆分范围

## 近期优先级

当前最优先的下一轮工作建议为：

`M3：Unix metadata contract 收口`

建议范围：

- 增强 owner / metadata restore 的验证深度，明确哪些 Unix 语义已是 v1 contract
- 保持 xattrs / ACL deferred，不在当前 change 中混入新的扩展元数据交付
- 让 metadata roundtrip、owner-policy 与现有 link / safety 场景一起进入可审计的仓库级验证
- 保持当前手动 release checklist 与 benchmark dry-run 作为稳定的发版前闸口

## 文档边界

本文件用于维护仓库级路线图与项目状态，不替代以下文档：

- `openspec/changes/...`：单次 change 的提案、设计与任务拆解
- `sfa-tech-solution/`：较完整的技术方案背景
- `spec/`：冻结后的协议与验证规范
