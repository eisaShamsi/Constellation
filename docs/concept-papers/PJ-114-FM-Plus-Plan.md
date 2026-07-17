# PJ-114 "FM+" — Migration Plan (Phase 2)

*Constellation `/migration` · Focus Mode Plus · awaiting Boss approval · 2026-07-17*

> Companion to the approved design brief `docs/concept-papers/PJ-114-FM-Plus-Design.md` and the Phase-1 Architect deliverable. Approved architecture: **D1(a)** FocusPane self-contained + semantic callbacks; **D2(a)** Follow routes through `openNoteTab`; **D3** split (two pre-migration commits, then the migration); **D4** build flag `FM_PLUS_ENABLED` + user setting `focusModePlus`. Approved UX: Essentials+ menu · Mod-click + Mod-Enter follow · back/forward via existing keys · footer `FM+` toggle (default off).
>
> **Nothing is built until the Boss approves this plan.** After approval, the build cascades step-by-step (Plan-Approval = Build-Approval), stopping only at the user-testable verification clauses.

---

## Phase 0 — Pre-migration standalone commits (unflagged, no behavior risk)

### §0.1 — PJ-116: wire the Focus title-blur signal to the rename engine
- **Touches:** `src/routes/+layout.svelte` `<FocusPane>` render (~8033-8070) — add the omitted `ontitlechange` prop wired to `handleRenameComplete(focusSessionPath, newTitle)` (guarded on `SINGLE_OWNERSHIP` + non-empty). `FocusPane.svelte` unchanged (`handleTitleBlur` ~110-117 already fires the signal). Mirrors NotePane's `NoteEditor.svelte:399-405` delegation.
- **Verify:** typing a title in Focus + blur renames the file on disk AND heals every `[[link]]` universe-wide (was silently discarded); freeze overlay shows during the cascade; on-screen === disk; equal/empty title = no-op.
- **Test (running app):** A, B with `[[A]]` in B → Focus A → type new title → blur → A's file renamed + B shows `[[NewTitle]]`.

### §0.2 — Extract the shared link-finder helper (prove against the main editor first)
- **Touches:** new `src/lib/editor/linkAtPos.ts` (~15 lines, `findWikilinkAtLineOffset`); refactor `CodeMirrorEditor.svelte:270-286` to call it (behavior identical). Only genuinely-new code the design allows; proven on the main editor before Focus imports it.
- **Verify:** Ctrl/⌘-click a `[[link]]` in the main editor still follows exactly as before; unit test covers inside plain / aliased `[[A|B]]` / heading `[[A#h]]` / boundaries / plain-text (null). Parser-free.
- **Test:** main editor Ctrl/⌘-click a wikilink → opens; no visible change vs today.

---

## Phase 1 — The migration (every step behind `FM_PLUS_ENABLED`, user setting default off)

### §1.1 — Flag + persisted setting + effective gate + second-screen propagation
- **Touches:** new `src/lib/editor/fmPlusFlag.ts` (`export const FM_PLUS_ENABLED = true`); `store.ts` add `focusModePlus: boolean` to `AppSettings` (~4682) + `false` to `DEFAULT_SETTINGS` (~5197); `+layout.svelte` derived `fmPlusEffective = FM_PLUS_ENABLED && $appSettings.focusModePlus` (~1509). Second-screen rides the existing `updateSettings → emit('screen:settings-changed')` for free.
- **Verify:** `svelte-check` passes; default-off = Focus identical to today; toggling persists across restart + reaches second screen; `FM_PLUS_ENABLED=false` forces effective false.

### §1.2 — The footer `FM+` toggle (RTL, ×15 tooltip, fade-on-type)
- **Touches:** `FocusPane.svelte` `.focus-footer` (~321-326/444-461) + new props `fmPlusEnabled/fmPlusActive/onToggleFmPlus`; `+layout.svelte` passes them (`onToggleFmPlus` flips `focusModePlus`). Token shown only when `fmPlusEnabled` (flag-off = no token = byte-for-byte). Dim when off, lit-with-dot when on; fades while typing, returns on pause; `+` isolated for RTL.
- **Verify:** token appears under the flag; click flips + persists + carries to second screen; dim/lit states; fade on type; Arabic/Hebrew mirroring with `+` correct; no follow/menu/rename yet.
- **Test:** enter Focus, click `FM+` on/off; switch to Arabic UI; type to confirm fade.
- **Locale ×15:** `focus.fmPlusTooltip`.

