# Layout Diagnosis DSL

Write a compact diagnosis before patching.

## Shape

```json
{
  "page": "InventoryPage",
  "sections": [
    {
      "name": "Right Sidebar Summary",
      "responsibility": "duplicate_summary",
      "verdict": "remove_or_merge",
      "evidence": ["repeats hero stats already shown above", "occupies full column", "content density is low"]
    },
    {
      "name": "Item Grid",
      "responsibility": "primary_content",
      "verdict": "promote",
      "evidence": ["contains the page's main task", "currently compressed by sidebar width"]
    }
  ],
  "skeleton": {
    "before": "header / two-column / footer-tools",
    "after": "header / primary-grid-with-compact-inspector"
  },
  "operations": [
    {
      "target": "section:Right Sidebar Summary",
      "action": "merge_into",
      "anchor": "section:Header Summary"
    },
    {
      "target": "section:Item Grid",
      "action": "promote_to_primary"
    }
  ]
}
```

## Allowed Responsibility Labels

- `summary`
- `primary_content`
- `decision_area`
- `navigation`
- `filters`
- `results_list`
- `inspector`
- `details`
- `progression`
- `support`
- `duplicate_summary`
- `layout_scaffold_only`

## Allowed Verdicts

- `keep`
- `merge`
- `promote`
- `demote`
- `remove_or_merge`
- `split`
- `reflow_only`

## Allowed Operations

- `merge_into`
- `promote_to_primary`
- `remove_section`
- `reflow`
- `replace_duplicate_summary`
- `move_below`
- `move_above`
- `move_into_sidebar`
- `move_out_of_sidebar`
- `change_columns`
- `change_ratio`
- `change_scroll_owner`
- `align_to_anchor`

## Writing Rules

- Keep it short.
- Base every verdict on measured evidence.
- If a section is useless, say so directly.
- If the page skeleton is wrong, describe the new skeleton before patching.
- If the problem is scroll ownership or overlap, name that explicitly.
