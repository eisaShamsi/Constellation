# MIG-014 §2 Plan v3 — One Custom Term, One Control

**Supersedes**: `MIG-014-NOTE-STAGE-PLAN-v2.md` (multi-type matrix model).
**Companion architect doc**: `docs/Stages-Concept-Paper-v1.1.md`.
**Status**: Approved-with-amendments by Eisa (2026-05-06). Cascade starts at §2A.

---

## What changed from Plan v2

Plan v2 modeled multiple custom types as orthogonal matrix columns, each registered as its own `NoteType`. Eisa's amendments simplified that:
1. **Only ONE custom term per Universe.** Universe.json carries a single `custom_term: String` — empty or a word.
2. **One control in PropertyEditor.** A single combobox listing 6 baseline + (6 paired if term set) entries.
3. **Dash separator.** Compound stages are `Spark-Concept`, encoded on disk as `spark-concept`.
4. **Dropdown order** = lifecycle rank (Spark → Archival), then paired in same order.

The 7-phase rollout structure stays. Implementation details inside each phase are slimmer.

---

## Phase rollout

| Phase | Scope                                                            | Visible? | Boss test? |
| ----- | ---------------------------------------------------------------- | -------- | ---------- |
| §2A   | Rust schema — `custom_term: String` + 3 commands                  | No       | No         |
| §2B   | Frontend store — `customTerm` writable + 3 wrappers + helpers     | No       | No         |
| §2C   | PropertyEditor — single combobox (6 / 12) + inline-set            | **Yes**  | **Yes**    |
| §2D   | NotePane breadcrumb — chain walks within suffix                   | **Yes**  | **Yes**    |
| §2E   | Settings → Notes → Custom Note Term panel                          | **Yes**  | **Yes**    |
| §2F   | Help + User Manual (en + ar)                                       | Doc      | No         |
| §2G   | Three-agent audit                                                  | No       | No         |

---

## §2A — Schema rename

### Goal
`UniverseMeta.custom_stages: Vec<CustomStage>` → `UniverseMeta.custom_term: String`. Three IPC commands replace the previous five.

### Files touched
- `src-tauri/src/universe.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/boot_bundle.rs`

### Algorithm
```rust
pub struct UniverseMeta {
    // …
    /// MIG-014 §2 — at most one custom term per Universe; verbatim case.
    /// Empty string = none. Paired stage values on disk are encoded
    /// `lifecycle-customterm-lowercased`. The stored `custom_term`
    /// preserves the user's typed case for display.
    #[serde(default)]
    pub custom_term: String,
}

#[tauri::command]
pub fn read_custom_term(app: tauri::AppHandle) -> Result<String, String> { … }

#[tauri::command]
pub fn set_custom_term(app: tauri::AppHandle, term: String) -> Result<(), String> {
    let trimmed = term.trim().to_string();  // verbatim case
    if trimmed.contains('-') {
        return Err("Custom term cannot contain '-' (used as separator)".into());
    }
    if LIVING_LINK_BASELINE_NAMES.iter().any(|b| b.eq_ignore_ascii_case(&trimmed)) {
        return Err(format!("'{}' is a reserved lifecycle name", trimmed));
    }
    mutate_universe(&app, |meta| { meta.custom_term = trimmed; Ok(()) })
}

#[tauri::command]
pub fn remove_custom_term(app: tauri::AppHandle) -> Result<(), String> {
    mutate_universe(&app, |meta| { meta.custom_term = String::new(); Ok(()) })
}
```

The dropped fields:
- `CustomStage` struct → gone.
- `custom_stages: Vec<CustomStage>` → gone (clean break; no field migration).
- 5 stage commands → 3 term commands.

`LIVING_LINK_BASELINE_NAMES` constant stays.

### Verification
1. `cargo build --release --lib` clean (warnings ≤ baseline).
2. `git diff src-tauri/src/lexicon/` empty (M11 zero-diff).
3. Manual round-trip: `universe.json` survives a read-modify-write cycle; missing `custom_term` field reads as empty string.

