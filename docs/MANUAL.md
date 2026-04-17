# Skill Hub Manual

## 1. 这是什么

Skill Hub 是一个本地 MCP 服务器，注册 9 个工具到 LLM 的 context，让它能按需访问 230+ 个专业技能——不需要的 skill 不占任何 token。内置 230+ 个 skill 只是起步，用户可通过 `install_skills` 持续扩充自己的 skill 库。

## 2. 路由流程

找到目标 skill 的标准路径：

```
search_skills(query)
  │
  ├─ 命中 → 检查 directMatch（如果有的话，看 description 判断是否对路）
  │         → 对路就 read_skill，不对路就重新选 group
  │
  └─ 没命中 → list_skill_groups → 阅读所有组描述 → 选定组
                → list_group_skills(group) → 看 skill 名称 → 选定目标
                → read_skill(skill)
```

**关键规则：**
- search_skills 只是辅助，搜不到就走 list_skill_groups 兜底，这是正常路径不是失败
- directMatch 出现时先看 description 判断意图是否匹配，不要直接跳到 read_skill
- 一次只读 1 个 skill，评估后再决定是否补读

## 3. 常见场景

### 用户明确说了领域

> "帮我优化数据库查询性能"

```
search_skills("数据库")  → 命中 engineering 组
  → list_group_skills("engineering")
  → 看到 "Database Optimizer"
  → read_skill("Database Optimizer")
```

### 用户意图模糊

> "我要做个项目"

```
search_skills("项目")  → 可能命中 project-management
  → 如果结果不确定 → list_skill_groups() 看所有组描述
  → 选定组 → list_group_skills → read_skill
```

### 用户指定了 skill 名称

> "用一下 code-reviewer"

```
search_skills("code-reviewer")  → directMatch 返回 description
  → description 和用户意图一致
  → read_skill("Code Reviewer")
```

### 用户要安装新 skill

```
install_skills(sourcePath="/path/to/skill-package")
validate_skills()  ← 检查有没有问题
```

## 4. 工具速查

### 路由（只读，随时可调）

| 工具 | 什么时候用 |
|------|-----------|
| `search_skills(query)` | 第一步，尝试快速定位 |
| `list_skill_groups()` | search 没结果时的兜底 |
| `list_group_skills(group)` | 选定组后看里面有什么 |
| `read_skill(skill)` | 最终步骤，加载完整 skill 正文 |

### 管理（只读）

| 工具 | 什么时候用 |
|------|-----------|
| `validate_skills(skill?)` | 安装后检查完整性，可校验单个或全部 |
| `get_hub_status()` | 看 skill 总数、组数、issue 数、watcher 状态 |

### 写操作

| 工具 | 什么时候用 | 注意 |
|------|-----------|------|
| `install_skills(sourcePath)` | 从目录批量安装 skill 包 | 会覆盖同 ID 的已有 skill |
| `create_skill(name, description, skillMarkdown)` | 在 hub 内直接创建新 skill | 自动分组 |
| `manage_group(mode, group, ...)` | 创建/更新/删除组 | 三合一，mode 选 create/update/delete |

## 5. Skill 包格式

每个 skill 是一个目录，唯一必须的文件是 `SKILL.md`：

```markdown
---
name: my-skill
description: 这个 skill 做什么
---

# My Skill

正文内容...
```

- `name` 和 `description` 是必填的 frontmatter，用于搜索和自动分组
- `references/` 和 `assets/` 是可选目录，read_skill 时一并返回
- `meta.json` 由系统自动生成，不要手动编辑

## 6. 组体系

16 个内置组：

`engineering` · `design` · `product` · `project-management` · `marketing` · `paid-media` · `sales` · `finance` · `legal-compliance` · `hr-talent` · `support-operations` · `supply-chain` · `academic-research` · `testing-qa` · `spatial-gaming` · `specialized-domain`（兜底）

安装 skill 时自动按关键词加权匹配分组。每个组的 keywords 为中英双语。无法匹配任何组的 skill 归入 `specialized-domain`。

## 7. 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `SKILL_HUB_ROOT` | `<package>/data/hub` | 数据根目录 |
| `SKILL_ROUTER_SEARCH_LIMIT` | `8` | search_skills 默认返回上限 |
| `SKILL_ROUTER_MAX_KEYWORDS` | `12` | 每个 skill 自动提取的最大关键词数 |
| `SKILL_ROUTER_MAX_RELATED_SKILLS` | `5` | read_skill 返回的最大关联 skill 数 |
| `SKILL_ROUTER_WATCH` | `1` | 是否启用文件监听 |
| `SKILL_ROUTER_WATCH_USE_POLLING` | `0` | 是否使用轮询 |
| `SKILL_ROUTER_WATCH_INTERVAL_MS` | `100` | 轮询间隔 |
| `SKILL_ROUTER_WATCH_STABILITY_MS` | `300` | 写入稳定等待时间 |
| `SKILL_ROUTER_WATCH_SYNC_DELETE` | `1` | 删除 SKILL.md 时是否同步移除索引 |
