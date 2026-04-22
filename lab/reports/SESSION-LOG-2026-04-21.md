# Session Log — 2026-04-21

## § 43. Right-sidebar UX polish (after P5 pass)

User remarks after P5 test pass:
1. Make all backlink elements collapsible.
2. Freeze the right-sidebar tabs so they remain visible while
   scrolling through any tab's details.
3. Drop the divider lines between Backlinks / Outgoing sections.

### Changes

- `BacklinksPanel`: Linked Mentions section header is now a
  chevron-toggle button (starts expanded). Unlinked Mentions was
  already collapsible.
- `OutgoingLinksPanel`: Outgoing Links section header is now a
  chevron-toggle button (starts expanded). Added `.ol-toggle` /
  `.ol-chev` CSS mirroring the Backlinks panel.
- `+layout.svelte` `.rs-tabs`: `position: sticky; top: 0; z-index:
  3` with a solid background so the tab row stays pinned as the
  content area scrolls.
- `+layout.svelte`: `.rs-section--flush` modifier applied to the
  two backlinks-tab sections so the horizontal divider between
  Backlinks and Outgoing Links disappears. Other tabs keep their
  original dividers.

## § 44. P5 deferred items — shipped

Two items that were listed as "deferred polish" at the end of § 42
of 2026-04-20 are now landed. P5 is feature-complete including
deferred polish.

### (a) User-driven tier promotion UI (contest / force-promote)

- `BacklinksPanel` and `OutgoingLinksPanel` grew an absolute-
  positioned popover that opens on **right-click** of any link
  row. The popover offers four buttons: Hypothesis / Evidence /
  Established / Contested. The current level is highlighted
  (bolded + accent-tinted). Selecting a value writes through the
  existing `constellation_link_set_confidence` Tauri command.
- `getBacklinks` / `getOutgoingLinks` now include `confidence` on
  the returned row so the popover can render the active level
  without a roundtrip.
- `+layout.svelte` carries a small `applyConfidenceLocally(sourcePath,
  targetName, confidence)` helper that mirrors the DB write into
  the in-memory `allLibraryLinks` array — the menu's "current"
  marker stays correct without a full link rescan.
- Dots next to each option use the same tier-gradient grammar as
  the P5-slice-3 chip: 14% tint → 40% tint → full accent fill →
  amber for contested.

### (b) One-shot DB backfill

- New Tauri command `constellation_link_backfill_confidence`
  (Rust, `search.rs`). Runs two UPDATEs:
  - `traversal_count ≥ 10` and confidence not already `established`
    / `contested` → `established`.
  - `traversal_count ≥ 3 AND < 10` and confidence = `hypothesis`
    → `evidence`.
- Never downgrades. Preserves user-set `contested`. Idempotent —
  safe to run multiple times.
- Returns `{ promoted_to_established, promoted_to_evidence,
  total }` so the UI can report what moved.
- Frontend wrapper: `backfillLinkConfidence()` in
  `libraries/store.ts`.
- UI: **Settings → Appearance → Living Link Lifecycle → Run back-
  fill**. Shows "Running…" while the call is in-flight, and on
  return prints "Promoted N link(s) (→evidence: X, →established:
  Y)." in accent color.

### i18n

Added two new key groups, propagated to all 15 locales via
`lab/scripts/i18n_link_confidence.py`:
- `linkConfidence.{setConfidence, rightClickHint, hypothesis,
  evidence, established, contested}` — popover labels + hover
  hint.
- `settings.appearance.{confidenceBackfill, confidenceBackfillDesc,
  confidenceBackfillBtn, confidenceBackfillRunning,
  confidenceBackfillResult}` — backfill setting.

### Verification

- `cargo check` in `src-tauri/`: compiles clean (59 pre-existing
  warnings, 0 errors, 0 new warnings from the new fn).
- `svelte-check`: no new type errors on any file I touched. The
  53 pre-existing errors are all outside the P5-deferred surface.
- Manual: right-click → pick Contested → right-click again shows
  Contested bolded. Confirmed by user ("P5 Pass").

### Commit

`<pending>` — PCS at end of § 44.

## § 45. Living Link — marked "Done"

After § 44 landed the deferred UI, the remaining Living Link
properties (annotation, archive/reversibility) were also shipped
so the full 8-property lifecycle from the spec is now user-visible
and user-controllable.

### Annotation display (read-only)

- `getBacklinks` / `getOutgoingLinks` now surface `annotation` on
  each row.
- BacklinksPanel: if a backlink has an annotation, render it on
  its own line in italic accent color below the context excerpt,
  wrapped in smart quotes.
- OutgoingLinksPanel: mirror of the same.
- Annotation is still authored inline in the note body via
  `[[type::target|your reasoning]]` — source of truth stays on
  disk. No edit UI in this slice (would require rewriting the
  source wikilink and is deferred as a separate UX question).

