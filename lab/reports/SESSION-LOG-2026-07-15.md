# Session Log — 2026-07-15

Continues the PJ-106 arc from 2026-07-14/15 (Part-A core, orientation v3.51, ledger v1.30).

---

## PJ-106 Part B — selection commands (Boss chose "selection commands first")

Built the selection half of Part B as three staged, tutorial-tested increments. Each was
built (svelte-check 0 + full vitest + `npm run build` before `cargo build --release` + binary
mtime verified) and committed **only after** Eisa's live-test PASS on the running release binary.

| § | Feature | Keys | Commit | Boss test |
|---|---------|------|--------|-----------|
| **B1** | Paragraph navigation | Ctrl+↑/↓ (+Shift to select) | `c0d668fc` | PASS |
| **B2** | Select line / paragraph block | Ctrl+L · Ctrl+Shift+L | `10abf799` | PASS |
| **B3** | Select sentence | Ctrl+click · Ctrl+Shift+S | `53b22c07` | PASS |

- **B1** — new `src/lib/editor/paragraphNav.ts` (pure-offset, direction-blind, parser-free).
  `Mod-ArrowUp/Down` verified free in CM6 `defaultKeymap` AND the app command registry (only
  `Alt+←/→` nav-back/forward + `Alt+↑/↓` move-line taken); window handler `+layout.svelte:4150`
  finds no command → no double-fire.
