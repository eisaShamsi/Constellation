# Session Log — 2026-06-23

## MIG-085 §B — maturity single-source — ARCHITECT INVESTIGATION (pre-build)

**Function in hand:** maturity's *inbound* signal — the count of notes linking to a note —
as consumed by four surfaces: the **Reviewer** (`review.rs`, reads `note_meta.incoming_count`),
the **maturity panel** (`maturity.rs::compute_note_maturity`, FS-walk), the **360 Inspector**
(`inspector360.rs::compute_maturity_for_note`, FS-walk), and the **Sky maturity trigger**
(`MATURITY_SQL_EXPR` in `search.rs`, maintains `sky_nodes.maturity`).

**Concept (the horse):** a note's maturity is *one fact*; it must read the same on every
surface. Single-sourcing inbound to the write-time `note_meta.incoming_count` (distinct-source,
alias-aware) makes maturity agree everywhere — the same value the Reviewer already uses.

### What the investigation found (measured on the live 7,660-note "Eisa Cognitive Knowledge" universe)

Two distinct discrepancies drive cross-surface maturity disagreement:

**(1) COUNT(\*) vs COUNT(DISTINCT source_path) — minor, anticipated.**
- `incoming_count` = `COUNT(DISTINCT source_path)` (a source linking twice counts once).
- `MATURITY_SQL_EXPR` (the Sky trigger) = `COUNT(*)` of active inbound edges.
- 1,250 notes have COUNT(*) ≠ COUNT(DISTINCT); only **8 notes** flip a maturity bucket
  (all evergreen→sapling at the 4→3 boundary). The lower distinct value is the *correct*
  one (the Reviewer already shows it). Aligning the trigger to `COUNT(DISTINCT source_path)`
  is a safe correctness fix. `status='active' ≡ status!='archived'` on the live DB
  (cache.rs:2209 "all-active DB"), so status is not a discrepancy axis.

**(2) Unicode case-folding defect — MAJOR, NOT anticipated; inverts §B's premise.**
- Root cause: **SQLite's built-in `LOWER()` folds ASCII A–Z only** — it leaves accented/non-Latin
  capitals unfolded (`É`,`Î`,`Ś`,`Đ`,`Š`). Link targets (`note_links.target_name`) are stored
  Rust-folded (full Unicode, lowercase); `sky_nodes.id` and `incoming_aggregate_assignments`
  fold the *name* side with SQLite `LOWER()` (ASCII-only). The two foldings disagree on accented
  capitals, so the name↔target match **misses entirely**.
- Effect (verified, exact): **13 notes** whose names contain a non-ASCII uppercase letter have
  real inbound (3–26 distinct sources) but `incoming_count = 0`:
  `Śramaṇa`(26), `Île-de-France`(17), `Étienne-Jules Marey`(16), `Île de la Cité`(13),
  `Émilie du Châtelet`(11), `Étienne-Louis Boullée`(7), `Đông Sơn culture`(7), `Étude`(6),
  `Śāriputra`(6), `Abū Ḥanīfa`(4), `Charles-Émile Reynaud`(4), `Notre-Dame de l'Épine`(3),
  `Š-L-M`(3).
- `sky_nodes.id` is also ASCII-folded (`'Île-de-france'`) → the Sky maturity/stratum triggers
  return 0 inbound for these → `sky_nodes.maturity = 'sapling'` (wrong; should be evergreen/
  canonical), stratum understated.
- **The maturity panel (`maturity.rs`) and 360 Inspector (`inspector360.rs`) use Rust
  `.to_lowercase()` (full Unicode) and show these notes CORRECTLY.**
- **Premise inversion:** the DB-derived surfaces (incoming_count, Sky, Reviewer) are
  *consistently wrong*; the FS-scan surfaces (panel, 360) are *right*. Naively "single-sourcing
  to incoming_count" would REGRESS the panel/360 to the broken value.
- **User-visible today:** **12 of these 13** satisfy `incoming_count = 0 AND word_count > 20`,
  so they appear as **false orphans** in the brand-new Rich Reviewer's 🔗 Orphan lens
  ("connect me") despite having up to 26 real backlinks. Also wrong: Backlinks badge count (0),
  Sky maturity/stratum.

### Status of the bug in current code (not just this DB)
Current code matches `target_name_lower` (generated col `LOWER(target_name)`, ASCII) against
`LOWER(np.name)` (ASCII). Because `target_name` is stored Rust-folded but `name` is Title-Case,
`LOWER(name)` leaves accented capitals unfolded → the mismatch persists on a fresh reindex.
**The defect is current, not an artifact of an old index.**

### Verification artifacts
- `lab/scratch/mig085b_measure3.py` — single-pass flip + incoming_count cross-check (authoritative).
- Per-note folding proof: `SELECT LOWER('Île-de-France')` → `'Île-de-france'` (Î unfolded) ≠
  Rust `'île-de-france'`; inbound via folded target = 17, via `LOWER(name)=LOWER(target)` = 0.
- (Caught my own measurement bug first: reusing one cursor for a nested query reset the outer
  loop and falsely reported "0 mis-folding notes" — re-ran with separate cursors → 13. BASIC rule.)

### Implication for scope (surfaced to Eisa for a ruling — WA#6 "never silently park it")
§B as scoped (a P3 consistency nicety + a `COUNT(DISTINCT)` trigger tweak) is dominated by a
real, pre-existing **multi-surface correctness bug** (false orphans in the Reviewer, wrong Sky
maturity/stratum, wrong backlink counts) for every accented-capital-named note. The root fix is
schema-touching (a Unicode-aware fold for the name side + `sky_nodes.id`) and needs a one-time
background reindex — i.e. its own focused `/migration`. Single-sourcing maturity to
`incoming_count` is only safe *after* that fix. **No code written yet — awaiting Eisa's scope ruling.**

