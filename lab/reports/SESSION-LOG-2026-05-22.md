# Session Log — 2026-05-22

**Theme:** MIG-042 (drop the dead `term_vocab.bridge_concept_id` column) shipped + validated — and, in the course of testing it, discovered & fixed **three** further bugs (BUG-020, BUG-021, BUG-022) plus a stale-test-data fix. All validated end-to-end on Eisa's real universes.

---

## 1. MIG-042 — drop `term_vocab.bridge_concept_id` (the planned task)

The deferred "optional cleanup" from MIG-041 §2.3 / Phase D. Ran the four-phase `/migration` workflow.

- **Architect + Plan** (`docs/MIG-042-drop-bridge-concept-id-column-ARCHITECT.md`): the column is dead schema (no reader anywhere, frontend grep-confirmed) from the abandoned §1C eager-tagging design; the base `term_vocab` CREATE never defined it (only the per-boot `ensure_term_vocab_bridge_column` add-path did). Chosen design: drop the index then the column in a one-time **Part 3** of the existing MIG-041 background worker (`run_bigram_purge`), reusing its `MIGRATION_ACTIVE` daemon-pause + retry-on-busy + self-checkpoint; gated by a new `schema_versions` module `term_vocab_dropcol`. `init_db` pre-stamps `dropcol=1` when the column is already absent so clean DBs never wake the worker.
- **Phase A** — stop maintaining the column: `ctse::hooks::apply_delta` INSERT drops the column; deleted `ensure_term_vocab_bridge_column` + its init_db call; updated the schema-version doc + `ctse/{mod,search}.rs` comments. `cargo test ctse::hooks` green (5).
- **Phase B** — the drop: helpers `term_vocab_has_bridge_column` + `drop_bridge_concept_id_column`; Part 3 in `run_bigram_purge`; pre-check `|| dropcol < 1`; init_db pre-stamp. New unit tests `mig042_dropcol` (3) pass incl. the crash-matrix re-entrancy case.
- **Copy-test on a copy of the real 1.63 GB DB**: caught a **real blocker** (see BUG-020). After the fix, the exact shipped DDL sequence ran: drop in **0.37 s**, **538,648 rows preserved (zero loss)**, integrity_check/quick_check ok.
- **Phase C `/simplify`**: clean; one misleading stale comment (`bridge_concept_id resolves to M11 concepts`) corrected.
- **Phase D audit** (3 parallel agents): invariants hold; no positional `SELECT *` on term_vocab anywhere (drift); **rollback proven safe** (init_db re-adds the column before `state.db` is published; both `apply_delta` sites guard `if let Some(conn)`); trigger-drop→column-drop ordering holds.
- **Honest payoff:** schema hygiene, ~negligible disk (the column was ~95% NULL but **NOT** 100% — ~24,827 / 538,648 rows carried a stale value; the earlier "all-NULL" claim was corrected). No user-visible effect.

## 2. BUG-020 — orphaned `sight_v5_layout_invalidate_ad` trigger (folded into MIG-042)

The MIG-042 copy-test failed at `ALTER TABLE … DROP COLUMN` with `error in trigger sight_v5_layout_invalidate_ad: no such table: main.sight_v5_layout`. `DROP COLUMN` re-validates the **whole** schema. Root cause: **MIG-028** (Sight v5 retirement) dropped the `sight_v5_layout` table + its AFTER-UPDATE trigger (`_au`) but **missed the AFTER-DELETE trigger (`_ad`)** on `note_meta`. With the table gone and `_ad` surviving, **every `DELETE FROM note_meta` failed** — and in `reindex_delete_note` the error was swallowed (`let _ =`), so **deleted notes silently ghosted in the index** since ~2026-05-18. Fix (Eisa-approved fold-in): added `DROP TRIGGER IF EXISTS sight_v5_layout_invalidate_ad;` to the existing MIG-028 cleanup batch in `init_db` — fixes note deletion on boot AND unblocks the column drop. Confirmed it's the sole `sight_v5` leftover.

## 3. Pre-existing test-failure investigation → BUG-021 + m12 fix

Spawned task: 5 pre-existing `search.rs` test failures (confirmed on clean HEAD).

- **BUG-021 (m8c ×4) — REAL latent bug.** `init_db` created `idx_link_target_path ON note_links` (~line 1832) **before** `CREATE TABLE note_links` (~line 2054) — unconditional, added during MIG-025. On a **fresh** DB this aborts init with "no such table: note_links"; existing DBs mask it (`CREATE INDEX IF NOT EXISTS` no-ops because note_links already exists). `ensure_search_db_ready` runs `init_db` on a fresh path for a new/rebuilt universe → **new-universe / rebuilt-universe init crashes.** Fix: moved the index into the `note_links` CREATE batch (after the table). m8c 4/4 green after the fix.
- **m12 ×1 — stale test data (not a bug).** `unknown_word_falls_back_to_none` assumed "quasar" was out-of-corpus; it was added to `lexicon_v1.tsv` since. Fix: use the nonsense token `zxqwborple`.

## 4. BUG-022 — empty index has no recovery path (found via Eisa's live test)

