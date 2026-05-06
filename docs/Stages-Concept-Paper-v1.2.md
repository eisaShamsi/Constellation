# Stages — Concept Paper v1.2

**Status**: Pending Eisa's confirmation · supersedes `Stages-Concept-Paper-v1.0.md` and the unfinished v1.1 draft.
**Companion plan**: `lab/reports/MIG-014-NOTE-STAGE-PLAN-v4.md`.

---

## What changed from v1.0 / unfinished v1.1

Three further amendments after v1.1 was drafted:

1. **§3 / §4 — Per-note scope, NOT per-Universe.** v1.0 and v1.1 treated the custom term as Universe-wide state. Eisa: it's a property of *the note itself*. Every note independently carries (or doesn't carry) a custom term — encoded in its on-disk `stage:` value's dash suffix. New notes start fresh with the 6 fixed stages; the user types a custom term per note when desired. There is **no Universe-wide custom-term setting**.
2. **§5.1 — Dropdown always 6 entries.** v1.1 said "6 entries when empty, 12 when set." Eisa: it's always 6. Empty input → 6 fixed. Custom-term typed → 6 paired (the fixed ones are hidden in this mode). Mode flips based on the note's current stage value or the user's typing. Either way, the dropdown shows exactly 6 entries.
3. **§5.5 — Settings panel REMOVED.** v1.1 had a Settings → Custom Note Term panel for managing the Universe-wide term. With per-note scope, there's nothing Universe-wide to manage — the panel is gone. The PropertyEditor combobox is the one and only surface.

Other v1.0 / v1.1 sections that survive:
- §2 — six fixed stages and their meanings (unchanged).
- §6 — data model (single `stage:` field, dash-encoded; no Universe schema change).
- §9 — dash separator (`Spark-Concept`).

---

## 1. Why stages exist

A note's lifecycle position. Knowledge isn't born finished — an idea begins as a flicker, takes shape, accumulates evidence, settles, fades, or is retired. Stages mark where the thinking sits *now*, not what the note is *about*. (Unchanged from v1.0.)

---

## 2. The Six Fixed Stages — the canonical lifecycle

(Unchanged from v1.0.)

| # | Name      | Emoji | Meaning |
| - | --------- | ----- | ------- |
| 1 | Spark     | ✨    | First ignition — a question, hypothesis, or hunch captured before substance. |
| 2 | Birth     | 🌱    | First concrete formulation; a defensible claim. |
| 3 | Growth    | 🌿    | Active development — evidence, structure, links accumulating. |
| 4 | Maturity  | 🌳    | Settled; depended-upon; cite-stable. |
| 5 | Dormancy  | 😴    | Quiet but preserved. A pause, not a retirement. |
| 6 | Archival  | 📦    | Retired or superseded; preserved for reference. |

Promote / demote walks this chain. Demote is hidden at Spark, promote is hidden at Archival.

---

## 3. Custom term — per-note, on-disk only

A note may carry an optional **custom term** as a suffix to its lifecycle stage. The encoding is the dash separator: `spark-concept`, `birth-idea`, `maturity-question`. The lifecycle prefix is always one of the six fixed names; the suffix is whatever the user typed (any script, lowercased on disk).

**Scope is per-note.** The custom term lives in the note's frontmatter `stage:` value. Nothing else stores it. The Universe has no list of custom terms, no setting for them, no shared registry. Two notes with `stage: spark-concept` simply happen to use the same term — they aren't linked.

**A new note starts with no stage** (or with a baseline stage if the user adds one through the default 6). Adding a stage property to an existing note that lacks one defaults to the 6 fixed.

**There is no emoji per custom term.** The lifecycle phase contributes the emoji; the custom term is a text suffix only.

---

## 4. Dropdown is always six entries

The PropertyEditor stage combobox shows exactly **6 entries** at all times. Which 6 depends on the note's current stage and the user's typing:

### Mode A — fixed (default)
- Note's stage is empty, OR matches a fixed lifecycle name.
- Dropdown lists: ✨ Spark, 🌱 Birth, 🌿 Growth, 🌳 Maturity, 😴 Dormancy, 📦 Archival.

### Mode B — paired
- Note's stage has a dash-encoded suffix (e.g. `spark-concept`), OR the user is actively typing a custom term in the input.
- Dropdown lists 6 entries paired with that suffix, in lifecycle rank — e.g.:
  - ✨ Spark-Concept
  - 🌱 Birth-Concept
  - 🌿 Growth-Concept
  - 🌳 Maturity-Concept
  - 😴 Dormancy-Concept
  - 📦 Archival-Concept

The fixed list is **hidden** in Mode B. Switching back to the fixed list happens by clearing the input or picking a fixed-name value.

