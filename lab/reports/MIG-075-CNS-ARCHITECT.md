# MIG-075 — The CNS Modernization (Architect / Phase 1)

**Status: ARCHITECT (Phase 1) — produced 2026-06-11, AWAITING EISA'S RATIFICATION. No Plan, no code.**
**Function in hand: the CNS (Constellation Nervous System) — `ConstellationSight2.svelte`, its `SightPanel`
insight sidebar, and the tension subsystem (`tension.rs`) — carrying the TWO Boss-approved items from
`MIG-074-CCS-ARCHITECT.md` §3-a (detect_tensions Rule-8 re-sourcing + the CNS-panel boundary cleanup) plus
the four in/out candidates (c)–(f) from `Handover-2026-06-10-CNS.md` §4.**

**Number allocation (verified, not assumed):** orientation v2.66 §8 tops at **MIG-074** (closed 2026-06-10);
a repo-wide search finds **zero** references to MIG-075 or higher (no files, no Pending-Jobs v1.13
reservations). **This migration is MIG-075.**

**FACT** = current code/docs, cited (every line read first-hand this session). **PROPOSAL** = design argued
here, for ratification.

---

## 0. SO #8 cross-check — the brief vs the canon (done before this doc)

Checked the handover §4/§5 brief against the orientation v2.66 **body** (§3, §4.2 row 4, §8, §12–§14), the
CCS Concept Paper v1.1 (§4 + §12, the ratified boundary), MIG-074-CCS-ARCHITECT (§0, §2.7, §3-a),
SESSION-LOG-2026-06-09 (§HEALTH-TAB FIX), SESSION-LOG-2026-06-10 (the MIG-074 arc incl. the Stage-1
cross-check record), Pending Jobs v1.13, and the code + the live DB. The brief holds. Reconciliations and
**new territory findings** (none blocking, several scope-relevant):

1. **The brief's unverified fact is now VERIFIED — tags ARE queryable from the DB.** `note_meta.tags_json`
   (schema search.rs:1916, default `'[]'`) is written at index time by `parse_frontmatter`
   (search.rs:3283–3338): frontmatter `tags:` lists (inline + multi-line) AND inline body `#hashtags`
   (`(?:^|\s)#([\w\p{L}\p{N}_/-]+)` — all scripts), lowercased, deduped. Live DB (probed this session):
   **7,592 of 7,661 notes carry tags · 19,523 distinct tags · 36,176 (note, tag) pairs**, queryable via
   `json_each(note_meta.tags_json)`. The tag-cluster input for structural gaps exists in the DB.
2. **The brief's consumer list is stale by one.** `detect_tensions` has exactly **ONE** caller today:
   `loadTensionReport` (+layout:3314–3327, fired by the health-tab activation `$effect` :3330–3334,
   cached per library). The "Sky View legend action" consumer named in MIG-074 §2.7 died in the
   `b2a23d4e` re-home (2026-06-09/10) — repo-wide grep finds no other `detect_tensions` call site.
3. **Component precision:** the two shed blocks ("Link Health BY TYPE" + "BY CONFIDENCE") live in
   **`SightPanel.svelte`** (290 lines, §2.3) — a separate component ConstellationSight2 imports (:17) and
   mounts behind its analytics toggle (`panelVisible`, :1099, mount :1274–1290). The handover's "in
   ConstellationSight2.svelte" is right at the surface level, one file off at the code level.
4. **NEW Rule-8 finding — the CNS open path itself is an fs walk.** `toggleLens` step 1
   (+layout:3852) invokes `constellation_sight_centrality` (sight.rs:55) — a **sync** `#[tauri::command]`
   that calls `scan_library_links` (libraries.rs:2003–2012, full recursive `.md` re-read + regex) **for
   every library in `$libraries`** (the federated list, store.ts:1402–1412). Cached frontend-side
   (`lensDataStale`) until the graph changes (`skyVersion`), then walks again.
