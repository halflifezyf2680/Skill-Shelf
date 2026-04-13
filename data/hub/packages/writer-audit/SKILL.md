---
name: writer-audit
description: "对 draft 做逻辑、连续性、POV、风格、物件流转和知情状态审校。用于正文已经写出，但在进入 sync 前需要判断是否合格的时候。"
---

# Writer Audit

这是 `writer-studio` 的草稿审校入口。

## 用途

- 判断 draft 是否可进入 sync
- 找出逻辑和连续性问题
- 输出明确 verdict 和必改项

## 先读

- `../writer-studio/references/draft-audit-spec.md`
- `../writer-studio/references/command-system.md`

## 可用模板

- `../writer-studio/assets/draft-audit-template.md`

## 固定产物

- audit report
- verdict
- 必改项

## 最短示例

### 输入

```text
请用 writer-audit 审一下第12章 draft，重点看逻辑一致性、POV 越界、物件流转和人物知情状态。
```

### 输入前提

- 已经有成型 draft
- 最好同时有对应的 chapter packet 或可查询的 truth surface
- 当前目标是判断这版稿子是否能进入 sync

### 预期输出

- audit report
- verdict
- 必改项
- 是否建议进入 sync

## 不要在什么时候用

- 你还没写 draft，此时应先做 `writer-build-packet`
- 你当前是在做设定收敛，不是在审正文，此时用 `writer-canonize`
- 你已经完成审校且 verdict 通过，此时直接做 `writer-sync`

## 完成标准

- verdict 明确
- 问题项明确
- 是否建议进入 sync 明确
