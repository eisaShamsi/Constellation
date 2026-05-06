# MIG-014 — Note-stage taxonomy Plan (Phase 2 of the Migration Rule)

**Date**: 2026-05-06
**Architect**: `lab/reports/MIG-014-NOTE-STAGE-ARCHITECT.md` (Boss-approved 2026-05-06)
**Closes**: PJ-007 (Pending Jobs v1.1)
**Hard constraint**: NotePane Specs §4 invariants preserved (no `$effect` echo, no value-prop → CM6 sync, no reactive recalc during typing). M11 zero-diff (`git diff src-tauri/src/lexicon/` empty). Verified mechanically before each commit.

---

## §0 · Pre-flight (no commit)

Before Phase 1A lands, confirm the following one-time decisions are locked:

| Question | Locked answer |
|---|---|
| Default emoji for the 6 baseline stages | `spark ✨ / birth 🌱 / growth 🌿 / maturity 🌳 / dormancy 😴 / archival 📦` (per Architect §3.6 `LIVING_LINK_BASELINE` literal). |
| Default emoji for newly-added custom stages | `🏷️` (per Boss decision in Pending Jobs v1.1). |
| Storage location for `custom_stages` | `<universe>/.constellation/universe.json` (per Architect §3.1). |
| Order in dropdown | Baseline first (canonical lifecycle order), then custom in chronological-add order. |
| Backward compat for old Zettelkasten values (`fleeting / literature / permanent / synthesis`) | Render verbatim, no auto-migration. The retired i18n keys stay as autocomplete labels for users who manually re-add them. |
| Number of locales updated for new UI strings in this MIG | 2 full (en + ar). 13 others queued via PJ-014 (User Manual backfill). |

**Pre-flight deliverables** (no commits, just session-log notes):
- Confirm `notePane.stage.<key>` exists for all 6 baseline values in 15 locales (already verified during Architect predecessor lookup).
- Confirm `LinkStage` in `store.ts:1521` is untouched scope.

---

## §1 · Phase 1A — Schema + Tauri commands

**Goal**: extend `UniverseMeta` with `custom_stages: Vec<CustomStage>` and ship the five Tauri commands. No frontend changes. No user-visible behavior change.

### Files touched

- `src-tauri/src/universe.rs` — extend `UniverseMeta` struct, add `CustomStage` struct, add five command implementations.
- `src-tauri/src/lib.rs` — register the five commands in the `invoke_handler!` macro.

### Algorithm

```rust
// universe.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomStage {
    pub name: String,
    pub emoji: String,
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
    #[serde(default)]                       // NEW
    pub custom_stages: Vec<CustomStage>,    // NEW
}

#[tauri::command]
pub fn read_custom_stages(state: State<UniverseState>) -> Result<Vec<CustomStage>, String> { … }

#[tauri::command]
pub fn add_custom_stage(state: State<UniverseState>, stage: CustomStage) -> Result<(), String> { … }

#[tauri::command]
pub fn update_custom_stage(state: State<UniverseState>, old_name: String, new_stage: CustomStage) -> Result<(), String> { … }

#[tauri::command]
pub fn remove_custom_stage(state: State<UniverseState>, name: String) -> Result<(), String> { … }

#[tauri::command]
pub fn reorder_custom_stages(state: State<UniverseState>, names_in_order: Vec<String>) -> Result<(), String> { … }
```

All five commands operate on the active Universe's `universe.json`. Each command:
1. Acquires `UniverseState.active_path` lock.
2. Reads `universe.json`, deserializes to `UniverseMeta`.
3. Mutates `custom_stages` (add/update/remove/reorder).
4. Serializes back to `universe.json` atomically (write to temp, rename).
5. Releases lock.

`#[serde(default)]` ensures backward compat: old `universe.json` files without `custom_stages` deserialize cleanly with `vec![]`.

### Verification (must pass before commit)

