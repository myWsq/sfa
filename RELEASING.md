# SFA 发版流程

本文档定义 SFA 仓库当前采用的发版流程。对外版本仍以 Git tag 为唯一发布触发器，但 GitHub Release 由 [release workflow](.github/workflows/release.yml) 在 tag 推送后自动完成；只有在 workflow 不可用时才退回手动创建 Release。

## 适用范围

当前流程适用于：

- 仓库级版本发布
- Git tag 与 GitHub Release
- 协议、测试资产、benchmark 基线的发版前检查

当前流程不包含：

- crates.io 发布
- 多平台安装器分发
- macOS notarization / codesign

## 发版原则

- 每个对外版本都必须有可追溯的 Git tag
- 发版内容必须与仓库状态、路线图和协议文档一致
- 协议相关改动不能只发代码，必须同时更新 spec、fixtures 和验证资产
- 工作区必须干净；存在未提交或未跟踪变更时不允许发版
- 发版前必须通过仓库定义的质量闸口

## 发版前提

满足以下条件后才进入发版：

1. 对应 OpenSpec change 已完成，或本次发版内容在仓库中已有明确结论。
2. [ROADMAP.md](ROADMAP.md) 与 [README.md](README.md) 中相关里程碑状态和顶层项目状态已同步。
3. 如果发版涉及协议或解码行为变化：
   - `spec/format-v1.md` 已更新
   - golden fixtures 已更新，并保持代表性的 canonical corpus 覆盖
   - 兼容性影响已记录在 release notes 中
4. 待发布内容已经完成代码 review。
5. `git status --short` 结果为空，工作区没有未提交或未跟踪变更。

## 标准发版步骤

### 1. 确认版本范围

首先确认工作区干净：

```bash
git status --short
```

如果输出非空，先整理工作区，再继续发版流程。

明确本次版本属于以下哪类：

- `patch`：修复缺陷，不改变预期接口和协议行为
- `minor`：新增能力，但保持兼容
- `major`：发生不兼容变更

如果 `format-v1` 仍未冻结，则即使版本号按 SemVer 递增，也必须在 release notes 中明确说明协议兼容性状态。

### 2. 更新版本与文档

至少同步以下内容：

- 根目录 [Cargo.toml](Cargo.toml) 中 `[workspace.package].version`
- [CHANGELOG.md](CHANGELOG.md)
- [ROADMAP.md](ROADMAP.md) 中受影响的里程碑状态
- [README.md](README.md) 中的状态说明
- 仓库内的 release notes draft（建议放在 `release-notes/vX.Y.Z.md`）

如果涉及协议或验证资产，还要同步：

- [spec/format-v1.md](spec/format-v1.md)
- [spec/verification-and-benchmark.md](spec/verification-and-benchmark.md)
- `tests/fixtures/` 下对应样例
- [tests/golden/README.md](tests/golden/README.md) 与每个 golden fixture README 的覆盖说明

### 3. 执行质量闸口

发版前至少执行以下命令：

```bash
cargo fmt --all --check
cargo test --workspace
bash tests/scripts/run_protocol_smoke.sh
bash tests/scripts/run_streaming_smoke.sh
bash tests/scripts/run_safety_smoke.sh
bash tests/scripts/run_roundtrip_smoke.sh
cargo run -p sfa-bench --bin tar_vs_sfa -- --dry-run --output benches/results/latest.json
```

以上命令构成当前仓库的 authoritative release checklist。

如果本次发版修改了 benchmark 逻辑、默认 benchmark 数据集、planner / pipeline 参数、codec 集成或 benchmark 支持环境，应额外刷新 committed benchmark baseline，并在 release notes 中说明：

```bash
CARGO_HOME=/tmp/cargo-home cargo build --release -p sfa-cli
./benches/scripts/run_tar_vs_sfa.sh \
  --execute \
  --sfa-bin target/release/sfa-cli \
  --output benches/results/baseline-v0.1.0.json
```

刷新后应确认 `benches/results/baseline-v0.1.0.json` 已提交，且 `cargo test -p sfa-bench` 仍能读取并校验该结果资产。
如果本次发版不涉及 benchmark 行为、默认数据集或结果 schema 的变化，则 benchmark dry-run 仍是必跑项，但不要求额外刷新 committed baseline。
如果当前发版依赖 benchmark 作为性能证据，还应确认 committed baseline 中：

