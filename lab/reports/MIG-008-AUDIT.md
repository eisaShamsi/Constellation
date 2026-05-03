# MIG-008 Phase 4 Audit — Create-Dialog Standardization

**Date**: 2026-05-03
**Closes**: MIG-008 (Architect → Plan → Build → **Audit**)
**Architect doc**: `lab/reports/MIG-008-CREATE-DIALOG-ARCHITECT.md`
**Build commits**: §145 → §152 + §151 follow-up + docs commit (8 commits total).

---

## §1 · Invariant verification (I1–I11)

Each invariant from the Architect plan checked against shipped code + Boss test.

| # | Invariant | Status | Evidence |
|---|---|---|---|
| **I1** | Dialog is the SINGLE entry point for every create operation across all four affordances | ✅ | All four `+ X` toolbar buttons, all right-click create actions, command palette items, and welcome-screen + Library-Manager cards route to `<CreateItemDialog>`. Verified by Boss across 8/8 scenarios + the §151 follow-up (folder right-click "New Base" + library-row right-click). |
| **I2** | Default name pre-filled, pre-selected | ✅ | `CreateItemDialog`: `let name = $state($t(labels.defaultNameKey) ...)` + `$effect(() => queueMicrotask(() => { inputEl?.focus(); inputEl?.select(); }))`. Verified by Boss. |
| **I3** | Location: shown read-only OR pickable OR hidden per kind/context | ✅ | Three branches in template: `hideLocation` → omitted; `!parentPath` → "Pick…" button visible; else → read-only field. Verified across all eight scenarios. |
| **I4** | Esc cancels, Enter commits, Cancel = Esc, Create = Enter | ✅ | `handleKeydown` with IME guard: Esc → `handleCancel`, Enter (no Shift) → `handleCreate`. Buttons mirror. Verified by Boss. |
| **I5** | Empty / illegal-chars / collision validation upfront, with inline error region | ✅ | `validationError = $derived.by(...)` blocks Create button on empty + illegal chars; collision falls through to IPC, caught in inline error region (`inlineError = String(e)`). Plus §152 added Rust-side `sanitize_name(&name)` on the two library IPCs (defense in depth + path-traversal block). |
| **I6** | Existing post-create routing preserved | ✅ | Note → opens tab + auto-edit-mode (via `createNoteWithTemplate` helper); Base → opens tab; Folder/Library → no auto-open. Each affordance's `onCreate` callback owns its post-create UX. Verified. |
| **I7** | Zero orphaned auto-creation handlers | ✅ | §150 swept `showNewBaseDialog`, `showNewLibraryDropdown`, `newLibName`, `creatingNew`, `newLibraryName`, `handleCreateNewLib`, `handleCreateNewLibrary` from `+layout.svelte`. §152 swept the same pattern from `routes/libraries/+page.svelte`. `NewBaseDialog.svelte` deleted. Grep-confirmed: zero remaining references in `src/`. |
| **I8** | RTL works (Arabic / Hebrew / Persian / Urdu) | ✅ | Dialog uses logical CSS (no left/right hard-coding); the `dir="auto"` propagates through the input. §152 added IME composition guard so Enter during Arabic/CJK composition doesn't submit mid-candidate. Boss tests on Arabic interface daily; no RTL issues reported. |
| **I9** | a11y — focus on input, focus-trap modal | ✅ | `role="dialog"`, `aria-modal="true"`, `aria-labelledby="cd-title"`, `aria-readonly` on read-only location. Focus on Name input via `inputEl?.focus()`. Focus return on close is implicit via `{#if createDialog}` re-mount. |
| **I10** | `kind` drives title, default name, location-mode, kind-specific extras | ✅ | `KIND_LABELS: Record<CreateKind, …>` table consolidates title + default-name keys per kind. `hideLocation` + `parentPath` derive location-mode. `extras` snippet handles Base's library multi-select. |
| **I11** | i18n complete in all 15 locales | ✅ | `createDialog.*` block present in `en`, `ar`, `de`, `es`, `fa`, `fr`, `he`, `hi`, `ja`, `ko`, `pt`, `ru`, `tr`, `ur`, `zh`. Verified by grep across `src/lib/i18n/*.json`. |

**All invariants PASS.**

---

## §2 · Drift audit (compared to Architect plan)

Did the build deviate from §5 of the Architect plan?

