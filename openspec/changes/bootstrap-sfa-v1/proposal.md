## Why

SFA 的技术方案已经明确，但仓库中还没有把方案落成可实现、可评审、可拆任务的 OpenSpec 变更。现在需要把 SFA v1 的首个可用范围收敛成正式 change，确保后续实现围绕统一的协议、CLI 语义、恢复语义和基准目标推进，而不是在开发过程中反复改边界。

## What Changes

- 新增 SFA v1 的首个实现范围定义，覆盖 `.sfa` 归档格式、打包、解包、CLI 与基准基线。
- 明确 v1 只支持 Unix 场景，聚焦海量小文件的高吞吐目录归档与恢复。
- 要求归档采用 `header + manifest + frame*` 的严格顺序流式结构，不依赖 seek。
- 要求 `pack`/`unpack` 支持 LZ4 和 Zstd、bundle 级并行、fast/strong 两档完整性策略。
- 要求解包能够从本地文件流与 HTTP 顺序流恢复 regular file、directory、symlink 和 hardlink。
- 新增与 `tar + 同算法` 的 benchmark 基线与最小测试矩阵，作为 v1 工程验收的一部分。

## Capabilities

### New Capabilities
- `archive-format-v1`: 定义 `.sfa` v1 的头部、manifest、frame、codec 元数据、完整性模式与严格流式读取约束。
- `archive-pack`: 定义目录扫描、bundle 规划、归档写出与压缩配置覆盖的打包行为。
- `archive-unpack`: 定义顺序流解包、Unix 安全恢复、元数据应用与损坏输入处理行为。
- `cli-and-benchmarks`: 定义 `sfa pack` / `sfa unpack` CLI 语义、退出行为、统计输出和 tar 同算法对标要求。

### Modified Capabilities

None.

## Impact

- 新增 OpenSpec 规格文件，作为 SFA v1 的实现与评审契约。
- 后续实现将影响 Rust workspace 的 crate 布局，至少包括 `sfa-core`、`sfa-unixfs`、`sfa-cli`、`sfa-bench`。
- 会引入 `.sfa` 格式说明、CLI 命令面、测试样例、golden fixture 和 benchmark 数据集/脚本。
- 会明确外部接口约束，包括归档扩展名、codec 选择、线程数覆盖、完整性策略与恢复策略。
