# Stages — Concept Paper v1.1

**Status**: Approved-with-amendments (Eisa, 2026-05-06) · supersedes `Stages-Concept-Paper-v1.0.md`
**Companion plan**: `lab/reports/MIG-014-NOTE-STAGE-PLAN-v3.md`

---

## What changed from v1.0

Three Eisa amendments after first read:

1. **§4 — only ONE custom term per Universe.** v1.0 modeled multiple types (Concept, Idea, Argument …). Eisa clarified: at most one custom term active at any time. Adding a new one replaces the previous.
2. **§5.1 — one control, not two.** v1.0 proposed Stage + Type as separate controls. Eisa: a single combobox. As soon as the user types a new custom term, the app generates the six paired stages immediately. No emoji picker.
3. **§9 — dash separator.** Compound labels are written `Spark-Concept`, not `Spark Concept`. Removes grammatical ambiguity ("is `Spark Concept` a noun phrase or a stage name?") and visually marks the value as a single token.

Sections 1–3, 6–8 stand as in v1.0; sections 4, 5.1, 9 are rewritten below.

---

## 1. Why stages exist

(Unchanged from v1.0.) A note's lifecycle position. Knowledge isn't born finished — an idea begins as a flicker, takes shape, accumulates evidence, settles, fades, or is retired.

---

## 2. The Six Fixed Stages — the canonical lifecycle

(Unchanged.) Spark ✨ → Birth 🌱 → Growth 🌿 → Maturity 🌳 → Dormancy 😴 → Archival 📦. Promote / demote walks this chain; demote is hidden at Spark, promote is hidden at Archival.

---

## 3. The Type dimension — at most one custom term

A Universe carries **zero or one** custom term — a single word like "Concept" / "Idea" / "Argument" / "Question" / "ملاحظة" (any script).

When set, the term **pairs with each of the six fixed stages**, generating six compound stages:

| Lifecycle | (No term) | Custom = "Concept" |
| --------- | --------- | ------------------ |
| ✨ Spark    | Spark    | **Spark-Concept**    |
| 🌱 Birth    | Birth    | **Birth-Concept**    |
| 🌿 Growth   | Growth   | **Growth-Concept**   |
| 🌳 Maturity | Maturity | **Maturity-Concept** |
| 😴 Dormancy | Dormancy | **Dormancy-Concept** |
| 📦 Archival | Archival | **Archival-Concept** |

Total stages in the Universe = **6** (no custom term) or **12** (one custom term). The promote / demote chain length is always **6** — within the same track (default or paired).

**Custom term carries no emoji.** The lifecycle phase contributes the emoji (✨, 🌱, 🌿, 🌳, 😴, 📦); the custom term is text-only.

**Replacing the custom term.** Setting a new one replaces the existing. Notes that referenced paired stages of the old term keep their on-disk values verbatim (no silent migration); the breadcrumb and dropdown render them via legacy fallback. The user can manually re-pick a stage from the new 12 if desired.

---

## 4. The 2D matrix — one column

```
                     ┌──────────────┬─────────────────────┐
                     │   Default    │  Custom = "Concept" │
┌────────────────────┼──────────────┼─────────────────────┤
│ ✨ Spark            │  Spark        │  Spark-Concept       │
│ 🌱 Birth            │  Birth        │  Birth-Concept       │
│ 🌿 Growth           │  Growth       │  Growth-Concept      │
│ 🌳 Maturity         │  Maturity     │  Maturity-Concept    │
│ 😴 Dormancy         │  Dormancy     │  Dormancy-Concept    │
│ 📦 Archival         │  Archival     │  Archival-Concept    │
└────────────────────┴──────────────┴─────────────────────┘
```

One row per lifecycle phase, two columns max (default + optional custom).

---

## 5. UX implications

### 5.1 Properties panel — one control

Single combobox in the stage row.

```
⚏ stage  [ ✨ Spark-Concept                              ▾ ]
```

When opened, the dropdown shows entries in this order:

