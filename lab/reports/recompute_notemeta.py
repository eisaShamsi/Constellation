# -*- coding: utf-8 -*-
"""One-time repair: recompute note_meta.outgoing_count / outgoing_link_types /
outgoing_top_rank from the (now correctly-typed) note_links. Same logic as the
app's outgoing_aggregate_assignments. Batched + busy_timeout so it coexists with
the live app (WAL). Idempotent."""
import sqlite3, time

db = r"E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\search.db"
IN_LIST = "('supports','contradicts','causes','exemplifies','generalizes','derives-from','part-of','supersedes')"
RANK = ("CASE link_type WHEN 'supports' THEN 1 WHEN 'contradicts' THEN 2 "
        "WHEN 'causes' THEN 3 WHEN 'exemplifies' THEN 4 WHEN 'generalizes' THEN 5 "
        "WHEN 'derives-from' THEN 6 WHEN 'part-of' THEN 7 WHEN 'supersedes' THEN 8 END")
ASSIGN = f"""
  outgoing_count = (SELECT COUNT(*) FROM note_links WHERE source_path = note_meta.path AND status='active'),
  outgoing_link_types = (SELECT COALESCE(GROUP_CONCAT(lt || ' (' || cnt || ')', ', '), '') FROM
     (SELECT link_type AS lt, COUNT(*) AS cnt FROM note_links
      WHERE source_path = note_meta.path AND status='active' AND link_type IN {IN_LIST}
      GROUP BY link_type ORDER BY {RANK})),
  outgoing_top_rank = COALESCE((SELECT MIN({RANK}) FROM note_links
     WHERE source_path = note_meta.path AND status='active' AND link_type IN {IN_LIST}), 9)
"""

con = sqlite3.connect(db, timeout=45)
con.execute("PRAGMA busy_timeout=45000")
after, total = "", 0
while True:
    rows = con.execute("SELECT path FROM note_meta WHERE path > ? ORDER BY path LIMIT 500", (after,)).fetchall()
    if not rows:
        break
    last = rows[-1][0]
    for attempt in range(6):
        try:
            con.execute(f"UPDATE note_meta SET {ASSIGN} WHERE path > ? AND path <= ?", (after, last))
            con.commit()
            break
        except sqlite3.OperationalError as e:
            if "locked" in str(e).lower() and attempt < 5:
                time.sleep(1.0); continue
            raise
    total += len(rows)
    after = last
print(f"DONE: {total} notes recomputed")
n = con.execute("SELECT COUNT(*) FROM note_meta WHERE outgoing_link_types <> ''").fetchone()[0]
print("notes with types populated:", n)
print("Ancient history:", con.execute(
    "SELECT outgoing_count, outgoing_link_types, outgoing_top_rank FROM note_meta WHERE name='Ancient history'").fetchone())
con.close()
