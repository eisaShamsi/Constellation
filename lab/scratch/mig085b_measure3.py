import sqlite3, time
from collections import defaultdict

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

# note_meta
notes = {}
name_to_paths = defaultdict(list)   # name_lower -> [paths]  (ALL, like incoming_count semantics)
for r in cur.execute("SELECT path, name, incoming_count, created_at, modified FROM note_meta"):
    nl = (r["name"] or "").lower()
    notes[r["path"]] = dict(name_lower=nl, incoming_count=r["incoming_count"] or 0,
                            created_at=r["created_at"], modified=r["modified"] or 0)
    name_to_paths[nl].append(r["path"])

# aliases: alias_lower -> [paths]
alias_to_paths = defaultdict(list)
for r in cur.execute("SELECT alias_lower, path FROM note_aliases"):
    alias_to_paths[r["alias_lower"]].append(r["path"])

# single pass over active edges; credit ALL notes whose name OR alias matches the target
total = defaultdict(int)
distinct = defaultdict(set)
n = 0
for r in cur.execute("SELECT source_path, LOWER(target_name) AS t FROM note_links WHERE status='active'"):
    n += 1
    t = r["t"]
    if t is None: continue
    paths = set(name_to_paths.get(t, ())) | set(alias_to_paths.get(t, ()))
    sp = r["source_path"]
    for p in paths:
        total[p] += 1
        distinct[p].add(sp)
print(f"notes={len(notes)} edges={n}")

nflip=0; flips=defaultdict(int); inc_mismatch=0; inc_samples=[]; multi=0
flip_samples=[]
for path, m in notes.items():
    tot = total.get(path,0); dis = len(distinct.get(path,()))
    if tot!=dis: multi+=1
    created = max((m["created_at"] if m["created_at"] is not None else m["modified"]) or 0, 0)
    modified = max(m["modified"],0)
    dsc = max(now-created,0)//86400; dsm = max(now-modified,0)//86400
    mt=compute_state(tot,dsc,dsm); md=compute_state(dis,dsc,dsm)
    if mt!=md:
        nflip+=1; flips[(mt,md)]+=1
        if len(flip_samples)<15: flip_samples.append((m["name_lower"][:38],tot,dis,mt,md))
    if dis != m["incoming_count"]:
        inc_mismatch+=1
        if len(inc_samples)<12: inc_samples.append((m["name_lower"][:30],"dis="+str(dis),"inc="+str(m["incoming_count"]),"tot="+str(tot)))

print(f"COUNT(*) != COUNT(DISTINCT): {multi} notes")
print(f"maturity FLIPS total->distinct: {nflip}")
for k,v in sorted(flips.items(),key=lambda x:-x[1]): print(f"   {k[0]} -> {k[1]}: {v}")
for s in flip_samples: print("    ",s)
print(f"\nincoming_count == my COUNT(DISTINCT) ? mismatches={inc_mismatch}/{len(notes)}")
for s in inc_samples: print("   ",s)
con.close()
