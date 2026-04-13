---
name: responsive-layout-debug
description: "Diagnose and fix responsive layout failures across breakpoints, aspect ratios, viewport sizes, and scroll containers in existing frontend pages. Use when the user says mobile/tablet/desktop layouts break, width or height changes do nothing, aspect ratio switching is ineffective, content clips at some resolutions, panels stop scrolling, or sticky/fixed layers cover content at certain sizes. Treat screenshots as symptoms only; use browser measurements across multiple viewport presets as the source of truth."
---

# Responsive Layout Debug

## Overview

Use this skill when the page is correct at one size but fails at others.

Core rule: compare the same page across a viewport matrix before changing the code.

Use the `playwright` skill alongside this one.

## When To Use

Trigger this skill when the user:

- says mobile, tablet, desktop, or fullscreen layouts differ in bad ways
- says resolution switching has no visible effect
- reports aspect-ratio presets that do not actually change the stage
- reports clipping, overlap, dead space, or non-scrolling panels only at some sizes
- wants a layout verified across `16:9`, `16:10`, `4:3`, portrait, or narrow desktop widths

Do not use this skill for:

- single-breakpoint layout bugs with no responsive dimension
- content hierarchy problems where the layout is consistently wrong at every size

## Workflow

### 1. Define The Viewport Matrix

Pick the smallest useful set:

- current failing viewport
- one known-good viewport
- one neighboring breakpoint
- one alternate aspect ratio if ratio is part of the complaint

Read [references/viewport-matrix.md](references/viewport-matrix.md).

### 2. Measure The Same Targets Across Sizes

Measure the same 3-6 target sections at each viewport:

- width and height
- left and top
- scroll ownership
- whether the target is clipped
- whether min-height or fixed height is dominating

Read [references/responsive-probes.md](references/responsive-probes.md).

If you export a raw JSON report, optionally run:

```powershell
python C:\Users\HomeAdmin\.codex\skills\responsive-layout-debug\scripts\summarize_viewport_matrix.py path\to\viewport-report.json
```

### 3. Name The Responsive Failure Mode

Before patching, identify the class of bug:

- breakpoint mismatch
- ratio switch not wired
- min-height or fixed height dominance
- wrong scroll owner
- flex or grid stretch pathology
- sticky or fixed overlap
- hidden overflow clipping

Write a compact diagnosis using [references/responsive-dsl.md](references/responsive-dsl.md).

### 4. Patch The Governing Rule

Patch the rule that governs the behavior:

- breakpoint config
- ratio token or display preset wiring
- container size rule
- flex/grid parent contract
- sticky/fixed layer offset
- overflow assignment

Do not paper over a responsive bug with one-off pixel offsets.

### 5. Verify Across The Matrix

After patching:

- rebuild or run the relevant frontend verification
- re-check the same targets at the same viewport matrix
- confirm the originally failing viewport changed in the intended way
- confirm the previously good viewport did not regress

## Non-Negotiable Rules

- Responsive bugs are comparison bugs. Always compare at least two sizes.
- If ratio switching changes state but not geometry, inspect the dominant size rule.
- If a panel should scroll only at smaller sizes, verify scroll ownership directly.
- A fix that works at one breakpoint but breaks an adjacent one is not a fix.

## References

- [references/viewport-matrix.md](references/viewport-matrix.md)
  Read when choosing the viewport set.
- [references/responsive-probes.md](references/responsive-probes.md)
  Read when collecting geometry and scroll facts across sizes.
- [references/responsive-dsl.md](references/responsive-dsl.md)
  Read before writing the diagnosis.
