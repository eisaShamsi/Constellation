# M11-data — Constellation Lexicon corpus

**Status**: v1 (~100 hand-curated concepts) — seed → production swap.
**Output**: `src-tauri/src/lexicon/data/lexicon_v1.tsv`
**Parser**: `src-tauri/src/lexicon/parse.rs` (unchanged from M10).

## What this is

The M10 Lexical Bridge shipped a 15-concept toy seed (`seed_v1.tsv`) so the
build/bake/expand pipeline could be exercised end-to-end. This directory
holds the tooling that produces the real corpus on top of that pipeline —
the data file `lexicon_v1.tsv` that `LexiconGraph::get()` will load in
production once wired.

**Every concept in this corpus is Constellation-original content.** No
third-party wordnet data is consumed. This was a deliberate decision (see
§ "Why no third-party sources") so the corpus carries zero upstream
licensing obligations and no redistribution constraints.

## Scale policy

v1 ships a tight, high-quality core (~100 concepts covering the most-used
vocabulary for knowledge work across all 15 supported languages). Every
row is hand-verified, especially Arabic (per project-owner rule: Arabic
coverage is non-negotiable — every row must carry at least one Arabic
lemma).

Expansion to larger scale (~500, ~2K, ~20K concepts) is tracked as
M11-data-scale, a separate milestone. The architecture doesn't need
20K to work; M12 wire-up and M13 UI can proceed against 100 as cleanly
as against 20K.

## Why no third-party sources

Princeton WordNet 3.1 and the Open Multilingual Wordnet bundle were the
natural candidates for bootstrapping. Both were evaluated and rejected
per the project-owner rule: "if it requires licensing or creates
obligations, we create our own."

- **Princeton WordNet 3.1** — BSD-style, but requires a retained copyright
  notice in derivative works. That is an obligation.
- **OMW bundle** — mixed licenses per-source-wordnet. Multiple members
  are CC BY-SA (share-alike), which IS a distribution constraint on
  derivative works.
- **Wiktionary / wiktextract** — CC BY-SA 4.0. Share-alike virality.
- **GermaNet** — non-commercial. Hard out.
- **FarsNet** — research-only. Hard out.

Under a strict reading of the owner's rule, everything above is out.
Building our own eliminates the question entirely — Constellation can
redistribute its lexicon under any license it chooses, now and
forever.

## File layout

```
lab/m11-data/
├── README.md          # this file
├── concepts.json      # source of truth: all concepts + lemmas in one file
├── build.py           # concepts.json → src-tauri/src/lexicon/data/lexicon_v1.tsv
├── validate.py        # post-build sanity checks against the emitted TSV
├── regenerate.sh      # one-command rebuild: build + validate
└── (no NOTICE / LICENSE file — no upstream attribution to carry)
```

## Concept data shape (in `concepts.json`)

```json
{
  "schema_version": 1,
  "concepts": [
    {
      "id": "book",
      "pos": "Noun",
      "category": "object",
      "notes": "Volume of text, physical or digital.",
      "lemmas": {
        "en": ["book", "books"],
        "ar": ["كتاب"],
        "de": ["Buch"],
        "es": ["libro"],
        "fa": ["کتاب"],
        "fr": ["livre", "bouquin"],
        "he": ["ספר"],
        "hi": ["किताब", "पुस्तक"],
        "ja": ["本"],
        "ko": ["책"],
        "pt": ["livro"],
        "ru": ["книга"],
        "tr": ["kitap"],
        "ur": ["کتاب"],
        "zh": ["书"]
      }
    }
  ]
}
```

- `id` — kebab-case slug. Must be unique. Rendered as `c:{id}` in the TSV's
  concept_id column.
- `pos` — one of `arabic::PartOfSpeech`: `Noun` / `Verb` / `Adjective` /
  `Adverb` / `ProperNoun` / `Particle` / `Foreign` / `Unknown`.
