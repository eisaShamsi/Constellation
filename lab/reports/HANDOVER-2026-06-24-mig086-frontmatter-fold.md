# Handover — 2026-06-24 — MIG-086 frontmatter fold (§F1–§F3) SHIPPED; §D + §E next

## State of standing

**Shipped + Boss-validated this session — COMMITTED `dc6056a4` on `main` (PUSHED at this handover):**
- **MIG-086 §F1** — `index_note` derives `note_links` from BOTH the body (`[[type::target]]`) and the
  FRONTMATTER (type-as-property, `supports:\n  - "[[X]]"`), merged + deduped. New
  `extract_frontmatter_typed_links` (search.rs); 7 unit tests; back-compat (non-type keys → associative).
- **MIG-086 §F2** — the one-click connect writes the frontmatter property via the PROPS save path
  (`store.ts::addLinkToNote` + `addTypedLinkToProps`); body never touched (no BUG-015 surface).
  `reconstructFrontmatter` now quotes list items (valid YAML; round-trip byte-stable). Connect is
  NON-BLOCKING (fire-and-forget reindex + optimistic Reviewer). **Boss-validated end-to-end** (clean link,
  content-integrity, instant, rename-safe, open-note refresh).
- **MIG-086 §F3** — frontmatter links survive rename (cascade is already content-wide); regression test added.
- **§A uncapped** — suggestions show ALL related notes closest-first; shared-term "why" via FTS-index
  membership (Rule 8). **§C UI kept** (LinkTypePicker + RelatedCandidates wiring); orphan "Connect" button removed.

**Binary:** `src-tauri/target/release/constellation.exe` (18:21, 2026-06-24) — has §F1–§F3 + all fixes.
**Key commit:** `dc6056a4`.

## The big reframe (read this before touching links)
- **§C body-append is RETIRED.** A *declared* typed link now lives in the note's **frontmatter** as
  type-as-property (Boss ruling: a dangling body link "without context is illogical"). Body `[[type::target]]`
  still works (contextual links). One unified `note_links` index over both.
- **DOC-DRIFT (corrected in Orientation v3.05):** the "`LINK` file" in CLAUDE.md/orientation **does not
  exist in code**. Links are derived (body + frontmatter) into `note_links`; earned props (weight/confidence/
  traversal) live only there. Don't build against a LINK file.

## NEXT — MIG-086 §D + §E (Plan: `docs/MIG-086-Plan.md` Part 2)

### §D — wire `<RelatedCandidates>` into the 4 other hosts (mount map already captured in the session log)
Add a host-set `direction: 'inbound' | 'outbound'` prop to `<RelatedCandidates>`; `choose()` routes
`addLinkToNote` (inbound = suggestion→in-hand/de-orphan; outbound = in-hand→suggestion). **Do NOT add a
user In/Out/Both toggle — that's PJ-067.**
- **NotePane Backlinks tab** — `+layout.svelte:7483` (after OutgoingLinksPanel). vars: `sidebarTab.path/.name/.libraryPath`. Direction **outbound**. No gap.
- **360 Inspector (BOTH mounts)** — `+layout.svelte:6574` (full) + `:7552` (compact). Mount inside `Inspector360.svelte` between matrix and HUD (~`:455`). **Gap: plumb `libraryPath` into Inspector360 props (both mount sites).** Gate `data.is_orphan ‖ single_point_of_failure ‖ missing_link_types`. Direction **inbound**.
- **Health tab — TensionPanel** — mount at `+layout.svelte:7527`; inside `TensionPanel.svelte` (~`:204`). **Gap: pass `libraryPath` prop from the mount.** Direction **inbound**.
- **Sky View per-node menu** — `GraphMindView.svelte:~1159`; node carries `{id,name,path,libraryName}`. **Gap: derive `libraryPath` from `libraryName` via `allNotes`.** Add a "Suggest connections…" item opening `<RelatedCandidates>` in a popover. Direction **outbound**.