- `environment.resource_sampler` 与支持环境说明一致
- 每条执行记录都包含命令 wall-time
- `sfa` 记录包含阶段级 `sfa_stats`
- 支持环境下的记录包含 `user_cpu_ms`、`system_cpu_ms` 和 `max_rss_kib`
- unpack `sfa_stats` 使用 `header`、`manifest`、`frame_read`、`decode`、`scatter`、`restore_finalize` 字段，而不是旧的 `decode_and_scatter`
- 如果本次发版涉及 unpack pipeline 或 `--threads` 语义，应确认结果资产保留了有效线程数，并在说明中指出 unpack split phases 属于并行诊断窗口、不是可直接求和的总账

### 4. 整理 release notes

每次发版的 release notes 至少应覆盖：

- 本次版本摘要
- 主要新增能力或修复
- 是否涉及协议变化
- 验证结果摘要
- 已知限制或后续工作

推荐结构：

```text
Highlights
Compatibility
Verification
Known gaps
```

建议先在仓库内起草 `release-notes/vX.Y.Z.md`，确认版本摘要、验证结果和已知缺口之后，再复制到 GitHub Release。

### 5. 创建 tag

在主分支内容确认无误后创建版本 tag：

```bash
git tag -a vX.Y.Z -m "sfa vX.Y.Z"
git push origin vX.Y.Z
```

如果本次发版需要同时推送主分支：

```bash
git push origin main
git push origin vX.Y.Z
```

推送 `vX.Y.Z` 后，release workflow 会：

- 在对应 tag 上重跑 authoritative release checklist
- 读取 `release-notes/vX.Y.Z.md`
- 编译 Linux x86_64、macOS x86_64、macOS arm64 CLI 二进制压缩包
- 创建或更新 GitHub Release，并附加生成的二进制和校验文件

如果 release workflow 尚未可用，或者需要为一个已经存在的 tag 补发 Release，可以手动触发 `.github/workflows/release.yml` 的 `workflow_dispatch`，并传入目标 tag。

### 6. 确认 GitHub Release

正常情况下，tag push 对应的 release workflow 会自动以 `vX.Y.Z` tag 创建 Release，并附上 `release-notes/vX.Y.Z.md` 的内容。
同一次 workflow 还会附加：

- `sfa-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz`
- `sfa-vX.Y.Z-x86_64-apple-darwin.tar.gz`
- `sfa-vX.Y.Z-aarch64-apple-darwin.tar.gz`
- 对应的 `.sha256` 校验文件

如果 workflow 不可用，或者需要手动回填，可以执行：

```bash
gh release create vX.Y.Z --verify-tag --title "sfa vX.Y.Z" --notes-file release-notes/vX.Y.Z.md
```

建议在 Release 中明确：

- 当前协议是否已冻结
- 推荐使用场景
- 与上一版本相比的兼容性变化
- 对应的路线图阶段

### 7. 发版后收尾

发版完成后，至少检查以下事项：

- [ROADMAP.md](ROADMAP.md) 与 [README.md](README.md) 是否需要更新状态
- [CHANGELOG.md](CHANGELOG.md) 是否已开启下一轮 `Unreleased`
- 若协议有演进，是否需要立新的 OpenSpec change

## 协议相关发版的额外要求

以下类型的改动必须视为协议相关发版：

- header、manifest、frame、trailer 结构变化
- codec / integrity 字段含义变化
- 解码器容错或校验语义变化
- 影响 golden fixture 的任何改动

这类发版必须额外满足：

- `spec/format-v1.md` 与实现一致
- golden fixture、dump 输出与 fixture README 覆盖说明已更新
- release notes 中明确兼容性影响

## 最小发版清单

发版前建议逐项确认：

- [ ] 版本号已更新
- [ ] `CHANGELOG.md` 已更新
- [ ] release notes draft 已整理并与仓库版本一致
- [ ] `git status --short` 为空
- [ ] authoritative release checklist 通过
- [ ] 如涉及协议，spec 与 fixtures 已同步
- [ ] Git tag 已创建并推送
- [ ] GitHub Release 已由 workflow 创建或手动回填
- [ ] Linux / macOS release assets 已上传
