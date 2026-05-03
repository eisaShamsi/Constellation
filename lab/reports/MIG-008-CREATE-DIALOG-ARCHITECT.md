# MIG-008 — Create-Dialog Standardization · Architect

**Date**: 2026-05-03
**Triggered by**: Boss directive 2026-05-03 — "Whenever I created a folder it is created in the respective location under the name 'New Folder'. It shouldn't work this way. What I want it to do is to follow the standard way of any file system. A popup dialog box should emerge to name the new folder and to choose the location. Same thing should happen when creating new note, base or library."
**Project memory**: `project_create_dialog_standardize.md`
**Related**: `project_rename_collision_popup_wanted.md` (composes with this work)

---

## 1 · The territory

Constellation has four "create new X" affordances, each implemented inconsistently. The inventory:

| Affordance | Entry points | Default name | On collision | Has dialog? | Post-create UX |
|---|---|---|---|---|---|
| **New Folder** | sidebar context menu only | `"New Folder"` | reject (Rust error) | ❌ | nothing — silent |
| **New Note** | sidebar context menu, command palette | `"Untitled"` | auto-increment (Rust `resolve_filename_collision`) | ❌ | open in tab + auto-enter edit mode |
| **New Base** | command palette → `NewBaseDialog`, folder context menu → inline | `"Untitled Base"` | auto-increment (frontend 100-iter loop) | ✓ `NewBaseDialog.svelte` (name + library picker) | open in tab |
| **New Library** | command palette, Library Manager `<input>` + Create button | `"My Library"` | reject (Rust `add_library` dedup) | partial — `pick_folder` for location, but name from inline input or hardcoded | open library |

**Pain:**
- Folders silently get named "New Folder" with no way to specify the name in advance.
- Note creation works but offers zero affordance for picking a non-default name.
- Base already has a dialog (`NewBaseDialog`) — the others should match its pattern.
- Library's folder picker exists but the name comes from a separate Library-Manager screen `<input>`, not a unified flow.
- Cross-cutting: no consistent collision-handling UX, no consistent illegal-character validation, no consistent location confirmation.

**What stays the same**: the Rust IPCs (`create_folder`, `create_note`, `create_workspace_base`, `create_new_library`) — they're correct primitives. This MIG is purely frontend UX standardization.

---

## 2 · Invariants

The new design must hold every one of these. Each has a verification clause attached at Build phase.

| # | Invariant | Why |
|---|---|---|
| **I1** | Dialog is the SINGLE entry point for every create operation across all four affordances — no inline auto-creation paths remain | The "consistency" half of Boss's ask |
| **I2** | Name input pre-filled with the affordance's default, pre-selected for one-keystroke overtype | OS-native UX expectation (Explorer / Finder behaviour) |
| **I3** | Location is **shown** when invocation context already knows it (right-click on folder X → location is X, displayed read-only); **picker** when invocation doesn't (command palette without context, Library Manager) | Match Explorer / Finder: don't ask the user something the system already knows |
| **I4** | Esc cancels with no side effect; Enter commits when valid; Cancel button equals Esc; Create button equals Enter | Keyboard parity with OS-native dialogs |
| **I5** | Empty name → Create disabled. Illegal characters (Windows: `\ / : * ? " < > \|`) → inline error + Create disabled. Collision → inline error + Create disabled UNTIL the planned filename-collision popup lands, at which point it routes there | Bad input doesn't waste an IPC roundtrip |
| **I6** | Existing post-create routing preserved: Note opens in tab + edit mode; Base opens in tab; Folder + Library do not auto-open anything | No regression on what actually happens after Create |
| **I7** | Every current entry point (4 affordances × multiple triggers each) routes to the dialog; no orphaned auto-creation handler remains | "Drift surface zero" — once shipped, there's only one way to create |
| **I8** | RTL works — Arabic / Hebrew / Persian / Urdu render correctly, focus order respects reading direction | CLAUDE.md "Language-First by Design" |
| **I9** | Accessibility — focus on name input at open, focus returns to invoking element on close, aria-labels on all controls, dialog is focus-trapped while open | Standard a11y for modals |
| **I10** | The dialog's `kind` (`'folder' | 'note' | 'base' | 'library'`) drives title, default name, location-picker mode, and any kind-specific extras (Base needs library multi-select; others don't) | Single component, type-driven specialization |
| **I11** | i18n complete in all 15 locales for any new strings | Constellation contract |

---

## 3 · Design options

### Option A — Single shared `<CreateItemDialog>` component, kind-driven, modal

