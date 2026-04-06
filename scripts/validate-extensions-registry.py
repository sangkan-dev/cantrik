#!/usr/bin/env python3
"""Validate apps/cantrik-site/static/registry/extensions.json (CI)."""
from __future__ import annotations

import json
import sys
from pathlib import Path
from urllib.parse import urlparse

ALLOWED_KINDS = frozenset(
    {"skill_pack", "lua_plugin", "wasm_plugin", "mcp_preset", "recipe_ref"}
)


def _is_reasonable_url(s: str) -> bool:
    u = urlparse(s)
    return u.scheme in ("http", "https") and bool(u.netloc)


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: validate-extensions-registry.py <path-to-extensions.json>", file=sys.stderr)
        return 2
    path = Path(sys.argv[1])
    data = json.loads(path.read_text(encoding="utf-8"))
    if data.get("schema_version") != 1:
        print("extensions: expected schema_version == 1", file=sys.stderr)
        return 1
    exts = data.get("extensions")
    if not isinstance(exts, list):
        print("extensions: expected 'extensions' array", file=sys.stderr)
        return 1
    seen: set[str] = set()
    for i, e in enumerate(exts):
        if not isinstance(e, dict):
            print(f"extensions[{i}]: expected object", file=sys.stderr)
            return 1
        for key in ("id", "name", "description", "kind", "source", "install_hint"):
            if key not in e or not isinstance(e[key], str) or not e[key].strip():
                print(
                    f"extensions[{i}]: missing or empty string field {key!r}",
                    file=sys.stderr,
                )
                return 1
        oid = e["id"].strip()
        if oid in seen:
            print(f"extensions[{i}]: duplicate id {oid!r}", file=sys.stderr)
            return 1
        seen.add(oid)
        kind = e["kind"].strip()
        if kind not in ALLOWED_KINDS:
            print(
                f"extensions[{i}]: kind {kind!r} not in {sorted(ALLOWED_KINDS)}",
                file=sys.stderr,
            )
            return 1
        if not _is_reasonable_url(e["source"].strip()):
            print(f"extensions[{i}]: source must be http(s) URL", file=sys.stderr)
            return 1
        if "verified" in e and not isinstance(e["verified"], bool):
            print(f"extensions[{i}]: field 'verified' must be boolean if present", file=sys.stderr)
            return 1
        if "recipe_id" in e:
            if not isinstance(e["recipe_id"], str) or not e["recipe_id"].strip():
                print(f"extensions[{i}]: recipe_id must be non-empty string if present", file=sys.stderr)
                return 1
            if kind != "recipe_ref":
                print(
                    f"extensions[{i}]: recipe_id only allowed when kind is recipe_ref",
                    file=sys.stderr,
                )
                return 1
        if kind == "recipe_ref" and "recipe_id" not in e:
            print(f"extensions[{i}]: recipe_ref requires recipe_id", file=sys.stderr)
            return 1
    print(f"extensions: OK ({len(exts)} entries)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
