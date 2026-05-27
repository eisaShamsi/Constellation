# Session log — 2026-05-27

**Function in hand:** MIG-056 §K.3 — v2 scatter-gather federated search.

## Context from 2026-05-26 carrying forward

Yesterday closed mid-§K Boss-test on MIG-056. Eisa's pasted diagnostic log nailed the §K Stage 4 failure: SQLite's FTS5 auxiliary functions (`bm25()`, `snippet()`) reject schema-qualified table references at PREPARE time, and the workaround (table aliases) failed for the same reason — the FTS5 internal pseudo-column is bound to the unqualified original table name and can't be schema-qualified or aliased.

Last night's §K.2 hotfix (working-tree, uncommitted as of session start) shipped a v1 functional workaround:
- Dropped `bm25()` + `snippet()` from the federated SQL entirely
- Used unqualified `notes_fts MATCH ?` in WHERE clauses (resolved against per-branch FROM scope)
- Ordered by `modified DESC` instead of relevance
- Synthesized snippets in Rust from `body_text`
- Also fixed the status-bar 1101 bug — `loadAllStats()` now re-fires in the 3s federation warning re-poll, so the count refreshes after federation attaches

Eisa retested §K.2.E with these results:
- ✅ Stage 2 passes: status bar shows **8751 notes** (1101 main + 7650 cu1 = exactly the sum)
- ✅ Stage 4 passes mechanically: `الرباط` returns 15 results from federated search, mostly cUniverse libraries
- ✅ ⚠ warning popup confirms `كون عيسى` cUniverse's `search.db` is missing (Stage 5 skip_unavailable behavior working as designed; Eisa just hasn't opened that universe yet so its index was never built)
- ⚠ But: Eisa flagged "I've got an irrelevant result." The top result was the most-recently-edited matching note ("مفهوم الاستراتيجية في الفكر العسكري الإسلامي المبكر"), not the most BM25-relevant note. That's the v1 ranking trade-off documented in Architect §7.2

Eisa's decision (this morning): **"Do v2 scatter-gather now."**

## §K.3 plan — v2 scatter-gather

