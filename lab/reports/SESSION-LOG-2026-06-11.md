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
