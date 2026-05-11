# Epistemic Metadata

*(MIG-022 §A — gap-analysis §6.1 schema extensions)*

This topic describes a small set of **optional frontmatter fields** Constellation now recognizes for richer epistemic classification of your notes. They were added in response to the gap analysis (`docs/epistemic-content-gap-analysis.md`) — the recognition that the two-axis Source × Content Type model the Constellation Epistemic Content Engine (CECE) classifies against doesn't cover everything you might want to record about how you came to know what you know.

These fields are **all optional**. Existing notes without them work unchanged. You add them by hand (or, in the future, via a structured editor) when a note is the kind of knowledge that benefits from the extra signal.

---

## The fields

### `held_by` — *whose stance is this?*

A short string indicating who holds the position the note describes. Defaults to `user` (your own stance). Other values you might use:
- A scholar's name: `held_by: "al-Shāfiʿī"`
- A school: `held_by: "Ḥanafī"`
- A historical figure: `held_by: "Aristotle"`

When you write a note that records *someone else's* position rather than your own, `held_by` is the field that says so. Without it, Constellation tacitly assumes the note's epistemic state is your own — which for serious scholarly work is often wrong.

### `domain` — *what subject matter is this about?*

A list of disciplinary tags. Distinct from your free-form `tags` field (folksonomy / mood / project), `domain` is the structured discipline/topic field for retrieval and filtering. Examples:

```yaml
domain: [fiqh, ʿibādāt]
domain: [photography, optics]
domain: [overland-travel, mechanics]
```

A note classified as `content_type: "proposition"` AND `source: "inference"` could be a logic theorem (domain: `[logic, mathematics]`) or a legal opinion (domain: `[fiqh, ʿibādāt]`) — same epistemic shape, very different retrieval contexts. `domain` lets you say which.

### `function` — *what is this note for?*

A single string identifying the note's intended use. Recognized values:

