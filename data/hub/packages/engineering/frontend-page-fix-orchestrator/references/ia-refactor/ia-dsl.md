# IA Refactor DSL

Write a compact page-model diagnosis before patching.

## Shape

```json
{
  "page": "CharacterPage",
  "contract": "show the full progression overview and let the user inspect build-defining choices",
  "current_skeleton": "top-summary / mid-two-column / bottom-three-column",
  "target_skeleton": "hero-summary / attributes-and-growth / skills-and-specialization",
  "sections": [
    {
      "name": "Survival And Combat",
      "verdict": "remove_or_merge",
      "reason": "repeats top summary and occupies disproportionate space"
    },
    {
      "name": "Skills",
      "verdict": "promote",
      "reason": "contains the page's build-defining choices"
    }
  ]
}
```

## Allowed Verdicts

- `keep`
- `promote`
- `demote`
- `merge`
- `remove_or_merge`
- `split`

## Writing Rules

- State the page contract first.
- Keep the skeleton names simple.
- Be direct about useless sections.
