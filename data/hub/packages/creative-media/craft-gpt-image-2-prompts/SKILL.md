---
name: craft-gpt-image-2-prompts
description: Design, expand, diagnose, and rewrite prompts specifically for GPT Image 2 and ChatGPT Images 2.0. Turn visual intent into controllable art direction with professional camera language, composition, and physically coherent spatial constraints. When a user specifies a consequential visual style, first interpret and confirm a reusable Style lock before drafting prompts. Use when a user provides a few visual keywords, a rough image idea, a character concept, an existing image to edit, reference images to combine, an underperforming image prompt, or asks for a complete controllable prompt for gpt-image-2. The primary deliverable is a polished prompt, not an automatically generated image.
---

# Craft GPT Image 2 Prompts

Turn sparse visual intent into one coherent, editable GPT Image 2 prompt. Supply the art-direction decisions the user omitted without making the user complete a long questionnaire.

## Two-stage workflow

Use two stages whenever the user names, implies, or combines a consequential visual style (for example, anime 3D character design, photorealistic editorial photography, a game-engine render, painterly concept art, or any hybrid). Do not skip Stage 1 merely because the scene brief is otherwise complete.

### Stage 1 — Style alignment

Before writing a final image prompt or generating an image:

1. Interpret the requested style as visible production decisions, separating:
   - design language (for example: anime character design vs. live-action human design)
   - rendering medium and material behavior (for example: 3D PBR render, watercolor, cel shading)
   - realism/stylization level
   - lighting, contrast, and palette behavior
   - likely visual failure modes created by the requested combination
2. Return a compact **Style read** of three to five bullets. State what the image will visibly look like; do not give generic style definitions.
3. Return one concise **Style lock** prompt block. It must contain only style decisions that should remain invariant across later images. Keep it separate from scene, character, action, camera, and story details.
4. Ask one direct confirmation question: whether to lock this style for the next prompt(s). Do not write the final scene prompt, edit prompt, or call image generation in this stage.

If the style wording contains a material conflict or ambiguity, explain the conflict in the Style read and make the Style lock resolve it explicitly. For example, distinguish a "photoreal person rendered in an engine" from an "anime-designed character rendered in 3D"; never treat those as interchangeable.

### Common-style lookup

Read [references/common-visual-styles.md](references/common-visual-styles.md) whenever the user names a common visual style, asks what style to use, or asks for a style without supplying a reference image. Match the user's wording to the listed name or alias, then turn the card's visible decisions into the Stage 1 Style lock. A named catalog style is enough to begin Stage 1; do not ask the user to explain its features.

Use the catalog as a vocabulary and decision aid, not as a list of fixed scene templates. Transfer only the card's rendering language, edge behavior, palette logic, and detail hierarchy. Keep the user's subject, setting, era, composition, and text separate. If no card matches, analyze the user's wording normally. If two cards materially differ, name the distinction and ask the single Stage 1 confirmation question.

### No-style and browse behavior

If the user explicitly asks to list styles, browse styles, or asks “有哪些风格/什么风格可选,” read the `目录速览` in `references/common-visual-styles.md` and return it as a compact categorized list. Do not create a Style lock, final scene prompt, or generated image in response to a catalog-only request. If they ask for the detail of one named style, return that card's visible decisions and then offer the Stage 1 Style lock for confirmation.

If an image request contains no named, implied, or reference-image style, do not silently impose a consequential style. Read the catalog, select three to five materially distinct styles that fit the depicted subject, and return a short **风格选择** list: each option must include the catalog name and a one-line visible result. Ask one direct question to choose one option or say “列出风格” for the full catalog. Do not write a final image prompt before a style is selected and locked.

When the user selects a style from the menu, continue with ordinary Stage 1: use that card, show `Style read` and the proposed `Style lock`, then ask for confirmation. The catalog list itself is not a style confirmation.

### Stage 2 — Prompt production

Begin only after the user confirms the Style lock. Treat the confirmed block as immutable context for the current visual sequence. Insert it verbatim under `Style lock:` in every final prompt, then add only the scene- or shot-specific decisions needed for the request.

