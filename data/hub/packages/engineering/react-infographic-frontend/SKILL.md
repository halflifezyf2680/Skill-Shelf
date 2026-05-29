---
name: react-infographic-frontend
description: Build interactive technical infographic websites with React. Use when Codex needs to create or iterate on Vite or React frontends for architecture explainers, product method pages, interactive diagrams, evidence filters, timelines, dashboards, or motion-infographic style web pages. Includes a Vite infographic template and responsive layout guidance.
---

# React Infographic Frontend

Use this skill to build interactive React websites that explain technical systems with dense, polished information graphics.

## Workflow

1. Create a starter with `scripts/create-frontend-project.mjs`.
2. Convert the content into sections: hero/work surface, flow, compare, evidence, timeline, closing.
3. Use `assets/vite-infographic` as the baseline visual system.
4. Read `references/interaction-patterns.md` when adding hover, tabs, filters, scroll reveals, or stateful diagrams.
5. Read `references/responsive-layout.md` before finishing.
6. Run `npm install`, `npm run check`, and `npm run build`.
7. Start the dev server and inspect desktop and mobile viewports.

## Command

From the skill folder:

```bash
node scripts/create-frontend-project.mjs --out path/to/project
```

In the generated project:

```bash
npm install
npm run dev
npm run check
npm run build
```

## Design Rules

- Build the usable page first, not a marketing landing page unless explicitly requested.
- Use full-width sections with constrained inner content.
- Do not nest cards inside cards.
- Use icons for tools and states.
- Keep repeated cards at 8px radius.
- Every technical section needs a result, evidence, or action area. Avoid empty lower thirds.
- Make labels concrete: function, file, caller, risk, status, phase, evidence, source.
- Preserve scanability over decorative composition.

## Interaction Rules

- Use hover and click states to reveal evidence or next checks.
- Use tabs or segmented controls for modes.
- Use filters for recall/evidence pages.
- Use CSS transitions for simple UI motion. Add Framer Motion only when the project already uses it or the interaction needs timeline control.

## Validation

Before final delivery:

- Inspect desktop and mobile viewports.
- Check that long code labels wrap.
- Check that no text overlaps on cards, buttons, chips, or nav.
- Run `npm run check` and `npm run build`.
- Keep the dev server running and provide the local URL.
