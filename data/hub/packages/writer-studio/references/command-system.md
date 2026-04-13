# 命令层设计

`writer-studio` 作为总工作室 skill 存在，但真正要保证“可用性”，必须把核心流程拆成动作级命令。

本文件定义这些命令的职责、输入、输出和完成标准。

## 命令列表

- `writer-status`
- `writer-init`
- `writer-canonize`
- `writer-build-packet`
- `writer-audit`
- `writer-sync`

## 一、writer-status

### 用途

- 判断项目当前阶段
- 找出当前缺口
- 推荐下一步命令

### 输入

- 项目现有 truth surface
- 项目协议文件
- 当前 draft / packet / audit / delta 文件

### 输出

- 阶段判断
- 缺口列表
- 下一步建议

### 完成标准

- 能说清当前阶段
- 能指出当前最关键缺口
- 能明确推荐下一步命令

## 二、writer-init

### 用途

- 初始化项目级闭环
- 定 truth surface、数据协议和脚本计划

### 输入

- 项目目标
- 项目复杂度
- 预期维护对象

### 输出

- 数据结构维护协议
- 脚本计划
- 第一版 truth surface 方案

### 完成标准

- truth surface 已明确
- YAML 与数据库的关系已明确
- 项目脚本计划已明确
- 项目进入 `protocol-ready`

## 三、writer-canonize

### 用途

- 把 brainstorm / candidate 收敛成 canon

### 输入

- brainstorm 材料
- candidate 列表
- 目标主题

### 输出

- canon 决策记录
- 可写入 truth surface 的摘要
- 受影响节点 / 文件清单

### 完成标准

- 候选与 canon 已分离
- 选中方案明确
- 被否决方案明确
- 新 canon 可落入 truth surface

## 四、writer-build-packet

### 用途

- 从 truth surface 与查询面生成正式 chapter packet

### 输入

- 目标章节
- truth surface
- 查询结果 / 关系快照 / 知情状态

### 输出

- 正式 chapter packet

### 完成标准

- packet 最小字段齐全
- 章节功能明确
- POV、时间、地点明确
- required beats 与 forbidden moves 明确

## 五、writer-audit

### 用途

- 对 draft 做一致性与逻辑审校

### 输入

- draft
- chapter packet
- truth surface
- 查询结果

### 输出

- audit report
- verdict
- 必改项

### 完成标准

- verdict 明确
- 问题项明确
- 同步建议明确

## 六、writer-sync

### 用途

- 提取 canon delta
- 回写 truth surface
- 编译查询面

### 输入

- draft
- audit report
- chapter packet
- truth surface

### 输出

- canon delta
- truth surface 更新
- 查询面重编译结果
- sync 结果摘要

### 完成标准

- delta 已提取
- truth surface 已更新
- 数据库查询面已重编译
- 派生视图已更新
- 当前章节可推进到 `synced`

## 推荐调用顺序

### 项目初始化

`writer-status -> writer-init -> writer-canonize`

### 正常写章

`writer-status -> writer-build-packet -> writer-audit -> writer-sync`

### 有 brainstorm 但还没定 canon

`writer-status -> writer-canonize -> writer-build-packet`

