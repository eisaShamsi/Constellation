---
title: "README - PJ-065 Test Book"
cid_cn: 20260627T120400Z_NOTE_A010
kind: note
created: 2026-06-27T12:04:00Z
---

# README — PJ-065 Test Book

A small, complete work used to validate the **structural (parent / TOC) link type**. Open this folder as a Library in Constellation.

## The book — "The Atlas of Lost Places"

```
The Atlas of Lost Places              (Book — declares its parts in order via contains:)
├── Part I - The Cartographer         (declares its own parent AND its chapters in order)
│   ├── Chapter 1 - The Old Atlas     (declares one scene)
│   │   └── Scene 1 - Dust and Ink    (level 4; declared from both sides → dedupe)
│   └── Chapter 2 - A Crease in the Paper   (also has a cognitive "supports" link)
├── Part II - The Voyage              (declares only its parent; children come from the child side)
│   ├── Chapter 3 - The Storm         (names Part II as parent)
│   └── Chapter 4 - Landfall          (names Part II as parent)
└── Part III - The Shore              (only listed by the Book; an empty leaf)
```

**What each note exercises**

| Note | Exercises |
|---|---|
| The Atlas of Lost Places | parent-declares-children, **ordering** (Part I, II, III) |
| Part I - The Cartographer | **both** directions on one note + **dedupe** (Book also lists it) |
| Part II - The Voyage | children assembled from the **child side** (union-on-read) |
| Part III - The Shore | child-known-only-from-parent; **empty subtree** |
| Chapter 1 / Scene 1 | **3–4 levels** deep + **dedupe at depth** |
| Chapter 2 | **no cognitive inflation** — its `supports` link counts, its placement does not |
| Chapter 3 / Chapter 4 | two children from the child side, stable fallback order |

## Guard Tests (subfolder)

| Note(s) | Exercises |
|---|---|
| Loop Note Alpha / Beta | **acyclicity** — the app rejects the loop-closing edge, never hangs |
| Owner A / Owner B / Contested Child | **single-parent** — the child's own `parent:` wins, deterministically; the conflict is surfaced, files are never rewritten |

The detailed step-by-step tutorial accompanies the Boss test gates (A / B / C).
