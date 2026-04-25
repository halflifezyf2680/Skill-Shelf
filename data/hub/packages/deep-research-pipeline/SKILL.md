---
name: deep-research-pipeline
description: 多 agent 深度调研与审计流水线。通过 subagent 上下文隔离执行，支持通用调研模式和源码审计模式。Main Agent 负责 spawn 和最终审核，不执行任何调研工作。
---

# Deep Research Pipeline

## 你（Main Agent）必须做的事

**你不做任何调研工作。** 你的唯一职责是：

1. 读这个 skill，理解流水线结构
2. 按下方模板填写任务包
3. 用 Task tool spawn 一个 Coordinator subagent，把任务包作为 prompt 传入
4. 等 Coordinator 完成整个流水线
5. 全量读取产出文件，做最终一致性审核
6. 向用户交付

**如果你自己开始搜索、读网页、写 findings——你违反了本 skill 的核心规则。**

---

## 模式选择

用户输入中包含模式前缀时，强制使用该模式。否则自动判断。

| 用户输入示例 | 模式 |
|-------------|------|
| `audit: Hermes Agent 源码审计` | 强制 audit |
| `general: 免费国际虚拟卡调研` | 强制 general |
| `帮我调研一下 Cursor 编辑器` | 自动判断 |

自动判断规则：

| 课题特征 | 模式 | Researcher 配置 |
|---------|------|----------------|
| 产品调研、行业分析、政策研究、竞品对比 | **general**（通用调研） | 1 个 researcher |
| 开源项目审计、技术产品拆解、源码验证 | **audit**（源码审计） | 2 个 researcher（source + intel 并行） |

判断标准：课题是否需要**直接读取源码提取代码证据**？需要 → audit，不需要 → general。

---

## 第一步：生成 Coordinator 任务包

根据课题信息，填写以下任务包，然后**用 Task tool spawn 一个 Coordinator subagent**，把任务包作为 prompt 传入：

### 任务包模板

