# Interaction Patterns

Use interaction to clarify system behavior.

## Good Patterns

- Hover a flow node to highlight its upstream and downstream edges.
- Click a candidate to move it into a kept evidence list.
- Use tabs for `overview / evidence / timeline`.
- Use segmented controls for `before / after` or `wide recall / strict evidence`.
- Use sticky side navigation for long explainers.

## Avoid

- Decorative animations that do not explain a state change.
- Large hero-only pages for tools or technical products.
- Hidden critical content that requires hover on mobile.
- Cards inside cards.

## Motion

- Use `transition: transform 180ms ease, opacity 180ms ease`.
- Prefer transform and opacity.
- Keep animated elements in stable containers so layout does not shift.
