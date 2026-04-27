# Skill Shelf

**用 6 个工具的固定开销，按需访问 500-1000+ 个专业技能，而不需要把任何 description 常驻上下文。**

传统做法：每个 skill 作为本地 skill 加载 → description 全量驻留 context → 230+ 个 skill = 数万 tokens 白白浪费，且每次对话都背着它们跑。

内置 233 个 skill 只是起步。用户可通过 `install_skills` 从目录批量安装，持续扩充自己的 skill 库。

Skill Shelf 的做法：skill 全部存放在本地包仓库，context 里只有 6 个轻量工具定义。LLM 需要某个专业能力时，通过搜索按需加载。不需要的时候，零开销。

### 不是什么

当前 AI skill 系统有一个普遍倾向：把 skill 当成可编程模块，试图构建 skill 图谱、执行链、skill 互相召唤。这是赌徒心态——在 LLM 的非确定性推理上叠加一层复杂的控制流，期望它可靠地按预设路径执行。结果就是：链越长越不可控，图越深越难调试，最终产出质量完全靠运气。

Skill Shelf 不做这些事。每个 skill 就是一份自包含的 Markdown 文档，搜索只负责一件事：确定性查找。从成百上千个 skill 中，精准定位到目标 skill。

搜索层不预设 skill 之间的协作关系，但 LLM 读到多个 skill 后自行判断协作需要——这是 LLM 的主动性，不是基础设施的事。

## 路由协议

```
search_skills(query)          ← Level 1: YAML frontmatter（name + description）
  │
  ├─ 命中 → 返回匹配的 skills（skillId + skillName + description + score）
  │
  └─ 没命中 → 空结果，LLM 换词重试
       │
       ▼
read_skill(skill)             ← Level 2: SKILL.md 正文（去 frontmatter）
       │
       └─ 需要更多 → Read references/ 目录下的文件  ← Level 3: 关联资源按需加载
```

两步完成。扁平搜索，不分层级，不依赖分组。

### search_skills 搜索机制

全量扁平模糊搜索。对每个 skill，用 `scoreSkillText` 评分，排序后返回 top N：

- **精确匹配**（name/id 完全相等）：120 分
- **前缀/后缀匹配**：65 分
- **子串包含**：40 分
- **token 重叠**：每个命中 token 18 分
- **Levenshtein 模糊**：距离越近分越高
- **组名命中**：10 分

多维度同时命中自然叠加高分，LLM 无需关心评分细节。

### 语言策略

工具会引导 LLM 先用用户语言搜索，没结果时自动换英文重试。skill 作者无需为每个 skill 写多语言 description。

### 中文搜索

支持两种输入方式：

1. **空格分词（推荐）**：`品牌 视觉 设计` — LLM 被工具描述引导使用此格式
2. **连续输入（兜底）**：`品牌设计视觉` — 自动切分为 CJK bigram（"品牌"、"牌设"、"设计"、"计视"、"视觉"），匹配精度略低于手动分词

## 存储结构

```
data/hub/
  config/groups.json              # 组定义（17 个内置组 + 自定义组）
  packages/{group}/{skill-id}/
    SKILL.md                      # skill 正文（必须）
    meta.json                     # 自动生成的元数据
    references/                   # 可选参考文件
    scripts/                      # 可选辅助脚本
    assets/                       # 可选资源文件
  staging/imports/                # 待审查的导入候选（运行时）
  staging/repaired/               # 已修复的候选（运行时）
  index/                          # 索引文件（运行时自动维护）
  logs/                           # 运行时日志
```

组只影响存储目录结构，不影响搜索路由（搜索是扁平的）。

可通过环境变量覆盖根目录：

```bash
SKILL_SHELF_ROOT=/your/custom/path
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

`name` 和 `description` 是必填 frontmatter 字段，用于搜索索引。`group` 可选，不填时由 LLM 在安装时分类。

## 工具清单（6 个）

### 路由（只读）

| 工具 | 用途 |
|------|------|
| `search_skills` | 扁平搜索全部 skill，返回 top N 匹配结果（name + description + score） |
| `read_skill` | 读取 skill 完整正文、资源、参考文件 |
| `validate_skills` | 校验所有 skill 的完整性和重复情况 |
| `get_shelf_status` | 查看索引和文件监听状态 |

### 写操作

| 工具 | 用途 |
|------|------|
| `install_skills` | 从目录安装 skill 包（支持新建和 LLM 辅助分组） |
| `manage_group` | 创建/更新/删除存储组（mode: create/update/delete） |

## 组体系

内置 17 个组，覆盖主要专业领域，作为 skill 的存储目录：

`engineering` · `design` · `product` · `project-management` · `marketing` · `paid-media` · `sales` · `finance` · `legal-compliance` · `hr-talent` · `support-operations` · `supply-chain` · `academic-research` · `testing-qa` · `spatial-gaming` · `specialized-domain` · `game-studios`

安装 skill 时，如果 SKILL.md frontmatter 未指定 `group`，工具会返回 skill 描述和可用组列表，由 LLM 选择最合适的组。创建新 skill 应使用 `skill-creator` skill 的完整方法论，完成后通过 `install_skills` 入库。

## 热重载

启动时自动监听 `packages/` 目录变更，新增、修改、删除 skill 后索引自动更新，无需重启。

## 安装

### 从 GitHub 克隆

```bash
git clone https://github.com/halflifezyf2680/Skill-Shelf.git
cd Skill-Shelf
npm install
```

## 配置 MCP Server

### Claude Code

在 `~/.claude.json` 的 `mcpServers` 中添加：

```json
{
  "mcpServers": {
    "skill-shelf": {
      "command": "npm",
      "args": ["run", "skill-shelf"],
      "cwd": "/your/path/to/Skill-Shelf"
    }
  }
}
```

### Claude Desktop

在 Claude Desktop 的 `claude_desktop_config.json` 中添加：

```json
{
  "mcpServers": {
    "skill-shelf": {
      "command": "npm",
      "args": ["run", "skill-shelf"],
      "cwd": "/your/path/to/Skill-Shelf"
    }
  }
}
```

将 `/your/path/to/Skill-Shelf` 替换为实际克隆路径。配置完成后重启客户端即可。

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
