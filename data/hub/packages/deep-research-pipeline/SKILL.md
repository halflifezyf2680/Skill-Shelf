---
name: deep-research-pipeline
description: 多 agent 深度调研流水线。coordinator 调度 researcher 调研、analyst 分析、reviewer 审查，产出从 brief 到最终 report 的完整调研报告。用于需要结构化深度研究的课题。
---

# Deep Research Pipeline

多 agent 串行流水线，产出可服务决策的调研报告。

## 流程

```
brief(coordinator) → findings(researcher) → report(analyst) → review-notes(reviewer)
                                                                    ↓
                                                        delta-tasks? → 补查轮或 done
```

## 状态机

用 `status.json` 推进，`{"phase":"...","round":1,"max_rounds":3}`：

- **need_research** → spawn researcher（参考 `researcher-prompt.md`）→ `sessions_yield` → phase 改 `need_analysis`
- **need_analysis** → spawn analyst（参考 `analyst-prompt.md`）→ `sessions_yield` → phase 改 `need_review`
- **need_review** → spawn reviewer（参考 `reviewer-prompt.md`）→ `sessions_yield`：
  - 有 delta-tasks 且 round < max_rounds → round+1，phase 改 `need_research`
  - 无 delta-tasks → phase 改 `done`
  - 只需局部修正 → phase 改 `need_revision`
  - reviewer 未产出 → coordinator 自行简化审查
- **need_revision** → spawn analyst 做局部修正 → phase 改 `need_review` 或 `done`

## 角色

| 角色 | 职责 | 产出 |
|------|------|------|
| coordinator | 调度流程、维护 brief、做最终判断 | brief.md, status.json |
| researcher | 搜集信息、量化优先、写 findings | findings.md, research-plan.md |
| analyst | 整合 findings 产出 report | report.md |
| reviewer | 独立审查，挑刺不表扬 | review-notes.md, delta-tasks.md |

## spawn 参数

`runtime: "subagent"`，超时：researcher 600s / analyst 420s / reviewer 300s / revision 300s。

spawn 后必须 `sessions_yield` 等结果。

subagent 超时时：检查部分产出，有则接受继续，无则重试一次（task 加"上次超时，精简输出"），仍超时则 phase 改 `blocked`。

## 产出文件

```
{project-dir}/
  brief.md           ← coordinator 写
  status.json         ← coordinator 维护
  research-plan.md    ← researcher 写（仅初始轮）
  findings.md         ← researcher 写
  report.md           ← analyst 写
  review-notes.md     ← reviewer 写
  delta-tasks.md      ← reviewer 写（有关键缺失时）
```

## 通用红线

- 不替子角色写产出（coordinator 不写 findings/report）
- 不跳 phase
- 不无限返工（max_rounds 默认 3）
- 不越出项目目录
- 不编数据或来源
- **禁止常识建议**（"要多验证"这种废话不写）
- **不要表格**，用列表、加粗、分段
- 每条结论必须有可验证来源或数据点
- 每条建议必须写反对理由或适用条件
