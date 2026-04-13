# Triage Matrix

Use this matrix to choose the lead skill.

## Start With `frontend-layout-diagnose`

When the complaint sounds like:

- "对不齐"
- "被挡住了"
- "空得离谱"
- "滚不动"
- "贴左/贴右"
- "这一块太宽/太窄"

Signal:

- the problem appears to be on the current viewport already
- the user points at a specific region, edge, panel, or overlap

## Start With `responsive-layout-debug`

When the complaint sounds like:

- "切换分辨率没用"
- "16:9 和 4:3 看起来一样"
- "桌面正常，平板不正常"
- "只有某个窗口大小下出问题"
- "横屏/竖屏表现不对"

Signal:

- same page behaves differently across widths or heights
- aspect ratio, breakpoint, sticky, or scroll ownership depends on viewport

## Start With `frontend-ia-refactor`

When the complaint sounds like:

- "这栏有什么用"
- "整个页面很怪"
- "这一堆应该打散重排"
- "这里重复了"
- "主任务被埋了"

Signal:

- the page's section model is suspicious
- the user questions why a section exists at all

## Combined Cases

- if the user questions section purpose and also reports a viewport-specific failure:
  start with `responsive-layout-debug`
- if the user questions section purpose and the bug is clearly present on the current viewport:
  start with `frontend-ia-refactor`
- if the page structure is already acceptable and only edges, overlap, or scrolling feel wrong:
  start with `frontend-layout-diagnose`
