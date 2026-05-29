---
name: remotion-infographic-video
description: Build animated technical infographic videos with Remotion. Use when Codex needs to create or iterate on Remotion-based explainer videos, architecture videos, methodology videos, narrated technical videos, motion infographic scenes, audio-timed compositions, or still-frame checks for video scenes. Includes blank and infographic Remotion templates plus validation guidance.
---

# Remotion Infographic Video

Use this skill to create Remotion videos that explain technical systems with animated information graphics.

## Workflow

1. Choose a template:
   - Use `assets/remotion-blank` for a plain Remotion starter.
   - Use `assets/remotion-infographic` for a styled technical infographic starter.
2. Create the project with `scripts/create-remotion-project.mjs`.
3. Convert the content brief into scenes. Prefer one React component per scene.
4. Put timing in `src/videoConfig.ts`. Derive durations from narration manifests when audio exists.
5. Use the visual grammar from `references/visual-system.md`.
6. Validate with `npm run check` and at least one `npm run still` frame per changed scene.
7. Render only after still-frame checks look clean.

## Commands

From the skill folder:

```bash
node scripts/create-remotion-project.mjs --template infographic --out path/to/project
node scripts/create-remotion-project.mjs --template blank --out path/to/project
```

In the generated project:

```bash
npm install
npm run dev
npm run check
npm run still
npm run render
```

## Scene Rules

- Build the actual video scene, not a landing page.
- Use a three-band structure for dense explanation scenes: title, main structure, result/evidence.
- Avoid empty lower thirds. If a scene has unused bottom space, add output, evidence, risk, next-step, or timeline content.
- Keep cards at 8px radius. Use lucide icons for tools and concepts.
- Prefer concrete labels: files, functions, caller/callee, risk, status, phase, gate, evidence.
- Hide or move global corner marks when they touch scene content.

## Animation Rules

- Use Remotion `useCurrentFrame`, `spring`, and `interpolate`.
- Stagger repeated items by index.
- Keep motion explanatory: reveal structure, converge noise, highlight evidence, or show state transition.
- Do not animate just for decoration.

## Audio Timing

Read `references/audio-timing.md` when adding narration. Use per-scene audio files and a manifest. Do not use concatenated preview audio as the final track.

## Validation

Read `references/layout-checklist.md` before final delivery. For each changed scene, generate a still frame near the scene's information-dense moment and inspect it for:

- text overlap
- clipped content
- excessive empty space
- corner mark collisions
- illegible code text
- layout shift caused by long labels
