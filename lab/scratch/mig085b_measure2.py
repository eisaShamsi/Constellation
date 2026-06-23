import sqlite3, time

DB = r"E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\search.db"
con = sqlite3.connect(f"file:{DB}?mode=ro", uri=True)
con.row_factory = sqlite3.Row
cur = con.cursor()
now = int(time.time())

def compute_state(inbound, dsc, dsm):
    if inbound >= 10 and dsm >= 30: return "canonical"
    if inbound >= 4 and dsc >= 7 and dsm >= 90: return "wilting"
    if inbound >= 4 and dsc >= 7: return "evergreen"
    if inbound >= 1 or dsc >= 2: return "sapling"
    return "seed"

cols = [c[1] for c in cur.execute("PRAGMA table_info(note_links)")]
tnl = "nl.target_name_lower" if "target_name_lower" in cols else "LOWER(nl.target_name)"

# Exactly mirror incoming_count's matched set, per note, both COUNT(*) and COUNT(DISTINCT source_path).
sql = f"""
SELECT nm.path, nm.name, nm.incoming_count, nm.created_at, nm.modified,
  (SELECT COUNT(*) FROM note_links nl
     WHERE nl.status='active'
       AND ({tnl} = LOWER(nm.name)
            OR {tnl} IN (SELECT alias_lower FROM note_aliases WHERE path = nm.path))) AS tot,
  (SELECT COUNT(DISTINCT nl.source_path) FROM note_links nl
     WHERE nl.status='active'
       AND ({tnl} = LOWER(nm.name)
            OR {tnl} IN (SELECT alias_lower FROM note_aliases WHERE path = nm.path))) AS dis
FROM note_meta nm
"""

t0 = time.time()
rows = list(cur.execute(sql))
print(f"query {time.time()-t0:.1f}s  notes={len(rows)}")

flips = {}
nflip = 0
inc_mismatch = 0
inc_samples = []
multi = 0
for r in rows:
    tot, dis, inc = r["tot"], r["dis"], r["incoming_count"] or 0
    if tot != dis: multi += 1
    created = r["created_at"] if r["created_at"] is not None else r["modified"]
    created = max(created or 0, 0); modified = max(r["modified"] or 0, 0)
    dsc = max(now-created,0)//86400; dsm = max(now-modified,0)//86400
    mt = compute_state(tot, dsc, dsm); md = compute_state(dis, dsc, dsm)
    if mt != md:
        nflip += 1
        flips[(mt,md)] = flips.get((mt,md),0)+1
    if dis != inc:
        inc_mismatch += 1
        if len(inc_samples) < 12: inc_samples.append((r["name"][:35], "dis="+str(dis), "inc="+str(inc), "tot="+str(tot)))

print(f"COUNT(*) != COUNT(DISTINCT): {multi} notes")
print(f"maturity FLIPS total->distinct: {nflip}")
for k,v in sorted(flips.items(), key=lambda x:-x[1]):
    print(f"   {k[0]} -> {k[1]}: {v}")
print(f"\nincoming_count == COUNT(DISTINCT) match check: mismatches={inc_mismatch}")
for s in inc_samples: print("   ", s)
con.close()
