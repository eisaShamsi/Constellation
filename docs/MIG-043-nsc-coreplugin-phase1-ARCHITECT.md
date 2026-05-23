# MIG-043 — NSC Core Plug-in, Phase 1: headline + shared service + first surfaces

**Status:** Architect (Phase 1 of the `/migration` workflow). Awaiting Eisa approval of this doc + the Plan before Build.
**Date:** 2026-05-22
**Lineage:** Phase 1 of the NSC Core Plug-in roadmap defined in `docs/Constellation-NSC-Concept-Paper-v2.0.md` §10. Builds on NSC v1.0 (MIG-040).

---

## 1. Goal

Lay the **service foundation** of the NSC Core Plug-in:

1. Add a **1-line `headline`** variant (top-1 TextRank sentence) to the engine + the `note_summaries` cache — the granularity the tiered Digest needs (Concept Paper §3.2/§4/§9.2).
2. Generalize the existing batched cache-first delivery (`nsc_get_summaries_for_notes`) into a **shared frontend summary store/composable** any surface can use.
3. **Wire the first two surfaces**: **search results** + the **editor header** — the highest day-to-day payoff and a deliberate two-shape test (a list view + a single-note view) that proves the pattern for Phases 2 + 3.

What Phase 1 does **NOT** do: the remaining surfaces (Sky View, backlinks/outgoing, Index, hover previews) ship in **Phase 2 (MIG-044)**; the Digest view ships in **Phase 3 (MIG-045)**.

---

## 2. Territory map (verified against current code, 2026-05-22)

### 2.1 The engine + cache (today, MIG-040)

- `src-tauri/src/nsc/mod.rs`:
  - **Schema (`ensure_note_summaries_table`, ~L439)**: `note_summaries(path TEXT PK, summary TEXT NOT NULL, source TEXT NOT NULL, content_hash TEXT NOT NULL, updated_at INTEGER NOT NULL, FOREIGN KEY (path) REFERENCES note_meta(path) ON DELETE CASCADE);`. Set up from `search::init_db`. No `headline` column today.
  - **Algo version**: `NSC_ALGO_VERSION = "v2"` (~L459), embedded in `body_hash` → bumping invalidates the whole cache (self-heal).
  - **Entry point**: `get_or_compute_cached(app, note_path)` (~L505) — reads `note_meta`, checks cache by `body_hash`, computes + caches on miss/stale, returns `Option<NoteSummaryEntry>`. Used by every read + the backfill worker.
  - **INSERT** (~L550): `INSERT OR REPLACE INTO note_summaries (path, summary, source, content_hash, updated_at) VALUES …`.
- `src-tauri/src/nsc/backfill.rs`: the deferred worker (start/status/cancel commands at L75-99). Drives `get_or_compute_cached` note-by-note, gentle + resumable.

### 2.2 The IPC surface (today)

| IPC | Shape | Caller(s) today |
|---|---|---|
| `nsc_get_summaries_for_notes(note_paths) -> Vec<NoteSummaryEntry>` | batched, `#[command(async)]`, cache-first | `SourceReviewPanel.svelte:221` (**only consumer**) |
| `nsc_get_summary(note_path) -> Option<NoteSummaryEntry>` | single, on-demand | (unused on the frontend right now) |
| `nsc_backfill_start / _status / _cancel` | worker control | Cataloger "Build all summaries" button |

`NoteSummaryEntry` carries `{ path, summary, source }`. **No `headline` field today.**

### 2.3 The single existing surface consumer

`src/lib/components/SourceReviewPanel.svelte` is the only place summaries are fetched + rendered. The fetch (~L221) is a direct `invoke('nsc_get_summaries_for_notes', { notePaths: paths })` for the visible queue — the batched-cache-first pattern that already proves zero per-card IPC. The render (~L1266) shows the summary under the card title with i18n key `nsc.summary` ("Summary"). **This is the pattern Phase 1 generalizes.**

### 2.4 The two Phase-1 surface targets

- **Search results** — the surface(s) returning hits to the user (the SearchHub family / lexical results UI). Today, results are rendered as titles + snippets/context; Phase 1 adds the note's NSC summary (1-line + expandable, or compact) under each hit. Exact rendering shape to be settled in Plan/Build with Eisa.
- **The editor header** — `NoteEditor.svelte` / `NotePane.svelte`. Phase 1 shows the active note's summary in/near the header so the user has the gist in-context. Exact placement (header strip vs. collapsible band vs. tooltip) to be settled in Plan/Build.

---

## 3. Invariants that MUST NOT break

