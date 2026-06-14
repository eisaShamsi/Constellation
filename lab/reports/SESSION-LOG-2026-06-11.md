# Session Log — 2026-06-11

## §MIG-075 OPENED — the CNS-modernization /migration, Phase 1 Architect produced (morning)

**Function in hand: the CNS modernization** — `ConstellationSight2.svelte` + its `SightPanel` insight
sidebar + the tension subsystem (`tension.rs`), per `docs/handover/Handover-2026-06-10-CNS.md` §4/§5
(kickoff prompt executed verbatim).

**Session ritual:** `git pull origin main` (already up to date at `607ffa2e`); orientation **v2.66 read**
(the full §0–§17 body + the live preambles v2.66→v2.37; the older preamble stack swept by targeted grep —
it is the doc's own "retained for diff visibility" historical record); handover read in full; the
read-before-writing list read first-hand: CCS Concept Paper v1.1 §4+§12, MIG-074-CCS-ARCHITECT §0/§2.7/§3-a,
tension.rs (all 291 lines), SightPanel.svelte (all 290), the ConstellationSight2 right-panel/header/legend
regions, clusterEngine.ts health functions, the +layout wiring (toggleLens, loadTensionReport, overlay,
dock), SESSION-LOG-2026-06-09 §HEALTH-TAB FIX, SESSION-LOG-2026-06-10 (the MIG-074 arc + the Stage-1 CNS
cross-check record), Pending Jobs v1.13 (SO #8 cross-check — no reserved CNS MIG, no conflicts).

**Number allocation verified:** orientation §8 tops at MIG-074 (closed); repo-wide grep finds zero
MIG-075+ references → **MIG-075**.

**Deliverable: `lab/reports/MIG-075-CNS-ARCHITECT.md`** — territory map (every claim line-cited, every
file read first-hand), explicit in/out rulings (a)–(j), design options D1–D6 with speed/effort/risk,
invariants I1–I13, migration path (no schema change, no back-fill, revert-clean), and the Q1–Q8
ratification gate. **STOPPED after Phase 1 per the brief — no Plan, no code.**

**The brief's mandated verification — SETTLED:** the tag-cluster input IS queryable from the DB.
`note_meta.tags_json` (search.rs:1916) is written at index time (frontmatter `tags:` + inline `#hashtags`,
all scripts, lowercased — parse_frontmatter search.rs:3283–3338). Probed the live universe DB direct
(python/sqlite3): 7,592/7,661 notes tagged · 19,523 distinct tags · 36,176 (note,tag) pairs ·
note_links 234,062 all-active (0 archived, 0 dormant-status) · contradicts 1,794 · derives-from 97,958.

**New territory findings beyond the brief (each in the Architect §0):**

1. `detect_tensions` has exactly ONE caller (the health tab's `loadTensionReport`, +layout:3314) — the
   brief's "Sky View legend action" consumer died in the b2a23d4e re-home.
2. The two shed blocks live in **SightPanel.svelte** (mounted by ConstellationSight2 :1274 behind the
   `panelVisible` toggle) — component precision for the Predecessor record.
3. **The CNS open path is itself an fs walk**: `constellation_sight_centrality` (sight.rs:55, sync) →
   `scan_library_links` per library = full .md re-read of the corpus on first toggle. Surfaced as ruling
   (h), recommended IN.
4. **CNS analytics are silently own-universe-only** while the well displays the federation: cUniverse
   paths fail `scan_library_links`' own-library validation and sight.rs:67 swallows the error
   (`.unwrap_or_default()`). Same scope for the health tab (`validate_path_in_any_library` rejects
   cUniverse paths → `unavailable`). Pre-existing MIG-063-family gaps; MIG-075 keeps exact parity.
5. **Dead surfaces**: `constellation_sight_tag_edges` (fs walk, zero callers) + `ConstellationSight.svelte`
   v1 (zero importers) — ruling (i), recommended delete.
6. SightPanel's "stagnating" insight tab queries `status='dormant'` = **0 rows since the decay fix** —
   data-dead; CCS Cooling is its live replacement.
7. **most_connected is duplicated** (KH cached card + CNS live insights tab) — feeds ruling (f).
8. The SightPanel dormant chip's count caps at 200 (`constellation_link_dormant` LIMIT 200) — silently
   wrong at scale; dies with the shed.
9. Registry-color drift: CNS canvas edges + legend + SightPanel all carry hardcoded typed-link color maps
   (user recolors never reach CNS; custom types fall to gray) — ruling (j), recommended registry-align.
10. Orientation §3.5 still lists `lens.rs` (renamed `sight.rs` in MIG-009) — fixed in v2.67.

**The four handover rulings, answered in the Architect:** (c) `lens.title` is byte-identical English in
ALL 15 locales (probed) → localize ×14, Arabic recommendation **الجهاز العصبي للكوكبة** (Eisa confirms);
(d) caption mechanics pinned — CNS 233,538 = resolved sky-graph edges (both endpoints exist, self-links
excluded, buildSimData Sight2:245–270) vs CCS/KH 234,062 = recorded note_links rows incl. unresolved
targets → one muted CNS-side caption recommended; (e) **the Universe-Health score is topology-pure**
(clusterEngine.ts:332–375: 25·modularity + 25·(1−dominance) + 25·entropy + 15·connectivity + 10·(1−gaps)
— zero circulatory inputs; no change needed); (f) options drafted (status quo / CNS-canonical + KH-card
retire (reverses MIG-073's "keep" — Eisa-only) / cached-key re-point).

**Honest output deltas of the D1-A re-source, surfaced for ratification (not silently shipped):** tag
coverage widens (tags_json = all scripts + frontmatter lists vs the walk's Latin+Arabic inline-only);
the contradictions ×N occurrence suffix is not reproducible from note_links (write-time dedupe per
(type,target) per source) → pair rows without the multiplier (Q2); archived links become filterable
(`status='active'` — zero visible change today, 0 archived rows).

**Next:** awaiting Eisa's Q1–Q8 answers → Phase 2 Plan.

**Commit:** MIG-075 Architect + this log + orientation v2.67 (same commit per LL-031/SO #6). → `73ca61b0`

## §MIG-075 RE-SCOPED — Boss correction → the full CNS audit (midday)

**Boss (verbatim):** *"No. Since we are reviewing the CNS, I want you to fully audit its concept,
functions, and purpose, all against the CCS, KH, and the remaining core plugins. Also, I want you to fix
its slowness and design bugs/issues."*

**Stop-On-Correction applied:** all in-flight work stopped (none was code — the cascade had stopped at the
Architect gate); changes since last approval enumerated (the `73ca61b0` docs only); corrected understanding
stated; the Q1–Q8 framing withdrawn. The v1 Architect's §2 territory facts stand; a supersession banner was
added to `MIG-075-CNS-ARCHITECT.md`.

**The audit (5 parallel read-only agents + first-hand verification): `lab/reports/MIG-075-CNS-AUDIT.md`.**
Agents covered Sky View (full inventory — registry/skyPalette colors, themed, worker-simulated), KH + CCS
section inventories, the remaining dock plug-ins (orthogonality rows), CNS design-fact checks, and the
concept-doc sweep. Verified centerpiece findings:

1. **4 of 9 toggleLens pipeline outputs are computed and never rendered** — `communityAssignments`,
   `communityColors`, `communityProfiles`, `gaps` (+ bridge suggestions), `contradictions` are declared
   props (Sight2:90–97) with zero further references. The help topic documents "Communities" and "Blind
   Spots (Structural Gaps)" sections that do not exist in the UI. CNS's unique analytical promise is
   ~half dark.
2. **No CNS concept paper exists** (file sweep) — the only core plug-in whose "why" was never ratified
   (gap G1); the help topic still frames CNS vs the Wings-disabled Sight (G2).
3. **Naming collisions:** CNS "Universe Health" vs the "Knowledge Health" plug-in (G3); "load-bearing"
   structural (CE paper, bridges) vs circulatory (CCS register) (G4).
4. **Triplication:** by_type + by_confidence render in KH AND CCS AND CNS today; most_connected +
   weak_foundations duplicated KH/CNS-insights.
5. **Sky View boundary:** CNS's rendered subset = a poorer, unthemed, registry-blind node-link canvas;
   its differentiators are the slow/unrendered half. Target re-asserted: the layout IS the answer.
6. **New visible bug (DF-08):** the Link-Types legend + a settings header render `searchHub.linksTo` —
   the UI literally says "Links to"/"الربط إلى" where "Link Types" is meant (Sight2:1231/:1184).
7. Theme story corrected vs the v2.27-era note: Sight2/SightPanel chrome uses theme vars with fallbacks
   (NOT dark-only); but zero `data-style-target` (not inspect-targetable) + hardcoded canvas label colors.
8. CAT_COLORS badge parity Sight2↔ConstellationMap: **identical** — the Badge-Taxonomy invariant HOLDS.
9. Orphans: the well renders ALL notes (no linked-subgraph filter in the component); SightPanel's orphan
   stat is real. (Corrects the MIG-060-era "CNS only shows the linked subgraph" note as it applies today.)
10. skyVersion invalidation fires on index updates/tag changes → the S1 walk recurs in normal editing use.

**Deliverable structure:** §1 canon + 4 identity gaps · §2 promise-vs-delivery (9 computed / 5 rendered) ·
§3 boundary matrix vs every core plug-in (+ WA#5 cross-check: Gephi/Obsidian/Burt) · §4 slowness anatomy
S1–S4 · §5 defect register DF-01…DF-18 · §6 the CNS purpose statement (Concept-Paper-v1.0 candidate) ·
§7 the directed fix set · §8 decisions D1–D7 · §9 STOP.

**Gate:** D1 concept paper · D2 render-vs-cut the dark analytics (recommend render: communities paint the
well + a Blind-Spots register; contradictions stay cut from CNS) · D3 rename "Universe Health" (recommend
"Structural Cohesion") · D4 most_connected home · D5 insights-strip fate · D6 tension re-source + the ×N
delta · D7 the Arabic CNS title. Slowness fixes S1–S3 + defect sweep = directed by the Boss, sequenced by
the Plan — not re-asked.

**Commit:** the audit + the Architect banner + this log §2 + orientation v2.68 (same commit per LL-031).
→ `22f93e3f`

## §MIG-075 — Boss direction: the CNS Concept Paper FIRST, built from the origin (10:10–10:30)

**Boss (verbatim):** *"Let's first develop a complete CNS Concept paper, and we will go from there. To
develop this paper, I want you to go all the way back to when I first created the Constellation Sight
concept. Originally, the CNS was one of many Sight failed projects. You have to UNDERSTAND the whole
idea."*

**The lineage read (3 parallel agents + first-hand reads):** read MYSELF in full —
`Constellation-Sight-Concept-Paper-v1.1.md` (the port of Eisa's April-2026 original; 447 lines) +
`Constellation-Sight-Subsystem-Concept-Paper-v1.0.md` (2026-05-19; 284 lines) + the original PDF's opening
pages via pypdf (port verified faithful). Agents covered: the v2.0→v3.1 papers arc, the v4.0/v4.1 +
redesign-v0.x + v5-audit arc, and the session-log chronology 2026-05-05→20 (MIG-016/017/018/019/024/025/
026/027/028/038 + the A/B verdict).

**The decisive records recovered:**
- April 2026: Eisa's founding paper — "The Constellation Lens — A Text Network Analysis Engine for
  Knowledge Discovery"; the question *"What patterns and gaps exist in my thinking?"*; Brandes + Louvain +
  Burt structural holes + the M/D/E/C composite; local-first; the plain-language label rule.
- 2026-05-07: v1.1 truth-status scores v2 at "~70–80% of the analytical promise delivered"; Eisa's
  secure-don't-muddle ruling shelves v2 as the known-good fallback (MIG-017).
- The odyssey: v3 died on architecture (13 close-button iterations → "abandon v3, start from scratch as
  v4"); v5 REVOKED the network-science framing ("Brandes… Louvain… universe-health metrics is OUT") and
  failed the outcome test ("What is unique about Sight? NOTHING!"); v6 = the 24-tradition domes.
- **2026-05-14 (SESSION-LOG-2026-05-14:298–359): the A/B verdict — "Sight v2 = Working. I decided to keep
  it." + Eisa's own naming: "Then, it is going to be: Constellation Nervous System (CNS)"** (anatomical
  pair with Sight=sensory; SME proposals rejected).
- 2026-05-19: Sight v6 + Map → Wings (MIG-038); the Core-Plug-in taxonomy ruling keeps CNS in the dock.
  **The Subsystem paper written that day omits CNS from its own diagnostic-subsystem list** — the identity
  gap made visible.
- Code-truth pinned for the paper: today's well = deterministic gravity-well (centrality rings × library
  sectors; "No community detection. Libraries are the user's own organization." — Sight2:292–294) + a
  15-tick collide-only relaxation; the original free force layout is gone.

**Deliverable: `docs/Constellation-Nervous-System-Concept-Paper-v1.0.md` (DRAFT for ratification).**
Thesis: **CNS is not a failed Sight project — it is the ORIGINAL project** (Eisa's April-2026 analytical
engine), which survived every Sight redesign because "it worked," while the Sight name wandered to the
taxonomy/domes and was externalized. Structure: §0 why this paper exists · §2 the full dated lineage ·
§3 Why "Nervous" · §4 the gravity well (the layout IS the answer) · §5 the four registers (Regions ·
Bridges · Blind Spots · Structural Cohesion) · §6 the CCS dual-organ boundary + the "load-bearing belongs
to CCS" vocabulary ruling · §8 the founding principles restated · §11 the truth-status matrix (founding
mechanics vs today: communities/gaps computed-but-dark; tags/similarity/peeling unbuilt; PJ-037 rejected
forever) · §13 the Q1–Q7 ratification gate (identity · Regions-as-color-lens · the score rename ·
Blind Spots restore · peeling/similarity disposition · Hubs/KH · the Arabic title الجهاز العصبي للكوكبة).

**STOPPED — awaiting Eisa's Q1–Q7 on the paper. MIG-075's Plan follows the ratified paper.**

**Commit:** the paper + this log §3 + orientation v2.69 + MoCh (same commit per LL-031). → `81a28111`

## §MIG-075 — the KEEP decision + paper RATIFIED + Plan produced (10:30–11:00)

**Boss asked the existential question (verbatim):** *"I want you to give me your honest opinion. I want
to avoid patching an earlier version. Instead, I want you to provide me with a decision: Does the
Constellation Cognitive system need it? And why?"*

**The delivered decision: YES — keep CNS as the structural organ of the Connection question, rebuilt to
its own founding spec, never patched.** Three reasons: (1) the Connection question is constitutionally
two-sided (KF §1.1) and CCS just shipped the flow side — killing CNS orphans the ratified CCS §4 boundary
("the blood shipped, the nerves amputated"); (2) **Blind Spots is the only forward-looking instrument in
the entire Cognitive system** — everything else records connections already made; structural gaps point
at connections NOT yet made (the Five Acts' Synthesis step instrumented; Burt; the founding paper's "dark
sky"); (3) the emergent-vs-declared overlay (found Regions over the user's declared library sectors)
exists nowhere else. Alternatives stated honestly: merge-into-Sky-View rejected (the exact modes shape
that killed Sight v5 — "What is unique about Sight? NOTHING!"); retire-to-Wings stated as the consistent
kill-path but not recommended; keeping CNS AS-IS ruled out absolutely.

**Boss (verbatim): "I accept your logic. Proceed!"** → the paper is RATIFIED with Q1–Q7 all as
recommended.

**Shipped this hour:**
1. **`docs/Constellation-Nervous-System-Concept-Paper-v1.1.md` — RATIFIED** (new file; v1.0 draft
   preserved). v1.1 locks the identity, adds **§2.1 The keep decision** (the question + the three-reason
   verdict + the rejected alternatives, verbatim-quoted), and converts §13 to the ratified rulings:
   Q1 identity ✅ · Q2 Regions = a color LENS (position untouched) ✅ · Q3 **Structural Cohesion /
   التماسك البنيوي** ✅ · Q4 Blind Spots restored with suggested bridges ✅ · Q5 layer peeling = roadmap,
   similarity = deferred ✅ · Q6 **CNS carries Hubs; KH's most-connected card retires** (explicit MIG-073
   reversal) ✅ · Q7 the native title ×15, ar **الجهاز العصبي للكوكبة** ✅.
2. **`lab/reports/MIG-075-CNS-PLAN.md` — Phase 2 produced, awaiting Plan approval.** §A engine (A1
   centrality DB-re-source + async + drop the dead contradictions field · A2 tension re-source, contract
   frozen · A3 delete tag_edges + v1 component) → §B boundary (B1 the shed + "Circulation → CCS" ·
   B2 insights strip → cached Hubs + the atomic KH-card retirement · B3 Cohesion rename + native titles +
   four sub-metrics + DF-08 headers + the scope caption) → §C registers (C1 Regions lens + list ·
   C2 Blind Spots + suggested bridges) → §D registry/theme alignment (one color source +
   data-style-target) → §E docs → §F /simplify + 3-agent audit + the perf gate (Louvain worker decision
   measured-not-assumed). ★ Boss stages: 1 speed+shed · 2 registers · 3 colors/AR. Accepted-delta class
   documented (§A1 occurrence-dedup; §A2 ×N + tag-coverage + archived-filter).

**Next: Eisa's Plan approval → the build cascades §A1 → §F.**

**Commit:** paper v1.1 + the Plan + this log §4 + orientation v2.70 (same commit per LL-031). → `6f788cf6`

## §MIG-075 BUILD — Plan APPROVED + the Q7 amendment; the cascade opens (11:05 →)

**Eisa (verbatim): "Approved. But, I will use 'الجهاز العصبي للمعرفة' as the Arabic equivalent to
'CNS'."** → Plan approved; the Arabic title is HIS term — **الجهاز العصبي للمعرفة** (the Nervous System
of Knowledge) — amending the recommended للكوكبة. Paper revised at
`docs/Constellation-Nervous-System-Concept-Paper-v1.2.md` (one-ruling revision; v1.0/v1.1 preserved);
orientation v2.70 carries the dated addendum. **Judgment surfaced, not hidden:** the 13 non-ar locales
will follow Eisa's semantic (*of Knowledge*, native forms) rather than the للكوكبة pattern; en keeps the
brand form per the 2026-05-14 ruling. This lands at §B3 and is called out in the Stage-2 tutorial for
cheap correction if Eisa wants different.

**Build order (per the approved Plan):** §A1 → §A2 → §A3 → §B1 → ★ Stage 1 → §B2 → §B3 → §C1 → §C2 →
★ Stage 2 → §D1 → ★ Stage 3 → §E → §F. Each § = one commit; Predecessor records land here before each
retiring edit.

**§A1 SHIPPED** — `constellation_sight_centrality` re-sourced from `note_links` (one indexed read,
`status='active'`; the DB lock held for the read only) + `#[tauri::command(async)]` + the `library_paths`
param dropped (scope = the active universe, exact parity — cUniverse scans previously failed validation
silently). The pure core extracted as `compute_centrality_from_links()` for tests. **Weight-parity
mapping**: the indexer stores plain untyped links as `'associative'` (legacy `'relates'`) while the
retired fs resolver returned `None` → both map to `None` so untyped edges keep weight 1.0. The
`contradictions` field dropped from `LensCentralityData` (its only consumer was a dead prop; the pair
list is `detect_tensions`' per the paper §5) — toggleLens step 9 + `lensContradictions` + the Sight2
prop removed with it. 3 new tests (`tests_mig075_sight`): bridge-ranks-highest · row hygiene +
occurrence-dedupe (the documented delta) · empty payload — **3/3 pass**; cargo check clean;
svelte-check **0 errors** (317-warning baseline unchanged). → `76597676`

**§A2 SHIPPED** — `detect_tensions` re-sourced from the DB: `load_notes_from_db()` (three scoped reads —
note_meta name/path/word_count · note_links active rows attached via a path→key map · tags via
`json_each(tags_json)`) feeding the UNCHANGED four detections, extracted as the pure `detect_from_notes()`.
Command async; `library_name` now does the scoping (was `_`-ignored); `validate_path_in_any_library` kept
(cUniverse paths still refuse → the tab's honest `unavailable`, parity). The fs walk + both regexes
deleted. TensionReport shape byte-identical; TensionPanel untouched. Deltas documented in the module doc
(tag coverage widens · per-pair contradictions without ×N · archived excluded · stripped-markdown
word_count at the severity margins). 4 new tests (`tests_mig075_tension`): the 3-input loader + library
scoping + active-only · the <50 gate · per-pair-no-multiplier · orphan/gap/SPOF on a 51-note chain
fixture — **7/7 MIG-075 tests pass** (with §A1's). *(Transient LNK1104 file-lock on the test exe ×2 —
cleared by retry; not a code issue.)* → `2bab0cfd`

**§A3 SHIPPED** — dead-surface deletion, all verified zero-callers/zero-importers BEFORE deleting:
`constellation_sight_tag_edges` + `scan_note_tags_recursive` + `TagEdge` (sight.rs §Shared-Tag-Edges —
an fs walk with no frontend caller) + the lib.rs registration; **`ConstellationSight.svelte` (v1)** —
zero importers; **`LensPanel.svelte`** — its v1-era sidebar, zero importers (found in the §A3 verify
sweep, same family); the three dead `+layout` states (`lensShowTagEdges`/`lensPeelCount`/`lensTagEdges` —
zero readers; layer peeling stays roadmap per paper Q5). cargo check clean; svelte-check **0 errors**,
warnings 317→313 (the dead files carried 4); repo-grep zero live references. → `7c9bb6c6`

**§B1 SHIPPED — the circulatory shed + "Circulation → CCS".** Predecessor → Replacement (recorded with
the retiring commit): SightPanel "Link Health" section — BY TYPE bars → **CCS Acts of Inquiry** · BY
CONFIDENCE bars → **CCS Conviction & Doubt** · the dormant chip (LIMIT-200-capped count) → **CCS Cooling
Inquiries** — replaced by ONE "Circulation → CCS" row dispatching the existing `constellation:open-ccs`
(gated `enabledFeatures.ccs !== false`, mirroring the +layout listener). The two onMount live IPCs
(`constellation_link_stats` + `constellation_link_dormant`) removed; `CONFIDENCE_COLORS` died with the
bars. ConstellationSight2: `loadLinkEnrichment` + the ≤10-link enrichment map + the second application
pass + `drawLinkAnnotation` + the hoveredLink hover loop + the thin/thick legend rows removed; SimLink
trimmed to `linkType` only; the draw constants fold in the values ≥99.99% of edges already rendered with
(`LINK_WIDTH_MUL = 0.7`; typed base 0.8) — pixel-equivalent at scale; the DF-15 ternary quirk died with
the enrichment branch (`linkType: l.linkType || undefined`). **Locales ×15 machine-gated** (parse OK ·
sightPanel leaves 18→15 = −4 dropped {linkHealth, byType, byConfidence, dormantLinks} +1 `openCcs` ·
en/ar CRLF + 13 LF preserved · outside-block bytes identical): `sightPanel.openCcs` = "Circulation →
Circulatory System" / "الدورة ← الجهاز الدوري" (RTL arrows per the MIG-074 hub precedent) /
"Циркуляция → Кровеносная система" / "循环 → 循环系统" …. svelte-check **0 errors**; zero stale key
consumers (repo-grep). → `63b097c8`

**★ STAGE 1 VERDICT (Eisa):** Test 1 **PASS** — "less than 3 seconds from click to paint" (the corpus-walk
freeze is gone; the S3 worker question parks as not-needed-now). Test 3 **PASS**. Test 2 **FAIL** —
clicking the CCS row did nothing. Plus two findings with screenshots: (1) the well not centered in the
center zone; (2) zooming revealed clusters hidden beyond the right edge — "I want to take advantage of
the whole available space."

**§B1-fix-1 — the dead CCS click, root-caused:** the +layout listener is registered on **`document`**
(+layout:2302; the MIG-007 hub dispatches on document too, SettingsModal:1394) — SightPanel dispatched on
**`window`**, and window-dispatched events never reach document listeners. One-line fix:
`document.dispatchEvent`.

**§B1-fix-2 — the off-center well + hidden content, root-caused:** the canvas was sized ONCE at mount
(`getBoundingClientRect` → inline `style="width:{width}px;height:{height}px"`) with **no resize handling
at all** — any later size change (maximize, dock/sidebar/panel width) left a stale coordinate space:
well off-center in the grown wrap, content clipped beyond the frozen box (the Stage-1 screenshots; the
PJ-062-family "fixed-size layout assumptions" bug, pulled into MIG-075 per the Boss's Stage-1 directive).
Fix: the canvas box is CSS-bound (`width/height: 100%`); a **ResizeObserver** on the wrap keeps the
bitmap + dpr transform (`setTransform`) + the view in step; `userAdjustedView` (set by wheel/pan/the
MIG-060 focus gesture, cleared by Fit-to-screen) decides re-fit vs preserve-view on resize; observer
disconnected in onDestroy (Rule 4). svelte-check 0 errors. → `4395c52c`

**★ STAGE 1 RE-TEST (Eisa): A PASS · B PASS** — with one new ask folded into §B2: *"I want to be able to
go back to the CNS from the CCS."*

**§B2 SHIPPED — Hubs to CNS · KH's card retired (atomic, ratified Q6) · the CCS→CNS back-link.**
Predecessor → Replacement: the SightPanel six-tab insights strip — strongest_evidence → **CCS
Load-Bearing** · weak_foundations → **KH's card (stays)** · stagnating → data-dead (CCS Cooling is the
live read) · tensions → **TensionPanel** · most_connected → **the new CNS Hubs register** (canonical
home; KH's Most-Connected card retired in the SAME commit — its `onOpenCcs` deep-link lives in the KH
HEADER, untouched) · knowledge_gaps → retired (no ratified register; documented). Hubs reads the
**cached** snapshot (`constellation_knowledge_health_snapshot` → `most_connected`, one ~0.17 ms call) —
**zero live link IPCs remain on the CNS panel-open path**; rows reuse the parametrized
`knowledgeHealth.incomingLinks` meta. SightPanel's `LINK_TYPE_COLORS` + the insight-tab CSS died with the
strip. **The back-link (Eisa's Stage-1 ask):** CCSView gains `onOpenCns` — a `{lens.title} →` header
button beside the KH one; wired in +layout mirroring the CNS dock button's open path (toggleLens
cache-or-compute + the exclusion list), gated `SIGHT_V2_ENABLED && enabledFeatures.constellationSight`.
**Locales ×15 machine-gated** (sightPanel −7 +`hubs`; knowledgeHealth −2 {connected, connectedEmpty};
first run found ZERO written files on a matcher miss — the leaf key `ribbon.knowledgeHealth` shadowed the
block; fixed to require the `": {"` opener; second run ALL 15 OK). svelte-check **0 errors**; stale-grep
clean (the long-flagged dead `store.ts` formulation wrapper stays for §F /simplify per the MIG-073
precedent). → `7994519a`

**§B3 SHIPPED — the names.** `lens.title` goes NATIVE ×14 (en keeps the brand form per the 2026-05-14
ruling): ar **الجهاز العصبي للمعرفة** (Eisa's v1.2 term) · de Nervensystem des Wissens · ru Нервная
система знаний · zh 知识神经系统 · ja 知識の神経系 · he מערכת העצבים של הידע … (all follow the
of-Knowledge semantic). **"Universe Health" → "Structural Cohesion"** (ratified Q3): new
`lens.structuralCohesion` ×15 (ar التماسك البنيوي), `lens.universeHealth` dropped ×15 (zero consumers
verified); the score card now shows **all four sub-metrics** (modularity + dominance added to
connectivity + entropy; presence of all four label keys verified ×15 first — no leak risk). **DF-08
fixed**: the legend + settings headers stop borrowing `searchHub.linksTo` ("الربط إلى") — proper
`lens.legendLinkTypes` + `lens.linksSection` ×15. **The scope caption** (paper ruling d): a faint
`lens.linkCountNote` ×15 under the header count ("resolved connections · this universe" / "روابط محلولة
· هذا الكون"). Locale gates ALL 15 OK (+4 −1 per lens block; title value-replace ×14; CRLF/LF
preserved). svelte-check **0 errors**. → `7c59f323`

**§C1 SHIPPED — the Regions lens + register** (`9e7def9f`). The header gains a Regions toggle: node fill
switches library-colors ↔ region-colors — **Louvain's output finally consumed** (the ratified Q2: a color
LENS; position never moves). The legend's color block goes live under the lens (top-5 regions: dot +
suggested name + count). The panel gains the **Regions register** (top-10 by size; dot · name `dir=auto` ·
count · the dominant-maturity character hint via the localized `graphView.ns*` vocabulary — presence
verified ×15 first); hovering a row dims the well to that region (`highlightRegionId` alpha branch).
`sightPanel.regions` ×15 gated. One new a11y warning caught and fixed (`role="list"`) — the 313 baseline
restored, not grown.

**§C2 SHIPPED — Blind Spots, the founding register restored.** The panel renders the top-8 structural
gaps — *Region A ↮ Region B* — each with up to 3 **suggested bridge notes** as clickable chips
(`onNoteClick` → select + center, the existing path); bridge ids resolve to display names via the
sim-node lookup. An empty list reads as good news (`noResults`). `panelGaps` built once per mount;
**every formerly-dark prop is now consumed** (communities, assignments, colors, profiles, gaps +
suggestions) — the paper §11's no-dark-compute rule is satisfied in full.
`sightPanel.blindSpots`/`suggestedBridges` ×15 gated. svelte-check **0 errors** (313 baseline).
→ `8d38acfb` (+ orientation addendum `e1ecc9f6`)

**★ STAGE 2 VERDICT (Eisa): "App Passed."** Two remarks → **§C-fix-1+2** (`6699430d`): the CNS panel
widens 280→380px (the Blind Spots pair rows were choking — screenshot; matches the app right-sidebar
convention; bridge chips widen too) + **the B/W region mute** — hovering a Regions row now renders
everything OUTSIDE the region in luminance-gray at 0.35 alpha with colored decorations skipped (was
near-invisible 0.08); gray conversion memoized per hex (no per-frame parsing).

**§D1 SHIPPED — one color source + Style-Setter reach.** The hardcoded `LINK_TYPE_COLORS` is GONE from
CNS: typed-edge colors read **`linkTypeColor()`** (custom types finally colored, not gray);
`'associative'` keeps its named tint (`ASSOCIATIVE_TINT`, today's value — the open-question edge);
legacy `'relates'` = plain untyped. **The legend's six hardcoded rows are replaced by the
registry-driven list** (canonical order, custom types included, labels via the locale → registry-label
→ id miss-guard chain, `dir=auto`). **Live recolors** reach the open well via `subscribeLinkTypes`
(unsubscribed in onDestroy, Rule 4). **DF-16**: the root carries `data-style-target="cns"`
(inspect-find-able; the Setter CATEGORY wiring — which CNS controls to expose — is a noted follow-up,
not silently built); the hover-label colors read optional `--cns-label-bg/-text` theme vars ONCE at
mount (Rule 3 — never on the draw path; unset = today's look). svelte-check **0 errors**;
`LINK_TYPE_COLORS` grep = gone from CNS. → `b8a5f411`

**★ STAGE 3 VERDICT (Eisa): "Perfect! Pass."** + a feature add → **§C-fix-3** (`4045d014`): **pinnable
regions** — clicking a Regions row pins its B/W mute (toggle; accent bar + dot on the pinned row); hover
previews over the pin; node hover/click work as usual inside the pinned view, and a selected node's
neighbors keep full color across regions (the cross-region read). Rows became real buttons.

**§E SHIPPED** (`1d9873ad`) — the EN CNS help topic rewritten in full to the ratified paper (the five
registers as built incl. hover-mute + pin; the layout grammar; the caption; the CCS dual-organ framing;
CNS-vs-neighbors replacing the stale CNS-vs-Sight; the old doc's over-claims corrected); User Manual §8b
rewritten likewise (incl. الجهاز العصبي للمعرفة). The 14-language help mirrors ride the standing batched
debt (the ar mirror is now stale against the rewrite — logged).

**§F /simplify** (`4a2b2d6f`) — 4 agents (reuse/simplification/efficiency/altitude): **2 real fixes
applied** (the unreachable ×N branch in tension.rs removed — the pair map stays as the defensive dedupe;
ONE `communityAssignments.get` per node in the draw loop + the maturity-alpha literal hoisted) — the rest
verified clean or skipped with reason (DB-lock idiom = codebase norm; toGray vs linkTypeTextColor =
different outputs, extraction not worth it; the null-type predicate centralization = REAL but needs
shared Rust+TS semantics → **noted follow-up**). svelte-check 0; MIG tests 7/7; **FULL Rust suite
903/903**. *(Process note: a PowerShell `-replace`+`Set-Content -NoNewline` attempt flattened Sight2's
newlines mid-fix — caught immediately, restored from HEAD, redone via the Edit tool. Dedicated tools for
source edits, always.)*

**§F 3-AGENT AUDIT — PASS.** **Invariants 11/11 HOLD** (4A delivered 8 HOLD + 3 cannot-determine on
file-access grounds; all 3 closed first-hand: I6 = the Predecessor records in this log's §B1/§B2; I10 =
`resizeObs?.disconnect()` + `unsubLinkTypes?.()` both in onDestroy, grep-verified; I11 = ZERO new
`$effect` in the whole range, diff-verified). **Drift CLEAN** (4B: single caller per changed IPC,
correct shapes; removed surfaces zero live references; added keys 15/15 per file; dropped keys zero
consumers; the open-ccs event = 2 document dispatchers → 1 document listener; the onOpenCns gate matches
the dock's; searchHub.linksTo's remaining consumers all legitimate) — **one finding followed through**:
the 7 `lens.link*` keys went dead when the legend turned registry-driven → **dropped ×15** (gated, −7
per lens block, zero consumers confirmed). 4B also noted `diversivity` is computed+serialized but
frontend-unread — left in the payload (harmless; possible future read), noted. **Migration path 8/8
PASS, zero risk** (4C: empty-universe paths, pre-MIG-067 NULL/'relates' rows, locale rollback both
directions via the fallback chains, no persisted state anywhere on the new paths, the cUniverse
`unavailable` parity, both feature gates, the snapshot self-heal).

**§F PERF RECORD (honest, no invented numbers):** CNS first-open — **Eisa's lived Stage-1 measurement:
"less than 3 seconds from click to paint"** on the 7,661-note / 234k-link universe (was: a corpus
re-read of every .md on the UI thread + per-edit recurrence; the walk is structurally gone — the engine
reads ONE indexed query). The CNS panel open: **zero live link IPCs** (was 3, incl. two full-table
GROUP BYs of the pre-MIG-073 KH-freeze family + a LIMIT-200 julianday scan). The health tab: DB-sourced,
async, Stage-1 PASS. Boot/typing: **zero boot-path changes** (audit-verified); editor untouched. Tests:
903/903 lib + 7/7 MIG-075. The S3 Louvain-worker question: **closed not-needed** at Eisa's <3s verdict.

---

## Follow-up block — "Pass. Proceed with the follow-ups" (afternoon)

Eisa's final Stage-3 verdict ("Pass") authorized the four noted follow-ups. All four shipped:

**FU-2 + FU-4 — `57f230c4` — the null-type predicate centralized + diversivity dropped.**
The ONE definition of "a null link type" (untyped / the open question, vs a typed cognitive act) now
lives in exactly two mirrored places: `src-tauri/src/link_types.rs::is_null_type(id)` (matches
`associative` | legacy `relates` | empty) and `src/lib/libraries/linkTypeRegistry.ts::isNullLinkType(id)`
(same membership + undefined/null defensively). Both prior inline call sites re-pointed: sight.rs's
centrality row-mapper (None-out null types so the edge renders as the untyped tint) and Sight2's
`edgeColorFor` (ASSOCIATIVE_TINT for `associative`, null for legacy/empty → base edge color). The
/simplify finding that motivated it is closed — membership can never drift between Rust and TS again.
FU-4 in the same commit: `diversivity` removed from `LensCentralityData` + its computation loop +
the test assertion — it was serialized on every centrality response and read by nothing (audit 4B
finding; the paper's §11 no-dark-compute rule applied to the payload itself). Tests 7/7; svelte-check 0.

**FU-3 — `aa8626bf` — the Style Setter CNS category.**
ELEMENTS gains `cns` (3 wired controls: `--cns-bg` — live on `.sight2-root` via the FU-3 root CSS
fallback chain; `--cns-label-bg` / `--cns-label-text` — mount-read by the canvas, documented
apply-on-next-CNS-open); CATEGORIES gains the `cns` row after Sky View (surface `cns` → the ⌖ inspect
crosshair finds the well via the §B-era `data-style-target="cns"` tag); the preview pane gains a
`{:else if activeSurface === 'cns'}` mini gravity-well (`.ss-cnsprev`: 3 dashed percentile rings +
a hover-label chip reading the three vars live). Caveat logged: the 3 control labels ride the
built-in English fallback (`L(en)`) until the next `styleSetter.labels` batch ×15. svelte-check 0.

**FU-1 — the 14-language help batch (28 files).**
14 parallel translator agents, one per locale (ar de es fa fr he hi ja ko pt ru tr ur zh), each writing:
(1) `docs/help.{lang}/Constellation Nervous System/Constellation Nervous System.md` — OVERWRITE of the
stale 2026-05-16 mirror (which still described the retired Communities/Universe-Health CNS) with the
full translation of the rewritten English topic (gravity well, five registers, Regions lens, Structural
Cohesion, interaction table, CNS-vs-neighbors); (2) `docs/help.{lang}/Constellation Circulatory System/
Constellation Circulatory System.md` — NEW (first translation of the MIG-074 topic; directory created).
Convention held by all 14: English folder/filenames; locale-verbatim UI vocabulary lifted from each
`{lang}.json` (lens.title natives — ar الجهاز العصبي للمعرفة per the v1.2 paper; the seven CCS register
titles + questions; sightPanel/cohesion/lifecycle/confidence terms); English + translated aliases in
frontmatter; `language: {lang}` · `source:` · `translation_status: AI-generated 2026-06-11 —
native-speaker review recommended`. Two source-vs-UI notes consistently resolved against the running
code by the agents: the doc's "Diversity" ingredient renders the `lens.entropy` UI label; the Regions
register uses `sightPanel.regions`. Verified on disk: 28/28 exist + frontmatter fields present.
Known-noted, NOT in scope: the 14 translated User Manuals remain stale snapshots (pre-existing debt,
logged at MIG-074); Panels/KF topics have no translated mirrors anywhere (logged).

**Orientation v2.71** amended in-place with a dated follow-through line (the four "noted follow-ups"
are closed) — same commit as the FU-1 batch per LL-031.

**Release binary**: `npm run tauri build -- --no-bundle` rebuilt after FU-2/3/4 (the 14:19 binary
pre-dated them — Stage-0 mtime rule).

---

## FU-3 fix — Boss screenshot caught two defects (same afternoon)

Eisa's first look at the Setter CNS category: **"The name!"** (raw `styleSetter.labels.cns` /
`...nervous_system_cns` / `...hover_label_*` strings rendered as the category chip, element header,
and two control labels) and **"There is NO miniature gravity well."**

**Root causes** (both mine, both FU-3):
1. **The L() fallback was dead code on a miss.** `L(en) = $t('styleSetter.labels.'+slug) || en` —
   but svelte-i18n returns the KEY ITSELF (truthy) when the id is missing, so `|| en` could never
   fire. It never showed before because all 303 prior labels had keys in en.json from day one;
   FU-3 added 4 unkeyed strings and exposed it.
2. **The preview branch was unreachable.** `twoZone = activeCategory !== 'editor' && !== 'sky'` —
   every other category renders NO centre stage (the live-behind app is the preview). My
   `{:else if activeSurface === 'cns'}` mini-well sat inside the centre stage → never rendered.

**Fixes:**
- `L()` made miss-robust: key-echo treated as a miss → English fallback. Protects every FUTURE
  unkeyed label, not just these 4 (altitude: the mechanism, not the instance).
- The 4 keys added **natively ×15** (gated python insertion before the unique `"label_thickness"`
  anchor; parse + leaf-delta(+4) + EOL(CRLF en/ar, LF rest) + outside-block-identity gates ALL
  green): `cns` (chip — ar = Eisa's الجهاز العصبي للمعرفة; others = native short form + "(CNS)"),
  `nervous_system_cns`, `hover_label_background`, `hover_label_text` — each composed from the
  locale's OWN existing vocabulary (its lens.title + its background/label/hover/text terms).
- **CNS joined the three-zone set** (`twoZone` now also excludes 'cns') — same rationale as
  MIG-072 §2 gave Sky View (a focused labelled preview beats hunting a live canvas), plus a harder
  one: the hover-label vars are mount-read by the canvas, so the live-behind app structurally
  CANNOT preview them; only the mini-well (CSS-read off the draft) can. Behavior shift logged:
  on the CNS category the Setter no longer live-pushes the draft to the real app; the mini-well
  previews live, Keep applies (background immediate via CSS var; labels next CNS open).

svelte-check 0 errors (313-warning baseline). Orientation v2.71 follow-through line amended in the
same commit. Fresh release binary rebuilt for the re-test.

---

## Style Setter Preview Rule (Boss-dictated) + the CNS preview resized to the full zone

Mid-re-test, Eisa dictated a new standing rule before proceeding: **"Taking advantage of the entire
center zone. Not to squeeze any element mimicry in a tiny box."** The fixed CNS preview (correct
names, correct three-zone layout) still violated it: a 180×140 well inside the default 560×360
`.ss-prev-alt` card, floating in a ~1100×600 stage.

**Rule written durably** (Eisa: "Write this Style Setter rule"):
- `CLAUDE.md` → new **§ Style Setter Preview Rule** (after Form-Aligns-To-Purpose; canonical
  violation recorded).
- Memory: `feedback_style_setter_full_center_zone.md` + MEMORY.md index line.
- Code comments at the `.ss-prev-alt--cns` / `.ss-cnsprev` CSS (the point of application).

**Fix applied** (the `--sky` precedent generalized):
- New `.ss-prev-alt--cns` modifier — the card stretches to the stage (width/height 100%,
  max-width 1100px, padded), exactly like `--sky` already did.
- `.ss-cnsprev` — fixed 180×140 → `align-self: stretch; flex: 1` (fills the card);
  rings fixed 36/76/116px → **28% / 56% / 84% of the well height** (`aspect-ratio: 1`,
  max-width 94% guard); label chip 10px/2×8 → 13px/4×12.

svelte-check 0 errors (313 baseline). Release binary rebuilt for the re-test. Noted in passing
(not touched): the org/index centre-preview branches are currently unreachable (those categories
are two-zone) — a future sweep candidate if they ever rejoin three-zone, at which point the
Preview Rule governs them too.

---

## Full-zone preview PASS + "Text size" added to the CNS category (Eisa's post-pass ask)

Eisa: **"Pass. I want you to add 'Text Size'"** — a fourth CNS control sizing the hover label.

- **StyleSetter** cns ELEMENTS: `{ label: 'Text size', type: 'range', var: '--cns-label-size',
  min: 9, max: 24, step: 1, unit: 'px', def: 12 }`. The label slug `text_size` ALREADY exists in
  all 15 locales (the universe-bar control introduced it in the 303-key batch) — zero locale work.
- **ConstellationSight2** drawHoverLabel: font 12px-fixed → `labelSize` (mount-read
  `--cns-label-size`, clamped 8–32, fallback 12 = no visual change when unset, the §D1/DF-16
  pattern); the label BOX scales proportionally (`k = labelSize/12` on the 10px pad + 18px height),
  so a 24px label never clips. Comment updated colors→vars.
- **Preview chip**: `font-size: var(--cns-label-size, 12px)` + em-padding — the mini-well chip
  grows/shrinks live as the slider drags, mirroring the real box's proportional scaling.

svelte-check 0 errors. Orientation v2.71 FU-3 clause updated (3 → 4 wired controls).
Mount-read caveat unchanged: size applies on next CNS open (the preview shows it live).

---

## "Text size" PASS — the Setter CNS category closes (4 controls, Boss-validated)

Eisa: **"Pass"** on the Text size control (17:09 binary). The FU-3 arc closes fully validated:
category + element + control names native ×15, the full-zone mini gravity-well preview (the new
Preview Rule), Background / Hover label background / Hover label text / Text size all live in the
preview, Keep-apply semantics confirmed.

**SO close-out in this commit:** the EN help topic (`Appearance and Themes.md`) gains the CNS
surface in the category list, the corrected centre-preview rule (Editor + Sky View + CNS — the old
text predated even Sky View's MIG-072 preview), and a "CNS — style the gravity well" paragraph
(four controls + apply timing); the User Manual §The Style Setter mirrors the same two corrections.
The 14 translated manuals + any translated mirrors of this help topic remain the standing batched
debt (logged at MIG-074; not expanded here). MoCh-1500 written (the FU-3 re-test saga block).

---

## State-of-standing snapshot (SO #5 — Eisa asked "What do we have next?")

**(a) Verified-shipped + protected (today):** MIG-075 CLOSED (milestone `milestone/mig-075-cns` + ZIP);
FU-1 (28 translated help files ×14 langs) · FU-2/FU-4 (null-type predicate + diversivity drop) ·
FU-3 corrected through 3 Boss rounds → the Setter CNS category at 4 controls, ALL Boss-validated;
the Style Setter Preview Rule written (CLAUDE.md + memory + code); EN help topic + User Manual
updated; binary 17:09 current. Tree clean, all pushed through `7c6e0097`.

**(b) At-risk / in-flight / uncommitted:** none. No open migration phase, no uncommitted work.

**(c) Known-broken (orientation §13):** BUG-013 open-editor cascade race (documented limitation);
**title-heading rename gap** (NoteEditor.svelte:179 rename does NOT call updateLinksOnRename — only
file-tree renames cascade; user-facing); sidebar active-item ~10s highlight lag (origin unresolved);
MIG-006 §4–§11 (cascade completeness) pending.

**(d) Pending, not started (reconciled against PJ v1.13 + orientation §8):**
- **PJ-060** `index_note` cache-hit short-circuit (search.rs:3004 returns early on matching mtime →
  write-time refresh skipped) — P1, "single most-leveraged open fix", mini-MIG. Queue #1.
- **MIG-063/064** — the remaining ~6 of 14 federation surfaces (Unlinked Mentions/Index/KH reads;
  Cataloger/Classifier/NSC writes with FK constraints). Reserved.
- **PJ-002** cid_cn collision scrub + **PJ-003** rename-collision popup (Override/Rename/Cancel) — P1 mini-MIGs.
- **PJ-008/PJ-009** Backlinks/Outgoing typed-link duplication + **PJ-010** Unlinked Mentions double-count — P2 panel fixes.
- **PJ-017/018/019** MIG-013 cleanup bundle (one MIG).
- **lenses.rs::apply_lens deletion** (Eisa decision 2026-05-09; Rule-8 hybrid per §12; cleanup MIG never opened).
- **PJ-041–043** CECE i18n + **PJ-044–050** MIG-022 polish backlog.
- **MIG-023** Warrant Research (reserved, Concept-Paper-first, multi-month) · **MIG-033** plugin sandbox
  (Architect-deferred) · **MIG-068** rank-aware sort (lands with CE columns) · MIG-002 §7–§10 (deprioritized).
- Dormant under Wings: Sight family (MIG-029/034/035/036/037, PJ-059, PJ-057.b) + Map (PJ-011).
- Standing docs debt: 14 translated User Manuals stale; Appearance-and-Themes (and other topics) translated
  mirrors; the MIG-014 P2/P3 six-item memory bundle still awaits PJ numbers.

**(e) Documentation drift:**
- **PJ-063 CLOSED BY EVIDENCE TODAY**: live DB GROUP BY link_type → supports 104,719 · derives-from 97,958 ·
  exemplifies 17,350 · part-of 5,693 · causes 5,185 · contradicts 1,794 · generalizes 1,019 · supersedes 4 ·
  inspires 1 (custom) · relates 301 + associative 38 (the null family, handled by is_null_type). The
  "globally 'relates'" bug died with MIG-067. Ledger still says Open → v1.14 flips it.
- **PJ-005 DONE** (MIG-007 closed 2026-06-09) and **PJ-064 effectively DONE** (fonts.ts installed-fonts
  shipped) — both still queued in v1.13's top-five → v1.14 flips both.
- Pending Jobs needs a **v1.14 bump** (the three flips + MIG-073/074/075 fold-in + today's state).
- Orientation §12's "no frontend test harness" row is itself stale (vitest since MIG-030; 58 sight-v6 +
  52 Base tests) → fix at next orientation bump.
- Memory hygiene done this turn: 5 stale memories deleted (relates-bug → resolved; user-definable link
  types → MIG-067; autoupdate-toggle placement → §12 v1.2 correction; 360.3D matrix doc → PJ-015 abandoned
  2026-05-18; Setter feature requests → both shipped).

**Snapshot correction (Eisa caught the omission):** two MIG-074-close "Eisa's later calls" were
missing from (d):
- **The §H pill-language question** — MIG-067's ratified rule (Eisa's own): *pills speak the NOTE's
  language, not the UI's*. Surfaced for re-ruling at MIG-074 Stage 2 (the "associative" pill on the
  English-titled "CSS with CSS" note rendered English in the Arabic UI — correct under §H; an
  Arabic-titled note shows ترابطي). Decision: keep note-language (zero code) vs switch to
  interface-language (affects every pill). Eisa's call; no work until ruled.
- **The archive-weight Guide-§10 drift** — the Guide (and the CLAUDE.md Living-Link principle "every
  link operation must be reversible") promises restore loses none of the 8 properties; the code zeroes
  raw weight on archive and restores at 1.0 (search.rs:5872/5894). traversal_count survives, so earned
  weight (1+ln(tc+1)) is exactly recomputable on restore → a small contract-restoring code fix is
  possible; alternative is documenting the reset in the Guide. Pending Eisa's ruling; recommended: fix
  the code (mini-MIG with tests).

---

## Eisa rulings (the two MIG-074 calls) + the work order approved

1. **§H pill-language: KEEP note-language** (the MIG-067 rule stands — pills speak the NOTE's
   language). Work: one help-file sentence documenting the behavior → quick-wins bundle.
2. **Archive-weight drift: FIX THE CODE — recompute earned weight from traversal_count on restore**
   (the Guide §10 / reversibility-principle contract restored; lossless since tc survives the
   round-trip). → quick-wins bundle, mini-MIG with round-trip tests.
3. **Slotting approved.** Order of work: **PJ-060 first**, then the remaining as presented
   (quick-wins bundle incl. the two rulings above + PJ-008/009/010 + PJ-003 + the title-heading
   rename gap → MIG-063/064 federation → housekeeping: PJ v1.14 · apply_lens deletion · translation debt).

## PJ-060 — OPEN (mini-MIG): index_note cache-hit short-circuit

Working on: **the `index_note` cache-hit short-circuit** — search.rs:3004 returns early when the
stored mtime matches, so a re-save with unchanged mtime never refreshes `note_meta` (the write-time-
derivation blocker flagged 2026-05-19; acceptance = unchanged-mtime re-save still refreshes
note_meta + every derived surface; no boot/typing regression on the 7,600-note universe).

**PJ-060 SHIPPED.** `index_note` gains a `force` flag: the bulk walk (`index_library_recursive`)
passes `false` — the mtime gate is byte-identical there, boot untouched; `reindex_single_note`
passes `true` — every one of its callers (the save IPC `constellation_search_reindex`, the rename
reindex libraries.rs:1020, the wikilink-cascade rewrite libraries.rs:4291 — whose own comment
records the stale-Outgoing-panel symptom this gate caused — and the Base cell edit bases.rs:748)
is a "this file just changed" context, and second-resolution mtime cannot prove it didn't.
The same-second blindspot (save → programmatic rewrite landing in the same second → note_meta and
every derived surface silently stale) is structurally closed. Idempotency verified in-code: the
note_meta UPSERT fires the FTS au-trigger; links delete+reinsert preserves earned properties;
the CTSE delta hook sees old==new → zero delta on a content-identical force.
**Perf:** boot path unchanged (gate intact); the forced path adds one bounded file read per
explicit save/rename/cascade-file — off the keystroke path (saves debounced 1500ms). 
**Tests:** new `tests_pj060_index_gate` ×3 (gate holds for the walk on a deterministic mtime-collision
fixture; force refreshes through the collision; fresh file indexes ungated). **Full lib suite 906/906**
(903 + 3). PJ ledger flip → the v1.14 bump (housekeeping, queued).

**Archive-weight fix SHIPPED (Eisa ruling: recompute from traversal_count).** One shared curve
`earned_link_weight(tc) = 1 + ln(1 + tc)` extracted (search.rs — traverse now calls it; the Guide §7
formula verbatim); `constellation_link_unarchive` → new testable core `unarchive_link_rows`:
two-step read-tc → compute-in-Rust → per-row UPDATE (the traverse pattern — no SQLite math-fn
dependency). A tc=20 link now restores at ≈4.04, not 1.0; tc=0 restores at exactly 1.0 (unchanged).
Archive-side zeroing untouched (scope per the ruling — restore-side only). Guide §10's promise
("restored without losing any of the 8 properties") is now TRUE as written — no Guide edit.
**Doc-truth flips:** the EN CCS help "weight restarts at 1.0" sentence + User Manual Tutorial-7
line rewritten to the new truth, AND the same sentence flipped in all 14 translations (gated
line-surgery, marker + outside-block gates, 14/14 OK — the FU-1 files were translated this morning
with the old behavior). **Tests:** `tests_archive_weight_roundtrip` ×3 (tc=20 round-trip ≈4.04 +
history preserved; tc=0 baseline; multi-typed-row pair coverage). **Full suite 909/909.**
*(Build note: one mid-edit E0433 — the helper initially landed between `#[tauri::command]` and
constellation_link_traverse, stealing the attribute; moved above the doc block, clean rebuild.)*

**§H ruling documented (keep note-language).** The Knowledge Formulation help topic (Tutorial 1)
and the User Manual typed-link tutorial now state the rule in user terms: the badge speaks the
NOTE's language, not the interface's (en note → *supports* even in ar UI; ar note → يدعم); switching
the app language never re-labels links inside notes. Adjacent factual drift fixed in the same touch:
the help's "Recognized types" line still listed 7 types + "anything else parsed as untyped relates"
(pre-MIG-022/067) → now the 8 built-ins + custom vocabulary types, unrecognized prefix = plain note
name, untyped = associative. KF topic is EN-only (no mirrors — logged at FU-1); Manual translations
are the standing stale-snapshot debt.

**PJ-008 + PJ-009 — CLOSED BY EVIDENCE (no code change).** SO#8 cross-check before tackling:
`getBacklinks` dedupes by source path, `getOutgoingLinks` by target name, type badges accumulate
into `linkTypes[]`, rendered as accumulated pills (`rowLinkTypes`) — exactly both PJs' acceptance
criteria. Shipped in **§89 `5c34b9ee` (2026-04-27)** — the SAME DAY the memories were filed; the
ledger entries were created from the memories and never flipped. → v1.14 flips both. The two stale
memories deleted (+ the long-superseded unlinked-mentions double-count one).

**PJ-010 SHIPPED — the frontmatter-alias bleed (the one genuine remaining gap).** scan_unlinked_mentions
already skips self and scans title-only via FTS (name+body_text — frontmatter-only matches structurally
impossible). The remaining case: a candidate whose body says the word AND who DECLARES the active title
among its own `aliases:` — that's the note referring to itself by its alias (MIG-004 already counts it
as an alias-aware backlink), not an unlinked mention. Fix: one indexed lookup on the write-time
`note_aliases` table (`alias_lower = normalize_alias_for_match(note_name) AND source='frontmatter'`,
Rule 8 — no file parsing) → those paths skipped in the verify loop. Suite 909/909.

---

## BUG-023 — the title-rename fix CORRUPTED DATA in Boss test; REVERTED same-hour

**Eisa's Stage-1 Test A verdict, verbatim: "This is NOT A BUG, this is a Disaster! FIX IT."**

Three screenshots: (1) after the title rename, the "Rename Probe v2" tab displayed PROBE POINTER's
properties (title: Probe Pointer, cid_cn ..._901F) with Probe Pointer's PRE-cascade body; (2) the
real Probe Pointer opened from the tree shows its body correctly cascaded to [[Rename Probe v2]];
(3) "Rename Probe v2" reopened from the tree shows Probe Pointer's title + cid and an EMPTY body.

**Forensic disk snapshots (read before any fix, both files mtime 18:53:27):**
- `Probe Pointer.md` — INTACT: own frontmatter (title: Probe Pointer, cid_cn 20260611T145216Z_NOTE_901F),
  body `[[Rename Probe v2]]` (the cascade worked).
- `Rename Probe v2.md` — CORRUPTED: **Probe Pointer's entire frontmatter** (its title, its cid_cn —
  the other note's identity) **and an empty body**. The renamed note's own identity + content destroyed.

**Action taken (same hour):** `git revert a086e1ee` — the `onTitleRename` delegation is OUT; title
renames return to the pre-existing bare-renameItem behavior (missing cascade, but never corrupting).
Orientation §13: the gap row re-opened with the BUG-023 record; a BUG-023 row added. Fresh safe
binary built for Eisa.

**Why the original record stays (history):** the reverted commit's session-log block described the
design as "verified safe" on two static arguments (tab.name still old at blur; canonical no-op
parity). Both arguments were about the INPUTS to the rename — neither examined the EDITOR LIFECYCLE
around the rename (the title-blur fires inside a live NotePane whose debounced save, flush, and
{#key} destroy/remount all interleave with the cascade's markCascading/flush/reload sequence — the
exact BUG-015 class the orchestration's own comments warn about, §3-redo (a)-(e)). The sidebar
rename never has this shape: its rename originates OUTSIDE the editor.

**Mechanism hypothesis (UNVERIFIED — recorded for the root-cause session, not as fact):** something
wrote `buildFullContent(<Probe Pointer's props>, "")` to the renamed path — the handleSave/handleFlush
shape with another tab's freshProps() and an empty doc. Candidates: the {#key} destroy flush of a
repurposed tab id; reloadTabsFromDisk interleaving with the in-flight title-blur save; freshProps()
resolving via tab.id against a tab whose content was swapped mid-cascade. NONE confirmed.

**Standing instruction (orientation §13 row):** no re-attempt without the full /migration treatment —
NotePane spec §2.6, the BUG-015 forensics (lab/forensics/), and the Rename Function Concept Paper are
required reading BEFORE the Architect phase. Working Agreement #4 was violated in spirit: the static
input analysis passed for a lifecycle problem. The lesson is the same as BUG-015's: every write-path
change that touches editor lifecycle gets the architectural-impact review, not an input check.

---

## STOP-EVERYTHING DIRECTIVE (Eisa, verbatim) → MIG-076 OPENS

**"We have to STOP EVERYTHING, and put our ALL EFFORTS into solving this BUG, once and forever.
It is an app KILLER. If a user faces this kind of issue, they will lose confidence in Constellation.
My target is 200% guarantee that you will solve it."**

**State at the stop (SO #5):** quick-wins bundle PAUSED mid-Boss-test (shipped+safe: PJ-060,
archive-weight, §H docs, PJ-010, PJ-008/009 evidence-closed; Test B alias-bleed untested; PJ-003
awaiting the Override ruling — all parked). BUG-023 vector reverted (`e99a2f56`); safe binary 19:10.
The corruption CLASS (write composed from divergent identities: tab-id props + pane text + captured
path) has now fired through 4 windows (BUG-012/§140 wab · BUG-015 §115 · F2 cascade-stomp · BUG-023)
— LL-014's three-strike law invoked: no fifth guard; root-cause migration MANDATORY.

## MIG-076 — Write Integrity / "Single Write Authority" — Phase 1 (Architect) OPEN

Target guarantee (the engineering form of Eisa's 200%): a cross-identity write CANNOT reach disk —
layered: (C) identity-verified write boundary (refuse+quarantine on mismatch — the backstop that
holds even against unknown future bugs), (A) single-snapshot write composition (kill the
three-identity assembly), (B) rename/cascade quiesces the editor (kill the race at its source),
plus a write journal (forensics) and lifecycle regression tests. Four parallel Architect agents
launched: W1 every-writer map, W2 identity/tab/WAB model, W3 NotePane lifecycle + BUG-023
interleaving pin, W4 proven-methods cross-check (WA #5).

## MIG-076 Architect + Plan WRITTEN — awaiting Plan approval

Four agents returned (W1 writers · W2 identity · W3 lifecycle+law · W4 prior art). Distilled into
`lab/reports/MIG-076-WRITE-INTEGRITY-ARCHITECT.md` (+ PLAN). Headlines: the class = writes assembled
from THREE independently-mutating identity sources (props-by-tab-id + pane text + captured path);
six write_note call sites; ten Rust writer sites; plain fs::write (no identity check / atomicity /
serialization); tab.id is a SLOT not a note identity (openNoteTab reuse); SIX direct tab.content=
mutation sites; the WAB identity check FAILS OPEN; two PropertyEditor instances (the sidebar one
survives tab switches). BUG-023: the fatal write is the ordinary flush shape with a poisoned store
input (proven); the store bridge = 5 ranked candidates (journal will pin at first dev recurrence).
W4 verdict: A+B+C+journal matches named industry ancestors (VS Code snapshot/sequentializer ·
Obsidian Vault.process · If-Match/409 CAS · WAL serialization · Dropbox conflict copies) with two
corrections ADOPTED: a per-path single-writer queue under everything, and a dual token (identity +
freshness, hash-on-mtime-ambiguity per racy-git). Design = locks L0–L4 + journal + quarantine +
refusal UX + cid backfill + lifecycle regression suite; invariants I1–I10; phases §A–§F with ★
Stages 1–3; ~5–6 sessions. STOP point per the Migration Rule: Plan presented to Eisa for approval.

## PLAN APPROVED (Eisa): "Approved. But not at the expense of speed."

The speed rider becomes a per-phase verification clause: all gate costs live on the ≥1.5s-debounced
save path (one uncontended lock + one fsync + one small CAS pre-read); ZERO keystroke-path work,
ZERO boot-path work; the §B3 backfill is background+throttled. Build cascade §A1 → §F under
Plan-Approval-Equals-Build-Approval; stops at ★ Stages 1–3.

## MIG-076 §A1 — write_gate.rs (L0 + L1 + journal) — building

Design refinements vs the Plan text (recorded per the genuine-surprise rule, both LESS invasive):
(1) journal = append-only JSONL at app-data dir (`write-journal.jsonl`, 5MB rotation) instead of a
search.db table — works for every writer with no connection threading, and survives exactly the
DB-unavailable moments when forensics matter most; (2) quarantine = Dropbox-style SIBLING files
next to the note (the proven in-place conflict-copy pattern) instead of a central folder.
ReplaceFileW (manual kernel32 extern, no new dependency) preserves the target's creation time —
plain temp+rename would have RESET fs creation time, which note_meta.created_at reads (a silent
§A1 regression caught at design time).

**§A1 SHIPPED — write_gate.rs (L0 per-path serialization + L1 atomic replace + JSONL journal).**
`gate_write(path, content, expect: Option<Expectation>, surface)` + `gate_create_exclusive`:
per-path lock registry (case/separator-normalized keys; poisoned-lock recovery so a panicked writer
can never wedge a file); same-dir temp → fsync → **ReplaceFileW** swap (manual kernel32 extern, zero
new deps; preserves target creation time — note_meta.created_at regression caught at design) with
5× backoff retry for the AV sharing-violation class; watcher_suppress marks BOTH temp+final;
append-only `write-journal.jsonl` (app-data dir via a new `.setup()` hook, 5MB rotation, fnv1a
content fingerprint, best-effort — never blocks a write). The §B Expectation seam is in place.
**Tests 5/5** (fresh write+journal · atomic replace + zero temp litter · 50-write two-thread
torture: never torn · create-exclusive refuses existing/creates fresh · timing print). **Full suite
914/914.** Speed rider: ~6.5ms/write debug incl. fsync, debounced-save path only; cascade worst case
N×6.5ms noted as the §A2 watch item (bulk-fsync lever available if Boss-test shows lag).

**§A2 SHIPPED — every Rust note-writer routed through the WriteGate.** The W1 map undercounted:
the exit audit found a second tier, ~24 note-content write sites total, ALL now gated:
write_note · create_note (create-exclusive: a resolver race now REFUSES instead of silently
overwriting) · rename_item (frontmatter write + gate_rename under both paths' locks) · move_item ·
the cascade walker (its own watcher_suppress::mark absorbed into the gate) · ensure_cid_cn ×2 ·
base_edit_cell · task_toggle · the classifier rewrites (sources_rewrite / content_type_rewrite /
bulk_accept — the programmatic-frontmatter class) · cid_dedupe (search.rs healer) · daily_note +
new_note (create-exclusive) · trash + rename_folder (gated renames) · canonicalize/de-canonicalize
+ import_adopt ×8 (canonical.rs bulk flows) · importers ×6 · welcome_note · system_note ·
mig003_step4. New gate primitive: `gate_rename` (both paths locked in sorted-key order — deadlock-
free; AV retry; journaled). **Exit audit CLEAN**: remaining bare fs::write/rename are all non-note
(.base defs, json configs, .canvas, lexicon binaries, markers, panic log, tests) or universe-
directory restructure ops — allow-list recorded in this entry. **Full suite 914/914.**

## Eisa ruling — name collisions: the conventional dialog (PJ-003 RULED, folded into MIG-076 §E)

**"I prefer the conventional way. The user will be notified and asked to change the name or
overwrite it."** Applies to BOTH flows (create-with-existing-name + rename-onto-existing-name —
the April PJ-003 ask covered both). Implementation semantics (recorded now, built in §E):
collision → modal "A note named X already exists" with **[Change name] (pre-filled suggestion) /
[Overwrite] / [Cancel]**; **Overwrite = move the existing note to .trash first** (recoverable —
the reversibility principle holds under conventional UX), then create/rename proceeds through the
gate. Sequencing: built in MIG-076 §E (same modal family + i18n batch as the refusal dialog; the
rename flow is §D-rebuilt first so it's built once). Until §E: create auto-suffixes, rename refuses
(both safe post-§A2). PJ-003 → resolved-by-ruling, lands with §E.

**§B1 SHIPPED — identity/freshness CAS in the gate (SHADOW mode).** `check_expectation` runs UNDER
the path lock (no TOCTOU): identity first (expected_cid vs the disk's frontmatter cid_cn — reusing
search.rs::extract_frontmatter_cid_cn, now pub(crate)), then freshness (mtime+size; the racy-git
hash escalation forgives metadata-only drift when base_hash matches the disk bytes). Verdicts:
would_refuse_identity (incl. file-gone — the §140 class) · would_refuse_stale · unverified_no_cid
(legacy population, closes via §B3) · ok. SHADOW: would-refuse verdicts journal loudly (journal now
carries expected_cid + found_cid) but the write proceeds — invariant I6; `WRITE_GATE_ENFORCE=false`
is the §F1 flip. `write_note` IPC gains an optional camelCase `expect` param (serde-default —
legacy callers unchanged). **Tests +6** (identity-mismatch shadow-writes · fresh-pass · stale-detect
(size catches same-second mtime) · hash-escalation · no-cid unverified · missing-file refusal).
**Full suite 920/920.** Next: §B2 frontend expectations + §B3 cid backfill → ★Stage 1.

**§B2 SHIPPED — SELF-ATTESTATION (a Plan improvement, logged as such) + §B3 VERIFIED-EXISTING.**
§B2 as planned would have plumbed expectations from the frontend — but the incoming content already
CARRIES its identity (frontmatter cid_cn). The gate now self-attests when no explicit Expectation
arrives: extract the incoming cid → compare against the disk's cid under the lock. Verdicts:
ok_self_attested · would_refuse_identity (the BUG-023 write would have produced EXACTLY this) ·
unverified_no_cid · created_by_write (write-to-missing-path with a cid — the soak decides which
surfaces move to create-exclusive) · ok_unchecked (cid-free content). EVERY writer is identity-
protected today — PropertyEditor, FocusPane, second screen, importers, the cascade — with ZERO
frontend changes and no second composition source (the content IS the snapshot). The explicit-
Expectation path (§B1) remains for freshness; full freshness attestation rides §C's snapshot work
where it belongs (revised sequencing logged: more coverage sooner, no double-build). Journal records
expected (explicit or self) + found cids on every line.
**§B3 — the cid backfill ALREADY EXISTS**: `mig003_backfill_cid_cn` runs from init_db at every boot
(Phase A injects via the §A2-gated ensure_cid_cn for files lacking a valid cid; Phase B dedup = the
gated cid_dedupe healer). Live-DB check: 7,663 notes, exactly 1 missing cid_cn — heals on next boot.
No new code; verified + recorded. **Tests +3 (suite 923/923).** → ★Stage 1.

## ★Stage-1 FINDING #1 — the rename-title stomp (journal-proven; pre-existing; FIXED same hour)

Eisa: created "New Note T1", sidebar-renamed to "New Note T1 v2", switched away, came back — the
title/properties showed v1. **The journal's FIRST real case nailed it in one read**:
`22:01:55.066 rename_title 209B` (title v2 + alias) → `22:01:55.066 rename_item → v2.md` →
**`22:01:55.415 write_note 176B ok_self_attested`** — the pre-rename content (byte-identical to the
21:58:41 save) stomping the fresh title 349ms later. Disk confirmed: title v1, alias line erased.
**Mechanism**: the §140 wab migration faithfully carried a PRE-rename buffer to the new path;
flushAllTabsInLibrary flushed it over rename_item's title write. Identity check passed (same cid —
correctly); this is the SAME-NOTE FRESHNESS class (§C's exact scope), pre-existing (all machinery
predates MIG-076 — the journal made it visible for the first time).
**Fix (sanctioned primitives only — the BUG-023 lesson)**: in store.renameItem after the IPC:
clearWriteAhead(old+new) — a pre-rename buffer is stale BY DEFINITION once the title is rewritten
(§140's old-path defense preserved by clearing both keys); then re-read the renamed file and refresh
matching tabs with a reloadVersion bump (the reloadTabsFromDisk / D6 recreate shape) so the
remounted pane + PropertyEditor can never resurrect v1. svelte-check 0 errors.

**★Stage-1 FINDING #2 — the empty-body resurrection (rename → switch → return); FIXED + journal upgraded.**
Eisa's re-test: title fix HELD (v2 + alias + body immediately after rename ✓ ✓ ✓), but returning to
the note after a switch showed an EMPTY body. Journal: rename_title 190B (fm v2+alias+BODY ✓) →
+11.4s write_note 159B = fm-v2-with-alias + NO body (disk confirmed body-less; same shape +0.5s on
his second probe). Three fixes:
1. **renameItem single-update**: my finding-#1 fix updated the tab twice (path, then content+bump) —
   the instantly-destroyed middle pane was a ZOMBIE able to flush its empty initial doc. Now ONE
   openTabs.update folds path+name+content+reloadVersion → exactly one remount, no zombie window.
2. **resolveNoteContent empty-body guard**: a same-cid wab whose BODY is empty while disk has one is
   never restored (a buffer preserves EDITS; empty preserves nothing) — disk wins, warn logged. The
   full same-cid freshness fix remains §C; this closes the destructive case.
3. **Journal origin granularity** (the gap this hunt exposed): write_note carries an optional
   `origin` — all 14 frontend writer sites labeled (editor_save / editor_flush / stage_promote /
   prop_save / flush_all / focus_pane / template_create / daily_template / template_insert /
   link_mention / bulk_tag / expression_forge / canvas_export) → the NEXT anomaly names its author
   in one journal line. svelte-check 0 errors; gate tests 14/14.

## ★Stage 1 — PASS (Eisa "Passed"; journal verdict 42 writes / 0 anomalies)

Re-test: rename → switch → return holds title+alias+BODY across the full cycle (3 screenshots).
Journal totals for the day: 42 writes — ok_self_attested 32 · created_exclusive 6 · renamed 4 ·
would_refuse_* **0** · unverified_no_cid **0**. The 2 newest entries already carry origin labels
(editor_save). Stage-1's two criteria met: behavior unchanged + clean journal — AND the stage
caught two pre-existing data-loss bugs (findings #1 #2), both fixed same-hour, both re-validated.
The soak continues passively on the 22:39 binary toward the §F1 enforcement flip. Next: §C
(single-snapshot composition + single store writer + WAB fail-closed).

## BOSS ORDER (05:40): "Stop it. Revert back prior to SC binary." — EXECUTED

git revert of 8c7c001a (wab TTL) + 4f061a3b (SC) — the source tree is back to the a269c2ed state:
the 22:39 binary Eisa validated ("Passed" on the rename cycle; Stage-1 PASS). SC (single-snapshot
composition) + the TTL fix are OUT; SA+SB (the WriteGate, all writers gated, shadow CAS,
self-attestation) and the Stage-1 findings #1+#2 fixes REMAIN (they were in the validated 22:39
binary). The SC display-layer bug remains UNDIAGNOSED (interrupted by the order — disk proven
intact throughout; zero journal writes from the SC binary). MIG-076 SC status -> REVERTED,
re-land only after the dev-console reproduction identifies the display failure. SD onward paused.

## Boss order (morning): "Re-structure §C, and we will take it step by step." — DONE

Forensic agent stopped per order (verdict not yet produced). §C restructured in the Plan from one
six-change monolith into FIVE separately-shipped steps, smallest risk first, one commit + one
binary + one Boss gate each: §C-1 WAB fail-closed → §C-2 single store writer (no composition
change) → §C-3 PE re-seed path key → §C-4 PE embedded routing → §C-5 the pane composer (the prime
suspect, LAST, pre-gated behind a sandbox repro on the real-shaped note copies before any Boss
build). The §C-monolith lesson recorded: bundling six lifecycle changes made the regression
un-isolatable. Sandbox (E:\ConstellationSandbox, 8 copied notes) + the §C worktree
(Constellation-wtSC) stay inert for §C-5's pre-gate. Code untouched — each step starts only on
Eisa's go.

## MIG-076 §C-1 SHIPPED (the restructured cascade, step 1 of 5) — WAB fail-closed restore

ONE change, store.ts resolveNoteContent only: the buffer restores ONLY when both cids present AND
equal AND not an empty-body resurrection; anything less proven → disk wins (cursor/scroll dropped
with the rejected snapshot). Zero editor-lifecycle change — read-path policy only; by construction
this step can only return MORE-authoritative content. svelte-check 0 errors / 313 baseline.
Boss gate next: open notes incl. large/federated ones + one rename cycle.

## §C-2 — Boss gate FAIL → REVERTED same hour. THE §C KILLER IS NAMED (journal-proven).

Symptom: every note's pane painted with the same content (tab-memory contamination). **Journal:
zero writes to any real note** — only the 3 test probes, each with its own typed content. Disk safe.
**ROOT CAUSE (both §C failures)**: handleFlush routed through updateTabContent → openTabs.update
fires INSIDE the {#key} pane TEARDOWN that the store itself drives → re-entrant render → panes
bind the wrong document. The old direct mutation's comment WAS the warning: "no store.update = no
cascade". §C-2's one-change diff + identical symptom = the proof the monolith lacked.
**Bonus finding (journal)**: FocusPane writes DISK PER KEYSTROKE (~170 writes/35s, +1B each) —
pre-existing perf bug, now on record (PJ at next ledger bump).
§C-2 reverted (`git revert e9788ce1`); §C-1 (passed) stays. The §C-2 re-design must keep flush
store-sync OUT of teardown (defer to a microtask/queued write, or flush-to-store only outside
{#key} transitions).

## §C-2 re-design — RESEARCH-FIRST (Boss order: "don't reinvent the wheel — check Obsidian, cross-check")

3 parallel agents: Obsidian (docs.obsidian.md, 1.7.2 deferred views, obsidian-api d.ts), VS Code
(source: textFileEditorModel.ts, fileService.ts, Working-Copies wiki), CM6 official + Emacs/Vim.
**Converged industry pattern:** document model (buffer) owns content, one per open file; tabs/views
are disposable VIEWPORTS holding zero content; saves read the MODEL, never the view; **no system
has a "flush content from view at teardown" step — the model is always-current at write time.**
VS Code: tabs = stateless pointers; save = model snapshot; per-resource write queue (≡ our
WriteGate L0 — validates §A). CM6 (Marijn, t/2946): keep one EditorState per "buffer", swap via
view.setState — string round-trip on switch is the documented anti-pattern (loses undo). Emacs/Vim:
buffer vs window, 40 years old. Obsidian = the middle case CLOSEST to ours: per-view string copy
(TextFileView.data) + 2s requestSave debounce + save-on-close — BUT content always travels WITH its
file identity (onUnloadFile(oldFile) receives the TFile; rename keeps identity-stable TFile object,
no path-string keying).
**Honest cross-check verdict:** §C-2 as shipped was at the WRONG LAYER — it decorated the teardown
flush (announce via store during teardown); mature editors don't make teardown carry content at all.
Correct §C-2 re-design = Rule-8 applied to the tab record: the debounced save path maintains
tab.content at WRITE TIME (pane life), teardown persists cursor/scroll only — nothing to announce.
Options to Boss: (2) Obsidian discipline now (minimal, staged §C continues) + (1) EditorState-per-
tab buffer pattern queued as the architectural follow-up (kills string round-trip + hand-rolled
history). Awaiting Boss ruling. No code written.

## MIG-076 §CB — Boss ruling: Option B, the Buffer Pattern (PRIORITY One; all else frozen)

Eisa: "I'll go with Option B. Proceed. This is PRIORITY One. We will NOT do anything until we
solve this issue for good." + side request: TWO ways to list open notes (tabs at top AND on the
side) — folded in as §CB-6 (buffer pattern makes both viewports of one metadata list).
Docs: ARCHITECT §7 addendum (design, D1-D6 decisions, territory census, risks R1-R5) + PLAN §C
rewritten as §CB-1..6 (each: own commit + own binary + own Boss gate; sandbox pre-gate on
lifecycle steps). Old §C-2..5 superseded. Memory: project_mig076_buffer_pattern_ruling.md.

## §CB-1 — the buffer registry (scaffolding, zero behavior change) — BUILT

- NEW src/lib/editor/noteBuffers.ts: NON-reactive Map<tabId, NoteBuffer{path, cid, props
  (snapshot-cloned), body: CM6 Text rope, paneState? (§CB-3)}>. Dependency LEAF (type-only
  store import; runtime dep = @codemirror/state only). Inert by construction at any lifecycle
  moment — a Map.set announces nothing (the §C-2 lesson, structurally enforced).
- 16 writer sites mirrored: store.ts openNoteTab (reuse+create), createEmptyTab,
  loadTabHistoryEntry (Alt-nav — census found it), reloadTabsFromDisk, renameItem (note:
  content+path; folder: path-only), moveItem, saveTabContent, restoreWorkspace (clearAll);
  NoteEditor handleSave/handleFlush/handlePromote; PropertyEditor debouncedSave + onDestroy;
  +layout second-screen onNoteSaved + FocusPane onchange; SecondScreenPage workspace-restore
  (clearAll). closeTab → deleteBuffer.
- DEV-only parityProbe at 4 paired sites (openNoteTab:reuse, handleFlush, handlePromote,
  PE:debouncedSave) — compares PARSED pieces (exact body + canonically-serialized props; raw
  string compare would false-alarm on legacy YAML quote/date normalization — caught in design).
- DEAD CODE deleted: store.ts updateTabContent (the §C-2-era writer, zero callers) + its dead
  +layout import.
- Tests: tests/mig-076/noteBuffers.test.ts — 12/12 green (round-trip incl. trailing-newline
  edges, snapshot-clone, cid extraction, paneState preservation, probe verdicts). vitest
  include + test:mig-076 script added. svelte-check: 0 errors (1456 files).

## §CB-1 — Boss gate PASSED (all 6 steps, 14:24-14:31 session, binary 13:49:53 standalone)

Steps 1-6 all Pass. Step 5 observation: rename took ~7s to update. Journal forensics: rename_title
14:30:46.904 + rename_item 14:30:46.905 — disk ops took 1 MILLISECOND; the 7s is the pre-existing
post-rename cascade title-scan (reads whole universe) + refresh, untouched by §CB-1 (which added
one parseFrontmatter to the path). §D quiesce-rename owns that UX (freeze overlay + progress).
NSIS bundling note: §CB-1 setup.exe never packaged (Boss launched the fresh standalone exe 9s
after link — file lock). Boss tested on standalone constellation.exe 13:49:53 = correct binary
(process start 13:50:02 verified). Proceeding to §CB-2 per Plan-Approval-=-Build-Approval.

## PCS + Orientation (Boss reminder 15:10 — SO #6 violation acknowledged)

Eisa had to ask "Don't forget to PCS + Orientation" — by the standing rule, that IS the violation:
the v-bump belonged inside the §CB-1 commit. Remediation: Orientation v2.73 written as a NEW file
(v2.72 preserved) — preamble covers §C/§C-2 failures + root cause, research verdict, Option B
PRIORITY One ruling + §CB-6 side-tabs, §CB-1 shipped + 6/6 gate, 7s-rename forensics, §CB-2
in-flight, explicit "no user-facing changes yet" (help/manual untouched by design this version).
MoCh-2026-06-12-1030.md written (block 10:30-15:15). Docs-only commit — the §CB-2 WIP Rust edit
(write_gate journal refactor) stays in the worktree for the §CB-2 commit.

## §CB-2 — Boss gate FAIL at step 4 (Focus mode) → REVERTED. Real notes safe (journal-proven).

Steps 1-3 PASS. Step 4: FocusPane opened showing ANOTHER note's body ("Hello Constellation
Base / [[1011]]"); typing in focus wrote {§CB2's correct props+cid + the WRONG body} to disk
(15:43:41, 564B→233B), later a THIRD note's body (15:47:01, 804B — link-test content).
**Journal: every write since the 15:09 binary touched ONLY §CB2 test.md** — all real notes
untouched. ZERO refused_compose entries: the §CB-2 identity guard (which-note/which-path)
passed honestly — the poison was the buffer's BODY VALUE, swapped at the focus enter/exit
transitions. Mechanism class = the §CB-3 disease (panes seeded from stale tab.content;
teardown hand-offs inject pane text via live-prop lookups); §CB-2's ensureBuffer re-seed +
focus seeding interacted with it instead of containing it.
DIAGNOSIS RULE (twice-burned, now standing): REPRODUCE in the dev sandbox with instrumentation
BEFORE any §CB-2 re-attempt — the §CB-3 sandbox pre-gate discipline now applies to EVERY
remaining §CB step, not just §CB-3.
Reverted `git revert 8ae4451f` (§CB-1 validated state restored; orientation v2.73 §CB-2 line
to be re-stamped FAILED+REVERTED in the same commit).

## BREAKTHROUGH — Boss repro'd the disease ON THE SAFE §CB-1 BINARY (16:33-16:38)

Recipe (reliable, ~30s): new note → type content → enter Focus → add line → exit → switch to
another note (file tree) → return → the returned view shows wrong content (display-level).
Journal forensics (16:02 binary):
- 16:36:05 focus_pane wrote **134B over the 282B note at FOCUS-ENTER** — body EMPTIED before
  any typing. The old code's focus path emitted a spurious write at mount. Disk recovered at
  16:37:22 (editor_save 317B — the pane still held the doc).
- Final 16:38:40 write = 607B = the note's OWN correct content (disk verified ✓).
- ZERO writes to any other note; مكتبة بودلي (step-2 focus test) intact 9414B ✓.
**VERDICT: the focus-transition bug is PRE-EXISTING** — present on the old validated code; the
§CB-2 step-4 failure was this same disease surfacing through the new compose path (its 15:43:41
233B write = the same focus-enter spurious-write shape). The §C-era display contaminations are
likely the same family. Diagnosis target NAMED: the FocusPane mount/seed/onchange ordering +
tab.content poisoning at focus transitions. Next: code-read FocusPane + focus branch wiring;
instrumented dev repro with the Boss recipe; THEN the §CB-2 redo designed against the named
injector. All disk files verified safe — display contamination only this round.

## BOSS ORDER (17:0x): "Revert back to safety. Use SME agents to audit what has been done."

§CB-1 reverted too (git revert bb3dc883; session-log conflict resolved keep-ours — history stays).
Code now = §C-1 validated state + §A/§B WriteGate+journal (Rust forensic net stays — it is what
proved disk safety all day). noteBuffers.ts + tests deleted; ARCHITECT §7 + PLAN §CB sections
reverted with the commit (history preserved in git + this log). Stop-On-Correction in force:
no fixes until the SME audit reports.

## SME AUDIT (3 agents, Boss-ordered) — verdicts recorded

AGENT 1 (forensics): reverts VERIFIED COMPLETE (HEAD vs pre-§CB-1 = docs-only delta; zero buffer
code remains). Disk-safety claims confirmed; refinements: إختبار الملاحظة final body is byte-
identical to التجربة الأولى body (paste vs disease — journal cannot discriminate); the
"pre-existing" proof ran on the §CB-1-state binary (inert mirrors) — pre-today is sound inference.
AGENT 2 (root cause, file:line mapped): S1 body-empty-at-focus-enter = FocusPane seeds from
tab.content which debounced saves NEVER update + Svelte 5 renders the NEW branch BEFORE old-branch
teardown flush back-fills it → focus opens empty → first keystroke writes frontmatter+empty (134B)
undebounced. S2 wrong-content-on-return = focus-enter flush plants a WAB snapshot never cleared on
the not-dirty path; resolveNoteContent gate checks IDENTITY not FRESHNESS → obsolete same-cid
snapshot legitimately restores over newer disk (one-shot, display-only). Landmine: tab-switch
WHILE in focus → FocusPane onDestroy composes new-note frontmatter + old-note body → REAL
cross-note disk write (likely the §CB-2 step-4 shape). Focus per-keystroke writes confirmed;
FocusPane saveTimer = dead code.
AGENT 3 (method): gate omitted Focus despite it being 1-of-2 surfaces + the morning write-storm
flag — negligent by the project constitution; §CB-2 = disguised multi-change (11 files +682/-421);
static checks 0-for-4 in this territory in 24h. VERDICT: Conditional GO for Buffer Pattern (the
disease PRE-DATES it; standing still is not safety) — NO-GO on any code until: (1) root cause
runtime-confirmed in sandbox via the Boss recipe, (2) a scripted runtime harness with view-vs-disk
parity gates EVERY step, (3) Reproduce-First + 8-surface gate checklist + one-writer-path-per-step
land in CLAUDE.md. Awaiting Boss ruling; Stop-On-Correction in force.

## BOSS RULING (17:2x): "Forget the clean slate. Solve this issue for good. NO more patching."

Interpretation: not clean-slate (keep the working editor/rendering), not symptom-patching, not live
half-step migrations. Deliver the STRUCTURAL end-state (single content ownership) built whole +
proven against a reproduction harness, landed as ONE validated swap behind a toggle. The §CB
direction (buffer = single ownership) was CONFIRMED-right by the audit; what failed was the live
incremental method. Three audit preconditions now adopted as top-principal CLAUDE.md rules
(Reproduce-First; Solve-the-Class-Not-the-Instance; the Editor-Surface Gate Checklist incl.
mandatory Focus). Path: rules locked (this commit) → harness + reproduce every recipe RED →
build single-ownership end-state → harness GREEN on all 8 surfaces → Boss test on real universe
LAST, behind a toggle. NotePane/editor live code untouched until the harness is green.

## §C rebuilt — step 2+3a: the single-ownership ENGINE built + proven in isolation (additive, zero live-code change)

Path step 2 (harness + reproduce) and the engine half of step 3, landed as PURELY ADDITIVE new
files — no live component or store.ts touched, so the running app is byte-identical to the
reverted §C-1 safe state.
- NEW src/lib/editor/noteModel.ts — SINGLE CONTENT OWNERSHIP, the one authority per open note
  (path, cid, props, body: CM6 Text, version, savedVersion). API: openModel/getModel/setBody/
  setProps/setPath/compose(identity-bound)/markSaved/isDirty/adoptDisk(freshness)/close/clearAll.
  Non-reactive module Map (the §C-2 lesson). Imports parse/build from store for now (leaf-extract
  deferred to integration to avoid a cycle); nothing live imports it yet.
- NEW tests/mig-076/noteModel.test.ts — the ACCEPTANCE HARNESS: I1 always-current (symptom 1),
  I2 freshness incl. echo-ignore + dirty-wins (symptom 2), I3 identity-bound compose REFUSES
  path mismatch (the in-focus-switch cross-note write), I4 single deterministic composition,
  I5 model independence, dirty tracking, lifecycle. 14 tests.
- NEW tests/mig-076/currentBugRepro.test.ts — CHARACTERIZATION against real store primitives:
  the WAB entry has no freshness field (symptom-2 root); buildFullContent(propsA, bodyB) is an
  unguarded frankenstein (the landmine). 2 tests. Deleted when the cure removes those structures.
- 16/16 vitest green; svelte-check 0/1457. vitest.config + both files registered.
WHAT REMAINS (the part that failed as §CB, now properly gated): integration — wire the live
components to read/write the model, retire tab.content + the WAB; preceded by the runtime
view-vs-disk harness (Editor-Surface Gate, 8 surfaces incl. Focus round-trip + in-focus switch),
then the Boss test behind a toggle. The engine being green is necessary, NOT sufficient — the
integration gets the runtime proof before it touches the live editor.

## §C rebuilt — step 2 COMPLETE: the runtime recipe harness is green (still additive, zero live-code change)

- NEW src/lib/editor/noteSession.ts — the BEHAVIOR layer over noteModel: the thin testable glue
  the components will call (open/editBody/editProps/bodyForView/save(identity-bound, injected
  DiskWriter)/repath/externalChange/isDirty/close/closeAll). The structural answer to "the §CB
  bugs hid in .svelte lifecycle" — glue pulled into plain TS where a harness can drive it.
- NEW tests/mig-076/runtimeHarness.test.ts — plays every named failure as a FULL RECIPE through
  the real noteSession path against an in-memory fake disk, asserting screen===disk + no
  cross-note contamination after each transition: A focus-on-fresh-note (symptom 1), B switch-
  away/return (symptom 2), C in-focus tab switch (the landmine — own-content save + REFUSED
  cross-path), D rename-with-link (BUG-023 shape), E second-screen freshness, F restart, +
  global "disk never holds a foreign cid". 7 scenarios.
- Suite: 23/23 mig-076 (14 acceptance + 7 runtime + 2 characterization); svelte-check 0/1459.
- SCOPE HONESTY (recorded): the harness proves content-flow LOGIC + GLUE headlessly; it does NOT
  mount Svelte/CM6, so a pure-template wiring slip at integration (seeding a view from tab.content
  instead of bodyForView) is the one residual the Boss test closes. Single ownership removes the
  stale alternative, shrinking that residual.
NEXT (step 3 integration — the part that failed as §CB, now thin wiring over a proven controller):
make NoteEditor/FocusPane/PropertyEditor/+layout/openNoteTab call noteSession + seed from
bodyForView; retire tab.content + the WAB; behind a toggle. Then Boss test on the real universe.

## §C integration — Commit 1: ADDITIVE FOUNDATION (model goes live in the real app; not yet used)

Applying the §CB lesson (don't bundle): the integration is split into the additive foundation
(this commit) and the flag-gated behavioral swap (next). After this commit the app behaves
IDENTICALLY to the §C-1 safe state — the model is maintained but nothing reads it for seed/save
(the safe §CB-1 shape that passed its gate).
- NEW src/lib/editor/ownershipFlag.ts — SINGLE_OWNERSHIP toggle (default true; flip+rebuild =
  instant rollback, no half-state). Unused until the swap.
- noteModel.setBody — keystroke-hot-path safe: ref-check only (CM6 yields a new doc per change),
  no O(N) eq; accepts the Text rope.
- noteSession — ensure(id, path, content): create-if-absent / re-seed on path change; existing
  same-path model untouched (live edits win). editBody accepts string | Text.
- NotePane — new onDocChange?(doc: Text) fired O(1) in the updateListener (passes the rope, no
  toString()).
- NoteEditor — $effect ensures the model per tab (covers ALL hosts) + onDocChange→editBody live
  push. Writes only the non-reactive Map → cannot re-enter a {#key} teardown (§C-2 lesson).
- svelte-check 0/1460; mig-076 harness 23/23. No binary (nothing visible changed) — the binary
  ships with the swap.
NEXT (Commit 2, flag-gated): NoteEditor seed=bodyForView + save=compose; FocusPane seed +
onchange→editBody + focusNoteId capture (kills the in-focus-switch cross-write); PropertyEditor
save via the model; store lifecycle (close/repath/reload). Then build + Boss test.

## §C integration — Commit 2: THE SWAP (flag-gated; model is now the seed + save source)

SINGLE_OWNERSHIP=true flips every editor surface to the model; flag off+rebuild = instant
rollback (legacy path preserved in the else branch of each chokepoint — no half-state).
- NoteEditor: NotePane value = seedBody(tab.id, tab.path, fallback); handleSave/handleFlush =
  editBody + compose(identity-bound) + markSaved (REFUSE on path mismatch); handlePromote =
  editProps + compose. freshProps/freshBody only feed the legacy else-branch now.
- FocusPane (+layout): value = seedBody(focusSessionId, focusSessionPath, fallback); onchange =
  editBody + compose for the CAPTURED focus-session identity (set at focus entry, held through
  teardown) — never live $activeTab → the in-focus-switch cross-note write is closed structurally.
- store.saveTabContent (PropertyEditor's path, both instances): editProps(auto-dated) + compose
  from the model (ignores PropertyEditor's possibly-stale body; embed uses the model body).
- store lifecycle: closeTab→close; renameItem→open(fresh)/repath; moveItem→repath;
  reloadTabsFromDisk→open(fresh, cascade-authored). Model identity follows every path change so
  compose never refuses a legitimate save.
- second-screen onNoteSaved→externalChange (freshness-gated adopt).
- svelte-check 0/1460; mig-076 harness 23/23. Import cycle store↔noteModel is eval-safe (hoisted
  fn declarations; no module-init use).
RESIDUAL (Boss test closes it): the template wiring the headless harness can't mount. Building the
binary now for the full 8-surface gate.

## §C — new-note-while-open identity leak: DIAGNOSED + FIXED (flag still off; harness-proven)

Boss test (flag on) surfaced it; rolled back to safe (flag off, commit fa27cc40) per Boss order.
ROOT CAUSE (confirmed by journal + code, not guessed): saveTabContent (PropertyEditor's path)
lacked the filePath===tab.path guard that handleSave/handleFlush have. New note reuses the active
tab; the torn-down PropertyEditor for the PREVIOUS note fires a last saveTabContent → editProps
poisoned the ALREADY-repurposed model with the old note's props (cid). compose then refused the
old path but the model was left poisoned; the next save to the NEW path composed the stale cid →
new note's file got the open note's identity (§C Eisa No. 2.md ← §C test's cid 9E76). Rust shadow
CAS flagged would_refuse_identity; real notes untouched.
FIX (structural, both directions now identity-guarded):
- setBody/setProps take expectPath; reject the write when model.path !== expectPath (the write-in
  mirror of compose's read-out guard). A stale caller can no longer poison a repurposed model.
- saveTabContent passes expectPath=filePath (THE fix); handleSave/handleFlush/handlePromote +
  FocusPane + onDocChange pass their path too (defense in depth).
- openNoteTab (reuse + new) + loadTabHistoryEntry now openNoteModel() SYNCHRONOUSLY — the model
  is driven by the explicit note-open event, not the async ensure $effect (closes the timing gap).
- Harness Recipe G reproduces the exact poison (stale prop-save + stale body-flush from the
  previous note) → both rejected; B's file keeps only B's identity. 25/25 mig-076; svelte-check 0.
Fix is committed behind SINGLE_OWNERSHIP=false (inert). Next: flip on + rebuild → corrected re-test.

## §C SINGLE CONTENT OWNERSHIP — BOSS GATE PASSED (5/5, corrected binary 20:56, re-test 21:xx)

Steps 1-5 all PASS, INCLUDING Step 2 (new-note-while-open — the exact flow that leaked §C test's
identity into §C Eisa No. 2 last round). Journal verdict for the re-test session: 90 writes, ZERO
anomalies (no would_refuse_identity, all ok_self_attested), only the 4 §D test notes written, real
notes untouched. Step 4 rename ~7s: journal proves rename_title + rename_item 1 MILLISECOND apart
(21:22:14.081/.082) — the 7s is the pre-existing whole-universe cascade title-scan + refresh,
unchanged by §C; owned by §D quiesce-rename (freeze overlay + progress).

**The content-integrity bug class (BUG-012/015/019/023, F2, the §C/§CB regressions) is now
STRUCTURALLY CLOSED and Boss-validated on the running app:** single content ownership — one model
per open note; identity-guarded BOTH directions (compose refuses read-out mismatch; setBody/
setProps refuse write-in mismatch); model driven synchronously by the explicit open event; focus
bound to a captured session identity. Harness 25/25 incl. the new-note poison reproduction +
view-vs-disk parity across every recipe. SINGLE_OWNERSHIP=true shipped (4cc77c2a).

REMAINING MIG-076 scope: §D quiesce-rename (+ the 7s rename UX) + title-rename re-land · §E
refusal/recovery/collision UX + i18n ×15 · §F enforcement flip (WRITE_GATE_ENFORCE=true after
soak) + permanent regression suite + 3-agent audit + /simplify + close.

## SESSION CLOSE (2026-06-12 night) — PCS + Orientation + Handover

PCS done: Orientation **v2.74** (new file; v2.73 preserved) — §C single content ownership SHIPPED +
Boss-validated, final architecture (noteModel/noteSession/ownershipFlag), the new-note leak find+fix,
remaining §D/E/F. MoCh-2026-06-12-1515.md written (the §C build arc). Handover: lab/reports/
HANDOVER-2026-06-12.md (state-of-standing + §D next + fresh-session prompt). Help files + User Manual:
NO change this version — §C is an invisible reliability fix (no user-facing strings); §E's recovery/
collision surfaces will carry help updates. Memory project_mig076_buffer_pattern_ruling.md updated to
VALIDATED. Milestone tagged + ZIP-backed. HEAD after this commit pushed to origin/main.
NEXT SESSION: MIG-076 §D (quiesce-rename + title-rename re-land) — see the handover.

---

## SESSION 2026-06-12 (continued) — MIG-076 §D (quiesced rename + title-rename re-land)

**Function in hand:** the quiesced rename protocol (a freeze overlay for the journal-proven ~7s wikilink cascade) + re-landing title-rename on the §C single-ownership foundation. Plan: `lab/reports/MIG-076-WRITE-INTEGRITY-PLAN.md` §D. Two Boss design forks ruled at session start: **Focus title stays read-only**; **freeze scope = all affected-library panes**.

**§D0 — Reproduce-First (tests only).** Added harness **Recipe H — title-rename quiesce** to `tests/mig-076/runtimeHarness.test.ts` (A links B, both open → B title-renamed → repath + cascade re-seed → assert view===disk, both identities intact, the stale pre-rename flush REFUSED `path_mismatch`, no ghost resurrected at the old path; + a freeze-invariant sub-test). Matching RED characterization in `currentBugRepro.test.ts` (the unguarded composition layer is identity-blind — the hole §C fills). **Proved teeth:** temporarily disabling the §C compose guard makes Recipe H fail at the exact `stale.ok===false` assertion (alongside Recipe C + noteModel I3); restored via `git checkout`. Harness **28/28**.

**§D1 — quiesce freeze + the flush-seam fix.**
- *Architectural finding* (2-agent impact review + first-hand code verification): under §C the write-ahead buffer is fed ONLY on tab teardown (`NoteEditor.handleFlush:284`), NEVER on keystroke (keystrokes go to the model, `NoteEditor:409`). So `flushAllTabsInLibrary`'s WAB read MISSED an actively-edited tab — a freshly-typed `[[link]]` to a just-renamed note, renamed within the sub-1.5s pre-autosave window, would be left broken by the cascade. **Latent in shipped §C**, not a §D regression. Boss ruled: fix in §D1.
- *Fix:* `flushAllTabsInLibrary` (store.ts) now composes the pre-cascade flush from the MODEL under `SINGLE_OWNERSHIP` (via `saveNoteSession`; identity-guarded skip on path-mismatch; `isNoteDirty` gate); legacy WAB path preserved in the `else`. One caller (the rename path).
- *Freeze:* new reactive `cascadeFreeze` writable store (set of frozen `tab.path` strings), set in `handleRenameComplete` around the cascade window, cleared in the `finally`. New self-contained `CascadeFreezeOverlay.svelte` (read-only, input-blocking `pointer-events:auto`, localized "Updating links…" + spinner) mounted on the 3 panes a main-window sidebar rename can reach (active `.flank-center`, split `.split-pane-wrap`, index `.index-note-pane` — each made `position:relative`). Focus intentionally excluded (a rename can't originate while the sidebar-less, read-only-title Focus surface is active; §C identity-binding covers Focus content — gate item 4). i18n `cascade.updating` ×15 (native).

**§D2 — title-rename re-land.** `NoteEditor` regains the `onTitleRename` prop; `handleTitleChange` delegates the whole rename to the host's `handleRenameComplete` (the SAME quiesced path the sidebar uses) when `onTitleRename && SINGLE_OWNERSHIP`, else the legacy bare-renameItem fallback (rollback-safe). Wired at the 3 main-window NoteEditor mounts; second screen keeps the direct fallback (display-not-domain). Closes the orientation-§13 gap the reverted `a086e1ee` opened — now safe because §C's identity-bound compose + §D1's freeze remove the BUG-023 vector (proven by Recipe H).

**Verification:** harness **28/28**; svelte-check **0 errors** (1461 files). Backend untouched (no Rust change — `rename_item`/cascade already route the WriteGate per-path locks; the 7s is the per-library full-read cascade; freeze is frontend-only). Diff: 20 files, +212/−13.

**PENDING — ★ Stage-2 Boss gate (runtime, not yet run).** Static green is NOT runtime verification for lifecycle changes (Reproduce-First). Boss test = the exact BUG-023 recipe (two probe notes, rename via the TITLE, inspect both files) + the freeze visible + the 8-surface regression sweep + the write-journal read (zero anomalies). Release binary building (Rust unchanged → frontend rebuild + bundle). **Not committed** — lands after the Boss verdict.

### §D Boss-test arc + the freeze/perf saga (2026-06-13) — SHIPPED

**Stage-2 Part A (title-rename probe) — PASS, integrity proven.** Boss created `MIG076 D Target` ← `MIG076 D Source` (`[[link]]`), renamed Target by TITLE → v2. Disk + write-journal verdict (Claude-side, per Working Agreement #1): Target v2 kept its OWN cid + body + old title preserved as alias; Source's link healed to `[[…v2]]` + kept its own cid; **zero journal anomalies, no cross-identity write** — the exact BUG-023 wound is gone through the title-rename path. Notes found in the `Eisa Cognitive Knowledge` library (a registered library ≠ the universe root) — the journal's path field located them.

**The instant-freeze regression + bisection.** Boss noted the overlay arrived ~3s late (Part A). The "fix" (instant-freeze v1: raise the freeze BEFORE `renameItem`, keyed by stable tab.id) **regressed** in B2/B3: when a recently-edited tab was open, the link-heal cascade + tree refresh silently skipped (journal: NO cascade write). Could not pin the mechanism by reading (`cascadeFreeze` has only the overlay as subscriber). Per **Reproduce-First + Stop-On-Correction**, secured first: reverted the instant-freeze to the Part-A-passing version (Boss-chosen) → re-test confirmed the cascade heals again. **Bisection conclusion: the pre-rename freeze raise was the culprit** (exact DOM/timing mechanism unconfirmed; not chased on a hunch).

**instant-freeze v2 (the safe redesign).** Raise the freeze **after `renameItem`, before the slow tree refresh**, path-keyed, in a `try/finally` that always clears it — staying in the proven-safe post-rename reactive context. Carried a one-shot timing trace (Boss-readable alert) to validate. Result: cascade heals (`rewrote=1`), and the trace **overturned the handover's assumption** — the slowness is NOT the wikilink scan (fast) nor the tree refresh (37ms); it's **`renameItem` itself, ~10.6s** when a note was edited just before.

**The 10.6s root cause — embed lock contention (FIXED).** Profiled `renameItem`: a NORMAL rename is **~400ms** (`invoke=379ms`); the 10.6s only happens **after an edit**. Root cause (read + DB-verified): `constellation_embed_notes` (the on-save semantic embed, 8,786 vectors active) held `search_state.db.lock()` across the WHOLE loop **including `run_embedding` (multi-second model inference)** — so the next rename's `reindex_single_note` (`db.lock()`) blocked ~10s behind it. (Ruled out: the DB path-cascade is indexed — `idx_link_source`/`idx_link_target_path`, query plans use them; the CTSE term hook is O(note-tokens), tiny.) **Fix** (`embeddings.rs`): scope the DB lock to only the existence-check + the INSERT; **inference now runs lock-free**. Re-test: rename-after-edit **10.6s → 1.77s** (6×). Residual ~1.4s (CPU during concurrent embed inference, not a lock) logged as a follow-up PJ; Boss chose to ship.

**As-shipped §D:** §D0 harness (Recipe H + RED), §D1 freeze v2 + flush-from-model seam fix, §D2 title-rename re-land, + the embed-lock perf win. Harness 28/28; svelte-check 0; cargo check clean. **All temp trace/profile instrumentation stripped before commit.** Boss directive: **PCS + Orientation.**

---

### §E-1 — the universe-wide name-collision dialog (2026-06-13) — SHIPPED + Boss-validated

**Function in hand:** the name-collision dialog (PJ-003 §E1b) — the conventional *Change name / Overwrite / Cancel* modal shown when a create-with-typed-name or a rename would land on a title that already exists in the universe. Scope ruled by Eisa this session: **§E = collision dialog (§E-1) + diagnostics line (§E-2)**; the refusal/quarantine/freshness recovery UX folds into **§F**. Two design forks ruled: the collision check is **universe-wide** (every library + every federated cUniverse, not per-path); create-collision interrupts **only on a typed name** (Quick Capture keeps its auto-suffix).

**The build (carried from the prior context, validated this session).** `src/lib/components/CollisionDialog.svelte` (NEW; modeled on CanonicalChoiceDialog, z-index 100000). Gates in `+layout.svelte`: `createNoteWithTemplate` (create) and `handleRenameComplete` (rename) each call `resolveWikilinkCrossLibrary(lib.path, name)` BEFORE the write; any hit (rename: excluding self via `normPathLC`) opens the dialog. Authoritative by construction — it reuses the SAME resolver the `[[wikilinks]]` use, fed by `resolve_universe_libraries` (federated → cUniverse-inclusive). Overwrite → `moveToTrash(existing)` then re-attempt with `force=true` (skips the gate). Quick Capture untouched (separate Rust `quick_capture` path); folders untouched (`!isDir`). i18n `collision.*` ×15 (native).

**Boss validation — Stage 1 + Stage 2, disk + content proven (Working Agreement #1).** All notes landed in the `Eisa Test` library (`E:\Cognitive Knowledge\Eisa Test` — a registered library whose path is OUTSIDE the universe root). Universe/library census this session also cleared a two-app-data-registry red herring: `world.uconstellation.app/universes.json` lists only `كون عيسى`, but the live session is demonstrably in `Eisa Cognitive Knowledge` (18 libraries, `children:[]` — no cUniverse linked to test federation live, so the cross-library test stands in for the same code path).
- **Stage 1 (same-library create + 3 buttons) — PASS.** Disk-verified: `Collision Test 1.md` (Change name), fresh `Collision Test.md` (Overwrite), displaced original safe in `.trash`. Cancel created nothing. (First Overwrite's displaced note had gone to per-library `.trash`, not System trash — the "System trash" the Boss saw was the *setting* value; `move_to_trash` hardcodes per-library `.trash`.)
- **Stage 2 (cross-library create + rename collision + Overwrite-to-trash) — PASS.** The dialog correctly showed "Already in: Eisa Test" while creating in a DIFFERENT library (the universe-wide proof). Rename-Overwrite: read confirmed the renamed note kept **its own** cid (`…F7B1`, the former "Collision Test 1") + `title: "Collision Test"` + **old name "Collision Test 1" preserved as an alias** (old links still resolve → collision detection is alias-aware); the displaced original (`…09F3`) intact in `.trash`. **No identity leak, no corruption.**

**The trash de-collide fix (Boss-approved this session).** Stage-2 surfaced a real edge: `move_to_trash` reused the exact filename, so trashing a second same-named note **atomically replaced — and silently lost — the earlier trashed copy** (Stage-1's `.trash\Collision Test.md` ts 14:32:33Z was overwritten by the Stage-2 one ts 14:36:35Z). Eisa: *"Follow Obsidian de-collides by suffixing."* Fix (`libraries.rs::move_to_trash`): on a `.trash` name clash, suffix the stem with ` {n}` (`Foo 1`, `Foo 2`) — matching the create-collision naming; never clobber; errors rather than clobber if 1..=9999 are all taken. Blast radius verified: `move_to_trash` is reached ONLY via the two collision-Overwrite handlers (normal delete uses `delete_item`); de-collide is strictly safer for any caller.

**Two trash gaps LOGGED as follow-ups (pre-existing, not §E-1).** (a) `delete_item` IGNORES the `trashDestination` setting and always hard-deletes (`// For now, always permanent delete`) — the System-trash / .trash / Permanent dropdown is a no-op for normal deletes; (b) collision-Overwrite always routes to per-library `.trash` regardless of that setting (safe-by-default, inconsistent). Both belong with §F's recovery UX. **Trash is per-library by design** (Boss-confirmed): a Library is a portable, self-contained vault (Obsidian convention; Eisa's libraries live under `E:\Cognitive Knowledge\…`, not the universe root — a universe-root trash would scatter deletions across folder trees + break portability). A unified universe-wide trash VIEW (storage stays per-library) offered as a future option.

**Verification:** release build green (2m40s, warnings only); harness 28/28; svelte-check 0. Docs (SO): Help (`Notes Management → When a name already exists`) + User Manual (EN) + orientation **v2.76** (preamble + §8 row). **PCS this commit.** Next: §E-2 (the write-integrity diagnostics line).

---

### §E-2 — the write-integrity diagnostics line (2026-06-13) — SHIPPED + Boss-validated; §E CLOSED

**Function in hand:** the Settings → Security & Privacy "Write integrity" readout (the plan's §E1 diagnostics line; the refusal/quarantine/freshness UX stays folded into §F per the §E scope ruling).

**Build.** Rust IPC `read_write_journal_stats` (`write_gate.rs`) streams `write-journal.jsonl` + its one rotated `.old`, returning `{writes, anomalies, would_refuse_identity/stale, last_anomaly_ts, refused_exists, created, enforce, exists, rotated, dir}`. Anomalies = the SHADOW would-refuse verdicts (the §F1-flip precondition). Registered in `lib.rs`. Frontend: `readWriteJournalStats` + `openPath` wrappers (store.ts); SettingsModal Security-section line (writes / anomalies ✓0·⚠N + most-recent-anomaly date / monitoring mode / Open journal folder), lazy-loaded. i18n `writeIntegrity.*` ×15 (native; scripted insert + JSON-validated).

**The diagnostic earned its keep on first run.** Claude-side disk read of the live journal: 640 writes / **2 anomalies** — both `editor_save` 2026-06-12 19:57/20:02, stale `…9E76` cid onto `§C Eisa No. 2.md` + `Target note.md` = the §C new-note identity leak the journal caught 06-12, since fixed; zero new since. Added `last_anomaly_ts` so the perpetual-but-historical ⚠2 shows its date (reads as stale, not live).

**The build-process miss + lesson.** First §E-2 binary had the IPC but NO Settings line — Eisa's screenshot showed the section unchanged. Root cause: `cargo build --release` (direct) recompiles Rust only + re-embeds the existing `../build`; the frontend bundle is regenerated ONLY by `npm run build` (tauri.conf `beforeBuildCommand`). Stage-0 (binary > source) passed but the binary held the §E-1-era frontend. **Fix + standing rule:** frontend change → `npm run build` THEN `cargo build --release`; verify by grepping `build/` for the new string (binary-mtime alone is insufficient). Memory `feedback_frontend_build_before_cargo` written + MEMORY.md indexed.

**Boss iteration (3 rounds).** R1 (proper rebuild): line appears, **Stage 1 PASS** (640 / ⚠2 / June 12, matches disk). R2 follow-ups: "Most recent" → **"Most recent anomaly"** (it's the last anomaly, not the last note); panel now **auto-re-reads on every Security-section entry** (the manual Refresh looked dead because the Settings modal blocks editing → no new writes while open → nothing new to fetch; NOT a reactivity bug — journal grew 640→647 with Eisa's test note, proving logging + the reassignment path). R3: **Refresh button dropped** (Boss call — auto-refresh makes it redundant), dead `refresh` i18n key removed ×15.

**Verification:** every rebuild proper (`npm run build` + `cargo build --release`); bundle grep-confirmed each round; svelte-check 0; cargo green. Docs (SO): User Manual §19 (Security and Privacy → Write integrity, EN) + orientation **v2.77** (preamble + §8 row). **§E CLOSED** (collision dialog + diagnostics both shipped + Boss-validated). **PCS this commit.** Remaining MIG-076: only **§F** (enforcement flip + refusal/quarantine/freshness recovery UX + regression suite + 3-agent audit + close).

---

### Delete-path — the "Deleted files" setting now governs deletion (2026-06-14) — SHIPPED + Boss-validated

**Function in hand:** the Delete path. Closes the data-loss gap where `delete_item` ALWAYS hard-deleted regardless of Settings → Universe & Libraries → "Deleted files" (`// For now, always permanent delete`). Default is 'system', so users expecting the Recycle Bin — or who picked ".trash folder" for safety — were silently losing data permanently. Eisa picked this (over §F / session-restore) as the next step.

**Build.** New Rust `delete_path(app, path, mode, trash_root)` (`libraries.rs`): "permanent" → remove; "trash" → move into `<trash_root>/.trash` (de-colliding like `move_to_trash`; cross-volume-safe via rename→copy+remove fallback + recursive dir copy); "system" → OS Recycle Bin (the **`trash` crate**, new dep). All modes `reindex_delete_note`. `.trash` already excluded from walks+index (`file_kinds.rs` EXCLUDED_DIRS / `embeds.rs` IGNORED_DIRS) → trashed notes vanish from tree+search. Frontend `deleteWithSetting(path)` (store.ts, ONE shared helper) reads the setting, resolves the `.trash` root (library = longest path-prefix among `libraryStats`; universe = the `is_universe_notes` library's path), closes tabs + clears aux state. Replaced the hard-coded delete at BOTH real sites: `handleDeleteConfirm` (+layout) and `handleBatchDelete` (NotebookNavigator). Workspace-base delete out of scope. The List-mode batch confirm now states the destination instead of falsely claiming "cannot be undone."

**Boss extension — .trash scope.** ".trash folder" gets a sub-choice: **within the note's library** OR **at the universe root**. New setting `trashFolderScope: 'library' | 'universe'` + a conditional control under the dropdown; the mover takes the chosen root (cross-volume-safe for the universe case — Eisa's libraries live on a different folder tree than the universe root). i18n `settings.files.{trashFolderScope,…,scopeLibrary,scopeUniverse}` ×15.

**Boss ruling — drop "Permanently delete."** *"Who would choose Permanently deleted? I wouldn't."* Cross-check: Obsidian offers it; macOS/Windows file managers have NO global always-permanent mode (permanent is an explicit one-off) — for Constellation's protect-your-knowledge ethos, the global option is a footgun. **Dropped** from the dropdown (now System trash / .trash folder, both recoverable); type → `'system' | 'local'`; `deleteWithSetting` normalizes legacy 'permanent' → 'system'; `applyParsedSettings` migrates a saved 'permanent' → 'system' on load. The engine KEEPS `delete_path`'s `permanent` mode for a FUTURE *explicit* "Delete permanently" action (never a silent global mode).

**Boss validation — Stage 1 + Stage 2, all disk-confirmed (Working Agreement #1).** S1: scope control appears only for ".trash folder"; ".trash"(library) → `E:\Cognitive Knowledge\Eisa Test\.trash\Delete Probe Local.md`; "System trash" → Windows Recycle Bin (orig location recorded). S2: ".trash"(universe) → `…\Eisa Cognitive Knowledge\.trash\Delete Probe Universe.md` (cross-tree move); "Permanently delete"(pre-drop) → gone everywhere; folder delete → whole folder to .trash; List batch → both notes to .trash. Verified via Claude-side disk + Recycle-Bin (Shell.Application) reads.

**Deferred follow-ups (NOT in this commit):** (1) right-click batch delete in List mode; (2) the native `confirm()` box → Constellation's styled dialog; (3) comprehensive app-wide right-click (an initiative — inventory + plan, not piecemeal); + the **unified Trash view** (Boss-chosen over renaming per-library `.trash` — same-name `.trash` is the standard, disambiguated by location; renaming would break the exact-name exclusion + Obsidian-compat); + an **explicit "Delete permanently"** action.

**Verification:** svelte-check 0; cargo check + release build green (trash crate compiled); bundle grep-confirmed each rebuild; every build proper. Docs (SO): User Manual (delete behavior) + help (Notes Management) + orientation **v2.78**. **PCS this commit.**
