# Camera Direction and Physical Validation

Use this reference for any image where composition, viewpoint, action, placement, exact counts, or post-generation review affects success. Keep the resulting prompt concise: select the checks that materially change the image.

## Direct a shot, not a list of objects

Choose the viewer relationship first. Translate it into observable decisions rather than genre labels.

| Intent | Useful visible decisions |
| --- | --- |
| intimacy or psychology | closer frame, stable eye-line, restrained environment, selective focus |
| power or confrontation | controlled height/angle, clear gaze path, stronger silhouette, deliberate space around the subject |
| motion or instability | directional axis, perspective recession, asymmetrical balance, a readable before-and-after path |
| scale or discovery | environmental foreground, smaller subject placement, depth layers, a clear reveal route |
| inspection or design clarity | neutral camera, legible planes, even detail light, reduced foreground obstruction |

Build the frame in this order:

1. Viewer relationship and primary focal subject.
2. Frame boundary, crop, and camera height.
3. Entry point, focal path, and resting or exit space.
4. Planes: foreground, subject plane, setting plane; name their overlap order.
5. Directional vectors from pose, gaze, architecture, light, fabric, vehicles, or repeated forms.
6. Negative space and exclusions that protect the primary read.

Do not treat central placement, symmetry, wide angle, low angle, shallow depth of field, or lens focal length as universal markers of quality. Select them only when they express the requested relationship. State lens behavior only as a visible result: deep recession, intimate compression, natural perspective, or graphic flattening.

## Turn constraints into a spatial model

Before writing the final prompt, resolve only the relations that affect the image:

- **Containment and attachment:** inside, on, under, behind, attached to, emerging from, held by.
- **Support and contact:** what bears weight; where hands, feet, wheels, objects, fabric, or bodies touch; which direction force or motion travels.
- **Depth and occlusion:** what overlaps what; what may be hidden; what must remain visible; what must not enter at a frame edge.
- **Counts and repeated structures:** exact number, common source, grouping, depth layers, readable separation, and overlap permission.
- **Scale and perspective:** compatible sizes, horizon/camera relation, and convergence behavior across important objects.
- **Light and materials:** source direction, blocked light, contact shadow, reflection, and surface response consistent with the scene.

Prefer a small number of relational statements over a long negative prompt. For instance, describe a repeated feature by its attachment, three depth groups, and focal exclusion zone, rather than demanding every instance be equally visible.

## Feasibility gate

Silently test the following before issuing the prompt:

- Can a single camera at the named position see every required visible feature without impossible transparency or contradictory cropping?
- Does every supported subject or prop have a plausible bearing surface, grip, suspension, or explicitly stated nonphysical cause?
- Do pose, gaze, hand contact, object path, and clothing deformation agree at the same captured instant?
- Can the count and identity of repeated elements be read at the requested distance without overwhelming the focal subject?
- Do light direction, shadow, reflection, and material behavior agree with the setting and degree of stylization?

If a requirement fails, repair the spatial model or declare the least disruptive tradeoff in the prompt. Do not bury the contradiction in extra adjectives.

## Surface integrity and repeated-detail control

For water, sky, fog, broad walls, fabric fields, skin, snow, clouds, or distant terrain, establish the large surface behavior before adding fine detail. A mark is valid only when it has a physical source, attachment or area, directional logic, scale, and limited density.

- **Water:** State the dominant plane, current or wave direction, reflection zones, and any local disturbance with its source. Use grouped ripples and broken reflections that follow the plane; do not fill water with unrequested spiderweb lines, crackle meshes, branching lightning veins, or repeated texture stamps.
- **Distant terrain and atmosphere:** Build distance from varied silhouettes, value groups, overlap, and reduced contrast. Use a few non-identical landforms, not cloned fracture maps, tiled foliage, identical cloud cells, branching line nets, or lightning-like marks. Distance may simplify detail, but must not become a filler field of blur, noise, or repeated motifs.
- **Large simple surfaces:** Keep calm areas calm. Do not add floating threads, random fibers, pseudo-cracks, dense speckles, or decorative branching unless the user asked for a named source such as cobwebs, lightning, roots, veins, frost, a specific fabric weave, or an actual cracked material.

When an output shows these artifacts, diagnose the missing surface grammar rather than merely adding “no noise.” Rewrite the prompt with the large-surface behavior and a short, specific exclusion for the failure mode.

## Rendered-output review

When an image has been generated or supplied for review, judge pixels rather than intent. Check in order:

1. **Shot:** Does the actual crop, viewpoint, focal path, and empty space express the desired viewer relationship?
2. **Hierarchy:** Is the focal subject unmistakable, with secondary elements supporting rather than competing?
3. **Geometry:** Are anatomy, attachment, support, contact, perspective, scale, and occlusion believable?
4. **Lighting:** Are source, shadows, reflections, and materials mutually consistent?
5. **Hard constraints:** Are counts, literal details, text, identity, and exclusions actually satisfied?
6. **Surface integrity:** Do water and large surfaces follow their plane and material behavior? Are the distant layers built from silhouette, value, and overlap rather than web-like filler, lightning-shaped branches, noise, or duplicated texture stamps?

If a visible failure remains, identify the smallest causal correction—camera/crop, pose, depth layer, attachment, grouping, light source, or literal constraint—and rewrite the complete prompt. Never call a render compliant merely because the prompt contained the requirement.
