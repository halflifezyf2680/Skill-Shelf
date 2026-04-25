---
name: create-agent-group
description: 引导式多 Agent 流水线设计器。从用例描述到完整的 agent-group/ 目录，生成 group.yaml、workflow.md、agent system prompt、产出物模板和发布规范。
---

# Create Agent Group

引导式多 Agent 流水线设计器。从用例描述出发，经 8 个阶段逐步设计，最终生成完整的 `agent-group/` 目录。

**你不做任何领域内容创作。** 你的职责是引导设计决策、生成结构化文件。所有领域知识由用户在交互中提供。

---

## 文件结构约定

每个 agent group 遵循以下结构：

```
{group-name}/
  agent-group/
    group.yaml              # 元数据、agents、拓扑、数据流、质量门控
    workflow.md             # 阶段执行手册
    publish-spec.md         # 发布流程规范
    agents/
      {agent-id}.md         # 各 agent 的 system prompt
    templates/
      {name}.md             # 产出物模板
```

---

## Phase 1：Inception

**目标**：理解用例。

如果有参数（如"代码审查流水线""内容翻译"），以此为起点。否则用 AskUserQuestion 询问。

### 需要搞清的问题

- **输入**：流水线接收什么？（用户描述 / 仓库 URL / 文档 / 数据集）
- **输出**：产出什么？（报告 / 代码变更 / 翻译文档 / 决策）
- **消费者**：最终用户是谁？（开发者 / 公众 / 内部团队）
- **质量要求**：事实准确性多重要？需要对抗审查吗？容错阈值？
- **复杂度**：估算有多少步骤？哪些可并行？

### 阶段产出

输出一句话问题定义，让用户确认：

```
问题：{流水线做什么}
输入：{什么进来}
输出：{什么出去}
质量：{关键质量要求}
复杂度：{simple / moderate / complex}
```

用户确认后进入 Phase 2。

---

## Phase 2：Pattern Selection

**目标**：选择编排模式。

### 模式 1：Sequential Pipeline

```
Agent A ──▶ Agent B ──▶ Agent C ──▶ Agent D
```

**适用**：线性变换，每步依赖上一步。
**典型 Agent 数**：2-6，各司其职。
**优势**：简单、可预测、易调试。
**劣势**：无并行，任何一步都是瓶颈。

---

### 模式 2：Orchestrator-Worker

```
            ┌──▶ Worker A ──┐
Orchestrator ├──▶ Worker B ──┤
            └──▶ Worker C ──┘
                    │
                    ▼
              Orchestrator (aggregate)
```

**适用**：大任务可拆解为独立子任务，中心节点负责分派和汇总。
**典型 Agent 数**：1 orchestrator + 2-N workers。
**优势**：并行度好，职责清晰。
**劣势**：Orchestrator 是单点瓶颈。

---

### 模式 3：Fan-out / Fan-in

```
Input ──┬──▶ Agent A ──┐
        ├──▶ Agent B ──┼──▶ Aggregator
        └──▶ Agent C ──┘
```

**适用**：同一任务应用于不同数据切片或视角。Workers 通常是同质的。
**典型 Agent 数**：1 router/aggregator + 2-N parallel workers。
**优势**：最大吞吐量。
**劣势**：无跨 worker 学习，聚合质量取决于覆盖率。

---

### 模式 4：Maker-Checker

```
  Maker ──▶ Checker ◀──┐
     ▲         │        │
     │    (有问题)──────┘
     │         │
     │    (通过)
     ▼
   交付
```

**适用**：质量关键的产出，需要对抗审查。Checker **只找问题，不确认通过**。
**典型 Agent 数**：1+ makers, 1+ checkers。
**优势**：高质量，捕捉单遍遗漏的错误。
**劣势**：较慢。必须设最大轮次上限。
**关键规则**：Checker 使用追加写入，与 Maker 上下文隔离。未修正问题跨轮自动升级等级。

---

### 模式 5：Dynamic Handoff

```
Router ──▶ Agent A ──▶ Router ──▶ Agent B ──▶ Router ──▶ Agent C
              ▲                                    │
              └────────────────────────────────────┘
```

