# Skill Hub Manual

## 1. 概述

Skill Hub 是一个本地 MCP（Model Context Protocol）服务器，为 LLM 提供专业技能的按需加载。

**核心解决的问题：** 传统做法把每个 skill 的 description 常驻在 context 里，200 个 skill 就要占用数万 tokens。Skill Hub 只注册 14 个轻量工具到 context，skill 正文全部存放在本地包仓库，LLM 需要时通过路由链按需加载。

**技术栈：** TypeScript + MCP SDK + chokidar（文件监听） + gray-matter（frontmatter 解析） + zod（参数校验）。

## 2. 架构总览

```
┌─────────────────────────────────────────────────────┐
│                   MCP Client (LLM)                   │
│  只看到 14 个工具定义，不加载任何 skill 正文           │
└────────────────────┬────────────────────────────────┘
                     │ stdio
┌────────────────────▼────────────────────────────────┐
│              server.ts (McpServer)                   │
│  注册工具、连接 transport、启动 watcher               │
└────────────────────┬────────────────────────────────┘
                     │
         ┌───────────┼───────────┐
         ▼           ▼           ▼
   ┌──────────┐ ┌────────┐ ┌────────┐
   │ registry  │ │ groups │ │staging │
   │ 索引+搜索 │ │ 组目录  │ │ 导入   │
   └──────────┘ └────────┘ └────────┘
         │
         ▼
   ┌──────────────────────────┐
   │    data/hub/packages/    │
   │  skill 正文（SKILL.md）   │
   └──────────────────────────┘
```

### 源码结构

```
src/
  server.ts              # 入口：MCP 服务器、工具注册、启动
  config.ts              # 配置加载、存储路径解析
  types.ts               # 所有类型定义
  registry/
    registry.ts          # 核心：索引构建、搜索、CRUD
    fuzzy.ts             # 模糊匹配工具（Levenshtein）
  groups/
    group-catalog.ts     # 组目录管理、skill 自动分组
  parser/
    skill-parser.ts      # SKILL.md 解析、关键词提取
  watcher/
    watch.ts             # 文件监听、热重载
  staging/
    staging.ts           # 导入候选收集与修复
  tools/
    search-skills.ts     # search_skills 工具
    list-groups.ts       # list_skill_groups 工具
    list-group-skills.ts # list_group_skills 工具
    read-skill.ts        # read_skill 工具
    install-skills.ts    # install_skills 工具
    create-skill.ts      # create_skill 工具
    validate-skills.ts   # validate_skills 工具
    get-hub-status.ts    # get_hub_status 工具
    create-group.ts      # create_group 工具
    update-group.ts      # update_group 工具
    delete-group.ts      # delete_group 工具
    list-staging-candidates.ts   # list_import_candidates 工具
    read-staging-candidate.ts    # read_import_candidate 工具
    write-repaired-skill.ts      # write_repaired_import 工具
```

## 3. 路由协议

这是 Skill Hub 最核心的设计——渐进式披露（progressive disclosure）。

### 完整流程

```
LLM 收到用户任务
      │
      ▼
search_skills(query)     ← 尝试快速定位
      │
      ├─ 有结果 ──→ 检查 directMatch
      │                 │
      │                 ├─ directMatch 存在 → 读 description，判断是否对路
      │                 │    ├─ 对路 → read_skill(skill)
      │                 │    └─ 不对路 → 阅读返回的 groupDescription，重新选
      │                 │
      │                 └─ 无 directMatch → 阅读返回的 groupDescription，选 group
      │
      └─ 无结果 ──→ list_skill_groups()
                         │
                         ▼
                  阅读所有 groupDescription，选定 group
                         │
                         ▼
                  list_group_skills(group)
                         │
                         ▼
                  看 skill 名称，选定目标
                         │
                         ▼
                  read_skill(skill)  → 加载完整正文
```

### 关键设计决策

1. **search_skills 是辅助，不是必须入口。** 搜不到就去 list_skill_groups 看所有组描述，这是标准兜底路径。

2. **directMatch 防 LLM 跳步。** 当 query 命中某个 skill 的名称时，搜索结果附带该 skill 的 description。LLM 可以直接在搜索结果内评估，不需要先调 read_skill 再判断。

3. **一次只读 1 个 skill。** read_skill 返回完整正文后，LLM 先评估是否足够。不够再补读，禁止一次调多个 read_skill。

4. **语义判断由 LLM 完成。** 搜索引擎只做 token 过滤，不评分不排序。相关性和精确度由 LLM 判断。

## 4. 搜索机制

### search_skills 的工作方式

**输入：** 一个 query 字符串（来自用户任务）。

**处理：**
1. 将 query 转小写，按非字母数字字符拆分为 token（长度 >= 2）
2. 对每个 group，检查是否有 token 命中：
   - group 的 `groupDescription`（包含 group 名称）
   - 该 group 内任何 skill 的 `skillName`
