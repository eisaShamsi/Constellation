# Constellation Safety & Integrity Audit — Charter

**Opened:** 2026-07-07 · **Mandate (Eisa):** *"Stop everything and put the app under inspection to find and fix those app-killing bugs. I don't care how long it will take or how much effort — what matters is declaring the app safe and secure."*

**Now a STANDING process (Eisa, 2026-07-07):** *"Create an inspection team to audit every code build… conduct an inspection once every cycle."* Institutionalized as the saved workflow **`safety-inspection`** (`.claude/workflows/safety-inspection.js`) + the CLAUDE.md "The Safety Inspection" standing order — run **diff-scoped on every build**, **whole-app once per cycle** (cycle = each `/migration` close, or a session-close PCS). This Charter is the durable Findings Register for every run.

**Trigger:** the rename→index durability bug (MIG-098) — a silent, source-of-truth-corrupting failure that hid for ~9 days and surfaced only by accident during unrelated tests. That is the signature of the most dangerous defect class, and the mandate is to hunt the whole class across the app, not just this instance.

## Definition — an "app-killer"

A defect that **silently** damages the user's knowledge or the app's ability to serve it, WITHOUT surfacing an error the user or a test would notice. Recoverability is irrelevant; *silence* is the defining trait. Ranked above all else.

## The taxonomy (hunt targets)

1. **Silent data loss / durability gaps** *(the trigger class)* — a source-of-truth write (`.md` file, `note_meta`, FTS/sky/links index) performed via a fire-and-forget / best-effort / unawaited task that can be lost (uninitialized DB, lock contention, app-close) with **no retry and no error**. Markers: `spawn`/`spawn_blocking` whose handle is dropped; `let _ =` on a fallible DB/FS write; `tokio::spawn` mutating the source of truth.
2. **Content integrity** — a note's on-screen or on-disk content acquiring ANOTHER note's data, or losing its own (the BUG-012/015/019/023 / LL-014 three-strike class). Editor lifecycle, save-composition, `{#key}` teardown, cross-note transitions, second-screen writes.
3. **Error swallowing / false success** — a function that reports success (`Ok(())`, resolved promise) when it actually **skipped** the work (e.g. `reindex_single_note` returning `Ok(())` on a `None` connection); `.catch(() => {})` on a write; `unwrap_or` that hides a failure. These MASK class 1 & 2.
4. **Index ↔ disk divergence** — any derived surface (`note_meta`, `notes_fts`, `sky_links`, backlinks, tag counts, aliases, `review_schedule`, embeddings) that can silently drift from the `.md` source (Rule 8 write-time-derivation gaps + missing reconcile).
5. **Init / ordering races** — an operation running before the DB/state is ready (the conn-`None` case), or a boot sequence that lets writes escape before the index is live.
6. **Concurrency / lifecycle races** — `$effect` read/write loops, cross-window sync, concurrent writers on the same file/row (TOCTOU), write-gate escapes.
7. **Freeze / hang killers** — an unbounded lock wait on the awaited IPC path (the §B2-4 freeze class), `invoke()` on the keystroke hot path.
8. **Resource leaks** — unclosed `listen`/timer/`EditorView`/`addEventListener` (the slow-death memory class).

Priority: **1 → 3 → 2 → 4** (the silent + hiding classes first), then 5–8.

## Method (no reinvention — WA#5)

