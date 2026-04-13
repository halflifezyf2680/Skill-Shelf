# Playwright Probes

Use these probes with `browser_evaluate` or `browser_run_code`.

## 1. Section Box Probe

Use when you need left/right/top/width/height facts.

```js
() => {
  const nodes = Array.from(document.querySelectorAll("header, main, section, aside, nav, footer, [data-testid], [role='region']"));
  return nodes
    .map((el, index) => {
      const rect = el.getBoundingClientRect();
      const title =
        el.querySelector("h1,h2,h3,h4,h5,h6,[aria-label]")?.textContent?.trim() ||
        el.getAttribute("aria-label") ||
        el.getAttribute("data-testid") ||
        `section-${index}`;
      return {
        title,
        tag: el.tagName.toLowerCase(),
        left: Math.round(rect.left),
        top: Math.round(rect.top),
        width: Math.round(rect.width),
        height: Math.round(rect.height)
      };
    })
    .filter((row) => row.width > 0 && row.height > 0);
}
```

## 2. Parent Layout Probe

Use when you need to know whether siblings share the same grid or flex row.

```js
() => {
  return Array.from(document.querySelectorAll("section, aside, main, nav"))
    .map((el, index) => {
      const parent = el.parentElement;
      const title =
        el.querySelector("h1,h2,h3,h4,h5,h6,[aria-label]")?.textContent?.trim() || `section-${index}`;
      if (!parent) return { title, parent: null };
      const style = getComputedStyle(parent);
      return {
        title,
        parentTag: parent.tagName.toLowerCase(),
        parentClass: String(parent.className || ""),
        display: style.display,
        gridTemplateColumns: style.gridTemplateColumns,
        gridAutoFlow: style.gridAutoFlow,
        flexDirection: style.flexDirection,
        alignItems: style.alignItems,
        justifyContent: style.justifyContent
      };
    });
}
```

## 3. Stretch And Empty-Ratio Probe

Use when one side looks suspiciously empty or inflated.

```js
() => {
  return Array.from(document.querySelectorAll("section, aside, main"))
    .map((el, index) => {
      const rect = el.getBoundingClientRect();
      const title =
        el.querySelector("h1,h2,h3,h4,h5,h6,[aria-label]")?.textContent?.trim() || `section-${index}`;
      return {
        title,
        height: Math.round(rect.height),
        scrollHeight: el.scrollHeight,
        clientHeight: el.clientHeight,
        emptyRatio:
          rect.height > 0 ? Number(Math.max(0, 1 - el.scrollHeight / rect.height).toFixed(2)) : 0
      };
    })
    .filter((row) => row.height > 0);
}
```

## 4. Scroll Ownership Probe

Use when a panel should scroll or should not scroll.

```js
() => {
  return Array.from(document.querySelectorAll("*"))
    .map((el) => {
      const style = getComputedStyle(el);
      if (!["auto", "scroll"].includes(style.overflowY)) return null;
      return {
        tag: el.tagName.toLowerCase(),
        className: String(el.className || ""),
        clientHeight: el.clientHeight,
        scrollHeight: el.scrollHeight,
        canScroll: el.scrollHeight > el.clientHeight,
        overflowY: style.overflowY
      };
    })
    .filter(Boolean);
}
```

## 5. Overlap Probe

Use when something is covered, clipped, or impossible to click.

```js
() => {
  const points = [
    [window.innerWidth * 0.5, window.innerHeight * 0.5],
    [window.innerWidth * 0.75, window.innerHeight * 0.5],
    [window.innerWidth * 0.9, window.innerHeight * 0.5]
  ];
  return points.map(([x, y]) => {
    const el = document.elementFromPoint(x, y);
    if (!el) return { x, y, element: null };
    const style = getComputedStyle(el);
    return {
      x: Math.round(x),
      y: Math.round(y),
      tag: el.tagName.toLowerCase(),
      className: String(el.className || ""),
      zIndex: style.zIndex,
      position: style.position,
      pointerEvents: style.pointerEvents,
      text: (el.textContent || "").trim().slice(0, 60)
    };
  });
}
```

## Recommended Probe Order

1. Section box probe
2. Parent layout probe
3. Stretch and empty-ratio probe
4. Scroll ownership probe
5. Overlap probe when needed

Take a screenshot only after these facts are collected.