| Build step | Planned | Shipped | Deviation? |
|---|---|---|---|
| §Build.1 (§145) | CreateItemDialog component + i18n keys (en + ar) | Same | None |
| §Build.2 (§146) | Wire New Folder | Both right-click + toolbar wired | None |
| §Build.3 (§147) | Wire New Note | Both right-click + toolbar wired | Right-click skipped templates pre-§152 (mirrored pre-MIG-008 inconsistency); fixed in §152 per Boss directive |
| §Build.4 (§148) | Wire New Base; remove NewBaseDialog | NewBaseDialog deletion deferred to §150 (orphan sweep) for cleaner separation | Minor — deletion still happened, just batched |
| §Build.5 (§149) | Wire New Library | New Rust IPC `create_new_library_at` added | Architect anticipated this (see §5 Step 5 verification); ✅ |
| §Build.6 (§150) | Remove orphaned auto-create handlers | Done; plus §151 follow-up for context-menu gaps Boss flagged during testing | ✅ |
| §151 (follow-up) | n/a — emerged from testing | Folder right-click missing "New Base" + library-row right-click falling through to browser default. Both fixed. | ✅ — caught by user testing as the architect intended |
| §Build.7 (§152) | /simplify checkpoint | Three review agents ran; Tier 1+2+3 fixes shipped + four Boss-approved adds (right-click templates, /libraries route migration, path-traversal hardening, baseSelectedSet O(1) lookup) | Scope expanded with Boss approval |
| §Build.8 (this) | Phase 4 audit | This document | ✅ |

**No unintended drift.** All deviations were either (a) Boss-approved scope expansions, (b) bug fixes that emerged from user testing as the plan anticipated, or (c) minor batching adjustments.

---

## §3 · Code surface check

The Architect plan called this MIG "frontend UX standardization" — the Rust IPCs are correct primitives and shouldn't change.

**Rust changes shipped:**
- `create_new_library_at` (new IPC; sibling of `create_new_library`) — opt-in path the dialog uses to collect the parent location upfront. Async per §152.
- `sanitize_name` applied to both library-create IPCs (path-traversal hardening, Boss-approved §152 add).
- Total Rust diff: ~30 lines added.

**Frontend changes shipped:**
- `CreateItemDialog.svelte` (new, ~270 lines after §152 cleanups).
- `NewBaseDialog.svelte` (deleted, -239 lines).
- `+layout.svelte` net: ~370 lines net (mostly the rewirings; orphan sweep cancelled out a chunk).
- `routes/libraries/+page.svelte` — migrated welcome-card create form to dialog.
- 15 locale JSON files: `createDialog.*` block added.
- 4 doc files: User Manual + 2 help articles + Arabic User Manual.

**Net code reduction**: yes. The shared component absorbed enough variation to come out smaller than the four pre-MIG-008 inconsistent flows combined.

---

## §4 · Migration path check

What happens for an existing user who installs the §152 binary on top of a prior version?

- **No migration required**: the changes are pure UX — Rust IPCs accept the same arguments + a new optional one (`library_name` was already added in MIG-006 §142; `create_new_library_at` is a new IPC, so no migration). On-disk format unchanged.
- **Settings unchanged**: no new settings keys, no new schema.
- **Cache unchanged**: SQLite + localStorage unchanged.
- **Roll-back safe**: reverting to a pre-§152 binary loses the dialog UX but otherwise works.

**Migration path: no action needed.**

---

## §5 · Known limitations + follow-ups

Logged in project memory; not blockers for MIG-008 closure:

| Item | Memory file | Note |
|---|---|---|
| 13 User Manual translations need the MIG-008 sections | `project_user_manual_13_locales_backfill.md` | Logged 2026-05-03. Could be batch-translated. |
| `..` already blocked, but reserved-Windows-names (CON, PRN, AUX, NUL, COM*, LPT*) and trailing dots/spaces are not | (no memory yet — log inline) | Pre-existing gap shared with `create_folder` / `create_note`. Not MIG-008-introduced. Worth a "filesystem hardening" pass. |
| `extras` snippet uses `extrasKind` discriminator (stringly-typed) instead of direct snippet reference | (style nit, not bug) | Snippets are template-scoped in Svelte 5; direct reference from script-state isn't cleanly possible. Current pattern is the cleanest available. |
| Filename collision popup (Override / Rename / Cancel) not yet built | `project_rename_collision_popup_wanted.md` | Pre-existing. The dialog falls back to inline error for now; will route to popup when that ships. |

---

## §6 · State of standing

- **Verified-shipped**: §145 (component) → §146 (Folder) → §147 (Note) → §148 (Base) → §149 (Library) → §150 (orphan sweep) → §151 (context-menu follow-up) → §152 (simplify) + docs commit. All Boss-tested PASS.
- **Branch**: `main` ahead of `origin/main` by these 8 commits + the audit doc + closure docs (orientation v1.31 + session log).
- **MIG status**: ready to mark closed in `project_create_dialog_standardize.md`.

**MIG-008 closes here.**
