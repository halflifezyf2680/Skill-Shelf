---
name: writer-studio
description: "面向长篇小说与复杂叙事项目的创作工作室 skill。既适用于从零开始建立树状 SSOT / 结构化 canon，也适用于在已有 truth surface 上持续运行严肃写作闭环。用于管理头脑风暴、候选方案收敛、SSOT 构建、章节执行包准备、在角色与风格约束下主写章节或场景、进行连续性与逻辑审校、提取 canon delta 并同步回项目文件，并为项目建立统一的结构化数据维护协议：YAML 作为作者维护面，数据库作为查询与分析面，两者共享同一套字段、主表/子表语义和稳定 id，由项目脚本负责扫描、校验与落库。"
---

# Writer Studio

把这个 skill 当作“长篇叙事项目的创作操作层”来用。

它既负责帮助项目建立 truth surface，也负责在 truth surface 建成后维持写作闭环。  
它不替代正文文件。  
它负责在这些工作面之间路由和控场：

- brainstorm 面
- ssot / canon 构建面
- canon 面
- packet 面
- draft 面
- audit 面
- sync 面
- 结构化数据维护面

核心闭环：

```text
数据协议 -> truth surface -> packet -> draft -> audit -> sync
```

## 什么时候用

当项目需要正式建立或维护长篇创作闭环时，就该用这个 skill。

典型触发场景：

- 项目还没有正式 SSOT，需要把 brainstorm 收敛成树状 truth surface
- 项目已经有 AST、世界观树、世界观索引、章节规划或类似 truth surface，需要继续沿它运行
- 你想让 AI 主写，但又不想让它跑出角色、设定或时间线
- 你有多个候选设定、情节、命名方案，需要收敛成 canon
- 你要写一章，但希望先得到正式的章节执行包
- 你已经有正文草稿，需要做逻辑、关系、POV、物件流转或知情链审校
- 你要为当前项目建立统一的数据结构维护协议，并生成 YAML 扫描 / 校验 / 落库脚本

不要把它用于完全随手的、一次性的小段落生成任务。

## 路由逻辑

从最小必要协议开始，不要一上来把整套流程全跑一遍。

- 如果当前工作还是探索期，先看 [references/brainstorm-to-canon.md](references/brainstorm-to-canon.md)
- 如果项目还没有正式 truth surface，就先看 [references/workflow.md](references/workflow.md) 和 [references/db-protocol.md](references/db-protocol.md)，先定维护协议，再建立候选与 canon 的准入机制
- 如果你需要一个像命令系统一样稳定的动作层，先看 [references/command-system.md](references/command-system.md)
- 如果你不确定项目现在处于哪个阶段，先看 [references/stage-model.md](references/stage-model.md)
- 如果你要先判断下一步做什么，先看 [references/status-gap-detect.md](references/status-gap-detect.md)
- 如果你准备正式开写，先看 [references/chapter-packet-spec.md](references/chapter-packet-spec.md)
- 如果草稿已经存在，先看 [references/draft-audit-spec.md](references/draft-audit-spec.md)
- 如果正文已经改变了项目事实面，继续看 [references/canon-sync-spec.md](references/canon-sync-spec.md)
- 如果项目需要结构化查询与一致性分析，使用 [references/db-protocol.md](references/db-protocol.md) 先定义项目的数据结构维护协议，再生成本项目专属的 YAML 扫描与落库脚本

## 命令分发总表

先判断你现在手上的东西，再选命令。不要按感觉乱跳步骤。

| 你现在的状态 | 直接用什么 | 固定产物 | 不该用什么 |
|---|---|---|---|
| 不知道项目现在缺什么、下一步该干嘛 | `writer-status` | 状态报告、缺口列表、下一步建议 | 不要直接跳进 `writer-build-packet` 或 `writer-sync` |
| 项目刚开始，或只有散乱文档，准备建立正式闭环 | `writer-init` | 数据结构维护协议、脚本计划、初始化结果 | 不要把它当成章节写作命令 |
| 已经有 brainstorm / candidate，要选正式方案进入 canon | `writer-canonize` | canon 决策记录、可写入 truth surface 的摘要、受影响节点清单 | 不要拿它直接审正文 |
| 准备写某一章，想先锁定 POV、时间锚点、beats 和禁止动作 | `writer-build-packet` | 正式 chapter packet | 不要在 truth surface 还没稳定时硬建 packet |
| 正文已经写出，要判断逻辑、连续性、POV、知情状态是否过关 | `writer-audit` | audit report、verdict、必改项 | 不要在没 draft 的时候调用 |
| audit 已通过，要把新事实纳入 truth surface 并重编译查询面 | `writer-sync` | canon delta、truth surface 更新、查询面重编译结果、sync 摘要 | 不要跳过 `writer-audit` 直接同步 |

### 最短判断法

- 你在问“现在该做什么”时，用 `writer-status`
- 你在定“这个项目的 truth surface 和协议怎么建”时，用 `writer-init`
- 你在定“多个候选里哪个算正式 canon”时，用 `writer-canonize`
- 你在定“这一章怎么写、边界是什么”时，用 `writer-build-packet`
- 你在定“这版正文能不能过”时，用 `writer-audit`
- 你在定“这章产生的新事实怎么回写项目”时，用 `writer-sync`

### 两条标准闭环

项目初始化闭环：
`writer-status -> writer-init -> writer-canonize`

