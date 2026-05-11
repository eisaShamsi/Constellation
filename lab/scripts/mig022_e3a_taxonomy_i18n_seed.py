#!/usr/bin/env python3
"""
MIG-022 §E.3.a — Seed cece.taxonomy.* i18n keys for en + ar from the Rust
taxonomy structs.

Parses src-tauri/src/sources/vertical_taxonomy.rs and horizontal_taxonomy.rs,
extracts every (id, en, ar) tuple from the VERTICAL_NODES and HORIZONTAL_NODES
arrays, and writes a single cece.taxonomy.<id> object to en.json + ar.json.

Per the additive §E.3 design (D-E1.c with Rust-struct fallback):
- Rust structs keep id + en + ar fields (unchanged) — used as labelForId
  fallback when the i18n key is missing in the active locale.
- en.json + ar.json get cece.taxonomy.<id> keys mirroring the struct values.
- The other 13 locales backfill via parallel agents in §E.3.d.

Run:
    python lab/scripts/mig022_e3a_taxonomy_i18n_seed.py

Idempotent: rewrites cece.taxonomy in en + ar from the source of truth
(the Rust struct) every time. Safe to re-run after taxonomy edits.
"""
import json
import re
from pathlib import Path

ROOT = Path(r"E:\مشاريع كلاود\Constellation")
VERT_RS = ROOT / "src-tauri/src/sources/vertical_taxonomy.rs"
HORIZ_RS = ROOT / "src-tauri/src/sources/horizontal_taxonomy.rs"
EN_JSON = ROOT / "src/lib/i18n/en.json"
AR_JSON = ROOT / "src/lib/i18n/ar.json"

# Match a Node literal:
#   VerticalNode { id: "x", en: "X", ar: "س", parent_id: ..., branch: ... }
#   HorizontalNode { id: "x", en: "X", ar: "س", ... }
# We only need id + en + ar. The other fields vary between vertical/horizontal.
NODE_RE = re.compile(
    r'(?:VerticalNode|HorizontalNode)\s*\{\s*'
    r'id:\s*"([^"]+)"\s*,\s*'
    r'en:\s*"([^"]+)"\s*,\s*'
    r'ar:\s*"([^"]+)"\s*,',
    re.MULTILINE,
)


def extract_nodes(path: Path) -> list[tuple[str, str, str]]:
    """Returns list of (id, en, ar) tuples in source order."""
    text = path.read_text(encoding="utf-8")
    return [(m.group(1), m.group(2), m.group(3)) for m in NODE_RE.finditer(text)]


def build_taxonomy_block(nodes: list[tuple[str, str, str]], lang: str) -> dict:
    """Build the cece.taxonomy.* sub-object for one locale.

    Uses the lang index (1=en, 2=ar) into each tuple. Every key is the
    full taxonomy id (slash-separated), e.g. "epistemic-states/doubt".
    """
    out = {}
    idx = {"en": 1, "ar": 2}[lang]
    for node in nodes:
        out[node[0]] = node[idx]
    return out


def merge_into_cece(json_path: Path, taxonomy_block: dict) -> tuple[int, int]:
    """Insert (or replace) cece.taxonomy in the locale file.

    Returns (added_count, replaced_count) for reporting.
    """
    with json_path.open("r", encoding="utf-8") as f:
        data = json.load(f)
    cece = data.setdefault("cece", {})
    existing = cece.get("taxonomy")
    cece["taxonomy"] = taxonomy_block
    with json_path.open("w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
        f.write("\n")
    if existing is None:
        return (len(taxonomy_block), 0)
    else:
        return (
            len(taxonomy_block) - len(existing),
            len(taxonomy_block) if existing != taxonomy_block else 0,
        )


def main():
    vert = extract_nodes(VERT_RS)
    horiz = extract_nodes(HORIZ_RS)
    print(f"Extracted {len(vert)} vertical + {len(horiz)} horizontal = {len(vert) + len(horiz)} nodes")
    if len(vert) < 200 or len(horiz) < 25:
        print(f"WARN: extracted counts look low (expected ~225 vertical + ~30 horizontal)")
        print(f"      regex may need adjustment if the Rust struct shape changed")

    # Combine into one flat dict — vertical and horizontal IDs are
    # disjoint by construction (different parent paths).
    combined = vert + horiz
    seen_ids = set()
    for node_id, _, _ in combined:
        if node_id in seen_ids:
            print(f"WARN: duplicate id {node_id!r} (vertical/horizontal collision?)")
        seen_ids.add(node_id)

    en_block = build_taxonomy_block(combined, "en")
    ar_block = build_taxonomy_block(combined, "ar")

    en_added, en_replaced = merge_into_cece(EN_JSON, en_block)
    ar_added, ar_replaced = merge_into_cece(AR_JSON, ar_block)
    print(f"en.json: cece.taxonomy now has {len(en_block)} keys "
          f"(+{en_added} added, {en_replaced} replaced)")
    print(f"ar.json: cece.taxonomy now has {len(ar_block)} keys "
          f"(+{ar_added} added, {ar_replaced} replaced)")


if __name__ == "__main__":
    main()
