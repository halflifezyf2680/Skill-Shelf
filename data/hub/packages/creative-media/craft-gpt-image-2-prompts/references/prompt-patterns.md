# GPT Image 2 prompt patterns

Use these as compositional patterns, not rigid forms. Omit fields that do not affect the requested image. Prefer short labeled sections for complex prompts and natural paragraphs for simple ones.

## New generation

```text
Create [intended image/use].

Scene and moment:
[setting, time, subject action, narrative instant, spatial context]

Subject:
[appearance, clothing/materials, expression, gaze, pose, object interaction]

Composition:
[viewer relationship; framing/crop; camera height and perspective behavior; focal path; subject placement; foreground/midground/background; overlap order; purposeful negative space; what stays outside frame]

Lighting and color:
[light source and quality, atmosphere, palette, contrast]

Medium and finish:
[photograph/illustration/3D/design medium plus observable texture and rendering traits]

Decisive details:
[only details that make the result specific]

Constraints:
[literal requirements; spatial, contact, quantity, and visibility constraints; exclusions; no unrequested text/watermark when relevant]
```

## Character or portrait

Build around identity and behavior rather than a beauty-word list.

```text
Create [portrait/character image and intended use].

Character:
[apparent age, identity traits, face shape, 2–4 distinctive features, hair, body type, clothing and material behavior]

Captured moment:
[specific action, expression, gaze, posture, weight distribution, hand/object interaction; for any exchange or off-screen cause, name visible participants, object path, and whether no outside limbs/figures may enter frame]

Setting:
[environment and the few details that reveal story, scale, era, or culture]

Composition:
[viewer relationship; body framing and intentional crop; camera height and perspective behavior; subject placement; focal path; foreground/midground/background; overlap order; negative space; what is deliberately outside frame]

Light, color, and medium:
[coherent light logic, palette, photograph or illustration language, surface texture]

Human realism or stylization:
[natural asymmetry and lived-in cues for realism, or explicit shape/line/paint logic for illustration]

Constraints:
[identity invariants; anatomy-sensitive, support, contact, quantity, attachment, and visibility relations; required/forbidden additions]
```

## Image edit

```text
Edit the provided image.

Change only:
[precise target area and requested transformation]

Preserve exactly:
[identity, facial features, pose, geometry, camera angle, framing, background, layout, text, brand elements, or other invariants]

Integration requirements:
[matching perspective, scale, occlusion order, support/contact geometry, material behavior, lighting direction, color temperature, contact shadows, reflections]

Do not:
[short list of realistic drift risks]
```

For an iterative edit, repeat the preserve list even if it appeared in an earlier turn.

For a scene whose detail feels copied and pasted, preserve the successful scene, density, and focal structure; change the *structural rhythm*, not merely the object count. Specify which repeated families must become distinct (for example roof silhouettes, bays, railings, stair segments, rock fracture scales, snow patterns), how they should vary (role, scale, spacing, weathering, interruption, occlusion), and what must not happen (no generic blur/fog, no ruinification, no simplified empty scene).

## Multi-image reference or compositing

```text
Use the supplied references as follows:
- Image 1 — [identity/base scene/composition role]
- Image 2 — [clothing/object/style/pose role]
- Image 3 — [additional role]

Create/edit:
[what the final image must depict]

Combine them by:
[what transfers from each source, where it goes, and what does not transfer]

Match:
[scale, perspective, depth/occlusion order, pose and contact geometry, lighting, color temperature, shadows, reflections, texture]

Preserve:
[base-image and identity invariants]

Constraints:
[no extra elements, text, logos, or unwanted redesign]
```

Never say only “use the same style.” Name the visible properties to transfer: line weight, shape language, palette, contrast, brush texture, material response, lighting, or layout rhythm.

## Text-bearing visual

```text
Create [poster/ad/cover/infographic/UI image] for [audience and use].

Visual concept:
[scene, subject, brand or communication idea]

Layout:
[format, grid or hierarchy, subject placement, text-safe areas, margins]

Include only this visible text, verbatim:
- Primary: "[EXACT COPY]"
- Secondary: "[EXACT COPY]"

Typography:
[type character, weight, size relationship, color, alignment, placement, contrast]

Style and finish:
[medium, palette, material and production cues]

Constraints:
[no other text, spelling must remain exact, no watermark, any brand invariants]
```

For uncommon spellings, optionally add a letter-by-letter spelling after the verbatim copy.

## Prompt repair

Diagnose only observable causes: contradictory art direction, absent camera intent, incoherent focal path, ambiguous crop or perspective, missing spatial relations, unsupported contact, unstructured repeated elements, generic subject definition, ambiguous action, absent edit invariants, overlong exclusions, or competing focal points. Preserve successful clauses, repair the weakest decisions, and return a complete prompt using the appropriate pattern above.