1. The 6 baseline stages (default track), in lifecycle rank — Spark, Birth, Growth, Maturity, Dormancy, Archival.
2. **If a custom term is set**: the 6 paired stages, in lifecycle rank — Spark-Concept, Birth-Concept, Growth-Concept, Maturity-Concept, Dormancy-Concept, Archival-Concept.
3. **If no custom term yet**: a single "+ Set custom term…" action at the bottom, which opens a small inline input.

Typing in the combobox filters the visible entries by substring.

**Setting / replacing the custom term**:
- If no custom term yet: clicking the "+ Set custom term…" entry opens an inline input. The user types a word (any script, case verbatim). Confirming sets `universe.json#custom_term`. The dropdown re-renders with 12 entries.
- If a custom term exists: replacing happens via Settings → Notes → Custom Note Term (§5.5). The PropertyEditor combobox does not offer in-place replacement once set — that surface is closed-set after the term exists, to prevent casual re-typing from blowing away every paired-stage assignment in the Universe.

### 5.2 Breadcrumb

`[← demote]  [✨ Spark-Concept]  [Promote →]`

Promote / demote walks the lifecycle within the same suffix:
- `Spark-Concept` → promote → `Birth-Concept`
- `Birth-Concept` → promote → `Growth-Concept`
- … `Archival-Concept` → no further (promote hidden)
- `Spark-Concept` → demote hidden (no Concept-track step before Spark)

Crossing tracks (default ↔ paired) is **not** a promote/demote action — it's a property change in the Properties panel.

### 5.3 File tree

(Unchanged from v1.0.) Each note shows only the lifecycle emoji. The custom-term suffix is not rendered in the tree. Tooltip shows the full label.

### 5.4 Inspector / 360.3D

Strip shows the full label: `✨ Spark-Concept`.

### 5.5 Settings → Notes → Custom Note Term

```
Custom note term
─────────────────
Term: [ Concept                  ]   [ Save ]   [ Remove ]

Notes currently using paired stages: 17
```

- Empty input + Save = no-op (use Remove instead).
- New value + Save = replaces. Confirmation dialog: "X notes have paired-stage values referencing the previous term. Their on-disk values stay; they will display via fallback. Continue?"
- Remove = clears `custom_term` to empty string. Existing paired-stage notes retain on-disk values; the dropdown shows only 6 entries until a new term is set.

---

## 6. Data model

### 6.1 Frontmatter — single field

```yaml
stage: spark            # default track
# OR
stage: spark-concept    # paired track ("spark" + "-" + custom term, lowercased)
```

A single `stage:` field carries either a baseline name or a paired name. The dash separator is the canonical encoding of "lifecycle-customterm" on disk. For parsing: split on the first dash; left side is the lifecycle phase; right side is the custom term (verbatim case **stripped** to lowercase on disk for consistency, but case-preserved version is in `universe.json#custom_term`).