1. **Existing Cataloger / Source Review rendering is unchanged** — SRP migrates from direct `invoke` to the shared store with the **same observable behavior** (same fetch, same render, same i18n).
2. **Cache-first + batched everywhere** — no surface introduces a per-item IPC on render. Surfaces with > 50 visible rows virtualize and request summaries only for visible paths (Rule 3).
3. **No hot-path heavy work** — summary + headline computation stays in the deferred worker; reads are cheap cache lookups (Rule 8). Nothing here adds work to the typing / save / boot path.
4. **No boot regression** — measured on the live 7,600-note universe before/after (CLAUDE.md hard constraint). The headline column-add is one ALTER on init_db (idempotent); no walk.
5. **Author authority extends to the headline** — for a frontmatter / callout / author summary, the headline is *that text's* first sentence; the author's words win (v1.0 invariant 2).
6. **Backward-compat / additive schema** — existing `note_summaries` rows keep working; `headline` is a nullable column; surfaces that don't yet use headline ignore it. Rollback to MIG-040 code is safe (old code reads `summary` only).
7. **Frontmatter + callout precedence preserved** — unchanged from v1.0 §3.
8. **File-Over-App** — NSC is still read-only on notes; the cache is the only thing that grows.

---

## 4. Design options

### A. Headline storage shape

- **A1 — Add `headline TEXT` column to `note_summaries` (CHOSEN — Concept Paper §9.2).** Faithful (top-1 from the ranking the engine already does), zero read-time cost, content-hash-invalidated like the rest of the cache. Schema change is additive.
- A2 — Derive at read time as the summary's first sentence. Rejected: less faithful than top-1; reduces the engine's value.

### B. Schema migration for existing DBs (Eisa's case)

- **B1 — Nullable column + lazy fill on next computation (CHOSEN).** `ALTER TABLE note_summaries ADD COLUMN headline TEXT;` (nullable; idempotent via `PRAGMA table_info` probe). Existing rows have NULL headline. `get_or_compute_cached` checks `headline IS NULL OR content_hash != body_hash(...)`; on either, recompute (cheap — same ranking already produces summary). Net: headlines fill in incrementally as the cache is touched, and the existing "Build all summaries" backfill ensures full coverage.
- B2 — Bump `NSC_ALGO_VERSION` v2 → v3 to invalidate the whole cache. Rejected: heavy + unnecessary — the algo is unchanged, only the *output shape* gains a field.
- B3 — One-time migration deriving `headline` from existing `summary` rows (first sentence). Rejected: less faithful than top-1; the lazy path achieves correctness without a write-walk.

### C. Shared frontend summary store

