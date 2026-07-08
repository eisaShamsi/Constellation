# APP-KILLER #2 — NoteModel-Ownership Silent Nav-Loss — Architect

*Safety-Audit migration · Phase 1 (Architect) · 2026-07-08 · adversarial workflow `wf_0862e784-829` (8 agents, 894 k tokens)*

## Concept (the horse)

> When you leave a note you were typing in — by clicking a `[[wikilink]]`, clicking
> another note, or pressing Back — the words you just typed must reach disk **before**
> the editor is handed to the next note. A navigation is a *departure*, and a departure
> flushes. Nothing you typed is allowed to vanish because you moved.

## Function in hand

The **note-model ownership / navigation-flush path** — `openNoteTab`'s in-place tab reuse
(`store.ts:1790`), `loadTabHistoryEntry` (`store.ts:1016`), and the two-tab same-note case
(`+layout.svelte:3320`). Class: **notemodel-ownership / content-integrity** (Solve-the-Class:
single content ownership — CLAUDE.md).

---

## 1 · The defect (confirmed end-to-end in code)

Open notes each own **one** in-memory content model, keyed by `tab.id`, in a non-reactive
module `Map` (`noteModel.ts:84`). Every keystroke pushes the live CM6 rope into that model
(`NoteEditor.svelte:471` → `editBody`, O(1)), so **the model is always current** with the
freshest character typed.

`openNoteModel` (= `noteSession.open` = `M.openModel`, `noteModel.ts:120`) **unconditionally
rebuilds** the model for an id — `models.set(id, freshModelFromDisk)`, version reset to 0.
Two navigation sites call it on the *currently-active* tab with **no flush of the outgoing
note first**:

- **`store.ts:1790`** — `openNoteTab` in-place reuse (default file-tree / wikilink / panel
  click, `newTab` falsy). **DEFECT #1 (primary).**
- **`store.ts:1016`** — `loadTabHistoryEntry` (Alt+←/→ back-forward). **DEFECT #2.**

Result: type in A, then within the **1500 ms** autosave debounce click a link / another note
→ A's model is overwritten by B → up to a debounce-window (worst case ~30 s, the idle-save
interval) of text is **lost silently**, no error.

### Why the existing teardown "safety" flush cannot catch it (confirmed)

The `{#key}` remount fires the old editor's `onDestroy` → `doFlush()` → `handleFlush`. But the
reuse branch runs `openTabs.update` (tab.path → **B**) at `:1766` *before* `openNoteModel` at
`:1790`. So by the time the old NotePane tears down, `handleFlush` sees
`filePath(old A) !== tab.path(new B)` and **bails** (`NoteEditor.svelte:289`); and even if it
didn't, `compose(id, A)` would **refuse** because the model already holds B
(`noteModel.ts:207`). The flush is refused twice over. The fix **must** live at the nav site,
as a flush-*before*-replace — not in teardown.

### The proven precedent

`renameItem` already does exactly this at **`store.ts:2630`**: `if (renamedTab &&
isNoteDirty(id)) await saveNoteSession(id, oldPath, standardSaveEnv({origin:'rename_flush'}))`
**before** its own `openNoteModel` at `:2677`. The fix generalises this one proven guard.

---

## 2 · Full territory — every model-replace path (audit-complete)

`openNoteModel` has **exactly 5** call sites (`store.ts`), plus one `ensure()` re-seed path:

| Site | What | Outgoing dirty loss? |
|---|---|---|
| `:1790` `openNoteTab` reuse | active tab → new note | **YES — DEFECT #1** |
| `:1016` `loadTabHistoryEntry` | Alt+←/→ | **YES — DEFECT #2** |
| `:1808` `openNoteTab` new tab | fresh tab id | No (no outgoing model) |
| `:2677` `renameItem` re-seed | rename identity | No (**flushed first at `:2630`**) |
| `:605` `reloadTabsFromDisk` | **adopts** cascade-authored disk | *Adopt path — see §4* |
| `NoteEditor.svelte:153` `ensure()` | absent-model creator | No-ops on same path; only replace path for index-preview / dashboard / SS hosts |

**Focus mode** (`+layout.svelte`): `focusMode` auto-resets only when the active **tab id**
changes (`:1473`); in-place reuse keeps the same id, so Focus stays on with `focusSessionPath`
frozen on the old note (set only at `switchToFocus`, `:7940`) → after a nav, Focus keystrokes
on the new note are **refused** by the identity guard and lost on exit. Same silent-loss class,
reachable from Focus via Quick Switcher / command palette.