A new component takes a `kind` prop and renders:
- Title: "New Folder" / "New Note" / "New Base" / "New Library"
- Read-only location label OR location picker (per I3)
- Name input (pre-filled per I2)
- Kind-specific extras slot (Base's library picker goes here)
- Inline validation error region (per I5)
- Cancel + Create buttons

Wiring: every existing entry point's handler stops calling `createX()` directly and instead opens the dialog with appropriate `kind` + `parentPath`. The dialog's Create handler calls the existing Rust IPC, then runs the affordance's existing post-create routing.

- **Speed**: medium. ~1 component, ~4 entry-point rewires, ~10 i18n keys × 15 locales.
- **Effort**: medium. Single component absorbs the variation; the rewires are mechanical.
- **Risk**: low. Additive — Rust IPCs unchanged, post-create routing unchanged. Failure mode is "dialog doesn't open" — easy to spot, easy to roll back per-affordance.
- **Pros**: matches Boss's stated UX expectation (Explorer/Finder), satisfies I1+I7 (single source of truth), supports I3 (shown vs picked location), composes naturally with the planned collision popup.
- **Cons**: one extra modal click for users who liked auto-create + inline-rename. Minor; Esc cancels.

### Option B — Inline tree-row input (no modal)

Click "+ New X" → an inline `<input>` appears at the create location in the sidebar (like macOS Finder's "New Folder" gesture). User types name + Enter; Esc cancels.

- **Speed**: medium. The inline-rename infrastructure already exists in `FileTree.svelte`; extend it to "rename a not-yet-created entry."
- **Effort**: medium. Per-affordance integration, but no new component.
- **Risk**: low.
- **Pros**: no modal, fastest gesture, feels native.
- **Cons**: doesn't satisfy I3 (location picking) — the location is implicit from where the user invoked. Doesn't fit Library / Base which need different parent surfaces (Library lives outside any sidebar tree; Base may target the workspace dir, not a library folder). Two-form solution would be needed: inline for Folder/Note, modal for Base/Library — re-introduces the inconsistency this MIG is meant to remove.

### Option C — Modal with full template / properties picker (rich)

Option A plus a template selector, frontmatter property pre-fill, kind="note" stage choice, etc.

- **Speed**: slow.
- **Effort**: high. Multiple sub-controls, more validation surface.
- **Risk**: medium. Complex UI = more bugs; harder to audit RTL + a11y.
- **Pros**: most powerful — users can set everything at create time.
- **Cons**: overkill for the immediate ask. Boss asked for "name + location," not "name + location + 8 other knobs." Risk of scope creep delaying the actual fix Boss wants.

---

## 4 · Recommendation

**Option A.** Matches Boss's stated expectation precisely; single shared component achieves the consistency goal in I1+I7; low risk because Rust IPCs and post-create UX both stay; composes naturally with the planned collision popup (I5's "until that lands"); doesn't preclude later Option-C-style enhancements (the kind-specific-extras slot is the extension point).

Option B's inline-input gesture stays available as a future "power user" mode AFTER Option A ships, if Boss wants it. Don't ship both at once — would re-fragment the UX.

---

## 5 · Build plan (proposed Phase 2 outline — for Boss approval before cascade)

Each step lands as one commit with a verification clause Boss can test.

| Step | What | Verification |
|---|---|---|
| **§Build.1** | Build `src/lib/components/CreateItemDialog.svelte` standalone (no integrations yet). i18n keys added to `en.json` + `ar.json`; other 13 locales use en-fallback. Component supports all four `kind` values, location-shown vs location-picker mode, kind-specific-extras slot, validation. | Component renders correctly when force-mounted in dev; svelte-check clean. |
| **§Build.2** | Wire **New Folder** to dialog. Right-click context menu → dialog opens with location pre-filled, default name "New Folder" pre-selected. Cancel/Esc → no side effect. Enter → folder created. | Boss tests right-click on a folder → New Folder → dialog → type name → Enter → folder appears in tree with the typed name (not "New Folder"). |
| **§Build.3** | Wire **New Note** to dialog. Both entry points (right-click + command palette). Default name "Untitled". Post-create: tab opens + edit mode (preserved). | Boss tests both entry points; new note opens in tab with chosen name. |
| **§Build.4** | Wire **New Base** to dialog. The kind-specific-extras slot renders the library picker (existing logic from `NewBaseDialog` factored out). Old `NewBaseDialog.svelte` removed. | Boss tests both entry points (workspace + folder-context); base file created with chosen name + library picker still works. |
| **§Build.5** | Wire **New Library** to dialog. The location picker (existing `pick_folder` IPC) is invoked from inside the dialog when user clicks "Pick location". Library Manager's inline `<input>` removed in favour of the dialog. | Boss tests Library Manager + command palette; new library appears in sidebar. |
| **§Build.6** | Remove every now-orphaned auto-create handler + default-name fallback from the existing flows (the Folder "New Folder", Note "Untitled", Base 100-iter loop, Library "My Library" string). | Search confirms zero remaining `'New Folder'` / `'Untitled'` / `'My Library'` literals in handler code (only in i18n keys for display). |
| **§Build.7** | `/simplify` checkpoint over §Build.1–.6. Audit drift, sweep duplicated patterns, consolidate i18n where possible. | Three review agents return clean / minor cleanups. |
| **§Build.8** | Phase 4 audit. Verify every invariant I1-I11 against the final code. State-of-standing. | Audit doc lists each invariant + pass/fail evidence. |

After §Build.8 the MIG closes; the `project_create_dialog_standardize.md` memory marks it shipped.

---

## 6 · What this MIG does NOT include

- **Collision-popup UX** (project memory `project_rename_collision_popup_wanted.md`) — out of scope. The dialog shows an inline error on collision; when the collision popup ships separately, this dialog routes to it.
- **Template picker / property pre-fill** (Option C extras) — out of scope. The kind-specific-extras slot is the extension point if Boss wants it later.
- **Inline tree-row create gesture** (Option B) — out of scope. Optional power-user mode for a later MIG.
- **Behaviour of existing notes' default frontmatter** — unchanged. The dialog only controls the filename + location; the note's initial body / frontmatter still come from `buildDefaultFrontmatter` (`store.ts`).

---

## 7 · Approval gate

Boss reads §3 (options) + §4 (recommendation) + §5 (build plan). On "Option A approved", I cascade through §Build.1 → §Build.8 per the Plan-Approval-Equals-Build-Approval rule. Stop only at user-testable verification clauses (each §Build.N step) or genuine architectural surprise.