- **C1 — Typed module with a cache-first batched API (CHOSEN).** New `src/lib/nsc/summaryStore.ts` exposing:
  - `getSummariesFor(notePaths: string[]): Promise<Map<string, NoteSummaryEntry>>` — batched cache-first; coalesces concurrent requests for the same paths; in-memory LRU/Map so re-renders don't re-IPC.
  - A reactive Svelte store for surfaces that want auto-update when a path's summary changes (e.g., after the backfill fills it).
  - Cache invalidation hook (a small Tauri-event listener: when the file watcher fires on a path, drop that path's cached summary so the next read re-IPCs).
- C2 — A pure function per call site. Rejected: no cross-component cache; defeats the "shared service" purpose; duplicates SRP's logic.
- C3 — Svelte context (provide/inject). Rejected: couples to component tree; awkward for non-component consumers (e.g., a `$derived` in `+layout.svelte`).

### D. First surfaces to wire in Phase 1

- **D1 — Search results + editor header (CHOSEN — Concept Paper §10 Phase 1).** Highest day-to-day payoff. Two distinct shapes (list, single-note) — proving the pattern on both before scaling to all of Phase 2's surfaces.
- D2 — Search only. Rejected: too narrow.
- D3 — Editor header only. Rejected: less visible (most users meet summaries in search first).
- D4 — All surfaces at once. Rejected: blows Phase-1 scope; the pattern + perf discipline must prove on a small set first per CLAUDE.md (no all-at-once cross-subsystem rollouts).

### E. Performance discipline (uniform across all surfaces)

- Surfaces request summaries for the **currently-visible** paths only (virtualized lists call `getSummariesFor` for the visible window, top up on scroll).
- Cache-first → memoized in `summaryStore.ts` → batched IPC on miss → background fill via the existing worker.
- **Hard rule**: no surface in Phase 1 ships if its first-render measurement (on Eisa's live universe) is slower than the current MIG-040 SRP render. Boot time on the same universe: unchanged.

---

## 5. Plan (each step = one commit + verification clause)

> **Step A — Engine + schema add (the `headline` variant).**
> - `ensure_note_summaries_table`: amend the `CREATE TABLE` to include `headline TEXT`; add an idempotent `ALTER TABLE … ADD COLUMN headline TEXT` for existing DBs (probe `PRAGMA table_info`).
> - `NoteSummaryEntry`: add `headline: String` with `#[serde(default)]` so the frontend's old `NoteSummaryEntry` shape deserializes without `headline` until the frontend is updated.
> - `get_or_compute_cached`: emit + store `headline` (top-1 from the same TextRank ranking that produces the summary; for author/callout/frontmatter sources, `headline` = the first sentence of the author's text).
> - `nsc::backfill`: compute + store `headline` alongside `summary`.
> - Unit tests: headline per source kind (frontmatter / callout / extractive / opening); cache-hit returns headline; lazy-fill on existing-row NULL-headline cache hit.
> *Verify:* `cargo test -p constellation nsc::` green; on a copy of the live DB, after one `nsc_get_summary` call, the row has a non-null headline.

> **Step B — Shared frontend summary store.**
> - New `src/lib/nsc/summaryStore.ts` per §4 C1.
> - **Migrate `SourceReviewPanel.svelte`** from direct `invoke` to the store — no behavior change.
> - File-watcher invalidation hook in the store (drop cached entry on `library-changed` for that path).
> *Verify:* Cataloger / Source Review still renders summaries identically (Boss-test on "Eisa Cognitive Knowledge"); no extra IPC observed; `svelte-check` 0 new errors.

> **Step C — Wire first surface: search results.**
> - Show the note's summary under each hit (compact line; 1-line headline by default, with the full summary on hover/expand — exact UX confirmed with Eisa in Plan).
> - Virtualization-aware: request summaries only for visible result rows; top up on scroll.
> *Verify:* Boss-test: search returns results with summaries; scrolling stays smooth; no per-row IPC in dev-tools network trace.

> **Step D — Wire first surface: editor header.**
> - Show the active note's summary near the title in the editor (placement: header strip vs. collapsible band — confirmed with Eisa).
> - Updates reactively when the note changes (or when the backfill fills it).
> *Verify:* Boss-test: open a note → its summary shows; edit + save → summary updates after the next backfill cycle (not on every keystroke); typing latency unchanged.

> **Step E — `/simplify` + Phase-D audit.**
> `/simplify` the diff. Three parallel agents per the Migration Rule: (1) invariants §3 hold; (2) drift — any unmapped consumer of `NoteSummaryEntry` (would it break on the new `headline` field?); (3) migration path — fresh DB, existing DB with NULL headlines, mid-backfill, rollback.

> **Step F — SO + docs.**
> Session log; Orientation v-bump; **help files** — search-results summary + editor-header summary ARE user-visible, so small help/User-Manual additions in **all 15 locales** (the full-localization rule).

---

## 6. Migration-path matrix

| Scenario | Behavior |
|---|---|
| **Fresh DB** | `note_summaries` includes `headline` from the CREATE TABLE; empty cache; lazy fill on first read + manual backfill via "Build all summaries". |
| **Existing DB (Eisa's)** | `ALTER TABLE … ADD COLUMN headline TEXT` (idempotent). All existing rows have NULL headline. First `get_or_compute_cached` per note recomputes (cheap) → stores `headline`. Backfill catches the rest. |
| **Mid-backfill interrupt** | Resumable (unchanged from v1.0 — gentle batched worker). |
| **Rollback to MIG-040** | Old code reads `summary` only, ignores `headline`. Column is additive; no breakage. The frontend store falls back to no-store consumption (SRP direct invoke is its prior shape). |
| **Cache-stale (content changes)** | Existing content_hash invalidation handles it — both `summary` and `headline` recompute together. |

---

## 7. Risk summary

**Low to moderate.** The backend changes are strictly additive (one nullable column + one new field on a returned struct, behind serde-default). The frontend store generalizes a pattern that already exists in SRP, so the Cataloger keeps working. The genuinely-new work is the **two surface integrations** (search results + editor header) — the risk there is UX (placement / styling / truncation) and performance discipline on the search-results list (must virtualize summary fetches). Both are mitigated by the same Rules 3/8 patterns Constellation already enforces everywhere.

**No schema-breaking change. No write-path change. No hot-path additions. Rollback is safe in both directions.**

---

*Phase 1 of the NSC Core Plug-in roadmap (Concept Paper v2.0 §10). On approval: I'll draft the Plan (commit-level steps with verification clauses + the exact UX placement decisions for the two surfaces). Phases 2 + 3 (full service reach + the Digest view) follow as their own `/migration`s — MIG-044, MIG-045.*