### Commit message skeleton
```
MIG-014 §2A — schema: custom_stages → custom_term

Replaces the flat Vec<CustomStage> with a single String field
per the 2D matrix model (Stages Concept Paper v1.1, c59bdfb +
amendments). Three IPC commands: read / set / remove_custom_term.
Verbatim case preserved in universe.json; on-disk paired stage
values are dash-encoded with the term lowercased.

Clean break — no migration of the §1A custom_stages field.
```

---

## §2B — Frontend store + wrappers

### Goal
TS mirror of §2A. Replace `customStages` writable with `customTerm: writable<string>`. Three wrappers. Updated helpers.

### Files touched
- `src/lib/libraries/store.ts`
- `src/routes/+layout.svelte` (boot bundle field)

### Algorithm
```typescript
// Drop CustomStage. Drop customStages writable. Drop addCustomStage,
// updateCustomStage, removeCustomStage, reorderCustomStages, isKnownStage.

export const customTerm = writable<string>('');

export async function readCustomTerm(): Promise<string> { return invoke('read_custom_term'); }
export async function setCustomTerm(term: string): Promise<void> {
    await invoke('set_custom_term', { term });
    customTerm.set(await readCustomTerm());
}
export async function removeCustomTerm(): Promise<void> {
    await invoke('remove_custom_term');
    customTerm.set('');
}

/** Resolve a stage value's lifecycle prefix + suffix. Lifecycle and suffix
 *  may be empty if the value is malformed; callers should treat empty
 *  lifecycle as "render the value verbatim". */
export function splitStage(stage: string): { lifecycle: string; suffix: string } {
    const i = stage.indexOf('-');
    return i < 0 ? { lifecycle: stage, suffix: '' } : { lifecycle: stage.slice(0, i), suffix: stage.slice(i + 1) };
}

/** Build the display label. Lifecycle goes through i18n; suffix is rendered
 *  with the Universe's verbatim case when it matches; otherwise capitalised. */
export function stageLabel(stage: string, term: string, t: (k: string) => string): string {
    const { lifecycle, suffix } = splitStage(stage);
    const isBaseline = LIVING_LINK_BASELINE.some(b => b.name === lifecycle);
    const lifecycleLabel = isBaseline
        ? t(`notePane.stage.${lifecycle}`)
        : lifecycle.charAt(0).toUpperCase() + lifecycle.slice(1);
    if (!suffix) return lifecycleLabel;
    const suffixDisplay = suffix === term.toLowerCase()
        ? term
        : suffix.charAt(0).toUpperCase() + suffix.slice(1);
    return `${lifecycleLabel}-${suffixDisplay}`;
}

/** Lifecycle-only emoji (custom term contributes none). */
export function lookupStageEmoji(stage: string): string {
    const { lifecycle } = splitStage(stage);
    if (!lifecycle) return '';
    const baseline = LIVING_LINK_BASELINE.find(b => b.name === lifecycle);
    if (baseline) return baseline.emoji;
    if (LEGACY_ZETTELKASTEN_EMOJI[lifecycle]) return LEGACY_ZETTELKASTEN_EMOJI[lifecycle];
    return '';
}

/** Compute the next/previous stage for promote/demote. */
export function nextStage(stage: string): string | null {
    const { lifecycle, suffix } = splitStage(stage);
    const idx = LIVING_LINK_BASELINE.findIndex(b => b.name === lifecycle);
    if (idx < 0 || idx === LIVING_LINK_BASELINE.length - 1) return null;
    const next = LIVING_LINK_BASELINE[idx + 1].name;
    return suffix ? `${next}-${suffix}` : next;
}
export function prevStage(stage: string): string | null { /* symmetric */ … }
```

`+layout.svelte` boot-bundle integration: `bundle.custom_stages` field is replaced with `bundle.custom_term: string`. Setter on the writable.

### Verification
1. `npm run check` — only the pre-existing LinkLifecycle error (now displaced by line-count again — confirm same error, different line).
2. `cargo build --release --lib` clean.

