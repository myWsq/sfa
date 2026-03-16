## Why

SFA 的技术方案和 v1 change 都把 `unpack` 定位为 bundle 级并发流水线，但当前实现仍以串行 frame 消费和串行 scatter 为主，`--threads` 对解包主路径几乎不产生可验证影响。针对真实 `node_modules` 级海量小文件负载的最新 benchmark 也表明 unpack 吞吐几乎不随线程数扩展，这已经偏离项目“海量小文件场景下优于 `tar + 同算法`”的核心承诺。

## What Changes

- 修复 `unpack` 主链路，使线程覆盖真正作用于 bundle 级解包/恢复执行，而不是仅作为 header 或 stats 元数据返回。
- 消除解包路径中的冗余 frame 解码工作，同时保持现有 frame/trailer 完整性语义不变。
- 调整 unpack 阶段观测模型，把当前过粗的 `decode_and_scatter` 拆分为可解释的稳定阶段，以便 benchmark 和人工排障能区分解码、写盘与 finalize 成本。
- 增加面向真实小文件负载的 unpack 回归验证，覆盖线程覆盖生效性、阶段统计一致性和 benchmark 结果可解释性。

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `archive-unpack`: 收紧解包线程覆盖、bundle 级并发执行与阶段边界的要求，确保顺序流 reader 与并行 restore pipeline 同时成立。
- `cli-and-benchmarks`: 调整 unpack machine-readable stats 与 benchmark 验证要求，使线程覆盖效果和更细粒度 unpack phase breakdown 成为可回归的契约。

## Impact

- 影响 `crates/sfa-core` 的 archive reader、codec/integrity 协作方式和 unpack stats 模型。
- 影响 `crates/sfa-unixfs` 的 unpack pipeline、restore 调度和文件句柄使用模式。
- 影响 `crates/sfa-cli` 的 unpack stats 输出与 `--threads` 语义一致性。
- 影响 `crates/sfa-bench`、benchmark 结果 schema、相关 smoke/回归测试，以及后续 committed baseline 的刷新策略。