- `reference` — read when needed (a definition, a citation, a fact you'll look up later)
- `seed` — incubate (an early-stage idea you're still developing)
- `actionable` — do something with this (a task, a follow-up, a decision to make)
- `shipped` — finished product (a published essay, a delivered analysis, a closed loop)

Distinct from CECE's content-type axis (which says what KIND of knowledge it is) — function says what you'll DO with the note.

### `provenance_civilization` — *what tradition's vocabulary is at work?*

An optional string identifying the civilizational footprint of the note's vocabulary. Useful for retrieval against tradition-specific corpora. Examples:

- `provenance_civilization: "sunni-usuli"` — Sunni *uṣūl al-fiqh* tradition (al-Bukhārī, al-Ghazālī, al-Āmidī)
- `provenance_civilization: "analytic-western"` — post-Frege analytic philosophy
- `provenance_civilization: "nyaya"` — Indian Nyāya school of pramāṇa epistemology
- `provenance_civilization: "buddhist-pramana"` — Buddhist epistemological tradition (Dignāga, Dharmakīrti)

Most notes don't need this. When you have, say, a note that draws on both Sunni *uṣūl* AND analytic Anglo-American epistemology, recording the primary footprint helps future-you retrieve the right comparable material.

### `updated_at` — *when did your stance last change?*

ISO date of the most recent deliberate revision of the note's epistemic content. Distinct from the file-system `modified` timestamp (which catches every save, even typo fixes); `updated_at` is the timestamp YOU set when you've actually rethought the position.

```yaml
updated_at: 2026-05-09
```

Useful when the rest of the §6.3 temporal axis lands (note state history) — until then, this is a single-snapshot field that records "the last time I revised my view."

### `ikhtilāf` — *structured scholarly disagreement*

The most complex of the new fields. Records *ikhtilāf* — the structured disagreement among scholars or schools on a question — as a list of `{school, position}` pairs. Constellation provides a custom Properties-panel widget for editing this; you can also edit the YAML directly.

Example:

```yaml
ikhtilāf:
  - school: Ḥanafī
    position: permissible
  - school: Mālikī
    position: discouraged
  - school: Shāfiʿī
    position: permissible with conditions
  - school: Ḥanbalī
    position: forbidden
```

A note with `ikhtilāf` is not in any single epistemic state — it records a *structured disagreement* among multiple agents. Without this field, Constellation would treat such a note as if it held one of these positions itself, which is wrong.

The Properties panel renders each row as an editor card with two inputs (school + position) plus a remove button, and an "Add school" button at the bottom.

### `warrant` and `warrant_notes` — *parsed but inert (for now)*

Two fields are parsed and stored on disk but **not surfaced in any UI yet**:

- `warrant: "mutawātir"` — a grade label for the warrant of the note's claim. The Sunni *uṣūl* hierarchy uses *mutawātir / mashhūr / āḥād* and within hadith specifically *ṣaḥīḥ / ḥasan / ḍaʿīf / mawḍūʿ*. Other traditions have their own grading vocabularies.
- `warrant_notes: "transmitted by 30+ companions in al-Bukhārī"` — free text supporting the warrant grade.

These are ready to use when the **Constellation Warrant Research workstream** ships its classifier (multi-month research project; see the gap analysis §6.2). Until then you can fill them in by hand and the data persists; nothing displays it. Future warrant-aware queries and badges read these values directly.

---

## Where these fields appear

When you fill any of the new fields in a note's frontmatter, they appear in the **Properties panel** (right-sidebar) the same way every other YAML field does — one row per key, with the type-appropriate editor:

- `held_by`, `function`, `provenance_civilization`, `warrant`, `warrant_notes` → text input
- `domain` → tag list (add by typing + Enter, remove with the × on each tag)
- `updated_at` → date picker
- `ikhtilāf` → custom widget with `school` / `position` rows + add/remove buttons

---

## What about `supersedes`?

`supersedes` is technically a *relationship between notes* rather than a property of a single note. Constellation handles it as a **typed link**, not a YAML scalar:

```markdown
This note replaces my earlier analysis: [[old-note-id|supersedes]]
```

The `|supersedes` suffix on the wikilink tells Constellation this is a typed-link of the `supersedes` kind — it gets a distinct pill color (slate blue-gray), shows up in Backlinks + Outgoing Links panels alongside other typed-links, and participates in the Living Link Architecture (weight, lifecycle, traversal counts).

This keeps note-to-note relationships in one place — the typed-link system — rather than splitting them between typed-links and frontmatter scalars. Same applies to `contradicts:` (already a typed-link in pre-MIG-022 vocabulary).

---

## What this is NOT

These fields are **NOT** consumed by CECE classification today. CECE classifies on Source × Content Type only; the new metadata fields are recorded for human-driven retrieval, future warrant-aware classifiers, and the temporal axis (when it ships).

In particular:
- `function: "actionable"` does NOT auto-create a task in the Tasks panel
- `held_by: "al-Shāfiʿī"` does NOT change how CECE classifies the note
- `domain: [fiqh]` does NOT filter your search results unless you write the search query to include it

The fields are **schema** — a recognized vocabulary for fields you can add. Future MIGs will ship features that consume them (warrant classifier, temporal queries, domain-aware filtering, etc.).

---

## A worked example

A note recording the Sunni schools' positions on whether breaking dawn fast obligation matters for the validity of the day:

```yaml
---
title: Niyyah for Ramadan fasting
held_by: user
domain: [fiqh, ʿibādāt, sawm]
function: reference
provenance_civilization: sunni-usuli
updated_at: 2026-05-09
warrant: mashhūr
ikhtilāf:
  - school: Ḥanafī
    position: night-before niyyah valid; same-day niyyah valid before zawāl
  - school: Mālikī
    position: night-before niyyah required; one general niyyah for the month suffices
  - school: Shāfiʿī
    position: night-before niyyah required for each obligatory fast
  - school: Ḥanbalī
    position: night-before niyyah required for each obligatory fast
---

The classical Mālikī position (one niyyah for the month) is described
by [[Ibn-Rushd-bidayah|derives-from]] in the bidāyat al-mujtahid passage
on niyyah. My current view: [[ramadan-niyyah-personal|supersedes]]
my earlier note that conflated the Mālikī position with the Shāfiʿī one.
```

Six of the seven new fields populated; `warrant_notes` omitted (no chain detail to record yet); `supersedes` and `derives-from` as typed-links in the body, not as YAML scalars.

---

*MIG-022 §A — schema extensions land in this Constellation build. The Warrant Research workstream (separate Concept Paper, multi-month) ships the warrant classifier that consumes the `warrant` field. The temporal axis (MIG-023, separate Architect cycle) consumes `updated_at` plus the broader note state history.*
