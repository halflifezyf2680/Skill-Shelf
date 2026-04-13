# 阶段模型

用这个模型判断项目或章节当前处于什么状态。

## 项目状态

- `idea`
  - 灵感、片段、模糊想法
  - 允许冲突，不要求自洽

- `candidate`
  - 已经形成候选方案
  - 可以比较、取舍、做决策

- `protocol-ready`
  - 项目已经确定数据结构维护协议
  - truth surface 与查询面的结构语义已定义
  - 还未必写入足够 canon

- `canon`
  - 已采纳事实
  - 可被后续章节正式引用

- `chapter-ready`
  - 相关设定、角色参数、时间锚点已经足够
  - 可以开始组装章节执行包

- `drafting`
  - 正在根据章节执行包生成正文

- `reviewed`
  - 草稿已经过审校
  - 但还没处理是否回写 canon

- `synced`
  - canon delta 已处理完成
  - truth surface 与正文重新闭环

## 阶段推进逻辑

- `idea -> candidate`
  - 当想法清晰到可以和别的方案比较时

- `candidate -> canon`
  - 当项目还很轻，且尚未进入结构化维护时可直接发生

- `candidate -> protocol-ready`
  - 当项目准备正式进入结构化维护，需要先冻结协议时

- `protocol-ready -> canon`
  - 当第一批正式事实已经按协议写入 truth surface 时

- `canon -> chapter-ready`
  - 当章节功能、POV、时间锚点、相关设定都已具备

- `chapter-ready -> drafting`
  - 当正式 chapter packet 已存在

- `drafting -> reviewed`
  - 当 draft audit 已完成

- `reviewed -> synced`
  - 当 canon delta 已回写、拒绝或转存完成

## 最小核对问题

- 当前项目的 truth surface 是什么？
- 当前项目的数据结构维护协议是否已经冻结？
- 当前内容是探索性的，还是有约束力的？
- 这章在不补新 canon 的情况下，是否真的可以写？
- 这次新 prose 是否改变了未来章节必须知道的事实？
