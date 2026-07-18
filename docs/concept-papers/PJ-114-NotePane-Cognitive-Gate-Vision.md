# NotePane as the Gate to Knowledge Cognition — Vision & Prioritized Roadmap

*PJ-114 · decision-ready synthesis for the Boss · 2026-07-18 · dig `wf_cfffaf11-0b3` · grounded on the Capability Audit (`PJ-114-NotePane-Capability-Audit.md`) + a source-level inventory · read-only, no code written*

> **Governing principle (Boss, 2026-07-18):** NotePane is the GATE to knowledge cognition — the key to one's knowledge, the start of the cognitive journey. Constellation's power is realized *through* NotePane; it must be well-equipped and well-instrumented.

---

## 1. The concept & principles

**The concept (the horse).** NotePane is the one surface where a thought becomes *connected* knowledge — where the writer both composes an idea and reasons about its relationships *without leaving the flow of writing*. Every other surface (Sky, Map, Sight, Index, Reviewer, Inspectors) only *reads* what NotePane *authored*. It is not "a text editor with panels bolted on" — it is the instrument through which all Five Acts are physically performed on one page.

**The one-line test.** A capability belongs in NotePane's gate **iff it lets the *writer* advance one of the Five Acts — Observe, Connect, hold Tension, Synthesize, Commit — in the flow of writing, using an engine we can already reach (or one small, justified new one).**

**Five principles:** (1) instrument every Act; (2) surface the living link *where you write*, not only in side panels; (3) make the invisible visible — type/reasoning/confidence/life legible at a glance; (4) wire, don't reinvent; (5) never tax the keystroke, never clutter the paper (write-time-derived, progressive disclosure: glance → inspector → panels). Editor-Parity, RTL/×15, and cross-platform ride along by construction. FM+ is a **later, complementary** surface — NotePane owns the full treatment; FM+ fills gaps against it, never duplicates.

---

## 2. The vision, by the cognitive journey (Five Acts)

Tags: **[have]** exists+surfaced · **[wire]** engine exists, needs connecting · **[build]** genuinely new.

**Act I — Observation** (capture + restate): frictionless capture (Focus + NotePane), instant typing **[have]**; provenance-at-entry **[wire]**; fleeting→permanent holding state **[build, low priority]**.

**Act II — Connection** *(the most-broken Act — center of gravity)*: author-a-typed-link-while-typing **[have]**; **re-type an existing link's kind** from editor/rows **[wire]** (`LinkTypePicker`); **edit a link's reasoning** **[build-small]** (no `setLinkAnnotation` IPC today); suggested connections + local graph **[have]**; body + backlinks + outgoing **co-visible** **[wire, layout]**.

**Act III — Tension** *(what separates formulation from note-taking)*: `contradicts` kind + `contested` confidence **[have set]/[wire]** (make it fluent + a badge); a per-note tension surface as a *destination* + a quiet **inline cue** **[wire]**; Claim/Evidence/Warrant capture (Toulmin) **[build, concept-paper]**.

**Act IV — Synthesis**: structural spine (breadcrumb + outline) **[have]**; compose with sources visible + auto-linked-back **[wire, layout]**; progressive summarization mode **[build]**; the 360° dossier **[have]**.

**Act V — Conviction**: stage promote/demote **[have]**; confidence made **visible** as a badge **[wire]**; **link state at a glance** (weight-tier, stage, last-traversed) **[wire]**; **per-link inspector — all 8 properties** **[wire]**; `supersedes` gesture **[wire]**; spaced resurfacing of decaying/contested links **[wire]**.

**Honest scorecard:** of 8 link properties → **1 settable** (Confidence), **2 raw-syntax-only** (Type, Annotation), **5 auto/near-invisible**. The whole roadmap closes that from the writer's chair — ~90% wiring.

---

## 3. The prioritized roadmap

Ordered by *(cognitive leverage × low cost)*, reuse-first; most-broken Act first.

- **Phase 0 — Truth in the instrument** *(hours; ships with Phase 1)* — remove/wire the 5 dead editor-menu items (Copy target, External link, Footnote, Math, Select all) + 3 dead toolbar align buttons. A lying affordance poisons trust in every real one.
- **Phase 1 — Own the Living Link** *(Boss-picked; the foundation)* — converts NotePane from a Markdown-link editor into a **living-link editor**:
  | # | Capability | Act | Work |
  |---|---|---|---|
  | 1.1 | Re-type an existing link's kind (editor right-click + panel rows) | Connection | **[wire]** drop `LinkTypePicker` into `EditorContextMenu` + rows |
  | 1.2 | Edit a link's annotation/reasoning (inline + panel) | Connection | **[build-small]** the *one* new engine: `setLinkAnnotation` IPC + field |
  | 1.3 | Link state at a glance (confidence badge + weight-tier + last-traversed) | Conviction | **[wire]** render existing lifecycle calculators |
  | 1.4 | Per-link inspector — all 8 properties | Synthesis | **[wire]** assemble already-computed values |
  **Exit:** every one of the 8 link properties is either settable or visible on the note surface. **Runs under `/migration`** (item 1.2 crosses Rust↔Svelte↔LINK-file↔index); `safety-inspection` diff-scoped before commit.