1. `cargo build --release --lib` from `src-tauri/` succeeds.
2. Round-trip test (manual via `cargo run` if needed, or unit test):
   - Fresh `UniverseMeta { … custom_stages: vec![] … }` serializes to JSON without breaking the existing schema.
   - Old `universe.json` (no `custom_stages` field) deserializes to `custom_stages: vec![]` cleanly.
   - `add_custom_stage` writes the new entry; `read_custom_stages` retrieves it.
   - `remove_custom_stage` by name removes it; subsequent read returns the list without it.
   - `reorder_custom_stages` with a permutation of existing names returns the list in the new order; with a non-existing name in the list returns an error.
3. **M11 zero-diff invariant**: `git diff src-tauri/src/lexicon/` returns empty.
4. No new warnings in `cargo build --lib` beyond the existing baseline.

### Commit message (skeleton)

```
MIG-014 §1A: schema + Tauri commands for custom_stages

Adds CustomStage { name, emoji } struct and extends UniverseMeta
with custom_stages: Vec<CustomStage> via #[serde(default)] for
backward compat. Five Tauri commands for read / add / update /
remove / reorder.

No user-visible change yet — frontend integration lands in §1B/§1C.

Architect: lab/reports/MIG-014-NOTE-STAGE-ARCHITECT.md §3.1 + §3.2
Plan: lab/reports/MIG-014-NOTE-STAGE-PLAN.md §1
```

---

## §2 · Phase 1B — Frontend store + IPC wrappers

**Goal**: expose the five Tauri commands as TS wrappers, define the `LIVING_LINK_BASELINE` constant, and wire a `customStages` writable that loads on Universe activation. No UI change.

### Files touched

- `src/lib/libraries/store.ts` — add `CustomStage` type, `LIVING_LINK_BASELINE` constant, `customStages` writable, five IPC wrappers.
- `src/routes/+layout.svelte` — extend Universe activation flow: after `set_active_universe` succeeds, call `readCustomStages()` and `customStages.set(result)`.

### Algorithm

```typescript
// store.ts (new section near LinkStage)
export interface CustomStage { name: string; emoji: string; }

export const LIVING_LINK_BASELINE: ReadonlyArray<CustomStage> = [
    { name: 'spark',     emoji: '✨' },
    { name: 'birth',     emoji: '🌱' },
    { name: 'growth',    emoji: '🌿' },
    { name: 'maturity',  emoji: '🌳' },
    { name: 'dormancy',  emoji: '😴' },
    { name: 'archival',  emoji: '📦' },
] as const;

export const customStages = writable<CustomStage[]>([]);

export async function readCustomStages(): Promise<CustomStage[]> { … }
export async function addCustomStage(stage: CustomStage): Promise<void> { … }
export async function updateCustomStage(oldName: string, newStage: CustomStage): Promise<void> { … }
export async function removeCustomStage(name: string): Promise<void> { … }
export async function reorderCustomStages(namesInOrder: string[]): Promise<void> { … }

/** Helper used by PropertyEditor + NotePane breadcrumb. */
export function isKnownStage(value: string, customs: CustomStage[]): boolean {
    if (LIVING_LINK_BASELINE.some(b => b.name === value)) return true;
    return customs.some(c => c.name === value);
}
```

Universe activation hook:

```typescript
// +layout.svelte (in the active-universe-changed handler)
const stages = await readCustomStages().catch(() => [] as CustomStage[]);
customStages.set(stages);
```

### Verification

1. `npm run check` passes (svelte-check + tsc).
2. `cargo build --release --lib` still passes.
3. Switch active Universe in dev mode → `customStages` writable updates correctly.
4. Add via `addCustomStage` → writable reflects change after explicit re-read; `universe.json` shows the new entry.
5. **M11 zero-diff invariant** check.

### Commit message (skeleton)

```
MIG-014 §1B: frontend store + IPC wrappers for custom_stages

Adds CustomStage type, LIVING_LINK_BASELINE constant, customStages
writable, and five IPC wrappers in store.ts. Universe activation
loads custom_stages into the writable.

No UI consumer yet — PropertyEditor + breadcrumb in §1C/§1D.
```

---

## §3 · Phase 1C — PropertyEditor combobox

