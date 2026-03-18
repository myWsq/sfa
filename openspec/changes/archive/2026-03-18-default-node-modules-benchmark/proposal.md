## Why

SFA 的设计目标是针对海量小文件和深层目录树，但当前仓库 benchmark 仍以小型 committed fixture 和 `lz4`/`zstd` codec matrix 为主，难以证明默认用户路径在真实目标 workload 上的优势。现在需要把 benchmark 收口成围绕默认参数和 `node_modules` 式深层嵌套小文件树的证据链，让性能主张和产品定位一致。

## What Changes

- 用默认用户路径 benchmark 替代当前以 `tar + same codec` codec matrix 为中心的 benchmark 叙事，只保留默认参数下的 pack/unpack 对照。
- 新增一个 `node_modules` 式深层嵌套 workload contract，目标规模为 `100k+` 小文件，并把它作为 benchmark 的主展示场景。
- 调整 benchmark runner、结果 schema 和文档，使输出优先呈现默认参数下的 wall time、files/s、archive size、CPU、RSS 与可解释的 unpack 观测字段。
- 将 `lz4` 从 benchmark 主路径中移除，不再要求默认 benchmark matrix 覆盖多 codec 组合。
- 更新 README、benchmark 文档和 release guidance，让对外 benchmark 结论明确来自默认参数和目标 workload，而不是小型 codec 对照样本。

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `cli-and-benchmarks`: benchmark 要求从“基于多个 committed dataset 的 `tar + same codec` matrix”调整为“围绕默认 pack/unpack 参数和 `node_modules` 式 `100k+` 深层嵌套 workload 的默认路径 benchmark”，并相应更新结果、文档和刷新规则。

## Impact

- `crates/sfa-bench` 的 benchmark workload 定义、runner 命令生成、结果 schema 和辅助脚本
- `benches/results/`、`benches/README.md`、`spec/verification-and-benchmark.md`、`RELEASING.md` 等 benchmark 资产和说明文档
- `README.md` 中 benchmark snapshot 的取材方式和表述
- 可能新增用于生成或描述 `node_modules` 式 `100k+` 小文件 workload 的 benchmark 输入资产或生成流程
- 不改变 `.sfa` v1 wire format，也不改变 pack/unpack 的协议兼容性边界
