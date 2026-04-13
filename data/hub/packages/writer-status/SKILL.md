---
name: writer-status
description: "检测长篇叙事项目当前所处阶段、当前缺口和下一步建议。用于你不知道现在该先做什么，或怀疑项目缺少 truth surface、协议、packet、audit、sync 其中某一环的时候。"
---

# Writer Status

这是 `writer-studio` 的状态入口。

## 用途

- 判断项目阶段
- 找出当前缺口
- 推荐下一步命令

## 先读

- `../writer-studio/references/stage-model.md`
- `../writer-studio/references/status-gap-detect.md`
- `../writer-studio/references/command-system.md`

## 固定产物

- 状态报告
- 缺口列表
- 下一步命令建议

## 最短示例

### 输入

```text
请用 writer-status 看一下我这个小说项目现在处于什么阶段，下一步该做什么。
```

### 输入前提

- 项目里已经存在部分写作文档，或你怀疑它们不完整
- 你现在不知道该先补协议、补 truth surface，还是先做 packet / audit / sync

### 预期输出

- 当前阶段判断
- 当前关键缺口
- 建议直接调用的下一个子 skill

## 不要在什么时候用

- 你已经明确知道下一步就是初始化项目，此时直接用 `writer-init`
- 你已经拿着多个候选方案要收敛，此时直接用 `writer-canonize`
- 你已经有 chapter packet 或 draft，要执行审校或同步，此时直接进入对应命令

## 完成标准

- 当前阶段明确
- 当前最关键缺口明确
- 下一步建议明确
