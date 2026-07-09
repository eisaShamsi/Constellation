# G3 — Second-Screen Cross-Window Sync — Architect

*Safety-Audit migration (the 2 remaining HIGH from sweep `wf_415a7214-4ad`) · Phase 1 · 2026-07-09 · adversarial workflow `wf_a6e3b69b-da7` (4 mappers verified; synthesis authored here from their findings).*

## Concept (the horse)

> A note open in **both** windows must show and save the **same truth**. The second screen can never silently undo — or be undone by — the main window. One note, one disk state, both windows agreeing.

## Function in hand

The **second-screen cross-window sync path** — the second screen (a separate Tauri webview / JS realm) fails to honor a main-window rename cascade, and never adopts main-window saves into its editable tabs. Class: cross-window-integrity (G3).

---

## 1 · Territory (verified in code)

- **The SS is a separate JS realm.** `screen-entry.ts` mounts `SecondScreenPage` as a standalone Svelte app, bypassing SvelteKit routing (so `+layout.svelte` — where all the cross-window adopt + cascade wiring lives — is **never loaded** in the SS). Every module singleton is a **separate per-realm instance**: noteModel's `models` Map, `writeAheadBuffer`, `saveHealth`, `cascadingPaths`, `cascadeFreeze`, `openTabs`, `recentWrites`. Only `localStorage` + Tauri `emit`/`listen` bridge the two windows.
- **The SS mounts SEVEN fully-editable `NoteEditor` instances** (active `:1771`, peek `:1588`, 5 companions `:1072/:1112/:1159/:1215/:1232`). Each autosaves to disk through the durability gate. `NoteEditor` has **no** `readOnly` prop today.

