# 10 — Outgoing Links Panel (Concept Paper)

> A satellite of the **Editor** (the gate). It reads what the active note links *out to*. Companion to the Backlinks panel (which reads what links *in*); the two share visual grammar, the confidence popover, and the same in-memory link source.

## 1. Function in hand
The **Outgoing links** panel — `src/lib/components/OutgoingLinksPanel.svelte`. UI header: `outgoingLinksPanel.header` → "Outgoing links". Mounted in the sidebar / right flank by `src/routes/+layout.svelte` (three mount sites: left flank, right flank, and the right-sidebar section under `panels.outgoingLinksHeader`).

## 2. Purpose
Show, for the active note, every wikilink it *emits* — the targets it points at — with each link's type pill(s), traversal count/tier, confidence, and annotation. It serves **Connection** (the 2nd of the Five Acts): it makes the note's outward reach visible so the user sees what this note already builds on. Secondary role in **Conviction**: the confidence popover (hypothesis → evidence → established → contested) lets the user grade each link in place, and archive it. Justified: outgoing reach is half of a note's link picture (Backlinks is the other half); without it the Living Link is one-directional in the UI.

## 3. What it is NOT
- **Not** a link *editor* — it does not create or rewrite `[[wikilinks]]`; that's the Editor's job. It only displays + grades + archives existing links.
- **Not** the source of truth — it reads the `note_links` index (mirrored in memory), never the `.md` file directly.
- **Not** Backlinks — Backlinks reads inbound; this reads outbound. Same grammar, opposite direction.
- **Not** a graph — no spatial layout; a flat weight-sorted list (graph is Phase 3, a separate surface).

## 4. Wiring
- **Inputs (props):** `outgoingLinks` (the pre-computed rows), `activeNoteName`, `activeNotePath`, `libraryPath`, `libraryColorMap`, plus `onConfidenceChange` / `onArchive` callbacks. Reads stores: `appSettings.linkPills` (pill shape), `dominantLocale` (note language). The rows are produced by `getOutgoingLinks(effectiveLibraryLinks, tab.path, decayCfg)` in `+layout.svelte` (filter → sort-by-weight-with-decay → map → `dedupeBySource`).
- **Inputs (IPC, lazy):** on tab change, an `$effect` resolves each target via `resolveWikilinkCrossLibrary` then batches `getSummariesFor` (NSC headline under each row, MIG-044 P2). Bounded to ~once per tab-open, not per render.
- **Outputs (IPC/writes):** `setLinkConfidence(sourcePath, target, level)` and `archiveLink(sourcePath, target)` from the right-click popover; `openNoteTab(...)` on row click (opens the target, Ctrl/Cmd-click = new tab).
- **Consumers:** the user (read-only surface) + the parent layout via the two callbacks (to refresh after a confidence/archive change).
- **Connection to the Editor (the gate):** the panel does **not** watch the file. It re-renders only because the Editor's save fires the reindex that refreshes `allLibraryLinks`, and the tab-switch handler in `+layout.svelte` recomputes `currentOutgoing`. No silent read of the note — the gate's dispatch is the only trigger.

## 5. Right-click / context menu
- **Has one — but HAND-ROLLED.** `oncontextmenu` on each row opens a custom confidence popover (`.conf-menu` / `.conf-overlay` built inline in this file), NOT the shared `<ContextMenu>` from MIG-077 (`src/lib/components/ContextMenu.svelte` exists and is used elsewhere). **Flag the debt:** this popover duplicates the same hand-rolled popover in `BacklinksPanel.svelte` (the comment even says "shared visual grammar with OutgoingLinksPanel"). Bring-up should fold both into `<ContextMenu>` (one source of truth, MIG-077 intent).
- **Items:** Set confidence → 4 levels (hypothesis / evidence / established / contested) with colored dots; separator; **Archive link**.
- **Reachable only by right-click:** **setting confidence and archiving a link** — there is no left-click affordance for either. Left-click on a row navigates to the target. So grading/archiving a link is right-click-exclusive in this panel.

## 6. Multilingual
- **Strings via `$t()`:** header (`outgoingLinksPanel.header`), empty state (`outgoingLinksPanel.noLinks`), and the popover (`linkConfidence.rightClickHint`, `.setConfidence`, `.<level>`, `.archive`). Verified present with native translations in `en.json` and `ar.json` (e.g. ar header "الروابط الصادرة", `rightClickHint` "انقر بزر الفأرة الأيمن لتعيين الثقة"). Full ×15-locale coverage of these keys — **verify in bring-up** (only en + ar spot-checked here).
- **Note-language principle (§H):** link-type pills + annotations render in the **note's** language via `noteLoc()` = `dominantLocale(activeNoteName)` and `tIn(noteLoc(), ...)`, not the UI language — matches BacklinksPanel.
- **RTL/dir:** rows and the NSC headline use `dir="auto"`; CSS uses logical `text-align: start`. No hardcoded user-facing English string found in the panel itself.
- **Minor flag:** `fmtTraversed()` returns hardcoded English tooltip fragments ("today", "yesterday", "2d ago") and the traversal chip `title` ("Traversed N times · tier · Last: …") is hardcoded English — tooltip-only, never localized. Flag for bring-up; low severity (hover text, not body UI).

