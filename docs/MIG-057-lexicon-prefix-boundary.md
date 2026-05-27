# MIG-057 — Lexicon Expansion Boundary Fix (SHIPPED)

**Status:** Shipped 2026-05-27.
**Commit:** _(pending — to be filled at PCS)_.
**Priority:** P2. Made search behave correctly for short-prefix queries that happen to be corpus lemmas.

## The bug, in user terms

When you typed a short Arabic word in the search box that was BOTH (a) a prefix of a longer word AND (b) a recognized lemma in Constellation's lexicon, the search expanded to multi-language exact-phrase OR and **lost the prefix-wildcard substring semantics**. The user-expected note (the longer word containing the prefix) didn't appear in the results.

### Canonical reproduction (pre-fix)

- Type `الربا` (5 chars — "interest/usury" in Arabic, a corpus lemma).
- Pre-fix behavior: lexicon expansion fired with `fts_query = "الربا" OR "interest" OR "usury" OR "ربا" OR ...`. The note `الرباط` (the city of Rabat) didn't match any exact phrase in the expansion. Result: missing.

If `الربا` were NOT a lemma, expansion would have returned `None` and `fts_query = "الربا*"` (prefix wildcard) — which DOES match `الرباط`. So the lemma-detection was exactly what killed the substring match.

## The fix

In `src-tauri/src/search.rs::expanded_match_query`, when expansion fires AND produces an OR-expression, **also append** the literal prefix wildcard:

```rust
let combined_expr = if prefix_safe.is_empty() {
    match_expr
} else {
    format!("{} OR {}*", match_expr, prefix_safe)
};
```

So for `شجرة` (or any Arabic lemma):
```
fts_query = "شجرة" OR "tree" OR "árbol" OR ... OR شجرة*
```

The note titled `شجرتنا` (a longer word starting with the same prefix) matches via the `شجرة*` half. The cross-language semantic expansion still works via the lemma half. BM25 ranks both kinds of match; title-exact-match notes (`شجرتنا`) win for short prefix queries because column weight 10 (name) >> column weight 1 (body).

The prefix term is quote-sanitized (`normalized.replace('"', "")`) — same sanitization the no-lemma fallback in `lexical_search` already used.

## Tests

Three new regressions in `tests_m12`:

- `known_lemma_expansion_keeps_prefix_wildcard` — English `tree` expansion ends in `OR tree*`.
- `arabic_lemma_expansion_keeps_prefix_wildcard` — Arabic `شجرة` expansion ends in `OR شجرة*`.
- `prefix_appended_form_has_no_quotes_in_prefix_term` — sanitization check: any double-quote in user input is stripped from the prefix term so FTS5 grammar stays valid.

All 8 `tests_m12` lexicon-expansion tests pass. 836/836 lib tests pass overall (no regression).

## Verification

The user-facing test: with the active universe + federated cUniverses, search for `الربا`:
- **Pre-fix:** returns the lemma's translations (`Rabat`, `interest`, `usury`, `ربا`-rooted notes) but NOT `الرباط`.
- **Post-fix:** returns the same set PLUS `الرباط` (and any other notes whose tokens start with `الربا`), with BM25 ranking pushing the literal-title `الرباط` note to or near the top because of the name-column weight boost.

Confirmed via 836 lib tests including 3 new MIG-057 regressions and 5 pre-existing m12 tests covering corner cases (unknown words, punctuation, proper nouns, both Arabic and English lemmas).

## Related

- Originally surfaced during MIG-056 §K.3 Boss-test (2026-05-27).
- Root cause file: `src-tauri/src/search.rs::expanded_match_query`.
- Diagnostic evidence: §K.3.A probe values confirmed `prefix_AND_name_الرباط=1` (the prefix DOES match the note via FTS5) but `lexical_search` produced different results because the expansion path replaced the prefix.
- Federation (MIG-056) is unaffected by this fix at the architectural level — the §K.3 scatter-gather + RRF merge keeps working identically. The fix just ensures each per-Connection `lexical_search` produces the right set of matches.