- `category` — organizational tag for humans (not emitted to TSV). Helps
  keep concepts.json navigable as it grows (object / action / quality /
  relation / time / space / cognition / affect / body / society).
- `notes` — one-line human-readable gloss. Not emitted to TSV; review aid.
- `lemmas` — required keys are the 15 ISO-639-1 codes supported by
  `arabic::Lang`: `ar de en es fa fr he hi ja ko pt ru tr ur zh`. Each
  value is a list of strings.

### Coverage floor

Enforced by `validate.py`:

1. Every concept MUST have `en` + `ar` lemma lists with ≥1 entry each.
   (Arabic is a must per owner rule; English is the engine's pivot
   language and the primary corpus source.)
2. Every concept SHOULD have at least 8 of the 15 languages populated
   (warning, not error — a degraded row still ships).
3. Arabic / Persian / Urdu lemmas MUST round-trip through
   `arabic::normalizer::normalize_stripped` to themselves (i.e. already
   stripped of tashkeel and tatweel — the parser does the same, so
   storing already-stripped saves a normalization pass on every load).
4. Script check per language (best-effort, using Unicode blocks):
   - `ar / fa / ur` → Arabic block (U+0600–U+06FF, U+0750–U+077F, FB50–FDFF, FE70–FEFF).
   - `he` → Hebrew block (U+0590–U+05FF).
   - `hi` → Devanagari block (U+0900–U+097F).
   - `ja` → Hiragana / Katakana / CJK / Latin (Japanese allows romaji for loans).
   - `ko` → Hangul block (U+AC00–U+D7AF).
   - `zh` → CJK Unified Ideographs (U+4E00–U+9FFF).
   - `ru` → Cyrillic (U+0400–U+04FF).
   - `en / de / es / fr / pt / tr` → Latin (U+0000–U+024F) with per-lang accent tolerance.
5. No duplicate lemmas within a single concept × language cell.

## Regeneration workflow

1. Edit `concepts.json` (add, remove, or tweak concepts).
2. Run `./regenerate.sh` (or `python build.py && python validate.py`).
3. The script writes `src-tauri/src/lexicon/data/lexicon_v1.tsv` (the
   production corpus). It does NOT touch `seed_v1.tsv` — that file is
   preserved as the M10 regression fixture.
4. Build Rust side — `cargo build --release -p constellation_lib`.
   The djb2 hash of the new TSV bytes changes, so the cached `.bin`
   bundle is invalidated automatically and the next boot writes a fresh
   cache file.
5. Run `cargo test --lib lexicon::` to verify end-to-end.

## How the Rust side consumes this

`src-tauri/src/lexicon/graph.rs::seed_tsv()` currently returns
`include_str!("data/seed_v1.tsv")`. Once this corpus is wired, that
function returns `include_str!("data/lexicon_v1.tsv")` instead. The seed
stays on disk as the fixture for the `real_seed_bundle_*` test. A
mirror test `real_lexicon_bundle_writes_reads_reconstructs` covers the
production corpus.

No other Rust code changes. The parser is scale-independent; the
baker is scale-independent; the expand / detect paths are
scale-independent. This directory is pure data-layer work.

## Follow-ons (outside v1 scope)

- **M11-data-scale**: expand from ~100 concepts to ~500, then ~2K, then
  ~20K. At ~2K hand-curation scales; past that, LLM-assisted generation
  + a validation harness may be needed. Ship v1 first, measure user
  value, decide.
- **M11-data-synonyms**: today each concept carries 1–3 lemmas per
  language. M8-style synonym edges (in-language near-equivalents) could
  be added by splitting each concept into multiple sense-tagged nodes.
  The M10 node shape (`SenseId`) is already prepared for this.
- **M11-data-domains**: domain-specific packs (science / philosophy /
  arts / Islamic studies / medicine). These can ship as expansion packs
  (M13 in the session-log numbering — not your M13) layered on top of
  the core corpus via `LexiconBundle::merge` (not yet implemented).
