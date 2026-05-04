# MIG-010 — Lexical Bridge Integration into Index Reads

**Date opened**: 2026-05-04
**Status**: Phase 1 (Architect) — Boss-approved
**Owner**: Index closure work stream
**Composes with**: MIG-006 (wikilink cascade), MIG-008 (create-dialog), the M11-data v2 corpus, M13 "via {lemma}" badges

---

## §1 · Goal

When the user clicks a term in the Index panel, today they see only notes that literally contain that exact surface. With the Lexical Bridge already shipped (M11-data v2 = ~20K concepts × 15 langs, baked into the binary; M13 badges working in general search), the Index should optionally surface cross-language equivalents of the clicked term — the same way general search does — so a click on "معرفة" can also reveal English notes containing "knowledge" / "cognition", with a "via knowledge" badge per row indicating *why* a cross-language hit appeared.

**Boss's constraint (approved 2026-05-04)**:
- **Exact match by default.** Today's behaviour is preserved on first install; no surprise in the Index for users who want it strictly literal.
- **Toggle in Settings**, not per-Index-panel. A single global preference: "Expand Index mentions cross-language via the Lexical Bridge." Off by default. When on, the Index mentions list pulls bridge terms in.
- **"via {lemma}" badge** on cross-language rows so the user can always tell which mentions are direct vs. bridged.

---

## §2 · Mapping the territory

### Current Index read paths
- `read_term_mentions(app, term, limit)` (libraries.rs:3393) — builds a phrase-quoted FTS5 MATCH (`"term"`), queries `notes_fts JOIN note_meta`, returns `Vec<IndexMention { note_path, note_name, snippet }>`. **No expansion.**
- `read_index_entries(app)` (libraries.rs:3319) — returns the vocabulary list from `notes_vocab`. Untouched by this MIG.
- `read_cooccurring_terms(app, term, …)` (libraries.rs:3481) — chip-strip data. Untouched (and just unblocked in commit `4a45b10`).

### Current Bridge wiring (search-side, M12 + M13)
- `expanded_match_query(normalized: &str) -> Option<LexicalExpansion>` (search.rs:2662) — **private** to search.rs. Walks the lexicon, returns the OR-joined MATCH expression + the lowercased non-source-language lemmas for badge rendering.
- `LexicalExpansion { match_expr, bridge_terms_lower }` struct — **private** to search.rs.
- `find_bridge_lemma_in_snippet` helper (search.rs ~2689) — scans a returned snippet for the first `<mark>…</mark>` whose contents matches a bridge term, returns the lemma to render. Already exposed via the search row return type.
- `lexical_search` wires all of the above into general search results.

### Current Settings shape
- App-level settings live in `app_settings.json` and propagate via `notifySettingsChanged()` (CLAUDE.md "Cross-window sync"). The Settings UI (`SettingsModal.svelte`) renders the editing form. Existing toggles include `interfaceFont`, `textFont`, `monoFont`, `fontSize`, `scriptFonts`, etc. There is no current category that fits "Index behaviour" — closest is the planned "Links" tab Boss greenlit (memory: `project_links_settings_tab.md`).

---

## §3 · Design options

### How to expose the expansion to `read_term_mentions`?

| Option | Shape | Speed | Risk |
|---|---|---|---|
| **3.A** Add `expand_cross_language: Option<bool>` parameter to existing `read_term_mentions`. Branch inside the function. | Smallest IPC surface change. One extra optional param; old callers unaffected. | Fast | Low — additive only. Default `None` ↔ today's exact behaviour. |
| **3.B** New IPC `read_term_mentions_expanded`. Two functions, two contracts. | Doubles surface area. | Slower | Medium — duplicate query construction; two places to maintain snippet shape. |
| **3.C** Settings stored in app state on Rust side; `read_term_mentions` reads it on every call. | Frontend doesn't pass the flag. | Slow + leaky | High — couples a read IPC to a settings store; harder to test; hides intent. |

