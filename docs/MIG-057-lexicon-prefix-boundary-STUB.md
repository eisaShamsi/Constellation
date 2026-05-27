# MIG-057 — Lexicon Expansion Boundary Fix (STUB)

**Status:** Open — pending architecture.
**Opened:** 2026-05-27 (post-MIG-056 §K Boss-test).
**Priority:** P2. Not blocking; degrades search relevance for short Arabic prefix queries that happen to be lemmas.

## The bug, in user terms

When you type a short Arabic word in the search box that is BOTH (a) a prefix of a longer word AND (b) a recognized lemma in Constellation's lexicon, the search expands to multi-language exact-phrase OR and **loses the prefix-wildcard substring semantics**. The user-expected note (the longer word containing the prefix) doesn't appear in the results.

### Canonical reproduction

- Type `الربا` (5 chars — "interest/usury" in Arabic, a corpus lemma).
- The lexicon expansion fires: `fts_query = "الربا" OR "interest" OR "usury" OR "ربا" OR ...`.
- The user wanted to find `الرباط` (6 chars — the city of Rabat). `الرباط` doesn't match any exact phrase in the expansion. Result: missing.

If `الربا` were NOT a lemma, expansion would return `None` and `fts_query = "الربا*"` (prefix wildcard) — which DOES match `الرباط`. So the lemma-detection is exactly what kills the substring match.

## Why this is its own MIG

Pre-existing in single-schema mode. MIG-056's federation just made it more visible because users now query across more notes. The fix touches `expanded_match_query` in `src-tauri/src/search.rs` — orthogonal to federation. Should be tested in single-schema mode and verified to continue working in federated mode.

## Proposed approach (to be validated by Architect agent)

Include BOTH the lexicon expansion AND the literal prefix wildcard in the FTS5 MATCH OR-expression:

```
fts_query = "(<expansion>) OR <input>*"
```

So for `الربا`:
```
fts_query = "(\"الربا\" OR \"interest\" OR \"usury\" OR ...) OR الربا*"
```

The note titled `الرباط` matches via the `الربا*` half. The cross-language semantic expansion still works via the lemma half. BM25 will rank both kinds of match; title-exact-match notes (`الرباط`) should win for short Arabic prefix queries.

## Verification clauses

- [ ] Single-schema search for `الربا` returns `الرباط` (the city note) at or near rank 1.
- [ ] Single-schema search for a non-prefix word like `interest` returns the cross-language expansion (notes with `الربا` / `usury` / `ربا` in body), as it does today.
- [ ] Federated search across Eisa Universe + Eisa Cognitive Knowledge for `الربا` returns `الرباط` at rank 1 (the RRF merge already in place handles the rest).
- [ ] No regression in 84/84 lens tests, 47/47 federation tests, 41/41 libraries tests.

## Related

- Surfaced during MIG-056 §K.3 Boss-test (2026-05-27).
- Root cause file: `src-tauri/src/search.rs::expanded_match_query`.
- Diagnostic evidence: `lab/reports/SESSION-LOG-2026-05-27.md` (state-of-standing record), specifically the diagnostic where `prefix_AND_name_الرباط=1` (prefix DOES match the note) but `lexical_search` produced different results because the expansion path replaced the prefix.
