# Right Sidebar → Note-Context-Only — Design Decision

> **Logged direction for a future `/migration`. NOT yet built.** Sequencing (Boss 2026-06-16): runs **after MIG-079 §C.2/§C.3** (boot perf). Sources: the right-sidebar scope audit (`wf_29860fe0`), the PKM-placement research (`wf_e09ede1a`, web-cited), and Boss dispositions 2026-06-16.

## The contract
The right sidebar is the **live, note-context extension of the open note** — every tab answers exactly *"tell me about THIS note."* No whole-universe / whole-library aggregates may render there. This is **Form-Aligns-To-Purpose** applied to a panel region, and it is the dominant convention across VS Code (Secondary Side Bar), Obsidian, Notion, Roam, Zed, Logseq (right dock = active-item context; navigation/aggregates live left, in a palette, or full-page).

## Right sidebar — the open note's diagnostic suite (note-scoped)
| Tab | Disposition |
|---|---|
| Properties (+ Outline) | keep — already note-scoped (`&& sidebarTab` gate) |
| Backlinks (+ Outgoing) | keep — computed relative to the open note |
| Tags **"This note"** | keep; the **"All tags"** toggle is removed from here |
| Sky View (local neighborhood graph) | keep — `localSkyNodes/Links` centered on the open note |
| Provenance | keep — `get_provenance_chain` for the open note |
| 360.3D | keep — *"where this note sits"*: stratum / maturity / link-types / network position |
| **Knowledge Health** | **REDESIGN → "what's weak/wrong about THIS note"** (the note's own tensions/health). **Distinct from 360.3D** (Boss 2026-06-16). |
| **Review Pulse** | **REDESIGN → THIS note's review status** (last reviewed / next due / interval) |
| **Source Review** | **REDESIGN → THIS note's sources/classification only** |

## Relocations — universe-scoped functions leave the right rail
| Function | New home | Grounding (PKM research + code) |
|---|---|---|
| Tags **"All tags"** | **Search Hub** (tag-index facet) **+ the Dashboard** | Tags already click-through to the Search Hub (`#tag` federated query, `+layout.svelte:4160-4161`); Obsidian/Tana "sidebar list w/ counts → filtered notes" pattern; reuses the **MIG-079 §C.1 `tag_counts`** table (cheap lookup, not a scan) |
| **Calendar** | **LEFT-sidebar launcher → main-pane daily note** | Field consensus (Logseq/Roam/Tana/Obsidian = launcher → dated page); month grid is a minority opt-in *left* widget. **Fixes the wrong-library daily-note defect.** |
| **Tasks** | integrated with the **LEFT-sidebar Calendar** (date/agenda) | Boss 2026-06-16 (the open note's *own* inline tasks may still surface in-note; the agenda lives left with the calendar) |
| Knowledge Health (universe) | the existing full-page **Knowledge Health Dashboard** | analytics → full page in every tool surveyed |
| Review queue (universe) | **full-page stepped reviewer** + a "N due" card on the Dashboard | Anki/RemNote/Mochi/roam-sr full-screen-reviewer consensus. **Fixes the `record_note_visit`-never-called defect.** |
| **Cataloger** (universe source/classification) | **LEFT sidebar** | Boss 2026-06-16: Source Review (right) = the open note only; the Cataloger (left) = the whole universe |

## The three splits (redesign, not just a move)
`Knowledge Health` (`detect_tensions`), `Review Pulse` (`get_due_notes`), and `Source Review` (`sources_list_pending_suggestions`) are **library-scoped today**. Each **splits in two**: a NEW note-scoped panel in the right rail + the universe version relocated (Dashboard / full-page reviewer / Cataloger-left). This is more work than a relocation, but it is the cleaner design.

## Defects fixed by the moves (not patched in place)
- **Calendar** — `onDayClick` always opens `libraries[0]`'s daily note regardless of the date's library → rebuilt correctly at the left-sidebar launcher.
- **Review** — `record_note_visit` exists in the backend but is never called from the frontend (reviewed notes keep resurfacing) → wired at the new reviewer.

## Footprint
**0 new dock buttons · 1 new overlay** (the full-page reviewer, which the field independently endorses). Reuses the Search Hub, the Knowledge Health Dashboard, the Cataloger, and the §C.1 `tag_counts` table. The Cataloger hosts none of the other three (it is a classification queue — orthogonal).

## Process
`/migration`-grade: crosses Svelte↔Rust + settings-schema↔code (`panelPlacements`, `NOTE_SCOPED_TABS`), 3 panel redesigns, and 2 defect fixes. Runs as its **own four-phase `/migration`** (Architect → Plan → Build → Audit) **after MIG-079 §C.2/§C.3**. This file is the seed for that Architect phase.
