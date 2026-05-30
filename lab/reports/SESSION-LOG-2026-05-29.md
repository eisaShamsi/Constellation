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

---

## MIG-065 build cascade — §A–§F shipped (backend complete; §F frontend table → Boss test)

Eisa approved the Plan ("Go"). Cascading; each § a commit with verification.

| § | Commit | What | Verify |
|---|---|---|---|
| A | `d8af1d5c` | Concept Paper v2.0 (Dual-World reconciliation; §5.0 Strong-yet-Simple; §3 refusal reframed; roadmap → MIG-065+) | v1.4 preserved |
| B | `5197749e` | `properties_json` faithful for **scalar** columns (characterization tests; RTL/empty/quotes/colons). Multi-line list/nested **dropped** — deferred parser upgrade (PJ). | 6 tests; blocker cleared for v1 scope |
| C+D | `a89fc1a9` | Engine: `LensView::Table`; `prop.<key>` frontmatter columns via `resolve_dim` → `json_extract`; Text filters (is/contains/…); federated symmetry. | 94 lens tests |
| E | `3c411031` | `discover_base_properties` command (federated json_each); **fix**: materializer used `lookup_dimension` (None for prop.*) → `resolve_dim`. | 97 lens tests |
| F | `76de5ed7` | `LensResult.view`+`columns`; `LensBlockWidget._renderTable` — inline ` ```base view:table ` renders the familiar table (clickable name + declared columns incl. prop.*; RTL; created_at→date). | cargo check clean; tauri build OK (exe 19:44:47) |

**Decisions/notes during build:**
- **`prop.` prefix** chosen over the Architect's separate `property:` key — same user outcome, zero struct churn, aligns with the prefix-namespace research. Logged in §C+§D commit.
- **Deferred (PJ candidates):** faithful list/nested `properties_json` (needs re-index); standalone `.base`-file-opens-as-table tab routing (remaining half of §F — inline block ships first); i18n keys `lensBlock.col*` (English fallbacks for the test; 15-locale at §L).
- **Build infra:** recurring Windows-Defender LNK1104 lock on the debug test binary; rode past with a background retry-with-backoff loop. `cargo check --tests` (no link) used to surface real compile errors cleanly.

**Pending:** §F Boss-test (Stage 1) → then §G (picker), §H (edit-in-place), §I (retire query_base), §J (audit), §K (staged Boss test), §L (PCS).

**§F Boss-test — PASS** (2026-05-29 ~20:10 build). Table renders (15 rows); headers Name/Summary/status (raw-key defect fixed, `d3f9f3c3`); count badge accent+white; RTL note names right-align (cell-level dir). Polish commit `d3f9f3c3`.

**Checkpoint (post-§F):** MIG-065 §A–§F shipped + committed (backend fully test-covered; inline `base view:table` familiar table Boss-validated). Remaining: §F.2 standalone `.base`-file-as-table tab routing (deferred from §F); §G add-column picker (couples to the full-tab table view — best done with §F.2); §H edit-in-place; §I retire `query_base`; §J audit; §K staged Boss test; §L PCS. All MIG-065 commits local (push at §L). Resumable: this log + per-§ commits are the trace.

---

## MIG-065 §F.2 — standalone `.base` file → full-tab table (resume session, 2026-05-29 PM)

**Function in hand:** Constellation Base — the Unified Progressive Base (MIG-065). Governing principle: *Strong yet Simple, by default.*

### Predecessor → Replacement (Predecessor Lookup Rule — written BEFORE any code edit)

- **Where it lives now.** A `.base` file open routes through `openNoteTab()` (`src/lib/libraries/store.ts:1186`; the title-stem strip at :1222 already handles `.base`) and mounts **`NoteEditor`** on the `.base` path — i.e. a `.base` opens as a *plain note*. The old MVP UI family — `BaseView.svelte` + `BaseTableView/BaseCardView/BaseListView/BaseFilterBuilder/BaseSortBuilder`, on `$lib/bases/store.ts` (`queryBase`/`saveBaseFile`/`updateNoteProperty`) + `$lib/bases/types.ts` (`BaseDefinition`/`BaseRow`/`ColumnDef`) — was built (MVP, commit c5b05f5c) but **NEVER wired into tab routing**: `BaseView` is referenced only by its own children (verified by grep). Orphaned.
- **Where its replacement lives — same place.** The two `NoteEditor` mount points in `src/routes/+layout.svelte` (split mode ~L5996, single mode ~L6117) gain a `.base` branch that mounts a NEW **`src/lib/lens/BaseTab.svelte`**, fed by the surviving engine `execute_lens` (`$lib/lens/store::executeLens`).
- **What's cut / kept.** §F.2 cuts **nothing user-facing** (the orphaned `BaseView` was never mounted). `query_base` + the rest of `bases.rs` stay until **§I** retires them. `update_note_property` is **KEPT** — reused for §H edit-in-place.

### Intentional deviation from Plan §F (logged, like the `prop.` prefix deviation)

Plan §F says "mounting the **reused** `BaseTableView.svelte`." Build reality made literal reuse the *wrong* kind of reuse:
1. `BaseTableView` consumes `BaseRow` (`properties: Record<string,string>`) + `ColumnDef` from `$lib/bases/types` — **the very types §I retires.** The surviving engine returns `LensRow` (`dimensions: Record<string,DimensionValue>`) + `columns: string[]`. Reuse would need an adapter + couple the NEW Base to a deleted type.
2. `ColumnDef` has no read-only/computed flag → §G (picker marks cognitive columns read-only) + §H (edit guards) would need it extended — on a retiring type.
3. `BaseTableView`'s first column header is a hardcoded English `"Name"` (line 152) → violates invariant #6 (15-locale, day one).
4. The genuinely **Boss-validated** renderer is the inline `_renderTable` (`livePreview.ts`), NOT the orphaned `BaseTableView`.

**Decision (Option B): secure the *right* winning.** Extract the validated render logic (`dataColumns` / `columnLabel` / `renderCellValue`) into a shared, pure **`src/lib/lens/tableModel.ts`** — one source of truth imported by *both* the inline CM6 widget and the new `BaseTab.svelte`. `BaseTab` is built on the surviving `LensResult` contract → same user outcome (familiar editable table), clean path through §G/§H, zero coupling to retiring types. CLAUDE.md "don't duplicate working code → extract a shared component" is honored by `tableModel.ts`. *(Boss informed in-chat; identical user outcome, internal scaffolding choice — proceeding per Plan-Approval-Equals-Build-Approval.)*

### §F.2 / §G regrouping (commit hygiene)

Resize/reorder, column add/remove, and filter/sort **all** require the `.base` `columns:` rewrite-and-save path. Building half of it in §F.2 then replacing it in §G is waste. So: **§F.2 ships the read-only familiar full-tab table** (the Simple default surface in a tab); the `columns:` save path + resize/reorder + add-column picker land together in **§G**; edit-in-place in **§H**. (Plan deliverables unchanged; only the per-commit grouping shifts — the §-boundaries are my decomposition tool.)

### Open-note path
`BaseTab` dispatches the existing `constellation:open-note` CustomEvent (listener at `+layout.svelte:2316`, resolves library by name then path-prefix) — the SAME path the Boss-validated inline `_renderTable` uses. One open-note path for every base/lens surface.

### §F.2 build — landed (commits, cap, fixtures)

| Commit | What | Verify |
|---|---|---|
| `97a8b52d` | `tableModel.ts` (shared) + `BaseTab.svelte` + livePreview refactor (uses tableModel, deleted dup `_labelFor`/`_renderCellValue`) + `.base` routing at both `NoteEditor` mounts in `+layout.svelte`. | svelte-check clean for all 4 files (3 remaining errors are pre-existing: `store.ts:2483` LinkLifecycle.fresh — known deferred; `PropertyEditor.svelte` ×2 — untouched). |
| `85793fdd` | **Render cap (Perf Rule 3).** `execute_lens` has NO SQL LIMIT (`total_count == rows.len`); over 7,651 notes an unscoped base would render thousands of un-virtualized rows → BaseTab caps at **500** + honest `lensBlock.rowCap` footer ("showing the first N of total"). en+ar added; 13 locales fall back to en. | svelte-check clean. |

**FOLLOW-UP (logged, PJ candidate):** proper row **virtualization** + an **engine-side `LIMIT`/separate `COUNT(*)`** split (so `execute_lens` doesn't materialize/IPC-transfer all 7,651 rows). The 500-cap is a Simple-default stopgap; the real fix lands with §G's engine work or a dedicated step.

### Boss-test fixture + the active-universe finding

- **Anomaly found:** the release registry (`%APPDATA%/world.uconstellation.app/universes.json`) marks **"كون عيسى"** active (4 notes, no `search.db`). But the universe actually worked in is **"Eisa Cognitive Knowledge"** (`E:\Constellation Universes\Eisa Cognitive Knowledge`) — `search.db` 1.77 GB, **7,651 notes**, mtime **2026-05-29 20:16** (the §F test window). No registry lists "Eisa Cognitive Knowledge" — tracked by neither the `world.uconstellation.app` nor the old `com.notesconstellation.app` registry. *(Unresolved why; not blocking — flagged for a later look.)*
- **Resolution (no blocking question):** staged the test base in **both** universes' `.constellation/bases/` — harmless YAML view files — so whichever the app opens, it's in the Workspace Bases sidebar. `scan_bases_dir` lists any `.base` (JSON-parse fails on YAML → name falls back to the filename stem), so it shows regardless.
- **Fixture:** `My Notes — overview.base` — minimal valid table lens (no where/order → zero validation risk): `columns: [note.name, note.created_at, prop.created]`. Verified read-only against the real DB: all 3 columns populate; names are Arabic (RTL cell test); `created` frontmatter on 7,646/7,651. Over 7,651 → caps to 500 + notice.

**Pending:** rebuild (must include the cap) → verify binary mtime (Stage 0) → Stage-1 Boss test (open the base, see the familiar table). Then §G.

### §F.2 Stage-1 Boss test — **PASS** (2026-05-29 ~21:08, build mtime 21:05:56)

Eisa opened "My Notes — overview" on **Eisa Cognitive Knowledge**: full-tab table rendered, count badge **7651**, footer **"Showing the first 500 of 7651 rows"** (the cap works), Arabic note names right-aligned (RTL cell test), query **415 ms**. §F.2 validated. Binary mtime (21:05:56) post-dated the cap commit (20:58:20) — Stage 0 confirmed fresh.

### Design insight from the Boss test → shapes §G (the add-column picker)

Eisa's question: *"why two columns both titled Created/created? Is the lowercase one the cid_cn?"* — verified answer: the lowercase **`created`** is the note's **frontmatter `created:` field** (`prop.created`), NOT the cid_cn. It repeats `2026-04-14T09:22:41.795Z` because those notes were **bulk-imported in one batch** (7,587 of 7,651 share that timestamp; only 60 distinct values). The **cid_cn is a separate, unique-per-note field** (`20260414T092241Z_NOTE_24A7` … 1,000+ distinct) — *built from* the same created-timestamp + a unique suffix, which is why they look related. The capital **"Created"** was `note.created_at` (a registered Constellation dim → friendly date); the demo just picked two overlapping date fields — a confusing fixture choice. Fixture swapped to `[note.name, prop.stage, prop.maturity, prop.source]` (fields that actually vary: spark/birth/growth · seed/sapling/evergreen · Wikipedia/ويكيبيديا).

**→ §G requirement (locked by this):** the add-column picker (and ideally the column headers themselves) MUST visually distinguish **"Your fields"** (frontmatter keys, raw names) from **"Constellation fields"** (registered dims, friendly labels, marked computed/read-only) — the Notion/Airtable tiered pattern + Concept Paper §4.4. Eisa's "two Createds" is the canonical motivating case: in the picker they'd sit in different sections ("Created — Constellation" vs "created — your field"), so the overlap reads as intentional, not a bug. Consider a subtle per-column-header marker (icon) for frontmatter vs Constellation columns so the distinction survives after the column is added.

---

## MIG-065 §G — the "+ Add column" picker (build plan + decisions)

**Function in hand:** §G — the tiered add-column picker. The literal embodiment of "power = add a column, never learn to code."

**Scope (tight, landable, Boss-testable):** add-column picker (tiered: **Your fields** = `discover_base_properties` / **Constellation fields** = registered dims, marked read-only) + **remove column** + the **`.base` `columns:` save+reload path**. **Filter/sort builders + resize/reorder are deferred to §G.2** (each its own commit; keeps §G focused on the thesis). Logged regrouping; Plan deliverables unchanged.

**New IPC (additive — Predecessor note):** `lens::query::update_base_columns(app, file_path, columns: Vec<String>) -> Result<String,String>`. Round-trips the `.base` through `parse_lens_yaml` → sets `columns` → `validate` → `serde_yaml::to_string` → write; returns the new YAML so the tab re-renders without a separate read. Security: reuses `bases::validate_base_path` (made `pub(crate)`) — universe/library-scoped; rejects non-`.base`, empty columns, and unresolvable dims. Registered in `lib.rs`. *The old MVP's column save (`save_base_file`, JSON, via the orphaned `BaseView`) is NOT touched — it dies with `query_base` at §I.*

**Reload mechanism:** BaseTab keeps a local `yaml` state seeded from the `content` prop (a `$effect` resyncs when the prop changes — Rule-2-safe: reads `content`, writes `yaml`, never reads `yaml`). Add/remove sets `yaml = <returned YAML>` → the query `$effect` re-runs `executeLens`. The parent tab's cached `content` goes briefly stale (only consumer is BaseTab) — acceptable; refreshed on reopen.

**Constellation-fields source:** frontend `tableModel.ADDABLE_REGISTERED_DIMS = [note.created_at, note.headline, note.path]` (mirrors `dimensions.rs`; `note.name` excluded — always col 1). A `list_base_dimensions` command can replace this when MIG-066+ grow the registry (logged).

**Files:** `bases.rs` (pub(crate) validate_base_path) · `lens/query.rs` (+update_base_columns) · `lib.rs` (+register) · `lens/store.ts` (+discoverBaseProperties, +updateBaseColumns) · `tableModel.ts` (+ADDABLE_REGISTERED_DIMS) · `BaseColumnPicker.svelte` (new) · `BaseTab.svelte` (integrate) · en+ar.

**i18n namespace correction:** keys landed under `lensBlock.*` (NOT a new `base.*` namespace) — consistent with the existing `lensBlock.colName`/`rowCap`; avoids a confusing `base` vs the old MVP's `bases` namespace. Keys: `addColumn`/`removeColumn`/`searchFields`/`yourFields`/`constellationFields`/`readOnly`/`allFieldsAdded` (en+ar; `constellationFields` ar = "كوكبة" per the full-localization principle; 13 locales at §L).

**§G build + Boss test — PASS** (commit `eef4d433`; build mtime 21:51:18 > commit 21:43:11). Eisa on **Eisa Cognitive Knowledge**: opened the base (Name·stage·maturity·source), opened the picker → **two sections confirmed** (Your fields / Constellation with read-only tags), added `library` (Your fields) + `Created` (Constellation), removed `source`, **persisted across reopen**. The saved `.base` round-trip is correct + canonical: `columns: [note.name, prop.stage, prop.maturity, prop.library, note.created_at]`, with `scope`/`where: []`/`order: []`/`view: table` serialized out (the expected fuller form from the LensDefinition round-trip). The "two Createds" confusion is resolved — `Created` is unambiguously under *Constellation*. Eisa: *"Wow, I am impressed! Good job."*

**Round-trip note (expected, not a bug):** `update_base_columns` re-serializes the whole `LensDefinition`, so a hand-minimal `.base` becomes the fuller canonical form (scope/where/order emitted) after the first column edit. Semantically identical; re-parses + queries fine. Comments in a hand-edited `.base` would be lost on a UI column edit — acceptable for a machine-managed view file (Obsidian `.base` parity); logged in case we later want a comment-preserving edit.

**Next:** §H (edit-in-place on `prop.*` cells; registered/Name read-only) → §G.2 (filter/sort builders + resize/reorder, deferred from §G) → §I (retire `query_base`) → §J (audit) → §K (staged Boss test) → §L (PCS).

---

## MIG-065 §G.2 — column sorting (Boss asked mid-§G; he chose "Header + multi-sort")

**Trigger:** after §G PASS, Eisa asked *"Are we going to add a sort function for each column?"* → AskUserQuestion → he chose **"Header + multi-sort"** (click-header as the Simple default + a multi-sort layer). Sorting was the part of §G I deferred; bringing it forward now (before §H). Staged build/test: **§G.2a = click-header single-sort** first, then **§G.2b = multi-sort panel**.

**§G.2a — click-header single sort (this commit).** Click a header → asc → desc → off (single sort; replaces). Arrow (↑/↓) marks the active sort. Saved into the `.base` `order:`, persists on reopen.
- **Backend:** `LensResult` gains `order: Vec<LensSort>` (exposes `def.order` so the table knows the current sort without re-parsing YAML). New IPC `lens::query::update_base_order(file_path, order)` — round-trips through `LensDefinition`, rewrites only `order:`, `validate` rejects a non-sortable dim, returns YAML. Registered in `lib.rs`. **No `dimensions.rs` change** — kept `note.path`/`note.headline` non-sortable (avoids the federated-JOIN-order question for `note.headline`); the frontend mirrors sortability.
- **Frontend:** `tableModel.isSortable(dim)` (false for `note.path`/`note.headline`, true for `note.name`/`note.created_at`/`prop.*`). `store.LensSort` type + `updateBaseOrder` wrapper. BaseTab: `sortDir`/`cycleSort`/`persistOrder`; Name + data headers are sort buttons (arrow + `sortBy` tooltip; non-sortable headers don't respond). i18n `lensBlock.sortBy` (en+ar).
- **Verify:** cargo check --lib exit 0; svelte-check clean. Boss test pending.

**§G.2b — multi-sort panel (built).** A **"Sort"** action button (next to "+ Add column", shows a count + accent when sorts are active) opens `BaseSortPanel.svelte`: lists the active `order:` entries with a priority number, direction toggle (↑ A–Z / ↓ Z–A), ↑/↓ priority reorder, and × remove; plus an "Add a column to sort by" list (sortable columns + `note.name`, minus those already sorted). Every edit calls `persistOrder` → `update_base_order` (reused from §G.2a) → re-query; the panel stays open so a multi-level sort is built in one place. Click-header (§G.2a) remains the quick single-sort. **Frontend-only** (no backend change). i18n `lensBlock.{sort,notSorted,toggleDirection,moveUp,moveDown,removeSort,addSort,allSorted}` (en+ar). svelte-check clean. Staged Boss test: **Stage A = click-header**, **Stage B = the Sort panel** (one binary).

**§G.2 Stage A — PASS** ("All passed", build mtime 22:21:30; §G.2b rebuild 22:29:44 has both features). Click-header asc/desc/off + arrow + persistence all confirmed. **Stage B (multi-sort panel) not yet tested** — Eisa pivoted to a design question first.

### NEW locked requirement — rank-aware (ordinal) sorting for ranked dimensions

Eisa (2026-05-29, right after §G.2 Stage A): *"for those statuses (stage, maturity…) which are Constellation's unique characteristics, when we sort them we are going to do it not only ascending/descending (conventional) but also by their ranks / superiority / seniority — because this is how Constellation is designed."*

**He's right, and his data proves it.** Today's sort is alphabetical:
- `maturity` A–Z = `canonical < evergreen < sapling < seed` → `seed` (the start) sorts LAST. Designed: `seed → sapling → evergreen → …`.
- `stage` A–Z = `birth < fleeting < growth < literature < maturity < spark < spark-idea` → `spark` (genesis) sorts to the END. Designed (Living-Link lifecycle): `spark → birth → growth → maturity → …`.

**Decision:** ranked cognitive dimensions sort by **canonical rank**, not alphabet. Proven pattern (Airtable/Notion select-field option-order sort). **Lands with the Cognitive Engine column family** (Architect §2 MIG-068 — stratum/maturity/provenance + their write-time derivation); each ranked dimension declares its canonical ordered values; the sort orders by rank-position; the direction toggle relabels "A–Z/Z–A" → "earliest→latest". Today `stage`/`maturity` are frontmatter text → alphabetical until then (or a sooner "ranked-field registry" if Eisa wants it before MIG-068). **Eisa specifies the canonical orders** (esp. edge values: `maturity: canonical`, stray `stage: fleeting/literature/spark-idea` — don't guess). Saved as memory `project_ranked_dimensions_sort_by_rank`. **Not blocking §G.2/§H** — it's a Cognitive-Engine-era enhancement.

### Deep study of the cognitive model (Eisa directed: "read as much as possible")

Boss pivoted from the sort/placement detail to a first-principles question: *bring ALL cognitive elements into perspective — why do we need them, how do they help shape KNOWLEDGE, how do we make it SIMPLE?* Studied the whole system (User Manual, help files, concept papers) + cross-checked against code (the authority):
- **Maturity** (`maturity.rs`/`search.rs`): RANK = seed→sapling→evergreen→canonical; **wilting = CONDITION** (any level untouched 90+ days), NOT a rank (Eisa ruling). Code currently only wilts evergreen-level — divergence to fix at MIG-068.
- **Stratum** (`strata.rs`): Datum→Information→Proposition→Concept→Principle→Theory→Paradigm→Worldview (1–8).
- **Stage** (`Stages-Concept-Paper-v1.2`): spark→birth→growth→maturity→dormancy→archival (6, terminal Archival, NO "renewal"). Legacy Zettelkasten values (fleeting/literature/permanent/synthesis) still render off-spine; `spark-idea` = per-note dash-suffix of spark.
- All sourced orders saved to memory `project_ranked_dimensions_sort_by_rank`.

**Zettelkasten research (WA#5, WebSearch):** fleeting/literature/permanent are *both* a workflow progression AND different kinds — they **conflate** what Constellation separates. `literature` = a *provenance* fact (Received), not a stage. `fleeting` ≈ spark+seed. `synthesis` ≈ high stratum + Act-IV. → the legacy labels are a low-dimensional shadow of Constellation's richer model. Sources: zettelkasten.de, bobdoto.

**★ The synthesis Eisa was after — "four questions, one process, one mirror":** crystallized into a NEW concept paper **`docs/Cognitive-Engine-One-Picture-Concept-Paper-v1.0.md`**. The ~10 cognitive elements collapse into **FOUR questions** every person already asks about an idea — **Development** (how grown? stage+maturity), **Altitude** (how high? stratum), **Origin** (where from / how known? provenance+source+content-type), **Connection** (how related? links+CNS) — all in service of **ONE process** (the Five Acts) toward **ONE destination** (Conviction). Drafter ruling (Eisa: "you decide"): **FOUR, not five** — certainty/confidence is the *destination* the four move toward (the Five Acts end in Conviction), not a fifth coordinate; a note can be developed-yet-contested, so certainty is orthogonal *because it is the goal*. The four = the classical anatomy of a knowledge claim (content/justification/relations/development; DIKW + isnad + pramāṇa). The paper's §6 mapping table is the **decision rule**: every element (and every legacy value — `literature`→Origin, etc.) is placed by "which of the four does it answer?" Eisa: *"Yes, you've nailed it."* **Open:** Eisa to ratify the four-vs-five cut + the four names; the paper's orientation reference lands at §L PCS. The lens now governs the Base's cognitive columns + the rank-sorts.
