## Context

原始技术方案要求 `unpack` 采用 manifest-first、顺序流读取、bundle 级并行解码与 scatter 的三段式流水线，并且所有 Unix 恢复操作都基于输出根 `dirfd` 做相对路径处理。当前仓库虽已实现顺序流 `ArchiveReader`、bundle worker 调度、`write_at` scatter 和基础的并发句柄缓存，但仍然存在下列偏差：

- `decode` 与 `scatter` 仍在同一个 worker 中串接执行，难以进一步独立优化两个阶段。
- 路径安全主要依赖 `safe_join + symlink_metadata + PathBuf`，没有使用 `openat/mkdirat/linkat/symlinkat` 风格的相对 fd 恢复模型。
- 对外只有 path-based `unpack_archive(path, ...)`，缺少原方案中的 `unpack_reader_to_dir<R: Read>` 和 CLI `stdin` 解压。
- `strong` trailer mismatch 只会返回错误，不会显式标记输出目录已恢复内容“不可信”。

## Goals / Non-Goals

**Goals:**
- 让 `unpack` 执行模型重新贴近原始方案的 `FrameReadQueue -> DecodeQueue -> ScatterQueue -> finalize` 拓扑。
- 将恢复层切换到 `dirfd/openat` 风格安全 IO，同时保持现有 overwrite/owner/mtime 语义。
- 增加 sync `Read` 解包入口与 CLI `stdin` 解包能力，不引入新的 stats schema 字段。
- 对 `strong` trailer mismatch` 给出稳定、可回归的 `.sfa-untrusted` 标记语义。

**Non-Goals:**
- 不更改 `.sfa` wire format、manifest 结构和任何冻结协议字段。
- 不实现 `AsyncRead`、xattrs、ACL、special file 或事务式临时目录切换。
- 不引入新的 CLI 参数，也不修改 benchmark JSON schema。

## Decisions

### 1. `unpack_archive(path, ...)` 下沉为 wrapper，真实执行入口改为 `unpack_reader_to_dir<R: Read>`

库层新增：

```rust
pub fn unpack_reader_to_dir<R: std::io::Read>(
    reader: R,
    output_dir: &Path,
    config: &UnpackConfig,
) -> Result<UnpackStats, UnixFsError>;
```

`unpack_archive(path, ...)` 只负责打开文件并调用该入口。这样 CLI `stdin` 解包与 fragmented reader 测试都可以复用同一条真实执行路径，而不是再维护一套 path-only 逻辑。

`CLI unpack - --dry-run` 不在本轮支持，因为 dry-run 需要 archive header/manifest 统计而 stdin 没有 seek/replay 能力；这里统一返回 usage error。

### 2. `unpack` 执行路径拆成三个明确阶段

内部拓扑固定为：

```text
sequential reader
  -> FrameReadQueue<EncodedFrame>
  -> DecodeQueue<DecodedBundleTask>
  -> ScatterQueue / scatter workers
  -> metadata/link/trailer finalize
```

语义约束：
- reader 线程只做 `next_frame()` 和 queue enqueue，记录 `frame_read_ms`
- decode worker 只做 `decode_data + frame_hash verify`，记录 `decode_ms`
- scatter worker 只做 extent `write_at`，记录 `scatter_ms`
- `restore_finalize_ms` 继续覆盖 close/finalize/link/directory/trailer 收尾

这里不强制 `decode_ms + scatter_ms` 等于 wall-time；两个阶段保持窗口化统计即可。队列深度固定为 `2 * effective_threads`。

### 3. 恢复层改为 dirfd 风格安全 IO，但保持现有 policy 语义

`sfa-unixfs` 内部新增/重写一层以输出根 dirfd 为锚点的恢复实现：

- 目录创建：相对父 dirfd 逐级创建，拒绝现有 symlink 逃逸
- regular file：第一次 write 前通过 `openat` 惰性创建或打开
- symlink：通过 `symlinkat`
- hardlink：在 master file finalize 完成后通过 `linkat`
- metadata：regular file 和 directory 都恢复 `mode/mtime`；如 `restore_owner=preserve` 且 root，则恢复 `uid/gid`

regular file 句柄缓存继续沿用“有上界 + 懒打开 + 分片”的思路，但 key 不再是 PathBuf 解析结果，而是以 `entry_id -> relative path parts -> dirfd/openat` 这套恢复模型驱动。symlink 自身 metadata 继续不恢复。

### 4. `strong` trailer mismatch 写固定 marker

若启用 strong integrity 且 trailer mismatch：
- 先在输出根写 `.sfa-untrusted`
- 内容固定至少包含 `strong trailer verification failed`
- 然后返回错误

执行前如果输出根已有旧 marker，则先删除，避免成功解包后残留旧状态。这里不引入事务式 temp-dir switch；marker 是最小且稳定的“不可信”语义。

### 5. 验证重点从“线程接线”扩展到“流式入口 + dirfd 安全 + marker 语义”

本轮新增的确定性验证包括：
- `unpack_reader_to_dir` 对 fragmented `Read` 的 roundtrip
- CLI `sfa unpack - -C out` 从 stdin 成功恢复
- 输出根内预置恶意 symlink 时，dirfd 风格恢复拒绝路径逃逸
- trailer 损坏时生成 `.sfa-untrusted`

真实 `node_modules` thread sweep 继续保留，用来确认这次拆段后至少不劣于当前已知基线：
- `lz4`: `t1=6001ms`, `best=4427ms@t4`
- `zstd`: `t1=5872ms`, `best=3796ms@t8`

## Risks / Trade-offs

- `dirfd/openat` 重写 restore 层会增加实现复杂度，但可以把安全模型与原始技术方案重新对齐。
- `stdin` dry-run 不支持会让 CLI 语义略显不对称，但这是为了避免在无 replay 能力的输入上伪造 summary。
- `decode` 与 `scatter` 真正拆段后，若 queue/backpressure 不合理，可能引入额外拷贝或内存峰值；本轮用固定 queue 深度和 bounded channel 控制风险。
- marker 方案比 temp-dir 切换更弱，但不会引入额外 IO/空间放大，符合本轮范围。
