# MIG-085 §B — Architect + Plan — Unicode Name-Fold (gating) → Maturity Single-Source

**Date:** 2026-06-23 · **Status:** Architected + Planned, awaiting Boss plan-approval before build.
**Supersedes the original §B scope** (a P3 maturity-consistency tidy-up) per the 2026-06-23
investigation, which found the real driver of cross-surface disagreement is a Unicode
case-folding correctness bug. Boss ruling: **fix the accent bug first, then maturity.**

---

## Concept (the horse)

A note's identity-by-name must resolve the same way whether the matcher is written in Rust or
in SQL. Today it doesn't: link **targets** are folded with Rust full-Unicode `.to_lowercase()`,
but note **names** are folded on the SQL side with SQLite's built-in `LOWER()` — which folds
**ASCII A–Z only**. For any note whose title carries a non-ASCII capital (`É Î Ś Đ Š …`) the two
foldings disagree, so the note fails to match its own inbound links. A note must recognise the
links pointing at it; that is the purpose this fixes.

Once names resolve correctly, the *secondary* goal — maturity reads the same inbound count on
every surface — becomes a small, safe tidy-up on top.

---

## Root cause (verified on the live 7,660-note universe)

- SQLite `LOWER()` / `COLLATE NOCASE` are **ASCII-only by design** (SQLite docs: *"works for ASCII
  characters only… load the ICU extension"* for Unicode). `LOWER('Île-de-France') = 'Île-de-france'`
  — the `Î` is left unfolded.
- Link targets (`note_links.target_name`) are stored **Rust-folded**:
  `normalize_arabic_for_search(&m.trim().to_lowercase())` (search.rs:4221) → `'île-de-france'`.
- Aliases (`note_aliases.alias_lower`) are likewise **Rust-folded** (search.rs:4323).
- But the **name side** folds with SQLite `LOWER()`:
  - `incoming_aggregate_assignments` matches `target_name_lower = LOWER(np.name)` (search.rs:1004).
  - `sky_nodes.id` is built `LOWER(name)` — the sky backfill (`sky_backfill.rs:265`) and the
    `note_meta → sky_nodes` INSERT/rename triggers (search.rs:3237, 3284). The
    `STRATUM_SQL_EXPR` / `MATURITY_SQL_EXPR` then match `target_name = sky_nodes.id`.
- **Net:** the name key and the target key diverge exactly on accented capitals → no match.

### Measured impact (13 notes; verified, not estimated)
Every note whose name contains a non-ASCII uppercase letter has real inbound but is recorded as
`incoming_count = 0`:

| note | real distinct inbound | incoming_count |
|---|---|---|
| Śramaṇa | 26 | 0 |
| Île-de-France | 17 | 0 |
| Étienne-Jules Marey | 16 | 0 |
| Île de la Cité | 13 | 0 |
| Émilie du Châtelet | 11 | 0 |
| Étienne-Louis Boullée / Đông Sơn culture | 7 | 0 |
| Étude / Śāriputra | 6 | 0 |
| Abū Ḥanīfa / Charles-Émile Reynaud | 4 | 0 |
| Notre-Dame de l'Épine / Š-L-M | 3 | 0 |

**User-visible today:** 12 of 13 satisfy `incoming_count = 0 AND word_count > 20` → they appear
as **false orphans** in the new Reviewer's 🔗 lens. Sky maturity is `sapling` (should be
evergreen/canonical); stratum understated; Backlinks badge shows 0. The maturity panel + 360
Inspector (Rust-folded) are **correct** — so naive single-sourcing to `incoming_count` would
*regress* them. That premise inversion is why the fold fix gates the maturity work.

---

## Fix strategy (research-validated — WA#5)

Three industry options were compared (sources in SESSION-LOG-2026-06-23 / agent research):
1. **Register a `ulower()` SQL function** — embeds a Rust dependency into the schema
   (triggers/generated columns/expression indexes) with a per-connection-registration landmine,
   and is non-sargable in `WHERE`. Rejected.
