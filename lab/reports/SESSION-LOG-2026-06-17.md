# Session Log — 2026-06-17 (MIG-079 §C.2b + §C.3)

> **Function in hand:** MIG-079 §C.2b — defer the 234k-row `read_links` off the boot critical path (the actual ~11 s boot win), then §C.3 covering index. Continues the bring-up program. Branch `main`. Active universe "Eisa Cognitive Knowledge" (7,653 notes, 1.97 GB DB).
> Picks up from `lab/reports/HANDOVER-2026-06-17-mig079-c2b.md`. §C.1 / §C.2a / save-path link-skip already SHIPPED + Boss-validated.

## Mapping (WA#4 impact, complete before any edit)

### Rust (`cache.rs`)
- `cache_boot_snapshot_graph` returns `{ links, tags, aliases }`. The `read_links_in_schema` scan (`SELECT source_path, source_name, target_name, link_type, library_name, weight, traversal_count, annotation, last_traversed, confidence FROM note_links WHERE status='active'`) is the 234k-row / ~11.3 s-cold cost.
- After §C.1, `read_tags_in_schema` already reads the `tag_counts` summary (~ms) when stamped; `read_aliases_in_schema` is ~ms (~1.4k rows). So dropping `read_links` leaves `cache_boot_snapshot_graph` genuinely fast (ensure_db + open_reader + count + tags + aliases).
- `cache_boot_snapshot` (back-compat shim) merges core+graph; used by ambient callers. Must keep returning FULL links (route through the extracted helper, not the now-empty graph).
- Index-build pattern: `idx_nl_tnl` is created in `incoming_links_backfill::run()` (off-boot, background, own conn, `IF NOT EXISTS`) — but that run is gated on the `incoming_links` stamp, already set on Eisa's DB, so it won't re-run. → a **dedicated** background builder for `idx_link_boot` with its own stamp (`link_boot_index`).

### Frontend consumers of the edge list (`allLibraryLinks` $state in `+layout.svelte`)
- `effectiveLibraryLinks` (derived wrap) → `getBacklinks`/`getOutgoingLinks` (sidebar effect 1328/1331) — **need full rows**.
- `linkTraversalMap` derived (759) — livePreview `×N` chips — needs full list (minor; fills when edges land).
- `buildSkyData` FALLBACK only (3239) — Sky View normally renders from the write-time `sky_*` payload (`cache_boot_snapshot_sky`); the edge-list fallback fires only when sky isn't ready → must `await ensureFullLinks()`.
- `applyConfidenceLocally`/`applyArchiveLocally` — local mutations of the loaded list (fine).
- **Four sites set `allLibraryLinks = graph.links`:** boot `loadGraph` (3190); federation:ready handler (2591) + post-init re-invoke (2644 — actually a *redundant second full-graph load at boot*); rename-cascade post-fix (5098). All re-route through `ensureFullLinks()`.
- **Sky View / GraphMindView / ConstellationSight / LocalSkyView** read `skyNodes`/`skyLinks` (own write-time source) — NOT the edge array. Untouched.
- **SecondScreenPage** fetches its OWN links via `scanLibraryLinks` (separate command) — untouched (display-not-domain holds).

## Predecessor → Replacement (Predecessor Lookup Rule)
- **Boot link load.** Now: `cache_boot_snapshot_graph.links` consumed at `+layout.svelte` boot `loadGraph` (3190) + 3 other sites; `read_links_in_schema` in `cache.rs`. Replacement: **same place / same module** — the edge read moves to a NEW `cache_full_links` command (same `cache.rs`, same `read_links_in_schema` helper), invoked lazily via a memoized `ensureFullLinks()` + `linksReady` flag in the SAME `+layout.svelte`. No feature relocated across components; no Settings/IPC surface dropped (the boot graph IPC stays, just stops shipping the edge array). The shim keeps full links.

## Design decision logged — §C.3 covering-index columns (deviation from the literal locked list, measurement-justified)
- Locked-design column list was `note_links(status, source_path, target_name, link_type, library_name, weight, traversal_count, last_traversed, confidence)` — it omitted `source_name` (REQUIRED by `getBacklinks`/`buildSkyData`) and `annotation`.
- **Measured (read-only, live DB):** 233,995 rows, ALL `status='active'`; 142,331 carry an annotation but total annotation text is only **1.33 MB** (~6 bytes avg — mostly the type word, often display-suppressed).
- For SQLite to report **USING COVERING INDEX**, the index MUST contain every column the scan SELECTs (a non-covering index on a full-table-returning query is ignored in favor of a table scan). So the index includes `source_name` + `annotation` → the bulk scan stays index-only AND every consumer stays byte-identical (NO new per-note annotation IPC, NO panel refactor). Lower-risk than the literal "drop annotation + fetch per row" reading; the 1.33 MB measurement makes inclusion negligible. `context` stays excluded (always `''` at boot).
- Final index: `idx_link_boot ON note_links(status, source_path, source_name, target_name, link_type, library_name, weight, traversal_count, last_traversed, confidence, annotation)`.

## Build sequence
1. **§C.2b** — Rust: drop `read_links` from `cache_boot_snapshot_graph`, add `cache_full_links`, shim via helper. Frontend: `ensureFullLinks()`/`linksReady`, panel `loading` prop, re-route the 4 link-load sites, universe-switch reset. Verify: cold `graph_ready` drops; Backlinks/Outgoing/Sky never-empty after open; Editor-Surface Gate.
2. **§C.3** — `link_boot_index.rs` background builder + schedule; assert `EXPLAIN QUERY PLAN = USING COVERING INDEX`; lazy scan faster.

