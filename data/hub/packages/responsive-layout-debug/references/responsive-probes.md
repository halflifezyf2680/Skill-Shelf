# Responsive Probes

Use these probes with Playwright after resizing the viewport.

## 1. Target Box Probe

```js
(selectors) => {
  return selectors.map((selector) => {
    const el = document.querySelector(selector);
    if (!el) return { selector, found: false };
    const rect = el.getBoundingClientRect();
    const style = getComputedStyle(el);
    return {
      selector,
      found: true,
      left: Math.round(rect.left),
      top: Math.round(rect.top),
      width: Math.round(rect.width),
      height: Math.round(rect.height),
      overflowY: style.overflowY,
      minHeight: style.minHeight,
      maxHeight: style.maxHeight,
      position: style.position
    };
  });
}
```

## 2. Scroll Owner Probe

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

## 3. Overlap Probe

```js
(selector) => {
  const el = document.querySelector(selector);
  if (!el) return { selector, found: false };
  const rect = el.getBoundingClientRect();
  const points = [
    [rect.left + rect.width / 2, rect.top + 8],
    [rect.left + rect.width / 2, rect.top + rect.height / 2]
  ];
  return points.map(([x, y]) => {
    const topEl = document.elementFromPoint(x, y);
    const style = topEl ? getComputedStyle(topEl) : null;
    return {
      x: Math.round(x),
      y: Math.round(y),
      tag: topEl?.tagName?.toLowerCase() || null,
      className: String(topEl?.className || ""),
      zIndex: style?.zIndex || null,
      position: style?.position || null
    };
  });
}
```

## 4. Stage Ratio Probe

```js
(selector) => {
  const el = document.querySelector(selector);
  if (!el) return { selector, found: false };
  const rect = el.getBoundingClientRect();
  return {
    selector,
    found: true,
    width: Math.round(rect.width),
    height: Math.round(rect.height),
    actualRatio: Number((rect.width / rect.height).toFixed(3))
  };
}
```
