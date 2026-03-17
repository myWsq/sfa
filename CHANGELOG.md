# Changelog

本文件用于记录 SFA 的仓库级变更。当前项目仍处于 v1 开发阶段，`format-v1` 已冻结，后续兼容性调整通过 OpenSpec change 管理。

## [Unreleased]

## [0.2.0] - 2026-03-17

### Added

- 冻结 `format-v1` 协议文本，补齐 canonical golden fixtures、冻结评审记录与 protocol smoke 基线
- 增加 committed benchmark datasets、`tar + same codec` machine-readable baseline，以及 SFA 阶段级 / 资源级 benchmark 观测
- 增加 `sync Read` / `stdin` 解包入口、真实 bundle 级 unpack worker 调度、`dirfd/openat` 风格 restore 路径，以及 `strong` trailer 失败时的 `.sfa-untrusted` 标记
- 增加 CLI 默认值、usage error、`stdin` / `--dry-run` 组合和 overwrite 语义的回归测试
- 新增面向发版与里程碑收口的 `release-readiness` OpenSpec capability 与 release candidate notes 草案

### Changed

- 将仓库状态同步为 M1 最小可用链路已完成、M2 性能主链路已完成，下一轮重点转向 M3 Unix 语义增强
- 将 `cargo fmt --all --check`、workspace tests、smoke checks 与 benchmark dry-run 明确为权威发版检查清单
- 同步 `README.md`、`ROADMAP.md`、`RELEASING.md` 与版本号，使发版文档、里程碑状态和仓库实现保持一致

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
