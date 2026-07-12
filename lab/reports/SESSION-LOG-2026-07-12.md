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

## Verification snapshot

svelte-check **0 errors** · vitest **334 passed** (13 new) · cargo check **clean** · release binary **2026-07-12 14:34** (frontend rebuilt first, conflict strings confirmed embedded).
