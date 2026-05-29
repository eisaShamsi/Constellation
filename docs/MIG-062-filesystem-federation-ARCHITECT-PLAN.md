# MIG-062 — Federate the filesystem-walk sidebar surfaces (P3)

**Date:** 2026-05-29
**Combined Architect + Plan** (lightweight — design is locked by Boss answers; this is small enough not to warrant separate docs).
**Predecessor:** MIG-061 (P1 boot-snapshot federation). This is P3 from `docs/MIG-061-federation-audit-findings.md`.

---

## Goal

Make the three P3 "filesystem-walk" sidebar surfaces federation-aware, per Boss's locked design: **read-only display aggregation — never a data move or delete; each cUniverse keeps its own files intact; detach is lossless.**

## What the scoping agent found (corrections to the audit)

| Surface | Audit said | Reality | MIG-062 action |
|---|---|---|---|
| **Tag Browser** | broken (filesystem walk) | `allLibraryTags` is already federated (MIG-061 §M) and the `scan_library_tags` fallback already iterates the federated `libraries` store. The ONLY bug: `NotebookNavigator.svelte:165` sets `tagMap` once on mount with **no reactivity** on the `initialTags` prop → the `federation:ready` refresh doesn't propagate. | **1-line `$effect`** (frontend) |
| **Five Acts sidebar** | broken (parent-only) | `list_five_acts_notes` reads only `{active}/Five Acts/`. Confirmed parent-only. | Federate (Boss: yes) |
| **Workspace Bases** | broken (parent-only) | `list_workspace_bases` reads only `{active}/.constellation/bases/`. Confirmed parent-only. | Federate (Boss: yes) |

## Locked decisions (Boss answers, 2026-05-29)

1. **Five Acts → federate**, but the cUniverse's own Five Acts stays intact ("don't delete, maybe just hide it"). The parent sidebar *reads & displays* cUniverse Five-Acts notes; touches nothing. Detach → cUniverse works standalone.
2. **Workspace Bases → federate**, same read-only/standalone guarantee.
3. **UI:** group by universe — parent's entries shown normally; each cUniverse's entries under a **collapsible sub-group** labeled by universe name (the "maybe just hide it").

## Architecture

- **cUniverse root enumeration:** `universe.rs::resolve_child_universe_roots` (line 425) already exists and its doc comment names this exact use case ("tag browser federation, sky view merging"). It reads cUniverse roots from `universe.json`'s `children` — **manifest-based, no federation-ATTACH dependency**, so no boot-race (unlike MIG-061's `federated_conn` timing). Make it `pub(crate)` + add a **recursive** variant so MIG-062 covers the same federated set CNS does (cUniverses-of-cUniverses).
- **Read-only invariant:** both backend commands only `fs::read_dir` cUniverse paths. No writes, no moves, no deletes. This IS the "wheel is already there" principle — each universe's files stay its own.
- **Disambiguation:** add a `universe_name: Option<String>` field to `FiveActsNoteEntry` and `WorkspaceBaseEntry` — `None` (or parent name) for the active universe, `Some(cuniverse_name)` for federated entries. Frontend groups on it.

---

## Steps

### §A — Tag Browser reactivity fix (1 commit, frontend)

`src/lib/components/NotebookNavigator.svelte`: add a `$effect` that re-syncs `tagMap` when the `initialTags` prop changes (post `federation:ready`). Currently `tagMap` is assigned once inside the `onMount` block (line 165). Verify `tagMap` is `$state`; if it's a plain `let`, promote it.

**Verify:** `svelte-check` clean. (Boss-test: Tag browser shows cUniverse tags in Eisa Universe.)

### §B — `resolve_child_universe_roots` → `pub(crate)` + recursive (1 commit, Rust)

