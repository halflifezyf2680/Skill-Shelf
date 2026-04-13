---
name: writer-build-packet
description: "从 truth surface 和查询结果生成正式 chapter packet。用于准备写章、写场景、或在 AI 主写前先锁定章节功能、POV、时间锚点和禁止动作的时候。"
---

# Writer Build Packet

这是 `writer-studio` 的章节执行包入口。

## 用途

- 为章节建立正式执行包
- 把 canon 压成可写、可审的受约束输入

## 先读

- `../writer-studio/references/chapter-packet-spec.md`
- `../writer-studio/references/command-system.md`

## 可用模板

- `../writer-studio/assets/chapter-packet-template.md`

## 固定产物

- 正式 chapter packet

## 最短示例

### 输入

```text
请用 writer-build-packet 为第12章生成正式 chapter packet，锁定 POV、时间锚点、required beats 和 forbidden moves。
```

### 输入前提

- 相关章节的 truth surface 已经存在
- 章节目标、前置事实和主要限制已经可查询
- 你准备进入 AI 主写或正式写作

### 预期输出

- 可直接用于写作的 chapter packet
- 本章必须命中的 beats
- 本章不能越界的动作和信息边界

## 不要在什么时候用

- 你还没有稳定的 truth surface，此时先做 `writer-init` 或 `writer-canonize`
- 你不是要写章，而是在判断系统缺口，此时用 `writer-status`
- 正文已经写完，要检查问题或回写 canon，此时用 `writer-audit` / `writer-sync`

## 完成标准

- 最小字段齐全
- 章节功能明确
- POV、时间、地点明确
- required beats 与 forbidden moves 明确
