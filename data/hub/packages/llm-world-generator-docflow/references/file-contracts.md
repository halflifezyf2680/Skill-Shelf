# 文件契约

## `00-intake/world-brief.yaml`

用途：

- 世界生成的最小有效起点

必须回答：

- 这个世界叫什么
- 这是什么类型的世界
- 第一轮生成应该产出什么

最小质量要求：

- `one_sentence_goal` 不能只是风格描述
- `preferred_first_scope` 必须明确 `full`、`branch` 或 `leaf`

## `01-brainstorm/brainstorm.md`

用途：

- 候选方向的推理面

应包含：

- 3 到 5 条真正可分流的方向
- 每条路线的世界主引擎
- 每条路线的扩展强项
- 每条路线的风险
- 每条路线的泛化风险
- 推荐理由与淘汰理由

## `01-brainstorm/candidates.yaml`

用途：

- 候选方向的结构化短名单

每个 candidate 应包含：

- `candidate id`
- 摘要
- `world_engine`
- `expansion_fit`
- `genericity_risk`
- 优势
- 风险
- 保留/淘汰建议

## `01-brainstorm/reusable-fragments.md`

用途：

- 保存被淘汰路线中仍然值得复用的局部机制和片段

## `02-canon/world-intent.yaml`

## `02-canon/world-intent.yaml`

用途：

- 最终选定的单一路线

必须包含：

- 世界承诺
- 必须具备的特征
- 绝不能出现的特征
- 优先展开的首批分支
- 主路线来源
- 可吸收片段来源

## `03-skeleton/world-skeleton.yaml`

用途：

- 世界的高层图谱

必须包含：

- 世界核心
- 世界规则
- 主要地理分支
- 主要势力分支
- 主要时间锚点
- 可选的生态/文化/资源分支

## `03-skeleton/branch-index.yaml`

用途：

- 为所有分支提供稳定 id

每个 branch 应包含：

- `branch id`
- `branch title`
- `branch type`
- `parent id`
- `generation depth`
- `status`
- `output_path`
- `depends_on`

## `04-runs/*.yaml`

用途：

- 显式的生成契约

每个 run 都必须包含：

- `run id`
- `mode`
- `target scope`
- `source skeleton refs`
- `required_inputs`
- `optional_inputs`
- `write_targets`
- `stop conditions`

## `05-world-data/index.md`

用途：

- 给人类读者提供最终世界事实入口

## `05-world-data/...`

用途：

- 结构化世界事实

规则：

- 生成数据必须始终可追溯到 `branch id` 或 `leaf id`
- 每个 branch 必须有稳定 canonical output path

## `06-audit/issues.md`

用途：

- 人类可读的修复问题台账

每个 issue 应回答：

- 出了什么问题
- 问题位于哪里
- 为什么这是问题
- 它是否阻断后续继续推进
