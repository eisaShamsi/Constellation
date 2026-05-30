import sqlite3, sys

db = r"E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\search.db"
con = sqlite3.connect(f"file:{db}?mode=ro", uri=True)
cur = con.cursor()

cols = [r[1] for r in cur.execute("PRAGMA table_info(note_meta)").fetchall()]
print("note_meta outgoing cols:", [c for c in cols if "outgoing" in c])

print("\n--- link_type distribution (top 25) ---")
for lt, n in cur.execute(
    "SELECT link_type, COUNT(*) FROM note_links GROUP BY link_type ORDER BY COUNT(*) DESC LIMIT 25"
):
    print(repr(lt), n)

print("\n--- Ancient history note_meta row ---")
rows = cur.execute(
    "SELECT path, name, outgoing_count, outgoing_link_types, outgoing_top_rank FROM note_meta WHERE name='Ancient history' LIMIT 3"
).fetchall()
for r in rows:
    print(r)

if rows:
    p = rows[0][0]
    print("\n--- Ancient history note_links: link_type x status (top 15) ---")
    for lt, st, n in cur.execute(
        "SELECT link_type, status, COUNT(*) FROM note_links WHERE source_path=? GROUP BY link_type, status ORDER BY COUNT(*) DESC LIMIT 15",
        (p,),
    ):
        print(repr(lt), repr(st), n)

    print("\n--- 5 raw note_links rows for Ancient history ---")
    for row in cur.execute(
        "SELECT source_path, target_name, link_type, status FROM note_links WHERE source_path=? LIMIT 5",
        (p,),
    ):
        print(row)

print("\n--- canonical-typed link rows total (the §A filter) ---")
n = cur.execute(
    "SELECT COUNT(*) FROM note_links WHERE status='active' AND link_type IN "
    "('supports','contradicts','causes','exemplifies','generalizes','derives-from','part-of','supersedes')"
).fetchone()[0]
print("active canonical-typed links:", n)

print("\n--- notes with a non-empty outgoing_link_types ---")
n = cur.execute("SELECT COUNT(*) FROM note_meta WHERE outgoing_link_types <> ''").fetchone()[0]
print("note_meta rows with types populated:", n)
con.close()