**Second screen** is a separate Tauri window = separate JS realm = its own `models` Map,
write-ahead buffer, and `saveHealth` (only `localStorage` is shared). It drives editing through
the **same** `openNoteTab` reuse path (`SecondScreenPage.svelte:818/1730/1742`), so it has the
**identical** DEFECT #1. Fixing `store.ts` covers both windows in one change.

---

## 3 · What the adversarial pass killed (naive fix → hardened fix)

The first-cut "just copy `:2630` to each site + fold it into `reloadTabsFromDisk` + add
dedup" design had **five real holes** (each verified in code, not hypothesised):

1. **Do NOT fold the flush into `reloadTabsFromDisk` (`:605`).** That path's job is to *adopt*
   freshly-authored disk (a link-rename cascade rewrote the file). Flushing the stale model
   there would write **stale-over-fresh** and silently re-break the cascade — resurrecting
   BUG-023 / the F2 cascade-stomp. `reloadTabsFromDisk` stays an adopt path; its callers already
   flush first where the user's edits should win.
2. **`nav_flush` must carry an `onSaved`** that reindexes (FTS5/`note_meta`) + broadcasts +
   re-embeds. Reindex/broadcast live *only* inside the `onSaved` hook (`NoteEditor.svelte:251`);
   `rename_flush` omits it (safe there — the rename pipeline reindexes afterward). A bare
   nav-flush would write the body to disk but leave **search stale** (index↔disk divergence — a
   named app-killer class) and never notify the second screen.
3. **Source the old path from the MODEL, not the tab.** At both sites the `openTabs.update`
   (tab.path → new) runs *before* the replace, so a helper reading `tab.path` gets the **new**
   path → `compose` refuses → the flush no-ops → the bug is NOT fixed. Insert the flush *before*
   the `openTabs.update`, sourcing `oldPath = getModel(id).path` (or the pre-update
   `currentTab.path`).
4. **`markSaved` must be swap-safe.** `noteSession.save` calls `M.markSaved(id, version)` *after*
   the awaited write, with no path guard (`noteModel.ts:218`). A save that resolves **after** the
   model was swapped writes a large `savedVersion` onto the *new* note → its first edits report
   `isDirty=false` → never autosaved → lost. Add a path-lineage guard (mirror the `compose`/
   `setBody` guards): `markSaved(id, version, expectPath)` no-ops on a path mismatch.
5. **Await-window keystroke + failed-flush + concurrent nav.** The flush awaits a real IPC write
   (~1–50 ms); a keystroke landing in that window re-dirties the model and would be discarded by
   the swap. Close it with a **bounded flush-while-dirty loop** whose final `isDirty→openNoteModel`
   step is synchronous, and a **shared supersede nav-token** (the same `_navTokens` map
   `loadTabHistoryEntry` uses) so a click-nav and an Alt-nav on one tab supersede each other.

### The failed-flush decision (recommended, baked into the design)

If the flush **write fails** (a `.md` momentarily locked by Syncthing/OneDrive/Defender):
**ABORT the navigation** — do not replace the model, keep the user on the note, and surface the
save-health banner (which auto-retries). Rationale (proven): if we replaced anyway, the tab no
longer holds the failing path, so `retrySaveFailure` early-returns and the banner points at a
note no tab owns — recoverable only by manually reopening. Abort keeps the live model where the
retry actually works. The edit is never lost either way (the write-ahead net holds it), but abort
is the clean outcome.

---

## 4 · The two genuine decisions for the Boss

Everything in §3 is engineering correctness (not a choice). Two things **are** choices:

### Decision A — Scope of this migration