**适用**：下一步取决于当前步输出的复杂工作流。Router 根据上下文决定转交给哪个专家。
**典型 Agent 数**：1 router + 2-N specialists。
**优势**：灵活，处理非线性工作流。
**劣势**：Router 逻辑复杂，所有路径难测试。
**关键规则**：Router 必须有明确的决策标准，不能随意路由。

---

### 模式 6：Adaptive Planning

```
Planner ──▶ Worker ──▶ Evaluator ──▶ Planner (revise)
                                        │
                                   (plan complete)
                                        ▼
                                     交付
```

**适用**：开放性任务，方案需要根据中间结果动态调整。
**典型 Agent 数**：1 planner + 1+ workers + 1 evaluator。
**优势**：处理不确定性，能发现更好的方法。
**劣势**：执行时间不可控。必须有最大迭代上限和明确的完成条件。

---

### 选择建议

| 用例特征 | 推荐模式 |
|---------|---------|
| 线性 A→B→C 变换 | Sequential Pipeline |
| 大任务拆子任务再汇总 | Orchestrator-Worker |
| 同一任务，不同数据 | Fan-out/Fan-in |
| 质量关键 + 对抗审查 | Maker-Checker |
| 下一步取决于当前输出 | Dynamic Handoff |
| 方案随进展演变 | Adaptive Planning |

**支持混合模式。** 复杂流水线通常组合多种模式（如 Fan-out + Sequential + Maker-Checker）。

用 AskUserQuestion 让用户选择模式，或根据问题定义推荐。

---

## Phase 3：Agent Design

**目标**：定义每个 agent。

### Agent 数量上限

**不超过 6 个 agent。** 超过时建议合并相似角色或拆分为独立流水线。协调开销随 agent 数平方增长。

### 每个 Agent 需定义

- `{agent-id}`: kebab-case 标识（如 `coordinator`, `researcher-source`）
- `{角色名}`: 中文角色名
- `model`: `sonnet`（编排/采集）或 `opus`（分析/写作/审查）
- `instance_count`: `1` / `1..N`（可并行）/ `N`（始终多实例）
- 职责描述（3-5 条）
- 核心原则（3-5 条）
- 隔离规则（能与谁通信，不能与谁通信）

### 模型分级指南

| 角色类型 | 推荐 | 原因 |
|---------|------|------|
| 编排 / 路由 | sonnet | 结构性工作，非创造性 |
| 数据采集 / 提取 | sonnet | 机械性、模式驱动 |
| 分析 / 综合 | opus | 需要深度理解 |
| 写作 / 内容创作 | opus | 创作质量重要 |
| 审查 / 质量检查 | opus | 需要批判性思维 |

### 阶段产出

呈现 agent 花名册表格，让用户确认：

```
| Agent ID    | 角色     | Model  | 实例  | 核心职责          |
|-------------|---------|--------|-------|-------------------|
| coordinator | 任务官   | sonnet | 1     | 流程调度          |
| researcher  | 调研员   | sonnet | 1..N  | 数据采集          |
| analyst     | 成文员   | opus   | 1     | 整合写作          |
| reviewer    | 审查员   | opus   | 1     | 对抗审查          |
```

用户确认后进入 Phase 4。

---

## Phase 4：Data Flow Design

**目标**：定义每个 agent 的输入/输出契约。

### 规则

1. 每个 agent 的输入/输出必须显式定义
2. 输出来自用户或上一个 agent 的输出
3. 输出始终是文件（不依赖内存状态）
4. 中间产物和最终交付物同样重要

### 每个 Agent 定义

**输入契约**：需要什么文件？从哪来？格式要求？

**输出契约**：产出什么文件？保存到哪里？遵循什么模板？

### 阶段产出

呈现数据流表格，让用户确认：

```
| Agent        | 输入文件                | 输出文件                     |
|-------------|------------------------|------------------------------|
| coordinator | 用户课题描述             | brief.md                    |
| researcher  | brief.md + 关键词        | findings.md                 |
| analyst     | brief.md + findings     | report.md, README.md       |
| reviewer    | analyst 产出 + findings | review-notes.md             |
```

用户确认后进入 Phase 5。

---

## Phase 5：Quality Gates

**目标**：定义质量检查点。

### 三级门控

**第一级：交接校验**（每个交接点）
- 文件存在性
- 必需章节标题存在
- 格式/结构合规
- 失败处理：拒绝接收，打回上游修复

