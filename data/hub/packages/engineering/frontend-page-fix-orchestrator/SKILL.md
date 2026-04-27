---
name: frontend-page-fix-orchestrator
description: "Orchestrate diagnosis and repair of problematic frontend pages by deciding whether the real issue is layout geometry, responsive behavior, information architecture, or a combination. Use when the user gives a screenshot or page and says you yourself judge the problem, this page looks weird, analyze before fixing, or wants the agent to choose the right workflow before changing code."
---

# Frontend Page Fix Orchestrator

## Overview

Use this skill when the user does not yet know whether the page is broken because of:

- layout geometry
- responsive behavior
- page-level information architecture
- multiple causes together

Core rule: triage first, then invoke the smallest specialist workflow that matches the real problem.

Built-in specialist workflows:

- **Layout Diagnose**: alignment, spacing, overflow, scroll, overlap
  - [references/layout-diagnose/playwright-probes.md](references/layout-diagnose/playwright-probes.md) — DOM/geometry probes
  - [references/layout-diagnose/layout-principles.md](references/layout-diagnose/layout-principles.md) — section judgment principles
  - [references/layout-diagnose/layout-dsl.md](references/layout-diagnose/layout-dsl.md) — structured diagnosis format
  - [scripts/diagnose_layout_report.py](scripts/diagnose_layout_report.py) — layout report analyzer

- **IA Refactor**: section purpose, page hierarchy, duplicate removal
  - [references/ia-refactor/ia-dsl.md](references/ia-refactor/ia-dsl.md) — IA diagnosis format
  - [references/ia-refactor/page-contracts.md](references/ia-refactor/page-contracts.md) — page contract patterns
  - [references/ia-refactor/section-verdicts.md](references/ia-refactor/section-verdicts.md) — section verdict guide

Use the `playwright` MCP tool whenever browser facts are needed.

## Triage Order

### 1. Classify The Complaint

Map the complaint to one or more categories:

- `geometry`: misalignment, whitespace, clipping, overlap, wrong scroll owner
- `responsive`: only broken at some resolutions, breakpoints, aspect ratios, or orientations
- `information_architecture`: duplicate sections, useless panels, wrong hierarchy, buried primary task

Read [references/triage-matrix.md](references/triage-matrix.md).

### 2. Choose The Lead Workflow

Use this rule:

- if the problem is mainly section purpose or page hierarchy, start with **IA Refactor**
- otherwise start with **Layout Diagnose**

If the complaint spans multiple categories, use one lead workflow and one follow-up, not both at once.

### 3. Sequence The Work

Preferred sequences:

- IA -> layout

Avoid:

- random CSS edits before triage
- layout polishing when the skeleton is wrong

### 4. Produce A Short Triage Note

Before editing, write:

- lead category
- lead workflow
- optional follow-up workflow
- one-sentence reason

Example:

```text
Lead category: information_architecture
Lead workflow: IA Refactor
Follow-up: Layout Diagnose
Reason: the page has duplicate summary sections and the primary content is buried, so section restructuring must happen first.
```

### 5. Execute The Specialist Workflow

Once the lead workflow is chosen:

- read the corresponding reference files
- follow that workflow's process
- only bring in the follow-up workflow if the first pass proves another root cause remains

## Non-Negotiable Rules

- Do not start with CSS guessing when the complaint is still unclassified.
- Do not run all specialist workflows in full if one is clearly dominant.
- Page contract problems outrank spacing polish when whole sections are useless or duplicated.
- The orchestrator's job is triage and sequencing, not replacing specialist diagnosis.

## References

- [references/triage-matrix.md](references/triage-matrix.md)
  Read when deciding which specialist workflow should lead.
