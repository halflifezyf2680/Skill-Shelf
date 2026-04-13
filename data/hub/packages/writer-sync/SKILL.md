---
name: writer-sync
description: "提取 canon delta，更新 truth surface，并重编译数据库查询面与派生视图。用于 draft audit 通过后，需要把新事实正式纳入项目结构层的时候。"
---

# Writer Sync

这是 `writer-studio` 的同步入口。

## 用途

- 提取 canon delta
- 更新 truth surface
- 重编译数据库查询面
- 更新派生视图

## 先读

- `../writer-studio/references/canon-sync-spec.md`
- `../writer-studio/references/db-protocol.md`
- `../writer-studio/references/command-system.md`

## 可用模板

- `../writer-studio/assets/canon-delta-template.md`
- `../writer-studio/assets/sync-result-template.md`

## 固定产物

- canon delta
- truth surface 更新
- 查询面重编译结果
- sync 结果摘要

## 最短示例

### 输入

```text
请用 writer-sync 把第12章 audit 通过后的新事实同步进 truth surface，并重编译数据库查询面。
```

### 输入前提

- draft 已经过审，或至少已确认哪些事实可以纳入 canon
- 项目已经存在结构化 truth surface
- 项目有对应的扫描 / 校验 / 落库 / 查询脚本计划

### 预期输出

- canon delta
- truth surface 更新项
- 查询面重编译结果
- 当前章节状态推进到 `synced`

## 不要在什么时候用

- draft 还没审，先用 `writer-audit`
- 当前任务是写作前准备，不是同步 canon，此时用 `writer-build-packet`
- 当前任务是初始化项目协议，而不是更新既有项目，此时用 `writer-init`

## 完成标准

- delta 已提取
- truth surface 已更新
- 查询面已重编译
- 派生视图已更新
- 当前章节可推进到 `synced`
