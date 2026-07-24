# SESSION LOG — 2026-07-24 (session close; arc spans 2026-07-22 → 07-24)

**Function in hand:** MIG-103 §4 — the Template Studio's recognition room + KEEP
(Slice 2). Boss-validated ALL PASS; committed as `9ab033be`.

## What shipped (all Boss-validated on the release binary)

- **KEEP** — name a kind → a real template. Create-exclusive; typed clash with
  Rename / Add-these-fields (additive merge); byte-guarded Undo via trash; live
  write manifest; tail fields as opt-in CHIPS (Boss corrected the checkbox column).
- **The Studio remembers** — `from_kind:` stamped in the mold; `list_kept_kinds`
  reads templates back on open. Restart-proven by the Boss (the original complaint).
- **The index made honest** — templates excluded from kinds (679→682 inflation
  gone); phantom `- "Mimesis"` properties fixed at the parser AND cleared by the
  new `props_reparse_backfill` (12 notes corrected, 0 phantoms verified in the DB).
- **Fixes that never reached the Boss, each reproduced first:** the NUL-byte
  separator in `kindSignature` (would have silently defeated kept-kind recognition
  forever); the missing `i += 1` infinite loop (11 hours of CPU in a background
  test binary); casts inheriting `template_kind:` from their mold.
- **From the Slice-2 inspection (Stage-2 validated):** PropertyEditor teardown
  reseed-gate; `set_frontmatter_parent` block-list fix; classifier writes announce;
  template writes reindex; `(async)` on the Studio commands; visible error banner
  for template actions ×15.

## The freeze saga — recorded so it is never repeated

Three wrong diagnoses before the truth: (1) "needs a loading state", (2) "sync
IPC commands" — both blamed Rust; the real cause was a REACTIVE-MAP WRITE DURING
RENDER (`picksFor` created map entries from markup) → infinite Svelte re-render.
Rule 2 violation in different clothes. Fixed by read-only render path +
`ReadonlySet` typing so the compiler enforces it. Lesson recorded: reproduce
under instrumentation BEFORE diagnosing; a torn frame means a pinned UI thread,
not a slow backend.

Also: six cargo processes deadlocked on the build lock from stacked background
builds (self-inflicted); `grep|head` buffering made an empty log look like
progress. One build sequence at a time, unbuffered markers, hard caps.

## Found by the Boss at close — THE NEXT ACTION

**New-note-from-template picks its location SILENTLY** — focused note's folder,
else the FIRST library, which is the universe root. Boss: "it didn't ask for a
location; we need to fix this." The locked interaction model already rules this:
**destination propose + show; library picker when nothing is open.** Design
settled in-session: reuse `MoveDialog` (the existing filterable all-libraries
folder tree, `+layout.svelte:6287` builds its entries) — reuse, don't rebuild.
NOT BUILT YET. First item next session.

## Honest state

- **Inspection over the Slice-2 batch still RUNNING at close** (`wf_acc3ca2c-4f6`;
  PJ-124 makes it whole-app). Boss-directed PCS closed the session before it
  returned. **Next session MUST triage its findings first** — nothing is hidden.
- PJ-132 Sight perf flake fired twice more under parallel load (passes isolated).
- PJ-124 re-confirmed (7th+): args.files ignored, every run whole-app.

**Rust 1133/0 · svelte-check 0 · vitest 609/609.** PJ ledger → v1.46.
Orientation → v3.64. MoCh → MoCh-2026-07-24-0600.md.

## Ready-to-paste next-session prompt

> Continue MIG-103. First: triage the safety-inspection run `wf_acc3ca2c-4f6`
> (was still running at PCS; journal under the session's workflows dir) — fix
> confirmed findings per WA#6. Then build the ruled destination-propose+show for
> new-note-from-template: reuse MoveDialog (+layout.svelte:6287 entry-builder),
> show the destination in the title prompt, library picker when nothing is open;
> Boss-test per the Testing Instructions Rule. Then §1 use-side remainder
> (mixing heads-up), then D4. Ledger v1.46 has the full queue.
