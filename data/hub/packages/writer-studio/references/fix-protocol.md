# 定点修正协议

当 draft audit 结论为 `fail` 或 `concerns`，或作者主动要求修改已有正文时，按本协议执行修正。

## 适用场景

- `writer-audit` 返回 `fail`，必须修正
- `writer-audit` 返回 `concerns`，作者决定修正
- 作者对已完成 sync 的章节要求回溯修改

## 三类修正路径

### 一、文风修正

**触发条件：**
- audit 报告中非致命项 N4（风格与角色语气一致性）、N5（情感空洞）、N6（对话纯功能化）、N7（刻意碎句）命中
- 作者对文风不满意，但结构和设定没有问题

**回退起点：** `draft → audit`
- 保留 packet，只重新起草 draft
- 重新运行 `writer-audit`
- 不需要重跑 `writer-build-packet`

**修正重点：**
- 对照 audit 报告中命中的文风相关条目逐条修正
- 检查 style_constraints 中的禁止句式模式
- 核验情感密度是否达标
- 检查对话非功能信息比例

### 二、结构修正

**触发条件：**
- audit 报告中致命项命中（F1-F4）
- N1（canon 一致性）、N2（时间锚点一致性）、N3（物件流转一致性）、N8（功能完成度）高严重度命中
- 本章 beats 或 exit_state 需要调整

**回退起点：** `packet → draft → audit`
- 先修正 packet（调整 required_beats、forbidden_moves、exit_state 等）
- 再按新 packet 重新起草 draft
- 再重新运行 `writer-audit`

**修正重点：**
- 先定位 packet 中哪些字段导致了问题
- 修正 packet 后标注变更理由
- 按 fix-protocol 的附带规则提供上次 audit 报告
- 新 draft 完成后进入 audit 时，检查修正项是否全部关闭

### 三、设定修正

**触发条件：**
- audit 发现 canon 本身有问题（正文中暴露的设定矛盾不是写法问题，而是 canon 需要修改）
- 作者决定修改已确立的 canon 事实

**回退起点：** `canonize → packet → draft → audit`
- 先通过 `writer-canonize` 修正 canon
- 再重新构建 packet
- 再重新起草 draft
- 再重新运行 `writer-audit`

**修正重点：**
- 必须生成 canon 修正决策记录（使用 canon-decision-template）
- 必须标注哪些 canon 节点被修改、新增或删除
- 必须评估修正对已有章节的连锁影响
- 修正后必须运行 `writer-sync` 更新 truth surface

## 修正轮次计数

| 修正类型 | 最大累计轮次 | 说明 |
|----------|-------------|------|
| 文风修正 | 3 次 | 超过后暂停，升级为结构修正 |
| 结构修正 | 3 次 | 超过后暂停，升级为设定修正 |
| 设定修正 | 2 次 | 超过后必须人工介入审查 canon 架构 |

"轮次"指从同一 audit 报告出发的修正尝试。每次 audit 返回新的 verdict，轮次计数重新开始。

如果一轮修正后 audit 再次 `fail` 且命中同类问题，视为同一轮次的延续。

## 修正附带规则

每次启动修正时，必须附带：

1. **上次 audit 报告全文**
   - 修正者必须逐条对照上次 audit 的 HIT/MISS 结果
   - 不允许"看了摘要就改"——必须看到完整的位置和说明

2. **修正范围声明**
   - 明确声明本次修正属于文风/结构/设定中的哪一类
   - 列出本次修正要关闭的 audit 条目编号

3. **lessons-learned 检查**
   - 读取项目级 `lessons-learned.md`（如存在）
   - 检查本次 audit 暴露的问题是否与已有教训重复
   - 如果是重复问题，在修正报告中标注

## 修正输出格式

每次修正完成后，输出以下修正记录：

```markdown
# 修正记录

- 修正类型：文风 / 结构 / 设定
- 原始 audit verdict：fail / concerns
- 修正轮次：第 N 轮
- 修正范围声明：
  - 本次要关闭的 audit 条目：F1, N4, N5
  - 本次不涉及的范围：canon 修改

## 逐条修正结果

| Audit 条目 | 原始结果 | 修正措施 | 预期新结果 |
|------------|----------|----------|-----------|
| F1 | MISS | 删除第 3 段元叙述句，改用角色内心独白 | HIT |
| N5 | MISS（高）| 在追逐段落中插入角色恐慌的生理反应 | HIT |

## 是否触发 lessons-learned

- [ ] 是，新增教训：（描述）
- [ ] 否，已有教训已覆盖
- [ ] 否，属于首次出现的问题
```

## 超限处理

当修正轮次用尽仍未通过 audit：

1. 暂停自动修正流程
2. 输出"修正超限报告"，包含：
   - 所有轮次的 audit 报告汇总
   - 反复命中的条目和原因分析
   - 建议：是否需要回退到更早的阶段（如 canonize 或 init）
3. 必须由人工决策下一步
