# Skill Shelf

**上千个专业技能，只占 context 7 个工具定义。**

给 AI agent 装 skill，最痛的问题是：skill 越多，context 越胖。每个 skill 的 description 常驻上下文，几百个 skill 就是几万 tokens 白白浪费，每次对话都背着跑。开多个客户端还各跑各的进程，内存翻倍。更烦的是，装了一堆 skill 自己都记不住哪个是干嘛的、哪些真的好用哪些是花架子，最后还是全靠人去翻文档。

Skill Shelf 连这个问题一起解决了：LLM 看见 MCP 工具就会自己去查、自己去用。你不需要记住每个 skill 的内容，也不需要判断什么时候该用——LLM 遇到合适的场景自己会去搜索和加载。

Skill Shelf 的解法：skill 全部存本地仓库，context 里只有 7 个工具定义。需要时搜索加载，不需要时零开销。Rust 单例 daemon，一个进程服务所有 MCP 客户端——Claude Code、Codex、Cursor、Windsurf 同时开也只有一个后台进程。

不只是用内置的几百个 skill 和 group。工作中积累的经验、踩过的坑、反复用的工作流，都可以整理成 skill 入库——一份 Markdown 文件就是一个 skill。内置的 18 个组不够用就自己建，`manage_group` 创建自定义分组，`install_skills` 批量入库。把自己团队的 know-how 变成可复用的 skill 库。

## 安装

### 从 npm 使用（推荐）

MCP 客户端里直接用 `npx` 启动即可，不需要手动 clone 仓库：

```json
{
  "mcpServers": {
    "skill-shelf": {
      "command": "npx",
      "args": ["-y", "skill-shelf", "mcp"]
    }
  }
}
```

首次调用时 stdio shim 会自动拉起 Rust daemon。多个 MCP 客户端会共享同一个 daemon，不需要分别管理后台进程。

### 从源码开发

```bash
git clone https://github.com/halflifezyf2680/Skill-Shelf.git
cd Skill-Shelf
npm install
npm run rust:build
```

本地开发时有两种接法：

```json
{
  "mcpServers": {
    "skill-shelf": {
      "command": "node",
      "args": ["D:/AI_Project/Skill-Shelf/bin/skill-shelf.js", "mcp"]
    }
  }
}
```

或者先 `npm link`，再使用 npm bin：

```bash
npm link
```

```json
{
  "mcpServers": {
    "skill-shelf": {
      "command": "skill-shelf",
      "args": ["mcp"],
      "cwd": "/your/path/to/Skill-Shelf"
    }
  }
}
```

支持 Claude Code（`~/.claude.json`）、Claude Desktop（`claude_desktop_config.json`）、Cursor、Windsurf 等所有 MCP 兼容客户端。每个客户端各自启动一个 stdio shim，共享同一个 daemon 进程。

## 使用

配置完成后，在 MCP 客户端里先调用：

```text
browse_shelf()
```

正常结果应包含：

```json
{
  "groupsCount": 18,
  "totalSkills": 393,
  "watcherStatus": {
    "running": true
  }
}
```

常用流程：

```text
browse_shelf()
  → 看到 group catalog
  → browse_shelf(group="marketing", limit=10)
  → read_skill(skill="微信公众号运营")
```

如果不知道该进哪个组，直接搜索：

```text
search_skills(query="运营 用户 增长 活动 社群 内容 数据", limit=10)
```

安装自己的 skill 包：

```text
install_skills(sourcePath="/path/to/my-skills")
validate_skills()
```

如果你希望把 skill 库放到包目录以外的位置，设置 `SKILL_SHELF_ROOT`。例如：

```json
{
  "mcpServers": {
    "skill-shelf": {
      "command": "npx",
      "args": ["-y", "skill-shelf", "mcp"],
      "env": {
        "SKILL_SHELF_ROOT": "D:/SkillShelf/hub"
      }
    }
  }
}
```

## 架构

```
Claude Code (stdio shim) ─┐
Claude Desktop (stdio)    ├─→ Rust daemon (单进程, 单端口) → 本地 skill 仓库
Cursor / Windsurf / ...  ─┘
```

- **单进程**: 整个仓库只有一个 Rust daemon 在跑，不会因为开多个客户端就跑出几十个 Node 进程
- **stdio shim**: 每个 MCP 客户端启动一个极轻量的 shim 进程（只做 stdin↔IPC 转发），真正的业务逻辑全在 daemon 里
- **共享状态**: 所有客户端共享同一个 skill 索引和缓存，热重载一次全局生效
- **workspace 隔离**: 不同 `SKILL_SHELF_ROOT` 的配置各自独立，互不干扰

