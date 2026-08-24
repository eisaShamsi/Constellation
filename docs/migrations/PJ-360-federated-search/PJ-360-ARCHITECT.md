<!--
PJ-360 Architect document — produced 2026-08-23 by a multi-agent Architect workflow
(4 parallel source mappings -> prior-art research -> 3 deliberately conflicting designs ->
an adversarial attack on each -> this synthesis). Phase 1 of the /migration workflow.

STATUS: NOT APPROVED. No code has been written against it. It exists so the Boss can approve
or reject a DIRECTION. Every code claim was read at source; unverified items are marked.

The Boss ruled YES on 2026-08-23 to "do you expect to start working across Linked Universes
soon?", which selected federated search as the next work ahead of everything else.
-->

# PJ-360 — ARCHITECT

**Federated search: making the Search Hub reach the whole federation, and say so when it cannot.**

Status: Architect (Phase 1 of `/migration`). No code written. Every code claim below was read at source in this session; measurement claims are attributed to the PJ-360 cost investigation and marked where I did not re-run them.

---

## 1. THE CONCEPT — the horse

**Federated search exists so that when you ask a question of your knowledge, the answer is drawn from everything you have connected — and when some part of it cannot be reached, you are told so rather than handed a shorter list.**

That is the whole purpose. Search in Constellation is not a file finder; it is the instrument you point at your own intellectual life. A Linked Universe is not a folder you happen to be able to view — it is part of the knowledge you decided to connect. So a search that stops at the universe boundary is not "scoped," it is **incomplete**. And a search that stops at the boundary *silently* is worse than incomplete: it is a wrong answer that looks like a right one. You cannot tell the difference between "there is nothing there" and "I did not look."

Two obligations follow, and they are separable. **Honesty** — the search must state where it looked. **Completeness** — it must look everywhere you connected. Honesty is the floor; completeness is the target. This document argues they should be delivered in that order, because honesty is cheap, certain, and immediately valuable, while completeness is expensive in exactly one specific place and needs measurement that does not exist yet.

---

## 2. WHAT IS TRUE TODAY

### 2.1 The everyday search box does not federate at all

When you type plain words into Search Hub, `SearchHub.svelte:166` takes the non-advanced branch → `universalSearch(q, qEmbed, 200)` (`:205`) → `constellation_search_universal` (`search.rs:13871`) → `execute_universal_search` (`search.rs:13922`).

The command **does** receive `app: tauri::AppHandle`. The inner function does not — its signature is `(conn: &Connection, query, query_embedding, limit)`. It is structurally incapable of reaching `state.federated_conn`. All six of its categories run unqualified SQL against the active universe's connection:

| Category | fn | line |
|---|---|---|
| titles | `search_titles` | `search.rs:13973` |
| contents | `search_contents` | `search.rs:14006` |
| tags | `search_tags` | `search.rs:14078` |
| properties | `search_properties` | `search.rs:14107` |
| wikilinks | `search_wikilinks` | `search.rs:14134` |
| semantic | `semantic_search` | `search.rs:13701` |

Every one of them ends `Err(_) => return Vec::new()`. **A failure and "no matches" are the same value.**

### 2.2 The advanced path federates one third of one mode

`parseSearchQuery` (`store.ts:4817`) picks the mode: `hasQuery && hasFilters ? 'hybrid' : hasQuery ? 'lexical' : 'structured'`.

- `"lexical"` (`search.rs:13564`) → `federated_lexical_search_or_fallback` — **federated**.
- `"structured"` (`:13582`) → `structured_search(conn, …)` — active-only.
- `"semantic"` (`:13630`) — active-only, and unreachable from the parser anyway.
- `"hybrid"` (`:13636`) — lexical half federated, semantic and structured halves not.

The in-code admission is at `search.rs:13644`: *"Semantic + structured stay active-only in v1 … documented gap."*

So `#tag`, `property=value`, `links to [[X]]`, `in:Library`, `orphans` and semantic all stop at the boundary. And `in:LibraryName` is worse than stopped — the picker offers you a Linked Universe's library name, the filter lands in `structured_search`'s `library_name IN (…)` against `main` only, and returns **empty**.

