# Plan: Universe Setup Wizard (Multi-Step)

Transform `UniverseSetup.svelte` from a single-step form into a multi-step wizard with 3 steps:

## Step 0: Welcome / Choose Action
- **Create New Universe** — proceed to Step 1
- **Open Existing Universe** — folder picker → reads `universe.json` from the selected directory → registers it in the registry → calls `onCreated` (skip Steps 1 & 2)

## Step 1: Name & Location (current screen, mostly unchanged)
- Universe name input
- Folder location picker
- "Next" button (instead of "Create Universe") → creates the universe directory + `universe.json`, then advances to Step 2
- "Back" button to return to Step 0

## Step 2: Add Vaults & Child Universes
- Shows the universe name at the top ("Setting up: MyFirstUniverseTest")
- **Add Vault** button — opens folder picker, validates `.obsidian/` folder, registers the vault
- Lists added vaults with remove buttons
- **Add Child Universe** button — opens folder picker to link an existing universe as a child
- Lists added child universes with remove buttons
- **Finish** button — calls `onCreated(entry)` to proceed to the main app
- "Skip" link to finish without adding anything

## Migration Flow
If `needsMigration` is true, skip Step 0 and go straight to Step 1 (migration path), then Step 2.

## Implementation

### 1. Rust: `open_existing_universe` command (`universe.rs`)
New Tauri command that:
- Takes a `path: String` (user-picked folder)
- Validates that `{path}/universe.json` exists and is valid JSON (`UniverseMeta`)
- Reads the `UniverseMeta` to get the name and created date
- Checks it's not already registered (by path)
- Adds an `UniverseEntry` to the registry
- Sets it as active
- Returns the `UniverseEntry`

Register in `lib.rs`.

### 2. TypeScript: `openExistingUniverse()` in `universe/store.ts`
IPC wrapper for the new command.

### 3. Rewrite `UniverseSetup.svelte` as wizard
- Add `step` state: `0 | 1 | 2`
- Step 0: Welcome screen with two large buttons (Create New / Open Existing)
- Step 1: Current name+location form, but button says "Next" and goes to step 2
- Step 2: Vault & child universe management with add/remove, then "Finish"
- After creating universe in Step 1, store the `UniverseEntry` and `setActiveUniverse` so that Step 2 vault operations go to the right universe

### 4. i18n: Add new keys in all 15 language files
New keys under `universe.setup`:
- `welcome`, `createNew`, `createNewDesc`, `openExisting`, `openExistingDesc`
- `next`, `back`
- `addVaultsHeading`, `addVaultsDescription`
- `addVault`, `addChildUniverse`, `finish`, `skip`
- `noVaultsYet`, `vaultAdded`, `childAdded`
- `openError` (for invalid universe folder)

### 5. Help docs
Update `docs/help.notesconstellation.com/Universe/Universe.md` with the new wizard flow.

## Files to modify
1. `src-tauri/src/universe.rs` — add `open_existing_universe` command
2. `src-tauri/src/lib.rs` — register new command
3. `src/lib/universe/store.ts` — add `openExistingUniverse()` wrapper
4. `src/lib/components/UniverseSetup.svelte` — rewrite as 3-step wizard
5. `src/lib/i18n/*.json` (15 files) — add new keys
6. `docs/help.notesconstellation.com/Universe/Universe.md` — update help docs
