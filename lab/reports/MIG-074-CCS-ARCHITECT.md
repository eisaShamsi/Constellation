# MIG-074 — CCS, the Constellation Circulatory System (Architect / Phase 1)

**Status: ARCHITECT (Phase 1) — produced 2026-06-10, AWAITING EISA'S RATIFICATION. No Plan, no code.**
**Function in hand: the CCS left-dock Core Plug-in — the circulatory register of the Connection question
(weight · decay · traversal · lifecycle · confidence-flow), per the RATIFIED
`docs/Constellation-Circulatory-System-Concept-Paper-v1.1.md`.**

**Number allocation (verified, not assumed):** orientation v2.64 §8 tops at **MIG-073** (closed 2026-06-10);
a repo-wide search finds **zero** references to MIG-074 or higher (no files, no PJ reservations in Pending
Jobs v1.13). **This migration is MIG-074.**

**FACT** = current code/docs, cited. **PROPOSAL** = design argued here, for ratification.

---

## 0. SO #8 cross-check — the brief vs the canon (done before this doc)

Checked the kickoff brief against the orientation v2.64 **body** (§4.4, §8, §12–§14), the CCS Concept Paper
v1.1, the Living-Link Concept Paper v1.0, the Living-Links Guide, the Knowledge-Formulation spec,
SESSION-LOG-2026-06-09 (§KH-DECAY FIX → §SESSION CLOSE), Pending Jobs v1.13, and the code. The brief holds.
Three reconciliations surfaced (none blocking):

1. **"P5 shipped" vs "P5 never built."** Orientation §4.4 titles the Living Link "P0–P5 all shipped";
   CCS Concept §2 says P5 ("Knowledge health dashboard / visualize circulatory health",
   `CONSTELLATION-KNOWLEDGE-FORMULATION.md` §VII) "was never built." Reading: the **KH overlay cards** were
   P5's first cut (shipped); the ratified CCS paper (2026-06-09, later and more specific) holds that the
   *deep circulatory instrument* P5 envisioned does not exist. CCS completes the P5 vision. The ratified
   paper governs.
2. **Two lifecycle vocabularies are BOTH canon** — a real design fork this doc surfaces as **D2** (§4):
   the shipped cache key `lifecycle` holds the 6-stage *life arc* (spark · birth · growth · maturity ·
   dormancy · archival — KF-spec §IV.2, `compute_lifecycle_distribution` search.rs:5531), while the ratified
   CCS §6 register "The Life of a Connection" names the 5-tier *usage census* (fresh · emerging · established
   · load-bearing · stale — Living-Links-Guide §8, `linkLifecycle()` store.ts:2320).
