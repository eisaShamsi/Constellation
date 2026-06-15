# 07 — Index Panel (Concept Paper)

> Per-function paper. Follows the template in [00-MASTER](00-MASTER-Bring-Up-Charter-and-Checklist.md) §3 and must trace back to [00-Constellation](00-Constellation-Core-Concept-Paper.md). The Index is the **canonical Rule 8 surface** — the one the core paper cites as proof that derived views are read, not recomputed.

## 1. Function in hand
The **Index panel** — `src/lib/components/IndexPanel.svelte` — the sidebar term-browser (ribbon "Index", `$t('ribbon.index')`). It is the alphabetised, frequency-rankable dictionary of every term in the active Universe, with per-term mentions, co-occurring terms, cross-language `via {lemma}` badges (MIG-011) and `≈ similar` semantic badges (MIG-013 §1D). Mounted in `src/routes/+layout.svelte` (~line 5955) inside the `index-overlay`.

## 2. Purpose
Show the user the **vocabulary of their own thinking** and let them pivot from a term to every note that uses it. It primarily serves **Connection** (the second Act): the panel surfaces which notes share a term, which terms co-occur, and which notes contain *all* of several selected terms (the commonality view). The cross-language bridge and the `≈ similar` semantic badge push that toward Connection across languages and concepts. It justifies its existence as the core paper's *diagnostic instrument* — a way to see the shape of one's corpus, not a file finder.

## 3. What it is NOT
- **Not** the search box (SearchHub / global search) — it browses the *vocabulary dictionary*, it does not run free-text queries against note bodies.
- **Not** a writer — it never modifies a note; it only reads the FTS5 index and opens notes through the Editor.
- **Not** a recompute engine — it reads the FTS5 `notes_vocab` view that triggers already maintain (§7); it never re-walks the Universe.
- **Not** the owner of the note it opens — the right-hand split mounts the shared `NoteEditor` (a display, not a domain — Working Agreement / "screens are displays").

## 4. Wiring
- **Inputs (props/stores):** `entries` (the `IndexEntry[]` from `readIndexEntries()`), `isLoading`, `activeNotePath`, `selectedTerms`; toggles `bridgeFilterEnabled` ← `$appSettings.index.expandCrossLanguage`, `searchHistoryEnabled` ← `$appSettings.index.searchHistoryEnabled`, `cacheKey` ← same expand flag (drops `mentionsCache` when it flips).
- **Inputs (IPC, lazy):** `read_index_entries` (full vocab, on first panel open); `read_term_mentions(term, 500, expandCrossLanguage)` on expand; `read_cooccurring_terms(term)` for the chip strip; `lexiconExpandForFilter` (MIG-011 bridge) and `ctseSearchTermsByConcept` (MIG-013 §1D semantic) per debounced keystroke; `readIndexHistory` / `writeIndexHistoryEntry` (MIG-012 history).
- **Outputs (events/writes):** no note writes. Callbacks bubble up: `onNoteClick` → opens the note in the index split; `onTermClick`/`onTermSelect` → mutate `indexSelectedTerms`; when the second screen is open, `emitIndexTermSelected` mirrors the selection. Persists the hidden-terms set to `localStorage` (`index-excluded-terms`) and search history via the history IPC.
- **Consumers:** the parent `+layout.svelte` (selection state, the index note split), the second screen (term-selection mirror), the clipboard (export).
- **Connection to the Editor (the gate):** the panel does **not** edit. Clicking a mention calls `onNoteClick`, which opens that note in a mounted `NoteEditor` in the index split — all save/load/edit stays in the Editor. The panel's data is downstream of edits: a note save fires the reindex, the FTS5 triggers update `notes_fts`/`notes_vocab`, and the next `read_index_entries` reflects it. No silent reads — the panel learns of changes only because the Editor fired the reindex.

## 5. Right-click / context menu
- **Has one.** Right-clicking a term row (`oncontextmenu={(e) => handleContextMenu(e, entry.term)}`, line 1257) opens the **shared `<ContextMenu>`** (line 1348), built by `getIndexTermMenuItems(term)` (MIG-077 A2). ✅ Shared, not hand-rolled — MIG-077 A2 explicitly converted this from the old inline + hardcoded-English menu.
- **Items (per term):** a single dynamic entry — **Hide term** (`$t('indexPanel.hideTerm')`) or **Show term** (`$t('indexPanel.showTerm')`) depending on whether the term is already in the excluded set.
- **Reachable only by right-click:** Hide/Show is the **only** right-click action and currently the only way to hide a term (un-hide is also reachable via the eye-toggle "show hidden" button). Verify in bring-up.
- **Gap to flag:** the menu is thin. A term row affords more (copy term, open all mentions, add to comparison set, copy as wikilink, export just this term) — all currently click-only or absent. Per the core paper's "right-click should include every aspect of the app," the Index term menu is a candidate to enrich during bring-up. Note rows in the mentions list have **no** context menu of their own (they use the bare `gp-ref` button) — flag whether they should offer the shared note menu.

