# MIG-076 — Write Integrity — Phase 2: PLAN

**Predecessor:** `MIG-076-WRITE-INTEGRITY-ARCHITECT.md` (same day). Five locks L0–L4 + journal + quarantine; invariants I1–I10. Each § below lands as one commit with its verification clause. ★ = Boss-test gate (staged per the Testing Instructions Rule).

---

## §A — The WriteGate (Rust foundation: L0 serialization + L1 atomic replace + journal)

**§A1 — `src-tauri/src/write_gate.rs`.** Per-path async lock map (`DashMap<PathBuf, Arc<Mutex<()>>>` or equivalent on the existing runtime); `gate_write(path, content, expect: Option<Expectation>) -> WriteOutcome` performing, under the path lock: optional CAS pre-read (§B), same-dir temp write → `sync_all` → `ReplaceFile`-class atomic swap → bounded AV retry (5 × 50–200 ms on sharing-violation/access-denied) → `watcher_suppress::mark` on BOTH temp + final path. `write_journal` table added in `init_db` (ts, path, surface, outcome, expected_cid, found_cid, content_hash, bytes) with a bounded retention sweep; `.constellation/quarantine/` helper with self-describing filenames.
*Verify:* new unit tests — two concurrent writes to one path serialize (no interleave); kill-between-temp-and-swap leaves the original intact; retry path exercised; journal rows written. Full suite green.

**§A2 — Route every Rust .md writer through the gate.** `write_note` (unconditional mode first — journals as `unchecked`), `create_note` via a new **create-exclusive** mode (fails if the path exists — collision behavior unchanged for callers), `rename_item` (frontmatter rewrite + `fs::rename` under locks on BOTH paths, ordered to avoid deadlock), the cascade walker per-file, `base_edit_cell`, `ensure_cid_cn` (both writes), `move_item`, tasks.rs:467. Zero call-shape changes to the frontend in this step.
*Verify:* grep proves no remaining bare `fs::write`/`fs::rename` on .md paths outside the gate (allow-listed: non-note files); full suite; manual smoke — typing/saving/renaming behaves identically.

## §B — CAS tokens (L2) in SHADOW mode + the cid backfill

**§B1 — Expectation plumbed.** `Expectation { expected_cid: Option<String>, base_mtime: u64, base_size: u64, base_hash: Option<String> }` (hash computed only on mtime-ambiguity — racy-git rule). `write_note` IPC gains an optional `expect` param. Gate, under the lock: identity mismatch → journal `would_refuse_identity` (SHADOW: proceed anyway, behind `WRITE_GATE_ENFORCE = false`); freshness mismatch → `would_refuse_stale`.
*Verify:* unit tests for all CAS verdicts; a deliberately-poisoned write journals `would_refuse_identity`.

**§B2 — Frontend passes expectations.** `writeNote` gains the optional expectation; the NotePane/NoteEditor save+flush path and `saveTabContent` populate it from what they last read (cid parsed at pane mount; base from the last read/write of that path). Surfaces that cannot yet attest (importers, templates) stay `unchecked` — visibly so in the journal.
*Verify:* svelte-check 0 errors; normal editing session journals `ok` outcomes with expectations present on editor saves.