## BUILT + Claude-verified (one combined commit — lib.rs interleaves both steps so they land together)
- **Files:** `cache.rs` (drop boot link read + `cache_full_links` + `BootLinks` + shim), `lib.rs` (register `cache_full_links` + `mod link_boot_index`), `search.rs` (schedule the index builder), `link_boot_index.rs` (NEW), `+layout.svelte` (`ensureFullLinks`/`linksReady`/guards/re-routes), `BacklinksPanel.svelte` + `OutgoingLinksPanel.svelte` (`loading` prop, reuse `common.loading`).
- **Verification done (Claude-side, pending Boss runtime validation):**
  - `svelte-check`: **0 errors** (315 pre-existing CSS warnings).
  - `cargo build --release`: clean (52 pre-existing dead-code warnings).
  - `cargo test --lib`: **935 passed / 0 failed / 6 ignored** — incl. cache:: per-schema link/alias isolation.
  - New `link_boot_index` tests: covering-index plan assertion (`USING COVERING INDEX idx_link_boot`) + stamp gate — **both pass**.
  - **Live-DB rehearsal (copy of the 1.97 GB DB):** before = `SCAN note_links`; after `CREATE INDEX idx_link_boot` (built in **2.0 s**) = `SEARCH note_links USING COVERING INDEX idx_link_boot (status=?)` — proven on the REAL 234k-row schema. (Warm scan ~450 ms both ways — materialization-bound; the covering win is on COLD page reads, which Eisa's boot-history will quantify.)
  - Frontend re-embed confirmed: `cache_full_links` + `_linksEpoch` present in `build/`.
- **Adversarial review (general-purpose agent on the diff):** no P0; all 7 audit questions PASS; Editor-Surface Gate clean (zero matches for `composeNoteModel|write_note|save_note|onDestroy|#key|.content =`). Two **P1 races found + FIXED before commit:**
  - **P1-1 universe-switch stale-edge race** — an in-flight `cache_full_links` from the OLD universe could resolve after the switch reset and write stale edges (empty-overwrite guard can't catch non-empty stale). **Fix:** `_linksEpoch` captured at call entry, bumped on switch, re-checked before every assignment → stale resolution discarded.
  - **P1-2 per-keystroke retry on failure** — the sidebar `$effect` re-runs per keystroke; a persistent `cache_full_links` failure would fire one IPC/char (keystroke-hot-path IPC ban). **Fix:** `_linksRetryAfter` 3 s cooldown gates the keystroke-driven retry; idle pre-fetch / force paths still retry.
  - Nits logged (defer to `/simplify`): orphaned non-schema wrappers `read_tags`/`read_links`/`read_untyped_links_fallback` now unused (pre-existing-style dead code).
- **Editor-Surface Gate:** §C.2b/§C.3 change only the link-edge READ path (a derived view of `note_links`); they touch NO note content / save / lifecycle code → the content-integrity class is structurally untouched (same class as §C.1/§C.2a derived views). Boss test still exercises the gate (Focus round-trip + tab switch + body-intact) as belt-and-suspenders.

## Pending Boss runtime validation (the gate before "validated")
Launch the rebuilt binary (mtime 2026-06-17 10:30). Stage 1: boot is faster (boot-history `graph_ready` drops from ~33 s) + Backlinks/Outgoing/Sky never-empty after open + the one-time `[link_boot_index]` diag line. Stage 2: Editor-Surface Gate (Focus round-trip, tab switch, body intact).

## ⚑ Boss Stage-1 result + MEASURED pivot (SO #5 state-of-standing) — 2026-06-17 ~10:45

**Boss findings (10:45 boot of the 10:30 binary):** boot ~20 s to populate (not the hoped ~2 s floor); backlink rows appear then a **~7 s freeze** before the list scrolls (Boss: pre-existing, "since we created it"); Sky View **PASS**; step-5 **lag/non-responsiveness/thrashing** within a note + switching notes (pre-existing).

**Measured (boot-perf history `[15]` = the 10:45 boot — times are UTC `Z`, so `06:45:57Z` = 10:45 local; file mtime confirms):**
- `read_links = None`, `cache_snapshot_graph_server_timings = [ensure_db=0, open_reader=0, read_tags=164, read_aliases=0]` → **§C.2b deferral WORKS** — the 234k link read is off the boot graph IPC; its body is **164 ms** (was ~11,500 ms).
- `[link_boot_index] idx_link_boot built + stamped` in `diagnostics.log`; `idx_link_boot` confirmed present on the live DB → **§C.3 WORKS.**
- **BUT `graph_ready = 16,935 ms`, of which `cache_snapshot_graph_queue_ms = 14,991 ms`** — the graph IPC sat ~15 s in the dispatcher QUEUE before its (now-trivial) body ran. The `ipc_arrival_log` shows `cache_boot_snapshot_sky` invoked at +2094 ms then a ~14.3 s gap with no other IPC → the graph waits behind **`cache_boot_snapshot_sky`**.
- **`sky_links` = 233,995 rows** (measured) — the IDENTICAL twin of `note_links`. So §C.2b deferred ONE 234k cold read and the boot now waits ~15 s on its untouched twin (the Sky read).

**Honest conclusion:** the Plan/handover mis-attributed the boot cost ENTIRELY to `read_links` (11 s). The data shows it was ~7 s of a ~24 s boot; the **Sky read (~15 s) is the co-equal/bigger half**, and the **234k-edge → JS main-thread load is the root of the panel/editor freezes** (deferring moved its timing, not its cost). §C.2b is a correct, committed ~7 s improvement — NOT the headline win.

**Boss ruling (pivot):** **Kill the 234k in-memory edge array.** Switch Backlinks/Outgoing to **per-note SQLite queries** (like the write-time count badges) so the app NEVER holds all 234k links in JS. Cures the scroll-freeze + thrashing at the root; likely removes the Sky boot cost as a side effect. → opened as **MIG-079 §C.2c** (read-path change crossing Rust↔Svelte → `/migration`: Architect → Plan → approval → Build → Audit). WA#5 note: per-note backlink queries are the STANDARD PKM pattern (Obsidian/Logseq), not invention.

**State protected on `main` (`6a4d5e20`):** §C.2b (edge deferral + `cache_full_links` + guards + 2 P1 fixes) + §C.3 (`idx_link_boot` covering index, built on the live DB). Functionally correct + never-empty verified by Boss (rows appear, Sky passes). Editor-Surface Gate (Stage 2) NOT yet run — deferred (the content path is structurally untouched, so low risk, but run it before final close).

## §C.2c — Architect + Plan + WA#5 cross-check (Boss-approved with conditions)
- Architect+Plan: `docs/MIG-079-C2c-Architect-Per-Note-Link-Queries.md` (committed `49127c2f`). End-state: per-note SQLite row queries replace the in-memory 234k array; retires §C.2b's `cache_full_links`/`ensureFullLinks`/`linksReady`/`idx_link_boot`. Sky boot read = separate §C.2d.
- **WA#5 cross-check (Boss-required before approval; `0f80abc6`):** CONFIRMED the design as the textbook split — COUNT = write-time materialized (§C.2a, an aggregate), ROWS = read-time indexed per-note query (a bounded access path). Backlinks == the inverted-index posting list for a note's name (`idx_nl_tnl`). Materializing rows would tax the save path (wrong trade for a keystroke-latency-sacred app). Corroborated: materialized-view-vs-index guidance, IR/Lucene posting-list practice, SQLite-as-graph (per-node query > load-all), Obsidian Backlink Cache + Logseq→SQLite.
- **Boss ruling:** approved, **stop after Step 1 for proof**; build now.

## §C.2c-1 — Rust per-note row queries: BUILT + rehearsal PROVEN (the Step-1 gate)
- `cache.rs`: `get_backlink_rows(note_name, aliases)` (the §C.2a alias-aware `target_name_lower IN (...)` set on `idx_nl_tnl`, federated) + `get_outgoing_rows(note_path)` (`idx_link_source`, federated) → the exact `NoteLink` shape `read_links_in_schema` returns (context lazy). Registered in lib.rs. **Additive — no frontend wired, no behavior change yet.**
- **In-memory unit test** (name+alias match, archived excluded, empty-type→None) PASS. **Full lib suite 936/0** (+6/7 ignored), no regression.
- **Live-DB rehearsal (copy of the 1.97 GB DB), 80 notes (40 hubs + 40 mid/aliased):** backlinks `set_mismatch=0` (query == brute-force oracle) AND `count_mismatch=0` (deduped sources == `incoming_count`, which §C.2a already tied to `getBacklinks`); outgoing `edge-count mismatch=0` (== oracle == `outgoing_count`). **Proven: the per-note query equals today's getBacklinks/getOutgoingLinks input byte-for-byte.** (A rehearsal-assertion bug — comparing distinct-targets to `outgoing_count`=COUNT(*) edges — was caught + fixed; the QUERY was always correct.)
- **Boss go/no-go: GO for Step 2.**

## §C.2c-2 — frontend swap behind `perNoteLinkQueries` (BUILT; pending Boss test)
- New `appSettings.perNoteLinkQueries` (default **true** for the test; old array path intact behind `false` for rollback until §C.2c-4).
- **When ON:** the sidebar `$effect` (still 500 ms-debounced, off keystroke) fetches `get_backlink_rows`/`get_outgoing_rows` for the active note and feeds the UNCHANGED `getBacklinks`/`getOutgoingLinks` sort/dedupe/tier logic (proven == array path by §C.2c-1). `ensureFullLinks` is a **no-op** → the 234k array NEVER loads (kills the freeze). The editor ×N `linkTraversalMap` derives from the open note's `activeOutgoingRows` (no array). `applyConfidence/ArchiveLocally` re-fetch via a nonce (DB already written; no array to patch). Panels show `panelLoading` (per-note in-flight) instead of `!linksReady`. Stale-guard: a tab change during the await discards the in-flight result (`sidebarTab?.path === tab.path`).
- **When OFF:** the §C.2b behavior is fully intact (rollback path).
- svelte-check **0 errors**; frontend re-embedded (`get_backlink_rows` in `build/`); binary mtime 2026-06-17 11:56. Editor-Surface Gate: READ-path only (no content/save/lifecycle).
- **Boss test §C.2c-2:** Steps 1–3 + 5 **PASS** (freeze gone, rows/counts identical incl. aliased, confidence/archive reflects). **Step 4 FAIL:** switching notes lags "until backlinks populate."

## §C.2c-2b → 2c — the switch-lag saga (a wrong guess, corrected)
- **§C.2c-2b (WRONG GUESS, `c22b2284`):** I diagnosed the switch-lag as the 500 ms TYPING debounce and moved the per-note fetch to a dedicated TAB-keyed effect that fired **immediately** (no debounce). **Boss re-test: WORSE** — "big lags / couldn't work / non-responsive" = a main-thread FREEZE, not a delay. **Mechanism (read off the regression):** removing the debounce removed the *coalescing* — every switch fired fetch + getBacklinks + `activeOutgoingRows`(→ editor re-decorate) with no batching, so a burst piled up and locked the thread. (Confirmed by the asymmetry: §C.2c-2 with a debounce = delay-not-freeze; 0 ms debounce = freeze. The debounce was *preventing* the freeze. Also: opening a single hub (Step 1) PASSED, so a single large render is NOT the freeze — the pile-up is.)
- **§C.2c-2c (FIX):** keep the architecturally-correct TAB-keyed fetch (typing never re-fetches) but **coalesce with a ~180 ms debounce** (via the `$effect` cleanup) — a burst of switches collapses to ONE fetch (the latest note); below the perceptual threshold for a single switch. Restores §C.2c-2's no-freeze property while removing its 500 ms single-switch delay. svelte-check 0; binary 12:41.
- **Lesson (logged):** I guessed the cause twice instead of reading the mechanism off the change. The tell was there: §C.2c-2 = delay, §C.2c-2b = freeze ⇒ the debounce was the lever. Reproduce-First / measure-don't-guess.
- **Boss re-test §C.2c-2c:** Step B (typing) PASS; Steps A/C still lag — "fighting for memory", scroll not smooth on hubs. Boss: **"Enough patching."**

## §C.2c-3 — STOP-AND-SME: the sound solution, built as ONE unit (pending Boss test)
- **Stop-On-Correction honored.** Three switch-timing patches on one symptom = the LL-014 three-strike / Solve-the-Class violation. Stopped; per Boss, dispatched **4 parallel SME agents** (diagnostic / virtualization design / WA#5 / correctness audit). Findings: `docs/MIG-079-C2c-SME-Findings-and-Solution.md` (commit `cee369d5`).
- **Root cause (SME-1):** the hub churn is the **un-virtualized RENDER** — ~55k DOM nodes + thousands of reactive `LinkTypePill` subscribers built+torn-down per switch — NOT the fetch timing the patches chased. Second: the NSC summary `$effect` fetched for ALL rows. **WA#5 (SME-3):** virtualize + visible-only detail + coalesce is THE proven pattern (Roam/Logseq/Obsidian froze precisely from NOT windowing — 10–60 s, 2 GB spikes). **Audit (SME-4):** per-note queries sound; 2 items (split-pane ×N P1, status-predicate nit) deferred + documented.
- **Built (one unit):**
  1. **Note-summaries toggle** (Boss-requested) — `appSettings.noteSummariesEnabled`, **default FALSE**, in Settings → Panels → Summaries. Gates the NSC headline fetch+render in both panels; when on, the fetch is **head-capped to 120 rows** (was "all rows"). Removes SME-1's second-biggest cost by default.
  2. **Virtualized BacklinksPanel + OutgoingLinksPanel** via the existing `VirtualList` (reuse). Row markup hoisted into ONE shared `{#snippet}` used by both `{#each}` (<50 rows → byte-identical to today) and `<VirtualList>` (≥50). `ROW_*` + `getItemHeight` (annotation/headline branches); `void summaryHeadlines.size` re-derive. Bounded height via `.bl-vlist-wrap/.ol-vlist-wrap { max-height:60vh; display:flex; min-height:0 }` (SME-2 Option A — no host-layout change). Filter+header outside the vlist.
- svelte-check **0 errors**; binary 13:45. Editor-Surface read-path only. i18n: EN keys added; `|| fallback` covers other locales; ×15 rides the translation debt.
- **Deferred (documented, audited):** split-pane ×N chips in non-focused panes (P1 — needs per-open-tab outgoing; edge case, adding it now risks the churn we're avoiding); `status='active'` vs `!='archived'` predicate nit (identical today).
- **Boss test §C.2c-3:** Steps 1–5 PASS — hub scrolls smoothly with its own scrollbar (the windowed list; "we will keep it"), small notes unchanged, summaries OFF by default, toggle works. Two follow-ups raised: (a) the brief open-lag on a hub ("not keeping it"), (b) a toggle for the note-TITLE summary too (the `.e-summary` under the title, distinct from the row headlines).

## §C.2c-3 follow-ups — note-title summary toggle + open-lag fix (Ultracode SME-investigated; pending Boss test)
- **Investigated via a 2-agent Workflow (`wf_23a65146`):** (1) located the note-title summary precisely; (2) diagnosed the open-lag.
- **Note-title summary toggle (BUILT).** Feature = the NSC `headline` rendered in `NotePane.svelte:1107` (`.e-summary`), fed `NoteEditor.svelte` `activeHeadline`→`summaryHeadline`. Ungated before. New `appSettings.noteTitleSummaryEnabled` (**default FALSE** — consistent with the sibling `noteSummariesEnabled` + the leaner editor; Boss told). Gated at the NoteEditor fetch `$effect` (skips the NSC IPC + hides the line when off; re-runs on toggle) — covers main + second screen + all 7 NoteEditor hosts in ONE guard (the agent confirmed SecondScreen mounts NoteEditor, no separate wiring). Toggle in Settings → Editor → "Note title summary". i18n EN + `|| fallback`.
- **Open-lag fix (BUILT).** Diagnosis: the CPU work (~15–35 ms getBacklinks+IPC+VirtualList-mount over 5,358 rows) is inherent, but the **dominant *felt* lag was the flat 180 ms coalesce** (`+layout.svelte`) — pure dead wait before any fetch on a single open. Fix = **leading-edge coalesce**: a settled single open (no fetch in the last 180 ms) fires near-immediately (`delay=0`); a burst falls back to the 180 ms trailing debounce (preserves the §C.2c-2b rapid-switch guard; the stale-guard discards any early fire whose note was switched away from). Removes ~180 ms of felt lag on the dominant single-open case, zero correctness risk. (Deeper getBacklinks-in-Rust pre-sort is a documented future option, not done — inherent + below threshold once the 180 ms wait is gone.)
- svelte-check **0**; binary 17:16. Editor-Surface read-path only.
- **Boss test (follow-ups): ALL PASS** — hub opens snappier; note-title summary off by default + the Settings→Editor toggle works. One last catch: toggling the backlink summaries OFF didn't clear the already-shown ones → **fixed** (`44acd79d` — clear the `summaryHeadlines` map on toggle-off; both panels). Boss: **Pass.**

## SESSION CLOSE (full PCS) — 2026-06-17
- **Whole MIG-079 §C.2c arc SHIPPED + Boss-validated.** Commits `6a4d5e20`→`44acd79d` on `main`. Boot-freeze + panel-thrashing fixed; per-note queries + virtualization + two summary toggles (default OFF).
- **The headline finding:** §C.2b's boot win was partial — the real remaining boot bottleneck is the **Sky read** (`cache_boot_snapshot_sky` over 233,995 `sky_links`), NOT links. That's **§C.2d** (defer the Sky read off boot, same model) — the next session's task.
- **PCS done:** orientation **v2.88** (NEW file), this session log, **MoCh** `docs/MoCh/MoCh-2026-06-17-0930.md`, handover `lab/reports/HANDOVER-2026-06-17-c2c-done.md`, `lab/reports/NEXT-SESSION-PROMPT.md` (points at §C.2d), User Manual (the two summary toggles + panel-perf note, EN; ×15 rides the debt).
- **Deferred (documented):** split-pane ×N chips (P1), the `status` predicate nit, the §C.2b cleanup (retire `cache_full_links`/`ensureFullLinks`/`idx_link_boot`), the ×15 i18n for the two toggles.

---

# §C.2d — Defer the Sky read off the boot critical path (the remaining boot win)

> **Function in hand:** the boot Sky read — `cache_boot_snapshot_sky` reading the 233,995-row `sky_links` table at boot. Full `/migration` (Rust↔Svelte boot contract). Continues from the §C.2c close handover.

## Measured root cause (the pivot grounding — measure-don't-guess)
Boot-perf history (active universe), post-§C.2c boots [16]–[21]: `hydrated ≈ 1.1 s` (good), but `graph_ready ≈ 11 s` of which `queue ≈ 10 s` — the graph IPC body is now trivial (`read_links=None`, `read_tags≈80 ms`). The IPC arrival trace (cold boot `mqfizrd9`) shows the smoking gun: `cache_boot_snapshot_sky` fires at +879 ms, then an **11,374 ms gap with NO other IPC dispatched** — a synchronous command monopolising the single IPC thread. Warm boot (`mqfj19sm`): the same sky read is **250 ms** → the 11.4 s is purely cold disk reads of 234k `sky_links`. The twin of the `note_links` read §C.2b deferred.

## Architect (Phase 1) — `docs/MIG-079-C2d-Architect-Defer-Sky-Read.md`
4 parallel mapping agents (Rust sky path / frontend consumers / render+cross-window / WA#5 prior-art). Findings:
- `cache_boot_snapshot_sky` was `#[tauri::command]` (sync); `constellation_map_universe` is `(async)` — the §9.1 precedent. Only the frontend boot + tests consume the sky read; `sky_*` is fully write-time-maintained (triggers) → Rule 8 already satisfied (defer the READ, not the maintenance). The read is a full-table scan; a covering index can't help the cold read (I/O-bound).
- Every sky consumer (Sky View/CNS/Lens/Sight/WiW/LocalSky/ExpressionForge) is already behind a visibility gate; `skyNodePathSet` degrades permissively at boot. No `skyEverOpened` existed. `ensureFullLinks` is a no-op under `perNoteLinkQueries` → the legacy `buildSkyData` fallback is already dead under defaults.
- **Render primitive: Option C (nodes-now/edges-later) RULED OUT** — the d3-force/PIXI engine needs all edges at init (no streaming/LOD). Edges + nodes load together on open.
- **Second screen independent** (builds its own sky) → no cross-window constraint.
- **WA#5:** defer-to-open is the STANDARD (Obsidian v1.7.2 DeferredView; lazy-bootstrap); eager-at-boot is the documented anti-pattern (Logseq/Obsidian freeze on large vaults). Refinement: after-idle background warm-up so first open is warm.

## Plan (Phase 2) — `docs/MIG-079-C2d-Plan.md` — Boss-approved **Option B+**
Boss chose **Lazy + background warm-up** (over pure-lazy). Steps: §C.2d-1 async-ify; §C.2d-2 defer-to-open + `_skyEpoch` guard + universe-switch reset + after-idle warm-up; §C.2d-3 `/simplify` + docs. Architect+Plan committed `41781d71`.

## §C.2d-1 — async-ify (SHIPPED, `23fdd45f`)
`cache.rs:788` `#[tauri::command]` → `#[tauri::command(async)]`. Body unchanged. `cargo check` clean. Foundation for the on-open load + warm-up (both must be async to not freeze the app).

## §C.2d-2 — defer-to-open + warm-up (BUILT; pending Boss test)
`+layout.svelte`: new `skyEverOpened`/`skyReady`/`_skyEpoch`/`_skyGeneration`/`_skyPromise`; `ensureSky(force?)` memoised async loader; open-trap `$effect` (flips `skyEverOpened` when any sky surface visible); load-trigger `$effect` (`if (skyEverOpened && !skyReady) void ensureSky()`); `loadGraph` boot sky kick-off + assignment REMOVED (graphReady fires on graph payload only) + after-idle warm-up `schedule(() => ensureSky())`; `handleUniverseSwitch` resets sky state + bumps `_skyEpoch`; `federation:ready` + post-init re-fetches routed through `ensureSky(true)` (only if `skyEverOpened`); Sky View spinner gates on `!skyReady` (reuses existing `layout.skyViewLoading` — **zero new i18n**); unused `buildSkyData` import removed. svelte-check **0 errors / 315 pre-existing warnings**.

## Audit (Phase 4) — 4 parallel agents on the diff, BEFORE Boss test
- **Invariants:** all 8 PASS. **Migration-path:** all 5 scenarios PASS (is_ready=false retry correctly armed). **Editor-Surface Gate:** read-path only (zero content/save/lifecycle).
- **P1 found + FIXED (adversarial):** double-write race — a `federation:ready` `ensureSky(true)` nulls the memo + starts a 2nd same-epoch read while the warm-up read is in flight; a slow parent-only warm-up could resolve last and clobber fresh federated data. Fix: `_skyGeneration` counter — force bumps it, each load captures it at entry + discards if superseded (epoch guarded universe switches only; generation guards same-epoch double-loads). Catch branch also generation-guarded.
- **P2 found + FIXED (drift):** the v2 Lens (`toggleLens`) computed clusters on EMPTY sky if opened during the cold warm-up window. Fix: `if (!skyReady) await ensureSky();` before the cluster compute (memoised; instant once warm). svelte-check still 0 errors after both fixes.

## Claude-side verification + commits
svelte-check **0 errors / 315 pre-existing warnings**; `cargo build --release` clean (1m 46s); `cargo test --lib` **936 passed / 0 failed / 7 ignored** (first run hit a transient Windows LNK1104 file-lock on the test exe; retry green). Binary mtime 2026-06-17 18:46. Commits: Architect+Plan `41781d71`, §C.2d-1 `23fdd45f`, §C.2d-2 `cff8f827`.

## ⚑ Boss Stage-1 result — MEASURED WIN (2026-06-17 ~15:04, cold boot via PC restart)
Boot-perf history boot **[22]** (`mqi7brj1`, cold after PC restart) vs the prior cold boots [20]/[21]:
- **graph_ready: ~11,000–11,300 ms → 1,555 ms** (~7× reduction).
- **cache_snapshot_graph_queue_ms: ~9,500–9,700 ms → 178 ms** (the sky no longer monopolises the IPC thread).
- `read_tags=7 ms`; `hydrated=1,316 ms` (unchanged — responsiveness preserved).
- **Boss observation:** "Sky View opened instantly" → the after-idle warm-up loaded the graph on the worker thread before the open, no freeze. Option B+ confirmed working.
- **Stage 1 PASS.** Stage 2 (Editor-Surface Gate + Lens P2 + on-open spinner path + universe-switch stale-guard) pending.

## ⚑ Boss Stage-2 result — ALL PASS (2026-06-17)
Boss ran Stage 2 on the validated binary: **all pass.** Notes untouched (Editor-Surface Gate — Focus round-trip + tab switch, body intact); Lens shows real clusters (P2 fix confirmed); **opening Sky View in TWO universes opened instantly** (the per-universe warm-up + the `_skyEpoch` stale-guard both confirmed — each universe shows its own graph). **§C.2d is fully Boss-validated.**

## §C.2d-3 — close-out (/simplify + SO #6 + manual)
- **/simplify (4 cleanup agents on the diff):** code is CLEAN. Reuse — no action (`ensureSky`/`_skyEpoch`/open-trap intentionally mirror `ensureFullLinks`/`_linksEpoch`/`mapEverOpened`, idiomatic per-domain). Efficiency — no concerns (warm-up coalesces via `_skyPromise`; effects tightly gated; closure captures 2 ints; warm-up off the critical path). Altitude — correct depth (frontend deferral + async command); `graphReady`/`skyReady` cleanly separated. **3 optional micro-refactors DEFERRED** (not applied — would change the compiled bundle and invalidate the just-validated binary; secure-don't-muddle): (a) merge the open-trap + load-trigger into one `$effect` (marginal; but `skyEverOpened` IS read by the two federation guards, so it must stay a flag); (b) `skyReady`→`$derived` (declined — keeping a plain flag mirrors `linksReady`; the derived form is riskier); (c) extract a `refreshSkyIfOpen()` helper for the two federation re-route sites (trivial DRY; defer). Allocate a PJ at the next Pending Jobs bump if desired.
- **SO #6:** orientation **v2.89** (NEW file, preserving v2.88) — preamble + §9.2 boot primitives + §15.2 boot pipeline + §17 (the `cache_boot_snapshot_sky`-bypass "mystery" resolved → now deferred by design). **User Manual** — Sky View "loads on open, not at startup" note (EN; ×15 rides the translation debt).
- **State:** §C.2d-1 `23fdd45f`, §C.2d-2 `cff8f827`, docs close-out `2f3c8aa0` (pushed). Validated binary unchanged (docs-only close-out).

## Deferred-cleanups pass (Boss chose "§C.2c/§C.2d cleanups") — SO #8 cross-checked first
Cross-checked each handover-deferred item against the CURRENT code before touching (SO #8):
- **i18n ×15 — DONE.** The two summary-toggle string sets existed in only **1/15 locales** (EN). Added proper native translations of all 5 strings (`settings.panels.summariesHeading/noteSummaries/noteSummariesDesc`, `settings.editor.noteTitleSummary/noteTitleSummaryDesc`) to the other 14 (ar/de/es/fa/fr/he/hi/ja/ko/pt/ru/tr/ur/zh), using each locale's existing Backlinks/Outgoing house terms. Surgical (JSON round-trips losslessly at tab-indent); all 15 valid; diff = additions only. **All 15 locales now complete.**
- **§C.2b cleanup — DEFERRED (kept as a documented PJ), with reason.** Cross-check shows `cache_full_links`/`ensureFullLinks`/`linksReady`/`idx_link_boot`/`perNoteLinkQueries` still referenced **~30× across Rust+Svelte**; retiring them deletes the **§C.2c rollback path that is only ~1 day old**. Per secure-don't-muddle, removing a freshly-shipped safety flag is not hygiene — keep the flag as cheap insurance until per-note queries are proven over more usage. (Also crosses Rust↔Svelte → /migration-grade, not a quick cleanup.)
- **status-predicate nit — DEFERRED.** Confirmed `!= 'archived'` (per-note, cache.rs:477/505) vs `= 'active'` (legacy, 1218/2113); **identical on the all-active DB** (zero observable change). Explicitly a "later, if a 3rd lifecycle state is added" alignment — not worth a rebuild today.
- **split-pane ×N chips — DEFERRED.** That's a *feature* completion (P1), not a cleanup; needs its own build+test.
- **§C.2d micro-refactors — DEFERRED.** Marginal (merge 2 effects — but `skyEverOpened` is read by the federation guards, so it must stay a flag; `skyReady`→derived declined for `linksReady` parity; extract `refreshSkyIfOpen()` — trivial DRY). Not worth touching the validated binary.

---

# §D — Phase-by-phase bring-up (Boss: "Start §D bring-up now")

> **Function in hand (D-1):** the **Calendar** panel's day-click → daily-note open (`CalendarPanel.svelte` + `+layout.svelte:7261` `onDayClick` + `get_daily_note_path`). Concept paper `docs/concept-papers/21-Calendar.md`; confirmed defect §7.E.1.

§D = re-enable each function against its concept-paper §10 checklist; the confirmed defects (§7.E) are fixed in their phase. Phases 1–3 (core / search / backlinks-graph-tags) are effectively done (the §C.2b/c/d work). Started with the highest-value confirmed defect.

## D-1 — Calendar day-click bug (BUILT; pending Boss test)
- **Reproduce-First (deterministic logic bug, traced end-to-end):** `CalendarPanel.svelte:61` emits `dateStr` as `YYYY-MM-DD` → `+layout.svelte:7261 onDayClick(dateStr)` → the `invoke('get_daily_note_path', …)` **dropped `dateStr`** → `libraries.rs:4318` used `chrono::Local::now()` for BOTH the filename and the `date:` frontmatter, taking no date param. So clicking ANY day created/opened **today's** note. Boss recipe: click a non-today day → today's note opens.
- **WA#4 impact (all callers checked):** only 3 primary-location callers — `store.ts:3124` wrapper, `+layout.svelte:4007` (`handleOpenDailyNote` — the "open TODAY" command, must stay today), `+layout.svelte:7266` (Calendar). (The nested `ConstellationEditor/` copies are ignored per WA#2.)
- **Fix (backward-compatible optional param):** `get_daily_note_path` gains `date: Option<String>` — `Some(YYYY-MM-DD)` → parse `NaiveDate`, format a midnight `NaiveDateTime` for the filename + `%Y-%m-%d` for the frontmatter; `None` → `Local::now()` (preserves `handleOpenDailyNote`). `store.ts` wrapper gains optional `date?`; the Calendar `invoke` now passes `date: dateStr`. Same sanctioned create path (`gate_create_exclusive`) — content-integrity model untouched (creates a fresh dated daily note, same as before).
- **Folded-in (same component, §10 multilingual checklist):** the hardcoded English `\`${count} notes\`` day tooltip → `$t('calendarPanel.notesCount', { count })` + added the missing `tasksCount` tooltip (both keys already in all 15 locales; `.toLocaleString()` for locale digits). svelte-check caught a `number`→`string` `$t`-values typing error → fixed before build (Test-Before-Commit).
- **Not /migration** (local bug fix per the Migration Rule; covered under the approved MIG-079 §D umbrella). svelte-check **0 errors**; `cargo check` clean. Binary rebuild in progress.
- **Remaining Calendar §10 items (documented, not in this fix):** right-click menu via shared `<ContextMenu>` (MIG-077 "genuinely missing"); Rule-8 WTD for the dot-scan (`scan_library_note_dates`/`scan_library_tasks` re-walk on panel open → persist per-date counts).

## ⚑ STOP-ON-CORRECTION + redirect → MIG-080 (SO #5 state-of-standing) — 2026-06-17
- **Boss correction:** the Calendar isn't a right-sidebar item. Boss screenshot: clicking the Calendar tab shows "No note selected" (it's wrongly in `NOTE_SCOPED_TABS`, `+layout.svelte:389`). Boss directed me to **read the past-three-days MoCh**.
- **What the MoCh sweep surfaced (the decision I MISSED):** `docs/Right-Sidebar-Note-Context-Design-Decision.md` (2026-06-16, commit `bfa66f0a`) — the right sidebar is **note-context-only**; universe functions relocate; **"defects fixed by the moves (NOT patched in place)"**; sequenced as its **own /migration AFTER §C.2/§C.3** (now done → unblocked). My §D-1 patched the Calendar **in place** — against this decision. Root miss: I read concept-paper-21 (stale on placement) + the kickoff handover's defect list, but did NOT cross-check the governing decision doc (SO #8 / Predecessor-Lookup miss). The kickoff handover + concept-paper-21 + `MIG-079-Plan.md` §D are **superseded** for Calendar/Tasks/Review (fixed by relocation, not in-phase patching). **Doc-drift to fix:** mark those stale.
- **§D-1 commit `a4152934` (pushed):** left as-is for now (not broken; the `get_daily_note_path` date param is reusable groundwork). Its disposition folds into the MIG-080 Architect (Boss steer: "first familiarize … then redesign").
- **MIG-080 OPENED — Right Sidebar → Note-Context-Only.** Boss-confirmed disposition (familiarization confirmed against code + decision doc):
  - **STAY (note-scoped):** Properties(+Outline), Backlinks(+Outgoing), Tags-"this note", Sky-local(star), Provenance, 360.3D.
  - **SPLIT (note version stays in right rail + universe version relocates) — ALL FOUR in this migration:** Knowledge Health (note tensions/health, distinct from 360.3D | universe → Dashboard); Review Pulse (this note's review status | universe queue → full-page reviewer, fixes dead `record_note_visit`); Source Review (this note's sources | universe Cataloger → left); **Tasks (Boss refinement 2026-06-17: CONTEXTUAL — surface the OPEN note's own task list in the right rail | universe agenda → left with the Calendar).**
  - **RELOCATE out entirely:** Calendar → left-sidebar launcher → daily note (fixes wrong-date + wrong-library defects at the new home); Tags "All tags" → Search Hub facet + Dashboard (reuses §C.1 `tag_counts`).
  - **Process:** four-phase `/migration` (crosses Svelte↔Rust + settings-schema `panelPlacements`/`NOTE_SCOPED_TABS`; 4 split-redesigns; 2 relocations; 2 defect fixes by relocation). Architect mapping running (4 agents: Calendar+Tasks / the 3 splits / Tags+SearchHub+Dashboard / rail-mechanics+left-rail).
- **Architect + Plan SHIPPED + Boss-approved.** `docs/MIG-080-Architect-Right-Sidebar-Note-Context.md` (`57d27877`) + `docs/MIG-080-Plan.md` (`8a0194bc`, pushed). 4-agent territory map → disposition confirmed; lots of reuse found (Tasks `scanNoteTasks` already note-scoped; KH Dashboard + CatalogerView + Dashboard tags section + left Daily-Note launcher + `tag_counts` + `#tag`→SearchHub all exist). Build = 3 small per-note IPCs + 1 new full-page reviewer. Plan = §A–§G, Boss approved **"cascade as far as we can"** (stop at each phase's test). Defaults locked: All-tags→Dashboard-only, Calendar→launcher, keep §D-1.

## MIG-080 §A — Calendar → left launcher (BUILT; pending Boss test)
**Function in hand:** the Calendar's right-rail tab → a left daily-note launcher.
- **Removed the right-rail Calendar tab** (`+layout.svelte` tab button + render branch) + dropped `'calendar'` from `NOTE_SCOPED_TABS` (`:389`) + removed the now-unused `CalendarPanel` import + removed the `calendar` row from the Settings → Panels placement list.
- **Left launcher:** the dock Daily-Note button (`:5496`) now opens a small popover — **"Today"** (opens today's daily note) + a native **date input** (pick any day → that day's daily note). Refactored `handleOpenDailyNote` → `openDailyNote(dateStr?)` (threads `dateStr` to `getDailyNotePath`'s §D-1 date param; `None`→today). Kept `handleOpenDailyNote` as a today-wrapper for the command-palette caller (`:2063`). Popover = anchored `position:absolute` (inset-inline-start, RTL-safe) + a transparent fixed backdrop for click-outside.
- **Wrong-library defect dissolved:** the launcher always opens in the daily-note home (`libraries[0]`) for the picked date — no cross-library dot ambiguity (the launcher isn't dot-driven). (The Boss-flagged "No note selected" is gone — Calendar is no longer a note-scoped right-rail tab.)
- **Migration-path (workspace restore):** set `tabVisible.calendar = false` + dropped `'calendar'` from the safety-`$effect` order list, so a stale saved `rightSidebarTab='calendar'` resets to the first visible tab (no blank rail). `panelPlacements.calendar` becomes a harmless vestigial key.
- **Scope note:** the dead calendar dot-scan `$effect` + `calendarNoteDates/TaskDates` state are LEFT in place (gated on the now-removed tab → never fires; zero cost) — §C repurposes that `scanLibraryTasks`+per-date grouping for the left Tasks agenda. The **inspector360 Settings-UI bug** (missing from the placement list) is **deferred to §G** (needs ×15 i18n; out of §A's focus).
- svelte-check **0 errors**; reused existing i18n only (`calendarPanel.today`, `ribbon.dailyNote`, `common.close`) — **zero new i18n**. No Rust change (date param was §D-1). Binary rebuild in progress.
- **§A Boss-test: Steps 1–5 ALL PASS.** Boss observations: (1) the other universe tabs (Tags-All 19,548 / Review 59 / Source 7,182) are still in the rail — confirmed = the queued §B/§D/§F targets (not yet built). (2) **Wants a full Calendar PAGE view** (month grid + highlighted events), SEPARATE from the Daily-Note launcher — **both on the left dock**. (3) Source Review → open-note only = confirmed §D.

## MIG-080 §A.2 — full-page Calendar view (Boss-requested; BUILT; pending Boss test)
**Boss directive:** "separate the Daily Note function from the Calendar page function; I want them both on the left dock." So §A's Daily-Note launcher stays; a NEW distinct **Calendar** dock button opens a full-page month view with highlighted note/task events.
- Re-imported `CalendarPanel`; new `showCalendarPage` + `calendarPageEverOpened` (LL-022 lazy-mount); added to `fullPageActive` + the content-hidden list.
- New **Calendar dock button** (calendar-with-event-dots icon, distinct from the Daily-Note button), gated `enabledFeatures.dailyNotes`; onclick toggles the page + closes the other full-page overlays. A **reverse-mutex `$effect`** closes the Calendar page if another full-page overlay opens (settles; no loop) — covers the direction the per-button mutexes don't.
- **Full-page overlay** (mirrors the map/cataloger `*-overlay` + `*-visible` lazy pattern): renders `CalendarPanel` scaled to the **full center zone** (`:global` overrides — 84px day cells, bigger nav/weekday/dots, max-width 1100px — per the Style-Setter "don't cram" rule). Day-click → close the page + `openDailyNote(dateStr)`.
- Repurposed the (previously dead) calendar dot-scan `$effect` to gate on `showCalendarPage` → populates the highlighted events. `handleUniverseSwitch` resets the page + EverOpened flag.
- svelte-check **0 errors**; zero new i18n (reused `panels.calendar`, `common.close`). Binary rebuild in progress.
- **§A.2 Boss-test: Steps 1–6 ALL PASS.** Three follow-ups raised → (#1) a bug, (#2/#3) → MIG-081.

## §A.2 follow-up #1 — file-tree didn't show the daily note (BUG; FIXED, pending re-test)
Boss: opening/creating a daily note via the launcher or the Calendar page didn't appear in the left file tree. Cause: `get_daily_note_path`→`gate_create_exclusive` creates the file in Rust, but `openDailyNote` never refreshed the frontend tree (the sanctioned `handleNewNote` path calls `refreshLibraryTree`; the daily-note path didn't). **Fix:** after `openNoteTab`, resolve the note's library via `$libraryStats.find(v => path.startsWith(v.path))`, ensure it's in `expandedLibraries`, and `await refreshLibraryTree(stat.library_id)`. Covers BOTH the launcher and the page (both call `openDailyNote`). svelte-check 0; binary 13:20.

## MIG-081 OPENED — Cultural Calendars + Calendar Settings (Boss: "build #2+#3 now as a /migration")
Boss follow-ups #2 (Calendar settings don't exist) + #3 (Constellation is multilingual → provide cultures' own calendars, integrated or standalone). **WA#5 research done** (a01e938727b623301): the proven approach —
- **Intl** (`-u-ca-`) for display + **Temporal API (polyfilled ~20KB, lazy into the Calendar chunk)** for non-Gregorian grid math (`Date` can't do Hijri/Hebrew/Persian month lengths).
- **UX:** Primary calendar (switches grid = standalone) + optional Secondary (shown alongside = integrated); per-locale defaults (ar→`islamic-umalqura`, fa→`persian`, he→`hebrew`, …) overridable.
- **Storage:** daily-note filename stays **Gregorian ISO `YYYY-MM-DD`** always (File-Over-App/sync); cultural date is display-only (optional `hijri:` frontmatter). Reuses §D-1.
- **Hijri default `islamic-umalqura`** (NOT bare `islamic`). Numerals: `nu-arab` (Arabic) vs `nu-arabext` (Persian) — different digits. Grid direction from UI locale via `detectDir()`.
- **Scope:** ship Hijri/Persian/Hebrew/Indian/Buddhist first; **defer Chinese/Korean** (lunisolar leap-months don't fit a fixed grid).
- **#2 Calendar settings** is the host for #3's selector + the existing daily-note format/folder/template + week-start.
- Proper four-phase /migration (multilingual + write-path-adjacent + new dep). Sources in the research result. Right-rail cascade §B–§F resumes after MIG-081 (or interleaved per Boss).

### MIG-081 Architect — Boss decisions + Eisa's Hijri engine (`ea9969bb` + updates)
- **Boss answered the 4 design Qs (2026-06-17):** (1) ship the **4** systems (Gregorian/Hijri/Solar-Hijri/Hebrew); defer Indian/Buddhist/Chinese. (2) **primary switch + optional secondary** (standalone + integrated). (3) **default Gregorian for everyone** (NOT per-locale auto-seed; cultural calendar is opt-in). (4) **write a cultural-date frontmatter field** (`hijri:`/`jalali:`/`hebrew:`) into new daily notes (non-authoritative; ISO filename stays the key).
- **★ Boss directive — use Eisa's OWN Hijri calendar (`github.com/eisaShamsi/hijri-calendar`).** Studied the repo: `hijri.js` (250 KB IIFE `const HijriCalendar`) is an **astronomical** Hijri engine (default `currentMode='astronomical'`; Meeus `newMoonJDE` → real new-moon month starts via JDN) + user **moon-sighting corrections** — more accurate than Intl `islamic-umalqura`. Clean API: `gregorianToHijri`/`hijriToGregorian`/`daysInMonth`/`monthName`(AR/EN)/`isSacredMonth`/`todayHijri` + Islamic-events/eclipse/Durur enrichment. **DOM-free except 8 graceful `localStorage` calls** → loads in the Tauri WebView. No LICENSE but Eisa's own repo (same owner) → no blocker. The repo is the **ديرة الدرور** heritage calendar (Durur/anwa/zodiac/seasons + Hijri), ref: Sheikh Zayed Grand Mosque Center book.
- **Design lock:** Hijri → **vendor `hijri.js`** as `src/lib/calendar/hijri.js` (lazy, ES-export adapter, source-commit pinned header; future npm pkg by Eisa replaces the vendor). Persian/Hebrew → Intl + Temporal polyfill (lazy, Calendar chunk only). Gregorian → native. Filename stays Gregorian ISO; §D-1 `get_daily_note_path` gains an optional cultural-date string to seed the frontmatter field (small Rust signature extension).
- Architect doc updated with all of the above (§2/§3/§5/§10). **Plan + build (the large part) NOT yet done.**
