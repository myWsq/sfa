## Context

SFA 当前的 benchmark 主链路已经具备 committed datasets、默认矩阵、`tar + same codec` 对照和首轮 machine-readable baseline，但观测粒度仍停留在“每条命令一条总耗时”。当前实现里：

- `crates/sfa-core` 的 `PackStats` / `UnpackStats` 只有总耗时与吞吐基础字段。
- `crates/sfa-unixfs` 的 pack / unpack 主链路没有暴露稳定的阶段计时边界。
- `crates/sfa-cli` 只输出人类可读 summary，benchmark runner 无法稳定提取内部阶段统计。
- `crates/sfa-bench` 只能记录命令 wall-time、stdout/stderr 和工具元数据，对资源占用也没有结构化记录。

这导致当前 baseline 能回答“这次比上次慢了多少”，但很难回答“慢在哪个阶段”或“是否伴随更高 CPU / RSS”。`ROADMAP.md`、`README.md` 和技术方案文档都把更细粒度观测列为剩余收口项，因此这个 change 需要在不修改 `.sfa` 协议和 benchmark 数据集矩阵的前提下，把 benchmark 结果推进到可诊断状态。

## Goals / Non-Goals

**Goals:**

- 为 SFA pack / unpack 提供稳定、可序列化的阶段级 wall-time breakdown。
- 为 benchmark report 增加基础资源观测字段，并明确哪些环境必须产出这些字段。
- 保持 benchmark runner 继续通过命令执行 `sfa` 与 `tar`，避免为 benchmark 单独实现一套执行路径。
- 让 committed baseline、CLI 摘要和文档对新增统计字段有一致语义。

**Non-Goals:**

- 不把 benchmark 基础设施升级为 profiler、trace system 或长期性能数据库。
- 不引入 per-file、per-bundle、队列峰值等高频细粒度遥测。
- 不改变 `.sfa` wire format、默认 benchmark 数据集矩阵或 `tar` 对照方法。
- 不在本 change 内扩展 golden / corruption / safety fixture 覆盖面。

## Decisions

### 1. 统计模型扩展为“总览 + 分阶段 + 可选资源观测”

`PackStats` / `UnpackStats` 将继续保留现有总览字段，同时新增嵌套的阶段 breakdown 结构；`BenchmarkRecord` 新增面向命令执行的资源观测结构。阶段 breakdown 只覆盖稳定主链路边界：

- pack: `scan`、`plan`、`encode`、`write`
- unpack: `header`、`manifest`、`decode_and_scatter`、`restore_finalize`

资源观测作为独立对象挂在 benchmark record 上，至少包含：

- `user_cpu_ms`
- `system_cpu_ms`
- `max_rss_kib`
- `sampler` / `notes`

这样做的原因是内部阶段统计和外部进程资源观测有不同来源与精度，混在同一扁平字段集合里会让语义变得模糊。嵌套结构也更利于后续增量扩展，而不必一次性把顶层 schema 撑得很散。

备选方案：

- 扁平化为大量 `scan_ms` / `plan_ms` / `rss_kib` 字段：实现直接，但 report、CLI 和 JSON 很快会变难维护。
- 只在 benchmark report 中增加 breakdown，不修改 core stats：会造成 CLI、库接口和 benchmark 对同一运行结果有两套不一致模型。

### 2. 阶段计时在 `sfa-unixfs` 主链路中采集，保持边界粗粒度且稳定

pack / unpack 的阶段时间将在 `sfa-unixfs` 的真实执行路径中采集，然后通过 `sfa-core` stats 结构向上暴露。计时边界以“用户可以理解、实现不易抖动”的稳定阶段为准，而不是对每个 bundle、文件或线程队列做细粒度埋点。

这样做的原因是 benchmark 基线首先需要可比较性。如果把阶段切得太细，轻微实现调整就会让字段语义漂移，反而削弱纵向对比价值。粗粒度阶段更接近 roadmap 和技术方案中定义的观测目标，也更容易写成稳定测试。

备选方案：

- 在 `sfa-bench` 中对 CLI stdout 做模糊解析推断阶段耗时：过于脆弱，输出文案一变就坏。
- 为 planner、codec、writer 等内部模块各自打点并透传更多明细：信息更多，但这个 change 的目标不是构建 profiling 系统。

### 3. Benchmark runner 继续走命令执行路径，SFA 通过 CLI 的 JSON stats 模式暴露结构化结果

