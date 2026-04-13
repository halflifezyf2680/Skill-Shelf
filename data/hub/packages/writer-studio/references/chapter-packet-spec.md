# Chapter Packet 规范

`chapter packet` 是 AI 主写前的章节执行包。

它的作用不是"写个摘要"，而是把 canon 压成一个受约束、可执行、可审校的写作面。

## 用途

- 防止裸写
- 降低设定漂移
- 让 draft audit 有可比对基线
- 让 canon sync 有明确输入来源

## 最小字段

每个正式 chapter packet 至少要有：

- `chapter_id`
- `arc_or_volume`
- `chapter_function`
- `pov`
- `time_anchor`
- `location`
- `prior_state`
- `required_beats`
- `forbidden_moves`
- `required_references`
- `character_constraints`
- `style_constraints`
- `exit_state`

## 字段解释

- `chapter_id`
  - 这个章节单元的稳定 id

- `arc_or_volume`
  - 它属于哪一卷、哪一线、哪一段结构

- `chapter_function`
  - 这章为什么存在
  - 不写这个字段，这章就很容易空转

- `pov`
  - 谁控制叙述边界

- `time_anchor`
  - 这章发生在时间线上的什么位置

- `location`
  - 这章主要发生在哪里

- `prior_state`
  - 开章前必须已经成立的事实

- `required_beats`
  - 本章必须完成的动作、冲突、推进点

- `forbidden_moves`
  - 本章不能发明、提前透露、破坏或跳过的内容

- `required_references`
  - 本章依赖的 canon 节点、文件或事实来源

- `character_constraints`
  - 角色的语气、动机、知识边界、行为边界

- `style_constraints`
  - 本章需要服从的风格参数与语言约束

- `exit_state`
  - 本章结束时必须成立的状态

## 字段强化规范

### style_constraints 必须包含

`style_constraints` 不能只写"保持一致"或"文学风格"等模糊描述。必须包含：

1. **文风正反例**
   - 正例：本章应该像什么样的文字（引用已有章节的具体段落或风格描述）
   - 反例：本章不应该出现的写法（与正例对比）

2. **情感密度下限**
   - 每多少字至少出现一个"人的时刻"（角色有真实情感反应的瞬间）
   - 示例：`每 800 字至少 1 个情感锚点`
   - 如果本章是高强度情感章节，密度要求应上调

3. **禁止句式模式**
   - 列出本章不允许的句式或修辞模式
   - 示例：
     - `禁止连续 3 句以上以相同结构开头`
     - `禁止用"他知道/她感到"直接陈述情绪，必须通过行为或环境映射`
     - `禁止为营造节奏感而刻意断句（主语和谓语之间无理由拆行）`

### forbidden_moves 模板

`forbidden_moves` 不能留空或只写"不违反 canon"。必须预填常见踩坑项，并根据本章实际情况增删：

```
## 通用预填（默认全部生效）

- [ ] 元叙述越界：不允许作者声音直接介入叙述
- [ ] 排查日志化：不允许把调查过程写成机械的清单或步骤汇报
- [ ] 对话纯功能化：对话必须携带性格信息或关系张力，不允许仅用于传递情报
- [ ] 刻意碎句：不允许为了"文学感"而人为拆碎完整语义
- [ ] 情节回顾：不允许角色在不自然的场合回忆或复述读者已知的信息

## 本章特定

- [ ] （根据本章 canon 补充，如：不允许角色 X 提前知道角色 Y 的真实身份）
- [ ] （如：不允许出现尚未发明的技术细节）
```

### character_constraints 必须包含

`character_constraints` 不能只写角色名字和简介。本章涉及的主要角色必须包含：

1. **知识边界**：该角色在当前时间点知道什么、不知道什么
2. **情感状态**：进入本章时的情绪基调和预期变化
3. **对话行为规则**：
   - 该角色说话的特征（用词习惯、句式偏好、口头禅）
   - 该角色在本章中与其他角色的关系动态
   - **强制要求**：该角色与其他角色的对话中，至少 30% 的对话行必须携带非功能信息（性格投射、情绪外溢、潜台词、关系暗示等）

## Packet 就绪规则

如果某个字段缺失，而这章又确实依赖它，那就先停下来补 canon，不要硬写。

不要用"模型自己会从上下文推断出来"来代替 packet 字段。

`style_constraints` 如果缺少文风正反例、情感密度下限或禁止句式模式中的任何一项，视为不完整，不允许进入 drafting。

`forbidden_moves` 如果通用预填项被删除且未给出理由，视为不完整。

## 可选字段

如果项目复杂度需要，可以增加：

- `scene_list`
- `object_flow`
- `knowledge_snapshot`
- `unresolved_tension`
- `canon_sync_risks`

## 推荐构建顺序

1. 先锁定章节目标
2. 再锁定章节功能
3. 再锁定 POV、时间、地点
4. 再收集必需 canon 引用
5. 再定义 required beats 和 forbidden moves
6. 再挂角色约束与风格约束
7. 最后定义 exit state
8. 必要时补风险提示

## Drafting 规则

正文必须从 packet 出发写，不要从模糊记忆直接写。

如果 prose 必须越过 packet 才成立，就说明 packet 或 canon 需要先被修正。

## 审校关系

packet 不只是写前输入，它也是写后审校基线。

审校至少要反问：

- 这章是否完成了声明的功能？
- 是否始终守住 POV 边界？
- 是否违反了 forbidden moves？
- 是否到达了声明的 exit state？
- style_constraints 中的禁止句式模式是否被遵守？
- 情感密度是否达到下限？
- 对话非功能信息比例是否达标？
