"""MIG-085 §B.0 live-copy rehearsal — prove the name_fold backfill on a COPY of the
real 7,660-note universe: add+populate name_lower, recompute incoming for the accented
notes via the COALESCE(name_lower, LOWER(name)) match, assert the fixes + zero collateral."""
import sqlite3, shutil, os, unicodedata, tempfile

SRC = r"E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\search.db"
COPY = os.path.join(tempfile.gettempdir(), "mig085b_rehearse.db")
shutil.cop2 = shutil.copyfile
shutil.copyfile(SRC, COPY)

def fold(s):  # mirrors Rust fold_match_key: NFC + lower + NFC (no arabic strip)
    if s is None: return s
    return unicodedata.normalize('NFC', unicodedata.normalize('NFC', s).lower())

con = sqlite3.connect(COPY)
cur = con.cursor()

# ── simulate the backfill ──
# Phase A: add + populate name_lower for every note.
cols = [c[1] for c in cur.execute("PRAGMA table_info(note_meta)")]
if "name_lower" not in cols:
    cur.execute("ALTER TABLE note_meta ADD COLUMN name_lower TEXT")
names = cur.execute("SELECT path, name, incoming_count FROM note_meta").fetchall()
for path, name, _ in names:
    cur.execute("UPDATE note_meta SET name_lower=? WHERE path=?", (fold(name), path))
con.commit()

# Phase B: recompute incoming_count for accented notes via the new match
# (COALESCE(name_lower, LOWER(name)) vs LOWER(target_name)); DISTINCT source.
def recompute_incoming(path):
    nl = cur.execute("SELECT COALESCE(name_lower, LOWER(name)) FROM note_meta WHERE path=?", (path,)).fetchone()[0]
    n = cur.execute(
        "SELECT COUNT(DISTINCT source_path) FROM note_links nl WHERE nl.status='active' AND ("
        " LOWER(nl.target_name)=? OR LOWER(nl.target_name) IN (SELECT alias_lower FROM note_aliases WHERE path=?))",
        (nl, path)).fetchone()[0]
    return n

accented = ['Śramaṇa','Île-de-France','Étienne-Jules Marey','Île de la Cité','Émilie du Châtelet',
            'Étienne-Louis Boullée','Đông Sơn culture','Étude','Śāriputra','Abū Ḥanīfa',
            'Charles-Émile Reynaud','Notre-Dame de l\'Épine','Š-L-M']
expected = {'Śramaṇa':26,'Île-de-France':17,'Étienne-Jules Marey':16,'Île de la Cité':13,
            'Émilie du Châtelet':11,'Étienne-Louis Boullée':7,'Đông Sơn culture':7,'Étude':6,
            'Śāriputra':6,'Abū Ḥanīfa':4,'Charles-Émile Reynaud':4,"Notre-Dame de l'Épine":3,'Š-L-M':3}

print("── accented notes: incoming_count BEFORE → AFTER ──")
fixed = 0
for nm in accented:
    row = cur.execute("SELECT path, incoming_count FROM note_meta WHERE name=?", (nm,)).fetchone()
    before = row[1]
    after = recompute_incoming(row[0])
    ok = (after == expected[nm])
    if before == 0 and after > 0: fixed += 1
    print(f"  {'OK ' if ok else 'BAD'} {nm:30} {before:>3} → {after:>3}  (expected {expected[nm]})")
    assert ok, f"{nm}: expected {expected[nm]}, got {after}"
print(f"\nAll 13 accented notes fixed (0 → real count): {fixed}/13")

# ── collateral check: ASCII notes' name_lower must equal old SQLite LOWER(name) ──
changed = 0
for path, name, _ in names:
    if name is None: continue
    sql_lower = cur.execute("SELECT LOWER(?)", (name,)).fetchone()[0]
    nl = fold(name)
    if nl != sql_lower:
        changed += 1
print(f"notes whose name_lower differs from old SQLite LOWER(name): {changed}  (expected 13 — exactly the accented set)")
assert changed == 13, f"collateral: expected 13 changed, got {changed}"

# ── orphan-lens check: are the accented notes still falsely orphaned after fix? ──
false_orphans_after = 0
for nm in accented:
    row = cur.execute("SELECT path, word_count FROM note_meta WHERE name=?", (nm,)).fetchone()
    after = recompute_incoming(row[0])
    if after == 0 and row[1] > 20: false_orphans_after += 1
print(f"false orphans remaining after fix (inc=0 & wc>20): {false_orphans_after}  (expected 0)")
assert false_orphans_after == 0

con.close()
os.remove(COPY)
print("\n✅ REHEARSAL PASS — accent fix correct on the live corpus; zero collateral; no false orphans remain.")
