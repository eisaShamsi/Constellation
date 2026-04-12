# Session Log — 2026-04-08 / 2026-04-09 / 2026-04-10 / 2026-04-11

## Phase: Canonical Filename System — Full Implementation
**Commits**: `8f21f65` → `005c7a5` (10 commits)

### Design (commit `1d6c896`)
- `CANONICAL-FILENAME-ARCHITECTURE.md` — full design document
- Format: `YYYYMMDDTHHMMSSZ_KIND_XXXX.ext`
- 12 core kinds: NOTE, BASE, TMPL, LINK, IMG, AUD, VID, ATT, CANVAS, DRAW, MARK, CLIP
- Auto-generate codes for unknown extensions
- Frontmatter contract: title, cid, kind, created, aliases
- Sidecar .meta.json for non-markdown files
- Wikilinks resolve by title/aliases, never by filename

### Core Engine (commits `1d6c896` → `24ecb45`)
- `file_kinds.rs` — 3-layer classification engine (extension → content heuristics → auto-generate)
- `canonical.rs` — filename generator, frontmatter injection, sidecar metadata, canonicalize commands
- `importers.rs` — `import_with_canonical` full pipeline
- `libraries.rs` — title-based wikilink resolution, canonical note creation, frontmatter rename

### Integration (commits `5a05ec2` → `6393c0b`)
- Search index uses frontmatter `title:` as display name (not file stem)
- Rename updates frontmatter title + aliases (file stays put)
- Reindex triggered after rename
- Template system preserves canonical frontmatter fields
- File tree shows `display_title` for canonical files
- Tab names extracted from frontmatter title

### 14-Agent Audit (commits `fd4823b` → `ebbfbac`)
- All 14 agents run: PA, AA, MA, SCA, RA, UXA, CQA, EA, LA, SIA, SA, DIA, CFS, OGA
- LA: full pass (all 15 locales clean)
- OGA: full pass (100% offline)
- Fixed: shared EXCLUDED_DIRS, alias duplication bug, regex → OnceLock, RTL arrow flip, orphan LIKE underscore escape, Arabic-normalized wikilinks, malformed frontmatter, depth limits, missing name index, ConstellationEditor sync

### Canonical as Default (commits `aa1adca` → `005c7a5`)
- `create_note` always generates canonical filenames (not opt-in)
- Auto-canonicalize ALL existing files on startup (`auto_canonicalize_all`)
- Import always canonical (toggle removed)
- `generateAutoTitle()` uses canonical format instead of CoNote pattern
- Schema v5: frontmatter titles, Arabic-normalized wikilinks, name index

### Semantic Search Threshold (commits `8f21f65`)
- Dynamic threshold: `top_score - 0.03` instead of fixed cutoff
- e5-small produces compressed similarity (0.73–0.88 range)
- "agriculture" went from 2180 → 31 results

### Safety Redesign (commits `1e170d7` → `adacfd6`)
Canonical filenames redesigned for user choice and safety:

**Library Modes:**
- `native` — Constellation-created, always canonical (mandatory)
- `canonical` — external, user chose "Adopt" (reversible)
- `compatible` — external, user chose "Keep Intact" (cid-only, non-destructive)

**Changes:**
- `LibraryInfo.canonical_mode` field added to all library constructions
- `CanonicalChoiceDialog.svelte` — appears when linking external folders
- `create_note` respects library mode (canonical vs human filename)
- `inject_cid_library` — non-destructive cid injection for compatible mode
- `de_canonicalize_library` — restore original filenames for users leaving
- Removed auto-canonicalize-all on startup (was destructive)
- Fixed auto-title: restores original title, never generates new names
- Fixed content replacement bug on title rename (handleTitleChange)
- i18n: choice dialog keys in all 15 locales

**Test Results:**
- Test 1 (new note in native library): PASS ✅
- Test 2 (link external folder, keep intact): PASS ✅
- Title clear restores original, cid preserves original date: PASS ✅

### Multilingual Natural Language Search Operators (commits `87f823c` → `79cf3e9`)
**A PKM first — no other system does this.**