**Goal**: replace the closed 4-stage `<select>` at `PropertyEditor.svelte:478-485` with a combobox sourced from `[...LIVING_LINK_BASELINE, ...$customStages]`. Inline-add on commit if the value is new.

This is the **first user-visible change** in the MIG. Boss-testable verification clause attached.

### Files touched

- `src/lib/components/PropertyEditor.svelte` — replace the stage `<select>` block at `:478-485`.
- `src/lib/i18n/en.json` — add `propertyEditor.stagePlaceholder` key.
- `src/lib/i18n/ar.json` — same key.

### Algorithm

Replace `:478-485` with:

```svelte
{#if prop.key.toLowerCase() === 'stage'}
    <input
        class="pe-val pe-stage-input"
        type="text"
        list="stage-suggestions-{idx}"
        value={prop.value}
        placeholder={$t('propertyEditor.stagePlaceholder') || 'Type or pick a stage…'}
        oninput={(e) => updateValue(idx, (e.target as HTMLInputElement).value)}
        onchange={(e) => {
            const v = (e.target as HTMLInputElement).value.toLowerCase().trim();
            updateValue(idx, v);
            onstagechange?.(v);
            if (v && !isKnownStage(v, $customStages)) {
                addCustomStage({ name: v, emoji: '🏷️' }).catch(err =>
                    console.warn('[PropertyEditor] addCustomStage failed:', err));
            }
        }}
    />
    <datalist id="stage-suggestions-{idx}">
        {#each LIVING_LINK_BASELINE as bs}
            <option value={bs.name}>{bs.emoji} {$t(`notePane.stage.${bs.name}`) || bs.name}</option>
        {/each}
        {#each $customStages as cs}
            <option value={cs.name}>{cs.emoji} {cs.name}</option>
        {/each}
    </datalist>
{:else if prop.type === 'checkbox'}
    …
```

Imports added at the top of the script block:
```typescript
import { LIVING_LINK_BASELINE, customStages, addCustomStage, isKnownStage, type CustomStage } from '$lib/libraries/store';
```

### Verification

**Automated** (must pass before commit):
1. `npm run check` passes.
2. `cargo build --release --lib` passes (no Rust changes but invocation routes through Tauri).
3. `git diff src-tauri/src/lexicon/` empty.

**Boss-testable** — sent as a tutorial after this commit:

> **What this is**: the stage field in the Properties panel of any note used to offer four fixed options (Fleeting / Literature / Permanent / Synthesis). Now it offers the **six Living Link lifecycle stages** as the baseline (Spark / Birth / Growth / Maturity / Dormancy / Archival), and you can **type any custom stage** you want — typing a value not in the list adds it as a custom stage automatically, with a default 🏷️ emoji you can later edit.
>
> **Step 1 — Pre-state**: open any note. Expand the Properties panel. Find the `stage:` row.
>
> **Step 2 — Pick a baseline stage**: click the input. A dropdown appears showing the 6 Living Link stages (with emoji prefix) plus any custom stages you've previously added (none yet, on first run). Pick `growth`. Press Tab to commit. Save the note.
>
> **Expected**: the breadcrumb at the top of the note updates to show `Growth` (with the appropriate emoji). The frontmatter on disk now has `stage: growth`.
>
> **Step 3 — Type a custom stage**: clear the field. Type `chrysalis` (or any value you want — Arabic, English, anything). Press Tab to commit.
>
> **Expected**: the value `chrysalis` is now your stage. It auto-added to the Universe's custom-stages list with the default 🏷️ emoji. Open another note in the same Universe → click the stage field → `chrysalis` now appears in the dropdown alongside the 6 baseline stages.
>
> **Step 4 — Universe scope**: switch to a different Universe (Universe Setup → pick another one). Open a note there. Click the stage field. **`chrysalis` does NOT appear** — custom stages are per-Universe by design.
>
> **Step 5 — Old notes preserved**: open a note that has `stage: fleeting` (Zettelkasten value from before this MIG). The breadcrumb shows it verbatim. The dropdown will offer baseline + custom + the literal `fleeting` is whatever was already there. Save the note without changing — the value `fleeting` is preserved on disk.
>
> **If you see this instead**:
> - Dropdown showing the 4 old Zettelkasten options → build didn't take, rebuild + reinstall.
> - Custom value typed but not appearing in next note's dropdown → IPC `addCustomStage` failed; check DevTools console for `[PropertyEditor] addCustomStage failed`.
> - Old `stage: fleeting` notes silently mutated to a baseline value → that's a bug; do NOT save anything else, copy the file content, and tell me.

