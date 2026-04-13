# 项目脚本规划示例

- 项目名：蓝色星球（示例）
- truth surface：`canon/*.yaml`
- YAML 主维护面位置：`canon/`
- 数据库查询面位置：`build/story.db`

- 扫描脚本：`scripts/scan_truth_surface.py`
  - 读取 `canon/*.yaml`
  - 抽取实体、关系、事件、知情状态、章节断言

- 校验脚本：`scripts/validate_truth_surface.py`
  - 检查字段完整性
  - 检查 id 唯一性
  - 检查引用是否存在
  - 检查枚举值是否合法

- 落库脚本：`scripts/compile_to_db.py`
  - 将 YAML 结构编译到 `build/story.db`
  - 不允许 ad hoc 写入

- 查询脚本：`scripts/query_story.py`
  - 支持树查询、链路查询、关系查询、知情状态查询

- 视图构建脚本：`scripts/build_views.py`
  - 从数据库生成树状视图
  - 生成 chapter packet 输入
  - 生成审校输入

- 编译顺序：
  1. 作者与 LLM 更新 YAML
  2. 扫描
  3. 校验
  4. 落库
  5. 构建视图

- 稳定 id 规则：
  - 角色：`char.*`
  - 地点：`place.*`
  - 文书 / 物件：`doc.*` / `obj.*`
  - 事件：`evt.*`
  - 关系：`rel.*`
  - 章节断言：`ch.*`

- 主表 / 子表约定：
  - 主表：`entities`, `events`, `relations`, `knowledge_states`, `chapter_assertions`
  - 子表 / 衍生表：由项目自定，但字段语义必须与 YAML 同构

- 查询能力清单：
  - 某时点谁知道什么
  - 某时点谁持有什么
  - 某关系在某阶段是否成立
  - 某章引入了什么
  - 某事件的前后链是什么
  - 当前有哪些 continuity 风险

