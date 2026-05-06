# Stages — Concept Paper v1.0

**Status**: Draft for Eisa's review · 2026-05-06
**Supersedes**: PJ-007 / Plan §1A–§1G flat-list model (committed in MIG-014 §1A → §1D)
**Replaces commit f4eef3e in spirit** — the flat extensibility model is rejected; this paper proposes the model that will replace it.

---

## 1. Why stages exist

A Constellation note has a **lifecycle position**. Knowledge isn't born finished: an idea begins as a flicker, takes shape, accumulates evidence, settles, fades, or is retired. Every note carries one stage value at any time — the user's read on where this note currently sits.

Stages are not classifications of *what* a note is about. They are markers of *how mature, settled, or dormant* the thinking is. A note about gardening and a note about epistemology both move through the same lifecycle.

This concept aligns with the **Living Link Architecture** — the same vocabulary describes a link's life and a note's life. PJ-007 confirmed the 6-phase Living Link baseline as the canonical lifecycle for notes too. This paper formalizes that confirmation and resolves the extensibility question that PJ-007 left open.

---

## 2. The Six Fixed Stages — the canonical lifecycle

The lifecycle is a **chain of six stages**. Their order is fixed, their meaning is intrinsic to Constellation, and they cannot be removed, renamed, or reordered by the user.

| # | Name      | Emoji | Meaning |
| - | --------- | ----- | ------- |
| 1 | **Spark**     | ✨    | First ignition. A question, hypothesis, or hunch — captured before it has substance. The note exists as a placeholder for thinking that hasn't yet happened. |
| 2 | **Birth**     | 🌱    | First concrete formulation. The thought is now articulated — at least one paragraph, one defensible claim, one observation worth keeping. The note is no longer a placeholder. |
| 3 | **Growth**    | 🌿    | The note is being actively developed. Evidence is accumulating, structure is forming, internal links are appearing. This is the longest-lived phase for most notes. |
| 4 | **Maturity**  | 🌳    | The note is settled. Other notes depend on it. Its claims are defended; its body has stopped churning. It can be cited as a stable position. |
| 5 | **Dormancy**  | 😴    | Quiet. The note is preserved and still relevant, but the user isn't actively working on it. Distinct from archival — dormancy is a pause, not a retirement. |
| 6 | **Archival**  | 📦    | Retired or superseded. Kept for reference and audit, but no longer load-bearing. The note is still searchable; it is no longer a primary surface. |

**Why these six.** They are the same six the Link Architecture uses. Notes and links live in the same conceptual fabric — knowledge is the connections between thoughts; both endpoints (note + link) age through the same arc. Sharing the vocabulary across both is what makes the cognitive grammar coherent.

**Promote / demote walks this chain.** From Spark, the natural forward step is Birth; from Birth → Growth; and so on. Demote walks back. The chain is bidirectional but visually asymmetric — promote is the canonical verb (prominent), demote is the legitimate-but-occasional revision verb (subdued). At Spark, demote is hidden; at Archival, promote is hidden.

---

## 3. The Type dimension — custom stages as orthogonal tracks

A user can add a **custom stage type** — a single word like "Concept", "Idea", "Argument", "Question". The type is a *qualifier* that runs **parallel** to the lifecycle. It does not replace the six stages; it pairs with each of them.

Adding "Concept" as a custom type generates six new compound stages:

| Lifecycle phase | Default track | "Concept" track |
| --------------- | ------------- | --------------- |
| ✨ Spark        | Spark         | **Spark Concept**     |
| 🌱 Birth        | Birth         | **Birth Concept**     |
| 🌿 Growth       | Growth        | **Growth Concept**    |
| 🌳 Maturity     | Maturity      | **Maturity Concept**  |
| 😴 Dormancy     | Dormancy      | **Dormancy Concept**  |
| 📦 Archival     | Archival      | **Archival Concept**  |

Adding "Idea" alongside "Concept" generates six more, all in a third column. The lifecycle dimension stays at six; the type dimension grows by one column per custom type.

**Custom types do not get their own emoji.** The emoji is always the lifecycle phase's — ✨ for Spark, 🌱 for Birth, etc. — regardless of type. A "Spark Concept" badge looks like ✨ Spark Concept; a "Spark Idea" badge looks like ✨ Spark Idea. This keeps the visual cue consistent across types and avoids visual clash.

**Promote / demote walks the lifecycle dimension only.** It does not change the type. A note in "Birth Concept" promotes to "Growth Concept", not to "Maturity Default". The type is stable across the lifecycle; the user chooses it once via the Properties panel.