Expose it and add `resolve_child_universe_roots_recursive(parent) -> Vec<PathBuf>` that walks the full federation tree (guard against cycles with a visited set, mirroring `resolve_libraries_recursive`'s pattern). Unit test: 2-level federation returns all roots.

**Verify:** `cargo test --lib universe` passes + new recursion test.

### §C — Five Acts federation (1 commit, Rust)

`lens/system_notes.rs::list_five_acts_notes`: after the active-universe pass, loop `resolve_child_universe_roots_recursive(active)`, call `list_five_acts_notes_at(cu_root)` per root, tag each entry with `universe_name = Some(cu_display_name)`. Add `universe_name: Option<String>` to `FiveActsNoteEntry`. Active-universe entries keep `universe_name = None`.

**Verify:** `cargo test --lib` passes; manual: returns parent + cUniverse entries.

### §D — Workspace Bases federation (1 commit, Rust)

`bases.rs::list_workspace_bases`: same pattern — loop cUniverse roots, read `{cu_root}/.constellation/bases/` per root, tag with `universe_name`. Add `universe_name: Option<String>` to `WorkspaceBaseEntry`.

**Verify:** `cargo test --lib` passes.

### §E — Frontend per-universe grouping (1 commit, Svelte)

`+layout.svelte` Five Acts section (~5030) + Bases section (~5058): partition entries by `universe_name`. Render active-universe entries as today; render each cUniverse group under a collapsible sub-header (universe name + chevron, collapsed-by-default per "maybe just hide it"). New `$state` Sets to track which cUniverse sub-groups are expanded. RTL-safe (`detectDir` on universe names — many are Arabic). 15-locale: reuse existing section labels; the cUniverse names are data, not translatable strings.

Also extend the `federation:ready` listener (the §N block) to re-invoke `listFiveActsNotes()` + `listWorkspaceBases()` so cUniverse entries appear once the manifest/libraries settle. (Manifest-based enum means they're usually available at first call, but the re-fetch is cheap insurance + handles universe-switch.)

**Verify:** `svelte-check` clean. Boss-test below.

### §F — Boss-test (1 commit, doc)

`docs/MIG-062-BOSS-TEST.md`:
1. **Tag Browser** — Eisa Universe: tag browser shows tags from cUniverse notes.
2. **Five Acts** — sidebar shows parent's "Observation — Recent Captures" + a collapsible group per cUniverse with its own Observation note.
3. **Workspace Bases** — if any cUniverse has saved bases, they appear under a collapsible cUniverse group.
4. **Standalone integrity** — (conceptual) detaching a cUniverse leaves its Five Acts + bases intact. We verify by confirming nothing is written to cUniverse dirs (read-only code review + the cUniverse's files unchanged after viewing).
5. **Single-universe** — Eisa Cognitive Knowledge: sidebar looks exactly as before (no empty cUniverse groups).

### §G — PCS (1 commit)

Orientation v2.42, MoCh (2026-05-29 block), 15-locale help-doc updates (batched: CNS/Backlinks/Tag/Five-Acts/Bases federation — deferred from MIG-061), milestone tag `milestone/mig-062-filesystem-federation-shipped`, ZIP backup. Mark Tag Browser + Five Acts + Workspace Bases ✓ in the audit findings doc (→ 8 of 14 surfaces closed after MIG-062).

---

## Invariants

| # | Invariant | Verification |
|---|---|---|
| INV-1 | **Read-only** — no writes/moves/deletes to any cUniverse path | Code review: both commands only `fs::read_dir`. Detach test conceptual. |
| INV-2 | Single-universe sidebar unchanged (no empty cUniverse groups, no perf change) | Boss-test §5 |
| INV-3 | `FiveActsNoteEntry` / `WorkspaceBaseEntry` shape is back-compat (new field is additive `Option`) | Type check; existing consumers ignore the new field |
| INV-4 | Recursive enum doesn't infinite-loop on a federation cycle | §B visited-set + unit test |
| INV-5 | RTL universe names render correctly in sub-group headers | Boss-test (Arabic cUniverse names) |

## Out of scope

- MIG-063 (P2 read-path: Index / Knowledge Health / Unlinked Mentions / previews).
- MIG-064 (P2+P4 write-path: Cataloger / Classifier / NSC + FK).
- Cross-universe wikilink resolution (locked Option A in MIG-061 §L — stays per-universe).

## Approval

Boss approves this combined doc → Build cascades §A → §G, stopping at §F (Boss-test) and §G completion. Lightweight audit (re-use the MIG-061 3-agent pattern only if the build surfaces surprises; otherwise the read-only invariant + Boss-test suffice for a P3 read-only change).
