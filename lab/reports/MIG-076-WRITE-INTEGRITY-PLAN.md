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

**§C1 — The snapshot.** NotePane becomes the sole composer for its file: at save/flush it builds one immutable `{path: mountedFilePath, cid, base, content, docVersion}` from its own epoch state (its doc + the embedded PropertyEditor's props routed through the pane). `freshProps()`-by-tab-id joins in NoteEditor are abolished; handleSave/handleFlush submit the snapshot as-is. `docVersion` guards the dirty-clear (no marking-clean over keystrokes typed mid-save).
**§C2 — One store writer.** A single `updateTabContent(tabId, content, origin)` authority replaces the six direct `tab.content =` mutation sites (PropertyEditor ×2, NoteEditor ×2, FocusPane, second-screen listener); each origin is journal-tagged in dev. The WAB identity check flips **fail-closed** (buffer restored only when both cids present AND equal). The sidebar PropertyEditor re-seeds keyed on path (not tabId) and submits CAS-carrying saves.
*Verify:* svelte-check; the 10-keystroke typing test (Perf Rule 7) in NotePane + FocusPane; grep proves zero direct `.content =` outside the authority; a vitest covering the WAB fail-closed verdicts.

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
