# 目录结构

```text
worldgen-docflow-template/
├── README.md
├── WORLD.md
├── 00-intake/
│   └── world-brief.yaml
├── 01-brainstorm/
│   ├── brainstorm.md
│   └── candidates.yaml
│   └── reusable-fragments.md
├── 02-canon/
│   ├── world-intent.yaml
│   └── open-questions.md
├── 03-skeleton/
│   ├── world-skeleton.yaml
│   ├── branch-index.yaml
│   └── generation-order.yaml
├── 04-runs/
│   ├── full-run.template.yaml
│   ├── branch-run.template.yaml
│   └── leaf-run.template.yaml
├── 05-world-data/
│   ├── index.md
│   ├── world/
│   │   ├── meta.yaml
│   │   ├── core.yaml
│   │   └── rules.yaml
│   └── branches/
│       └── README.md
└── 06-audit/
    ├── issues.md
    └── issues/
        └── .gitkeep
```

## 各阶段含义

- `00-intake`：最小起始契约
- `01-brainstorm`：定稿前的多方向探索
- `02-canon`：已经锁定的意图包
- `03-skeleton`：稳定的世界图谱骨架
- `04-runs`：显式的生成运行计划
- `05-world-data`：生成出的结构化世界数据
- `06-audit`：冲突、缺口与修复目标

## 人类阅读入口

- `WORLD.md`：人类友好的顶层入口
- `05-world-data/index.md`：最终世界事实入口

普通阅读者默认应从这两个入口之一开始，而不是从 workflow 目录开始。

## 核心设计规则

这套系统必须做到：按顺序浏览文件夹时，人就能理解它的工作流。

如果未来的设计需要在这棵目录树之外保存隐藏工作流状态，那就是一种设计异味。
