# Session Log — 2026-07-03

## Function in hand: the `get_360_view` note-open freeze (the one real perf bug)

**Concept (the horse):** the 360° view answers *"what is the complete cognitive neighbourhood of this one note — every link, tension, blind-spot, and stratum around it?"* Its purpose is an instant, whole-surround read of a single note's place in the graph. A view that answers "where does this note sit?" must *appear* — a multi-second freeze is the opposite of the concept.

Boss chose (AskUserQuestion) the **360 freeze** over MIG-088 Phase 6 / Arabic-caret; then chose the **safe fix (async + loading state)** over a full index rewrite this session.

---

## Reproduce-First — measured, not guessed

**Verified facts (code-read):**
- `get_360_view` was `#[tauri::command]` (SYNC) — [inspector360.rs:91](../../src-tauri/src/inspector360.rs) — and re-walks the note's **whole library from disk** on every call (`scan_all_notes` — `read_to_string` + 2 regex + `fs::metadata` per `.md`).
- It fires **only when the 360 surface is visible** — [+layout.svelte:1433](../../src/routes/+layout.svelte) `(rightSidebarOpen && rightSidebarTab === 'inspector360') || showInspector360`, debounced 200 ms, seq-guarded, last-key-guarded. So the freeze reproduces on *switch/open a note with the 360 panel open* / *open a 360 tab* — **not** every note open.

**Library survey (Boss's real data):** Boss-confirmed the active (and always-active) universe is **`Eisa Cognitive Knowledge`** = **7,682 note_meta / 234,036 note_links** across ~19 external libraries under `E:\Cognitive Knowledge\` — the freeze library. (The dev `world.uconstellation.app/universes.json` I read listed `كون عيسى` as active — a stale/dev-install config; the running app is authoritative.) `get_360_view` walks **one library** (the open note's) — worst case **History = 549 md / 36 MB**.

**Measurement (a `#[test]` running the REAL `scan_all_notes` + `precompute_all_strata` over History, released, then removed):**

| Run | scan_all_notes (549 notes) | precompute_all_strata | total |
|---|---|---|---|
| Cold | 2.80 s | 18 ms | **2.82 s** |
| Warm | 0.32 s | 19 ms | 0.33 s |
| Warm | 0.35 s | 22 ms | 0.37 s |

**Diagnosis:** `get_360_view`'s isolated cost is **~2.8 s cold / ~0.35 s warm** — a real SYNC-on-IPC-thread freeze, but **NOT the "tens of seconds"** the Boss felt. That was dominated by note-open background **indexing**, with the 360 scan stacked on top (and only when the panel is open). Confirms the phantom-reset conclusion. **The tens-of-seconds note-open indexing hang is a SEPARATE issue** (its own reproduce-first pass).

---

## Design fan-out — workflow `wf_f5388985-d2a` (3 agents)

