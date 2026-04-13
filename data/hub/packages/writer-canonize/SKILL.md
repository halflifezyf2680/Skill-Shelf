---
name: writer-canonize
description: "把 brainstorm 或 candidate 材料收敛成正式 canon。用于命名、设定、情节、角色关系、事件方案已经出现多个候选，需要选定并进入 truth surface 的时候。"
---

# Writer Canonize

这是 `writer-studio` 的 canon 收敛入口。

## 用途

- 分离 brainstorm、candidate、canon
- 形成正式决策
- 将可采纳内容写入 truth surface

## 先读

- `../writer-studio/references/brainstorm-to-canon.md`
- `../writer-studio/references/command-system.md`
- `../writer-studio/references/db-protocol.md`

## 可用模板

- `../writer-studio/assets/canon-decision-template.md`

## 固定产物

- canon 决策记录
- 可写入 truth surface 的摘要
- 受影响节点 / 文件清单

## 最短示例

### 输入

```text
请用 writer-canonize 帮我在这三个男主初登场方案里选一个正式 canon，并指出要更新哪些 truth surface 节点。
```

### 输入前提

- 已经有 brainstorm 或 candidate 材料
- 候选方案之间存在冲突、重叠或取舍关系
- 现在要把其中一部分升级为正式 canon

### 预期输出

- 选中方案
- 否决或暂缓方案
- 可同步进 truth surface 的正式摘要
- 受影响节点 / 文件清单

## 不要在什么时候用

- 你还在头脑风暴，没有形成候选集
- 你不是在做决策，而是在准备写某一章，此时用 `writer-build-packet`
- 你已经有成稿，需要判断能否纳入 canon，此时先用 `writer-audit`

## 完成标准

- 候选和正式事实已分开
- 选中方案明确
- 被否决方案明确
- 新 canon 可落入 truth surface
