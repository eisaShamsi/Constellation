# Handover — 2026-07-08 session close (Save-Durability shipped → next: APP-KILLER #2)

**Read first each session:** `docs/Constellation Orientation & Onboarding v3.33.md` (highest version). This handover is the fast pick-up for the *next* session.

---

## What shipped this session (both Boss-validated, all on `origin/main`, tree clean)

Two Safety-Audit remediations, each a full four-phase `/migration` (Architect → Plan+Boss-approval → Build cascade → per-cycle safety sweep). **Both migration diffs came back CLEAN** (no new app-killer).

1. **Watcher-Index-Freshness** (orientation **v3.32**, commits `c24a430d..adbe373e`). The file watcher was emit-only → external `.md` changes (Obsidian sync / git pull / file drop) never reindexed `note_meta` at runtime → Quick Switcher / Search / Index / backlinks / counts stale until reboot. Fixed: `reindex_changed_paths()` on the watcher flush (reuses `reindex_single_note`/`_delete`; dir-rename via `reindex_md_descendants` + `delete_rows_under_prefix`; offline-drive guard). Architect `docs/Watcher-Index-Freshness-Architect.md`. Boss-validated: single add · folder rename · bulk sync.

2. **Save-Durability = APP-KILLER #1 FIXED** (orientation **v3.33**, commits `81d5873c..d4d9f218`). `NoteEditor.svelte:233` marked the note clean BEFORE the disk write + swallowed the error → a transient `.md` lock silently lost the edit + defeated the F2 cascade guard. Fixed: all 5 save sites consolidated onto the hardened `noteSession.save()` (net-before-write → await → mark clean ONLY on a durable write → compare-and-clear; on failure keep dirty + retain net + surface the `saveHealth` banner). `SaveHealthBanner.svelte` + Retry + ~10 s auto-retry + i18n×15. WA#6 fold-in: `store.ts:824` saveLocks-drop fixed. Architect `docs/APP-KILLER-Save-Durability-Architect.md`; harness `tests/mig-076/runtimeHarness.test.ts` (16/16). Boss-validated: Stage A (no regression) · Stage B (read-only `.md` → banner + edit survives + Retry recovers).

Also this session: MIG-099 create-latency + G4 frontmatter round-trip (from the prior block; v3.31). Docs: help/manual notes (Syncing + Saving-and-Recovery) propagated to all 15 languages. MoCh `docs/MoCh/MoCh-2026-07-08-1500.md`. Safety register appended to `docs/Constellation-Safety-Audit-CHARTER.md` (both `wf_8a41970f-36d` + `wf_5f9b257d-a99`).

---

## NEXT MIGRATION — APP-KILLER #2 (the 3rd and last worst-case silent-loss bug)

**Boss-approved to take as its own `/migration`.** Class: **notemodel-ownership** (single content ownership — CLAUDE.md content-integrity / Solve-the-Class).

**The defect (confirmed, `wf_5f9b257d-a99`):**
- **`store.ts:1787`** — `openNoteTab` in-place tab reuse (default file-tree / wikilink click, `newTab` falsy) calls `openNoteModel(currentTab.id, B, contentB)` which **unconditionally replaces** the tab's note model with B, **discarding the outgoing note A's dirty (unsaved) edits with NO flush**. The `{#key}` teardown flush that should save A bails: `handleFlush` returns because `filePath(A) !== tab.path(B)` (NoteEditor.svelte guard), and `compose(id, A)` would refuse (model now holds B). Trigger: type in A, click a `[[wikilink]]`/another note **within the 1500 ms debounce window** → up to ~30 s of just-typed text lost silently. **2nd instance `store.ts:1013`** (`loadTabHistoryEntry`, Alt+Left/back-forward).
- **`+layout.svelte:3320`** (sibling, HIGH) — the same note open in **two tabs** = two independent models; a save from one is never reconciled into the sibling (only the second-screen `onNoteSaved` calls `externalChange`/`adoptDisk`, and only first-match). A later stale-sibling save then **clobbers** the first tab's on-disk edits. Related: `openNoteTab` dedups only against the ACTIVE tab (`store.ts:1553/1647`), so Ctrl+click a note already open in a background tab creates a **duplicate model** for the same path.