**Decision: 3.A.** Additive parameter; frontend reads the Settings toggle and passes it explicitly. The IPC stays declarative ("here's what I want") rather than implicitly reading shared state.

### How to expose `LexicalExpansion` + `find_bridge_lemma_in_snippet` to libraries.rs?

| Option | Shape | Speed | Risk |
|---|---|---|---|
| **3.D** Promote `expanded_match_query` and helpers from `fn` (private) to `pub(crate) fn`. libraries.rs calls them directly. | Minimal code churn. | Fastest | Low — same semantics, just visibility change. The struct + fn already have stable signatures the search-side test suite covers. |
| **3.E** Extract a new `crate::search::bridge` submodule that owns the public surface; refactor search.rs to use it. | Cleaner long-term. | Slower | Medium — touches search.rs in more places; risk of regressing the M13 badge path. |
| **3.F** Duplicate the call logic inside libraries.rs (use `crate::lexicon::expand` directly). | Zero coupling to search.rs internals. | Medium | Medium — drift risk: if M13 ever changes its filter rules (e.g. "drop In-language synonyms"), the Index path won't follow. |

**Decision: 3.D.** Visibility bump only. The function is small, stable, well-tested, and its semantics are exactly what the Index needs. If a future MIG cleans up search.rs structure, the bridge helpers can move into a submodule then — no premature factoring now.

### How to render badges in IndexPanel?

| Option | Shape | UX |
|---|---|---|
| **3.G** Each mention row carries an optional `via_lemma: Option<String>`. Render a small chip after the note name when present: `Lunch Plan · via knowledge`. | Mirrors the search-side M13 pattern user already knows. |
| **3.H** Group mentions into "Direct" + "Cross-language" sections. | More UI; option Boss explicitly rejected (γ in the design poll). |
| **3.I** Mark cross-language rows with a colored stripe / icon, no text label. | Less informative; users can't tell which lemma bridged. |

**Decision: 3.G.** Match M13's existing pattern verbatim for consistency. Same component if the search-side renderer is already a snippet/component reusable in the Index context.

### Where does the Settings toggle live?

| Option | Placement | Cost |
|---|---|---|
| **3.J** New "Index" tab in Settings. | Future-proofs for more Index settings (term exclusion list could move here too). | One new tab. |
| **3.K** Under existing "Search" / "Display" tab. | Lower friction. | Less discoverable. |
| **3.L** Bundle with the Boss-greenlit "Links" tab. | Conceptually adjacent (both are about cross-language / cross-reference behaviour). | Couples MIG-007 (Links tab) to MIG-010. |

**Decision: 3.J — new "Index" tab.** Rationale: (a) the term-exclusion list (today persisted only in localStorage per IndexPanel.svelte) really belongs in Settings too, and (b) the planned Links tab (project_links_settings_tab.md) is for *link* concerns, not *vocabulary/index* concerns — they're orthogonal. Spawning the Index tab now sets up the destination for the term-exclusion list to move into in a later MIG.

---

## §4 · Invariants that must not break

