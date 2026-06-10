# Session Log — 2026-06-09

> Note: the 2026-06-09 **localization-complete milestone close-out** (MIG-072 close, orientation v2.60,
> handover) was logged in `SESSION-LOG-2026-06-08.md` (Post-close sections). This file begins the
> 2026-06-09 afternoon block: the **Pending Jobs v1.13 ledger reconciliation**.

---

## §RECON · Pending Jobs v1.13 + orientation §8 reconciliation (doc-only; no code)

**Function in hand:** the project backlog ledger — `docs/Constellation Pending Jobs vX.Y.md` (was stale at
v1.12) and its companion, the orientation §8 Migrations table (was stale at MIG-035).

**Trigger:** fresh-session kickoff picked the handover §4 candidate #1 ("reconcile the stale Pending Jobs").
The kickoff/handover premise was that **Pending Jobs v1.9** was the latest and a v1.10 should be created.

### Cross-check findings (SO #8)

1. **Handover premise wrong.** v1.10 / v1.11 / v1.12 already exist on disk. Git confirms the true latest is
   **v1.12 (2026-05-19, `08e0e2e8`)** — so the reconciliation target is **v1.13**, not v1.10. (Following the
   handover literally would have clobbered a real file with a 3-versions-stale one.)
2. **Both ledgers drifted together, stopping mid-May.** v1.12 ended at MIG-026 / PJ-058; orientation §8 had
   not been extended since its v2.16 full refresh (2026-05-18) and stopped at **MIG-035**. Meanwhile **37
   migration numbers (MIG-036 → MIG-072) shipped** in the three weeks since — none in either ledger.
3. **Ledger error caught + corrected.** The 2026-05-29 session log labelled two federation-scale fixes
   **"PJ-10 / PJ-11"** — colliding with the canonical PJ-010 (Unlinked Mentions) and PJ-011 (Map). They were
   actually filed *unnumbered* on 2026-05-28 ("PJ-NNN-A/B"). Allocated proper numbers **PJ-061 / PJ-062**
   (both DONE); canonical PJ-010/011 left untouched (renumbering forbidden). Eisa-confirmed.

### Method

5 parallel read-only Explore agents enumerated MIG-036→072 (4 agents by range) + reconciled the PJ backlog
(1 agent), every status cited to a commit / orientation preamble / session-log date (BASIC RULE). I then
verified the one ambiguous knot (the PJ-10/11 collision) directly against the 05-28/05-29 logs before
recording anything.

### MIG-036 → 072 outcome (now authoritative in orientation §8 v2.61)

- **Shipped / Closed (23):** 038 (disable Sight+Map → Wings), 039 (Cataloger), 040 (NSC), 041 (term_vocab
  shrink), 042 (drop bridge_concept_id → closes PJ-016), 043/044/045 (NSC P1–P3 + Digest), 055 (Base
  rebuild), 056 (Federation), 057 (Lexicon), 058/059 (search latency), 060 (Base threading), 061/062
  (federation boot-snapshot + filesystem-walk + Tag Browser), 065 (Unified Base), 066 (Living-Link cols),
  067 (User-Definable Link Types), 069 (Style Presets), 070 (Style Setter), 071 (theme removal), 072 (Sky
  View under Setter).
- **Reverted (4):** 046/047/048 Constellation Mind local-LLM stack (`a9cf4d62`, v2.34); 054 first Base attempt.
- **Reserved / never-opened (8):** 049–053 (Mind roadmap), 063/064 (remaining federation surfaces), 068.
- **Dormant / Frozen (2):** 036 (Sight v7), 037 (Time Dome).
- Highest MIG = **072**.

### Deliverables (uncommitted in worktree; pending Eisa's commit go)

- **`docs/Constellation Pending Jobs v1.13.md`** — new file (cp v1.12 → surgical edits). New preamble; PJ-016
  → DONE (MIG-042); PJ-011 → DORMANT (MIG-038); allocates PJ-059→064 (new "Newly filed" section); §9 Done
  table back-filled (PJ-015/035/036/038/040/052–058/016/061/062); new top-of-queue led by **PJ-060**
  (`index_note` cache-hit short-circuit — flagged 2026-05-19 as the highest-leverage open fix).