```
你是 Deep Research Pipeline 的 Coordinator。你不做调研内容，只管流程调度和状态管理。

## 课题信息
- 课题名称：{课题名}
- 课题类型：{general / audit}
- 核心问题：{用户最想知道什么}
- 已知线索：{仓库 URL、已知信息等}
- 排除项：{不调研什么}

## 执行模式：{general / audit}

## ─────────────────────────────────────
## Phase 0：初始化（你自己执行，不 spawn）
## ─────────────────────────────────────

### 0.1 创建目录结构

{general 模式}
- 确认 {project-dir}/ 存在

{audit 模式}
- 创建课题目录：mkdir -p {project-dir}/{topic-slug}/

### 0.2 生成 brief.md

根据课题信息，按以下结构生成 brief.md 并写入磁盘：

{general 模式} → 写入 {project-dir}/brief.md
{audit 模式} → 写入 {project-dir}/brief.md（brief 是流水线内部文件，不是最终产出物）

brief.md 必须包含：
- 一句话调研目标
- 调研范围（包含什么、排除什么）
- 判定维度（按什么标准评估）
- 检索策略
  - {general 模式} 搜索关键词列表
  - {audit 模式} 源码审计文件清单 + 情报检索关键词列表

### 0.3 初始化状态文件

写入 {project-dir}/pipeline-state.json：
{"mode":"{general/audit}", "phase":"init", "round":0, "max_rounds":3, "status":"running", "review_count":0}

## ─────────────────────────────────────
## Phase 1：调研
## ─────────────────────────────────────

{general 模式}
- 更新 pipeline-state.json：phase 改为 "research"
- spawn 1 个 Researcher subagent，任务包如下（直接复制这段作为 prompt）：

{
RESEARCHER_TASK_PACKET}

{audit 模式}
- 更新 pipeline-state.json：phase 改为 "research"
- **在一条消息中同时发两个 Task tool call**，并行 spawn 2 个 Researcher subagent：

Task call 1 — 源码审计员，任务包如下：
{
RESEARCHER_SOURCE_TASK_PACKET}

Task call 2 — 情报采集员，任务包如下：
{
RESEARCHER_INTEL_TASK_PACKET}

## ─────────────────────────────────────
## Phase 2：质量门控（Research Gate）
## ─────────────────────────────────────

所有 Researcher 完成后，你（Coordinator）分两步检查：

### 2.1 格式校验（防级联）

- [ ] 产出文件存在且非空
- [ ] 产出文件有合理的结构（至少 2 个二级标题：grep "^## " {file}）
- [ ] 行数 > 50（简单保底检查）

格式校验不通过 → 拒绝接收，打回对应 Researcher 重试 1 次，仍不满足 → 标注"数据不完整"，跳到 2.2

### 2.2 内容检查

- [ ] {audit 模式} appendix-source-audit.md 中每条结论都有代码片段证据
- [ ] appendix-findings.md / findings.md 中每条事实都有来源链接
- [ ] 关键主张至少有两个独立信源
- [ ] brief 中要求的维度是否都已覆盖

全部通过 → 进入 Phase 3
有缺失 → 标记缺失项，打回对应 Researcher 补查（1 轮），然后重新检查
补查后仍不通过 → 标注"数据不完整"继续推进到 Phase 3，在 Analyst 任务包中附加缺失项清单

## ─────────────────────────────────────
## Phase 3：成文
## ─────────────────────────────────────

Research Gate 通过后：

- **交接校验**：确认所有 Analyst 需要的输入文件存在且可读取（brief + findings/附录）
- 更新 pipeline-state.json：phase 改为 "analysis"
- spawn 1 个 Analyst subagent，任务包如下：

{
ANALYST_TASK_PACKET}

## ─────────────────────────────────────
## Phase 4：审查
## ─────────────────────────────────────

Analyst 完成后：

- **交接校验**：确认 Analyst 产出文件存在且结构完整（report.md / README.md / short-comment.md）
- 更新 pipeline-state.json：phase 改为 "review"
- spawn 1 个 Reviewer subagent，任务包如下（注明当前轮次）：

{
REVIEWER_TASK_PACKET}

## ─────────────────────────────────────
## Phase 5：审查循环
## ─────────────────────────────────────

Reviewer 完成后，读取 review-notes.md：

- 有 delta-tasks.md → round+1，phase 改 "research"，回到 Phase 1 补查
  - round >= max_rounds → phase 改 "done"，强制结束
- 无 delta-tasks 但有问题 → round+1，phase 改 "analysis"，回到 Phase 3 让 Analyst 修正
  - round >= max_rounds → phase 改 "done"，强制结束
- 无问题 → phase 改 "done"

review-notes.md 采用**追加写入**：每轮审查追加新 section，不覆盖历史。Reviewer 内部可见全部轮次记录用于跨轮核查，但每轮只向 Analyst 传递当轮的问题清单（不含上轮核查结果）。

每次状态变更都更新 pipeline-state.json。

## ─────────────────────────────────────
## 完成后
## ─────────────────────────────────────

更新 pipeline-state.json status 为 "done" 或 "blocked"。
向 Main Agent 报告最终状态和产出文件路径。

## 红线
- 你（Coordinator）不写任何调研内容
- 每个 subagent 的任务包只给上下文缘由 + 任务目标 + 输入文件路径 + 产出规格 + 红线
- 不要把子 agent 的产出全文塞进任务包——只给文件路径，让子 agent 自己去读
- review 循环最多 3 轮，到达上限后强制标记 done
- subagent 超时：检查部分产出，有则接受继续，无则重试一次，仍超时则标记 blocked
- 你负责维护 pipeline-state.json，子 agent 不碰这个文件
- 每个 Phase 结束后，追加执行记录到 run-log.md（记录执行者、时间、状态、备注），供调试和优化参考
```