1. Boss confirms the Properties panel offers the 6 baseline + can add custom values.
2. Custom values persist across note opens within the same Universe.
3. Old Zettelkasten values are preserved verbatim.
4. **M11 zero-diff** check.

### Commit message (skeleton)

```
MIG-014 §1C: PropertyEditor combobox replaces 4-stage Zettelkasten

Replaces the closed Zettelkasten dropdown at PropertyEditor.svelte
:478-485 with a <input list="…"> combobox sourced from
[...LIVING_LINK_BASELINE, ...$customStages]. Inline-add on
onchange: typing a value not in the combined list calls
addCustomStage with default 🏷️ emoji.

i18n: en + ar add propertyEditor.stagePlaceholder. 13 others
queued via PJ-014.

Old Zettelkasten values (fleeting/literature/permanent/synthesis)
preserved verbatim on disk; their notePane.stage.* i18n keys are
kept as autocomplete labels per Architect §5.2.

Boss test passed Stage 1 (combobox + inline-add + per-Universe
scope + old-value preservation).
```

---

## §4 · Phase 1D — NotePane breadcrumb dropdown

**Goal**: update the NotePane breadcrumb's stage dropdown to read from the same combined source list as PropertyEditor. Read-only — no inline-add.

### Files touched

- `src/lib/components/NotePane.svelte` — locate the breadcrumb stage dropdown (per `90c1ea8` §136 redesign; current dropdown source list is the 4 Zettelkasten values), replace with combined source.

### Algorithm

Locate the breadcrumb stage dropdown (in the script + template sections of `NotePane.svelte`). Replace any hardcoded list of Zettelkasten values with:

```svelte
<script>
    import { LIVING_LINK_BASELINE, customStages, type CustomStage } from '$lib/libraries/store';
    // …
    const stageOptions = $derived([
        ...LIVING_LINK_BASELINE,
        ...$customStages,
    ]);
</script>

<!-- in the breadcrumb template -->
{#each stageOptions as opt}
    <button class="bc-stage-option" onclick={() => selectStage(opt.name)}>
        {opt.emoji} {LIVING_LINK_BASELINE.some(b => b.name === opt.name)
            ? ($t(`notePane.stage.${opt.name}`) || opt.name)
            : opt.name}
    </button>
{/each}
```

`selectStage` flows through the existing `onpromote` / stage-update callback path (NotePane Specs §4.8 — no `$effect` echo).

### Verification

**Automated**:
1. `npm run check` passes.
2. `cargo build --lib` passes.
3. **M11 zero-diff** check.

**Boss-testable**:

> **What this is**: the small stage badge in the breadcrumb at the top of each note (Back ‹ Title · Stage ▾ · ⋮ · Trail). Until now it offered the 4 Zettelkasten stages. Now it offers the **same combined list** as the Properties panel — 6 Living Link baseline + your custom stages. Read-only here: you change the stage, but you don't add new stages from the breadcrumb (use the Properties panel for that).
>
> **Step 1 — Pre-state**: open any note. Find the stage badge in the breadcrumb. Click it.
>
> **Expected**: dropdown opens showing the 6 baseline stages (with emoji) followed by any custom stages you added in §1C (e.g. `chrysalis 🏷️`).
>
> **Step 2 — Switch via breadcrumb**: pick `dormancy`. Dropdown closes.
>
> **Expected**: breadcrumb badge updates to `Dormancy 😴`. The `stage:` value in the note's frontmatter on disk is now `dormancy`. Properties panel reflects the same value.
>
> **Step 3 — Custom-only Universe**: in a Universe with custom stages, the breadcrumb's dropdown reflects them in the same chronological-add order as PropertyEditor.

