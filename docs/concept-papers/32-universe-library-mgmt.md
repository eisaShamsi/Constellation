# 32 — Universe & Library Management (Concept Paper)

> Boots the world the Editor edits. The Editor (paper 01) is the gate; this is the function that decides *which* set of `.md` files the gate opens onto. Unconditional core — there is no Constellation without an active Universe.

## 1. Function in hand
**Universe & Library Management** — three surfaces over one concern: the first-launch **Universe Setup** wizard (`src/lib/components/UniverseSetup.svelte`), the **Universe Manager** modal (`src/lib/components/UniverseManager.svelte`, opened by the sidebar Universe pill), and the **Library Manager** modal (`src/lib/components/LibraryManager.svelte`, opened by the sidebar "Manage"). IPC bridges: `src/lib/universe/store.ts` and `src/lib/libraries/store.ts`.

## 2. Purpose
Establish and switch the **active Universe**, and register/remove the **Libraries** (and optional cUniverse children) inside it — so the rest of the app has a concrete file set to read, index, and edit. It answers: *"which knowledge base am I in, and what's inside it?"* This is **upstream of the Five Acts** — it does not itself perform Observation/Connection/Tension/Synthesis/Conviction; it provides the *ground* on which they happen. Justified beyond doubt under *File Over App*: the source of truth is `.md` files in a directory tree, and something must point Constellation at that tree, register additional roots, and let the user move between them. Without it the Editor has nothing to open.

## 3. What it is NOT
- **Not** a file manager — it manages *Library registration* (roots in `libraries.json`) and *Universe membership*, not individual notes/folders (that's the File Tree, paper 02).
- **Not** the place derived views are computed — it registers roots; indexing/search/graph happen write-time downstream.
- **Not** a copy/sync tool — Libraries are read **in place**; "remove" de-registers, it never deletes files (`removeUniverseFromRegistry`, `removeLibraryWithCleanup` drop the entry, not the disk content).
- The **Importer** is folded in here (it lives inside `UniverseSetup` steps 3–4 + `src/lib/importers/store.ts`) only as a Universe-creation path; it is not a standalone running surface.

## 4. Wiring
- **Inputs (IPC read):** `list_universes`, `get_active_universe_path`, `check_migration_needed`, `resolve_universe_libraries`, `get_all_library_stats`, `pick_folder`. Stores read: `libraryStats`, `libraries` (`src/lib/libraries/store.ts`); `appSettings` (active-universe name in the sidebar).
- **Outputs (IPC write):** `create_universe`, `set_active_universe`, `open_existing_universe`, `link_library_as_universe`, `migrate_legacy_data`, `add_child_universe` / `remove_child_universe`, `add_library` / `remove_library`, `scaffold_starter_library`; importer: `importExecute`. Frontend: `onSwitch` / `onCreated` callbacks into `+layout.svelte` drive `loadLibraries()` + `loadAllStats()` and a full re-init.
- **Consumers:** every satellite — File Tree, Search/Index, Backlinks/Outgoing/Tags, Sky View, the boot snapshot IPCs — all read the library list this function resolves. A Universe switch calls `closeAll()` + `clearAllCascading()` and re-runs the boot path.
- **Connection to the Editor (the gate):** indirect but foundational. This function never edits a note; it sets the **active Universe** whose `libraries.json` the Editor's tabs draw from. On switch it tears down open tabs (the Editor's models close) and the new Library set re-seeds them. It attaches *behind* the gate — it decides what the gate can open, then hands control to it.

## 5. Right-click / context menu
- **None — confirmed by grep** (no `oncontextmenu` / `contextmenu` / `ContextMenu` / `buildContextMenu` in any of the three components). All actions are explicit buttons: per-Universe **Switch / Remove / Add child**; per-Library **Open folder / Remove**; wizard **Create / Open existing / Link library / Import**.
- **Gap flagged:** a Universe/Library row is exactly the kind of target that benefits from a right-click action menu (Switch, Rename, Open folder, Reveal in OS, Remove). The shared **`<ContextMenu>` / `buildContextMenu`** path (MIG-077) used by the File Tree/Tabs/panels should be wired here per-row rather than hand-rolling one — **bring-up action: decide whether to add it; if added, it MUST be the shared menu, not a new hand-rolled one.** Note `rename_universe` IPC already exists but has **no UI surface** — a right-click "Rename" would expose it.

## 6. Multilingual
- **`$t()` coverage: good.** Every user-facing string in all three components routes through `$t()` (`universe.setup.*`, `universe.manager.*`, `libraryManager.*`, `importer.*`). No hardcoded English in the rendered UI was found, with **one caveat**: a defensive fallback `$t('universe.manager.openExisting') ?? 'Open Existing'` (UniverseManager.svelte ~L233) and a few `|| 'literal'` fallbacks in the sidebar — these are safety nets, not primary strings, but should be verified present in all 15 locales so the fallback never fires.
- **RTL:** `UniverseSetup` (`dir={$dir}`) and `UniverseManager` (`dir={$dir}` on `.um-modal`) flip correctly. **`LibraryManager` does NOT set `dir`** on its modal — **flagged gap: add `dir={$dir}`** to match the other two. Path strings are intentionally `direction: ltr` (filesystem paths read LTR even in RTL UI) — correct.
- **15-locale completeness** (ar de en es fa fr he hi ja ko pt ru tr ur zh) must be verified for the `universe.*` / `libraryManager.*` / `importer.*` key families during bring-up — **unknown whether all keys exist in all 15 files; verify in bring-up.**

