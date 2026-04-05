#!/usr/bin/env python3
"""Validate apps/cantrik-site/static/registry/recipes.json minimal schema (CI)."""
from __future__ import annotations

import json
import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: validate-recipes-registry.py <path-to-recipes.json>", file=sys.stderr)
        return 2
    path = Path(sys.argv[1])
    data = json.loads(path.read_text(encoding="utf-8"))
    if data.get("schema_version") != 1:
        print("recipes: expected schema_version == 1", file=sys.stderr)
        return 1
    recipes = data.get("recipes")
    if not isinstance(recipes, list):
        print("recipes: expected 'recipes' array", file=sys.stderr)
        return 1
    for i, r in enumerate(recipes):
        if not isinstance(r, dict):
            print(f"recipes[{i}]: expected object", file=sys.stderr)
            return 1
        for key in ("id", "title", "init_template"):
            if key not in r or not isinstance(r[key], str) or not r[key].strip():
                print(
                    f"recipes[{i}]: missing or empty string field {key!r}",
                    file=sys.stderr,
                )
                return 1
        if "verified" in r and not isinstance(r["verified"], bool):
            print(f"recipes[{i}]: field 'verified' must be boolean if present", file=sys.stderr)
            return 1
    print(f"recipes: OK ({len(recipes)} entries)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
