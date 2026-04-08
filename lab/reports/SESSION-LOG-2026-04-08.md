# Session Log — 2026-04-08

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
