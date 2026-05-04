# MIG-011 — Cross-language Index *Filter* Expansion

**Date opened**: 2026-05-04
**Status**: Phase 1 (Architect) — Boss-approved 2026-05-04
**Owner**: Index closure work-stream (continuation of MIG-010)
**Composes with**: MIG-010 (mentions-side cross-language expansion, just closed); the M11-data v2 Lexical Bridge corpus; the just-shipped script-filter substring fix.

---

## §1 · Goal

MIG-010 made the Index **mentions list** lexical-bridge-aware: clicking "knowledge" surfaces Arabic notes about "معرفة" / "علم" with `via {lemma}` badges. Boss-observed 2026-05-04 (during MIG-010 G3 testing): typing "knowledge" in the Index **filter box** still only finds index entries whose text literally contains the letters k-n-o-w-l-e-d-g-e — Arabic terms like "معرفة" are invisible to that filter.

**MIG-011 extends the same bridge logic to the filter**. Typing "knowledge" should surface:
- (current) all index terms containing "knowledge" as substring — direct same-language matches.
- (new) all index terms that are bridge-equivalents of "knowledge" in other languages — "معرفة", "علم", "إدراك" in Arabic; "connaissance", "savoir" in French; "知识", "认识" in Chinese; etc.

Each cross-language term in the result list carries a small `via {source}` badge so the user can tell which entries are direct hits vs. lexicon-bridged. Same UX pattern as MIG-010's mention badges.

The toggle stays the same — `Settings → Index → Expand mentions cross-language` — but its scope expands to cover both surfaces. One toggle, two behaviors that compose.

---

## §2 · Mapping the territory

### Current Index filter logic (`src/lib/components/IndexPanel.svelte`)

- `filterQuery`: `$state<string>('')` — what the user types in the search box.
- `filteredEntries`: `$derived.by(() => entries.filter(...))`. Per-keystroke recomputation. Substring match with bidirectional check (just shipped).
- `prepareQuery(q)`: per-keystroke Light10-style normalize for Arabic. Comment explicitly says it's a JS port of the Rust stem path; in practice the Rust path is now CAE, so the JS port is approximate.
- 50–500k entries on a 7,600-note Universe; the per-keystroke filter loop is hot.

### Current Bridge surfaces (post-MIG-010)

- `crate::lexicon::expand(lemma, source_lang, &opts) -> ExpansionResult` (Rust, public).
- `crate::lexicon::detect_source_lang(s) -> Option<Lang>` (Rust, public).
- `crate::search::expanded_match_query(normalized) -> Option<LexicalExpansion>` (Rust, `pub(crate)`) — produces FTS5 MATCH expressions; not the right shape for filter-side work because we don't need a MATCH expression here.
- No frontend-callable bridge IPC exists. MIG-010 routed everything through `read_term_mentions` server-side.

### Why the current substring filter can't be patched in place

The bridge expansion produces a *set of lemmas across languages*. Substring-matching against a set means the per-keystroke filter has to:
1. Get the bridge-expansion of the typed query (a Tauri IPC call).
2. For each entry, check `entries[i].term` against every lemma in the set, in addition to the existing substring checks.
3. Annotate matched entries with the source lemma that bridged them, for the badge.

(2) is hot-path frontend work; (1) is the new IPC; (3) needs new state on the entries.

---

## §3 · Design options

### How to expose the lexicon to the frontend?

| Option | Shape | Speed | Risk |
|---|---|---|---|
| **3.A** New Tauri command `lexicon_expand_for_filter(query: String) -> Result<Option<FilterExpansion>, String>` returning `{ source_lang: Lang, lemmas_lower: Vec<String> }`. Frontend calls per-keystroke, debounced. | One IPC round-trip per debounced keystroke (~300ms). The `expand` call itself is sub-millisecond. | Fast | Low — additive, mirrors `read_term_mentions` shape. |
| **3.B** Boot-time materialization: dump every `(source_lemma, source_lang) → bridge_lemmas` pair to a frontend-side Map. ~20K concepts × 15 langs ≈ 300K pairs. ~5 MB of JSON. | Zero-IPC at filter-time. | Slow at boot (extra IPC + JSON parse + Map build). | Medium — adds 5 MB to boot payload; recomputes on every Universe switch. |
| **3.C** Hybrid: lazy-cache. First Arabic/non-en query triggers a focused IPC; subsequent queries hit the cache. Cache scoped to the filter session. | Fast after first hit. | Moderate complexity. | Low. |

