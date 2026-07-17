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

## Per-cycle whole-app sweep — 2026-07-08 (Save-Durability migration close) — `wf_5f9b257d-a99`

42-agent adversarial hunt→verify, **23 confirmed** (2 APP-KILLER · 5 HIGH · 14 MED · 2 LOW). Run at the close of the **Save-Durability** `/migration` (remediation of the `NoteEditor.svelte:233` APP-KILLER — mark-clean-before-durable-write). **The migration's own diff is CLEAN — ZERO NEW app-killers introduced by the 5-site reroute onto `noteSession.save()` + `standardSaveEnv`.** Every confirmed finding was cross-checked against the new code: the durability primitive, the reroutes, the save-health banner/retry, and the compare-and-clear net introduce no new silent-loss path (the harness proves the primitive red→green).

**Remediation #1 — the `NoteEditor.svelte:233` APP-KILLER is FIXED** (mark clean only on a durable write; net + surface on failure; the save-health banner). The sibling mark-clean-before-write sites (`handleFlush`, `handlePromote`, `saveTabContent`, `commitFocusSave`) are fixed in the same migration.

**WA#6 fix folded in this build:** `store.ts:824` — `saveTabContent`'s single-flight `saveLocks` guard silently dropped a concurrent property edit (it early-returned BEFORE `editNoteProps` reached the model). Same silent-save-loss class; fixed by pushing `editNoteProps` (the model update) BEFORE the guard, so a concurrent edit always lands in the model (dirty) and the next save/flush persists it. The guard now serializes only the WRITE.

The other 21 are the standing G2–G8 backlog (mostly re-confirms). **Highest-priority remaining = the 2nd APP-KILLER, still open:**
- **APP-KILLER (HIGH)** `store.ts:1787` — `openNoteTab` in-place tab reuse (`openNoteModel` unconditional replace) discards the outgoing note's dirty model with no flush; the `{#key}` teardown flush bails on the path-identity guard → up to ~30 s of just-typed text lost on the most common navigation (file-tree/wikilink click). Second instance `store.ts:1013` (Alt-nav). → **the next migration (notemodel-ownership: flush-before-replace).**
- **HIGH** `+layout.svelte:3320` — same note in two tabs = two independent models; a save from one is never reconciled into the sibling → a later stale-sibling save silently clobbers the first's on-disk edits. → same notemodel-ownership migration.
- HIGH: `SecondScreenPage.svelte:722/723` SS cross-window cascade-gate + save-reconcile (**G3**); `universe.rs:921` `rename_universe` non-atomic `libraries.json` (**G6**).
- MED (in the diff files, pre-existing): `store.ts:854` reindex-on-save fire-and-forget swallowed → index-drift until next edit/boot (self-heals; **not** data-loss — the `.md` is durable); `search.rs:9285` reindex Ok-on-None (general). NEW: `cece/orchestrator.rs:153` per-cataloger timeout defeated by `thread::scope` join; `livePreview.ts:242` `_imageCache` unbounded (leak). Plus the standing `yamlDoc.ts:150` (G4), Rust non-atomic JSON (`universe.rs:1210`, `review.rs:762`), `bulk_ops.rs:305` TOCTOU, `libraries.rs:858/5965/5212` folder-cascade/delete-descendants/sync-walk, `search.rs:8231/8286` link archive/unarchive incoming-recompute.

**Triage:** the **2nd APP-KILLER (`store.ts:1787` nav-loss + `+layout.svelte:3320` two-tab clobber)** is the next Reproduce-First migration (notemodel-ownership: flush the outgoing model before replacing it; reconcile same-note siblings). Surfaced to Eisa.

## Per-cycle whole-app sweep — 2026-07-08 (NoteModel-Ownership / APP-KILLER #2 migration close) — `wf_415a7214-4ad`

44-agent adversarial hunt→verify, **23 confirmed** (1 APP-KILLER · 3 HIGH · 13 MED · 6 LOW). Run at the close of the **NoteModel-Ownership nav-loss** `/migration` (flush-before-replace at `openNoteTab` reuse + `loadTabHistoryEntry`, `markSaved` path-guard, Focus-exit-on-nav, rename-on-locked-file durability, one-path-one-tab dedup). **The migration's own diff (§1–§7b) is CLEAN — ZERO NEW app-killers** introduced by `flushIfDirty`/`flushOutgoing`/`navFlushEnv`/the `markSaved` guard/the dedup (the harness proves the primitives red→green; the LL-023 dev tripwire in `openModel` future-proofs the class).