- **Phase 2 — Make Tension First-Class** *(Act III)* — `contradicts`/`supersedes` authoring as fluent as `supports`; the tension signal as a quiet inline cue; `contested` at a glance. Mostly **[wire]**; highest differentiation-per-effort after Phase 1.
- **Phase 3 — Read the note inside its web** *(Acts II+IV)* — end the one-tab-at-a-time limit: body + backlinks + outgoing + structure **summoned co-visible** (progressive disclosure, not a permanent cockpit); frontmatter typed links shown as living links. **Own `/migration`** (sidebar/tab architecture).
- **Phase 4 — Connection discovery, deepened** *(Act II)* — unlinked mentions → one-click **typed + confidence** link; semantic "find related" copilot in-pane (embedding engine exists). **[wire]** + real UX design.
- **Phase 5 — Distillation, synthesis & temporal cognition** *(Acts IV+V; highest ceiling, most new build)* — progressive summarization; highlight→extract-to-claim with a `derives-from` link back; graph-aware AI synthesis ("what have I concluded about X?"); weight-decay staleness + resurfacing. Each item = its own concept paper + `/migration` + WA#5 cross-check.

---

## 4. What makes this uniquely Constellation

The moat is the cluster **no mainstream tool instruments at the editor level:**
- **Typed *epistemic* relationships as a guided gesture** — the 8 cognitive kinds force "*how* are these related?" (filing → reasoning); re-typable from the surface. (Obsidian/Roam links are untyped.)
- **Reasoning-on-the-link (annotation)** — a bare edge is data; an annotated edge is an *argument*. Near-unique.
- **Doubt & stance instrumented at the editor** — hypothesis→evidence→established→**contested**, legible + revisable in flow. No mainstream PKM does this — open lane.
- **The living link's lifecycle made visible** — weight earned by traversal, 5%/month decay, stage, last-traversed; *neglect made visible*.
- **Tension held open, not filed away** — a `contradicts` link + a tension surface that *collects* disagreement is formulation's signature move.

---

## 5. Decisions — BOSS-RULED 2026-07-18 ✅

- **D1 → (b) STRETCH IT.** Phase 1 = the four picks + Phase-0 housekeeping **+ the `supersedes` gesture + the confidence badge** folded in from later phases.
- **D2 → (c) BOTH, at three depths.** Inline right-click on the `[[token]]` (fast glance gesture) · panel rows (aggregate) · per-link inspector (deep, all 8).
- **D3 → (a) EARLY.** Tension (Act III) stays **Phase 2**.
- **D4 → MINIMAL BY DEFAULT, RICHER BY CHOICE VIA SETTINGS.** *(Boss refinement — better than either offered option.)* Rows show the glance (type pill + confidence badge) by default; a new **Settings** option lets the user turn density up (weight-tier, last-traversed, etc.). Build the density as a setting, not a hard-coded choice.
- **D5 → deferred to Phase 3** (co-visibility: summoned, not a permanent cockpit — recommendation stands, decide at that phase).

### Original options as presented

- **D1 — Phase-1 ambition:** (a) exactly the four picks + Phase-0 housekeeping, or (b) + fold in `supersedes` + confidence-badge from later. **Rec (a)** — keep the one new engine isolated, migration small + cleanly auditable.
- **D2 — Inline vs panel for the living-link controls:** (a) inline on the token only, (b) panels only, (c) **both** at three depths. **Rec (c)** — inline = fast glance gesture; panels = aggregate; inspector = deep single-link.
- **D3 — Surface Tension (Act III) early or defer:** (a) Phase 2, (b) push to the end. **Rec (a) early** — the Act that most distinguishes formulation, almost pure wiring.
- **D4 — How much "at a glance":** (a) minimal (type pill + one confidence badge on the row), (b) rich (pill + confidence + weight + last-traversed). **Rec (a)** — row = glance; inspector = all 8. Don't turn a row into a dashboard.
- **D5 — Co-visibility (Phase 3):** (a) permanent multi-pane cockpit, (b) **summoned** arrangement. **Rec (b)** — a permanent cockpit buries writing; summon it at synthesis moments.

**The single new engine in all of Phase 1:** `setLinkAnnotation` IPC + a field. Everything else reuses `LinkTypePicker`, `ConfidencePicker`, `getLinkStage`/`linkLifecycle`/`effectiveLinkWeight`, `archiveLink`, the panels, `LinkTypePill` + the `×N` chip. Predecessor-Lookup + exact call-sites verified at build time.
