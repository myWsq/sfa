## Context

SFA 当前的 benchmark 基线是围绕 `small-text`、`small-binary`、`large-control` 三类 committed dataset 和 `lz4`/`zstd` codec matrix 建立的。这个结构适合做 release gate 和纵向回归，但不适合证明 SFA 面向“海量小文件 + 深层目录树”的核心定位：默认 small-file fixture 太小，README 的 benchmark snapshot 也因此更像小样本对照，而不是默认用户路径在目标 workload 上的性能证据。

另一个偏差是默认参数已经收口为 `zstd -3`、默认线程数、默认 bundle 参数和默认完整性策略，但 benchmark 规格仍强调 `tar + same codec`。这使 benchmark 的主叙事停留在“底层 codec 对照”，而不是“用户实际会运行的默认命令对照”。

这个 change 的目标不是改动 `.sfa` 协议、pack/unpack 语义或优化算法本身，而是重新定义 benchmark 的工作负载、命令口径、结果模型和对外解释方式，让 benchmark 与产品定位一致。

## Goals / Non-Goals

**Goals:**

- 把 benchmark 主路径改为默认用户路径对照：`sfa pack` / `sfa unpack` 默认参数 vs 固定的 `tar | zstd -3` 基线。
- 定义一个可重复生成的 `node_modules` 式深层嵌套 workload contract，主规模为 `100k+` 小文件。
- 保留 benchmark 结果的机器可读、可诊断和可审计属性，同时让结果更容易支撑 README 和 release 叙事。
- 将现有 benchmark 文档、README snapshot 和 release guidance 对齐到默认路径 benchmark，而不是 codec matrix。

**Non-Goals:**

- 不修改 `.sfa` v1 wire format、bundle 语义、restore 语义或现有协议兼容边界。
- 不在本 change 内顺带实现新的性能优化；benchmark 仅负责衡量和展示默认路径。
- 不把 benchmark 升级为通用性能实验平台或覆盖所有 codec / 参数组合。
- 不把 `100k+` 小文件树作为静态 fixture 直接提交进仓库。

## Decisions

### 1. Benchmark 主路径改为单一 default-path，对外不再维护 codec matrix

benchmark runner 将不再把 `lz4` / `zstd` 作为默认矩阵维度，也不再以 “`tar + same codec`” 作为 headline 要求。默认 benchmark 的 SFA 命令应直接使用用户默认入口：

- `sfa pack <input> <archive>`
- `sfa unpack <archive> -C <output>`

对应的 tar baseline 采用固定的 canonical pipeline：

- `tar -cf - <input> | zstd -3 > <archive>`
- `zstd -d -c <archive> | tar -xf - -C <output>`

这样做的原因是当前产品默认值已经是 `zstd -3` 和默认线程数，benchmark 应证明“默认体验是否有说服力”，而不是继续围绕 codec 维度做实验矩阵。相比保留 `lz4` 作为并列主路径，单一 default-path 更能避免 README、文档和结果解释被次要维度稀释。

备选方案：

- 保留 `lz4` / `zstd` matrix，但 README 只展示 `zstd`：实现改动较小，但结果模型和文档口径仍然会围绕 matrix 组织，定位不够聚焦。
- 完全改成 “same codec, same level” 泛化框架：可比性更强，但仍然偏 benchmark 实验平台视角，而不是默认用户路径视角。

### 2. `node_modules-100k` workload 通过 committed recipe 生成，而不是提交完整树

新的 benchmark workload 将使用仓库内 committed recipe、模板文件和生成脚本来构造 `node_modules` 式深层嵌套目录树。生成结果应满足至少 `100k` regular files，目录深度和分支形态应接近典型 `npm` / `yarn classic` 依赖树，文件类型以 `.js`、`.json`、`.d.ts`、README、LICENSE 等小文本为主，并允许掺入少量中等大小文件作为噪声。

这样做的原因是：

- 真实目标 workload 需要 `100k+` 文件和大量目录层级，直接提交完整输入树会让仓库体积、审阅成本和维护复杂度失控。
- 纯复制小 seed 目录虽然能放大文件数，但会失真为大量平行子树，不能代表 `node_modules` 的深层依赖拓扑。
- committed recipe 既能保持可重复，也能把“为什么这是 `node_modules` 式 workload”写成审计资产。

