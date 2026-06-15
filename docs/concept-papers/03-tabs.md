# 03 — Tabs & Tab Bar (Concept Paper)

> A satellite of the **gate** ([01 — Note Editor](01-Note-Editor.md)). The tab bar chooses *which* note the Editor mounts; it owns selection, not content. It must never re-implement save/load/edit (additional surfaces are displays, not domains).

## 1. Function in hand
The **Tab Bar** — the horizontal tab strip rendered in `src/routes/+layout.svelte` (the `.tab-scroll` row, `{#each $openTabs as tab}`), backed by the `openTabs` / `activeTabId` writable stores in `src/lib/libraries/store.ts`. Each tab is one open note; the active tab is what `NoteEditor` mounts.

## 2. Purpose
Hold several notes open at once and let the user pick which one is in front, instantly — so a train of thought can span multiple notes without losing place. It serves **Connection** (Act 2): keeping the supporting note, the contradicting note, and the synthesis note all reachable in one click is the manual precondition for noticing how they relate. It does not *create* knowledge; it sustains the working set the other Acts operate over. Justified: without it, every note-switch would be a full close-and-reopen, breaking the Connection workflow.

## 3. What it is NOT
- **Not** the Editor — it owns *selection and lifecycle* (open/close/pin/reorder), never a note's content or its save path.
- **Not** a derived view of the Universe — the tab set is transient session state, not something computed from notes on disk.
- **Not** a persisted/auto-restored surface today — the app boots to zero tabs; tab restoration is **manual only**, via named workspaces (`restoreWorkspace`).
- **Not** the split-pane engine — split layout is adjacent (`splitActive`/`focusedTabId`) and out of scope here.

