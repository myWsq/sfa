# SFA

SFA（Streaming Folder Archive）是一个面向 Unix 目录树的流式归档格式与工具链。它的目标不是简单替代 `tar`，而是提供一套更适合 manifest-first、顺序读取、完整性校验与性能基线对比的归档方案。

## 项目状态

当前项目处于 `开发中` 状态，仓库已经具备可运行的 v1 MVP，并且 `format-v1` 协议已经冻结。

这意味着：

- `sfa pack` / `sfa unpack` 已可运行
- 主要主链路已经存在并有测试覆盖
- `.sfa` v1 的协议边界、canonical golden fixture 和冻结评审记录已经提交
- 当前主要收口项转向 benchmark baseline、真实 dataset 和更完整的回归资产

项目阶段和里程碑见 [ROADMAP.md](ROADMAP.md)。

## 当前能力

- `sfa pack` / `sfa unpack` 端到端归档与恢复
- manifest-first 的 `.sfa` 结构：header、manifest、frame、optional trailer
- `lz4`、`zstd` 编解码支持
- 确定性目录扫描、稳定 bundle 规划与顺序读写
- regular file、directory、symlink、hardlink 支持
- 完整性校验、路径安全检查与基础损坏检测
- roundtrip、streaming、corruption、safety 测试框架
- `tar + same codec` benchmark harness

## 当前不承诺的范围

- 全量 Unix 扩展语义，例如 xattrs / ACL
- 非 Unix 平台上的完全一致行为
- 已发布的安装包或 crates.io 分发流程

## 快速开始

环境要求：

- Rust `1.85` 或更高版本
- Unix-like 环境

构建 CLI：

```bash
cargo build --release -p sfa-cli
```

打包目录：

```bash
cargo run -p sfa-cli -- pack ./input ./archive.sfa --codec zstd --integrity strong
```

解包归档：

```bash
cargo run -p sfa-cli -- unpack ./archive.sfa -C ./restore
```

运行测试：

```bash
cargo test --workspace
bash tests/scripts/run_protocol_smoke.sh
bash tests/scripts/run_streaming_smoke.sh
bash tests/scripts/run_safety_smoke.sh
bash tests/scripts/run_roundtrip_smoke.sh
```

## 仓库结构

- `crates/sfa-core`：归档格式、codec、integrity、planner、顺序 reader
- `crates/sfa-unixfs`：Unix 文件系统扫描、打包与恢复实现
- `crates/sfa-cli`：命令行入口
- `crates/sfa-bench`：benchmark 与 fixture dump 工具
- `spec/`：冻结后的协议与验证规范
- `tests/`：回归测试、fixture 与 smoke scripts
- `sfa-tech-solution/`：当前技术方案文档
- `openspec/`：change proposal、design 与 task 拆解

## 文档导航

- [ROADMAP.md](ROADMAP.md)：仓库级路线图与项目状态
- [RELEASING.md](RELEASING.md)：发版流程与质量闸口
- [CHANGELOG.md](CHANGELOG.md)：版本变更记录
- [spec/README.md](spec/README.md)：协议与验证规范入口
- [spec/format-v1-freeze-review.md](spec/format-v1-freeze-review.md)：v1 协议冻结评审记录
- [sfa-tech-solution/README.md](sfa-tech-solution/README.md)：技术方案总览

## 发版

当前仓库采用手动发版流程：

- 使用 Git tag 标记版本
- 在 GitHub 上创建对应 Release
- 以仓库中的测试、smoke checks 和 benchmark baseline 作为发版前质量闸口

具体步骤见 [RELEASING.md](RELEASING.md)。

## 贡献

当前仓库仍在 v1 主链路建设阶段。提交较大改动前，建议先对照：

- [ROADMAP.md](ROADMAP.md)
- [spec/README.md](spec/README.md)
- [sfa-tech-solution/README.md](sfa-tech-solution/README.md)

如果改动涉及协议、fixture 或 benchmark 基线，优先保持文档、测试资产与实现同步更新。

## 许可

本项目采用 MIT License，详见 [LICENSE](LICENSE)。
