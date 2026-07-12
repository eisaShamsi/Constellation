# PJ-070 — Watcher External-Change Adopt — Plan

*/migration Phase 2 · 2026-07-12 · Boss-ratified: **Option B** (shared store helper) + **conflict policy: `.conflict` sidecar + banner**. Plan drafted + adversarially verified (workflow `wf_c13e1f13-972`: planner + coverage + feasibility, both "ready-with-fixes" — all ~15 fixes folded in below).*

## Design (locked)
- **Shared helper** `adoptExternalChangeIntoTabs(paths, hooks?, readDisk?)` in `store.ts` — the ONE freshness-gated adopt both main-window ingress paths call (the watcher flush + the `onNoteSaved` SS→main path). Injectable `readDisk` so the harness drives it (genuine Recipe O RED→GREEN). In-repo prior art: `SecondScreenPage.adoptFreshDiskIntoSS` (the read-only SS mirror).
- **Conflict policy** (dirty model / debounce-race): never clobber local work; write the incoming disk to a **`<stem>.conflict-<UTCcompactZ>.md.txt`** sidecar (the `.txt` final extension makes it inert to every `.md`-gated surface — `watcher.rs:79`, `index_note` `search.rs:5662`, tree walker `libraries.rs:1739` — so no index/tree pollution, no clobber loop, no duplicate `cid_cn`), + a non-blocking banner.
- **Conflict detection**: add `diskBaseline: string` to `NoteModel` (the exact bytes last synced with disk); a genuine external edit while dirty = `isDirty(id) && incomingDisk !== diskBaseline`. Full-string compare (zero false-negative — a hash collision could silently drop a real conflict = data loss).
- **Feature flag** `WATCHER_ADOPT_ENABLED = true` (mirror `NAV_FLUSH_ENABLED` store.ts:216). **false = today's content-only clobber — a rollback lever, NOT a safe steady state** (documented as such).

## Verified primitives
`gate_create_exclusive` (write_gate.rs:457, refuse-if-exists) · `constellation_show_in_folder` (lib.rs:111) for "Show copy" · flush gate `isCascading(filePath)` (NoteEditor.svelte:301) where the reseed-gate hooks in · flag pattern store.ts:216.

## Build steps (each = one commit, verification clause, rollback)

### §1 — `diskBaseline` primitive (model layer)
- `noteModel.ts`: add `diskBaseline: string` to `NoteModel`; set = content in `openModel`, = diskContent in `adoptDisk` (adoptDisk signature/return **unchanged** — Recipe N/O stay green). Add `noteDiskSynced(id, content, expectPath?)` (path-guarded) + `diskDiffersFromBaseline(id, disk)`.
- `noteSession.ts`: in `save`'s success block, after `markSaved`, call `noteDiskSynced(id, r.content, r.path)` so a durable write re-baselines.
- Harness: add a discriminator recipe (dirty + differs → conflict; dirty + same → no conflict).
- **Verify:** vitest 0-fail (Recipe N/O + discriminator); svelte-check 0/0. No live wiring → app unchanged. **Rollback:** additive; revert one commit (save re-baselines to what it already wrote — zero behavior change).

