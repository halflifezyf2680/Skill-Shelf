# Canon Sync 规范

在 draft audit 完成之后，再决定 prose 中哪些内容需要影响项目 truth surface。

## 目标

只提取那些会影响未来写作与一致性的变化。

## 应提取的 Canon Delta

- 新名词
- 新事实
- 关系变化
- 知情状态变化
- 物件或文书转移
- 新时间锚点
- 新的跨章节约束

## 不应提取的内容

- 纯修辞性描写
- 一次性情绪化表述
- 装饰性细节
- 已经存在于 canon 的内容

## Sync 决策

每个 delta 项都要明确落点，但顺序必须统一：

1. 先写回 YAML / 结构化文档 truth surface
2. 再由项目脚本编译到数据库查询面
3. 如有必要，再写入 memo / 决策日志
4. 如果不应进入 canon，明确拒绝为非 canon

## 禁止事项

- 不要先改数据库、后补文档
- 不要让数据库先于 truth surface 成为人工修改面
- 不要同时维护两套不同口径的 canon

## Sync 完成标准

- truth surface 已更新
- 数据库查询面已重新编译
- 关键派生视图已更新
- 当前章节状态可以从 `reviewed` 推进到 `synced`
