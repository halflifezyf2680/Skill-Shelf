---
name: frontend-ia-refactor
description: "Refactor page-level information architecture in existing frontends by deciding which sections should exist, which are primary, which are duplicate, and how the page should be regrouped or reordered. Use when the user says a page feels confusing, some panels seem useless, sections repeat each other, the main task is buried, or the whole page needs to be split, merged, promoted, or rearranged. Treat screenshots as complaint input only; use visible section responsibilities and measured page structure as the source of truth."
---

# Frontend IA Refactor

## Overview

Use this skill when the main problem is not one CSS property but the page's section model.

Core rule: decide the page's primary task first, then rebuild the page around that task.

Use `frontend-layout-diagnose` when geometry facts are also needed.

## When To Use

Trigger this skill when the user:

- says a whole page feels wrong, confusing, or badly organized
- says a panel has no purpose
- says the page has duplicate summaries or duplicate sidebars
- asks to merge, promote, remove, or reorder major sections
- wants the page to become more focused, cleaner, or more professional

Do not use this skill for:

- narrow CSS bugs with otherwise correct page structure
- one-off spacing polish with no information-architecture question

## Workflow

### 1. Identify The Page Contract

State in one sentence:

- why this page exists
- what the primary user task is
- what must be visible immediately
- what can be secondary or deferred

Read [references/page-contracts.md](references/page-contracts.md).

### 2. Inventory The Sections

Map each visible section:

- name
- job
- density
- duplication
- user value

Read [references/section-verdicts.md](references/section-verdicts.md).

### 3. Produce The New Skeleton

Before changing code, write the current skeleton and the target skeleton using [references/ia-dsl.md](references/ia-dsl.md).

Good target skeletons are concise. They answer:

- what is primary
- what is secondary
- what disappears
- what moves

### 4. Patch By Responsibility

Refactor the page so that:

- one canonical summary exists
- the primary task gets the best space
- support panels stop dominating the page
- duplicated sidebars or inspectors are merged or demoted

Prefer deleting redundant shells over restyling them.

### 5. Verify Readability

After patching, verify:

- the page purpose is obvious at first glance
- the main task is no longer buried
- duplicated information is gone
- the reading order is natural

## Non-Negotiable Rules

- A page without a clear primary task will always feel wrong.
- Duplicate summaries are debt, not richness.
- A support panel should not dominate the page.
- If a section exists only because the code produced it, remove or merge it.

## References

- [references/page-contracts.md](references/page-contracts.md)
  Read when deciding what the page is for.
- [references/section-verdicts.md](references/section-verdicts.md)
  Read when deciding keep, merge, promote, or remove.
- [references/ia-dsl.md](references/ia-dsl.md)
  Read before producing the new page skeleton.
