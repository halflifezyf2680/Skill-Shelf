#!/usr/bin/env python3
"""
Summarize a raw layout report JSON into compact findings.

Expected input:
{
  "page": "InventoryPage",
  "sections": [
    {
      "title": "Right Sidebar Summary",
      "left": 940,
      "top": 180,
      "width": 280,
      "height": 720,
      "scrollHeight": 168,
      "clientHeight": 720,
      "emptyRatio": 0.77
    }
  ]
}
"""

from __future__ import annotations

import json
import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: diagnose_layout_report.py <layout-report.json>", file=sys.stderr)
        return 2

    path = Path(sys.argv[1]).resolve()
    data = json.loads(path.read_text(encoding="utf-8"))
    sections = list(data.get("sections") or [])

    if not sections:
        print("No sections found.")
        return 0

    print(f"# Layout Report: {data.get('page') or path.name}")
    print()

    suspicious = sorted(
        (section for section in sections if float(section.get("emptyRatio") or 0) >= 0.4),
        key=lambda row: float(row.get("emptyRatio") or 0),
        reverse=True,
    )

    if suspicious:
        print("## High Empty-Ratio Sections")
        for row in suspicious:
            print(
                f"- {row.get('title') or 'section'}: emptyRatio={row.get('emptyRatio')} "
                f"height={row.get('height')} clientHeight={row.get('clientHeight')} "
                f"scrollHeight={row.get('scrollHeight')}"
            )
        print()

    print("## All Sections")
    for row in sorted(sections, key=lambda item: (item.get("top", 0), item.get("left", 0))):
        print(
            f"- {row.get('title') or 'section'} | "
            f"x={row.get('left')} y={row.get('top')} "
            f"w={row.get('width')} h={row.get('height')}"
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