## 7. Boot behavior
- **Runs at boot?** No. The panel mounts with the layout but is empty until a note is active. The link table (`allLibraryLinks`) is loaded by the boot snapshot / reindex path, not by this panel.
- **Rule 8 status:** ⚠️ **RECOMPUTES-on-read.** `getOutgoingLinks` runs a `.filter()` + `.sort()` + `.map()` + `dedupeBySource` over the *entire* in-memory link array on **every tab switch** (read/focus time), not a persisted per-note derived view. This is exactly the CLAUDE.md Rule 8 audit-pending item: *"Backlinks/Outgoing panels (recomputed on tab focus)."* The fix shape: persist per-source outgoing rows, maintained by a trigger/hook on the link write path, so a tab switch is a cheap lookup.
- **Cost:** O(total links) per tab switch, in JS. Bounded but not free on a large Universe (7,600+ notes → many links). **Estimated** — not measured; measure before/after if persisting.

## 8. Flag / gate & bring-up position
- **Gate today:** none — it mounts unconditionally with the layout (no `enabledFeatures.X` / no SIGHT flag). In a minimal-mode shell it would need a satellite guard to flip off with the other panels.
- **Bring-up phase:** **2 (Editor satellites)** — alongside Backlinks/Tags. Depends on: the Editor (the gate) for change dispatch, the link index (`allLibraryLinks` / reindex), `resolveWikilinkCrossLibrary` + `getSummariesFor` (NSC), and `LinkTypePill`. **Phase 3 (graph)** is a separate surface, not this panel.

## 9. Budget
- **Boot budget:** zero added boot cost (empty until a note opens) — must stay off the boot path.
- **Interaction budget:** tab-switch render must be imperceptible. Today's per-tab recompute is the watch item (see §7); the lazy NSC `$effect` must stay ~once-per-tab-open, never per render.
- **Regression guard:** switch tabs rapidly on a large Universe and confirm no lag; confirm the NSC `$effect` fires once per tab-open (temporary `console.log`, Rule 7); confirm confidence/archive write fires exactly one IPC and refreshes both this panel and Backlinks.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** outbound links of the active note appear, weight-sorted, with type pills + traversal tier + confidence + annotation.
- [ ] **Serves Constellation's core purpose:** makes **Connection** visible and lets **Conviction** be graded in place (Five Acts — see [00-Constellation](00-Constellation-Core-Concept-Paper.md)).
- [ ] **Wires correctly to the Editor:** updates only via the gate's reindex + tab-switch recompute; no silent file read; row click opens target; Ctrl/Cmd-click new tab.
- [ ] **Right-click present + correct (shared, not hand-rolled):** confidence + archive reachable; **fold the hand-rolled popover into shared `<ContextMenu>`** (currently duplicated with BacklinksPanel — debt).
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** header/empty/popover localized in all 15 locales; pills/annotations in the note's language; `dir="auto"`; **fix the hardcoded `fmtTraversed` + traversal-chip tooltip English.**
- [ ] **Within budget:** rapid tab-switch on 7,600+ notes shows no lag; NSC `$effect` ~once per tab-open.
- [ ] **Obeys Rule 8:** ⚠️ currently NO — persist the derived outgoing view + maintain via a write-path trigger; turn the tab-switch into a lookup.
- [ ] **Holds its invariants:** dedupe by target (X with `[[X]]`+`[[X|type]]` shows once, both chips); archived links excluded; weight+decay sort stable.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **—** (Rule 8 violation outstanding; cost unmeasured)
Notes: Two carried items for bring-up — (1) **Rule 8:** recomputes outgoing rows on every tab focus instead of reading a persisted view (the CLAUDE.md audit item, shared with Backlinks); (2) **context-menu debt:** hand-rolled confidence/archive popover duplicated with BacklinksPanel — fold both into the shared `<ContextMenu>` (MIG-077). Minor i18n flag: `fmtTraversed` + traversal-chip tooltips are hardcoded English (hover-only). Full ×15-locale key coverage spot-checked on en+ar only — verify in bring-up.
