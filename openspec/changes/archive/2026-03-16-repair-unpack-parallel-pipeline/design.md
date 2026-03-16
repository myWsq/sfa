## Context

SFA 的 pack 路径已经具备 bundle 级并行，但 unpack 仍停留在“顺序 reader + 串行 decode/scatter”的实现状态。当前 `ArchiveReader::next_frame()` 会先为 frame hash 校验解码 payload，而 `unpack_archive()` 随后再次解码同一 payload 做 scatter，导致每个 frame 在正常解包路径上发生冗余解码。与此同时，`UnpackConfig.threads` 虽然已经暴露在 CLI 和 stats 中，但并没有进入实际的 bundle 级工作调度。

最近针对真实 Next.js `node_modules` 语料的 thread sweep 暴露了这个偏差：pack 随线程数明显扩展，而 unpack 基本不随线程数变化，甚至出现高线程下的异常抖动。项目当前的 benchmark 观测又把 unpack 主成本压缩成 `decode_and_scatter` 一个桶，难以判断问题究竟在 frame 读取、codec 解码、文件散写还是 finalize 阶段。

这个 change 需要在不改变 `.sfa` wire format 和顺序流读取语义的前提下，把 unpack 拉回到技术方案中承诺的 bundle 级并发流水线，并让 `--threads`、CLI stats、benchmark 结果三者重新一致。

## Goals / Non-Goals

**Goals:**
- 让 `unpack --threads` 真正控制 bundle 级解包/恢复并发，而不是只回显到 stats。
- 消除正常 unpack 路径中的冗余 frame 解码，同时保持现有 frame / trailer 完整性语义。
- 将 unpack 观测从 `decode_and_scatter` 拆分为可解释的稳定阶段，支持 benchmark 与人工排障。
- 为 unpack 并发接线增加非脆弱的回归验证，确保后续不会再次出现“接口存在但执行未接线”的情况。

**Non-Goals:**
- 不改变 `.sfa` wire format、manifest 结构或 frozen protocol 语义。
- 不重新设计 pack pipeline、planner 或 benchmark 数据集矩阵。
- 不在这个 change 中引入随机访问、async unpack API 或新的归档功能面。
- 不用绝对性能倍数作为 CI 闸口；性能趋势由 benchmark 记录，功能性接线由确定性回归测试保证。

## Decisions

### 1. 将 `ArchiveReader` 收敛为顺序 framing 层，解码与 frame 校验下沉到 unpack pipeline

当前 `ArchiveReader::next_frame()` 在返回 `EncodedFrame` 之前就完成 payload 解码和 frame hash 校验，导致上层 unpack 只能再次解码同一 payload 才能拿到 raw bundle。这个职责分层不利于 phase 统计，也让协议层被迫承担高成本 codec 工作。

本 change 将把 `ArchiveReader` 调整为顺序 framing / payload 读取层：它仍负责 header、manifest、frame header 和 payload 的顺序读取与边界校验，但不再在 `next_frame()` 内部执行 codec 解码。解码和 frame hash 校验移动到 unpack pipeline 的 decode 阶段完成，从而保证每个 frame 在正常解包路径上只做一次解码。

选择这种方式，而不是让 `ArchiveReader` 直接返回已解码 raw bundle，原因是：
- 它更符合“顺序读取层”和“restore 执行层”的边界；
- 它允许 unpack stats 把 `frame_read` 和 `decode` 分开建模；
- 它避免未来其他消费者被迫承担不需要的 decode 成本。

### 2. unpack 改为有界三段式流水线：`FrameReadQueue -> DecodeQueue -> ScatterQueue`

解包仍然必须以单顺序 reader 消费 archive 字节流，但 reader 读出 frame 后，应把 bundle 工作分发给后续并发阶段。目标拓扑是：

```text
sequential reader
  -> bounded frame queue
  -> decode+verify worker pool
  -> scatter writer stage
  -> metadata/link/trailer finalize
```

具体语义：
- reader 线程顺序读取 frame header + payload，并把 bundle 工作单元放入有界队列；
- decode worker 负责 codec 解码与 frame hash 校验；
- scatter 阶段将 raw bundle extents 写入目标 regular file；
- symlink、hardlink、mtime/mode/owner、directory finalize 和 strong trailer 校验仍在收尾阶段统一处理。

这里不要求所有后续阶段都严格按 `bundle_id` 顺序完成；要求的是顺序流输入不依赖 seek，且对外恢复语义保持安全正确。内部允许多个 bundle 的 decode / scatter 并发执行，只要 regular-file extents 仍按 `file_offset` 写入且链接/目录 finalize 语义不被破坏。

