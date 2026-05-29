# Session log — 2026-05-29

Continuation of the 2026-05-28 MIG-060/061 federation marathon (crossed midnight). Model switched to `claude-opus-4-8`. Eisa set a forward agenda: MoCh → PJ-10/11 → MIG-062 → MIG-063 → MIG-064 → 15-locale help-docs.

## Block 1 (morning) — PJ-10 / PJ-11 federation-scale polish

Two visual issues surfaced once MIG-061 made CNS + Sky View show the full 8 751-node federation.

**PJ-11 — CNS gravity well canvas.** `maxR = min(width,height)×0.45` left big margins on wide monitors. Bumped to `×0.58` + fitToScreen zoom `0.85→0.93`. Stayed circular (no ellipse stretch — Form-Aligns-To-Purpose: the radial layout encodes centrality=distance, library=angle). Boss-verified pass first try. Commit `9a2d9890`.

**PJ-10 — Sky View node size.** Took 3 rounds:
- r1 (`9a2d9890`): count-aware damping `sqrt(1500/count)` — single-universe (≤1500) untouched, federated 8 751 → 0.41×.
- r2 (`62a9a198`): Boss "shrink ~1x more" → exponent 0.85 → 0.22×.
- r3 (`f05fe6f9`): Boss diagnosed the real culprit — "the bubble frame thickness made it big." The node's decorative rings/halos (stratum glow `r+5`, provenance `r+6`, maturity ring, MOC ring) use FIXED pixel offsets that didn't scale with the shrunk fill, so the frame dominated. r3 halves the frames in dense mode (`>1500` nodes) + fill exponent 1.2 → 0.12× at 8 751. Boss-verified pass.

Lesson: when the user says "it's too big" and a size knob didn't fix it, the apparent size may be a DIFFERENT element (here, fixed-offset decorations) — listen to the precise diagnosis ("the frame").

## Block 2 (midday) — MIG-062 P3 filesystem-federation