`hybrid` is the ugliest shape: `rrf_fuse` merges a federation-wide lexical ranking with an active-only semantic one, then appends active-only structured rows. The same note can be simultaneously visible (found by words) and invisible (cannot satisfy a filter).

### 2.3 The federated machinery is built, works, and is well-designed — it just isn't wired to the front door

`federated_lexical_search_or_fallback` (`search.rs:9358`) scatters one single-schema query per attached universe on one warm read-only connection and merges by **Reciprocal Rank Fusion, k=60** (`:9450`). The code cites its own prior art correctly: this is the Lucene `MultiSearcher` / Elasticsearch cross-cluster merge, and RRF is used because raw BM25 scores from two separately-maintained indexes are not comparable — only ranks are.

It also already solved the single most non-obvious constraint in this area: FTS5's `bm25()` and `snippet()` **cannot take a schema-qualified table**, so federation must be N single-schema queries, never a `UNION ALL`. Four unit tests pin this (`search.rs:15996, 16017, 16048, 16073`).

The only caller in the whole app that reaches it from plain text is `CatalogerView.svelte:104`, which hand-builds `{mode:'lexical'}`. **The Cataloger's note picker is federated. The Search Hub is not.**

### 2.4 A universe linked mid-session is never searched, even by the path that does federate

`attach_all` has exactly one caller (`search.rs:12170`), inside a background thread on the slow path of `ensure_search_db_ready`, which is gated shut for the rest of the activation. Its only re-arm is `invalidate_search_state` (`search.rs:11537`), which has exactly one production caller: `set_active_universe` (`universe.rs:1184`).

`add_child_universe` (`universe.rs:1590`) writes `universe.json` and calls **only** `invalidate_libraries_cache()`. `remove_child_universe` (`:1630`) is identical. So:

- **Link a universe today → it is not searched until you restart** (re-selecting the same universe is a no-op by the idempotency guard at `universe.rs:1005`).
- **Unlink a universe today → it stays attached and keeps returning results for the rest of the session.**

### 2.5 The app has one honesty channel, and it is in the wrong place and half-deaf

`federation_get_warnings` (`federation/mod.rs:69`) → a status-bar triangle badge (`+layout.svelte:10597`). It fires only when a universe **fails to attach**. It is polled at boot and **once more at +3 s**, on a comment assuming attach takes "tens-to-low-hundreds ms." The `federation:ready` handler (`+layout.svelte:3601-3658`) re-fetches sky, graph, core, links, five-acts and bases — and **never re-fetches warnings**. On the owner's parent universe, `init_db` alone runs ~15 s before attach begins, so the +3 s poll routinely completes before any warning exists.

Three failure shapes never produce a warning at all:
- **A Linked Universe whose folder has moved or been deleted** is dropped at `universe.rs:641-644` (`fs::canonicalize` → `continue`) *before* the warning layer exists. **This is the live state on the owner's machine**: `كون عيسى` declares a child at `E:\Constellation Universes\Two universe UNIVERSE\…` which does not exist on disk.
- **A universe linked mid-session** (§2.4).
- **A universe that attaches cleanly but has a stale schema.** `verify_schema` (`federation/attach.rs:49`) checks exactly five `note_meta` columns — `path, name, library_name, created_at, modified`. It never checks `notes_fts`, `note_links`, `note_embeddings`, `tag_counts`. Such a universe attaches, counts as ready, and every query against it returns an empty branch, swallowed to zero rows.

And `SearchResult` (`search.rs:1268`) carries **no universe identity** — only `library_name`, a name that can collide. Even on the path that *does* federate, you cannot see which universe a hit came from.

### 2.6 The help file already contradicts itself

`docs/help.uConstellation.World/Federation/Federation.md` line 8 (frontmatter) and line 32 both claim search spans all linked universes. Lines 69–82 of the same file carry an honest "A Known Limit" section saying it does not. Whatever ships, this file is wrong today in two places.

### 2.7 THE FINDING THAT REORDERS EVERYTHING — the index is already being contaminated, by the save path

This is not in the original defect statement and it is the most consequential thing in this document. Verified end to end at the backend this session:

