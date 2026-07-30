# -*- coding: utf-8 -*-
"""MIG-108 — build the Stage-A/Slice-6 rehearsal universe.

Creates a COMPLETE, SELF-CONSISTENT scratch copy of a universe whose libraries live outside
its root, so the real migration engine can be run against it with nothing at stake:

  1. copy the universe root -> <scratch root>            (excluding the stale SV-Test db trio)
  2. copy every EXTERNAL library tree -> the scratch External folder, by basename
  3. re-point the scratch's libraries.json at the scratch externals (ids/names unchanged)
  4. remap every path-carrying DB row real-external -> scratch-external (the scratch db still
     points at the REAL trees otherwise, and the rehearsal would "pass" without exercising
     the rewrite at all). All triggers are dropped first (the O(N^2) outgoing trigger would
     take minutes; init_db recreates every trigger on next boot - the documented self-heal).
  5. deep-remap the 8 path-bearing JSON stores (keys AND values)
  6. rename the scratch universe so the picker cannot be confused with the real one

Read-only toward the REAL universe: it is never written, never locked, never registered.

Usage:  python lab/tools/mig108_make_rehearsal.py
        (paths are fixed to the Boss universe below; edit SRC_ROOT/SCRATCH to retarget)
"""
import json
import os
import shutil
import sqlite3
import sys
import unicodedata

SRC_ROOT = r"E:\Constellation Universes\Eisa Cognitive Knowledge"
SCRATCH = r"E:\Constellation Universes\MIG108 Rehearsal"
SCRATCH_EXT = r"E:\Constellation Universes\MIG108 Rehearsal External"
EXCLUDE_FILES = {"constellation sv test.db", "constellation sv test.db-shm", "constellation sv test.db-wal"}

PATH_TABLES = [
    ("note_meta", "path"), ("note_links", "source_path"), ("note_aliases", "path"),
    ("note_embeddings", "path"), ("note_body", "path"), ("review_schedule", "path"),
    ("note_summaries", "path"), ("sources_suggestions", "note_path"),
    ("sight_v3_layout", "note_path"), ("note_state_history", "note_path"),
    ("shape_history", "path"), ("sky_nodes", "path"), ("sky_links", "source_path"),
]

def norm(p):
    return unicodedata.normalize("NFC", p).replace("\\", "/").rstrip("/").lower()

def remap(stored, mapping):
    """Component-wise prefix remap (same rule as the engine): NFC/case/separator-insensitive
    match on the prefix, raw suffix carried verbatim."""
    parts = [c for c in stored.replace("\\", "/").split("/") if c]
    for old_root, new_root in mapping:
        op = [c for c in old_root.replace("\\", "/").split("/") if c]
        if len(parts) < len(op):
            continue
        if all(unicodedata.normalize("NFC", parts[i]).lower() ==
               unicodedata.normalize("NFC", op[i]).lower() for i in range(len(op))):
            out = new_root
            for c in parts[len(op):]:
                out = os.path.join(out, c)
            return out
    return None

def longpath(p):
    """Windows extended-length prefix: shutil dies at MAX_PATH (260) without it — the Boss
    universe has attachment filenames near 200 chars on their own. Rust's std handles this
    internally, so the ENGINE is immune; Python is not."""
    p = os.path.abspath(p)
    if os.name == "nt" and not p.startswith("\\\\?\\"):
        return "\\\\?\\" + p
    return p

def copy_tree(src, dst):
    def ignore(directory, names):
        return [n for n in names if n.lower() in EXCLUDE_FILES]
    shutil.copytree(longpath(src), longpath(dst), ignore=ignore)

def deep_remap_json(value, mapping):
    if isinstance(value, str):
        r = remap(value, mapping)
        return r if r is not None else value
    if isinstance(value, list):
        return [deep_remap_json(v, mapping) for v in value]
    if isinstance(value, dict):
        return {
            (remap(k, mapping) or k): deep_remap_json(v, mapping)
            for k, v in value.items()
        }
    return value