**第二级：内容检查**（关键转换点）
- 证据完整性（每条结论有支撑）
- 覆盖度（所有必需维度已涉及）
- 交叉验证（关键事实多源确认）
- 失败处理：标记问题，补查 1 轮
- 降级策略：补查仍不通过 → 标注"不完整"继续推进

**第三级：审查循环**（Maker-Checker 场景）
- 对抗性审查，问题分级（致命/高/中/低）
- 最大轮次上限（通常 3 轮）
- 追加写入，跨轮核查，未修正问题自动升级
- 达到上限 → 带已知问题强制交付

### 阶段产出

呈现门控摘要表，让用户确认：

```
| 门控       | 位置         | 检查内容           | 失败处理              |
|-----------|-------------|-------------------|---------------------|
| 交接校验   | 每个 agent  | 文件存在 + 结构   | 拒绝，修复          |
| 内容检查   | 关键转换点  | 证据 + 覆盖度     | 1 轮补查 / 降级推进 |
| 审查循环   | Maker→Checker | 全维度审查        | 最多 3 轮，强制交付 |
```

用户确认后进入 Phase 6。

---

## Phase 6：Output Structure

**目标**：定义产出物目录结构。

### 需要确认

- 最终交付物有哪些文件？
- 中间产物是否保留？
- 发布目标？（GitHub / 本地 / 其他）
- 每次运行独立目录还是共享目录？

### 阶段产出

呈现目录树模板，让用户确认。确认后进入 Phase 7。

---

## Phase 7：Generation

**目标**：生成全部文件。

### 生成顺序

1. `agent-group/group.yaml`
2. `agent-group/workflow.md`
3. `agent-group/agents/{agent-id}.md`（每个 agent 一个文件）
4. `agent-group/templates/{name}.md`（每个产出物一个模板）
5. `agent-group/publish-spec.md`

### 7.1 group.yaml 模板

```yaml
# {Group Name} Agent Group
# {一句话描述}
#
# 本目录是设计文档，不是运行时入口。
# skill 改了要同步本目录，反之亦然。

group:
  name: {group-name}
  version: "1.0"
  description: >
    {2-3 句话描述}

agents:
  {agent-id}:
    role: {角色名}
    model: {sonnet / opus}
    instance_count: {1 / 1..N / N}
    description: >
      {职责描述}

topology:
  parallel_groups:
    - id: {group-name}
      agents: [{agent-id}, {agent-id}]
      trigger: {触发条件}
      wait: {all / any}

  sequential_chain:
    - id: {phase-name}
      agents: [{agent-id}]
      trigger: {触发条件}
      max_rounds: {N}
      loop_condition: {条件}

  diagram: |
    {ASCII 拓扑图}

data_flow:
  {agent-id}_input:
    - {输入项}
  {agent-id}_output:
    - "{path}/{file.md}"

quality_gates:
  {gate-name}:
    description: {门控描述}
    checks:
      - {检查项}
    on_fail: {处理方式}
    on_fail_persist: {降级处理}

output_structure:
  template: |
    {目录树模板}

runtime:
  run_log: {run-dir}/run-log.md
  log_format: |
    ## Phase {N} — {阶段名}
    - 执行者：{Agent}
    - 开始时间：{ISO 时间}
    - 结束时间：{ISO 时间}
    - 状态：{成功 / 部分失败 / 失败}
    - 备注：{异常或需要关注的情况}
```

### 7.2 workflow.md 模板

```markdown
# Workflow — 流水线执行手册

## 阶段总览

{ASCII 或表格展示所有阶段}

## 通用规则：交接点校验

每个 Agent 之间的数据交接，接收方必须先做格式校验再开始工作。
校验不通过则拒绝接收，向 {编排角色} 报告具体缺失项。

| 交接点 | 校验内容 | 校验方式 |
|--------|---------|---------|
| {Agent A} → {Agent B} | {文件} 存在且包含必需 section | 标题检查 |
| ... | ... | ... |

## 执行记录

每个阶段结束时，{编排角色} 记录执行元数据（追加到 `{run-dir}/run-log.md`）。

---

## Phase {N} — {阶段名}

**执行者：** {Agent Role}
**触发：** {触发条件}

### 步骤
1. {步骤}
2. {步骤}

{按需附加：并行规则 / 质量检查 / 决策树}
```

### 7.3 Agent System Prompt 模板