1. `write_note` (`libraries.rs:1113`) authorises through `validate_path_in_any_library` — the **federated** resolver. A Linked Universe's note is writable.
2. `afterDurableSave` (`store.ts:4188`) **unconditionally** calls `reindexNote(savedPath, libraryName)` (`:4196`). No ownership branch.
3. `constellation_search_reindex` (`search.rs:12645`) → `reindex_single_note` (`:13113`) → writes into **`state.db`, the ACTIVE universe's index**. There is no `require_own_library` on this path. (That guard exists and is called from `canonical.rs:1483`, `libraries.rs:1183`, `:3271`, `tension.rs:99`, `sources/mod.rs:726` — just not here.)
4. `index_note` classifies with `active_universe_vocabulary()` (`search.rs:8172`) — the **parent's** vocabulary.

The codebase already documented this class and its non-recoverability. `libraries.rs:286`, verbatim: *"four write tails resolved their reindex library through `load_all_libraries`, so touching a linked universe's note filed it into THIS universe's index. Neither is recoverable by the boot reconcile: it skips every row under no owned root and counts it as `foreign_rows`."* The reconcile's own tests pin the non-healing: `reconcile.rs:1060` — *"and nothing here removes a linked universe's row"* — and `reconcile.rs:1081` asserts `foreign_rows: 621` is **not** a finding, so it never reaches you.

**Why this matters for PJ-360 specifically:** federated search merges results by path. If `main` already holds a shadow row for a Linked Universe note, that path appears in **two** branches, RRF accumulates both contributions (`search.rs:9463`), and the row kept is `main`'s — stale content, the wrong universe's name, the parent's vocabulary. Delete the note in its own universe and the parent's shadow row survives forever, so search returns a result that opens nothing. And PJ-360's entire purpose — making Linked Universe notes findable — makes them more openable and more editable, which **accelerates the contamination it would be built on top of.**

**The good news, and it changes the fix from a question into a chore:** the correct pattern already shipped. MIG-111 B6 routed the *rename* tail by owner — `libraries.rs:2092`, in-code: *"the first production WriteScope wiring. A note in a LINKED UNIVERSE does its rename bookkeeping in THAT universe's own database with THAT universe's vocabulary (Boss ruling 4)."* The save tail is the same concern at a different surface, left inconsistent. Under the Whole-Ecosystem Fix Law that is a missed surface, not a new design.

*Verified: every backend link in the chain. Not verified: I did not drive the UI to watch a Linked Universe note open into an editable tab. The write is accepted by the backend and the save tail reindexes it unguarded; the user-facing trigger is inferred from `afterDurableSave`'s unconditional call site.*

---

## 3. THE OPTIONS

### Option 1 — "Say what you searched." (Honesty only)

Search keeps reaching exactly as far as it reaches today, but stops pretending. Every search returns, alongside its results, a short statement of coverage: which universes were searched, which were not, and why — *"Searched: this universe. Not searched: Cognitive Knowledge (linked in this session — available after restart)."* The empty state stops saying "No matching notes" when the truth is "I did not look there."

**What it costs you:** nothing in speed. Nothing gets slower; nothing new is queried.
**Effort:** small-to-medium. Mostly plumbing plus 15 locale files.
**Risk:** low. It touches no query, no write path, no index.
**What you get:** the "shown a shorter list" half of the defect closed completely, permanently, for *every* search form including the ones that will never federate cheaply.
**What you don't get:** your linked universes' notes still do not appear in search results.

### Option 2 — "Search everything, everywhere." (Full federation, one migration)

Every search form — words, tags, properties, wikilinks, filters, semantic — spans every linked universe.

**What it costs you: speed, in one specific and measurable way.** Three of the six categories (`tags`, `properties`, `wikilinks`) scan a wide JSON column across the whole corpus. On your machine — which is the decisive fact, and is not written down in any prior design document — every universe and this repo live on a **mechanical USB drive**, on a machine measured with **0.6 GB of free RAM**. The PJ-360 cost investigation measured those three filters, against your live databases, at **2.6 to 14.8 seconds per universe, cold**. Federating them multiplies that by the number of universes, on one serialized connection, behind a 300 ms debounce with **no backend cancellation** — every abandoned keystroke pays the full cost across every branch. *(I did not re-run those measurements; they are attributed to that pass.)*

**Effort:** large. **Risk:** high, and it lands on the thing you notice most.
**It also inherits §2.7 wholesale** — federating a merge that keys on path, over a `main` schema that already holds shadow copies, produces double-ranked and phantom results.

