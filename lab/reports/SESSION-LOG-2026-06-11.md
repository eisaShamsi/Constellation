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
