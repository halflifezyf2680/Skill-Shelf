---
name: llm-world-generator-docflow
description: 用纯 markdown/yaml 文档流构建和运行 LLM 世界生成器。当用户要做世界观生成、骨架优先的 worldbuilding、分支或叶子级扩写、目录驱动的生成状态管理，或想避免沉重编辑器 UI 时，务必使用这个 skill。优先采用显式 run 文件、稳定 branch id / leaf id、渐进披露读取范围，以及把最终世界事实沉淀到 05-world-data，而不是一次性大量读取和自由散写。
---

# LLM 世界生成器文档流

这个 skill 用于构建或运行一种由文件和文件夹控制工作流、而不是依赖复杂 UI 的世界生成器。

## 核心判断

- 把目录树视为工作流状态机。
- 把 `md` 和 `yaml` 视为唯一的一手主输出。
- 把 `world name` 只当作命名信息，绝不能把它当成足以启动生成的输入。
- 把 `world skeleton` 视为头脑风暴与具体世界数据之间的中枢对象。
- 把 `05-world-data` 视为最终世界事实层；其余阶段目录主要服务于生成与治理。

## 何时使用

当用户需要以下任一能力时，使用这个 skill：

- 由 LLM 驱动，而不是靠手工编辑推进的世界生成器
- 骨架优先的世界构建工作流
- 基于同一套底层骨架，既支持整体现世生成，也支持局部分支生成
- 基于 markdown/yaml 的工作流控制方式
- 用目录驱动生产状态，而不是做一个控制台式仪表盘
- 作为重编辑器式世界构建系统的替代方案

## 产品立场

不要设计一个沉重的“总控室”。

默认用户流程必须是：

1. 定义世界目标
2. 头脑风暴多个方向
3. 锁定规范意图
4. 生成世界骨架
5. 选择整体现世生成或局部分支生成
6. 写入结构化世界数据
7. 审计并迭代

可以有调试产物，但它们绝不能变成主工作流本身。

## 读者与操作者分层

这套系统默认服务两类对象：

- 人类读者：默认主要查看 `05-world-data`，必要时通过 `WORLD.md` 或 `05-world-data/index.md` 进入。
- 生成操作者或 LLM：使用 `00` 到 `06` 全套目录进行规划、生成、修订与审计。

不要要求普通阅读者先理解 docflow 才能看世界内容。

## 规范阶段

### 阶段 0：输入建档

目标：

- 建立最小可用的起始输入

必需输出：

- `00-intake/world-brief.yaml`

最小字段：

- 世界名称
- 一句话世界目标
- 可选的类型/语气标签
- 硬约束
- 禁忌项
- 首轮生成的偏好范围

质量门槛：

- `world goal` 必须描述“要生成什么”，不能只是世界名称或一句空泛风格词。
- `preferred_first_scope` 必须明确首轮是 `full`、`branch` 还是 `leaf`。

如果缺少 `world goal`，必须停止，禁止启动生成。

### 阶段 1：头脑风暴

目标：

- 在不提前定稿的前提下探索多个方向

必需输出：

- `01-brainstorm/brainstorm.md`
- `01-brainstorm/candidates.yaml`
- `01-brainstorm/reusable-fragments.md`

规则：

- 至少产出 3 条、至多 5 条真正可分流的路线。
- 候选方向必须在同一组维度上比较，而不是随意写灵感段落。
- 不要把四个不同强调点伪装成四条路线。
- 此阶段不要写世界数据。

推荐比较维度：

- 世界主引擎
- 当代主要压力
- 地图结构驱动力
- 势力结构驱动力
- 历史或谜团驱动力
- 扩展潜力
- 泛化风险

### 阶段 2：意图定稿

目标：

- 将头脑风暴收敛为一个单一的意图包

必需输出：

- `02-canon/world-intent.yaml`
- `02-canon/open-questions.md`

规则：

- 只能选择 1 条主路线进入 canon。
- 其他路线只能作为“可吸收碎片”进入 canon，不能直接并列成为第二主路线。
- 记录仍然刻意保留未知的部分。
- 保持简短、可执行。

### 阶段 3：骨架

目标：

- 在进入细节生成前，先定义世界的高层对象图

必需输出：

- `03-skeleton/world-skeleton.yaml`
- `03-skeleton/branch-index.yaml`
- `03-skeleton/generation-order.yaml`

规则：

