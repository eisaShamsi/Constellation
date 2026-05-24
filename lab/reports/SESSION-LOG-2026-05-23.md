# Session Log — 2026-05-23

**Continuation of the 2026-05-22 marathon session** (see `SESSION-LOG-2026-05-22.md` for the lead-in: MIG-042 + BUG-020 + BUG-021 + BUG-022 + the NSC Core Plug-in design captured + PCS-1 of the design docs at commit `1b64a7c4`). Today's work crossed midnight to ship **MIG-043 Phase 1**.

---

## 1. MIG-043 Phase 1 — NSC Core Plug-in foundation (SHIPPED + Boss-validated)

Phase 1 of the NSC Core Plug-in roadmap (Concept Paper v2.0 §10): the **service foundation + first two surfaces**. All six Architect-Plan steps cascaded in one session per Eisa's "PCS + Orientation > cascade the Build (Steps A-F)" directive.

### Step A — engine + schema (`headline` variant)
- **`NoteSummary` + `NoteSummaryEntry`** grew a `headline: String` field (the latter with `#[serde(default)]` for backward-compat — old frontends that don't know about it deserialize fine).
- **`textrank_top_k`** refactored to **`textrank_pick`** returning a `TextRankPick { top_k_doc_order: Vec<usize>, top_1_by_rank: Option<usize> }` — both from **one** score pass (free; `idx[0]` was already computed). 2 unit tests updated.
- **`first_sentence(&str) -> String`** helper added (UAX#29-based; falls back to trimmed text when no boundary).
- **`summarize_from_parts`**: every branch (frontmatter / callout / extractive / opening / few-sentence) computes `headline` — author authority extends to it (frontmatter + callout headline = first sentence of the author's text).
- **Schema**: `ensure_note_summaries_table` includes `headline TEXT` in the CREATE for fresh DBs + an idempotent `ALTER TABLE … ADD COLUMN headline TEXT` (probe `PRAGMA table_info` first) for existing DBs. Lazy-fill: `get_or_compute_cached` treats a fresh `content_hash` with NULL/empty `headline` as a cache miss → recomputes both summary + headline together. **Backward-compat is hard**: rollback to MIG-040 reads work (extra column ignored), and writes via `INSERT OR REPLACE` simply null the unnamed `headline` column (audit-verified against `780713b6`).
- **3 new `first_sentence` unit tests** added (multi-sentence en, no-boundary fallback, Arabic). **Full NSC test suite: 19/19 pass.**

### Step B — shared frontend summary store
- **NEW `src/lib/nsc/summaryStore.ts`**: `getSummariesFor(notePaths)` cache-first + batched + coalesced (in-flight per-path promises so concurrent callers share one IPC); `getSummaryFor(path)` single-path wrapper; `invalidate(paths)`; `clearAll()`. Lazy-init `library-changed` watcher drops cached entries when their file changes on disk (payload shape `{libraryId, paths}` verified against `watcher.rs:82-88`).
- **`SourceReviewPanel.svelte`** migrated from direct `invoke('nsc_get_summaries_for_notes')` to `getSummariesFor(paths)`. **No behavior change** — same render, same gentle-fill, same i18n. Local `NoteSummaryEntry` type alias (now stale — missed `headline`) deleted in the audit cleanup.

### Step C — search-results wiring (`SearchHub.svelte`)
- `summaryHeadlines: Map<path, string>` `$state` + a single `$effect` watching `allFlatResults` + `advancedGroups` → batched fetch via the store → merge with a `changed` guard to avoid unnecessary re-fires.
- A `.sh-item-headline` line (italic, muted, single-line ellipsis) renders under each result in **all three loop sites** (grouped-advanced, flat-advanced, basic-categorized). The existing snippet is preserved alongside — snippet shows *why* it matched, headline shows what it's *about*.

### Step D — editor-header wiring (`NoteEditor.svelte`)
- `activeHeadline: string` `$state` + an `$effect` on `tab.path` → `getSummaryFor(path)` → set `activeHeadline` (with a stale-promise guard so a tab switch mid-fetch doesn't write the old note's headline).
- The `<NotePane>` is wrapped in a flex `.ne-wrap` with a thin `.ne-summary-band` band above when non-empty (italic, muted, ellipsis, `dir="auto"`, full-text `title` attribute for hover).

### Step E — `/simplify` + 3-agent audit
**All clean.** Three parallel agents (invariants / drift / migration-path):
- **Invariants** — all 8 of MIG-043-ARCHITECT §3 hold (SRP unchanged, cache-first/batched, no hot-path, no boot regression, author authority for headline, additive schema, frontmatter+callout precedence preserved, File-Over-App).
- **Drift** — one HIGH-severity-but-latent: SRP's stale local `NoteSummaryEntry` type missing `headline` (cleaned up inline before commit). All other axes clean: `textrank_top_k` zero remaining refs; no `$effect` loops; watcher payload matches; CSS scoped; backfill writes via `get_or_compute_cached` (auto-inherits `headline`).
- **Migration-path** — all 6 scenarios PASS (fresh DB, existing DB lazy-fill, mid-backfill, rollback both directions verified against `780713b6`, frontend rollback-compat all consumers `?? ''`-guarded, CASCADE-DELETE FK preserved).

### Step F — SO + docs + 15-locale help additions + PCS-2
- **Help files**: English `Note Summaries` help expanded — intro now lists all 3 surfaces (Cataloger / Search results / Editor), and the `## Where summaries appear, and how they fill in` section (renamed from `## When …`) now bullets the 3 surfaces + rewrites the lazy-fill paragraph. **14 other-locale Note Summaries files** translated in parallel by a sub-agent using each locale's established native term (المُصنِّف, 分类器, Klassifikator, etc.). User Manual already defers to the help topic — no UM change.
- **Orientation v2.27** (new file alongside v2.26).
- **Session log** (this file) + MoCh append.

### Validation (Boss live test, end-to-end on "Eisa Cognitive Knowledge")
- **Stage 1 — SRP regression**: Cataloger renders summaries identically to before. **PASS.**
- **Stage 2 — search-results headlines**: faint italic headline appears under each hit, alongside snippet. **PASS.**
- **Stage 3 — editor-header band**: thin band above the editor shows the note's headline. **PASS.**

## 2. Cross-day note

This session began on 2026-05-22 14:00 (NSC plug-in design kickoff) and continued through the build cascade past midnight. Today's binary built at **2026-05-23 17:45**; Boss validation followed. Per the SO daily-log convention, today's ship work is captured here; prior work is in `SESSION-LOG-2026-05-22.md`.

## 3. Open / deferred (UPDATED 2026-05-23 after MIG-044 ship)

- ~~**Phase 2 (MIG-044)** — wire the remaining enabled surfaces.~~ **SHIPPED** — see §4 below.
- **Phase 3 (MIG-045)** — the Universe Digest left-dock view: tiered Library → Folder → 1-line headline, expandable to full summary, recency-sorted, searchable, virtualized, with cUniverse-children federated in. Will go through full `/migration` per the project's discipline.

---

## 4. MIG-044 Phase 2 — NSC Core Plug-in full service reach (SHIPPED)

Per Eisa's directive earlier today ("Proceed through all the remaining phases" + "Both Architect+Build now, PCS each"), Phase 2 cascaded immediately after Phase 1's Boss-validation. **Frontend-only MIG**: zero Rust changes, zero schema deltas, zero new IPC. The four remaining note-displaying surfaces now consume the shared `summaryStore` from Phase 1 and render the same italic-muted-ellipsis headline shape under each row / tooltip.

### Architect doc
- `docs/MIG-044-nsc-coreplugin-phase2-ARCHITECT.md` — wrote first, then cascaded the Build per Eisa's pre-approval. Territory map: Backlinks, Outgoing, Index, Sky View. Map disabled (skip), wikilink-preview doesn't exist (skip), LocalSkyView excluded per `feedback_display_not_domain`.

### Step A — Backlinks panel (`src/lib/components/BacklinksPanel.svelte`)
- `summaryHeadlines: Map<path, string>` `$state` + `$effect` over `filteredBacklinks` + `filteredUnlinked` → batched fetch via `getSummariesFor`. `changed`-guard merge. `.bl-headline` (italic, muted, single-line ellipsis) renders under BOTH the linked-mentions row AND the unlinked-mentions row.

### Step B — Outgoing links panel (`src/lib/components/OutgoingLinksPanel.svelte`)
- Same shape — BUT `NoteLink` has no `target_path` (only the wikilink-string). The panel resolves visible targets to paths in parallel via `Promise.all(resolveWikilinkCrossLibrary)` BEFORE the batched summaries fetch. `summaryHeadlines` is **target-keyed** (not path-keyed) so the render-side lookup is a plain `summaryHeadlines.get(link.target)`. The resolve cost is bounded — fires only when `outgoingLinks` ref changes (tab switch), not on every render. Documented inline as a future Rule-8 cleanup candidate (persist `target_path` on `note_links`); deferred — outgoing lists rarely exceed a few dozen rows.

### Step C — Index panel (`src/lib/components/IndexPanel.svelte`)
- `$effect` tracks **`mentionsCache.size`** (gated by term expansion), body wrapped in `untrack` per the existing Rule-2 discipline in that file. Headline renders below the FTS5 snippet inside each mention row.
- **Virtualized row-height update:** new `ROW_HEIGHT_MENTION_HEADLINE = 16` added to `getRowHeight`'s per-mention sum **only when `summaryHeadlines.get(m.note_path)` is truthy** — rows without headlines stay compact. `void summaryHeadlines.size` added inside the `rows` `$derived.by` so VirtualList sees a new prop ref and re-measures when headlines arrive.

### Step D — Sky View — three-attempt saga ending at `LocalSkyView.svelte`
The architect doc said "wire the bubble inspector for the focused node." Three attempts:

**Attempt #1 (Build #1, installed 21:21):** Wired `SkyView.svelte`'s hover tooltip (the architect doc's "inspector" mapped to the hover tooltip per BASIC RULE — SkyView has no rich inspector). Renamed orphan `.star-tooltip` CSS → `.graph-tooltip` to match the template. svelte-check 0 new. Boss tested.

**Attempt #2 (post Stage 1.4 fail):** Boss said "no second line" with a screenshot showing the full-window Sky View (`7347 nodes · 217040 edges · 6564 MOCs · Atomic diffusion` status string on canvas — that's `FullSkyView.svelte`, not the embedded SkyView). I read this as "FullSkyView wasn't wired, it should be." Wired `FullSkyView.svelte` the same way. Stop-On-Correction Rule fired (correctly logged the change-list before touching code). Rebuilt → Boss tested again → **STILL FAIL**.

**Attempt #3 (the real fix — `LocalSkyView.svelte`):** Boss clarified "I meant the SV panel" + sent a screenshot of the embedded panel. I escalated diagnostics: searched the bundle for `tooltipHeadline`, found ZERO occurrences. Investigated → found that **NEITHER `SkyView.svelte` NOR `FullSkyView.svelte` is statically imported anywhere in `src/`** — both are dead code; Vite tree-shakes them; my fixes to them compiled but never rendered. The ONLY Sky View component actually imported by `+layout.svelte` and `SecondScreenPage.svelte` is `LocalSkyView.svelte` — despite the "Local" prefix suggesting second-screen-only.

Wired `LocalSkyView.svelte`:
- Namespaced CSS (`.local-star-tooltip{,-name,-headline}`) — narrow class names to keep Svelte's CSS pruner from clipping them (the original SkyView/FullSkyView CSS issue was rules being pruned because no template element matched their selector after a past `.star-*` → `.graph-*` template rename left CSS dead).
- Two-line tooltip: name (single line, ellipsis) + headline (3-line `line-clamp`, wraps).
- Monotonic `tooltipHeadlineToken` stale-promise guard.
- Edge-aware positioning: tooltip flips to LEFT of cursor when it would overflow the panel's right edge; flips UP when near bottom. Boss Stage 1-C confirmed PASS.

**Attempt #4 (the OTHER real component — `GraphMindView.svelte`):** Boss followed up: *"What about the main SV? Have you added the NSC function to it?"* The "main SV" (full-window) is mounted from `+layout.svelte:5331` as `<GraphMindView>` — NOT a `*SkyView*`-named file. The earlier import grep used `grep -r "import.*SkyView"` and missed it because the filename doesn't match. This is a follow-up note in LL-029: import-graph greps must cover ALL files that render a feature, not just files whose names match the feature.

Wired `GraphMindView.svelte`:
- Hooked the existing `onNodeHover(node)` callback (already wired to update `hoveredName` in the bottom status bar) to also start the headline fetch when `node.path` changes — same stale-promise token pattern.
- Added container-level `onmousemove` to track cursor coords in container-local space (doesn't interfere with `GraphEngine`'s canvas hit-testing — events bubble after canvas handlers).
- Same two-line tooltip shape with namespaced `.gm-tooltip{,-name,-headline}` classes.
- Same edge-aware flip math (max-width 280px to match GraphMindView's typical panel proportions).
- Bundle-verified: `getSummaryFor(` call count = **3** (NoteEditor + LocalSkyView + GraphMindView). Boss Stage 1-D confirmed PASS.

### Lessons in flight (will land in `docs/LESSONS-LEARNED.md`)
- **LL-028 — Windows + Tauri release builds silently fail to update `build/` while the app is running** (and even WebView2 instances from a recent close keep the dir locked). The build reports "✓ built" but EPERM-skips the copy-to-build/ step. Always close Constellation + verify no `msedgewebview2.exe` processes remain before `npm run tauri build`. Confirm fix landed by `grep -c <unique-token> build/_app/immutable/chunks/*.js` for any user-visible class/string change.
- **LL-029 — BASIC RULE / Predecessor Lookup violation:** before editing what looks like the right `.svelte` file based on its name, **always `grep -r "import.*ComponentName" src/`**. File names lie; the import graph is the ground truth. `SkyView.svelte` and `FullSkyView.svelte` looked authoritative; both were dead code. Two build cycles wasted before the grep happened.

### Out of Phase 2 scope (corrected)
- **`SkyView.svelte` + `FullSkyView.svelte`** — dead code; no importer; worth a future cleanup MIG.
- **Second-screen mount of LocalSkyView** — same component, will render the tooltip there too automatically. If Boss wants it suppressed on the display, that's a separate feature.
- **Map** — disabled (MIG-038).
- **Hover/wikilink-previews** — no such surface exists.

### Step E — `/simplify` + 3-agent audit (all clean)
- **Invariants:** 8/8 HOLD. Cache-first + batched everywhere (OutgoingLinksPanel's per-target resolve fires on tab switch, not render — within spec). No `$effect` loops. No boot regression. Author authority preserved. No new IPC. No schema change. Existing surfaces unchanged.
- **Drift:** PASS. No `$effect` self-read/write. No IPC on hot path. No missed consumer of `summaryStore` (grep-confirmed: `nsc_get_summaries_for_notes` invoked only in `summaryStore.ts`). VirtualList re-measure correct via Svelte 5 auto-tracking (`heights` derives reads `summaryHeadlines.get` through `getRowHeight`); the `void summaryHeadlines.size` in `rows` is belt-and-suspenders. CSS scope correct (all new classes namespaced; no `:global`). SkyView stale-token guard validated.
- **Migration-path:** PASS on all 4 scenarios. Fresh/existing DB unchanged. Rollback to MIG-043 reverts cleanly (additive diffs + `{#if summaryHeadlines.get(...)}` guards everywhere). Mid-backfill renders empty headlines as nothing (not blank gaps). Empty-NSC + BUG-022 auto-rebuild path untouched. Old-binary + new-frontend safe (Rust `#[serde(default)]` on `headline` + `?? ''` in every consumer).
- Two soft observations (no code changes needed): universe-switch cache bleed impossible by path-keying (documented in `summaryStore.ts:144-148`); SkyView "inspector" wording divergence acknowledged inline in code comment.

### svelte-check
- 3 pre-existing errors in unrelated files (`store.ts:2470`, `PropertyEditor.svelte:236/252`). **0 new errors from Phase 2.** Warnings count decreased by 1.

### Step F — SO + docs + 15-locale help additions + PCS-3
- **Help files:** English `Note Summaries` help expanded — `description:` enumerates all 7 surfaces; intro paragraph names each surface with its role; `## Where summaries appear, and how they fill in` grows from 3 bullets to 7 + the lazy-fill paragraph mentions the new gestures (expand-term, hover-bubble). **14 other-locale Note Summaries files** translated in parallel by a sub-agent using each locale's established native term.
- **Orientation v2.28** (new file alongside v2.27).
- **Session log** (this addendum) + MoCh fresh file for the Phase-2 cascade.

### Validation
- Boss live test pending. Build kicked off after svelte-check; targets `src-tauri/target/release/constellation.exe` for Eisa's preferred `.exe` install (`feedback_prefer_exe_over_msi`).

---

## 5. MIG-045 Phase 3 — Universe Digest left-dock view (SHIPPED + Boss-validated, the third pillar of the NSC Core Plug-in)

The work crossed midnight from 2026-05-23 into 2026-05-24; per the daily-log convention this is one continuous narrative under the day the cascade started. Eisa's "PCS-3 > MIG-045 Phase 3" directive (after Phase 2 PCS-3 commit `a7e1a5bf` pushed) green-lit the cascade.

### Architect doc
- `docs/MIG-045-nsc-coreplugin-phase3-ARCHITECT.md` — locked decisions enumerated (name "Digest", stored headline, cUniverse federation in scope v1, extractive only, recency sort, tiered Library→Folder→Headline), 10 invariants laid down, 5 design options (placement / expansion / federation / toolbar / data source / fetch strategy) each with chosen + rejected variants, 7-step Plan (A–G), 6-scenario migration-path matrix, risk summary.

### Build Steps A–E
- **Step A — DigestPane skeleton:** new `src/lib/components/DigestPane.svelte` with toolbar shell + empty placeholder. Wired into `+layout.svelte` (extended `sidebarMode` type to include `'digest'`, new mode-tab button, render branch with `<DigestPane>`, escape-handler entry). Extended `SidebarMode` in `src/lib/secondScreen.ts`. New `digest.*` i18n block + `navigator.digest` key in `src/lib/i18n/en.json`.
- **Step B — Tiered tree derivation:** `treeRows: VRow[]` `$derived.by` from `nodes + libraries`. VRow types: `'library-header' | 'folder-header' | 'note'` (expanded summary handled inline within the note row, not as a separate VRow kind). Path-separator-agnostic `folderOf()` helper. Bucket-by-libraryName-then-folder, sort folders + notes per `sortMode` ('recency' default → max child createdAt desc; 'alpha' → name asc), emit headers + rows.
- **Step C — Headline fetch + render + inline expansion:** `summaryHeadlines: Map<path,string>` + `fullSummaries: Map<path,string>` + `expanded: Set<path>` `$state`. `$effect` over `treeRows` → `getSummariesFor(visiblePaths)` → merge with `changed` guard. Each note row shows name + faint italic headline; when expanded, the full summary appears inline with multi-line wrap. Toggle handler `toggleExpand(path)`.
- **Step D — Filter + sort:** `passesFilter(node)` reads name + headline + full summary case-insensitively. Filter applies in `treeRows` derivation — folders/libraries with zero passing notes are skipped entirely (no empty headers). Sort toggle button (clock ↔ bars icon). **cUniverse on/off toggle REMOVED from v1** — identifying child-universe libraries requires a `universe_id` field on `LibraryInfo` (a Rust change, violating the frontend-only invariant). Federation still works via the existing flatten; v1 just doesn't offer a UI toggle. Documented inline in `DigestPane.svelte:43-50`.
- **Step E — VirtualList wiring + row heights:** `VirtualList` mounted with `items={treeRows}`, `getItemHeight={getRowHeight}`, `scrollResetKey={`${sortMode}|${filterQuery}`}`, `overscan={10}`. Per-row heights: library-header = 30, folder-header = 24, note base = 22 + 14 (headline if loaded) + dynamic expanded-summary height (clamped to 14 lines). svelte-check 0 new across all 5 steps.

### Step F — `/simplify` + 2-agent audit (caught + fixed 2 issues BEFORE Boss test)
The audit ran in parallel with the first build. **Both issues were caught before any Boss-test install shipped**, which is meaningfully better than the Phase 2 saga (3 wrong-target wirings caught DURING Boss test).

- **Audit #1 (invariants + drift)** — 10/10 invariants checked. **Two findings:**
  - **Rule-2 violation** (HIGH): the headline-fetch `$effect` read `summaryHeadlines` / `fullSummaries` via `new Map(...)` AND wrote to them. Combined with `passesFilter` (used in the `treeRows` derivation) reading the same Maps, this created a potential self-fire loop. Fix: wrap the effect body in `untrack(() => { ... })` (same pattern as `IndexPanel.svelte:90-101` / `134-163`). Only `treeRows` remains as the tracked dep.
  - **a11y nested-interactive** (MEDIUM): the row was a `<div role="button">` containing a `<button>` for the note name → Svelte's compiler warning + would be flagged by strict a11y. Fix: row is now a plain layout `<div>` (no role) with THREE sibling buttons — chevron-only (toggles expand), name (opens note), headline (also toggles expand — wider expand target). Each interactive is keyboard-accessible; no nesting.
- **Audit #2 (migration-path)** — 6/6 scenarios PASS. Fresh DB + existing DB + mid-backfill + universe switch + rollback to MIG-044 + cUniverse runtime add/remove all clean. SidebarMode type extension verified across both `secondScreen.ts:229` and `+layout.svelte:265`; `SecondScreenPage.svelte` uses `if/else-if` not `switch`, so `'digest'` falls through to the note-companion (architect doc §3 defers SS mount).

### Bundle verification (per LL-028)
Build #7 (after audit fixes): bundle-grep confirms `dg-chev-btn`, `dg-note-headline`, `DigestPane`, `dg-library-header` all present (2 lines each = CSS rule + template attribute). `untrack(` present in the chunk that contains DigestPane. Installer fresh at 11:01:29.

### Boss test (Stage 1, 8 tests) — PASS
- Test 1.1 — Dock entry appears ✅
- Test 1.2 — Open the Digest ✅
- Test 1.3 — Headlines fill in ✅
- Test 1.4 — Click-to-expand ✅
- Test 1.5 — Click name opens note ✅
- Test 1.6 — Filter narrows correctly (incl. multi-tier collapse) ✅
- Test 1.7 — Sort toggle switches recency↔alphabetical ✅
- Test 1.8 — Smooth scroll on 7,600+ note library ✅

Eisa: *"Pass. You've created an amazing function, Claude. Thank you!"*

### Step G — SO + 15-locale help additions + PCS-4
- **New help topic in English:** `docs/help.uConstellation.World/The Digest/The Digest.md` — what the Digest is, why it exists, full UX walkthrough, common workflows, what's NOT in v1.
- **14-locale translation** of the new help topic + the Note Summaries 8th-surface update + the `digest.*` i18n strings — dispatched as a single background sub-agent (3 concerns × 14 locales).
- **Note Summaries help** updated in all 15 locales: 8th surface (Universe Digest) added, intro paragraph + frontmatter description extended, closing "lazy-fill" paragraph mentions "scroll the Digest" gesture.
- **Orientation v2.29** (new file alongside v2.28).
- **Session log §5** (this section).
- **MoCh fresh file** `docs/MoCh/MoCh-2026-05-24-0900.md` for the Phase 3 work.
- **PCS-4** — pending Eisa explicit go.

### The NSC Core Plug-in roadmap is now SHIPPED end-to-end
Three months of work, three MIGs, all live:
- MIG-043 (2026-05-23): engine `headline` + shared `summaryStore` + 2 surfaces.
- MIG-044 (2026-05-23): 5 remaining surfaces wired (with 3 wrong-target wirings caught + fixed → LL-028/029).
- MIG-045 (2026-05-24): the Universe Digest pane (both audit issues caught + fixed BEFORE Boss test — proof the LL-028/029 discipline compounds).

The "summary service feeding every Constellation function" half of Concept Paper v2.0 + the "dock view to skim the whole knowledge base at summary level" half are both delivered. Phase 4 work (if any) is at Eisa's direction — no MIG-046 currently planned.