**Recommended end-state (Solve-the-Class):** before any `openNoteModel` REPLACES a tab's existing model, **flush the outgoing dirty model to disk first** (through the `noteSession.save()` durability gate shipped this session — reuse it, don't reinvent) — never a silent replace. And for the duplicate-model case: either dedup `openNoteTab` against ALL tabs (focus the existing tab instead of minting a 2nd model), or reconcile same-path siblings on save via `adoptDisk`. Design it WHOLE (Architect), prove red→green in the harness (extend `tests/mig-076/runtimeHarness.test.ts` — a "nav-away-while-dirty flushes the outgoing model" case), land behind the Editor-Surface Gate checklist, Boss-test.

**Process (do NOT skip — this is editor-lifecycle, feedback_bug023):** full `/migration` — Architect workflow (adversarial, map every `openNoteModel`/`models.set` replace site + the teardown-flush interaction + cross-window) → Plan + **Boss approval** → Build cascade (Reproduce-First) → per-cycle safety sweep. The Editor-Surface Gate (8 items) is the runtime proof.

---

## Open follow-ups (not blocking the next migration)

- **Focus-reconcile for open-note external edits** — Boss-approved (during watcher-freshness) but deferred: today `+layout.svelte:3146` live-reloads an open tab's buffer on an external change; switch to reconcile-on-focus (Obsidian-style, protects unsaved edits). Its own small change (Editor-Surface Gate).
- **G4 gap** — `yamlDoc.ts:150/254` `serializeLine` has no nested-object-list branch → editing an ikhtilāf/nested-object-list frontmatter property flattens its structured YAML on disk. Fresh G4 defect; own fix.
- The standing **G2–G8 backlog** in the Charter (Rust non-atomic JSON writes `universe.rs`/`review.rs`/`link_types.rs`; `search.rs` archive/unarchive incoming-recompute; `libraries.rs` folder-cascade/delete-descendants/sync-walk; `livePreview.ts:242` image-cache leak; `cece/orchestrator.rs:153` timeout). None are silent-content-loss app-killers; sequence per the Boss.

## Where to read (fresh session)
`docs/Constellation Orientation & Onboarding v3.33.md` (§ preamble + migration table) → `lab/reports/SESSION-LOG-2026-07-08.md` → `docs/Constellation-Safety-Audit-CHARTER.md` (the full 23-finding register + triage) → `docs/APP-KILLER-Save-Durability-Architect.md` (the pattern the next migration reuses). Memory: `project_watcher_index_freshness_shipped`, `project_safety_audit_active`.

---

## Ready-to-paste next-session prompt

> Start the next Safety-Audit migration: **APP-KILLER #2 — notemodel-ownership silent nav-loss.** Read `docs/handover/Handover-2026-07-08-save-durability-close.md` and orientation v3.33 first. The defect: `openNoteTab` in-place tab reuse (`store.ts:1787`, and `loadTabHistoryEntry` at `store.ts:1013`) replaces a tab's note model via `openNoteModel` WITHOUT flushing the outgoing note's dirty edits — clicking a `[[wikilink]]`/another note mid-typing silently loses up to ~30 s of text; plus the two-tab same-note clobber (`+layout.svelte:3320`). Take it as its own `/migration`: adversarial Architect workflow first (map every model-replace site + the teardown-flush interaction + the duplicate-model case + cross-window), then bring me the Plan for approval before building. Reproduce-First — extend the `tests/mig-076/runtimeHarness.test.ts` harness with a "nav-away-while-dirty flushes the outgoing model" red→green case. Reuse the `noteSession.save()` durability gate shipped in v3.33 — don't reinvent. End with the per-cycle safety sweep + the Editor-Surface Gate Boss test.