3. 命中则返回该 group

**匹配条件：** token 的 `includes` 检查。不做词根还原，不做词形变换。

**directMatch 检测：** 当 query 命中某个 group 后，额外检查该组内 skill 名称是否与 query 匹配：
- 精确匹配：`skillName === query` 或 `skillName.includes(query)` 或反之
- 模糊匹配：Levenshtein 距离比率 <= 0.3（解决 LLM 编造 query 时差一两个字的问题）

directMatch 命中时，返回该 skill 的 description 供 LLM 快速评估。

**不做什么：** 不评分、不排序、不做语义嵌入。这些交给 LLM。

### skill 自动分组

安装 skill 时，`skill-parser` 解析 SKILL.md 的 frontmatter（`name` + `description`），从 description 中提取关键词，然后 `group-catalog` 用加权匹配把 skill 分配到最相关的组：

- `keywords` 命中：权重 4
- `aliases` 命中：权重 3
- `group id` 片段命中：权重 2
- `groupDescription` 片段命中：权重 1

无法匹配任何组的 skill 归入 `specialized-domain`。

## 5. 工具清单（14 个）

### 只读工具（8 个）

| 工具 | 用途 | 输入 |
|------|------|------|
| `search_skills` | 按关键词搜索 group | `query`, `limit?` |
| `list_skill_groups` | 列出所有组及描述 | 无 |
| `list_group_skills` | 列出组内 skill | `group` |
| `read_skill` | 读取 skill 完整正文 | `skill`（名称或 ID） |
| `validate_skills` | 校验 skill 完整性 | `skill?`（可选） |
| `get_hub_status` | 查看整体状态 | 无 |
| `list_import_candidates` | 列出待审查候选 | `status?`, `limit?` |
| `read_import_candidate` | 查看候选详情 | `id` |

### 写工具（6 个）

| 工具 | 用途 | 输入 |
|------|------|------|
| `install_skills` | 批量安装 skill 包 | `sourcePath` |
| `create_skill` | 创建新 skill | `name`, `description`, `skillMarkdown` |
| `create_group` | 创建新组 | `group`, `groupDescription`, `keywords?`, `aliases?` |
| `update_group` | 更新组定义 | `group`, `newGroup?`, `groupDescription?`, `keywords?`, `aliases?` |
| `delete_group` | 删除空组 | `group` |
| `write_repaired_import` | 提交修复后的候选 | `id`, `skillMarkdown`, `targetId?`, `notes?` |

### 工具分类规则

**只读工具**可以随时调用。**写工具**调用前必须先完成必要的只读检查——比如删除组前确认组内没有 skill。

## 6. 存储结构

```
data/hub/
├── config/
│   └── groups.json              # 组目录（16 内置 + 自定义）
├── packages/
│   └── {skill-id}/
│       ├── SKILL.md             # skill 正文（必须）
│       ├── meta.json            # 自动生成的元数据
│       ├── references/          # 可选参考文件
│       └── assets/              # 可选资源文件
├── staging/
│   ├── imports/                 # 待审查的导入候选
│   └── repaired/                # 已修复的候选
├── index/                       # 运行时自动维护
│   ├── group-list.json
│   ├── groups/{group}.json
│   └── skills/{skill-id}.json
└── logs/                        # 运行时日志
```

可通过 `SKILL_HUB_ROOT` 环境变量覆盖根目录。

## 7. Skill 包格式

每个 skill 是一个目录，必须包含 `SKILL.md`：

```markdown
---
name: my-skill
description: 这个 skill 做什么
---

# My Skill

Skill 正文内容...
```

**必填 frontmatter 字段：**
- `name`：skill 显示名称（至少 1 字符）
- `description`：skill 描述（至少 1 字符，用于搜索索引和自动分组）

**可选目录：**
- `references/`：参考文件，read_skill 时一并返回
- `assets/`：资源文件，read_skill 时一并返回

**自动生成：**
- `meta.json`：包含 skillId、skillName、description、group、keywords、updatedAtMs、status

## 8. 组体系

### 内置 16 个组

| 组 ID | 描述 |
|-------|------|
| `engineering` | 软件工程、系统架构、前后端实现、数据与 AI 工程 |
| `design` | UI/UX、品牌、视觉系统、设计语言 |
| `product` | 产品规划、需求分析、用户反馈、路线图 |
| `project-management` | 项目推进、任务协调、里程碑、交付 |
| `marketing` | 品牌营销、内容营销、增长叙事 |
| `paid-media` | 广告投放、媒介优化、买量 |
| `sales` | 销售、售前、线索管理、成交推进 |
| `finance` | 财务、预算、核算、定价 |
| `legal-compliance` | 法务、合规、审计、政策、风险 |
| `hr-talent` | 招聘、人才、培训、组织发展 |
| `support-operations` | 客服、运营支持、行政协助 |
| `supply-chain` | 供应链、采购、物流、库存 |
| `academic-research` | 学术研究、学习规划、人文社科 |
| `testing-qa` | 测试、质量保障、评估、验证 |
| `spatial-gaming` | 游戏开发、空间计算、XR、3D |
| `specialized-domain` | 无法归入其他组的垂直领域（兜底） |

