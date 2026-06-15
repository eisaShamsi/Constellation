#!/usr/bin/env python3
"""Read the append-only boot-perf history (MIG-079 §B) and print every boot's
breakdown — cold AND warm, however many per session, never overwritten.

Usage:
  python lab/boot-perf/read-boot-history.py [path-to-boot-perf.history.jsonl]

Default path = the active universe's .constellation/boot-perf.history.jsonl.
Each launch writes up to 2 lines (phase=core, phase=graph) sharing one boot_id;
this groups them and prints the key fields, with the DB-read breakdown
(ensure_db / open_reader / read_notes) that tells where a slow cold boot's time
actually goes — measured, not guessed.
"""
import json, sys, os
from collections import OrderedDict

DEFAULT = r"E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\boot-perf.history.jsonl"
path = sys.argv[1] if len(sys.argv) > 1 else DEFAULT

if not os.path.exists(path):
    print(f"no history yet at: {path}\n(launch the app at least once with the §B tool build)")
    sys.exit(0)

boots = OrderedDict()  # boot_id -> merged record (graph phase overrides core where present)
for line in open(path, encoding="utf-8"):
    line = line.strip()
    if not line:
        continue
    try:
        r = json.loads(line)
    except Exception:
        continue
    bid = r.get("boot_id") or r.get("timestamp")
    cur = boots.get(bid, {})
    # later/graph-phase writes are more complete; merge, preferring non-null new values
    for k, v in r.items():
        if v is not None or k not in cur:
            cur[k] = v
    boots[bid] = cur

def kv(timings):
    return {k: v for k, v in (timings or [])}

print(f"=== boot-perf history: {len(boots)} launch(es) ===  ({path})\n")
for i, (bid, r) in enumerate(boots.items(), 1):
    core = kv(r.get("cache_snapshot_core_server_timings"))
    safe = r.get("safe_boot_mode")
    print(f"[{i}] {r.get('timestamp','?')}  boot_id={bid}  safe_boot_mode={safe}  notes={r.get('note_count')}")
    print(f"     paint={r.get('paint_ms')}ms  hydrated={r.get('hydrated_ms')}ms  graph_ready={r.get('graph_ready_ms')}ms")
    print(f"     core read [ensure_db={core.get('ensure_db')}  open_reader={core.get('open_reader')}  read_notes={core.get('read_notes')}]  core_wall={r.get('cache_snapshot_core_wall_ms')}ms")
    g = kv(r.get("cache_snapshot_graph_server_timings"))
    if g:
        print(f"     graph read [read_links={g.get('read_links')}  read_tags={g.get('read_tags')}]  graph_wall={r.get('cache_snapshot_graph_wall_ms')}ms  queue={r.get('cache_snapshot_graph_queue_ms')}ms")
    print()
