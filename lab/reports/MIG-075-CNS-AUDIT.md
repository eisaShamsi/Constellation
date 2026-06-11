# MIG-075 — The CNS Full Audit (concept · functions · purpose · boundaries · slowness · defects)

**Status: AUDIT (the re-scoped Phase 1) — produced 2026-06-11, AWAITING EISA'S RATIFICATION. No Plan, no code.**
**Function in hand: the CNS (Constellation Nervous System) — `ConstellationSight2.svelte` + its `SightPanel`
+ the tension subsystem — audited in full against CCS, Knowledge Health, and every other core plug-in,
per Eisa's 2026-06-11 direction: "fully audit its concept, functions, and purpose… Also fix its slowness
and design bugs/issues."**

**Correction record (Stop-On-Correction discipline):** Eisa said **No** to the v1 Architect's Q1–Q8 gate as
framed — the scope was too narrow. Since his last approval, nothing was built: the only changes are the
committed documents (`MIG-075-CNS-ARCHITECT.md`, session log, orientation v2.67); zero code touched. The
v1 Architect's **§2 territory facts all stand** and this audit builds on them (every §2 claim was read
first-hand); its §3/§8 *ruling framing* is superseded by this document. Five parallel read-only agents
covered the surfaces not previously deep-read (Sky View, KH, CCS, the remaining plug-ins, the concept
docs); their findings are folded in below with citations.

**FACT** = code/docs/DB, cited. **PROPOSAL** = design argued here, for ratification.

---

## 1. What the canon says CNS is — and the four identity gaps

**The documented identity (FACT):**

- The help topic (`docs/help.uConstellation.World/Constellation Nervous System/…md:13–19`): CNS is "the
  **connection-traversal** view of your universe… CNS shows the *wiring* — the typed-link graph that
  connects them and the structural patterns hidden in that graph. It answers: **'How are the ideas in my
  universe connected, and where are the gaps?'**" It documents: the Universe Health card (modularity ·
  dominance · entropy · connectivity), the gravity well, Top Bridges, **Communities**, **Blind Spots
  (Structural Gaps)**, and the locked interaction (single-click preview / double-click open). An Arabic
  mirror exists.
- The founding spec (`CONSTELLATION-KNOWLEDGE-FORMULATION.md` §1.1): the explicit **dual** biological
  design — Nervous + Circulatory ("vessels strengthen under heavy flow, weaken without use").
- The Cognitive-Engine paper (Q4, `Cognitive-Engine-One-Picture…v1.0.md:73–76`): Connection is answered by
  the 8 typed links + "**CNS network measures (community, centrality, bridges, blind-spots,
  load-bearing)**".
- The ratified CCS boundary (`Constellation-Circulatory-System-Concept-Paper-v1.1.md` §4): CNS reads
  **structure** (who connects to whom), ignores **flow** (age, weight, traversal); owns communities ·
  centrality · bridges · structural blind-spots.

**The identity gaps (FACT, audit findings):**

| # | Gap |
|---|---|
| G1 | **CNS has no concept paper.** CCS, Sight, Map, the Base, the Living Link, 360.3D all have ratified papers; CNS's identity lives only in a help topic + scattered passages (confirmed by file sweep — no `docs/*Nervous*`/`*CNS*` paper exists). It is the only core plug-in whose "why" was never written down and ratified. |
| G2 | **The help topic frames CNS against a disabled surface.** Its "CNS vs Sight — When to Use Which" section compares against Constellation Sight — Wings-disabled since MIG-038. The live comparison the user actually needs (CNS vs **Sky View**, CNS vs **CCS**) is undocumented. |
| G3 | **"Health" means two unrelated things one click apart.** CNS's header card says "Universe Health" (a topology composite, §2.6 of the v1 Architect: modularity/dominance/entropy/connectivity/gaps); the dock has a "Knowledge Health" plug-in (link-stats count cards). Different numbers, different questions, same word. |
| G4 | **"Load-bearing" means two things.** The CE paper assigns "load-bearing" to CNS (structural: bridges); the ratified CCS §6 names "Load-Bearing Reasoning" (circulatory: earned weight). The vocabulary split is real and currently unexplained anywhere. |

---

## 2. What CNS actually does today — the promise-vs-delivery table

The `toggleLens` pipeline (+layout:3823–3950) computes **nine** results. The surface renders **five**.

