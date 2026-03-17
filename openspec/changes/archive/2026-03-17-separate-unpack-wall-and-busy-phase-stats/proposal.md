## Why

当前 `sfa unpack` 的 phase breakdown 主要服务于并行 pipeline 诊断，但这些字段又以阶段耗时的形式暴露给 CLI 和 benchmark 消费者，容易被误读为应当与总 wall-time 对账。随着 unpack pipeline 拆成 reader/decode/scatter/finalize 后，这组指标天然会重叠，再继续让同一组字段同时承担“瓶颈诊断”和“总账解释”两种职责，会让 benchmark 分析、thread sweep 结论和后续回归判断持续混乱。

## What Changes

- 明确把 unpack 统计拆成两类语义不同的观测：
  - 可加总的 wall-time buckets，用于解释总耗时落在哪些互斥阶段。
  - 可重叠的 busy/diagnostic windows，用于分析 decode、scatter、queue backpressure 等并行执行热点。
- 调整 machine-readable stats 和 benchmark schema，让消费者能够区分“可对账”与“仅诊断”字段，而不是继续把现有 split phase 当作总账。
- 保持现有 unpack pipeline、restore 语义、线程覆盖行为和 diagnostics JSON 的问题定位能力，但补齐文档、测试和基线约束，避免旧读法继续扩散。
- 将当前实现里已经修复的“先按子任务截断为毫秒再累加”高精度累计方式纳入交付边界，确保新的统计语义不会被低精度采样破坏。

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `cli-and-benchmarks`: unpack 的 machine-readable stats、benchmark 结果以及相关文档需要区分 additive wall-time buckets 与 overlapping diagnostic windows，并定义两类字段各自的稳定语义。

## Impact

- `crates/sfa-core` 的 stats 结构与序列化 schema
- `crates/sfa-unixfs` 的 unpack phase 采样与 diagnostics 对接
- `crates/sfa-cli` 的 JSON stats 输出与可能的 diagnostics 报告组织
- `crates/sfa-bench` 的 report schema、解析、基线验证与文档
- `spec/verification-and-benchmark.md`、README/benchmark 文档，以及需要时的 committed baseline 刷新