### §E — i18n×15, RTL per host, /simplify, **help + User Manual ×15** (now user-actionable), Orientation full v-bump (body §4.x Living Link + §8 migrations + §12 doc-drift), MoCh, mark MIG-086 shipped. §F4 polish (PropertyEditor type-pill + clickable frontmatter wikilink) folds in here.

## Open follow-ups (filed, Concept-Paper-first / deferred)
- **PJ-065** — brand-new frontmatter parent/child/TOC link type (authors/screenwriters). Depends on the fold.
- **PJ-066** — **P1 perf** — sky-trigger reindex storm on link-dense notes (~2 min): `index_note` re-fires
  per-edge `note_links_sky_stratum/maturity` triggers (COUNT(DISTINCT) over 234k rows). Options: composite
  index / defer-batch (MIG-079 §C.2a pattern) / diff-edges. Sky/MIG-079 domain; needs Rule-8 measurement.
- **PJ-067** — Living Link Relationship Model v2 (Concept-Paper-first). The typology: dimensions (symmetry/
  transitivity/inverse/**arity**/cardinality/taxonomic-thematic) + uncharted families (thematic/functional,
  **analogy**, **n-ary synthesis** = `co-completes`, **undercuts/undermines**, **problematizes/answers**).
  Research doc: `docs/Living-Link-Relationship-Typology-Research-2026-06-24.md`. **`complements`→`co-completes`** (lexical complementarity = the opposite).

## Invariants locked
Single-writer (links born as file text — body OR frontmatter — `index_note` derives `note_links`; nothing
writes `note_links` directly); dual-source parity (panels read the index, source-agnostic); content-integrity
via the props save path (no body touch); confidence default `hypothesis` from `index_note` (C-4); Rule 8
(suggest on-demand; frontmatter parse is part of write-time index_note); back-compat (every body link works).

## To resume
Read Orientation **v3.05** (highest) + this handover + `docs/MIG-086-Plan.md` (Part 2) + the typology research
doc. Then cascade §D → pause at §D Boss test → §E. Pre-existing PJ-066 makes link-dense-note reindex slow in
the background (Boss-deferred); don't be alarmed by it during §D testing.

---

## READY-TO-PASTE NEXT-SESSION PROMPT
```
Resume MIG-086 §D — wire the connect into the 4 remaining hosts. Read Orientation v3.05 (highest) +
lab/reports/HANDOVER-2026-06-24-mig086-frontmatter-fold.md + docs/MIG-086-Plan.md (Part 2) first.

§F1–§F3 (the frontmatter fold) shipped + Boss-validated, committed dc6056a4: typed links are now declared
as frontmatter type-as-property (supports: ["[[X]]"]); index_note reads body + frontmatter into note_links;
the connect writes frontmatter via the props save path (non-blocking). §C's body-append is retired.

Build §D: add a host-set direction prop ('inbound'|'outbound') to <RelatedCandidates> (NO user In/Out/Both
toggle — that's PJ-067), then wire it into: (1) NotePane Backlinks tab [outbound, no gap], (2) 360 Inspector
both mounts [inbound; plumb libraryPath into Inspector360 props], (3) Health/TensionPanel [inbound; pass
libraryPath prop], (4) Sky View per-node menu [outbound; derive libraryPath from libraryName via allNotes;
open RelatedCandidates in a popover]. Gates + exact mount points in the handover. Pause at the §D Boss test.
Then §E: i18n×15, RTL, /simplify, help + User Manual ×15 (now user-actionable), full Orientation v-bump
(body §4.x/§8/§12), §F4 display polish (type-pill + clickable frontmatter wikilink), MoCh, mark MIG-086 shipped.

Heads-up: PJ-066 (pre-existing sky-trigger reindex storm) makes link-dense-note reindex slow in the
BACKGROUND — Boss-deferred to its own perf migration; not a §D bug. Do NOT fold the typology (PJ-067:
co-completes, n-ary, analogy, undercuts, thematic/functional, dimensions) into MIG-086 — it's Concept-Paper-first.
Cascade per Plan-Approval=Build-Approval, pausing only at the §D Boss test.
```
