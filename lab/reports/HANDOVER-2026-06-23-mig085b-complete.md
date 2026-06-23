# Handover — 2026-06-23 — MIG-085 §B COMPLETE (Unicode name-fold + maturity single-source); MIG-086 architected

## State of standing

**Shipped + Boss-validated end-to-end this session (on `main`):**
- **MIG-085 §B.0 — Unicode name-fold (the false-orphan fix).** SQLite `LOWER()` folds ASCII only, so 13
  accented-title notes (`Île-de-France` 17 inbound, `Śramaṇa` 26, `Étienne-Jules Marey` 16, …) failed to
  match their own inbound links → `incoming_count=0` → false orphans in the Reviewer, wrong Sky
  maturity/stratum, 0 backlink badge. Fix: one canonical `fold_match_key()` (NFC → full-Unicode lower →
  NFC) + a new `note_meta.name_lower`; name-side matches → `COALESCE(name_lower, LOWER(name))` (NULL =
  old behaviour, the rollout gate); `name_fold_backfill` (resumable) fills it + NFC-re-folds
  `alias_lower` + recomputes the accented notes' counts. **Stage-1 Boss test: PASS.**
- **MIG-085 §B.1 — maturity single-source.** `MATURITY_SQL_EXPR` inbound → `COUNT(DISTINCT source_path)`
  + `status != 'archived'` (== `incoming_count` by construction). `maturity.rs` rewritten FS-walk → pure
  `note_meta` read (Rule-8 win). STRATUM ≥5 kept `COUNT(*)` (matches `strata.rs`).
- **MIG-085 §B.2 — outbound single-source (Boss-caught at Stage 2).** `inspector360` overrides BOTH
  `total_inbound` and `total_outbound` from `note_meta.incoming_count`/`outgoing_count` (one query). The
  360 header (↑16 ↓17) now matches Backlinks both directions. **Stage-2 Boss test (after the fix): PASS.**

**Commits:** `dd836f36` (MIG-085 §B.0 + §B.1) + the §B.2 + close-out docs commit (this handover lands
with it). **Binary:** `src-tauri/target/release/constellation.exe` (18:13, includes §B.0/§B.1/§B.2 +
frontend getBacklinks NFC). On first open of the live universe the `name_fold` backfill runs once,
fixes the 13, stamps `schema_versions.name_fold`.

**Verification:** full Rust suite **969 passed / 0 failed**; svelte-check 0; `fold_match_key` +
cross-surface agreement unit/integration tests; **live-copy rehearsal on the real universe** (all 13
fixed, zero collateral, zero false orphans remain); high-effort review's **6 findings all fixed**.

## Open / next

### MIG-086 — Reviewer link suggestions ("Connect to [[X]]") — ARCHITECTED, awaiting Boss decisions
Doc: `docs/MIG-086-Architect-Reviewer-Link-Suggestions.md`. **Recommendation: BM25 "More Like This"**
over the note's top-IDF terms — query-time over the existing FTS5 index (Rule-8 clean, no model, no
precomputed matrix), reusing `read_cooccurring_terms`'s tokenizer + `term_vocab` IDF + the existing
wikilink-create path (NO parallel `note_links` writer — the Living Link invariant). UI: a ranked
"Connect to:" list with shared-term chips + a one-click 🔗 Link button, in the Reviewer's orphan/fragile
prescription block (in place — predecessor: `ReviewerView.svelte` prescription `:246-247`, inert Connect
button `:397`). **5 open Boss decisions** (doc §9): (1) default link type for one-click connect —
recommended orphan→`associative`, fragile→`derives-from`; (2) candidate count — recommended 5; (3)
fragile vs orphan suggestion — recommended same list, different default type/heading; (4) Reviewer-only
vs everywhere — recommended Reviewer-only v1 + a reusable `<RelatedCandidates>` component; (5) fate of
the inert "Connect" button — recommended remove. **Boss rules §9 → Plan (4 steps, doc §8) → build.**

### Deferred / honest residuals (not bugs, flagged not parked)
- The 360's type-breakdown **bars + dot matrix** still visualize every link *instance* (duplicates show
  as multiple dots); only the header counts are canonical now. Boss was offered the matrix-dedup
  follow-up; left as-is unless he asks.
- `maturity.rs` now shares the Reviewer's dependency on `incoming_count` being stamped (narrow
  first-boot window before the incoming backfill; self-heals; consistent with the Reviewer).

## Invariants locked (don't regress)
One canonical `fold_match_key` for name↔target↔alias matching (NFC, no Arabic strip — the target side
never stripped); `COALESCE(name_lower, LOWER(name))` everywhere the name is a match key; sky_nodes.id is
Rust-folded; MATURITY inbound == `incoming_count` (DISTINCT + `!= archived`); STRATUM ≥5 stays `COUNT(*)`
(== `strata.rs`); the Living Link single-writer (links born as `[[wikilink]]` body text → reindex
derives `note_links` — no parallel writer); Rule 8 (all the new reads are indexed, the backfill is
background + resumable).

## To resume
Read orientation **v3.03** (highest) + this handover + SESSION-LOG-2026-06-23.md. Then bring Boss the
**MIG-086 §9 decisions** (or he answers them), write the Plan, and cascade the build.