**Decision: 3.A.** Per-keystroke IPC with 300ms debounce + a small in-memory result cache keyed by query string. The expand call is microseconds; the IPC overhead is the dominant cost (~1ms). On modern hardware this is imperceptible — the user types, pauses 300ms, then 1ms IPC + filter recomputation. (3.B's boot-time payload conflicts with the boot-perf budget; (3.C)'s hybrid is just (3.A) with a cache, which (3.A) already has.

### What does the IPC return?

| Option | Shape |
|---|---|
| **3.D** `Option<FilterExpansion> { source_lang: Lang, lemmas_lower: Vec<String> }`. The frontend picks lemmas, lowercases them, runs a per-entry includes-or-equals scan. Source lemma included for badge rendering. |
| **3.E** Just `Vec<String>` (lemmas). Simpler but loses the source_lang context — which means the frontend can't filter "same-language inflections only" the way M13 does for badges. |
| **3.F** Pre-filtered `Vec<{ lemma: String, lang: Lang }>` so the frontend can scope which-script-shows-which-badge. |

**Decision: 3.F.** The frontend already has the entry's first-character script via `getScript()`. Pairing each bridge lemma with its source language lets the per-entry match also produce the *correct* `via {lemma}` annotation: when the entry is Arabic and matched a bridge lemma "knowledge" (en), the badge says "via knowledge"; when the entry is English and matched "معرفة" (ar), the badge says "via معرفة". This is consistent with MIG-010's M13-derived rule (badges show the *cross-language* lemma, not the same-language one).

### How to render the cross-language entries in the filter results?

| Option | UX |
|---|---|
| **3.G** Same flat list, with a `via {lemma}` badge on cross-language matches. Sort: direct matches first, then bridged. |
| **3.H** Two-section list: "Direct matches" then "Cross-language matches via lexicon" below. |
| **3.I** Ad-hoc — direct matches in normal style, bridged matches in italic/muted. |

**Decision: 3.G.** Same flat list, badge on cross-language entries, mentioning preserved sort order (direct → bridged within sort criteria). Mirrors MIG-010's mentions-side pattern; one mental model for both surfaces. The two-section split (3.H) was rejected for MIG-010; same reasoning here.

### Does the toggle drive both surfaces?

| Option | Behavior |
|---|---|
| **3.J** ONE toggle (the existing `expandCrossLanguage`) drives both mentions AND filter. |
| **3.K** TWO separate toggles — one for mentions, one for filter. |

**Decision: 3.J.** One toggle for both. The mental model is "expand cross-language in the Index" (singular setting); having two toggles requires the user to know they're separate concerns, which they aren't from the user's perspective. If a user wants asymmetric behavior in the future (filter cross-language but not mentions, etc.), that's a follow-up MIG with explicit demand.

### What about substring + bridge composition?

When toggle is on AND the query is in-corpus AND the user has typed enough to substring-match SOME entries directly, do we show:
- The substring-match results PLUS the bridge-match results? (union)
- Or only the bridge-match results when bridge produces hits? (replace)

**Decision: union.** Show direct-substring matches first (no badge), then bridge-matches (with badges). This is the most informative view. The user sees their literal query AND the conceptual neighborhood it bridges into.

---

## §4 · Invariants that must not break

| # | Invariant | How verified |
|---|---|---|
| **F1** | Default install (toggle off) behaviour unchanged: filter is pure substring (with the bidirectional check shipped just before MIG-011). | Existing filter tests + manual: type "knowledge" off → only English-substring entries. |
| **F2** | Toggle on + out-of-corpus query: behaves as toggle off. No errors, no badges, just substring matches. | Manual: type "Xzyqwop" with toggle on. |
| **F3** | Toggle on + in-corpus query: union of direct substring matches and bridge-equivalent entries. Cross-language entries carry `via {lemma}` badge. | Boss G-test post-Build. |
| **F4** | The bridge IPC is debounced (≥300ms). No keystroke fires more than one IPC. Cache prevents re-firing for repeated queries within session. | Manual + console.log instrumentation in dev. |
| **F5** | Filter performance unchanged: per-keystroke filter loop stays bounded by `entries.length × O(query_len + lemma_count)`. Bridge lemma set typically ~5-15 strings; effectively constant overhead. | Manual: type quickly in a 50k-entry library; no perceptible lag. |
| **F6** | The same `Settings → Index → Expand mentions cross-language` toggle drives both mentions and filter. | Manual: flip toggle; verify mentions list AND filter both update accordingly. |
| **F7** | i18n: badge label "via {lemma}" already shipped in MIG-010 (`indexPanel.viaLemma` in 15 locales). Reused as-is — no new keys. | Grep i18n files. |
| **F8** | RTL: badge layout flows correctly (already proven by MIG-010). | Boss G-test on Arabic interface. |
| **F9** | Rule 8 compliance: no recompute on toggle flip. Toggle off → cache + bridge are unused; just substring. Toggle on → IPC fires lazily on next non-empty query. No work otherwise. | Manual: flip toggle without typing → no new IPCs. |
| **F10** | Cooccurrence chip-strip is unaffected (separate IPC, not touched). | Verified by code: bridge IPC call is in IndexPanel.svelte filter logic, not in cooccurrence. |
| **F11** | The new IPC integrates with the script-filter bug fix shipped 5dbb43f: bridge-matched entries automatically clear stale letter filters via the existing $effect. | Manual: click "K" letter → type Arabic with toggle on → bridge-matched Arabic entries appear, "K" letter clears. |

---

## §5 · Phased plan preview

Full plan in `MIG-011-INDEX-FILTER-BRIDGE-PLAN.md` (Phase 2, drafting next). Sketch:

1. **Build.1** — Rust IPC `lexicon_expand_for_filter(query)`. Returns `Option<FilterExpansion>`. Tests on the helper.
2. **Build.2** — Frontend wrapper + debounce + in-memory cache. New `bridgeFilterExpansion: $state` holding the latest expansion.
3. **Build.3** — Filter loop extended: when expansion present, also check entries against bridge lemmas. Annotate matched entries with `viaFilterLemma` field for badge.
4. **Build.4** — Render `via {lemma}` badge on cross-language filter results. CSS reuse from MIG-010.
5. **Build.5** — `/simplify` checkpoint.
6. **Audit** — Phase 4: invariant verification + drift + migration path.

Each step lands as one commit, each with a verification clause. Build.4 is the Boss-test gate.

---

## §6 · Open questions

**Recommended decision in each, but Boss can override:**

- **Q1**: Is the bridge IPC OK as a per-keystroke debounced call, or should it pre-cache the corpus at boot? **Recommended: per-keystroke (3.A).** Boss override needed if 300ms debounce feels too sluggish.
- **Q2**: Should the toggle be one for both surfaces (3.J) or split (3.K)? **Recommended: one (3.J).** Boss override needed if asymmetric behavior is desired.
- **Q3**: Substring + bridge → union or replace? **Recommended: union.** Boss override if the bridged matches feel noisy.

**Plan-Approval = Build-Approval applies.** Boss approves this Architect doc → I cascade through Plan + Build + Audit autonomously, stopping only at the Build.4 Boss-test gate.

---

## §7 · Risk register

| Risk | Likelihood | Mitigation |
|---|---|---|
| Per-keystroke IPC adds perceptible lag on slow hardware | Low | 300ms debounce + cache. Empirically the IPC is ~1ms; debounce dominates. |
| Bridge lemma set produces noisy false positives ("via {lemma}" appears on entries that aren't really conceptually related) | Low | Reuses MIG-010's `bridge_terms_lower` filter (non-source-language only). Tested by 12/12 M13 tests. |
| Toggle compounds with the just-shipped script-filter fix in unexpected ways | Low | F11 invariant explicit; Boss G-test catches. |
| 300K-pair boot dump (option 3.B) was attractive but rejected — could come back as scope creep | Low | Architect-doc decision is locked. If perf concerns surface during Audit, that's the right time to revisit, not before. |

---

**Phase 1 closes here. Awaiting Boss approval; then Phase 2 (Plan).**
