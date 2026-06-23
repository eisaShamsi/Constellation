import sqlite3, time, sys

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

# 1) note_meta rows
notes = {}            # path -> dict
name_to_path = {}     # name_lower -> path  (first wins, like the app)
for r in cur.execute("SELECT path, name, incoming_count, created_at, modified FROM note_meta"):
    nl = (r["name"] or "").lower()
    notes[r["path"]] = {
        "name_lower": nl,
        "incoming_count": r["incoming_count"] or 0,
        "created_at": r["created_at"],
        "modified": r["modified"] or 0,
    }
    name_to_path.setdefault(nl, r["path"])

# 2) aliases: alias_lower -> path
alias_to_path = {}
try:
    for r in cur.execute("SELECT alias_lower, path FROM note_aliases"):
        alias_to_path.setdefault(r["alias_lower"], r["path"])
except sqlite3.OperationalError as e:
    print("no note_aliases:", e)

# 3) active inbound edges, resolve target_name_lower -> target note path
#    Replicates the incoming_count matched set (name OR alias), distinct-source.
# Detect target_name_lower column
cols = [c[1] for c in cur.execute("PRAGMA table_info(note_links)")]
tnl = "target_name_lower" if "target_name_lower" in cols else "LOWER(target_name)"

total = {}      # target_path -> total edge count (COUNT(*))
distinct = {}   # target_path -> set(source_path)
q = f"SELECT source_path, {tnl} AS tnl FROM note_links WHERE status='active'"
nrows = 0
for r in cur.execute(q):
    nrows += 1
    t = r["tnl"]
    if t is None: continue
    tp = name_to_path.get(t) or alias_to_path.get(t)
    if tp is None or tp not in notes:
        continue
    total[tp] = total.get(tp, 0) + 1
    distinct.setdefault(tp, set()).add(r["source_path"])

print(f"notes={len(notes)} active_link_rows={nrows} aliases={len(alias_to_path)}")

# 4) compare maturities
flips = []
inc_mismatch = 0
inc_mismatch_samples = []
bucket_from = {}
multi_link_notes = 0
for path, n in notes.items():
    tot = total.get(path, 0)
    dis = len(distinct.get(path, ()))
    if tot != dis:
        multi_link_notes += 1
    created = n["created_at"] if n["created_at"] is not None else n["modified"]
    created = max(created or 0, 0)
    modified = max(n["modified"], 0)
    dsc = max(now - created, 0)//86400
    dsm = max(now - modified, 0)//86400
    m_tot = compute_state(tot, dsc, dsm)
    m_dis = compute_state(dis, dsc, dsm)
    if m_tot != m_dis:
        flips.append((path, n["name_lower"], tot, dis, m_tot, m_dis))
        bucket_from[(m_tot, m_dis)] = bucket_from.get((m_tot, m_dis), 0) + 1
    # validate: does my distinct match the stored incoming_count?
    if dis != n["incoming_count"]:
        inc_mismatch += 1
        if len(inc_mismatch_samples) < 8:
            inc_mismatch_samples.append((n["name_lower"], dis, n["incoming_count"]))

print(f"\nnotes where COUNT(*) != COUNT(DISTINCT source): {multi_link_notes}")
print(f"maturity FLIPS (total->distinct): {len(flips)}")
for k,v in sorted(bucket_from.items(), key=lambda x:-x[1]):
    print(f"   {k[0]:>10} -> {k[1]:<10}  {v}")
print("\nsample flips (name, total, distinct, mat_total, mat_distinct):")
for f in flips[:20]:
    print("  ", f[1][:40], f[2], f[3], f[4], "->", f[5])

print(f"\nmy_distinct != stored incoming_count: {inc_mismatch} notes")
for s in inc_mismatch_samples:
    print("   ", s)

# 5) cross-check stored sky_nodes.maturity vs COUNT(*) model (sanity)
try:
    sky = {r["path"]: r["maturity"] for r in cur.execute("SELECT path, maturity FROM sky_nodes")}
    sky_vs_total = 0
    sky_vs_distinct = 0
    checked = 0
    for path, n in notes.items():
        if path not in sky or sky[path] is None: continue
        checked += 1
        tot = total.get(path, 0); dis = len(distinct.get(path, ()))
        created = max((n["created_at"] if n["created_at"] is not None else n["modified"]) or 0, 0)
        modified = max(n["modified"], 0)
        dsc = max(now-created,0)//86400; dsm = max(now-modified,0)//86400
        if compute_state(tot, dsc, dsm) != sky[path]: sky_vs_total += 1
        if compute_state(dis, dsc, dsm) != sky[path]: sky_vs_distinct += 1
    print(f"\nsky_nodes.maturity checked={checked}")
    print(f"   disagree with COUNT(*) model:        {sky_vs_total}")
    print(f"   disagree with COUNT(DISTINCT) model: {sky_vs_distinct}")
except sqlite3.OperationalError as e:
    print("sky_nodes read failed:", e)

con.close()