`agents/{agent-id}.md` 遵循以下结构：

```markdown
# {Agent Name} — {角色名} System Prompt

你是 {Group Name} 流水线的{角色}。{一句话核心职责}。

## 核心原则

1. **{原则名}** — {描述}
2. **{原则名}** — {描述}
3. **{原则名}** — {描述}

## 工作流程

### Step 1: {步骤名}
{详细步骤}

### Step 2: {步骤名}
{详细步骤}

## 输入材料

| 文件 | 内容 | 用途 |
|------|------|------|
| `{file.md}` | {描述} | {用途} |

## 产出物

{列出产出文件及格式要求}

## 禁止

- 禁止{行为 1}
- 禁止{行为 2}
```

**Agent prompt 规则：**
- 开头一句清晰的身分声明
- 核心原则必须领域相关，不能是泛泛的
- 每个步骤必须具体到可无歧义地执行
- 如果是 Checker 角色：加"只找问题，不确认通过"
- 如果有隔离要求：加"禁止与 {other agent} 通信"
- 禁止项覆盖最可能的失败模式

### 7.4 Template 模板

`templates/{name}.md` 遵循以下结构：

```markdown
# {Template Name} — {用途描述}模板

> 由 {Agent Role} 生成。

---

# {产出物标题}

> {元数据}

---

## {Section 1}

{内容或 placeholder}

## {Section 2}

{内容或 placeholder}
```

### 7.5 publish-spec.md 模板

```markdown
# Publish Workflow — 发布流程

> 由 {编排角色} 在交付阶段触发。

## 仓库结构约定

{目录树}

## Commit 规范

{commit message 格式}

## 执行步骤

### Step 1: {步骤}
{操作}

## 错误处理

| 场景 | 处理 |
|------|------|
| {场景} | {方式} |

## 禁止

- 禁止 `git add -A`
- 禁止 `git push --force`
- 禁止提交 agent-group/ 目录
```

---

## Phase 8：Review and Iterate

全部文件生成后，呈现摘要：

```
## Agent Group 设计摘要

名称：{group-name}
模式：{pattern name}
Agent 数量：{N}
质量门控：{N} 个
产出物：{列表}

### 生成文件
- [x] agent-group/group.yaml
- [x] agent-group/workflow.md
- [x] agent-group/agents/{agent-1}.md
- [x] agent-group/templates/{template-1}.md
- [x] agent-group/publish-spec.md
```

用 AskUserQuestion 询问是否需要调整。如需调整，修改后重新呈现。

---

## 设计原则

### 要做的

- **从最简单的模式开始。** 能用 Sequential 就不用 Orchestrator-Worker。
- **每个 agent 只有一个核心职责。** 一个 agent 做两件事就该拆分。
- **显式定义输入/输出契约。** 模糊是流水线 bug 的头号原因。
- **编排用便宜模型，质量关键用强模型。**
- **每个流水线都加执行记录。** 没有日志的调试是猜谜。
- **每个循环都必须有退出条件。** 最大轮次 + 强制交付。
- **Agent prompt 用领域语言写。** 领域具体的指令优于泛泛而谈。
- **"完成"标准必须明确。** "完成"不应该有主观判断空间。

### 不要做的

- **不要超过 6 个 agent。**
- **不要跳过交接校验。**
- **不要让 agent 直接修改其他 agent 的产出。** 始终通过编排者中转。
- **不要创建循环依赖。** 数据流可以循环但不能有环。
- **不要留交接校验未定义。**
- **不要用同一个 agent 做 maker 和 checker。** 同一个模型不能对抗性审查自己。
- **不要让编排者做领域工作。** 编排者管流程，不管内容。

### 常见反模式

| 反模式 | 症状 | 修正 |
|--------|------|------|
| Agent 过载 | 一个 agent 做 5+ 件事 | 拆分为专注 agent |
| 无编排者 | Agent 随意通信 | 加 coordinator |
| 无限审查循环 | Reviewer 和 Maker 永远循环 | 设最大轮次 + 强制交付 |
| 交接未定义 | 下游收到垃圾输入 | 每个交接加格式校验 |
| 全串行 | 明明可并行的步骤串行执行 | 识别独立步骤 |
| 全能编排者 | Coordinator 做了所有实际工作 | 把领域工作推给专家 agent |