## 路由协议

```
browse_shelf()                ← Level 1: group catalog（name + description + count）
  │
  ├─ 选定 group → browse_shelf(group="engineering") → skill summaries
  │
  ├─ 选定 skill → read_skill(skill) → 默认返回 summary
  │                  └─ 需要全文 → read_skill(skill, full=true)
  │
  └─ 组路由不足时 → search_skills(query) 作为兜底
       │
       ▼
search_skills(query)          ← fallback: 直接按关键字兜底定位
```

先看组，再看组内 skill，最后才用 search_skills 兜底。

### 语言策略

搜索会先用用户语言尝试，没结果时再换英文重试。skill 作者无需为每个 skill 写多语言 description。

### 中文搜索

`search_skills` 支持两种输入方式：

1. **空格分词（推荐）**：`品牌 视觉 设计`
2. **连续输入（兜底）**：`品牌设计视觉` — 自动切分为 CJK bigram，匹配精度略低于手动分词

## 工具清单（7 个）

### 只读

| 工具 | 用途 |
|------|------|
| `browse_shelf` | 不传参返回 group catalog + 状态信息；传 `group` 返回组内 skill summaries |
| `search_skills` | 兜底搜索全部 skill，返回 top N 匹配结果 |
| `read_skill` | 默认读取 skill summary；`full=true` 时读取完整正文、资源、参考文件 |

### 写操作

| 工具 | 用途 |
|------|------|
| `install_skills` | 从目录安装 skill 包（支持新建和 LLM 辅助分组） |
| `validate_skills` | 校验完整性；`clean=true` 时自动删除有问题的 skill |
| `manage_group` | 创建/更新/删除存储组（mode: create/update/delete） |
| `reclassify_skill` | 将 skill 移至新的组（更新 frontmatter + 移动目录 + 重建索引） |

## 组体系

18 个内置组：

`engineering` · `design` · `product` · `project-management` · `marketing` · `paid-media` · `sales` · `finance` · `legal-compliance` · `hr-talent` · `support-operations` · `supply-chain` · `academic-research` · `testing-qa` · `spatial-gaming` · `specialized-domain` · `game-studios` · `creative-media`

安装 skill 时，如果 SKILL.md frontmatter 未指定 `group`，工具会返回 skill 描述和可用组列表，由 LLM 选择最合适的组。

## 存储结构

```
data/hub/
  config/groups.json              # 组定义（18 个内置组 + 自定义组）
  packages/{group}/{skill-id}/
    SKILL.md                      # skill 正文（必须）
    meta.json                     # 自动生成的元数据
    references/                   # 可选参考文件
    scripts/                      # 可选辅助脚本
    assets/                       # 可选资源文件
  staging/imports/                # 待审查的导入候选
  index/                          # 索引文件（运行时自动维护）
```

## Skill 包格式

每个 skill 是一个包含 `SKILL.md` 的目录：

```markdown
---
name: my-skill
description: 这个 skill 做什么
group: engineering
---

# My Skill

Skill 正文内容...
```

`name` 和 `description` 是必填 frontmatter 字段。`group` 可选，不填时由 LLM 在安装时分类。

## 热重载

daemon 启动时自动监听 `packages/` 目录变更，新增、修改、删除 skill 后索引自动更新，无需重启。

## CLI 命令

```bash
skill-shelf mcp      # 启动 stdio shim（MCP 客户端调用）
skill-shelf daemon    # 启动/连接 daemon
skill-shelf status    # 查看 daemon 状态
skill-shelf stop      # 停止 daemon
```

## 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `SKILL_SHELF_ROOT` | `<package>/data/hub` | 数据根目录 |
| `SKILL_SHELF_SEARCH_LIMIT` | `8` | search_skills 默认返回上限 |
| `SKILL_SHELF_MAX_RELATED_SKILLS` | `5` | read_skill 返回的最大关联 skill 数 |
| `SKILL_SHELF_WATCH` | `1` | 是否启用文件监听 |

## 致谢

部分 Skill 内容来源于以下开源项目：

- [agency-agents-zh](https://github.com/jnMetaCode/agency-agents-zh)（MIT License）— 211 个中文 AI 专家智能体
- [awesome-design-md](https://github.com/VoltAgent/awesome-design-md)（MIT License）— 品牌设计系统 markdown 文件
- [scientific-agent-skills](https://github.com/K-Dense-AI/scientific-agent-skills)（MIT License）— 139 个科学研究技能（生物信息、药物发现、量子计算等）
