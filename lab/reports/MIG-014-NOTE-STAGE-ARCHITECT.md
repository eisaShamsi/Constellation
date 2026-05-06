# MIG-014 — Note-stage taxonomy: Living Link 6-stage baseline + extensible custom stages

**Date**: 2026-05-06
**Closes**: PJ-007 (Pending Jobs v1.1)
**Status**: Phase 1 — Architect (Migration Rule §1)
**Boss-approved scope**: Pending Jobs v1.1 PJ-007 entry, fully detailed; this doc maps the territory and surfaces the implementation invariants. Plan doc (Phase 2) follows for Boss approval before any code edit.

---

## §1 · Mission

Replace the closed 4-stage Zettelkasten taxonomy in `PropertyEditor.svelte:478-485` with the **Living Link 6-stage baseline** (`spark / birth / growth / maturity / dormancy / archival`) plus **per-Universe extensible custom stages**. The system ships a canonical baseline; users can add their own stage values; PropertyEditor and the NotePane breadcrumb both surface baseline + custom in a single combobox; downstream surfaces (filter, search, future Knowledge Health Dashboard) treat the values uniformly.

**Why this MIG exists**: PKM systems impose closed taxonomies; Constellation surpasses them. Boss manifesto (2026-05-05/06):

> The note is the heart and mind of Constellation. It is not solid, but flexible and liquid. It can be formed in any way, shape, or method. It should surpass any PKM note systems.

The closed-baseline-with-extensibility model honors the Living Link Architecture's native vocabulary AND the user's right to bring their own method.

---

## §2 · Predecessor Lookup (Law 3.2)

Verified against current code, not memory.

### Where it lives now

| Surface | File:line | Current behavior |
|---|---|---|
| **Note frontmatter** | YAML in every `.md` file | `stage:` field, free-form value |
| **PropertyEditor stage dropdown** | `src/lib/components/PropertyEditor.svelte:478-485` | Hardcoded `<select>` with 4 Zettelkasten options: `fleeting / literature / permanent / synthesis`. `onchange` fires `updateValue(idx, v)` + `onstagechange?.(v)` callback. Lowercase normalization applied. |
| **NotePane breadcrumb stage dropdown** | `src/lib/components/NotePane.svelte` (per `90c1ea8` §136 redesign) | Reads `stage` prop, shows current value as a clickable badge that opens a dropdown. Same value-set as PropertyEditor today. |
| **i18n keys** | `src/lib/i18n/<locale>.json` → `notePane.stage.*` | All 15 locales already have **10 keys**: 4 Zettelkasten (`fleeting / literature / permanent / synthesis`) + 6 Living Link (`spark / birth / growth / maturity / dormancy / archival`). Lifecycle keys shipped in `c95a0e6`. |
| **`LinkStage` type (links, separate)** | `src/lib/libraries/store.ts:1521` | Living Link 6-stage applied to LINKS. **Stays unchanged.** |
| **`UniverseMeta` schema** | `src-tauri/src/universe.rs:11-22` | Currently: `name`, `created`, `version`, `children` (cUniverse children), `notes_folder`. No `custom_stages` field yet. |

### Where the replacement goes

**Same place** for both UI surfaces (PropertyEditor + NotePane breadcrumb). A different place is forbidden by Law 3.2 unless explicitly justified — and there's no reason to relocate.

| Surface | Replacement |
|---|---|
| PropertyEditor stage dropdown | Replaced in-place. `<select>` becomes a combobox sourced from `[...BASELINE, ...universe.custom_stages]`. Inline-add: typing a new value + Enter persists to `custom_stages`. |
| NotePane breadcrumb dropdown | Replaced in-place. Read-only dropdown sourced from same combined list (no inline-add — PropertyEditor is the add surface). |
| `UniverseMeta.custom_stages` | NEW field on existing struct. Persists to `<universe>/.constellation/universe.json`. Default `[]`. |
| Settings → Notes → "Manage custom stages" panel | NEW Settings tab/section. List, rename, reorder, delete. Emoji picker. |
| i18n keys for new UI strings | NEW keys for `propertyEditor.addCustomStage`, `settings.notes.manageCustomStages`, etc. en + ar this MIG; 13 others queued via PJ-014. |