Architecture (Excel/LibreOffice pattern):
- `canonicalizeSearchQuery()` — pre-processes input, replaces localized operators with English
- `hasAdvancedSyntaxMultilingual()` — detects operators in any language
- `getSearchOps()` — returns current locale's operator map from i18n
- Unicode-aware regex with `\p{L}` boundaries (not `\b`)
- English always works as fallback in any locale

Operators translated in all 15 locales: linksTo, linksFrom, mutual, mentions, orphans, linksBetween, linksAll, and, scope

Arabic terminology (user-refined):
- الربط إلى (links to), الربط من (links from)
- الروابط البينية (links between), جميع الروابط (links all)
- متبادل (mutual), يذكر (mentions), يتيم (orphans)

Syntax chips localized — show operators in user's language.
SearchHub + Sky View both wired with canonicalization.

### Multilingual Search — Bug Fixes (commits `af25ee2` → `5023637`)

- Localized search result badges per language (ع م خ ر د for Arabic)
- Arabic terminology refined: الربط إلى/من, الروابط البينية, جميع الروابط
- ROOT FIX: Windows keyboard drivers inject invisible bidi chars (U+200F RLM, U+061C ALM) at Arabic→bracket transitions. `stripInvisibleChars()` applied at input source in `handleInput()` — every downstream consumer gets clean text
- ROOT FIX: Wikilink autocomplete regex was English-only — canonicalize query before matching, so Arabic operators trigger `[[` autocomplete
- Disabled auto-bracket pairing in all search inputs (caused extra brackets in RTL). Editor keeps CM6 `closeBrackets()`
- Partial name matching: link operators fall back to LIKE when exact match fails
- Arabic normalization: أ→ا, ة→ه, ى→ي, strip diacritics for fuzzy matching

### Search Engine Testing + Fixes (commits `3d17059` → `ef1de51`)

**Test Results:**
- Test 1 (Universal search): PASS ✅
- Test 2 (Advanced syntax English): PASS ✅ (orphans optimized O(n²)→O(n), freeze fixed)
- Test 3 (Multilingual Arabic): PASS ✅ (chips reactive, bidi chars stripped)
- Test 4 (Sky View search): IN PROGRESS — badges + counts now match

**Fixes applied:**
- Orphans query O(n²)→O(n): pre-compute incoming links in HashSet, use temp table (30s→instant)
- CM6 freeze: try-catch on EditorView creation, fallback editor without livePreview
- Highlight term: strip operator syntax before passing to editor
- Chip locale reactivity: $derived.by reads $t for reactive dependency on locale
- Star*→Sky* rename: 206 occurrences across 25 files + 3 component files renamed
- All 2999 notes as graph nodes (not just 419 linked)
- collect_library_notes reads frontmatter title for canonical files
- Search index name stored ORIGINAL (not Arabic-normalized) — schema v6
- Dual-form search: queries both original AND normalized Arabic for titles/tags/wikilinks
- Counts match: Search Hub and Sky View show identical "396 from 181 notes"
- Sky View category breakdown dropdown: click count badge to see per-category hits

### Open Items
- Frontmatter parser unification (3 parsers → 1 shared) — tracked for future
- Vec batch streaming in canonicalize_execute for 100K+ file vaults — optimization
- De-canonicalize UI button in library settings — not yet wired
- 7 Cognitive typed-link search operators — IMPLEMENTED (commit f8065a4)

### Living Link System (commits `ec80db9` → `f8065a4`)

**Knowledge Formulation Philosophy:**
- Complete specification: docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md (+ .docx)
- 8 link properties: type, direction, annotation, weight, confidence, created, last_traversed, traversal_count
- 6 lifecycle stages: spark → birth → growth → maturity → dormancy → renewal/archival
- 5 Acts of Knowledge Creation: observation → connection → tension → synthesis → conviction
- CLAUDE.md updated with Knowledge Formulation + Living Link Architecture sections
- Help file + User Manual chapter created

**P0: Link Storage Foundation (commit `4ddcb83`):**
- `note_links` SQLite table with all 8 properties + indexes
- `extract_typed_links()` parses `[[type::target|annotation]]` syntax
- `constellation_link_stats` diagnostic command
- Verified: 19,062 links indexed across 2999 notes, 1,834 with annotations