### 自定义组

通过 `create_group` 创建，通过 `update_group` 修改。内置组不可删除，自定义组在组内无 skill 时可删除。

## 9. 导入管线

用于将外部 skill 文件审查后纳入正式包仓库。

```
外部 skill 文件
      │
      ▼
放入 staging/imports/
      │
      ▼
list_import_candidates()     ← LLM 查看待审查列表
      │
      ▼
read_import_candidate(id)    ← 查看候选内容和问题
      │
      ▼
LLM 修复内容
      │
      ▼
write_repaired_import()      ← 提交修复到 staging/repaired/
```

候选状态：
- `ready`：frontmatter 合法，可审查
- `review_required`：需要人工审查（如 raw markdown 需转 package 格式）
- `blocked`：缺少必要字段
- `repaired`：已提交修复版本

## 10. 热重载

启动时自动监听 `packages/` 目录（基于 chokidar）。

- **新增** SKILL.md：自动解析、索引、写入元数据
- **修改** SKILL.md：自动重新解析、更新索引和元数据
- **删除** SKILL.md：自动从索引中移除（需 `syncDelete` 策略开启）

无需重启 MCP 服务器。索引更新是单文件粒度的——只刷新变更的那个 skill。

## 11. 校验（validate_skills）

检测以下问题：

| 问题代码 | 严重级别 | 说明 |
|----------|----------|------|
| `missing_skill_file` | blocked | 目录存在但 SKILL.md 缺失 |
| `invalid_frontmatter` | blocked | frontmatter 缺少 name 或 description |
| `duplicate_skill_name` | review_required | 多个 skill 使用相同 name |
| `generic_group` | review_required | skill 被分到 specialized-domain 兜底组 |

对 `review_required` 项的处理建议：先 read_skill 了解内容，再对照 list_skill_groups 判断是否能归入更精确的组。能稳定命中就改组，否则保留在 specialized-domain。不要因为单个 skill 轻易新建 group。

## 12. 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `SKILL_HUB_ROOT` | `<package>/data/hub` | 数据根目录 |
| `SKILL_ROUTER_SEARCH_LIMIT` | `8` | search_skills 默认返回上限 |
| `SKILL_ROUTER_MAX_KEYWORDS` | `12` | 每个 skill 自动提取的最大关键词数 |
| `SKILL_ROUTER_MAX_RELATED_SKILLS` | `5` | read_skill 返回的最大关联 skill 数 |
| `SKILL_ROUTER_WATCH` | `1` | 是否启用文件监听 |
| `SKILL_ROUTER_WATCH_USE_POLLING` | `0` | 是否使用轮询（而非原生 fs 事件） |
| `SKILL_ROUTER_WATCH_INTERVAL_MS` | `100` | 轮询间隔 |
| `SKILL_ROUTER_WATCH_STABILITY_MS` | `300` | 写入稳定等待时间 |
| `SKILL_ROUTER_WATCH_SYNC_DELETE` | `1` | 删除 SKILL.md 时是否同步移除索引 |

## 13. 安装与运行

```bash
npm install
npm run skill-hub
```

### MCP 配置示例（Claude Desktop）

```json
{
  "mcpServers": {
    "skill-hub": {
      "command": "npm",
      "args": ["run", "skill-hub"],
      "cwd": "/path/to/skill-hub"
    }
  }
}
```

### Claude Code 配置示例

在 `~/.claude.json` 的 `mcpServers` 中添加：

```json
{
  "skill-hub": {
    "command": "npm",
    "args": ["run", "skill-hub"],
    "cwd": "/path/to/skill-hub"
  }
}
```

## 14. 常见操作

### 安装一个 skill 包

```
1. install_skills(sourcePath="/path/to/skill-package")
2. validate_skills()  ← 确认安装正确
3. 如有 review_required → 检查是否需要调整分组
```

### 创建一个新 skill

```
1. create_skill(name, description, skillMarkdown)
2. 自动完成分组、索引、元数据生成
```

### 审查导入候选

```
1. list_import_candidates(status="review_required")
2. read_import_candidate(id="...")
3. 分析问题，修复内容
4. write_repaired_import(id, skillMarkdown)
```

### 查看系统状态

```
get_hub_status()
→ 返回：组数、skill 数、导入候选数、索引更新时间、watcher 状态、issue 数
```
