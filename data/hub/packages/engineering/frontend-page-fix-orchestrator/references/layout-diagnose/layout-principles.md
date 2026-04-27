# Frontend Layout Principles

Use these rules before changing layout.

## Core Rule

Judge section responsibility first. Styling is secondary.

## Page-Level Standards

- Each major section must have an independent job.
- Repeated information should have one canonical home.
- Large empty areas are bugs unless they intentionally stage a focal element.
- The page should read in a natural order: context -> current state -> decision or action -> support details.
- A low-density panel should not consume the space needed by a high-density panel.
- The scroll owner should be obvious. Users should not need to fight nested scroll regions.

## What Usually Means "Merge Or Remove"

- A section only repeats values already visible in the top summary or sidebar.
- A section exists only because the code happened to create another card.
- A section title sounds meaningful, but the contents are only a few tiny rows.
- A section becomes large only because a sibling in the same row is taller.
- A sidebar claims a full column but contains only secondary metadata.

## What Usually Means "Promote"

- The section contains the page's primary task or decision.
- The section answers "what should I do here".
- The section contains ongoing progress, key results, or build-defining choices.
- The section is the reason this page exists at all.

## What Usually Means "Reflow Only"

- The information architecture is correct, but the section order, width ratio, or breakpoint behavior is wrong.
- The right content exists, but the scroll owner is wrong.
- The layout is blocked by flex stretch, fixed height, min-height, sticky overlap, or an accidental width cap.

## Common Pathologies

- duplicate summary blocks in multiple columns
- wide shells with tiny islands of content
- one column mostly empty because another column stretched the row
- important actions pushed below low-value panels
- content clipped by fixed heights or absolute overlays
- panels that look scrollable but do not actually scroll
- visually centered content that should be edge-aligned to a nearby anchor
