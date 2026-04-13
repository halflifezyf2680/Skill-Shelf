---
name: design-md
description: Apply a real-world brand design system to a project by selecting from 58 curated DESIGN.md files extracted from websites like Vercel, Stripe, Notion, Linear, Figma, Apple, Tesla, etc. Use whenever the user wants to match a brand's visual style, says "make it look like X", asks for a design system, design tokens, brand guide, style guide, UI theme, color palette inspiration, or wants consistent UI that matches a specific company's aesthetic. Also trigger when the user mentions DESIGN.md, design-md, brand styling, or wants to copy/apply/adopt a company's design language.
argument-hint: "[brand name or category, e.g. vercel, stripe, AI, fintech]"
---

# DESIGN.md Brand Design System Selector

Copy a DESIGN.md into any project to give AI agents a complete visual design system they can follow when generating UI. This skill provides access to 58 real-world brand design systems extracted from public websites, following the [DESIGN.md format](https://stitch.withgoogle.com/docs/design-md/overview/) introduced by Google Stitch.

## The DESIGN.md Collection

The source files live at: `references/brands/`

Each brand folder contains:
- `DESIGN.md` — the full design system document (what agents read)
- `preview.html` — visual catalog (light mode)
- `preview-dark.html` — visual catalog (dark mode)

## Brand Index

### AI & Machine Learning
| Brand | Key Characteristics |
|-------|-------------------|
| claude | Warm terracotta accent, clean editorial |
| cohere | Vibrant gradients, data-rich dashboard |
| elevenlabs | Dark cinematic UI, audio-waveform aesthetics |
| minimax | Bold dark interface, neon accents |
| mistral.ai | French-engineered minimalism, purple-toned |
| ollama | Terminal-first, monochrome simplicity |
| opencode.ai | Developer-centric dark theme |
| replicate | Clean white canvas, code-forward |
| runwayml | Cinematic dark UI, media-rich |
| together.ai | Technical, blueprint-style |
| voltagent | Void-black canvas, emerald accent |
| x.ai | Stark monochrome, futuristic minimalism |

### Developer Tools & Platforms
| Brand | Key Characteristics |
|-------|-------------------|
| cursor | Sleek dark interface, gradient accents |
| expo | Dark theme, tight letter-spacing |
| linear.app | Ultra-minimal, precise, purple accent |
| lovable | Playful gradients, friendly dev aesthetic |
| mintlify | Clean, green-accented, reading-optimized |
| posthog | Playful hedgehog branding, dark UI |
| raycast | Sleek dark chrome, vibrant gradients |
| resend | Minimal dark theme, monospace accents |
| sentry | Dark dashboard, pink-purple accent |
| supabase | Dark emerald theme, code-first |
| superhuman | Premium dark UI, keyboard-first |
| vercel | Black & white precision, Geist font |
| warp | Dark IDE-like, block-based command UI |
| zapier | Warm orange, friendly illustration-driven |

### Infrastructure & Cloud
| Brand | Key Characteristics |
|-------|-------------------|
| clickhouse | Yellow-accented, technical docs |
| composio | Modern dark, colorful icons |
| hashicorp | Enterprise-clean, black & white |
| mongodb | Green leaf branding, dev docs |
| sanity | Red accent, content-first editorial |
| stripe | Signature purple gradients, weight-300 elegance |

### Design & Productivity
| Brand | Key Characteristics |
|-------|-------------------|
| airtable | Colorful, friendly, structured data |
| cal | Clean neutral UI, developer-oriented |
| clay | Organic shapes, soft gradients |
| figma | Vibrant multi-color, playful & professional |
| framer | Bold black & blue, motion-first |
| intercom | Friendly blue, conversational UI |
| miro | Bright yellow accent, infinite canvas |
| notion | Warm minimalism, serif headings |
| pinterest | Red accent, masonry grid |
| webflow | Blue-accented, polished marketing |

### Fintech & Crypto
| Brand | Key Characteristics |
|-------|-------------------|
| coinbase | Clean blue, trust-focused, institutional |
| kraken | Purple-accented dark, data-dense |
| revolut | Sleek dark, gradient cards |
| wise | Bright green accent, friendly & clear |

### Enterprise & Consumer
| Brand | Key Characteristics |
|-------|-------------------|
| airbnb | Warm coral accent, photography-driven |
| apple | Premium white space, SF Pro |
| ibm | Carbon design system, structured blue |
| nvidia | Green-black energy, technical power |
| spacex | Stark black & white, full-bleed imagery |
| spotify | Vibrant green on dark, bold type |
| uber | Bold black & white, tight type |

### Car Brands
| Brand | Key Characteristics |
|-------|-------------------|
| bmw | Dark premium, precise German engineering |
| ferrari | Chiaroscuro editorial, extreme sparseness |
| lamborghini | True black cathedral, gold accent |
| renault | Aurora gradients, zero-radius buttons |
| tesla | Radical subtraction, cinematic photography |

## How This Skill Works

### Step 1: Understand the User's Need

When the user invokes this skill, determine what they want:

- **"Apply X brand"** — They know the brand. Go directly to Step 2.
- **"Show me what's available"** or **"browse brands"** — List categories and brands. Ask which they want.
- **"I want a dark theme for my SaaS"** or **"something minimal"** — Recommend 2-3 brands that fit, let them pick.
- **"Use a design system"** or **"make my UI consistent"** without specifying a brand — Ask about their project's mood/vibe and recommend accordingly.

### Step 2: Load the DESIGN.md

Read the selected brand's `DESIGN.md` from the collection:

```
references/brands/<brand>/DESIGN.md
```

Brand folder names may contain dots (e.g., `linear.app`, `mistral.ai`, `x.ai`). Match case-insensitively.

If the user's requested brand isn't in the collection, check for close matches or inform them which brands are available.

### Step 3: Present a Design System Summary

After reading the DESIGN.md, present a concise summary to the user covering:

1. **Visual identity** — 1-2 sentences on the overall feel
2. **Key colors** — Primary, accent, and background colors with hex values
3. **Typography** — Font family, heading size range, weight usage
4. **Signature technique** — What makes this brand's design unique (e.g., Vercel's shadow-as-border, Notion's warm neutrals, Stripe's weight-300 elegance)
5. **Do's and Don'ts** — The 3-4 most important rules

