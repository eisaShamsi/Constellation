# Wikipedia-Import Table-Explosion — Findings (2026-06-15)

> Investigated as its own workstream (Boss-approved) after MIG-078 §A′ pure-index rendering surfaced a 123 MB note. **Read-only investigation — no source or notes modified.**

## TL;DR
A combinatorial table-explosion bug in the **trial-universe generator** (NOT the shipped app) bloated 17 taxonomy notes — Spirochete.md alone is 122.9 MB (`| --- | --- |` repeated **1,078,846×** for a ~64-clade tree). 19 notes exceed 300 KB (146 MB total body; ~36% of the entire 404 MB corpus). The shipped Constellation app has **no** Wikipedia importer, so normal use cannot reproduce it — the risk is confined to re-running `lab/trial-universe/generator`.

## Where it lives
- Converter: `lab/trial-universe/generator/html-to-md.mjs` (cheerio). Offender: **`renderTable($, $table)` at lines 257-277**, reached via `render()` case `'table'` (line 240).
- Call site: `lab/trial-universe/generator/build-note.mjs:80` — `htmlToMarkdown(parsed.text)`, no size guard.
- Source HTML: `fetch-wikipedia.mjs:69` (MediaWiki Action API, full page HTML).
- Confirmed: **no** `htmlToMarkdown`/`renderTable` in `src/` or `src-tauri/` — there is no in-app "Import from Wikipedia" feature.

## Root cause (two compounding bugs)
Wikipedia renders `{{clade}}` phylogenetic trees as tables nested 10-15 levels deep.
- **Bug A (line 259):** `$table.find('tr')` is a *descendant* selector → the outer table grabs **every** `<tr>` at any depth, not just its own rows.
- **Bug B (line 262):** each cell re-renders nested tables via `render()` → a cell containing a child `<table>` re-enters `renderTable`, which (per Bug A) again pulls all *its* descendants. Multiplies at every nesting level → combinatorial blow-up; each level emits its own `| --- | --- |` separator (the dominant repeated token).

## Universe-wide (read-only `note_meta` scan, LENGTH(body_text) > 300 KB)
**19 notes, 146.41 MB total.** 17 are clade-explosions (Spirochete 122.9 MB = 84% of all bloat). Top: Spirochete 122.88 / Archaea 6.66 / Brown algae 4.70 / Green plant 1.58 / Borrelia 1.57 / Plant 1.34 / Eukaryote(s)(ic) 0.83×3 / Fungus+Fungi 0.82×2 / Electroreception+Electrolocation 0.66×2 / Evolutionary history of life 0.46 / Crown group 0.40 / Dinosaur 0.40 / Mammalian 0.31. **Not explosions (legit-large, leave alone):** Fourier transform.md (0.35 MB, 12 separators), Permian-Triassic extinction.md (0.31 MB, 1 separator).

## Recommendation (agent): Option (c) — fix the converter, then re-import
1. `renderTable`: replace `$table.find('tr')` with a direct-child row walk; decide a nested-table policy (render a clade tree as an indented list, or skip it — a recursive phylogeny is not a GFM grid).
2. Add a defensive per-table output cap in `build-note.mjs` (~line 80) as a backstop.
3. Re-import the 19 notes from their `source_url` frontmatter (all carry it). For Spirochete, optionally clear/repair in place first to unblock.
- Option (a) repair-in-place alone: frees ~141 MB fast but can't reliably reconstruct the tree (safest is to delete the phylogeny block → content loss).
- Option (b) re-import without the fix: re-runs the same bug → re-explodes. Not viable alone.

**Recurrence:** only on re-running the trial generator over `{{clade}}`/deeply-nested-table articles, until `renderTable` is fixed. Shipped app unaffected.

## Status
Findings recorded. The FIX (option c) is a separate follow-up task for Boss scheduling — independent of MIG-078 (which relocates/VACUUMs whatever body remains; cleaning these 19 at the source would reclaim ~141 MB on top of the freelist).
