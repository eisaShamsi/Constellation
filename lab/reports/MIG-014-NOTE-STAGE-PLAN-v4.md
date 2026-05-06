# MIG-014 §2 Plan v4 — Per-Note Custom Term, 6-Entry Dropdown

**Supersedes**: drafts v2 (multi-type matrix) and v3 (Universe-wide single term).
**Companion architect doc**: `docs/Stages-Concept-Paper-v1.2.md`.
**Status**: Pending Eisa's confirmation. Cascade starts at §2A on "go."

---

## What changed from v3

Two further amendments:
1. **Per-note scope** — no Universe-wide custom-term setting. The custom term lives only as the dash suffix in each note's frontmatter `stage:`. Drops the §2A schema change and the §2E Settings panel.
2. **Dropdown is always 6 entries** — empty input → 6 fixed; custom term in input → 6 paired (the fixed are hidden in this mode).

Result: simpler than v3. Universe schema returns to pre-MIG-014 shape. The plan is now 5 phases instead of 7.

---

## Phase rollout

| Phase | Scope                                                                | Visible? | Boss test? |
| ----- | -------------------------------------------------------------------- | -------- | ---------- |
| §2A   | Rust schema cleanup — drop `CustomStage` struct + `custom_stages` field + 5 IPC commands | No       | No         |
| §2B   | Frontend store cleanup — drop `customStages` writable + 5 wrappers + `isKnownStage`. New helpers: `splitStage`, `stageLabel`, `nextStage`, `prevStage`. Single-arg `lookupStageEmoji(stage)`. | No       | No         |
| §2C   | PropertyEditor combobox — 6-entry dropdown, mode-flips (A/B) on input value. | **Yes**  | **Yes**    |
| §2D   | NotePane breadcrumb — chain walks within suffix via `nextStage`/`prevStage`. | **Yes**  | **Yes**    |
| §2E   | Help + User Manual (en + ar)                                         | Doc      | No         |
| §2F   | Three-agent audit                                                    | No       | No         |

The previous v3 §2E "Settings → Custom Note Term" phase is gone — per-note scope means there's nothing to manage Universe-wide.

---

## §2A — Schema cleanup

### Goal
Restore `UniverseMeta` to its pre-§1A shape. Drop `CustomStage` struct, `custom_stages: Vec<CustomStage>` field, the 5 IPC commands, the `LIVING_LINK_BASELINE_NAMES` constant, and the `mutate_custom_stages` helper. The boot bundle's `custom_stages` field is removed.

### Files touched
- `src-tauri/src/universe.rs`
- `src-tauri/src/lib.rs` — remove the 5 `invoke_handler` registrations.
- `src-tauri/src/boot_bundle.rs` — remove the `custom_stages` field.

### Algorithm
Pure deletion — no replacement code on the Rust side. The 3 `UniverseMeta` instantiation sites (lines 522, 996, 1372 from §1A) lose the `custom_stages: vec![]` initialiser.

### Verification
1. `cargo build --release --lib` clean (warnings ≤ baseline).
2. **M11 zero-diff**: `git diff src-tauri/src/lexicon/` empty.
3. Round-trip: `universe.json` survives a read-modify-write cycle. Pre-§1A `universe.json` files (no `custom_stages` field) read cleanly. §1A-era `universe.json` files (with `custom_stages: []`) **also** read cleanly because Serde ignores unknown fields by default — but if Serde is configured strict somewhere, the field should be tolerated explicitly. Check the deserializer config.

### Commit message skeleton
```
MIG-014 §2A — drop custom_stages schema (per-note scope)

Per Stages Concept Paper v1.2, the custom term is per-note (lives
as the dash suffix in each note's stage value), not Universe-wide.
The §1A schema was based on the wrong model — drop the entire
CustomStage / custom_stages / 5-IPC apparatus.

UniverseMeta restored to pre-MIG-014 shape. boot_bundle.custom_stages
removed. lib.rs unregistered the 5 commands.

Clean break — no migration of stale custom_stages data.
```

---

## §2B — Frontend store cleanup + new helpers

### Goal
Drop the §1B store additions. Add the new helpers per Concept Paper §6.3 / §6.4. Single-arg `lookupStageEmoji`. Boot bundle integration cleaned.

