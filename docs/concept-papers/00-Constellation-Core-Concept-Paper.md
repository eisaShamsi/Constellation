# 00 — Constellation (Core Concept Paper)

> **THE root concept paper.** Every per-function paper (01…NN) must show how its function serves *this* purpose. If a function cannot trace its existence back to this paper, it has not justified itself (Constraint as Design). Boss-designated the core, 2026-06-15.

## 1. Function in hand
**Constellation** — a **Personal Knowledge *Formulation*** (PKF) system. A Tauri desktop app (Rust + SvelteKit/Svelte 5) whose window opens onto plain `.md` files on disk. Not management — **formulation**.

## 2. Purpose
Constellation exists to help one mind **connect, challenge, synthesize, and build understanding** — not to store information. Storage volume is not the goal; *awareness of connections* is. The whole system is organized around **The Five Acts of Knowledge Creation**, a single pipeline with a single destination:

> **Observation → Connection → Tension → Synthesis → Conviction.**

Every function in Constellation must advance at least one Act toward **Conviction** (a held, defended position). That is the paper's one test for any feature: *which Act does it serve?* The search engine is a **diagnostic instrument** for intellectual life, not a file finder. Links are **living vessels** carrying type, annotation, weight, confidence, and temporal data — not flat strings.

## 3. What it is NOT
- **Not a file manager** — a Library is a first-class knowledge base, not a folder; a Folder is mere organization inside it.
- **Not storage-PKM** — it does not measure success by how much you've saved.
- **Not a single-language app, ever** — multilingual is foundational, not a setting (§6).
- **Not a read-time recompute engine** — derived views are maintained at write time (§7, Rule 8).

## 4. The architecture (the spine every function attaches to)
- **The Note Editor is the gate** ([01](01-Note-Editor.md)). Knowledge enters through it; every downstream surface (search, backlinks, graph, tags, second screen) learns of a change only because the Editor dispatched an event or fired the reindex. No silent reads.
- **File Over App** — `.md` files on disk are the source of truth; the app is just a window. Never modify file content silently; never a proprietary format. Standard Markdown + YAML frontmatter.
- **Local-First** — all data on device; no telemetry, no cloud dependency; works fully offline, instantly, always. Sync is the user's choice (Git/Syncthing/iCloud).
- **The Knowledge Hierarchy** — **Universe → Library → Folder → Note**, with an optional **cUniverse** federation layer at the top. The Universe root is itself a Library (`universe_notes`, Obsidian-flat). Federation flattens child universes at runtime.
- **The Living Link Architecture** — links are first-class objects (the `LINK` file kind) with eight properties (type, direction, annotation, weight, confidence, created, last-traversed, traversal-count), four confidence levels (hypothesis→evidence→established→contested), weight earned through use, every operation reversible (archival, not deletion). The **8 typed links** are the cognitive vocabulary: supports, contradicts, causes, exemplifies, generalizes, derives-from, part-of, supersedes (+ `associative` as the null default).
- **Constraint as Design** — every feature must justify its existence; **Form-Aligns-To-Purpose** — every part of every feature must justify its presence within it. When in doubt, do less.

## 5. Right-click / context menu — a first-class surface
Constellation is **navigable by right-click**: "right-click should include every aspect of the app" (Boss, the MIG-077 charter). Context menus route through one shared builder (`buildContextMenu()` / `<ContextMenu>`) so every element (note, folder, library, tab, link, term, graph node, second screen) exposes its actions consistently — never a hand-rolled per-surface menu. **Every per-function paper states that function's right-click contract** (§5 of the template); a function with actions reachable *only* by right-click must have them verified in the bring-up.

## 6. Multilingual — by default, from the ground up
Constellation supports **all 15 languages simultaneously, by design** (ar, de, en, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh). This is architectural, not an add-on:
- Every user-facing string flows through `$t()` and exists in all 15 locale files.
- **Per-line bidirectional text** (`bidiPlugin`) + per-script fonts are a core editor feature — Arabic, Hebrew, CJK, Devanagari, Cyrillic, mixed-script, in any view.
- **RTL** everywhere: `dir` / `detectDir()`, flipped chevrons/arrows.
- **Native equivalents, not transliteration** — when the language switches, *everything* adapts (chip names, dome labels, stratum/sector labels), using the right native term (مصادر, not "masādir"). This top-principal overrides "brand names stay English" where they conflict.
- No layout, font, cursor, or input assumption may be single-language. **Every per-function paper states its multilingual conformance** (§6 of the template).

## 7. Performance ethos — "Fast Software, the Best Software"
Speed is a proxy for engineering quality. The eight Performance Rules are binding: every keystroke instant (no `invoke()` on the hot path); no `$effect` loops; no heavy work on the main thread (Rust does indexing/search/IO); no memory leaks; minimal DOM; no unnecessary imports; test before commit; and — the spine of the bring-up — **Rule 8, Write-Time Derivation: every computed view is maintained at write time, not read time; the app does not recompute on boot, it reads what's already stored.** No new feature may regress boot time, typing latency, or IPC responsiveness.

## 8. The contract every function serves (how the per-function papers gate on this)
A function earns its place in Constellation only if its concept paper can answer:
1. **Which Act does it serve?** (Observation / Connection / Tension / Synthesis / Conviction.) If none — it doesn't belong.
2. **Does it justify its existence, and is every part of it justified?** (Constraint as Design + Form-Aligns-To-Purpose.)
3. **Does it obey File-Over-App, Local-First, and Rule 8?** (No silent writes; no cloud; no boot recompute.)
4. **Is it right-clickable where it should be, and fully multilingual?** (§5, §6.)
5. **Does it wire to the Editor (the gate) without re-implementing save/load/edit?** (Additional screens are displays, not domains.)

The bring-up re-enables each function only when its paper's §10 checklist passes against this contract.

## 9. Status
Concept paper: **draft (the core / root)** · Applies to: **the whole system** · Owner decision: **Boss-designated core, 2026-06-15**.
Notes: This paper is the reference all per-function papers cite. Full specification: `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`. Governance + the bring-up program: [00-MASTER](00-MASTER-Bring-Up-Charter-and-Checklist.md). The first function paper is [01 — Note Editor](01-Note-Editor.md), the gate.