If the user changes a style-critical decision, revise the Style lock and return to Stage 1. If they change only scene, action, camera, or subject details, retain the confirmed Style lock and continue directly in Stage 2. Never quietly add a conflicting style term later in a prompt.

## Operating contract

- End with a prompt by default. Do not invoke image generation unless the user separately and explicitly asks to generate an image.
- When the two-stage workflow applies and no Style lock has been confirmed in the current visual sequence, return Stage 1 instead of a final prompt, even if image generation was requested.
- When revising an existing Stage 2 prompt for any user correction, return one complete, copy-ready replacement prompt. Never return only a replacement sentence, patch, or instruction to locate text in the prior prompt.
- Never respond to sparse keywords by merely restating or comma-joining them.
- Infer ordinary visual decisions. Ask at most one question, and only when two plausible answers would produce fundamentally different images.
- Produce one strong primary prompt. Do not create a batch of alternatives unless requested.
- Keep the user's fixed facts fixed. Mark only consequential inferred choices in the brief summary so the user has a clear correction point.
- Do not expose slot-filling analysis, chain-of-thought, generic prompting lessons, or a second bilingual copy unless requested.
- Write the final prompt in the user's language. Preserve literal on-image text exactly as supplied.

## Route the request

After Stage 1 is complete when it applies, choose one mode before drafting:

1. **New generation** — Create a scene from text.
2. **Character or portrait** — Prioritize identity, pose, expression, anatomy, styling, and believable human detail.
3. **Image edit** — Change a bounded part of an existing image while protecting invariants.
4. **Multi-image reference** — Assign an explicit role to each reference image and define how they combine.
5. **Prompt repair** — Diagnose a weak prompt or disappointing output and rewrite the full prompt with the smallest useful changes.
6. **Text-bearing visual** — Treat exact copy, hierarchy, typography, and placement as first-class constraints.

Read [references/prompt-patterns.md](references/prompt-patterns.md) for the selected mode. Read [references/camera-and-physical-validation.md](references/camera-and-physical-validation.md) whenever composition, viewpoint, action, exact counts, object placement, multiple subjects, or review of a generated image matters. Read [references/examples.md](references/examples.md) only when sparse-input behavior or output formatting is unclear.

## Build the art direction

### 1. Preprocess for visual economy

Before expanding the brief, reduce signals that make GPT Image 2 overfit to surface texture, ornamental density, or a reference image's incidental content. Do this silently; preserve user-required details.

- Identify the image's focal stack: one subject, one dominant silhouette or action, and one setting anchor. Rich costumes may use several detail systems, but assign them to readable zones—base garment, structural layers, and ornament—rather than making jewelry, particles, filigree, architecture, fabric, and magic effects all compete at equal importance.
- Convert repeated texture adjectives into one material decision. For example, replace a stack such as “ornate, intricate, rich, detailed, shimmering, elaborate” with one observable instruction such as “selective gold embroidery at the collar and belt.”
- Set a texture budget by zone, not by accessory count. A costume may be richly layered, embroidered, jewelled, and accessorized when requested; give each zone one dominant material/readability rule, group repeated motifs, and vary pattern scale so its construction remains legible. Reserve uncontrolled micro-detail and repeated specular noise for nowhere.
- Give every fine surface mark a source, attachment, direction, scale, and local density. Do not use unrequested branching filaments, web-like line networks, crackle meshes, lightning-shaped veins, or repeated micro-patterns as generic filler on water, sky, fog, skin, cloth, broad walls, or distant terrain. Water must read first as coherent planes, wave direction, reflections, and source-bound ripples; distant terrain must read first as varied silhouette and value groups, not copied texture stamps.
- For environments, distinguish authored richness from modular repetition. Preserve the scene's complexity, but rebuild its structural rhythm: give architecture unequal silhouettes, spans, proportions, functions, spacing, weathering, and occlusion; organize terrain as a few varied landforms with different fracture scales. Do not fix visible copy-paste repetition merely by deleting elements, blurring with fog, or making the setting ruined.
- State a hierarchy when density is requested: keep the face and silhouette legible first; let the costume carry rich, structured detail across its defined layers; simplify distant architecture, crowd, particles, and trim. Prefer varied scales, calm resting areas, and deliberate contrast over uniform sharp detail everywhere.
- Treat style references as style-only unless the user explicitly wants their content. Extract rendering traits, value range, linework, palette behavior, and material treatment; explicitly prohibit copying the reference's subject, costume, setting, pose, palette, or composition when those are not requested.
- When the user asks for random variants from a reference, randomize the semantic axes deliberately: subject identity, role, setting, costume silhouette, palette family, time/weather, and camera distance. Do not ask the model to “randomize” while leaving reference-content leakage unbounded.
- Do not use cumulative booster language such as “ultra-detailed,” “exquisite,” “intricate,” “luxurious,” and “rich texture” together. Keep only terms that change a visible decision.
- When a Style lock exists, keep its concepts out of this expansion step unless they need to be copied verbatim. Do not re-interpret or intensify the locked style while adding scene detail.

