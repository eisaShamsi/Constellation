# Handover — 2026-06-24 — MIG-086 §A+§B SHIPPED; §C–§E next (save-path, build fresh)

## State of standing

**Shipped + Boss-validated this session (on `main`, committed; PUSH at this handover):**
- **MIG-085 §B** (§B.0 Unicode name-fold + §B.1 maturity single-source + §B.2 360 outbound) — the
  accented-title false-orphan fix + maturity agrees on every surface. End-to-end Boss-validated.
- **MIG-086 §A** — `libraries.rs::suggest_related_notes` (BM25 "More Like This"); backend only,
  query-time over `notes_fts`. Perf bounded (typical <300 ms; the 3 biggest notes 0.5–2.3 s). 3 unit
  tests + a live rehearsal. **Reads `notes_vocab` for df, NOT `term_vocab` (which is drift-broken).**
- **MIG-086 §B** — `src/lib/components/RelatedCandidates.svelte` (read-only): the "Connect to:" list
  in the Reviewer's orphan/fragile detail. The Link button is INERT (wired in §C). Boss-validated
  incl. RTL both UI directions. `suggestRelatedNotes` wrapper + `RelatedCandidate` in store.ts.

**Binary:** `src-tauri/target/release/constellation.exe` (10:25, 2026-06-24).
**Key commits:** `16fffbe9` (§A) · `f076acfc`/`7979d345`/`b667d01b`/`ac67f19a` (§B + fixes) ·
`050462eb` (FTS forensics).

## NEXT — MIG-086 §C–§E (the Plan: `docs/MIG-086-Plan.md`; concept invariants in §1b)

### §C — the one-click typed-link action (THE SAVE-PATH STEP — build fresh, full Editor-Surface Gate)
Wire the inert Link button: click → open the 8-type **`<LinkTypePicker>`** (NEW small component on
`linkTypeRegistry` + `LinkTypePill`; pre-select orphan→`associative`, fragile→`derives-from` per Boss
ruling §9b "always ask the type") → on choose, **`addLinkToNote(sourcePath, type, target)`** (NEW
headless helper, clone `addTagToNote`'s open/closed branch at `+layout.svelte:5248`): append
`[[type::Target]]` to the source body via `writeNote`/`saveTabContent` (open-note path MUST go through
`composeNoteModel`+`saveTabContent`, identity-guarded — the BUG-015 invariant) → `reindexNote` →
refresh via the Reviewer `act()` pattern. **NO `note_links` writer** — the link is born as body text;
`index_note` derives the row (Living Link single-writer). **Concept invariant C-1: NO bulk "Link all".**
Suggestion-born links should default to `hypothesis` confidence (C-4 — confirm `index_note`'s default).
**Run the FULL 8-point Editor-Surface Gate** (it touches the save path) + a Boss tutorial.

### §D — wire `<RelatedCandidates>` into the other 4 hosts
NotePane right-sidebar (Backlinks tab, `+layout.svelte:7483`); 360 Inspector BOTH mounts (plumb
`libraryPath`); the note's Health tab (TensionPanel, beside the orphan rows `:162–183`); Sky View's
per-node right-click menu item (`GraphMindView.svelte:1153` — a menu item, NOT the canvas).

### §E — i18n×15 finish (§C/§D keys), RTL on every host, `/simplify`, docs, **help/User-Manual** (now
the feature is user-actionable), Orientation v-bump, MoCh, mark MIG-086 shipped.

## Open follow-ups (separate, flagged — two worktree sessions already spawned)
- **term_vocab ~92% empty** (P2) — re-point the "≈ similar" consumer to `notes_vocab`, retire
  term_vocab's counts. `docs/FTS-Health-Forensics-2026-06-23.md`.
- **FTS index 1.9 GB + a background `optimize`** (modest tune-up; the code has a TODO at search.rs:7459).

## Invariants locked
BM25 ranking uses the BARE FTS query (no JOIN/NOT-IN — it defeats the rank-limit fast path); df from
`notes_vocab`; the concept invariants (no bulk accept; mandatory why; hypothesis confidence;
invitational; honest empty state); the Living Link single-writer; Rule 1/3 (suggest is on-demand,
never per-keystroke); RTL = mirror when UI-RTL OR content-RTL.

## To resume
Read Orientation **v3.04** + this handover + SESSION-LOG-2026-06-23.md + `docs/MIG-086-Plan.md`. Then
build §C with care.

---

## READY-TO-PASTE NEXT-SESSION PROMPT
```
Resume MIG-086 §C — the one-click typed-link action (THE save-path step). Read Orientation v3.04
(highest) + lab/reports/HANDOVER-2026-06-24-mig086-ab-complete.md + docs/MIG-086-Plan.md first.

§A (suggest_related_notes backend) and §B (the read-only <RelatedCandidates> list in the Reviewer)
are SHIPPED + Boss-validated; the Link button is currently INERT. Build §C: wire it to open the
8-type picker (build a small <LinkTypePicker> on linkTypeRegistry+LinkTypePill; pre-select
orphan→associative, fragile→derives-from per Boss's "always ask the type" ruling) → on choose, a NEW
headless addLinkToNote() that appends [[type::Target]] to the source note's body via the EXISTING
writeNote/saveTabContent path (open-note path through composeNoteModel+saveTabContent, identity-guarded
— BUG-015) → reindex → refresh. NO note_links writer (Living Link single-writer). NO bulk "Link all"
(concept C-1). Default suggestion-born links to hypothesis confidence (C-4). This touches the save
path — run the FULL 8-point Editor-Surface Gate in the reproduction harness AND a Boss tutorial before
declaring it done. Then §D (wire the other 4 hosts: NotePane sidebar, 360 both mounts, Health
tab/TensionPanel, Sky View node menu) → §E (i18n×15, RTL per host, /simplify, help/User-Manual,
Orientation v-bump). Cascade per Plan-Approval=Build-Approval, pausing at the §C and §D Boss tests.
```
