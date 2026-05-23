# Constellation NSC — Core Plug-in Concept Paper

**Version 2.0 | 2026-05-22**

> **Purpose**: Define the elevation of the **Note Summary Creator (NSC)** from a single-surface *subsystem* (v1.0, MIG-040) into a standalone **Core Plug-in** — the app-wide "what is this note about?" layer. Born from Eisa's 2026-05-22 direction (after the MIG-042 session): *"grow the NSC into a standalone Core Plug-in serving every Constellation function."*
>
> **Vision decisions locked this session (Eisa):**
> - **Shape:** *Both* — a shared summary **service** feeding every surface **and** a dedicated left-dock **view**.
> - **Dock-view purpose:** a **Universe Digest** — skim the whole knowledge base at summary level without opening notes.
> - **Service reach:** **all surfaces** (full coverage is the target; build sequenced sensibly).
> - **Digest granularity:** **tiered** — library → folder → 1-line headline per note, expandable to the full summary.

---

## §1 — The core concept (one sentence)

**NSC becomes Constellation's universal summary layer: a single cache-first service that supplies every note's summary to every surface that displays a note, plus a left-dock Universe Digest that lets the user read the entire knowledge base at a glance — tiered library → folder → one-line headline, expandable to the full summary.**

## §2 — What exists today (v1.0 recap)