### §2 — the shared helper (unwired) + genuine Recipe O + the reseed-gate
- `store.ts`: `WATCHER_ADOPT_ENABLED = true`; `export async function adoptExternalChangeIntoTabs(paths, hooks?, readDisk = readNote)`. Logic: flag-off → today's `{...t, content}` update + return. Else intersect `paths` with open tabs (inv #9); per path read once via `readDisk` with `.catch(() => null)` (inv #8, deleted file can't reject batch); per tab — `if (isCascading(t.path)) continue` (inv #3); `if (externalChange(t.id, disk))` [= adoptDisk, inv #2 — never `reloadTabsFromDisk`/`openNoteModel`] → **if `t.path === hooks.focusPath && focusMode` → `hooks.focusReseed(t.path)` instead of a reloadVersion bump** (inv #7 handoff); else collect for ONE batched `openTabs.update` bumping `reloadVersion` **only on adopters** (inv #4/#5) + `clearWriteAhead(t.path)` **only on adopters** (inv #10); `else if (isDirty(t.id) && diskDiffersFromBaseline(t.id, disk))` → `await hooks?.conflict?.(t.path, t.name, disk)` (stub until §6).
- **Hazard #6 reseed-gate (dedicated, concurrency-safe):** a per-path `reseedingPaths` set + `isReseeding(path)` in store.ts; mark before the `openTabs.update`, **`await tick()`** (from `svelte`), then clear. Extend NoteEditor's flush gates (`doFlush` :243, `handleFlush` :301) to `isCascading(p) || isReseeding(p)`. *(Dedicated flag — NOT the shared cascade flag — so a concurrent rename cascade's clear cannot lift it. Reuse `markCascading` only if build-time inspection confirms the cascade's clear is strictly per-path-refcounted; default = dedicated.)*
- **Recipe O rewrite (the named deliverable):** import + drive `adoptExternalChangeIntoTabs` against a seeded `openTabs` + fake `readDisk` — RED (flag-off) clobbers; GREEN adopts + bumps reloadVersion **only on adopters** + routes a dirty tab to the conflict hook; DIRTY local-wins.
- **Verify:** vitest genuine RED→GREEN; svelte-check 0/0; **diff-scoped safety-inspection** over `store.ts` + `NoteEditor.svelte` + `/simplify`, every finding fixed. Called by nothing yet → app unchanged. **Rollback:** flag=false or revert.

### §3 — Focus suppressed-reseed machinery (hazard #7 — the 2026-06-12 site)
- `+layout.svelte`: `focusReloadVersion` + `focusReseedSuppress` near `focusSessionId` (~1488); wrap the `{:else if focusMode}` FocusPane branch (7855-7883) in `{#key focusSessionId + '|' + focusSessionPath + '|' + focusReloadVersion}`. Gate **every** FocusPane teardown write path with `if (focusReseedSuppress) return;` — `onchange` (7867), `onflush` (7881), AND FocusPane's `beforeunload`/`visibilitychange`/idle flush (`FocusPane.svelte:47-55,215-217,232-240`). Provide `focusReseed(path)` = suppress → bump `focusReloadVersion` → `await tick()` → unsuppress.
- **Verify (running-app Editor-Surface-Gate #2 + #4):** Focus enter→type→exit (body intact, **no spurious enter write** in journal); external edit while IN Focus → FocusPane reseeds, old teardown writes **no** stale body (screen===disk); **window-blur DURING the reseed** → gated (no stale `focus_pane` write). Safety-inspection. **Rollback:** revert → FocusPane un-keyed (helper simply won't remount a focus tab; model adopt still no-ops on a dirty focus note).

### §4 — wire the watcher flush → the helper
- `+layout.svelte:3218-3230`: replace the bare read+`{...t, content}` loop with `await adoptExternalChangeIntoTabs(tabPaths, { conflict, focusReseed, focusPath: focusSessionPath })`, positioned **before** the reindex/`loadAllStats` awaits (above 3207) to narrow the debounce-race (inv #11) while preserving the tree/reindex/stats side-effects (3193-3216). Keep `wasRecentlyWritten` (3245); Rust `watcher_suppress` untouched (inv #1).
- **Verify (running-app):** (a) NotePane external edit → adopts, no clobber, no data loss; (b) **hazard #6** — remount shows zero stale `editor_save`/`editor_flush` at the remount instant, screen===disk; (c) **inv #3 / Gate #6** — rename open note B (cascades wikilinks in open note A) → watcher does NOT adopt mid-cascade, both `cid`s intact, exactly one remount of A; (d) **Gate #5** — external *frontmatter* edit → embedded + standalone PropertyEditor reseed on remount; (e) **inv #9 burst** — >250-path pull incl. ≥1 open tab → adopt stays O(open∩changed), reindex still backgrounds, no flush-latency regression; (f) **inv #1** — Rust watcher_suppress confirmed untouched (own writes emit no `library-changed`). Safety-inspection + simplify. **Rollback:** flag=false or revert (restores the bare loop).

### §5 — fold `onNoteSaved` through the helper (WA#6 sibling gap)
- `+layout.svelte:3377-3391`: replace the hand-rolled adopt (which sets `tab.content` + `externalChangeNoteModel` but **forgets the reloadVersion bump**) with `adoptExternalChangeIntoTabs([path], { conflict, focusReseed, focusPath })`. Keep the `wasRecentlyWritten(path)` early-return.
- **Verify (running-app Gate #7 second-screen):** edit+save on the second screen → the MAIN window editor now **remounts + shows new content** (was silently stale); a dirty main-window model is NOT clobbered. Safety-inspection. **Rollback:** revert → restores the direct `externalChangeNoteModel` (pre-existing no-remount behavior).

### §6 — sidecar Rust command + conflict banner + wire the conflict branch + i18n ×15
- **Rust** (`libraries.rs` + register `lib.rs`): `write_conflict_sidecar(note_path, disk_content) -> Result<String,String>` → `<stem>.conflict-<UTCcompactZ>.md.txt` in the note's parent dir via `gate_create_exclusive(..., "conflict_sidecar")`; on `RefusedExists` append `-N` and retry once; return the sidecar path. (NOT via `write_note` — it rejects non-`.md`.) UTC timestamp passed from the frontend (Rust `SystemTime` is fine; script-side `Date.now` is not — this is app code, not a workflow script).
- **store.ts:** `saveConflicts` writable Map (sidecarPath → {noteName}) + `reportConflict`/`dismissConflict` (mirror `saveHealth` 296-313; **no auto-retry, no auto-clear** — a conflict is not a failure).
- **`SaveHealthBanner.svelte`:** a conflict row-set backed by `saveConflicts` — message `conflict.externalKept`, **"Show copy"** → `constellation_show_in_folder(sidecarPath)`, manual `×` dismiss. detectDir on the note name (RTL).
- **Wire** the helper's `conflict` hook to `write_conflict_sidecar` → `reportConflict`.
- **i18n:** `conflict.externalKept` / `conflict.showCopy` / `conflict.dismiss` in **all 15 locales** (ar, de, en, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh).
- **Verify (running-app Gate #1 + #6):** (a) dirty note + external edit → non-blocking banner + `A.conflict-<ts>.md.txt` holds the external content, sidecar NOT in tree/index/search, local text intact; (b) **debounce-race** — external edit lands then type within 300 ms → same (sidecar + banner, keystroke preserved); grep all 15 locales for the three keys. Safety-inspection. **Rollback:** revert → conflict branch no-ops (dirty refuse as today; no loss).

## Phase 4 — Audit (after §6)
Three parallel agents on the full `§1..§6` diff + this plan: **4A invariants** (walk #1–#11), **4B drift** (LL-023 — every new guard `reseedingPaths`/`diskBaseline`/`saveConflicts` + who bypasses it), **4C migration-path** (first boot, flag off↔on, mid-burst interrupt, rollback). Fix every real finding before close.

## Close (SO#9 + PCS)
Citation fixes (`SecondScreenPage.svelte:731-732` → `:3388`/`:3278`); document `WATCHER_ADOPT_ENABLED`; **file the rename-cascade synchronous-`clearCascading` latent hazard (open-risk, +layout.svelte:6333) as a NEW PJ-NNN** with the reproduction hypothesis; **PJ ledger → v1.19** (PJ-070 Done w/ evidence, re-rank ► Next action) in the same commit as the work; orientation **v3.40**; session log; help + User Manual (+14 translations) for the user-facing conflict-copy behavior; per-cycle whole-app safety-inspection register → Charter; MoCh; handover.

## Open risks carried
- Rename-cascade's own synchronous `clearCascading` (6333) may share hazard #6's shape — **filed as a new PJ at close** (not touched here; suspected masked by the pre-cascade `flushAllTabsInLibrary`).
- Two external edits in one 300 ms debounce on the same dirty note → two timestamped sidecars (no loss; dedupe `saveConflicts` by note path if noisy).
- `.md.txt` inertness depends on every surface staying `.md`-gated — documented at the sidecar write site.
