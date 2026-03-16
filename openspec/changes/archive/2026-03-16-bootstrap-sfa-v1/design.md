## Context

SFA 目前只有完整的技术方案文档，还没有任何实现代码、规格契约或任务拆解。这个 change 的目标不是一次性覆盖全部路线图，而是把 SFA v1 的首个开源可用范围冻结下来，让协议、打包/解包行为、CLI 入口和 benchmark 基线先形成稳定主线。

这是一个典型的跨模块设计变更：协议、文件系统恢复、CLI、测试与 benchmark 都会一起落地。约束也非常明确：Rust 实现、Unix-only、严格顺序流式读取、不依赖 seek、吞吐优先于压缩比、与 `tar + 同算法` 做同口径对比。

## Goals / Non-Goals

**Goals:**
- 建立 Rust workspace 与 crate 边界，使协议层、Unix 文件系统层、CLI 层、benchmark 层职责清晰。
- 冻结 `.sfa` v1 的 manifest-first 归档结构，支撑顺序流解包和 bundle 级并行。
- 提供首个可用的 `pack` / `unpack` MVP，覆盖 regular file、directory、symlink、hardlink。
- 把 benchmark、golden、streaming、安全与损坏样例纳入首轮交付，而不是实现后补。
- 保持实现可重复、可测试、可对标，便于后续以开源项目方式演进。

**Non-Goals:**
- 不在 v1 首轮交付中实现 Windows 兼容、随机访问、增量归档、去重、加密、签名或断点续传。
- 不在首轮实现中交付 xattrs、ACL、special file 恢复，只保留协议扩展位。
- 不为追求极致压缩比而牺牲 bundle 级吞吐和稳定顺序流。
- 不在这个 change 中引入服务化接口、远程控制面或复杂 async-first API。

## Decisions

### 1. 采用四层 Rust workspace 结构，而不是单体 crate

仓库将按 `sfa-core`、`sfa-unixfs`、`sfa-cli`、`sfa-bench` 组织：
- `sfa-core` 负责 Header / Manifest / Frame、planner、codec、integrity、pipeline。
- `sfa-unixfs` 负责 Unix 扫描、安全恢复、元数据应用和路径安全。
- `sfa-cli` 负责参数解析、命令调度、日志、错误码和统计输出。
- `sfa-bench` 负责 tar 基线和数据集驱动 benchmark。

这样拆分的原因是协议与 Unix 恢复逻辑都足够复杂，放进一个 crate 会让测试和边界迅速失控。相比继续细拆更多 crate，这个结构更容易在绿地项目早期保持演进速度。

备选方案：
- 单体 crate：实现快，但协议、恢复、安全和 CLI 耦合过重，后续难维护。
- 更细粒度 crate：边界更纯，但在项目刚起步时会引入过多工程开销。

### 2. 归档格式固定为 manifest-first 的顺序流布局

`.sfa` v1 采用 `HeaderV1 + ManifestSection + DataFrame* + TrailerV1?`。Header 固定长度、小端编码；Manifest 在数据段前部完整可得；Frame 顺序出现；Trailer 仅在 strong 模式或显式启用时存在。

这个决定直接服务于两个核心目标：
- 解包只依赖顺序读取，不依赖 seek，因此本地文件流和 HTTP body 可以复用同一条逻辑。
- 解包在读到第一个 frame 前就能拿到完整恢复计划，包括目录树、extent 映射、symlink/hardlink 信息和预期 bundle 序列。

备选方案：
- tar 风格“每文件 header + body”线性布局：实现简单，但无法把小文件聚合为 bundle，也不能在严格流式前提下提前拿到全量恢复计划。
- 数据段先写、回填 manifest/offset：要求 seek，直接违背网络流输入约束。

### 3. bundle 规划采用稳定线性聚合，而不是复杂装箱

规划器会先用 `lstat` 扫描整棵目录树，稳定排序后再做 bundle 规划：
- 小于 `small_file_threshold` 的 regular file 进入聚合 bundle。
- 大于等于阈值的 regular file 按 `bundle_target_bytes` 切分成单文件 bundle。
- directory / symlink / hardlink / empty file 不进入 bundle 数据体。
- hardlink 以 `(st_dev, st_ino)` 识别，master 才拥有 extents。

这里故意选择稳定线性聚合，而不是全局 bin packing。原因是 v1 更需要 deterministic output、实现可诊断和足够好的吞吐收益，而不是更难解释的最优装箱结果。

备选方案：
- first-fit / best-fit 等全局装箱：理论上可能提高局部打包密度，但复杂度更高，输出更不稳定，收益不确定。
- 每文件一个压缩流：会保留小文件场景最关键的上下文切换和 syscall 开销。