After installing the BUG-021 binary, "Eisa Universe" still showed **"0 notes"** + disk thrashing. DB probe: schema healed (BUG-021 worked — `user_version=1`, `note_links` present, MIG-042's `term_vocab_dropcol=1`) but **`note_meta` = 0 rows** — the content index was never populated. Root cause: the **warm-boot "ZERO BOOT-TIME WALKS"** optimization removed the boot-time index walk; the promised replacements (a "Settings → Rebuild Index" button + a per-universe empty-cache prompt) were **never built** — so a universe with an empty index (BUG-021's victims, a wiped/restored DB, or files synced in while closed) had **no automatic or manual recovery**. Fix (Eisa chose "auto-build only"): in `initializeApp`'s post-stats fan-out (`+layout.svelte`), if the active universe's indexed note-count is 0 but it has libraries, kick off `initSearchIndex()` (the same builder `add_library` uses) in the background. **Gated on empty**, so already-indexed universes never walk — the ZERO-BOOT-WALKS rule holds for the common case. Runs on boot AND universe-switch (both go through `initializeApp`).

## 5. Validation (Eisa, live, on real universes)

- "Eisa Universe" **repopulated** on open (BUG-022); **switching** to another universe rebuilt it too. (BUG-021 + BUG-022.)
- "Eisa Cognitive Knowledge": search returns results normally (MIG-042 column drop — no regression); a created+deleted test note **disappears from search** (BUG-020 fix — no ghost). 
- **All Stage tests: PASS.**

## 6. Commits (Eisa: "2 commits")

Three of the four fixes share `search.rs`; strict 4-way per-concern commits would need interactive hunk-staging (disallowed) or destructive working-tree reconstruction (rejected). Grouping:
- **Commit 1 (backend):** `search.rs` + `ctse/{hooks,mod,search}.rs` + the architect doc — MIG-042 + BUG-020 + BUG-021 + m12 test fix.
- **Commit 2 (frontend + docs):** `+layout.svelte` (BUG-022) + session log + orientation v2.25 + LESSONS-LEARNED + this MoCh.

## 7. Lessons (LESSONS-LEARNED)

- **LL-025** — Test DB migrations under live app concurrency, not just an isolated copy (the deferred MIG-041 lesson, now formalized; reinforced by the MIG-042 copy-test catching BUG-020/the orphan trigger only on a copy of the *real* DB).
- **LL-026** — A `CREATE INDEX` before its `CREATE TABLE` in init_db aborts fresh-DB init; `IF NOT EXISTS` makes existing DBs mask it, so only new universes / rebuilds break (BUG-021).
- **LL-027** — Removing an automatic maintenance pass (here the boot index walk) for performance requires a *verified* recovery path; don't trust a code comment that says "now triggered by X" without confirming X was actually built (BUG-022).

## 8. Docs / Standing Order

- Orientation bumped to **v2.25** (new file alongside v2.24; SO #6).
- **Help files / User Manual: no change.** MIG-042 is invisible; BUG-020/021/022 are bug fixes with no new user-facing surface (the auto-recover is automatic + invisible). Docs-sync rule satisfied **by exception**, noted here.
- Nothing pushed (commit only, per Eisa's standing rule).

## 9. Open / deferred

- MIG-041's dead `bridge_concept_id` column is now **dropped** (MIG-042) — that deferred item is closed.
- **The missing "Settings → Rebuild Index" button** (referenced in `+layout.svelte` comments but never implemented) — BUG-022's auto-recover covers the *empty* case; a manual rebuild for a *stale/corrupt-but-non-empty* index is still unbuilt. Candidate follow-up (Eisa chose "auto-build only" for now).
- Other universes flagged `user_version=0` (e.g. "Constellation Test") will self-heal on next open via BUG-021 + BUG-022.

## 10. Late-session: NSC Core Plug-in design captured (Eisa direction)

After the MIG-042 cascade closed (pushed at 780713b6), Eisa picked the next direction from the post-session menu: **grow the NSC into a standalone Core Plug-in serving every Constellation function** (his stated future direction from the handover). Design-first per the project pattern.

**Vision elicited (4 questions, locked):**
1. **Shape:** Both — a shared summary *service* + a left-dock *view*.
2. **Dock-view purpose:** **Universe Digest** — skim the whole KB at summary-level without opening notes.
3. **Service reach:** **all surfaces** (full coverage is the target; sequenced sensibly).
4. **Digest granularity:** **tiered Library → Folder → 1-line headline**, expandable to the full summary.

**Five design decisions locked** (during validation): name = **"Digest"**; **stored** `headline` column; Digest **spans cUniverse children** from v1; **extractive only** (abstractive deferred); **default sort = recency**.

**Artifacts written:**
- `docs/Constellation-NSC-Concept-Paper-v2.0.md` — elevates NSC from subsystem (v1.0, MIG-040) to Core Plug-in. Two pillars: shared service + Universe Digest. Reuses engine/cache/content-hash/backfill. Phasing = 3 `/migration`s.
- `docs/MIG-043-nsc-coreplugin-phase1-ARCHITECT.md` — Phase 1: engine `headline` variant (top-1 TextRank) + additive nullable column + shared frontend summary store + migrate SourceReviewPanel as no-behavior-change refactor + wire 2 first surfaces (search results + editor header). 6 steps A–F; low-moderate risk; rollback safe both ways.

**Next:** PCS the design docs (this commit) + orientation v2.26 bump, then cascade Build Steps A–F (Eisa direction: "PCS + Orientation > And cascade the Build (Steps A–F)").
