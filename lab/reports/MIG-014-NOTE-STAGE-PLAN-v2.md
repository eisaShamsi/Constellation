# MIG-014 §2 Plan v2 — 2D Stage Matrix

**Supersedes**: `lab/reports/MIG-014-NOTE-STAGE-PLAN.md` (flat-list model)
**Companion architect doc**: `docs/Stages-Concept-Paper-v1.0.md`
**Status**: Draft for Eisa's approval · 2026-05-06

---

## Open questions resolved (per Eisa, 2026-05-06)

| # | Question                       | Resolution                                                    |
| - | ------------------------------ | ------------------------------------------------------------- |
| 1 | Default-type label             | Just `Spark` when type is empty (no `Default` qualifier)        |
| 2 | Type input case                | Store verbatim — no auto-lowercase                              |
| 3 | Type dropdown order            | Like the ranks of the fixed stages — chronological-add order   |
| 4 | Search facets                  | Yes — `note_type:concept` is a filterable query token          |
| 5 | Old §1A–§1D commits in main    | Leave as iteration record — no history rewrite                  |

---

## Phase rollout

§2A → §2G. Each phase commits independently, has automated verification, and (where user-visible) a Boss-test gate.

| Phase | Scope                                                | User-visible? | Test gate? |
| ----- | ---------------------------------------------------- | ------------- | ---------- |
| §2A   | Rust schema — `NoteType { name }`, drop emoji         | No            | No         |
| §2B   | Frontend store — `noteTypes`, simplified wrappers     | No            | No         |
| §2C   | PropertyEditor — two controls (stage + type)          | **Yes**       | **Yes**    |
| §2D   | NotePane breadcrumb — chain = 6 only, label appends type | **Yes**       | **Yes**    |
| §2E   | Settings → Manage Note Types panel                    | **Yes**       | **Yes**    |
| §2F   | Help + User Manual (en + ar)                          | Doc only      | No         |
| §2G   | Three-agent audit                                     | No            | No         |

---

## §2A — Schema rename + drop emoji