NSC v1.0 (MIG-040, Concept Paper v1.0) is a *summarization engine* surfaced in *one place*:
- **Engine:** author-summary precedence (frontmatter `summary`/`description`/`abstract`/`excerpt` → `[!summary]`/`[!abstract]`/`[!tldr]` callout) → otherwise **language-agnostic extractive TextRank** over e5-small sentence embeddings (UAX #29 segmentation; opening-text fallback). 100% on-device, all languages, faithful (never invents text).
- **Storage:** `note_summaries (path PK, summary, source, content_hash, updated_at)`; content-hash invalidation; `NSC_ALGO_VERSION` prefix self-heals on algorithm change.
- **Delivery:** deferred background worker (gentle, resumable, off the hot path) + batched cache-first IPC `nsc_get_summaries_for_notes` + single `nsc_get_summary`; the manual **"Build all summaries"** backfill button (in the Cataloger header).
- **Surface:** rendered under the title on **Cataloger / Source-Review cards only.**

v1.0 §2 already anticipated this paper: *"a faithful per-note summary is a reusable knowledge primitive (search previews, hover cards, future surfaces) — Eisa frames it as a Constellation differentiator."* v2.0 cashes that in.

## §3 — The two pillars

### §3.1 — Pillar 1: the Summary Service (everywhere)

The engine generalizes into the app's single summary provider. The mechanism already exists (the batched, cache-first `nsc_get_summaries_for_notes` built for the Cataloger to avoid per-card IPC); v2.0 lifts it into a **shared frontend summary store/composable** that any component can ask for a batch of summaries and get cache-first results with zero per-item IPC.

**Candidate consuming surfaces** (the Architect pass confirms the exact *enabled* set — e.g. **Map is currently disabled (MIG-038), so it's out**):
- **Search results** — a summary line under each hit (the highest day-to-day payoff: "is this the note I want?").
- **Sky View bubbles** — summary in the node tooltip/inspector.
- **Backlinks / Outgoing panels** — summary under each linked note.
- **The Index panel** — summary alongside term/entry rows where a note is shown.
- **The editor header** — the active note's own summary, in-context.
- **Hover / wikilink previews** — *if/where such a preview surface exists* (Architect to confirm; not fabricated here).

**Discipline (unchanged from v1.0):** cache-first, batched, **zero per-item IPC on render**, nothing on the keystroke/save hot path. A surface that can show >50 items virtualizes and requests summaries only for visible rows.

### §3.2 — Pillar 2: the Universe Digest (left-dock view)

A new **Core Plug-in dock surface**, built on the proven Cataloger/OrgChart left-dock pattern (lazy-mounted full-page overlay, dock button, command-palette entry, escape/close wiring). Its one job: **read the whole knowledge base at summary level without opening notes.**

- **Tiered structure (locked):** collapsible **Library → Folder → note**, each note shown as a **1-line headline**; expand a note to reveal its **full 2–3 sentence summary**; click to open the note.
- **Navigation:** search/filter across summaries (reuse the existing lexical search); sort (recency / title / library order); collapse-all / expand-all.
- **Performance:** the tree is **virtualized** (render only visible rows regardless of universe size — Rule 3); summaries arrive via the batched service (§3.1), never per-row IPC.
- **Empty/partial state:** the digest is only as complete as the cache. If summaries are missing, the view surfaces the **"Build all summaries"** backfill with progress (the backfill becomes load-bearing here) — and, per the BUG-022 lesson this session, **an empty cache must offer/await its build, never silently show nothing.**

## §4 — Engine change: the 1-line headline

The tiered digest needs a **single-sentence headline** per note in addition to the full summary. This is cheap and faithful:
- **Extractive notes:** the top-1 sentence from the *same* TextRank ranking the full summary already computes (k=1 instead of k≈3).
- **Author summary / callout / frontmatter:** the headline is that text's first sentence (the author's words win — invariant 2).
- **Opening fallback:** the opening sentence.

**Storage:** add a `headline TEXT` column to `note_summaries` (write-time derivation, same content-hash invalidation) — a minor, additive schema change. (Alternative considered: derive the headline at read time as the summary's first sentence — cheaper to ship but less faithful than top-1; Architect decides. Recommendation: store `headline` for fidelity + zero read-time cost.)

## §5 — Architecture & reuse

**Reuse, not rebuild.** Unchanged: the summarization algorithm, `note_summaries` cache, content-hash invalidation, `NSC_ALGO_VERSION` self-heal, the deferred background worker, the "Build all summaries" backfill. **New:**
1. The `headline` variant (§4) — small engine + schema add.
2. A **shared frontend summary store** (cache-first, batched) — generalizes the Cataloger's existing fetch so any surface uses one path.
3. Wiring summaries into the §3.1 surfaces (the bulk of the integration work, sequenced).
4. The **Universe Digest** dock view (§3.2).

## §6 — Performance invariants (hard constraints)

Inherits CLAUDE.md Rules 1/3/8 and the boot-perf hard constraint. Specifically:
- **Cache-first + batched everywhere** — no per-item IPC on any render path; no `invoke()` on a keystroke/scroll hot path.
- **Virtualize** the Digest (and any surface list >50 rows).
- **Off the hot path** — summary computation stays in the deferred worker; reads are cheap lookups (Rule 8 write-time derivation).
- **No boot regression** — the Digest is lazy-mounted (mounts on first open, like the Cataloger); the service adds no boot-time walk.
- **Backfill is gentle + resumable** — unchanged; the Digest's usefulness depends on it, so its progress must be visible.

## §7 — Architectural invariants (inherited from v1.0)

1. **File-Over-App** — read-only on notes; the cache is a rebuildable derived view; NSC never writes summaries into note files.
2. **Author authority** — a frontmatter/callout summary always overrides the generated one (headline included).
3. **Local-only** — all inference on-device (e5-small ONNX); zero cloud path.
4. **Language-agnostic** — UAX #29 + multilingual embeddings + graceful fallback; works for every language.
5. **Faithful** — extractive output (full + headline) is the note's own sentences; never invented.
6. **No hot-path heavy work** — deferred worker only.

## §8 — Form-Aligns-To-Purpose

The Digest's purpose is *understanding the whole knowledge base at a glance.* Every element must serve skim-and-comprehend: the tier headers carry note counts (orientation), the 1-line headline carries the gist, expansion carries depth. No decorative spread, no metric that doesn't help the user decide "do I need to open this?" If a sort/filter doesn't aid comprehension or navigation, it doesn't ship.

## §9 — Decisions (locked 2026-05-22 by Eisa, same session)

1. **Dock-view name:** **"Digest"** (localized across all 15 locales — native equivalents set during the digest-view phase i18n; the convention is the right native term per locale, not transliteration).
2. **Headline storage:** **stored** — a `headline TEXT` column added to `note_summaries`. Faithful (top-1 from TextRank), zero read-time cost, content-hash-invalidated like the rest of the cache.
3. **cUniverse federation:** **the Digest spans linked (child) universes** from v1 — federation is in scope from the start, not deferred. (`resolve_libraries_recursive` already flattens the federation tree; the Digest reads from that flattened library list.)
4. **Abstractive mode:** **v2.0 is extractive only.** Abstractive (LLM rewrite) remains the future upgrade per v1.0 §8, behind a setting, when the local LLM is wired.
5. **Default sort:** **recency** (most-recently-modified first) — within each folder, after the Library → Folder tiering.

## §10 — Phasing (each phase = its own `/migration`, MIG numbers assigned at Architect time)

- **Phase 1 — Service foundation + first surfaces.** Add the `headline` variant + shared frontend summary store; wire the 2 highest-value surfaces (proposed: **search results** + **editor header**). Establishes the pattern + proves the performance discipline.
- **Phase 2 — Full service reach.** Wire the remaining enabled surfaces (Sky View, backlinks/outgoing, Index, hovers-if-present).
- **Phase 3 — The Universe Digest view.** The left-dock tiered view (§3.2) + backfill-completeness UX.

Each phase is cross-subsystem (Rust engine ↔ Svelte surfaces, or a new dock view) → full four-phase `/migration` (Architect → Plan → Build → Audit). No phase may regress boot, typing latency, or IPC responsiveness — measured before/after on the 7,600+-note universe.

---

*Concept Paper v2.0, cut 2026-05-22. Elevates `Constellation-NSC-Concept-Paper-v1.0.md` (the engine/subsystem contract) to the Core Plug-in vision. Vision decisions sourced from Eisa, 2026-05-22 (this session). Companion to the plugin taxonomy (Core Plug-in = left-dock, stays in-app; cf. Cataloger MIG-039) and the write-time-derivation discipline (CLAUDE.md Rule 8).*
