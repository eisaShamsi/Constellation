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

**Commit:** MIG-075 Architect + this log + orientation v2.67 (same commit per LL-031/SO #6).