### §1.3 — Right-click menu shell + no-risk items (Copy link text · Copy path · Cut/Copy/Paste)
- **Touches:** `FocusPane.svelte` add a gated `contextmenu` `domEventHandler` using `linkAtPos` + prop `onLinkContextMenu({x,y,linkText|null})`; `+layout.svelte` owns menu construction — resolve via `resolveWikilinkCrossLibrary` (read-only), render existing `<ContextMenu>`. Link menu: Copy link text + Copy path. Plain menu: Cut/Copy/Paste. Follow + Rename deliberately deferred to §1.4/§1.5.
- **Verify:** link right-click shows Copy link text + Copy path; plain right-click shows clipboard items; labels ×15; menu clamps + RTL-anchors; Copy path resolves cross-library; effective-off = inert; no note write (except ordinary Cut/Paste).
- **Test:** right-click link → Copy path → paste elsewhere → correct absolute path; Arabic UI anchoring.
- **Locale ×15:** `contextMenu.copyLinkText` (reuse `copyPath`, `cut/copy/paste`).

### §1.4 — In-Focus Follow (migration core) + auto-exit re-capture + reproduce-first harness
- **Touches:** `FocusPane.svelte` prop `onLinkFollow(linkText)` (menu "Open in Focus" + later the gesture call it); `+layout.svelte` new `followFocusLink` → resolve → `openNoteTab(resolved.path, …, newTab=false, fromNotePath=focusSessionPath)`; **modify the auto-exit `$effect` (1495-1498)** so a FM+ path-only change on the same tab RE-CAPTURES `focusSessionId/Path` + advances `_focusModeTabId` + sets `focusReseedSuppress` for one `tick()` + keeps `focusMode` true (a genuine tab-id change still exits).
- **Safety ordering:** `openNoteTab` flushes the outgoing model first (aborts nav on failure) + pushes history; the `{#key}` remount reseeds from the target's fresh model; `focusReseedSuppress` (PJ-070 hazard #7, reused verbatim) makes the outgoing FocusPane teardown flush a no-op so A's buffer can't bleed into B.
- **Verify:** Follow swaps the paper to the target without leaving Focus; outgoing note's last keystroke is on the OUTGOING disk and nothing bleeds into the arrived note; a failed flush aborts the nav (banner) rather than losing data; `_focusModeTabId` tracks the new note.
- **Test (REPRODUCE-FIRST, the marquee gate):** A(`[[B]]`), B → Focus A → type `ZZZ` at A's end → **immediately** (before the 1500ms debounce) Follow `[[B]]` → assert (1) A's disk ends `ZZZ`, (2) B's disk has no `ZZZ`, (3) paper shows B, still in Focus. Repeat with reduce-motion on/off.
- **Locale ×15:** `contextMenu.openInFocus`.

### §1.5 — Right-click Rename (link target) + "Rename this note…" (self-rename) — lights up sweep-#3 freeze
- **Touches:** `+layout.svelte` extend the menus — link "Rename…" → resolve → `renameDialog={path:resolved,name:targetTitle}` → `<RenameDialog> onConfirm → handleRenameComplete`; plain "Rename this note…" → `renameDialog={path:focusSessionPath,…}` → same. Freeze wiring already parked (`focusFrozen` 1509, `editGate` 275-277, overlay 330) becomes REACHABLE here.
- **Safety:** reuses the one orchestrator (`handleRenameComplete` 6215) — universe cascade, collision dialog, dup guard. When the renamed note is the focus note, its path enters `$cascadeFreeze` → FocusPane hard read-only + "Updating…"; `commitFocusSave` `isCascading/isReseeding` gate prevents a pre-cascade stomp.
- **Verify:** renaming a link target from Focus renames on disk + rewrites all `[[links]]`; "Rename this note…" is the only in-Focus route (tree hidden) and Focus stays on it; freeze overlay visible + keystrokes refused during a focus-note rename; after cascade on-screen === disk, no revert.
- **Test (linked probe pair):** A, B(`[[A]]`), C(`[[A]]`) → Focus → right-click `[[A]]` → Rename… → B and C both updated on disk. Separately: Focus A → Rename this note… → mash keys during "Updating…" (none land) → A renamed, Focus still on A.
- **Locale ×15:** `contextMenu.renameThisNote` (reuse `actions.rename`).

