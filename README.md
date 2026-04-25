# Skill Hub

**用 9 个工具的固定开销，按需访问 230+ 个专业技能，而不需要把任何 skill 正文常驻上下文。**

<p align="center">
  <img src="docs/diagrams/png/01-overview.png" width="270" alt="Skill Hub Overview"/>
</p>

传统做法：每个 skill 作为本地 skill 加载 → description 全量驻留 context → 230+ 个 skill = 数万 tokens 白白浪费，且每次对话都背着它们跑。

内置 230+ 个 skill 只是起步。用户可通过 `install_skills` 从目录批量安装，持续扩充自己的 skill 库。

Skill Hub 的做法：skill 全部存放在本地包仓库，context 里只有 9 个轻量工具定义。LLM 需要某个专业能力时，通过路由按需加载。不需要的时候，零开销。

## 路由协议

<p align="center">
  <img src="docs/diagrams/png/02-routing.png" width="270" alt="Routing Protocol"/>
</p>

```
search_skills(query)
  │
  ├─ 命中了 → 返回匹配的 groups（带 groupDescription + skillNames + directMatch）
  │
  └─ 没命中 → 空结果
       │
       ▼
  list_skill_groups()
       │
       ▼
  LLM 阅读所有 groupDescription，选定最相关的 group
       │
       ▼
  list_group_skills(group)
       │
       ▼
  LLM 看 skill 名称，选定目标
       │
       ▼
  read_skill(skill) → 加载完整 skill 正文
```

search_skills 是辅助过滤，不是必须入口。搜不到就去 list_skill_groups 看所有组描述，这是标准路径。

### search_skills 搜索机制

纯 token 匹配，不评分不排序。对每个 group，检查 query token 是否命中：
- group ID、`groupDescription`、`keywords`、`aliases`
- 该 group 内任何 skill 的 `skillName`

关键词为中英双语（如 engineering 同时有 "software"、"编程"），支持跨语言搜索。语义判断由 LLM 完成，不由算法代劳。

### directMatch

当 query 精确匹配某个 skill 的 `skillName` 或 `skillId` 时，搜索结果会额外附带 `directMatch`，包含 skill 的 `description`。LLM 可以在搜索结果内直接评估是否对路，防止盲目跳步。

## 存储结构

<p align="center">
  <img src="docs/diagrams/png/04-storage.png" width="270" alt="Storage Structure"/>
</p>

```
data/hub/
  config/groups.json           # 组目录（16 个内置组 + 自定义组）
  packages/{skill-id}/SKILL.md    # skill 正文（必须）
  packages/{skill-id}/meta.json   # 自动生成的元数据
  packages/{skill-id}/references/    # 可选参考文件
  packages/{skill-id}/assets/        # 可选资源文件
  staging/imports/            # 待审查的导入候选（运行时）
  staging/repaired/           # 已修复的候选（运行时）
  index/                       # 索引文件（运行时自动维护）
  logs/                        # 运行时日志
```

可通过环境变量覆盖根目录：

```bash
SKILL_HUB_ROOT=/your/custom/path
```

## Skill 包格式

每个 skill 是一个包含 `SKILL.md` 的目录：

```markdown
---
name: my-skill
description: 这个 skill 做什么
---

# My Skill

Skill 正文内容...
```

`name` 和 `description` 是必填的 frontmatter 字段，用于搜索索引和组分类。

## 工具清单（9 个）

<p align="center">
  <img src="docs/diagrams/png/03-tools.png" width="270" alt="Tool List"/>
</p>

### 路由（只读）

| 工具 | 用途 |
|------|------|
| `search_skills` | 按关键词搜索 group（同时匹配组描述和 skill 名称） |
| `list_skill_groups` | 列出所有组及描述 |
| `list_group_skills` | 列出组内所有 skill 名称和关键词 |
| `read_skill` | 读取 skill 完整正文、资源、参考文件 |
| `validate_skills` | 校验所有 skill 的完整性和重复情况 |
| `get_hub_status` | 查看索引和文件监听状态 |

### 写操作

| 工具 | 用途 |
|------|------|
| `install_skills` | 从目录批量安装 skill 包 |
| `create_skill` | 创建新 skill |
| `manage_group` | 创建/更新/删除组（mode: create/update/delete） |

## 组体系

<p align="center">
  <img src="docs/diagrams/png/05-classification.png" width="270" alt="Classification"/>
</p>

内置 16 个组，覆盖主要专业领域：

`engineering` · `design` · `product` · `project-management` · `marketing` · `paid-media` · `sales` · `finance` · `legal-compliance` · `hr-talent` · `support-operations` · `supply-chain` · `academic-research` · `testing-qa` · `spatial-gaming` · `specialized-domain`

skill 在索引时按关键词加权匹配自动分配到最相关的组。无法匹配任何组的 skill 归入 `specialized-domain`。

## 热重载

启动时自动监听 `packages/` 目录变更，新增、修改、删除 skill 后索引自动更新，无需重启。

## 安装

### npx 一键启动（推荐）

无需克隆仓库，直接配置 MCP Server 即可：

```json
{
  "mcpServers": {
    "skill-hub": {
      "command": "npx",
      "args": ["-y", "skill-router-mcp"]
    }
  }
}
```

npx 会自动下载并运行，首次稍慢，后续走缓存。

### 从源码安装

```bash
git clone https://github.com/halflifezyf2680/skill-hub.git
cd skill-hub
npm install
```

## 配置 MCP Server

### Claude Code

在 `~/.claude.json` 的 `mcpServers` 中添加：

```json
{
  "mcpServers": {
    "skill-hub": {
      "command": "npx",
      "args": ["-y", "skill-router-mcp"]
    }
  }
}
```

从源码安装时使用：

```json
{
  "mcpServers": {
    "skill-hub": {
      "command": "npm",
      "args": ["run", "skill-hub"],
      "cwd": "/your/path/to/skill-hub"
    }
  }
}
```

### Claude Desktop

在 Claude Desktop 的 `claude_desktop_config.json` 中添加：

```json
{
  "mcpServers": {
    "skill-hub": {
      "command": "npx",
      "args": ["-y", "skill-router-mcp"]
    }
  }
}
```

配置完成后重启客户端即可。

## 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `SKILL_HUB_ROOT` | `<package>/data/hub` | 数据根目录 |
| `SKILL_ROUTER_SEARCH_LIMIT` | `8` | search_skills 默认返回上限 |
| `SKILL_ROUTER_MAX_KEYWORDS` | `12` | 每个 skill 自动提取的最大关键词数 |
| `SKILL_ROUTER_MAX_RELATED_SKILLS` | `5` | read_skill 返回的最大关联 skill 数 |
| `SKILL_ROUTER_WATCH` | `1` | 是否启用文件监听 |

## 致谢

Skill 内容来源于以下开源项目：

- [agency-agents-zh](https://github.com/jnMetaCode/agency-agents-zh)（MIT License）— 211 个中文 AI 专家智能体
- [awesome-design-md](https://github.com/VoltAgent/awesome-design-md)（MIT License）— 品牌设计系统 markdown 文件
- [superpowers](https://github.com/obra/superpowers)（MIT License）— AI 编程工作流技能框架