### 2. Extract the fixed brief

Privately identify:

- intended use and image type
- subject, setting, and depicted moment
- required objects, clothing, era, culture, and visible text
- requested medium, mood, palette, composition, and aspect ratio
- reference-image roles
- required changes, preserved elements, and exclusions

Treat explicit user details as constraints, not inspiration to replace.

### 3. Form one visual thesis

Choose a single governing idea for the picture: what the viewer notices first, what emotion the image carries, and what makes this rendering specific rather than generic. Resolve missing choices in service of that thesis.

- Prefer one coherent medium and art direction over a stack of fashionable style words.
- Convert abstractions into visible evidence. Express “lonely” through distance, gaze, posture, empty space, weather, or human traces rather than relying on the adjective alone.
- Choose a concrete narrative instant. Depict what the subject is doing at the captured moment.
- Add only details that reinforce subject, story, scale, culture, or mood.
- Avoid filler such as “masterpiece,” “best quality,” “8K,” or long generic negative-prompt inventories unless a visible requirement justifies it.
- Describe the desired visual effect of camera and optics. Do not pretend exact lens metadata guarantees physical simulation.

### 4. Direct the shot before decorating it

For every scene, character, and composition-sensitive image, make a shot-direction pass before adding atmosphere or surface detail. Convert the requested feeling into observable camera and layout decisions; do not use words such as “cinematic” or “professional” as substitutes for them.

- Define the viewer relationship: intimate observation, confrontation, surveillance, spectacle, discovery, vulnerability, product inspection, or another scene-appropriate relation. Let it determine framing distance, camera height, and angle.
- Define a focal path in viewing order: entry point, primary subject, secondary reveal, and exit or resting space. Give each large element one compositional job; remove or subordinate elements without one.
- Choose a frame grammar appropriate to the brief: crop and aspect ratio, primary axis or directional flow, subject placement, foreground/midground/background separation, purposeful negative space, and overlap order. Use asymmetry only when it serves tension or movement; use symmetry only when it serves ritual, control, calm, or formal display.
- State what is intentionally outside the frame. Do not ask for every detail, limb, prop, or repeated element to be equally prominent when the shot needs hierarchy.
- Describe camera behavior through visible outcomes—compression versus depth, eye-line relationship, perspective emphasis, and amount of environment—not arbitrary focal-length numbers. Name a lens only when its observable spatial behavior is required.
- For repeated or decorative elements, assign their compositional role, grouping, depth layer, and exclusion zone around the focal subject. Give environmental repetitions a non-uniform structural rhythm—variation in silhouette, scale, spacing, interruption, and occlusion—rather than a continuous cloned sequence.

### 5. Validate spatial and physical coherence

Before finalizing a prompt, perform a silent feasibility pass. Treat the image as a single frozen moment that must still make physical, optical, and anatomical sense. Read the validation reference for the full checklist.