### Step 4: Apply to Project

Ask the user how they want to proceed:

- **Copy DESIGN.md to project root** — Use `cp` to copy the file. This makes it available to any AI agent working in the project.
- **Read and internalize** — Keep the design system in context for the current session without writing to disk.
- **Generate CSS variables / tokens** — Convert the color palette and typography into CSS custom properties or Tailwind config.
- **Build a specific component** — Use the design system to create a button, card, navigation, or page layout.

### Step 5: Generate UI with the Design System

When generating UI, follow the DESIGN.md's specifications precisely:

- Use the exact hex colors, font sizes, weights, and spacing values from the document
- Follow the Do's and Don'ts section as hard constraints
- Reference the Agent Prompt Guide (section 9) for ready-to-use component prompts
- Apply the component stylings (section 4) for buttons, cards, inputs, etc.
- Respect the layout principles (section 5) for spacing and grid behavior
- Match the depth/elevation system (section 6) for shadows and borders

## DESIGN.md Format Reference

Each DESIGN.md follows this 9-section structure:

| # | Section | What It Contains |
|---|---------|-----------------|
| 1 | Visual Theme & Atmosphere | Mood, density, design philosophy, key characteristics |
| 2 | Color Palette & Roles | Semantic name + hex value + functional role for every color |
| 3 | Typography Rules | Font families, full hierarchy table with sizes/weights/spacing |
| 4 | Component Stylings | Buttons, cards, inputs, navigation — with hover/focus/disabled states |
| 5 | Layout Principles | Spacing scale, grid system, whitespace philosophy |
| 6 | Depth & Elevation | Shadow system, surface hierarchy, border techniques |
| 7 | Do's and Don'ts | Design guardrails and anti-patterns |
| 8 | Responsive Behavior | Breakpoints, touch targets, collapsing strategy |
| 9 | Agent Prompt Guide | Quick color reference, ready-to-use component prompts |

## Recommendations by Project Type

If the user doesn't know which brand to pick, suggest based on their project:

| Project Type | Recommended Brands |
|-------------|-------------------|
| Developer tool / CLI | vercel, linear.app, warp, ollama |
| AI / ML product | claude, voltagent, together.ai, x.ai |
| SaaS dashboard | sentry, posthog, clickhouse |
| Documentation site | mintlify, hashicorp, mongodb |
| Fintech / payments | stripe, revolut, wise |
| Consumer app | airbnb, spotify, notion |
| Creative / design tool | figma, framer, miro, clay |
| Premium / luxury | apple, ferrari, lamborghini, tesla |
| Dark-mode-first product | superhuman, elevenlabs, runwayml |
| Startup landing page | linear.app, vercel, resend |

## Notes

- All DESIGN.md files are extracted from public websites, not provided by the companies themselves. Values may drift as sites update.
- The DESIGN.md is the source of truth; preview HTML files are supplementary visual references.
- These files work with any AI coding agent — Claude Code, Google Stitch, Cursor, Copilot, etc.