1. Boss confirms the breadcrumb dropdown matches PropertyEditor's source list.
2. Stage change via breadcrumb propagates to Properties panel + frontmatter.
3. **M11 zero-diff** check.

### Commit message (skeleton)

```
MIG-014 §1D: NotePane breadcrumb dropdown reads combined stage list

Updates the breadcrumb's stage dropdown to source from
[...LIVING_LINK_BASELINE, ...$customStages]. Read-only — no
inline-add (PropertyEditor is the add surface). Stage change
flows through existing onpromote/onstagechange callback (no
$effect; NotePane Specs §4.8 preserved).
```

---

## §5 · Phase 1E — Settings → Notes → Manage Custom Stages panel

**Goal**: new Settings panel that lists user's custom stages and supports add / edit / rename / reorder / delete + emoji picker.

### Files touched

- `src/lib/components/SettingsModal.svelte` — new section under Settings → Notes (or the closest existing tab — verify during build).
- `src/lib/i18n/en.json` — new keys for the panel.
- `src/lib/i18n/ar.json` — same keys.

### Algorithm

New section template:

```svelte
<section class="settings-section">
    <h3>{$t('settings.notes.manageCustomStages')}</h3>
    <p class="section-intro">{$t('settings.notes.customStagesIntro')}</p>

    <h4>{$t('settings.notes.builtInStages')}</h4>
    <ul class="cs-built-in">
        {#each LIVING_LINK_BASELINE as bs}
            <li>{bs.emoji} {$t(`notePane.stage.${bs.name}`)} — <em>{bs.name}</em></li>
        {/each}
    </ul>

    <h4>{$t('settings.notes.yourCustomStages')}</h4>
    {#if $customStages.length === 0}
        <p class="cs-empty">{$t('settings.notes.noCustomStagesYet')}</p>
    {:else}
        <ul class="cs-custom" use:dragSortable={...}>
            {#each $customStages as cs (cs.name)}
                <li class="cs-row">
                    <button class="cs-emoji" onclick={() => openEmojiPicker(cs)}>{cs.emoji}</button>
                    <input class="cs-name" type="text" value={cs.name}
                        onchange={(e) => renameCustomStage(cs.name, (e.target as HTMLInputElement).value)} />
                    <button class="cs-delete" onclick={() => confirmDeleteCustomStage(cs.name)} title={$t('settings.notes.deleteCustomStageTooltip')}>×</button>
                </li>
            {/each}
        </ul>
    {/if}

    <button class="cs-add-btn" onclick={() => addCustomStage({ name: '', emoji: '🏷️' })}>
        + {$t('settings.notes.addCustomStageButton')}
    </button>
</section>
```

`renameCustomStage` calls `updateCustomStage(oldName, newStage)`. `confirmDeleteCustomStage` opens an existing `ConfirmDialog` with the localized warning that notes already using the value keep it but new selections won't include it. `openEmojiPicker` reuses the existing `EmojiIconPicker` component (`6dce15c` lineage).

### i18n keys to add (en + ar)

```json
"settings.notes.manageCustomStages": "Manage custom stages",
"settings.notes.customStagesIntro": "Constellation ships with six built-in stages. Add your own to suit your workflow — anything you type is accepted.",
"settings.notes.builtInStages": "Built-in stages (cannot be edited)",
"settings.notes.yourCustomStages": "Your custom stages",
"settings.notes.noCustomStagesYet": "No custom stages yet. Add one inline in any note's Properties panel, or use the button below.",
"settings.notes.addCustomStageButton": "Add stage",
"settings.notes.deleteCustomStageConfirm": "Delete the custom stage \"{name}\"? Notes already using it will keep the value, but new selections won't include it.",
"settings.notes.deleteCustomStageTooltip": "Delete this custom stage"
```

Arabic translations included in same commit.

### Verification

**Automated**:
1. `npm run check` passes.
2. `cargo build --lib` passes.
3. **M11 zero-diff** check.

**Boss-testable**:

