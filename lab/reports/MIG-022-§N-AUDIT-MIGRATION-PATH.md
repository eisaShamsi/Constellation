# MIG-022 §N Audit — Migration Path

**Agent:** Migration Path (3-of-3 close-out audit, parallel to Invariants + Drift)
**Date:** 2026-05-12
**Method:** Read every relevant code path in the official tree (`E:\مشاريع كلاود\Constellation`); no fabrication. Where a scenario can only be verified by running the binary, marked UNVERIFIED.

---

## Summary: 3 PASS · 1 FAIL · 3 UNVERIFIED

The §B temporal-axis cluster has one structural gap (Scenario 4-adjacent / fire-path) that affects what the trigger actually catches in practice — not a migration regression, but a verification correctness issue. Migration plumbing itself (table creation, sentinel idempotency, schema_versions tolerance, additive-only field changes) is sound.

---

## Scenario 1 — First-boot of new build on fresh universe — **PASS**

**Evidence:** `src-tauri/src/search.rs:1471-1520` calls in order:
1. `ensure_note_meta_sources_column` (idempotent ALTER)
2. `ensure_note_meta_content_type_column` (idempotent ALTER)
3. `ensure_note_state_history_table` (`history.rs:61` — `CREATE TABLE IF NOT EXISTS`)
4. `ensure_note_state_history_trigger` (`history.rs:105` — `CREATE TRIGGER IF NOT EXISTS`)
5. `backfill_initial_history` (`history.rs:185` — sentinel-skipped after first run)

On a fresh universe, `note_meta` is empty, so the §B.3 backfill `INSERT ... SELECT` writes 0 rows; sentinel still stamps; subsequent boots skip. Schema matches `MIG-022-PLAN §2.1` (single-JSON-diff column shape).

---

## Scenario 2 — Upgrade from CECE v1 (pre-MIG-021v3) — **PASS**

**Evidence:** Schema additions to `note_meta` (sources at `sources/mod.rs:140`, content_type at `sources/mod.rs:802`) use idempotent `ALTER TABLE ... ADD COLUMN`. The history `FOREIGN KEY (note_path) REFERENCES note_meta(path)` resolves because `note_meta(path)` has been the PRIMARY KEY since pre-MIG-013. `sources_suggestions` data is untouched (different table). Pre-existing notes remain readable — `properties_json` column existed from pre-MIG-022 (`search.rs:1411`).

---

## Scenario 3 — Upgrade from MIG-021v3 (post-CECE; pre-MIG-022) — **PASS**

**Evidence:** `ensure_note_state_history_table` is `CREATE TABLE IF NOT EXISTS` — runs cleanly on a DB that already has the sources/content_type columns. Backfill seeds one `_seed` row per existing classified note (the `WHERE sources IS NOT NULL OR content_type IS NOT NULL OR properties_json != '{}'` filter at `history.rs:230-232`). Pre-existing classified notes remain legible — backfill only adds history rows; never modifies `note_meta`.

---

## Scenario 4 — Mid-backfill interrupt — **PASS** (with one caveat)

**Evidence:** `history.rs:200-273` wraps the entire backfill in `BEGIN IMMEDIATE` with the sentinel stamp as the LAST statement before `tx.commit()` (line 267-271). If the process dies mid-transaction, SQLite rolls back: no partial `note_state_history` rows written, sentinel never stamped. Next boot re-runs cleanly — the sentinel check at line 187-194 still returns 0 (no row exists yet), and the bulk INSERT runs again.

**Caveat (not a FAIL):** the bulk INSERT is a single statement, so SQLite either writes all 7,636 rows or none — there's no partial-write recovery scenario possible at this scale. Idempotency is ensured by the all-or-nothing transaction, not by row-level dedup. No double-counted events.

---

## Scenario 5 — Schema-version mismatch (future entry) — **PASS**

**Evidence:** Every `schema_versions` query uses `.unwrap_or(0)` (e.g. `history.rs:194`, `search.rs:1378/1438/2453/2531/2576`). Unknown module rows are simply ignored — `init_db` only checks specific module names it knows about. Each query is a `SELECT version FROM schema_versions WHERE module = '<known>'`. Unknown future entries (e.g. a downgraded universe carrying a `module = 'note_state_history_v2'` row from a future build) are silently skipped. No `init_db` panics on unknown rows.

---

## Scenario 6 — i18n locale switching — **UNVERIFIED**

