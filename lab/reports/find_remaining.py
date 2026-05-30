# -*- coding: utf-8 -*-
"""Locate the residual type-last typed links + show whether each sits in a code fence."""
import os, re, json
LIB_JSON = r"E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\libraries.json"
SKIP = {".constellation", ".git", "attachments", ".obsidian", ".trash"}
TYPES = {"supports","contradicts","causes","exemplifies","generalizes","derives-from","part-of","supersedes","associative"}
LINK = re.compile(r"(?<!\!)\[\[([^\]]+?)\]\]")
roots = sorted({os.path.normpath(l["path"]) for l in json.load(open(LIB_JSON, encoding="utf-8"))})
roots = [r for r in roots if not any(r != o and r.startswith(o + os.sep) for o in roots)]
found = 0
for root in roots:
    for dp, dn, fns in os.walk(root):
        dn[:] = [d for d in dn if d not in SKIP]
        for fn in fns:
            if not fn.lower().endswith(".md"): continue
            path = os.path.join(dp, fn)
            try: lines = open(path, encoding="utf-8", newline="").read().split("\n")
            except: continue
            fence = False
            for ln, line in enumerate(lines, 1):
                if re.match(r"^\s*(```|~~~)", line): fence = not fence
                for m in LINK.finditer(line):
                    body = m.group(1)
                    if "::" in body: continue
                    parts = body.split("|")
                    if len(parts) in (2,3) and parts[-1].strip().lower() in TYPES:
                        found += 1
                        print(f"[fence={fence}] {os.path.basename(path)}:{ln}  [[{body}]]")
print(f"\ntotal residual type-last typed links: {found}")