5. **NEW federation fact — CNS analytics are silently own-universe-only.** `scan_library_links` validates
   against `load_all_libraries` (own libraries.json); cUniverse library paths fail and sight.rs:67
   swallows the error (`.unwrap_or_default()`). So the gravity well **displays** the full federation
   (8,751 nodes via MIG-061) while centrality / bridges / contradiction pairs are computed from
   own-universe links only. Same scope story for the health tab: `validate_path_in_any_library`
   (libraries.rs:162–181) rejects cUniverse paths → `detect_tensions` on a cUniverse note's library errors
   → the tab shows `unavailable`. Both pre-date this MIG (MIG-063-family gaps).
6. **NEW dead surfaces:** `constellation_sight_tag_edges` (sight.rs:320, another fs walk) has **zero**
   frontend callers (registered lib.rs:420 — dead IPC). `ConstellationSight.svelte` (the v1 component)
   has **zero** importers (dead component; still consumes `lens.title` at :675).
7. **NEW data-dead insight:** SightPanel's "stagnating" tab queries `status='dormant'` (search.rs:5424) —
   **0 rows** in the live DB since the decay-write fix (`aa9941ee`; nothing writes 'dormant' anymore;
   MIG-074's Q3 derived dormancy lives in the cache layer, not in `note_links.status`). The tab returns
   empty forever. CCS's Cooling Inquiries register is its live replacement.
8. **most_connected is duplicated today** (feeds ruling f): KH renders the cached `fmt_most_connected`
   card (MIG-073, "keep" ruling) AND SightPanel's Knowledge Insights carries a `most_connected` tab
   (live `constellation_formulation_analysis` GROUP BY, search.rs:5473).
9. **Pending Jobs v1.13 cross-check:** no reserved CNS MIG; no conflicting open PJ. The v1.11 line
   "Cleanup MIG (TBD) retires Sight v2…" is OBE — Sight-v2 became CNS (kept, live, retitled).
   `lenses.rs::apply_lens` deletion (ruled 2026-05-09) is a **different module** (Multi-Lens, CE Phase 9)
   and stays its own queued item, not MIG-075's.
10. **Orientation §3.5 micro-drift (noted for the close-out):** the Rust module table still lists
    `lens.rs` — renamed `sight.rs` by MIG-009.

---

## 1. What MIG-075 does (from the two Boss-approved items)

Modernize the CNS family on both of its axes:

- **Data honesty (Rule 8):** `detect_tensions` stops re-reading every `.md` per run and reads the same
  facts from the indexed DB (and, if ratified, the CNS open path's own walk goes the same way).
- **Boundary honesty (CCS §4):** the CNS panel sheds its circulatory blocks to CCS (which renders them
  better, registry-true, ×15) and gains a **"Circulation → CCS"** deep-link; the raw `lens.link*` key
  rendering and non-registry custom-type labels vanish with the shed. The four handover rulings (c)–(f)
  are settled explicitly, plus the rulings this Architect's territory pass surfaced (g)–(j).

---

## 2. Territory map (FACT)

### 2.1 CNS identity & wiring

CNS = `ConstellationSight2.svelte` (1,363 lines) — the Sight-v2 gravity well retitled "Constellation
Nervous System (CNS)" — live under `SIGHT_V2_ENABLED = true` (src/lib/sight/engine.ts:131). Dock button
+layout:4888 (tooltip `lens.title`, :4895); overlay mount +layout:5749–5781 (props: `skyNodes`, `skyLinks`,
centrality/communities/gaps/health/bridges/profiles/contradictions, `focusNoteId` for the MIG-060 gesture);
header title `lens.title` (Sight2:1087); header counts `{simNodes.length} nodes · {simLinks.length} links`
(:1088). `toggleLens` (+layout:3823–3950) runs the 9-step pipeline: Rust centrality → JS Louvain
(`detectClusters`) → structural gaps → **universe health** → stratum-weighted centrality → top-10 bridges →
community profiles → bridge suggestions → contradictions (returned by the centrality IPC). Results cached
in memory (`lensDataStale`) and invalidated when `skyVersion` increments.

### 2.2 The CNS open cost (the walk family)

- `constellation_sight_centrality` (sight.rs:39–160): **sync command**; per library calls
  `scan_library_links` → full recursive read of every `.md` (libraries.rs:2014–2060); builds a petgraph,
  Brandes' betweenness (sampled >500 nodes); also collects `contradicts` pairs (sight.rs:73–76). First CNS
  open after boot/graph-change re-reads the corpus on the UI thread. cUniverse paths silently contribute
  nothing (§0.5).
- `loadLinkEnrichment` (Sight2:274–290): calls `constellation_link_stats` and keeps only
  `stats.sample_links` — the **top-10-by-weight rows** (search.rs:5139 LIMIT 10). Those ≤10 links of
  234,062 get weight/confidence/annotation on the canvas (confidence-thickness draw, Sight2:660; hover
  annotation, :867; :1066). At scale the advertised "thin = hypothesis / thick = established" legend
  distinction is invisible — effectively decorative data from one more live IPC per open.
- `constellation_sight_tag_edges` (sight.rs:320+): fs walk; **zero callers** (dead).

### 2.3 SightPanel (the insight sidebar — 290 lines, mounted at Sight2:1274–1290)

Four sections; on mount fires **three live IPCs** on the shared DB mutex:

| # | Section | Data | Register per CCS §4 | Notes |
|---|---|---|---|---|
| 1 | Overview | props: nodeCount/linkCount/orphanCount (computed from sim data), `health` (§2.6), library breakdown | topology | stays |
| 2 | **Link Health** — "BY TYPE" + "BY CONFIDENCE" bars + dormant chip | `constellation_link_stats` (live full-table GROUP BYs, search.rs:5114–5131 — the pre-MIG-073 KH freeze shape) + `constellation_link_dormant` (live julianday scan, **LIMIT 200** → the chip's count silently caps at 200; search.rs:5286–5306) | **circulatory** | **the Boss-approved shed** — duplicates CCS Acts-of-Inquiry + Conviction-&-Doubt |
| 3 | Top Bridges | `bridges` prop (frontend centrality) | topology | stays |
| 4 | Knowledge Insights — 6 tabs | `constellation_formulation_analysis` live LIMIT-50 queries (search.rs:5388–5500): strongest_evidence (weight×confidence) · weak_foundations (hypothesis ∧ weight>2) · tensions (contradicts list) · stagnating (**data-dead**, §0.7) · most_connected (inbound GROUP BY) · knowledge_gaps (out-but-few-in) | **mixed** — 4 circulatory/tension, 2 topology | **not named in the approved shed** → ruling **g** |

Defects the shed removes (verified at both layers): the BY-TYPE labels render
`$t('lens.link<Camelized>')` (SightPanel:156) and en.json carries exactly **7** `lens.link*` keys
(supports/contradicts/causes/exemplifies/generalizes/derivesFrom/partOf) — so `associative`, `supersedes`,
legacy `relates`, and every custom type (e.g. `inspires`) render the **raw key verbatim** ($t returns the
key on miss; confirmed against the Stage-1 screenshot record in SESSION-LOG-2026-06-10). SightPanel also
carries its own hardcoded `LINK_TYPE_COLORS` + `CONFIDENCE_COLORS` maps (:49–57) — registry-blind.

### 2.4 `detect_tensions` (tension.rs:54–238 + scan at :240–291)

Sync command; per run: recursive fs walk of the given library, regex wikilink + tag extraction per file,
then four detections. **Output contract (frozen):** `TensionReport { contradictions, orphans,
structural_gaps, single_points, total_linked_notes, total_notes, active }`; `active=false` under 50 linked
notes (earned complexity). Detections: contradictions deduped per (source, target) with ×N occurrence
suffix + stable sort (the Boss-approved 2026-06-10 dedupe); orphans (0 inbound ∧ >20 words; severity by
word count); structural gaps (tags with ≥3 members ∧ cross-link density <20%; top 20); single points
(≥5 inbound sources ∧ ≤1 derives-from).

**Input → DB mapping** (all in the active universe's search.db, scoped `WHERE library_name = ?`):

| fs-walk input | DB source | Parity |
|---|---|---|
| note name (frontmatter title) + path | `note_meta.name` / `.path` | exact (the indexer stores the display name) |
| word_count (frontmatter-stripped) | `note_meta.word_count` | exact (same writer) |
| outgoing (target_lower, link_type) | `note_links.target_name` / `.link_type` (MIG-067-correct both syntaxes; untyped = `associative`) | see deltas below |
| tags | `note_meta.tags_json` via `json_each` | superset (delta 1) |
| name-resolution (target → in-library note by lowercase title; alias-blind) | JOIN `LOWER(note_meta.name) = note_links.target_name` ∧ same library | exact, incl. equal alias-blindness |

**Honest output deltas of a DB re-source** (each user-visible, none silent):

1. **Tag coverage widens.** tension.rs's own regex covers Latin+Arabic-script inline tags only and never
   reads frontmatter `tags:`; `tags_json` covers all scripts + frontmatter lists. Gap clusters will see
   more (and more correct) tags — an improvement, but cluster sets will change.
2. **The ×N occurrence suffix goes.** `extract_typed_links` (search.rs:3688–3704) dedupes
   `(type, target)` per source note at write time — the DB holds ONE row per (source, type, target), so
   "contradicts X ×40" (40 occurrences inside one bibliography note) becomes one pair row with no
   multiplier. The list of *pairs* is identical; the per-pair repeat count is not reproducible from the DB.
3. **Archived links can finally be excluded.** The fs walk can't see `status` (the wikilink stays in the
   .md after archival); the DB read filters `status='active'` — archival starts being honored (today:
   archived rows = 0, so day-one output is identical).
4. **Frontmatter-embedded wikilinks** are read by both paths (both regex the full content) — no delta.

Live counts feeding the detections (probed): contradicts 1,794 · derives-from 97,958 · all rows 234,062
active (0 archived / 0 dormant-status).

### 2.5 TensionPanel (the frozen consumer)

`TensionPanel.svelte` — renders the four sections (orphans displayed capped at 30 + "+N more"), severity
dots, `tensionPanel.*` ×15 (incl. the `analyzing`/`unavailable` states from b2a23d4e). Single mount:
+layout:6616–6620, behind the right-sidebar health tab. Loader: `loadTensionReport` — lazy on tab
activation, scoped to the active note's library, cached per library path, retry on failure. **Nothing in
MIG-075 changes this component or its contract** (unless a ruling explicitly says otherwise).

### 2.6 The Universe-Health score — ruling (e) ANSWERED

`computeUniverseHealth` (clusterEngine.ts:332–375):
`score = 25·norm(modularity/0.6) + 25·(1−dominance) + 25·norm(entropy) + 15·norm(edges/nodes ÷ 4) +
10·(1−gapPenalty)`, clamped 0–100. Inputs: Louvain modularity, community sizes, Shannon entropy of the
partition, the boot-graph edge/node counts, structural-gap count. **Purely topological — zero circulatory
inputs (no weight, decay, traversal, confidence).** The "91" does not mix registers; no boundary change
is needed. (Its companion metrics shown in Overview — connectivity = links/note, entropy % — same story.)

### 2.7 The two link counts — ruling (d) mechanics

- **CNS header (233,538):** `simLinks.length` — sky-graph edges where BOTH endpoints resolve to existing
  notes and source ≠ target (buildSimData, Sight2:245–270, over the alias-resolved boot sky snapshot).
  The graph layer.
- **CCS / KH (234,062):** `note_links` rows (probed: all active) — every recorded link record incl.
  unresolved targets (index_note writes a row per `(type, target)` with `target_cid_cn = NULL` when the
  title doesn't resolve; search.rs:4117–4141). The record layer.
- Delta today ≈ 524 = unresolved-target rows + self-links. Both numbers are true; they answer different
  questions ("resolved connections on the graph" vs "recorded acts of linking").

### 2.8 `lens.title` — ruling (c) VERIFIED

The value is **byte-identical English** — `Constellation Nervous System (CNS)` — in **all 15 locales**
(probed this session), while sibling keys (`lens.universeHealth`, `lens.linkSupports`, …) are translated.
Live consumers: the dock tooltip (+layout:4895) + the CNS header (Sight2:1087); plus the dead v1 component
(§0.6). Precedents for the fix: the ratified native names from the v2.03 rename (الجهاز العصبي ·
دستگاه عصبی · 神经系统 · מערכת העצבים …) and the MIG-074 CCS native-title pattern (الجهاز الدوري للكوكبة ·
星座循环系统 · …). The full-localization TOP PRINCIPAL (2026-05-18) overrules the old §A.15
brand-names-stay-English convention.

### 2.9 Registry-color drift (context for rulings b/j)

Three hardcoded typed-link color sites in the CNS family, all predating MIG-067's one-registry invariant:
SightPanel's maps (:49–57, dies with the shed), **the canvas edge draw** (Sight2:61–63 `LINK_TYPE_COLORS`,
used at :644/:654 — user recolors via Style Setter → Links never reach CNS edges; custom types fall to
gray), and **the legend** (Sight2:1232–1255 — six hardcoded rows/hexes; no part-of, supersedes, or custom
types). Sky View solved exactly this in MIG-072 (registry-fed palette).

### 2.10 i18n inventory affected

- `sightPanel.*` = 18 keys ×15 (verified en + ar; same set). The Link-Health shed kills `linkHealth`,
  `byType`, `byConfidence`, `dormantLinks`; ruling g decides `insights` + the 6 tab keys + `noResults`;
  `overview`, `totalNodes`, `totalLinks`, `orphans`, `linksPerNote`, `topBridges` stay.
- `lens.link*` 7 keys stay regardless (the legend consumes them); `lens.title` changes value ×14 (en
  stays); ruling d adds one new key ×15.
- All locale edits ride the established machine-gated merge pattern (parse + leaf-delta + endings;
  en+ar CRLF, others LF).

### 2.11 Adjacent, NOT included (each its own item)

`lenses.rs::apply_lens` deletion (2026-05-09 ruling, separate module) · the §H pill-language question
(Eisa's parked call) · the archive-weight Guide-§10 drift (Eisa's later call) · any KH content change
beyond ruling f's explicit option · CCS code (consumes the existing `constellation:open-ccs` event as-is) ·
federation of CNS/tension reads (MIG-063 family) · the 1.7 GB search.db investigation · PJ-060.

---

## 3. Explicit in/out rulings (the brief's a–f + the territory's g–j)

| # | Item | Ruling (PROPOSAL) | Why |
|---|---|---|---|
| a | `detect_tensions` fs-walk modernization | **IN** — D1 Option A (same algorithm, DB-sourced inputs, async, active-only) | The brief's item 1; inputs verified queryable (§2.4); contract frozen; one lazy consumer |
| b | The CNS-panel boundary cleanup | **IN** — Boss-approved 2026-06-10; D2 mechanics (shed Section 2 incl. the dormant chip → "Circulation → CCS" deep-link, gated `enabledFeatures.ccs`; drop the ≤10-link enrichment call with it) | CCS §4 boundary; the raw-key + non-registry defects vanish; removes 2–3 live IPCs from the CNS open |
| c | `lens.title` localization | **IN** — native titles ×14 (en keeps "Constellation Nervous System (CNS)"), following the v2.03 ratified equivalents + the CCS compound pattern; Eisa confirms the Arabic (recommend **الجهاز العصبي للكوكبة**, the sibling of الجهاز الدوري للكوكبة) | Full-localization TOP PRINCIPAL; verified ×15 English today |
| d | Link-count caption | **IN** — one muted CNS-side caption under the header count (new `lens.linkCountNote` ×15, e.g. "resolved connections on the graph"); CCS/KH unchanged | Both numbers true, different layers (§2.7); one line kills the recurring "why do they differ" question |
| e | Universe-Health score inputs | **VERIFIED — no change.** The composite is topology-pure (§2.6) | The question the brief ordered read-not-assumed is answered; nothing to move |
| f | `most_connected`'s home | **Eisa's call — D4**: (i) status quo (KH card + CNS insights tab, duplicated); (ii) ★ CNS-canonical: keep the CNS insights tab, retire KH's most-connected card (explicitly reverses MIG-073's "keep" — only Eisa can); (iii) keep both, re-point the CNS tab to the cached `fmt_most_connected` key (consistency, no retirement) | It is topology (CCS §12 assigns it CNS-ward); today it's duplicated (§0.8). (ii) is boundary-pure; (iii) is the no-reversal compromise |
| g | Knowledge-Insights section fate *(territory-surfaced)* | **Eisa's call — D3**: (i) keep all 6; (ii) ★ shed the four circulatory/tension tabs (strongest_evidence → CCS Load-Bearing · weak_foundations → KH card exists · stagnating → data-dead, CCS Cooling · tensions → TensionPanel owns the pair list), keep the two topology tabs (most_connected, knowledge_gaps); (iii) shed the whole section | The approved shed names only Section 2; an honest map can't leave Section 4's register mix unruled |
| h | `constellation_sight_centrality` fs-walk *(territory-surfaced)* | **IN recommended** — D5 Option A (re-source from `note_links`, async; identical return shape; exact scope parity since cUniverse scans already silently fail) | It is THE CNS open cost (§2.2) and the same Rule-8 family as (a); modernizing tension but leaving CNS's own walk would be half a modernization. *(If Eisa prefers: OUT, queued — the rest of the MIG doesn't depend on it)* |
| i | Dead-surface cleanup *(territory-surfaced)* | **IN recommended** — delete `constellation_sight_tag_edges` (+ registration) and `ConstellationSight.svelte` (v1) | Zero callers / zero importers (§0.6); same subsystem; keeps the retire honest. `lenses.rs` stays OUT (§2.11) |
| j | Edge/legend registry alignment *(territory-surfaced)* | **IN recommended** — canvas edge colors + legend read the Link-Type Registry (`linkTypeColor()`), legend rows become registry-driven (gains supersedes/part-of/custom; user recolors finally reach CNS) | The MIG-067/072 single-source invariant; today CNS is the last typed-color island (§2.9). Visual change only where a user recolored or uses custom types |
| — | OUT (all) | §2.11 list | Each named in the handover/orientation as its own item |

---

## 4. Design options

### D1 — How `detect_tensions` gets its facts (the brief's core)

| Option | Mechanics | Speed (per run, 7.7k-note universe) | Effort | Risk |
|---|---|---|---|---|
| **A — same algorithm, DB-sourced inputs** ★ | Replace `scan_notes_recursive` with three indexed reads (notes: name/path/word_count; links: source/target/type, `status='active'`; tags: `json_each(tags_json)`) scoped `library_name = ?`, build the same in-memory `NoteInfo` map, run the **existing** detection code unchanged; command goes `(async)` | ms-scale reads vs re-reading every file; detections unchanged (in-memory) | **small-medium** — one Rust commit + unit tests pinning the §2.4 deltas | **low** — contract byte-compatible; deltas 1–3 are the only output changes, each documented + ratified up-front |
| B — full-SQL detections | Rewrite the four detections as SQL aggregates | similar | medium-high (the gap-density pairwise join is awkward in SQL; severity/order/truncate parity re-derived) | medium — re-implementation drift for zero user gain |
| C — write-time persisted tension report | Per-library `tension_*` cache keys + recompute hooks on note writes | instant read | high (per-library keys, staleness, hook frequency — tension inputs change on every note save) | medium — heavy machinery for ONE lazy, per-library-cached consumer; the WA#5 industry norm for this shape (indexed-store aggregate on demand) is Option A; matches the ratified CCS I1 carve-out (bounded on-demand reads). Available later if tension trends are ever wanted |

**Recommendation: A.** Sub-choice inside A (Eisa): keep counting archived links (pure parity) or filter
`status='active'` ★ (honest improvement, zero visible change today — 0 archived rows).

### D2 — The shed mechanics (b)

Remove SightPanel Section 2 (the two bars + the dormant chip — one section, one i18n family) and the
`onMount` calls to `constellation_link_stats` + `constellation_link_dormant`; in its place one
`sp-section`-style row — **"Circulation → CCS"** — dispatching the existing `constellation:open-ccs`
event (the MIG-074 hub listener opens CCS with the full dock-exclusion list), hidden when
`enabledFeatures.ccs === false` (mirror of the listener's own gate). Drop `loadLinkEnrichment` from
ConstellationSight2 (the ≤10-link cosmetic enrichment, §2.2) and the two confidence-thickness legend rows
that advertise it — **unless** Eisa wants the thin/thick vocabulary kept for the 10 sample edges (not
recommended: at 234k links it reads as a broken promise). Dead keys drop ×15 in the same commit
(machine-gated). Predecessor → Replacement table for every removed element lands in the session log
before the Build edit (the §2.3 table is its draft).

### D3 — Knowledge Insights (g): see ruling table. If (ii)★: the section header stays, four tabs go, their
keys drop ×15; `tensions` discovery is covered by TensionPanel (right sidebar) + CCS's Acts register;
nothing un-homed. If (iii): also re-home most_connected per f, and knowledge_gaps' only surface dies —
note `lensGaps` (the canvas + Blind-Spots rendering) still covers structural gaps visually.

### D4 — most_connected (f): see ruling table. (iii) detail: the CNS tab reads the snapshot key via the
existing KH snapshot IPC instead of the live GROUP BY — one consistent number set, no KH change.

### D5 — The CNS open path (h)

| Option | Mechanics | Speed | Effort | Risk |
|---|---|---|---|---|
| **A — DB re-source** ★ | `constellation_sight_centrality` reads `(source_name, target_name, link_type)` from `note_links` per own library (`status='active'`), keeps petgraph/Brandes/contradiction collection identical; `#[tauri::command(async)]`; return shape unchanged | first-open seconds → ms-scale read + the (unchanged) Brandes compute; UI thread freed | **small-medium** — one Rust commit + a parity test | **low** — same names, same lowercasing, same unresolved-target behavior as the scan; scope = exact parity (own-universe; §0.5) |
| B — leave as-is, queue | nothing now | unchanged | zero | the MIG's headline surface keeps its biggest walk |
| C — async-only band-aid | just `(async)` | UI unblocked; walk remains | trivial | Rule-8 violation persists; rejected as the *only* action |

### D6 — lens.title (c) + caption (d): mechanics are §2.8/§2.7; both are locale-file + one-line-markup
changes riding the merge-script pattern; en value unchanged; Eisa confirms the Arabic title term.

---

## 5. Invariants (must not break)

- **I1** The `TensionReport` contract and `TensionPanel` rendering stay byte-compatible (the panel is not
  edited unless a ratified ruling names it).
- **I2** No new write path anywhere; CNS/tension stay strictly read-only; **no observer effect** — nothing
  in CNS ever fires `constellation_link_traverse` (the MIG-074 I2b rule holds here too).
- **I3** CCS and KH code/content untouched except per ruling f's explicit option; coordination reuses the
  existing `constellation:open-ccs` event — no second listener, no new event names.
- **I4** Complement-not-overlap (CCS §4/§11): after the shed, CNS carries zero circulatory rendering;
  topology stays CNS-side; the contradiction pair list stays TensionPanel's.
- **I5** Registry single-source where touched: any color/label/order this MIG touches reads the Link-Type
  Registry (no new hardcoded maps; ruling j removes the existing ones).
- **I6** No silent feature loss: every removed surface has a named destination (the §2.3/§3 maps); the
  Predecessor → Replacement record lands in the session log before any Build edit.
- **I7** 15-locale i18n from day one for every new/changed string; RTL-correct; machine-gated locale
  deltas (parse + leaf-delta + endings; en+ar CRLF, others LF).
- **I8** Zero boot-path additions; zero IPC while CNS is closed; the health tab's lazy/cached load
  pattern (untrack'd `$effect`, per-library cache, retry-on-failure) is preserved as-is.
- **I9** Sync→async conversions keep identical return shapes (the LL-021/watcher precedent for why async).
- **I10** Perf measured before/after on the live universe (7,661 notes / 234k links / 25-cUniverse
  federation): CNS first-open time, health-tab first-activation time, boot, typing latency — no
  regression anywhere; the first two are the MIG's headline improvements.
- **I11** Federation honesty: scope stays active-universe and is no *worse* than today for cUniverse
  contexts (the tab's `unavailable` state, the well's display-vs-analytics split); no pretend-federation.
- **I12** Reversibility: each § lands as one revert-clean commit; the shed commit is atomic (UI + IPC
  calls + keys ×15 together); no schema change at any step.
- **I13** BUG-015 discipline: no value-sync `$effect`s; any new reactive wiring follows the
  untrack/guard patterns already in the file.

---

## 6. Migration path / back-fill / rollback

- **No schema change, no data migration, no back-fill** at any step — every option reads existing tables
  (`note_meta`, `note_links`) already maintained at write time since MIG-066/067.
- **First boot after update:** nothing to populate; the DB-sourced tension/centrality reads work against
  any MIG-067+ DB as-is. Pre-MIG-067 DBs (no `tags_json`? — column exists since the original schema;
  `outgoing` aggregates not needed here) are not a concern: the columns read here predate MIG-066.
- **Rollback:** revert the commit(s); the fs-walk code paths are deleted-not-flagged, so rollback = git
  revert (no dual-path drift). The shed commit reverts cleanly (UI + keys together).
- **Mid-anything interrupts:** no persisted state is written by any of these reads; nothing to corrupt.
- **Boss-test Stage 0:** binary mtime vs the §-commits under test (standing rule).

---

## 7. What MIG-075 is NOT

- **Not a CCS or KH change** — CCS is consumed via its existing event; KH changes only under ruling f's
  explicit option (ii), which reverses a standing MIG-073 ruling and only Eisa can pick it.
- **Not the federation migration** — MIG-063-family scope (federated tension/centrality/KH reads) stays
  reserved; this MIG keeps exact scope parity.
- **Not a TensionPanel redesign** — the panel and its tab keep their current UX.
- **Not the `lenses.rs` cleanup** (different module, own 2026-05-09 ruling) and **not** the §H
  pill-language or archive-weight calls (parked, Eisa's).
- **Not a Sky View change** — `sky_nodes`/`sky_links` and the PIXI renderer are untouched.

---

## 8. Open questions for Eisa (the ratification gate)

| Q | Decision | My recommendation |
|---|---|---|
| **Q1** | `detect_tensions` re-source design | **D1 Option A** — same algorithm, DB inputs, async, `status='active'` filter |
| **Q2** | The contradictions ×N suffix can't survive the DB re-source (one row per pair; §2.4 delta 2) — accept pair-rows without the occurrence multiplier? | **Yes** — the pair list (what you act on) is identical; the ×40 was repeat-noise inside one source note |
| **Q3** | The shed package (b): Section 2 + dormant chip + the ≤10-link enrichment + the two thin/thick legend rows → "Circulation → CCS" deep-link | **Yes, all of it** (D2) — every removed datum has a better CCS/KH home; CNS opens lighter by 2–3 live IPCs |
| **Q4** | Knowledge-Insights fate (g) | **(ii)** — shed the four circulatory/tension tabs, keep most_connected + knowledge_gaps (topology) |
| **Q5** | most_connected's home (f) | **(ii)** CNS-canonical + retire KH's card (explicit reversal of MIG-073's "keep" — your call); if you'd rather not touch KH: **(iii)** re-point the CNS tab to the cached key |
| **Q6** | `constellation_sight_centrality` re-source (h) | **IN — D5 Option A** (the CNS open stops walking files) |
| **Q7** | `lens.title` ×15 (c) + the caption (d) | **Both IN**; Arabic title **الجهاز العصبي للكوكبة** (confirm the term); caption = one muted line under the CNS count |
| **Q8** | Dead-surface cleanup (i) + registry alignment (j) | **Both IN** — delete tag_edges + the v1 component; edges/legend read the registry (your recolors finally reach CNS) |

---

## 9. Recommendation (one paragraph)

Ratify **D1-A + D5-A + the shed package as recommended**: MIG-075 re-sources `detect_tensions` and
`constellation_sight_centrality` from the indexed DB (two small Rust commits, contracts frozen, async,
exact scope parity, the three honest output deltas ratified up-front), sheds the CNS panel's circulatory
blocks — Link Health, the dormant chip, the decorative 10-link enrichment, and (per Q4) the four
circulatory insight tabs — replacing them with one "Circulation → CCS" deep-link on the existing event,
localizes the CNS title in all 15 languages with a one-line count caption, aligns the canvas/legend
typed-link colors to the Link-Type Registry, and deletes the subsystem's two dead surfaces. KH changes
only if Q5 picks the re-home. No schema change, no back-fill, every step one revert-clean commit, perf
gated before/after on the live universe. **STOP: awaiting Eisa's answers to Q1–Q8 before any Phase-2
Plan.**
