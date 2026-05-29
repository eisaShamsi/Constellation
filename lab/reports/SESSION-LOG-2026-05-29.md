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