**Custom types are scoped per-Universe** — the same constraint as before. Adding "Concept" in Universe A does not surface it in Universe B.

---

## 4. The 2D matrix — example

A Universe with two custom types ("Concept", "Idea") has the following stage matrix:

| Lifecycle | Default | Concept       | Idea       |
| --------- | ------- | ------------- | ---------- |
| ✨ Spark    | Spark    | Spark Concept    | Spark Idea    |
| 🌱 Birth    | Birth    | Birth Concept    | Birth Idea    |
| 🌿 Growth   | Growth   | Growth Concept   | Growth Idea   |
| 🌳 Maturity | Maturity | Maturity Concept | Maturity Idea |
| 😴 Dormancy | Dormancy | Dormancy Concept | Dormancy Idea |
| 📦 Archival | Archival | Archival Concept | Archival Idea |

A note has exactly one cell at any time — one row (lifecycle) × one column (type). Promote moves down the row; type-change moves across columns; both are allowed.

**Total stage count = 6 × (1 + N)** where N is the number of custom types in the Universe. The promote chain length stays at **6** regardless of N.

---

## 5. UX implications

### 5.1 Properties panel — two controls, not one

The stage row in the Properties panel becomes **two adjacent controls**:

```
stage:  [ Stage:  Birth        ▾ ]   [ Type:  Concept     ▾ ]
```

- **Stage selector** — closed dropdown of the six baselines. Required field.
- **Type selector** — closed dropdown of `Default` + the Universe's custom types. Optional; default is `Default` (which renders as the bare lifecycle name).
- A "+ Add type…" entry in the Type dropdown opens a small inline input to add a new type. No emoji picker.

The combined value displayed in compact contexts (breadcrumb, tab title, search results) is `<Lifecycle> <Type>` — e.g. `Birth Concept`. When type is `Default`, the combined value is just `<Lifecycle>` — `Birth`.

### 5.2 Breadcrumb

```
[← demote]  [✨ Spark Concept]  [Promote →]
```

- Badge: lifecycle emoji + lifecycle label + (if type ≠ Default) space + type label.
- Promote → advances to the next lifecycle phase, type unchanged.
- Demote ← retreats; type unchanged.
- At Spark, demote hidden; at Archival, promote hidden.
- The Type is **not** changeable from the breadcrumb — only the Properties panel changes type. The breadcrumb is a **verb** for lifecycle progression; type is a **property** of the note.

### 5.3 File tree