### Option 3 — Honest first; the cheap half of completeness second; the expensive half third, behind measurement. (Phased)

Ship Option 1. Then federate only the two categories that are genuinely cheap — **title matches and body-word matches** — which is what most searches actually are. Then treat the expensive categories as their own migration, gated on a shape fix and on measurement that does not exist yet.

The reason this is not a fudge: **the two cheap categories are cheap by construction, not by luck.**
- `search_titles` (`search.rs:13973`) is index-only `bm25` plus a narrow four-column join. It never touches `body_text`.
- `search_contents` (`search.rs:14006`) was already rewritten (MIG-093 §D-2) to rank index-only first, then fetch **only the ≤ limit winners** — cost bounded by the limit, never by the match count.
- Both already use the unqualified-`bm25`, single-schema shape the four `option_c_*` tests prove correct for attached schemas.
- Neither reads link vocabulary, so the MIG-111 vocabulary-routing invariant is untouched and the vocabulary census stays where it is.

**Effort:** medium, spread. **Risk:** low per phase, and each phase is independently valuable and independently abandonable.

---

## 4. THE INVARIANTS

| # | Invariant | Where it lives | Which option threatens it |
|---|---|---|---|
| **I1** | **A universe's rows must be classified with that universe's own vocabulary.** | `link_types.rs:748` `registry_for_schema`; the whole of MIG-111 | **Option 2.** `structured_search` reads the *ambient active* vocabulary mid-filter (`search.rs:9855`). Federated as written, it would classify a Linked Universe's typed links under the parent's structural set — LL-047's exact shape: counts right, rows wrong. **Also threatened today by §2.7**, and that half cannot be fixed from the read side — a per-schema registry cannot repair a row that is in the wrong schema. |
| **I2** | **Scores from two databases are not comparable; only ranks are.** | `search.rs:9344`; RRF k=60 | **Option 2**, if it merges by score. Note the genuinely good news the investigation surfaced: `tags`, `properties`, `wikilinks` and `structured` all emit `score = 1.0` and `ORDER BY modified DESC` — so those four merge **exactly** on `modified`, an absolute integer, with no normalisation needed. RRF is required only for the two BM25 categories. |
| **I3** | **Reading everywhere must never become writing anywhere.** | `federation/write_scope.rs:43`; attachments are `?mode=ro` | **Broken today** on the save tail (§2.7). Not introduced by any option; inherited by Options 2 and 3-Phase-2. |
| **I4** | **A branch that fails must say so, never return an empty list.** | violated at `search.rs:9155`, `13977`, `14003`, `14082`, `14109`, `14136` | **Every option must fix this.** It is the single change that separates "federated" from "federated and honest," and it is also the only mechanism that can distinguish a stale-schema universe (§2.5) from one with genuinely no matches. |
| **I5** | **No boot, typing or IPC regression — measured on 7,600+ notes.** | CLAUDE.md Rule 8; `lab/boot-perf/BOOT-BUDGET.md` | **Option 2 severely.** And there is no headroom: the last recorded boot on the big universe was `hydrated_ms` **11,059** against a 6 s gate — already failing before federation does anything. Worse, **there is no search-latency instrumentation in the app at all**, so Rule 8's "measure before/after" is currently unenforceable for search. |
| **I6** | **The parent must work fully when a child is missing or offline.** | MIG-056 §3.2/§3.4 | All options. Note the trap: `resolve_child_universe_roots_recursive_strict` returns `Err` for the **whole tree** if any one universe is unreadable (permission denied, sleeping drive). One offline universe must not become "no results." |
| **I7** | **Results cap 200 per response.** | `docs/IPC-CONTRACT.md:15` | **Violated today.** `GraphMindView.svelte:625` and `:644` pass `limit: 0`, which both commands expand to **100,000** (`search.rs:13560`, `:13897`). And `execute_universal_search` truncates **per category**, so one response can already carry ~1,200 rows. Behind that sits `dedup_results` (`search.rs:13908`), which is **O(n²)** — it calls `deduped.retain(…)` for every accepted row. Federation multiplies *n* by the branch count and the cost by its square. **This is a prerequisite, not garnish.** |
| **I8** | **15 locales, RTL, and "Linked Universe" — never "cUniverse."** | i18n parity test; CLAUDE.md naming ruling | Every option that adds a string. The existing federation badge still renders `federation.cuniverseLabel` → "cUniverse" in all 15 locales (PJ-331). A new coverage line next to the old badge would put two names for one thing on one screen. |

