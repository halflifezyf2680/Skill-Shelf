# Skill Shelf Manual

## 1. 这是什么

Skill Shelf 是一个本地 MCP 服务器，注册一组轻量工具到 LLM 的 context，让它能按需访问 393 个专业技能——只常驻 group 级路由信息，不常驻全部 skill description。内置 skill 只是起步，用户可通过 `install_skills` 持续扩充自己的 skill 库。

## 1.1 Rust singleton daemon migration

仓库的发布入口已经收敛到 Rust 单例 daemon 架构，默认命令是 `skill-shelf mcp`。

- `skill-shelf` npm bin 会优先查找随包或本地构建的 Rust 可执行文件，然后转发 CLI 参数。
- 找不到 Rust 可执行文件时会直接失败，并提示先执行 `npm run rust:build`。
- `daemon` / `status` / `stop` 保持 Rust CLI 语义，建议直接使用 `npm run daemon`、`npm run status`、`npm run stop`。
- Rust crate 位置：`rust/skill-shelf`
- 已实现范围：config / model / parser / search / registry / ipc / lock / lifecycle / workspace / mcp_shim，以及 `skill-shelf mcp|daemon|status|stop` CLI 主流程

因此，本手册里默认启动方式就是 Rust 路径。

## 2. 安装与接入

### 2.1 从源码使用

```bash
git clone https://github.com/halflifezyf2680/Skill-Shelf.git
cd Skill-Shelf
npm install
npm run rust:build
```

MCP 客户端可以直接指向仓库里的 npm wrapper：

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

也可以先执行 `npm link`，再用：

```json
{
  "mcpServers": {
    "skill-shelf": {
      "command": "skill-shelf",
      "args": ["mcp"]
    }
  }
}
```

首次工具调用会自动拉起 Rust daemon。多个客户端会共享同一个 daemon。

### 2.2 验证

配置完成后先调用：

```text
browse_shelf()
```

正常返回应包含：

```json
{
  "groupsCount": 18,
  "totalSkills": 393,
  "watcherStatus": {
    "running": true
  }
}
```

如果返回 `groupsCount: 0` / `totalSkills: 0`，优先检查：

- 是否设置了错误的 `SKILL_SHELF_ROOT`
- 源码仓库内是否存在 `data/hub/packages`
- 本地开发时是否已经执行 `npm run rust:build`

## 3. 路由流程

找到目标 skill 的标准路径：

```
browse_shelf()
  │
  ├─ 看 group catalog（name + description + count）
  │
  ├─ 选定 group → browse_shelf(group="engineering")
  │             → 看 skill summaries → 选定目标
  │
  ├─ 选定 skill → read_skill(skill)
  │             → 默认拿 summary
  │             → 需要全文时 read_skill(skill, full=true)
  │
  └─ 组路由不够准时 → search_skills(query) 兜底
```

**关键规则：**
- browse_shelf 先看组，不要把它当成一级搜索入口
- browse_shelf(group=...) 只看组内 skill summaries，先选定再读
- read_skill 默认读 summary，只有确实需要全文时才开 `full=true`
- search_skills 只是兜底，搜不到或不够准时再用
- 一次只读 1 个 skill，评估后再决定是否补读

## 4. 常见场景

### 用户明确说了领域

> "帮我优化数据库查询性能"

```
browse_shelf()
  → 看到 engineering 组
  → browse_shelf(group="engineering")
  → 看到 "Database Optimizer"
  → read_skill("Database Optimizer")
```

### 用户意图模糊

> "我要做个项目"

```
browse_shelf()
  → 先看所有组描述
  → 选定 project-management
  → browse_shelf(group="project-management")
  → read_skill
```

### 用户指定了 skill 名称

> "用一下 code-reviewer"

```
search_skills("code-reviewer")  → 兜底直接定位
  → description 和用户意图一致
  → read_skill("Code Reviewer")
```

### 用户要安装新 skill

