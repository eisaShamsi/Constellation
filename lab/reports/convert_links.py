# -*- coding: utf-8 -*-
"""Link-Type Syntax Correction converter (one-time, Eisa Cognitive Knowledge).

Rewrites legacy predicate-LAST typed wikilinks to the canonical predicate-FIRST
form, preserving display text + target byte-for-byte:

    [[Stone Age|supports]]                 -> [[supports::Stone Age]]
    [[Time period|time period|supports]]   -> [[supports::Time period|time period]]

DEFAULT = DRY-RUN (writes nothing; reports counts + samples). Pass --apply to
write. Idempotent (skips existing `::`), skips ![[embeds]] and fenced code,
leaves untyped / display-only / malformed links untouched. UTF-8 + RTL safe;
line endings preserved.

Scope = the universe's registered library paths (libraries.json), deduped.
"""
import os, re, sys, json, collections

LIB_JSON = r"E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\libraries.json"
SKIP_DIRS = {".constellation", ".git", "attachments", ".obsidian", ".trash"}
KNOWN_TYPES = {
    "supports", "contradicts", "causes", "exemplifies", "generalizes",
    "derives-from", "part-of", "supersedes", "associative",
}  # NB: 'relates' deliberately excluded — it was never real data, only the bug default.
LINK_RE = re.compile(r"(?<!\!)\[\[(.+?)\]\]")
FENCE_RE = re.compile(r"^\s*(```|~~~)")

APPLY = "--apply" in sys.argv

def convert_body(body):
    """Return (new_body, changed)."""
    if "::" in body:
        return body, False                      # already canonical
    parts = body.split("|")
    if len(parts) == 2:
        target, tail = parts[0], parts[1].strip().lower()
        if target.strip() and tail in KNOWN_TYPES:
            return f"{tail}::{parts[0]}", True
    elif len(parts) == 3:
        target, display, tail = parts[0], parts[1], parts[2].strip().lower()
        if target.strip() and tail in KNOWN_TYPES:
            return f"{tail}::{parts[0]}|{display}", True
    return body, False

# ── gather library roots (deduped; drop a root nested inside another) ──
libs = json.load(open(LIB_JSON, encoding="utf-8"))
roots = sorted({os.path.normpath(l["path"]) for l in libs})
roots = [r for r in roots if not any(r != o and r.startswith(o + os.sep) for o in roots)]

per_lib = collections.Counter()
samples_by_lib = collections.defaultdict(list)
file_changed = 0
links_changed = 0
files_seen = 0

def process(text, lib_name):
    """Convert links in non-fenced-code lines. Returns (new_text, n_changed)."""
    n = 0
    in_fence = False
    out_lines = []
    for line in text.split("\n"):
        if FENCE_RE.match(line):
            in_fence = not in_fence
            out_lines.append(line)
            continue
        if in_fence or "[[" not in line:
            out_lines.append(line)
            continue
        def repl(m):
            nonlocal n
            nb, ch = convert_body(m.group(1))
            if ch:
                n += 1
                if len(samples_by_lib[lib_name]) < 2:
                    samples_by_lib[lib_name].append((m.group(0), f"[[{nb}]]"))
            return f"[[{nb}]]" if ch else m.group(0)
        out_lines.append(LINK_RE.sub(repl, line))
    return "\n".join(out_lines), n

for root in roots:
    lib_name = os.path.basename(root)
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for fn in filenames:
            if not fn.lower().endswith(".md"):
                continue
            files_seen += 1
            p = os.path.join(dirpath, fn)
            try:
                with open(p, encoding="utf-8", newline="") as f:
                    text = f.read()
            except Exception as e:
                print(f"  UNREADABLE {p}: {e}")
                continue
            new_text, n = process(text, lib_name)
            if n:
                file_changed += 1
                links_changed += n
                per_lib[lib_name] += n
                if APPLY and new_text != text:
                    with open(p, "w", encoding="utf-8", newline="") as f:
                        f.write(new_text)

mode = "APPLY (files written)" if APPLY else "DRY-RUN (no files changed)"
print(f"\n=== {mode} ===")
print(f"library roots: {len(roots)}")
print(f"files scanned: {files_seen}")
print(f"files with conversions: {file_changed}")
print(f"links converted: {links_changed}\n")
print("=== per-library ===")
for k, v in per_lib.most_common():
    print(f"{v:>8}  {k}")
print("\n=== before -> after samples (up to 2 per library) ===")
for lib in per_lib:
    for old, new in samples_by_lib.get(lib, []):
        print(f"  [{lib}]")
        print(f"    {old}\n      ->  {new}")
