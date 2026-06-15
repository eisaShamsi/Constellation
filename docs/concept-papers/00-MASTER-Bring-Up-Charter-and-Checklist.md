# Constellation Bring-Up — Master Charter, Concept-Paper Template & Function Checklist

> **Boss-directed method (Eisa, 2026-06-15).** *"Constellation works its magic through its entry point, the Note Editor — the gate to every aspect, element, function, and plugin. We disable everything except the editor, examine how Constellation functions (most of all, how fast it boots), fix the editor in isolation if needed, then re-enable functions/elements/plugins ONE BY ONE — making sure each wires correctly to the editor and to Constellation, fixing any lag, until we reach the ultimate, fast PKF/PKM system. And every function must have a concept paper defining its purpose — it acts as the checklist."*

This is the master index for that program. It holds the **charter** (the method), the **concept-paper template** (the standard every function's paper follows), the **function inventory** (the checklist), the **dependency-ordered bring-up sequence**, and the **`safeBootMode` gate design**.

---

## 1. Charter — why this method, and what "done" means

The method is *ablation / incremental bring-up* — the most rigorous way to get a **measured, per-function boot+latency budget** instead of arguing about causes. It is the operational form of three rules Constellation already holds:

- **Constraint as Design** — *"every feature must justify its existence."* A function with no concept paper hasn't justified itself.
- **Form-Aligns-To-Purpose** — *"every part of every feature must justify its presence within it."*
- **Rule 8 (Write-Time Derivation)** — *"the app does not recompute on boot; it reads what's already stored."* Each function is re-enabled only once it obeys this.

**The baseline truth (measured 2026-06-15, live 1.83 GB / 7,653-note "Eisa Cognitive Knowledge"):** `paint_ms=941` (PASS), `hydrated_ms=1671` (PASS) — **the editor mounts fast and is already a solid baseline.** The pain is entirely in the satellites that load around it; the single biggest is the boot graph snapshot (~30.8 s — `cache_boot_snapshot_graph`).

**Done =** every function has a concept paper; each is re-enabled in dependency order; each meets its boot+latency budget and obeys Rule 8; the full system boots fast with nothing recomputed that could have been read.

---

## 2. The `safeBootMode` gate (how "minimal mode" is achieved without scissors)

**Finding (grounded in code):** most satellites are *already* gated by `appSettings.enabledFeatures.*` (`src/lib/libraries/store.ts:~3530`) — `skyView`, `backlinks`, `outgoingLinks`, `tags`, `search`, `index`, `orgChart`, `inspector360`, `cece`, `ccs`, `secondScreen`, etc. — and Sight/Map use the MIG-038 compile-time pattern (`SIGHT_*_ENABLED` in `src/lib/sight/engine.ts`; Map force-disabled at `store.ts:~4038`). **The MIG-038 "code stays on disk, flag off" philosophy is the precedent we extend — flags, not deletion.**

**The gap:** four boot IPCs fire **unconditionally**, regardless of those flags (`src/routes/+layout.svelte`):
1. `cache_boot_snapshot_graph` — Phase 2 (~`:3144`) — **the 30.8 s** (read_links 234k rows + read_tags + queue).
2. The `federation:ready` listener + defensive snapshot re-invoke (~`:2552`).
3. `listFiveActsNotes()` (~`:2038`).
4. `getFederationWarnings()` (~`:2042`).

**Design:** one new settings flag `safeBootMode: boolean` (default `false`), read once at boot, wrapping those four in `if (!safeBootMode) { … }`. Paint + `constellation_boot_bundle` + `cache_boot_snapshot_core` (the hydration gate) always run — they're the editor+tree spine. This is a *reversible* switch; re-enable is a flag flip, never a re-wire. (Detailed design in the bring-up plan, once the template is signed off.)

---

## 3. Concept-Paper Template (the standard — every function's paper follows this)

> File: `docs/concept-papers/NN-<Function-Name>.md`. Substantive functions/elements/plugins get their own paper; trivial UI primitives (Rename/Collision/Template dialogs) are folded under their parent function. Keep each paper to ~1 page — it is a contract + checklist, not a manual.

```
# <NN> — <Function Name> (Concept Paper)

## 1. Function in hand
<The name EXACTLY as the orientation doc / UI names it. One line.>

## 2. Purpose
<The cognitive/functional job. The ONE question it answers. Why it exists
(justify its existence — Constraint as Design). If it can't, say so.>

## 3. What it is NOT
<Scope boundary — prevents drift. The "this is not X" line.>

## 4. Wiring
- Inputs: <events / IPC / stores it reads>
- Outputs: <events / IPC / writes it emits>
- Consumers: <who depends on its output>
- Connection to the Editor (the gate): <how it attaches to the entry point>

## 5. Right-click / context menu
- Has a right-click menu? <yes/no — if no, SHOULD it? flag the gap>
- Items per target kind, and whether they route through the shared
  `buildContextMenu()` / `<ContextMenu>` (MIG-077) or a HAND-ROLLED menu (a debt to flag).
- Actions reachable ONLY by right-click (so the bring-up verifies them).

## 6. Multilingual (Constellation is multilingual BY DEFAULT)
- Every user-facing string through `$t()` and present in ALL 15 locales
  (ar, de, en, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh).
- RTL: `dir` / `detectDir()`, flipped chevrons/arrows, mixed-script content.
- Native equivalents (not transliteration), per the full-localization top-principal
  (e.g. مصادر, not "masādir"). Per-script fonts where it shows content.
- Any hardcoded English = a gap to flag.

## 7. Boot behavior
- Runs at boot? <yes/no/on-demand> — which IPC(s)?
- Rule 8 status: <reads persisted data | RECOMPUTES (a violation to fix)>
- Cost: <measured ms where known; else estimated, marked as such>

## 8. Flag / gate & bring-up position
- Gate today: <existing enabledFeatures.X | SIGHT_*_ENABLED | needs new gate>
- Bring-up phase: <MASTER §5 phase 0–6>  · Depends on: <function(s) beneath it>

## 9. Budget
- Boot budget: <target ms it must stay under when enabled>
- Interaction budget: <keystroke/open/query latency target>
- Regression guard: <what to measure before/after>

## 10. Acceptance checklist (the gate to "re-enabled")
- [ ] Serves its stated purpose — does the ONE job, nothing extra (Form-Aligns-To-Purpose)
- [ ] Serves Constellation's core purpose (advances a Five Act toward Conviction — see [00-Constellation](00-Constellation-Core-Concept-Paper.md))
- [ ] Wires correctly to the Editor and Constellation (inputs/outputs verified)
- [ ] **Right-click** menu present + correct (shared `<ContextMenu>`, not hand-rolled); all items work
- [ ] **Multilingual**: strings ×15, RTL correct, native equivalents, no hardcoded English
- [ ] Within boot + interaction budget (measured before/after)
- [ ] Obeys Rule 8 (reads stored; recomputes nothing it could have read)
- [ ] Holds its top-principal invariants (content-integrity / etc. as applicable)
- [ ] Boss-tested per the Testing Instructions Rule

## 11. Status
Concept paper: <draft|reviewed> · Enabled in bring-up: <no|yes> · Budget met: <—|✓>
Notes: <...>
```

> **Two cross-cutting requirements every paper MUST address** (Boss-added 2026-06-15): the element's **right-click / context-menu** behavior (§5) and its **multilingual** conformance (§6). They are also acceptance-checklist gates. The root paper that every per-function paper serves is **[00-Constellation — Core Concept Paper](00-Constellation-Core-Concept-Paper.md)**.

---

## 4. Function Inventory (the checklist)

Grouped by spine and bring-up phase. Cost = **M**easured or **E**stimated. "Gate" = how it's disabled today (or `NEW` if minimal mode needs one). Concept paper (CP) status tracked per row.

### Phase 1 — Core spine (the baseline; stays on in minimal mode)
| # | Function | Purpose (1-line) | Boot IPC | Cost | Gate | CP |
|---|---|---|---|---|---|---|
| 1 | **Note Editor** (NotePane + FocusPane) | Edit the note; the gate to everything | first-tab `read_note` only | ~1–3 ms/open (M: mount cheap) | core (NEW flag to disable) | ✅ `01` |
| 2 | **File Tree** | Browse notes/folders | `cache_boot_snapshot_core` (notes) | core | core | ☐ |
| 3 | **Tab Bar** | Open-tab strip + context menu | — | ~0 (reactive) | core | ☐ |
| 4 | **Properties panel** | Frontmatter editor | — (parses tab content) | ~5–10 ms/tab (E) | core | ☐ |
| 5 | **Outline panel** | Heading navigator | — | ~1–2 ms (E) | core | ☐ |
| — | Boot bundle / settings / theme | App-shell config (single IPC) | `constellation_boot_bundle` | ~100–200 ms (E) | core | ☐ |

### Phase 2 — Search + Index
| # | Function | Purpose | Boot IPC | Cost | Gate | CP |
|---|---|---|---|---|---|---|
| 6 | **Search Hub** | FTS5 query w/ operators | `cache_mark_search_ready` (~800 ms) | ~5–50 ms/query (E) | `enabledFeatures.search` | ☐ |
| 7 | **Index panel** (Term Browser) | Vocabulary / term→mentions | on-demand reads | ~100–500 ms first (E) | `enabledFeatures.index` | ☐ |
| 8 | **Quick Switcher** | Fuzzy open by name | — (in-memory) | ~0 | `quickSwitcher` | ☐ |

### Phase 3 — Backlinks / Graph / Tags  ⚠️ the 30.8 s lives here
| # | Function | Purpose | Boot IPC | Cost | Gate | CP |
|---|---|---|---|---|---|---|
| 9 | **Backlinks panel** | Incoming links + mentions | `cache_boot_snapshot_graph` | **part of 30.8 s (M)** | `backlinks` (but IPC unconditional → **NEW**) | ☐ |
| 10 | **Outgoing Links panel** | Notes this note links to | `cache_boot_snapshot_graph` | (shared) | `outgoingLinks` (IPC **NEW**) | ☐ |
| 11 | **Tags panel** | Federated tag browser | `cache_boot_snapshot_graph` (read_tags 5.6 s) | **5.6 s (M)** | `tags` (IPC **NEW**) | ☐ |
| 12 | **Local Sky** (sidebar star) | Compact graph around active note | uses `allLibraryLinks` | ~2–5 ms (E) | (graph-dependent) | ☐ |
> **Rule-8 status: VIOLATION.** This whole row recomputes 234k links + every note's tags at read time on every boot. **This is MIG-079's target** (the WTD fix: persist + write-time-maintain; Boss chose: defer the full links read off boot + a `tag_counts` summary table maintained in the indexer). The bring-up re-enables this row *only after* MIG-079 lands.

### Phase 4 — Sky / Map / Sight (visualization)
| # | Function | Purpose | Cost | Gate | CP |
|---|---|---|---|---|---|
| 13 | **Sky View** (PIXI bubbles) | 2D force graph of all notes | on-demand (lazy) | `skyView` | ☐ |
| 14 | **Constellation Map** (D3 sunburst arcs) | Library/folder/note tree | on-demand | **force-disabled** (MIG-038) | ☐ |
| 15 | **Constellation Sight** (v2 CNS; v3/v4/v6/v7) | Epistemic-content lens(es) | on-demand | `SIGHT_*_ENABLED` (mostly off, MIG-038 Wings) | ☐ |
| 16 | **OrgChart** | Hierarchy tree (fullscreen <2 s post §A′.1) | on-demand | `orgChart` | ☐ |
| 17 | **Window-in-Window / Inspector360 / Local views** | Sub-graph / 360° context | on-demand | various | ☐ |

### Phase 5 — Knowledge curation & analysis
| # | Function | Purpose | Gate | CP |
|---|---|---|---|---|
| 18 | **The Cataloger (CECE)** | Source/evidence capture workflow | `cece` | ☐ |
| 19 | **CCS (Circulatory System)** | Link dashboard + federated metrics | `ccs` | ☐ |
| 20 | **Knowledge Health Dashboard** | Universe health metrics | `showKnowledgeHealth` | ☐ |
| 21 | **Tasks / Global Tasks** | Per-note + universe task lists | (panel) | ☐ |
| 22 | **Calendar** | Date → daily note | (panel) · `dailyNotes` | ☐ |
| 23 | **Review Pulse** | Spaced-repetition queue | (panel) | ☐ |
| 24 | **Tension / Provenance / Source Review** | Conflict / lineage / source queue | (panels) | ☐ |
| 25 | **Expression Forge / Sense-Making Canvas** | Note generation / canvas authoring (CE 10/11) | on-demand | ☐ |

### Phase 6 — Federation, second screen, infra
| # | Function | Purpose | Boot IPC | Gate | CP |
|---|---|---|---|---|---|
| 26 | **Federation** (cUniverse attach + warnings) | Federate child universes | `federation:ready`, `getFederationWarnings` (**unconditional → NEW**) | — | ☐ |
| 27 | **Second Screen** | Companion display | `open_second_screen`; **calls `setActiveUniverse` (bug)** | `secondScreen` | ☐ |
| 28 | **Five Acts notes** | Base host-notes sidebar section | `listFiveActsNotes` (**unconditional → NEW**) | (sidebar) | ☐ |
| 29 | **Workspace Bases** | `.base` lens/table blocks | `listWorkspaceBases` | (sidebar) | ☐ |
| 30 | **Style Setter / Command Palette / Settings / Importer / Universe Manager / Quick Capture** | UI infrastructure | on-demand | various | ☐ |

> Trivial UI primitives (Rename / Move / Collision / Canonical-Choice / Template-Prompt / Emoji-Picker / Page-Preview-hover / Lock-screen dialogs) are folded under their parent function — no separate concept paper.

---

## 5. Dependency-ordered Bring-Up Sequence

Each phase is re-enabled (flag on) and validated against its concept-paper checklist before the next. The second screen comes **last** on purpose (thinnest display + the double-init culprit).

0. **App shell** (always on) — window, i18n, theme, IPC dispatcher, stores.
1. **Core spine** — Editor (NotePane/FocusPane) + Tab bar + File tree + Properties + Outline + `boot_bundle` + `cache_boot_snapshot_core`. **← measure the editor-only baseline here.**
2. **Search + Index** — `cache_mark_search_ready`; Search Hub, Index, Quick Switcher.
3. **Backlinks / Graph / Tags** — **gated on MIG-079 landing** (persist + write-time-maintain; no boot recompute).
4. **Sky / Map / Sight / OrgChart** — visualization (mostly on-demand already).
5. **Knowledge curation & analysis** — Cataloger, CCS, Knowledge Health, Tasks, Calendar, Review/Tension/Provenance/Source, Forge/Canvas.
6. **Federation → Second screen → infra** — federate; then the second screen as a pure display (read-only `get_active_universe`, never re-activates).

---

## 6. Status

- **Method:** Boss-approved 2026-06-15. Flags-not-scissors confirmed (extends MIG-038 Wings pattern).
- **MIG-079** (Boot graph WTD + single-owner activation) — Architect done ([doc](../MIG-079-Architect-Boot-WTD-Graph-Snapshot.md)); Boss decisions captured (links: **defer off boot**; tags: **maintain in the indexer**; sequencing: **activation fix first**). It is the Phase-3 fix in this program.
- **Concept papers:** **ALL 31 per-function papers (02–32) written 2026-06-15** + the 3 foundation docs (00-Constellation core, 00-MASTER, 01-Editor). The ☐ in §4's CP column now tracks *bring-up* status (enabled + budget-met), **not** paper-written. Consolidated findings → §7.
- **`safeBootMode` flag:** SHIPPED (MIG-079 §B.1, `+layout.svelte` + `store.ts`) — gates the 4 satellite boot IPCs.
- **★ THE EDITOR BASELINE (measured 2026-06-15, minimal mode on the live 1.83 GB / 7,653-note universe) — the bring-up's regression reference:** `paint 452 ms · hydrated 588 ms · graph_ready 603 ms` (vs full mode `941 / 1671 / 32,519`). The graph/sky/federation/Five-Acts IPCs never fired. **The editor + file-tree spine boots in ~0.6 s** — the entire ~32 s was satellites. Every function re-enabled in §D must keep full boot close to this; §C removes the 30 s recompute so full boot approaches it.

---

## 7. Findings — the Debt Register (the concept-paper pass became an app-wide audit)

Writing one paper per function surfaced **systemic** debt across the app. This register is the prioritized work the bring-up pays down, function by function. Each item is grounded in code by the per-function paper; **all are to be re-verified live in bring-up** (Reproduce-First) before a fix lands.

### A. Rule 8 violations (recompute-on-read) — the core disease, confirmed systemic
The boot graph snapshot was **not** the only offender. Read-time recompute is the dominant pattern across satellites:
- **Graph snapshot** (Backlinks/Tags/Sky data) — 234k links + tags rebuilt per boot. → **MIG-079** (Phase 3).
- **File Tree** — stage emoji + maturity border recompute on every expand (`scan_note_stages` re-reads every `.md`).
- **Outgoing Links** — full link array filtered/sorted per tab switch.
- **Tags** — tag→count re-aggregated from `tags_json` per boot (the MIG-079 `tag_counts` target).
- **Constellation Map** — whole tree recomputed per open (`constellation_map_universe`).
- **Constellation Sight** — `sight_v6_get_layout` **rebuilds the ENTIRE `sight_v6_layout` table** from a universe-wide JOIN on every call (invalidate only DELETEs).
- **OrgChart** — tree recompute (MIG-078 is the active fix).
- **Inspector360** — two full disk walks per open.
- **Tasks / Global Tasks** — `scan_library_tasks` re-walks the library per open.
- **Calendar** — `scan_library_note_dates` full-tree walk per open.
- **Review Pulse** — `scan_due_recursive` reads every note per open.
- **Provenance** — full fs walk per note focus; **Tension** — report recomputed per focus.
- **Second Screen** — companion data re-walked on each load.
- **Federation** — re-attaches every cUniverse `search.db` + 10–15 s/cUniverse FTS5 pre-warm per boot.
> **The cure is one pattern** (the in-house triggers: notes_fts, sky_nodes, outgoing aggregates): persist + maintain at write time + resumable backfill + schema_versions gate. Each function's bring-up applies it.

### B. Right-click / context-menu gaps — MIG-077 is incomplete
- **Hand-rolled menus to fold into the shared `<ContextMenu>`:** Backlinks + Outgoing (duplicated confidence/archive popover), Sky View (`.gm-context-menu`), Workspace Bases (Delete bypasses `buildContextMenu`).
- **Missing entirely (add shared menu, or formally rule out):** Properties, Search Hub, Quick Switcher, Tags, Constellation Map, Sight, Inspector360, CCS, Knowledge Health, Tasks, Calendar, Review Pulse, Tension/Provenance/Source, Forge/Canvas, Federation, Second Screen, Style Setter, Universe/Library Mgmt. (Per the core paper's *right-clickable everywhere* contract.)

### C. Multilingual gaps (hardcoded English / unverified ×15) — against multilingual-by-default
Hardcoded English (tooltips/placeholders/labels bypassing `$t()`) found in: Tabs, Index, Backlinks, Outgoing, Tags, Sky View, Map, OrgChart, Inspector360, Calendar, Review Pulse, Forge/Canvas, Federation (`FederationWarning.reason` raw English from Rust), Second Screen, Command Palette, Style Setter, Universe/Library Mgmt. Plus pervasive `$t('key') || 'English'` inline fallbacks (Index, Settings, CCS, Knowledge Health) that mask missing-key gaps — every key must be verified present in all 15 locales.

### D. Missing feature gates (can't be flipped in minimal mode)
- **The 4 unconditional boot IPCs** (the `safeBootMode` targets, §2): `cache_boot_snapshot_graph`, the `federation:ready` listener, `listFiveActsNotes`, `getFederationWarnings`.
- **No `enabledFeatures` gate exists for:** **Search Hub** (the charter wrongly assumed `enabledFeatures.search` — the dock button is unconditional), Quick Switcher, Knowledge Health, Tasks, Tension/Provenance, Forge/Canvas, Second Screen, Style Setter.

### E. Confirmed defects the audit surfaced (verify, then fix — highest priority)
1. **Calendar — clicking a day always opens TODAY's note.** `get_daily_note_path` (`libraries.rs:4318`) uses `chrono::Local::now()` and ignores the clicked `dateStr`. *(Real functional bug.)*
2. **Tasks — `toggle_task` bypasses the Editor gate.** Writes via WriteGate directly without `write_note` + reindex → search/backlinks/tags drift after a checkbox toggle. *(Content-integrity-adjacent.)*
3. **Second Screen — `setActiveUniverse(...)` at `SecondScreenPage.svelte:923`** — a display re-activating the universe (the double-init culprit + Display-Not-Domain breach). Fixed by MIG-079's single-owner activation.
4. **Review Pulse — `record_note_visit` is dead code** (registered, never called); last-reviewed only advances via the explicit checkmark.
5. **Command Palette — 6 no-op command stubs** (toggle-bold/italic, insert-link, duplicate-line, toggle-comment, select-next) — wire or remove.

### F. Boss decisions the papers raised
- **Tasks (§2 self-justification):** a task list is a management/file-manager affordance — does it belong in a *formulation* system, or is it out of scope? (The paper flagged it can't cleanly trace to a Five Act.)
- **Quick Capture / Command Palette labels:** localize vs treat as brand names (the full-localization top-principal vs §A.15 brand-names-stay-English).
- **Read-only navigators (Outline, Quick Switcher, Properties):** right-click "none-OK" vs add a shared menu — confirm per surface.