| # | Invariant | How verified |
|---|---|---|
| **I1** | Default install behaviour unchanged: `read_term_mentions` with no expansion flag returns the same rows as today. | Existing libraries.rs callers + a new test asserting parity with the no-flag path. |
| **I2** | Settings toggle persists across restart, propagates to second screen via `notifySettingsChanged()`. | Manual test: set toggle, restart, verify still on; open second screen, verify visible (settings UI doesn't apply but the value is reachable). |
| **I3** | Cross-language expansion only fires when (a) the toggle is on AND (b) `expanded_match_query` actually returns Some (i.e. the term is in the corpus and produces a true OR-joined expansion, not a degenerate single-term). | Test: toggle on + out-of-corpus term ("Xzyqwop") returns exact-only mentions, no badges. |
| **I4** | Badges use the same source-of-truth filter as M13 search (non-source-language lemmas only, lowercased). | Reuse `bridge_terms_lower` directly; no parallel filter in libraries.rs. |
| **I5** | Snippet HTML safety preserved: STX/ETX sentinels (not `<mark>`) per the existing comment at libraries.rs:3425, so user content can't inject DOM. | Visual diff vs. today's mention rendering; the Rust query unchanged in the no-bridge path. |
| **I6** | Cooccurrence chip-strip (`read_cooccurring_terms`) is unaffected. | Manual: toggle on/off; chips render same. |
| **I7** | i18n complete in 15 locales for new Settings label, toggle description, and badge tooltip. | Grep `src/lib/i18n/*.json` for new keys after Build. |
| **I8** | RTL works (Arabic / Hebrew / Persian / Urdu): badge layout flows correctly with `dir="auto"`. | Boss visual check on Arabic interface. |
| **I9** | Performance: toggle-on path is O(same FTS query) — no extra round-trip. The Bridge expansion happens at query-build time before the SQL fires. | No new IPC calls per mention; budget unchanged. |
| **I10** | Rule 8 compliance: no recompute work on toggle flip. The next click on an Index term re-issues `read_term_mentions` with the new flag — read-time decision only, no rebuild. | Explicit: toggle does not trigger any cache invalidation. |
| **I11** | "via {lemma}" badge text uses the actual matched bridge lemma (from snippet scan), not just any bridge term in the expansion set. | Reuse `find_bridge_lemma_in_snippet`; do not invent a new matcher. |

---

## §5 · Phased plan preview

Full plan in `MIG-010-INDEX-LEXICAL-BRIDGE-PLAN.md` (Phase 2). Sketch:

1. **Build.1** — Promote `expanded_match_query`, `LexicalExpansion`, `find_bridge_lemma_in_snippet` to `pub(crate)` in search.rs. Zero behaviour change. Tests still pass.
2. **Build.2** — Extend `read_term_mentions`: add `expand_cross_language: Option<bool>` param; when true, build expanded MATCH expression; extend `IndexMention` with `via_lemma: Option<String>`; populate via snippet scan.
3. **Build.3** — Add Settings: new "Index" tab in `SettingsModal.svelte`; new key `indexExpandCrossLanguage: bool` (default false) in app settings; i18n keys for label/description/badge in 15 locales.
4. **Build.4** — Wire IndexPanel: read setting on mount, pass to `readTermMentions`, render `via_lemma` badge on each mention row.
5. **Build.5** — `/simplify` checkpoint: three review agents on the diff.
6. **Audit** — Phase 4: invariant verification + drift + migration path.

Each step lands as one commit, each with a verification clause.

---

## §6 · Open questions

**None — Boss-approved 2026-05-04**:
- Default behaviour: **exact** (β).
- Toggle location: **Settings** (global, not per-panel).
- New tab: **"Index"** (per §3 decision 3.J — informed inference; if Boss wants it bundled elsewhere, surface during Build.3).

---

## §7 · Risk register

| Risk | Likelihood | Mitigation |
|---|---|---|
| Visibility bump on `expanded_match_query` causes a downstream caller to grow that bypasses the test boundary | Low | The fn signature doesn't change; only the visibility keyword. Search-side tests still cover it. |
| Snippet scan for bridge lemma is per-row work; on a high-frequency term (500+ mentions) this could add latency | Low | The scan is one regex per snippet. M13 already runs it on every search result without complaint. Same rate. |
| Adding a new Settings tab requires re-laying out `SettingsModal.svelte`'s tab list — risk of breaking existing tabs visually | Low-Medium | Build.3 runs through Boss visual check before merge. If pattern is established (each tab is a snippet), additive only. |
| The "Index" tab created here pre-empts a future MIG that consolidates Index settings (term exclusion, etc.) — wasted work if that MIG decides differently | Low | The tab created here is intentionally minimal (one toggle). Future MIG can add to it without re-architecting. |

---

**Phase 1 closes here. Proceed to Phase 2 (Plan).**
