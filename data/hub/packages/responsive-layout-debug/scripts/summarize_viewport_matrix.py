#!/usr/bin/env python3
"""
Summarize viewport matrix JSON into a compact comparison report.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: summarize_viewport_matrix.py <viewport-report.json>", file=sys.stderr)
        return 2

    path = Path(sys.argv[1]).resolve()
    data = json.loads(path.read_text(encoding="utf-8"))
    rows = list(data.get("viewports") or [])

    if not rows:
      print("No viewport data found.")
      return 0

    print(f"# Viewport Matrix: {data.get('page') or path.name}")
    print()
    for row in rows:
      name = row.get("viewport") or "unknown"
      print(f"## {name}")
      for target in row.get("targets") or []:
        print(
          f"- {target.get('selector')}: "
          f"x={target.get('left')} y={target.get('top')} "
          f"w={target.get('width')} h={target.get('height')} "
          f"overflowY={target.get('overflowY')}"
        )
      print()

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