```
install_skills(sourcePath="/path/to/skill-package")
validate_skills()  ← 检查有没有问题
```

如果返回 `needs_classification`，说明 skill 没有可确定的组。选择合适的组后再次调用：

```text
install_skills(sourcePath="/path/to/skill-package", group="engineering")
```

## 5. 工具速查

### 路由（只读，随时可调）

| 工具 | 什么时候用 |
|------|-----------|
| `browse_shelf()` | 第一步，先看 group catalog |
| `browse_shelf(group)` | 选定组后看里面有什么 |
| `read_skill(skill)` | 默认加载 summary；需要全文时用 `full=true` |
| `search_skills(query)` | 组路由不够准时的兜底定位 |

### 写操作

| 工具 | 什么时候用 | 注意 |
|------|-----------|------|
| `install_skills(sourcePath)` | 从目录批量安装 skill 包 | 会覆盖同 ID 的已有 skill |
| `validate_skills(skill?)` | 安装后检查完整性，可校验单个或全部 | `clean=true` 会删除 blocked 问题包 |
| `manage_group(mode, group, ...)` | 创建/更新/删除组 | 三合一，mode 选 create/update/delete |
| `reclassify_skill(skill, target_group)` | 把 skill 移到另一个组 | 更新 frontmatter、移动目录并重建索引 |

## 6. Skill 包格式

每个 skill 是一个目录，唯一必须的文件是 `SKILL.md`：

```markdown
---
name: my-skill
description: 这个 skill 做什么
---

# My Skill

正文内容...
```

- `name` 和 `description` 是必填的 frontmatter，用于搜索和安装识别
- `group` 是可选 frontmatter；不填时安装工具会返回 `needs_classification`，再由调用方带 `group` 重试
- `references/` 和 `assets/` 是可选目录，read_skill 时一并返回
- `meta.json` 由系统自动生成，不要手动编辑

## 7. 组体系

18 个内置组：

`engineering` · `design` · `product` · `project-management` · `marketing` · `paid-media` · `sales` · `finance` · `legal-compliance` · `hr-talent` · `support-operations` · `supply-chain` · `academic-research` · `testing-qa` · `spatial-gaming` · `specialized-domain` · `game-studios` · `creative-media`

组负责浏览入口和存储组织。安装 skill 时如果 frontmatter 没有 `group`，工具会返回候选 skill 描述和可用组列表；选择组后再次带 `group` 参数安装。

## 8. CLI 与 daemon 运维

```bash
skill-shelf mcp       # MCP stdio shim，通常由 MCP 客户端启动
skill-shelf status    # 查看 daemon 状态
skill-shelf stop      # 停止 daemon
```

源码开发时也可以用 npm scripts：

```bash
npm run status
npm run stop
```

daemon 状态文件默认位于系统用户数据目录。Windows 下通常是：

```text
%LOCALAPPDATA%/skill-shelf/daemon-state.json
```

## 9. 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `SKILL_SHELF_ROOT` | `<package>/data/hub` | 数据根目录 |
| `SKILL_SHELF_SEARCH_LIMIT` | `8` | search_skills 默认返回上限 |
| `SKILL_SHELF_MAX_KEYWORDS` | `12` | 每个 skill 自动提取的最大关键词数 |
| `SKILL_SHELF_MAX_RELATED_SKILLS` | `5` | read_skill 返回的最大关联 skill 数 |
| `SKILL_SHELF_WATCH` | `1` | 是否启用文件监听 |
| `SKILL_SHELF_WATCH_USE_POLLING` | `1` | 是否使用轮询 |
| `SKILL_SHELF_WATCH_INTERVAL_MS` | `100` | 轮询间隔 |
| `SKILL_SHELF_WATCH_STABILITY_MS` | `300` | 写入稳定等待时间 |
| `SKILL_SHELF_WATCH_SYNC_DELETE` | `1` | 删除 SKILL.md 时是否同步移除索引 |