### 4. pack/unpack 流水线采用有界队列 + 并行 worker + 有序收尾

pack 侧将采用 `Scan/Plan -> ReadQueue -> CompressQueue -> OrderedWriteQueue`；unpack 侧采用 `FrameReadQueue -> DecodeQueue -> RestoreQueue`。所有阶段都使用 bounded channel 控制在途 bundle 数量，防止内存峰值失控。

关键实现点：
- 压缩 worker 可以乱序完成，但 `OrderedWriter` 必须按 `bundle_id` 顺序落盘。
- 解包在 Manifest 就绪后预构建 `bundle_to_extents` 索引，raw bundle 解码后用 `write_at` scatter 到目标文件。
- 解包线程数默认参考 header 中的 `suggested_parallelism`，但允许 CLI 或库调用方覆盖。

备选方案：
- 完全串行 pack/unpack：实现简单，但直接放弃多核吞吐收益。
- 依赖 codec 自带线程模型：无法稳定覆盖 bundle 级调度和 writer/read order 约束。

### 5. Unix 恢复层默认安全优先，所有文件系统操作基于 dirfd 风格接口

`sfa-unixfs` 会以输出根 `dirfd` 为锚点执行路径解析和恢复操作，拒绝绝对路径、`..`、空路径段、NUL 和通过现有 symlink 的路径逃逸。symlink target 只作为恢复数据，不参与父路径解析；special file 默认禁止创建；owner 恢复仅在显式策略允许时启用。

这样做的理由很直接：解包本质上是高风险输入处理，如果走简单的字符串拼接路径 API，很难保证不会被恶意归档或恶意现存目录结构绕出输出根。

备选方案：
- 直接使用路径拼接和进程 cwd：实现更短，但无法提供可靠的路径安全保证。
- 先恢复到临时目录再搬运：会增加额外 IO 和空间占用，也偏离严格流式恢复目标。

### 6. benchmark 与协议回归测试作为首轮交付的一部分

SFA 的卖点不是“能工作”，而是“在海量小文件场景下比 `tar + 同算法` 更有吞吐优势”。因此 benchmark、golden 样例、streaming 分片输入测试、corruption/security 测试必须与 MVP 一起进入仓库，而不是等实现完成后补。

首轮测试/基准将覆盖：
- roundtrip 正确性
- 逐字节或随机分块输入的流式解包
- header / manifest / frame / trailer 损坏
- 路径安全和 special file 拒绝
- tar + lz4 / tar + zstd 的同口径对标

备选方案：
- 先做功能、后补 benchmark：会导致性能目标缺乏可追踪基线，偏离项目定位。

## Risks / Trade-offs

- [Manifest 在超大目录树下变大] → 使用 manifest codec 压缩、限制实现首版的输入边界，并把 manifest 大小和解析耗时纳入统计输出。
- [bundle 过大导致内存峰值偏高] → 使用有界队列、buffer 复用和可调 bundle 参数，默认保守在 4 MiB 量级。
- [hardlink / symlink 恢复顺序出错会破坏文件树语义] → 在 `sfa-unixfs` 中单独建模恢复顺序，并以 roundtrip 和 inode 断言覆盖。
- [协议字段在实现初期频繁漂移] → 先冻结 v1 线框并尽早生成 golden 样例，再做性能优化。
- [benchmark 结果受环境噪声影响] → 统一数据集、同算法对照、固定指标口径，并把基准脚本纳入仓库。
- [HTTP 流支持与 async API 绑定过早] → v1 先保证 sync `Read`/状态机可顺序消费，异步接口后续作为扩展层补充。

## Migration Plan

这是一个绿地 change，没有既有线上用户或历史归档需要迁移，因此不存在传统意义上的数据迁移或回滚窗口。交付顺序按以下阶段推进：

1. 冻结协议和 capability specs，建立 workspace、crate 和测试目录骨架。
2. 实现 Header / Manifest / Frame 编解码、planner 和基础 pack 路径，生成第一批 golden 样例。
3. 实现顺序流 unpack、安全恢复、LZ4 fast integrity MVP，并接通 CLI。
4. 补齐 Zstd、strong integrity、benchmark 基线和更多损坏/安全样例。

如果实现过程中发现 wire format 或 CLI 语义存在重大问题，应在首次 release 前直接修改 change/spec，而不是兼容一个未发布的错误接口。

## Open Questions

- `manifest_codec` 的默认级别参数是否需要在首次实现中暴露，还是先固定为内部默认值，仅暴露算法选择。
- benchmark 数据集的具体来源与许可证需要在仓库初始化时确认，但这不阻塞当前 change 的规格与任务拆解。