### Commit message skeleton
```
MIG-014 §2B — store rename + simplified wrappers

customStages → customTerm: writable<string>. 5 wrappers → 3
(read/set/remove). Drops isKnownStage, addCustomStage,
updateCustomStage, reorderCustomStages.

Adds splitStage, stageLabel(stage, term, t), nextStage, prevStage.
lookupStageEmoji(stage) — single-arg lifecycle-only lookup.

Boot bundle: bundle.custom_stages → bundle.custom_term.
```

---

## §2C — PropertyEditor: single combobox

### Goal
The existing custom dropdown shell from §1C.5 is reused — just the data feeding it changes. Dropdown shows 6 (no term) or 12 (term set) entries plus the "+ Set custom term…" inline-set affordance when no term is set.

### Files touched
- `src/lib/components/PropertyEditor.svelte`
- `src/lib/i18n/en.json` + `ar.json` — `propertyEditor.setCustomTerm`, `propertyEditor.customTermPrompt`.

### Algorithm
```svelte
{#if prop.key.toLowerCase() === 'stage'}
    {@const opts = buildStageOptions($customTerm)}
    <div class="pe-stage-wrap">
        <span class="pe-stage-current-emoji">{lookupStageEmoji(prop.value)}</span>
        <input
            class="pe-val pe-stage-input"
            type="text"
            value={stageLabel(prop.value, $customTerm, $t)}
            placeholder={$t('propertyEditor.stagePlaceholder')}
            oninput={(e) => setStageFilter(idx, e.target.value)}
            onfocus={() => stageMenuOpen = idx}
            onkeydown={(e) => handleStageKeydown(e, idx)}
        />
        {#if stageMenuOpen === idx}
            <div class="pe-stage-dropdown">
                {#each opts.filter(matchesFilter) as opt}
                    <button class="pe-stage-option" onclick={() => commitStage(idx, opt.value)}>
                        <span class="pe-stage-emoji">{opt.emoji}</span>
                        <span class="pe-stage-label">{opt.label}</span>
                    </button>
                {/each}
                {#if !$customTerm}
                    <button class="pe-stage-option pe-stage-add"
                        onclick={() => beginInlineSetTerm(idx)}>
                        ＋ {$t('propertyEditor.setCustomTerm')}
                    </button>
                {/if}
            </div>
        {/if}
    </div>
{/if}
```

`buildStageOptions(term)` returns:
- Always: 6 baseline entries with `value = name`, `emoji`, `label = i18n("notePane.stage." + name)`.
- If `term`: 6 paired entries with `value = "{name}-{term.toLowerCase()}"`, same emoji, `label = "{baseline-label}-{term-verbatim}"`.

`commitStage(idx, value)` writes the property value (lowercase canonical with dash).

`beginInlineSetTerm(idx)` swaps the dropdown for an inline input. On submit, calls `setCustomTerm(input)`. On success, the dropdown re-renders with 12 entries — and the user can pick one.

