# -*- coding: utf-8 -*-
"""Analyze every wikilink form in the Eisa Cognitive Knowledge universe.
Read-only: walks the library roots, categorizes [[...]] links, reports the
distribution + examples + anomalies so the converter design is grounded in
the real data shape (no guessing)."""
import os, re, sys, collections

ROOTS = [
    r"E:\Cognitive Knowledge",
    r"E:\Constellation Universes\Eisa Cognitive Knowledge",
]
SKIP_DIRS = {".constellation", ".git", "attachments", ".obsidian", ".trash"}

KNOWN_TYPES = {
    "supports", "contradicts", "causes", "exemplifies", "generalizes",
    "derives-from", "part-of", "supersedes", "associative", "relates",
}

# Capture wikilink bodies, but NOT image embeds ![[...]] (negative lookbehind).
LINK_RE = re.compile(r"(?<!\!)\[\[(.+?)\]\]")

cat = collections.Counter()
last_seg = collections.Counter()       # value of the trailing segment when 2-3 parts
type_when_typed = collections.Counter() # recognized type distribution
examples = collections.defaultdict(list)
anomalies = collections.defaultdict(list)
n_files = 0
n_links = 0

def add_example(key, s, cap=6):
    if len(examples[key]) < cap:
        examples[key].append(s)

for root in ROOTS:
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for fn in filenames:
            if not fn.lower().endswith(".md"):
                continue
            n_files += 1
            p = os.path.join(dirpath, fn)
            try:
                text = open(p, encoding="utf-8").read()
            except Exception as e:
                anomalies["unreadable"].append(f"{p}: {e}")
                continue
            for m in LINK_RE.finditer(text):
                body = m.group(1)
                n_links += 1
                if "::" in body:
                    cat["already_colon_form (type::X)"] += 1
                    add_example("already_colon_form (type::X)", body)
                    continue
                parts = body.split("|")
                npart = len(parts)
                if npart == 1:
                    cat["untyped [[X]]"] += 1
                    add_example("untyped [[X]]", body)
                elif npart == 2:
                    last = parts[1].strip().lower()
                    last_seg[last] += 1
                    if last in KNOWN_TYPES:
                        cat["2-part typed [[X|type]]"] += 1
                        type_when_typed[last] += 1
                        add_example("2-part typed [[X|type]]", body)
                    else:
                        cat["2-part display [[X|display]]"] += 1
                        add_example("2-part display [[X|display]]", body)
                elif npart == 3:
                    last = parts[2].strip().lower()
                    last_seg[last] += 1
                    if last in KNOWN_TYPES:
                        cat["3-part typed [[X|display|type]]"] += 1
                        type_when_typed[last] += 1
                        add_example("3-part typed [[X|display|type]]", body)
                    else:
                        cat["3-part NON-type tail [[X|Y|Z]]"] += 1
                        add_example("3-part NON-type tail [[X|Y|Z]]", body)
                        anomalies["3-part non-type tail"].append(body)
                else:
                    cat[f"{npart}-part (4+) anomaly"] += 1
                    add_example(f"{npart}-part (4+) anomaly", body)
                    anomalies[f"{npart}-part"].append(body)

print(f"files scanned: {n_files}")
print(f"wikilinks found: {n_links}\n")

print("=== category distribution ===")
for k, v in cat.most_common():
    print(f"{v:>8}  {k}")

print("\n=== recognized type distribution (typed links) ===")
for k, v in type_when_typed.most_common():
    print(f"{v:>8}  {k}")

print("\n=== top 25 trailing-segment values (2/3-part) — to spot unknown 'types' ===")
for k, v in last_seg.most_common(25):
    flag = "TYPE" if k in KNOWN_TYPES else ""
    print(f"{v:>8}  {flag:<5} {k!r}")

print("\n=== examples per category ===")
for k in cat:
    print(f"\n# {k}")
    for s in examples[k]:
        print(f"    [[{s}]]")

print("\n=== anomalies (count) ===")
for k, lst in anomalies.items():
    print(f"{len(lst):>8}  {k}")
    for s in lst[:5]:
        print(f"           {s!r}")
