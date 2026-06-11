# MIG-075 — The CNS Rebuild-to-Spec (Plan / Phase 2)

**Status: PLAN — produced 2026-06-11, awaiting Eisa's approval. Build cascades on approval
(Plan-Approval-Equals-Build-Approval; stops only at the ★ Boss-test stages, genuine architectural
surprise, or completion).**

**Executes:** `docs/Constellation-Nervous-System-Concept-Paper-v1.1.md` (RATIFIED — Q1–Q7) +
`lab/reports/MIG-075-CNS-AUDIT.md` (slowness S1–S4 · defects DF-01…18, Boss-directed fixes).
**Function in hand: the CNS** — `ConstellationSight2.svelte` + `SightPanel.svelte` + `sight.rs` +
`tension.rs` + the `+layout.svelte` wiring.

**Ground rules binding every step:** the paper's principles §8 (registry single-source · no observer
effect · Rule-8 reads · reveal-on-demand · ×15 day-one) · TensionReport contract frozen · CCS/KH code
untouched except §B2's ratified Q6 retirement · no schema change anywhere · every locale edit
machine-gated (parse + leaf-delta + endings; en+ar CRLF, others LF) · Predecessor → Replacement records
in the session log BEFORE each retiring edit · each § = one revert-clean commit · svelte-check 0 new
errors + cargo check clean per commit · zero boot-path additions (nothing here touches boot or typing).

---

## §A — The engine (slowness S1 + the tension re-source)

### §A1 — `constellation_sight_centrality` re-sourced from the DB, async
- `src-tauri/src/sight.rs`: replace the per-library `scan_library_links` fs walk (sight.rs:64–78) with
  one indexed read off the active universe's DB: `SELECT source_name, target_name, link_type FROM
  note_links WHERE status='active'` (own-universe scope = exact parity — cUniverse scans already fail
  silently today, audit §0.5). Petgraph build, typed-link weights, Brandes + sampling, stratum weighting
  unchanged. Command becomes `#[tauri::command(async)]`. **The `contradictions` field is dropped from
  `LensCentralityData`** — its only consumer was a dead frontend prop (audit §2); the pair list is
  `detect_tensions`' (paper §5). Frontend: `toggleLens` step 9 + `lensContradictions` + the Sight2
  `contradictions` prop removed in the same commit (they are one contract).
- **Documented behavior note (accepted-delta class):** the fs scan pushed one edge per wikilink
  *occurrence*; `note_links` stores one row per (source, type, target). Repeated identical links inside
  one note no longer stack edge weight — centrality values may shift marginally. Same honesty class as
  the ratified ×N delta; recorded here, not hidden.