3. **Pending Jobs v1.13 staleness (noted, not CCS's to fix):** PJ-005 still reads open but MIG-007 closed it
   2026-06-09 (v1.13 predates the close by hours); PJ-063 (`link_type` globally 'relates') looks possibly
   stale — the live `by_type` is healthy (supports 104,719 · derives-from 97,958 · relates only 301, per the
   2026-06-10 handover §3). Both belong to the next Pending Jobs version.

**CNS identity pinned** (it was fuzzy in the docs): CNS **is** `ConstellationSight2.svelte` — the old
Sight-v2 gravity-well surface, user-retitled "Constellation Nervous System (CNS)" in all 15 locales
(`en.json:2894` `lens.title`), live under `SIGHT_V2_ENABLED = true` (`src/lib/sight/engine.ts:131`),
opened from the dock (`+layout.svelte:4880`, `lensActive`) into a full-page overlay (`.lens-overlay`,
`+layout.svelte:5729`). It computes communities / centrality / bridges via `clusterEngine`
(`+layout.svelte:3854–3920`). There is **no CNS concept paper** at docs/ root — only the help topic. CCS is
its peer, not its dependent: nothing in MIG-074 touches CNS code.

---

## 1. What MIG-074 builds (from the ratified concept)

A **left-dock Core Plug-in** (peer of CNS and the Cataloger) rendering the **seven cognition-named
registers** of CCS §6 — Living Connections · Load-Bearing Reasoning · Cooling Inquiries · Conviction & Doubt
· The Life of a Connection · Retired Reasoning · The Acts of Inquiry — reading the MIG-073
`link_stats_cache` layer + the Link-Type Registry, writing only via the existing lifecycle commands. It
**fully retires** the right-sidebar Link Dashboard panel (§15 ruling 6), **re-points** the MIG-007 Settings
hub (ruling 7), and **coordinates with** (never absorbs) Knowledge Health (ruling 5).

---

## 2. Territory map (FACT)

### 2.1 The data layer (MIG-073, shipped + audited)

- **`note_links`** (search.rs:2306–2322): `source_path · source_name · target_path · target_name ·
  link_type (DEFAULT 'relates') · annotation · confidence (DEFAULT 'hypothesis') · weight (DEFAULT 1.0) ·
  created · last_traversed · traversal_count · library_name · status (DEFAULT 'active')`.
  **Nine indexes** (search.rs:2323–2335) including `idx_link_status`, `idx_link_last_traversed`,
  `idx_link_traversal_count`, `idx_link_weight`, `idx_link_confidence` — every CCS sort key is indexed.
- **`link_stats_cache`** (search.rs:2440): `stat_key PK · payload JSON · computed_at`. Six keys today:
  `stats`, `lifecycle`, `fmt_emerging`, `fmt_bias_check`, `fmt_most_connected`, `fmt_weak_foundations`
  (search.rs:5613–5616; the schema comment says "CCS adds more" — extension was designed in).
- **Read path**: `constellation_knowledge_health_snapshot` (search.rs:5725) — stale-while-revalidate,
  2-min window (`KH_CACHE_FRESH_MINUTES`, search.rs:5623), `{ready:false}` + self-healing populate on
  empty/missing table, `kh-snapshot-ready` event, measured **0.17 ms**.
- **Recompute**: `recompute_link_stats_cache(conn)` (search.rs:5633) — one background pass on a **dedicated
  connection**, never the open path. Hooks: boot first-time population `only_if_empty` (cache.rs:1137–1145,
  walk-free boot intact) + **unconditional** post-reconcile recompute (cache.rs:1123) + the SWR kick.
- **Decay is display-only** (`effectiveLinkWeight`, Living-Links-Guide §7); `constellation_link_decay` is
  strictly read-only since `aa9941ee`. **Nothing writes `status='dormant'`** — the only status writers are
  INSERT defaults ('active'), traverse (dormant→active, search.rs:5211), archive ('archived', :5872),
  unarchive ('active', :5894).

### 2.2 The seven registers → data (the build's core mapping)

| CCS §6 register | Signal | Cache key today | Gap |
|---|---|---|---|
| Living Connections | most-traversed, warm (recent) | — (`fmt_emerging` is hypothesis-only) | **new key** `ccs_living`: top-N active, traversed within 90d, ORDER traversal_count/recency |
| Load-Bearing Reasoning | high earned weight + recently walked | `stats.sample_links` is top-10 by weight, unfiltered | **new key** `ccs_load_bearing`: top-N weight DESC, traversed within 90d |
| Cooling Inquiries | tc≥1, untraversed 90+d, decaying | — | **new key** `ccs_cooling`: top-N by idle-age DESC *(this IS the read-time dormancy derivation — §3-b)* |
| Conviction & Doubt | confidence spread + contested | `stats.by_confidence` ✓ | optional **new key** `ccs_contested`: top-N confidence='contested' |
| The Life of a Connection | lifecycle census | `lifecycle` (6-stage arc) | per **D2**: new `ccs_tiers` (5-tier) and/or reuse |
| Retired Reasoning | archived, restorable | — (`constellation_link_archived` is a live indexed query) | **new key** `ccs_retired` (count + recent top-N); the full browse + unarchive stays the live IPC (action surface must be current) |
| The Acts of Inquiry | distribution of cognitive acts | `stats.by_type` ✓ | none — frontend orders/labels via the Link-Type Registry; untyped/legacy ids (`relates`/`associative`) surface as **"open inquiries"** (guardrail 1), never as a defect |

Net backend delta: **~4–5 additive cache keys computed inside the existing single recompute pass.** No new
table, no schema change to any source of truth, no new scan (each key is one indexed aggregate in the same
background transaction). This is the bounded backend the concept's "frontend-mostly" (§13) anticipated.

### 2.3 The census duality (the D2 fork — both vocabularies are canon)

- **6-stage life arc** (KF-spec §IV.2; `compute_lifecycle_distribution` search.rs:5531; KHD renders it via
  `notePane.stage.*` ×15): spark (tc=0, <7d) · birth (tc=0, ≥7d) · growth (tc>0, weight<5) · maturity
  (weight≥5) · **dormancy (status='dormant' — write-orphaned, historical rows only)** · archival.
- **5-tier usage census** (Guide §8; `linkLifecycle()` store.ts:2308–2340, pure read-time): fresh (tc=0) ·
  emerging (1–2, ≤90d) · established (3–9, ≤90d) · load-bearing (≥10, ≤90d) · stale (tc≥1, >90d).
  `LINK_STALE_DAYS = 90` (store.ts:2310). The ratified CCS §6 text names **this** census for the register.

### 2.4 The predecessor — `LinkDashboard.svelte` (the panel ruling 6 retires)

~378 lines, right-sidebar tab **`links`** (universe-scoped, not note-gated — outside `NOTE_SCOPED_TABS`).
Mounted `+layout.svelte:6472`. Reached two ways: the tab strip, and the MIG-007 hub event
(`SettingsModal.svelte:1391` dispatches `constellation:open-link-dashboard` → listener `+layout.svelte:2300`
sets `rightSidebarTab='links'`). **Data model: in-memory** — props `allLinks`/`allNotes` from the boot
graph snapshot, sections computed in JS per open (234k links sorted on the main thread); its only invokes
are `listArchivedLinks` / `unarchiveLink`. i18n namespace `linkDashboard.*` ×15.

**Section-by-section retirement map (Predecessor → Replacement — the Predecessor Lookup Rule record;
the same table lands in the session log before any Build edit):**

| LinkDashboard section | Register family | Destination |
|---|---|---|
| `mostTraveled` (top-20 by decayed weight) | circulatory | **CCS Living Connections / Load-Bearing** |
| `stale` (>90d, oldest first) | circulatory | **CCS Cooling Inquiries** |
| `archived` (+ unarchive action) | circulatory | **CCS Retired Reasoning** (reuses the same two IPCs) |
| `mostConnected` (top-10 bidirectional count) | topology (CNS's per CCS §12) | **drops here; lives on in KH** (`fmt_most_connected`, MIG-073 kept it; re-homing to CNS is CNS-MIG scope) |
| `orphan` (unlinked notes) | topology | **drops here; lives on in TensionPanel** (`detect_tensions` orphans — already shown there) |
| `crossLibrary` (inter-library links) | neither (structural curiosity) | **Eisa ruling needed — Q6** (drop vs a small CCS footnote section) |
| `broken` (unresolved targets) | integrity (≠ untyped, per guardrail 1) | **Eisa ruling needed — Q6** (drop — in-editor red links already mark them — vs keep a small CCS list) |

### 2.5 The surface pattern (precedents to copy, not reinvent)

Dock buttons live at `+layout.svelte:4800–4986`; full-page Core Plug-ins mount as overlay divs gated by a
sticky lazy-mount flag (LL-022) — the Cataloger is the cleanest precedent (`catalogerEverOpened`
+layout:623/626; `{#if catalogerEverOpened}<div class="cataloger-overlay" class:cataloger-visible=…>`
+layout:5686–5698). KH already owns a dock button (`showKnowledgeHealth`, `ribbon.knowledgeHealth`,
+layout:4825–4841) and the command-palette entry (+layout:1826). CNS mounts the same overlay way
(.lens-overlay, +layout:5729). Style-Setter targeting = a `data-style-target` on the surface root (pattern:
`cDock` +layout:4800, `cTags` TagsPanel:72). Live dock order today (default flags): Search Hub · OrgChart ·
**Knowledge Health** · Sky View · Daily Note · AI Skills · Index · **Cataloger** · **CNS** · Inspector360 —
then bottom: Second Screen · Style Setter · Settings.

### 2.6 The MIG-007 hub seam (ruling 7)

One dispatch site (`SettingsModal.svelte:1391`, label `settings.links.dashboardBtn`) + one listener
(`+layout.svelte:2300`). Re-point = flip the listener to open CCS + relabel the button (×15 locales).
Atomic with the retire commit so the hub never dangles (invariant I11).

### 2.7 `detect_tensions` (the §3-a ruling's subject)

`src-tauri/src/tension.rs:54–217` — a **filesystem walk** (re-reads every .md per run; regex-extracts
wikilinks + tags) computing four things: contradiction pairs (deduped ×N), orphans (0 inbound, >20 words),
structural gaps (tag clusters <20% link density), single points (5+ inbound, ≤1 derives-from). Consumers:
the right-sidebar **health** tab (TensionPanel, lazy-loaded per-library since `b2a23d4e`) + the Sky View
legend action (+layout:5789). Its outputs are **CNS-register material** (orphans/gaps/single-points =
topology; the contradiction *list* is the tension surface) — none are circulatory. Its inputs exist in the
DB in principle (note_links is MIG-067-correct; word_count in note_meta), **except the tag-cluster input,
whose DB-side queryability is unverified** — a fact to establish in whichever MIG takes it.

### 2.8 The write surface CCS may touch (FACT — all existing, none new)

`constellation_link_traverse` (weight=1+ln(1+tc), auto-promote confidence, dormant→active) ·
`_link_set_confidence` · `_link_backfill_confidence` · `_link_archive` (status='archived', weight=0) ·
`_link_unarchive` (status='active', weight=1) · `_link_archived` (read) — search.rs:5105–5932. CCS v1 needs
exactly two as actions: **unarchive** (Retired Reasoning) and optionally **set_confidence** (Conviction &
Doubt). **CCS must never call `_link_traverse`** — see invariant **I2b** (observer effect).

### 2.9 Federation (FACT)

The cache + snapshot IPC are **active-universe only** (the universe's own search.db), matching KH today.
The federated-read precedent is `get_federated_schemas` + per-schema loops (cache.rs:456, 587–613; MIG-061
§L per-schema isolation). KH federation itself is the reserved **MIG-063** family, not built. A federated
CCS would also hit a real edge: a cUniverse's search.db only *has* a `link_stats_cache` if that universe was
opened under a MIG-073+ binary (else the per-schema read needs Scenario-2-style self-healing).

---

## 3. Explicit in/out rulings (the brief's a/b/c + the retirement set)

| # | Item | Ruling (PROPOSAL) | Why |
|---|---|---|---|
| a | **`detect_tensions` fs-walk Rule-8 modernization** | **OUT of MIG-074** — queued as the opening § of the future CNS-modernization MIG (number allocated when opened) | Its four outputs are all topology/tension (CNS register, §2.7); CCS invariant I4 (complement-not-overlap) forbids CCS owning them. Folding it in would push MIG-074 across the Rust tension-subsystem boundary for zero circulatory gain. The boundary case — contradiction *counts* — is already circulatory-served by `stats.by_type` (the Acts of Inquiry shows `contradicts` distribution); the contradiction *pair list* stays TensionPanel's. *(If Eisa prefers IN: it lands as a self-contained §-step re-sourcing tension.rs from note_links/note_meta — with the tags-in-DB fact verified first; adds ~1 session + a Rust-side test surface.)* |
| b | **Read-time dormancy derivation** | **IN** — as derived buckets inside the existing recompute (no status writes; I9 intact) | The `ccs_cooling` key derives "cooling/stale" from `last_traversed` at recompute time — dormancy as a *read-time judgment*, exactly what the decay fix left missing. Whether the shared 6-stage `lifecycle` key's dormancy bucket is ALSO repaired (derived `status='dormant' OR (active ∧ tc>0 ∧ idle>90d)`) is **Q3** — it changes KH's displayed census (today: dormancy≈0, growth/maturity silently include stale links), so it's Eisa's call, not silently shipped. |
| c1 | **Left-dock placement + icon** (§15 open) | **IN** — decided at ratification (**Q1**) | Options in §4-D4. |
| c2 | **Per-register trends** (§15 open) | **OUT of v1 (defer)** — unless Eisa picks the thin-history option (**Q4**) | The cache is a single snapshot (PK stat_key); trends need an append-only history. Deferring honors Constraint-as-Design; the thin option stays additive + droppable if wanted. |
| c3 | **Knowledge-Health coordination surface** (§15 open) | **IN** — mutual deep-links, not embedding (**Q5**) | Options in §4-D5. |
| d | **`crossLibrary` + `broken` LinkDashboard sections** | **Eisa ruling needed (Q6)** — both are user-facing today; neither is circulatory | Predecessor Lookup forbids silent feature loss. Recommendation: drop both with the panel (broken = in-editor red links already; crossLibrary = structural curiosity with no register home), noted in the help/manual change log. |
| e | **`most_connected` + the fmt_* insight lists in KH** | **OUT — untouched** | MIG-073 locked "don't change KH's content"; CCS §15 ruling 5 = coordinate. KH keeps its cards; CCS links to it. Re-homing most_connected to CNS is CNS-MIG scope. |
| f | **Federation of CCS** | **OUT of v1** — active-universe with an honest scope label; joins the MIG-063 family later | §2.9: per-schema cache presence isn't guaranteed; KH (the layer's first consumer) is active-universe today. v1 parity + a visible "this universe" label beats silent partial federation (I14). |
| g | Also OUT | PJ-063 re-verification · the 1.7 GB search.db investigation · per-locale labels for custom registry types · the deferred on-disk LINK-files layer · any CNS code change | Each named in the handover/orientation as its own item. |

---

## 4. Design options

### D1 — How the registers get their data (the load-bearing choice)

| Option | Mechanics | Speed (open) | Effort | Risk |
|---|---|---|---|---|
| **A — live indexed queries** | each register queries note_links on open (9 indexes exist) | first cold touch of the 1.7 GB DB pays seconds — the KH freeze shape returns | low | **high** — re-violates CCS §8 ("write-time-maintained — no graph walk on open") + Rule 8; rejected |
| **B — extend `link_stats_cache`** ★ | ~4–5 additive `ccs_*` keys computed in the existing background recompute; panel = ONE snapshot read (the KH pattern, 0.17 ms); drill-downs ("show more", archived browse) = bounded LIMIT-ed indexed queries on demand, actions live | instant (cache) + bounded on drill-down | **medium** — one Rust commit (keys + helpers in the same pass) + the frontend surface | **low** — proven layer (MIG-073 audit 8/8); snapshot can't drift, worst case bounded staleness (2-min SWR); additive keys are invisible to old readers |
| **C — frontend-only from in-memory `allLinks`** | what LinkDashboard does today: JS sorts of the 234k-link boot array per open | main-thread sorts at federation scale; no SWR | low | **medium** — violates the ratified §8 contract; perpetuates the predecessor's scaling model; rejected |

**Recommendation: B.** It is the design the concept paper assumes and the cache's schema comment
pre-announced.

### D2 — Which census "The Life of a Connection" shows (Q2)

| Option | What the register renders | Cost | Note |
|---|---|---|---|
| **A — 5-tier usage census** ★ | fresh · emerging · established · load-bearing · stale (new `ccs_tiers` key; SQL port of `linkLifecycle()`) | one more aggregate in the recompute | matches the **ratified §6 text** + Guide §8; "stale" carries the dormancy signal natively |
| B — 6-stage life arc | reuse the `lifecycle` key (KH parity) | zero | diverges from the ratified register text; its dormancy bucket is write-orphaned until Q3 repairs it |
| C — both layers | arc + usage census stacked | both of the above | richest; more UI; risks the "two vocabularies" confusion CCS exists to dispel |

**Recommendation: A** for the register; **Q3** decides separately whether the shared 6-stage key's dormancy
bucket gets the derived repair (which also heals KH's census).

### D3 — `detect_tensions` in/out → ruled in §3-a (OUT recommended; IN variant scoped there).

### D4 — Dock placement + icon (Q1, §15 open)

Placement options: **(i) immediately adjacent to CNS** ★ — the two Connection-question instruments read as
the pair the concept says they are; **(ii) adjacent to Knowledge Health** — groups the circulatory family
(KH + CCS); **(iii) end of the top group.** Icon options (inline SVG like the rest of the dock):
**(α) an ECG/pulse waveform** ★ — "the pulse of your thinking", visually distinct from the health tab's
heart-pulse glyph; **(β) a heart with a pulse line**; **(γ) a circulation loop** (two curved arrows in a
ring). Gating mirrors the Cataloger: `enabledFeatures.ccs !== false`, a Settings → Plug-Ins entry, default ON.

### D5 — The KH coordination surface (Q5, §15 open)

**(i) Mutual deep-links** ★ — a "Knowledge Health →" affordance in the CCS header opening the existing
overlay, and an "Open CCS →" in KH's header; zero duplication, honors "coordinate, not subsume".
**(ii) An embedded KH summary card inside CCS** — reads the same cache so no extra compute, but visually
absorbs KH (subsume-creep) and duplicates rendering ×15 locales. **(iii) Nothing** — under-delivers ruling 5.

### D6 — Per-register trends (Q4, §15 open)

**(i) Defer to v2** ★ — ship the seven registers; revisit once lived-in. **(ii) Thin history** — an additive
`link_stats_history (stat_key, computed_at, payload_lite)` appended per recompute, pruned to ~90 days;
derived + droppable (same reversibility class as the cache); adds a small sparkline per register. If (ii) is
chosen, the Plan runs the WA#5 cross-check on the time-series-in-SQLite pattern before building.

---

## 5. Invariants (must not break)

- **I1** No full `note_links` scan on CCS open — registers read the snapshot; drill-downs are bounded
  (LIMIT-ed, indexed) on-demand queries.
- **I2** No new write path — actions are the existing lifecycle commands only (unarchive; optionally
  set_confidence).
- **I2b — no observer effect**: opening/navigating from a CCS row must **never** fire
  `constellation_link_traverse` — CCS observes circulation; it must not feed the metric it displays.
- **I3** Decay stays display-only (MIG-073 I9): the recompute stores raw aggregates; nothing re-decays or
  writes `weight`/`status` as a side effect of viewing.
- **I4** Complement-not-overlap (CCS §11): no topology in CCS — most-connected/orphans/gaps/single-points
  stay with KH/TensionPanel/CNS.
- **I5** Registry-driven acts: order, colors, labels, custom types via the Link-Type Registry +
  `LinkTypePill` (self-contained, locale → registry-label → raw-id fallback) — never a second color/name map.
- **I6** Facts rest: Cooling Inquiries + Conviction & Doubt are invitations; no per-note nagging of
  fact-notes. (Whether v1 also stratum-scopes those registers' rows is a Plan-level option — the framing
  guardrail is non-negotiable either way.)
- **I7** Untyped = the open question: `relates`/`associative`/untyped surface as **open inquiries**, never
  as a deficiency or "broken".
- **I8** Reversibility: archive/unarchive round-trip intact; every cache key droppable + rebuildable; the
  retire commit is a clean revert target.
- **I9** The KH overlay keeps working unchanged through every commit — the 6 existing keys' shapes are
  frozen; new keys are additive.
- **I10** 15-locale i18n from day one (`ccs.*` namespace), RTL-correct (dir-aware rows, `match-parent`
  lesson), Style-Setter-targetable root (`data-style-target`), theme vars only.
- **I11** The MIG-007 hub never dangles: the Settings button's destination flips to CCS **in the same
  commit** that retires the LinkDashboard tab.
- **I12** No silent feature loss: every LinkDashboard section lands per the §2.4 map; `crossLibrary`/`broken`
  go only where Q6's ruling says.
- **I13** Perf Rule 8 + boot gate: the new keys ride the existing single recompute pass (no second scan);
  zero IPC while CCS is closed (LL-022 lazy-mount); boot/typing/IPC measured before/after on the 7,661-note /
  234k-link universe.
- **I14** Federation honesty: v1 states its active-universe scope visibly; no pretend-federation.

---

## 6. Migration path / back-fill / rollback

- **First boot after update:** new `ccs_*` keys are absent until the first recompute (boot `only_if_empty`
  fires only on an empty cache, and existing users have 6 rows). The snapshot/CCS read therefore treats
  *missing ccs_ keys* as `{ready:false}`-per-section and the SWR kick populates them — the MIG-073
  Scenario-2 self-healing pattern, extended per-key. No wipe, no migration step.
- **Old binary on a new DB (rollback):** extra cache rows are inert (key/value reads by name); KH ignores
  them. Rollback = revert the frontend + (optionally) DELETE the ccs_ rows — derived data, droppable.
- **Mid-recompute interrupt:** unchanged from MIG-073 — per-key `INSERT OR REPLACE`; a partial key-set
  self-heals on the next kick.
- **The retire commit** (LinkDashboard + tab + listener re-point + hub relabel) is the LAST build step,
  single-commit, revertible; `linkDashboard.*` keys drop ×15 in the same commit (machine-verified deltas,
  per the established merge-script pattern).
- **No schema change** to any source of truth at any step.

---

## 7. What MIG-074 is NOT

- **Not the CNS migration** — no CNS code, no detect_tensions rewrite (per §3-a), no re-homing of
  most_connected.
- **Not a KH redesign** — KH's content and IPCs are frozen here (coordinate only).
- **Not a `note_links` or registry schema change** — additive cache keys only.
- **Not the federation migration** (MIG-063 family) and **not the 1.7 GB DB investigation**.

---

## 8. Open questions for Eisa (the ratification gate)

| Q | Decision | My recommendation |
|---|---|---|
| **Q1** | Dock placement + icon | adjacent to **CNS**; **ECG pulse-waveform** icon (D4 i/α) |
| **Q2** | "The Life of a Connection" census | the **5-tier usage census** (D2-A, the ratified §6 wording) |
| **Q3** | Repair the shared 6-stage `lifecycle` key's dormancy bucket to the derived definition (changes KH's census numbers honestly: stale links leave growth/maturity, dormancy stops reading ~0)? | **Yes** — heal it at the data layer for both consumers |
| **Q4** | Per-register trends | **defer to v2** (D6-i) |
| **Q5** | KH coordination | **mutual deep-links** (D5-i) |
| **Q6** | `crossLibrary` + `broken` sections' fate at retirement | **drop both with the panel** (documented in help/manual); say the word and either becomes a small CCS section instead |
| **Q7** | `detect_tensions` ruling | **OUT** (§3-a) — queued as the opening § of the future CNS MIG |
| **Q8** | Data path | **Option B** (D1) — extend the cache |

---

## 9. Recommendation (one paragraph)

Ratify **Option B + the §3 rulings as recommended**: MIG-074 adds ~4–5 additive `ccs_*` keys to the proven
MIG-073 recompute (one Rust commit), stands up the CCS left-dock Core Plug-in on the Cataloger's
overlay/lazy-mount pattern reading ONE snapshot call, builds the seven ratified registers with
registry-driven pills and the facts-rest framing, wires unarchive (+ optional set-confidence) through the
existing commands, then — as the final, atomic, revertible step — retires the LinkDashboard tab and
re-points the MIG-007 hub. Dormancy becomes a read-time judgment inside the shared recompute (Q3), CNS and
detect_tensions stay untouched for their own future MIG, and Knowledge Health keeps its cards with a mutual
deep-link. **STOP: awaiting Eisa's answers to Q1–Q8 before any Phase-2 Plan.**