---

## 5. RECOMMENDATION

**Take Option 3, phased — and do one thing before it.**

### Phase 0 (prior, and not really PJ-360): stop the index contamination.

Apply the MIG-111 B6 owner-routing pattern (`libraries.rs:2092`) to the save/reindex tail, and purge the existing foreign rows. This is not a new design and it is not a new ruling — the Boss already ruled that an operation on a Linked Universe's note does its bookkeeping in that universe's own database, and the rename tail already obeys it. The save tail is a surface the fix missed. Under the Whole-Ecosystem Fix Law it is owed regardless of PJ-360.

**Why it must come first:** every completeness claim PJ-360 makes is made over a `main` schema that is knowingly accumulating other universes' rows, and the boot reconcile is written never to heal them. Federating on top of that produces double-ranked results, wrong attribution, and results that open nothing — and PJ-360 itself accelerates the accumulation.

### Phase 1 — Honesty. Ship this whether or not anything else is ever built.

1. **Every branch returns a `Result`, and an error becomes a named skip** — never `Vec::new()`. (I4.)
2. **Coverage rides in the response**, not in a badge in another corner of the screen. This is the Elasticsearch `_clusters` / Solr `partialResults` pattern, and it is the one thing every mature federated system converged on independently. `UniversalSearchResponse` can carry a `coverage` field cheaply. `constellation_search` returns a bare `Vec<SearchResult>` and cannot — that is an envelope change across **8 verified call sites** (`store.ts:4120` wrapper, `CatalogerView.svelte:104` direct invoke, and 6 wrapper callers). Decide that shape here, not during the build.
3. **The coverage denominator comes from the federation manifest, not from what attached** — so a universe that was dropped before the warning layer existed (the live dangling-child case, §2.5) is still named. It must be recomputed when the federated set changes — hook it to `invalidate_libraries_cache()`, which is the one thing `add_child_universe` already calls. **Do not snapshot it inside `attach_all`**: that runs once at boot and would be blind to exactly the mid-session case it is meant to catch.
4. **Warning reasons become a typed enum, not free-text English.** They are currently `pub reason: String`, *"surfaced verbatim in the frontend popup"* (`federation/failure.rs:36`). A coverage line cannot be localized from an English sentence, and substring-matching Rust prose in Svelte is not acceptable against the full-localization standing order. One string set per warning kind, ×15.
5. **Re-fetch warnings on `federation:ready`** (`+layout.svelte:3601`) — today they are polled at boot and once at +3 s, and on this machine attach finishes long after that.
6. **`SearchResult` gains a universe field** — 9 construction sites — so a result can say where it came from, and so name collisions across universes stop being invisible.
7. **PJ-331's four naming keys ×15 fold in here**, or the screen shows two names for one concept.
8. **Search-latency instrumentation is part of this phase, not a follow-up** — per-command, per-branch, per-phase timings into `diagnostics.log`, reported as a **cold/warm pair**. Rule 8 already demands a before/after measurement on a large universe and there is currently no way to produce one. The cautionary tale is in the tree: `federation_prewarm` claims in two places that its FTS optimize is "0 ms on subsequent boots — idempotent," and the diagnostics log shows **73 runs, 12 of them over 10 seconds, up to 104 seconds**. The diagnostic that would have shown it was broken and nobody was reading it.

### Phase 2 — The cheap half of completeness.

Give `execute_universal_search` the `AppHandle` it lacks, and federate **`search_titles` and `search_contents` only**, per-schema, in the proven Option C shape. Plain-word searching then spans the whole federation on the everyday path, and the coverage line from Phase 1 carries the honest register for the five things that still stop at the boundary.

**Prerequisites inside this phase, not after it:** bound `limit` (kill the two `limit: 0` call sites and fix the per-category cap against the 200-row IPC contract), and fix `dedup_results`' O(n²). Federating an unbounded quadratic is how a fix becomes a freeze.