## 4. Wiring
- **Inputs:** `openNoteTab(...)` (tree click, wikilink, search result, trail step — ~15 call sites); `createEmptyTab()` (the `+`/bulb new-tab button); `switchTab(id)` (click); `closeTab(id)` (× button, middle-click `onauxclick`, context-menu close-*); `reorderTab(fromId, toId)` (drag); the `openTabs`/`activeTabId`/`activeTab` stores.
- **Outputs:** `openTabs.set(...)` / `activeTabId.set(...)` store mutations; `closeNoteModel(tabId)` on close (MIG-076 §C — disposes that tab's content model); a `reloadVersion` bump on a tab drives the Editor's `{#key}` remount.
- **Consumers:** `NoteEditor` (mounts `$activeTab`, or each split tab); the second screen (mirrors `$activeTab`); the file tree (`class:active={$activeTabId === tab.id}`); maturity dot map; library-color stripe.
- **Connection to the Editor (the gate):** the tab strip emits *only a selection*; the Editor attaches via `{#key tab.id + '|' + tab.path + '|' + (tab.reloadVersion ?? 0)}` in `NoteEditor.svelte` (line ~408) — switching the active tab destroys + remounts the pane with that note's model. The tab never reads or writes note content; it hands the Editor an identity and steps back.

## 5. Right-click / context menu
- **Has one — shared, MIG-077-compliant.** `oncontextmenu={(e) => showTabContextMenu(e, tab.id)}` opens the shared `<ContextMenu>` (not hand-rolled) with items from `getTabContextMenuItems(tabId)`. MIG-077 A1 explicitly converted this from an earlier inline hardcoded-English menu to the shared component — **no debt here**.
- **Items:** Pin/Unpin · — · Close (disabled when pinned) · Close Others · Close to the Right · Close to the Left · Close All · — · Copy Path · Copy Name.
- **Reachable only by right-click:** Close Others / Close to the Right / Close to the Left / Close All / Copy Path / Copy Name. (Pin and single-Close also have non-menu paths: pinned tabs show 📌; close has the × and middle-click.) These are conventional tab actions and correctly menu-gated.

## 6. Multilingual
- **Context-menu labels:** all nine flow through `$t('tabContextMenu.*')`; the block is present and natively translated in the locales (verified `en.json` line 1495 and `ar.json` line 1428 — Arabic uses native terms, e.g. `إغلاق ما على اليمين`).
- **Chrome strings:** the empty-state hint (`layout.clickToStart`), pinned tooltip (`layout.pinned`), and new-tab placeholder (`tabs.newTab`) use `$t()`.
- **Gap — hardcoded English:** the new-tab button's tooltip is `title="New tab"` (`+layout.svelte` line ~5907), NOT `$t()`. A `tabs.newTab` key already exists — this is a one-line miss to fix at bring-up.
- **Gap — RTL on tab titles:** `<span class="tab-title">{tab.name}` has **no** `dir="auto"`/`detectDir()`. An Arabic/Hebrew note name on the tab won't get per-element bidi direction (same class as the NotePane-badge `dir="auto"` polish item). Flag for bring-up.

## 7. Boot behavior
- **Runs at boot?** No IPC of its own. `openTabs` initializes to `[]`; the strip paints empty (dashboard / new-tab state). Tabs appear only on explicit user action or a manual workspace restore.
- **Rule 8 status:** **N/A (compliant by construction).** The tab set is *not* a derived view of the Universe — there is nothing to recompute. It does **not** re-walk notes at boot, and `restoreWorkspace` re-opens saved paths only on user command, not automatically. No write-time-derivation obligation applies; it stores its own ephemeral state directly.
- **Cost:** zero measured boot cost (no boot IPC; empty store). Per-switch cost = one `{#key}` remount of the Editor (the Editor's mount cost, ~1–3 ms disk read estimated), not a tab-bar cost.

## 8. Flag / gate & bring-up position
- **Gate today:** **none** — there is no `enabledFeatures.tabs` or SIGHT flag; the tab bar is unconditional core, like the Editor. (Confirmed: grep for a tabs feature-flag returns nothing.)
- **Bring-up phase:** **1 (Core spine)**, alongside the Editor. Depends on: the app shell + `NoteEditor` (it has no value without a pane to mount) and `openNoteTab`/the `openTabs` store. No satellite (search, graph, Sight) depends on it; it depends only on the gate.

## 9. Budget
- **Boot budget:** zero — must stay free of boot IPC and start empty (no auto-restore that re-reads the Universe).
- **Interaction budget:** tab switch must feel instant; the only work is a store set + one `{#key}` remount — no `invoke()` on the switch path. Drag-reorder is an in-memory array splice. Virtualize the strip only if open-tab counts can realistically exceed ~50 (today they don't; native `overflow-x` scroll + arrows handle the row).
- **Regression guard:** open 10 notes, switch rapidly — no lag, no stale content (on-screen === disk per the active tab). Close/pin/reorder must not corrupt the active selection or leave an orphaned content model (`closeNoteModel` must fire — MIG-076 §C). Tab-switch *while in Focus mode* is on the Editor-Surface Gate (the cross-note disk-write landmine).

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** multiple notes stay open; one click switches; pin/close/reorder behave; active selection always coherent.
- [ ] **Serves Constellation's core purpose:** sustains the working set for **Connection** (Act 2) without re-implementing any Editor operation.
- [ ] **Wires correctly to the Editor:** switching the active tab remounts `NoteEditor` via `{#key}`; the tab carries identity only, never content; `closeNoteModel` disposes on close.
- [ ] **Right-click present + correct:** shared `<ContextMenu>` (MIG-077), not hand-rolled; nine items work; close-* respect pinned state.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** fix `title="New tab"` → `$t('tabs.newTab')`; add `dir="auto"`/`detectDir()` to the tab title so RTL note names render correctly; `tabContextMenu.*` verified across all 15 locales.
- [ ] **Within budget:** zero boot IPC; instant switch; no `invoke()` on the switch path.
- [ ] **Obeys Rule 8:** does not recompute a derived view at boot/read (N/A — transient session state, not a derived surface).
- [ ] **Holds its invariants:** no orphaned content models on close; pinned tabs survive Close-All/Others; active tab never dangles after close; reorder preserves identity.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **— (no measurement yet; expected trivially met — zero boot IPC)**
Notes: Right-click is the strong point — already shared + fully localized via MIG-077 A1. Two i18n/RTL gaps to close at bring-up: the hardcoded `title="New tab"` tooltip and the missing `dir="auto"` on tab titles. Rule 8 is **not a violation** here — tabs are transient in-memory session state, not a write-time-derived view; the only persistence is opt-in named workspaces. Open question for bring-up: should the last session's open tabs auto-restore at boot? If yes, it must re-open *saved paths* (as `restoreWorkspace` does) and must NOT become a Universe re-walk. Folded here (no separate paper): pin/reorder/drag, middle-click close, the scroll arrows, the new-tab `+`/bulb button, the maturity dot + library-color stripe on each tab.
