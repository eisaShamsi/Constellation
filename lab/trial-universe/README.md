# Constellation — Trial Universe Generator

A Node script that builds a **~5000-note demonstration Universe** from public sources (Wikipedia + Wikimedia Commons + Wikidata). The output is a portable Constellation Universe — ready to be opened in Constellation and explored — designed to showcase every unique feature of the app, with emphasis on:

- **Living Link Architecture** — real typed links (supports / contradicts / causes / exemplifies / generalizes / derives-from / part-of), not just wikilinks.
- **Cognitive Engine phases** — deliberate contradictions between notes so Tension Detector finds real tension; provenance chains for historical evolution of ideas; maturity distribution from seed to canonical.
- **Multi-domain hierarchy** — a parent Universe holds **cUniverses**, each with **Libraries**, each with **Folders**, each with **Notes**. Deep nesting demonstrates the five-level hierarchy.
- **Rich media** — photos from Wikimedia Commons, embedded tables from Wikipedia content, `.base` files backed by Wikidata queries, external links to CC video clips.

## Scope

| Level | Count | Example |
|---|---|---|
| Universe | 1 | "Constellation Discovery" |
| cUniverses | 3 | Science, Humanities, Arts & Culture |
| Libraries | 12 | Physics, Biology, Earth Sciences, Computer Science, Philosophy, History, Linguistics, Religion, Literature, Music, Architecture, Film |
| Folders | ~80 | (nested, 2–3 levels deep per library) |
| Notes | ~5000 | (400–500 per library) |
| Images | ~1500 | Wikimedia Commons, attribution preserved |
| .base files | 12+ | Wikidata-backed (e.g. "Scientists by field of work") |

## Sources & licensing

- **Wikipedia content** — CC BY-SA 4.0. Each note's YAML frontmatter includes `source`, `source_url`, and `license`.
- **Wikimedia Commons media** — varying CC licenses, preserved per file.
- **Wikidata** — CC0.

Attribution is mandatory. Generated notes carry:
```yaml
source: Wikipedia
source_url: https://en.wikipedia.org/wiki/...
license: CC BY-SA 4.0
attribution: "Content derived from the Wikipedia article ... by its contributors."
```

## Build & run

```bash
cd lab/trial-universe
npm install          # installs only: gray-matter, cheerio, turndown (if we end up needing them)
node generator/index.js --stage poc      # 20 notes, fast validation
node generator/index.js --stage pilot    # ~200 notes, 2 libraries
node generator/index.js --stage full     # ~5000 notes, all 12 libraries
```

Output lands in `lab/trial-universe/output/Constellation Discovery/`. The `output/` directory is gitignored — the final Universe ships as a separate release (ZIP or its own repository).

## Link-generation strategy

Each note gets 3–8 wikilinks with cognitive types chosen algorithmically from structural and heuristic signals:

- **part-of** — note is inside a parent folder's topic (Kinematics → Classical Mechanics)
- **exemplifies** — a named instance of a category (Einstein → Physicist)
- **generalizes** — an abstraction of a concrete term (Energy → Kinetic Energy)
- **supports** — two notes on related topics in the same folder
- **derives-from** — when Wikipedia mentions a historical/intellectual predecessor
- **causes** — when the source text uses causal verbs ("led to", "caused", "resulted in")
- **contradicts** — **seeded deliberately** from a curated pairs list (e.g. Newtonian Mechanics ↔ General Relativity; Geocentrism ↔ Heliocentrism; Behaviorism ↔ Cognitivism; Determinism ↔ Free Will)

See `config/link-rules.json` for the full rule set and `config/contradiction-pairs.json` for the seeded contradictions.

## File layout of the output

```
Constellation Discovery/                   # Universe root
├── universe.json                          # Universe metadata
├── libraries.json                         # Library registry
├── child-universes/
│   ├── Science/
│   │   ├── universe.json
│   │   └── libraries/
│   │       ├── Physics/
│   │       │   ├── library.json
│   │       │   ├── .base/Classical Mechanics.base
│   │       │   ├── Classical Mechanics/
│   │       │   │   ├── Newton's Laws.md
│   │       │   │   ├── Kinematics.md
│   │       │   │   └── ...
│   │       │   ├── Relativity/
│   │       │   ├── Quantum Mechanics/
│   │       │   └── attachments/
│   │       │       └── img/
│   │       │           ├── Isaac_Newton.jpg
│   │       │           └── ...
│   │       └── Biology/
│   │           └── ...
│   ├── Humanities/
│   └── Arts & Culture/
└── LINK/                                  # Typed LINK files (CE Phase 1)
    ├── 20260414T120000Z_LINK_a7b2.md
    └── ...
```

## Integrity checks

Before publishing each note, the generator validates:
- Every `[[wikilink]]` target resolves to a real note within the Universe
- No orphans unless marked deliberate
- At least one `contradicts` link in every library that has a curated pair
- Link-type distribution roughly matches: 30% part-of, 20% supports, 15% exemplifies, 10% generalizes, 10% derives-from, 8% causes, 7% contradicts
- Maturity distribution: 1% canonical, 10% evergreen, 40% sapling, 49% seed

## Performance notes

- Rate-limited to 1 request/second against Wikipedia (polite-use policy).
- Total runtime for full build: ~4–6 hours.
- Output size: ~80–120 MB (mostly thumbnail images).
- Safe to kill and resume — state is checkpointed per-library.