### Files touched
- `src/lib/libraries/store.ts`
- `src/routes/+layout.svelte` — boot bundle field removal.
- Any consumer that imported the dropped symbols.

### Algorithm
Drop:
- `CustomStage` interface
- `customStages` writable
- `addCustomStage`, `updateCustomStage`, `removeCustomStage`, `reorderCustomStages` wrappers
- `isKnownStage` helper
- `lookupStageEmoji(name, customs)` two-arg signature

Keep:
- `LIVING_LINK_BASELINE` constant (drives the 6-step lifecycle)
- `LEGACY_ZETTELKASTEN_EMOJI` (legacy display fallback)

Add:
```typescript
export function splitStage(stage: string): { lifecycle: string; suffix: string } {
    const i = stage.indexOf('-');
    return i < 0 ? { lifecycle: stage, suffix: '' } : { lifecycle: stage.slice(0, i), suffix: stage.slice(i + 1) };
}

export function stageLabel(stage: string, t: (k: string) => string): string {
    const { lifecycle, suffix } = splitStage(stage);
    const isBaseline = LIVING_LINK_BASELINE.some(b => b.name === lifecycle);
    const lifecycleLabel = isBaseline
        ? t(`notePane.stage.${lifecycle}`)
        : lifecycle.charAt(0).toUpperCase() + lifecycle.slice(1);
    if (!suffix) return lifecycleLabel;
    const suffixDisplay = suffix.charAt(0).toUpperCase() + suffix.slice(1);
    return `${lifecycleLabel}-${suffixDisplay}`;
}

export function lookupStageEmoji(stage: string): string {
    const { lifecycle } = splitStage(stage);
    if (!lifecycle) return '';
    const baseline = LIVING_LINK_BASELINE.find(b => b.name === lifecycle);
    if (baseline) return baseline.emoji;
    if (LEGACY_ZETTELKASTEN_EMOJI[lifecycle]) return LEGACY_ZETTELKASTEN_EMOJI[lifecycle];
    return '';
}

export function nextStage(stage: string): string | null {
    const { lifecycle, suffix } = splitStage(stage);
    const idx = LIVING_LINK_BASELINE.findIndex(b => b.name === lifecycle);
    if (idx < 0 || idx === LIVING_LINK_BASELINE.length - 1) return null;
    const next = LIVING_LINK_BASELINE[idx + 1].name;
    return suffix ? `${next}-${suffix}` : next;
}

export function prevStage(stage: string): string | null {
    const { lifecycle, suffix } = splitStage(stage);
    const idx = LIVING_LINK_BASELINE.findIndex(b => b.name === lifecycle);
    if (idx <= 0) return null;
    const prev = LIVING_LINK_BASELINE[idx - 1].name;
    return suffix ? `${prev}-${suffix}` : prev;
}
```

`+layout.svelte` boot bundle: drop the `custom_stages` field handler (set the writable + the BootBundle TS type).

### Verification
1. `npm run check` — only the pre-existing LinkLifecycle error.
2. `cargo build --release --lib` clean.

### Commit message skeleton
```
MIG-014 §2B — store cleanup + new pure helpers

Drops customStages writable, 5 wrappers, isKnownStage, two-arg
lookupStageEmoji. Adds splitStage, stageLabel(stage, t), nextStage,
prevStage. New lookupStageEmoji is single-arg (lifecycle prefix only).

All helpers are pure — same input → same output. No Universe state
threaded through; per-note scope per Concept Paper v1.2.

Boot bundle integration: bundle.custom_stages field removed.
```

---

## §2C — PropertyEditor combobox: 6-entry dropdown with mode flips

### Goal
The §1C.5 inline dropdown shell stays. The DATA feeding it changes:
- Mode A (default): 6 fixed entries.
- Mode B (paired): 6 paired entries with the user's typed suffix.
- Mode toggle: derived from the current input value.

### Files touched
- `src/lib/components/PropertyEditor.svelte`
- `src/lib/i18n/en.json` + `ar.json` — strings if any new ones needed (placeholder text already exists from §1C).

