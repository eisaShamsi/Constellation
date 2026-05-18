# Constellation — Subsystem State Handover (2026-05-18)

**Written**: 2026-05-18, post-`75fc0be6` (the MIG-026 baseline SHIP gate). Pairs with the MIG-026-specific handover at `lab/reports/MIG-026-HANDOVER-2026-05-18.md`.

**Purpose**: a fresh AI session reads this to know the **current state of every core subsystem** — what's shipped, what works under Boss-verification, what's partial or fragile, and what surrounds it (dependencies, integrations, related Pending Jobs).

**What this doc is NOT**: the canonical project orientation (that's `docs/Constellation Orientation & Onboarding v2.13.md`). This is a current-state-of-subsystems snapshot, organized for fast intake.

---

## Read order on session open

1. `docs/Constellation Orientation & Onboarding v2.13.md` — canonical orientation; first read every session
2. **This document** — subsystem state snapshot
3. `lab/reports/MIG-026-HANDOVER-2026-05-18.md` — MIG-026-specific bridge (state of the just-shipped 24-tradition cascade + remaining ι/κ/λ/μ)
4. `docs/Constellation Pending Jobs v1.11.md` — canonical PJ list (51 jobs across 9 sections; new MIG-026 PJs to be filed as v1.12)
5. `lab/reports/SESSION-LOG-2026-05-18.md` + `lab/reports/SESSION-LOG-2026-05-17.md` — operational context (today + yesterday)
6. `CLAUDE.md` — standing operating rules (top-principals, working agreement, SO, basic rules)

---

## 1 · Sight — the universe visualization subsystem

**What it is**: the primary "see your whole universe at one glance" visualization. Polar dome + stratum rings + calendar rim + stars (per-note) + connector lines (per-link) + facet sidebar + tradition chip + mini-domes for diagnostic channels.

**Current canonical version**: **Sight v6** (`src/lib/sight/v6/`). Feature flag `SIGHT_V6_ENABLED = true` in `src/lib/sight/engine.ts:135`. Eisa's default Sight surface.

**Sight v2 / v3 / v4 / v5**: feature-flagged off in `engine.ts:131-134`. Preserved on disk as fallbacks; not user-reachable in production. Each was a stepping-stone:
- v2 — original force-directed PIXI graph (MIG-016 instant-toggle perf was here; partial-shipped per PJ-034)
- v3 — star-chart Lambert projection foundation (MIG-018 closed 2026-05-07; was the v3 trajectory per PJ-038 — superseded by v6)
- v4 — raw DOM event handler iteration (commit `3a977c8f`)
- v5 — Layer 2 diagnostic prototype (mentioned in earlier PJ doc preambles; superseded by v6)
- v6 — current. Built ground-up against `docs/Constellation-Sight-Concept-Paper-v4.0.md` (the canonical contract).

**Recently shipped** (last 4 days, all on `main`):
- **MIG-025** (2026-05-14): Sight v6 anchor dome + chrome + 5 strata + calendar rim + facet sidebar (Hearst-Flamenco cross-filter, 6 facets, Folder TOP) + first-boot tour + perf-harness skeletons. 14 commits §A.1 → §A.14, shipped with 7 NSIS Boss-test cycles + 16 fixes.
- **MIG-027** (2026-05-17): Sight follows the interface theme. Chrome/semantic color split in `dome.ts`. ChromePalette interface + `readChromePalette(el)` + dark-fallback const. `:global(body.theme-light) .sight-v6-root` override for semantic gold (theme-aware `--sight-highlight` CSS var). Boss-tested across all 6 themes (Constellation / Nord / Solarized × Light / Dark).
- **MIG-026** (2026-05-17 → 2026-05-18): tradition expansion. 24 curated baseline traditions + 9 of 9 shape renderers + per-shape star-size-boost + per-shape opacity treatment. See `MIG-026-HANDOVER-2026-05-18.md` for the full cascade detail.

**The 24 traditions across 10 family sections** (all live + Boss-verified):
- Western classical: Aristotelian
- Indian Nyāya: pramāṇa
- Sunni Islamic uṣūl: masādir
- Arabic / Islamic beyond uṣūl: Ibn Rushd burhān · Shāṭibī maqāṣid · Ibn Khaldūn ʿumrān
- Modern Western: Polanyi · Peirce · Habermas · Dewey · Husserl · Longino
- Jewish (Abrahamic): PaRDeS · Maimonidean prophecy · Talmudic 13 middot
- East Asian Confucian: Mencian sprouts · Wang Yangming · Korean Sŏngnihak
- Chinese pragmatist: Mohist sān biǎo
- Latin American decolonial: Mignolo pluriversal · Dussel transmodernity · Maldonado-Torres
- African philosophical: Akan Wiredu · Ibuanyidanda

**The 9 of 9 shape renderers** (all implemented in `src/lib/sight/v6/anchor.ts`):
sectoral · gradient · horizontal-bands · cyclic-flow · rings · grid · binary-flow (3 layout variants) · ladder · relational.

**Surrounding / integrations**:
- Theme system (MIG-027 chrome split)
- Search Hub (Shift+click facet → filter sync; star hover → facet highlight)
- Filename/identity architecture (notePath = canonical identity for star)
- Editor (click star → open note)
- AI not yet integrated into Sight (Phase κ.2 plugin loader will allow user TS plugins)

**Pending / fragile**:
- Phases ι (manifests + disclosure UI), κ (user-definable plugin loader), λ (translations for 15 locales), μ (ship gate + audit) — see `MIG-026-HANDOVER-2026-05-18.md`
- Per-tradition frontmatter integration (20 fields, Rust LayoutCacheRow extension — listed in handover)
- Concept Paper §4.1.2 (pramāṇa) + §4.1.3 (masādir) NE/SE/SW/NW description doc-drift after §δ.2-fix-1 + §θ-fix-1 rotations
- Sight v2 / v3 / v4 / v5 still on disk — cleanup MIG candidate when v6 stable across multiple sessions

---

## 2 · Editor — NotePane + FocusPane

**What it is**: the primary text-editing surface. CodeMirror 6 base + Constellation-specific extensions (livePreview, callouts, wikilinks, math, bidirectional text). Two modes:
- **NotePane** (`src/lib/components/NotePane.svelte`, 388 lines): full markdown editor with livePreview + syntax + callouts + property panel + RTL-aware bidiPlugin
- **FocusPane** (`src/lib/components/FocusPane.svelte`, 213 lines): plain text only, NO markdown parser, NO syntax highlighting, NO decorations. Capture ideas fast.

**Current state**: shipped + stable. CM6 extension stack lives in `$lib/editor/` and is imported by every editor instance per the **Editor Parity Rule** (orientation §0 / CLAUDE.md): standard NotePane and any future note types share the same extensions. FocusPane is the documented exception.

**Recent activity** (pre-Sight cascade):
- MIG-006 Wikilink Rename Cascade — §1, §2 + 11 cascade tests shipped; §3 expanded reverted at `5afe0c2` per BUG-015 (value-prop → CM6 doc sync `$effect` raced with `{#key}` onDestroy); §3 redo + §4-§11 still pending per orientation §8
- Properties panel reorder drag — broken per PJ-046

**Surrounding**:
- `store.ts` write-ahead buffer (memory + localStorage), navigation supersede tokens, recentWrites 2s gate, save coalescing — see orientation §7.4
- `secondScreen.ts` event API: 12 main→screen events, 4 screen→main, 1 bidirectional
- Per-note Properties Editor — `src/lib/components/PropertyEditor.svelte`. 2 pre-existing TypeScript errors at lines 236, 252 (`HorizontalNode | VerticalNode` union assignment — known, never blocked anything)

**Pending / fragile**:
- MIG-006 §3 redo (Wikilink rename cascade), §4-§11 — per orientation §8
- PJ-046 Properties panel reorder drag
- PJ-012 `LinkLifecycle.fresh` TS error in `store.ts:2470` — a Record<LinkLifecycle, number> missing `fresh` property; persistent pre-existing across this session and the prior one (counted as expected by every type-check)

---

## 3 · Sky View — graph visualization

**What it is**: a PIXI-rendered force-directed graph of notes and their links. Each note = a bubble; each link = a line. Layer view, community detection, structural gaps, universe-health metric.

**Current state**: shipped + Write-Time-Derived per CLAUDE.md Rule 8. `sky_nodes` and `sky_links` SQLite tables maintained by triggers on `note_meta` writes (`init_db` in `search.rs`). Reads are cheap lookups.

**Recently shipped**:
- MIG-001 (Sky View Write-Time Derivation) — closed
- LL-022 lazy-mount + always-mounted IPC fan-out trade-off ("$effect violation candidates flagged" per orientation §3.2)

**Surrounding**:
- Constellation Map (sister visualization — see §4)
- Backlinks / Outgoing Links panels (downstream consumers of link data)
- Brandes' betweenness centrality (`constellation_sight_centrality` IPC) — was for Sight v2/v3 analytical pipeline; status under v6 unclear (Sight v6 may not consume it)
- Louvain community detection (`constellation_sight_communities` IPC) — same

**Pending / fragile**:
- PJ-021 Sky View Write-Time Derivation audit (orientation v1.2 scope update; verify the trigger chain is actually maintaining state correctly under all write paths)
- Sidebar active-item highlight ~10 s lag — unresolved per orientation §17 history
- Auto-update-links toggle correctly under "Sky View & Links" Settings tab per orientation v1.2/v1.3 correction

---

## 4 · Constellation Map — sunburst visualization

**What it is**: D3 sunburst (radial concentric arcs) showing the universe's folder hierarchy + note distribution by hierarchical depth. Different conceptual view from Sky View — same data, different geometric grammar.

**Current state**: shipped. Has known issues per PJ-011 (perf/memory leak; tooltip shows canonical filename instead of human title; search doesn't highlight matched arcs).

**Surrounding**:
- Sky View (sister viz)
- Search Hub (search → highlight matched arcs is the broken behavior in PJ-011)
- Map ↔ Sight integration was once proposed (PJ-037) but **REJECTED** 2026-05-07 per Sight v3 Concept Paper review

**Pending / fragile**:
- PJ-011 Constellation Map open issues (perf, tooltip, search) — listed in orientation memory `project_constellation_map_backlog`

---

## 5 · 360.3D / Inspector 360 — per-note diagnostic matrix

**What it is**: per-note 12×3 stratification matrix showing the note's position across 12 epistemic dimensions × 3 strata each = 36 cells. Reveals the note's epistemic "shape" — where it concentrates, where it's empty.

**Current state**: shipped Stage 3 work per memory. Rust module `src-tauri/src/inspector360.rs`.

**Pending / fragile**:
- PJ-015 360.3D Stratification Matrix guidance doc — Boss-requested teaching doc explaining how to read/interpret the matrix (three reads: Position / Profile / Absence; mental shapes catalogue; matrix→action examples). Write after 360.3D work fully closes.

---

## 6 · Search Hub — search engine + diagnostic instrument

**What it is**: the search interface. Multiple search categories (text, title, content, properties, semantic, structural). Not "find a file"; framed in Constellation as a **diagnostic instrument for intellectual life** (orientation §4 + Concept Paper).

**Current state**: shipped. SQLite FTS5 backend; vocabulary at `notes_vocab` (fts5vocab virtual table). Triggers on `note_meta` keep FTS in sync. Concept search uses term embeddings (ONNX multilingual-e5-small).

**Recent activity**:
- MIG-022 cascade (Source classification — Sources/CECE — see §8 below) ran here partially; closed §A Gate 3 PASS per PJ v1.11 preamble.
- MIG-010 / MIG-011 / MIG-012 added `via {lemma}` and `≈ similar` badges to the Index panel filter (see §7).
- §1D-A wrong-target incident 2026-05-05: a Predecessor Lookup Rule violation that ended with restoring the IndexPanel filter after mistakenly building it in SearchHub. The "Predecessor Lookup Rule" was created out of this incident — read CLAUDE.md for the full rule.

**Surrounding**:
- Index Panel (term browser) — consumes FTS5 vocabulary
- Sources / CECE (UA Cataloger) — orthogonal classification layer on top of search
- AI / Embeddings — semantic search subset
- IPC contract: ≥300 ms debounce on search queries; cancel previous on new arrival; never invoke from a keystroke hot path

---

## 7 · Index Panel — term browser

**What it is**: sidebar panel that browses all terms used across the universe. Term frequency, semantic similarity, badge taxonomy.

**Current state**: shipped + actively-developed. Badges: T = Title, C = Content, P = Property, S = Semantic (and more — `docs/Badge-Taxonomy.md` is the canonical reference per orientation §13.1).

**Recent activity**:
- MIG-010 / 011 / 012 added `via {lemma}` and `≈ similar` badges
- §1D-A wrong-target incident (see §6) corrected after mistakenly building IndexPanel filter logic in SearchHub
- MIG-014 stage-taxonomy work has open follow-ups: PJ-028 / PJ-029 / PJ-030 / PJ-031 / PJ-032 / PJ-033

**Surrounding**:
- Search Hub (sibling — both read from FTS5)
- Sources / CECE (classification axis exposed via Index)
- AI / Embeddings (semantic similarity uses term embeddings)

**Pending / fragile**:
- PJ-028 through PJ-033 — six MIG-014 §2F audit follow-ups (leading-dash, multi-dash drift, stale custom_stages, trailing-dash, uppercase, NotePane badge dir="auto") — all P2/P3 polish, non-blocking
- PJ-017 / PJ-018 / PJ-019 / PJ-020 — index-related cleanup PJs (term_embeddings drop, semanticSearchEnabled flag drop, concept i18n keys cleanup, optional ≈ similar kill-switch)

---

## 8 · Sources / CECE Classification — UA Cataloger

**What it is**: epistemic source classification overlay. Each note can be classified by its sources (Sunni / Shia / Critical / Heard / etc. — per Sources Taxonomy) and content type. The "UA Cataloger" runs classifier passes; the "Source Review" UI exposes the results.

**Current state**: MIG-022 §A Gate 3 PASS (closed 2026-05-12). §B (Temporal axis — `note_state_history` table + Sight v3 overlay) — Rust foundation §B.1-§B.4 shipped; UI §B.5 contradicted-and-deferred because Sight v3 was retired in favor of Sight v6.

**Recently shipped**:
- 5 PJs closed during MIG-022 §0 + §D + §E + §A cascade:
  - PJ-040 (UA short-circuit per-axis dispatch)
  - PJ-041 (cataloger reasoning prose i18n)
  - PJ-042 (Confidence enum i18n)
  - PJ-043 (taxonomy node labels 15-locale backfill)
  - PJ-045 (composite_reasoning paren-dedup)

**Surrounding**:
- Search Hub (source filter cross-axis)
- Index Panel (classification axis exposed)
- Help / User Manual (epistemic metadata chapter shipped in 15 locales)

**Pending / fragile**:
- §B.5 UI deferred indefinitely — natural consumer is now Sight v5 Layer 2 / v6 trajectory but no concrete plan
- §B.6 i18n + help + UM deferred (would have driven §B.5 UI content)
- §N final integration audit + close-out — PARALLEL-READY per PJ v1.11 preamble
- PJ-044 (right-click Classify Sources menu entry missing — workaround via header button)
- PJ-046 (Properties panel reorder drag broken)
- PJ-047 (typed-link editor colors visually indistinguishable at body font size)
- PJ-048 (Type picker dropdown clips longer non-English translations)
- PJ-049 (In-app Help viewer not implemented — no F1 / command palette / ? menu)
- PJ-050 (Backlinks panel section header hardcoded English in non-en locales)
- PJ-051 (Mock B1 SVG follow-up as Sight v5 evolves — P3)

---

## 9 · Cognitive Engine + Living Link Architecture

**What it is**: the link layer. Each link is a first-class knowledge object with 8 properties (Type, Direction, Annotation, Weight, Confidence, Created, Last Traversed, Traversal Count) + 4 confidence levels (hypothesis / evidence / established / contested) + 7 link types (supports / contradicts / causes / exemplifies / generalizes / derives-from / part-of). Links earn weight through use (logarithmic growth on traversal; 5% monthly decay without use).

**Current state**: Living Link Architecture **P0-P5 ALL SHIPPED + user-validated** (per orientation §4.4 + PJ-006 closed 2026-05-06).

**Storage**:
- LINK files on disk: `YYYYMMDDTHHMMSSZ_LINK_XXXX.md` (source of truth)
- `note_links` SQLite table (fast index)
- Dual-layer; trigger-maintained sync

**Surrounding**:
- Sky View (consumes link data for graph)
- Constellation Map (consumes link data for hierarchy)
- Backlinks panel (per-note inbound links)
- Outgoing Links panel (per-note outbound links)
- Unlinked Mentions (alias-aware in-memory inbound — MIG-005 partial-shipped per orientation §8)

**Pending / fragile**:
- PJ-008 Outgoing Links typed-link duplication (alias bleed showing typed-link as both badge + plain text row)
- PJ-009 Backlinks typed-link duplication (Lunch Plan shows twice for Apple Tree Fruit — regular + supports)
- PJ-010 Unlinked Mentions frontmatter alias bleed
- MIG-005 Steps 4-8 pending (paused after fabrication caught — see orientation §8)
- MIG-006 §3 redo + §4-§11 pending (Wikilink Rename Cascade)

---

## 10 · Theme System

**What it is**: 6 built-in themes (Constellation / Nord / Solarized × Light / Dark) + custom themes. `appSettings.activeThemeId` controls the active theme; `deriveThemeVariables()` derives ~30 CSS variables from a 5-color palette per theme. Variables are written to `document.body.style` + `theme-light` / `theme-dark` body class set by `+layout.svelte`'s theme `$effect` (lines 1408-1514).

**Current state**: shipped + stable. MIG-027 wired Sight to honor the theme; previously Sight always rendered dark regardless of user theme.

**Recent activity**:
- MIG-027 (2026-05-17): Sight follows interface theme. 3 commits initial + fix-1 + fix-2. Boss-tested across all 6 themes.

**Surrounding**:
- Settings system (theme picker)
- All subsystems consume CSS vars; mostly clean
- Sight v6 consumes via `readChromePalette(canvasHostEl)` for canvas chrome

**Pending / fragile**:
- CNS (Constellation Nervous System) — likely has same dark-only assumption Sight had pre-MIG-027. Spawn_task scoped CNS out of MIG-027. Probable MIG-028 if requested. (New PJ candidate.)

---

## 11 · Settings System

**What it is**: hierarchical settings: app-wide (`appSettings` — interface font, text font, mono font, font size, scripts, theme, etc.), per-Universe (`.constellation/settings.json`), per-Library (libraryAppearances). Cross-window sync via `notifySettingsChanged()` event over Tauri.

**Current state**: shipped + stable.

**Surrounding**:
- Theme system (theme picker lives here)
- Second screen (settings must propagate)
- Sight v6 added 4 new fields in MIG-025 §A.12 settings migration

**Pending / fragile**:
- `store.ts:3483` literal-union duplicate of `TraditionId` — every TraditionId extension requires updating both `types.ts` AND `store.ts`. (New PJ candidate.)
- PJ-018 Drop `index.semanticSearchEnabled` settings flag

---

## 12 · Universe / Library / cUniverse Federation

**What it is**: the five-level knowledge hierarchy (Universe → cUniverse → Library → Folder → Note). Universe = portable directory with `.constellation/`. Library = self-contained vault-equivalent. cUniverse = federation of one or more child universes whose libraries get merged at runtime.

**Current state**: shipped + stable. Legacy migration (`migrate_legacy_data` in `universe.rs:1306`) handles old v1 layouts.

**Recent activity**:
- v2 migration shipped (universe.json relocated to `.constellation/`, vaults.json → libraries.json, registry to app_data_dir/universes.json)

**Surrounding**:
- Filename + Identity Architecture (notes named by human-name within Library context)
- Settings (per-Universe + per-Library)

---

## 13 · Arabic Engine + Lexical Bridge

**What it is**: a 5-layer morphological engine for Arabic text + a polylingual lemma graph that bridges Arabic-script lemmas to English / Persian / Urdu / Hebrew counterparts. mmap-baked FST for fast morphology lookup.

**Current state**: shipped + stable per orientation §5.

**Recent activity**:
- M-numbered milestones (M1, M2, ...) tracked separately from MIG-numbered work
- Lexical Bridge: 6 modules (`src-tauri/src/lexicon/`)
- Custom FTS5 tokenizer (`constellation`) for Arabic-aware search

**Surrounding**:
- Search Hub (Arabic-aware queries)
- AI / Embeddings (multilingual-e5-small handles Arabic alongside other scripts)
- Help / UM in Arabic (1328 lines per orientation §17, parity confirmed with other 14 locales)

---

## 14 · Filename + Identity Architecture

**What it is**: every file has a canonical identity (`YYYYMMDDTHHMMSSZ_KIND_XXXX.md` for notes; same pattern for LINKs and other kinds) + a human-name (the readable filename). Two ids, two purposes per orientation §6.1.

**Current state**: shipped per MIG-003 (Human-name Filenames, closed 2026-04-28).

**Recent activity**:
- MIG-003 closed 2026-04-28 (Steps 1-5 shipped, Step 6 PK promotion skipped by Boss decision, Steps 7-9 docs + audit + PCS shipped)
- 12 file kinds defined (orientation §6.3)
- `cid_cn` generator handles collisions

**Surrounding**:
- Wikilink resolution (alias-aware per MIG-004 closed; 9/12 invariants verified)
- New-note creation flow
- Cataloger reads canonical identity for note tracking

**Pending / fragile**:
- PJ-002 Pre-§140 `cid_cn` collision scrub utility
- PJ-003 Rename-collision popup (Override / Rename / Cancel) — silent refuse on rename now

---

## 15 · Help System + User Manual

**What it is**: per-topic help docs at `docs/help.<lang>.uConstellation.World/` for each of 15 locales + User Manual at `docs/User Manual.md` + `docs/help.<lang>/User Manual.md` for each locale.

**Current state**: 24 help topics in English (per orientation §17); 14 locales backfilled (orientation v1.2 confirms 1120 lines parity; ar is 1328 lines). PJ-014 closed scope.

**Recent activity**:
- All MIG cascades update help files per CLAUDE.md SO #2 ("Standing Order — update help files and User Manual with any user-facing changes")
- §A.4.a + §A.4.b in MIG-022 cascade backfilled Epistemic Metadata content across 15 locales

**Surrounding**:
- Every user-facing MIG must update help
- Standing Order #2 enforced

**Pending / fragile**:
- PJ-049 In-app Help viewer not implemented — no F1 / command palette / ? menu access surface; help files exist on disk but no in-app navigation
- PJ-050 Backlinks panel section header hardcoded English in non-en locales

---

## 16 · Boot Performance — 5 ship-gate criteria

**What it is**: boot must meet 5 perf criteria. Tracked per-boot in `<universe>/.constellation/boot-perf.latest.json`. Boot-bundle pattern (10 IPCs collapsed into 1 round-trip) — see orientation §9.3.

**Current state**: 4 of 5 criteria closed. Criterion 2 closed per orientation §9.1.

**Recent activity**:
- Boot bundle 10→1 IPCs
- LL-021 IPC arrival tracer (`perf_trace::record`) wraps `invoke_handler` to measure dispatch latency

**Surrounding**:
- IPC contract (zero `invoke()` calls on keystroke hot path)
- Settings + Universe load on boot

---

## 17 · Bases + Dataview + Importers

**What it is**:
- **Bases** (`bases.rs`): `.base` file format CRUD (read-time queries against universe data)
- **Dataview** (`dataview.rs`): DQL queries (read-time)
- **Importers** (`importers.rs`): 7 source formats (one-off async imports of external corpora)

**Current state**: shipped per orientation §3.

**Surrounding**:
- File watcher (`watcher.rs` — must be async, uses notify-rs)
- Universe / Library hierarchy (importers populate libraries)

---

## 18 · AI / Embeddings — ONNX subsystem

**What it is**: ONNX Runtime + multilingual-e5-small for semantic embeddings (write-time computed; cached in `note_embeddings` table). AI providers: OpenAI / Anthropic / Gemini / Ollama (`ai/mod.rs`).

**Current state**: shipped per orientation §3.

**Surrounding**:
- Search Hub (semantic search consumes embeddings)
- Index Panel (≈ similar badge from embeddings)
- Lexicon (polylingual lemma graph uses embeddings for cross-script bridge)

**Pending / fragile**:
- PJ-017 Drop orphaned `term_embeddings` table on existing DBs (legacy from earlier index design)
- PJ-018 Drop `index.semanticSearchEnabled` flag (consolidate gating)
- PJ-019 Drop `searchHub.concept` / `searchBadges.concept` i18n keys (concept feature retired)

---

## 19 · Second Screen — separate Tauri window

**What it is**: secondary Tauri window mounted via `static/screen.html`. Mirrors the main window's note view for presentation. Communicates via Tauri events (`emit`/`listen` from `@tauri-apps/api/event`).

**Current state**: shipped + stable. Event API: 12 main→screen events, 4 screen→main, 1 bidirectional (per orientation §7.5).

**Surrounding**:
- Editor (note content mirrored to second screen)
- Theme system (must apply to both windows)
- Settings (sync via notifySettingsChanged)

**Per CLAUDE.md "Display not Domain" rule**: second screen MOUNTS core components; never re-implements save/load/edit. Core editor handles all operations regardless of which window. No competing tab management.

---

## Cross-cutting standing rules (from CLAUDE.md)

These apply universally; the new session must respect them before any subsystem work:

1. **BASIC RULE — Don't Make Things Up.** Say "I don't know" when uncertain. No invented file paths / line numbers / function names / badge taxonomies / prior-art summaries.
2. **Working Agreement #2**: one location — `E:\مشاريع كلاود\Constellation` on branch `main`. Operate via absolute paths from any worktree.
3. **Working Agreement #4**: don't ship changes without validating against full architecture. Spawn parallel agents for cross-cutting risk reviews.
4. **Working Agreement #5**: cross-check every non-trivial fix against proven methods (WebSearch industry patterns) before applying.
5. **Top principal — State the function in hand**: one-line anchor naming the active feature. Precedes every other rule. Re-fire on session start, pivot, correction, or multi-day resume.
6. **Top principal — Predecessor Lookup Rule**: before removing/relocating any feature, write a Predecessor → Replacement entry into the session log. Default: replacement lives in the same place as predecessor.
7. **Top principal — Stop-On-Correction Rule**: when Boss says "wrong target", "you're confused", "no", "unacceptable", or any equivalent — STOP all in-flight edits, summarize what's changed since last approval, state corrected understanding, wait for "proceed".
8. **Top principal — Plan Approval = Build Approval**: cascade autonomously through approved plan steps. Stop only at user-testable verification clauses, genuine architectural surprises, or plan completion.
9. **Top principal — Testing Instructions Rule**: every Boss test is a tutorial. Define feature first, then walk through every interaction step in plain language. Pre-state, action, post-state. Failure modes spelled out.
10. **Standing Order #1**: log progress after every phase to `lab/reports/SESSION-LOG-YYYY-MM-DD.md`.
11. **Standing Order #2**: update help files + User Manual (all 15 locales) with any user-facing changes.
12. **Standing Order #5**: state-of-standing record before any pivot or major triage.
13. **Standing Order #6**: orientation v-bump lands in the SAME commit as any SO #6 trigger.
14. **Standing Order #7**: MoCh every ~3 hours of direct Boss↔Claude chat. `docs/MoCh/MoCh-YYYY-MM-DD-HHMM.md`.
15. **Standing Order #8**: cross-check any PJ before tackling it. Read orientation body (not just preamble) + relevant session log. Don't act on a stale PJ.
16. **Performance Rule 8 — Write-Time Derivation**: every computed view is maintained at write-time, not read-time. Never write a `scan_*` command that re-walks the universe to produce a derived view.

---

## Quick subsystem-state matrix

| Subsystem | Status | Last MIG | Pending PJs |
|---|---|---|---|
| **Sight v6** | ✅ Stable | MIG-025/026/027 (this week) | MIG-026 Phases ι/κ/λ/μ + audit; PJ for per-tradition frontmatter, CNS theming |
| Editor (NotePane / FocusPane) | ✅ Stable | MIG-006 (partial — §3 reverted) | MIG-006 §3 redo + §4-§11; PJ-012 fresh dedupe; PJ-046 Properties drag |
| Sky View | ✅ WTD-shipped | MIG-001 (closed) | PJ-021 WTD audit |
| Constellation Map | ⚠️ Functional + known issues | n/a | PJ-011 perf/tooltip/search |
| 360.3D / Inspector 360 | ✅ Stable | n/a recent | PJ-015 guidance doc |
| Search Hub | ✅ Stable | MIG-022 §A | PJ-005 Links Settings tab, MIG-022 §N close |
| Index Panel | ✅ Stable | MIG-010/011/012 | PJ-028→033 stage taxonomy follow-ups |
| Sources / CECE | 🟡 §A shipped, §B partial | MIG-022 §A | §B.5/§B.6 deferred, §N audit, 8 PJs filed |
| Cognitive Engine + Living Link | ✅ P0-P5 shipped | MIG-006 (link cascade) | PJ-008/009/010 typed-link dedupe |
| Theme System | ✅ Stable | MIG-027 (this week) | CNS theming (new PJ candidate) |
| Settings System | ✅ Stable | MIG-025 §A.12 (this week) | store.ts TraditionId dedup (new PJ) |
| Universe / Library / cUniverse | ✅ Stable | universe.rs v2 migration | n/a |
| Arabic Engine + Lexical Bridge | ✅ Stable | M-numbered milestones | n/a |
| Filename + Identity | ✅ Stable | MIG-003 (closed) | PJ-002 collision scrub, PJ-003 rename popup |
| Help System + UM | 🟡 In-app viewer missing | MIG-022 §A.4 backfill | PJ-049 in-app Help viewer, PJ-050 Backlinks header i18n |
| Boot Performance | 🟢 4/5 closed | n/a recent | Criterion 5 (last one) |
| Bases + Dataview + Importers | ✅ Stable | n/a recent | n/a |
| AI / Embeddings | ✅ Stable | n/a recent | PJ-017/018/019 cleanup |
| Second Screen | ✅ Stable | n/a recent | n/a |

**Legend**: ✅ Stable · 🟡 Partial/known-issues · 🟢 Mostly done · ⚠️ Known issues · 🔴 Broken / blocked

---

End of subsystem-state handover.

For MIG-026 specifics (24 traditions just shipped, remaining ι/κ/λ/μ phases): see `lab/reports/MIG-026-HANDOVER-2026-05-18.md`.

For the canonical full PJ list: `docs/Constellation Pending Jobs v1.11.md` (51 jobs across 9 sections).

For the canonical orientation: `docs/Constellation Orientation & Onboarding v2.13.md` (4261 lines covering hierarchy, architecture, CE, Arabic Engine, filename architecture, editor, migrations, boot perf, standing rules, Lessons Learned, drift list, badge taxonomy, where-to-read-what).