The Properties input field renders `stageLabel(...)` (display label with the suffix capitalized) but commits the lowercase canonical to disk on selection. (The input is not directly typeable for free-form values — it's a display.)

### Boss-test (Stage 1)

> **What this is**: the stage row in Properties is a single combobox. It shows the 6 lifecycle stages always; when you've set a custom term, it shows 6 more — paired versions like `Spark-Concept`. Setting the custom term happens inline from this dropdown the first time.
>
> **Step 1 — No custom term yet (fresh universe)**: open a note, click the stage value.
> **Expected**: 6 entries (✨ Spark, 🌱 Birth, 🌿 Growth, 🌳 Maturity, 😴 Dormancy, 📦 Archival) plus a `＋ Set custom term…` action at the bottom.
>
> **Step 2 — Set the custom term**: click `＋ Set custom term…`. Type `Concept`. Press Enter.
> **Expected**: dropdown re-renders with **12 entries** in this order: Spark, Birth, Growth, Maturity, Dormancy, Archival, then Spark-Concept, Birth-Concept, Growth-Concept, Maturity-Concept, Dormancy-Concept, Archival-Concept. Emoji on the paired ones is the lifecycle's emoji (no extra emoji for Concept).
>
> **Step 3 — Pick a paired stage**: click `🌿 Growth-Concept`.
> **Expected**: input shows `🌿 Growth-Concept`. Frontmatter on disk reads `stage: growth-concept`.
>
> **Step 4 — Switch back to default**: open the dropdown, click `🌱 Birth`.
> **Expected**: frontmatter on disk reads `stage: birth`. The dropdown still shows all 12 entries (the custom term Concept stays in the Universe).

### Verification
1. `npm run check` clean.
2. `cargo build --release --lib` clean.
3. **M11 zero-diff**.
4. Boss-test passed.

### Commit message
```
MIG-014 §2C — PropertyEditor: single combobox

One combobox feeds from buildStageOptions(customTerm) — 6 entries
when term empty, 12 when set. "+ Set custom term…" action appears
inline when no term is set; clicking opens an input that calls
setCustomTerm on submit. Dropdown re-renders with 12 entries.

i18n: en + ar — propertyEditor.setCustomTerm, customTermPrompt.

Boss test passed Stage 1.
```

---

## §2D — NotePane breadcrumb: chain walks within suffix

### Goal
Promote / demote walks the lifecycle phase only, preserving the suffix. Chain length stays at 6.

### Files touched
- `src/lib/components/NotePane.svelte`

### Algorithm
```svelte
{#if currentStage}
    {@const np = nextStage(currentStage)}
    {@const pp = prevStage(currentStage)}
    {@const stageEmoji = lookupStageEmoji(currentStage)}
    {@const labelText = stageLabel(currentStage, $customTerm, $t)}
    <div class="e-bc-stage-wrap">
        {#if pp}
            <button class="e-bc-demote" onclick={() => commitPromote(pp)}>{isRTL ? '→' : '←'}</button>
        {/if}
        <span class="e-bc-stage-badge" title={labelText}>{stageEmoji} {labelText}</span>
        {#if np}
            <button class="e-bc-promote" onclick={() => commitPromote(np)}>{$t('notePane.promote')} {isRTL ? '←' : '→'}</button>
        {/if}
    </div>
{/if}
```

Promote / demote no longer iterate through customs; the chain is exactly the lifecycle. Suffix carried via the value's encoding.

### Boss-test (Stage 1)

> **Pre-state**: from §2C, you have a note at `stage: growth-concept`. Breadcrumb shows `🌿 Growth-Concept`.
>
> **Step 1 — Promote**: click promote.
> **Expected**: breadcrumb advances to `🌳 Maturity-Concept`. Frontmatter `stage: maturity-concept`. Properties combobox value updates.
>
> **Step 2 — Demote**: click demote.
> **Expected**: back to `🌿 Growth-Concept`.
>
> **Step 3 — Boundary at Archival**: promote until you reach `📦 Archival-Concept`. The promote arrow disappears.
>
> **Step 4 — Boundary at Spark**: from `📦 Archival-Concept`, demote until `✨ Spark-Concept`. The demote arrow disappears.
>
> **Step 5 — Default-track promote**: change the note's stage to `🌱 Birth` (no suffix). Click promote.
> **Expected**: advances to `🌿 Growth` (no suffix carried because there wasn't one).

### Verification
1. `npm run check` clean.
2. `cargo build --release --lib` clean.
3. **M11 zero-diff**.
4. Boss-test passed.

### Commit message
```
MIG-014 §2D — NotePane breadcrumb walks lifecycle only

Promote/demote arrows step through the 6 lifecycle phases,
carrying the suffix verbatim. Chain length = 6 regardless of
custom term presence. Properties + breadcrumb + file tree
update through the existing onpromote → handlePromote path.

Removes the §1D flat customs-in-chain; replaces with nextStage/
prevStage helpers in store.ts.

Boss test passed Stage 1.
```

---

## §2E — Settings → Custom Note Term

### Goal
Surface for changing or removing the Universe's custom term, with a count of affected notes.

### Files touched
- Settings panel (likely `src/routes/+layout.svelte` or wherever Settings → Notes lives — confirm during build).
- `src/lib/i18n/en.json` + `ar.json`.

### UX
```
┌──────────────────────────────────────────────────┐
│  Custom note term                                │
│                                                  │
│  Term:  [ Concept                       ]        │
│                                                  │
│  Notes currently using paired stages: 17         │
│                                                  │
│              [ Save ]   [ Remove ]               │
└──────────────────────────────────────────────────┘
```

- Save with new value: confirm dialog if term changes ("X notes have paired-stage values referencing the previous term — they keep on-disk values verbatim and render via fallback. Continue?"). On confirm, calls `setCustomTerm`.
- Remove: confirm dialog. Calls `removeCustomTerm`.

The "Notes currently using paired stages" count is computed by a new `count_paired_notes` IPC (or by the boot snapshot's stage map). If too expensive for §2E, simplify the dialog: just "Notes using paired stages will display via fallback. Continue?" without a count.

### Boss-test (Stage 1)

> **What this is**: Settings → Notes → Custom Note Term — the canonical surface for changing or removing the Universe's single custom term.
>
> **Step 1 — Open settings**: open Settings → Notes → Custom Note Term.
> **Expected**: shows current term (`Concept` after §2C), count of notes using paired stages.
>
> **Step 2 — Replace**: edit to `Idea`. Click Save.
> **Expected**: confirm dialog. On confirm, term changes. Existing notes with `stage: spark-concept` etc. keep their on-disk values; their breadcrumb labels render via fallback (capitalized suffix). Properties dropdown now offers 12 entries with `Idea` paired.
>
> **Step 3 — Remove**: click Remove. Confirm.
> **Expected**: term cleared. Properties dropdown shows 6 entries again. Existing paired-stage notes still show their on-disk values; user can manually re-pick.

### Verification
1. Boss-test passed.
2. `npm run check` + `cargo build --release --lib` clean.
3. **M11 zero-diff**.

### Commit message
```
MIG-014 §2E — Settings → Custom Note Term

Single text-input panel with Save + Remove. Confirms before
replace/remove (warns about existing paired-stage notes).
i18n: en + ar.

Boss test passed Stage 1.
```

---

## §2F — Help + User Manual

### Goal
Doc the new model. Stages help topic + User Manual section rewrite.

### Files touched
- New `docs/help.uConstellation.World/Stages.md` (en).
- New `docs/help.uConstellation.World.ar/Stages.md` (ar).
- `docs/User Manual.md` Stages section rewrite.
- `docs/User Manual.ar.md` ditto.

13 other locales queued via PJ-014.

### Verification
- Spot-check links and embedded screenshots.
- No build step.

---

## §2G — Three-agent audit

Three parallel agents:
1. **Invariants agent** — verifies: (a) `LIVING_LINK_BASELINE` length is exactly 6 in all consumers; (b) promote/demote chain length = 6 always; (c) `custom_term` is the only persistent custom-stage shape; (d) M11 zero-diff; (e) `lookupStageEmoji` is single-arg; (f) `stageLabel` is the only function producing display labels.
2. **Drift agent** — checks no UI surface (Sky View, Inspector360, Constellation Map, Search, Dashboard, Sight, Tasks View) still references `customStages`, `CustomStage`, the old 5 commands, the old emoji-on-NoteType assumption, or hardcoded Zettelkasten 4-stage emoji.
3. **Migration-path agent** — checks: (a) fresh Universe (no `custom_term` field) reads as empty cleanly; (b) Universe with old `custom_stages` field deserializes harmlessly (field ignored); (c) note with `stage: fleeting` (legacy Zettelkasten) renders correctly via `LEGACY_ZETTELKASTEN_EMOJI`; (d) note with `stage: spark-old_term` after Universe term change to `new_term` renders via fallback (capitalized suffix); (e) on-disk `stage: spark-` (trailing dash) does not crash (treated as `spark`, suffix empty).

P0/P1 fixed before close. P2/P3 logged as PJ-NNN.

---

## Closing the cascade

After §2G:
- `Constellation Pending Jobs v1.x.md` — PJ-007 status: **shipped (1-term, 1-control model)**.
- Orientation doc bumped — §17 cleaned of flat-list assumption.
- NotePane Specs — §3.5.1 added, covering the dash-encoded suffix.
- MoCh next-block file written.

---

**Cascade starts at §2A on Eisa's "go" — Plan v3 is approved-with-amendments per 2026-05-06 chat.**