### Algorithm
```svelte
{#if prop.key.toLowerCase() === 'stage'}
    {@const inputVal = prop.value || ''}
    {@const { lifecycle, suffix } = splitStage(inputVal)}
    {@const isFixedMatch = LIVING_LINK_BASELINE.some(b => b.name === inputVal.toLowerCase())}
    {@const mode = (!inputVal || (isFixedMatch && !suffix)) ? 'A' : 'B'}
    {@const term = mode === 'B' ? (suffix || inputVal) : ''}
    {@const opts = mode === 'A'
        ? LIVING_LINK_BASELINE.map(b => ({ value: b.name, emoji: b.emoji, label: $t(`notePane.stage.${b.name}`) }))
        : LIVING_LINK_BASELINE.map(b => ({ value: `${b.name}-${term.toLowerCase()}`, emoji: b.emoji, label: `${$t(`notePane.stage.${b.name}`)}-${capitalize(term)}` }))}

    <div class="pe-stage-wrap">
        <span class="pe-stage-current-emoji">{lookupStageEmoji(inputVal)}</span>
        <input
            class="pe-val pe-stage-input"
            type="text"
            value={inputVal}
            placeholder={$t('propertyEditor.stagePlaceholder')}
            oninput={(e) => updateValue(idx, (e.target as HTMLInputElement).value)}
            onfocus={() => stageMenuOpen = idx}
            onkeydown={(e) => handleStageKeydown(e, idx)}
        />
        {#if stageMenuOpen === idx}
            <div class="pe-stage-dropdown">
                {#each opts as opt}
                    <button class="pe-stage-option" onclick={() => commitStage(idx, opt.value)}>
                        <span class="pe-stage-emoji">{opt.emoji}</span>
                        <span class="pe-stage-label">{opt.label}</span>
                    </button>
                {/each}
            </div>
        {/if}
    </div>
{/if}
```

`commitStage(idx, value)` writes the lowercase canonical to the property. No `addCustomStage` IPC call — there's no Universe-side custom term anymore.

`handleStageKeydown` simplified — no `stageUserNavigated` flag is needed because there's no inline-add ambiguity (typing always *transforms the dropdown*; pressing Enter commits the highlighted dropdown item if one is selected, otherwise commits the input value verbatim if it parses to a valid baseline or paired form, otherwise no-op + warn).

### Boss-test (Stage 1)

> **What this is**: the Properties stage row is a single combobox. Empty stage shows 6 fixed entries (Spark, Birth, Growth, Maturity, Dormancy, Archival). Type a custom word (e.g. `concept`) and the dropdown swaps to 6 paired entries (Spark-Concept, Birth-Concept, …, Archival-Concept) — fixed ones hide. Each note carries its own suffix; nothing is Universe-wide.
>
> **Step 1 — Mode A**: open a note with empty stage. Click the stage value.
> **Expected**: dropdown shows 6 fixed entries with emoji prefix.
>
> **Step 2 — Pick a fixed**: click `🌿 Growth`.
> **Expected**: input shows `growth`. Frontmatter `stage: growth`.
>
> **Step 3 — Type a custom term**: click the input again. Clear it (or type to replace). Type `concept`.
> **Expected**: dropdown swaps to 6 paired entries — `Spark-Concept` through `Archival-Concept`. The 6 fixed are hidden.
>
> **Step 4 — Pick a paired**: click `🌱 Birth-Concept`.
> **Expected**: input shows `birth-concept`. Frontmatter `stage: birth-concept`.
>
> **Step 5 — Switch back to fixed**: click input. Edit value to just `birth`.
> **Expected**: dropdown swaps to 6 fixed with `Birth` highlighted. Pick to confirm; or just leave the input as `birth` and click outside to commit.
>
> **Step 6 — Per-note scope**: open a different note in the same Universe. Click its stage value.
> **Expected**: dropdown shows 6 fixed (Mode A) — the custom term `concept` from the previous note does NOT appear here. Each note is independent.

### Verification
1. `npm run check` clean.
2. `cargo build --release --lib` clean.
3. **M11 zero-diff**.
4. Boss-test passed.

### Commit message
```
MIG-014 §2C — PropertyEditor combobox: 6 entries, two modes

Single combobox feeds buildStageOptions(inputValue):
- Mode A (input empty or matches a fixed name) → 6 fixed entries.
- Mode B (input has dash suffix or is a custom word) → 6 paired
  entries with the suffix; fixed entries hidden.

Mode toggle is automatic — the input value drives it. No Universe
state involved; per-note scope per Concept Paper v1.2.

addCustomStage IPC removed (was wrong-model). handleStageKeydown
simplified — no userNavigated flag.

Boss test passed Stage 1.
```

