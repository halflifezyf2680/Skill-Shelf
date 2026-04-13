# 状态与缺口检测

`writer-status` 是 `writer-studio` 的入口级动作。

它的职责不是写正文，而是回答三件事：

1. 现在处于哪个阶段
2. 当前最关键的缺口是什么
3. 下一步该调用哪个命令

## 检测顺序

按这个顺序检测：

1. 是否已有 truth surface
2. 是否已有冻结后的数据结构维护协议
3. 是否已有项目脚本计划
4. 是否已有可用的 chapter packet
5. 是否已有 draft
6. 是否已有 audit
7. 是否已有待处理 canon delta

## 常见缺口

- 有 brainstorm，但没有 canon 决策
- 有 truth surface，但没有数据结构维护协议
- 有协议，但没有脚本计划
- 有 canon，但没有 packet
- 有 draft，但没有 audit
- 有 delta，但没有 sync
- 有 truth surface 更新，但没有查询面重编译

## 推荐输出格式

- 当前阶段
- 已有产物
- 缺口列表
- 推荐下一步命令