- **Tests:** a fixture-DB Rust unit test pinning node/edge counts + a known bridge ranking.
- **Verify:** CNS opens on the live universe with plausible counts; the first open no longer reads
  note files (no fs in the command); UI thread never blocks (async). Record open-time before/after via
  the existing `sight:*` performance.marks (S3's measure-first datum).

### §A2 — `detect_tensions` re-sourced from the DB, async (ratified D6)
- `src-tauri/src/tension.rs`: `scan_notes_recursive` replaced by three indexed reads scoped
  `library_name = ?` — notes (`note_meta.name/path/word_count`), links (`note_links` … `status='active'`),
  tags (`json_each(note_meta.tags_json)`) — building the same in-memory `NoteInfo` map; the four
  detections' code unchanged; command async. **TensionReport shape byte-identical**; TensionPanel
  untouched. Ratified deltas: tag coverage widens (all scripts + frontmatter); contradiction rows are
  per-pair (no ×N multiplier — the suffix branch goes); archived links excluded.
- **Tests:** unit tests pinning each delta + the <50-linked-notes inactive gate.
- **Verify:** the health tab analyzes the active note's library and renders all four sections; counts
  plausible vs pre-change (deltas documented in the session log).

### §A3 — dead-surface deletion
- Delete `constellation_sight_tag_edges` + `scan_note_tags_recursive` (sight.rs) + the lib.rs:420
  registration (zero callers, audit §0.6) and `src/lib/components/ConstellationSight.svelte` (v1, zero
  importers).
- **Verify:** repo-wide grep zero references; cargo + svelte-check clean.

## §B — The shed + the panel recomposition (boundary; Boss-approved + Q6)

### §B1 — the circulatory shed + "Circulation → CCS"
- `SightPanel.svelte`: Section 2 (Link Health — BY TYPE + BY CONFIDENCE bars + the dormant chip) removed
  with its two onMount IPCs (`constellation_link_stats`, `constellation_link_dormant`) and the local
  `LINK_TYPE_COLORS`/`CONFIDENCE_COLORS` maps; in its place one **"Circulation → CCS"** row dispatching
  the existing `constellation:open-ccs` event, hidden when `enabledFeatures.ccs === false`.
- `ConstellationSight2.svelte`: `loadLinkEnrichment` + the enrichment map + the two thin/thick legend
  rows removed (≤10-of-234k decoration, DF-11); the line-258 ternary quirk (DF-15) dies with the
  enrichment branch.
- i18n ×15 (machine-gated): drop `sightPanel.linkHealth/byType/byConfidence/dormantLinks`; add the
  CCS-link key.
- **Verify:** CNS open path fires zero live link-stat IPCs; the CCS link opens CCS; Overview + Bridges
  render unchanged.

### §B2 — Knowledge-Insights strip → Hubs (ratified Q6, atomic re-home)
- `SightPanel.svelte`: the 6-tab insights strip + `loadInsights` + `constellation_formulation_analysis`
  usage removed. A **Hubs** register row-set (top most-connected notes) renders from the **cached**
  `fmt_most_connected` via the existing KH snapshot IPC (one ~0.17 ms call; no live GROUP BY).
- `KnowledgeHealthDashboard.svelte`: the Most-Connected card retires **in the same commit** (Q6's
  explicit MIG-073 reversal); its keys drop ×15; KH's other cards + the mutual deep-links untouched.
- Predecessor map (recorded in the session log first): strongest_evidence → CCS Load-Bearing ·
  weak_foundations → KH card (stays) · stagnating → data-dead, CCS Cooling is the live read · tensions →
  TensionPanel · most_connected → **CNS Hubs** (new canonical home) · knowledge_gaps → retired (no
  ratified register; documented). i18n ×15: the 6 tab keys + `insights`/`noResults` drop; `hubs` keys add.
- **Verify:** zero `formulation_analysis` calls remain in CNS; Hubs renders; KH shows no most-connected
  card; repo grep clean.

### §B3 — names, title, captions (Q3 · Q7 · DF-07/08/18 + the four sub-metrics)
- **Structural Cohesion**: new key ×15 (en "Structural Cohesion", ar **التماسك البنيوي**, 13 native);
  `lens.universeHealth` dropped ×15; the score card shows **all four sub-metrics** (modularity ·
  dominance · entropy · connectivity — today only two render).
- **`lens.title` native ×14** (en unchanged): ar **الجهاز العصبي للكوكبة** + the 13 per the CCS pattern.
- **DF-08**: the two wrong-key headers get proper keys ×15 (`lens.legendLinkTypes`, `lens.linksSection` —
  exact names at build; the legend stops reading `searchHub.linksTo`).
- **DF-18/17**: one muted caption under the header count ×15 — the graph-layer scope line ("resolved
  connections · this universe").
- **Verify:** AR UI shows the native title + correct headers + caption; EN identical except the renames.

## §C — The registers (ratified Q2 + Q4; the dark half rendered)

### §C1 — Regions: the color lens + the register list
- `ConstellationSight2.svelte`: a header toggle (beside the panel/settings buttons) switches node fill
  **library-colors ↔ region-colors** (`communityColors` — the prop finally consumed); position never
  changes (Q2). The legend's static region dots become the live top-N regions (color · name · count).
- `SightPanel.svelte`: a **Regions** register — ranked list (suggested name · member count · dominant
  maturity from `communityProfiles`); rows highlight their members in the well on hover/select.
- i18n ×15: `lens.regions` family reused/extended; new keys machine-gated.
- **Verify (★ part of Stage 2):** toggle recolors live; legend follows; list matches the well.

### §C2 — Blind Spots: the founding register restored
- `SightPanel.svelte`: a **Blind Spots** register — ranked gap pairs (Region A ↔ Region B + gap score)
  each with its **suggested bridge notes** (`potentialBridges`, clickable → preview/open per the locked
  grammar). The `gaps` prop is finally consumed; `suggestBridges` output rendered.
- The unused `communityAssignments` prop: consumed by §C1's lens; whatever remains unused after §C1/§C2
  is removed (no dark computes — paper §11 rule).
- i18n ×15: `lens.blindSpots` family (reuse `structuralGaps`/`blindSpotDesc` where the values fit).
- **Verify (★ Stage 2):** gaps list renders with suggestions; clicking a suggestion previews/opens.

## §D — Registry + theme alignment (DF-06/16)

### §D1 — one color source + Style-Setter reach
- `ConstellationSight2.svelte`: the hardcoded `LINK_TYPE_COLORS` (canvas draw :61–63/:644/:654) and the
  six hardcoded legend hexes replaced by `linkTypeColor()` (Link-Type Registry); the legend's type rows
  become registry-driven (canonical order, 8 + custom — raw-key leak impossible: labels via the
  registry-label fallback chain, DF-05's family closed for the legend too).
- Root gets `data-style-target` (inspect-targetable); the canvas hover-label colors (:852/:876) move to
  theme vars with today's values as fallbacks.
- **Verify (★ Stage 3):** recoloring a link type in the Style Setter recolors CNS edges + legend live;
  a custom type shows its color + label; inspect finds CNS.

## §E — Docs + PCS (Standing Order)

### §E1 — help + manual + records
- The EN CNS help topic rewritten to the ratified paper (the registers as they now exist; the stale
  "CNS vs Sight" section replaced by CNS-vs-Sky-View + CNS-vs-CCS; the locked grammar; the Cohesion
  name). User Manual section updated. The 14-language help mirrors ride the standing batched
  translation-sync debt (the MIG-074 pattern) — noted, not silently skipped.
- Orientation bump + session log per commit throughout (LL-031); Pending-Jobs note at close-out: PJ-035
  similarity = deferred-indefinitely (Q5), PJ-036 layer peeling = CNS-roadmap (Q5), PJ-037 = rejected
  (already closed).

## §F — /simplify + audit + perf gate

- `/simplify` on the full diff range (§A1 → §E1).
- **3-agent audit**: invariants (the paper §8 set + I2b + TensionReport freeze + KH-untouched-except-Q6)
  · drift (LL-023: every caller of the changed IPCs; the retired keys ×15; single color source) ·
  migration path (first boot on an existing universe; old binary on the new locale files; rollback;
  mid-anything interrupts — no schema, so the surface is small).
- **Perf gate (before/after on the live universe, binary mtime Stage-0-checked):** CNS first-open ·
  panel open · health-tab first activation · boot · typing. The S3 ruling executes here: if the
  `sight:*` marks show the JS Louvain still janks at federation scale post-§A1, the worker-offload
  decision is brought to Eisa as a follow-up — measured, not assumed (LL-015).

---

## Boss-test stages (staged per the Tutorial Rule — sent one at a time)

| ★ | After | What you'll test (tutorials at the gate, per the Testing Instructions Rule) |
|---|---|---|
| **Stage 1** | §A1–§B1 | **Speed + the shed**: CNS opens without the freeze (was: a corpus walk on first open); the side panel shows Overview + Bridges + the "Circulation → CCS" link (the type/confidence bars now live in CCS); the right-sidebar health tab still analyzes and renders its four lists |
| **Stage 2** | §B2–§C2 | **The registers**: the Regions color toggle; the Blind Spots list with suggested bridges; Hubs (and its card gone from KH); "Structural Cohesion" with four sub-metrics; the native Arabic title الجهاز العصبي للكوكبة |
| **Stage 3** | §D1 | **One color source**: recolor a link type in the Style Setter → CNS edges + legend follow; custom types show their colors; the AR pass (headers, captions) |

Close-out: §E/§F + milestone tag `milestone/mig-075-cns` + ZIP backup + PCS.

---

## Risks → mitigations (Phase-1/audit risks, each owned)

| Risk | Mitigation |
|---|---|
| §A1 centrality drift (occurrence-dedup) | documented accepted-delta + fixture test pinning rankings; Boss sees Stage-1 counts |
| §A2 output deltas surprise the panel | TensionReport shape frozen; deltas pinned by tests + stated in the Stage-1 tutorial |
| §B2 touches KH (the one allowed edit) | ratified Q6; atomic commit; Predecessor record first; revert-clean |
| Locale-file regressions ×15 | the established merge scripts + per-file delta gates (parse/leaf-delta/endings) |
| Louvain still slow after §A1 | measured at Stage 1 via existing marks; worker decision escalated, never silent |
| BUG-015-class reactivity | no new value-sync `$effect`s; §C toggles are plain `$state` + the untrack patterns already in file |
| Rollback | per-§ revert; no schema change; locale drops restorable from git; the §B2 KH card revert restores its keys |

**STOP: awaiting Eisa's Plan approval. On approval the build cascades §A1 → §F with stops only at
Stages 1–3.**