Also in this phase: **federate `constellation_search_link_counts`** (`search.rs:14302`, `main`-only, and never re-fetched on `federation:ready`). It is the tie-breaker in the advanced results sort, so without it every Linked Universe row ties at zero incoming links and sorts last — the feature would look half-broken. And close the **mid-session attach gap** (§2.4) with a re-attach door, carrying the full generation-check discipline at `search.rs:12220-12291` — those four checkpoints exist to close a race that was reproducible by double-clicking the universe picker, and a second builder that skips them re-opens it.

### Phase 3 — The expensive half. Its own migration, gated on measurement.

`tags`, `properties`, `wikilinks`, `structured`, `semantic`. Each needs work that Phase 2 does not:

- **A filter-shape rewrite before federation.** These three filters currently read wide rows off disk to answer a narrow question. The cost investigation measured a large improvement from filtering index-only — but the adversarial pass established that **the existing covering indexes do not actually cover these queries as written** (`idx_note_meta_tags` carries no `name`, no `library_name`, no `modified`), so the measured figure is an upper bound on a query the app does not issue. Delivering it needs **new, wider indexes**, whose write cost lands on the save path. That trade must be measured, not assumed. Two filters — `mentions` (which *is* a `body_text` scan) and `orphans` (a per-universe write-time count) — have no index-only form at all and should be honestly excluded and labelled.
- **`structured_search` needs vocabulary routing** (I1), anchor resolution across universes, and `in:Library` fixed.
- **`semantic_search` needs restructuring, not qualifying** — its threshold is corpus-relative (`max(0.75, top − 0.03)`, `search.rs:13741`), so candidate generation must be split from thresholding and the threshold applied after the merge. It also needs a `model_id`/`dimensions` guard before scattering: every universe currently uses `multilingual-e5-small` at 384 dimensions **by luck, not by construction** — nothing checks, and a same-dimension different model would produce silently meaningless similarity scores.
- **Fix the `hybrid` merge order first.** Today the lexical half is fetched at `limit × 2`, the fused list is returned untruncated, structured rows are appended *after* it, and the whole thing is truncated to 200 — so whenever free text yields ≥200 rows, **the filter half contributes nothing to what you see**. Federating `structured_search` before fixing that triples the cost of a stage whose output is currently discarded.

### What this recommendation trades away

**You will not get "search everything, everywhere" soon.** After Phase 2 your linked universes' notes will be findable by title and by words in the body — which is most of how anyone searches — but `#tag`, `property=`, `links to [[X]]`, `in:Library` and semantic will still stop at the boundary. The difference from today is that **the app will say so, by name, every time**, instead of showing you a shorter list.

I am trading completeness-now for speed-and-certainty. I think that is the right trade on this hardware, on this timeline, given that the boot budget is already failing with no headroom and there is no instrument in the app that could tell us if we made it worse. And I think the honest coverage line is worth more than it sounds: it converts a search you cannot trust into a search you can, immediately, for every search form — including the ones that may never federate cheaply.

**What I am explicitly rejecting**, and why:

