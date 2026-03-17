## Context

当前 `sfa unpack` 的 machine-readable stats 暴露了 `header`、`manifest`、`frame_read`、`decode`、`scatter`、`restore_finalize` 这组阶段字段。它们来自并行 unpack pipeline 的观测窗口，适合回答“哪里在忙”，但不适合回答“总耗时落在哪”。随着 unpack 采用顺序 reader 加 decode/scatter worker 的有界流水线，这些阶段存在天然重叠；即便改成高精度累计后，它们也不应该被解释为可与 `duration_ms` 严格对账的一组 bucket。

这导致三个现实问题：

- CLI JSON 和 benchmark baseline 的消费者会自然尝试把 split phase 与总耗时相加比较，得出错误结论。
- 当前 `diagnostics` JSON 已经能说明 queue wait、writer 锁竞争、文件打开等热点，但这套数据没有和正式 stats schema 建立清晰分工。
- 仅靠文档提醒“不要求相加”不够，因为用户仍然需要一组真正 additive 的时间分布来判断 wall-time 去向。

这个 change 需要在不改变 unpack pipeline 语义、restore 顺序、线程接线行为和归档协议的前提下，把“总账解释”和“并行诊断”拆成两套稳定指标。

## Goals / Non-Goals

**Goals:**

- 为 unpack 提供一组 additive wall-time buckets，使序列化后的 bucket 总和能够与 `duration_ms` 对账。
- 保留现有 `header`、`manifest`、`frame_read`、`decode`、`scatter`、`restore_finalize` 这组诊断窗口，用于 benchmark、thread sweep 和瓶颈分析。
- 明确 CLI stats、benchmark schema、diagnostics JSON 和文档各自承诺的统计语义，避免同一字段被双重解读。
- 统一 unpack 统计的内部采样方式为高精度 `Duration` 累计，所有面向外部的截断只发生在最终输出阶段。

**Non-Goals:**

- 不改变 `.sfa` wire format、manifest 结构或 unpack restore 语义。
- 不重新设计 decode/scatter worker 数量策略、queue 深度策略或 diagnostics collector 的具体细项。
- 不为 pack 同步引入新的 additive wall bucket schema，除非实现中发现必须共用基础设施。
- 不把所有内部等待时间都提升为顶层稳定公共字段；更细粒度的锁等待、文件打开和 cache 指标仍留在 diagnostics 层。

## Decisions

### 1. 保留现有 `phase_breakdown` 作为 diagnostic windows，并新增独立的 `wall_breakdown`

`phase_breakdown` 已经被 CLI JSON、benchmark baseline 和回归测试消费；直接把它改造成 additive buckets，会丢失 decode/scatter 的并行可见性，也会让现有 thread sweep 结果失去连续性。这个 change 将保持现有 `phase_breakdown` 字段及其阶段名称，但明确其语义是 overlapping diagnostic windows，而非互斥 wall-time buckets。

同时，为 unpack stats 新增一个独立的 `wall_breakdown`，专门表达可加总的 wall-time 分段。建议的顶层 bucket 为：

- `setup_ms`: 从 unpack 开始到进入 `run_unpack_pipeline()` 之前的连续 wall-time，包括输出目录准备、header/manifest 读取、目录创建、regular file 预备和 writer 初始化。
- `pipeline_ms`: `run_unpack_pipeline()` 的端到端 wall-time，覆盖顺序 reader、decode worker、scatter worker 及其等待。
- `finalize_ms`: pipeline 返回后的连续 wall-time，包括多 extent regular file finalize、link 创建、directory finalize、trailer 校验以及成功返回前的收尾。

选择这三个 bucket，而不是把现有更细粒度阶段也改造成 additive 的原因是：

- 它们对应真实连续且互斥的执行区间，容易保证无重叠、无遗漏。
- 它们能直接回答“总耗时大头在 setup、pipeline 还是 finalize”。
- 更细的 additive 颗粒度会要求把 queue wait、reader idle、worker join、prepare path 等细节再切成多个公共字段，复杂度高且语义更难稳定。

备选方案：

- 直接重定义现有 `phase_breakdown` 为 additive buckets：会破坏现有 benchmark 解释方式，并丢失 decode/scatter busy window 的价值。
- 只加文档说明 phase 不可相加：无法满足“总耗时去哪了”的真实需求。

### 2. Additive wall buckets 在序列化精度上必须与 `duration_ms` 对账

如果仅仅把多个 `Duration` 分别截断为毫秒后输出，`setup_ms + pipeline_ms + finalize_ms` 仍可能因为逐项截断而小于 `duration_ms` 若干毫秒。为避免新的 additive buckets 继续在最终显示层失真，序列化规则需要明确：

