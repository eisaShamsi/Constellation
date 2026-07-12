# Session Log — 2026-07-12 — PJ-070 (Watcher External-Change Adopt)

**Function in hand:** the main-window file-watcher's external-change reconciliation — what Constellation does when a note's `.md` is changed *outside* the app while that note is open.

**Outcome:** PJ-070 — the silent-clobber APP-KILLER — SHIPPED + Boss-validated end-to-end + `/migration` CLOSED. First Group-1 item off the re-prioritized backlog (Pending Jobs v1.18 → v1.19).

---

## The bug

Under Single-Ownership (MIG-076) the open editor's source of truth is the in-memory note **model**, not the store's `tab.content`. The watcher `library-changed` flush (`+layout.svelte`) updated only `{ ...t, content }` — it never called `adoptDisk` into the model and never bumped `reloadVersion`. So an external edit (git-pull / Syncthing / Obsidian) to an OPEN note left the mounted editor + model **stale**; the next keystroke marked the stale model dirty and the debounced `editor_save` durably **overwrote** the external edit, then reindexed so search agreed with the stomp. The main-window mirror of the closed G3 cross-window class.

## `/migration` — the four phases

- **SO#8 cross-check — PASS.** Both PJ-070 sites confirmed (watcher flush `+layout.svelte:~3218`; `SecondScreenPage.adoptFreshDiskIntoSS:735` wrapping `noteModel.adoptDisk:265`). Line numbers had drifted a few lines from the PJ's `:3172`.
- **Reproduce-First.** Recipe O added to `tests/mig-076/runtimeHarness.test.ts` — RED (no adopt → clobber), GREEN (`adoptDisk` preserves both), DIRTY (local wins). Running-app reproduction recipe in `lab/reports/PJ-070-reproduction.md`. `svelte-check`/`vitest` are NOT runtime verification for this class — the running-app Boss test closed the wiring residual.
- **Architect** (`docs/PJ-070-Watcher-External-Change-Adopt-Architect.md`, workflow `wf_2c3313ab-542`, 12 agents: census ×4 + WA#5 prior art + 3 competing designs + 3 adversarial refuters + synthesis). WA#5: VS Code / IntelliJ / Sublime all do "reload-if-clean, keep-both-if-dirty"; Obsidian NOT reloading is the documented data-loss cautionary tale = our exact bug. The refutation caught two teardown/Focus hazards (the 2026-06-12 corruption class) as mandatory Plan items.
- **Boss decisions:** Option **B** (shared, harness-testable store helper) over A (inline, untestable) / C (overload the load-bearing cascade primitive — an APP-KILLER risk on a safe path). Conflict policy: **banner + `.conflict` side-copy** (zero loss) over banner-only.
- **Plan** (`docs/PJ-070-Watcher-External-Change-Adopt-Plan.md`, workflow `wf_c13e1f13-972`: planner + coverage + feasibility, both "ready-with-fixes" — ~15 fixes folded in, incl. the `.md.txt`-final sidecar-inertness decision, the `diskBaseline` conflict-detection signal, and all-15-locale i18n).
- **Build — §1–§6** (commit `b1a3e388`):
  - §1 `noteModel`: `diskBaseline` + `noteDiskSynced` (re-baseline on durable save, path-guarded) + `diskDiffersFromBaseline`. **Plus the class fix** (see below).
  - §2 `store`: `adoptExternalChangeIntoTabs(paths, hooks, readDisk)` — clean → `adoptDisk` + `reloadVersion`; dirty + `diskDiffersFromBaseline` → `hooks.conflict`. Dedicated **refcounted** `reseedingPaths`/`isReseeding` gate (try/finally) spanning the async `{#key}` teardown (hazard #6). NoteEditor flush gates honor it. Genuine store-boundary test `tests/mig-076/watcherAdoptStore.test.ts`.
  - §3 Focus (hazard #7): FocusPane keyed on `focusReloadVersion` + `focusReseedSuppress` gating every teardown-write path (all funnel through `onflush`).
  - §4/§5: watcher flush + `onNoteSaved` both routed through the helper via `adoptExternalHooks()`; the `onNoteSaved` fold closes a WA#6 sibling gap (it adopted the model but forgot the remount). Adopt runs BEFORE the reindex/stats awaits to narrow the race.
  - §6: `write_conflict_sidecar` (Rust) → `<stem>.conflict-<UTCz>.md.txt` via `gate_create_exclusive` (RefusedExists → `-N` retry); `saveConflicts` store; conflict rows on `SaveHealthBanner`; `conflict.*` i18n ×15 (RTL-aware) via workflow `wf_bfdbef80-4ab`.
- **The `setBody` CLASS FIX (A1, from `/simplify` + the safety-inspection finding [7]).** A merely-VIEWED note's teardown flush pushed its unchanged body via a string `editBody`, which — because a fresh `Text` is never ref-equal — spuriously bumped version → `isDirty` while content equalled disk. That silently **defeated `adoptDisk`** (refuses on dirty) on every background/focus tab (reintroducing the clobber there) AND raised **phantom `.conflict` sidecars** AND churned untouched notes on universe switch. Fixed at the source: `setBody`'s STRING path no-ops an identical-content push; the keystroke `Text` path keeps the O(1) ref check (Rule 1). Closes NotePane + Focus + `flushAllDirtyTabs` in one place (the surface-level `handleFlush` gate was reverted in favour of this root fix).
- **`/simplify`** (4 agents): applied 7 — the O(changed+open) Set intersection, collapsed `adoptedIds`/`adoptedPaths` → one Set, scalar `focusReseedPath`, extracted `adoptExternalHooks()`, Rust name collapse, refcounted reseed gate, and the A1 class fix. Skipped 5 (negligible/style/out-of-diff; the SS shared-primitive → PJ-084).
- **Audit** (workflow `wf_352f1a07-ec7`, 3 agents): **clean** on all three — all 11 invariants + the class-fix invariant HOLD; no drift; migration-path safe. 2 low hardenings applied (commit `cd5e53fd`): reseed clear in try/finally; honest flag-off rollback-scope comment.

## Safety-inspection register (per-cycle whole-app sweep, `wf_1b7addb3-822`, 38 agents, 15 confirmed)

1 finding in the PJ-070 diff (finding [7], the spurious-dirty class) — **fixed pre-commit** via the class fix. The other 14 are pre-existing: filed **PJ-085** (composeFrontmatter H1 passthrough, HIGH), **PJ-086** (switchTab flush gap, HIGH), **PJ-087** (universe.rs shared-tmp race); the rest map to PJ-074/075 or are LOW batch items. Register appended to the Charter.

## Boss validation (running app, fresh binary 14:34)

- **Stage 1 (core fix):** created `PJ-070 test` in "Eisa Cognitive Knowledge" / "Eisa Test"; I wrote an external line from outside; the open note **adopted it live** (Boss watched); Boss typed on top; disk-verified all three lines survived + `cid_cn` intact. **PASS.**
- **Stage 2 (conflict net):** Boss typed continuously (dirty) while I landed a conflicting external edit; a **banner** appeared ("…kept as a separate copy — your version is unchanged" + Show copy), the Boss's typing stayed in the editor, and a `PJ-070 test.conflict-20260712T113943Z.md.txt` sidecar held the outside version (journalled `created_exclusive`; 0 `.conflict.md` files → inert). Both versions preserved. **PASS.**
- Second-screen (§5): not separately live-tested (same adopt mechanism as Stage 1; covered by the store-boundary test + audit).

## PJ-072 lead

The Boss's active "Eisa Cognitive Knowledge" universe resolves to on-disk root **`E:\Cognitive Knowledge\`** (write-journal-confirmed via the test note's writes), distinct from `E:\Constellation Universes\Eisa Cognitive Knowledge\`. WHERE the data lives is now known; the diagnostic build is still wanted for WHERE the name→root mapping persists. Charter + Pending Jobs v1.19 updated.

## Ledger (SO#9)

Reconciled FIRST at close → `docs/Constellation Pending Jobs v1.19.md`: PJ-070 Done (evidence); PJ-083→087 filed; ► Next action → **PJ-071**.

## Post-close follow-up — "Show copy" reveal-in-explorer fix (Boss-found, 2026-07-12)

Boss test of the conflict banner's **Show copy** button: it opened "My Documents" instead of revealing the `.conflict` side-copy. Root cause: `constellation_show_in_folder` (lib.rs) built `explorer /select,<path>` via `.arg()`, which auto-quotes the WHOLE `/select,<path>` token when the path has spaces (`E:\Cognitive Knowledge\Eisa Test\…`) → explorer can't parse the `/select,` flag → falls back to Documents. **Fix:** `raw_arg(format!("/select,\"{}\"", win_path))` — quote ONLY the path + normalize `/`→`\`. Fixes the same latent bug in all 7 reveal callers (file-explorer right-click, NoteEditor, etc.). cargo check clean; release binary 2026-07-12 18:03. **Boss-validated: "Passed perfectly"** — Show copy now opens the folder with the file highlighted.

**Boss follow-up request (deferred as a feature):** the option to MERGE the two copies (note version + `.conflict` side-copy) — the user's choice. This is the conflict-RESOLUTION layer the Architect explicitly deferred ("the full 3-way Adopt/Keep-mine/Show-diff dialog is a separate follow-up PJ"). Being scoped as a new PJ; PJ-070's zero-loss safety net stands complete + validated.

## Verification snapshot

svelte-check **0 errors** · vitest **334 passed** (13 new) · cargo check **clean** · release binary **2026-07-12 14:34** (frontend rebuilt first, conflict strings confirmed embedded).

---

## PJ-088 — conflict-resolution side-by-side MERGE view (Boss-requested follow-up, same day)

After PJ-070 closed, the Boss asked for the option to **merge** the two conflict versions ("the user's choice to decide") — the resolution layer the PJ-070 Architect had deferred. Boss chose: shape = **side-by-side**, **build now**, panes = **full live preview**.

- **Design (Art Director team, per the standing ruling).** Workflow `wf_d7453254-50e` (11 agents: 3 census — incl. the safety-critical save-wire map — + WA#5 prior art + 3 competing designs + 3 adversarial judges + synthesis) → `docs/PJ-088-Conflict-Merge-Design.md`. Chosen: `@codemirror/merge` `MergeView` (2-way — no common ancestor stored; lazy-imported, 29KB chunk, Rule 6) in a full-center overlay; live-preview panes; per-chunk copy-across.
- **Build (§1–§5).** `mergeView.ts` store; `ConflictMergeView.svelte` (the overlay); banner **Merge…** button (+ expose `notePath`); mount in `+layout` (with `focusReseed`); **the safety wire `resolveConflictMerge`** (store.ts) — writes the merge through the model + durability gate, never a raw write; sidecar→trash + dismiss only after durable success; Cancel = pure no-op. `conflict.*` (13 keys) ×15.
- **The one in-diff safety-inspection finding, FIXED pre-commit.** The whole-app sweep (`wf_c0dac305-85e`, 40 agents, 19 confirmed) flagged: the merge save pushed props via `editNoteProps` but left `m.base` stale → compose diffed the merge against the open-time base → non-projectable frontmatter (nested maps/block scalars) the merge changed was silently dropped. Fixed at the model layer: new **`replaceContent`** re-bases to the merged source so compose emits it verbatim. Reproduce-First: runtimeHarness **Recipe P** (nested-map removal sticks). The other 18 findings are pre-existing (2 new HIGH filed: **PJ-089** Index-preview two-writable-model clobber, **PJ-090** SS Tasks-panel toggle no-broadcast).
- **Boss UX feedback → refinement.** First merge test PASSED but the default `@codemirror/merge` gutter chevron "wasn't noticeable." Replaced via `renderRevertControl` with a prominent **◀ Copy to mine** accent button per chunk (the mockup's wording); `conflict.copyAcross` ×15.
- **Boss-validated end-to-end.** The two-column view renders with live preview (the spike worked — clear highlighted diffs, unchanged lines folded); the **◀ Copy to mine** button pulls the outside chunk in; **Save merged** writes the reconciled note (disk-verified: all lines + `cid_cn` intact) and moves the side-copy to `.trash` (recoverable); the banner clears. Commits `bc6a1e43` (§1–§5) + `59295333` (button). svelte-check 0, vitest **335**, cargo clean.
- **Show-copy note:** the PJ-070 "Show copy" reveal was also fixed this session (`621fffaf`, spaced-path explorer-select) — Boss-validated separately.

## PJ-072 note

The whole session's Boss tests ran against the `E:\Cognitive Knowledge\` universe root — reconfirming the PJ-072 lead (the active "Eisa Cognitive Knowledge" universe lives there, not `E:\Constellation Universes\...`).

---

## PJ-071 — bulk Accept-All read-modify-write race (Group-1 continuation, Boss "Proceed")

**Function in hand:** the CECE bulk "Approve All sources" path — `accept_one` (`sources/bulk_ops.rs:269`).

- **SO#8 cross-check — PASS.** Both sites confirmed: `accept_one` reads unlocked (line 305) then `gate_write` (line 310); `gate_rmw` exists (write_gate.rs:627); the per-card path already uses it (`sources/mod.rs:539`).
- **The bug.** The read→modify→write was not atomic — `gate_write` locks only the write, not the disk read at 305. A concurrent editor `write_note` (dispatch thread) landing between the unlocked read and the gated write is silently overwritten by the stale-based frontmatter rewrite; no error, no sidecar.
- **The fix (proven-pattern migration, no `/migration` — single write-path fn).** Replaced the unlocked read + `gate_write` with one `gate_rmw(path, "bulk_accept", |content| { rewrite_frontmatter_sources → rewrite_frontmatter_content_type; Ok(None if unchanged else Some) })` — read+mutate+write under the per-path lock the editor also takes, so a save lands before or after but never inside. Runs on the existing `thread::spawn` worker (no dispatch freeze; gate_rmw's two rules honoured — pure closure, DB-mirror update after). Behaviour-preserving + idempotent-skip.
- **Reproduce-First / verify.** The race window is visible in the old code; the fix inherits `gate_rmw`'s proof (`concurrent_writers_serialize_never_tear` + the gate_rmw unit tests, in the 22 write_gate tests that pass). cargo check clean; 31 sources tests + 22 write_gate tests pass. Backend-only, no user-visible change → no live Boss test needed.
- **Per-build whole-app sweep (`wf_4dd12a39-694`, 46 agents, 24 confirmed):** the PJ-071 diff's OWN gate_rmw change = **ZERO findings**. ONE new HIGH in the same function (a DISTINCT bug — accept REPLACES a note's manual multi-value `sources:`/`content_type:` with the suggestion's ids because the suggestion builder `classifier/mod.rs:128-148` drops `.secondary`) → filed **PJ-091** (needs a classifier-synthesis look + a Boss ruling on accept semantics; NOT fixed inline — a separate concern from the race + a design decision). The other 22 are pre-existing backlog.
- **Close (SO#9):** Pending Jobs **v1.21** (PJ-071 Done, PJ-091 filed, ► Next action → PJ-091); Orientation **v3.42**; Charter register appended.

---

## PJ-091 — accept silently truncated manual multi-value frontmatter — FIXED (2026-07-12)

**Function in hand:** the CECE "accept a classifier suggestion" path — bulk "Approve All" (`accept_one`, `sources/bulk_ops.rs`), per-card Accept (`SourceReviewPanel.acceptSuggestion` → `sources_set_manual`/`content_type_set_manual`), and `cece_resolve_disambiguation`.

**Boss ruling (AskUserQuestion, 2026-07-12):** *merge — never lose a manual value* (+ fix the dropped-`.secondary`). Then: *tackle PJ-091 now.*

- **SO#8 + Explore map (`Map CECE accept + suggestion paths`).** Confirmed: truncation is REACHABLE via a STALE suggestion (queued when the note had fewer values; user types more directly in the editor — which does NOT clear the suggestion; the scan skips notes with a pending row, so the stale suggestion survives; Approve-All then replaces). ALL callers of `set_manual`/`content_type_set_manual` are accept-class (per-card + disambiguation) — there is NO direct-set/PropertyEditor caller today; the exact-set primitive is only used by clear + (latently) any future direct-edit. Readers `extract_sources`/`extract_content_type` already exist. `startEdit` seeds the edit UI from the suggestion ONLY (never shows current manual values) → the edit-override path also truncates.
- **Root cause (sharp).** "Accept" reused the exact-set primitive `set_manual(suggestion)` — a machine proposal treated as the user's exact manual assertion → REPLACE.
- **Reproduce-First.** Deterministic frontmatter logic → a Rust test is the on-demand reproduction. `pj091_repro_accept_replace_truncates_manual_multivalue` (proves `[testimony, perception]` + suggestion `[testimony]` → `perception` dropped) — RED confirmed (passes = bug reproduces).
- **The fix (single structural end-state — union at every accept seam, under the write lock).** `union_preserve_order(existing, additions)` (existing-first, append new). Default-off `merge` flag through `rewrite_note_sources_on_disk`/`rewrite_note_content_type_on_disk` (return the effective merged set) → `sources_set_manual`/`content_type_set_manual` (mirror the effective set to `note_meta`). Bulk `accept_one` unions both axes inline in its one dual-axis `gate_rmw`. Per-card (plain + edit-override) + disambiguation (×4) pass `merge:true`. Exact-set preserved for clear + future direct-set. **Part A:** `build_axis_suggestions` (DRYs the H/V builder) now carries the classifier's `.secondary`.
- **Verify.** 4 PJ-091 tests pass; sources 32 / classifier 15 / write_gate 22 green; svelte-check 0 errors; `/simplify` applied (removed RefCell machinery + 2 clones — `gate_rmw`'s FnOnce closure captures a plain `let mut`); release binary rebuilt (npm build → cargo release, `merge:!0` confirmed embedded).
- **Whole-app sweep (`wf_f2a07366-fc5`, 37 agents, 17 confirmed).** PJ-091 diff = **ZERO findings**. ONE NEW APP-KILLER → **PJ-092** (`flushAllTabsInLibrary` rename-cascade silent edit-loss; fix = mirror `renameItem`'s `renameFlushOk` gate). NEW MED→HIGH → **PJ-093** (reindex-skip-when-db-None + reindex-error swallow). Other 15 → PJ-074/075/073/085/087/077 + 3 LOWs (Charter register appended).
- **Close (SO#9):** Pending Jobs **v1.22** (PJ-091 Done; PJ-092/093 filed; ► Next action → PJ-092 the app-killer); Orientation **v3.43**; Charter register appended. **PJ-091 is user-visible but deterministically proven — a live Boss test is offered as optional confirmation (fiddly stale-suggestion setup), not a gate (as with PJ-071).**