| # | Pipeline step | Computed | Rendered? | Where |
|---|---|---|---|---|
| 1 | Rust centrality (`constellation_sight_centrality`) | ✓ | ✓ | radial layout (distance from center) |
| 2 | **Louvain communities** (JS `detectClusters`) | ✓ | **✗ — props `communityAssignments`/`communityColors` declared (Sight2:90–91) and never referenced again** | nowhere |
| 3 | **Structural gaps** (`computeStructuralGaps`, Burt's structural holes) | ✓ | **✗ — only the gap COUNT feeds the health score; the gap list renders nowhere** (prop `gaps` Sight2:92, unused) | nowhere |
| 4 | Universe Health composite | ✓ | ✓ | SightPanel Overview |
| 5 | Stratum-weighted centrality | ✓ | ✓ | replaces step 1's values |
| 6 | Top-10 bridges | ✓ | ✓ | SightPanel Top Bridges |
| 7 | **Community profiles** (maturity + provenance per community) | ✓ | **✗ — prop unused** (Sight2:96) | nowhere |
| 8 | **Bridge suggestions for gaps** (`suggestBridges`) | ✓ | **✗ — mutates the unused `gaps`** | nowhere |
| 9 | **Contradiction pairs** (from the centrality IPC) | ✓ | **✗ — prop `contradictions` unused** (Sight2:97) | nowhere |

**This is the audit's central finding: the help topic documents "Communities" and "Blind Spots (Structural
Gaps)" as CNS sections — the exact registers the Cognitive-Engine paper assigns to CNS — and the code
computes them on every open and renders none of them.** What the user actually sees is: the radial canvas
(centrality × library sectors), search, a legend, and a side panel of Overview / Link Health (circulatory,
the shed) / Top Bridges / Knowledge Insights. CNS's unique analytical promise is ~half dark.