- **`docs/Constellation Orientation & Onboarding v2.61.md`** — new file (cp v2.60); §8 table extended
  MIG-036→072 (32 rows); §8 header re-dated; v2.61 preamble added. SO #6 satisfied.
- **Memory fix:** `project_mig013_v2_migration_blocking_boot` was stale ("still pending, ship before v1.0")
  — the fix shipped via MIG-015/PJ-001 (2026-05-06). Updated to RESOLVED + MEMORY.md pointer updated.

### Notable side-finding

Handover candidate #5 ("user-definable link types") **largely shipped already in MIG-067** (the Link-Type
Registry) — exactly the kind of thing the reconciliation surfaces before a session is spent on it. Candidate
#4 (`note_links.link_type` 'relates' bug) is now PJ-063 and needs **re-verification under MIG-067**.

### Verification

- v1.13: 1228 lines; top-of-queue + new PJ section + §9 Done render; single `## §9 · Done`; PJ-059→064 present.
- v2.61: §8 flows MIG-035 → 036 → … → 072 → §8.1 cleanly (67 MIG rows total).
- No code touched → no build / svelte-check / boot-perf gate applicable.

### Open / next

- **Commit** both new files + the memory edits together (SO #6: orientation bump in the same commit). Pending Eisa's go.
- New top-of-queue #1 is **PJ-060**; #3 is **PJ-063** (`link_type` 'relates', re-verify under MIG-067 first).
- Deferred (not done here, low priority): deeper per-PJ code-audit of the ~60 carried-forward v1.12 entries;
  refresh of v1.13's stale "Cross-references" appendix.

---

## §MIG-007 · Links Settings tab (PJ-005) — /migration, Option A

**Function in hand:** the Links Settings tab — a new consolidated 'Links' section in the Settings modal.
**Reconciliation (SO #8):** MIG-007 confirmed STILL-OPEN-VALID (no Architect, no work started). Scope
re-mapped post-MIG-067/070 — the Style Setter had since become the home for link types + colours, so
"consolidate every link control" was reframed. **Eisa picked Option A** (behaviour controls + hub links;
styling stays in the Setter → I1 single-styling-home preserved).

**Phase 1 (Architect)** + **Phase 2 (Plan)** docs: `lab/reports/MIG-007-LINKS-SETTINGS-{ARCHITECT,PLAN}.md`.
Plan approved by Eisa.

**Phase 3 build — §1–§4 shipped** (commit `6752d4a8`, frontend-only):
- New Settings 'Links' section (after Sky View).
- Moved the whole **Sky View 'Linking'** group (link format · auto-update-on-rename · use-wikilinks) and the
  whole **Appearance 'Living Link Lifecycle'** group (decay · half-life · confidence back-fill) into it.
  *(Build-time discovery: the 2 plan-named controls sat inside coherent sub-sections with siblings; moved the
  whole groups to avoid orphaning headings. Flagged to Eisa at the gate.)*
- 'Related' hub: Open Style Setter (`openStyleSetter()`), Open Link Dashboard (new
  `constellation:open-link-dashboard` event → `rightSidebarTab='links'` in `+layout`), + Panels pointer.
- All moved controls keep their exact `$appSettings` bindings (no settings migration). Panel visibility
  stayed in Panels (pointer only).

**Verify:** `svelte-check` 0 errors (318 pre-existing warnings). Release binary rebuilt
(`constellation.exe`, mtime 2026-06-09 17:03, Stage 0 fresh).

**Gate:** Stage-1 functional Boss test sent (English; §5 i18n deliberately deferred until the structure is
Boss-approved, so 15-locale translation isn't wasted on a layout that may change). §5 (i18n) + §6
(/simplify + proportionate audit 4A/4B) follow after sign-off.

**Stage-1 result (Boss):** All Pass + 5 remarks → all addressed:
- §5 i18n shipped (`5a39b6e6`): root cause was `$t()` returns the key on miss, so inline English fallbacks
  never fired → added `settings.sections.links` + `settings.links{}` (9 keys) to all 15 locales (14 via
  parallel translator agents reusing each locale's terms; line endings preserved). Half-life **number-entry**
  input added (clamped 7–365). **Appearance section retired** — Title alignment moved to Editor → Display
  (its last control after MIG-071); default section is 'dashboard', nothing deep-links to 'appearance'.
- Re-test result: A pass, B pass. Two refinement remarks → both fixed (`63a6ec27`): "Open Style Setter"
  now deep-links to the **Links category** (new `styleSetterCategoryRequest` seam mirroring inspect-request);
  "Open Link Dashboard" pulled OUT of the `isHome && sidebarTab` note-gate to a top-level branch (like Tags)
  so it opens library-wide. (First tried weakening the gate → broke `sidebarTab` non-null narrowing for the
  note panels → reverted; out-of-gate relocation is the correct fix.)

## §STATE-OF-STANDING + PIVOT — Link Dashboard concept (Eisa, 2026-06-09)

Re-test B passed mechanically, but **Eisa questioned the Link Dashboard's concept**: "could serve an open
note, but not as a universal Link Analysis instrument… what is the concept and rule of the Link Dashboard?"
Investigation (FACT): `LinkDashboard.svelte` is **already universe-wide** (`allLinks`+`allNotes`, no
`activeNotePath`) with **7 sections** (Most Connected · Most Traveled · Stale · Cross-Library · Broken ·
Orphans · Archived), but is **mounted only as a ~300px right-sidebar tab** and has **no concept doc anywhere**.
The Owner's tension is exact: universal function, note-context vessel.

**MIG-007 state:** §1–§5 + hub fixes **shipped + Boss-validated** (commits `6752d4a8`, `5a39b6e6`,
`63a6ec27`). PAUSED before §6 close-out (/simplify + audit + orientation v2.62). The "Open Link Dashboard"
hub button is **held as-is** pending the Dashboard's new home. Nothing is broken; MIG-007 is effectively
shippable.

**Eisa decision:** **Write a Concept Paper first** (define concept/rule/home, ratify, *then* a `/migration`
promotes it). Delivered `docs/Constellation-Link-Dashboard-Concept-Paper-v1.0.md` (FACT vs PROPOSAL marked
throughout): defines it as the diagnostic instrument for the Living-Link graph at universe scale; the §4
reads (Load-bearing / Erosion / Isolation / Curation); §6 what-it-is-NOT vs Backlinks/360.3D/Sky View/Index/
Cataloger/Map; §7 the home rule (first-class surface, never a note panel); §11 five questions for the Owner
(home · section set · right-sidebar remnant · name · MIG-007 hub button). Awaiting ratification → v1.1 → MIG.

## §LINKS-CORPUS + COMPLEMENTARITY CROSS-CHECK → CCS reconception (Eisa, 2026-06-09)

**Eisa challenge 1:** "Have you read all links/typed-links docs?" — Honest: NO. Read the full corpus this turn:
`Living-Link-Concept-Paper-v1.0` (ratified concept), `CONSTELLATION-KNOWLEDGE-FORMULATION` (philosophy),
`Living-Links-Guide-v1.0`, `USER-DEFINABLE-LINK-TYPES-ARCHITECT` (the registry / "living vocabulary"),
`LINKTYPE-SYNTAX-CORRECTION-ARCHITECT`. Key concepts my v1.0 Dashboard draft had MISSED: **untyped link = the
question / live edge of inquiry** (not "broken"); **facts rest, formulations inquire** (never nag a fact;
signals scope to the formulation layer); **`contradicts` = the engine**; the 8 acts + canonical order; the
**living-vocabulary registry** (8 + custom). Plus the diagnostic-instrument canon (§V.3) + P5 "knowledge
health dashboard / circulatory health."

**Eisa challenge 2:** cross-check against the OTHER core-plugin concept papers — each function must
*complement* the others to draw one clear picture and *formulate* (not manage). Read (self + 5 parallel
agents): Cognitive-Engine One-Picture (the unifying frame), master Concept Paper, CECE/Cataloger, Base v2.0,
NSC v2.0, 360.3D, Stages v1.2, Sight Subsystem, CNS (help + MIG-061).

**The frame:** Cognitive Engine = one process (Five Acts) → FOUR questions mirrored back
(**Development / Altitude / Origin / Connection**) → destination (Conviction). Instruments: Development =
Stages/maturity · Altitude = stratum (Sky View size, 360.3D vertical) · Origin = **the Cataloger** · Connection
= typed links + **CNS** + 360.3D (per-note) + Sky View (spatial). Cross-cutting: the **Base** (survey/compare
table, threads into deep-read surfaces) · **NSC** (aboutness service) · Index. Sight/Map = disabled Wings.

**The finding (ratified by Eisa):** the founding metaphor is a **dual** system — Nervous + Circulatory.
**CNS** is the **Nervous** register (topology: communities/centrality/bridges/structural-blind-spots,
ignores time/weight/traversal). The **Circulatory** register (weight/decay/dormancy/traversal/lifecycle/
confidence-flow) is **architecturally present but had no universe-scale home** — the open slot. The new
instrument is that circulatory complement: **CCS — Constellation Circulatory System (الجهاز الدوري)**, peer of
CNS. *("CNS never tells you a load-bearing bridge is going stale; CCS never tells you a worn link is a
bridge.")* Also corrected: the name "Link Dashboard" is **already** the authoring archive panel (Sight
Subsystem §3.2/§7) — so the diagnostic needs its own identity; my v1.0 draft's "Most-Connected" section is
**CNS's topology**, not CCS's.

**Eisa ratified:** (1) placement = circulatory complement to CNS; (2) name = **CCS**.

**Delivered:** `docs/Constellation-Circulatory-System-Concept-Paper-v1.0.md` (CCS v1.0) — the dual-system
completion; CCS-vs-CNS boundary table; the circulatory diagnostics (ECG/pressure/decay/blood-test/lifecycle/
autopsy/typed-balance); the canon guardrails (untyped-as-question, facts-rest, contradicts-as-engine,
registry-driven); what-it-is-NOT vs every sibling; left-dock Core-Plugin home; §13 four owner questions
(subsume Knowledge-Health? · keep a slim authoring panel? · section set · re-point MIG-007 hub). The
`Link-Dashboard-Concept-Paper-v1.0` is banner-marked **SUPERSEDED by CCS**. Awaiting §13 answers → v1.1 → MIG.

**MIG-007 still PAUSED at §6 close-out**, shippable; the hub button now re-points to CCS when CCS ships.

**§13 ANSWERED + CCS v1.1 RATIFIED (Eisa, 2026-06-09).** Rulings: Knowledge-Health = **coordinate** (not
subsume); authoring "Link Dashboard" panel = **fully retire** (universe view → CCS, per-note → Backlinks/
Outgoing); MIG-007 hub = **re-point to CCS**; section set = **No to medical labels → cognition terms**. Eisa
then asked me to **defend the circulatory theory + produce updated terms + convince future users** — delivered
the 4-argument "Why Circulatory" defense (the data was *built* as circulation: `weight=1+ln(1+tc)`,
decay=vessel-narrowing, lifecycle=vessel-life; dual-system founding design; topology-can't-see-death; formulation-
not-management) + the user pitch + the 7 cognition-named registers (**Living Connections · Load-Bearing
Reasoning · Cooling Inquiries · Conviction & Doubt · The Life of a Connection · Retired Reasoning · The Acts
of Inquiry**). **Eisa: "Approved."** Delivered `Constellation-Circulatory-System-Concept-Paper-v1.1.md`
(RATIFIED); v1.0 banner-marked superseded. **Next: CCS `/migration` Architect** (frontend-mostly: stand up the
left-dock surface, build the 7 registers, retire the panel, coordinate Knowledge-Health, re-point the hub).
MIG-007 §6 close-out still pending in parallel.

## §MIG-007 §6 CLOSE-OUT — MIG-007 CLOSED (2026-06-09)

**`/simplify`** (4 cleanup agents on the diff): the diff was ~90% clean relocation + correct pattern-mirroring.
Applied 3 in-scope fixes to `SettingsModal.svelte`: (1) half-life number input inline `width:72px` magic-px →
named `.setting-num` class; (2) deleted the orphaned `<!-- APPEARANCE -->` banner left by the section removal;
(3) retitled the stale `SKY VIEW & LINKS` banner over the links branch → `LINKS`. Skipped (justified): the
two hub-button mechanisms (real scope asymmetry — `styleSetterOpen` is a global store, `rightSidebarTab` is
local `+layout` state); merging the two StyleSetter `$effect`s (independent intents). svelte-check **0 errors**.

**Filed as follow-up** (`spawn_task` `task_19b5319d`): the altitude finding — the right-sidebar `isHome &&
sidebarTab` note-gate special-cases `tags`/`links` (hoisted) and **duplicates** `review`/`sourceReview`
(rendered twice). Pre-existing debt my `links` hoist made visible; a larger `+layout` refactor → its own task.

**Phase-4 audit (2 parallel agents):** **Invariants 6/6 HOLD** (I1 single-styling-home — Links tab has zero
styling controls; I2 all moved controls bind to unchanged `$appSettings` paths; I3 autoUpdateLinks cascade
untouched at `+layout:4617`; I4 i18n parity all 15; I5 no duplicates + `'appearance'` fully excised, 0 dead
refs; I6 no `$effect` loop / no hot-path invoke). **Drift 5/5 PASS** (LinkDashboard branch exactly once;
each moved control single-homed; one dispatcher/one listener for `open-link-dashboard`; one writer/reader for
`styleSetterCategoryRequest`, valid `'links'` category target). Non-blocking note: the new document listener
isn't removed in cleanup — but mirrors the pre-existing `show-importer` sibling (no *new* leak class).
4C migration-path: light — no schema/data change; existing settings render in new sections; rollback = revert UI.

**MIG-007 → CLOSED.** Architect→Plan→Build(§1–§5 + hub fixes)→/simplify→Audit all done; Boss-validated A/B.
Commits `6752d4a8` · `5a39b6e6` · `63a6ec27` + this §6 cleanup. Orientation §8 updated → v2.62.

## §NOTE-GATE REFACTOR — task_19b5319d DONE (2026-06-10)

Eisa directed the filed follow-up. `+layout.svelte` right-sidebar tab content: replaced the ad-hoc
`isHome && sidebarTab` note-gate (which hoisted `tags`/`links` above it and **duplicated** `review`/
`sourceReview` inside + outside it) with a declarative `const NOTE_SCOPED_TABS = new Set([properties,
backlinks, star, tasks, calendar, health, provenance, inspector360])`, gated ONCE:
`{#if NOTE_SCOPED_TABS.has(rightSidebarTab) && !(isHome && sidebarTab)}` → "no note selected"; else each tab
renders **once**. Verified the two `review` copies byte-identical and the two `sourceReview` copies merge
under `activeNotePath={sidebarTab?.path ?? null}` (SourceReviewPanel defaults that prop null + treats falsy as
no-note, `:671/:715`). `properties` keeps `sidebarTab` non-null via `&& sidebarTab` (only branch that
dereferences it). Diff: **11 insertions / 32 deletions**, all gating. svelte-check **0 errors**.

**Boss test:** Stage 2 (the actual gating test) **PASS** — universe-wide tabs (Tags/Links/Review/SourceReview)
render with no note; note-scoped show "No note selected." Stage-1 findings (Knowledge Health "Loading…",
Provenance/Review partial) **VERIFIED pre-existing, not regressions** — the diff proves those panel bodies are
untouched (only NOTE_SCOPED_TABS *names* them + the duplicate ReviewPulsePanel was *deleted*); KnowledgeHealth
loads via its own `onMount` (`khd-loading`), mount condition unchanged. Chip `task_19b5319d` dismissed.

**Next (Eisa):** investigate the pre-existing **Knowledge Health stuck-on-Loading** (read-time diagnostics on
the 7,600-note / 656k-link universe — a §12 read-time CE surface).

## §KH-DECAY FIX — Knowledge Health "Loading…" root-caused (2026-06-10)

Investigated the pre-existing KH freeze. Root cause was worse than slow: `constellation_link_decay` —
called on **panel open + boot (`+layout:2709`) + a 24h periodic job (`+layout:2444`)** — ran a **write**
loop on a read-time diagnostic. It (a) UPDATEd the raw `weight` column though decay is **display-only**
(Living-Links-Guide §7 / `effectiveLinkWeight`), (b) **compounded** — re-reading the already-decayed weight
and decaying it again every call → silent weight corruption, (c) scanned 234k rows with per-row `julianday`.

**Measured on the real DB** (`Eisa Cognitive Knowledge`, **234,061 links**, read-only Python): decay SELECT
~11s; `most_connected`/`bias_check` GROUP-BY-target_name ~3s/~1s; serialized on one mutex → ~18–20s frozen +
DB-lock held. **Correction logged:** the residual freeze is NOT julianday CPU — a "string-threshold" rewrite
was *also* 11s cold / 161ms warm. It's a **cold read of the 1.7 GB `search.db`** (whichever query hits
`note_links` first pays ~11s; `spark`/`birth` match `traversal_count=0` = 234,040/234,061 ≈ all rows). The
1.7 GB DB is itself suspect (FTS bloat; LL-XXX vacuum history) — flagged separately.

**Fix shipped (focused — Eisa-approved):** `constellation_link_decay` → **read-only** (Step-1 write loop
deleted; `decayed`/`new_dormant` kept 0; Step-2 lifecycle counts unchanged). Removed the boot
(`+layout:2709`) + periodic (`+layout:2444`) decay triggers (decay is display-only — no job to run). The
useless julianday→string rewrite was **reverted** (cold-read, not CPU). `cargo check` 0 errors (pre-existing
warnings only); `svelte-check` 0 errors. **Note:** `store.ts:1610 linkDecay()` wrapper now unused (dead
export, harmless). Dormancy count no longer auto-written — moot on this data (0 traversed-then-idle links).

**Decision (Eisa):** commit this fix, **then a Rule-8 write-time-caching `/migration`** for KH so the panel
reads precomputed counts (no cold scan; also makes dormancy read-time-derived). Migration Architect opens next.

## §MIG-073 — KH snapshot cache: Architect + Plan + P1 + P2 (2026-06-10)

**Architect** (`MIG-073-KH-CACHE-ARCHITECT.md`): recon mapped 11 `note_links` write paths + the 12 existing
triggers/3 derived surfaces (sky_links, note_meta.outgoing_*, sky_nodes.stratum/maturity) + the FTS trigger
blueprint. Scope finding: KHD's content IS CCS's circulatory domain (CCS §8 mandates exactly this layer) →
build the cache GENERAL, KH = first consumer. Crux: write-driven vs **time-driven** buckets (spark→birth at
7d — no write event) → pure triggers can't work. Options A/B/C; **Eisa picked B + general scope.**

**Plan** (`MIG-073-KH-CACHE-PLAN.md`, approved): stale-while-revalidate snapshot (`link_stats_cache`),
10-min freshness, recompute always background on a dedicated connection. P1 backend → P2 panel (Boss test)
→ P3 refresh hooks (Boss test) → P4 simplify+audit.

**P1 shipped `556d401e`:** table in init_db; query bodies extracted to `&Connection` helpers (IPCs stay as
wrappers — one set of SQL); `recompute_link_stats_cache()`; `constellation_knowledge_health_snapshot` IPC
(missing keys → `{ready:false}` + background populate + `kh-snapshot-ready`; stale → instant return +
background refresh); first-time population on both `cache-reconciled` emitters (`only_if_empty` — single
COUNT on later boots, MIG-067 walk-free boot intact); AtomicBool in-flight guard. cargo check 0 errors;
SQL dry-run on a copied test DB: snapshot read **0.17 ms**, payload round-trip + NULL-age edge verified.

**P2 shipped `77ed2786`:** KHD onMount → ONE snapshot call; cold-cache → loading state + event listener +
5s self-healing poll; listener/interval cleaned on destroy. svelte-check 0 errors.

**⚠ Architect mis-map found during P2 (logged in the Architect doc):** the right-sidebar **'health' tab =
`TensionPanel`** (`detect_tensions`, `+layout:3293`) — NOT KnowledgeHealthDashboard (the full-screen overlay
via command palette 🧠 / dock). Eisa's tab screenshot is the **TensionPanel** "Loading…" (null report on
error — separate pre-existing bug, out of MIG-073 scope; triage after). KHD i18n drift also noted (panel is
hardcoded EN — pre-existing, for P4/PJ). Binary building; P2 Boss test next (points at the OVERLAY).

## §HEALTH-TAB FIX — the REAL original complaint root-caused + fixed (2026-06-10)

**Boss verdict on MIG-073 Stage 1:** core Knowledge Health overlay renders full data (screenshot) — pass.
**Boss correction:** the original issue was never the overlay — it's the right-sidebar tab *labeled*
"Knowledge Health" (note-scoped), stuck on "Loading…". + Observation: the overlay's big card numbers
(234,061 / 142,348) bleed past the card edges.

**Tab root cause:** the tab renders `TensionPanel` fed by `tensionReport`, which was loaded ONLY inside
`enrichNodesBackground` — removed from boot by the zero-boot-walks rule and now reachable solely from the
on-demand Sky View legend action (`+layout:5789`). Loader disconnected, tab never re-pointed → eternal
"Loading…". Also: the old call scanned `libPaths[0]` — the wrong library in any multi-library universe.
(`detect_tensions` itself is MIG-067-correct on link types via `resolve_wikilink_type` — no detector rewrite;
its fs-walk Rule-8 modernization belongs to the CNS/CCS reorganization, queued there.)

**Fix:** `loadTensionReport(notePath)` — lazy, fired by a `$effect` when the health tab activates with a
note open (`untrack`ed call; guards in plain vars — no loops); scoped to the **active note's library**;
cached per library path; failure clears the guard (retry on next activation). `TensionPanel` gains a
`loading` prop + two honest states: `analyzing` (spinner copy) and `unavailable` (error/no-run — never an
eternal "Loading…"). i18n: `tensionPanel.analyzing` + `.unavailable` added to ALL 15 locales (merge script;
verified +2/-0/~0 per file; CRLF en+ar / LF others preserved; pre-existing `}\n\n\n\n,` wart + missing EOF
newline normalized). **CSS:** `.khd-card` `flex:1` → `flex:1 1 auto` (basis=content → 7-digit counts widen
their card instead of bleeding; wrap absorbs). svelte-check 0 errors.

**Boss test (health tab + bleed):** Part A PASS (tab analyzes + renders 434 contradictions etc.) — but
**no scroll**; Part B PASS (cards no longer bleed; ANNOTATED wraps to its own row — Boss-accepted).
**Scroll root cause:** the `.rs-full-height` host column is `overflow:hidden` (children own their scroll) —
and THREE panel roots never did: `.tension-panel`, `.rp-panel` (= Eisa's earlier "Review Pulse not displaying
the whole list" remark — same family bug), `.prov-panel`. All three → `flex:1; min-height:0; overflow-y:auto`
(self-contained, identical pattern). svelte-check 0 errors.
**Observation parked for Eisa:** the contradictions list shows mass-duplicate rows (Thomas Aquinas
contradicts "ISBN (identifier)" ×~40) — `detect_tensions` emits one row per wikilink occurrence, not per
(source,target) pair; dedupe offered, awaiting Boss call (semantics change, not silently shipped).

**Scroll Stage-3 Boss test: PASS** (tension + Review Pulse lists scroll; full content reachable).
**Dedupe: Boss-approved → shipped.** Detection-1 now aggregates per (source_path, target) pair with a ×N
suffix on repeats + a stable sort (count desc, then source name — HashMap order was random per open).
Orphans/single-points unaffected (`inbound_sources` was already a per-source set). cargo check 0 errors.

**P3 shipped (as-built deviations logged in the Plan doc §P3):** reconcile-walk hook → **unconditional**
recompute (the true bulk settle point); `links_backfill` hook **dropped** (verified: it never mutates
`note_links` — hooking it adds nothing); freshness window **10 → 2 min** (plan-marked tunable; open-driven
kick, background dedicated-connection recompute); the panel now listens for `kh-snapshot-ready` the whole
time it's open (registered before the first fetch — no missed-event race) so a stale snapshot renders
instantly then **updates in place**. Plan's original P3 test corrected: single-note saves go via
`reindex_single_note` (never `cache_reconcile`) and no live Rebuild-Index invoke exists — SWR is the
propagation path for single-link changes. svelte-check 0 / cargo errors 0. One binary carries P2+P3;
staged Boss tests (Stage 1 instant-open, Stage 2 freshness) at the stop.
