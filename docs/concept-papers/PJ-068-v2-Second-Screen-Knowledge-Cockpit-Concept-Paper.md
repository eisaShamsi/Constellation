# PJ-068 v2 — The Second Screen as a Knowledge Cockpit (Concept Paper)

**Date:** 2026-07-09 · **Status:** Concept (Boss **reopened + expanded** PJ-068 on 2026-07-09, during G3 Stage-1 testing). **Extends** [PJ-068](PJ-068-Second-Screen-Contextual-Companion-Concept-Paper.md) (keeps its razor + history + replication audit) and [26-second-screen.md](26-second-screen.md); **supersedes** the same-day G3 ruling "read-only *default* + editable toggle" with "**read-only, always**." Needs per-surface Boss rulings → `/migration`.

> **Concept before Function (the horse and the carriage).** This paper states the *concept* of the re-envisioned Second Screen and is deliberately grounded in (a) every existing SS doc and (b) how professional software splits work across a second screen (WA#5 — cross-checked against proven methods, workflow `wf_8b4fdfa4-86d`, 10 fronts). No code is written until the concept is ratified.

---

## 0. Why this paper exists — the three Boss decisions (2026-07-09)

During the G3 two-window test the Boss corrected the premise and re-scoped the Second Screen:

1. **The SS is READ-ONLY, always.** Not "read-only by default with a toggle" — the SS is *never* an editing domain. Drop the `secondScreenEditable` toggle and the cross-window cascade-freeze that existed only to make SS editing safe.
2. **The SS complements EVERY surface — not only the NotePane.** The note editor, Sky View, Constellation Map, the Index, the Dashboard, Tasks, Sight: each gets its contextual complement.
3. **Re-conceive the SS as a unified, 100%-MS-interactive complement** — *"bring the whole surfaces together when we enable the SS — a **Control Dashboard**, a **General Estimation Map**, an **Operation Map**."*

These restate — and finally *unify* — the original law: on 2026-04-05 the SS was defined as *"an extension of the mind"* with the panels migrating to it so the main window becomes a clean writing space; PJ-068 (2026-07-04) ratified the razor **Contextual / Complementary / Chosen**. This v2 supplies the missing piece PJ-068 flagged: *what each surface's complement should actually be*, and *how one SS scales across all of them.*

---

## 1. The concept (the horse)

> **The Second Screen is the "Presenter Display" for knowledge formulation: a read-only cockpit that always shows the hidden dimensions *around* whatever you are looking at in the main window — never a mirror of it.**

It shares exactly **one anchor** with the main window — *the current focus* (the open note, the selected Sky node, the hovered Map arc, the active Index term, the selected task) — and fills the rest of its glass with the three altitudes a single-focus main surface structurally **cannot show at the same time**:

- **WHERE** this focus sits in the whole Universe — the *estimation map / "you-are-here" locator*.
- **The HEALTH** of the living links around it — the *control dashboard* (weight, confidence, decay, review-due, tensions surfaced as exceptions).
- **The DECISION-SPACE** of where the Five Acts can go next — the *operation map* (backlinks, typed relations, unlinked mentions, candidate connections).

It couples to the main window through **one explicit, releasable dial** — *Normal / Live / Locked* — and offers **exactly one action: click-to-navigate**, which moves the main window's focus but never mutates content. Every panel is a **cheap lookup of write-time-maintained derived data**, so it re-renders on focus-change without ever re-walking the Universe. *A display that spans the space the editor cannot occupy — never a second domain.*

Why "Presenter Display" is the right north star: in PowerPoint/Keynote **Presenter View**, the two screens share exactly one element (the live slide); everything else on the presenter's screen is the hidden dimension the audience never sees (notes, next slide, timer). That is the purest *complement-not-duplicate* archetype in professional software — and it is exactly Constellation's law.

---

## 2. The governing law (the PJ-068 razor + two new absolutes)

Every SS surface must pass the **three-C razor** (unchanged from PJ-068):

- **Contextual** — does it respond to what the main window is doing *right now*?
- **Complementary** — does it show something the main window is *not* showing?
- **Chosen** — does it appear only because the user deliberately opened the SS, never self-initiating?

Plus two absolutes ratified 2026-07-09:

- **Read-only, always** — the SS never writes to disk, never renames, never edits properties. (Reinforces the settled **Display-Not-Domain** law; the whole cross-window *write*-conflict class disappears by construction.)
- **Span-the-axis** — the SS must always sit at a **different point on the overview↔detail axis**, or in a **different representation**, than the active main surface. Editor (detail) → SS is the graph/overview; Sky View (overview) → SS is the node's text/detail. *If the SS would show what the MS already shows, it must change what it shows.* This is the operational test that makes "complementary" enforceable.

---

## 3. The three-zone cockpit — the Boss's three metaphors, made literal

The single most important structural decision from the prior art (NASA "Big Boards"; ISA-101 control-room standard): **fixed spatial zones**. Give the SS stable regions so the eye lands in a known place *no matter which main surface is active*. This is how **one** SS complements editor + Sky + Map + Index + Dashboard + Tasks + Sight **without a per-surface redesign** (the sprawl trap — seven half-built apps — is thereby avoided).

| Zone (Boss's metaphor) | What it is | Constellation content | Prior art |
|---|---|---|---|
| **① Estimation Map** *(General Estimation Map)* | **The ONE holistic view of the whole Universe, across time — past · present · future** (Boss, 2026-07-09; see §3.1). Not merely a locator: it shows where the knowledge *has been*, *is now*, and is *going next*; the current focus is located *within* that temporal field. Answers *"what is the state and trajectory of my whole knowledge — and where does this focus sit in it?"* | The whole field (persisted `sky_nodes`/`sky_links`) layered across time: **past** (creation timeline, link maturation spark→growth→maturity, established/superseded), **present** (active/load-bearing links, live clusters, stratum/maturity), **future** (review-due, decaying/dormant links approaching renewal, unresolved tensions/contested links, orphans) — with a live marker on the focus. | ArcGIS Overview window; Bloomberg Monitor group; Endsley "perceive WHERE"; + a temporal spine (the living-link lifecycle + Five Acts arc). |
| **② Control Dashboard** *(Control Dashboard)* | The **health/lifecycle** strip — *management-by-exception*. Answers *"is my knowledge healthy; what needs attention?"* — legible from across the room. | Per focus **and** universe-wide: each living link's **stage** (Spark→Birth→Growth→Maturity→Dormancy→Renewal), **weight** + decay, **last-traversed**, **confidence** (hypothesis→evidence→established→contested); review-due pulses; unresolved **tensions/contradictions**; orphans. Only what is *notable*. | OBS **Multiview** status-by-color; ISA-101 Level-1 "confidence monitoring"; NOC/SOC *show-only-exceptions*; Presenter View timer/ready-bar. |
| **③ Operation Map** *(Operation Map)* | The **trajectory / decision-space** around the focus — a consequence-free audition of the next move. Answers *"where could I go next; which of the Five Acts is available here?"* | Backlinks + outgoing links; the **typed-link inventory** (supports / contradicts / causes / exemplifies / generalizes / derives-from / part-of / supersedes / associative) with each link's data-block; **unlinked mentions**; sibling notes; the tensions that seed the next Act. | DJ **Prepare/look-ahead** crate + prelisten; OBS **Preview** candidates; Presenter View next/previous; ATC data-block + fly-out. |

The three zones are the **shared vocabulary**: whatever the MS is doing, the SS re-fills these same three regions from the one focus anchor. (§6 gives the per-surface content.)

### 3.1 The Estimation Map — the one holistic universe view (Boss ruling, 2026-07-09)

> *"The General Estimation Map should act as the holistic universe view. It should reflect the past, the present, and the future (what's next). If we could create ONE Holistic View of the universe, it would be through this map."* — Eisa, 2026-07-09.

The Estimation Map is the **marquee zone** — the single place the whole knowledge universe is seen holistically, and *across time*. It is not a "you-are-here" dot; the locator is the smallest thing it does. Its three temporal layers:

- **Past — how the knowledge got here.** The growth & maturation history: note creation timeline (from `cid_cn` timestamps), how links matured (spark→birth→growth→maturity via traversal), what has been established, superseded, or archived. The provenance and shape of the field's formation.
- **Present — the live field.** The current whole-Universe shape (persisted `sky_nodes`/`sky_links`): active and load-bearing links, live clusters, the distribution of stratum/maturity — what is alive right now.
- **Future — what's next (the "estimation").** The projection: review-due notes (`get_due_notes`), decaying/dormant links approaching renewal, unresolved tensions and contested links awaiting synthesis, orphans awaiting connection, emerging clusters. This is the holistic *estimate* of where the knowledge is heading and what it needs next — the strategic read the writing surface can never give.

This is the literal embodiment of Constellation's temporal knowledge philosophy: the **living-link lifecycle** (Spark → Birth → Growth → Maturity → Dormancy → Renewal/Archival) and the **Five Acts** (Observation → Connection → Tension → Synthesis → Conviction) are *both* past→future arcs — the Estimation Map renders the whole Universe along that arc. All three layers are derivable from **persisted** data (creation timestamps, `note_links` history, `sky_nodes`, `review_schedule`), so it stays schema-free; the design challenge is the **rendering** (how to show past/present/future in one coherent map — temporal encoding, a time spine, layered overlays), which earns a **dedicated design pass before it is built at P4** (research + a focused enrichment of this section). The rest of the cockpit (Control Dashboard, Operation Map) stays focus-relative; the Estimation Map is the zoom-all-the-way-out, all-of-time view.

---

## 4. The coupling dial — Normal / Live / Locked

The single most transferable idea from the research is **Lightroom's tri-modal Secondary Display Loupe**. The SS gets one **visible, releasable** selector with three positions — the *entire* interactivity model plus the read-only guarantee in one control:

- **Normal (selection-follow).** The SS reflects whatever the MS has **selected/opened** — re-renders on the committed focus. The default resting coupling.
- **Live (hover-peek, non-committing).** As the cursor **hovers** a wikilink, a Sky bubble, a Map arc, or an Index term in the MS, the SS previews *that target's* context **without changing the MS's selection** — a consequence-free peek channel (the DJ prelisten / ArcGIS non-disturbing Magnifier of knowledge). This is the read-only, never-mutate discipline made into a feature.
- **Locked (pinned reference anchor).** The SS **detaches** and pins one note/graph/dashboard as a held reference while the MS roams freely — e.g. keep the *source* note locked on the SS while writing the *synthesis* note that cites it in the MS (the **Synthesis Act** made ergonomic).

The bond is **explicit and releasable** (Obsidian's visible link toggle) — which also *structurally forbids* a hidden cross-window `$effect` feedback loop. Data flows **one-way**: the MS pushes focus changes over the existing `emit`/`listen` channel; the SS observes and reflects; the SS never pushes a value the MS re-reads.

---

## 5. The one interaction — click-to-navigate (100% interactive, 0% mutating)

The SS is **"a navigator, never an editor."** It is 100% interactive but strictly non-mutating: the **only** drive it exerts on the MS is **click-to-navigate** — click any item in the SS (a backlink, a typed relation, a term mention, an exception in the dashboard, a note under a Map arc) and the **MS focus moves to it**. Moving focus changes the *cursor*, not the *content*, so it is safe under the read-only law and satisfies the "100% interactive" requirement.

Every richer drive affordance from the prior art is **deliberately dropped**: Lightroom's click-thumbnail-changes-selection, OBS double-click-to-Program, the trader's bidirectional link-as-driver, ArcGIS drag-overview-to-pan — anything that could mutate content or commit state. (The DJ "load-the-track" hand-off is the exact mental model: audition freely on the SS; one deliberate click promotes a focus into the MS.)

---

## 6. Per-surface complement map (what the SS shows for each MS surface)

The three zones stay fixed; their *content* is filled from whatever the MS is focused on. Verdicts reconcile PJ-068's replication audit.

| Main surface (altitude) | SS complement (span-the-axis) | Interaction |
|---|---|---|
| **Note editor** *(prose, note-altitude)* | The **derived reading** of the note (Figma "Dev-Mode Inspect: specs, not artwork"): its local link-graph, backlinks/outgoing, the typed-link inventory with each link's data-block, unlinked mentions, a review/freshness pulse. **Never the note body.** | Live-follow to the open note. Click a backlink/neighbor/relation → MS editor opens it. **Locked**: pin the source note while writing its synthesis in the MS. |
| **Sky View** *(force-graph, graph-altitude)* | The **selected node's detail** the bubble can't render: the note's text/preview + its living-link data-block + typed-relation list (the ArcGIS-Magnifier non-disturbing zoom). Graph shows *position*; SS shows *content + semantics*. | **Live** (hover a bubble) peeks its card without moving the graph. Normal selects. Click a relation → MS graph re-centers. |
| **Constellation Map** *(radial sunburst, taxonomy overview)* | The **drill-path + leaf detail**: taxonomy breadcrumb, the notes contained under the hovered/selected arc, the you-are-here footprint, each note's compact data-block. | **Live** (hover an arc) lists its notes with zero effect on the sunburst. Click a note → MS opens it. Progressive disclosure: arc → note list → fly-out. |
| **Index** *(FTS5 term-browser — the diagnostic instrument)* | The selected term's **mention-expansion + scope** (brushing-and-linking): co-occurring terms, the notes that mention it, the `via {lemma}` + `≈ similar` expansions, per-term density. Answers *"how does this term spread across the field?"* | Hover/select a term brushes its instances into the SS. Click a mention → MS opens the note; click a co-occurring term → MS Index re-focuses. |
| **Dashboard / Sight overview** *(whole-Universe)* | The purest **Control-Dashboard + Estimation-Map** instance: universe-scale locator + a **management-by-exception** strip (review-due, decaying/dormant links, unresolved tensions, orphans, recent activity, star/count health). | One-way live-follow of whole-corpus state (all write-time-maintained). Click any exception → MS navigates straight to it (a triage queue that drives focus). |
| **Tasks** *(list/board)* | The task's **knowledge context + timeline**: the notes and living links the task references (its provenance), the due/lifecycle timeline, related tasks. Complements by showing the knowledge the task *points at*. | SS follows the selected task. Click a referenced note → MS opens it. **Locked**: pin a task's source context while working the list. |
| **Sight** *(tradition-lens epistemic rendering)* | The notes/cluster **behind the selected region/stratum** of the lens + its taxonomy source-provenance — expressing **only** what that tradition's grammar affords (**Form-Aligns-To-Purpose**; no imported dimensions). | Select a region/stratum → SS lists its notes. Click → MS opens it. Read-only throughout; **no lens recomputation** triggered from the SS (pre-derived reads only). |

---

## 7. Prior art — the cross-software basis (WA#5)

Distilled from 10 researched fronts; each principle carries its source so the design is *borrowed, not invented*:

- **Share exactly ONE anchor, complement everything else** — Presenter View (live slide is the only shared element).
- **Span-the-axis / two-altitude viewing** — Lightroom (Grid on main, Loupe on second, simultaneously); Cockburn overview+detail; ISA-101 altitude split; Blender Quad-View (same object, distortion-free reference frames).
- **The tri-modal coupling dial (Normal/Live/Locked)** — Lightroom Secondary Display Loupe. *The single most transferable idea.*
- **Read-only reflect is first-class, not degraded** — OBS Projector; ArcGIS Magnifier ("zoom without changing the extent of your data view"). The non-disturbing guarantee is a feature.
- **Management-by-exception** — NOC "show only exceptions"; SOC triage. Surface what is *notable*, never a data dump.
- **Fixed spatial zones** — NASA Big Boards ("divided into sections, each dedicated to a set of data"); ISA-101. Spatial consistency is what makes glance-reading fast.
- **Overview-drives-detail via one-way linked selection** — Bloomberg named link-groups; VS Code minimap; Figma Inspect (live-follow selection, computed/derived projection).
- **You-are-here locator** — ArcGIS Overview extent box.
- **Consequence-free audition + deliberate hand-off** — DJ prelisten + load-the-track; OBS Preview→Program two-state discipline.
- **Explicit, releasable bond (never a hidden always-on echo)** — Obsidian linked-panes visible toggle.
- **Chosen + persistent** — universal: opens on deliberate invocation; restores its arrangement per workspace.

---

## 8. Reconciling with today's code (the PJ-068 replication audit, resolved)

The v2 concept *decides* the rulings PJ-068 left open. Today's 14-branch mode if-chain (`SecondScreenPage.svelte`) becomes a **focus-channel + three-zone state machine**.

- **KEEP / fold into the zones (already complement):** Sky-graph companion → Operation-Map/detail; Split-compare → the compare instance of the three zones; Editor-panels migration → the Operation-Map/Control-Dashboard for a note (the "model to generalize"); Index *compare* leg → Estimation/Operation for terms.
- **RETIRE (fail the razor):** **Navigator companion** (replicates the same NotebookNavigator the main sidebar shows — the canonical violation); **OrgChart mode** (replicating *and* unreachable dead code); the **fallback tab-strip editor** (a freestanding second editor — non-contextual *and* now forbidden by read-only-always: a second domain).
- **REDESIGN into the three zones:** Map-drilldown (keep the note-list + you-are-here; drop the hierarchy re-render the sunburst already shows); Index-term (keep the mention-expansion; the duplicate list becomes the brush channel); Universe-Dashboard (becomes the explicit idle Estimation-Map + Control-Dashboard state).
- **Engineering riders (from PJ-068 §6):** real mode *state machine* (kills the editor-panels "shadowing" bug); retire the dead `screen:open-note` wire + dead imports; **Rule-8 persisted reads** for all companion data (no `scanLibraryLinks`/`buildSkyData` re-walks); alias-aware neighborhood; hardcoded-English sweep ×15 + shared right-click; re-sync manual/help/CP-26.

---

## 9. What the in-flight G3 build keeps and drops

The uncommitted G3 diff is reconciled to the ratified concept:

- **KEEP — `readOnly` on `NoteEditor`/`NotePane`/`PropertyEditor`** (§1's core): every SS note-view mount is a read-only viewer. This *is* the read-only-always law at the component layer.
- **KEEP — the freshness adopt (§2/§3):** `adoptFreshDiskIntoSS` (main→SS saves) + the `cascade:rewrote` listener keep the SS's contextual views *fresh* (a read-only complement must still show current truth, not stale content). Freshness-gated `externalChange` stays.
- **DROP — the editable toggle + `secondScreenEditable` setting + i18n ×15** (superseded by read-only-always).
- **DROP — §4 cross-window cascade-freeze** (existed only to make *editing* safe; with no SS writes there is nothing to freeze). The two-sided-dirty residual the safety-inspection flagged **vanishes**.

---

## 10. Invariants & risks to avoid (from the prior-art risk list)

1. **Replication trap** — an SS pane mirroring the MS is wasted glass. Enforce span-the-axis.
2. **Self-initiation** — never open itself, grab focus, or swap views the user didn't invoke.
3. **Second domain/editor** — never re-implement save/load/edit, never own a tab lifecycle, no `onNoteSaved` re-reads (mount read-only views; the core editor owns operations).
4. **Rule-8 re-walk** — no scan/rebuild of any derived view on open or focus-change; every panel is a cheap lookup of write-time-maintained data (the canonical OOM/3 GB-WAL disaster this prevents).
5. **Over-reaching drive** — the *only* drive is click-to-navigate; drop every mutate/commit affordance from the prior art.
6. **Hidden/always-on coupling** — the bond is visible + releasable; the one-way path must never let an SS render write back a value the MS re-reads (the `$effect`-echo class, cross-window).
7. **Per-surface redesign sprawl** — one SS, three fixed zones fed by one focus channel; not seven bespoke apps.
8. **MS performance regression** — no `invoke()` on the keystroke hot path to feed the SS; batch/debounce focus pushes (≥300 ms); measure boot + typing latency on a 7,600+ note Universe before shipping (**hard constraint**).
9. **Glance overload** — management-by-exception + compact data-blocks with fly-out detail; keep drill-down shallow.

---

## 11. Open Boss rulings needed (before the `/migration`)

1. **Zone layout:** three fixed zones (Estimation Map / Control Dashboard / Operation Map) — accept the split, and their arrangement (e.g. top locator strip · left health · main operation), or adjust?
2. **The dial:** ship all three positions (Normal/Live/Locked), or start with Normal + Live and add Locked later? Default position?
3. **Idle state:** when nothing is focused (or the MS is on the Dashboard), the SS shows the whole-Universe Estimation Map + Control Dashboard — agreed?
4. **Retire rulings:** confirm retiring Navigator companion, OrgChart mode, and the fallback tab-strip editor (all fail the razor / read-only law).
5. **Scope of first cut:** all seven surfaces at once, or land the editor + Sky + Dashboard trio first (the highest-value complements) and extend?
6. **Sight:** Sight is currently a Wings plug-in (disabled in core) — include its complement in the concept now (build later), or defer entirely?

---

## 12. Migration sketch (after rulings)

Frontend-only — an **event-vocabulary + mode-state-machine redesign**, not a window redesign (all main↔SS coupling is the Tauri `emit`/`listen` vocabulary in `src/lib/secondScreen.ts`; no schema). Rough shape, each phase landable + Boss-testable:

- **P1** — the focus channel + three-zone shell (fixed zones; one contextual `emit` carrying the current focus; the Normal/Live/Locked dial).
- **P2** — the note-editor complement (Operation-Map + Control-Dashboard for a note) reading **persisted** neighborhood/link data (Rule-8); retire the fallback tab editor.
- **P3** — Sky View + Map complements; retire Navigator + OrgChart.
- **P4** — Index + Dashboard/Sight (Estimation-Map + exception strip); Tasks.
- **P5** — click-to-navigate everywhere; the Editor-Surface Gate item 7 (now *read-only* — on-screen === disk after every MS transition, the SS never writes); the two-window Boss test on a 7,600-note Universe (boot + typing latency unchanged).

Standing constraints: Display-Not-Domain; two-monitor gate; never self-initiate; Rule-8 persisted reads; ×15 i18n + RTL.

---

*Basis: docs read — [26-second-screen.md](26-second-screen.md), [PJ-068](PJ-068-Second-Screen-Contextual-Companion-Concept-Paper.md), the Second Screen help topic, the 2026-04-05 "7 SS principles" (SESSION-MEMORY-2026-04-04-06.md), the G3 Architect/Plan. External research: workflow `wf_8b4fdfa4-86d` (Lightroom · Presenter View · DaVinci/Premiere · DJ/DAW · OBS · ops/mission-control · Bloomberg · CAD/GIS · IDE/design detach · PKM multi-window) → synthesis.*