**Remediation this build (WA#6 — fixed BEFORE commit, in-class gaps the migration should have covered):**
- **APP-KILLER FIXED** `store.ts:1917` — `closeTab` disposed the model (`closeNoteModel`) before the editor unmount, so a type-then-close within the 1.5 s debounce lost the edit (the teardown flush bails — model already gone). `closeTab` is now `async` and flushes the dirty model first via `flushOutgoing` (best-effort, net-backed; no abort — the tab is being dismissed). Was `store.ts:1738` in the 07-08 Save-Durability register. **This is the third departure** (alongside the two nav sites the migration fixed) — the class is now complete.
- **HIGH FIXED** `FocusPane.svelte` — no `beforeunload`/`visibilitychange`/idle flush; continuous typing then a window close lost the run (`onDestroy` doesn't fire on unload). Now mirrors NotePane: flush on `beforeunload` + tab-hide + 30 s idle → the synchronous write-ahead net (recoverable on reopen). Closes the 07-08 register's `FocusPane.svelte:209`.

The other 21 are the **standing G2–G8 backlog, re-confirms of the 07-07/07-08 registers** (nothing between the sweeps touched those paths) plus a few sharpened:
- **HIGH (G3 cross-window, next migration):** `SecondScreenPage.svelte:1771` SS is blind to a main-window rename cascade (no `cascade:rewrote` listener, no `isCascading` gate) → an in-flight SS edit stomps the cascade's wikilink rewrite; `SecondScreenPage.svelte:723` SS `onNoteSaved` refreshes only the read-only companions, never adopts main→SS saves into its WRITABLE editor tabs → a later SS save clobbers the main window's committed edits. Both re-confirm 07-08 #3/#4.
- **MED backlog (G4/G5/G6/derived-index):** `yamlDoc.ts:150` nested-object-list flatten (G4); `search.rs:8242/8274` link archive/unarchive don't recompute the target's incoming aggregates/sky maturity (derived-index, **already on the G-backlog**); `libraries.rs:858` folder-rename no DB cascade (G5); `universe.rs:921/300` non-atomic swallowed `libraries.json`/`universe.json` writes + `:939` missing cache-invalidate (G6); `bulk_ops.rs:305` bulk-accept read→write TOCTOU lost-update; `BacklinksPanel.svelte:195` `linkMention` unguarded `\b(name)\b` replace can corrupt frontmatter; `+layout.svelte:4668` `template_insert` fallback writes disk but not the model; `+layout.svelte:3324` main `onNoteSaved` overwrites `tab.content` unconditionally (display/model desync); the reindex-fire-and-forget class (`NoteEditor.svelte:256`, `search.rs:9285`, `store.ts` save paths) → index-drift, self-heals (not data-loss).
- **LOW backlog:** `move_item` reindex `let _ =`; `flushAllTabsInLibrary` bare-flush (index-freshness — the `navFlushEnv` pattern could be applied here); `FocusPane` `ontitlechange` not wired (title typed in Focus dropped — W2-15); `review.rs:762` `save_pulse_data` non-atomic; `saveSettings`/`persistWorkspaces`/`saveCollections` fire-and-forget persists (config/layout/working-set, not knowledge).

**Triage:** the APP-KILLER + the FocusPane-unload HIGH were **in this migration's class and are FIXED this build** (WA#6). The 2 remaining HIGH are both **G3 second-screen cross-window** — a distinct surface needing its own `/migration` (my Architect explicitly scoped cross-realm OUT). The MED/LOW backlog is the standing G2–G8 set, several already Boss-sequenced. **None block this migration (its diff is clean).** Full register + sequencing recommendation surfaced to Eisa for a ruling (WA#6 — not silently parked).

---

## Per-cycle whole-app sweep — 2026-07-09 (G3 Second-Screen Cross-Window migration — build audit) — `wf_a19eb032-ab4`

44-agent adversarial hunt→verify, **22 confirmed** (6 HIGH · 9 MED · 7 LOW). Run over the G3 build (read-only-by-default SS + editable toggle + main→SS adopt + cascade-react + cross-window freeze). **G3 CLOSES the two prior-registered G3 HIGHs** (07-08 register: `SecondScreenPage:1771` cascade-blind + `:723` never-adopts-main→SS): the SS now (§2) adopts main→SS saves into every view, (§3) listens to `cascade:rewrote`, and (§4) honors a cross-window freeze — all freshness-gated so a dirty edit is never clobbered.

**The G3 diff itself introduces ZERO NEW app-killers.** Exactly ONE confirmed finding lands in the new G3 code, and it is the **documented, Boss-approved residual**:
- **MED (my diff — DOCUMENTED RESIDUAL)** `SecondScreenPage.svelte:877` — **two-sided-dirty cascade revert.** When `secondScreenEditable=ON` (non-default) AND the user is *simultaneously* mid-edit on the *exact* note a main-window rename cascade rewrites, the SS's freshness-gated adopt (§3) correctly REFUSES to clobber the dirty SS buffer, but after the §4 freeze lifts the SS's autosave writes its pre-cascade buffer, silently reverting the cascade's `[[wikilink]]` rewrite (disk↔note_links diverge; the user's SS body edit is preserved — a narrow link-integrity divergence, not bulk loss). This is the **two-sided conflict the plan's "Residual" section + Architect §4 explicitly scoped OUT** (the never-built §E conflict dialog). **The read-only DEFAULT avoids it entirely**; G3 is a strict improvement over the pre-G3 SS (which had no freeze/adopt and stomped every cascade). Per Solve-the-Class, the real fix is the conflict-resolution end-state, not a symptom-patch that trades one silent loss (link revert) for another (SS edit force-discarded). **Surfaced to Eisa for a ruling** (keep as residual / cheap cascade-wins hardening / full conflict `/migration`).

**NEW / sharpened pre-existing findings (all OUTSIDE the G3 diff — Charter register for the G-plan; none block G3):**
- **HIGH (main-window external-change adopt — the SIBLING of G3, main-window side):** `+layout.svelte:3171` the file-watcher reload of an OPEN note updates only `tab.content` — it never `adoptDisk`s into the noteModel nor bumps `reloadVersion`, so a git-pull/Syncthing edit to an open note is invisible in the editor and silently overwritten by the next keystroke (the EXACT class G3 just fixed on the SS side — the main window has the same gap). `+layout.svelte:3329` the main `onNoteSaved` (SS→main) adopts into the model but never bumps `reloadVersion`, so the mounted CM6 buffer stays stale and the next keystroke overwrites the adopted content (G3's SS `adoptFreshDiskIntoSS` DOES bump reloadVersion — the main window should adopt the same fix). **Strong candidate for a "main-window external-change adopt" follow-up (G3's mirror).**
- **HIGH (rename-cascade net-wipe):** `store.ts:659` `reloadTabsFromDisk` force-replaces an open note's model AND wipes its write-ahead recovery net; if that note's pre-cascade flush write FAILED (lock/AV), its unsaved edits are silently destroyed by the reload.
- **HIGH (move/rename DB cascade):** `libraries.rs:1676` `move_item` uses delete+reindex instead of the `rename_item_db_tail` path-migration → a moved note silently loses its `review_schedule` history + orphans `note_aliases`/`note_embeddings` at the dead old path (review spacing resets, note drops out of semantic search). (Sharpens the prior LOW `move_item reindex let _`.)
- **HIGH (boot/persisted-json):** `universe.rs:356` `ensure_universe_notes_folder` swallows a `libraries.json` parse error → empty vec → re-inserts a default library (potential mis-registration on a corrupt manifest).
- **MED:** `NotePane.svelte:293` `doSave` clears local `dirty` BEFORE `handleSave` runs; if `handleSave` early-returns on its in-flight `saving` guard, that burst is neither written nor netted and no timer re-drives it → a crash in that window loses it (crash-recovery gap, notemodel-ownership). · `search.rs:1447/1489` a link RE-TYPE (same target, changed type) leaves the target's incoming-type breakdown + the source's sky stratum stale (derived-index). · `+layout.svelte:5930` `addTagToNote` closed-note branch has no `isCascading` gate (its twin `addLinkToNote` was given one). · `universe.rs:1210` `universe.json` writes are non-atomic `fs::write` (G6). · `SenseMakingCanvas.svelte:147` canvas save swallows errors (no dirty/retry). · `libraries.rs:5211` `collect_library_notes` is a SYNC command that walks + reads the whole library (freeze risk on a large library).
- **LOW:** `store.ts:843` `flushAllTabsInLibrary` bare env → flushed non-cascade notes not reindexed (index-freshness). · `FocusPane.svelte:88` `ontitlechange` still not wired (title typed in Focus dropped — long-standing W2-15). · `+layout.svelte:5939` closed-note tag write surfaces only `console.error` (invisible in release). · `review.rs:762` `save_pulse_data` non-atomic. · `store.ts:1104` `saveCollections` + `store.ts:5041` `persistWorkspaces` fire-and-forget persists (config/layout, not knowledge). · `libraries.rs:763` `create_note` MIG-099 §3 reindex silently no-ops when `state.db` is None.

**Triage:** the two prior-registered **G3 HIGHs are FIXED by this migration** (its purpose). The G3 diff is CLEAN of new app-killers; its ONE in-diff finding is the **Boss-approved two-sided-dirty residual** (read-only default avoids it). The strongest NEW items are the **main-window external-change adopt gaps** (`+layout:3171`/`:3329`) — literally G3's mirror on the main window, a natural G3-follow-up — plus the `reloadTabsFromDisk` net-wipe and the `move_item` DB-cascade HIGHs. **None block G3 (all pre-existing, all outside its diff).** Full register + sequencing surfaced to Eisa for a ruling (WA#6 — not silently parked).

## Per-cycle whole-app sweep — 2026-07-11 (MIG-100 Auto-Restore-Tabs migration) — `wf_127a517c-479`

**87-agent adversarial hunt→verify, 63 confirmed.** Run mid-build as the cycle-boundary sweep. **MIG-100's own diff is CLEAN — zero confirmed findings in the new auto-restore code** (verified again by the 3 diff-scoped inspections below + the Phase-4 close audit). Two of the 63 are pre-existing **APP-KILLERs** promoted for the register:
- **APP-KILLER — `+layout.svelte:3172` watcher external-change never adopts into the note model.** An external `.md` edit (Obsidian sync / `git pull`) updates only `tab.content`, never the single-ownership model or `reloadVersion`; the mounted editor + model keep the stale body, and the user's next keystroke's debounced save durably overwrites the external edit (then reindexes, so search agrees with the stomp). The `adoptDisk` clean-model-adopt gate exists but this path never calls it (the SS path `SecondScreenPage.adoptFreshDiskIntoSS` does it correctly). **The main-window mirror of the G3 external-change class.** → G-plan follow-up.
- **APP-KILLER — `sources/bulk_ops.rs:305` bulk Accept-All unlocked read→modify→write.** `accept_one` does an unlocked `fs::read_to_string` then `gate_write`, the exact race `gate_rmw` exists to prevent (per-card accept WAS migrated to `gate_rmw`; bulk was not) — a concurrent editor save landing in the window is silently overwritten. → G-plan (source-review write path).

The remaining 61 re-confirm the standing **G2–G8** backlog. Full register in the workflow journal; surfaced to Eisa (WA#6 — not parked).

## MIG-100 diff-scoped inspections + Phase-4 close audit — 2026-07-11/12

The auto-restore migration's own changes were inspected at every commit and audited whole at close. **Every in-diff finding was FIXED before its commit (WA#6) — none parked.**
- **Diff inspection 1** (`wf_f5d6b0f3-6c6`, §2–§7 code) — 7 confirmed, all fixed: **APP-KILLER** the restore consumed the write-ahead crash-recovery net (restored model born clean → recovered content unreachable) → `resolveNoteContent(preserveNet)`; MED deferred-cid `reloadTabsFromDisk` discards mid-drain keystrokes → dirty-guarded adopt; MED the 5 persisted-JSON saves + fsync were sync on the dispatch thread → `(async)`; 3× LOW swallowed toggle-off delete + universe-switch model leak → surfaced/`flushDisposeClearTabs`.
- **Diff inspection 2** (`wf_0394637a-1a9`, Stage-2 hotfix) — 2 confirmed, fixed: the wab REJECT branch also honors `preserveNet`; transiently-unreadable tabs get one-boot `carried` grace (2nd strike drops) instead of silent pruning.
- **Diff inspection 3** (`wf_24ee6467-b2b`, write-auth widening) — **ZERO in the widening** (read-only, security bound intact). Surfaced **4 pre-existing MED** — the G6 class (non-atomic `fs::write` on live `universe.json`/`libraries.json` → crash-truncate → silent library/federation loss): `ensure_universe_notes_folder`, `set_active_universe` heal, `rename_universe`, `add`/`remove_child_universe`. **All FIXED this cycle** (`persist_json_best_effort` + `atomic_write`) — closes a chunk of the standing G6 backlog.
- **Phase-4 close audit** (`wf_c2960e95-207`, 3 agents over the full net diff) — **all 7 invariants HOLD; all 6 migration-path scenarios PASS** (no app-killer, no silent `.md` corruption; the write-auth widening confirmed NOT to re-open cross-universe contamination — `write_note` writes `content→file_path`, never redirects; federated cUniverses aren't registry-registered so unreachable). Drift check found **3 lifecycle leaks in the deferred-cid machinery** (LL-023 class): the pending-set + activation watchers survived a universe switch, a closed pending tab wedged the auto-unsubscribe, and `startSessionTracking` didn't cancel a stray deferred-arm. **All 3 FIXED** (`disposeDeferredCidEnsure`/`dropPendingCidEnsure` wired into `flushDisposeClearTabs` + `closeTab`; deferred-arm cancel in `startSessionTracking`) with regressions R15/R16. One LOW documented (future-version session downgrade overwrite — harmless while schema fixed at 1).

## OPEN INVESTIGATION (2026-07-12, surfaced during MIG-100 Boss testing) — universes booting from an unfound registry

During the MIG-100 Stage-2 forensics, the on-disk registry `%APPDATA%/world.uconstellation.app/universes.json` was found to contain ONLY `كون عيسى`, yet the Boss's SAME-exe instance boots **"Eisa Cognitive Knowledge"** (7,751 notes) and created/switched to **"Scratch"** — neither name appears in ANY registry file findable on the machine, while the shared `write-journal.jsonl` records both universes' activity. **Where the Boss's instance persists its universe registry is UNKNOWN.** Hypotheses: a silent registry-save failure (now partly mitigated by the atomic-write fixes above), a divergent app-data dir resolution, or a second app-data root. **Needs a diagnostic build** (log the resolved `app_data_dir` + registry path at boot). Filed for a Boss ruling; does NOT block MIG-100 (the feature works correctly against whatever registry the running instance uses).

**UPDATE (2026-07-12, during the PJ-070 Boss test) — concrete lead.** The running instance's **"Eisa Cognitive Knowledge"** universe resolves to the on-disk root **`E:\Cognitive Knowledge\`** — the write-journal recorded the Boss's test note (`PJ-070 test`, folder `Eisa Test`) with `create_note` + `editor_save` + `conflict_sidecar` all under `E:\Cognitive Knowledge\Eisa Test\`. This is a DIFFERENT path than the `E:\Constellation Universes\Eisa Cognitive Knowledge\` folder that also exists on disk (and is what the earlier registry search scanned). So WHERE the active universe's data lives is now known; the diagnostic build is still wanted to explain WHERE the display-name→`E:\Cognitive Knowledge` mapping is persisted (not in the findable `universes.json`). Tracked in Pending Jobs v1.19 as PJ-072.

## Findings Register — PJ-070 close cycle (2026-07-12)

**Per-cycle whole-app sweep** (`wf_1b7addb3-822`, 38 agents, mode whole-app, **15 confirmed** — 4 HIGH / 8 MED / 3 LOW; every candidate adversarially refuted before confirming). Run at the PJ-070 `/migration` close boundary.

- **In the PJ-070 diff: 1 finding — FIXED pre-commit.** `NoteEditor.handleFlush` pushed the editor text into the model on EVERY teardown; `editBody(string)` always allocates a fresh `Text` (never ref-equal), so a merely-VIEWED note spuriously went `isDirty` while its content equalled disk — which **silently defeated `adoptDisk`** on background/focus tabs (reintroducing the PJ-070 clobber there) and raised **phantom `.conflict` sidecars**, and made `flushAllDirtyTabs` re-write untouched notes on a universe switch. Fixed at the source: `setBody`'s STRING path no-ops an identical-content push (keystroke `Text` path stays O(1)). *(class 2 / 4.)*
- **Pre-existing HIGH — filed as PJs:** `switchTab` (`store.ts:2412`) never flushes the outgoing dirty model → last ≤1.5 s of edits on a non-active tab lost on quit (APP-KILLER #2 class, the one unwired departure) → **PJ-086**; `composeFrontmatter` H1 error-passthrough (`yamlDoc.ts:213`) silently discards ALL property edits on lenient-but-eemeli-invalid frontmatter → **PJ-085**; `universe.rs::atomic_write` fixed shared tmp filename race → **PJ-087**. (`review-pulse.json` non-atomic = already-tracked **PJ-075**.)
- **Pre-existing MED/LOW — mapped or batched:** folder delete/rename descendant-cascade + link archive/unarchive incoming-aggregate recompute → **PJ-074**; `flushAllTabsInLibrary` reindex gap, `saveCollections` un-awaited, `parseFrontmatter` comma-split, `sources` prefix-strip, `BacklinksPanel.linkMention` swallow, `perf_trace` unbounded Vec → the Group-4/hygiene batch. None block PJ-070.

**Diff-scoped inspections** (per-build, on the changed files) folded their findings into the build before commit; the PJ-070 diff's own only finding was the spurious-dirty class above.

## Findings Register — PJ-088 close cycle (2026-07-12)

**Per-build whole-app sweep** (`wf_c0dac305-85e`, 40 agents, **19 confirmed**) at the PJ-088 (conflict-merge view) build boundary.

- **In the PJ-088 diff: 1 finding — FIXED pre-commit.** `resolveConflictMerge` pushed the merge into the model via `editNoteProps`/`editBody` but left `m.base` at its open-time bytes → `composeFrontmatter` diffed the merge against the stale base (a UNIFICATION-invariant violation), so non-projectable frontmatter (nested maps / block scalars) the merge ADDED/REMOVED was silently dropped from the saved note. Fixed at the model layer: new `replaceContent(id, content)` RE-BASES (`baseOf` the merged source) so compose emits the merged frontmatter verbatim. Reproduce-First: `tests/mig-076/runtimeHarness.test.ts` **Recipe P** (a merge removing a nested-map key sticks). *(class 2 — content-integrity.)*
- **Pre-existing HIGH — filed as PJs:** the Index-panel preview mounts a WRITABLE `index_preview_*` NoteEditor NOT deduped against the open store tabs → two writable models per path → last-writer-wins silent clobber with no `.conflict` sidecar (`+layout.svelte:6442`) → **PJ-089**; the second-screen Tasks-panel `onToggle` writes the shared `.md` (Display-not-Domain) with NO `broadcastNoteSaved`, watcher-suppressed → main window clobbers the toggle on its next save (`SecondScreenPage.svelte:1681`/`:1537`) → **PJ-090**. Also re-confirmed: `yamlDoc.ts:150` nested-object-list (PJ-085/G4), `search.rs:9172` reindex-no-ensure, folder-rename no-cascade (PJ-074), and the persisted-JSON non-atomic/fire-and-forget cluster (PJ-075/link_types/saveCollections/saveSettings/persistWorkspaces).
- **Note:** the PJ-088 merge save path is otherwise CLEAN — the safety census designed the write-wire (model + durability gate, sidecar→trash reversible, zero-loss-until-durable-save) up front, and the 3 adversarial judges + this sweep confirmed it beyond the one re-base finding.

## Findings Register — PJ-071 close cycle (2026-07-12)

**Per-build whole-app sweep** (`wf_4dd12a39-694`, 46 agents, **24 confirmed**) at the PJ-071 (bulk Accept-All RMW race) build boundary.

- **PJ-071's own fix (bulk_ops.rs `accept_one` → `gate_rmw`): ZERO in-diff findings.** A clean mechanical migration to the proven primitive; the read→modify→write is now atomic under the per-path lock, off the dispatch thread.
- **NEW HIGH in the same function — filed PJ-091 (NOT PJ-071's change).** Accepting a classifier suggestion REPLACES a note's `sources:`/`content_type:` frontmatter with the suggestion's ids, silently truncating the user's MANUAL multi-value axis (`sources: [testimony, perception]` → `[testimony]`). Root: the suggestion builder (`classifier/mod.rs:128-148`) reads only `primary + see_also` and drops `.secondary` (`cece/synthesis.rs:209-222` `ua_short_circuit_axis`); `accept_one` then REPLACES (not merges). Needs a classifier-synthesis look + a Boss ruling on accept semantics — filed, not silently parked (WA#6).
- **Pre-existing (mapped/re-confirmed):** Index-preview two-writable-model clobber (PJ-089, `+layout.svelte:7300`); the persisted-JSON non-atomic + fire-and-forget cluster — `link_types.rs:535`, `review.rs:762`, `style_presets.rs:51`, `saveCollections`, `set_review_priority` (PJ-087/075/076); folder-rename no-cascade + incoming-aggregate recompute + link archive/unarchive (PJ-074); `yamlDoc.ts:150` nested (PJ-085); `reindex_single_note` Ok-on-None (search.rs); the SS peek/companion model-reuse + leak (PJ-068/090 area); `wasRecentlyWritten` echo-window; `provenance.rs:100` sync-walk (PJ-077). None block PJ-071.

## Findings Register — PJ-091 close cycle · WHOLE-APP sweep (2026-07-12)

**Whole-app cycle sweep** (`wf_f2a07366-fc5`, 37 agents, 14 hunt groups, **17 confirmed** — every candidate refuted before confirm). Ran at the PJ-091 (accept-merge) build boundary; the diff-scope arg didn't apply, so it swept the whole app. Severity histogram: **1 APP-KILLER · 3 HIGH · 8 MED · 5 LOW.** Full per-finding register: `scratchpad/pj091-sweep-register.md`.

- **PJ-091's own fix: ZERO in-diff findings.** The `cece-sources-derived` scope returned **0 confirmed**; no finding touches `sources/mod.rs`, `sources/bulk_ops.rs`, `classifier/mod.rs`, or `SourceReviewPanel.svelte`. The accept-merge change introduced no silent failure.
- **NEW APP-KILLER — filed PJ-092 (NOT PJ-091's change).** `flushAllTabsInLibrary` (`store.ts:1030`) discards each pre-cascade flush's `SaveOutcome` (`saveNoteSession(...).then(()=>{})`); `noteSession.save` does NOT throw on a durable-write failure (it surfaces the banner, keeps the net, leaves the model dirty, returns `{ok:false}`), so a FAILED flush of an open, dirty backlink-source tab is treated as success. The rename cascade then rewrites that tab's STALE on-disk `[[Old]]`→`[[New]]`, `reloadTabsFromDisk` force-reseeds the model CLEAN from stale disk (version→0, silent even in dev — the tripwire only warns on a path CHANGE), and `clearWriteAhead` deletes the recovery net — the user's unsaved edits are silently, permanently lost, and the save-health banner self-heals to green. The sibling `renameItem` path was explicitly hardened against this exact class (`renameFlushOk` gates the reseed + net-clear, store.ts:3194-3240); the whole-library cascade path was not. Reachable with autoUpdateLinks on + a second dirty tab containing the renamed link within the 1.5 s autosave window + a transient `.md` lock (Syncthing/OneDrive/Defender). **Fix = mirror `renameFlushOk`** (capture each flush outcome; gate the reseed + net-clear on success).
- **3 HIGH:** folder rename does `gate_rename` only (no descendant DB cascade) AND watcher-suppresses both folder paths, defeating the freshness heal (`libraries.rs:927`) → **PJ-074** (the watcher-suppress nuance is a new addition to PJ-074's scope); `save_pulse_data` non-atomic truncate-then-write of the review source-of-truth (`review.rs:762`) → **PJ-075**; PropertyEditor injects a forced/registered property TYPE onto every projected key → breaks the G4 `composeFrontmatter` unification invariant (`PropertyEditor.svelte:364`) → **NEW**, folded into the frontmatter cluster **PJ-073/085**.
- **8 MED / 5 LOW — mapped:** the reindex-silently-skipped-when-`state.db`-is-None class (`search.rs:9285` + `:9172` no `ensure_search_db_ready`, + `NoteEditor.svelte:264` reindex `.catch(()=>{})` swallow) → **NEW PJ-093**; rename cascade consumes only `result.rewritten`, ignores `CascadeResult.failed[]` (`+layout.svelte:6304`) → PJ-074; the persisted-JSON fire-and-forget cluster (`saveCollections` store.ts:1279, `saveSettings` :5404, `persistWorkspaces` :5570) → PJ-075/087; `collect_library_notes` sync `#[tauri::command]` full-tree walk+read (`libraries.rs:5280`) → PJ-077 (a third sync-walk command); `flushAllTabsInLibrary` no-onSaved-reindex (MED, same file as PJ-092) → folded into PJ-092. LOW batch: `rename_item_db_tail` `.find()` picks first-prefix (parent) library not most-specific (`libraries.rs:1115`); FocusPane title edit silently discarded — host never wires `ontitlechange` (`+layout.svelte:7879`); SS companion note-view model created under a fresh `Date.now()` id, never `close()`d → unbounded per-window Map growth (`SecondScreenPage.svelte:995`).

## Findings Register — PJ-092 close cycle · WHOLE-APP sweep (2026-07-12)

**Whole-app sweep** (`wf_b57f6cd6-be3`, 34 agents, **15 confirmed**: 2 HIGH · 10 MED · 3 LOW). Ran at the PJ-092 (rename-cascade dirty-guard) build boundary. Full per-finding register: `scratchpad/pj092-sweep-register.md`.

- **PJ-092's own fix: ZERO in-diff findings.** The `rename-cascade-integrity` scope returned **0 confirmed**; no finding touches the `reloadTabsFromDisk` region (`store.ts` 686-745). **The APP-KILLER that headlined the PJ-091 sweep (`flushAllTabsInLibrary` → force-reseed-stale) is now ABSENT from the sweep** — the dirty-guard closed it. Verified: the fix introduced no new silent failure and removed the app-killer.
- **2 HIGH — both already filed:** folder rename does `gate_rename` only (no descendant DB cascade) + watcher-suppresses both folder paths, defeating the freshness heal (`libraries.rs:927`) → **PJ-074**; SS Tasks-panel toggle no `broadcastNoteSaved` → main clobbers it (`SecondScreenPage.svelte:1681`) → **PJ-090**.
- **3 NEW (from this sweep):** **PJ-094** — `moveItem` (`store.ts:3307`) repaths the model without the flush-before-op guard its siblings (`renameItem`/`toggleTaskReconciled`/`drainCidEnsure`) have (MED, notemodel-ownership; needs verification of actual loss vs. repath-preserves). **PJ-095** — a debounced save dropped by `NoteEditor`'s `saving` single-flight guard (`NoteEditor.svelte:241`) is never rescheduled and leaves a stale write-ahead net → latest edit lost on a crash in that window (MED, editor-lifecycle). **PJ-096** — a dirty-note external-edit whose `.conflict` sidecar write FAILS is swallowed (`store.ts:364`, console.error only) → the PJ-070 zero-loss guarantee has a silent hole if the sidecar write itself fails (LOW).
- **10 MED/LOW mapped to existing:** `flushAllTabsInLibrary` no-onSaved-reindex (`store.ts:1055`) + `search.rs:9179` reindex-no-ensure → PJ-093; link archive/unarchive TARGET aggregate not recomputed (`search.rs:8243`/`:8275`) → PJ-074; `yamlDoc.ts:150` nested-object-list flatten → PJ-073/085; `review.rs:762` non-atomic + `saveCollections`/`persistWorkspaces` fire-and-forget → PJ-075/087; FocusPane title discarded (`+layout.svelte:7879`), `resolveStructuralConflict` no-reindex (`store.ts:4199`) → LOW batch.

## REVERT NOTE — PJ-092 (2026-07-13)

The PJ-092 "dirty-guard" fix (commit `0a605f02`, recorded in the PJ-091/PJ-092 close-cycle registers above as closing the rename-cascade APP-KILLER) was **REVERTED** — it introduced a deterministic FREEZE in the exact scenario it protected (a note left dirty + disk-mismatched after the cascade hangs the Svelte reactive layer). Boss ruling: "FIX IT, don't patch it, or revert PJ-092." Code restored to pre-PJ-092 (`fd6008bc`); the original bug is re-opened (Pending Jobs v1.24, Group 1) to be redone via the full `/migration`, Reproduce-First on the RUNNING app. **The above PJ-092 sweep registers are historical** — they reflect the (now-reverted) state at those build boundaries. Lesson: the freeze was invisible to the store-level vitest — editor-lifecycle/rename-cascade changes require running-app verification + a Boss test before commit.

## PJ-092 REDO — /migration close (2026-07-13)

PJ-092 (reverted 2026-07-13 as a frozen band-aid) was redone via the full `/migration` + a NEW **design-stage safety inspection** (`wf_f922a5cc-f78`) that adversarially reviewed the PLAN before any code and caught **5 hazards for free** (Arabic-NFC path-match, H2 await-window race, H3 focus-blind reseed, the 4 sibling callers, H5 alias-refresh). `/simplify` caught 2 more contract gaps (belt-not-NFC, siblings-not-bounded). The Phase-4 Audit (`wf_abf7f854-5cd`): 11/11 invariants HOLD, migration-path PASS, 1 drift (the `cascade:rewrote` listener bypassed the fail-closed belt) FIXED. Boss live-tested A1/A2/B1/B2 + clean-binary sanity — all PASS. **Process change (Boss-endorsed): the Safety Inspection now reviews the DESIGN (Architect/Plan), not just the code — the highest-leverage place to catch a design flaw.** Follow-up filed: **PJ-097** (FocusPane not under the CascadeFreezeOverlay during a cascade — pre-existing, contrived re-type race).

## PJ-089 close-cycle register (2026-07-13)

**PJ-089 — Index-panel preview two-writable-model silent clobber — FIXED (read-only peek).** The Index split-pane preview mounted a WRITABLE `NoteEditor` on a standalone `index_preview_*` tab whose unique id keyed a SECOND single-ownership NoteModel for one path → last-writer-wins silent clobber, invisible to `adoptExternalChangeIntoTabs` (outside `openTabs`). Fixed by mounting the preview `readOnly={true}` (removes the second writer structurally) + an "Open to edit" promotion via `openNoteTab` path-dedup + a lifecycle-owned `$effect` model disposal + a link-follow override. Reproduce-First `indexPreviewClobber.test.ts` (Recipe Q). Boss live-tested Stage 1 + link + Stage 2 — all PASS.

**Per-build safety inspection = 0 in-diff findings; focused adversarial review of the link increment = SAFE on all 6 vectors.**

**Per-cycle whole-app sweep** (`wf_ca0d3aa9-3d6`, 34 agents, **14 confirmed**). The PJ-089 diff's OWN change: ZERO findings.
- **4 NEW filed:** **PJ-098** *(HIGH)* — OrgChart drag-drop move calls raw `invoke('move_item')` instead of the `moveItem()` store wrapper → skips `migratePathKeyedAuxStateOnRename` + `openTabs` repath/repathNoteModel → open-tab stale path + aux-state divergence (`OrgChart.svelte:254`). **PJ-099** *(MED)* — `loadTabHistoryEntry` (Alt+←/→) flushes then AWAITS `read_note` before force-reseeding → a keystroke in the post-flush await window re-dirties then is discarded by the reseed (`store.ts:1295`). **PJ-100** *(MED)* — SenseMakingCanvas `write_canvas` auto-save swallows ALL errors on a 1000ms debounce, no save-health/retry/net/flush-on-destroy — the canvas's ONLY persistence path (`SenseMakingCanvas.svelte:147`). **PJ-101** *(LOW)* — SS peek preview stale-on-re-peek (`SecondScreenPage.svelte:427`).
- **10 mapped to existing:** folder-rename no-cascade + watcher-suppress (`libraries.rs:927`) + link archive/unarchive TARGET aggregate (`search.rs:8242`/`:8274`) → PJ-074; `yamlDoc.ts:150` nested-object-list ikhtilāf collapse → PJ-073/085; `search.rs:9179` reindex-no-ensure_db → PJ-093; `review.rs:762` save_pulse non-atomic → PJ-075; `libraries.rs:5281` collect_library_notes sync-walk → PJ-077; FocusPane title discarded (`+layout.svelte:7963`) + BacklinksPanel linkMention swallow (`:197`) + SS companion model leak (`SecondScreenPage.svelte:1127`) → existing LOW batch.

## PJ-090 — RESOLVED BY CUT (2026-07-13)

The SS Tasks-panel cross-window clobber was **not fixed — the toggle was CUT** (Display-not-Domain). SO#8 revealed the ledger's premise rested on stale code: the current read-only Knowledge Cockpit's Tasks facet is a STUB, so the default SS cannot toggle tasks; only the split-view split-companion mode could. The Art-Director-&-Team honest audit (`wf_043756ee-352`) ruled the SS Tasks toggle a Display-Not-Domain duplication; the earlier cross-window-broadcast fix (adversarially SAFE ×7 but conceptually wrong — it "makes an illegal write work") was **reverted**. Fix: `TasksPanel.readOnly` prop; both SS mounts read-only; the write handlers + `toggleTaskReconciled` import removed. Main-window toggling untouched. svelte-check 0, vitest 341, Boss-tested PASS. **Standing lesson: SO#8 must verify a PJ against the RUNNING SS structure (the orientation Second-Screen §), not just the presence of the code.** The SS is being re-conceived as the three-zone Cockpit (`/migration`), which will delete the Tasks facet + the stub-facet duplication entirely.

## The 2026-07-14 whole-app sweep — register pointer (drift fix, appended 2026-07-17)

The SS-Cockpit §1-boundary whole-app sweep (`wf_8b0a5104-6e8`, 83 agents, **55 confirmed**; the
Part-A diff itself: ZERO) was filed to the ledger (v1.28: PJ-102/103/104/105 + save_pulse=PJ-075)
but its register was never appended here — the ledger's "Open · Charter" marks pointed at nothing.
Full register: `lab/reports/SWEEP-REGISTER-2026-07-14-wf_8b0a5104.md`. NOTE: that register's PJ-103
mechanism claim ("the staleness guard drops the switch-away teardown flush") was REFUTED by the
2026-07-16 live reproduction — see the PJ-103 close entry below; the register stands as the
historical record, the close entry is the correction.

## PJ-103 close-cycle register (2026-07-16/17)

**PJ-103 — app close never flushed dirty note models — FIXED (APP-KILLER, Boss-validated live).**
The Reproduce-First arc on the release binary: (1) REFUTED the filed switch-drop mechanism (2/2
fast switches persisted the outgoing note — PJ-086 flagged re-examine-first); (2) CONFIRMED the
real loss at the close instant (type + ✕ inside the 1.5 s debounce → the Boss's MARKER-THREE never
reached disk; beforeunload's sync net-stash landed but its async disk write was cut by
`win.destroy()`); (3) exposed the localStorage net as NON-DURABLE — a Chromium leveldb
MANIFEST/log-orphan inconsistency deleted the whole session's net on reopen (`Delete type=0 #3`,
leveldb LOG; evidence `lab/reports/pj103-evidence-000003.log`) → TOTAL silent loss → **PJ-110**.
**Fix:** the `session:final-flush` handshake now runs persist → `flushAllForAppClose` (durable
per-model flush + re-pass + `final_flush_residual_dirty` journal marker + awaited FTS reindex) →
ack; Rust cap 700 ms → 5000 ms + `final_flush_no_ack_5s` marker; listener registered at the top of
onMount; per-id save serialization at the gate (sync-prefix preserved); updater `relaunch()`
flushes first.

**Stand-in adversarial review** (`wf_5bb5c713`, 4 refute-first lenses — the automated
`safety-inspection` is rate-limited until Jul 18; its whole-app sweep is scheduled for Jul 18 4am):
**12 findings — every one fixed pre-commit or filed:**
- FIXED: post-flush typing window (re-pass before ack) · unserialized same-id saves
  (APP-KILLER-PLAUSIBLE; per-id chain) · journal-invisible flush failure (`final_flush_residual_dirty`)
  · false timeout marker (renamed `final_flush_no_ack_5s`, honest semantics) · boot-window 5 s
  stall (listener first) · index↔disk divergence at close (awaited FTS reindex) · arrangement
  starvation (persist-first) · stale `session_flush_notify` contract doc · updater relaunch bypass.
- FILED: PJ-110 (net durability) · PJ-111 (flushOutgoing cascade-gate, design-needed) ·
  PJ-112 (OS-shutdown bypass — tao routes no WM_ENDSESSION) · PJ-113 (close-time embed staleness).
- Review-of-the-review lesson: 2 vitest recipes (Recipe: type-during-await, compare-and-clear)
  caught my first serialization draft breaking save()'s synchronous compose+setNet prefix — the
  contract the beforeunload stash depends on. The recipes are load-bearing; the fast-path preserved it.

**Gates at close:** vitest 427 · svelte-check 0 · cargo clean · Boss test PASS (MARKER-FOUR on disk
at the close instant · clean close instant · typing burst clean).
