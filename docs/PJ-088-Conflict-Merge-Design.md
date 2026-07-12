# PJ-088 — Conflict-Resolution Side-by-Side Merge — Design

*Art Director design workflow `wf_d7453254-50e` (11 agents: 3 census + WA#5 prior art + 3 competing designs + 3 adversarial judges + lead synthesis). Boss-requested 2026-07-12 (shape = side-by-side, build now). The follow-up conflict-resolution layer the PJ-070 Architect deferred.*

## Concept (one sentence)
When an outside edit lands on a note you were editing, Constellation keeps *both* versions and opens a two-column view — **your version** beside **the outside copy** — where you pull across whatever you want and edit freely into **one** reconciled note; the app never chooses for you, and nothing is lost until you press **Save merged**.

## The design (chosen: synthesis "Design C")
- **Diff engine off-the-shelf:** the official CodeMirror-6 **`@codemirror/merge`** `MergeView` — 2-way side-by-side, per-chunk accept arrows, change gutter, within-line highlight, collapse-of-identical — a drop-in match for the mockup. Prior art (Obsidian conflict-file-then-manual-merge, Meld 2-way, Syncthing sidecars) converges on **2-way side-by-side + copy-across + free edit** for a notes app; 3-way is rejected (no common ancestor is stored — the sidecar is only the incoming snapshot).
- **Surface:** a full-center-zone overlay (Style-Setter mount pattern), never a cramped modal. Left = **Your version** (editable, seeded from the live model's current content — your unsaved edits); Right = **Outside copy** (read-only, `read_note(sidecarPath)`). Both panes hold the full note (frontmatter + body); collapse-identical folds matching regions.
- **Copy-across:** each differing chunk has an arrow (outside → yours); the left pane free-edits. Footer: **[Cancel]** · **[Save merged]**.

## The safety wire (non-negotiable — lead-engineer-owned)
**Save merged writes through the note's single in-memory model + the durability gate — never a raw fs write over the open note** (a raw write reintroduces the PJ-070 Recipe-O clobber: the still-dirty model's next autosave would compose the stale body over the merge). New `resolveConflictMerge(notePath, sidecarPath, mergedText)` — a near-verbatim adaptation of the shipped, audited `resolveStructuralConflict`/`toggleTaskReconciled` template:
1. Find the tab (open the note as a tab first if needed — a model must exist).
2. `markCascading(notePath)` (gates the armed autosave + the outgoing `{#key}` teardown flush) in a try/finally.
3. `parseFrontmatter(mergedText)` → props+body.
4. **Push the merge INTO the one model, path-guarded:** `editProps(id, props, notePath)` + `editBody(id, body, notePath)` — the model *is* now the merge; no stale copy left to overwrite it (the core defense).
5. `markRecentWrite(notePath)` (watcher-echo suppression).
6. `await saveNoteSession(id, notePath, standardSaveEnv({ origin:'merge_resolve', onSaved:<reindex+broadcast+embed> }))` — composes from the model, net-before-write, durable atomic/journaled write, mark-clean+re-baseline only on proven durability.
7. **Gate:** `if (!outcome.ok) return` — model stays dirty, net retained, save-health banner surfaces it; nothing irreversible ran.
8. `reloadTabsFromDisk([notePath])` — remount NotePane on the merged disk; **bracket with `markReseeding` + `await tick()`** so the outgoing editor's teardown can't re-stale (the strongest existing gate, from `adoptExternalChangeIntoTabs`).
9. **FocusPane** (not under the `{#key}`): `focusReseed(notePath)`.
10. Sidecar resolution — **only after `outcome.ok`:** `moveToTrash(sidecarPath, <longest-prefix library>)` (reversible, never hard-delete; handles the `.md.txt` extension) + `dismissConflict(sidecarPath)`.

**Cancel = pure no-op:** the merged text lives inside the MergeView and is pushed into the model only at Save — so Cancel touches nothing (model keeps its unsaved edits, sidecar + banner stay). **Both versions live on disk until an explicit durable Save.**

## Reuse (feedback_reuse_components + Rule 6)
Banner (one-line fix to expose `notePath` in the `conflicts` derived) + **Merge…** button · the whole save wire (`resolveStructuralConflict` template + `standardSaveEnv` + `saveNoteSession` + `reloadTabsFromDisk` + `markReseeding`/`focusReseed`) · `read_note` (no `.md` restriction — reads the `.md.txt`) · `moveToTrash` + `dismissConflict` · the Style-Setter overlay + top-level mount · the `conflict.*` i18n block (15 locales, RTL). **New:** `@codemirror/merge` (**lazy-imported** in the overlay's open action → out of the main bundle + hot path; post-build bundle check), `mergeView.ts` store (~15 lines), `ConflictMergeView.svelte`, `resolveConflictMerge()` (~30 lines).

## Build plan (commit-sized, each with a verification clause)
1. **Dep + store + host skeleton** — `@codemirror/merge`; `mergeView.ts`; empty overlay at `+layout.svelte` top level. *Verify: builds; bundle check confirms the dep is dynamic-imported + absent from the main chunk.*
2. **Entry point** — expose `notePath` in the banner `conflicts` derived; add **Merge…** to the conflict row (ensures the note is open first). *Verify: Merge… appears (LTR+RTL), opens the overlay with the right paths.*
3. **Two-pane view (read-only, no save)** — dynamic-import `@codemirror/merge`; left = read-only `composeForView(id)` accessor (side-effect-free over `composeModel`); right = `read_note(sidecar)`; 2-way, arrows outside→yours, collapse-identical, plain-ish panes (syntax + bidi; live-preview OFF — documented comparison-surface exception, like FocusPane). *Verify: diffs show, arrows copy, left free-edits; RTL ar+he pair test.*
4. **The safety wire** *(safety-inspection diff-scoped here)* — `resolveConflictMerge` (the 10-step sequence incl. the teardown-span hardening + focusReseed); wire Save merged → on ok: trash + dismiss + close; on !ok: keep everything. Cancel → close only. *Verify (running-app Editor-Surface Gate): merge-save round-trip on-screen===disk; NotePane + Focus + tab-switch-in-Focus show the merge; forced write-failure leaves sidecar+banner+dirty model intact; Cancel changes nothing; Reproduce-First the re-stale teardown red→green.*
5. **i18n ×15 + docs** — new `conflict.*` keys (RTL); help topic + User Manual (+14); orientation v-bump; PJ ledger v1.20 (close PJ-088) — same commit. *Verify: every locale renders; no missing-key fallbacks.*

*Deferred (not v1): the file-tree secondary entry (the `.md.txt` sidecar is hidden from the tree, so it would target the note with a `saveConflicts`-gated menu item).*

## Invariants
1. Save through model+gate, never a raw write. 2. Identity-bound writes (path-guarded compose). 3. Trash+dismiss only after `outcome.ok` (zero-loss until durable Save). 4. Reversible (`.trash`, never hard-delete). 5. Both surfaces remount (NotePane `{#key}`, FocusPane `focusReseed`). 6. Outgoing teardown can't re-stale (`markCascading`+`markReseeding`+`await tick()`). 7. Cancel = pure no-op. 8. No hot-path regression (lazy dep, out of main bundle). 9. Language-First (RTL two-pane verified, not assumed).

## Open question for the Boss — RESOLVED
**Boss ruling 2026-07-12: FULL LIVE PREVIEW in the panes.** The panes render with the shared NotePane live-preview extension set (Editor Parity Rule — no exception needed). Build-spike caveat carried: live-preview decorations + the merge diff-highlight are two decoration layers; if they genuinely clash (Step 3 verification), surface it to the Boss with the plain-ish (syntax-only) panes as the safe fallback.

*Transparency (not a decision): the merged **body** is written byte-for-byte; the **frontmatter** is normalized on save (quote style / key order / comments re-projected) — the same YAML tidying every property-panel save already does. The left pane is seeded from that same composed content, so what you see is what gets saved.*
