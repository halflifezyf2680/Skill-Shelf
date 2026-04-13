# Responsive Diagnosis DSL

Write a compact diagnosis before patching.

## Shape

```json
{
  "page": "PlayPage",
  "targets": ["stage", "right-panel", "history-panel"],
  "matrix": ["1600x900", "1440x900", "1440x1080"],
  "failure_mode": "min_height_dominates_ratio",
  "evidence": [
    "preset changes from 16:9 to 4:3",
    "stage width remains constant",
    "stage min-height prevents height from following ratio"
  ],
  "operation": {
    "target": "stage",
    "action": "remove_dominant_min_height"
  }
}
```

## Allowed Failure Modes

- `breakpoint_mismatch`
- `ratio_switch_not_wired`
- `min_height_dominates_ratio`
- `fixed_height_clips_content`
- `wrong_scroll_owner`
- `flex_stretch_pathology`
- `grid_ratio_pathology`
- `sticky_overlap`
- `hidden_overflow_clipping`

## Writing Rules

- Mention the failing viewport matrix explicitly.
- Name the dominant rule that causes the bug.
- Prefer one governing fix over multiple cosmetic fixes.