备选方案：
- 只并行 decode，scatter 继续串行：改动较小，但对海量小文件 workload 的收益有限，不能充分兑现 `--threads` 语义。
- 完全改为 async-first pipeline：与 v1 当前的 sync `Read` 主线不匹配，复杂度过高。

### 3. 将 `LocalRestorer` 拆成“并发 regular-file 写入”与“串行元数据 finalize”两个职责层

当前 `LocalRestorer` 是单个可变对象，文件句柄缓存、extent 写入、链接创建和 metadata finalize 全部由一个串行调用链驱动。这种设计天然阻碍 bundle 级 scatter 并发。

本 change 会把 restore 模型分成两层：
- 并发数据层：负责 regular-file 的按偏移写入、必要的文件创建/句柄缓存和 extent 生命周期；
- 串行 finalize 层：负责 symlink、hardlink、directory finalize、mtime/mode/owner 和 trailer 收尾。

regular-file 写入允许并发执行，因为 extents 已经明确给出 `entry_id + file_offset + raw_len`，而 `write_at` 对不同 offset 的写入可以独立调度。为了避免 fd 爆炸，句柄缓存仍需保留上界，但实现要改成支持多 worker 安全共享，而不是依赖单个可变 `HashMap<u32, File>`。

### 4. unpack phase breakdown 改为稳定但可重叠解释的阶段窗口

当前 unpack stats 只有 `header`、`manifest`、`decode_and_scatter`、`restore_finalize`。在真正并行后，`decode_and_scatter` 会继续掩盖问题，而强行要求所有阶段时长可相加又会扭曲流水线设计。

本 change 将 unpack 的 machine-readable phase breakdown 调整为：
- `header`
- `manifest`
- `frame_read`
- `decode`
- `scatter`
- `restore_finalize`

这些字段定义为稳定的阶段观测窗口，不要求严格相加等于 total duration。文档和 benchmark schema 会明确：并行 unpack 的 phase breakdown 用于诊断瓶颈和解释扩展性，不能简单把所有 phase duration 求和当作总 wall-time。

### 5. 用确定性回归测试证明“线程接线有效”和“每 frame 只解码一次”，而不是依赖 timing ratio

仅靠 benchmark 结果很容易再次出现“看起来快/慢，但不知道 wiring 是否真的生效”的情况。因此这个 change 需要补一类更硬的验证：
- 多 bundle fixture 上，显式线程覆盖必须进入真正的 unpack worker 调度；
- 正常 unpack 路径中，frame decode 次数必须与 bundle 数一致，而不是每个 frame 多次解码；
- 新的 unpack stats 字段在 dry-run 和真实执行中都具有稳定语义。

为此，实现应引入最小必要的测试缝隙，例如对 decode/scatter 调度加 crate-private probe 或计数器，而不是在 CI 中写基于 wall-time 倍数的脆弱断言。benchmark 和 thread sweep 仍保留，但它们是解释性能结果的证据，不是唯一的正确性闸口。

## Risks / Trade-offs

- [并发 scatter 增加 restore 层复杂度] → 把 regular-file 数据写入和 metadata finalize 明确分层，并保留目录/链接收尾的串行安全边界。
- [phase breakdown 改为可重叠窗口后更难直观相加] → 在 spec、README 和 benchmark 文档中明确这些字段的诊断语义，避免继续把它们误读为可加总账。
- [线程接线测试需要新的内部探针] → 控制探针范围为 crate-private/test-only，不把内部调度细节暴露成稳定公共 API。
- [更高并发可能放大 fd / 内存峰值] → 使用有界队列、受限句柄缓存和有限在途 bundle 数作为背压机制。

## Migration Plan

这个 change 不涉及 archive 迁移或协议兼容性切换；现有 `.sfa` 归档仍应保持可读。交付顺序应为：

1. 先调整 spec 和 stats 契约，明确 unpack phase 与线程语义。
2. 重构 `ArchiveReader` / unpack pipeline / restore 层，确保无双解码且线程覆盖真正生效。
3. 补上回归测试、benchmark schema 调整和文档更新。
4. 在受支持环境中刷新 benchmark 结果与相关说明，确认新 unpack 观测与 thread sweep 可解释。

若实现过程中发现并发 scatter 需要更大范围的 restore API 变动，应以“不破坏冻结协议和现有 CLI 语义”为回退边界，优先缩减内部实现方式，而不是放弃线程生效性目标。

## Open Questions

- 并发 scatter 的具体执行器是否继续复用 `rayon`，还是引入更适合 bounded MPMC pipeline 的小型 channel 依赖。
- regular-file 句柄缓存是否需要改成分片/并发 map，还是通过更积极的“先创建、后按需打开”策略就足够。
- unpack phase breakdown 是否还需要保留一个兼容别名字段帮助旧 baseline 消费者过渡，还是直接一次性切换并刷新仓库内 baseline。
