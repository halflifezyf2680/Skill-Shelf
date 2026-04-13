# Viewport Matrix

Choose the smallest matrix that can expose the bug.

## Common Presets

- narrow mobile: `390 x 844`
- wide mobile landscape: `844 x 390`
- tablet portrait: `768 x 1024`
- tablet landscape: `1024 x 768`
- laptop 16:10: `1440 x 900`
- desktop 16:9: `1600 x 900`
- classic 4:3: `1440 x 1080`

## Selection Rules

- Always include the failing viewport.
- Include one nearby viewport to reveal the breakpoint edge.
- If the issue is about aspect ratio, compare two viewports with similar width but different height ratio.
- If the issue is desktop-only, compare at least one narrower desktop width and one wider desktop width.
- If the issue is mobile-only, compare portrait and landscape.

## What To Record

- page size
- target section boxes
- scroll owner
- whether content is clipped
- whether sticky or fixed layers overlap targets
