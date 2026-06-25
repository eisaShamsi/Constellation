# SESSION LOG — 2026-06-25 (MIG-086 §D Boss-test rounds)

(Continues 2026-06-24 §D build, commit `eb0cc280`. New calendar day; same §D work.)

## §D Boss test — Round 1 (Stage 1: NotePane Backlinks tab, outbound)
- **Verdict: PASS.** The outbound suggest + one-click typed link works from the Backlinks tab.
- **Boss remark — "still taking time to link":** diagnosed (Reproduce-First, from code) — NOT a §D defect.
  The connect WRITE is fast: `addLinkToNote` open branch = composeNoteModel (sync) → saveTabContent
  (`writeNote` disk write, then a FIRE-AND-FORGET reindex, line 755) → reloadTabsFromDisk (one file read).
  The frontmatter link (source of truth) lands instantly; the suggestion is removed optimistically. The
  residual latency is the **reindex-gated DERIVED views** (Outgoing list, orphan-status) refreshing only
  after the background reindex completes — and on a link-dense note that reindex is the **PJ-066 sky-trigger
  storm** (COUNT(DISTINCT) over ~234k rows per edge), already Boss-deferred to its own perf migration. Not
  re-fixed here per the §D scope ruling; flagged to Boss for prioritization.
- **Boss finding — type picker truncated at the screen bottom (explicit "Fix it"):** FIXED.
  `LinkTypePicker` clamp measured `r.height` while the base CSS capped it at `max-height: 60vh`, and the
  webview wasn't honoring the `vh` unit — so a long list (8 seeds + associative + the user's CUSTOM types)
  opened taller than the window and the top-clamp under-shifted. Fix: removed the CSS `max-height: 60vh`,
  measure the menu's NATURAL height, then set an explicit px `max-height = window.innerHeight − 16` inline
  and clamp the top off the capped height → the menu always fits and scrolls in place. (Benefits the
  Reviewer §C picker too — shared component.) svelte-check 0 errors; frontend + binary rebuilt.

**Next:** Boss re-verify the picker (no truncation; scrolls if long) → Stage 2 (360 Inspector + Health,
inbound) → Stage 3 (Sky View node menu, outbound). Then §E.

## §D Boss test — Round 2
- **Picker re-verify: FAILED** — still truncated at the screen bottom. Round-1 fix (measure natural height
  + JS px max-height + clamp top off the capped height) was insufficient: the measured height was stale
  (the custom-type list / layout settled after the first measure), so the top under-shifted. **Fix v2
  (bulletproof):** cap `max-height = window.innerHeight − top − pad`, so the menu's bottom is ALWAYS
  `vh − pad` regardless of whether the measured height is accurate — it scrolls in place. (Also forced a
  clean frontend re-embed: `touch src-tauri/src/lib.rs` so `generate_context!` re-expands — guards against a
  stale embedded `build/`.)
- **Stage 2.2a (360° Inspector, inbound): PASS.** Connect worked.
- **Stage 2.2b (Health/TensionPanel, inbound): PASS.** Connect worked.
- **Boss finding (2a + 2b) — ~1-minute app FREEZE + background thrashing after each connect:** confirmed
  **PJ-066**. The connect fires a reindex of the source note; on a link-dense note the sky-trigger storm
  holds the SQLite write lock for ~1–2 min, so every frontend IPC blocks → the UI appears frozen. NOT a
  §D-introduced defect (any reindex triggers it), but §D's connect is a frequent trigger, so it now hurts
  usability acutely. **Recommend pulling PJ-066 forward** (its own /migration — composite index / batched
  triggers / diff-edges); awaiting Boss ruling. The frontmatter link itself still writes instantly.

**Next:** Boss re-verify picker (v2) → Stage 3 (Sky View node menu, outbound) → PJ-066 ruling → §E.

## §D Boss test — Round 3 — ALL PASS → §D COMPLETE (Boss-validated)
- **Picker re-verify (v2): PASS** (no truncation; scrolls in place).
- **Stage 3 (Sky View per-node menu, outbound): PASS.**
- **§D fully validated:** Backlinks tab (outbound) · 360° Inspector ×2 (inbound) · Health/TensionPanel
  (inbound) · Sky View node menu (outbound) · LinkTypePicker viewport clamp. Direction split (diagnostic =
  inbound / general = outbound) Boss-accepted.
- Commits this §D arc: `eb0cc280` (§D wiring + 4 review fixes), `f4e1f3cd` (picker v1), `f5492543` (picker
  v2 bulletproof + re-embed). Local on `main`; not yet pushed (session-close PCS or on request).
