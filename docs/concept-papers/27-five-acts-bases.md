# 27 — Five Acts & Workspace Bases (Concept Paper)

> Two sidebar sections that surface saved **`.base` lenses**: the system-shipped **Five Acts** host notes (`{universe}/Five Acts/*.md`, each embedding a ` ```base ` block) and the user's **Workspace Bases** (`.constellation/bases/*.base`). Both attach to the Editor — clicking an entry opens it as a tab. The lens *engine* that renders a base is a separate function; this paper covers the **sidebar listing + open + manage** surface only.

## 1. Function in hand
The **Five Acts** and **Bases** sidebar sections — rendered in `src/routes/+layout.svelte` (the section guarded by `{#if fiveActsNotes.length > 0}` and `{#if workspaceBases.length > 0}`), backed by Rust `src-tauri/src/lens/system_notes.rs` (`list_five_acts_notes`) and `src-tauri/src/bases.rs` (`list_workspace_bases`).

## 2. Purpose
**One job: surface the saved lenses that frame the Universe, and open them.** *Five Acts* serves the **Observation** Act directly — its v1 host note, "Observation — Recent Captures," is the intake queue (last 14 days). The directory is named after the Five Acts and is built to grow into host notes for **Connection / Tension / Synthesis / Conviction** (`system_notes.rs` module docstring). *Workspace Bases* are the user's own saved lenses — reusable cross-library queries, the same primitive applied to whatever cut of knowledge the user keeps returning to. Both exist so a saved view is one click away instead of re-typed each session. Justified: a saved lens with no list to find it in is unreachable; this section is that list.

## 3. What it is NOT
- **NOT** the lens engine. It does not parse or render the ` ```base ` block / `.base` YAML — `$lib/lens/store` + `BaseTab.svelte` do that when the tab opens (`bases/store.ts` header is explicit: "READING and EDITING a base is the unified lens engine's job").
- **NOT** a file tree. The "Five Acts" folder is *hidden* from the library tree (`hideFiveActsFolder`) and re-surfaced here as a curated section.
- **NOT** a writer of note content. It opens existing files; creation/deletion are separate explicit commands.

## 4. Wiring
- **Inputs (IPC read):** `list_five_acts_notes` → `FiveActsNoteEntry[]`; `list_workspace_bases` → `WorkspaceBaseEntry[]`. `workspace_bases` also arrives in the boot bundle (`boot_bundle.rs`); Five Acts is loaded right after boot (fire-and-forget, line ~2038). Both re-fetch on universe switch and after create/delete.
- **Outputs (IPC write):** none from the list itself. Sibling commands `create_workspace_base` / `delete_workspace_base` (Bases) write; `init_five_acts_system_notes` auto-creates the canonical host note at boot (idempotent, transfer-on-edit — never overwrites a user-edited file).
- **Consumers:** the Editor/tab system (a clicked entry becomes a tab), the lens-block renderer (downstream, in the opened tab), the federation sidebar grouping (cUniverse sub-groups).
- **Connection to the Editor (the gate):** active-universe entries call `openNoteTab(absolute_path, …)`; cUniverse Five Acts entries dispatch `constellation:open-note`. Either way the **Editor** mounts the `.md`/`.base` file — this section is a launcher *into* the gate, never an editor itself.

## 5. Right-click / context menu
- **Five Acts entries: NONE.** Neither active nor cUniverse entries have an `oncontextmenu`. Plausible gap — open in new tab / reveal / copy path would be reasonable; **flag for bring-up** (likely intentional minimalism for system notes, but unverified).
- **Workspace Bases (active universe): present, ONE item — Delete.** Triggered by a **hand-rolled `oncontextmenu`** that sets the shared `contextMenu` state, which renders via the **shared `<ContextMenu>`** component (line ~7337) — so the *menu chrome* is the MIG-077 shared component, but the *trigger + item list* is bespoke: `getContextMenuItems` special-cases `libraryId === '__workspace__'` to return only `[{ Delete }]`. Delete IS reachable only by right-click (no other affordance). **Debt to flag:** the trigger is hand-rolled rather than routed through `buildContextMenu` like the file tree; the single-item menu is intentionally simplified but should be confirmed against the MIG-077 one-source-of-truth intent in bring-up.
- **cUniverse Bases: NONE by design** — read-only federation (deleting a cUniverse's base would break the read-only guarantee; the source comment is explicit).

## 6. Multilingual
- Section headers go through `$t('sidebar.fiveActs')` and `$t('sidebar.bases')` — **keys present in `en.json`** (and must exist in all 15 locales — verify in bring-up). The Delete action uses `$t('actions.delete')`.
- **RTL:** entry names and cUniverse group names use `dir={detectDir(...)}`. Good.
- **Hardcoded English to flag:** the file-stem `display_name` ("Observation — Recent Captures") is the *filename on disk* and is shown verbatim — by File-Over-App design it is user/system data, not a UI string, so it is *not* a localization defect. The canonical host-note body text in `RECENT_CAPTURES_CONTENT` (system_notes.rs) is English-only; whether the shipped system note should localize is a **product decision to flag in bring-up**, not a code defect. No hardcoded English found in the section's own chrome.

## 7. Boot behavior
- **Runs at boot? Yes.** `list_workspace_bases` is in the boot bundle (`time_step!`-measured); `list_five_acts_notes` fires immediately after boot. `init_five_acts_system_notes` runs once at universe-init (from `ensure_search_db_ready`) to create the canonical host note.
- **Rule 8 status: ✅ reads-persisted.** Both commands are **directory scans of files already on disk** (`scan_bases_dir` reads `*.base`; `list_five_acts_notes_at` reads `*.md`). The `.base`/`.md` files ARE the persisted derived view (File-Over-App) — nothing is recomputed by walking the Universe. *Caveat:* the **lens results** rendered inside an opened base are computed by the lens engine at open time — that is the engine's Rule-8 question (separate paper), not this section's.
- **Cost:** a `read_dir` + per-file stat + (for bases) a small JSON parse for the display name, over a handful of files. Negligible (estimated <1 ms each); `list_workspace_bases` is individually timed in the boot bundle — **read the measured value in bring-up**.

## 8. Flag / gate & bring-up position
- **Gate today: NONE.** Neither section has an `enabledFeatures.*` or `SIGHT_*_ENABLED` guard — they render whenever their list is non-empty (`{#if …length > 0}`). To bring up behind a flag, a **new gate is needed** (e.g. `enabledFeatures.fiveActs` / `enabledFeatures.bases`), following the existing `enabledFeatures.*` pattern.
- **Bring-up phase:** depends on the **Editor (Phase 1, the gate)** for open, and on the **lens engine** for render. List itself is light; safest after the Editor + lens engine are proven. **Phase 6 (federation):** the cUniverse grouping (`fiveActsByCu` / `basesByCu`, read-only) brings up with federation, not before.

## 9. Budget
- **Boot budget:** within the boot-bundle envelope; the Five Acts fetch must not delay paint (it is post-boot, fire-and-forget — keep it so).
- **Interaction budget:** expand/collapse and click-to-open are instant (state toggles + one `openNoteTab`); no `invoke()` on the open path beyond the Editor's own load.
- **Regression guard:** open a Five Acts host note and a workspace base — each opens its tab and the embedded lens renders; right-click a workspace base → Delete works and the list refreshes; with a cUniverse attached, federated sub-groups appear read-only (no context menu). Switch universe → both lists clear and reload.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** clicking a Five Acts host note / a workspace base opens it and the embedded lens renders.
- [ ] **Serves Constellation's core purpose:** Five Acts surfaces the **Observation** Act (intake queue); Bases are reusable lenses framing the Universe.
- [ ] **Wires correctly to the Editor:** open routes through `openNoteTab` / `constellation:open-note` → the Editor gate; no silent reads.
- [ ] **Right-click present + correct (shared, not hand-rolled):** route the workspace-base trigger through `buildContextMenu` like the file tree; decide whether Five Acts entries deserve a (shared) menu; cUniverse entries stay menu-less by design.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** `sidebar.fiveActs` / `sidebar.bases` / `actions.delete` present in all 15 locales; `detectDir` on names; decide on host-note body localization.
- [ ] **Within budget:** post-boot fetch doesn't delay paint; open/expand instant.
- [ ] **Obeys Rule 8:** lists are disk scans (reads-persisted) — confirmed; no universe re-walk to build the list.
- [ ] **Holds its invariants:** transfer-on-edit (never overwrite a user-edited host note); read-only federation (never write into a cUniverse); "Five Acts" folder stays hidden from the library tree.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **— (boot-bundle `list_workspace_bases` measured; Five Acts fetch unmeasured — verify in bring-up)**
Notes: Rule 8 ✅ (both lists are disk scans of persisted `.base`/`.md` files — File-Over-App). Two debts to resolve before re-enable: (1) the workspace-base context menu is a **hand-rolled trigger** on top of the shared `<ContextMenu>` — fold into `buildContextMenu` (MIG-077 intent); (2) Five Acts entries have **no context menu** — confirm intentional. Five Acts is built to grow (Connection/Tension/Synthesis/Conviction host notes); only the Observation host note ships in v1. The lens *rendering* Rule-8 question lives in the lens-engine paper, not here.