**Evidence read:** `src/lib/i18n/de.json` confirmed contains `cece.taxonomy.*` (~225 keys), `cece.confidence.*`, `cece.reasoning.*`, `supersedes` (line 1873: `"ersetzt"`), and `epistemic` properties keys at lines 2080-2230+. All 15 locale files (`ar/de/en/es/fa/fr/he/hi/ja/ko/pt/ru/tr/ur/zh.json`) present.

**What's UNVERIFIED:** RTL rendering of the new `epistemic` Properties panel section in he/ar/fa/ur, the §A.3 `ikhtilāf` widget's nested-form layout under RTL, and key-completeness across all 15 locales for the §A.4 epistemic field labels.

**To resolve empirically:** Switch to fa/he/ur in the running binary, open a classified note, expand the Epistemic section, verify labels render and form layout flips correctly. Run the §E.3 cargo-test scaffold (the locale-completeness check Plan §E.3 promised) against all 15 locales.

---

## Scenario 7 — Large-universe perf — **UNVERIFIED**

**Evidence read:** §B.3 backfill is a single bulk `INSERT ... SELECT` (`history.rs:219-234`) — fastest possible SQLite pattern. Trigger drop happens before insert (`history.rs:207`), avoiding 7,636 sequential trigger fires. §B.4 `cece_get_note_history` uses the covering index `idx_note_state_history_note_time` (history.rs:71) — O(log N + k) per lookup. §B.4 `cece_query_history` builds parameterized SQL; LIMIT defaults to 1000.

**What's UNVERIFIED:** actual wall-clock numbers on Eisa's 7,636-note universe — backfill duration, boot time delta vs pre-MIG-022, IPC latency for typical query sizes. No Tauri progress event is emitted during backfill; for a 7,636-note Universe the bulk INSERT should complete in <1s, but this is a code-reading prediction, not a measurement.

**To resolve empirically:** Build the new binary, replace existing exe, time the first boot. Check `[search] init_db: MIG-022 §B.3 backfilled N initial-state history rows` log entry. Compare against pre-MIG-022 boot baseline.

---

## ⚠ Adjacent finding: Trigger fire-path narrower than the §B Plan implies — **FAIL**

**Evidence:** `index_note` (`search.rs:3045-3054`) is the canonical note-save indexing path. It uses `DELETE FROM note_meta WHERE path = ?1` followed by `INSERT INTO note_meta (...) VALUES (...)`. SQLite triggers fire on `UPDATE`, NOT on `DELETE+INSERT`. The trigger therefore only fires for the explicit CECE classifier writes:
- `sources/mod.rs:338` — `UPDATE note_meta SET sources = ?1 WHERE path = ?2`
- `sources/mod.rs:926` — `UPDATE note_meta SET content_type = ?1 WHERE path = ?2`

**What this means:** when the user edits frontmatter via NotePane (e.g., changes `held_by:` value, adds an `ikhtilāf:` entry), the resulting note re-index goes through `DELETE+INSERT` and the trigger NEVER FIRES. The MIG-022-PLAN §B.5 Boss-Test Gate 4 ("Eisa changes a note's `source` field, reloads, verifies the history panel shows the change") will pass for CECE classifier writes but fail for direct YAML edits to the new §A.1 fields (held_by/domain/function/warrant/ikhtilāf).

**This is not a migration regression** (no existing data corrupts; no boot blocks; no IPC breaks). It is a feature-coverage gap: the temporal axis as shipped tracks classifier writes but not direct YAML edits.

**Recommended remediation (PJ-NNN):**
1. **Option α** — change `index_note` from DELETE+INSERT to a real UPSERT (`INSERT ... ON CONFLICT(path) DO UPDATE SET ...`). Trigger then fires on every re-index. Risk: triggers fire on every note save, including body-only edits — the WHEN guard at `history.rs:110-112` filters by sources/content_type/properties_json change so this is mostly safe, but properties_json changes on every body edit (because tag/heading extraction may shift).
2. **Option β** — explicitly capture frontmatter diffs in `index_note` Rust-side (compare old vs new properties_json) and INSERT into `note_state_history` directly when a watched field changed.
3. **Option γ** — keep §B.2 trigger as classifier-only telemetry; add a separate explicit history-write call inside `index_note` for properties_json changes.

Decision is Eisa's. Until then, the §B temporal axis effectively tracks CECE source/content_type changes only — still useful, just narrower than the gap-analysis §6.3 framing implied.

---

*Filed at MIG-022 §N (Migration Path agent). Read against commits c63a2e3, 5c4f1e5, 6ecf8ec, c3c5c66 + the index_note path in search.rs:3045-3054. No fabrication.*
