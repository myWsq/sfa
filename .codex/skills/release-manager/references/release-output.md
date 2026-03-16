# 发版输出结构

除非仓库已经有更强的既有约定，否则优先使用以下结构。

## 发版摘要

```text
Scope
Version proposal
Compatibility
Verification status
Blocked items
Recommended next actions
```

## CHANGELOG 条目

```markdown
## [Unreleased]

### Added

- ...

### Changed

- ...

### Fixed

- ...
```

只保留真正有内容的分节。

## GitHub Release Notes

```text
Highlights
Compatibility
Verification
Known gaps
```

说明：

- `Highlights`：用户可感知的新能力、修复和重要仓库变化
- `Compatibility`：协议、CLI、API、迁移或升级相关说明
- `Verification`：已执行的测试、smoke checks、benchmark 或其他发版闸口
- `Known gaps`：明确延期的工作、占位项或后续待办

## Recommended Next Actions

```text
Release status: ready | almost ready | blocked
1. ...
2. ...
3. ...
```

说明：

- 这一段要偏操作，不要写成背景说明
- 优先给出最小正确下一步，不要展开成长计划
- 如果下一步依赖终端命令，要给出精确命令
- 如果需要手动去 GitHub UI 操作，要明确写出来
- 如果 `git status --short` 非空，状态必须是 `blocked`

## 版本号判断规则

- `patch`：修复、文档、重构或内部构建变更，不应改变兼容性预期
- `minor`：新增能力，并保持向后兼容
- `major`：任何破坏 API、CLI、协议或兼容性的变更

如果项目还没到 `1.0.0`，也要用文字解释兼容性，不要假设只靠 SemVer 就足够。
