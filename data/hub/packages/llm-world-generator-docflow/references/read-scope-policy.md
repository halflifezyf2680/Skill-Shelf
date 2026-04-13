# 渐进披露读取策略

## 总原则

默认不要一次性把全部 yaml 文件塞给 LLM。

先确定当前 run 的 scope，再决定读取范围。读取量应该随着 scope 变小而缩小。

## 全局最小必读文件

无论什么模式，默认都先读：

- `00-intake/world-brief.yaml`
- `02-canon/world-intent.yaml`
- `03-skeleton/world-skeleton.yaml`
- `03-skeleton/branch-index.yaml`
- 当前 run 文件

必要时再读：

- `03-skeleton/generation-order.yaml`
- `06-audit/issues.md`

## `full` 模式

适用：

- 首轮建世界骨架
- 跨多个 branch 的全局重构

默认追加读取：

- `05-world-data/world/*.yaml`
- 目标 branch 的现有 `branch.yaml`
- 相关 branch 的现有 `leaves/*.yaml` 摘要
- `06-audit/issues.md`

禁止：

- 在 full 模式里无脑重写全部历史文件而不保留稳定 id

## `branch` 模式

适用：

- 扩写一个 branch
- 修订一个 branch 的结构

默认追加读取：

- 目标 branch 的 `branch.yaml`
- 目标 branch 的直接 leaf 文件
- `branch-index.yaml` 中该 branch 的 `depends_on`
- 依赖 branch 的最小摘要文件
- 相关 audit 问题

默认不读：

- 不相关 branch 的全部 leaf
- 整个 `05-world-data`

## `leaf` 模式

适用：

- 扩写一个具体局部节点
- 修补一个明确的小事实单元

默认追加读取：

- 父 branch 的 `branch.yaml`
- 目标 leaf 自身文件
- 同 branch 下极少量直接邻近 leaf，仅在存在强依赖时才读
- 直接相关 audit 问题

默认不读：

- 同 branch 的所有 leaf
- 其他 branch 的完整数据
- 整个世界时间线或势力全集

## 写入规则

读取范围和写入范围必须联动：

- `full` 可以写多个 branch，但只写 run 声明的 `write_targets`
- `branch` 只写目标 branch 及其子叶文件
- `leaf` 只写目标 leaf，必要时最多补写父 branch 摘要

如果发现需要越权写入，先升级 run，而不是偷偷改文件。