### The two confirmed HIGH bugs
- **Bug 1 — SS is blind to a main-window rename cascade.** `markCascading`/`isCascading`/`cascadeFreeze` are per-realm, so the SS's `cascadingPaths` stays empty → `isCascading()` is **always false** in the SS → the SS's armed autosave is **not gated** during a main-window cascade and composes from its stale model, **reverting the cascade's `[[wikilink]]` rewrite on disk** (index↔disk divergence, no error). The `cascade:rewrote` event *is* emitted globally (`app.emit`, `libraries.rs:5483/1618`) so it physically reaches the SS transport — but **no SS code listens** (the listener is only in `+layout.svelte:3223`). No `screen:*` cascade/freeze event exists.
- **Bug 2 — SS never adopts main→SS saves into its editable tabs.** SS `onNoteSaved` (`SecondScreenPage.svelte:723`) refreshes **only** the read-only companion panels (`editorPanelsData`/`splitCompanionData`); it never calls `externalChange` on a writable `NoteEditor` tab model. The **main** window *does* the symmetric adopt (`+layout.svelte:3320` `externalChangeNoteModel(tab.id, content)`, freshness-gated, with a comment saying it exists to adopt the SS's writes). **The asymmetry is the bug:** a note edited+saved in the main window leaves the SS editor model stale → the SS's next autosave silently overwrites the main window's committed edit.

### What already exists to build on
- `externalChange` → `adoptDisk` (`noteModel.ts`): freshness-gated — **refuses a dirty model**, ignores its own echo (`compose === disk`), else adopts + marks clean. Exported, unit-testable, no Svelte mount.
- `reloadTabsFromDisk` (store): the adopt-disk primitive the main cascade uses (its callers flush dirty tabs first).
- `wasRecentlyWritten` echo guard: per-realm, correctly suppresses a realm adopting its own broadcast.
- **The "§E conflict dialog" does NOT exist** (only a `noteModel.ts:256` comment). A genuine two-sided dirty conflict today is **silent last-writer-wins** — no UI, no error.

### The doctrine tension
CLAUDE.md **"Additional screens are displays, not domains — never re-implement save/load/edit,"** and `+layout.svelte:3338` even *claims* "the 2nd screen never writes." But the SS mounts 7 editable, disk-writing editors — **that claim is already false.** This is the crux of the fork below.

---

## 2 · The design fork (the one decision that shapes everything)

Both bugs share **one root**: the SS mounts editable editors but has no inbound path that pushes fresh disk content (from a save *or* a cascade) into those models. Two ways to fix the root:

### Option A — Keep the SS an editing domain; make it cascade-aware + adopt-correct
- SS `onNoteSaved` adopts main→SS: for **every** SS editor tab whose `path === saved path`, call `externalChange(tab.id, freshDisk)` (freshness-gated; **all-tabs**, not first-match — the SS has up to 7 editors).
- SS reacts to the cascade: add a `cascade:rewrote` listener in the SS → adopt/reload the rewritten paths; **plus** a new cross-window **freeze signal** (start-freeze → SS raises its own `markCascading` and stops its autosave; end-freeze → SS clears it) so the SS can't stomp mid-cascade.

| | |
|---|---|
| Speed / Effort / Risk | medium / **large** / **medium** |
| Failure modes | (1) the freeze must arrive **before** the SS's armed autosave fires — Tauri `emit` is async/best-effort, a real race; (2) freeze-stuck-on if the clear event is lost → SS silently stops saving; (3) main↔SS adopt echo loop if the `wasRecentlyWritten` guard is mis-timed; (4) **the dirty-vs-dirty conflict stays silent** (adopt refuses on both sides, no UI) — A does not solve it. |

### Option B — Make the SS editors READ-ONLY (true Display-not-Domain)
- Add a `readOnly` prop to `NoteEditor`; the SS mounts all editors read-only; editing routes to the main window (the SS already has "open in main" plumbing). The SS **never writes** → the entire cross-window write-conflict class **disappears** (no cascade-stomp, no clobber, no dirty-vs-dirty).

| | |
|---|---|
| Speed / Effort / Risk | medium / **medium** / **low** |
| Failure modes | (1) **removes an existing capability** — you can edit on the second screen today; B ends that (edits happen in the main window); (2) any SS edit affordance must clearly route to main, not silently no-op; (3) the read-only editor must still render live (companions, links) — a real but contained prop. |

**The deciding question is yours, and code can't answer it: do you actually *edit* on the second screen, or is it a viewing/reference surface?**
- If you **edit on it** → **Option A** (sync it properly; accept the cross-window race + the still-silent dirty-conflict as a documented residual, or add a small conflict indicator).
- If it's a **display/reference** surface → **Option B** — simpler, lower-risk, kills the whole class, and it's what the Display-not-Domain doctrine already prescribes. **(My lean, absent a reason you need SS editing.)**

---

## 3 · Invariants · Rollback · Reproduce-First

**Invariants (Editor-Surface Gate item 7 + content-integrity):** after a main-window save/rename, an SS view of the same note shows on-screen === disk; an SS action never silently overwrites a main-window edit; a dirty model (either realm) is never clobbered without a surfaced choice; `adoptDisk`'s freshness gate preserved.

**Rollback:** frontend-only (SS component + event wiring / a `readOnly` prop), no schema. Land behind a flag (`SS_SYNC_ENABLED` for A, or the `readOnly` wiring for B) — one-line revert.

**Reproduce-First:** extend `tests/mig-076/runtimeHarness.test.ts` with a **two-sessions-one-path** recipe (two model ids `main`/`ss`, same path): main edits+saves → `ss` adopts (`externalChange`) when clean; main edits+saves → `ss` **refuses** when `ss` is dirty (no clobber); cascade-rewrite → `ss` reloads not-stomps. **Residual only the Boss two-window test closes:** the harness is single-realm — it can't exercise the two real webview realms, the Tauri transport, or CM6 seeding.

## 4 · What's NOT in scope
The genuine **two-sided dirty conflict UI** (the never-built §E dialog) — if kept editable (A), a real conflict remains silent last-writer-wins; building the conflict dialog is its own feature. Option B sidesteps it entirely (in the default read-only mode).

---

## BOSS RULING (2026-07-09) — B-default + editable toggle (a hybrid)

> *"I will use it for viewing/reference; it shall be **read-only by default**, but I want to be able to **toggle it On/Off in the Settings** if I want to make it editable."*

**Decision: Option B is the DEFAULT, with a Settings toggle to opt into editing.** This is the safest posture AND keeps the capability. Because the toggle can make the SS a real editing domain, the editable path must ALSO be made safe (WA#6 — never ship a known-unsafe toggle). So the migration delivers B **and** the sync from A, gated by the toggle:

- **Read-only by default** (`readOnly` prop on `NoteEditor`; SS mounts read-only; a Settings toggle "Make the second screen editable", default OFF). This alone fixes the 2 HIGH bugs for the default config — the SS never writes → no clobber, no cascade-stomp.
- **The SS still stays FRESH in both modes** — it adopts main→SS saves (§2) and reacts to rename cascades (§3) into its editor views (freshness-gated `externalChange`/adopt; a read-only model is always clean, so it always adopts the latest disk).
- **Editable mode is made safe** — when the toggle is ON, the SS also honors a cross-window cascade **freeze** (§4) so its autosave can't stomp a main-window cascade while the SS holds a dirty edit (mirroring the main window's `markCascading` discipline). The two-sided dirty conflict remains the documented residual (as it is for the main window today).

→ Phase 2 Plan proceeds on this basis. See `docs/G3-SecondScreen-CrossWindow-Plan.md`.
