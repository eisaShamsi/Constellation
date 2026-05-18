---
id: README
purpose: index + format key for the 24 tradition manifests shipped under MIG-026 Phase ι.1
version: 1
changelog:
  - 2026-05-18 initial (Phase ι.1)
---

# Constellation tradition manifests

This folder contains the **24 curated baseline tradition manifests** that
ship with Constellation Sight. Each manifest is the canonical
human-readable scope statement for a tradition — the document the
in-app **ⓘ disclosure button** on each chip opens (Phase ι.2 wires the
button; Phase λ ships translations).

## What a manifest is

A short scholarly briefing on what a tradition encodes when used as a
Sight visualization. Each manifest is structured to answer **four
questions** in plain language:

1. **Hero metaphor.** What does the dome look like under this tradition,
   and what visual claim does it make about your universe?
2. **Scope.** When to use this tradition — and, more important, when
   not to. Per Drabinski's "make the ideology visible" principle, every
   tradition has limits; the manifest names them.
3. **Lineage + citation.** Where this tradition comes from, who shaped
   it, and the primary + modern scholarly references a user can follow.
4. **Critique.** Honest acknowledgment of known scholarly contestations
   — what the tradition cannot do, where it has been criticized.

## Why these 24

The set is the **post-religious-lineage-filter curated baseline** locked
by the project owner on 2026-05-17 (orientation v2.10). It expands the
3 pre-existing baselines (Aristotelian + pramāṇa + masādir) plus the 2
Phase γ shippings (Polanyi + Mohist) by 19 new traditions across 7
families. The selection process is documented in:

- `docs/research/MIG-026-candidate-registers.md` — full survey of ~25
  candidates with citations + geometric implications + honest scope.
- `docs/Constellation Orientation & Onboarding v2.09.md` — the
  religious-lineage rule (Abrahamic-only for religious-source
  traditions; open door for heritage / philosophical traditions).
- `docs/Constellation Orientation & Onboarding v2.10.md` — the
  curated picks.

## Families

The 24 traditions sit in 10 family sections (per
`src/lib/sight/v6/traditions/index.ts::FAMILIES`):

| Family | Traditions | Count |
|---|---|---|
| Western classical | aristotelian | 1 |
| Indian Nyāya | pramana | 1 |
| Sunni Islamic uṣūl | masadir | 1 |
| Arabic / Islamic beyond uṣūl | ibn-rushd-burhan · shatibi-maqasid · ibn-khaldun-umran | 3 |
| Modern Western | polanyi · peirce · habermas · dewey · husserl · longino | 6 |
| Jewish (Abrahamic) | pardes · maimonidean-prophecy · talmudic-middot | 3 |
| East Asian Confucian | mencian-sprouts · wang-yangming · korean-songnihak | 3 |
| Chinese pragmatist | mohist-san-biao | 1 |
| Latin American decolonial | mignolo-pluriversal · dussel-transmodernity · maldonado-torres | 3 |
| African philosophical | akan-wiredu · ibuanyidanda | 2 |

## Geometric shapes

The 24 traditions span 9 visual shapes implemented in
`src/lib/sight/v6/anchor.ts`:

- **sectoral** — angular slices (Aristotelian, pramāṇa, masādir,
  peirce, habermas, longino, mencian, songnihak, akan-wiredu)
- **gradient** — opacity overlay (polanyi)
- **horizontal-bands** — stacked horizontal zones (mohist)
- **cyclic-flow** — ring with directional arrows (dewey)
- **rings** — concentric depth tiers (husserl, ibn-rushd, pardes,
  maldonado-torres)
- **grid** — sectoral × ring composition (shatibi)
- **binary-flow** — two-pole with directional flow (3 layouts:
  horizontal = ibn-khaldun · vertical = wang-yangming · concentric =
  dussel)
- **ladder** — N-step hierarchy (maimonidean spiral, talmudic spiral)
- **relational** — hub-and-spoke network (mignolo, ibuanyidanda)

## Per-note frontmatter

Each tradition can read a per-note frontmatter field to decide where
that note lands. The field names are documented in each manifest's
**applicability** section. Defaults apply when the field is absent
(currently always, until the Rust `LayoutCacheRow` extension lands).
The pending follow-up is filed as a Pending Job (see
`docs/Constellation Pending Jobs v1.12.md`).

## Reading order for a fresh AI

1. Read this README.
2. Read `aristotelian.md` — the default; sets the visual vocabulary.
3. Read `pramana.md` and `masadir.md` — the 4-quadrant sectoral
   pattern.
4. Read the family the user is asking about.

Translations live at `docs/traditions/<lang>/<id>.md` (Phase λ
follow-up; not yet present).