Approved Architect+Plan (`62eb36b3`, combined lightweight doc). A scoping agent corrected the audit: Tag Browser was never a filesystem problem (it's `allLibraryTags`, federated in MIG-061 §M) — just a reactivity bug. So MIG-062 = 1 reactivity fix + 2 read-only filesystem-federations.

| § | Commit | What |
|---|---|---|
| A | `ca97c38a` | Tag Browser `$effect` — re-sync federated tags (NotebookNavigator) |
| B | `f3d5cdae` | `resolve_child_universe_roots` pub(crate) + recursive variant + 2 cycle-guarded tests |
| C | `130be036` | Five Acts federation (read-only) + `universe_display_name`; `FiveActsNoteEntry.universe_name` |
| D | `c41d52cf` | Workspace Bases federation (read-only `scan_bases_dir`, no create into cUniverses) |
| E | `56cfa153` | Frontend per-universe collapsible grouping; cUniverse bases open-only |
| E.2 | `5b02f3ca` | Hide system `Five Acts/` folder from library trees (recursive) — Boss option "A" |
| F | `9fe0b2ef` | Boss-test doc |

**INV-1 (read-only) enforced:** both backend commands only `fs::read_dir` cUniverse paths; cUniverse bases have no delete menu; the Five Acts folder-hide is display-only (files on disk). Detach is lossless — Eisa's "the wheel is already there" principle.

**Boss-test (live, Eisa Universe + Eisa Cognitive Knowledge cUniverse):**
- Five Acts grouping — ✅ (cUniverse collapsible group under top section).
- Workspace Bases grouping — ✅.
- Tag Browser §A — reachable only via Notes Navigator mode Eisa doesn't use → §A is correct but for an unused surface; the REAL tag browser became a new feature request.
- §E.2 Five Acts folder-hide — ✅ verified in BOTH contexts (federated: folder gone from cUniverse tree, Observation in top section; switched-to-active: folder gone from tree, Observation in top section by default — the caveat).
- Standalone integrity — ✅ structural (Eisa switched universes; data intact).

848→852 lib tests pass (the 2 new §B recursion tests). svelte-check: 3 pre-existing errors only throughout.

## Test counts

- 852/852 lib tests pass.
- 10/10 cache federation tests (MIG-061 §G/§Q).
- 84/84 lens tests.

## New feature queued (task #12)

Eisa: "I want a real universe-wide tag browser." Current tag surfaces: right-sidebar per-note panel (open-note only) + the undiscoverable Notes-Navigator universe-wide list. Queued as a new feature with its own Architect (placement, access, click-to-filter, RTL, 15-locale). `allLibraryTags` already federated.

## §G PCS (this block)

Orientation v2.42, this session log, MoCh-2026-05-29, English Federation help topic (14 translations queued), milestone tag `milestone/mig-062-filesystem-federation-shipped`, ZIP backup, push all unpushed commits.

## What's next

Build the universe-wide Tag Browser (task #12) — Architect first. Then MIG-063 (P2 read-paths), MIG-064 (P2+P4 write-paths).

---

## Tag Browser shipped (task #12) — PCS

The universe-wide Tag Browser — the "new feature queued" from the MIG-062 block — built and Boss-tested across every sub-feature.

| Commit | What |
|---|---|
| `ae180dbc` | Lightweight Architect (`docs/TAG-BROWSER-ARCHITECT.md`) — placement / access / click-to-filter / RTL / 15-locale |
| `6d6bc2b7` | Universe-wide federated tag tree in the right-sidebar Tags tab (`This note | All tags` toggle; reusable `TagsPanel` fed by federated `allLibraryTags`) |
| `fbda7f86` | Render the Tags tab without an open note (pulled out of the `{#if isHome && sidebarTab}` gate to a top-level branch) |
| `f80956ee` | Polish (Boss remarks) — right sidebar 340→380 px; live total counter; `.rs-tags-body` scroll region; padded header |
| `44325ad3` | Sort modes — A→Z / Z→A / by count (`#`, alphabetical tie-break), recursive across every tree level |
| `e5b56c98` | Freeze sort + filter bar while scrolling — `.tp-controls` `position: sticky; top: 0` + full-width opaque bg |

**Architecture footprint:** reused existing pieces end-to-end — `allLibraryTags` was already federated (MIG-061 §M), `TagsPanel` already existed, `handleTagClick` already routes to the federated Search Hub. Net new backend: **zero** (no IPC, no schema, no federation wiring). Frontend: the toggle + gate fix in `+layout.svelte`, sort + sticky bar in `TagsPanel.svelte`.

**Boss-test (live, Eisa Universe, 21 068 distinct tags):**
- All-tags tree renders federated, click-to-filter → Search Hub — ✅.
- Width / counter / scroll / header padding — ✅ (after the polish round).
- Sort A→Z / Z→A / # — ✅.
- Header + sort + filter freeze on scroll, only tags move — ✅.

**Federation scorecard unchanged at 8/14** — the Tag Browser was already counted closed in v2.41 (§M) / v2.42 (§A, navigator path); this is the discoverable front-end surface for it, not a new closure.

## §PCS (this block)

Orientation **v2.43** (Tag Browser shipped; v2.42 §"queued" superseded), this session log, `MoCh-2026-05-29-1400`, Federation help topic sentence (All-tags toggle + sort + freeze; 14 translations covered by batch #13), milestone tag `milestone/tag-browser-shipped`, push all unpushed commits, ZIP backup.

## What's next

**MIG-063** (P2 read-paths: Index entries/mentions, Knowledge Health, Unlinked Mentions, right-sidebar previews, Org Chart alias-map) — begins on explicit Eisa go. Then MIG-064 (P2+P4 write-paths). Clean stopping point reached.

---

## State of standing — Constellation Base (asked 2026-05-29, post Tag Browser)

Eisa: *"Where are we standing regarding Constellation Base?"* — read the full Base doc set in `docs/` (HANDOVER-bases-review-2026-05-25, help Bases.md, Concept-Paper v1.4, MIG-055 Architect, MIG-060 Architect) + cross-checked orientation v2.43 body/changelogs. Snapshot per SO #5:

**Crux: there are TWO "Bases" coexisting.**
- **OLD MVP** (`bases.rs`, commit `c5b05f5c` 2026-03-12) — generic Obsidian-Bases clone: auto-detect YAML keys → table/card/list, filter/sort builders, in-place cell edit. 10 IPC commands, `BaseView.svelte` family. Sidebar section **"Workspace Bases"**. Still live, still user-visible, **federated this session** (MIG-062 §D read-only `scan_bases_dir`).
- **NEW lens system** (Concept-Paper-v1.4 vision) — "knowledge lens parameterized by curated cognitive dimensions," NOT free-form YAML. Sidebar section **"Five Acts"** (sits above Workspace Bases). This is the trajectory that survives.

**(a) Verified shipped & protected**
- **Phase 1 — MIG-055** (clean rebuild, 2026-05-26, orientation v2.37 "live"): `lens/` Rust module; dimension registry (4 dims: `note.name`/`note.headline`/`note.created_at`/`note.path`); `execute_lens` Tauri command, SQL against `note_meta` (write-time, **Rule-8-compliant**); `build_sql`; 25 tests. `LensBlock.svelte` CM6 widget renders inline ` ```base ` blocks. "Five Acts" sidebar + `list_five_acts_notes`. One shipped template: **"Observation — Recent Captures"**. 15-locale.
- **MIG-056** cross-universe federation: `execute_lens` federates across cUniverses (single-schema fallback if federation not ready).
- **Phase 1.5 — MIG-060** (2026-05-28, orientation v2.40 "ships"): three threading gestures per lens row (Open in 360.3D / CNS / Cataloger), `constellation:open-note-in-surface` event.
- OLD Workspace Bases: federated read-only (MIG-062 §D), per-universe grouping (§E), `Five Acts/` system folder hidden from trees (§E.2).

**(b) In-flight / uncommitted:** none for Base — all Base-adjacent work this session (MIG-062 §D/E/E.2) shipped + pushed.

**(c) Known broken**
- **Cataloger threading gesture** (MIG-060 Stage 4): `FOREIGN KEY constraint failed: sources_suggestions(note_path) → note_meta(path)` on orphan/federated notes. Pre-existing Cataloger federation gap → **MIG-064**.
- **OLD `query_base` is a Rule-8 violation** (read-time live filesystem scan) — the headline concern from the 2026-05-25 handover. MIG-054's fix attempt was reverted; the NEW lens system is Rule-8-clean, but the OLD Bases still scans live. Unaddressed.

**(d) Pending / not started — the roadmap stalls after Phase 1.5** (federation work MIG-056→062 consumed the MIG numbers the Base roadmap expected). No MIG/commit yet for:
- Phase 2 — Living Link columns · Phase 2.5 — CE/360.3D dimensions · Phase 2.6 — CNS measurements (+ freshness-strategy decision) · Phase 2.7 — CECE/Cataloger classifications.
- Phase 3 — the other 4 Five Acts templates (Connection / Tension / Synthesis / Conviction); only Observation ships.
- Phase 5+ — user-composed lenses (build-a-base UI), table/card view shapes (only `list` ships), aggregations, NL→lens.
- Old-MVP cleanup MIG (remove dead `BaseView` family) — deferred housekeeping.

**(e) Doc drift**
- Orientation v2.43 **body** §4.x (≈ lines 4736/4856/5108) still describes `bases.rs` as "the" Bases ("**5 commands**" — actually **10** per the handover; "read-time Rule-8 violation") and does NOT carry the new lens system into the canonical-state body (lens system lives only in the v2.37 changelog preamble). 5→10 drift flagged 2026-05-25, still open.
- Help `docs/help.uConstellation.World/Bases/Bases.md` documents ONLY the OLD MVP (`.base` as **JSON**, table/card/list, generic filter ops). No mention of the Five Acts lens system, `LensBlock`, threading gestures, or the YAML lens schema. User-facing help is for the legacy feature, not the current direction.

**Bottom line:** the *new* Constellation Base (Five Acts lens system) is **shipped through Phase 1.5** and federated — a working thin vertical (one template, list view, 4 dimensions, 3 threading gestures). Phases 2–7 (the cognitive-dimension columns that make it "Constellation's, not Obsidian's") are **designed (v1.4) but unbuilt**. The legacy MVP Bases still ships alongside it. No Base work is in flight; next Base step would be a fresh `/migration` for Phase 2 (Living Link columns) — Eisa's call.

---

## Constellation Base — Dual-World: research + Architect + Plan (MIG-065)

Eisa's direction: *"have both worlds"* — the familiar Obsidian-style Base + the PKF-powered Constellation Base, unified. Chose **Model 2 (one unified Base, progressive depth)**; governing principle locked: **"Strong yet Simple, by default."** Then: *"I like your analysis, I trust your judgment, proceed as you see fit"* + *"do the necessary research."*

**Research (WA #5, 2 sourced web agents):**
- Obsidian (Bases/Dataview/Datacore) — `.base` is YAML (`filters/formulas/properties/views`); columns = ordered list inside each view, prefix-namespaced (`note.`/`file.`/`formula.`); formula columns exist. Their power-path is a CLIFF (GUI→DQL→React) — users run two engines forever. **Invert it: power = "add a column," never "learn to code."**
- Notion/Airtable/Coda — progressive disclosure inside ONE object (not a mode switch); Notion tiers types Basics→Organizers→Power Tools; computed columns read-only + visually marked; Notion's OWN docs name read-time formula/rollup cascades as the thing that stalls past 2–3k rows → **our write-time derivation (Rule 8) is the competitive advantage**; relations already ours (typed links exist).

**Grounded code map (Explore, commit 731e06cf):**
- NEW lens engine (`lens/`): `execute_lens(app, lens_yaml)→LensResult`, registry of 4 dims (note.name/path/created_at/headline), `build_sql`+`build_federated_sql`, federated-then-fallback. `view: List` only. Renderer = `LensBlockWidget` in `livePreview.ts` (+ MIG-060 threading buttons).
- OLD MVP (`bases.rs`, ACTIVE): 10 cmds; `query_base` = **live filesystem walk** (Rule-8 violation); `BaseView`+Table/Card/List+Filter/Sort builders (6 components exist).
- **Cheap dims available NOW:** Living Links (note_links: confidence/weight/link_type/traversal_count) ✓; Epistemics (note_meta.sources + content_type, MIG-021) ✓; headline (note_summaries) ✓; word_count ✓.
- **NOT persisted (needs WTD first):** stratum/maturity/provenance — compute-on-demand from scanners. → deferred to MIG-068 w/ its own derivation step.
- **BLOCKER (R1):** familiar table needs arbitrary frontmatter cols from `note_meta.properties_json`, but the lens engine currently avoids it over a flagged `parse_frontmatter` bug → MIG-065 §B must verify/repair first.

**Decisions locked (Eisa, "proceed as you see fit"):** (1) YAML `.base`, old JSON ignored; (2) both standalone files + inline blocks; (3) curated picker + finite aggregations, no formula language v1; (4) one engine (extend `execute_lens`, retire `query_base`); (5) MIG-065 = Simple foundation, then 066 Links / 067 Epistemics; (6) Concept Paper v2.0 reconciliation first. ★ Strong yet Simple by default.

**Artifacts:** `docs/MIG-065-constellation-base-unified-ARCHITECT.md` (dual-world umbrella) + `docs/MIG-065-constellation-base-unified-PLAN.md` (steps §A–§L, gates A→B-blocker→…→K-Boss→L). **Status: awaiting Eisa's Plan approval (the one Migration gate before code).** MIG-063/064 (federation read/write paths) remain queued but Base is now the active workstream.
