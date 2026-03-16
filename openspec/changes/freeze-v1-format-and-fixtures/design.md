## Context

SFA v1 的 pack / unpack 主链路已经可运行，但协议冻结所需的仓库资产还没有闭环。当前 [spec/format-v1.md](/Users/bytedance/github/sfa/spec/format-v1.md) 仍是 placeholder，[tests/fixtures/golden/README.md](/Users/bytedance/github/sfa/tests/fixtures/golden/README.md) 下没有真正的 canonical 样例，而 [tests/scripts/run_protocol_smoke.sh](/Users/bytedance/github/sfa/tests/scripts/run_protocol_smoke.sh) 也还没有消费 golden fixture。

这意味着实现虽然存在，但仓库还缺少以下能力：
- 一个可被引用的权威协议文本，用来定义 v1 的兼容性边界。
- 一组与协议文本绑定的 canonical archive 资产，用来判断未来改动是否漂移 wire format。
- 一份协议评审留痕，用来说明冻结依据、冻结范围和明确延后的事项。

这个 change 是跨文档、测试资产和脚本的收口工作。它不引入新的运行时能力，但会把现有运行时实现固定到一个可验证、可审阅、可回归的 v1 协议基线上。

## Goals / Non-Goals

**Goals:**
- 将 `spec/format-v1.md` 收敛为 SFA v1 的权威协议定义，并与现有技术方案文档形成清晰边界。
- 定义第一批 canonical golden fixture 的目录结构、生成流程和验证口径。
- 为协议冻结补充仓库内可追溯的评审记录，明确冻结结论和后续待办。
- 让现有 protocol smoke / 回归入口至少能检查 golden assets 存在、可解析、与冻结文本一致。

**Non-Goals:**
- 不在本 change 中引入真实 benchmark dataset 或正式 `tar + same codec` 性能基线。
- 不在本 change 中扩展新的 wire format 字段、CLI 参数或恢复语义。
- 不要求一次性补齐所有 corruption、streaming、safety 资产，只要求定义与协议冻结直接相关的首批 canonical assets。
- 不把技术方案文档整体搬运为规范；方案文档继续承担背景和设计推演，冻结 spec 只保留规范性内容。

## Decisions

### 1. 以 `spec/format-v1.md` 作为唯一权威协议入口

冻结后的 v1 协议只保留一个权威入口：`spec/format-v1.md`。现有 `sfa-tech-solution/04-format-v1.md` 继续作为设计背景文档存在，但不再承担“最终协议定义”职责。

这样做的理由是协议冻结需要一个稳定、短链路、适合 code review 和 golden 校验引用的文本入口。如果继续让技术方案文档和 spec 文档并列承担规范职责，未来实现偏差时会出现“以哪份文档为准”的歧义。

备选方案：
- 继续把技术方案文档视为事实来源，再让 `spec/format-v1.md` 做索引。这样成本低，但无法真正形成冻结协议。
- 将完整方案直接迁入 `spec/format-v1.md`。这样最省心，但会把背景讨论、候选方案和规范性要求混在一起，降低可读性。

### 2. canonical golden fixture 采用“输入树 + archive + dump + 摘要”四件套

首批 golden fixture 将按固定目录组织，每个 fixture 至少包含：
- 生成 fixture 的输入树说明或原始输入目录；
- canonical `.sfa` archive；
- 由现有 dump 工具导出的 manifest / header 摘要；
- 与协议回归对应的统计摘要或校验说明。

建议的结构是：

```text
tests/fixtures/golden/<fixture-name>/
├── input/
├── archive.sfa
├── manifest.json
├── stats.json
└── README.md
```

这样设计的重点不是追求覆盖面最大，而是保证每个 fixture 都能回答三个问题：它是由什么输入生成的、它冻结了什么协议特征、后续实现应该怎样验证它。

备选方案：
- 只提交 archive 文件。这样提交量最小，但 review 时很难直接看出冻结了哪些协议信息。
- 只提交 dump JSON，不提交 archive。这样便于 diff，但失去真实 wire format 回归价值。

### 3. fixture 生成沿用现有打包链路，但输出必须可重建且命名稳定

fixture 生成不会引入另一套“特殊协议写出器”，而是继续通过现有 `sfa pack` 和 `dump_archive_fixture` 工具链生成。这能确保冻结资产来自真实实现路径，而不是测试专用分支逻辑。

