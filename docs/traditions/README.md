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

## User-defined traditions (Phase κ.1 — 2026-05-18)

You can author your own traditions as **declarative JSON files**
without writing TypeScript. They appear in the chip dropdown
alongside the curated baseline.

### Setup

1. In your active Universe, create the folder
   `<Universe>/.constellation/traditions/` if it doesn't already
   exist.
2. Drop a JSON file into it. The filename must end in `.json` —
   anything else (including subfolders) is ignored.
3. Restart Constellation. On Sight open, the file loads.

### Template

Start from `docs/traditions/schema/EXAMPLE.json`. Copy it into your
Universe's traditions folder, rename to anything ending in `.json`,
and edit the `id`, `name`, `family`, sector labels, and angles to
make your own.

The full schema reference is at
`docs/traditions/schema/tradition.v1.schema.json`.

### Required fields (all v1 schemas)

- `schema_version: 1` — the only version this Constellation build
  recognizes. A mismatched value causes the file to be skipped with
  a console warning (Plan §12.1 Stage 3).
- `id` — must start with `user-` prefix; pattern
  `^user-[a-z0-9][a-z0-9-]{2,40}$`. Namespaces user traditions away
  from curated ones.
- `name` — display label, 1–60 chars.
- `shape` — one of `sectoral`, `rings`, `horizontal-bands`,
  `gradient`. Other shapes (grid, ladder, relational, cyclic-flow,
  binary-flow) need the κ.2 TS plugin loader.

### Per-shape required spec

- `sectoral` → `sectorDividers` array (2–8 entries, each with
  `angleStartDeg`, `angleEndDeg`, `label`).
- `rings` → `rings` array (2–8 entries, each with `innerFrac`,
  `outerFrac`, `label`).
- `horizontal-bands` → `horizontalBands` array (2–6 entries, each
  with `label`).
- `gradient` → `gradient` object with `centerOpacity`,
  `edgeOpacity`, optional labels.

### Optional fields

- `family` — defaults to `user-defined` (groups all user traditions
  under a "User-defined" section at the bottom of the dropdown).
- `tooltip`, `scope`, `citation` — surfaced in the chip tooltip,
  scope strip, and ⓘ disclosure modal respectively.

### Behavior

- The chip dropdown gets a new "User-defined" section at the bottom
  listing every valid user tradition (filename-sorted).
- Click a user-tradition row → dome re-arranges per the declarative
  shape (stars are hash-bucketed across sectors/rings/bands
  deterministically until per-note frontmatter integration ships).
- Click the ⓘ button on a user-tradition row → modal shows a
  synthesized manifest with the JSON's name, scope, and citation.

### Validation behavior

- Missing required fields → file skipped with a console warning
  naming the file + the specific violation.
- Schema version mismatch (e.g. `schema_version: 99`) → file
  skipped with a console warning.
- One bad file does NOT prevent other files in the folder from
  loading.

### TS plugin loader (Phase κ.2 — not shipped yet)

For traditions that need arbitrary remap functions or shapes
beyond the 4 declarative ones, Phase κ.2 will add a TS plugin
loader with Obsidian-trust security model. Until then, the
declarative JSON layer is the only user-extension path.
