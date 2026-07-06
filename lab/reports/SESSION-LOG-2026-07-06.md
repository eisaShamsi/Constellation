# Session Log — 2026-07-06 (Quick Switcher speed · reproduce-first)

**Branch:** `main` · **Predecessor:** MIG-092 DONE (`a617def4`; see SESSION-LOG-2026-07-05.md).

## Function in hand: the Quick Switcher (Ctrl+O) — retrieval speed

**Boss symptom (2026-07-06):** typing is fine, but **getting results takes 2+ MINUTES
with heavy thrashing.** That is a pathology, not inherent slowness.

**Reproduce-First:** no fix designed/shipped until the delay is reproduced under
instrumentation and the mechanism is read off the trace.

### Path mapped (code-read, not yet the diagnosis)
`QuickSwitcher.svelte` (fed the in-memory `allNotes` cache): 300ms debounce →
instant local substring filter → for ≥3-char queries **`await constellationSearch`**
(mode `lexical` via `parseSearchQuery`) → `search.rs::constellation_search`
(`(async)`, holds `state.db.lock()` for the duration) →
`federated_lexical_search_or_fallback` → sequential per-schema FTS5 branches
(`main` + every attached cUniverse) on the shared `federated_conn` → RRF merge.
**The instant local title hits are held back behind the awaited federated search.**
MIG-058's own comment records "10+s on cold federated FTS5" — the Boss sees 2+ min.

**Candidate mechanisms the trace must separate:** (a) `state.db` lock WAIT
(contention/pile-up — matches the thrashing), (b) per-branch federated FTS5 cost
(which cUniverse), (c) merge/other.

### Instrumentation shipped (TEMPORARY) — commit `52ea2e2d`
- `search.rs`: `LAST_SEARCH_TRACE` phase log per `constellation_search` —
  `db_lock_wait`, `execute`, `federated_conn_lock_wait`, `branch:main`,
  `branch:cuN`…, + the fallback paths. New `get_last_search_trace` command.
- `QuickSwitcher.svelte`: per-run on-screen diagnostic line (devtools is OFF in
  release): `run #N · local Xms (hits) · rust Ys [phase trace]`; stale runs
  report too (pile-up visibility).
- No retrieval behavior changed. cargo check + svelte-check clean.

### Next
Boss reproduces on the instrumented binary (Ctrl+O → type `knowledge`) → read the
trace → THEN design the fix (likely shape, unconfirmed: show local title hits
immediately; make the federated search non-blocking/cancellable; fix whatever the
trace fingers — but the trace decides).
