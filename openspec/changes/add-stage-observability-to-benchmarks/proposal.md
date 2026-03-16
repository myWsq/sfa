## Why

SFA v1 已经具备 committed benchmark datasets、默认矩阵和首轮 machine-readable baseline，但当前结果只记录每条命令的总耗时，难以解释 planner、encode、write、decode、restore 等阶段的性能回归。`ROADMAP.md` 和 benchmark 文档已经把“更细粒度阶段级/资源级观测”列为剩余收口项，因此现在需要把 benchmark 结果从“能跑”推进到“能诊断、能比较、能作为发版证据”。

## What Changes

- 扩展 pack / unpack 统计结构，记录关键阶段的 wall-time breakdown，而不仅是单个总耗时。
- 扩展 benchmark report schema，保留阶段级指标，并在支持环境中记录基础资源观测信息，至少包括可稳定获取的峰值 RSS 和 CPU 时间摘要。
- 收口 benchmark runner、CLI 摘要和文档，使新统计字段在 dry-run、真实执行和 committed baseline 中都有明确语义与兼容边界。
- 更新 benchmark 基线与发版文档，明确何时必须刷新结果，以及如何解释缺少资源观测字段的受限环境。

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `cli-and-benchmarks`: benchmark requirements 从“提供 committed 数据集、可执行矩阵和总耗时结果”扩展为“提供阶段级耗时与受支持环境下的基础资源观测，并将其记录到 committed baseline 与相关文档中”。

## Impact

- 影响 `crates/sfa-core` 中的 pack / unpack stats 定义和序列化结果。
- 影响 `crates/sfa-unixfs` 中 pack / unpack 主链路的阶段计时插桩方式。
- 影响 `crates/sfa-cli` 的人类可读 summary 输出与 benchmark 兼容接口。
- 影响 `crates/sfa-bench` 的 report schema、runner、baseline 结果文件与校验逻辑。
- 影响 `benches/results/`、`spec/verification-and-benchmark.md`、`README.md`、`ROADMAP.md`、`RELEASING.md` 等对 benchmark 基线的说明。