- 骨架必须命名世界的主要分支。
- 骨架必须足以支撑整体现世生成或局部生成。
- 不要把骨架埋进大段说明文字里。
- 每个 branch 都必须在 `branch-index.yaml` 中有稳定 id 和明确输出落点。

### 阶段 4：运行规划

目标：

- 明确下一轮生成应如何运行

允许模式：

- 整体现世生成
- 分支生成
- 叶子生成

必需的 run 文件：

- `04-runs/full-run.template.yaml`
- `04-runs/branch-run.template.yaml`
- `04-runs/leaf-run.template.yaml`

规则：

- 每一轮生成都必须显式声明目标范围。
- 禁止“模糊地继续往下写”。
- 每个 run 都必须回指 skeleton 中的 branch id。
- 每个 run 都必须声明最小必读输入和允许写入的输出。

### 阶段 5：世界数据生成

目标：

- 在稳定路径下写入具体的世界数据

必需输出族：

- `05-world-data/index.md`
- `05-world-data/world/*.yaml`
- `05-world-data/branches/<branch-id>/*.yaml`
- `05-world-data/branches/<branch-id>/leaves/<leaf-id>.yaml`

规则：

- 整体现世生成会写入多个分支。
- 分支生成只写入一个分支。
- 叶子生成只写入一个明确的局部节点。
- 所有生成结果都必须与 `branch-index` 和 skeleton id 保持兼容。
- 人类默认阅读入口应落在 `05-world-data`。

### 阶段 6：审计

目标：

- 发现冲突、不完整和错误漂移

必需输出：

- `06-audit/issues.md`
- 可选的 issue 专用 yaml 文件，放在 `06-audit/issues/` 下

规则：

- 生成后必须执行审计。
- 审计问题必须能映射回 branch id 或 leaf id。
- issue 必须区分阻塞与非阻塞。
- 不要把未解决冲突藏进说明文字里。

## 渐进披露读取规则

默认不要一次性读取全部 `05-world-data`。

读取策略：

- `full`：读取 `00-intake/world-brief.yaml`、`02-canon/world-intent.yaml`、`03-skeleton/*.yaml`、目标 run 文件、相关 `05-world-data/world/*.yaml`、目标 branch 现有输出、`06-audit/issues.md`。
- `branch`：读取全局最小上游文件、目标 branch 在 `branch-index.yaml` 中的记录、目标 branch 当前输出、直接依赖分支的最小摘要、相关 audit 问题。
- `leaf`：读取全局最小上游文件、目标 branch 记录、父 branch 的 `branch.yaml`、目标 leaf 当前文件、必要时才读取少量相邻 leaf，默认禁止把整个 branch 的所有 leaf 全读入。

禁止项：

- 默认禁止把整个 `05-world-data` 全量读入再生成一个小 leaf。
- 默认禁止在 `leaf` 模式中读取无关分支。
- 默认禁止在没有 run 文件的情况下“凭感觉继续扩写”。

需要细节时，阅读：

- `references/read-scope-policy.md`

## branch 到输出文件映射

每个 branch 必须有唯一、稳定的 canonical output path。

要求：

- 在 `03-skeleton/branch-index.yaml` 中为每个 branch 写明 `output_path`。
- `world_core`、`world_rules` 这类全局 branch 也必须有清晰落点，不要混在模糊的全局文件里。
- 允许 branch 输出落在 `05-world-data/world/` 或 `05-world-data/branches/<branch-id>/`，但映射必须一对一。

如果一个 branch 没有稳定输出落点，就不能算完成 skeleton 设计。

## 写入纪律

写入时遵守以下规则：

- 只写 run 文件中声明的 `write_targets`。
- 若需要改动 run 作用域之外的文件，必须先升级为新的 run 或新增 audit issue。
- 不要在一次 branch 生成中顺手改其他 branch 的事实。
- 不要把流程说明写进世界事实文件。

## 输出策略

在创建或扩展这套系统时，优先选择：

- 新目录，而不是巨大的单体文档
- branch id，而不是只给人看的标题
- 显式 run yaml，而不是只存在于聊天里的指令
- `canon intent + skeleton + run files`，而不是直接自由生成
- 人类阅读入口与生成控制面分层，而不是让所有人都先看 workflow 文件

## 参考资料

需要细节时，阅读以下文件：

- `references/directory-structure.md`
- `references/file-contracts.md`
- `references/brainstorm-rules.md`
- `references/read-scope-policy.md`
