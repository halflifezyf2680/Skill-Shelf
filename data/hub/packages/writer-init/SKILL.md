---
name: writer-init
description: "初始化长篇叙事项目的 truth surface、结构化数据维护协议和脚本计划。用于项目刚开始、只有散乱文档、或准备把写作系统正式工程化的时候。"
---

# Writer Init

这是 `writer-studio` 的项目初始化入口。

## 用途

- 定义 truth surface
- 定义 YAML / 数据库同构协议
- 生成项目脚本计划

## 先读

- `../writer-studio/references/workflow.md`
- `../writer-studio/references/db-protocol.md`
- `../writer-studio/references/command-system.md`

## 可用模板

- `../writer-studio/assets/db-protocol-template.md`
- `../writer-studio/assets/script-plan-template.md`
- `../writer-studio/assets/init-result-template.md`

## 固定产物

- 数据结构维护协议
- 脚本计划
- 初始化结果摘要

## 最短示例

### 输入

```text
请用 writer-init 为这个长篇项目建立 truth surface、YAML/数据库同构协议和脚本计划。
```

### 输入前提

- 项目刚开始，或只有散乱设定文档
- 你准备把写作系统正式工程化
- 你还没有统一的数据结构维护协议

### 预期输出

- 项目 truth surface 边界说明
- 结构化维护协议草案
- 脚本层建设计划
- 项目进入 `protocol-ready`

## 不要在什么时候用

- 项目协议已经稳定，只是想判断当前缺口，此时用 `writer-status`
- 当前任务是从多个候选中选正式 canon，此时用 `writer-canonize`
- 当前任务是写章前准备执行包，此时用 `writer-build-packet`

## 完成标准

- truth surface 已明确
- 结构语义已明确
- 脚本计划已明确
- 项目进入 `protocol-ready`
