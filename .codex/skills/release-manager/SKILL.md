---
name: release-manager
description: 准备仓库发版、更新 CHANGELOG.md、起草 GitHub Release notes、判断版本号增量并检查发版就绪状态。用于需要准备发版、汇总自上个 tag 以来的变更、维护 changelog、检查发版闸口、增加 release note 自动化，或响应“发版”“准备发版”“整理 changelog”“起草 release notes”“下一步怎么发”等请求。
---

# 发版管理

用这个 skill 为 git 仓库准备一套基于事实的发版材料。

## 简写触发

在上下文足够清晰时，把简短请求视为完整的发版准备请求。

例如：

- `发版`
- `准备发版`
- `整理 changelog`
- `起草 release notes`
- `下一步怎么发`

遇到这类简写请求时，优先从仓库上下文补全缺失默认值，并产出标准交付物。只有在确实被关键缺失信息阻塞时才追问用户。

## 快速流程

1. 在编辑任何内容前，先读取仓库当前的发版规则。
   - 检查 `README.md`、`RELEASING.md`、`CHANGELOG.md`、版本清单文件、workflow 文件，以及任何路线图或兼容性文档。
2. 检查 git 工作区状态。
   - 如果 `git status --short` 非空，把发版判定为阻塞，直到工作区干净为止。
3. 用 `scripts/collect_release_facts.js` 收集基于 git 的事实变更数据。
4. 判断本次发版范围。
   - `patch`：修复问题或内部整理，不改变预期接口或协议行为
   - `minor`：新增能力，但保持向后兼容
   - `major`：存在破坏性接口、协议或兼容性变更
5. 更新仓库内归属的发版材料。
   - 常见文件包括 `CHANGELOG.md`、版本清单、路线图状态和发版检查文档。
6. 基于已验证事实起草 GitHub Release notes。
7. 运行仓库定义的发版闸口。
8. 如果用户要真正切版，准备精确的 `git tag` 和发布命令；环境允许时再执行。

## 必须遵守的行为

- 优先遵循仓库已有的发版规则，不要临时发明新流程。
- 在用户确认前，把生成的 release notes 视为草稿。
- 不要虚构 PR 编号、tag、issue 链接、兼容性保证、benchmark 结果或协议结论。
- 对兼容性敏感变更要明确指出。
- 当 `git status --short` 非空时，绝不能把发版标记为 `ready`。
- 如果工作区不干净，必须把发版状态标记为 `blocked`，并把清理、提交或处理这些无关变更作为第一条下一步建议。
- 如果仓库已经在使用 GitHub 自动 release notes、`release-please`、`Release Drafter`、`Changesets` 或 `git-cliff`，优先扩展它，而不是随手另起一套。
- 如果环境阻止 `git push`、创建 tag 或 GitHub API 操作，停在“文件已准备好 + 精确后续命令”这一层。

## 附带资源

- `scripts/collect_release_facts.js`
  以确定性方式汇总最新 tag、提交范围、变更文件和受影响的顶层区域。
- `references/release-output.md`
  定义 changelog、GitHub Release notes 和发版就绪摘要的输出结构。

## SFA 仓库约定

在 SFA 仓库内使用这个 skill 时，要与以下文档保持一致：

- `RELEASING.md`：发版检查清单和质量闸口
- `CHANGELOG.md`：仓库变更记录
- `ROADMAP.md`：里程碑状态更新
- `spec/format-v1.md` 和 fixtures：涉及协议变更时必须同步检查

对 SFA 来说，在 `M0` 关闭前，协议兼容性都不应视为已冻结。只要相关，release notes 里就必须明确写出这一点。

## 收集事实

运行：

```bash
node .codex/skills/release-manager/scripts/collect_release_facts.js --repo .
```

常用参数：

```bash
node .codex/skills/release-manager/scripts/collect_release_facts.js --repo . --from-ref v0.1.0
node .codex/skills/release-manager/scripts/collect_release_facts.js --repo . --to-ref HEAD
node .codex/skills/release-manager/scripts/collect_release_facts.js --repo . --limit 100
```

把脚本输出作为以下内容的事实基础：

- changelog 条目
- release notes 草稿
- 版本号建议理由
- 发版检查摘要

## 交付物

除非用户明确只要其中一项，否则默认产出：

1. 发版摘要
2. 版本号建议及理由
3. `CHANGELOG.md` 草稿或更新
4. GitHub Release notes 草稿
5. 发版就绪结论以及阻塞项
6. 一段“下一步建议”，给出最小正确行动

## 下一步建议规则

始终以 `Recommended next actions` 结尾。

这一段必须：

- 说明当前状态是 `ready`、`almost ready` 还是 `blocked`
- 按优先级列出接下来的 1 到 3 个具体动作
- 如果下一步是命令，给出精确命令
- 区分哪些下一步可以由 Codex 继续完成，哪些需要用户批准或手动操作

如果仓库还没准备好发版，第一条下一步建议必须是影响最大的阻塞项。

输出结构遵循 `references/release-output.md`。
