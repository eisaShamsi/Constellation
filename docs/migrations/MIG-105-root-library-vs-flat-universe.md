# MIG-105 — The Root Library vs. the Flat Universe

**Status: SUPERSEDED — the Architect phase ran 2026-07-26 and the design is LOCKED by Boss
rulings.** The Boss's *Brain / Core Organizer* concept (2026-07-26) re-founded the question: the
entity **remains**, its scope is amended. Canonical current document:
`MIG-105-Architect-root-library-vs-flat-universe.md` (v2, locked design + rulings), with companion
verdicts `MIG-105-ArtDirector-verdict-Core-Organizer.md` and
`MIG-105-Inspectors-verdict-Core-Organizer.md`. This file remains as the original 2026-07-24
concept + evidence log. *(Original status: Boss-directed 2026-07-24: "Log it in its own /migration,
where we are going to handle it as soon as we finish the job in hand." No design was decided here.)*

---

## The concept (the horse)

> **A Library is a first-class citizen; a Folder is not. When you open your Universe
> you should see each library once, and every note should have exactly one owner —
> with no phantom copy of a library appearing as a folder inside another.**

The question the Boss actually asked: *"Do we need a root library that shares the
same universe title? Why shouldn't we have all the files under the root universe
directly, like Obsidian?"* — and the observed defect that prompted it: **a library
created inside the Universe root is duplicated in the sidebar**, appearing both as a
top-level library and as a folder inside the root library.

## What exists today (verified 2026-07-24)

- `ensure_universe_notes_folder` (`src-tauri/src/universe.rs:403`) auto-registers a
  library on Universe init with:
  - `name: meta.name.clone()` — **named after the Universe itself**,
  - `path` = the **Universe root**,
  - inserted at **index 0** of `libraries.json`,
  - `is_universe_notes: true`.
- Because its `path` is the root, it **physically contains every other registered
  library** that lives under the root. Rendering its tree (`read_library_tree` →
  `read_dir_recursive`, `libraries.rs`) descends into those nested libraries as if
  they were ordinary folders. → the sidebar duplication.
- Because it sits at **index 0**, any resolver that answers "which library owns this
  path?" by **first-match `starts_with`** returns the root library for *every* path
  in the Universe.

## Why this is a root cause, not an incident — the evidence trail

This one design decision has already produced, and been patched around, in at least
six places:

1. **Four first-match library-resolver bugs** fixed 2026-07-24 (`libraries.rs:918`,
   `:1203`, `:1807`; `universe.rs:2119`) — each filed a note under the wrong library
   because `universe_notes` at index 0 always won the prefix match.
2. **The Move dialog's path-dedupe hack** (`+layout.svelte` ~6315), whose comment
   names the Boss's own crash: *"froze+crashed on Eisa's 'New Library Test' nested
   under the universe root."*
3. **The sidebar duplication** (this migration's trigger; a display-layer fix is
   being applied under the current job — see the session log — but that is the fifth
   local patch for the same cause).

`library_name_for_path` (`libraries.rs:218`, longest-root-wins + separator-bounded)
is the *correct* resolver and already exists; the recurring bug is that call sites
reach for `starts_with` instead. The data model invites the mistake.

## The design options (to be deepened in the Architect phase)

**A — Exclusive-scope root library (recommended starting hypothesis).** Keep a root
owner so every note still has a library (indexing, search scoping, per-library
fonts/colours/appearance all key off `library_name`), but define its scope as **the
Universe root MINUS any registered library**. Also stop its display name mirroring
the Universe. Delivers the flat feel; kills the duplication, the resolver ambiguity,
and the Move-dialog hack at the source. Smallest behavioural change.

**B — Remove the root library entirely (true Obsidian flat).** Notes live directly
in the Universe, owned by nothing. Closest to the Boss's phrasing, but every path
that assumes a note has a library needs a "no library" case — indexing
(`note_meta.library_name`), search scoping, appearance/colour resolution. Larger
blast radius; from the user's side it looks identical to A.

**C — Forbid nesting.** Libraries must live outside the Universe root. Topologically
cleanest, but dictates the user's folder layout and forces migrating existing setups.

## Invariants the Architect phase must protect

- **Every existing note keeps an owner** through the transition (no note becomes
  un-indexed / invisible / mis-scoped).
- **Library ≠ Folder** becomes a rule the data model *enforces*, not a doc sentence.
- Boot time, indexing, and the file watcher must not regress on the 7,600+ note
  Universe.
- Rename/move/reindex must resolve the owning library via the canonical
  longest-root-wins resolver, never first-match.
- Cross-platform: path casing / NFC-NFD / separators (Windows now, macOS later).
- Reversible rollout with a back-fill for existing Universes.

## Relationship to the job in hand

The **sidebar duplication** is being fixed now as a **display-layer** change
(exclude nested registered libraries from the parent tree's walk) so the sidebar
stops lying — pending the blast-radius verdict that no functional path (indexing,
watcher, reveal-in-tree) depends on discovering a nested library's notes *through*
the parent's tree. That fix is deliberately scoped to the symptom; **it does not
touch the data model.** This migration is where the data model itself is corrected.

## Boss direction captured 2026-07-25 (icon-change conversation)

> "It shouldn't have any icon after MIG-105. There isn't going to be anything under
> the root, just the cUniverse(s) and Libraries." — Eisa

This is a strong steer toward **end-state B/C flavour**: after MIG-105 the Universe
root is a **pure container** — it holds **cUniverses and Libraries only**, with **no
notes/folders directly under it** and therefore **no library-content icon** on the
root row. Implication for the Architect phase: the auto-registered `universe_notes`
library (root-as-library) is retired or demoted to a non-content container; every
note lives inside a Library, never "loose at the root." The icon work done 2026-07-25
already anticipates this — the root row renders **no icon** now (libraries = building
icon, cUniverses = planet), so the visual is forward-compatible.

## Open questions for the Boss (Architect phase)

1. **A, B, or C** — which end-state? (Leaning A; B looks identical from the UI but
   is a bigger internal change.)
2. If A: what should the root owner be **called** in the sidebar, if not the
   Universe name? (e.g. a neutral "Universe notes" / "Loose notes" label, localised.)
3. Should nesting a library under the Universe root remain **allowed** at all, or be
   steered against at creation time (relates to option C)?

## Next step

Run the four-phase `/migration` Architect workflow (territory → options →
judge-panel → Architect doc) as the first task after the current job closes, exactly
as MIG-104 was produced. Filed in the ledger as **PJ-145**.