- Resolve a compact spatial model: which subject or object is in front of, behind, on, inside, attached to, held by, supported by, or looking toward each other relevant element.
- Check support and force: feet, seats, hands, carried objects, cloth, hair, water, smoke, and suspended elements need a plausible support, contact point, direction, or stated supernatural cause.
- Check camera visibility: the requested crop, viewpoint, occlusion order, and required details must be simultaneously possible. If not, prioritize the user’s declared focal requirement and revise the shot rather than stacking contradictory visibility demands.
- For quantities or repeated parts, specify the count, attachment/source, grouping, readable separation where needed, and which may overlap or leave frame. For environmental repetition, also state what makes instances visibly non-identical: role, scale, spacing, damage, weathering, or occlusion. Do not rely on a bare numeral to control structure.
- Make light physically consistent: source direction, occlusion, contact shadows, reflections, and material response must agree. Use intentional stylization only when the brief calls for it.
- When asked to generate or review an output, inspect the rendered image against the same acceptance criteria. Do not infer success from the prompt alone; identify and repair visible violations before presenting it as compliant.

### 5a. Close every depicted interaction before writing it

For each stated action, construct a single-frame, self-contained staging model before decoration. An interaction is valid only if all of its visible participants, prop path, contact points, gaze target, and frame boundary can coexist in the selected crop.

- Do not write a handoff such as “a vendor gives her a sachet” unless both giver and receiver are intentionally visible, with the giver’s body and hand assigned a valid place in frame. A named but off-screen giver cannot provide a visible prop.
- If the composition protects a solo portrait or excludes third-party limbs, resolve the object before the captured moment: use wording such as “she has already selected/purchased/is carrying the sachet,” then show only the subject’s own hand holding it.
- Do not combine “no third-party limbs or partial figures at the frame edge” with an action that requires an unseen person’s hand. Reframe the action, widen the shot to include the other actor, or change the prop to a self-contained state.
- Treat gaze as an action target. State whether the subject looks at a held object, a visible person, a known off-screen presence whose body is absent, or the viewer; do not leave a gaze toward an unspecified source.
- For two-handed poses, account for both hands separately and make their contacts compatible with the garment, carried object, and crop. Do not make a hand both hold an object and perform an incompatible gesture.
- If an implied action cannot be frozen legibly in the requested portrait crop, choose a stable aftermath, preparation, or pause rather than an in-progress exchange.

### 6. Complete the visible decisions

Use a consistent order: style lock → purpose and scene → subject and action → composition → lighting and color → medium and surface qualities → decisive details → constraints.

The Style lock controls only invariant style decisions. Do not let it replace concrete scene, character, action, or camera instructions.

For photorealism, include the term `photorealistic` directly and specify believable light, materials, skin, fabric, scale, and environmental interaction. For illustration or design, name the medium and its observable properties instead of vaguely requesting “artistic.”

For a person, make the character identifiable rather than generically attractive:

- specify apparent age range, face shape, a few distinctive features, hair behavior, expression, and gaze
- define body framing, pose, weight distribution, hand or object interaction, and whether feet must be visible
- retain natural asymmetry, skin texture, fabric tension, flyaway hair, or contextual wear when realism is intended
- avoid beauty homogenization, porcelain skin, frozen fashion poses, or glamour lighting unless requested
- keep ancestry, identity, body type, disability, scars, or other user-defined traits exact; do not silently normalize them

For a sparse brief, infer composition from intent: favor portrait framing for a single-character study, a wider frame for environmental storytelling, and clear negative space when the image is meant to carry copy. Do not state an aspect ratio unless it materially helps the layout or the user asked for it.

### 7. Stage interactions and frame boundaries

For any action involving an object, another actor, a vehicle, an animal, or an off-screen source, make the action physically closed and visually accountable. State who performs it, where the object is before and after the action, and which body parts or actors may appear in frame.

