## Context

SFA 当前已经有默认 benchmark matrix 和 `tar_vs_sfa` runner，但 `tests/fixtures/datasets/` 里的三个数据集仍是 placeholder，仓库内也没有首轮正式结果。现有 runner 更偏“命令拼装器”，对外部依赖能力、输入数据有效性、输出/解包目录生命周期的处理还不够严格，导致 benchmark 很难稳定复现，更难作为 release gate 的证据。

这个 change 的目标不是重新设计 benchmark 维度，而是把已有比较框架收口成发布前可用的基线资产。协议已经冻结，因此设计必须避免牵动 `.sfa` wire format、pack/unpack 语义或 fixture 解释方式。

## Goals / Non-Goals

**Goals:**

- 为默认 benchmark matrix 提供仓库内可直接使用的真实数据集和配套说明。
- 让 `tar + same codec` 对照基线在受支持环境中能够稳定执行，并在前置条件不满足时给出可操作的错误。
- 生成并提交首轮机器可读 benchmark 结果，作为后续纵向回归的参考基线。
- 把 benchmark 执行条件、结果刷新时机和发版闸口文档化，减少后续维护歧义。

**Non-Goals:**

- 不修改 `.sfa` v1 格式、header/manifest/frame/trailer 语义或协议冻结资产。
- 不引入外部下载型 benchmark 数据源，也不依赖联网拉取数据集。
- 不在本 change 中扩展 Unix 元数据语义、xattrs/ACL 或跨平台兼容承诺。
- 不把 benchmark 基础设施升级为复杂的专用性能实验平台。

## Decisions

### 1. Benchmark 数据集采用“仓库内提交 + 可再生说明”的策略

默认矩阵继续使用 `small-text`、`small-binary` 和 `large-control` 三类数据集，但占位 README 将被真实输入树和数据集说明替代。每个数据集都应在仓库内直接可用，并附带来源/构造方式、规模摘要和再生成提示。

这样做的原因是 release gate 和 longitudinal baseline 需要离线、稳定、无额外依赖地运行。相比引用外部公开数据集，提交可控规模的仓库内数据虽然会增加仓库体积，但可以避免许可证、网络可用性和上游内容漂移导致的不可复现。

备选方案：

- 使用外部下载脚本按需拉取数据集：仓库更轻，但会把 benchmark 变成“依赖环境和网络”的流程，和当前 release gate 目标冲突。
- 继续保留 placeholder 目录，仅在维护者本地临时放数据：实现最省事，但没有可审计基线，不满足当前 change 的目标。

### 2. Runner 需要显式 preflight 和工作目录生命周期管理

benchmark runner 在正式执行前应完成 preflight：校验数据集目录存在且非空、确认 `sfa` 可执行路径可用、验证当前 `tar` 对请求 codec 的支持情况，并为 archive 输出目录和解包目录执行创建/清理逻辑。真实运行时如果前置条件不满足，runner 应在产生半套结果前直接失败，并返回能指导修复的错误。

这样做的原因是当前实现只保证 case output 根目录存在，无法覆盖 tar 解包目录和脏产物清理；同时外部工具能力差异会把失败推迟到中途命令执行阶段，导致结果难以比较。

备选方案：

- 将不满足环境前提的 case 标记为跳过：对探索性 benchmark 有帮助，但对于 release baseline 会留下不完整矩阵，难以作为闸口证据。
- 继续依赖 shell 脚本或人工预清理目录：维护成本高，而且容易出现“本地偶现、CI 失败”的状态漂移。

### 3. 首轮结果应作为版本化资产提交，而不是临时运行产物

默认 benchmark matrix 的首轮正式结果应提交到 `benches/results/` 下，并附带运行命令、执行环境摘要、使用的数据集和刷新策略说明。结果文件应保持机器可读、适合后续脚本消费；文档层面再提供人类可读的解释入口。

这样做可以让 release reviewer 在不重跑 benchmark 的情况下审查基线，并为后续 planner/pipeline 变化提供稳定对照。相比只在 CI 日志或本地终端保留结果，仓库内结果更利于追溯和对比。

备选方案：

- 只要求维护者在 release notes 中粘贴 benchmark 摘要：可读但不利于自动比较，且细节容易丢失。
- 只保留最新一次未提交的本地 JSON：不能形成可审计历史，也不适合作为 release gate 证据。

### 4. 文档将明确“支持环境”与“普适开发环境”的边界

`--dry-run` 继续作为任何开发环境都能执行的 benchmark smoke 入口；真实 baseline 执行则要求满足文档声明的本地工具前提。相关约束会同步到 `README.md`、`RELEASING.md` 和 benchmark 说明文档中，避免贡献者把“支持开发”误解为“支持生成正式 baseline”。

这样做的原因是 benchmark 对外部工具能力的依赖天生强于单元测试和 smoke checks。与其隐含假设所有环境都一致，不如在文档和 runner 中把支持边界写清楚。

## Risks / Trade-offs

- [仓库体积增加] → 控制提交数据集规模，只保留能代表三类工作负载的必要样本，并在说明文档中记录取舍。
- [环境差异导致结果不可比] → 在结果和文档中记录执行环境摘要，并把真实 baseline 限定在支持环境内运行。
- [旧输出污染新结果] → runner 在每个 job 前统一处理 archive/unpack 目标路径，避免复用脏目录。
- [结果很快过时] → 在发版流程和 benchmark 文档中明确“何时必须刷新 baseline”，例如修改 planner、pipeline、codec 行为或 benchmark 逻辑时。
- [scope 漂移到性能优化] → 本 change 只建立基线和可复现机制，不承诺顺带解决所有性能问题。

## Migration Plan

1. 提交真实 benchmark 数据集和数据集说明，使默认矩阵从仓库 checkout 后即可执行。
2. 调整 runner、脚本和报告输出，完成 preflight、目录生命周期和失败诊断收口。
3. 在受支持环境中运行默认 benchmark matrix，提交首轮结果到 `benches/results/`。
4. 同步更新 benchmark / release 文档，说明如何重跑、何时刷新、哪些结果应随版本保留。

如果后续发现结果格式或环境约束仍不足以支撑 release gate，可在不改变协议的前提下通过 follow-up change 继续收口；不需要对现有归档数据做迁移。

## Open Questions

- 首轮 baseline 结果文件是否按版本命名，还是使用固定文件名加文档指向最新结果。
- CPU/RSS 等资源指标是否在本 change 一并落地为结构化字段，还是先保证 wall-time 基线与可复现性。
- 是否需要把更多 golden/corruption/streaming/safety 资产纳入同一 change，还是在 benchmark 基线落稳后再单开 follow-up。