Also rendered today (FACT): the well shows **all** notes including orphans — there is no linked-subgraph
filter in the component (verified; SightPanel's orphan stat counts real zero-link nodes). Node color
encodes **library**, not community; the legend's three "region" dots are static hexes (Sight2:1220–1222).

---

## 3. The boundary audit — CNS vs every core plug-in

| Surface | Its question | Link/graph data it shows | Verdict vs CNS |
|---|---|---|---|
| **CCS** | "How is my thinking *circulating*?" (flow) | 7 registers off the `link_stats_cache`: living/load-bearing/cooling/conviction/tiers/retired/acts | **Boundary ratified (CCS §4) — and CNS still violates it**: SightPanel's BY TYPE + BY CONFIDENCE + dormant chip + 4 circulatory insight tabs are flow data inside the structure instrument. The shed (Boss-approved 2026-06-10) resolves it. |
| **Knowledge Health** | "At-a-glance counts" | lifecycle cards · totals · **by_type** · **by_confidence** · **most-connected** · emerging · **weak-foundations** · bias (KHD.svelte:157–284) | **Duplication web**: by_type + by_confidence render in **three** places today (KH, CCS, CNS-panel); most_connected and weak_foundations in **two** (KH, CNS-insights). Plus the G3 naming collision. After the shed, the remaining overlap is most_connected/weak_foundations/knowledge_gaps in CNS-insights — ruled in D4/D5. |
| **Sky View** | "Show me the sky" (the spatial picture) | force-directed, ALL notes, full visual vocabulary (rings/glows/maturity/MOC), **registry-fed typed colors via skyPalette (MIG-072)**, full Style-Setter + dark/light theming, worker-simulated, dense-mode damping | **Complement — picture vs instrument — but the differentiation is currently weak**: CNS's rendered subset (a node-link canvas + search + legend) is a poorer, unthemed, registry-blind Sky View. CNS's actual differentiators (analytic layout, communities, gaps, bridges, health) are the unrendered/slow half. The audit's target (§7) re-asserts: in CNS **the layout IS the answer**; everything decorative stays Sky View's. |
| **TensionPanel** (health tab) | per-library tension lists | contradiction **pair list** (×N) · orphans · **tag-cluster gaps** · single-points | **Two different "structural gap" definitions coexist**: tag-cluster density (tension.rs) vs community-pair holes (clusterEngine, Burt). Both are honest, different lenses (vocabulary-based vs link-topology-based). Division to ratify (D2): pair **lists** = TensionPanel; graph-level gap **geometry + bridge suggestions** = CNS; contradiction **counts** = CCS Acts; per-note flags = 360.3D. CNS should NOT add a fourth contradiction list (step 9 stays cut). |
| **Inspector 360 / 360.3D** | ONE note's connection signature | stratification matrix, per-note flags (orphan/SPOF/contradictions) | Orthogonal by scale (note vs universe) — the ratified Sight/360 split transfers cleanly. No change. |
| **Search Hub** | finding | link operators (linksTo/orphans/…), no analytics | Orthogonal. Shared vocabulary is healthy: CAT_COLORS badge maps are **identical** in Sight2 vs ConstellationMap (verified — the Badge-Taxonomy invariant HOLDS); CNS reuses the searchHub chips deliberately. |
| **OrgChart** | hierarchy | per-node link_count/maturity/stratum, no centrality/communities | Orthogonal (density of the tree, not topology of the graph). |
| **Index** | the lexicon | terms/mentions, no links | Orthogonal. |
| **Cataloger** | Origin (source × content-type) | none | Orthogonal. |
| **Tasks / Daily Note / AI Skills** | — | none | Orthogonal. |
| **Map / Sight** | — | Wings-disabled (MIG-038) | Out of the comparison; the help topic's Sight framing is G2. |

**Cross-check vs proven practice (WA#5):** the "picture vs instrument" split is the standard shape —
Obsidian's graph is picture-only (no analytics; Constellation's differentiator), Gephi separates the canvas
from the statistics panel (modularity coloring + metrics), and gap/structural-hole analytics (Burt 1992,
already cited in clusterEngine.ts) are the established graph-analysis registers. Rendering communities as
node/region color and gaps as a ranked list with suggested bridges is the mature pattern — no invention
required.

---

## 4. The slowness anatomy (why CNS freezes) — and the fix set

All four costs verified in code; magnitudes from the live universe (7,661 notes · 234,062 links · 1.7 GB
search.db · 25-cUniverse federation):

| # | Cost | Mechanics | Fix (directed) |
|---|---|---|---|
| S1 | **The centrality walk** — the dominant cost | `constellation_sight_centrality` is a **sync** command (UI thread blocked) that re-reads **every .md** via `scan_library_links` per library on first open; cUniverse paths error out and are silently swallowed (`.unwrap_or_default()`, sight.rs:67) — the walk pays full price for the own universe and returns nothing for the federation it displays | **Re-source from `note_links`** (source_name/target_name/link_type are the exact inputs; indexed; ms-scale per the MIG-073/074 precedent) + `(async)`. Same Brandes, same graph, same contradictions collection. Exact scope parity (own-universe). |
| S2 | **Three live IPCs per panel open** | `constellation_link_stats` (two full-table GROUP BYs — the same cold-scan family MIG-073 measured at 12–20 s on the KH overlay before caching) + `constellation_link_dormant` (julianday scan) + `constellation_formulation_analysis` (LIMIT 50) — all on the shared DB mutex | The shed removes the first two entirely; the surviving insight tabs (per D5) re-point to the **cached snapshot** (`fmt_most_connected` / the KH snapshot IPC, 0.17 ms measured) — zero live aggregate queries left on the open path. |
| S3 | **Main-thread Louvain + gap analysis** | `detectClusters` + `computeStructuralGaps` + profiles + suggestions run in JS on the UI thread over 233,538 edges on every recompute | After S1/S2 land: **measure first** (the MIG-016 §1A `sight:*` performance.marks are still in place — LL-015 discipline). If Louvain still janks at federation scale, the Plan's option is a worker offload (the Sky View force-sim precedent) — decided on numbers, not guesses. |
| S4 | **Invalidation makes it recur** | `skyVersion` bumps on library load/sync, rebuilds, index updates, tag/property changes — i.e., normal editing invalidates `lensDataStale`, so the S1 walk recurs in everyday use | With S1 the recompute is cheap reads; invalidation behavior can stay as-is. |

---

## 5. The design-defect register

| ID | Defect | Severity | Resolution |
|---|---|---|---|
| DF-01 | The sync fs-walk centrality (S1) | **Critical (perf)** | S1 re-source + async |
| DF-02 | Three live IPCs per panel open (S2) | **High (perf)** | shed + snapshot re-point |
| DF-03 | **4 of 9 pipeline outputs computed-never-rendered** (communities, profiles, gap list, bridge suggestions, contradictions) | **High (waste + broken promise)** | ruling **D2** — render or cut; never compute-and-drop |
| DF-04 | Help topic documents Communities + Blind Spots sections that don't exist; frames CNS vs the disabled Sight (G2) | High (doc-drift) | rewrite with D2's outcome (EN now; 14-language batch) |
| DF-05 | Raw `lens.link*` keys render verbatim (associative/supersedes/relates/custom) in BY TYPE bars | High (visible) | dies with the shed |
| DF-06 | Registry-blind typed-link colors ×3 sites (canvas draw Sight2:61–63/644/654 · legend hexes :1232–1255 · SightPanel maps :49–57) — user recolors never reach CNS; custom types fall to gray | Medium | registry-align (`linkTypeColor()`), legend becomes registry-driven |
| DF-07 | `lens.title` is English in all 15 locales (TOP PRINCIPAL violation) | Medium | localize ×14 (D7 confirms the Arabic) |
| DF-08 | **Wrong-key headers**: the Link-Types legend title and a settings section title render `searchHub.linksTo` — the UI literally says "Links to" (EN) / "الربط إلى" (AR) where "Link Types" is meant (Sight2:1231/:1184) | Medium (visible ×15) | proper keys ×15 |
| DF-09 | The dormant chip's count silently caps at 200 (`LIMIT 200`) | Medium (honesty) | dies with the shed |
| DF-10 | The "stagnating" insight tab queries `status='dormant'` = **0 rows** since the decay fix — returns empty forever | Medium | dies with D5 |
| DF-11 | Link enrichment reaches ≤10 of 234,062 links (`sample_links LIMIT 10`); the legend's thin/thick-confidence promise is invisible at scale | Medium | drop the call + the two legend rows (with the shed) |
| DF-12 | by_type/by_confidence **triplicated** (KH + CCS + CNS); most_connected + weak_foundations duplicated (KH + CNS-insights) | Medium (concept) | shed + D4/D5 |
| DF-13 | "Universe Health" vs "Knowledge Health" naming collision (G3) | Medium (concept) | ruling **D3** (rename CNS's score) |
| DF-14 | Dead surfaces: `constellation_sight_tag_edges` (fs walk, zero callers) + `ConstellationSight.svelte` v1 (zero importers, still consumes lens.title) | Low | delete both |
| DF-15 | Sight2:258 operator-precedence quirk — untyped links get `linkType: undefined` (no enrichment) vs `'relates'` (enriched); visually masked today | Low | cleaned up in the same file pass |
| DF-16 | No `data-style-target` on the CNS root (not inspect-targetable in the Style Setter); hover-label colors hardcoded (`rgba(30,30,40,.9)`/white — Sight2:852/876) | Low | tag the root; var the labels |
| DF-17 | Federation honesty: the well displays the federation while analytics silently cover the own universe only (S1 swallow) | Medium | one honest scope+layer caption (with DF-18); true federation = MIG-063 family, untouched here |
| DF-18 | The unexplained 233,538-vs-234,062 count difference (resolved graph edges vs recorded link rows) | Low | the same caption line (new key ×15) |

---

## 6. CNS's purpose, restated (the concept to ratify — PROPOSAL)

> **CNS is the topology instrument of the Connection question — the wiring diagram of the universe.**
> It answers one question: **"What is the SHAPE of my thinking — its regions, its bridges, its silences?"**
> Its registers are exactly four: **Regions** (communities — what clusters my thinking has formed),
> **Bridges** (centrality — which notes hold the structure together), **Blind Spots** (structural gaps —
> which regions should touch and don't, with suggested bridge notes), and the **Shape score** (the
> topology composite, renamed per D3). It reads structure only — who connects to whom — never flow:
> weight, decay, traversal, confidence, lifecycle are CCS's. Its **layout IS its answer**: distance from
> center = structural centrality; sector = your own organization; color = region. Sky View shows the sky;
> CNS tells you what the sky *means* structurally. Per-note signatures live in 360.3D; per-library tension
> lists in the health tab; counts in KH; the pulse in CCS.

This restores the dual-system design honestly: CCS shipped the circulatory register end-to-end (MIG-074);
CNS today is a nervous register that computes its diagnostics and shows almost none — MIG-075 makes the
Nervous side equally real. **Recommended: this statement ships as
`docs/Constellation-Nervous-System-Concept-Paper-v1.0.md` (short, CCS-v1.1-shaped) ratified before the
build — closing gap G1.**

---

## 7. The fix set (directed by Eisa 2026-06-11 — sequenced by the Plan, not re-asked)

**Slowness:** S1 centrality re-source (DB, async, contradictions kept for D2) · S2 shed + snapshot
re-points (zero live aggregates on open) · S3 measure-then-decide for the JS analytics (marks exist) ·
the tension re-source per D6.

**Design defects:** DF-05/06/07/08/09/10/11/14/15/16/17/18 as resolved above; DF-03/04/12/13 per the
D-rulings; the orientation §3.5 sight.rs row already fixed (v2.67).

**Unchanged invariants** (carried from the v1 Architect §5, all still binding): TensionReport contract
frozen · no new write path, no observer effect (no `_link_traverse` from CNS, I2b) · CCS/KH code untouched
except per D4 · registry single-source wherever touched · no silent feature loss (Predecessor records
before edits) · ×15 i18n day one, machine-gated · zero boot-path additions, zero IPC while closed ·
sync→async with identical shapes · perf gated before/after on the live universe · revert-clean commits ·
BUG-015 discipline.

---

## 8. The decisions (the new ratification gate)

| D | Decision | My recommendation |
|---|---|---|
| **D1** | Write + ratify the **CNS Concept Paper v1.0** (§6 statement, CCS-shaped) as the build's docs-first step? | **Yes** — it closes G1 and becomes the contract every later change is audited against |
| **D2** | **The unrendered analytics** — what does CNS do with the computed-but-dark half? **(i)** Render the promise: communities paint the well (node/region color, the legend becomes real) + a **Blind Spots register** in the panel (ranked gap pairs + suggested bridge notes); contradictions stay CUT from CNS (CCS counts · TensionPanel pairs · 360 flags already cover them — no fourth surface). **(ii)** Cut the dead compute instead: pipeline shrinks to centrality+health+bridges, help doc shrinks to match. | **(i)** — it is the documented promise, the CE-paper's assignment, and the field-standard register set; (ii) is honest but makes CNS a thinner Sky View duplicate |
| **D3** | Rename CNS's "Universe Health" score (the G3 collision with the KH plug-in). Candidates: **"Structural Cohesion" (التماسك البنيوي)** · "Shape Score" · keep as-is | **Structural Cohesion** — says what it measures, ends the two-healths confusion; your naming call |
| **D4** | most_connected's single home: **(i)** CNS-insights canonical, KH's card retires (reverses MIG-073's "keep" — only you can) · **(ii)** both stay, CNS re-points to the cached key (consistency, no retirement) | **(i)** if D2-(i) ships (hubs are a topology register); **(ii)** if you'd rather not touch KH |
| **D5** | Knowledge-Insights tab strip's fate after the shed: the 4 circulatory/tension tabs go (strongest_evidence → CCS · weak_foundations → KH · stagnating → data-dead · tensions → TensionPanel); do the 2 topology tabs (most_connected, knowledge_gaps) **(i)** fold into the D2 analytics registers (one coherent panel, no tab strip) or **(ii)** stay as a small insights section | **(i)** — one panel, four registers, no leftover strip |
| **D6** | `detect_tensions` re-source (the v1 Q1/Q2, output-changing): same algorithm on DB inputs, async, `status='active'`; tag coverage widens (all scripts + frontmatter); the contradictions **×N occurrence suffix can't survive** (the DB stores one row per pair) | **Approve** — the pair list you act on is identical; ×N was repeat-noise within single source notes |
| **D7** | The Arabic CNS title (DF-07): **الجهاز العصبي للكوكبة** (sibling of الجهاز الدوري للكوكبة)? | Confirm the term (or give yours) — the other 13 follow the CCS native-title pattern |

---

## 9. STOP

Awaiting Eisa's D1–D7. On ratification: Phase-2 Plan (docs-first Concept Paper → S1/S2 perf commits →
the shed + panel recomposition → D2 registers → defect sweep → /simplify → 3-agent audit → staged Boss
tests per the Tutorial Rule), then the build cascades under Plan-Approval-Equals-Build-Approval.