Replace the §K.2 UNION-ALL-without-bm25 workaround with the architecturally correct scatter-gather pattern (Agent 2's Lucene CCS recommendation):

1. **Per-cUniverse standalone Connections** in SearchState (new `federated_search_conns: Mutex<Vec<(PathBuf, Connection)>>` field).
2. **Background thread** opens these Connections after `attach_all` succeeds, registers `constellation` tokenizer per Connection.
3. **`federated_lexical_search_or_fallback` rewritten as scatter-gather:**
   - Calls existing `lexical_search()` once per Connection (main state.db + each cUniverse Connection). Each call runs as single-schema — `bm25()` and `snippet()` work fine because there's only one `notes_fts` in scope per Connection.
   - Collects per-branch ranked Vec<SearchResult>.
   - **RRF merges** them in Rust: `score(d) = Σ 1/(k + rank_in_branch(d))`, k=60. Standard rank-fusion, no cross-corpus BM25 comparison needed.
4. **`build_federated_lexical_sql`** becomes dead code → removed along with its tests.
5. **`invalidate_search_state`** clears `federated_search_conns` (lifecycle).
6. **Generation counter check** (§J.1) extends to the new field — background thread re-checks generation before writing into state.

## §K.3 implementation — what landed (working tree, pre-commit)

### Code changes

**`src-tauri/src/search.rs`:**
- Added `federated_search_conns: Mutex<Vec<(PathBuf, Connection)>>` field to `SearchState` — the per-cUniverse standalone Connection pool. Doc-string explains the FTS5-aux-function constraint that motivates the design and how RRF avoids cross-corpus BM25 incomparability.
- Updated `SearchState::new()` + both `#[cfg(test)]` SearchState constructions (lines 6237, 6431) to include the new field.
- Background-attach thread now opens per-cUniverse Connections after `attach_all` succeeds, registers the `constellation` FTS5 tokenizer on each, populates `federated_search_conns`. Per-cUniverse failures (open or tokenizer registration) are non-fatal: logged + omitted from the pool. The §J.1 generation counter check still gates the write into state.
- `invalidate_search_state` now also clears `federated_search_conns` on universe switch (lifecycle parity with `federated_conn`).
- **Rewrote `federated_lexical_search_or_fallback`** — replaces the §G/§K.2 UNION-ALL-without-bm25 implementation with scatter-gather + RRF:
  - Scatter: runs existing `lexical_search()` once per Connection (1 main + N cUniverse). Each call is single-schema where bm25/snippet work normally.
  - Gather: RRF merge with k=60 (Cormack & Clarke / Elasticsearch CCS default). Each branch contributes 1/(60+rank) per doc; overlapping paths (theoretical — v1 universes don't overlap) sum contributions; sort by combined score DESC; truncate to outer LIMIT.
  - Diagnostic logging continues via `fed_diag` (re-tagged from `[mig-056-§K.2]` to `[mig-056-§K.3]`) so the boss-test verification can confirm scatter-gather engaged.
- **Deleted `build_federated_lexical_sql`** and the old `federated_lexical_search` function — dead code post-§K.3 (no more UNION ALL).
- **Replaced the `mig056_federated_search` test module** — old tests were SQL-shape assertions for the now-deleted builder. New tests exercise the RRF merge directly: empty branches, single branch passthrough, two-branch interleave, three-branch with strong rank-1s, overlapping-path accumulation, LIMIT truncation, k=60 head-softening math.

### Test verification
- 7/7 new RRF unit tests pass.
- 46/46 federation tests still pass (no regression).
- 84/84 lens tests still pass.
- 41/41 libraries tests still pass.
- `cargo check --lib` clean (42 pre-existing baseline warnings unchanged).

### NSIS build
Installer at `E:\مشاريع كلاود\Constellation\src-tauri\target\release\bundle\nsis\Constellation_0.1.0_x64-setup.MIG056-K3-scatter.exe`.

### BOSS-TESTS doc
Added `Stage 4.F — Retest after §K.3 v2 scatter-gather hotfix` to `docs/MIG-056-BOSS-TESTS.md`. Eisa retests by (1) installing the new build, (2) confirming status bar still ~8751 (no regression from §K.2), (3) searching `الرباط` and verifying the top result is now BM25-relevant not modified-DESC.

### Working-tree status
Uncommitted (pending Eisa's §K.3 retest pass): §K.2 + §K.3 changes in `src-tauri/src/search.rs`, `src/routes/+layout.svelte`, `docs/MIG-056-BOSS-TESTS.md`, and this session log.

If §K.3 retest passes, the commit sequence will be:
1. `MIG-056 §K.2 — Hotfix federated FTS5 SQL + libraryStats lifecycle` (the SQL drop + frontend re-fire — historical record even though §K.3 supersedes the SQL workaround; we leave the cleanup to §K.3's commit so the progression is auditable)
2. `MIG-056 §K.3 — v2 scatter-gather federated search + RRF merge` (the architectural fix Eisa requested)

OR collapse into one commit:
- `MIG-056 §K.2+§K.3 — Federated search hotfix + scatter-gather + RRF` (cleaner if Eisa just wants the final state in main).

Eisa decides at commit time.

---

## State-of-standing snapshot — 2026-05-27 afternoon

### (a) Verified-shipped and protected (on `main`, committed)

- **MIG-055 §A–§I:** Constellation Base lens system (clean rebuild). 84/84 lens tests, Boss-tested Stages 1-4 pass, Stage 5 surfaced the federation gap that opened MIG-056. (10 commits `15c41504` → `0ce98593` on 2026-05-26).
- **MIG-056 §A–§J + §K.1:** Cross-universe federation foundation. Commits through `0d5c1f8f MIG-056 §K.1 — Register FTS5 tokenizer on federation Connection`. 47/47 federation tests + integration tests passing. §J audit PASS-WITH-NOTES with 2 P1 hotfixes applied inline as §J.1.

### (b) At-risk / in-flight / uncommitted in the worktree

- **§K.2 — Hotfix federated FTS5 SQL + libraryStats lifecycle re-fire** (working tree, NOT committed). Eisa verified Stage 2 (8751 notes) + Stage 4 (`الرباط` returns results) PASS but flagged ranking quality.
- **§K.3 — v2 scatter-gather + RRF merge** (working tree, NOT committed). Architecture is correct, 7/7 RRF unit tests + 46/46 federation tests pass. But Boss-test reveals the symptom isn't fully fixed: when the typed query is a SHORT PREFIX that ALSO matches a corpus lemma (e.g. "الربا"), the lexicon expansion fires and replaces the prefix wildcard `الربا*` with a multi-language exact-phrase OR — which doesn't match the user's intended target "الرباط".
- **§K.3.A diagnostic probes** (working tree). Per-branch `name_like` / `exact_match` / `prefix_AND_name` probes + top-5 rank logging. To be removed after the underlying issue is resolved.

### (c) Known-broken

1. **Lexicon expansion boundary** — when a typed Arabic input is BOTH a short prefix of a longer word AND a lemma in the corpus, the expansion replaces the prefix wildcard with exact-phrase OR. The user-expected note (the longer word containing the prefix) doesn't appear. Reproducible in single-schema mode too — federation just made it more visible. Root cause: `expanded_match_query` in `src-tauri/src/search.rs` short-circuits the prefix wildcard when expansion fires. Fix: include BOTH the expansion AND the literal prefix wildcard in the OR. This is its own MIG (call it MIG-057 or §K.4) — NOT a federation patch.

2. **Arabic input truncation in QuickSwitcher** — Eisa reports the search box truncates Arabic text when she types at normal pace; she can only get full multi-character Arabic words in by pasting or typing very fast. Suspected: `filtered` `$derived` + async `constellationSearch` debounce + Svelte two-way binding racing with IME composition. Pre-existing (was masked by federation issues drawing all attention). Surfaces as: typed "الرباط" appears truncated to "الربا" or shorter. Investigation pending.

3. **Slow cu1 branch lexical_search (~27 seconds)** — observed in §K.3 diag log from morning run. After PRAGMA setup parity (§K.3.A change), morning numbers showed cu1 at 23 seconds — still much slower than the single-schema active-mode equivalent (~1 second). Root cause not yet diagnosed. Could be FTS5 cache cold-start; could be lock contention with the ATTACH-based `federated_conn` reading the same file; could be something else. Doesn't block functionality (correct results still come back, just slowly).

4. **One cUniverse `كون عيسى` shows federation warning `search.db missing`** — documented Stage 5 skip_unavailable behavior. Resolves itself if Eisa opens that cUniverse as active universe once (to let `init_db` build its `search.db`). Not a bug per se; documented as expected behavior.

### (d) Pending but not started

- **§L — PCS (MIG-055 + MIG-056 combined release):** push, orientation v2.37, 15-locale help-doc additions for lens + federation, git tag `milestone/mig-055-mig-056-combined`, final session log + MoCh entry. Gated on §K Boss-test gate going green.
- **MIG-057 (or §K.4) — Lexicon expansion boundary fix.** Include literal prefix wildcard alongside lexicon expansion so short-prefix queries don't lose the substring-search semantics.
- **MIG-058 (or similar) — QuickSwitcher Arabic IME / input race fix.** Investigate the truncation; likely a Svelte $effect + debounce + bind:value interaction.
- **MIG-059 (or §K.5) — Slow cu1 branch lexical_search investigation.** Why does the standalone-Connection lexical_search take 23-27s when the active-mode equivalent takes ~1s? Possibly fixable by switching to a Connection pool with eagerly-warmed FTS5 indexes, or by using read-only URI flags.
- **§J P2/P3 deferred items** (5 items from yesterday's §J audit, slated for §L cleanup pass).

### (e) Documentation drift

- `docs/MIG-056-BOSS-TESTS.md` now has §K.2.D, §K.2.E, §K.3 retest, §K.3.A probe sections accumulating. After §K passes, this doc should be CLEANED UP to the final boss-test tutorial — currently every retest stage is in the file.
- Orientation v2.36 → needs bump to v2.37 with MIG-055 + MIG-056 details. Not started.
- 15-locale help docs not yet updated.

### Recommended next action (my read, awaiting Eisa's call)

**Stop, regroup, plan tomorrow.**

The federation architecture works. The status bar works. The skip_unavailable model works. The exact-word search works. What remains is a *pre-existing* lexicon expansion issue and a *pre-existing* input truncation issue — both of which the federation work *surfaced* but didn't *cause*.

The clean path: commit §K.2 + §K.3 as the federation foundation, ship them, and open separate focused MIGs for the lexicon boundary fix and the input truncation. Bundling everything into the federation MIG would muddle attribution and make audit harder.

Boss decides.