为避免 golden 资产不可维护，生成流程必须满足：
- 输入目录内容固定且可提交到仓库；
- 归档参数固定并写入 fixture README 或旁路元数据；
- dump 结果和统计摘要采用稳定字段、稳定排序、稳定命名；
- 未来若实现 bug 修复导致 archive 变化，必须伴随 spec 或评审结论更新，而不是直接覆盖。

备选方案：
- 在测试中动态生成 golden 资产。这样仓库更轻，但失去 review 历史和冻结留痕。
- 手写 fixture dump。这样最可控，但很容易与真实 writer 行为脱节。

### 4. 协议评审记录与冻结资产一起提交，而不是放在外部流程里

本 change 会要求在仓库中新增一份协议评审记录，记录至少以下内容：
- 本次冻结引用的 spec 版本与 fixture 集合；
- 明确纳入冻结的协议语义；
- 明确延后的事项，例如 benchmark 正式基线、扩展 Unix 元数据等；
- 参与评审的结论与后续触发重新开 change 的条件。

这样做是为了让 `M0` 的关闭条件可以在仓库内自洽成立，而不依赖外部会议记录、聊天记录或口头结论。

备选方案：
- 只在 PR 描述中留下评审意见。这样操作轻，但后续难以追溯。
- 把评审记录写进 roadmap。这样入口集中，但会混淆路线图和协议冻结证据。

### 5. protocol smoke 先接“存在性 + 可解析性 + 基本一致性”，不提前承诺全量 byte-for-byte CI

当前 [tests/scripts/run_protocol_smoke.sh](/Users/bytedance/github/sfa/tests/scripts/run_protocol_smoke.sh) 只运行 `cargo test -p sfa-core`。本 change 会把它提升到最小可用的协议冻结守门人，但只要求：
- golden fixture 文件完整存在；
- archive 可被当前 reader 成功读取；
- dump 结果与已提交摘要一致；
- 如脚本中有明确定义的统计字段，也需要一致。

这里刻意不把“所有平台上都做 byte-for-byte 重打包比对”纳入冻结 change。因为当前协议冻结的关键是读者兼容性和资产可追溯性，不是立即把所有 writer 实现细节都变成跨环境强约束。

备选方案：
- 仅检查文件存在。这样门槛太低，不能证明资产真能被消费。
- 立即要求完全重打包复现。这样约束过强，容易把非协议层的实现抖动误判为协议破坏。

## Risks / Trade-offs

- [冻结 spec 与现有实现存在偏差] → 先用 golden 资产暴露偏差，再在后续 apply 阶段决定修实现还是修 spec，不允许带着歧义进入 benchmark 阶段。
- [fixture 资产过少导致冻结结论过弱] → 首批 fixture 明确覆盖至少一种 canonical 小文件聚合路径，并在评审记录中写清未覆盖项。
- [fixture 资产过多导致 change 过重] → 第一批只提交支撑 M0 的最小集合，把更大覆盖面留给后续验证 change。
- [协议评审记录流于形式] → 评审记录必须引用具体 spec 路径、fixture 名称和冻结日期，而不是只写抽象结论。
- [smoke check 约束过强阻碍正常迭代] → 冻结 change 只守住 reader-compatible 的协议资产，不把性能和实现优化一并绑定。

## Migration Plan

这是一次仓库资产冻结，不涉及线上迁移或已发布归档的兼容负担。落地顺序应为：

1. 将 `spec/format-v1.md` 从 placeholder 收敛为规范性文本。
2. 生成并提交首批 canonical golden fixture 及配套 dump/summary。
3. 提交协议评审记录并在相关入口文档中标注冻结状态。
4. 更新 protocol smoke，使其消费已提交的 golden assets。

如果在落地过程中发现现有实现与冻结文本存在重大冲突，应在进入 benchmark change 之前先关闭该差异；不允许通过跳过 fixture 校验来“带病冻结”。

## Open Questions

- 首批 golden fixture 是只覆盖一个 canonical 小文件聚合案例，还是同时加入大文件切分案例；这个取舍会影响本 change 的体量。
- `stats.json` 是直接复用 CLI summary 字段，还是定义一个更稳定、与 CLI 呈现解耦的 fixture 摘要格式。
- 协议评审记录最终放在 `spec/` 还是 `tests/fixtures/golden/` 邻近目录，更利于后续维护和发现。