1. **Tier-2 feasibility (index-read):** YES — the index already holds everything `get_360_view` re-derives (`note_meta.outgoing_links_json`/`outgoing_link_types_json`/`outgoing_count`/`incoming_count`/`incoming_link_types_json`/`tags_json`/`word_count`/`body_text`/`properties_json`/`modified`/`created_at`; `note_links` = indexed 234k-row graph). The whole command can become **O(neighbours)** (target row + direct + 2nd-order via `idx_link_source` / `idx_nl_tnl`) with **no full-library disk walk** — the Rule-8 fix.
2. **Async blast-radius (WA#4):** SAFE. One-token change; body has **no `.await`** (pure thread-offload); all helpers Send; the **sole caller** already awaits + seq-guards; 3 shipped siblings use the exact pattern (`constellation_embed_notes`, `scan_unlinked_mentions`, `save_universe_link_types`). Output byte-identical → **zero verdict change**.
3. **Verdict-parity (index-read):** **NOT a pure re-plumb.** Only counts (already index-sourced, MIG-085) stay identical; maturity/orphan likely-identical; **stratum / SPOF / provenance / stage / tags / gaps / trails = drift-risk** — from cross-library scope (index is cross-library, FS scan is same-library), DISTINCT-vs-occurrence definitional split, and parser/value mismatches (has_external not stored as a bool; extract_stage quirks; 360 tags read inline-only today). → the index rewrite needs its own `/migration` with per-verdict decisions. **Queued, NOT this session.**

---

## SHIPPED this session (safe fix — async + loading state)

- **Step 1 — `get_360_view` → `#[tauri::command(async)]`** ([inspector360.rs:91](../../src-tauri/src/inspector360.rs)) + a PJ-066 rationale comment. Moves the ~2.8 s cold disk-walk off the IPC dispatch thread → the UI never freezes. Byte-identical output. Temporary measurement bench removed.
- **Step 2 — loading-vs-stale isolation** (the resume's "isolate settled-vs-loading from the start"):
  - `inspector360Loading` `$state` in `+layout.svelte`; the fetch `$effect` now **drops the previous note's data + sets loading=true** on a new-note key, clears both on settle. Reactivity checked (Perf Rule 2): the new-note path short-circuits before reading `inspector360Data`, so the bare `= null` write is **not** a tracked dep → no `$effect` self-trigger; cached same-note reselect returns early (no spinner flash).
  - `loading` prop wired to **both** mount sites (full overlay + right-sidebar tab).
  - `Inspector360.svelte`: `loading` prop + a loading branch (compact + full) showing a self-contained CSS spinner (`--interactive-accent` colour, `prefers-reduced-motion` aware) + `$t('inspector360.loading')`. Without this, async would leave a different note's 360 data on screen for ~2.8 s.
- **i18n ×15:** `inspector360.loading` added to all 15 locales (native translations; ar matches the existing "المنظور الكروي" terminology).

**Verify:** `cargo check --release` 0 errors (55 pre-existing warnings, none new); `svelte-check` **0 errors** (324 warnings, baseline — none new); all 15 locales parse; frontend rebuilt (new string confirmed in bundle); release binary rebuilt **Jul 3 10:16** (mtime verified fresh; new string + `i360-spinner` confirmed in `build/assets/screen-*.js`).

**Boss test:** **PASS** (2026-07-03) — on `Eisa Cognitive Knowledge` (Boss-confirmed always-active): opening a link-dense note / the 360° panel no longer freezes the app; the "Loading 360° view…" spinner shows, then the correct note's data fills; note-switch never shows the previous note's data; cached reselect is instant.

**/simplify (SO #4 gate):** 4 cleanup agents (reuse/simplification/efficiency/altitude) — **all CLEAN, ship as-is.** Reuse: per-component spinner CSS is the established convention (`.cmap-spinner`/`.oc-spinner`/…; no shared Spinner util) → `.i360-spinner` follows it. Simplification: imperative `$state` is the correct minimal form (loading tracks async settlement + a non-reactive `lastFetchedInspectorKey` guard — a `$derived` can't). Efficiency: no `$effect` self-trigger (Svelte-5 batches internal writes; new-note path short-circuits before reading `inspector360Data`); spinner animation destroyed with the element. Altitude: async is the right depth (index rewrite correctly deferred); host-owned loading is the correct altitude (sibling of `data` — debounce/seq-guard live in the host). No fixes applied.

---

## QUEUED (honest, deferred)
- **`get_360_view` index-read rewrite** — its own `/migration` (Architect input = the verdict-parity findings above: cross-library scope? DISTINCT-vs-occurrence? add has_external/trail-kind index surfaces? align strata.rs + neighbour strata). Makes 360 ~ms; the proper Rule-8 write-time-derivation fix.
- MIG-088 Phases 6–10; Arabic callout End/Home caret known-issue.

---

# Function in hand (pivot, Boss-directed): the NOTE-OPEN FREEZE

**Concept (the horse):** opening a note is the single most frequent act in a knowledge tool — it must never cost the user control of the app. Background derivation may take as long as it needs; the user's hand on the app may not be taken hostage by it.

**Boss context:** Boss didn't recall "facing" a note-open freeze — correct, he never experienced it under that name; he experienced it as the "reset freeze" (10s→39s rounds, 2026-07-02/03). The mechanism was proven then: *note-open background work holds the DB writer lock; any SYNC command on the IPC dispatch thread that needs the lock (or is slow) freezes the whole app.* Two victims already fixed: the reset save (`save_universe_link_types`, last session) + the 360 view (`get_360_view`, this session). Boss: "proceed with this issue. Fix it."

**Approach (Solve-the-Class, not the instance):** the class = *SYNC `#[tauri::command]` + writer-lock touch, callable during normal use*. Sizing: **227 SYNC commands vs 21 async**; writer-lock touches concentrate in `search.rs` (44), `sources/mod.rs` (11), `review.rs` (4), `libraries.rs` (3), `tension.rs` (2)… Discovery workflow `wf_2d6ff3e5-edb` (4 agents): note-open call graph · full SYNC+lock sweep · writer-lock holders at note-open · user-action vectors. Fix design after synthesis (per the PJ-066 canonical rule: async + route pure reads via `with_read_conn`), with a per-command race check before any write command goes async.

## Discovery synthesis (wf_2d6ff3e5-edb — 4 agents, 209 tool uses)

**THE MECHANISM (all four agents converge, verified):**
- **Note-open itself is writer-lock-clean (ms-scale).** No reindex/embed/backfill fires on a plain open. `ensure_cid_cn_cmd` = no DB lock; panel reads ride the PJ-066 §C3 read-only reader; `get_360_view` async since this morning.
- **The multi-second continuous lock holder is the SAVE path:** leaving a *dirty* note (edited within its 1500 ms debounce) fires the teardown flush → `constellation_search_reindex` (async, background) holds the writer mutex **CONTINUOUSLY 2.5 s warm / 11 s cold** (measured, 533-link note) across index_note + CTSE + maintain_incoming + maintain_sky. The embed's ~32 s ONNX inference is lock-free (engine mutex; writer grabs are ms-scale) since MIG-076 §D.
- **The freeze = composition:** any of the **59 remaining SYNC writer-lock commands** dispatched during that 2.5–11 s window parks the single WebView2 IPC dispatch thread on the mutex → the ENTIRE app freezes (including the next note's `read_note`). This — not any note-open job — is the felt "note-open freeze" and was the "reset freeze" (then-SYNC `save_universe_link_types` was the waiter).
- **The reproducible recipe (by construction):** edit note A → click a wikilink to navigate → flush reindex A (lock held 2.5–11 s) + `constellation_link_traverse` (SYNC + writer lock, fired at that exact moment, store.ts:1477) → app frozen for the reindex remainder.

**THE SWEEP (59 SYNC writer-lock vectors: 14 high / 21 medium / 9 low / 15 dormant-no-callers):**
- High (freeze during normal use): `constellation_search` + `constellation_search_universal` (every debounced search keystroke), `constellation_link_traverse` (every wikilink click), `get_note_review_status` (note switch w/ Review tab), `get_due_notes` (Reviewer open), `compute_note_maturity` (sidebar library expand), `ctse_search_terms_by_concept` (Index concept filter), `toggle_task`, `update_note_property`, `rename_item`, `move_item`, `delete_item`, `delete_path`, `update_links_on_rename`.
- Also flagged (SYNC + multi-second body, no lock): `constellation_embed_text`/`embed_texts` (e5 inference on the IPC thread — the un-fixed sibling of PJ-066's embed_notes), `resolve_wikilink_cross_library` (recursive FS walk on the nav path), `get_provenance_chain` (SYNC full-library FS walk — the un-migrated sibling of get_360_view), `suggest_related_notes` (BM25 on the dispatch thread), `cache_full_links` (legacy flag-off path, 234k-row read).
- Verified NON-vectors: job-starters lock only inside `thread::spawn`; `ensure_search_db_ready` fast path is lock-free (db_ready atomic); `get_backlink_rows`/`get_outgoing_rows`/structural trio ride with_read_conn; backfills are chunked (500/1000-row batches + 50 ms sleeps).

**BATCH DESIGN:**
- **Batch 1 (this pass):** ~22 safe `(async)` conversions — pure reads + self-contained DB writes touching NO note file (review actions write review_schedule only) + stale-result **seq guards** on search callers that lack one (SearchHub confirmed guard-less; QuickSwitcher already has MIG-058/059 stale-discard). Caller-verification workflow `wf_7ba74bfc-7b7` (4 agents) runs the WA#4 per-caller ordering check before any edit.
- **Batch 2 (DEFERRED, explicit — needs the Editor-Surface Gate / BUG-023 harness):** commands that write note files: `rename_item`/`move_item`/`delete_item`/`delete_path`/`update_links_on_rename`, `toggle_task`, `update_note_property`, `resolve_structural_conflict`, `constellation_link_set_confidence`/`archive`/`unarchive` (dual-layer), sources/CECE accepts (may write frontmatter), `set_active_universe` (global teardown). Sync→async changes ordering vs the debounced save — content-integrity class, not this pass.
- **Queued optimization (later):** route the pure-read conversions through `with_read_conn` (kills the wait itself, not just the freeze — search results during a reindex window would be instant instead of delayed); chunk the reindex's continuous 2.5–11 s hold. Both deeper changes, own passes.

## BUILT — Batch 1 (caller-verification `wf_7ba74bfc-7b7`, 4 agents, then applied)

**Caller verification results (WA#4):** all 24 Rust bodies confirmed `pub fn`, no `.await`, Send params, no note-file writes. Frontend: 9 already-safe callers (QuickSwitcher MIG-058/059 value guard; ReviewStatusPanel/ReviewerView monotonic `gen` tokens; IndexPanel `semanticFetchToken`; SourceReviewPanel `_srpLoadSeq`; RelatedCandidates `cancelled` flag; maturity/strata keyed-by-path writes; link_traverse fire-and-forget nothing-consumes-result; resolve_wikilink inline-click flows). **9 hazard callers needed guards** + **1 Rust-side hazard found**: mark_reviewed/snooze_note/dismiss_note do an UNLOCKED whole-file RMW of `review-pulse.json` — safe only under sync dispatch serialization → needs a lock once async.

**Applied:**
1. **24 × `#[tauri::command(async)]`** (uniform 3-line rationale comment): search.rs ×7 (constellation_search, _search_universal, _link_traverse, _ccs_snapshot, _knowledge_health_snapshot, _link_archived, _search_link_counts) · embeddings.rs ×3 (_embed_text, _embed_texts, _embedding_status) · review.rs ×6 (get_due_notes, get_note_review_status, set_review_priority, mark_reviewed, snooze_note, dismiss_note) · libraries.rs ×2 (resolve_wikilink_cross_library, suggest_related_notes) · provenance.rs ×1 (get_provenance_chain) · maturity.rs ×1 · strata.rs ×1 · ctse/search.rs ×1 · sources/mod.rs ×2 (get_suggestions, list_pending_suggestions). *(First scripted attempt mangled the files — doubled attrs + clipped comments; caught by spot-check, `git checkout` reverted, redone line-based clean.)*
2. **`PULSE_LOCK`** static mutex in review.rs serializing the review-pulse.json RMW in all three action commands.
3. **Stale-result seq guards ×9 files** (3 parallel edit agents, each spec from the verification): SearchHub (`searchSeq`, incl. guarded spinner clear) · ConstellationMap (`mapSearchSeq` ×3 checks) · ConstellationSight2 (`sightSearchSeq` ×3) · GraphMindView (`searchSeq` ×2, downstream-synchronicity verified) · OrgChart (`orgSearchSeq` ×1, single-await verified) · CatalogerView (`pickerSeq` + stuck-spinner hole closed on the clear path) · CCSView (`_snapSeq`) · KnowledgeHealthDashboard (`_snapSeq`) · +layout.svelte (provenance `reqPath` gate on then+catch; `_linkCountsSeq` on the cache-reconciled listener).

**Verify:** cargo check --release clean (pre-existing warnings only); svelte-check **0 errors** (324 warnings = exact baseline). Frontend rebuilt (39 s) + release binary rebuilt **Jul 3 11:17** (freshness-verified BEFORE test instructions, per the standing rule).

**Boss test — PASS ×2 (2026-07-03):**
- **Stage 1 (the collision recipe): PASS.** Edit a large link-dense note → immediately click a wikilink to hop → navigation instant, app fully alive; repeated hops all clean. (Before: this exact sequence froze the whole app 2.5–11 s per hop.)
- **Stage 2 (busy-window actions): PASS.** Typing in search during the background window = every keystroke instant, results match the final query (seq guards working); Review tab + "✓ Reviewed" during the window = responsive; panels fill smoothly.
- Honest known-interim: during the ~2.5–11 s background reindex, search *results* can arrive delayed (the async read waits for the writer lock off-thread) — the app never freezes. The `with_read_conn` routing that removes even the wait is the queued follow-up.

### /simplify (SO #4 gate) — 4 agents. 1 fix applied; 1 characterized risk banked; rest clean.
- **Applied:** CatalogerView — removed the redundant `pickerSeq++` in the empty-query early-return (the function-top `++pickerSeq` already invalidates in-flight requests; the early return skips the await). svelte-check re-run 0 errors; frontend + binary rebuilt after (shipped binary == committed source).
- **Reuse CLEAN:** hand-rolled per-component seq guards = the repo convention (inspector360RequestSeq / gen tokens / `_srpLoadSeq` all predate this diff; no shared helper exists or is justified). `PULSE_LOCK` matches the `write_gate::journal_lock()` idiom; per-path gate doesn't apply (single shared file).
- **Simplification:** the uniform 3-line comment ×24 judged acceptable for a uniform batch (detail lives in this log); SearchHub const→guard→assign is minimal; PULSE_LOCK placement correct.
- **Efficiency — the one substantive finding, VERIFIED against Tauri 2.10.3 SOURCE (not guessed):** `tauri-macros::wrapper` maps a sync-bodied `(async)` command to the "sync_threadpool" kind whose generated body goes through `respond_async_serialized` → **`async_runtime::spawn` on the CORE Tokio pool** (ipc/mod.rs:375) — NOT `spawn_blocking`. So lock-waiters park core workers for the wait duration. Honest severity: **bounded degradation, never a freeze** — a pathological window (cold 11 s reindex + rapid typing + several panels) could park ~8+ workers of an N-core pool, briefly queueing OTHER async work (360 fetch, embeds); it self-heals the instant the lock frees, and it is strictly better than the pre-fix state (the same waiters parked the ONE UI-critical thread). Superseded searches also keep their worker parked until the lock frees (guards discard results, not the in-flight call). **Consequence: the queued `with_read_conn` routing pass is now evidence-prioritized** — routing the pure reads to the read-only WAL connection removes both the parking AND the result delay.
- **Altitude CLEAN:** per-command flips are Tauri's only mechanism (no global async switch — verified); host-level guards right for Svelte 5; PULSE_LOCK at the right weight.

### CLASS STATUS after this pass
- **Closed:** every high-frequency everyday action (navigate, search, review, panels, sidebar, suggestions, provenance, embeds).
- **Deferred explicitly (Batch 2, gated):** note-file-writing commands (rename/move/delete family, toggle_task, update_note_property, resolve_structural_conflict, link-confidence trio, sources/CECE accepts, set_active_universe) — Editor-Surface-Gate territory; still freeze if clicked during a reindex window (rare, deliberate actions).
- **Queued follow-ups:** `with_read_conn` routing (evidence-prioritized by the Tokio finding) · chunking the reindex's continuous 2.5–11 s hold · the `get_360_view` index-read `/migration`.
- **Dormant:** 15 no-caller SYNC lock-takers (convert if revived); Sight Wings when re-enabled.