备选方案：

- 直接提交一个真实 `node_modules` 树：最接近实物，但仓库膨胀、许可证边界和升级漂移都过高。
- 只复制现有 `small-text` 若干千份：能达到文件数目标，但目录拓扑和文件分布不够贴近目标场景。

### 3. Result schema 改为围绕单次默认命令结果，而不是 dataset × codec 矩阵

benchmark 报告仍保持机器可读 JSON，但组织方式应优先表达默认路径 benchmark 的核心字段：

- workload identity 和生成参数摘要
- pack / unpack 的 wall time
- files/s、MiB/s、output/archive size
- CPU / RSS
- SFA 的结构化 pack/unpack stats 与 unpack 观测字段

结果不再要求通过 `dataset × baseline × codec × phase` 的矩阵心智来消费；相反，应让 README、脚本和 reviewer 直接能读取“默认 benchmark 在目标 workload 上的对照结果”。如果保留 dry-run 输出，则 dry-run 也应反映默认命令与 workload 生成摘要，而不是展开已经不存在的 codec matrix。

备选方案：

- 继续复用当前 record 扁平列表，只减少 case 数量：兼容成本低，但消费者仍需要自己还原“哪一组是默认路径 headline”。
- 单独为 README 生成另一份摘要文件：可读性高，但会形成 benchmark JSON 与 README 摘要双重真相源。

### 4. Release gate 和 showcase benchmark 分层，而不是继续让一个小矩阵同时承担两种角色

仓库仍然保留 benchmark dry-run 作为 release checklist 的一部分，但文档需要明确区分：

- release smoke / preflight：验证 runner、workload recipe 和命令生成仍可执行
- supported-host baseline refresh：在受支持环境上生成默认路径 benchmark 结果
- README / release evidence：引用默认路径 benchmark 的结果，而不是小型 fixture 对照

这样做的原因是当前问题的根源之一就是小矩阵同时承担 release gate 和 positioning evidence，导致两边都不够理想。显式分层后，dry-run 继续轻量，真正的 benchmark 证据则围绕目标 workload 组织。

备选方案：

- 继续把同一组 tiny committed fixture 当作 release gate 和对外证据：最省事，但不会解决“benchmark 不吸引人”的核心问题。

## Risks / Trade-offs

- [生成型 workload 漂移出真实 `node_modules` 形态] → 在 recipe 中固定目录深度、分支分布、文件类型分布和路径模式，并把这些约束写入 spec 和 README。
- [100k+ workload 运行成本较高] → 保留 dry-run 作为日常 smoke，真实 baseline 仅在支持环境和需要刷新结果时执行。
- [默认线程数使跨机器比较变弱] → 在结果和文档中明确 benchmark 的解释边界，将其定位为固定 benchmark host 上的默认体验证据。
- [移除 codec matrix 后失去某些底层可比性] → 接受这一取舍，把 matrix 视为可选诊断手段而不是主 benchmark 合同。
- [历史结果和脚本消费者依赖旧 schema] → 通过文档和测试明确 schema 变更，并在刷新 baseline 时同步更新消费者。

## Migration Plan

1. 定义 `node_modules-100k` workload contract，并引入 committed recipe / generator 资产。
2. 调整 benchmark runner、脚本和报告 schema，使其围绕默认 pack/unpack 路径和 canonical tar baseline 执行。
3. 在受支持环境上运行新的默认 benchmark，提交刷新后的结果资产。
4. 同步更新 README、benchmark 文档和 release guidance，移除对 `lz4` matrix 和 tiny fixture headline 的依赖。

如果实现中发现生成型 workload 仍不足以代表目标场景，可在 follow-up change 中调整 recipe 或再补一个 link-heavy workload；不需要迁移 `.sfa` 归档协议资产。

## Open Questions

- canonical tar baseline 是否需要把线程参数也固定为显式值，还是仅固定 `zstd -3` 并接受 `zstd` CLI 默认线程行为。
- 现有 `small-text` / `small-binary` / `large-control` fixture 是完全退出 benchmark 主链路，还是保留给非 headline 的诊断或回归测试使用。
