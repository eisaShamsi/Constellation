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