> **What this is**: a new section in Settings called "Manage custom stages." Shows the 6 built-in (read-only) and lets you add / rename / change emoji / reorder / delete your custom stages.
>
> **Step 1 — Open Settings → Notes section**. Scroll to "Manage custom stages." See the 6 baseline stages with emoji.
>
> **Step 2 — Add a stage**: click "+ Add stage". A new row appears with a default 🏷️ emoji and an empty name field. Type `brewing` (or anything). Press Tab.
>
> **Expected**: row persists. Open any note → Properties → stage dropdown → `brewing 🏷️` appears in the list.
>
> **Step 3 — Change emoji**: click the 🏷️ on the `brewing` row. Emoji picker opens. Pick ☕.
>
> **Expected**: row shows `☕ brewing`. Open any note → stage dropdown → `brewing ☕`.
>
> **Step 4 — Rename**: click the `brewing` text, type `infusing`. Tab.
>
> **Expected**: row updates. Notes that previously had `stage: brewing` still have `stage: brewing` in their frontmatter (no auto-rewrite); but the dropdown now shows `infusing ☕` for new selections.
>
> **Step 5 — Delete**: click the × on `infusing`. Confirm dialog appears.
>
> **Expected**: confirms with the localized warning. On confirm, the row disappears. New notes' stage dropdown no longer offers `infusing`. Notes that had `stage: infusing` keep the value verbatim.
>
> **Step 6 — Reorder**: drag a row. Order persists across sessions.
>
> **If you see this instead**: deleting a stage silently mutates other notes → bug; do NOT continue.

1. Boss confirms add / rename / emoji / delete / reorder all work end-to-end.
2. Old notes' values are preserved verbatim across all operations.
3. Confirm dialog shows the localized warning.
4. **M11 zero-diff** check.

### Commit message (skeleton)

```
MIG-014 §1E: Settings → Notes → Manage Custom Stages panel

New panel listing baseline + custom stages. Add / rename /
change-emoji / reorder / delete with ConfirmDialog on destructive
ops. Reuses existing EmojiIconPicker for emoji selection.

i18n: en + ar — 8 new keys under settings.notes.*. 13 others
queued via PJ-014.

Boss test passed Stage 1.
```

---

## §6 · Phase 1F — Help + User Manual (en + ar)

**Goal**: document the new feature in `docs/User Manual.md` (en) and `docs/help.ar/User Manual.md` (ar). 13 other locales queued via PJ-014.

### Files touched

- `docs/User Manual.md` — new subsection under §7 "Index" or the closest existing Notes section.
- `docs/help.ar/User Manual.md` — same subsection, Arabic.
- `docs/help.uConstellation.World/Index/Index.md` — only if Index page references stage taxonomy (verify during build).
- (Possibly) `docs/help.uConstellation.World/NotePane/<NotePane help>.md` — if a NotePane help file exists, add the stage section.

### Content sketch

```markdown
### Stages — describing the maturity of a note

Each note can carry a `stage:` value in its frontmatter. Constellation ships with six **built-in stages** based on the Living Link lifecycle:

| Emoji | Stage | When to use |
|---|---|---|
| ✨ | Spark | Initial idea, just captured. Not yet connected. |
| 🌱 | Birth | Connection made; first link or first context applied. |
| 🌿 | Growth | Actively developing — adding evidence, sources, links. |
| 🌳 | Maturity | Load-bearing — others depend on it; well-supported. |
| 😴 | Dormancy | Not actively used; resting but not forgotten. |
| 📦 | Archival | Retired but preserved in history. |

**Custom stages** — if the six built-in stages don't fit your workflow, you can add your own:

- **In any note's Properties panel**: type a new value in the stage field and press Tab. It saves automatically with a default 🏷️ emoji.
- **In Settings → Notes → Manage custom stages**: add / rename / change emoji / reorder / delete.

Custom stages are **per-Universe** — different Universes can have different vocabularies. A research Universe might use `preprint / submitted / published`; a personal Universe might use `idea / brewing / done`.

Old stage values from earlier note systems (e.g. Zettelkasten's `fleeting / literature / permanent / synthesis`) are preserved verbatim on disk. They appear in the breadcrumb of any note that uses them. Add them to your custom stages via Settings if you want them in the dropdown.
```