| Option | What it fixes | Effort | Risk |
|---|---|---|---|
| **A1 — Core only** | The two nav-loss sites (`:1790`, `:1016`) + their correctness hardening (§3). | small | low |
| **A2 — Core + adjacent silent-loss (RECOMMENDED)** | A1 **plus** the Focus-mode nav variant, the `markSaved` swap-poison, and the rename-on-locked-file net-clear gap (`store.ts:2664` clears the recovery net regardless of flush outcome → a failed rename flush loses the edit). All the same "your edit silently vanishes" family the audit exists to kill (WA#6 "fix what you discover"). | small–medium | low |
| **A3 — A2 + duplicate-tab clobber** | A2 **plus** Decision B below (the two-tabs-same-note lost-update the handover lists in scope). | medium | medium |

**Recommendation: A3** — the handover explicitly scopes the two-tab clobber into APP-KILLER #2,
and A2's items are cheap fixes in the exact class. If the Boss prefers to keep the migration
tight, A2 is a clean stopping point and the duplicate-tab case becomes its own follow-up.

### Decision B — Two-tabs-same-note behavior (only if A3)

Today, opening a note already open in a **background** tab (or Ctrl+click) mints a **second**
model for the same path → whichever tab saves last clobbers the other.

| Option | Behavior | Cost |
|---|---|---|
| **B1 — Focus the existing tab (RECOMMENDED)** | Opening an already-open note jumps to its existing tab — one path → one tab → one model, structurally. | Removes "same note open in two panes side-by-side" (split view of one note). A deliberate constraint. |
| **B2 — Reconcile-on-save** | Allow duplicates; after each save, push disk into same-path siblings. | Cannot fix the dirty-vs-dirty case (`adoptDisk` refuses a dirty sibling) → **still loses data**. Rejected by Solve-the-Class (leaves two live owners). |

**Recommendation: B1**, with the open question: *does the Boss use / want "same note in two panes
at once"?* If yes, B1 carves out an explicit split-pane exception; if no, B1 is unconditional.

---

## BOSS RULING (2026-07-08)

- **Decision A → A3 (Everything).** Core nav-loss fix + all 3 cousins (Focus-mode loss,
  locked-file rename loss, late-save `markSaved` poison) + the two-tabs-same-note clobber.
- **Decision B → B1 (Jump to the existing tab), unconditional.** Opening an already-open note
  activates its existing tab; the same note is never open in two tabs at once. Same-note
  split-view is intentionally dropped. → Phase 2 Plan proceeds on this basis.

---

## 5 · Invariants that must not break (each tied to a Gate item / BUG)

1. On-screen === disk after **every** nav transition (reuse, Alt+←/→, wikilink, file click) — Gate 1/3/4; content-integrity class (BUG-012/015/019/023, §C series).
2. No spurious write at Focus enter; no cross-note write on switch-while-in-Focus — Gate 2/4 (the 2026-06-12 corruption site).
3. A dirty outgoing model is flushed **durably** before its slot is re-seeded, **or the nav aborts** — never discard-without-flush — Gate 1/3.
4. At most one live model per path per window — no sibling lost-update — Gate 6 (linked probe pair, BUG-023 shape).
5. Rename cycle with a linked probe pair keeps both identities intact; the `:2630` precedent's behavior is preserved (or refactored with **zero** behavior change) — Gate 6.
6. Second-screen edit + sync round-trips; `adoptDisk` freshness gate (dirty→refuse) preserved — Gate 7.
7. Restart / workspace restore re-reads disk as truth — Gate 8.
8. A failed flush never marks the model clean and never silently loses edits — write-ahead net retained + surfaced (Save-Durability APP-KILLER #1 contract).

## 6 · Rollback / migration path

**Frontend behavior only** — no schema, no persisted-format change. First-boot and rollback are
trivial (models are ephemeral, rebuilt from disk). Land behind a module-level `NAV_FLUSH_ENABLED`
(and `DEDUP_ALL_TABS_ENABLED` for B1) so the whole change is one-const-toggle revertible if a Boss
test regresses — matching Solve-the-Class "land as ONE validated swap behind a toggle." The
write-ahead net (`localStorage` `constellation-wab`) is already forward/backward compatible and is
the recovery net during rollout. Second screen shares `store.ts`, so one flag covers both realms.

## 7 · Explicitly OUT of scope (documented residuals, not introduced by this fix)

- **Cross-realm dirty-vs-dirty conflict** (main + second screen both dirty on one path): the "§E
  conflict dialog" referenced in comments **was never built**. This fix makes the *clean*
  cross-window case reconcile (nav_flush broadcasts via `onSaved`); a true two-window dirty
  conflict remains freshness-gated (`adoptDisk` refuses, no auto-merge) and is a separate feature.
- **`closeAll` (universe switch / restart)** flushing dirty models before `clearAllModels` — no
  verified live caller today; its own trace/Gate-8 case, separate from the 5 nav sites.
- **`ensure()` replace branch** for index-preview / dashboard hosts — verify those hosts never
  hold a dirty model (they are read-only previews); guard only if proven otherwise.

## 8 · Reproduce-First

Extend `tests/mig-076/runtimeHarness.test.ts` (red→green, **before** any wiring):
- **Recipe I** — nav-away-while-dirty flushes the outgoing model (screen === disk on both notes; A's edit persisted).
- **Recipe J** — failed nav-flush ABORTS (model stays dirty, net retained, nav not applied).
- **Recipe K** — late-resolving save does not poison a swapped-in model (`markSaved` path-guard).
- **Recipe L** (if A3/B1) — opening an already-open path yields ONE model (no duplicate/clobber).