1. **Anti-pattern sweep** — mechanically grep the syntactic markers of each class → candidate-site register (fast, mechanical, high-recall).
2. **Multi-agent semantic audit** — agents read each candidate + its data-flow and construct a CONCRETE failure scenario (inputs → silent damage). Fan-out by (subsystem × class).
3. **Adversarial verification** — every candidate independently attacked by a skeptic prompted to REFUTE it; only findings with a concrete, defensible repro survive → **Confirmed Register**. (No crying wolf.)
4. **Reproduce-First** — a confirmed app-killer is reproduced on the running app / via a test BEFORE any fix.
5. **Prior-art fixes (WA#5)** — fixes follow proven patterns (transactional outbox / WAL / translog / disk-reconcile for durability; single-ownership for content-integrity), cross-checked against mature systems. No inventive fixes where a battle-tested one exists.
6. **Verify each fix** — every fix proven against its repro (red→green) before the next.

## Phases

- **P0 — Charter + state-of-standing** (this doc + the session log snapshot). *DONE.*
- **P1 — Recon + anti-pattern sweep** — map every source-of-truth write path; grep the class-1/3 markers; build the candidate register.
- **P2 — Wave hunts** (fan-out + adversarial verify), in priority order:
  - Wave 1: durability / silent-loss / false-success (classes 1, 3) — the trigger family.
  - Wave 2: content-integrity + index↔disk divergence (classes 2, 4).
  - Wave 3: init/ordering + concurrency/lifecycle (classes 5, 6).
  - Wave 4: freeze + leaks (classes 7, 8).
- **P3 — Confirmed Register + severity ranking** (app-killer > integrity > freeze/leak).
- **P4 — Remediation** — fix confirmed app-killers with proven patterns, each Reproduce-First + verified. The MIG-098 rename-durability fix is remediation item #1.
- **P5 — Safe & Secure declaration** — a signed-off register with every app-killer confirmed-fixed + verified; regression tests where feasible.

## Findings Register

*(each entry: id · severity · class · file:line · one-line · fix status. All adversarially verified — CONFIRMED.)*

### Wave 1 — silent data-loss / durability / false-success (`wf_c4054ac3`, 31 agents, 12 confirmed)

Search-index-maintenance + boot/init-ordering came back **CLEAN** (no confirmed silent-loss). Grouped by theme:

**Theme A — the FocusPane save path is broken (the primary fast-capture surface).** *HIGH.*
- **W1-1** HIGH false-success — `+layout.svelte:7709` — Focus save writes the body to disk but **never reindexes** → note_meta/FTS/backlinks/word_count silently diverge for every focus edit until a later NotePane edit or the next boot reconcile. (NoteEditor.handleSave pairs every write with a reindex; the focus path is the sole outlier.) *OPEN.*
- **W1-2** HIGH durability-gap — `+layout.svelte:7709` — Focus final write is **fire-and-forget, `markNoteSaved()` runs BEFORE the write, no write-ahead-buffer net**, error swallowed → a transient write rejection silently loses just-captured content with no error, no retry, no recovery (NoteEditor.handleFlush has the WAB net; focus bypasses it). *OPEN.*
- **W1-5** MED swallowed-write-error — `+layout.svelte:7709` — same path, the `.catch(()=>{})` facet. *OPEN (folds into W1-2 fix).*

**Theme B — folder rename/delete don't cascade to the index.** *MED.*
- **W1-3** MED false-success — `libraries.rs:761` — folder **rename** returns Ok with NO descendant reindex → every note under a renamed folder keeps its old path in note_meta/FTS/note_links/sky/review_schedule (move_item cascades; rename doesn't). Reconcile relocates by cid on next boot UNLESS the count exceeds the cap → then permanent. *OPEN.*
- **W1-4** MED false-success — `libraries.rs:5475` — folder **delete** calls `reindex_delete_note` with only the FOLDER path (exact `WHERE path=?1` match) → every note row inside a deleted folder is orphaned in the index. *OPEN.*

**Theme C — second-screen writes don't broadcast.** *MED.*
- **W1-6** MED durability-gap — `SecondScreenPage.svelte:1552` — SS editor-panels Tasks toggle writes the note on disk but never `broadcastNoteSaved` → main's stale in-memory model can clobber/diverge. *OPEN.*
- **W1-7** MED durability-gap — `SecondScreenPage.svelte:1419` — same defect in the SS split-view Tasks panel. *OPEN.*

**Theme D — non-atomic / swallowed persisted-state writes.** *MED/LOW.*
- **W1-8** MED durability-gap — `libraries.rs:141` — `save_libraries` uses plain truncate-then-write `fs::write` (no temp+atomic-rename+fsync); a crash mid-write truncates `libraries.json`, and the load path swallows the parse error → **silently returns an empty library list** (all registrations gone until manual recovery). *OPEN (highest blast radius in Theme D).*
- **W1-9** MED swallowed-write-error — `universe.rs:1417` — legacy collections adoption swallows BOTH the `collections.json` write AND the `workbench.json` rename (`let _ =`), then returns as if persisted. *OPEN.*
- **W1-10** MED swallowed-write-error — `classifier/mod.rs:404,407` — `cece_resolve_disambiguation` discards the co-write of the other axis (`let _ =`) yet returns Ok(None) + removes the card from the queue. *OPEN.*
- **W1-11** MED false-success — `NoteEditor.svelte:207` — stage-promote reports success (badge updates) but the disk write is fire-and-forget + `markSaved()` before the write → a failed stage write silently diverges disk from UI. *OPEN.*
- **W1-12** LOW swallowed-write-error — `propertyTypeRegistry.ts:44` — `property-types.json` debounced fire-and-forget with swallowed error while the in-memory cache updates synchronously → a failed persist silently drops the user's property-type assignment. *OPEN.*

### Wave 2 — content-integrity + index↔disk divergence (`wf_4c7d9c3a`, 27 agents, 15 confirmed)

**Theme E — the save/model-ownership path leaks unsaved edits (MIG-076 §C has holes).** *HIGH.*
- **W2-1 / W2-3** HIGH content-loss — `store.ts:1669` (+ `loadTabHistoryEntry:915`) — clicking a **[[wikilink]]** or Alt-navigating within the 1.5 s autosave window replaces the outgoing note's model via `openNoteModel` WITHOUT flushing its dirty body; the `{#key}` teardown flush then bails on the path-identity guard (`filePath !== tab.path`) → the source note's last ≤1.5 s of typing is silently lost (memory AND disk). **Common path.** *OPEN.*
- **W2-2** HIGH cross-window-clobber — `store.ts:1678` — `openNoteTab` never dedupes by path → the SAME note can back TWO tabs with two independent models (per-tab, not per-note — a direct MIG-076 violation); each save composes only its own model → one tab's save silently overwrites the other's committed edits (no conflict dialog). *OPEN.*
- **W2-10** MED content-loss — `store.ts:761` (+ NoteEditor handleSave/handleFlush) — `markNoteSaved` runs BEFORE `writeNote` resolves → a swallowed write failure leaves the model falsely clean → edit never re-persisted, later `adoptDisk` clobbers it. *OPEN.*

**Theme F — cross-window (second-screen) integrity is largely unprotected.** *HIGH.*
- **W2-4** HIGH — `SecondScreenPage.svelte:1768` — the rename/cascade freeze (`cascadingPaths`/`cascadeFreeze`) is **module-local to the window running the cascade**, so a note open+editable on the second screen is never frozen/reload-adopted during a MAIN-window rename cascade → its stale model save-stomps the cascade's wikilink rewrite. *OPEN.*
- **W2-5 / W2-6 / W2-11 / W2-12** HIGH/MED — `+layout.svelte:3231/3237/3242`, `SecondScreenPage.svelte:722` — `screen:note-saved` adopts the disk write into the MODEL but never remounts/re-seeds the live editor (main OR the SS editable companions) → display keeps the pre-edit body → the next keystroke silently clobbers the other window's save; and main overwrites `tab.content` even when the local model is DIRTY. *OPEN.*
- **W1-6 / W1-7** (Wave 1) belong to this theme too (SS task-toggle no-broadcast).

**Theme G — the frontmatter parser is lossy.** *HIGH.*
- **W2-7** HIGH content-loss — `store.ts:1179` — `parseFrontmatter` cannot represent YAML **block scalars / plain nested maps**, and `reconstructFrontmatter` rebuilds the WHOLE frontmatter from the lossy parse on EVERY save → those keys are silently dropped from disk on the first property/tag/stage edit — or even a plain body edit — of any note that has them. *OPEN.*
- **W2-8** HIGH content-corruption — `store.ts:1196` — asymmetric quote handling (`parse` strips quotes without unescaping; `reconstruct` re-escapes `"`→`\"`) → a value with a double-quote gets a stray backslash injected on every save, corrupting progressively. *OPEN.*

**Theme H — index / derived-data divergence (beyond Wave-1's folder cascade).** *HIGH/MED.*
- **W2-9** HIGH index-divergence — `search.rs:8814` — `reconcile_filesystem` walks the RECURSIVE (federated) library set and indexes foreign cUniverse notes INTO the active universe's `search.db`, a SECOND copy alongside the MIG-056 read-only ATTACH → the two drift (the foreign note is only updated in its own universe's DB). *OPEN — needs a scoping decision.*
- **W2-14** MED index-divergence — `search.rs:1365` — the save-path incoming diff keys on target NAMES only → retyping a link (same target, different link_type; or cognitive↔structural) never recomputes B's `incoming_link_types(_json)` / `incoming_top_rank` (/ `incoming_count`) → stale until the next boot reconcile. *OPEN.*

**Theme I — the write-gate's lost-update protection is inert.** *MED.*
- **W2-13** MED — `write_gate.rs:420` — staleness (`WouldRefuseStale`) is only computed inside `check_expectation`, which the live editor save path never supplies → `gate_write` takes the self-attestation branch (identity-only) and returns `SelfAttestedOk` even when the disk copy is NEWER → the lost-update class the gate names is never actually guarded on the real save path, even after the enforce flip. *OPEN.*

**Theme A (cont.)** — **W2-15** LOW content-loss — `+layout.svelte:7689` — FocusPane mounts with no `ontitlechange` → a title edited in Focus is discarded. *OPEN (folds into the FocusPane fix).*

### Wave 3 — concurrency / freeze / leaks (`wf_858afde9`, 17 agents, 3 confirmed)

**Reactivity-loops CLEAN · concurrency-TOCTOU CLEAN** (the Rule-2 discipline + the seq-token/single-flight/renamesInFlight guards held).

**Theme J — sync commands freeze the UI (dispatch-thread blocking).** *HIGH/MED.*
- **W3-1** HIGH freeze-hang — `libraries.rs:3263` — `scan_note_stages` is a **sync** `#[tauri::command]` that walks the whole library reading every `.md`'s frontmatter on the IPC dispatch thread → the UI freezes for seconds on library-expand (7,600 notes). Every other heavy command is `(async)`; this one wasn't. *OPEN (add `(async)`, matching `get_360_view`).*
- **W3-2** MED freeze-hang — `lens/query.rs:73` — `execute_lens` is a **sync** command running a federated `note_meta` query on the dispatch thread, fired from a CM6 live-preview widget's `toDOM()` → freezes on every lens-block edit. *OPEN (one-word fix: `(async)`).*

**Theme K — listener leak.** *MED.*
- **W3-3** MED resource-leak — `SourceReviewPanel.svelte:1024` — `classifier:scan` / `sources:bulk_accept` listeners are registered AFTER an awaited taxonomy load; unmounting during the load leaves `onDestroy` cleaning nothing → the listeners leak for the session (Rule 4 violation). *OPEN (destroyed-flag guard).*

---

## FIND PHASE COMPLETE — 30 confirmed silent-failure defects (Waves 1–3)

Consolidated into **root-cause fix-groups** (remediation units):

| Group | Sev | Root cause | Findings | Fix shape |
|---|---|---|---|---|
| **G1** FocusPane save | 🔴 HIGH | fast-capture save skips reindex + WAB + write-then-mark + title | W1-1,2,5 · W2-15 | give Focus NoteEditor's save discipline |
| **G2** save/model ownership | 🔴 HIGH | nav discards unsaved edits; 2-tabs-1-note clobber; mark-before-write | W2-1,2,3,10 · W1-11 | flush-before-nav + one-model-per-note + write-then-mark |
| **G3** cross-window integrity | 🔴 HIGH | window-local freeze; no reconcile/remount/broadcast | W2-4,5,6,11,12 · W1-6,7 | cross-window freeze + reconcile-remount protocol |
| **G4** frontmatter parser | 🔴 HIGH | block-scalars/nested-maps dropped; quote corruption — on EVERY save | W2-7,8 | proven YAML round-trip lib (WA#5) |
| **G5** index cascade + durability | 🟠 MED | folder rename/delete no-cascade; link-retype no-recompute; federated double-index; **MIG-098 rename durability** | W1-3,4 · W2-9,14 · MIG-098 | durable-intent/reconcile + cascade |
| **G6** persisted-state writes | 🟠 MED | non-atomic `libraries.json` (→ empty list); swallowed state writes | W1-8,9,10,12 | atomic temp+rename+fsync + honest errors |
| **G7** write-gate staleness | 🟠 MED | lost-update guard never fires on the real save path | W2-13 | wire staleness into the editor save |
| **G8** sync-command freeze | 🟠 HIGH/MED | dispatch-thread blocking | W3-1,2 | `(async)` |
| **G9** listener leak | 🟠 MED | post-await unguarded listen | W3-3 | destroyed-flag guard |

**Remediation order (worst-silent-knowledge-loss first):** G4 → G2 → G1 → G3 → G5 → G6 → G7 → G8 → G9. Deep groups (G2/G3/G4, and G5's durable fix) each rate a `/migration`; G6/G8/G9 are direct fixes. P4 remediation begins now.

---

## P4 — Remediation progress

**Quick-wins batch (direct fixes) — LANDED:**
- **G8 ✅** (`90ba3829`) — `scan_note_stages` + `execute_lens` → `(async)` (off the dispatch thread; no more full-library/lens-query UI freeze).
- **G9 ✅** (`90ba3829`) — SourceReviewPanel listeners guarded with a `destroyed` flag (no post-await leak).
- **G6 ✅** — atomic writes for the persisted-state class: `libraries.json` (`90ba3829`) + a shared `universe::atomic_write` applied to `save_registry` / `save_universe_settings` / `_workspaces` / `_collections` / `_property_types`; corrupt `libraries.json` preserved as a backup; the legacy-collections adoption made atomic; **swallowed writes surfaced** (W1-9 collections adoption, W1-10 cece co-write, W1-12 property-types persist). *(Remaining in-class: `universe.json` saves + the federation `let _ = fs::write` sites — REGISTERED below.)*
- **G1 ✅** — FocusPane save rewritten to NoteEditor's discipline: **debounced** (1500 ms) write + reindex (was per-keystroke — a regression the per-build inspection caught), **write-ahead recovery net**, **`onflush`** for an immediate persist on exit/destroy, error surfaced. Fixes W1-1 / W1-2 / W1-5. *(Boss test pending: focus-capture round-trip — type in Focus, exit, confirm the note is findable in search + survives a forced-quit.)*

**Per-build inspection (first live run, `wf_012f1593`, 49 agents, whole-app fallback) — new findings added to the register:**
- **NEW → G5/index:** `libraries.rs:1614` move_item drops the moved note's `review_schedule` row (MED) · `search.rs:7991` `constellation_link_archive` mutates `note_links.status` but never recomputes the target's incoming aggregates (MED) · `review.rs:1056` `sync_action_to_row` silently drops a ✓Reviewed/snooze/dismiss into `review_schedule` (MED).
- **NEW → G6 (registered, not yet fixed):** `universe.rs` `universe.json` saves (1189/1205/etc.) + the federation `let _ = fs::write` sites are still plain `fs::write` — apply `atomic_write` in the G6 follow-up.
- **Re-confirmed (existing groups):** NoteEditor `handleSave`/`saveTabContent` markSaved-before-write (G2) · store.ts nav-loss + two-tabs-one-note (G2) · parseFrontmatter/reconstruct loss+corruption (G4) · SecondScreenPage adopt-without-remount (G3).

**Deep `/migration` fixes remaining (worst-first):** **G4** frontmatter (real YAML round-trip) → **G2** save/model-ownership (flush-before-nav + one-model-per-note + write-then-mark) → **G3** cross-window → **G5** durable rename + the index cascades → **G7** write-gate staleness. Each its own `/migration` with an Audit-phase inspection.

---

## Per-cycle whole-app sweep — 2026-07-07 (MIG-099 close) — `wf_a7d5e452-16d`

44-agent adversarial hunt→verify, **25 confirmed** (7 HIGH · 10 MED · 8 LOW). Run at the MIG-099 (create-latency) close as the cycle-boundary sweep. **MIG-099's own diff is CLEAN — zero confirmed findings in the changed resolver/create/trash code.** The 25 are the standing backlog; most re-confirm G2/G3/G4, several are NEW.

| # | Sev | Site | Class | Group | Note |
|---|-----|------|-------|-------|------|
| 1 | HIGH | `store.ts:1669` | silent-data-loss | **G2** | in-place tab-reuse nav (`openNoteModel` unconditional `models.set`) discards the outgoing note's dirty model; teardown flush bails on path-mismatch → sub-1500 ms edit lost, no error/crumb. (re-confirm: nav-loss) |
| 2 | HIGH | `NoteEditor.svelte:238` | swallowed-write | **G2** | `handleSave` autosave `.catch(()=>{})` with NO write-ahead (unlike `handleFlush`); dirty already false → no retry → transient write-fail silently loses the edit. |
| 3 | HIGH | `SecondScreenPage.svelte:1069` | cross-window-clobber | **G3** | rename cascade freeze/`markCascading` is per-window; SS editable NoteEditor keeps `[[Foo]]`, its `isCascading` guard is inert → SS autosave overwrites the walker's `[[Foo v2]]` rewrite; disk↔index diverge. **NEW.** |
| 4 | HIGH | `SecondScreenPage.svelte:722` | cross-window-clobber | **G3** | SS `screen:note-saved` refreshes only read-only panels, never its editable NoteEditors → stale companion editor clobbers a main-window save. **NEW.** |
| 5 | HIGH | `store.ts:1096` | content-loss | **G4** | `parseFrontmatter` drops nested-map / block-scalar (`\|`/`>`) keys; next save persists the note WITHOUT them — silent frontmatter loss. (re-confirm) |
| 6 | HIGH | `review.rs:762` | silent-data-loss | **G6** | `review-pulse.json` (all review history: last_reviewed/intervals/snooze/dismiss) saved via plain non-atomic `fs::write` → crash mid-write loses/corrupts the whole review history. **NEW.** |
| 7 | HIGH | `store.ts:761` | false-success | **G2** | `saveTabContent` calls `markNoteSaved` BEFORE `await writeNote` → model clean before disk write; a failed write leaves a false-saved model. (re-confirm) |
| 8 | MED | `libraries.rs:840` | index-divergence | **G5** | folder rename = bare `fs::rename`, NO index cascade for descendant `.md` → every descendant's `note_meta`/`note_links`/`note_aliases` row points at the old path. (known: folder-cascade) |
| 9 | MED | `libraries.rs:5908` | index-divergence | **G5** | `delete_path` DirAll removes the tree but `reindex_delete_note`s only the folder path → descendant notes keep index rows at destroyed paths. **NEW.** |
| 10 | MED | `search.rs:9103` | index-divergence | **G5** | `reindex_single_note` returns `Ok(())` when conn is None → `constellation_search_reindex` reports success while `note_meta`/`notes_fts` never update. **NEW** (weakens MIG-099 §3's freshness in the DB-not-ready edge). |
| 11 | MED | `store.ts:761` | swallowed-write | **G2** | `saveTabContent` (PropertyEditor/typed-link connect) markSaved-before-write, no write-ahead. (re-confirm) |
| 12 | MED | `NoteEditor.svelte:233` | false-success | **G2** | `handleSave` `markSaved` synchronously before `writeNote` resolves → `isDirty()` false even on write failure. (re-confirm) |
| 13 | MED | `search.rs:7991` | index-divergence | **G5** | `constellation_link_archive` flips `note_links.status` but never recomputes the TARGET's incoming aggregates. (known) |
| 14 | MED | `search.rs:8023` | index-divergence | **G5** | `constellation_link_unarchive` — same target-recompute gap as archive. **NEW** (unarchive counterpart). |
| 15 | MED | `store.ts:1290` | content-corruption | **G4** | `reconstructFrontmatter` escapes `"`→`\"` but `parseFrontmatter` only strips outer quotes → embedded-quote asymmetry corrupts on round-trip. (re-confirm) |
| 16 | MED | `universe.rs:795` | swallowed-write | **G6** | the "fix stale library paths on activation" path writes `libraries.json` via non-atomic `let _ = fs::write` — **bypasses the G6 atomic_write remediation**. **NEW** (G6 gap). |
| 17 | MED | `ReviewerView.svelte:286` | swallowed-write | **G6** | `commitPriority`/`resetPriority` `catch {}` swallow a failed `set_review_priority` write to the review_schedule source-of-truth. **NEW.** |
| 18 | LOW | `search.rs:8968` | index-divergence | **G5** | `reindex_delete_note` never deletes `note_aliases`/`note_embeddings` → orphan rows. (known; MIG-099 exists()-guards the read side) |
| 19 | LOW | `cece/reliability.rs:95` | concurrency-race | new | unlocked sync read → `sweep_tmp_orphans` can delete a concurrent writer's in-flight NamedTempFile. **NEW.** |
| 20 | LOW | `cece/reliability.rs:102` | content-loss | new | `load_or_default` returns empty on parse-fail; next `save()` overwrites with near-empty → silent reliability-data wipe. **NEW.** |
| 21 | LOW | `ReviewStatusPanel.svelte:97` | swallowed-write | **G6** | `commitPriority` `catch {}` swallows a failed review_schedule write. **NEW.** |
| 22 | LOW | `tasks.rs:532` | false-success | **G5** | `toggle_task` `let _ = reindex_single_note(...)` no `ensure_search_db_ready` → FTS drift (its own comment warns of this). **NEW.** |
| 23 | LOW | `watcher.rs:78` | toctou | new | `atomic_write` now `mark()`s on EVERY gated write; watcher suppression was cascade-only → a real external edit during the window can be suppressed. **NEW.** |
| 24 | LOW | `provenance.rs:100` | freeze-hang | **G8** | `compute_note_origins` sync command, full recursive walk+regex of every `.md` on the dispatch thread. **NEW.** |
| 25 | LOW | `libraries.rs:5148` | freeze-hang | **G8** | `collect_library_notes` sync command, recursive walk + `read_to_string` every canonical `.md` on the dispatch thread. **NEW.** |

**Triage:** the HIGH cluster (1,2,3,4,5,7) is the already-planned **G2 (save/model-ownership)** + **G3 (cross-window)** deep `/migration`s — this sweep sharpens their reproduction recipes. NEW additions to the register: **G6** gaps (#6 review-pulse.json, #16 libraries.json activation path, #17/#21 swallowed priority writes), **G5** (#9 folder-delete descendants, #10 reindex silent-skip, #14 unarchive recompute, #22 toggle_task), **G8** (#24/#25 sync-command walks), and standalone LOW (#19/#20 cece reliability, #23 watcher TOCTOU). None block MIG-099. Remediation continues on the G-plan order (G4 next, per Boss).

## Per-cycle whole-app sweep — 2026-07-08 (Watcher-Index-Freshness migration close) — `wf_8a41970f-36d`

43-agent adversarial hunt→verify, **22 confirmed** (2 APP-KILLER · 7 HIGH · 11 MED · 2 LOW). Run at the close of the **Watcher-Index-Freshness** `/migration` (runtime reindex of external `.md` changes). **The migration's own diff is CLEAN — ZERO confirmed findings in the new code** (`reindex_changed_paths`, `reindex_md_descendants`, `delete_rows_under_prefix`, `library_name_for_path`, the relaxed `watcher.rs` filter, the `scheduleWatcherFlush` wiring). The one finding *touching* the new command (`search.rs:9285` — `reindex_single_note` returns `Ok` when `state.db` is None) is a pre-existing behavior the new command inherited; **fixed this build** by adding `ensure_search_db_ready(&app)?` to `reindex_changed_paths`.

The other 21 are the **standing G2–G8 backlog, mostly re-confirms of the 2026-07-07 register** (nothing between the two sweeps touched those paths):
- **APP-KILLER** `NoteEditor.svelte:233` — `handleSave` marks the model saved (`markSaved`) BEFORE the awaited `writeNote`; the rejection is swallowed (`.catch(()=>{})`) with NO write-ahead net on the debounced path → a transient `.md` lock (Syncthing/OneDrive/Defender) silently loses the edit AND defeats the F2 cascade-staleness guard (`flushAllTabsInLibrary` skips the falsely-clean tab). Re-confirms 07-07 #2/#12. → **G2**.
- **APP-KILLER** `store.ts:1693` — same-tab navigation (wikilink click within the 1500 ms debounce) overwrites the outgoing note's model (`openNoteModel` unconditional) with no flush → up to ~30 s of just-typed text lost, no error. Second instance `store.ts:919` (Alt-nav). Re-confirms 07-07 #1. → **G2**.
- HIGH: `store.ts:765` prop-save markSaved-before-write (07-07 #7/#11) · `store.ts:1553` one note in two tabs, models clobber · `store.ts:1738` `closeTab` no dirty-flush · `+layout.svelte:6184` SS not cascade-gated (07-07 #3) · `SecondScreenPage.svelte:722` SS never adopts main saves (07-07 #4) · `NoteEditor.svelte:207` `handlePromote` markSaved-before-write + no reindex · `+layout.svelte:3146` watcher reload updates `tab.content` but not the `noteModel` (**= the open-note-external-edit path; the Boss-approved focus-reconcile follow-up addresses exactly this**).
- **NEW this sweep:** `yamlDoc.ts:150/254` — `serializeLine` has no nested-object-list branch → editing an ikhtilāf/nested-object-list property flattens its structured YAML to a quoted scalar on disk (a **G4 regression** — the round-trip engine G4 hardened has a remaining gap on this property shape). `link_types.rs:535` + `universe.rs:953` non-atomic JSON writes (**G6** gaps). `bulk_ops.rs:305` ungated read→write TOCTOU. `FocusPane.svelte:209` no beforeunload/visibilitychange net. `libraries.rs:1648` `move_item` resets review history (**G5** — same aux-loss class this migration documented for external folder rename).

**Triage:** the 2 APP-KILLERs (`NoteEditor.svelte:233`, `store.ts:1693`) are the highest-priority remediations — both **G2 (save/model-ownership)** silent content-loss, Reproduce-First. The `yamlDoc.ts` nested-object-list flatten is a fresh **G4** gap worth its own fix. None block the Watcher-Index-Freshness fix (its diff is clean). Surfaced to Eisa for sequencing.