### What gets cut

- The 4 hardcoded Zettelkasten `<option>` lines at `PropertyEditor.svelte:481-484`. The i18n keys for those values stay (autocomplete suggestions for users who've used them before — see §5.2).

### What gets kept

- Frontmatter `stage:` field shape (string value, no schema change to YAML).
- `onstagechange` callback pattern (NotePane Specs §4.8 — no `$effect` echo).
- Lowercase normalization (PropertyEditor:479 `prop.value.toLowerCase()`).
- Living Link 6-stage applied to LINKS (separate axis; this MIG doesn't touch link-side semantics).
- All NotePane Specs §4 hard invariants.

---

## §3 · Architecture

### §3.1 Schema

```rust
// src-tauri/src/universe.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomStage {
    pub name: String,    // user's chosen label (verbatim, any language)
    pub emoji: String,   // user-picked emoji; default "🏷️"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniverseMeta {
    pub name: String,
    pub created: String,
    pub version: u32,
    #[serde(default)]
    pub children: Vec<String>,
    #[serde(default)]
    pub notes_folder: Option<String>,
    #[serde(default)]                 // NEW
    pub custom_stages: Vec<CustomStage>, // NEW
}
```

`#[serde(default)]` ensures backward compat — older `universe.json` files without the field deserialize cleanly with `custom_stages: vec![]`.

`CustomStage` is an object array (not a flat string array) for forward-compat: the v1 ships with emoji as a stored field; future versions can add color, sort order, archived flag, etc. without another schema bump.

### §3.2 Tauri commands

```rust
#[tauri::command]
pub fn read_custom_stages(app: AppHandle) -> Result<Vec<CustomStage>, String>;

#[tauri::command]
pub fn add_custom_stage(app: AppHandle, stage: CustomStage) -> Result<(), String>;

#[tauri::command]
pub fn update_custom_stage(app: AppHandle, old_name: String, new_stage: CustomStage) -> Result<(), String>;

#[tauri::command]
pub fn remove_custom_stage(app: AppHandle, name: String) -> Result<(), String>;

#[tauri::command]
pub fn reorder_custom_stages(app: AppHandle, names_in_order: Vec<String>) -> Result<(), String>;
```

All commands operate on the active Universe's `universe.json`. Atomic write per call (no batching needed; custom_stages count is small).

### §3.3 PropertyEditor combobox

Replace `:478-485` with:

```svelte
{#if prop.key.toLowerCase() === 'stage'}
    <input
        class="pe-val pe-stage-input"
        type="text"
        list="stage-suggestions-{idx}"
        value={prop.value}
        placeholder={$t('propertyEditor.stagePlaceholder')}
        oninput={(e) => updateValue(idx, (e.target as HTMLInputElement).value)}
        onchange={(e) => {
            const v = (e.target as HTMLInputElement).value.toLowerCase().trim();
            updateValue(idx, v);
            onstagechange?.(v);
            // If the value is new (not in baseline + custom), persist as a new custom stage.
            if (v && !isKnownStage(v)) addCustomStage({ name: v, emoji: '🏷️' });
        }}
    />
    <datalist id="stage-suggestions-{idx}">
        {#each LIVING_LINK_BASELINE as bs}
            <option value={bs.name}>{bs.emoji} {$t(`notePane.stage.${bs.name}`)}</option>
        {/each}
        {#each $customStages as cs}
            <option value={cs.name}>{cs.emoji} {cs.name}</option>
        {/each}
    </datalist>
{:else if prop.type === 'checkbox'}
```

Browser-native `<datalist>` gives free-text + autocomplete + zero JS for the dropdown UX. Boss's "free-form if user wants" semantic is delivered without a dropdown widget reinvention.

### §3.4 NotePane breadcrumb dropdown

The breadcrumb (per `90c1ea8` §136 redesign) shows the current stage as a badge with a click-to-open dropdown. Update the dropdown's source list to `[...LIVING_LINK_BASELINE, ...customStages]`. **No inline-add in the breadcrumb** — keeps the breadcrumb simple; the user adds new stages in PropertyEditor or in Settings. Reads the same `customStages` writable store as PropertyEditor.

### §3.5 Settings → Notes → Manage Custom Stages panel

New panel under Settings → Notes. Lists user's custom stages with emoji + name. Per-row actions: edit name, change emoji (opens existing emoji picker), delete, reorder via drag handle. Shows the baseline 6 as a read-only header section ("Built-in stages — these always exist").

### §3.6 Frontend store

```typescript
// src/lib/libraries/store.ts (new section)
export interface CustomStage { name: string; emoji: string; }
export const LIVING_LINK_BASELINE: CustomStage[] = [
    { name: 'spark',     emoji: '✨' },
    { name: 'birth',     emoji: '🌱' },
    { name: 'growth',    emoji: '🌿' },
    { name: 'maturity',  emoji: '🌳' },
    { name: 'dormancy',  emoji: '😴' },
    { name: 'archival',  emoji: '📦' },
];
export const customStages = writable<CustomStage[]>([]);
// IPC wrappers: readCustomStages / addCustomStage / updateCustomStage /
// removeCustomStage / reorderCustomStages — thin wrappers over invoke().
```

The `customStages` writable is loaded once on Universe activation (extending the existing `set_active_universe` flow).

### §3.7 i18n

**Already shipped** (no work in this MIG): the 6 baseline labels in `notePane.stage.<key>` across 15 locales (per `c95a0e6`). Verified during predecessor lookup.

**New keys this MIG** (en + ar full; 13 others placeholder via PJ-014):
- `propertyEditor.stagePlaceholder` — "Type or pick a stage…"
- `propertyEditor.addCustomStage` — "Add custom stage" (used in inline-add ARIA label).
- `settings.notes.manageCustomStages` — section title.
- `settings.notes.customStagesIntro` — section description.
- `settings.notes.builtInStages` — "Built-in stages (cannot be edited)" header.
- `settings.notes.addCustomStageButton` — "+ Add stage".
- `settings.notes.deleteCustomStageConfirm` — confirmation message ("Delete the custom stage `{name}`? Notes already using it will keep the value but new selections won't include it.").

### §3.8 Order in the dropdown

```
[baseline in canonical lifecycle order]: spark, birth, growth, maturity, dormancy, archival
[user's custom_stages in chronological-add order]: <as the user added them>
```

Boss-confirmed. `customStages` is stored as an ordered `Vec<CustomStage>` so the order is durable across reads.

---

## §4 · Invariants (must not break)

Cited against `docs/NotePane Specs v1.0.md` and `docs/Constellation Development Laws v1.3.md`:

1. **NotePane Specs §4.1** — No `$effect` echo loops. Stage updates flow via `onstagechange` callback, not via reactive `$effect`. The new combobox preserves this contract.
2. **NotePane Specs §4.2** — No value-prop → CM6 doc sync `$effect` (BUG-015). This MIG doesn't touch the editor; pattern preserved.
3. **NotePane Specs §4.5** — No reactive recalculation during typing. The combobox `oninput` writes to `prop.value` but the parsed-frontmatter cache (NotePane Specs §4.7) is untouched until tab switch / mode transition.
4. **NotePane Specs §4.7** — Cached parsed frontmatter. Stage edits trigger `reparse()` only on save, not per keystroke.
5. **NotePane Specs §4.8** — Stage sync via callback. The new combobox keeps the existing `onstagechange?.(v)` invocation path.
6. **CLAUDE.md Law 2.6** — Universe is the right scope. `custom_stages` lives in `universe.json` per Law 2.6's "Universe holds bookmarks, settings, bases, and now this." Different Universes can have different vocabularies.
7. **CLAUDE.md Law 4.3** — Display, not domain. Second Screen's PropertyEditor mounts the same component; no duplication.
8. **CLAUDE.md Performance Rule 3** — No `invoke()` on the keystroke hot path. The inline-add fires `addCustomStage` only on `onchange` (commit), never `oninput` (per keystroke).
9. **Living Link spec §IV** — 6-stage lifecycle on LINKS unchanged. This MIG extends the vocabulary to NOTES; the link-side mechanics (weight decay, traversal tracking, etc.) stay specified for PJ-006 P2/P3.
10. **M11 zero-diff** — `git diff src-tauri/src/lexicon/` empty across this MIG (verified mechanically). No reason to touch lexicon.
11. **File Over App (Law 1.3)** — `custom_stages` lives in `universe.json` (a file the user can read/edit). Frontmatter `stage:` values are user-readable strings.

---

## §5 · Open questions resolved (Boss-approved or proposed-default-with-rationale)

### §5.1 Backward compat for unknown stage values

Some existing notes carry `stage: fleeting` (Zettelkasten value, no longer in baseline). Two approaches were considered:

- (a) Auto-migrate on first boot — walk every note, add Zettelkasten values to `custom_stages`. **Rejected**: violates Law 1.3 (no silent file modification) and Rule 8 (no boot-time corpus walk).
- (b) Render verbatim, let the user upgrade incrementally. **Approved**: when a note opens with a stage value not in baseline + user's custom, the combobox shows it as the current value (rendered verbatim, no special styling). If the user then types a different value, the old value becomes orphaned (no longer surfaces in suggestions for new notes). If the user keeps it and saves, the value persists in frontmatter unchanged. The user can manually add `fleeting` to their `custom_stages` via Settings if they want it back as a suggestion.

This honors "the system tracks but doesn't enforce" — old data is preserved, the user controls migration.

### §5.2 i18n keys for retired Zettelkasten stages

`notePane.stage.fleeting / literature / permanent / synthesis` (4 keys × 15 locales) currently exist. **Keep them**. They serve as autocomplete labels if a user manually adds those values to their `custom_stages`. Removing the keys would break old notes' breadcrumb display when those notes appear in a fresh Universe that doesn't have the values in `custom_stages`. Bytes are negligible; clarity wins.

### §5.3 Emoji storage

`CustomStage` is an object `{name, emoji}`, not a flat string. Approved by Boss in Pending Jobs v1.1. Forward-compat for picker UI in v1; allows future fields without schema bump.

### §5.4 Order in the dropdown

Baseline first (canonical lifecycle order), custom in chronological-add order. Approved.

### §5.5 Custom stages and the stage filter / search

Out of scope for this MIG. Search currently filters by literal `stage:` value match. Custom stages will work because they're stored verbatim in frontmatter. Future MIG can add stage-aware UI to SearchHub.

### §5.6 Custom stages and the future Knowledge Health Dashboard (PJ-006 P5)

Out of scope. Dashboard will treat baseline + custom uniformly when it ships.

### §5.7 What if two Universes have different `custom_stages`?

That's the design point. A research Universe might have `chrysalis / preprint / published`; a personal Universe might have `idea / brewing / done`. The note's frontmatter `stage:` value is preserved verbatim across Universes; the *interpretation* (whether a value is "known" or "orphaned") is per-Universe. Cross-Universe federation (cUniverse) inherits the parent's `custom_stages` for display purposes — this is the same pattern federated libraries use. **No-op for this MIG** — the combobox just reads the active Universe's stages.

---

## §6 · Phased rollout

Six phases under Migration Rule §3 (Build), each landable as one commit, each with its own verification clause. The Plan doc (Phase 2 of the Migration Rule) will spell out the verification clauses verbatim.

### Phase 1A — Schema + Tauri commands

- `UniverseMeta.custom_stages: Vec<CustomStage>` with `#[serde(default)]`.
- Five Tauri commands: `read_custom_stages`, `add_custom_stage`, `update_custom_stage`, `remove_custom_stage`, `reorder_custom_stages`.
- Commands registered in `lib.rs`.
- Verification: cargo build clean; round-trip a write → read on a fresh Universe; round-trip on an old Universe (no `custom_stages` field in `universe.json`) — should deserialize as `[]`.

### Phase 1B — Frontend store + IPC wrappers

- `LIVING_LINK_BASELINE` constant.
- `customStages` writable store loaded on Universe activation.
- IPC wrappers for the five commands.
- Verification: store updates on add/update/remove/reorder; persists to disk; reloads correctly on Universe switch.

### Phase 1C — PropertyEditor combobox

- Replace `:478-485` with the `<input list="…">` combobox sourced from `[...LIVING_LINK_BASELINE, ...customStages]`.
- Inline-add on `onchange` if the value is new.
- Verification: type baseline value → autocompletes; type new value → persists as custom; old `stage: fleeting` notes render verbatim without throwing.

### Phase 1D — NotePane breadcrumb

- Update breadcrumb dropdown source list to combined baseline + custom.
- Read-only (no inline-add).
- Verification: breadcrumb shows current value; opens dropdown showing all options; switching value fires `onstagechange` callback as before.

### Phase 1E — Settings → Notes → Manage Custom Stages

- New panel.
- List, edit, delete, reorder, emoji picker.
- Verification: round-trip every operation through the IPC layer; verify `universe.json` reflects changes.

### Phase 1F — i18n (en + ar) + help + User Manual

- New UI keys in en + ar (13 others queued via PJ-014).
- Index help (`docs/help.uConstellation.World/Index/Index.md`) — no change (Index is term-level, not stage-level).
- NotePane help (if exists) — add a section on stages.
- User Manual (en + ar) — new section under Notes describing baseline + custom stages.

### Phase 1G — Audit (Migration Rule §4)

Three parallel agents:
- **Invariant checker** — NotePane Specs §4 + CLAUDE.md Performance Rules + Law 2.6 universe-scoped storage.
- **Drift checker** — any code that imports the old Zettelkasten 4-stage list; any settings flag that's now orphaned.
- **Migration-path checker** — fresh Universe (no `custom_stages` in JSON), pre-MIG-014 Universe (no field at all), mid-edit interrupt during `add_custom_stage`, rollback (revert this MIG and verify Universe still loads).

Findings into `lab/reports/MIG-014-NOTE-STAGE-AUDIT.md`. P0 fixed before close.

---

## §7 · Cross-checks against proven methods (Law 1.5)

User-extensible categorical metadata is a well-established pattern in PKM and beyond. Brief survey:

| System | Pattern | What we borrow |
|---|---|---|
| **Notion** | "Select" property type — closed list of options users can extend inline. Database-scoped (per database the options differ). | Our combobox follows the same UX (type-or-pick + inline add). Universe-scoped storage matches "database-scoped" granularity. |
| **Tana** | Supertags carry custom field configurations including categorical fields with user-extensible value lists. | Confirms per-scope storage of the value list is the right abstraction. |
| **Logseq / Obsidian** | Frontmatter accepts arbitrary values; no built-in extensibility UI. | Our v1 ships extensibility UI; gives us a clear differentiator. |
| **Linear / GitHub Issues** | Labels are user-extensible at the workspace/repo level. | Same scope-and-management pattern. |
| **Standard `<datalist>` HTML element** | Browser-native combobox; no custom widget. | Used directly — gives us free-text + autocomplete + zero JS for the picker. |

The combobox + per-scope storage pattern is industry-standard. No invention required.

---

## §8 · What this MIG does NOT do

- **Does not retire `LinkStage`** (`store.ts:1521`) — Living Link 6-stage on LINKS continues as specified for PJ-006 P3.
- **Does not change frontmatter parsing** — `stage:` field is read/written by existing YAML logic.
- **Does not auto-migrate old notes' Zettelkasten values** — preserved verbatim per §5.1.
- **Does not touch CM6 / `$lib/editor/`** — NotePane Specs §4.4 (editor parity) protected.
- **Does not extend to filter / search UI** — out of scope; future MIG.
- **Does not extend to the future Knowledge Health Dashboard** — out of scope; PJ-006 P5 will handle.

---

## §9 · Closing

This MIG is a focused single-MIG with six landable phases. The Plan doc (Phase 2) follows: phase-by-phase commits with verification clauses. Boss approves the Plan before any code edit.

When MIG-014 closes:

- PJ-007 → status `Done`, ID retired (kept in §7 of Pending Jobs with closing date + commit hash).
- PJ-006 P3 (link-side lifecycle) becomes the natural next step, sharing the 6-stage vocabulary that's now anchored in both `LIVING_LINK_BASELINE` (notes) and `LinkStage` (links).
- Orientation bumps to v1.41 in the close-out commit.

**Approval needed**: this Architect doc, then a Plan doc breaking Phase 1A–1G into landable commits with verification clauses. No code lands until the Plan is approved.