### §1.6 — The Mod-click + Mod-Enter gesture, disambiguated from sentence-select
- **Touches:** `FocusPane.svelte` two gated CM6 extensions — `Prec.highest` `mousedown` handler: on `ctrlKey||metaKey`, `linkAtPos` under `posAtCoords` → **on a link: preventDefault + emit onLinkFollow + consume** (so `ctrlClickSentence` never fires); **off a link: return false** (sentence-select unchanged). Plus `Prec.high` `Mod-Enter` keymap: caret-in-link → follow, else inert. **`sentenceSelect.ts` NOT edited** — disambiguation by consuming the event upstream (preserves the RTL invariant, zero risk).
- **Verify:** Mod-click on a link follows (stays in Focus); Mod-click off a link still selects the sentence (RTL parity intact); Mod-Enter in a link follows, elsewhere inert; ⌘ on Mac / Ctrl on Win; effective-off = always sentence-select.
- **Test (Mac + Win parity):** Ctrl/⌘-click a link → follows; Ctrl/⌘-click mid-sentence → sentence highlights; caret in link + Ctrl/⌘-Enter → follows; Arabic sentence breaks on ؟ ۔ ! . but keeps ؛.

### §1.7 — Back / forward: wire-through verification (no new chrome)
- **Touches:** confirm the §1.4 re-capture covers `loadTabHistoryEntry` (store.ts:1348, same path-change-on-same-tab shape); confirm `Alt`+←/→ (Win) / `⌘[`/`⌘]` (Mac) reach `navigateBack/Forward` in Focus (add a small `Prec` passthrough only if a Focus keymap swallows them — verify first).
- **Verify:** Follow A→B→C, Back → B, Forward → C; each hop flushes the departing note + keeps Focus; on-screen === disk each hop; history ends are inert.
- **Test (reproduce-first):** A(`[[B]]`), B(`[[C]]`), C → Focus A → follow B → follow C → type `QQQ` in C → Back to B (C's disk has `QQQ`, B unchanged) → Forward to C (`QQQ` present). Test Win (Alt+arrows) + Mac (⌘[ ⌘]).

---

## Risk-mitigation (per Phase-1 invariant)
- **Single-ownership:** Follow routes through `openNoteTab`/`loadTabHistoryEntry` dedup — never mints a 2nd model.
- **Flush-before-replace:** both nav routes flush-first and abort on failure; FM+ adds no bypassing write.
- **Parser-free:** all link detection via `linkAtPos` (one-line regex); no rendering extension added.
- **No-spurious-write-on-enter:** toggle/menu/Copy/gesture-yield never call `onchange`/`editNoteBody`/`commitFocusSave`.
- **On-screen === disk:** `{#key}` remount reseeds from fresh model; `focusReseedSuppress` blocks stale teardown flush; `commitFocusSave` cascade-gate blocks stomps.
- **Sweep-#3:** parked freeze becomes reachable at §1.5; correctness is §1.5's reproduce-first target.
- **RTL/i18n:** `<ContextMenu>` RTL + ×15; footer mirrors with isolated `+`; sentence-select RTL parity is §1.6's check.
- **macOS/Win parity:** `metaKey||ctrlKey` follow; Alt-arrows / ⌘[ ⌘] history; no new accelerator.
- **FM+-off byte-for-byte:** flag-off = no token/handlers/props; flag-on + setting-off = only a dim inert token.

## Rollback
- **Primary:** `FM_PLUS_ENABLED = false` → whole feature inert, Focus reverts, no other edits.
- **User:** `focusModePlus=false` (default) keeps verbs inert per-user.
- **Persisted-key safety:** a stored `focusModePlus` is inert under flag-off; no migration/cleanup to roll back.

## Editor-Surface Gate mapping
| Gate item | Steps |
|---|---|
| Focus enter/type/exit — no spurious write | §1.2, §1.3, §1.6 |
| Tab-switch while in Focus (id-change still exits) | §1.4, §1.7 |
| Rename cycle with a linked probe pair | §0.1, §1.5 |
| On-screen === disk after every transition | §0.1, §1.4, §1.5, §1.7 |
| Flush-before-replace | §1.4, §1.7 |
| Sweep-#3 freeze reachable + correct | §1.5 |

**Locale files (all 15 — ar de en es fa fr he hi ja ko pt ru tr ur zh):** §1.2 `focus.fmPlusTooltip` · §1.3 `contextMenu.copyLinkText` · §1.4 `contextMenu.openInFocus` · §1.5 `contextMenu.renameThisNote`. Reused: `contextMenu.copyPath`, `contextMenu.cut/copy/paste`, `actions.rename`.

**Dependencies:** §0.1, §0.2 independent (ship first). §1.1 gates all. §1.3 needs §0.2 + §1.1. §1.4 needs §1.3. §1.5 needs §1.3. §1.6 needs §1.4 + §0.2. §1.7 needs §1.4.

## Per-build safety (standing order)
Each lifecycle-touching commit (§0.1, §1.4, §1.5, §1.7) runs the diff-scoped `safety-inspection` before commit, and every confirmed finding is fixed before the commit. Boss tests each user-testable step before it is committed.
