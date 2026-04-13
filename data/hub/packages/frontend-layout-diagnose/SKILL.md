---
name: frontend-layout-diagnose
description: "Diagnose and fix layout, alignment, spacing, overflow, scroll, overlap, and information-architecture problems in existing frontend pages. Use when the user points at a screenshot or running page and says things like 看图, 元素不对齐, 被挡住了, 这里空得离谱, 滚不动, 重排, 贴左/贴右, or asks to move, merge, remove, resize, or reorder UI sections. Treat screenshots as complaint input only; use DOM structure, computed geometry, overflow facts, and the page code as the source of truth."
---

# Frontend Layout Diagnose

## Overview

Use this skill for diagnosis and repair of existing frontend pages. It is for layout surgery, not screenshot-to-code generation.

Core rule: define section responsibility first, then measure layout facts, then patch the smallest relevant surface, then verify in the browser.

Use the `playwright` skill alongside this one when browser probing is needed.

## When To Use

Trigger this skill when the user:

- points at a screenshot and says `看图`
- says a page feels wrong but cannot name the exact CSS bug
- complains about alignment, whitespace, compression, overlap, clipping, stretch, or unusable scrolling
- says a section is useless, duplicated, too wide, too empty, or in the wrong place
- asks to reflow, reorder, merge, remove, or promote sections on an existing page

Do not use this skill for:

- net-new page creation from scratch
- pure visual polish where layout and section responsibility are already correct
- backend or data problems that only happen to show up in the UI

## Workflow

### 1. Locate The Real Surface

Before editing:

- find the route, page, component, or template that actually owns the problematic layout
- identify whether the problem lives in the page, a shared layout shell, a reusable card/list component, or global CSS
- confirm the runtime state needed to reproduce the issue

Prefer `rg`, project router files, and component imports over guessing.

### 2. Build A Section Responsibility Map

Summarize the visible page into 3-7 sections:

- section name
- current job
- whether it duplicates another section
- whether it deserves independent space
- whether it should be primary, secondary, merged, or removed

Read [references/layout-principles.md](references/layout-principles.md) before deciding whether a section should stay, merge, move, or disappear.

### 3. Probe Structure, Not Just Pixels

Do not diagnose from screenshot alone.

Use Playwright to gather the minimum facts needed:

- DOM and heading structure
- parent-child relationships
- bounding boxes
- grid or flex parent rules
- overflow and scroll facts
- content height versus container height
- overlap, sticky, and z-index facts when something looks blocked

Read [references/playwright-probes.md](references/playwright-probes.md) and run the smallest probe that answers the current question.

If you export a raw JSON report, optionally run:

```powershell
python C:\Users\HomeAdmin\.codex\skills\frontend-layout-diagnose\scripts\diagnose_layout_report.py path\to\layout-report.json
```

### 4. Write A Diagnosis DSL

Before patching code, write a compact diagnosis using [references/layout-dsl.md](references/layout-dsl.md).

The diagnosis must answer:

- which sections are redundant
- which whitespace is structural waste
- which container is stretching or clipping another
- which scroll region is wrong
- what the new page skeleton should be

Do not skip this step when the complaint is vague. This is the guardrail against random CSS pokes.

### 5. Patch The Smallest Relevant Surface

Usually patch only:

- the page component or template
- a shared layout shell if the bug is truly systemic
- a duplicated summary or sidebar component if duplication is the root cause
- shared CSS only when the same pathology affects multiple pages

Prefer structural fixes over padding or margin tuning. If the skeleton is wrong, fix the skeleton.

### 6. Validate Hard

After patching:

- run the relevant frontend build or test command
- re-open the real page in Playwright
- re-measure the sections you changed
- verify the target scroll containers actually scroll when they should
- check at least the primary breakpoint and one secondary breakpoint if the layout is responsive
- take a screenshot only as the final visual check

If DOM facts and screenshots disagree, trust DOM facts first and keep investigating.

## Non-Negotiable Rules

- Screenshot is evidence of pain, not the source of truth.
- Never explain a layout bug from intuition alone if the browser can measure it.
- A section without an independent job should be merged, demoted, or removed.
- A large container with tiny content is a bug until proven intentional.
- Wrong scroll ownership is a structural bug, not a polish issue.
- Do not preserve a bad skeleton just because spacing tweaks can hide it.
- Build and browser verification are required before claiming the page is fixed.

## References

- [references/layout-principles.md](references/layout-principles.md)
  Read when judging whether a section should exist, merge, move, or shrink.
- [references/layout-dsl.md](references/layout-dsl.md)
  Read before writing the diagnosis or planning a reflow.
- [references/playwright-probes.md](references/playwright-probes.md)
  Read when collecting DOM, geometry, overflow, or overlap facts.
