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
- **The dominant note-open indexing freeze** — separate reproduce-first pass.
- MIG-088 Phases 6–10; Arabic callout End/Home caret known-issue.