## 6. Multilingual
- **Mostly localized.** Term strings flow through `$t('indexPanel.*')`; keys verified present in `src/lib/i18n/en.json` (line 1585) and `ar.json` (line 1518) — `hideTerm`, `showTerm`, `filterPlaceholder`, `viaLemma`, `semanticMatch`, etc. Full ×15 coverage **unverified — verify in bring-up** (only en + ar checked here).
- **RTL / script-aware by design:** the panel detects 8 scripts (Arabic, Hebrew, Latin, Cyrillic, Devanagari, Hangul, Kana, CJK), groups the alphabet bar per-script with per-row `dir`, sets the panel `dir` to `rtl`/`ltr`/`auto` by active script, ports the backend Arabic Light10 stemmer to JS for filter matching, and uses `dir="auto"` on every term/mention/chip span. This is strong, native multilingual handling.
- **Hardcoded English — FLAG:** two `title=` tooltips bypass `$t()` — the sort-toggle (`'Sort by frequency'` / `'Sort alphabetically'`, line 1117) and the export button (`'Copy to clipboard'`, line 1124). Also several visible strings use the `$t('key') || 'English fallback'` pattern (e.g. `recentSearches`, `comparing`, `clearAll`, `alsoAppearsWith`) — localized when the key exists, but the inline English fallback is a hardcoded-English smell to scrub. Export-markdown headers (`'# Index'`, `'Showing first N of …'`) are hardcoded English. **Fix before re-enable.**

## 7. Boot behavior
- **Runs at boot?** **No.** The load effect (`+layout.svelte` ~line 4229) is gated on `showIndex` — `read_index_entries` fires the *first time the user opens the panel*, not on boot. Subsequent opens hit cached `allIndexEntries`; a Universe switch invalidates via `indexLoadedKey`.
- **Rule 8 status:** ✅ **reads-persisted.** `read_index_entries` (libraries.rs ~line 3580) reads directly from `notes_vocab` — the `fts5vocab(notes_fts, 'row')` view FTS5 maintains on disk via the `note_meta_ai/ad/au` triggers. Mentions, co-occurrence, and `≈ similar` are all read-time `MATCH`/lookup queries against that already-current index. **Nothing is rebuilt on boot or on open.** This is the canonical Rule 8 surface the core paper cites.
- **Cost (measured, per code comment libraries.rs ~line 3578):** ~350 ms for ~50k vocab rows on a 7,600-note Arabic-heavy Universe — paid once, on-demand, off the boot path. Per-term mention/co-occurrence fetches are tens of ms (estimated). The filter loop is pure substring over the in-memory list; the bridge/semantic IPCs are debounced ≥300 ms with stale-token cancellation.

## 8. Flag / gate & bring-up position
- **Gate today:** `$appSettings.enabledFeatures?.index !== false` (`+layout.svelte` line 5245 dock button; the overlay is always rendered, hidden by CSS to preserve state). Sub-features behind their own settings flags: `index.expandCrossLanguage` (bridge), `index.searchHistoryEnabled` (history). No SIGHT flag — the Index is **not** part of the Sight/Map plug-in family.
- **Bring-up phase:** **2 (search/index layer)** — depends on the Editor (the gate, phase 1) being the thing that fires the reindex, and on the FTS5 chain (`notes_fts`/`notes_vocab` + triggers) being live. The bridge and semantic badges further depend on the M11 Lexical Bridge and the CTSE concept index (their own bring-up).

## 9. Budget
- **Boot budget:** **zero** — must not touch the boot path (it doesn't; load is on first open). Re-enable must keep it off boot.
- **Interaction budget:** filter keystroke instant (pure in-memory substring; no `invoke()` on the keystroke path — the bridge/semantic IPCs are debounced ≥300 ms and token-cancelled). The list is virtualized (`VirtualList`) so a 50k-term vocab renders only the visible window. Panel open ≤ the measured ~350 ms first-load, instant thereafter.
- **Regression guard:** open the panel on a 7,600-note Universe; type rapidly in the filter (no lag, no IPC storm); scroll the full vocab (virtualization holds); expand a term (mentions + co-occurrence load once, cached). Measure first-open before/after any change to `read_index_entries` or the FTS5 chain.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** browsing the vocab and pivoting term→notes works; commonality (2+ selected terms) returns the right intersection.
- [ ] **Serves Constellation's core purpose:** advances **Connection** — surfaces shared terms, co-occurrence, cross-language and conceptual neighbours (see [00-Constellation](00-Constellation-Core-Concept-Paper.md)).
- [ ] **Wires to the Editor:** clicking a mention opens it in a mounted `NoteEditor` (display, not domain — no re-implemented save/load); the panel never writes a note.
- [ ] **Right-click present + correct:** uses the shared `<ContextMenu>` (MIG-077 A2), not hand-rolled; Hide/Show works; the enrichment gap (§5) is triaged.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** verify all 15 locales carry the `indexPanel.*` keys; fix the two hardcoded `title=` tooltips and the `|| 'English'` fallbacks and the export-markdown headers; confirm per-script grouping + `dir` in RTL.
- [ ] **Within budget:** off the boot path; filter instant; list virtualized; no `invoke()` on the keystroke path.
- [ ] **Obeys Rule 8:** reads `notes_vocab`/FTS5; recomputes nothing on boot or open.
- [ ] **Holds its invariants:** hidden-terms set persists; cache invalidates on Universe switch and on the `expandCrossLanguage` toggle; stale bridge/semantic fetches are cancelled (no race-in result).
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **— (Rule 8 ✓ confirmed in code; perf + ×15 i18n to re-verify in bring-up)**
Notes: The Index is the textbook Rule 8 surface — pure FTS5-vocab read, no boot recompute, ~350 ms first-open on a 7,600-note Universe. Two debts to clear before re-enable: (1) thin right-click menu (only Hide/Show — enrich per the core paper's right-click mandate; mention rows have no menu); (2) hardcoded-English strings (two `title=` tooltips at lines 1117/1124, several `|| 'English'` fallbacks, export-markdown headers). Sub-elements folded here (no separate paper): the cross-language `via {lemma}` bridge (MIG-011), the `≈ similar` semantic badge (MIG-013 §1D), search history (MIG-012), co-occurrence chips, multi-term commonality, NSC summary headlines (MIG-044 P2), and the alphabet/script bar.