def main():
    for target in (SCRATCH, SCRATCH_EXT):
        if os.path.exists(target):
            print(f"REFUSING: {target} already exists — remove it first (nothing was touched).")
            sys.exit(1)

    libs_file = os.path.join(SRC_ROOT, ".constellation", "libraries.json")
    libs = json.load(open(libs_file, encoding="utf-8"))
    externals = [l for l in libs
                 if not (norm(l["path"]) == norm(SRC_ROOT) or norm(l["path"]).startswith(norm(SRC_ROOT) + "/"))]
    basenames = [os.path.basename(l["path"].rstrip("\\/")) for l in externals]
    assert len(set(n.lower() for n in basenames)) == len(basenames), "basename collision — extend the maker"

    print(f"Universe root : {SRC_ROOT}")
    print(f"Externals     : {len(externals)} of {len(libs)} registered libraries")

    # 1 - the root (includes .constellation with search.db)
    print("Copying the universe root (this carries the ~2 GB index — give it a minute)…")
    copy_tree(SRC_ROOT, SCRATCH)

    # 2 - the external trees + the mapping real -> scratch
    mapping = []  # (real library path, scratch library path)
    os.makedirs(SCRATCH_EXT)
    for l in externals:
        base = os.path.basename(l["path"].rstrip("\\/"))
        dest = os.path.join(SCRATCH_EXT, base)
        print(f"  external: {l['name']} -> {dest}")
        copy_tree(l["path"], dest)
        mapping.append((l["path"], dest))

    # 3 - re-point the scratch registry (+ the root entry itself)
    mapping_with_root = mapping + [(SRC_ROOT, SCRATCH)]
    scratch_libs = []
    for l in libs:
        l = dict(l)
        r = remap(l["path"], mapping_with_root)
        assert r is not None, f"unmapped registry path: {l['path']}"
        l["path"] = r
        scratch_libs.append(l)
    scratch_libs_file = os.path.join(SCRATCH, ".constellation", "libraries.json")
    json.dump(scratch_libs, open(scratch_libs_file, "w", encoding="utf-8"), ensure_ascii=False, indent=2)

    # 4 - DB remap (real-external -> scratch-external AND real-root -> scratch-root)
    db = os.path.join(SCRATCH, ".constellation", "search.db")
    conn = sqlite3.connect(db)
    conn.execute("PRAGMA journal_mode=WAL")
    triggers = [r[0] for r in conn.execute("SELECT name FROM sqlite_master WHERE type='trigger'")]
    for t in triggers:
        conn.execute(f'DROP TRIGGER IF EXISTS "{t}"')
    print(f"Dropped {len(triggers)} triggers (init_db recreates them on next boot).")
    conn.execute("BEGIN")
    total = 0
    for table, col in PATH_TABLES:
        try:
            rows = conn.execute(f'SELECT DISTINCT "{col}" FROM "{table}"').fetchall()
        except sqlite3.OperationalError:
            continue  # lazily-created table absent
        n = 0
        for (old,) in rows:
            if old is None:
                continue
            new = remap(old, mapping_with_root)
            if new is not None and new != old:
                conn.execute(f'DELETE FROM "{table}" WHERE "{col}" = ?', (new,))
                conn.execute(f'UPDATE "{table}" SET "{col}" = ? WHERE "{col}" = ?', (new, old))
                n += 1
        total += n
        if n:
            print(f"  db: {table}.{col}: {n} distinct paths remapped")
    conn.execute("COMMIT")
    conn.execute("PRAGMA wal_checkpoint(TRUNCATE)")
    # verification: nothing in the scratch db may still reference the REAL trees
    leftovers = 0
    for table, col in PATH_TABLES:
        try:
            rows = conn.execute(f'SELECT "{col}" FROM "{table}"').fetchall()
        except sqlite3.OperationalError:
            continue
        for (p,) in rows:
            if p and remap(p, [(SRC_ROOT, ""), *[(a, "") for a, _ in mapping]]) is not None:
                leftovers += 1
    conn.close()
    print(f"DB remap: {total} distinct paths across tables; leftovers referencing the REAL trees: {leftovers}")
    assert leftovers == 0, "scratch db still references the real universe — DO NOT use this rehearsal"

    # 5 - JSON stores
    for store in ["libraries.json", "review-pulse.json", "workspaces.json", "session.json",
                  "session.prev.json", "collections.json", "settings.json", "bookmarks.json"]:
        p = os.path.join(SCRATCH, ".constellation", store)
        if os.path.exists(p):
            try:
                v = json.load(open(p, encoding="utf-8"))
            except Exception:
                continue
            json.dump(deep_remap_json(v, mapping_with_root), open(p, "w", encoding="utf-8"),
                      ensure_ascii=False, indent=2)
            print(f"  json: {store} remapped")

    # 6 - rename the scratch universe for the picker
    uj = os.path.join(SCRATCH, ".constellation", "universe.json")
    if os.path.exists(uj):
        meta = json.load(open(uj, encoding="utf-8"))
        if isinstance(meta, dict) and "name" in meta:
            meta["name"] = "MIG108 Rehearsal"
            json.dump(meta, open(uj, "w", encoding="utf-8"), ensure_ascii=False, indent=2)

    print("\nREHEARSAL READY.")
    print(f"  Open this universe in Constellation: {SCRATCH}")
    print(f"  Its {len(externals)} external libraries live at: {SCRATCH_EXT}")
    print("  The real universe was only READ — never written, never registered.")

if __name__ == "__main__":
    main()