### Goal
Rename `custom_stages: Vec<CustomStage>` → `note_types: Vec<NoteType>`. Drop the `emoji` field. Replace the 5 IPC commands accordingly. The legacy field `custom_stages` is **not** preserved — clean break, dev-only builds (Eisa hasn't persisted any to disk per §1C.5 retest).

### Files touched
- `src-tauri/src/universe.rs` — drop `CustomStage`; add `NoteType { name: String }`. Rename `UniverseMeta.custom_stages` → `note_types`. Rename + simplify the 5 commands.
- `src-tauri/src/lib.rs` — `invoke_handler` registrations.
- `src-tauri/src/boot_bundle.rs` — field rename + helper call rename.

### Algorithm
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteType {
    pub name: String,
}

pub struct UniverseMeta {
    // …
    #[serde(default)]
    pub note_types: Vec<NoteType>,
}

#[tauri::command]
pub fn read_note_types(app: tauri::AppHandle) -> Result<Vec<NoteType>, String> { … }

#[tauri::command]
pub fn add_note_type(app: tauri::AppHandle, note_type: NoteType) -> Result<(), String> {
    let name = note_type.name.trim().to_string();  // verbatim case (per Q2)
    if name.is_empty() { return Err("Name cannot be empty".into()); }
    if LIVING_LINK_BASELINE_NAMES.iter().any(|b| b.eq_ignore_ascii_case(&name)) {
        return Err(format!("'{}' is a reserved lifecycle name", name));
    }
    mutate_note_types(&app, |types| {
        if types.iter().any(|t| t.name.eq_ignore_ascii_case(&name)) {
            return Err(format!("Type '{}' already exists", name));
        }
        types.push(NoteType { name });
        Ok(())
    })
}

#[tauri::command]
pub fn update_note_type(app: tauri::AppHandle, old_name: String, new_name: String) -> Result<(), String> { … }

#[tauri::command]
pub fn remove_note_type(app: tauri::AppHandle, name: String) -> Result<(), String> { … }

#[tauri::command]
pub fn reorder_note_types(app: tauri::AppHandle, names_in_order: Vec<String>) -> Result<(), String> { … }
```

Comparison is case-insensitive ASCII (`eq_ignore_ascii_case`) so the user can't add `Concept` and `concept` both. Storage is verbatim per Q2.

### Verification
1. `cargo build --release --lib` clean (no new warnings beyond baseline).
2. Round-trip test (manual): `universe.json` survives a read-modify-write cycle.
3. **M11 zero-diff**: `git diff src-tauri/src/lexicon/` empty.

### Commit message
```
MIG-014 §2A — schema rename: custom_stages → note_types

Per Stages Concept Paper v1.0 (commit c59bdfb), customs are not
flat additions to the lifecycle — they're an orthogonal type
dimension. CustomStage { name, emoji } becomes NoteType { name };
emoji is dropped (types follow the lifecycle phase's emoji).
UniverseMeta.custom_stages → note_types. Five Tauri commands
renamed: read/add/update/remove/reorder_note_types.

Clean break — no legacy custom_stages field preserved. Dev-only
builds; no users have persisted custom stages.

Supersedes the §1A schema (commit c3b9454) at the model level.
```

---

## §2B — Frontend store + IPC wrappers

### Goal
Mirror §2A on the TS side. Rename `customStages` writable → `noteTypes`. Drop `CustomStage.emoji`. Simplify `lookupStageEmoji` — emoji depends only on lifecycle now. Add `stageLabel(stage, noteType)` helper.

### Files touched
- `src/lib/libraries/store.ts`
- `src/routes/+layout.svelte` — bundle field rename (boot bundle returns `note_types`).

### Algorithm
```typescript
export interface NoteType { name: string; }

// LIVING_LINK_BASELINE stays — drives the 6-step chain.

export const noteTypes = writable<NoteType[]>([]);

export async function readNoteTypes(): Promise<NoteType[]> { … }
export async function addNoteType(noteType: NoteType): Promise<void> { … }
export async function updateNoteType(oldName: string, newName: string): Promise<void> { … }
export async function removeNoteType(name: string): Promise<void> { … }
export async function reorderNoteTypes(namesInOrder: string[]): Promise<void> { … }

/** Display label: `<lifecycle-label>` or `<lifecycle-label> <Type>`.
 *  Type qualifier is stored verbatim and rendered verbatim (per Q2). */
export function stageLabel(stage: string, noteType: string | undefined, t: (key: string) => string): string {
    const isBaseline = LIVING_LINK_BASELINE.some(b => b.name === stage);
    const baselineLabel = isBaseline
        ? t(`notePane.stage.${stage}`)
        : stage.charAt(0).toUpperCase() + stage.slice(1);  // legacy Zettelkasten or unknown
    if (!noteType || !noteType.trim()) return baselineLabel;
    return `${baselineLabel} ${noteType.trim()}`;
}

/** Emoji depends only on lifecycle. Legacy Zettelkasten fallback retained. */
export function lookupStageEmoji(stage: string): string {
    const v = stage.trim().toLowerCase();
    if (!v) return '';
    const baseline = LIVING_LINK_BASELINE.find(b => b.name === v);
    if (baseline) return baseline.emoji;
    if (LEGACY_ZETTELKASTEN_EMOJI[v]) return LEGACY_ZETTELKASTEN_EMOJI[v];
    return '';  // unknown stage — no emoji
}
```

The `customs` parameter of the old `lookupStageEmoji` is gone. `isKnownStage` is also gone (no inline-add into the lifecycle list — adding goes via `addNoteType` to the type dimension only).

### Verification
1. `npm run check` — only the pre-existing LinkLifecycle error remains.
2. `cargo build --release --lib` clean.

### Commit message
```
MIG-014 §2B — store rename + simplified wrappers

customStages → noteTypes. CustomStage type dropped; NoteType { name }
takes its place. lookupStageEmoji simplifies to a lifecycle-only
lookup (no customs param). New stageLabel helper produces the 2D
combined label. Removes isKnownStage — no inline-add into lifecycle.

Boot bundle: bundle.custom_stages → bundle.note_types.
```

---

## §2C — PropertyEditor: two controls (stage + type)

### Goal
Replace the current single combobox with **two controls** in the stage row:
1. **Stage** — closed dropdown of the 6 lifecycle baselines. Mandatory.
2. **Type** — closed dropdown of "—" (Default) + the Universe's `noteTypes` + an "+ Add type…" inline action.

The on-disk frontmatter becomes two fields: `stage:` (one of 6) and `note_type:` (optional, verbatim).

### Files touched
- `src/lib/components/PropertyEditor.svelte` — replace stage row.
- `src/lib/libraries/store.ts` — frontmatter parse/build needs to recognise `note_type` alongside `stage` when the row is the canonical stage property.
- `src/lib/i18n/en.json` + `ar.json` — `propertyEditor.typeLabel` ("Type"), `propertyEditor.typeDefault` ("—"), `propertyEditor.addTypePrompt` ("New type name…").

### Algorithm
- The Properties panel has **one stage row** where the value side renders both controls side-by-side:
  ```
  ⚏ stage  [ ✨ Spark    ▾ ]   [ Concept   ▾ ]
  ```
- The lifecycle dropdown lists exactly the 6 baselines, each rendered as `<emoji> <Capitalized-i18n-label>`. Closed list — no inline-add.
- The type dropdown lists `—` (renders as the literal em-dash to mean Default) followed by the `noteTypes` in chronological-add order, plus an `+ Add type…` action at the bottom.
- Selecting a baseline writes `stage: <name>` (lowercase canonical).
- Selecting a type writes `note_type: <verbatim>`. Selecting `—` removes the `note_type:` field from frontmatter.
- The "+ Add type…" entry replaces the dropdown with an inline text input + "Add" + "Cancel". Submit calls `addNoteType({ name: trimmed })`. On success, the new type becomes the selection.
- Both changes flow through the existing PropertyEditor save path (debounce + writeNote).

### Boss-test (Stage 1 of §2C tutorial)

> **What this is**: the `stage:` field in the Properties panel of a note now has **two controls**, not one. The first picks **the lifecycle phase** (Spark / Birth / Growth / Maturity / Dormancy / Archival). The second picks **the type** — "—" means no qualifier (the default track); any other value is one of your custom types. Custom types pair with every lifecycle phase: if you've added "Concept", every phase has a "Concept" variant — `Spark Concept`, `Birth Concept`, etc.
>
> **Step 1 — Pre-state**: open a note with a `stage:` row in Properties.
>
> **Expected**: you see two side-by-side dropdowns. Left one shows current lifecycle (e.g. `✨ Spark`), right one shows `—` (default).
>
> **Step 2 — Pick a baseline**: open the left dropdown, pick `🌿 Growth`. Press Tab.
> **Expected**: frontmatter now reads `stage: growth`. Breadcrumb badge shows `🌿 Growth`.
>
> **Step 3 — Add a custom type**: open the right dropdown. Click `+ Add type…`. Type `Concept` (any case — stored verbatim). Press Enter.
> **Expected**: the type dropdown now shows `Concept` selected. Frontmatter has `stage: growth` AND `note_type: Concept` (case preserved). Breadcrumb badge shows `🌿 Growth Concept`.
>
> **Step 4 — Switch back to default**: open the right dropdown. Pick `—`.
> **Expected**: frontmatter loses `note_type:`. Breadcrumb badge shows `🌿 Growth`. The Concept type stays in the Universe (still in the dropdown for next time).

If any of these miss, tell me and I'll trace.

### Verification
1. `npm run check` — same pre-existing baseline.
2. `cargo build --release --lib` clean.
3. **M11 zero-diff** check.
4. Boss-test passed.

### Commit message
```
MIG-014 §2C — PropertyEditor: two controls (stage + type)

Replaces the single combobox with two side-by-side dropdowns:
- Stage  — closed list of 6 lifecycle baselines.
- Type   — Default (—) + Universe's note_types + "+ Add type…" inline action.

Frontmatter: `stage:` (lowercase canonical, one of 6) +
optional `note_type:` (verbatim). Selecting "—" removes note_type.

i18n: en + ar add propertyEditor.typeLabel, typeDefault,
addTypePrompt. 13 others queued via PJ-014.

Boss test passed Stage 1.
```

---

## §2D — NotePane breadcrumb: chain = 6 only

### Goal
Revert the breadcrumb's stageOrder to **lifecycle baselines only**. Promote/demote walks the 6 phases regardless of `note_type`. Breadcrumb badge shows `<emoji> <stageLabel>` where `stageLabel` includes the type qualifier when present.

### Files touched
- `src/lib/components/NotePane.svelte` — revise stage block.

### Algorithm
```svelte
{#if currentStage}
    {@const stageOrder = LIVING_LINK_BASELINE.map(s => s.name)}
    {@const idx = stageOrder.indexOf(currentStage)}
    {@const stageEmoji = lookupStageEmoji(currentStage)}
    {@const labelText = stageLabel(currentStage, currentNoteType, $t)}
    …
{/if}
```

`currentNoteType` is a new `$derived` that reads `note_type` from the parsed properties array. Promote/demote update `currentStage` only (not `currentNoteType`). The type stays constant across lifecycle moves — the Properties panel is the surface that changes type.

### Boss-test

> **What this is**: the breadcrumb's stage badge now shows `<emoji> <Lifecycle> <Type>` — e.g. `🌿 Growth Concept`. The promote and demote arrows still walk the 6 lifecycle stages **only**. Type stays constant across promote/demote — change type via Properties.
>
> **Step 1 — Promote inside a type**: on the note from §2C with `stage: growth`, `note_type: Concept`, click promote.
> **Expected**: breadcrumb advances to `🌳 Maturity Concept`. Frontmatter `stage:` is now `maturity`. `note_type:` is still `Concept`.
>
> **Step 2 — Demote**: click demote.
> **Expected**: back to `🌿 Growth Concept`.
>
> **Step 3 — Promote at boundary**: promote from Growth → Maturity → Dormancy → Archival. At Archival, the promote arrow disappears.
>
> **Step 4 — Demote at boundary**: from Spark, demote arrow is hidden.
>
> **Step 5 — Cross-type promote**: change type to Default (—) via Properties; promote/demote still works exactly the same on the 6 phases.

### Verification
1. `npm run check` clean.
2. `cargo build --release --lib` clean.
3. **M11 zero-diff** check.
4. Boss-test passed.

---

## §2E — Settings → Notes → Manage Note Types

### Goal
A dedicated panel to add, rename, remove, and reorder note types. No emoji picker.

### Files touched
- New panel within Settings (likely `src/routes/+layout.svelte` Settings drawer; need to find the existing Notes section).
- `src/lib/i18n/en.json` + `ar.json` — `settings.notes.manageNoteTypes` block.

### UX
- List view of types in chronological-add order.
- Each row: type name + Rename + Remove buttons.
- Drag handle for reorder (or up/down arrows).
- Footer: `+ Add type` text input.
- Remove confirmation: "X notes currently use this type. Removing it will fall back those notes to Default. Continue?" (count comes from a count IPC; if not feasible in §2E, defer the count to §2G audit and just say "All notes using this type will fall back to Default.")

### Boss-test

> **What this is**: Settings → Notes → Manage Note Types — the surface for adding, renaming, removing, and reordering custom note types Universe-wide.
>
> **Step 1 — Add via Settings**: open Settings → Notes → Manage Note Types. Click `+ Add type`. Type `Idea`. Save.
> **Expected**: `Idea` appears in the list. In any open note's PropertyEditor type dropdown, `Idea` now appears alongside `Concept` (in chronological-add order).
>
> **Step 2 — Rename**: rename `Concept` to `Hypothesis`. Save.
> **Expected**: every note that had `note_type: Concept` now reflects `note_type: Hypothesis` in its breadcrumb. Frontmatter on disk is updated.
>
> **Step 3 — Remove**: remove `Idea`. Confirm.
> **Expected**: `Idea` gone from the dropdown. Any note using `note_type: Idea` falls back to `—` (Default). Frontmatter `note_type:` field removed.
>
> **Step 4 — Reorder**: drag `Hypothesis` above the others.
> **Expected**: PropertyEditor type dropdown now lists `Hypothesis` first.

### Verification
1. Boss-test passed.
2. `npm run check` + `cargo build --release --lib` clean.
3. **M11 zero-diff** check.

---

## §2F — Help + User Manual (en + ar)

### Goal
Help topic + User Manual section explaining the 2D matrix model. Update what changed since the flat-list draft.

### Files touched
- `docs/help.uConstellation.World/Stages.md` (new) — long-form help, en.
- `docs/help.uConstellation.World.ar/Stages.md` (new) — ar.
- `docs/User Manual.md` — Stages section rewrite to reflect the 2D matrix.
- `docs/User Manual.ar.md` — same in Arabic.

13 other locales queued via PJ-014.

---

## §2G — Three-agent audit

Three parallel agents:
1. **Invariants agent** — verifies: (a) lifecycle chain length is exactly 6 in every consumer; (b) `lookupStageEmoji` no longer takes a customs param anywhere; (c) `note_types` is the only persistent custom-stage shape; (d) M11 zero-diff.
2. **Drift agent** — checks that no other UI surface (Sky View, Inspector360, Constellation Map, search, dashboard) still references the flat-list shape or expects an emoji on a NoteType.
3. **Migration-path agent** — checks: (a) first-boot on a fresh Universe (no `note_types` field) loads cleanly; (b) a `universe.json` with the *old* `custom_stages` field is harmless (field ignored, no parse error); (c) a note with `stage: fleeting` (legacy Zettelkasten) renders correctly via `LEGACY_ZETTELKASTEN_EMOJI`; (d) a note with `note_type:` referencing a removed type falls back to Default cleanly.

Each agent reports a punch list. P0/P1 fixed before close; P2/P3 logged as PJ-NNN.

---

## Closing the cascade

After §2G:
- `Constellation Pending Jobs v1.x.md` — PJ-007 status: **shipped (2D matrix model)**. Note that §1A–§1D were the iteration record; §2A–§2G are what actually shipped.
- Orientation doc bumped — §17 ("what Claude has NOT read in detail") cleaned of the flat-list assumption.
- NotePane Specs — §3.5 already corrected re: dropdown vs arrows; add a §3.5.1 covering the Type dimension.

---

**Awaiting Eisa's "Plan approved" before starting §2A.**
