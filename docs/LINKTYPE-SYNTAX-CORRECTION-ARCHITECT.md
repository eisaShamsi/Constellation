# Link-Type Syntax Correction — Architect

**Prerequisite to MIG-066 §B.** Date: 2026-05-30. Author of facts: Eisa. Maintainer: Claude.
**Governing:** "NEVER accept the wrong approaches" (Eisa, 2026-05-30) · file-over-app · secure-don't-muddle.

## 1. Why

MIG-066 §B exposed that **every link in the index has `link_type='relates'`** — the backend `extract_typed_links` (search.rs:3614) parses a predicate-FIRST convention (`[[type::target]]`, the form Semantic MediaWiki uses) while the data + the editor use a predicate-LAST convention (`[[target|display|type]]`). They disagree, so the backend defaults all 644k links to `'relates'`. Eisa's decision: don't patch the parser to read the wrong syntax — **convert the data to the right, field-standard, predicate-first form.**

## 2. The canonical form (decision)

**`[[type::Target]]`** and **`[[type::Target|display]]`** — predicate-first, `::`-marked, single inline token.

- Matches Semantic MediaWiki's `[[property::value|display]]`; predicate-first like Dataview's `key:: [[value]]`; both are the field standard ([SMW in-text annotation](https://www.semantic-mediawiki.org/wiki/Help:In-text_annotation), [Dataview metadata](https://blacksmithgu.github.io/obsidian-dataview/annotation/add-metadata/)).
- Reads as a sentence ("supports Stone Age"), doesn't overload the display pipe `|`, stays one inline token (works mid-prose, unlike Dataview's separate field).
- **The backend regex already parses it** — `([a-zA-Z\-]+)::` — so no backend parse change is needed for the new form.

## 3. Data shape (verified — `lab/reports/analyze_links.py`)

7,659 files, 644,604 wikilinks. 99.99% cleanly typed; vocabulary = exactly the canonical 8.

| Form | Count | Action |
|---|--:|---|
| `[[X\|display\|type]]` | 483,577 | → `[[type::X\|display]]` |
| `[[X\|type]]` | 160,957 | → `[[type::X]]` |
| `[[X]]` (untyped) | 58 | leave |
| `[[X\|display]]` (tail ∉ types) | 9 | leave |
| `[[type::X…]]` (already done) | 2 | skip (idempotent) |
| 4-part broken-nested | 1 | leave |

Edge cases to honor: empty `[[|]]`, interwiki `[[:ur:…|ʔ]]`, LaTeX/code `[[{\text{…}}]]` / `[[nodiscard]]` — all have a non-type tail (or no pipe) so they're **left untouched** by the "tail must be a canonical type" rule.

## 4. Scope — a coordinated change (the honest full picture)

Converting the data to `::` would break the current editor rendering (it reads type-LAST) unless the editor is updated in lockstep. To keep **nothing broken at any moment**, the migration makes both ends accept BOTH forms first, then converts, then re-indexes:

1. **Backend** (`extract_typed_links`): accept BOTH `[[type::target]]` (already) AND legacy `[[target|display|type]]` (add last-segment-as-type-if-known). Default → `associative` (canonical null), not `relates`. *(This alone already fixes the index even before conversion — the safety net.)*
2. **Editor** (`livePreview.ts` decoration + `completions.ts` autocomplete): render + suggest the `::` form; keep reading the legacy form during transition.
3. **Converter** (one-time, this universe): rewrite the 644k typed links to `::`. Backed-up, dry-run-first, idempotent, code-block-safe, UTF-8/RTL-safe.
4. **Re-index**: the §A.2 reconcile rebuilds `note_links` on next boot with correct types.
5. **§E**: the link-type lists/colors unify to the canonical 8 (already planned).

## 5. Converter rules (per link `[[body]]`, skipping `![[…]]`, fenced/inline code)

- `::` in body → **skip** (already canonical).
- split on `|`:
  - 1 part → leave.
  - 2 parts, tail ∈ canonical-8 → `[[tail::part0]]`.
  - 3 parts, tail ∈ canonical-8 → `[[tail::part0|part1]]` (display preserved verbatim — the converter fixes the TYPE position only, never alters displayed text).
  - otherwise → leave.
- Preserve exact bytes elsewhere (read/write UTF-8, `newline=''` to keep line endings).

## 6. Safe execution plan

1. **Backup** the universe (ZIP `E:\Cognitive Knowledge` + `E:\Constellation Universes\Eisa Cognitive Knowledge`) to `E:/Backups/` before any write. Non-negotiable.
2. Ship backend+editor "accept both forms" (built + tested) so the app tolerates either at every step.
3. **Dry-run** the converter → report per-library counts + 20 before/after samples (incl. Arabic + 3-part). Eisa reviews.
4. **Convert** for real (idempotent; re-runnable).
5. Re-launch → §A.2 reconcile rebuilds the index → spot-check `note_links` types via `analyze_links` / a SQL probe.
6. **Resume MIG-066 §B** — the Link types column now populates.

## 7. Invariants

- File-over-app: only typed-link TYPE position changes; displayed text, targets, annotations, body, frontmatter untouched. No data loss.
- Reversible: full backup + idempotent converter + git-tracked code.
- Nothing-broken-mid-flight: both forms accepted before conversion.
- Unicode/RTL: Arabic targets/displays preserved byte-for-byte.
- Scope = the "Eisa Cognitive Knowledge" universe's 18 libraries (no cUniverse children).