2. **Store a Rust-folded key column, match column-to-column** — index-friendly, connection-
   independent, the field's dominant production pattern (Lucene index-time `LowerCaseFilter`,
   Postgres normalized columns, better-sqlite3 guidance). **Chosen** — and it matches the
   convention Constellation already uses for `target_name` / `alias_lower`.
3. **ICU extension** — not in rusqlite `bundled`, ~10–26 MB, per-platform linkage. Rejected for a
   local-first desktop app.

**Chosen fix: add a Rust-folded `note_meta.name_lower` and route every name-side match through it.**
The canonical fold is the byte-identical reuse of the target-side fold:
`fold_name_key(s) = normalize_arabic_for_search(&s.to_lowercase())`. No SQL function, no schema
landmine, index-friendly.

### NFC normalization — deferred, surfaced to Boss (not silently parked — WA#6)
Research flags that `.to_lowercase()` alone does **not** normalize NFC vs NFD (precomposed "é" vs
"e"+◌́). This matters for cross-device sync (macOS filenames are NFD) — relevant to Eisa's
iOS/macOS/Windows setup. **But:** (a) none of the 13 current failures are NFC/NFD — they are
pure ASCII-lower failures; (b) adding NFC correctly requires applying it to **both** sides
(`target_name` + `alias_lower` too), i.e. re-folding every link/alias — a larger reindex than the
name-fold fix. **Recommendation:** ship the name-fold fix now (resolves all 13), and treat
canonical NFC folding across all three key producers as a tracked follow-up (its own §). Raised
with Boss for a ruling on timing.

---

## Blast radius — name-side match sites (to route through `name_lower`)

Verified per-site (Agent 2 list, de-noised — `target_name_lower` generated col + cache.rs:477
backlinks are **NOT** buggy because `target_name` is pre-folded; `LOWER(name)=LOWER(?)` cid_cn
self-consistent ASCII on both sides — left as-is unless proven harmful):

**Must fix (one side Rust-folded, other ASCII):**
- `incoming_aggregate_assignments` (search.rs:1004) — `= LOWER(np.name)` → `= np.name_lower`.
- `sky_nodes.id` producers: `sky_backfill.rs:265` + triggers search.rs:3237, 3284 (and the
  rename `sky_links` cascade 3299, origin-dirty 3343) — derive `id` from `name_lower`.
  This fixes `STRATUM_SQL_EXPR`/`MATURITY_SQL_EXPR` (`target_name = sky_nodes.id`, search.rs:200,
  216, 253, 264, 281, 295) and the per-edge sky triggers (3419–3515) **for free**.
- `incoming_signature` (search.rs:1044) + `resolve_incoming_target_paths` (search.rs:1065) —
  `LOWER(name)` → `name_lower`.
- FTS outgoing-name lookups (search.rs:5656, 5690, 5781, 8115) — verify each compares against a
  Rust-folded value; route through `name_lower` where so.

**Leave as-is (verified not buggy for the accent class):** `target_name_lower` generated column;
`cache.rs:477` backlink IN-list; `LOWER(name)=LOWER(?)` symmetric ASCII matches.

---

## Invariants that must not break
1. **Rule 8** — name_lower is write-time-maintained in `index_note`; reads stay cheap lookups; the
   backfill runs in the background after paint, resumable, stamped.
2. **Reviewer correctness** — after the fix, the 13 accented notes leave the Orphan lens; no
   *real* orphan is hidden (an actually-unlinked note still has name_lower with 0 matches).
3. **Sky View** — accented notes gain their correct maturity/stratum; the only other Sky change is
   the §B.1 `COUNT(*)→COUNT(DISTINCT)` trigger alignment (8 notes evergreen→sapling, the correct
   lower value).
4. **No new orphans/regressions for ASCII notes** — `fold_name_key` is a no-op vs the old
   `LOWER()` for pure-ASCII names (both lowercase ASCII identically), so the ~7,647 ASCII notes are
   byte-identical before/after. (Verified by measurement: only the 13 change.)
5. **Editor-Surface Gate** — this is index/schema only; no Note content/save/lifecycle code. The
   gate is run as a sanity pass but no editor surface is touched.
6. **One canonical fold** — `fold_name_key` reused by the name side AND the existing target-side
   path (extract via a shared fn) so they cannot drift.