- **OPEN DECISION (Boss):** sequence PJ-066 (the ~1-min connect freeze — its own /migration) BEFORE §E, or
  §E first. Recommended: PJ-066 next, then §E (don't mark MIG-086 shipped while connecting freezes the app).

---

## PJ-066 — sky-trigger reindex perf migration (Boss chose A+B → near-instant)
**Decision:** PJ-066 BEFORE §E (Boss). Then A+B together (Boss). Docs: docs/PJ-066-Architect-*.md, docs/PJ-066-Plan.md.

### Shipped (committed on main)
- **§A1** `8bd9039b` — rewrote the sky COUNT(DISTINCT) `(target=id OR target IN alias-subq)` disjunction →
  `IN (SELECT id UNION SELECT alias…)`. MEASURED 5,572ms→26ms (215×), 0 mismatches across 18 heavy targets.
  EXPLAIN proved the index was already used → the handover's "composite index" guess would NOT have worked.
- **§B1** `30434318` — recompute_all_sky (links_backfill.rs) + wired into reconcile (the bulk self-heal).
- **§B2+§B3** `41bbf55a` — maintain_sky_after_save (save/delete diff, mirror of MIG-079 §C.2a); only changed
  targets + source recompute. Unit test green.
- **§B4** `65d17041` — dropped the per-edge note_links_sky_stratum/maturity triggers; kept note_meta_sky_*;
  added drop_sky_aggregate_triggers + reconcile-window drop. Full lib suite 981 pass.

### Boss test (Ancient history, 533 links): PASS, **~1–2 min → 30–50 s**.

### Reproduce-First measurement (live-DB copy + Rust timing test; test then reverted)
- **maintain_sky = 1.3 ms** — §B is perfect; the sky cost is GONE.
- The outgoing-trigger guess was WRONG (0.4 s, covering index) — not the residual.
- **Steady-state per-connect backend `index_note` = ~6.1 s** (run2/run3), NOT 30–50 s. Run-1 was 15.9 s =
  a ONE-TIME FTS re-tokenize; the `note_meta_au` FTS trigger ALREADY guards `WHEN body/name changed`, so a
  frontmatter-only connect (body unchanged) SKIPS FTS in steady state. The 30–50 s the Boss saw was
  dominated by the ONE-TIME first-boot reconcile (recompute_all_* over 7,659 notes, ~15 s) + frontend.
- Residual 6.1 s = ~3.8 s note_links DELETE-all+INSERT-all rebuild + ~2.3 s read/parse/upsert of the 88 KB
  file. **diff-edges** (only touch the 1 changed edge) would cut the 3.8 s → backend ~2.3 s, but touches
  the traversal-data-preserving note_links rebuild. Frontend (remount + 533-link panels) unmeasured.
- **Re-framed decision surfaced to Boss:** steady-state is ~6 s (not 30–50 s); the big number was one-time.
  Pursue diff-edges (+frontend) or ship §A+§B and finish MIG-086 §E?

### PJ-066 §C — split-measurement (Boss-approved instrumentation, then reverted) + the real root cause
Boss steer: "don't patch/reinvent — check how others solve it." Research (WA#5): the two universal patterns
are **async/non-blocking indexing** (Lucene/ES) + **incremental/diff indexing** (Obsidian/LightRAG). Then ONE
split-measurement (FE+BE timing → diagnostics.log) on the live universe:
- **PJ066-BE reindex = 46.18 s** (first connect) · **FE addLinkToNote reloadStore = 47.9 s (waiting on it)** ·
  **FE NotePane remount = 8 ms** (frontend INNOCENT). Boss: 2nd connect 7 s, 3rd 5 s.
- **Root cause:** `constellation_search_reindex` is a SYNC Tauri command → runs on the WebView2 UI thread →
  freezes the whole app + blocks the connect's `read_note` for the full reindex (LL-021 class). First connect
  ~46 s = one-time FTS re-tokenize (stale stored body); steady-state ~5–7 s = note_links rebuild + parse.
- **§C plan (docs/PJ-066-Plan.md PART 3):** §C1 — `#[tauri::command(async)]` on the reindex (off the UI
  thread; connect instant; codebase idiom LL-021). §C2 — incremental diff-edges in index_note (don't rebuild
  all 533 links; Obsidian/LightRAG + our MIG-079/PJ-066 §B precedent; safer for traversal data). §C3 (opt) —
  the one-time FTS re-tokenize. Instrumentation reverted; awaiting Boss approval of §C.
