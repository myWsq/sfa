## Why

SFA 的 `unpack` 主链路已经修复了线程接线和双解码问题，但与原始技术方案相比仍有几处实质偏差：当前执行路径仍然把 `decode` 和 `scatter` 绑定在同一个 worker 中；恢复层继续依赖路径拼接和 `PathBuf` 语义，而不是原方案承诺的 `dirfd/openat` 风格安全 IO；对外只暴露 path-based unpack API，也没有 CLI `stdin` 解包入口；`strong` trailer 失败时只返回错误，没有对已恢复内容写出明确的不可信标记。

这些偏差会直接影响项目“按原始技术方案落地”的可信度，也会限制后续继续优化 `unpack` 吞吐和安全语义的一致性。这个 follow-up change 的目标不是重新设计协议或扩大功能面，而是把 `unpack` 重新收敛到最初方案承诺的架构、恢复模型和对外接口边界。

## What Changes

- 将 `unpack` 执行路径从当前的 `reader -> worker(decode+scatter)` 进一步重构为更贴近原方案的 `FrameReadQueue -> DecodeQueue -> ScatterQueue -> finalize` 流水线。
- 将 Unix 恢复层切换到 `dirfd/openat` 风格安全 IO，实现相对父目录 fd 的目录创建、regular file 打开、symlink/hardlink 创建和元数据应用。
- 增加 `unpack_reader_to_dir<R: Read>` 公开入口，并让 CLI `sfa unpack -` 支持从 `stdin` 解包。
- 在 `strong` trailer 校验失败时，于输出根写固定 `.sfa-untrusted` marker 后返回错误。
- 刷新相关文档、baseline 和 thread-sweep 说明，使仓库内对 `unpack` 的行为描述与原始技术方案重新一致。

## Capabilities

### New Capabilities

- `archive-unpack`: 新增 sync `Read` 解包入口与 `strong` trailer failure marker 语义。

### Modified Capabilities

- `archive-unpack`: 收紧 `unpack` 流水线结构、Unix 安全恢复语义和 metadata finalize 行为，使其更贴近原始技术方案。
- `cli-and-benchmarks`: `sfa unpack` 支持 `stdin`，并要求 benchmark / verification 文档覆盖新的流水线结构与 `.sfa-untrusted` 语义。

## Impact

- 影响 `crates/sfa-unixfs` 的 `unpack` pipeline、restore 层实现、trailer failure handling 和公开 API。
- 影响 `crates/sfa-cli` 的 `unpack` 输入模型、usage 错误分支和 `stdin` 执行路径。
- 影响 `README.md`、`ROADMAP.md`、`spec/verification-and-benchmark.md` 以及 benchmark 结果说明。