**P1: Cognitive Search Operators (commit `f8065a4`):**
- 7 typed-link operators: supports, contradicts, causes, exemplifies, generalizes, derives-from, part-of
- Rust: TypedLinkFilter + note_links query in constellation_search
- TypeScript: parseSearchQuery recognizes all 7 operators
- Canonicalization: all 7 translated in 15 languages
- Syntax chips: 7 cognitive chips in SearchHub + Sky View
- Arabic: يدعم، يناقض، يسبب، يمثل، يعمم، مشتق من، جزء من

**P2: Traversal Tracking (commit `2fb8c82`):**
- `constellation_link_traverse`: records link click, increments count, updates weight via `1.0 + ln(1 + tc)`
- Dormancy auto-wake: dormant links revive on traversal
- Data preservation: weight/traversal/confidence survives re-index
- `constellation_link_dormant`: finds links idle 90+ days
- Wired: NoteEditor + BacklinksPanel pass source path on wikilink click
- Target name uses display title (not canonical filename)
- Verified: 2 links with weight > 1.0 after traversal testing
- Fixed: rename error silenced for non-existent files during title blur

---

## Phase: Canonical Filename Architecture
**Commit**: `1d6c896`

### New Files
- `src-tauri/src/file_kinds.rs` — Kind registry + 3-layer classification engine
- `src-tauri/src/canonical.rs` — YYYYMMDDTHHMMSSZ_KIND_XXXX.ext generator, frontmatter injection, sidecar metadata
- `docs/CANONICAL-FILENAME-ARCHITECTURE.md` — Full design document

### Architecture
Every file Constellation manages gets an immutable canonical filename (PK). Users never see it — they work with human titles resolved via frontmatter `title` + `aliases` index.

- **12 core kinds**: NOTE, BASE, TMPL, LINK, IMG, AUD, VID, ATT, CANVAS, DRAW, MARK, CLIP
- **Auto-generation**: Unknown extensions get auto-generated codes, persisted in per-universe `file_kinds.json`
- **Classification**: Layer 1 (extension), Layer 2 (markdown content heuristics), Layer 3 (auto-generate)
- **Frontmatter contract**: `title`, `cid`, `kind`, `created`, `aliases` — cid is immutable PK
- **Sidecar .meta.json**: For non-markdown files (images, audio, video, attachments)
- **Wikilinks**: Resolve by title/aliases, not filename — zero broken links on rename

### Tauri Commands
- `classify_file_cmd` — classify a file by content
- `generate_canonical_name` — generate canonical filename for new files
- `canonicalize_preview` — preview library canonicalization (no changes)
- `canonicalize_execute` — execute canonicalization (rename + enrich)

### Phase 2: Integration (commit `24ecb45`)

**Import Pipeline** (`importers.rs`):
- `import_with_canonical` — full pipeline: scan → classify → generate canonical → enrich frontmatter → write + sidecars
- Supports markdown/folder/obsidian/notion formats natively
- Legacy formats (enex, html, csv, txt) fall back to old pipeline
- Writes `.constellation/canonical` marker after import

**Wikilink Resolution** (`libraries.rs`):
- `find_note_by_title_or_alias` — 3-step resolution: filename stem → frontmatter `title:` → `aliases:`
- `has_title()` function checks frontmatter title field
- Enables `[[Human Title]]` to resolve to `YYYYMMDDTHHMMSSZ_NOTE_XXXX.md` files

**Note Creation** (`libraries.rs`):
- `create_note` detects canonical libraries (`.constellation/canonical` marker)
- Auto-generates canonical filename + full frontmatter (title, cid, kind, created)
- Legacy libraries keep human filenames (backward compatible)

### Status
- Compiles clean (zero new warnings from canonical system)
- All Tauri commands wired: classify_file_cmd, generate_canonical_name, canonicalize_preview, canonicalize_execute, import_with_canonical
- TypeScript bindings complete in `src/lib/importers/store.ts`

---

## Phase: Dynamic Semantic Threshold
**Commits**: `9d2a221` → `8f21f65`

