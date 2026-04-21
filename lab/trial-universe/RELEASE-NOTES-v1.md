# Constellation Trial Universe v1 — 2026-04-14

The first public release of the Constellation Discovery trial Universe.

## What it is

A **7,600-note demonstration Universe** built from Wikipedia, Wikimedia Commons, and Wikidata. Ships as a drop-in Constellation Universe that showcases every unique feature of the app — from the five-level Knowledge Hierarchy to the Living Link Architecture's seeded intellectual contradictions.

## Download

`constellation-trial-universe-v1-20260414.zip` — available in this project's GitHub Releases.

Uncompressed: **967 MB** on disk (including 4,064 Wikimedia Commons images).

## Headline numbers

| Metric | Value |
|---|---|
| Notes | **7,600** |
| cUniverses | 4 |
| Libraries | 16 |
| Folders | ~80 |
| Images (CC-attributed) | 4,064 |
| Typed links total | 656,855 |
| Curated `contradicts` links | **19,321** (3.0%) |
| Wikipedia source languages | English + Arabic |
| Skipped seeds | 49 (disambiguation / 404) |

## Link type distribution

| Type | Count | % |
|---|---|---|
| derives-from | 283,078 | 43.9% |
| supports | 276,771 | 42.9% |
| exemplifies | 36,917 | 5.7% |
| **contradicts** | **19,321** | **3.0%** |
| causes | 17,405 | 2.7% |
| part-of | 9,313 | 1.4% |
| generalizes | 2,050 | 0.3% |

The `contradicts` links are the most valuable part of the dataset — they're seeded from curated intellectual-disagreement pairs (Newton ↔ Einstein, Rationalism ↔ Empiricism, Copenhagen ↔ Many-Worlds, Monotheism ↔ Polytheism, المعتزلة ↔ الأشاعرة, Neptunism ↔ Plutonism, and dozens more). Constellation's Tension Detector uses these to demonstrate what "knowledge formulation, not management" looks like in practice.

## What to try

1. **Extract the ZIP** to any location on your drive.
2. **Open Constellation** and point it at the extracted `Constellation Discovery/` folder as a Universe.
3. **Add each library** via the "New Library" flow (or add the top-level folder to include everything).
4. **Try these**:
   - Open the **Sky View** on any library — see the typed link graph.
   - Open the **Knowledge Health Dashboard** (brain icon in the dock) — get the Universe Health score, top strongest evidence chains, active tensions.
   - Open any two notes from a curated contradiction pair (e.g. `Classical mechanics` and `General relativity`) and follow the `contradicts` link between them.
   - Navigate via **Constellation Sight** and **Constellation Map**.

## Known limitations

- `derives-from` is overweighted (43.9%) — the in-body heuristic and infobox "influenced_by" field both feed it. Future generator tuning can rebalance by scaling back the infobox contribution.
- A few hundred notes have minimal body text because their source Wikipedia article was a stub.
- Canvas / Excalidraw / native `.base` files are not yet included — planned for v2.
- Note transclusion between notes is supported by Constellation but not pre-seeded in the dataset.

## License

All content is derived from Wikipedia / Wikimedia Commons under **CC BY-SA 4.0**. Wikidata structured data is **CC0**. See `LICENSES.md` inside the Universe for full attribution details. Each note's frontmatter carries its own `source_url` and `attribution` fields.

## Credits

- **Data**: Wikipedia, Wikimedia Commons, Wikidata communities (many thousands of contributors).
- **Generator**: Constellation team. See `lab/trial-universe/generator/` in the Constellation repository for the MIT-licensed build pipeline.
- **Target app**: Constellation (MIT) — <https://github.com/eisaShamsi/Constellation>.