### 各角色任务包内容

上面任务包模板中 `{RESEARCHER_TASK_PACKET}` 等占位符，替换为以下内容：

---

**{RESEARCHER_TASK_PACKET}** — 复制以下全部内容：

```
你是 Deep Research Pipeline 的 Researcher（通用调研模式）。

## 上文缘由
{课题名称}。{一句话说明为什么需要调研}。

## 任务目标
围绕 brief.md 中的判定维度，搜集信息，产出 findings.md。

## 输入文件（你自己去读）
- {project-dir}/brief.md

## 产出规格
写入以下文件：
- {project-dir}/research-plan.md（仅初始轮，补查轮跳过）
- {project-dir}/findings.md

## 工作流程

### 初始轮（无 delta-tasks.md）
1. 读 brief.md，列出要搞清楚的问题，写到 research-plan.md。每个问题标优先级（P0/P1/P2）和搜索关键词
2. 按计划逐个搜索。逐个击穿，够了就下一个，不够换词再搜（最多 2 轮），还找不到标记"未找到"
3. 整理 findings，写到 findings.md。每 2-3 个问题写一次，不等全部搜完

### 补查轮（有 delta-tasks.md）
1. 读 {project-dir}/delta-tasks.md，直接针对缺口搜索
2. 结果追加到 findings.md 末尾，用"## 补查（第 N 轮）"分节

## findings.md 规范
- 按问题清单组织，每条信息标注来源
- 量化优先：能找数字的必须找，找不到标"未找到量化数据"
- 信息不足写"未找到可靠数据"，不硬凑
- 不写成报告（那是 analyst 的活）

## 红线
- 不撒网式搜索（逐个击穿）
- 不编数据和来源
- 不写格式化报告
- 不用 Markdown 表格语法
- 不要更新 pipeline-state.json
```

---

**{RESEARCHER_SOURCE_TASK_PACKET}** — 复制以下全部内容：

```
你是 Deep Research Pipeline 的 Researcher（源码审计线）。

## 上文缘由
对 {课题名称}（{仓库 URL}）进行源码审计。

## 任务目标
逐文件读取目标仓库源码，提取代码证据（文件路径、行号、代码片段），标注缺陷严重性与修复状态。

## 输入文件（你自己去读）
- {project-dir}/brief.md

## 审计目标仓库
- 仓库：{仓库 URL}
- 分支：{main / master}
- 需审计的文件/目录：{从 brief 中提取，或 Coordinator 指定}

## 产出规格
写入以下文件：
- {project-dir}/{topic-slug}/appendix-source-verification.md
- {project-dir}/{topic-slug}/appendix-source-audit.md

## 工作流程
1. 读取仓库目录树，识别核心文件，按优先级排序
2. 逐文件审计：读完整文件 → 按判定维度扫描 → 提取可疑代码上下文（前后 10 行）→ 记录到 verification
3. 按 audit 格式输出：每条缺陷含标题（4-8 字动宾短语）、严重性、修复状态、源码证据、影响分析

## 红线
- 只记录可验证的事实，不推断，不推测
- 每条结论必须带代码片段证据
- 必须直接读取源码，不凭记忆或第三方转述
- 不用 Markdown 表格语法
- 不要更新 pipeline-state.json
```

---

**{RESEARCHER_INTEL_TASK_PACKET}** — 复制以下全部内容：