### Archive / unarchive — reversibility

Spec says "every link operation must be reversible — archival,
not deletion." Implementation:

- **Rust**: existing `constellation_link_archive` + two new
  commands:
  - `constellation_link_unarchive(source_path, target_name)` —
    sets status back to 'active', resets weight to 1.0. Traversal
    count + confidence are preserved so history isn't lost.
  - `constellation_link_archived()` — returns all archived rows
    ordered by `last_traversed DESC` for the dashboard.
- **Frontend**:
  - `archiveLink`, `unarchiveLink`, `listArchivedLinks` wrappers
    in `libraries/store.ts`.
  - Right-click popover in BacklinksPanel / OutgoingLinksPanel
    gained an "Archive link" row below the 4 confidence tiers,
    separated by a horizontal rule. Clicking it archives and
    removes the row from the panel via `applyArchiveLocally`
    mirror.
  - `getBacklinks` / `getOutgoingLinks` now filter out
    `status === 'archived'` so archived links disappear
    everywhere they're rendered.
  - LinkDashboard: new **Archived** tab (7th tab). Lazy-loads
    via `listArchivedLinks()` on first open (and on subsequent
    activations). Each row shows source → target with a circular-
    arrow restore button at the end. Source name is italic muted
    to signal "inactive."

### i18n

- `linkConfidence.archive` added (right-click menu label).
- `linkDashboard.{archived, noArchived, unarchiveTitle, loading}`
  added (tab + empty state + tooltip + loading).
- All 15 locales updated via
  `lab/scripts/i18n_link_archive.py`.

### User Manual

Two new bullets under "The Living Link":
- Annotation — what it is, how to author, where to read.
- Archive — how to archive and restore.

### Verification

- `cargo check`: clean, 60 pre-existing warnings, 0 errors.
- `svelte-check`: no new type errors on any file in this slice.
- Manual plan: right-click a backlink → Archive link → row
  disappears. Open Link Dashboard → Archived tab → see the row.
  Click the restore button → row disappears from Archived. Return
  to the source note → backlink is present again.

### Commit

`<pending>` — PCS at end of § 45.

### Living Link — remaining out-of-scope items

These are NOT required for the spec's "Living Link" marker — they
were notes in the original spec that we deliberately didn't
build:

1. **LINK files on disk** (`YYYYMMDDTHHMMSSZ_LINK_XXXX.md` as
   canonical storage). We persist in SQLite with the `.md`
   wikilink as source of truth; a per-link file layer was never
   implemented and would be a major storage model change with
   no current user-visible benefit. If ever pursued, it belongs
   in its own architectural phase.
2. **Annotation inline editor** — to edit the annotation from
   the sidebar without typing the raw `[[foo|ann]]` syntax,
   we'd need to rewrite the source wikilink. Deferred as a
   follow-up polish item.
3. **Legend surface inside Link Dashboard** — a dedicated "what
   do these tiers mean" side panel. The tooltip on the chip +
   the User Manual cover this for now.

## § 46. Docs + tutorials + Word export

Following the "Living Link Done" marker in §45, documentation
was brought up to parity with the shipped surface.

### What landed

- **User Manual (EN)** — `docs/User Manual.md` grew 11 step-by-
  step Living Link tutorials inline in the Knowledge Formulation
  section: typed links, annotations, tier growth, tier-vs-
  confidence, contest/force-promote, archive, restore, back-fill,
  decay tuning, the seven-tab Link Dashboard, search.
- **User Manual (AR)** — `docs/help.ar/User Manual.md` was
  missing the Knowledge Formulation section entirely (TOC
  jumped from line 9 to line 37). Added the full section plus
  the same 11 tutorials in Arabic, with a TOC entry at #0.
- **Help file (EN)** — `docs/help.uConstellation.World/Knowledge
  Formulation/Knowledge Formulation.md` expanded to 243 lines
  with the same tutorial block.
- **Help file (AR)** — `docs/help.ar/Knowledge Formulation/
  Knowledge Formulation.md` created fresh at 243 lines matching
  the English (the directory didn't exist prior).
- **Word exports** — `docs/Constellation User Manual.docx`
  regenerated (54 KB) with the new tutorials. Added
  `docs/generate-docx-ar.cjs` as an Arabic sibling of the
  existing generator; produces `docs/Constellation User Manual
  (AR).docx` (48 KB).

### RTL note

The Arabic docx was generated with the same pipeline as the
English one (no explicit `bidirectional: true` + right-align).
Word auto-detects character-level direction correctly for the
Arabic runs; paragraph alignment may need an explicit pass if
the Word rendering looks off. Deferred until user confirms.

### Commit

`<pending>` — PCS.

## Still on the queue (orthogonal)

- navTrace instrumentation dev-gate
- Settings → Debug Boot Performance scorecard UI
- Isolated throttle stress-test helper
- RTL alignment pass on Arabic docx (if needed)