### Mode toggle is automatic
- Combobox reads the current input value.
- If the input matches a fixed lifecycle name (case-insensitive, no dash), Mode A.
- Otherwise (custom word, or `name-suffix` dash-encoded form), Mode B with the suffix being either the part after the dash or the whole input.

The user's typing always controls the mode. There is no separate toggle, no "Default vs Custom" switcher — just the input.

---

## 5. UX

### 5.1 Properties panel — single combobox

Single combobox in the stage row. Layout matches §1C.5 (custom inline dropdown component, replacing native `<datalist>`).

```
⚏ stage  [ 🌿  spark-concept                                ▾ ]
```

- Input value: on-disk canonical (lowercase, dash-encoded) for visibility.
- Leading emoji indicator: lifecycle prefix's emoji (✨ for `spark-*`, 🌳 for `maturity-*`, etc.).
- Dropdown opens on focus / click / arrow.
- Dropdown content: 6 entries computed live from the input value (Mode A or Mode B).
- Picking a dropdown item commits the canonical value.
- Pressing Enter on a typed value commits it verbatim if it's a valid form (fixed name or dash-encoded paired); otherwise the user is asked to pick from the dropdown.

#### Typing flow examples

1. Empty stage → click → Mode A (6 fixed). Pick `🌱 Birth`. On disk: `stage: birth`.
2. Note at `stage: birth` → click → Mode A. Type `concept` (replacing `birth` in the input). Mode flips to B. Dropdown shows 6 Concept-paired. Pick `🌱 Birth-Concept`. On disk: `stage: birth-concept`.
3. Note at `stage: spark-concept` → click → Mode B (6 Concept-paired) shown. Pick `🌿 Growth-Concept`. On disk: `stage: growth-concept`.
4. Note at `stage: spark-concept` → click → edit input from `spark-concept` to just `growth`. Mode flips to A. Pick `🌿 Growth`. On disk: `stage: growth`.

### 5.2 Breadcrumb

`[← demote]  [✨ Spark-Concept]  [Promote →]`

- Badge shows the lifecycle's emoji + the display label (`Spark`, or `Spark-Concept` if a suffix exists).
- Promote / demote walks the lifecycle WITHIN THE SAME SUFFIX:
  - `Spark-Concept` → promote → `Birth-Concept` → … → `Archival-Concept`
  - `Spark` → promote → `Birth` → … → `Archival`
- At `Spark` / `Spark-Concept`: demote hidden.
- At `Archival` / `Archival-Concept`: promote hidden.

The suffix is *not* a verb the breadcrumb exposes — to change it, the user opens the Properties panel and edits the input.

### 5.3 File tree

Each note shows the lifecycle emoji only. The suffix is not rendered in the tree. Hover tooltip shows the full label.

### 5.4 Inspector / 360.3D

Strip shows the full label: `✨ Spark-Concept`.

### 5.5 (intentionally empty — no Settings panel)

There is no Universe-wide custom-term setting and therefore no Settings panel. The Properties combobox is the one and only place the user types or changes a custom term. Per-note scope.

---

## 6. Data model

### 6.1 Frontmatter — single field

```yaml
# default track
stage: spark

# paired track
stage: spark-concept
```

The `stage:` field is the one and only persistence point. No `note_type:` field, no `custom_term:` field on the Universe.

**Encoding rules**:
- Lifecycle prefix is always one of the six fixed names: `spark | birth | growth | maturity | dormancy | archival`. Lowercase.
- If a suffix is present, it follows the dash and is **lowercased** on disk for canonical form.
- No nested dashes (suffix may not contain `-`). Validation rejects suffixes with dashes.
- Empty suffix (`spark-`) is treated as no suffix (`spark`); the trailing dash is stripped on commit.

**Legacy backward compat**: `stage: fleeting`, `stage: literature`, `stage: permanent`, `stage: synthesis` (the old Zettelkasten values) continue to render via `LEGACY_ZETTELKASTEN_EMOJI`. They aren't promote/demoteable (the fixed chain is now the 6 baselines), but they display correctly.

### 6.2 Universe schema — UNCHANGED

The §1A `custom_stages: Vec<CustomStage>` field is **dropped entirely**. `UniverseMeta` returns to its pre-MIG-014 shape. There is no Universe-level state for stages.

### 6.3 Display label resolution