- **A unified index** (one index holding copies of linked universes' content) — the enterprise default, and genuinely faster. Rejected because its documented downside *is* your own ruling: it requires the parent to hold and own a copy of a member's content. Federation is the correct architecture here and was chosen correctly; the gap is only that the front door was never connected to it.
- **Collection selection** (deciding which universes are worth querying) — the federated-IR literature's signature technique, and the most seductive-sounding fix. Every justification for it is a *network* cost. Your universes are ≤10 local files on an already-open connection. Importing it would rebuild the exact defect PJ-360 exists to remove: a heuristic that decides not to look somewhere. **Query every attached universe, always.**
- **A "Search these too" button** as the delivery vehicle for completeness. Its entire payload would be word matches in titles and bodies — which is precisely the two cheapest categories, i.e. exactly what Phase 2 gives you by default. A button would cost about the same, duplicate the active universe's own results inside its own budget, and use different matching rules than the list beside it.

**No PKM tool to copy.** Obsidian, Logseq and Zotero all scope search to one corpus and treat cross-corpus as a wish-list item. DEVONthink does search across open databases — with a **visible scope selector**, which is the same conclusion Elasticsearch reached from the other direction — but answers "can wikilinks resolve across databases?" with *"Short answer: no."* On cross-universe linking Constellation is not behind the field; it is attempting what the field declined to build. Which means the design cannot be validated by copying a PKM tool, and has to be validated against search-engine practice.

---

## 6. WHAT THIS DOES NOT COVER, AND WHAT MUST BE MEASURED BEFORE PHASE 3

**Not covered by any phase above** (each is real, verified, and owed separately):

1. **Living Links stop at the boundary and lie about it.** `constellation_link_traverse` (`search.rs:10039`) runs `SELECT … FROM note_links WHERE source_path = ?1` against `main`. Walk a link whose source is a Linked Universe note and the row is not there, nothing updates, and the command **returns success**. The link earns nothing, silently — while the Backlinks and Outgoing panels (which *do* federate) show that same link. Every earned link property is in the same position.
2. **Collections break for Linked Universe members** — `collections_hydrate` (`collections.rs:171`) reads `state.db` only; a linked member renders as "missing."
3. **The Index panel, Reviewer, Tensions, Strata, Maturity, Sight, Structure, Inspector 360, unlinked mentions** — all active-universe-only.
4. **Unlinking mid-session leaves a universe attached** for the rest of the session. Today that is invisible; **after Phase 1 the coverage line will positively assert it was searched**, which is honest about the machine and wrong to you. Must be closed alongside the mid-session link gap.
5. **The status bar already counts notes that search cannot find** (`aggregate_library_counts` federates). That inconsistency is visible today.

**Must be measured before Phase 3 can be designed** (all currently unknown):

- **The cost of an FTS5 `MATCH` query.** This is the single biggest hole. It cannot be measured outside the running app, because the custom tokenizer is connection-local. It is the part of the path that federates *today* and nobody has a number for it.
- **Query-embedding latency** and the first-use cost of loading the 118 MB ONNX model off the USB drive.
- **The write cost of the new indexes** Phase 3's shape fix requires, on the save path.
- **The real ATTACH ceiling.** The code's cap of 25 (`attach.rs:45`) is fiction: no `SQLITE_MAX_ATTACHED` flag is set anywhere in the repo, so the bundled default of **10** applies. Universe 11 fails inside SQLite — honestly, as a warning — but three documents encode the wrong number.
- **Memory under federation.** The idle budget is 350 MB; there is no measurement of a federated session, and the three JSON categories ship the *entire* JSON column as their snippet.
- **A federated test fixture at real scale.** The four `option_c_*` tests run against a **five-column** `note_meta` with **no custom tokenizer** — a fixture more privileged than the production caller, which is precisely the shape `LESSONS-LEARNED.md` records as letting 52 green tests miss a 100% write loss. They prove SQLite name resolution and nothing about Phase 3.

**Flagged, and I could not find it filed anywhere** — a correctness hazard, not a performance one: FTS5 stems the query symmetrically using the *process-global* Arabic override store, which holds the **active** universe's overrides (`libraries.rs:5263`, installed only by `set_active_universe`). A query run against a Linked Universe's index is therefore stemmed with the parent's overrides while that index was written with its own. Where the two differ, the note silently does not match. This is LL-047's shape in a second subsystem, it affects the *already-federated* lexical path today, and it should be filed regardless of which option is chosen.

---

## 7. THE ONE QUESTION

You have already ruled the queue: PJ-344, then PJ-367, then PJ-366, then PJ-360 — the order you will physically hit them.

Phase 1 above (honesty) fits that queue without disturbing it. **Phase 2 (making linked universes' notes actually appear in search) should not ship before the save-tail fix in Phase 0**, because it would be building completeness on an index that is quietly accumulating other universes' rows and can never clean them out.

**So: do I take the save-tail fix now, ahead of PJ-367 and PJ-366 — or file it and hold Phase 2 behind it while the queue runs as you set it?**

What I'd recommend, and why it is a real choice and not a formality: taking it now means one more thing between you and the search fix you asked for, but it is a small, already-proven change — the same owner-routing pattern MIG-111 shipped for renames, applied to the surface it missed — and every day it waits, more shadow rows land in your index that nothing can remove. Holding it means the queue stays as you ordered it and Phase 1 still ships honest, but Phase 2 waits on it either way.

Everything else in this document I can decide without you.