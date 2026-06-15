#!/usr/bin/env python3
"""MIG-079 §C.1 rehearsal step (a) — examine the live tags_json shape.

Read-only. Establishes the EXACT target aggregate that `read_tags`
(cache.rs:1027) produces today, so the Rust `tag_counts` backfill can be
proven byte-identical to it. Also measures whether a SQL `json_each`
aggregate would diverge from `serde_json::from_str::<Vec<String>>`
semantics on the REAL corpus (the only place the two can differ:
malformed JSON, non-array JSON, or arrays with non-string elements).

Usage:  python lab/tag-counts/analyze-live-tags.py "<path-to-search.db>"
Writes: lab/tag-counts/live-read-tags-target.json  (the serde target)
"""
import json, sqlite3, sys, time
from collections import Counter
from pathlib import Path

db = sys.argv[1] if len(sys.argv) > 1 else \
    r"E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\search.db"

uri = f"file:{Path(db).as_posix()}?mode=ro&immutable=1"
conn = sqlite3.connect(uri, uri=True)

t0 = time.time()
rows = conn.execute("SELECT tags_json FROM note_meta").fetchall()
fetch_s = time.time() - t0

n = len(rows)
null_rows = malformed = not_list = nonstr_elem = empty_elem_rows = dup_rows = 0

# serde_json::from_str::<Vec<String>>().unwrap_or_default() semantics:
#   parse; if list AND every element is a str -> count each non-empty
#   else -> contributes NOTHING (the whole note's tags drop)
serde = Counter()
# json_each-style (permissive): any list -> count each element's string form,
#   skipping '' — what a naive SQL aggregate would produce.
jeach = Counter()

for (tj,) in rows:
    if tj is None:
        null_rows += 1
        continue
    try:
        v = json.loads(tj)
    except Exception:
        malformed += 1
        continue
    if not isinstance(v, list):
        not_list += 1
        # json_each on a non-array JSON object would still yield members;
        # but tags are always arrays in practice — record the divergence.
        continue
    all_str = all(isinstance(x, str) for x in v)
    if not all_str:
        nonstr_elem += 1
    # serde path: only if every element is a string
    if all_str:
        if any(x == "" for x in v):
            empty_elem_rows += 1
        if len(set(v)) != len(v):
            dup_rows += 1
        for x in v:
            if x != "":
                serde[x] += 1
    # json_each path: count every element coerced to its string form
    for x in v:
        s = x if isinstance(x, str) else json.dumps(x, ensure_ascii=False)
        if s != "":
            jeach[s] += 1

print(f"note_meta rows:            {n}")
print(f"  tags_json NULL:          {null_rows}")
print(f"  malformed JSON:          {malformed}")
print(f"  not a JSON array:        {not_list}")
print(f"  array w/ non-str elem:   {nonstr_elem}")
print(f"  array w/ empty-str elem: {empty_elem_rows}")
print(f"  array w/ duplicate tags: {dup_rows}")
print(f"raw SELECT tags_json fetch: {fetch_s:.3f}s")
print()
print(f"serde aggregate:   distinct={len(serde)}  occurrences={sum(serde.values())}")
print(f"json_each aggregate: distinct={len(jeach)}  occurrences={sum(jeach.values())}")

# Where do the two diverge? (the design-deciding question)
diff_keys = set(serde) ^ set(jeach)
diff_counts = {k for k in (set(serde) & set(jeach)) if serde[k] != jeach[k]}
print(f"\nserde vs json_each: {len(diff_keys)} key-set diffs, {len(diff_counts)} count diffs")
if diff_keys:
    for k in list(diff_keys)[:20]:
        print(f"  only-one: {k!r}  serde={serde.get(k,0)} jeach={jeach.get(k,0)}")
if diff_counts:
    for k in list(diff_counts)[:20]:
        print(f"  count:    {k!r}  serde={serde[k]} jeach={jeach[k]}")

print("\ntop 15 tags (serde):")
for tag, c in serde.most_common(15):
    print(f"  {c:6d}  {tag}")

out = Path(__file__).parent / "live-read-tags-target.json"
out.write_text(json.dumps(dict(sorted(serde.items())), ensure_ascii=False, indent=0),
               encoding="utf-8")
print(f"\nwrote serde target -> {out}")
