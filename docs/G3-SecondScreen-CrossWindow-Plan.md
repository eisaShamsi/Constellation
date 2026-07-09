# G3 — Second-Screen Cross-Window Sync — Build Plan (Phase 2)

*Follows `docs/G3-SecondScreen-CrossWindow-Architect.md`. Boss ruling: **read-only by default + a Settings toggle to make it editable**; editable mode must be safe (WA#6). Frontend-only, no schema.*

## Shape

The default (read-only) fixes the 2 HIGH bugs by construction — the second screen never writes. The sync (adopt + cascade-react) keeps the read-only view **fresh**, and makes the opt-in **editable** mode safe. Everything lands behind small flags/props (one-line revert). The logic handshake is proven in the harness; the **cross-window wiring is proven by the Boss two-window test** (the one residual a single-realm harness can't reach).

---

## §1 — Read-only `NoteEditor` + the Settings toggle *(default read-only)*
- `NoteEditor.svelte` (+ `NotePane.svelte`): add a `readOnly?: boolean` prop. When true — CM6 `EditorState.readOnly.of(true)` + `EditorView.editable.of(false)` (no keystrokes → no autosave → no disk write), and `handleSave`/`handleFlush`/`handleTitleChange`/`handlePromote` early-return as a belt. A read-only editor still renders live (livePreview, links, decorations).
- `store.ts appSettings`: add `secondScreenEditable: boolean` (default **false**); propagate to the SS realm via the existing `screen:settings-changed` bus.
- `SecondScreenPage.svelte`: pass `readOnly={!$appSettings.secondScreenEditable}` to all **7** `NoteEditor` mounts.
- Settings UI: a toggle **"Make the second screen editable"** (default off) in the second-screen / appearance section; i18n ×15.

**Verify (Boss, two windows):** by default the second-screen note view does not accept typing; flip the Settings toggle → it becomes editable. `svelte-check` 0 errors.

## §2 — SS adopts main→SS saves *(freshness both modes; conflict-safety when editable)*
- `SecondScreenPage.svelte onNoteSaved` (`:723`): after the `wasRecentlyWritten` echo guard, for **every** SS tab (the `openTabs` editor tabs **and** the companion note tabs) whose `path === saved path`, re-read disk and `externalChange(tab.id, content)` + bump its `reloadVersion` so the view reseeds from the fresh model. Freshness-gated (`adoptDisk` refuses a dirty model). **All matching tabs**, not first-match (the SS can hold ≤7 editors).

**Verify (Boss, two windows):** edit + save note X in the main window → the second screen's view of X updates to the new content within ~1 s (no manual reopen).

## §3 — SS reacts to the rename cascade *(freshness)*
- `SecondScreenPage.svelte`: add a `listen('cascade:rewrote')` (the event is already broadcast globally via `app.emit`) → for each rewritten path, re-read disk and adopt + `reloadVersion` bump on matching SS tabs (same freshness-gated adopt as §2). Reuses the main window's `+layout.svelte:3223` shape.

**Verify (Boss, two windows):** with a note that links to Foo open on the second screen, rename Foo → Foo v2 in the main window → the second screen shows the updated `[[Foo v2]]`.

## §4 — Editable-mode cross-window freeze *(safety when the toggle is ON)*
- Add a cross-window cascade **freeze signal**: the main window, in its rename cascade (`+layout.svelte handleRenameComplete`), emits `screen:cascade-freeze {paths, active:true}` at start and `{active:false}` at end (around the existing `markCascading`/`clearCascading`). The SS listens → raises/clears its **own** realm `markCascading` for those paths, so its autosave (`isCascading` gate in `NoteEditor.handleSave`) is suppressed during the cascade window, then §3 adopts the rewrite. Guard against a stuck-freeze (a max-duration auto-clear timer, mirroring the main cascade-freeze safety).
- No-op when read-only (the SS never writes) — but wired unconditionally so the toggle is safe the instant it's flipped.

**Verify (Boss, two windows, SS editable):** with the toggle ON, type into note X on the second screen while the main window renames a note X links to → the cascade's rewrite survives on disk (the SS does not stomp it); X's own edit is preserved (freshness-gated).

## §5 — Reproduce-First harness + simplify + safety sweep + Boss test
- `tests/mig-076/runtimeHarness.test.ts`: add a **two-sessions-one-path** recipe (two model ids `main`/`ss`, same path) — main saves → `ss` adopts when clean; main saves → `ss` **refuses** when `ss` dirty (no clobber); cascade-rewrite → `ss` reloads-not-stomps. (Regression guard for the handshake; the primitives already pass — the real proof is the Boss test.)
- `/simplify` on the diff. Diff-scoped `safety-inspection` over the changed files; fix every confirmed finding before commit (WA#6).
- **Editor-Surface Gate item 7** (second-screen edit + sync) Boss two-window test — the definitive proof.

**Rollback:** frontend-only; `secondScreenEditable` default false + a `SS_SYNC_ENABLED` flag → one-line revert; no schema, models are ephemeral.

## Residual (documented, not in scope)
A genuine **two-sided dirty conflict** (both windows editing the same note at once, with the toggle ON) resolves silently last-writer-wins — the same as the main window today (the §E conflict dialog was never built). The read-only default avoids it entirely; building the conflict UI is separate future work.
