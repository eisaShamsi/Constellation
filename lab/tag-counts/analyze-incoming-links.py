#!/usr/bin/env python3
"""MIG-079 §C.2a — examine incoming-link shape + build the getBacklinks rehearsal target.

Read-only. Computes, per note, the EXACT count getBacklinks (store.ts:2706) produces:
distinct SOURCE paths among note_links whose LOWER(target_name) matches the note's
name OR any alias, with status != 'archived'. This is the target the write-time
incoming aggregate must equal (the P0 badge==panel pin). Also quantifies:
- dedupe impact (distinct-source vs raw-edge count),
- alias contribution,
- drift of the CURRENT badge (constellation_search_link_counts via outgoing_links_json).

Usage: python lab/tag-counts/analyze-incoming-links.py "<search.db>"
Writes: lab/tag-counts/incoming-target.json  (path -> getBacklinks count)
"""
import json, sqlite3, sys
from collections import defaultdict
from pathlib import Path

db = sys.argv[1] if len(sys.argv) > 1 else \
    r"E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\search.db"
conn = sqlite3.connect(f"file:{Path(db).as_posix()}?mode=ro", uri=True)

# --- note_links: build target_name_lower -> {source_path}, and raw edge count ---
src_by_target = defaultdict(set)       # distinct sources (getBacklinks dedupe)
raw_by_target = defaultdict(int)       # raw active edges
n_links = n_archived = 0
for src, tgt, status in conn.execute("SELECT source_path, target_name, status FROM note_links"):
    n_links += 1
    if status == 'archived':
        n_archived += 1
        continue
    t = (tgt or "").lower()
    src_by_target[t].add(src)
    raw_by_target[t] += 1

# --- note_meta names + note_aliases ---
notes = conn.execute("SELECT path, LOWER(name) FROM note_meta").fetchall()
aliases_by_path = defaultdict(list)
try:
    for path, al in conn.execute("SELECT path, alias_lower FROM note_aliases"):
        if al:
            aliases_by_path[path].append(al)
except Exception as e:
    print("note_aliases read failed:", e)

# --- per-note getBacklinks-equivalent (distinct source over name+aliases) ---
target = {}          # path -> distinct-source incoming count
raw_target = {}      # path -> raw-edge incoming count (no dedupe)
with_aliases = alias_helps = 0
dedupe_differs = 0
for path, name in notes:
    names = {name} | set(aliases_by_path.get(path, []))
    if aliases_by_path.get(path):
        with_aliases += 1
    srcs = set()
    raw = 0
    for nm in names:
        srcs |= src_by_target.get(nm, set())
        raw += raw_by_target.get(nm, 0)
    target[path] = len(srcs)
    raw_target[path] = raw
    if len(srcs) != raw:
        dedupe_differs += 1
    # does the alias add any source the bare name wouldn't?
    bare = src_by_target.get(name, set())
    if len(srcs) > len(bare):
        alias_helps += 1

# --- current badge (constellation_search_link_counts): name-only, raw occurrences in outgoing_links_json ---
badge = defaultdict(int)
for (lj,) in conn.execute("SELECT outgoing_links_json FROM note_meta"):
    try:
        for tnm in json.loads(lj):
            badge[(tnm or "").lower()] += 1
    except Exception:
        pass
# compare badge(name) vs getBacklinks(path) — keyed by name
name_by_path = {p: n for p, n in notes}
badge_vs_target_diff = sum(1 for p, n in notes if badge.get(n, 0) != target[p])

nonzero = sum(1 for v in target.values() if v > 0)
print(f"note_links rows:            {n_links}  (archived {n_archived})")
print(f"notes:                      {len(notes)}   with aliases: {with_aliases}")
print(f"notes with >=1 backlink:    {nonzero}")
print(f"total backlinks (distinct): {sum(target.values())}   raw(no dedupe): {sum(raw_target.values())}")
print(f"notes where distinct != raw (dedupe matters): {dedupe_differs}")
print(f"notes where an ALIAS adds a source:           {alias_helps}")
mx = max(target.items(), key=lambda kv: kv[1])
print(f"max incoming: {mx[1]}  ({name_by_path[mx[0]]})")
print(f"CURRENT badge vs getBacklinks disagree (by name): {badge_vs_target_diff} / {len(notes)} notes  <-- the drift")

out = Path(__file__).parent / "incoming-target.json"
out.write_text(json.dumps(target, ensure_ascii=False), encoding="utf-8")
print(f"wrote rehearsal target -> {out}  ({len(target)} notes)")