## 7. Boot behavior
- **Runs at boot? YES** — this is the *first* thing the shell does (`+layout.svelte` ~L2485): `list_universes` → `check_migration_needed` → if empty, mount `UniverseSetup`; else `set_active_universe` on the first activatable entry, then `loadLibraries()` (`resolve_universe_libraries`) + `loadAllStats()` (`get_all_library_stats`).
- **Rule 8 status: ✅ reads-persisted (compliant).** `resolve_libraries_recursive` reads `.constellation/libraries.json` (a stored manifest) and recurses children from `universe.json` — no Universe walk to derive the list. `get_all_library_stats` reads counts from the always-current **`note_meta` index**, NOT a filesystem walk — the old stat-walk impl (~7,600 stat calls, the ~1.5–3 s "counts trail in at ~3.5 s" cost) was **deliberately removed** (per the in-code comment citing LL-024).
- **Cost:** library-list resolve + stats aggregation = a few manifest reads + one indexed query → **milliseconds (estimated; the stat-walk it replaced was the measured 1.5–3 s cost now eliminated).** Not a boot bottleneck. The wizard mounts only on first launch / migration.

## 8. Flag / gate & bring-up position
- **Gate today: none.** There is no `enabledFeatures.*` flag and no `SIGHT_*` gate — this function is **unconditional core**, because the app cannot start without resolving an active Universe + its Libraries. `safeBootMode` (the master charter's minimal-mode flag) must **leave this ON**; it is part of the editor+tree spine alongside `cache_boot_snapshot_core`.
- **Bring-up phase: 1 (Core spine), immediately ahead of / alongside the Editor.** Depends on: the Rust universe/libraries layer (`universe.rs`, `libraries.rs`), `libraries.json` + `universe.json` on disk, and the `note_meta` index for stats. Nothing else depends on *it* being a togglable satellite — it is a precondition for every satellite.

## 9. Budget
- **Boot budget:** library resolve + stats within the `hydrated_ms` envelope (<2 s; today ≈ ms). Must **never** regress to a filesystem walk for counts (the removed cost).
- **Interaction budget:** opening either modal is instant (reads cached stores); a **Universe switch** is the heavy op — it tears down tabs and re-runs boot. Switch should feel like a fast re-boot, not a hang; the overlay/quiesce must cover the teardown.
- **Regression guard:** on a large Universe (7,600+ notes) open Library Manager and confirm counts appear instantly (no trailing-in); switch Universe and confirm tabs close cleanly + the new Library set loads; measure before/after any change to `resolve_universe_libraries` / `get_all_library_stats`.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** create / open-existing / link / import all produce a working active Universe; add/remove Library updates the tree; switch swaps the world without data loss.
- [ ] **Serves Constellation's core purpose:** provides the File-Over-App ground (the `.md` roots) on which the Five Acts run; "remove" never deletes disk content (reversible).
- [ ] **Wires correctly to the Editor:** a Universe switch closes open tabs/models cleanly and re-seeds from the new Library set; no stale tab survives the switch.
- [ ] **Right-click present + correct:** decision recorded; if added, per-row menu uses the **shared `<ContextMenu>` / `buildContextMenu`** (MIG-077), not a hand-rolled one; `rename_universe` exposed via UI.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** `universe.*` / `libraryManager.*` / `importer.*` keys present in all 15 locales; **`LibraryManager` gains `dir={$dir}`**; `?? 'literal'` fallbacks never fire.
- [ ] **Within budget:** counts instant on a 7,600-note Universe; switch is a fast re-boot, not a hang.
- [ ] **Obeys Rule 8:** library list from `libraries.json`; counts from `note_meta` — never a Universe walk on read/boot.
- [ ] **Holds its invariants:** remove = de-register (files untouched); active-Universe resolution skips moved/deleted entries and falls back to setup; cUniverse cycles are guarded (`visited` set in `resolve_libraries_recursive`).
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** (paper written; bring-up audit pending) · Budget met: **— (Rule 8 already compliant in code; switch-cost + i18n-completeness unverified)**
Notes: Rule 8 is **already satisfied** — the costly filesystem-walk for counts was removed in favor of the `note_meta` index. Two concrete debts to clear in bring-up: (1) **`LibraryManager` missing `dir={$dir}`** (RTL gap); (2) **no right-click menu** on Universe/Library rows despite a useful action set + an unexposed `rename_universe` IPC — if added, use the shared MIG-077 menu. 15-locale key completeness for the `universe.*` / `libraryManager.*` / `importer.*` families is **unknown — verify in bring-up.**