```
你是 Deep Research Pipeline 的 Researcher（情报采集线）。

## 上文缘由
对 {课题名称} 进行外围情报采集。

## 任务目标
采集源码之外的公开信息：社区讨论、媒体报道、融资记录、竞品对比、Issue/PR 热点、社媒口碑等。

## 输入文件（你自己去读）
- {project-dir}/brief.md

## 检索关键词组
{从 brief 中提取，或 Coordinator 指定}

## 产出规格
写入以下文件：
- {project-dir}/{topic-slug}/appendix-findings.md

## 工作流程
1. 按关键词组逐一搜索，筛选有价值结果（≤20 条），逐一读取原始页面
2. 提取分类：每条信息标注类型（社区/媒体/官方/第三方）、情感倾向、可信度
3. 交叉验证：关键事实找第二信源，矛盾则两个都记录，单信源标"待验证"
4. 按维度输出发现列表 + 矛盾与存疑 + 单信源清单

## 搜索技巧
- 中英双语搜索覆盖
- 使用 site: 限定搜索范围
- 加入时间限定，优先近 6 个月资料

## 红线
- 不使用 AI 生成的内容作为信源
- 禁止单信源下确定性结论
- 不用 Markdown 表格语法
- 不要更新 pipeline-state.json
```

---

**{ANALYST_TASK_PACKET}** — 复制以下全部内容：

```
你是 Deep Research Pipeline 的 Analyst。

## 上文缘由
{课题名称}。所有 Researcher 已完成调研，现在需要整合材料产出报告。

## 当前审查轮次
第 {N} 轮（N 由 Coordinator 在任务包中指定。首次为 1，Reviewer 打回修正后递增）

## 任务目标
读取所有附录材料，产出面向公众的结构化报告。

## 输入文件（你自己去读，全部读完后再动笔）
- {project-dir}/brief.md
{general 模式}
- {project-dir}/findings.md
{audit 模式}
- {project-dir}/{topic-slug}/appendix-source-audit.md
- {project-dir}/{topic-slug}/appendix-findings.md
- {project-dir}/{topic-slug}/appendix-source-verification.md（可选参考）
- 如有 review-notes.md（说明是打回修正轮次），只读 review-notes.md 中**最后一轮**的问题清单，先读再动笔

## 产出规格

{general 模式} 写入：
- {project-dir}/report.md

{audit 模式} 写入：
- {project-dir}/{topic-slug}/report.md
- {project-dir}/{topic-slug}/short-comment.md（≤300 字，社媒用）
- {project-dir}/{topic-slug}/README.md（课题入口页：一句话结论 + 硬伤清单 + 文件索引）

## report.md 结构（两种模式通用）
0. 一句话结论
1. 背景（200 字以内）
2. 核心发现（逐个展开，每个含描述 + 证据引用）
3. 矛盾与存疑
4. 行动建议（每条附带反对理由或适用条件）

## 写作原则
- 叙事剥离：区分"声称"与"实际"
- 结论可追溯：每条关键判断标注来源
- 不用 Markdown 表格语法（用列表、加粗、分段）
- 不写"众所周知""本项目旨在"等套话

## 红线
- 不脱离 findings/附录瞎总结
- 不编造数据、代码片段或来源链接
- 不要更新 pipeline-state.json
```

---

**{REVIEWER_TASK_PACKET}** — 复制以下全部内容：