**§B3 — cid_cn backfill (closes L2's biggest hole).** Resumable background walk (Rule-8 first-population shape, status-bar progress): inject `cid_cn` into every legacy note lacking it, via the gate. Idempotent, interrupt-safe, throttled.
*Verify:* DB query — 100% of note_meta rows carry cid_cn; re-run is a no-op; boot/typing unaffected (criterion §9 reasoning recorded).

★ **Stage 1 (Boss):** use Constellation normally for a session (type, save, rename from the sidebar, edit properties, run a Base edit). Tutorial will show where to see the journal's health line. Expected: behavior identical to today; journal shows `ok` everywhere, zero `would_refuse_*`.

## §C — Frontend single-snapshot composition + single store writer (L3)

> **RESTRUCTURED AGAIN 2026-06-12 (Boss ruling: Option B — the Buffer Pattern. PRIORITY One).**
> History: the §C monolith (6 changes, 1 commit) failed → reverted → restructured into §C-1..5
> step-gated pieces. §C-1 (WAB fail-closed) PASSED and stays. §C-2 (single store writer) failed
> its Boss gate — the journal + one-change diff named the root cause: ANY store announcement
> from the teardown flush re-enters the `{#key}` render. Research-first (Working Agreement #5,
> Boss order) across Obsidian / VS Code / CM6-author / Emacs-Vim returned the converged industry
> pattern: **buffers own content; views own nothing; teardown carries nothing.** Boss ruled
> Option B (full buffer pattern) over Option A (Obsidian-discipline minimal fix).
> Full mapping: ARCHITECT doc §7. Old §C-2..5 are SUPERSEDED by §CB-1..6 below.
> Protocol unchanged: one commit + one binary + one Boss gate per step; sandbox repro pre-gate
> on lifecycle-touching steps; no step starts before the previous passes.

**§C-1 — WAB fail-closed restore. ✅ SHIPPED + Boss-validated** (f194b282, 2026-06-12 gates 1–3 passed). Stays as-is.

**§CB-1 — The buffer registry (scaffolding, zero behavior change).** New `src/lib/editor/noteBuffers.ts`: non-reactive `Map<tabId, NoteBuffer{path, cid, props, body: Text, paneState?}>` + create/mirror/get/delete/compose API. All 9 live writer sites ALSO mirror into the buffer (inert Map writes — safe even inside teardown); `closeTab` deletes. Dev-only parity assertion (buffer compose === tab.content at save points) + vitest round-trip tests. Dead `updateTabContent` (store.ts:1024, zero callers) deleted. *Verify: svelte-check 0 / vitest green / typing perf unchanged. Boss gate: smoke — type, switch, property edit, stage, restart; NOTHING visibly changes.*

**§CB-2 — Saves compose from the buffer (identity travels with content).** `composeBuffer(tabId, expectPath)` becomes the ONLY content source for handleSave / handleFlush / handlePromote / PropertyEditor saves / saveTabContent: props + body from ONE object, REFUSED (journaled, not composed) when the callback's filePath ≠ buffer.path. freshProps()/freshBody() joins die. tab.content mirror updated at the same points via today's mechanics (direct mutation — no reactivity change in this step). *Boss gate: typing, property edit, stage change, second screen, restart.*

**§CB-3 — Panes mount from the buffer; the teardown flush DIES (the heart).** NotePane receives the buffer's `Text` (no string round-trip); updateListener maintains `buffer.body` per keystroke (O(1) ref assign); switch-time captures `view.state` into `buffer.paneState` — restored on return (UNDO SURVIVES TAB SWITCHES — new user-visible capability); `{#key}` recreation stays; handleFlush content-half deleted (cursor/scroll + WAB-from-buffer only). **Pre-gate: sandbox universe repro (typing burst + rapid switching + the §C-2 failure scenario) BEFORE any Boss build.** *Boss gate: the full §C-2 gate re-run + undo-across-switch demo.*

**§CB-4 — Readers re-point; `tab.content` retires.** All 30 reader occurrences (4 files) re-point to buffer accessors or metadata; `OpenTab.content` field deleted; reloadTabsFromDisk updates the buffer + bumps reloadVersion; WAB feeds/hydrates via the buffer. *Verify: grep-zero clause — no `.content` reads on tab objects remain. Boss gate: second screen, split view, search/preview surfaces, restart.*

**§CB-5 — FocusPane on the buffer + the per-keystroke disk-write fix (journal finding, 2026-06-12).** Focus onchange updates buffer.body + rides the same ≥1.5s debounced save; disk writes leave the keystroke path; mode switch NotePane↔Focus shares the buffer's Text — no string round-trip. *Boss gate: focus typing burst (journal shows ≤1 write per pause, not 170), mode switch round-trip.*

**§CB-6 — The side tab list (Boss request 2026-06-12: two ways to list open notes).** With openTabs metadata-only, add the optional vertical open-notes list (sidebar) as a SECOND viewport on the same list — switch/close/reorder parity with the top strip, zero content logic. i18n ×15 + RTL. *Boss gate: visual + both surfaces stay in sync.*

*Each step: own commit, own binary, Boss verdict before the next. §D (quiesce rename) then proceeds on buffer identity (cid-keyed, ≈ Obsidian's TFile).*

## §D — Quiesce protocol for rename/cascade (L4) + the title-rename re-land

**§D1 — `renameNoteQuiesced`.** The one rename path for sidebar AND title renames: (1) freeze the affected pane (read-only overlay + input blocked), (2) final flush through the gate, (3) `rename_item` + cascade as gate-locked operations, (4) deliberate reload/remount, (5) unfreeze. The path-keyed `isCascading` gate is superseded by the freeze (keyed to the tab) + the gate's locks; CAS refuses anything stale that still slips a write.
**§D2 — Title rename re-lands** through §D1 only — closing the original orientation-§13 gap (BUG-023's trigger) on safe foundations.
*Verify:* an automated lifecycle test replaying the BUG-023 sequence (A links B; B open; title rename; assert BOTH files' identities intact) plus the BUG-015 and F2 interleavings; full suites.

★ **Stage 2 (Boss):** the exact test that caught BUG-023, re-run — create the two probe notes, rename via the TITLE, inspect both files (tutorial spells the click path). Expected: link healed in A; B keeps its own identity; a brief freeze shimmer during the cascade is the only visible change. Plus a sidebar-rename regression pass.

## §E — Refusal / recovery / collision UX + i18n

**§E1 — The refusal surface.** Identity refusal → quarantine + a clear dialog (what happened, where the quarantined copy lives, open-folder button). Freshness conflict → Compare / Overwrite / Keep both (VS Code semantics; minimal compare = open both versions side by side). A Settings → diagnostics line: writes journaled / anomalies count.
**§E1b — The name-collision dialog (PJ-003, Eisa ruling 2026-06-11: "the conventional way").** Creating a note with an existing name, or renaming onto one, opens a modal — "A note named X already exists" with **Change name** (input pre-filled with the suggestion) / **Overwrite** / **Cancel**. Overwrite moves the existing note to `.trash` first (recoverable — reversibility holds under conventional UX), then proceeds through the gate. Replaces create's silent auto-suffix and rename's swallowed refusal in BOTH flows.
**§E2 — i18n ×15** for every new string (gated locale insertion).
*Verify:* forced-refusal dev test shows the dialog + produces the quarantine file; collision dialog covers create AND rename paths (Boss-staged); locale parse gates 15/15.

## §F — Enforcement flip + regression suite + audit + close

**§F1 — Soak then enforce.** After Stage 2 passes and a normal-use soak shows a clean journal, flip `WRITE_GATE_ENFORCE = true` (one-line, reversible — I10).
**§F2 — The regression suite** becomes permanent CI surface: gate unit tests (Rust) + lifecycle vitest (BUG-015/023/F2 replays + WAB verdicts).
**§F3 — Close per the Migration Rule:** /simplify on the full diff; 3-agent audit (invariants I1–I10, drift, migration path incl. first-boot, mid-backfill interrupt, rollback); docs (orientation vNext, help topic + User Manual on the recovery surface, PJ ledger); milestone tag + ZIP.

★ **Stage 3 (Boss):** enforcement on — normal use + one deliberate conflict scenario (tutorial-staged) to see the refusal dialog work; final verdict closes MIG-076.

---

**Estimated effort:** §A+§B ≈ 1.5 sessions · §C ≈ 1.5 · §D ≈ 1 · §E ≈ 0.5 · §F ≈ 1 → **~5–6 working sessions**, each § a separate commit, Boss gates at the three ★ stages. Rollback story: every layer behind its own seam (gate unconditional → shadow → enforce), no schema change outside the additive journal table.