- 内部以 `Duration` 记录 contiguous wall segments；
- `duration_ms` 继续来自总 wall-time；
- 输出 `wall_breakdown` 时，前两个 bucket 按统一截断规则转换；
- 最后一个 bucket 通过 `duration_ms - prior_buckets_sum` 求得，保证序列化后的总和与 `duration_ms` 严格一致。

这样做的原因是用户和 benchmark 消费者最终看到的是序列化值，而不是内部 `Duration`。如果“可对账”只在内部成立、在 JSON 上不成立，这个 change 仍然没有真正解决问题。

备选方案：

- 把全部时间字段升级为微秒或纳秒：可以减小误差，但不能从根本上保证在展示精度下严格相加，还会放大 schema 变更范围。
- 接受 1-2ms 的显示误差：会让新的 additive buckets 再次落入“原则上能对账，实际上总差一点”的灰区。

### 3. `diagnostics` 继续承载 wait/lock/open 等细粒度指标，不并入顶层 stable schema

当前 diagnostics collector 已经能记录 `decode_dispatch_wait_ns`、`scatter_dispatch_wait_ns`、`writer_lock_wait_ns`、`file_open_ns`、`write_ns` 等具体热点，这些数据更适合定位瓶颈，而不是成为默认 machine-readable stats 的公共契约。这个 change 保持 diagnostics 层为“深挖工具”，但要求文档和 benchmark/CLI stats 对它的关系说明更清楚：

- `wall_breakdown` 回答“总耗时去哪了”；
- `phase_breakdown` 回答“pipeline 哪些阶段在忙”；
- `diagnostics` 回答“为什么某个 busy window 或 wall bucket 变重”。

这样可以避免把过多内部实现细节冻结进顶层 stats schema，同时保持现有诊断能力。

备选方案：

- 直接把 queue wait / writer lock wait 提升到顶层 `UnpackStats`：字段过多，且与具体实现耦合过深。

### 4. Benchmark 与 baseline 同时保存 additive 与 diagnostic 两类 unpack 观测

`sfa-bench` 当前已经保存 unpack 的 `phase_breakdown`。在引入 `wall_breakdown` 后，benchmark record 需要同时保留两组数据：

- additive `wall_breakdown`，用于与 command wall-time 对账、解释 setup/pipeline/finalize 的占比；
- diagnostic `phase_breakdown`，用于 thread sweep 和 pipeline 行为分析。

仓库内的 committed baseline 因 schema 变化需要刷新，并在文档中明确旧记录与新记录的解读差异。这里不尝试保留“旧消费者完全无感”的兼容错觉；相反，应把 schema 演进和 baseline refresh 一次性收口。

## Risks / Trade-offs

- [两套时间字段会增加认知负担] → 在 spec、README、benchmark 文档和 JSON 示例中明确区分 “wall” 与 “diagnostic” 的用途，并避免复用模糊命名。
- [序列化级别的严格对账需要 residual 分配规则] → 将“最后一个 bucket 吸收残差”写成稳定契约，并用回归测试锁住。
- [benchmark schema 变化会触发 baseline refresh] → 在 tasks 和 release-facing 文档中把刷新 committed baseline 作为显式步骤。
- [实现时容易遗漏 setup/finalize 中的某个连续区间] → 用端到端测试校验 `wall_breakdown` 求和等于 `duration_ms`，并覆盖 stdin/file 两种 unpack 入口。

## Migration Plan

1. 先修改 `cli-and-benchmarks` spec，明确 unpack stats 同时暴露 additive `wall_breakdown` 与 diagnostic `phase_breakdown`，并说明两者的对外语义。
2. 在 `crates/sfa-core` 中扩展 unpack stats schema，在 `crates/sfa-unixfs` 中实现新的 wall bucket 采样与高精度累计。
3. 更新 `crates/sfa-cli` 和 `crates/sfa-bench` 的 JSON 解析/输出、测试和 committed baseline。
4. 更新 benchmark/verification 文档，说明如何用 `wall_breakdown` 对账、用 `phase_breakdown` 诊断，以及何时必须刷新 baseline。

## Open Questions

- additive bucket 是否只给 unpack 增加，还是顺手为 pack 也补一组对账型 wall buckets；当前倾向只做 unpack，避免扩大变更面。
- `wall_breakdown` 的字段名是否要显式带 `wall_` 前缀，还是通过嵌套对象名表达语义；需要在实现时结合现有 JSON 风格收口。