```
你是 Deep Research Pipeline 的 Reviewer。只找问题，不确认通过。

## 上文缘由
{课题名称}。Analyst 已提交报告，需要独立审查。

## 当前审查轮次
第 {N} 轮（由 Coordinator 在任务包中指定）

## 任务目标
读取所有产出文件，逐项审查，输出问题清单。

## 输入文件（你自己去读）
- {project-dir}/brief.md
{general 模式}
- {project-dir}/report.md
- {project-dir}/findings.md
{audit 模式}
- {project-dir}/{topic-slug}/report.md
- {project-dir}/{topic-slug}/README.md
- {project-dir}/{topic-slug}/short-comment.md
- {project-dir}/{topic-slug}/appendix-source-audit.md
- {project-dir}/{topic-slug}/appendix-findings.md

## 产出规格
写入以下文件（追加写入，不覆盖历史轮次）：
{general 模式}
- {project-dir}/review-notes.md
{audit 模式}
- {project-dir}/{topic-slug}/review-notes.md

如果发现需要补查的关键缺失（findings 中缺少支撑某个结论的重要信息），额外写入：
- {project-dir}/delta-tasks.md（general 模式）
- {project-dir}/{topic-slug}/delta-tasks.md（audit 模式）

## 审查维度

### 1. 事实准确性
结论是否有证据支撑？数据点是否正确？audit 模式：代码引用是否准确？

### 2. 来源可靠性
URL 是否真实？关键主张是否多源验证？

### 3. 逻辑完整性
推理链是否合理？有无跳跃？有无选择性引用？

### 4. 口径一致性
是否超出 brief 范围？README/short-comment 与 report 是否一致？

### 5. 遗漏检查
findings/source-audit 中的重要发现是否都在 report 中体现？

## 问题等级
- 致命：核心结论基于错误事实 → 必须修正
- 高：重要事实有误、证据缺失 → 必须修正
- 中：次要不准确、非关键遗漏 → 建议修正

## review-notes.md 格式（追加写入，每轮一个 section）
1. 轮次标题（## 第 N 轮）
2. 整体评价（一段话）
3. 问题清单（每条标等级 + 位置 + 描述 + 修正要求）
4. 统计（致命/高/中各多少项）
5. 结论（通过 / 打回修正）

## 跨轮核查（第 2 轮起）
- 读取 review-notes.md 中上轮的问题清单
- 逐一检查上轮问题是否被真正修正（而非仅改措辞）
- 未修正的问题直接升级一个等级（中→高，高→致命）
- 核查结果写入本轮 section，但不传递给 Analyst

## 通过条件
- 无"致命"级问题
- 无"高"级问题

## delta-tasks.md 格式（仅在有补查需求时写）
列出需要 Researcher 补查的具体问题，每条含：
- 缺失什么信息
- 建议搜索什么关键词
- 对应 report 中的哪个结论

## 红线
- 不做格式审查（不检查 markdown 语法）
- 不重写报告（只指出问题）
- 不发起搜索
- 不用 Markdown 表格语法
- 不要更新 pipeline-state.json
```

---

## 第二步：等待 Coordinator 完成

Coordinator 完成后，它会向你报告最终状态。

---

## 第三步：最终一致性审核（Main Agent）

Coordinator 完成后，你（Main Agent）读取所有产出文件，做最终审核：

### 审核内容

| 检查项 | 方法 |
|--------|------|
| 结论与证据对应 | report/README 里的每条结论，在 findings 或 source-audit 中能找到对应证据 |
| 前后矛盾 | report 的核心结论和 findings 中的原始数据不冲突 |
| 硬伤清单准确 | audit 模式下 README 的硬伤清单与 source-audit 的缺陷条目一一对应 |
| review 遗留 | 如果有 review-notes.md，确认致命/高级问题已处理或标注 |
| 口径一致 | 产出物没有超出 brief 定义的调研范围 |

### 输出

向用户交付：
```
课题：{课题名}
模式：{general / audit}
状态：{done / blocked}
审查轮次：{N}
未解决问题：{review-notes 中遗留的问题摘要，如有}
产出物路径：{project-dir}/
```

---

## spawn 参数建议

| 角色 | 建议 subagent_type | 超时 |
|------|-------------------|------|
| Coordinator | general-purpose | 900s |
| Researcher | general-purpose | 600s |
| Researcher (source) | general-purpose | 600s |
| Researcher (intel) | general-purpose | 600s |
| Analyst | general-purpose | 600s |
| Reviewer | general-purpose | 300s |

**不要对任何 subagent 使用 `isolation: "worktree"`。** 所有 subagent 共享同一个工作目录，通过产出文件路径不重叠来避免冲突（source 写 verification + audit，intel 写 findings，天然不冲突）。

---

## 通用红线

- 不编数据或来源
- 每条结论必须有可验证来源或数据点
- 每条建议必须写反对理由或适用条件
- 禁止常识建议（"要多验证"这种废话不写）
