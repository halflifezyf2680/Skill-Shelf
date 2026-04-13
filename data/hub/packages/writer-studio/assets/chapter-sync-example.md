# Packet 到 Sync 示例链路

## 1. Chapter Packet

- 章节 ID：`ch.01.04`
- 所属卷 / 所属线：卷01 / 倒叙1
- 章节功能：写出“真话被压下”的制度性失败
- POV：`char.wang_zhiyan`
- 时间锚点：城破前1年春
- 地点：`place.capital`
- 开章前状态：
  - 王知言已整理冀州真实数据
  - 周延仍在户部任职
- 必须完成的 beats：
  - 王知言递交奏报
  - 周延接收并判断风险
  - 奏报被压下
- 禁止动作：
  - 不允许让皇帝直接出场
  - 不允许让奏报当场进入公开流程
- 必须引用的 canon：
  - `doc.report_jizhou_real`
  - `evt.report_submission`
  - `rel.submission_001`
- 角色约束：
  - 王知言：理性、克制、老迈但仍有硬度
  - 周延：清楚后果，不是脸谱化反派
- 风格约束：
  - 信息密度高
  - 不煽情
  - 不越 POV
- 出章状态：
  - 奏报未被采纳
  - 周延成为压报链路的一环
- sync 风险提示：
  - 如果出现新的压报执行人，需要形成新关系或新事件

## 2. Draft Audit

- 章节 ID：`ch.01.04`
- Verdict：`concerns`
- canon 一致性：通过
- POV 边界：通过
- 风格一致性：通过
- 时间线一致性：通过
- 物件连续性：通过
- 知情状态一致性：需要补充
- 功能完成度：通过
- 是否误造 canon：轻微风险
- 必改项：
  - 如果正文明确写出“周延把奏报锁入柜中”，则必须形成新的文书状态或关系断言

## 3. Canon Delta

- 来源章节：`ch.01.04`
- 新名词：无
- 新事实：
  - 奏报在当日未进入公开流程
- 关系变化：
  - 周延与 `doc.report_jizhou_real` 形成“持有并压下”关系
- 知情状态变化：
  - 周延确认知道奏报真实内容
- 物件 / 文书转移：
  - 奏报从王知言提交到周延处
- 时间锚点：
  - 无新增
- 跨章节约束：
  - 后续若写奏报再出现，必须说明流转路径
- YAML / 结构化文档落点：
  - `relations`
  - `knowledge_states`
  - `chapter_assertions`
- 数据库编译影响：
  - 更新 `relations`
  - 更新 `knowledge_states`
  - 更新 `chapter_assertions`
- 明确拒绝为非 canon：
  - 王知言离开户部时看到的景物细节

## 4. Sync 完成

1. 先更新 YAML truth surface
2. 再运行扫描、校验、落库脚本
3. 最后重建章节上下文和关系视图