```typescript
function splitStage(stage: string): { lifecycle: string; suffix: string } {
    const i = stage.indexOf('-');
    return i < 0 ? { lifecycle: stage, suffix: '' } : { lifecycle: stage.slice(0, i), suffix: stage.slice(i + 1) };
}

function stageLabel(stage: string, t: (k: string) => string): string {
    const { lifecycle, suffix } = splitStage(stage);
    const isBaseline = LIVING_LINK_BASELINE.some(b => b.name === lifecycle);
    const lifecycleLabel = isBaseline
        ? t(`notePane.stage.${lifecycle}`)
        : lifecycle.charAt(0).toUpperCase() + lifecycle.slice(1);
    if (!suffix) return lifecycleLabel;
    const suffixDisplay = suffix.charAt(0).toUpperCase() + suffix.slice(1);
    return `${lifecycleLabel}-${suffixDisplay}`;
}

function lookupStageEmoji(stage: string): string {
    const { lifecycle } = splitStage(stage);
    if (!lifecycle) return '';
    const baseline = LIVING_LINK_BASELINE.find(b => b.name === lifecycle);
    if (baseline) return baseline.emoji;
    if (LEGACY_ZETTELKASTEN_EMOJI[lifecycle]) return LEGACY_ZETTELKASTEN_EMOJI[lifecycle];
    return '';
}
```

`stageLabel` no longer takes a `term` parameter — display is derived purely from the on-disk value. The function is also pure: same input → same output, no Universe state.

### 6.4 Promote / demote logic

```typescript
function nextStage(stage: string): string | null {
    const { lifecycle, suffix } = splitStage(stage);
    const idx = LIVING_LINK_BASELINE.findIndex(b => b.name === lifecycle);
    if (idx < 0 || idx === LIVING_LINK_BASELINE.length - 1) return null;
    const nextLifecycle = LIVING_LINK_BASELINE[idx + 1].name;
    return suffix ? `${nextLifecycle}-${suffix}` : nextLifecycle;
}
function prevStage(stage: string): string | null { /* symmetric */ }
```

(Same as v1.1.)

---

## 7. Migration impact on already-shipped MIG-014 §1A → §1D

| Phase | Shipped state                                                | Required change                                                                                          |
| ----- | ------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------ |
| §1A   | `CustomStage { name, emoji }` + `custom_stages: Vec<CustomStage>` + 5 IPC commands | Drop the struct, the field, all 5 commands. UniverseMeta restored to pre-MIG-014 shape. |
| §1B   | `customStages` writable, `LIVING_LINK_BASELINE`, 5 wrappers, `isKnownStage`, `lookupStageEmoji(name, customs)` | Drop `customStages`, all 5 wrappers, `isKnownStage`. Keep `LIVING_LINK_BASELINE`. Single-arg `lookupStageEmoji(stage)`. New helpers: `splitStage`, `stageLabel`, `nextStage`, `prevStage`. |
| §1C   | Inline-add of flat custom values via `addCustomStage` IPC     | Combobox: 6 fixed (Mode A) or 6 paired (Mode B) computed from input. No IPC for custom terms. |
| §1C.5 | Custom inline dropdown component                              | Stays as the rendering shell — feed shape changes.                                                       |
| §1D   | NotePane stageOrder = baselines + customs                      | stageOrder = LIVING_LINK_BASELINE only; suffix carried through `nextStage` / `prevStage`.                |
| §1E   | (not built) Settings → Manage Custom Stages with emoji picker | **Phase removed entirely**. Per-note scope; no Settings surface.                                           |

The §1A→§1D commits stay in `main` as iteration record per Eisa's question-5 answer.

---

## 8. Open questions resolved

| # | Question                       | Resolution                                                                            |
| - | ------------------------------ | ------------------------------------------------------------------------------------- |
| 1 | Default-type label             | Just `Spark` — no qualifier when suffix is empty.                                      |
| 2 | Type input language / case     | Verbatim case-preserving on input; lowercased on disk for canonical form.              |
| 3 | Order in dropdowns             | Lifecycle rank in both modes — Spark, Birth, Growth, Maturity, Dormancy, Archival.   |
| 4 | Search facets                  | Yes — `note_type:concept` is a filterable token over the dash suffix.                  |
| 5 | §1A–§1D commits in main        | Leave as iteration record. No history rewrite.                                         |

---

## 9. Mental anchor — the dash + per-note scope

The dash is the canonical separator: on-disk `spark-concept`, displayed `Spark-Concept`. This avoids grammatical ambiguity ("Spark Concept" could be read as a noun phrase) and keeps the value as a single token in URL fragments, search facets, and serialised forms.

**Per-note scope is the philosophical commitment.** The custom term isn't a Universe-level vocabulary the user manages once and reuses everywhere; it's an act of formulation **per note**. Each note declares its own categorization. Two notes that happen to share the term "concept" aren't linked — they're independently formed.

This aligns with Constellation's commitment to **knowledge formulation, not knowledge management**. Categorisation is part of the formulation, not a precondition imposed by the system. The cost is friction (typing the term per note); the benefit is honesty (the user is forced to think about the term they're applying, every time).

---

**Awaiting Eisa's confirmation** that this matches the intent before Plan v4 is committed and the cascade resumes.