### 6.2 Universe schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniverseMeta {
    // … existing fields …
    /// MIG-014 — at most one custom term per Universe; verbatim case
    /// (e.g. "Concept", "ملاحظة", "Idée"). Empty string = no custom term.
    /// Paired stage values on disk are encoded `lifecycle-customterm-lowercased`
    /// (`spark-concept`); `custom_term` here preserves the user's typed case
    /// for display in the breadcrumb and dropdowns.
    #[serde(default)]
    pub custom_term: String,
}
```

The §1A `CustomStage { name, emoji }` struct is dropped. The `note_types: Vec<NoteType>` v1.0-paper concept is dropped. One field, one optional value.

### 6.3 Display label resolution

```typescript
function stageLabel(stage: string, customTerm: string, t: (key: string) => string): string {
    const dashIdx = stage.indexOf('-');
    const lifecycle = dashIdx >= 0 ? stage.slice(0, dashIdx) : stage;
    const suffix = dashIdx >= 0 ? stage.slice(dashIdx + 1) : '';
    const isBaseline = LIVING_LINK_BASELINE.some(b => b.name === lifecycle);
    const lifecycleLabel = isBaseline
        ? t(`notePane.stage.${lifecycle}`)
        : lifecycle.charAt(0).toUpperCase() + lifecycle.slice(1);
    if (!suffix) return lifecycleLabel;
    // Render the suffix with the Universe's verbatim case if it matches the
    // current custom_term; otherwise capitalize the on-disk lowercase version.
    const suffixDisplay = suffix === customTerm.toLowerCase()
        ? customTerm
        : suffix.charAt(0).toUpperCase() + suffix.slice(1);
    return `${lifecycleLabel}-${suffixDisplay}`;
}
```

`lookupStageEmoji(stage)` — same as v1.0: parse the lifecycle prefix; resolve via `LIVING_LINK_BASELINE` or `LEGACY_ZETTELKASTEN_EMOJI`.

### 6.4 Promote / demote logic

```typescript
function nextStage(current: string): string | null {
    const [lifecycle, ...suffixParts] = current.split('-');
    const suffix = suffixParts.join('-');  // safety for any edge case
    const idx = LIVING_LINK_BASELINE.findIndex(b => b.name === lifecycle);
    if (idx < 0 || idx === LIVING_LINK_BASELINE.length - 1) return null;
    const nextLifecycle = LIVING_LINK_BASELINE[idx + 1].name;
    return suffix ? `${nextLifecycle}-${suffix}` : nextLifecycle;
}
```

Demote symmetric. Suffix carried verbatim across the chain.

---

## 7. Migration impact on already-shipped MIG-014 §1A → §1D

| Phase | Shipped state                                                        | Required change                                                                                              |
| ----- | -------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| §1A   | `CustomStage { name, emoji }` + `custom_stages: Vec<CustomStage>` + 5 IPC commands | Drop the struct; replace with `custom_term: String`. Two IPC commands suffice: `read_custom_term`, `set_custom_term`, `remove_custom_term`. |
| §1B   | `customStages` writable + `LIVING_LINK_BASELINE` + 5 wrappers + `isKnownStage` + `lookupStageEmoji(name, customs)` | Writable: `customTerm` (single string). Wrappers: 3. Drop `isKnownStage`. `lookupStageEmoji(stage)` takes only the stage value.            |
| §1C   | Inline-add of custom values into a flat list                          | Single combobox: 6 entries (no term) or 12 entries (term set) + `+ Set custom term…` action when none.       |
| §1C.5 | Custom inline dropdown component                                      | Stays as the rendering shell — the data feeding it changes.                                                   |
| §1D   | NotePane stageOrder = baselines + customs                              | stageOrder = `LIVING_LINK_BASELINE` only; suffix carried via the value's dash encoding.                       |
| §1E   | (not built) Settings → Manage Custom Stages                           | Becomes Settings → Custom Note Term — single text input + Save / Remove.                                      |

Already-committed §1A → §1D stay as iteration record per Eisa's question-5 answer.

---

## 8. Open questions resolved

| # | Question                       | Resolution                                                    |
| - | ------------------------------ | ------------------------------------------------------------- |
| 1 | Default-type label             | Just `Spark` (no qualifier).                                    |
| 2 | Type input language / case     | Verbatim — any script, case preserved in `custom_term`.        |
| 3 | Order in dropdowns             | Lifecycle rank — Spark, Birth, Growth, …, Archival; then paired in same order. |
| 4 | Search facets                  | Yes — `note_type:concept` is a filterable token.               |
| 5 | §1A–§1D commits in main        | Leave as iteration record.                                     |

---

## 9. Mental anchor — the dash

Compound stages use a dash: `Spark-Concept`, `Birth-Concept`. This is the canonical separator both on disk and in display.

Why a dash:
- It marks the value as a single token. `Spark-Concept` reads as one stage; `Spark Concept` could be a noun phrase ("the spark of a concept") or two separate words.
- It's URL-safe and search-token-safe (search facet `note_type:concept` keys cleanly off the suffix).
- It avoids parser ambiguity for users who put adjective-like custom terms ("New", "Old") that could mis-parse against the lifecycle name.

Visual: **Spark-Concept** with bold capitalization on each word makes the structure clear at a glance.

---

**Plan v3 below**: `lab/reports/MIG-014-NOTE-STAGE-PLAN-v3.md`. Already adjusted for the §3 / §4 / §5.1 / §9 amendments.