### Problem
e5-small produces compressed cosine similarity scores (0.73–0.88 range for all notes). Fixed thresholds (0.3, 0.65) either return everything or still return too many.

### Solution — Dynamic Threshold
- Two-pass approach: compute all scores first, then apply `top_score - 0.03` as cutoff (minimum 0.75)
- This keeps only results within 3% of the best match — the top 20% of the model's effective discrimination range
- Debug logging: `[SEMANTIC] top=X, threshold=Y, candidates=Z` in stderr

### Results (query: "agriculture")
- Fixed 0.3 → 2180 results (all notes)
- Fixed 0.65 → 2180 results (all notes still above)
- Dynamic 0.05 → 222 results
- **Dynamic 0.03 → 31 results (Search Hub), 46 results (Sky View)** ✅

### Also in this phase
- Sky View now embeds query via `embedText()` for semantic search (was passing `null`)

---

## Phase: Sky View Full Search + Canvas Badges + Link Visualization + Auto-Brackets
**Commits**: `1dc859d` → `6e8a514`

### Sky View Search (full SearchHub replication)
- Wide search bar with ×reset, ⋯chips, count badge, ×close
- Search history dropdown, wikilink [[autocomplete
- Advanced syntax routing: #tags, property=value, link operators → constellationSearch; plain text → universalSearch
- Canvas badges: stacked vertically per node with note title, RTL-aware
- universalSearch for multi-type badges (T+C+P per node)
- setSearchExtendedMulti() for Map<string, Set<string>> types

### Canvas Link Visualization
- setSearchLinkHighlights() colors link lines by direction
- Green (incoming), Red (outgoing), Purple (bidirectional)
- Arrowheads: single for to/from, double-ended for all/mutual

### Multi-Color Editor Highlighting
- Custom StateField + StateEffect + 6 CSS classes
- Title (blue), Content (green), Tag (pink), Property (amber), Wikilink (light blue), Semantic (purple)
- Context-aware: classifies each match by position (title area, frontmatter, body, wikilink line, tag line)

### Auto-Bracket Pairing
- NotePane: conditional on appSettings.autoPairBrackets
- SearchHub + Sky View: [→[], [[→[[]], (→(), {→{}, "→"", '→''
- Double [[ places cursor between for wikilink entry

### New Library Button + Knowledge Hierarchy
- 4th toolbar button (book+ icon) with Create/Link dropdown
- Constellation Knowledge Hierarchy documented in CLAUDE.md + help file
- Search index rebuilt after library creation

### Return to Sky View
- starViewReturnPending button in tab bar
- Opens note with highlightTerm, return preserves SV state

### Post-Implementation 14-Agent Audit
- All 14 agents ran: 4 CRITICAL (fixed), 13 HIGH (fixed), 16 MEDIUM, 10 LOW
- Key fixes: orphan query, version file race, concurrent reindex, XSS, dead code, RTL

### Bugs Fixed
- SV crash: searchMatchIds prop missing default
- Tag search substring false positives (JSON-quoted LIKE)
- Schema v3 forced reindex
- Dashboard tag click panel not showing (onTagSelect=undefined fix)
- Startup crash: stale searchMode references
- Badge direction: detectDir(r.name) instead of dir="auto"
- fullPage sidebar auto-collapse $effect

---

## Phase: 14-Agent Audit + Search Integration Across All Functions
**Commit**: `4b70c67` — 14-agent audit: fix XSS, dead code, RTL, data integrity, search integration

### Audit System
- Created `docs/AUDIT-SYSTEM.md` — spec for 14-agent audit system
- Ran all 14 agents in parallel (11/14 completed, PA/AA/MA timed out)
- Consolidated report: 1 CRITICAL, 19 HIGH, 16 MEDIUM, 8 LOW

### Fixes applied
- **XSS**: All `{@html r.snippet}` in SearchHub now escaped via `highlightInText()`
- **Orphan detection**: LIKE now matches JSON-quoted `"name"` to avoid substring false positives
- **Embedding BLOB panic**: Validate blob length before `chunks_exact(4)` — skip malformed
- **Dead code**: ~250 lines removed from +layout.svelte (sidebar search functions, state, imports)
- **RTL**: `dir="auto"` added to SearchHub input, BacklinksPanel filter, TagsPanel filter
- **Keyboard nav**: Arrow Up/Down/Enter for SearchHub results with visual selection
- **Search integration**: Sky View, OrgChart, Sight, Quick Switcher all use `parseSearchQuery()` — same syntax everywhere (#tags, properties, link operators)
- **Filter inputs**: BacklinksPanel and TagsPanel now have filter inputs
- **Note save reindex**: `constellation_search_reindex` called on every save
- **Debounce**: QuickSwitcher 200ms → 300ms
- **ort version**: Pinned to 2.0.0-rc.12
- **Localization**: semanticSearch keys added to all 13 locale files
- **Tag click**: Now opens Search Hub with `#tag` query

### Deferred
- Hardcoded English strings in OrgChart/Sight (needs full i18n pass)
- `@xenova/transformers` removal (still used by Sky View semanticEngine.ts)

---

## Phase: Search Hub + Universal Categorized Search + Link Operators
**Commit**: `ea398c4` — Search Hub: full-screen universal categorized search + 6 link operators

### What was done (Phase 2)
1. **6 link-topology search operators** (Rust `search.rs` + JS `store.ts`):
   - `links to [[X]]` — backlinks: which notes link to X
   - `links from [[X]]` — outgoing: which notes does X link to
   - `mutual [[X]]` — bidirectional: notes linked both ways
   - `mentions [[X]]` — unlinked mentions: name in body without wikilink
   - `orphans` — notes with no incoming or outgoing links
   - `links between [[X]] and [[Y]]` — notes linking to both targets

2. **Universal categorized search** (new Rust command `constellation_search_universal`):
   - Single query searches everywhere simultaneously
   - Returns 5 categories: Titles, Contents, Tags, Properties, Wikilinks
   - Each category runs optimized SQL (FTS5 for text, LIKE for JSON columns)
   - Arabic normalization for FTS5, raw lowercase for JSON LIKE queries

3. **Search Hub** (new `SearchHub.svelte` component):
   - Full-screen overlay activated from dock search icon
   - Both sidebars collapse for maximum space
   - Categorized results with collapsible sections and count badges
   - Click result → opens in editor with term highlighted, "Return to Search Hub" button
   - Component stays alive (no remount) — returning preserves exact search state
   - Search history dropdown when field is empty
   - Search term highlighted in result names and snippets (tags in pink)

4. **Wikilink autocomplete in search** (`+layout.svelte`):
   - Type `links to [[` → dropdown of all notes appears
   - Type `[[NoteName#` → shows headings for that note
   - Type `[[NoteName|type:` → shows link types (related-to, supports, etc.)
   - Works for all 6 link operators

5. **Editor highlight fix** (`NotePane.svelte`):
   - Added `.cm-searchMatch` styles (were missing from NotePane)
   - Open search panel hidden via CSS to activate all-match highlighting
   - Every instance of search term now highlighted in note content

6. **Search bug fixes**:
   - Tag search: fixed bracket-format `tags: [a, b]` parsing + inline `#hashtag` extraction
   - Schema v2 forced reindex on startup
   - × button resets query but keeps search box open (Escape exits)
   - Match-type detection: title vs content in lexical search

---

## Phase: Semantic Search — Rust-Native ONNX (100% Offline)
**Commit**: `d622fc8` — Semantic search: Rust-native ONNX inference, 100% offline, 100 languages

### What was done
1. **Rust ONNX engine** (`src-tauri/src/embeddings.rs`):
   - `ort` crate (ONNX Runtime) + `tokenizers` crate (HuggingFace)
   - `EmbeddingEngine` struct with `Session` + `Tokenizer`, cached in Tauri state
   - Lazy loading: model loads on first embed call, not on startup
   - 4 Tauri commands: `init_embeddings`, `embed_text`, `embed_notes` (batch+force), `embedding_status`
   - Token truncation to 512 (model max), char-safe UTF-8 truncation for Arabic
   - Mean pooling + L2 normalization on ONNX output
   - e5 prefix: "query: " for search queries, "passage: " for documents

2. **Model bundled with app** (Git LFS):
   - `src-tauri/models/model.onnx` (~113MB) — multilingual-e5-small, 100 languages, 384-dim
   - `src-tauri/models/tokenizer.json` (~17MB) — HuggingFace tokenizer vocabulary
   - Tauri resource bundling: `"resources": ["models/*"]`
   - 100% offline — no internet, no CDN, no downloads at runtime

3. **Universal search integration**:
   - `UniversalSearchResponse` has 6th `semantic` field
   - `execute_universal_search` accepts `query_embedding: Option<&[f32]>`
   - SearchHub embeds query via Rust IPC, shows Semantic (purple S badge) results
   - Cross-lingual: English "agriculture" finds Arabic الزراعة notes

4. **Frontend**:
   - Settings → Features → 🧠 Semantic Search toggle
   - Background indexing: embeds all notes incrementally (skips existing)
   - Edited notes re-embedded automatically (`force: true`)
   - `searchEngineReady` changed from `let` to `$state` (was preventing $effect triggers)

5. **Multi-term search**:
   - Comma-separated queries: `,` (Latin) `،` (Arabic) `、` (CJK)
   - Rust splits terms, searches each category for all terms, deduplicates
   - Frontend highlights all terms in results and editor

6. **Editor highlighting fix**:
   - Added `.cm-searchMatch` styles to NotePane (were missing)
   - Search panel opened hidden (`display: none`) to activate all-match highlighting
   - Multi-term support via regex alternation pattern

### Bugs fixed
- UTF-8 char boundary panic when truncating Arabic text at byte 2000
- ONNX token sequence > 512 causing broadcast shape error
- Model path not found in dev mode (now checks multiple locations)
- `searchEngineReady` not reactive (was plain `let`, now `$state`)
- Indexing starting before search DB initialized

---

## Phase: Search UX Enhancements (earlier)
**Commit**: `214feda` — Search UX: accessible badges, pinned results, search history, highlight term

### What was done
1. **Match-type detection fix** (Rust `search.rs`):
   - `lexical_search()` now detects title vs content matches by checking if query appears in note name
   - `structured_search()` returns specific types: `tag`, `property`, `wikilink` (not generic `structured`)

2. **Accessible character badges** (replaces colored dots):
   - 14px rounded badges with localized single letters (e.g., Arabic: ع/م/د/خ/#/ر; English: T/C/S/P/#/W)
   - Colors preserved for sighted users, letters provide color-blind differentiation
   - All 15 locale files updated with `matchTitle`, `matchContent`, `matchSemantic`, `matchProperty`, `matchTag`, `matchWikilink` keys

3. **Pinned search results**:
   - Clicking a result opens the note but keeps the result list visible
   - Active note highlighted with existing `.s-result.active` class
   - Keyboard navigation: Arrow Up/Down to select (with visual highlight), Enter to open, Escape to clear
   - Users clear search explicitly with × button or Escape

4. **Search term highlighting in content**:
   - Search query passed as `highlightTerm` to `openNoteTab()`
   - NotePane's existing CodeMirror SearchQuery infrastructure highlights all occurrences
   - Arabic-aware diacritic-insensitive matching already supported

5. **Search history**:
   - New `searchHistory.ts` module (localStorage, max 20 entries, deduplicated)
   - Dropdown shown when search field focused and empty
   - Each entry shows query + relative time (2m, 3h, 1d)
   - "Clear history" link at bottom
   - i18n keys: `searchHistory`, `clearHistory` in all 15 locales

### Files changed
- `src-tauri/src/search.rs` — match-type detection in lexical_search + structured_search
- `src/lib/libraries/searchHistory.ts` — NEW: localStorage search history module
- `src/routes/+layout.svelte` — badges, pinned results, keyboard nav, history dropdown, highlight term
- All 15 `src/lib/i18n/*.json` — 8 new sidebar keys each

### Tests
- `cargo check` — 0 errors (3 pre-existing warnings)
- `npx vite build` — clean build

### Open items
- Help files and user manual need updating for search UX changes
- ONNX Runtime semantic search migration (deferred)
- Map search integration (deferred)
