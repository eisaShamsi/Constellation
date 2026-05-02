# Constellation — Living Links Guide

**Version 1.0 | 2026-05-02**

**Author of facts**: Eisa ALSHAMSI (project owner, designer, IT Boss)
**Maintainer**: Claude (consultant / engineer / SME)

This guide is the practical reference for how links work in Constellation today. It is grounded in the running code as of 2026-05-02; every non-obvious claim cites a `file:line` source. The philosophical foundation lives in [`docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`](docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md) (the spec / design doc), and this guide reads against the actual codebase to mark what's shipped, what's partial, and what's still proposed.

---

## Table of contents

1. [Why Constellation calls them "living"](#1-why-constellation-calls-them-living)
2. [The 7 typed directions (cognitive vocabulary)](#2-the-7-typed-directions-cognitive-vocabulary)
3. [Untyped wikilinks](#3-untyped-wikilinks)
4. [Wikilink syntax — what works today](#4-wikilink-syntax--what-works-today)
5. [The 8 properties every link carries](#5-the-8-properties-every-link-carries)
6. [Confidence levels (4 tiers)](#6-confidence-levels-4-tiers)
7. [Weight — earned through use](#7-weight--earned-through-use)
8. [The lifecycle — `fresh → emerging → established → load-bearing → stale`](#8-the-lifecycle--fresh--emerging--established--load-bearing--stale)
9. [Dual-layer storage — LINK files + `note_links` table](#9-dual-layer-storage--link-files--note_links-table)
10. [Reversibility — archival, not deletion](#10-reversibility--archival-not-deletion)
11. [Settings that govern link behaviour](#11-settings-that-govern-link-behaviour)
12. [Links and Sky View](#12-links-and-sky-view)
13. [Links and the 360.3D Inspector](#13-links-and-the-3603d-inspector)
14. [The Five Acts of Knowledge Creation](#14-the-five-acts-of-knowledge-creation)
15. [Search as a diagnostic instrument](#15-search-as-a-diagnostic-instrument)
16. [State of the architecture — what's shipped, what's pending](#16-state-of-the-architecture--whats-shipped-whats-pending)
17. [Known divergences between spec and code](#17-known-divergences-between-spec-and-code)
18. [Open questions and deferred fixes](#18-open-questions-and-deferred-fixes)

---

## 1. Why Constellation calls them "living"

Most knowledge tools treat a link as a pointer — a string in one file that names another file. Click it, jump there. The link itself carries nothing.

Constellation treats a link as a **first-class knowledge object**. It has its own identity, its own state, its own history, and its own lifecycle. It accumulates significance through use, decays without it, and surfaces signals about how your thinking is moving.

The design doc ([`CONSTELLATION-KNOWLEDGE-FORMULATION.md`](docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md)) frames this as a biological metaphor:

- **Notes are neurons** — knowledge cells that receive, process, and transmit.
- **Links are synapses + blood vessels** — the synapse carries the typed signal (this note *contradicts* that one); the vessel keeps the connection alive through circulation.
- **The Cognitive Engine is the heart** — pumping the analytical layer that makes the network make sense.

The slogan from the design doc:

> A note without links is an observation.
> A note with links is knowledge.
> A network of typed links is understanding.
> Understanding that survives contradiction is wisdom.

Constellation is not a Personal Knowledge **Management** tool. It is a Personal Knowledge **Formulation** tool. The link layer is where formulation happens — by typing, weighting, contradicting, and synthesising. The rest of this guide is the mechanics underneath that claim.

---

## 2. The 7 typed directions (cognitive vocabulary)

Seven typed-link directions form Constellation's cognitive vocabulary. Every link is either one of the seven or untyped (§3 below). The seven are defined and parsed throughout the stack — see [`store.ts:1750-1753`](src/lib/libraries/store.ts:1750) for the canonical client-side set, mirrored in `src-tauri/src/libraries.rs::KNOWN_LINK_TYPES`.

| Type | Arabic (per `inspector360.link_*`) | What it asserts | Use it when… |
|------|---|---|---|
| `supports` | يدعم | This note **strengthens** another note's claim. | The target's argument gains weight from what this note contains. |
| `contradicts` | يناقض | This note **disputes or challenges** another note's claim. | You're explicitly challenging the target. Tracking these is intellectual honesty: every `supports` without a `contradicts` is one-sided thinking. |
| `causes` | يسبب | A → B. This note describes a **directional cause-and-effect** with the target. | There's a real arrow of cause, not just correlation. |
| `derives-from` | مشتق من | This note's reasoning is **based on** another note. The target is the source. | Tracking provenance and lineage. The trust-depth number `dN` in the dimension strip counts how deep the chain runs to a root. |
| `generalizes` | يعمم | This note **abstracts upward** from the target — it draws a broader pattern from the target's specifics. | The target is an instance; this note is the pattern. |
| `exemplifies` | يمثل | This note is an **instance** of a broader idea in the target. | The opposite direction of `generalizes`. |
| `part-of` | جزء من | This note is a **component** of a larger idea, system, or hierarchy in the target. | Structural composition, not logical inference. |

**Why these seven, and not five or twelve.** The design doc Part II.4 frames each as a distinct **cognitive act**: `supports` is argument-building, `contradicts` is critical thinking, `causes` is causal reasoning, `derives-from` is provenance, `generalizes` and `exemplifies` are the two arrows of abstraction, `part-of` is decomposition. Seven distinct cognitive acts that recur across every domain of thought.

**Inverse pairs.** `generalizes` and `exemplifies` are direct inverses (A generalizes B ↔ B exemplifies A). `causes` has no perfect inverse — Constellation does not ship `caused-by` because in practice `derives-from` covers that direction. `supports` and `contradicts` are not inverses of each other; they are two different stances toward the same target.

---

## 3. Untyped wikilinks

Plain wikilinks — `[[target]]` with no type — are not a defect. They're a starting state.

**What "untyped" means cognitively.** The connection exists; you haven't yet committed to *what it means*. You sensed a relationship and named it, but didn't declare whether the target supports your argument, contradicts it, derives from it, or anything else.

**Why mature thinking drifts toward typed forms.** A note where every connection is untyped is a note where the *relational geometry* hasn't been declared. The 360.3D matrix surfaces this directly: a column where Untyped is much larger than every typed total combined is a note whose connections exist on autopilot. Typing a link is a small cognitive act of commitment.

**Why Constellation displays Untyped as its own column** (in the matrix, in panels, etc.) rather than hiding it. The signal is honest. Your library has whatever ratio of typed-to-untyped you've actually chosen. Hiding Untyped would falsify the picture; showing it makes the gap legible and actionable.

A practical pattern: write `[[target]]` first when capturing the connection, then come back during a review pass and upgrade to `[[target|supports]]` (or whichever) once you've decided what the relationship asserts.

---

## 4. Wikilink syntax — what works today

What the parser actually accepts ([`store.ts:1761-1786`](src/lib/libraries/store.ts:1761) shows the disambiguation logic):

| Syntax | Parsed as | Example |
|---|---|---|
| `[[target]]` | Plain wikilink — Untyped | `[[Soil Health Principles]]` |
| `[[target\|alias]]` | Wikilink with display alias (when the post-pipe text is **not** a known type) | `[[Soil Health Principles\|that note about soil]]` |
| `[[target\|type]]` | **Typed** link (when the post-pipe text **is** one of the seven types or `associative`) | `[[Soil Health Principles\|supports]]` |
| `![[target]]` | Embed (transcludes the target's content) | `![[Soil Health Principles]]` |

**The dual role of the `|` field.** It's used for both alias and type. The disambiguation is by lookup against `KNOWN_LINK_TYPES` ([`store.ts:1750`](src/lib/libraries/store.ts:1750)). If the post-pipe text matches a known type, it's stored as the link's `annotation` field and promoted to a typed badge by `displayLinkType()` ([`store.ts:1761`](src/lib/libraries/store.ts:1761)). Otherwise it's treated as an alias.

**Consequence: you cannot today write a typed link AND a free-form annotation in the same wikilink.** `[[Note|supports]]` types the link; `[[Note|My reasoning]]` annotates it. Combining them would require a new syntax (the design doc proposes `[[type::target|reason]]`, but that has not landed in the parser as of 2026-05-02).

**Aliases and typed links interact during dedupe.** A note like *Lunch Plan* can contain BOTH `[[X]]` (plain) AND `[[X|supports]]` (typed) targeting the same target — the database stores both rows, then the panel-level `dedupeBySource()` ([`store.ts:1840`](src/lib/libraries/store.ts:1840)) collapses them into one row carrying every distinct typed badge. This is how the Backlinks / Outgoing panels avoid double-counting (the §89 fix; see [SESSION-LOG-2026-04-27.md](lab/reports/SESSION-LOG-2026-04-27.md)).

---

## 5. The 8 properties every link carries

| # | Property | Type | Where it's set | Where it's stored |
|---|---|---|---|---|
| 1 | **Type** | one of the 7 typed names or untyped | parsed from the wikilink's post-pipe slot | `annotation` field in `note_links`; promoted to badge via `displayLinkType()` |
| 2 | **Direction** | outgoing or incoming (per-row perspective) | implicit from `source_path` vs `target_path` | indexed in both directions in `note_links` |
| 3 | **Annotation** | free-form text — author's WHY for this link | when present and **not** a known type, stored verbatim | `annotation` field; rendered in panels as italic prose under the badge |
| 4 | **Weight** | numeric, accumulates with traversal | computed at write time from `traversal_count` | `weight` column. Formula per [`store.ts:1717`](src/lib/libraries/store.ts:1717): `weight = 1 + ln(1 + traversal_count)` |
| 5 | **Confidence** | one of `hypothesis` / `evidence` / `established` / `contested` | user-set via `setLinkConfidence()` ([`store.ts:1110`](src/lib/libraries/store.ts:1110)); auto-promoted by traversal-threshold backfill (see §6) | `confidence` column |
| 6 | **Created** | ISO-8601 timestamp | written at link birth | `created` column |
| 7 | **Last Traversed** | ISO-8601 timestamp or empty | updated on click-through (intent: never on every render) | `last_traversed` column |
| 8 | **Traversal Count** | integer, monotonically increasing under normal use | incremented on click-through | `traversal_count` column |

Two of those eight are **stable identity** (Type, Direction). Three are **author-set context** (Annotation, Confidence, Created). Three are **earned-through-use signals** (Weight, Last Traversed, Traversal Count) — the temporal layer that makes a link "alive" rather than static.

---

## 6. Confidence levels (4 tiers)

`LinkConfidence` is defined at [`store.ts:1103`](src/lib/libraries/store.ts:1103):

```ts
export type LinkConfidence = 'hypothesis' | 'evidence' | 'established' | 'contested';
```

| Level | Meaning | When you'd set it | Visual treatment (per design doc Part II.5) |
|---|---|---|---|
| `hypothesis` | "I think this might be true." | Speculative connection, hunch, or initial draft. | Dashed line. |
| `evidence` | "I have some basis for this." | Supported by data, by another note, or by traversal record. | Solid line. |
| `established` | "This is well-supported." | Accumulated traversal + multiple corroborations. | Thick solid line. |
| `contested` | "This is actively debated." | User flag — reflects ongoing tension, even if traversal count is high. | Red pulsing line. |

**Auto-promotion.** A backfill helper `backfillLinkConfidence` (referenced at [`store.ts:1114`](src/lib/libraries/store.ts:1114)) auto-promotes hypothesis → evidence at traversal count ≥ 3, and evidence → established at ≥ 10, **without ever downgrading** and **without overwriting `contested`**. Contested is a user-set override; the system never silently revises it.

**Why four and not three.** `contested` is not just "lower than established" — it's an entirely different axis. A heavily-traversed link can still be contested: high traffic doesn't make it correct.

---

## 7. Weight — earned through use

A link's weight is **not** declared by the user. It accumulates from traversal. Frequently used links become the arteries of the user's thinking.

**Accumulation formula** ([`store.ts:1717`](src/lib/libraries/store.ts:1717)):

```
weight = 1 + ln(1 + traversal_count)
```

A logarithmic curve — early traversals matter most:

| `traversal_count` | `weight` |
|---|---|
| 0 | 1.000 |
| 1 | 1.693 |
| 3 | 2.386 |
| 5 | 2.792 |
| 10 | 3.398 |
| 50 | 4.932 |
| 100 | 5.615 |

Diminishing returns past ~10 traversals. A link traversed 100 times isn't 100× more important than one traversed 1 time — it's about 3× more important.

**Decay formula** ([`store.ts:1714, 1724-1744`](src/lib/libraries/store.ts:1714) — `effectiveLinkWeight()`):

```
effectiveWeight = weight × exp(-ln(2) × daysSinceTraversal / halfLifeDays)
```

This is **not stored** — it's computed at read time, every time. Two reasons:

1. **Threshold-tuning takes effect immediately.** Move the half-life slider from 60 to 30 days, and every sort updates instantly without a migration.
2. **Ground truth is preserved.** The raw `weight` column is the integral of all the user's traversals — it never loses fidelity against a future revisit. Decay is purely a display/ordering concern.

**Defaults:**
- `halfLifeDays = 60` — an untouched link's effective weight halves after 60 days.
- `decayEnabled = true` — when off, links sort by raw traversal count only.

Both are user-tunable in Settings → Appearance → "Living Link Lifecycle" ([`SettingsModal.svelte:2106-2129`](src/lib/components/SettingsModal.svelte:2106)).

**Where decay is applied vs ignored.** Decay applies to **sort order** in the Backlinks / Outgoing / Most-Traveled panels, and to display tier-coloring. It does NOT apply to the raw `weight` value stored in the database; that integral remains intact.

---

## 8. The lifecycle — `fresh → emerging → established → load-bearing → stale`

Five tier values, computed at read time from `traversal_count` and `last_traversed`. The function is `linkLifecycle()` at [`store.ts:1690-1709`](src/lib/libraries/store.ts:1690):

| Tier | Condition (verbatim from code) | Meaning |
|---|---|---|
| `fresh` | `traversal_count === 0` | Never traversed. The link exists, but has not yet been used. |
| `emerging` | 1 ≤ `traversal_count` < 3, AND traversed within `LINK_STALE_DAYS` (90) | Just-found path. Used once or twice; recent. |
| `established` | 3 ≤ `traversal_count` < 10, AND traversed within 90 days | Recurring path. Used regularly; recent. |
| `load-bearing` | `traversal_count` ≥ 10, AND traversed within 90 days | Critical path. Used often; recent. |
| `stale` | `traversal_count` ≥ 1, AND last traversed > 90 days ago | Decay branch. Was active once; hasn't been touched in three months. |

**Three observations from the code:**

1. The `stale` check happens **before** the `tc ≥ 10` check, so a link with 50 traversals 100 days ago resolves to `stale` — not `load-bearing`. Recency wins over volume.
2. `LINK_STALE_DAYS` is a single source of truth at [`store.ts:1680`](src/lib/libraries/store.ts:1680): `90`. Changing this constant changes the staleness boundary across the app.
3. The classification is **pure** — given the link's fields and the current time, no DB writes, no side effects. Tier is recomputed on every read, so threshold changes apply instantly.

**The relationship between tier and the lifecycle stages in the design doc** (Part IV.1 names six stages: Spark → Birth → Growth → Maturity → Dormancy → Renewal/Archival). The five computed tiers are an implementation projection of that conceptual lifecycle:

| Design-doc stage | Computed tier | Notes |
|---|---|---|
| Spark | (pre-link) | Not yet a link; semantic suggestions at `[[` autocomplete. |
| Birth | `fresh` | Link committed, never traversed. |
| Growth | `emerging` → `established` | Each traversal moves the link forward through the curve. |
| Maturity | `load-bearing` | High traversal count, recent activity. |
| Dormancy | `stale` | No traversals for 90+ days. |
| Renewal | `emerging` (rebound) | A user revisit on a `stale` link snaps it back to `emerging` (depending on tc). |
| Archival | (out-of-band) | User-driven; archived links are hidden from panels but preserved on disk (see §10). |

---

## 9. Dual-layer storage — LINK files + `note_links` table

Constellation's link layer lives in two places. The **disk** is the source of truth; the **database** is a fast index.

```
DISK (source of truth)
├── LINK files: YYYYMMDDTHHMMSSZ_LINK_XXXX.md
│   └── frontmatter carries all 8 properties
└── Notes: any wikilink in any note's body is also persisted

         │ indexed into
         ▼

SQLITE (fast index, rebuildable)
└── note_links table
    └── source, target, type, annotation, weight,
        confidence, created, last_traversed,
        traversal_count, status
```

**Why both.**

- The disk layer respects the **File Over App** principle — every link is a file you can read, port, version, or back up without Constellation running. Designed to outlive the app.
- The SQLite layer makes panel queries instant. Backlinks / Outgoing / Sky View / 360.3D Matrix all read from `note_links` joins. Per-link metadata updates (traversal_count++) hit SQLite, not files — no per-click I/O.
- The two stay in sync via a **periodic write-back**: traversal/weight data accumulates in SQLite at runtime; the LINK files on disk are updated periodically (not on every click).

**`note_links` schema columns** (referenced across [`libraries.rs`](src-tauri/src/libraries.rs), [`cache.rs`](src-tauri/src/cache.rs), and [`search.rs`](src-tauri/src/search.rs)):

- `source_path`, `target_path` — both indexed
- `target_name` — for resolution at scan time
- `link_type` — one of the 7 typed names or empty (untyped)
- `annotation` — the post-pipe text from the wikilink
- `weight`, `confidence`, `created`, `last_traversed`, `traversal_count`
- `status` — `'active'` (default) or `'archived'` (see §10)

**The index is rebuildable.** If `note_links` ever diverges from disk truth, deleting the table and re-scanning all LINK files reconstructs it. This is how MIG-003 (canonical filename rename) cascaded — the rename touched the disk LINK files first, then the index reflected it through a re-scan.

---

## 10. Reversibility — archival, not deletion

Every link operation is reversible. When a user "removes" a link, Constellation does not delete the row — it sets `status = 'archived'` ([`store.ts:1660-1662`](src/lib/libraries/store.ts:1660)).

**What archived means in practice:**

- Archived links are **hidden** from Backlinks / Outgoing / Most-Traveled panels.
- They remain in the `note_links` table (with `status = 'archived'`).
- They remain in the LINK file on disk.
- They can be restored without losing any of the 8 properties or the traversal history.

This implements the design doc's principle that archival is **apoptosis, not necrosis** — the link's history is part of the user's intellectual record, even after the link is no longer active in the visible graph.

---

## 11. Settings that govern link behaviour

Two link-related blocks in [`SettingsModal.svelte`](src/lib/components/SettingsModal.svelte). One affects how new links are created from rename cascades; the other governs the lifecycle decay model.

### 11.1 Files → Auto-update Links

`autoUpdateLinks` ([SettingsModal.svelte:1420-1426](src/lib/components/SettingsModal.svelte:1420)) — when on, renaming a note cascades `[[OldName]]` → `[[NewName]]` across every note that references it. When off, renames silently leave dangling references.

The placement of this toggle is currently miscategorised under "Sky View & Links" — there's a backlog item to consolidate every link-related control into a dedicated "Links" Settings tab (`MIG-007` per project memory).

### 11.2 Appearance → Living Link Lifecycle

[SettingsModal.svelte:2106-2129](src/lib/components/SettingsModal.svelte:2106) — two controls.

| Setting | Default | What it does |
|---|---|---|
| `linkLifecycle.decayEnabled` | `true` | When on, the Backlinks / Outgoing / Most-Traveled panels sort by **effective** weight (with decay). When off, sort by raw `weight` only. |
| `linkLifecycle.halfLifeDays` | `60` | Days after which an untouched link's effective weight halves. Lower = faster drop-off; higher = slower. Slider control, disabled when decay is off. |

**Important: decay is a display concern only.** Whatever the user sets here, the underlying `weight` and `traversal_count` columns in `note_links` are unaffected. Tuning the half-life slider takes effect immediately because every sort recomputes `effectiveLinkWeight()` at read time.

---

## 12. Links and Sky View

Sky View renders the universe-level link graph. The matrix uses `note_links` rows directly:

- **Bubbles** are notes (size driven by note metadata, not links).
- **Edges** are typed wikilink rows from `note_links`. Each edge inherits the link's typed `link_type` (when present) — Sky View styling treats typed and untyped edges differently.
- **Edge thickness** scales with `effectiveLinkWeight()` (subject to the user's decay setting).
- **Edge styling** can reflect the lifecycle tier (the LinkDashboard surfaces this; Sky View's edge-styling integration is partial — see §16).

**Alias awareness.** When a note's title resolves through an alias (for renamed notes), Sky View's edge population must be alias-aware or it silently drops edges. The §88 fix ([SESSION-LOG-2026-04-27.md](lab/reports/SESSION-LOG-2026-04-27.md)) closed this regression.

---

## 13. Links and the 360.3D Inspector

The Inspector's **Stratification Matrix** (§112 onward) is where Constellation's link semantics meet visual cognition. Per the [360.3D concept paper](docs/360.3D-Concept-Paper-v1.0.md), the matrix encodes:

- **Vertical axis (rows)**: stratum — the 8-level hierarchy from L1 Datum to L8 Worldview.
- **Horizontal axis (columns)**: the 7 typed link directions plus Untyped.

Every connected note becomes a coloured dot in the cell where its **own stratum** meets the **typed direction it shares** with the active note.

**The recently-shipped backend optimisation** (`precompute_all_strata` in [inspector360.rs](src-tauri/src/inspector360.rs:357)) computes every note's stratum in one O(N + total_links) pass at the top of `get_360_view`, so each `LinkedNote` arrives at the frontend with `stratum: u8` already stamped.

**Blind-spot detection** (§122). When a typed column's total is zero, the column header is rendered with a `var(--text-error)` warning treatment — the user can spot the missing direction at a glance instead of reading the row of count totals. Untyped is excluded from blind-spot flagging because its zero means "no plain wikilinks", not "an untyped direction is missing".

**Read-time aggregation.** The IPC `get_360_view` walks the link graph for the active note on every call — a Rule-8 violation that's been monitored for performance. Boss's lived verdict on the 7,600-note Universe is "first fetch almost instantly," so MIG-010-scale write-time caching has been deferred to LOW priority.

---

## 14. The Five Acts of Knowledge Creation

The design doc's framing for how knowledge formulates over time. Each act maps to a phase of link activity.

| Act | Name | What happens | Link's role |
|---|---|---|---|
| I | **Observation** (المُلاحظة) | A note is born. | No links yet. |
| II | **Connection** (الربط) | First link created. | The first heartbeat. Knowledge begins to flow. Often `supports` or `derives-from`. |
| III | **Tension** (التوتر) | Contradiction discovered. | A `contradicts` link is added. White blood cells activate. The system detects conflict. |
| IV | **Synthesis** (التركيب) | New understanding emerges that resolves the tension. | A `generalizes` link or a higher-stratum note connects the contested nodes. Knowledge is created at L4-L7. |
| V | **Conviction** (الاقتناع) | Evidence accumulates. | Weight grows, confidence auto-promotes hypothesis → evidence → established. The idea becomes load-bearing. |

The sequence isn't linear — most notes oscillate between Acts II–V many times — but the framing makes the cognitive role of each link type explicit. `supports` belongs to Acts II and V. `contradicts` belongs to Act III. `generalizes` belongs to Act IV.

---

## 15. Search as a diagnostic instrument

The design doc Part V.3 frames Constellation's search engine as a **stethoscope for intellectual life**. Cognitive search operators query the link layer directly:

| Operator | What it asks | What it returns |
|---|---|---|
| `links to [[X]]` | What flows TO X? | Notes that wikilink to X (any type, any direction). |
| `links from [[X]]` | What flows FROM X? | Notes X wikilinks to. |
| `mutual [[X]]` | Where is the heartbeat double? | Notes that link to X AND that X links to. |
| `orphans` | What has no inbound? | Notes nothing references. |
| `links between [[X]] and [[Y]]` | Where do these two intersect? | Common neighbours of X and Y. |
| `links all [[X]] [[Y]] [[Z]]` | What surrounds this set? | Multi-target topology. |

These are **topology** queries — they read the graph without typed-link semantics. Cognitive queries — `supports [[X]]`, `contradicts [[X]]`, `causes [[X]]` — return only links of that type pointing at the target.

The 15-language search-operator translation is in each locale's `searchOps` block (`getSearchOps()` at [`i18n/index.ts:118`](src/lib/i18n/index.ts:118)).

---

## 16. State of the architecture — what's shipped, what's pending

The design doc's Part VII Implementation Priority Map planned six phases (P0–P5). Status as of 2026-05-02:

| Phase | What | Status |
|---|---|---|
| **P0** | Link storage table in SQLite | ✅ Shipped (`note_links` table) |
| **P1** | Annotation as the typed-link slot (`[[target\|type]]`) | ✅ Shipped (parser disambiguates against `KNOWN_LINK_TYPES`) |
| **P1** | 7 cognitive search operators in 15 languages | ✅ Shipped (`searchOps` per locale) |
| **P2** | Traversal tracking | ✅ Shipped (`traversal_count`, `last_traversed`) |
| **P2** | Confidence levels | ✅ Shipped (4 tiers; auto-promotion via `backfillLinkConfidence`) |
| **P3** | Weight accumulation | ✅ Shipped (`weight = 1 + ln(1 + tc)`) |
| **P3** | Weight decay | ✅ Shipped (`effectiveLinkWeight` with `halfLifeDays`) |
| **P3** | Lifecycle stages | ✅ Shipped (`linkLifecycle` returns 5 tiers) |
| **P4** | Formulation analysis (strongest evidence, knowledge gaps, tensions) | ⚠️ Partial — surfaces in 360.3D Inspector + LinkDashboard, full query surface pending |
| **P4** | Spark detection (semantic suggestions at `[[`) | ⚠️ Partial — autocomplete works structurally; semantic ranking exists via embeddings |
| **P5** | Knowledge health dashboard | ⚠️ Partial — `LinkDashboard.svelte` exists; full circulatory-health visualisation is the 360.3D matrix evolution |

**Note on the design doc's Section 6.1 "Gap Analysis" (10/37 score).** That table dates from April 2026 and was the snapshot before P0–P3 landed. As of May 2026, the implemented coverage is closer to ~30/37 — the foundation is shipped, the visualization layer is shipping (the 360.3D Inspector is the operational surface for P4–P5), and a small handful of items (the cognitive-query expansion, full health dashboard, knowledge-rhythm signals) remain.

---

## 17. Known divergences between spec and code

Where the design doc and the running code disagree, the **code is authoritative** for what users actually see today; the doc captures the intended philosophy.

| Topic | Design doc says | Code does | Status |
|---|---|---|---|
| Wikilink syntax | `[[type::target\|reason]]` (Part III.2) | `[[target\|type]]` (annotation slot disambiguated against `KNOWN_LINK_TYPES`) | The `::` syntax was a future proposal; the current single-pipe disambiguation ships and works. The `::` syntax would let a typed link carry a separate annotation — currently you have to choose. |
| Decay rate | "5% per month" → `weight × 0.95^months` (Part IV.3) | `weight × exp(-ln(2) × days / halfLifeDays)` (default 60-day half-life) | The half-life model decays faster (≈30%/month at default) than the doc's 5%/month. The implementation chose a steeper curve; the doc framing was an early estimate. |
| Confidence tiers | hypothesis / evidence / established / contested | identical | ✅ Shipped to spec. |
| Lifecycle stages | 6 (Spark / Birth / Growth / Maturity / Dormancy / Renewal-or-Archival) | 5 computed tiers (`fresh / emerging / established / load-bearing / stale`) + archival as a separate `status` column | The 5 tiers map onto 4 of the 6 stages (Birth → Growth → Maturity → Dormancy). Spark is pre-link (autocomplete-time). Renewal is implicit (a stale link revisited rebounds to `emerging`). Archival is `status = 'archived'` rather than a tier. |
| Default link type | `relates` is the default (Part II.4) | The DB stores `'relates'` as a vacuous default; `displayLinkType` ([`store.ts:1761`](src/lib/libraries/store.ts:1761)) drops it from the badge UI, so users never see it. The matrix and panels treat such links as Untyped. | Functional difference is zero — `relates`-only links render as Untyped throughout. |
| `KNOWN_LINK_TYPES` | 7 typed (Part II.4 lists 7) | 8 entries: 7 typed + `'associative'` ([`store.ts:1750-1753`](src/lib/libraries/store.ts:1750)) | `associative` is a legacy synonym for untyped; it round-trips through the parser without surfacing in the user-facing UI. |

---

## 18. Open questions and deferred fixes

Active link-related backlog items as of 2026-05-02:

- **`store.ts:1850` LinkLifecycle dedupe — Option B approved, deferred until post-CE** (project memory: `project_link_lifecycle_dedupe_fix.md`). The `TIER_RANK` literal is missing the `fresh` key. Boss approved the new ranking `stale: 0, fresh: 1, emerging: 2, established: 3, load-bearing: 4`. Held until after CE Phase 12 (360.3D Inspector) closes.

- **`MIG-007` Links Settings tab** (project memory: `project_links_settings_tab.md`). Consolidate every link-related control (`autoUpdateLinks`, `linkLifecycle.decayEnabled`, `linkLifecycle.halfLifeDays`, future link-creation defaults) into a dedicated Settings tab. Currently they're scattered across "Files" and "Appearance".

- **`MIG-010`-scale `note_360_view` write-time cache.** The 360.3D Inspector reads at IPC time (acknowledged Rule-8 violation). Boss's lived perf is "almost instantly" so this is LOW priority — the cache pattern would be a write-time view materialised through triggers on `note_links` changes, mirroring how Sky View already caches its graph snapshot.

- **CE Phase 9 Path B (Multi-Lens)** — `lenses.rs::apply_lens` is dead code today (verified 2026-04-27). Re-wire on a Rule-8-compliant write-time approach (Path B), queued behind MIG-006 §3 redo. Project memory: `project_lenses_apply_lens_dead_code.md`.

- **`MIG-006 §3` redo** — pending; the re-do approach is queued before any of the above bigger migrations.

- **Cognitive-query expansion (P4 design-doc target)**. The doc imagines queries like "strongest evidence for X" or "where are the tensions". Today the building blocks (typed-link operators, weight, confidence) all exist; the surface that turns them into a one-line query language hasn't been built. Path forward: extend `searchOps` to include cognitive operators, and surface them in the Search Hub.

- **`note_links` write-back to LINK files**. The dual-layer storage design says traversal/weight data is written back to LINK files periodically. The current write-back cadence and whether it's user-controllable hasn't been audited recently — flag for a future review.

---

## Sources

- [`docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`](docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md) — design philosophy (April 2026)
- [`docs/Constellation Orientation & Onboarding v1.25.md`](docs/Constellation Orientation & Onboarding v1.25.md) — current onboarding
- [`docs/360.3D-Concept-Paper-v1.0.md`](docs/360.3D-Concept-Paper-v1.0.md) — 360.3D Inspector concept paper
- [`docs/Badge-Taxonomy.md`](docs/Badge-Taxonomy.md) — badge taxonomy reference
- [`src/lib/libraries/store.ts:1660-1885`](src/lib/libraries/store.ts:1660) — `LinkLifecycle`, `linkLifecycle()`, `effectiveLinkWeight()`, `KNOWN_LINK_TYPES`, `displayLinkType()`, `dedupeBySource()`
- [`src/lib/components/SettingsModal.svelte:1420-2129`](src/lib/components/SettingsModal.svelte:1420) — link-related settings UI
- [`src-tauri/src/inspector360.rs:357`](src-tauri/src/inspector360.rs:357) — `precompute_all_strata`
- Project memory: `project_link_lifecycle_dedupe_fix.md`, `project_links_settings_tab.md`, `project_lenses_apply_lens_dead_code.md`, `project_outgoing_typedlink_duplication.md`, `project_backlinks_typed_link_duplication.md`, `project_unlinked_mentions_double_count.md`
- Recent session logs: [`SESSION-LOG-2026-04-27.md`](lab/reports/SESSION-LOG-2026-04-27.md) (§88, §89, §90), [`SESSION-LOG-2026-04-29.md`](lab/reports/SESSION-LOG-2026-04-29.md) (§92–§111), [`SESSION-LOG-2026-05-01.md`](lab/reports/SESSION-LOG-2026-05-01.md) (§115–§121), [`SESSION-LOG-2026-05-02.md`](lab/reports/SESSION-LOG-2026-05-02.md) (§122)

---

*End of Living Links Guide v1.0. Maintained per Standing Order #6 — when any of the link-layer facts change (a phase ships, a setting moves, a lifecycle threshold changes, a divergence is closed), bump this guide to v1.1 in the same commit that lands the change.*