- **B2** — extended `paragraphNav.ts`: `Ctrl+L` select line, `Ctrl+Shift+L` select the whole
  blank-delimited paragraph block. Both **text-only** (§B0 rule — no trailing newline). CM6's own
  `Alt-l selectLine` includes `to+1` (the exact §B0 RTL bug) → overridden to the text-only select
  (WA#6). `select-page` (Shift+PageUp/Down) + `select-all` (Ctrl+A) already in CM6 → verify-only.
- **B3** — new `src/lib/editor/sentenceSelect.ts`. `Ctrl+click` → sentence (via
  `EditorView.mouseSelectionStyle`, consulted before CM6's Mod+click multi-cursor at
  `view/index.js:4865` — deliberately replaces multi-cursor per the Round-4 ruling);
  `Ctrl+Shift+S` keyboard command (Boss-confirmed it fires on the **Arabic keyboard**).
  `Intl.Segmenter` (UAX #29) used **unconditionally** — breaks on ؟ ۔ ! . but NOT ؛, no decimal
  false-break (design-inspection H4 verified live on V8). Segmenter typed locally (compiles
  regardless of tsconfig `lib`); degrades to whole-line select if ever absent.
- All three wired into **NotePane + FocusPane + ConflictMergeView** behind `RTL_MOTION_ENABLED`
  (Editor Parity, one rollback lever). Tests: `tests/pj-106/paragraphNav.test.ts` (19) +
  `sentenceSelect.test.ts` (10) — 40 pj-106 tests total, all green; svelte-check 0.
- **Note:** the full-suite vitest showed a flaky failure in `tests/sight-v6/perf.test.ts` (a tight
  32 ms render-budget assertion; measured ~49 ms under concurrent build CPU load). Unrelated to
  the editor; passes clean (4/4) in isolation. Not a B3 regression.

**Boss's Round-1 selection list is now fully covered:** word (double-click) · sentence
(Ctrl+click / Ctrl+Shift+S) · line (Ctrl+L) · paragraph (Ctrl+Shift+L / triple-click) ·
page (Shift+PgUp/Dn) · all (Ctrl+A).

---

## STATE OF STANDING (SO#5 — recorded before the B4 pause / PJ-108 pivot)

Eisa ruled: **pause B4, pick up PJ-108 now, build B4 after Jul 18.**

- **(a) Verified-shipped & protected:** PJ-106 Part-A core (§A1/§B0/§A3/§A2/§A5, prior session) +
  Part-B selection commands (§B1/§B2/§B3, this session, 3 commits, all Boss-validated). `main`
  clean, `HEAD == origin` after each commit. Rollback lever `RTL_MOTION_ENABLED` covers all of it.
- **(b) In-flight / deferred:** **PJ-106 §B4** — the per-paragraph direction override
  (Right-Ctrl+Shift → 100% RTL, Left-Ctrl+Shift → 100% LTR, persisted as an invisible RLM/LRM
  leading mark). **DEFERRED to Jul 18** (Boss ruling). It is the one Part-B step that **writes to
  the note** (content-integrity class), so it needs the diff-scoped `safety-inspection` (weekly
  agent limit resets **Jul 18 4am**) + the 8-point Editor-Surface Gate. Two open WebView2 unknowns
  to spot-check first: (1) does the app reliably distinguish `ControlLeft` vs `ControlRight`
  (`KeyboardEvent.code`); (2) does Windows consume Ctrl+Shift as an OS keyboard-layout switch
  before the WebView sees it (→ fallback binding: toolbar button + alternate shortcut). Plan:
  `docs/PJ-106-RTL-Typing-PLAN.md` §B4 + amendment SI4-06; symptoms doc Round 3. §A4 (isolate
  ranges) remains Reproduce-First-gated on the callout-caret repro (symptoms ②/③ already PASS).
- **(c) Known-broken / picked up now:** **PJ-108** (APP-KILLER) — a second-screen `[[wikilink]]`
  click destroys an unsaved, save-failed note's crash-recovery net. Working it next this session
  (Reproduce-First → fix → Editor-Surface Gate + hand-review in lieu of the rate-limited
  inspection → Boss test → commit).
- **(d) Pending, not started:** rest of Group 1 (PJ-103 app-close flush · PJ-104 → PJ-072 ·
  PJ-105 template raw-write · …); Group 3 paused SS-Cockpit Parts B–F.
- **(e) Doc drift:** ledger v1.30 + orientation v3.51 still show PJ-106 Part B as pending — to be
  reconciled at session-close PCS (ledger → v1.31 closing the selection half, orientation v-bump).
  **PJ-107** stays parked (Boss "closed", polish-class).

---

## PJ-108 — a read-only second-screen note-open destroys the crash-recovery net (APP-KILLER)

**The bug (confirmed against live code, SO#8).** The write-ahead net (memory + shared localStorage,
`store.ts:250-298`) is the ONLY copy of an unsaved, save-failed note's edits. The second screen is a
DISPLAY-ONLY window (separate JS context + own `openTabs`), but it shares the net via localStorage.
Every note-open in the SS runs the store's `openNoteTab → resolveNoteContent`, which CONSUMES the net
(`clearWriteAhead`, `:2004/:2014`) unless `preserveNet` is set — and a read-only window never mounts a
writable editor to re-stash it. So opening a note in the SS silently destroys the main window's only
recovery copy of a save-failed note; disk holds the pre-edit body; nothing surfaced. The SS's separate
`openTabs` means `openNoteTab`'s dedup early-returns never fire for a main-window-open note → the
resolve always runs. (The `handleLinkClick` at `NoteEditor.svelte:448` was the only link handler with
no `readOnly` belt; the restore path `:2450` was already `preserveNet:true`.)

**Reproduce-First (RED→GREEN).** `tests/mig-076/readonlyLinkPreservesNet.test.ts` (Recipe RO) drives the
REAL `openNoteTab` against the mocked IPC bridge: RO1 documents the wound (a plain open consumes the
net), RO2 the `preserveNet` contract, RO4 the window-flag Solve-the-Class path, RO5 the precedence.
Confirmed RED before the fix (RO2 failed: "expected undefined to be truthy").

**The fix — Solve-the-Class (single display-only-window flag), not per-call-site.** A completeness review
(WA#4, standing in for the rate-limited `safety-inspection`; my own trace + an adversarial agent) found
THREE sibling SS entrypoints that bypass `handleLinkClick` and call `openNoteTab` directly with no
`preserveNet`: the Dashboard note-list click (`SecondScreenPage.svelte:1261`), the split-companion Tasks
file-link (`TasksPanel.svelte:87`), and the session-restore replay (`:717`). Rather than patch each
(the fragile pattern that caused the bug), a store-level `displayOnlyWindow` flag (`setDisplayOnlyWindow()`,
set once at SS init) makes `openNoteTab` default `preserveNet` to it — so EVERY SS note-open (current +
future) preserves the net; the main window's flag stays false (separate context) → writable consume-and-
re-stash unchanged. Plus: `handleLinkClick` keeps a `readOnly` belt (defense-in-depth for a read-only
NoteEditor in the MAIN window) and now never CREATES a note from a read-only display (a second, smaller
Display-not-Domain leak — an unresolved link in the SS is inert).

**Completeness review — refuted as SAFE (traced, no net mutation):** `closeTab`/`switchTab` (flush is a
no-op on a never-dirty read-only tab; no `clearWriteAhead` in the path), the peek preview (`read_note`
into a synthetic tab, no `resolveNoteContent`), `adoptFreshDiskIntoSS`/`onNoteSaved` (clean-only adopt),
and the delegate-to-main actions (`sendNoteToMain`/`requestNoteActionOnMain`). The SS `NoteEditor`
teardown can't re-stash (every write callback early-returns on `readOnly`).

**Files:** `store.ts` (flag + `openNoteTab` preserveNet default), `NoteEditor.svelte` (handleLinkClick
belt + no-create), `SecondScreenPage.svelte` (`setDisplayOnlyWindow()` at init), Recipe RO test +
`vitest.config.ts`. Gates: svelte-check 0; vitest 395 (32 files); the recovery-net/mig-076 suite 78 → 80
green (no regression).

**→ Boss live-tested + PASSED + committed 2026-07-16 — full arc in `SESSION-LOG-2026-07-16.md`**
(a real-lock, real-force-kill crash-recovery run through the split-companion Tasks door; the edit
survived on screen and on disk).
