---
name: awesome-design-md
description: "Brand design system reference library with 58 company design templates (Airbnb, Stripe, Vercel, Apple, Tesla, etc.). Use this skill whenever the user asks about brand design, design systems, visual identity, color palettes, typography, UI components, or wants to replicate/reference a specific company's design language. Also use when the user mentions 'design system', 'brand guide', 'design tokens', 'visual language', 'company style', or wants to build something that looks like a known product's website."
group: design
---

# Awesome Design MD

A curated library of 58 brand design system documents, each detailing a company's visual language: color palette, typography, spacing, components, and interaction patterns.

## How to use

1. Identify which brand the user wants (e.g., "make it look like Stripe", "use Vercel's design system").
2. Read the corresponding reference file from `references/` using the Read tool.
3. Apply the design tokens, typography, and component patterns described in the reference to the user's project.

The reference files are in `references/` named by brand slug (e.g., `references/stripe.md`, `references/airbnb.md`).

## Available brands (58)

### Tech & SaaS
airtable, airbnb, claude, clickhouse, clay, cohere, composio, cursor, elevenlabs, expo, figma, framer, hashicorp, ibm, intercom, linear.app, lovable, mintlify, minimax, miro, mistral.ai, mongodb, notion, nvidia, ollama, opencode.ai, posthog, raycast, replicate, resend, revolut, runwml, sanity, sentry, spacex, spotify, stripe, supabase, superhuman, together.ai, uber, vercel, voltagent, warp, webflow, wise, x.ai, zapier

### Automotive
bmw, ferrari, lamborghini, renault, tesla

### Finance & Crypto
coinbase, kraken

### Consumer
apple, cal, pinterest

## What each reference contains

Every brand reference follows a consistent structure:

1. **Visual Theme & Atmosphere** — Overall design philosophy and personality
2. **Color Palette & Roles** — Primary, secondary, semantic colors with hex values and CSS variable names
3. **Typography** — Font families, weight scale, size hierarchy, letter-spacing
4. **Spacing & Layout** — Grid system, padding/margin tokens, breakpoints
5. **Component Patterns** — Buttons, cards, inputs, navigation patterns
6. **Motion & Interaction** — Animation timing, easing, hover/focus states

## Usage notes

- These are design **references**, not component libraries. Use them to understand the design language and translate it into your project's framework.
- When a user asks to "make it look like X", read the reference first, then apply the relevant tokens to the code.
- Multiple brands can be combined for inspiration, but avoid directly copying proprietary assets or icons.
- Dark mode variants are included where the brand supports them.