---

## §2D — NotePane breadcrumb: chain walks within suffix

### Goal
Promote / demote walks the lifecycle phase, carrying the suffix verbatim. Chain length stays at 6 in both modes.

### Files touched
- `src/lib/components/NotePane.svelte`

### Algorithm
```svelte
{#if currentStage}
    {@const np = nextStage(currentStage)}
    {@const pp = prevStage(currentStage)}
    {@const stageEmoji = lookupStageEmoji(currentStage)}
    {@const labelText = stageLabel(currentStage, $t)}
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

The §1D additions that pulled in `customStages` go away.

### Boss-test (Stage 1)

> **Pre-state**: from §2C, you have a note at `stage: birth-concept`. Breadcrumb shows `🌱 Birth-Concept`.
>
> **Step 1 — Promote inside suffix**: click promote. Expected: `🌿 Growth-Concept`. Frontmatter `stage: growth-concept`.
> **Step 2 — Demote**: back to `🌱 Birth-Concept`.
> **Step 3 — Boundary at Archival**: promote to `📦 Archival-Concept`. Promote arrow disappears.
> **Step 4 — Boundary at Spark**: from `📦 Archival-Concept`, demote to `✨ Spark-Concept`. Demote arrow disappears.
> **Step 5 — Default-track**: change stage to `🌱 Birth` (no suffix). Promote → `🌿 Growth`. No suffix carried.
> **Step 6 — Cross-note independence**: open a different note. Its breadcrumb arrows behave entirely based on its own stage value.

### Verification
1. `npm run check` clean.
2. `cargo build --release --lib` clean.
3. **M11 zero-diff**.
4. Boss-test passed.

### Commit message
```
MIG-014 §2D — NotePane breadcrumb: lifecycle-only chain

Promote / demote use nextStage / prevStage from store.ts. Suffix
carried verbatim across the chain. Removes the §1D customs-in-chain
that pulled in the dropped customStages writable.

Boss test passed Stage 1.
```

---

## §2E — Help + User Manual

### Goal
Document the 6-fixed + per-note custom-term model.

### Files touched
- New `docs/help.uConstellation.World/Stages.md` (en) — full topic.
- New `docs/help.uConstellation.World.ar/Stages.md` (ar).
- `docs/User Manual.md` Stages section rewrite.
- `docs/User Manual.ar.md` Stages section rewrite.

13 other locales queued via PJ-014.

---

## §2F — Three-agent audit

Three parallel agents:
1. **Invariants agent** — verifies: (a) `LIVING_LINK_BASELINE.length === 6` checked everywhere; (b) chain length always 6 via `nextStage` / `prevStage`; (c) no remaining references to `customStages`, `CustomStage`, `addCustomStage`, etc.; (d) M11 zero-diff; (e) `stageLabel` / `lookupStageEmoji` are pure and signature-clean.
2. **Drift agent** — checks no UI surface (Sky View, Inspector360, Constellation Map, Search, Dashboard, Sight, Tasks, FocusPane, secondary screen) still references the dropped symbols or assumes a Universe-wide custom-term setting.
3. **Migration-path agent** — checks: (a) Universe with old `custom_stages: [...]` field reads cleanly (Serde ignores or `#[serde(default)]` tolerates); (b) note with legacy Zettelkasten value renders via `LEGACY_ZETTELKASTEN_EMOJI`; (c) note with `stage: spark-` (trailing dash) is treated as `spark` (suffix dropped); (d) note with `stage: -concept` (leading dash) is malformed but doesn't crash — display falls back to verbatim render; (e) on-disk dash-encoded value with non-ASCII suffix (`stage: spark-مفهوم`) renders correctly.

P0/P1 fixed before close. P2/P3 logged as PJ-NNN.

---

## Closing the cascade

After §2F:
- `Constellation Pending Jobs v1.x.md` — PJ-007 status: **shipped (per-note dash-encoded model)**. The §1A→§1D commits are the iteration record.
- Orientation doc bumped — §17 cleaned of flat-list / Universe-wide assumptions.
- NotePane Specs — §3.5.1 added covering the dash-encoded suffix and 6-entry mode-flip dropdown.
- MoCh next-block file written.

---

**Cascade starts at §2A on Eisa's "go."**