### Verification

1. en + ar User Manuals updated.
2. Index help only updated if it currently references stages (verify; otherwise no edit).
3. NotePane help updated if present.

### Commit message (skeleton)

```
MIG-014 §1F: User Manual + help — Stages section (en + ar)

Documents the 6 baseline Living Link stages, the Properties-panel
inline-add, the Settings → Notes → Manage Custom Stages panel,
per-Universe scope, and old-value preservation.

13 other locales queued via PJ-014 backfill.
```

---

## §7 · Phase 1G — Audit (Migration Rule §4)

Three parallel agents on the cumulative diff `MIG-014 §1A..§1F`:

1. **Invariant checker**:
   - NotePane Specs §4 (no `$effect` echo, no value-prop sync, no reactive recalc, cached parsed FM).
   - CLAUDE.md Performance Rules 1–8 (especially Rule 3 — no IPC on keystroke hot path).
   - Law 2.6 (Universe-scoped storage).
   - Law 4.3 (display, not domain — Second Screen mounts same PropertyEditor).
   - M11 zero-diff (`git diff src-tauri/src/lexicon/` empty).
2. **Drift checker**:
   - Any code still importing the old Zettelkasten 4-list constant.
   - Any settings flag rendered orphan.
   - Any i18n key now unused (the retired Zettelkasten labels stay per Architect §5.2 — confirm intent).
   - Any `invoke()` referencing renamed/removed commands.
3. **Migration-path checker**:
   - Fresh Universe (no `custom_stages` in JSON).
   - Pre-MIG-014 Universe (no field at all).
   - Mid-edit interrupt during `add_custom_stage`.
   - Rollback (revert all six commits — does the app boot? Do old notes still load?).

Findings into `lab/reports/MIG-014-NOTE-STAGE-AUDIT.md`. P0 fixed before close. P1 either applied in close-out commit or deferred with documentation.

### Commit message (skeleton)

```
MIG-014 §1G: Audit + close-out

Three-agent audit per Migration Rule §4. Findings:
- Invariant: …
- Drift: …
- Migration-path: …

[P0 fixes / P1 disposition.]

PJ-007 → Done. Number retired in Pending Jobs §7 with closing
commit hash.
```

---

## §8 · Verification log (filled in as we go)

| Phase | Commit hash | Boss tested? | Result | Notes |
|---|---|---|---|---|
| 1A | _pending_ | n/a (silent) | _pending_ | |
| 1B | _pending_ | n/a (silent) | _pending_ | |
| 1C | _pending_ | yes | _pending_ | |
| 1D | _pending_ | yes | _pending_ | |
| 1E | _pending_ | yes | _pending_ | |
| 1F | _pending_ | n/a (docs) | _pending_ | |
| 1G | _pending_ | n/a | _pending_ | _audit findings: pending_ |

---

## §9 · Rollback plan

If anything in 1A–1F blocks Boss's day:
1. `git revert` the offending commit(s) — they're independent enough that a §1F revert leaves §1A–§1E usable, etc.
2. The `custom_stages` field in `universe.json` is harmless if the frontend doesn't read it (`#[serde(default)]` on the Rust side; the frontend just renders the baseline 6).
3. The old Zettelkasten 4-stage dropdown is reachable via `git revert` of §1C alone. If we revert that, the Settings panel from §1E becomes orphaned but doesn't crash anything.
4. Per CLAUDE.md, prefer revert + new-MIG over in-place engineering when the racing surface is unclear.

---

## §10 · Approval gate

This Plan needs explicit Boss approval before §1A lands. Once approved, per Plan-Approval-Equals-Build-Approval (CLAUDE.md), the cascade runs through 1A → 1B → 1C → 1D → 1E → 1F → 1G without per-step approval, stopping only at:

- **Boss-testable verification clauses** at the end of §1C, §1D, §1E.
- **Genuine architectural surprise** — if a phase reveals an unmapped invariant.
- **§1G completion** — final summary + PJ-007 → Done in Pending Jobs.

The Standing Order session-log discipline applies between phases — log each `§N{letter}` commit as it lands.