### Eisa's ruling (2026-06-23): "Fix accent bug first, then maturity" + "include NFC normalization"

Architect+Plan doc: `docs/MIG-085B-Architect-Plan-Unicode-Name-Fold.md`. Research-validated fix
(WA#5, 5-source web research): **store one Rust-folded key column, match key-to-key** (the field's
dominant pattern — Lucene index-time fold, Postgres normalized columns; NOT a `ulower()` SQL
function, which embeds a per-connection landmine; NOT ICU, not in rusqlite bundled). Eisa opted to
INCLUDE NFC.

### §B.0 — Unicode name-fold — BUILT + verified

- `fold_match_key(s)` (search.rs) = NFC → full-Unicode lower → NFC. **No** Arabic strip (the match
  key on the target side — `parse_link_body` — never stripped, nor did the old `LOWER(name)`; adding
  a strip would fold the name side but not the target → break Arabic links). Routed: `name_lower`
  (new note_meta col), `sky_nodes.id` (trigger + sky_backfill), `parse_link_body` target, and
  `normalize_alias_for_match` (NFC added, Arabic strip kept).
- Name-side matches → `COALESCE(name_lower, LOWER(name))` (the column's NULL-ness IS the rollout
  gate: pre-backfill rows fall back to today's ASCII behaviour, no regression, no race).
- `name_fold_backfill.rs` — resumable, convergent backfill: fills `name_lower` for all rows, NFC-
  re-folds `alias_lower`, then recomputes the accented notes' `incoming_count` + `sky_nodes.id` +
  maturity + stratum. Affected-set keyed on `name_lower != LOWER(name)` (the accented/NFD set) —
  independent of sky_nodes so a note lacking a sky row still gets its count fixed (review finding).

### §B.1 — maturity single-source — BUILT + verified

- `MATURITY_SQL_EXPR` inbound: `COUNT(*)` → `COUNT(DISTINCT source_path)` AND `status='active'` →
  `status != 'archived'` — now matches `note_meta.incoming_count` by construction (count + status +
  name match all identical). STRATUM ≥5 inbound LEFT as `COUNT(*)` (it must match the FS
  `strata.rs` the 360 uses — reverted after the review flagged the over-application).
- `maturity.rs::compute_note_maturity` rewritten FS-walk → pure `note_meta` read (incoming_count +
  same `compute_state`) — Rule-8 win + agreement. `inspector360` overrides `total_inbound` with
  `incoming_count` so its stratum/maturity/orphan/SPOF + displayed count match every surface.

### Verification
- `fold_match_key` unit tests (accent set, ASCII unchanged, NFC==NFD) + the cross-surface
  agreement integration test (`accented_note_distinct_source_agrees_across_surfaces`): **green.**
- Full Rust suite: **969 passed, 0 failed** (lone flaky `fst_bake` cache test unrelated, passes on
  re-run). `cargo check` clean. `svelte-check` 0 errors.
- **Live-copy rehearsal on the real 7,660-note universe** (`lab/scratch/mig085b_rehearse.py`): all 13
  accented notes 0 → correct count (Île-de-France 17, Śramaṇa 26, …); exactly 13 notes change (zero
  collateral on the 7,647 ASCII notes); zero false orphans remain. **PASS.**
- **High-effort code review** (parallel finder agents): 6 real findings, **all fixed** (WA#6, none
  deferred): stratum-DISTINCT over-application (reverted); maturity status `active`→`!=archived`;
  backfill INNER-JOIN miss (re-keyed); maturity scope trailing-separator trim; alias_lower NFC
  re-fold; getBacklinks JS NFC. Documented-not-blocking: maturity panel now shares the Reviewer's
  dependency on incoming_count being stamped (narrow first-boot window; self-heals).
- Release binary rebuilt (Rust + frontend) for the Boss test.

**NEXT: Boss test (Stage 1 — the accent fix), then MIG-086.**

### Boss test results + §B.2 (outbound single-source)

- **Stage 1 (§B.0 accent fix): PASS** — Boss-validated live. `Île-de-France` et al. left the Reviewer's
  false-orphan lens; Backlinks shows the real ~17 inbound; maturity is correct.
- **Stage 2 (§B.1 maturity agreement): inbound PASS** (360 ↓17 == Backlinks 17 == Sky); Boss CAUGHT a
  pre-existing **outbound** mismatch — the 360 header showed **↑34** while Backlinks/`outgoing_count`
  showed **16**. Same occurrence-vs-distinct class as the inbound bug, on the OUTBOUND side, which §B.1
  hadn't touched. Verified on the live DB: `note_meta.outgoing_count = 16` == 16 active `note_links`
  edges; the 360 was counting **34 raw `[[link]]` occurrences** (a Wikipedia import duplicates targets).
- **§B.2 fix:** `inspector360` now overrides BOTH `total_inbound` AND `total_outbound` from the
  write-time `note_meta.incoming_count`/`outgoing_count` (one `read_connection_counts` query). Verified
  the 360 frontend's type-breakdown percentages derive from the `typed_links`/`untyped_links` lists, NOT
  `total_outbound` — so the override changes only the header (↑16 ↓17), no percentage breakage. The
  matrix below still visualizes all link instances (a separate representation; flagged to Boss if he
  wants it deduped too). Rebuilt for re-test.
