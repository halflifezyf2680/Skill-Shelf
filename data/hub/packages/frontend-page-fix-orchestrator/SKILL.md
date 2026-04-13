---
name: frontend-page-fix-orchestrator
description: "Orchestrate diagnosis and repair of problematic frontend pages by deciding whether the real issue is layout geometry, responsive behavior, information architecture, or a combination. Use when the user gives a screenshot or page and says 你自己判断问题, 这个页面很怪, 先分析再修, or wants the agent to choose the right workflow before changing code. This skill does not replace specialist skills; it selects and sequences them."
---

# Frontend Page Fix Orchestrator

## Overview

Use this skill when the user does not yet know whether the page is broken because of:

- layout geometry
- responsive behavior
- page-level information architecture
- multiple causes together

Core rule: triage first, then invoke the smallest specialist workflow that matches the real problem.

Primary specialists:

- `$frontend-layout-diagnose`
- `$responsive-layout-debug`
- `$frontend-ia-refactor`

Use the `playwright` skill whenever browser facts are needed.

## Triage Order

### 1. Classify The Complaint

Map the complaint to one or more categories:

- `geometry`: misalignment, whitespace, clipping, overlap, wrong scroll owner
- `responsive`: only broken at some resolutions, breakpoints, aspect ratios, or orientations
- `information_architecture`: duplicate sections, useless panels, wrong hierarchy, buried primary task

Read [references/triage-matrix.md](references/triage-matrix.md).

### 2. Choose The Lead Skill

Use this rule:

- if the problem changes across viewport sizes, start with `$responsive-layout-debug`
- if the problem is mainly section purpose or page hierarchy, start with `$frontend-ia-refactor`
- otherwise start with `$frontend-layout-diagnose`

If the complaint spans multiple categories, use one lead skill and one follow-up skill, not all three at once.

### 3. Sequence The Work

Preferred sequences:

- responsive -> layout
- IA -> layout
- responsive -> IA -> layout only if viewport behavior is blocking section judgment

Avoid:

- random CSS edits before triage
- IA refactors before proving the bug is not just breakpoint-specific
- responsive debugging after large page rewrites unless needed

### 4. Produce A Short Triage Note

Before editing, write:

- lead category
- lead skill
- optional follow-up skill
- one-sentence reason

Example:

```text
Lead category: responsive
Lead skill: $responsive-layout-debug
Follow-up: $frontend-layout-diagnose
Reason: the bug only appears when aspect ratio changes, so geometry facts must be compared across viewports first.
```

### 5. Execute The Specialist Workflow

Once the lead skill is chosen:

- follow that skill's workflow
- only bring in the follow-up skill if the first pass proves another root cause remains

## Non-Negotiable Rules

- Do not start with CSS guessing when the complaint is still unclassified.
- Do not run all specialist workflows in full if one is clearly dominant.
- Responsive causes outrank page-level restructuring when the layout changes by viewport.
- Page contract problems outrank spacing polish when whole sections are useless or duplicated.
- The orchestrator's job is triage and sequencing, not replacing specialist diagnosis.

## References

- [references/triage-matrix.md](references/triage-matrix.md)
  Read when deciding which specialist skill should lead.
