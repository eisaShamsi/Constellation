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

## 3. Open / deferred

- **Phase 2 (MIG-044)** — wire the remaining enabled surfaces: Sky View bubbles, backlinks/outgoing panels, Index panel, hover previews (where they exist).
- **Phase 3 (MIG-045)** — the Universe Digest left-dock view: tiered Library → Folder → 1-line headline, expandable to full summary, recency-sorted, searchable, virtualized, with cUniverse-children federated in.
- Both Phases will go through full `/migration` (Architect → Plan → Build → Audit) per the project's discipline.