benchmark runner 仍然执行真实 `sfa` / `tar` 命令，不直接链接 `sfa-unixfs` 库。对于 SFA 命令，CLI 将新增 machine-readable stats 输出模式，benchmark runner 通过该模式读取 pack / unpack 统计；默认 CLI 输出仍保持人类可读 summary，不影响常规手动使用。

这样做的原因是 benchmark 目标是比较实际用户入口的行为，而不是旁路库调用。CLI 提供 JSON stats 也能让 benchmark 与未来自动化脚本共享同一接口，同时避免依赖临时文件路径或 stdout 文本解析。

备选方案：

- 让 benchmark runner 直接调用 Rust 库：能更容易拿到结构化 stats，但会偏离真实 CLI 行为，并增加 runner 与业务库的耦合。
- 让 CLI 写临时 `stats.json` 文件：可以工作，但 runner 需要管理更多临时路径，命令重现也更笨重。

### 4. 资源观测由 benchmark runner 的外层包装器采集，并显式声明支持环境

资源观测不放进 `sfa` pack / unpack 内部 stats，而由 `sfa-bench` 在执行命令时统一采集。runner 将探测当前主机可用的资源采样方式，并支持解析至少一种受支持环境中的稳定输出；当环境不支持时，report 必须把资源字段留空并给出原因说明，而不是伪造 0 值。

这样做的原因是 `tar` 基线没有内部 stats 接口，资源观测必须在命令层统一处理，才能保证 SFA 与 TAR 的可比性。把“支持环境”和“缺失语义”写清楚，也比默认假设所有主机都能给出同质量资源数据更稳妥。

备选方案：

- 只给 SFA 记录资源统计：会破坏和 TAR 的对比价值。
- 把不支持资源采样的环境直接视为执行失败：对 release baseline 可接受，但会降低日常探索和本地验证的可用性。

### 5. Committed baseline 保持单一 JSON 资产，但新增字段必须可向后缺省

现有 `benches/results/baseline-v0.1.0.json` 仍然作为 repository baseline 载体；report schema 通过新增可选字段扩展，而不是拆成多文件或引入额外 sidecar。相关测试将保证 committed baseline 仍可被当前代码读取，并校验默认矩阵与必需观测字段的存在关系。

这样做的原因是仓库已经围绕单一 JSON 资产建立了 README、release 和测试约定，继续复用这一路径的变更成本最低。新增字段做成可选可以兼容旧结果，也能为 dry-run 或受限环境保留清晰缺省语义。

备选方案：

- 为资源观测另建 `resources.json`：消费者要拼接多份资产，审阅成本更高。
- 用新文件名替换旧 baseline：可行，但会平白增加文档和回归成本。

## Risks / Trade-offs

- [阶段边界定义不稳] → 只选择 scan/plan/encode/write 与 header/manifest/decode/restore 这类粗粒度边界，并在设计与测试中固定语义。
- [CLI 新增 JSON 输出后接口变复杂] → 保持默认 human summary 不变，把 JSON 作为显式 opt-in 模式。
- [资源采样跨平台差异大] → 在 runner 中做显式探测和解析，并在文档中写清楚支持环境与缺失语义。
- [新增字段导致 baseline 更新频繁] → 只记录稳定且有解释价值的字段，避免把实验性统计写入 committed 资产。
- [dry-run 语义变含糊] → 明确 dry-run 只允许输出估算总览，不伪造真实阶段或资源观测。

## Migration Plan

1. 扩展 `sfa-core` stats 结构，定义阶段 breakdown 的序列化模型。
2. 在 `sfa-unixfs` pack / unpack 主链路加入稳定阶段计时，并通过 CLI 暴露 JSON stats 模式。
3. 扩展 `sfa-bench` report schema 与 runner，让 SFA 记录内部阶段统计，并在支持环境下为 SFA/TAR 记录资源观测。
4. 刷新 committed baseline，更新 benchmark / release / roadmap 文档，并补充相应测试与 smoke-level 校验。

如果实现过程中发现资源采样在某些主机上不稳定，可以保留字段但通过 `notes` 标记 unsupported，并在文档中将该主机移出正式 baseline 支持范围；不需要迁移任何 `.sfa` 归档资产。

## Open Questions

- CLI 的 machine-readable stats 模式是否只输出 stats 对象，还是统一为包含 `command` / `status` / `stats` 的更通用 JSON 结构。
- 资源采样的首个受支持实现应优先覆盖 GNU `time`、BSD `time`，还是仅记录当前 release 环境最稳定的一种。
- 是否要把 benchmark report 中“wall-time”与“SFA 内部 total duration”并列保留，还是以命令 wall-time 为准、内部 total 仅作诊断字段。