---

## Plan (phased; each step landable as one commit + a verification clause)

### §B.0 — Unicode name-fold (the gating correctness fix)

**Step 1 — `fold_name_key` + `name_lower` column (inert).**
Extract the canonical fold into one `pub(crate) fn fold_name_key(&str) -> String`
(= `normalize_arabic_for_search(&s.to_lowercase())`), unit-tested against the accent set
(`Île-de-France → île-de-france`, ASCII no-op, Arabic). Add `note_meta.name_lower TEXT` (nullable),
gated behind a new `schema_versions` module `name_fold` (version 1, written only when backfill
completes). Column added, no reads yet. *Verify:* `cargo test` green; column exists; existing
behaviour unchanged (name_lower NULL everywhere; nothing reads it).

**Step 2 — write-path maintenance.**
`index_note` writes `name_lower = fold_name_key(name)` on every note upsert. *Verify:* save a note,
assert its `name_lower` is the folded form; `cargo test`.

**Step 3 — route name-side matches through `name_lower` (gated).**
Behind `name_fold::is_stamped`, switch the must-fix sites to `name_lower`:
`incoming_aggregate_assignments`; `incoming_signature` / `resolve_incoming_target_paths`; the
`sky_nodes.id` producers (trigger + sky_backfill derive id from `name_lower`); the verified FTS
lookups. Pre-stamp, the old `LOWER()` paths remain (zero-risk rollout, the tag_counts/incoming
pattern). *Verify:* unit tests on a fixture with an accented note assert incoming_count + sky id
match; `cargo test`.

**Step 4 — one-time resumable backfill + reconcile self-heal.**
`name_fold_backfill::maybe_schedule` (mirrors `incoming_links_backfill`): batched, resumable,
populates `name_lower`, then recomputes `sky_nodes.id` + `incoming_count` + re-fires
stratum/maturity for the affected rows, stamps `name_fold`. A `recompute_all_in` reconcile mirrors
the review/tag_counts self-heal. *Verify on the live copy:* a rehearsal asserts
`Île-de-France.incoming_count = 17`, leaves the Orphan lens, sky maturity = correct; ASCII notes
unchanged; <Rule-3 time.

**Step 5 — Boss test (tutorial).** Open the Reviewer → 🔗 Orphan lens no longer lists
`Île-de-France` / `Śramaṇa` / etc.; open one of them → Backlinks shows its real count; Sky View
maturity is correct. (Stage-1 of the staged test.)

### §B.1 — maturity single-source (the original §B, now safe)

**Step 6 — trigger `COUNT(*) → COUNT(DISTINCT source_path)`.** Align `MATURITY_SQL_EXPR`
(and `STRATUM_SQL_EXPR`'s inbound predicates) to distinct-source, matching `incoming_count` by
construction. *Verify:* the 8 boundary notes (carl seashore, tatmadaw, …) recompute to sapling;
a test asserts trigger-maturity == `compute_state(incoming_count)`.

**Step 7 — point the panel + 360 at the DB.** `maturity.rs::compute_note_maturity` and
`inspector360.rs::compute_maturity_for_note` read inbound from `note_meta.incoming_count` (a cheap
indexed lookup) instead of the FS occurrence-count, calling the shared `compute_state`. *Verify:* a
test asserts all four surfaces (Reviewer / panel / 360 / sky trigger) agree on a fixture incl. an
accented note + a multi-link note.

**Step 8 — `/simplify` + Phase-4 audit (invariants / drift / migration-path) + docs (orientation
v-bump, session log, help if user-facing, MoCh) + Boss Stage-2 test.**

---

## Test asserting all surfaces agree (the deliverable the handover asked for)
A Rust integration test builds a fixture universe with: (a) an accented-named note with N inbound,
(b) a note whose single source links it twice, (c) a true orphan. It asserts, for each:
`incoming_count` == distinct-source truth; `compute_state(incoming_count)` ==
`MATURITY_SQL_EXPR` result == `maturity.rs` result == `inspector360.rs` result; and the orphan
lens contains only the true orphan.