- Do not leave a handoff, reception, touch, rescue, pursuit, attack, or gaze target implicit. Name the visible giver and receiver, or explicitly make the object self-propelled, suspended, or already in place.
- If a source remains outside frame, prohibit its body parts, tools, shadows, and silhouettes from entering the composition. Describe only the visible trace that establishes the source, such as a light trail, wake, signal, or motion path.
- When a person and object interact, specify the holding hand, contact point, pose, and the object's location relative to the body. Do not combine incompatible instructions such as "receives from afar" and "is handed by someone" unless both actors are intentionally visible.
- When a companion, creature, or vehicle is present, assign it a clear spatial role and interaction target so it is not replaced by an unintended extra character.
- Add a short boundary constraint only when needed: for example, "no third-party limbs or partial figures entering from the frame edge."

### 8. Express constraints precisely

Use natural-language constraints, not a detached “negative prompt” dump.

- For new images, exclude only likely visible failure modes or prohibited additions.
- For edits, state `change only X`, then explicitly list what must remain unchanged.
- For repeated edits, restate invariants every time to prevent drift.
- For exact image text, put the text in quotation marks, state that it must appear verbatim, and specify hierarchy, placement, color, and legibility.
- For multiple references, label them by index and role, then specify source, destination, scale, perspective, lighting, and shadows.

### 9. Quality-check the prompt

Before answering, verify:

- The image has a focal subject, a legible depicted moment, and a coherent visual thesis.
- Every concrete user requirement appears once and is not contradicted later.
- Composition, gaze, hands, object interactions, and spatial relations are physically understandable.
- The shot has a deliberate viewer relationship, focal path, frame grammar, and depth/overlap order rather than a centered inventory of requested elements.
- Required features can coexist in the named crop and viewpoint; repeated elements have a count, source, grouping, and legible spatial role where this matters.
- Support, contact, scale, perspective, occlusion, and light direction remain plausible within the chosen degree of stylization.
- Every interaction has accountable visible participants and a valid object path; no unspecified actor, limb, or edge intrusion is needed to complete the action.
- Style, lighting, palette, materials, and era agree with one another.
- When present, the Style lock appears verbatim and no later clause contradicts its design language, rendering medium, stylization level, or lighting behavior.
- Texture and ornament follow a visible hierarchy: the costume may be elaborate, but its layers, motifs, and material zones remain readable; the subject reads at a glance, and distant areas are allowed to stay simple.
- Environment detail is authored rather than modular: no tiled rock or snow noise, long cloned stair/railing/column/roof sequences, or equally weighted kitbash fragments. The repair retains successful richness and does not substitute object deletion, generic haze, or artificial ruin for structural variation.
- Large quiet surfaces, water, sky, fog, and distant terrain remain structurally clean: any visible fine mark has a named physical source and direction; no unrequested webbing, crackle mesh, lightning-like branching, fractal veins, or copied texture stamps are filling low-information areas.
- Reference images contribute only the roles explicitly assigned to them; when used for style, their incidental character, setting, palette, pose, and composition do not leak into the new image.
- Constraints protect real failure points without smothering useful creative latitude.
- The prompt is skimmable and self-contained; delete ornamental adjectives that do not change pixels.

## Output contract

For **Stage 1**, return this compact structure in the user's language:

**Style read** — Three to five concrete visible consequences and any material ambiguity.

**Style lock**

```text
[the invariant style prompt block]
```

**Confirm** — One direct question asking the user to confirm or revise the Style lock.

For **Stage 2**, return this compact structure in the user's language:

**创作方向** — One sentence naming the inferred visual thesis and any consequential assumption.

**最终提示词**

```text
Style lock:
[the confirmed style prompt block, verbatim]

[the complete scene/character/edit prompt]
```

Add **可纠偏项** only when two to four inferred decisions are genuinely worth exposing. Write them as short, directly editable choices. Do not append an offer to generate the image.

For prompt repair or any user-directed revision of an already written Stage 2 prompt, precede the replacement with **问题定位** containing at most three observable problems when useful. Always provide the full revised prompt, not only a delta. If repairing a style-critical failure, revise and reconfirm the Style lock before producing the repaired prompt.
