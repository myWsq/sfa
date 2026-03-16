# Changelog

本文件用于记录 SFA 的仓库级变更。当前项目仍处于 v1 开发阶段，在 `M0` 协议冻结前，协议兼容性可能发生调整。

## [Unreleased]

## [0.1.0] - 2026-03-16

### Added

- 建立 Rust workspace，包含 `sfa-core`、`sfa-unixfs`、`sfa-cli`、`sfa-bench`
- 实现 `sfa pack` / `sfa unpack` MVP 主链路，以及 `lz4` / `zstd` 编解码与 fast / strong 完整性校验选项
- 实现 manifest-first 的 `.sfa` 结构，包括 header、manifest、frame 与 optional trailer
- 支持 regular file、directory、symlink、hardlink 的扫描、打包与恢复
- 增加 `cargo test --workspace`、protocol / streaming / safety / roundtrip smoke checks，以及 benchmark dry-run CI 流程

### Changed

- 补充 `README.md`、`RELEASING.md`、`ROADMAP.md`，明确项目状态、发版流程与里程碑
- 建立 benchmark harness、fixture 目录结构与 verification baseline，作为后续协议冻结和基线收口的基础