Each note shows only the lifecycle emoji (the row's emoji). The type is intentionally **not** rendered in the tree — adding type tags to every line would create visual noise that defeats the at-a-glance lifecycle indicator the file tree provides today.

The full combined label is available in the tooltip on hover and in the right-click context menu.

### 5.4 360.3D / Inspector

The Inspector strip shows the full combined label: `✨ Spark Concept`. This is where the user inspects a single note in detail; the type qualifier earns its display space here.

### 5.5 Settings → Notes → Manage Custom Types

A new panel under Settings → Notes:

```
Custom note types
─────────────────
[ Concept     ]  [ Rename ] [ Remove ]
[ Idea        ]  [ Rename ] [ Remove ]

[ + Add type ]
```

- Add: type a single word; pressed Enter; appears in the list.
- Rename: in-place edit; updates all notes that reference the old name.
- Remove: warns "X notes currently use this type; they will fall back to Default. Continue?"
- No emoji picker.
- No reorder (alphabetical by default — types are unordered, the lifecycle is what's ordered).

---

## 6. Data model

### 6.1 Frontmatter — two fields

```yaml
stage: spark        # always one of the 6 baseline names; REQUIRED for staged notes
note_type: concept  # OPTIONAL; lowercase; absent = Default track
```

- `stage` — canonical lowercase lifecycle name. Always one of `spark | birth | growth | maturity | dormancy | archival`. The promote/demote chain operates only on this field.
- `note_type` — optional lowercase type name. Absent or empty string = the Default track. When present, it must reference a registered type in the Universe's `note_types` list (else falls back to Default at display time).

### 6.2 Universe schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteType {
    pub name: String,  // lowercase; e.g. "concept", "idea"
    // No emoji field — inherits from lifecycle phase.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniverseMeta {
    // … existing fields …
    #[serde(default)]
    pub note_types: Vec<NoteType>,
}
```

The MIG-014 §1A `custom_stages: Vec<CustomStage>` field is renamed to `note_types: Vec<NoteType>` and the `emoji` field on each entry is dropped.

### 6.3 Display label resolution

```typescript
function stageLabel(stage: string, noteType: string | undefined, t: i18n): string {
    const baseline = t(`notePane.stage.${stage}`);  // "Birth"
    if (!noteType) return baseline;
    const cap = noteType.charAt(0).toUpperCase() + noteType.slice(1);  // "Concept"
    return `${baseline} ${cap}`;  // "Birth Concept"
}
```

`lookupStageEmoji(stage)` simplifies — emoji depends only on the lifecycle. The legacy Zettelkasten fallback (`fleeting`, `literature`, etc.) stays for backward compat with old on-disk values.

---

## 7. Migration impact on what's already shipped

What's already committed in MIG-014 §1A → §1D needs revision. Here's the impact:

| Phase   | Current shipped state                                          | Required change                                                          |
| ------- | -------------------------------------------------------------- | ------------------------------------------------------------------------ |
| §1A     | `CustomStage { name, emoji }` + 5 IPC commands                   | Drop `emoji` field; rename `CustomStage` → `NoteType`; rename `custom_stages` → `note_types`. IPC command names follow. |
| §1B     | `customStages` writable; `LIVING_LINK_BASELINE`; 5 wrappers      | Rename writable to `noteTypes`; LIVING_LINK_BASELINE stays; wrapper signatures simplify.   |
| §1C     | Single combobox for stage; inline-add of custom flat values     | Two controls: stage selector (6 baselines, no inline-add) + type selector (registered types only, with "+ Add type…").  |
| §1C.5   | Inline custom dropdown w/ datalist replaced by custom panel      | Refactored into the type selector. The lifecycle picker uses a closed select. |
| §1D     | NotePane breadcrumb: stageOrder = 6 baselines + customs flat     | stageOrder = 6 baselines ONLY. Type label appended in display, not in chain. File tree unchanged (lifecycle emoji only). |
| §1E     | (not yet built) Settings → Manage Custom Stages with emoji picker | Becomes Manage Note Types — no emoji picker; just name + rename + remove. |

### Path forward

The existing commits stay (they record the iteration). A new MIG-014 phase set — call it §2 — supersedes the flat model:

- §2A — schema rename + drop emoji field (Rust)
- §2B — store rename + simplified wrappers (TS)
- §2C — Properties panel: two controls (stage + type)
- §2D — breadcrumb stageOrder = 6 only, label = lifecycle + (optional type)
- §2E — Settings → Manage Note Types
- §2F — help + User Manual rewrite
- §2G — three-agent audit

---

## 8. Open questions for Eisa

1. **Default type label.** When `note_type` is absent, the UI shows just `Spark`. Is that right, or should the label be `Spark Default` / `Spark General` for explicitness? (Recommendation: just `Spark`. "Default" is a programming concept; a normal user shouldn't see it.)
2. **Type input language.** A user might type a custom type in Arabic, English, or any script. Storing lowercase is sensible for Latin scripts but may not apply to all. (Recommendation: store the user's input verbatim, only trim whitespace; case-fold on comparison only for ASCII.)
3. **Type ordering in dropdowns.** Alphabetical, chronological-add order, or user-customisable? (Recommendation: chronological-add — same as the existing `custom_stages` ordering — so the user's mental model is "the order I added them.")
4. **Search facets.** Should `note_type:concept` be a filterable token in the search query language? (Recommendation: yes — simple addition, large analytical payoff.)
5. **What happens to MIG-014 §1A–§1D commits already in `main`?** Leave them as the iteration record (proven wrong, kept honest), or rewrite history? (Recommendation: leave as-is. The history is the truth, and reverting only confuses future readers.)

---

## 9. Mental anchor — "Spark Concept reads naturally"

The grammar Eisa proposed — `Spark Concept`, `Birth Concept`, `Growth Concept` — is English-correct adjective + noun. Lifecycle stage as adjective ("how mature?"), type as noun ("of what kind?"). When a user reads `Birth Concept` in the breadcrumb, they parse it as "a concept that has just been born." This is the test of whether the model is sound: the language reads naturally.

Counter-test: `Spark Argument` reads naturally ("the spark of an argument"). `Maturity Question` reads naturally ("a mature question — one we keep returning to"). `Archival Hypothesis` reads naturally ("an archived hypothesis"). The grammar holds.

What does **not** read naturally: a custom stage that tries to be a lifecycle phase by itself — `Drafting`, `Reviewing`. Those are *activities*, not types. They belong in something like a `task` or `phase` field, separate from `stage`. The concept paper deliberately rejects them as stage-type values.

---

**Awaiting Eisa's confirmation** on:
- The 2D matrix model
- Two-control UX in Properties (stage + type)
- Open questions §8.1 – §8.5

Then MIG-014 §2 plan is drafted, approved, cascaded.