章节写作闭环：
`writer-status -> writer-build-packet -> writer-audit -> writer-sync`

修正闭环（audit 未通过时触发，参见 [references/fix-protocol.md](references/fix-protocol.md)）：
`writer-audit(fail/concerns) -> 修正(文风/结构/设定) -> writer-audit(重新)`

## 工作流

1. 识别当前项目或章节所处阶段。
2. 用 `writer-status` 判断缺口和下一步动作。
3. 如果项目尚未建成 truth surface，先用 `writer-init` 定义统一的数据结构维护协议。
4. 让 YAML 维护面与数据库查询面共享同一套字段、主表 / 子表语义和稳定 id。
5. 生成本项目自己的 YAML 扫描、校验、落库、查询与视图脚本。
6. 再用 `writer-canonize` 把 brainstorm 材料收敛为 SSOT / canon。
7. 把已采纳的 canon 压成章节执行包。
8. 按 packet 起草，不凭裸记忆硬写。
9. 用 `writer-audit` 对草稿做逻辑、连续性、POV 与风格审校。
10. 用 `writer-sync` 提取 canon delta，先更新 YAML / 结构化文档，再由脚本编译进数据库查询面。

## 操作规则

- `brainstorm`、`canon`、`draft`、`sync` 必须被视为不同工作面。
- 每个项目只保留一个有效 truth surface，除非你明确知道为什么要双核心。
- 如果项目已经足够结构化，就不要裸写正式章节。
- 不要把 prose 里的普通细节悄悄升级成 canon。
- 如果项目启用了结构化数据维护协议，必须遵守该项目冻结后的字段、主表 / 子表语义、稳定 id 和脚本入口，不允许即兴造表、乱改字段或临时换写法。
- YAML 是作者维护面，数据库是查询与分析面；两者必须共享同一套结构语义，不允许双规范并存。
- 本 skill 规定项目如何设计、冻结、维护结构化数据协议，但不强制所有项目共用同一套 schema。

## 默认行为

- 默认优先做路由与控场，而不是立刻生成正文。
- 默认优先使用字段、清单、状态，而不是模糊总结。
- 默认优先保持一个稳定 truth surface，再派生执行工件。
- 默认优先让 LLM 写受协议约束的 YAML / 文档块，而不是直接自由落库。
- 默认优先让项目脚本承担扫描、校验、落库和查询，而不是让 LLM 在日常写作中直接发明数据库操作。
- 默认优先使用项目协议文件，而不是依赖 session 内的临时习惯。

## 参考文件

- [references/stage-model.md](references/stage-model.md)：项目或章节阶段模型
- [references/workflow.md](references/workflow.md)：完整业务闭环与项目初始化顺序
- [references/command-system.md](references/command-system.md)：命令层设计、输入输出与完成标准
- [references/status-gap-detect.md](references/status-gap-detect.md)：状态与缺口检测入口
- [references/brainstorm-to-canon.md](references/brainstorm-to-canon.md)：从发散到 canon 的收敛协议
- [references/chapter-packet-spec.md](references/chapter-packet-spec.md)：章节执行包规范
- [references/draft-audit-spec.md](references/draft-audit-spec.md)：草稿审校规范（12 条必查项，致命/非致命分级，结构化报告）
- [references/fix-protocol.md](references/fix-protocol.md)：定点修正协议（文风/结构/设定三类修正路径，轮次限制）
- [references/canon-sync-spec.md](references/canon-sync-spec.md)：canon delta 提取与同步规范
- [references/db-protocol.md](references/db-protocol.md)：项目级结构化数据维护协议

## 子 Skill

如果你不想一次调总 skill，可以直接使用这些动作级子 skill：

- `writer-status`：先判断阶段、缺口和下一步
- `writer-init`：先把项目 truth surface、协议和脚本层建起来
- `writer-canonize`：把候选内容收敛成正式 canon
- `writer-build-packet`：为某一章生成正式执行包
- `writer-audit`：审正文能不能进入 sync
- `writer-sync`：把本章新增 canon 同步回 truth surface 和查询面

## 模板

如果项目还没有自己的等价模板，可以使用这些模板：

- [assets/brainstorm-template.md](assets/brainstorm-template.md)
- [assets/canon-decision-template.md](assets/canon-decision-template.md)
- [assets/chapter-packet-template.md](assets/chapter-packet-template.md)
- [assets/draft-audit-template.md](assets/draft-audit-template.md)
- [assets/canon-delta-template.md](assets/canon-delta-template.md)
- [assets/db-protocol-template.md](assets/db-protocol-template.md)
- [assets/script-plan-template.md](assets/script-plan-template.md)
- [assets/status-report-template.md](assets/status-report-template.md)
- [assets/init-result-template.md](assets/init-result-template.md)
- [assets/sync-result-template.md](assets/sync-result-template.md)

如果需要直接参考最小可运行样板，可以看：

- [assets/truth-surface-example.yaml](assets/truth-surface-example.yaml)
- [assets/script-plan-example.md](assets/script-plan-example.md)
- [assets/chapter-sync-example.md](assets/chapter-sync-example.md)

## 输出纪律

- 控制层尽量简洁，正文层才允许展开。
- 尽量写成可核对、可审校、可同步的字段与清单。
- 如果项目已经有自己的工作流文档，优先适配，不重复造一套平行规则。
