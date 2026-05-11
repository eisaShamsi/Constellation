# MIG-022 — Phase 2 Plan

**Phase:** 2 of 4 (`/migration` — Architect → **Plan** → Build → Audit)
**Date:** 2026-05-11
**Predecessor:** `lab/reports/MIG-022-ARCHITECT.md` (Phase 1, filed `04080a2`)
**Function in hand:** Lay out the phase-by-phase build steps for MIG-022 with Eisa's eight Architect decisions incorporated, the WA #5 cross-check applied, and verification clauses per phase. Two new Plan-level decisions surfaced (D-A4 ikhtilāf UX, D-B4 temporal-axis UI surface).

---

## 1 · Architect decisions locked (recap)

| # | Decision | Eisa's pick | Plan implication |
|---|---|---|---|
| **D-A1** | `supersedes`/`contradicts` representation | **β** — typed-link names | §A.2 extends `KNOWN_LINK_TYPES` from 7 → 8 (add `supersedes`; `contradicts` already exists). YAML scalars NOT added for these two. |
| **D-A2** | Warrant display in Cluster A or all warrant deferred | **β** — defer all warrant UI | §A.1 parser ACCEPTS `warrant:` and `warrant_notes:` fields (no schema error) but Properties panel does NOT surface them. They sit inert until the Warrant Research workstream ships. |
| **D-A3** | `taxonomy_version` field | **Implicit** — only add if/when a breaking change forces it | §A.1 parser does NOT add `taxonomy_version`. Stays implicit through MIG-022. |
| **D-B1** | Temporal storage substrate | **SQLite `note_state_history` + triggers on `note_meta`** | §B uses SQLite. Cross-check refinement: JSON-diff column shape (see §2). |
| **D-B2** | Which fields get history | **Only the epistemic fields** | §B trigger fires on changes to `source`, `content_type`, plus the §A.1 epistemic additions. NOT title, NOT tags. |
| **D-B3** | Cluster B in MIG-022 | **IN MIG-022** | §B.1–§B.6 are in-scope. |
| **D-C1** | Warrant classifier | **Workstream-deferred** with explicit commitment to start after MIG-022 closes | Out of MIG-022 scope. New workstream spec (Concept Paper) opens after MIG-022's audit closes. |
| **D-E1** | PJ-043 storage | **(c) frontend i18n keys** | §E.3 moves taxonomy labels to `src/lib/i18n/{locale}.json::cece.taxonomy.{nodeId}`. Rust struct keeps `id`, `parent_id`, `branch`; English label retained as fallback only. |
| **D-E2** | Cluster E bundling | **One MIG with three sub-phases** | §E.1 → §E.2 → §E.3, smallest first. |
| **D-F1** | Cluster F (legacy classifier cleanup) | **§0 housekeeping** | §0.1 reachability + §0.2 deletion. First phase to land. |

---

## 2 · Cross-check incorporated (Working Agreement #5)

The Architect §2.2 / D-B1 storage choice was cross-checked against proven temporal-data patterns: SQL:2011 SYSTEM_TIME (Postgres/DB2) — not available in SQLite. Datomic/XTDB bitemporal — overkill for transaction-time-only queries. Event sourcing — Fowler explicitly recommends a separate audit table for non-financial domains. Per-file Git versioning — wrong shape (can't query field-axis evolution).

**Verdict: SQLite trigger-maintained history table is the right pattern.** Used by Datasette via Simon Willison's `sqlite-history-json`. Refinement adopted in this Plan:

### 2.1 Refinement: JSON-diff column shape (not triples)

The schema in the Architect was sketched as `(note_id, captured_at, axis_changed, old_value, new_value)`. The cross-check shows `(note_id, captured_at, changes_json)` is more durable: when MIG-022 §A.1 adds new epistemic fields, the trigger does NOT need rewriting — it captures whatever changed via `json_object('field_name', NEW.field_name, ...)`.

Schema for §B.1 becomes:

```sql
CREATE TABLE IF NOT EXISTS note_state_history (
    history_id INTEGER PRIMARY KEY AUTOINCREMENT,
    note_id INTEGER NOT NULL REFERENCES note_meta(note_id) ON DELETE CASCADE,
    captured_at INTEGER NOT NULL,  -- Unix epoch ms
    changes_json TEXT NOT NULL     -- {"source": {"old": ["testimony"], "new": ["inference"]}, ...}
);
CREATE INDEX IF NOT EXISTS idx_note_state_history_note_time
    ON note_state_history(note_id, captured_at DESC);
```

(Note: the `ON DELETE CASCADE` is intentional only if `note_meta` deletes are themselves rare — see §2.4 below.)

### 2.2 Refinement: trigger guard against no-op writes

Every UPDATE to `note_meta` would otherwise fire the trigger even when no epistemic field changed. Guard with `WHEN`:

```sql
CREATE TRIGGER IF NOT EXISTS note_state_history_au
AFTER UPDATE ON note_meta
WHEN OLD.epistemic_source IS NOT NEW.epistemic_source
  OR OLD.epistemic_content_type IS NOT NEW.epistemic_content_type
  OR OLD.epistemic_held_by IS NOT NEW.epistemic_held_by
  OR OLD.epistemic_function IS NOT NEW.epistemic_function
  -- ... add per epistemic field
BEGIN
    INSERT INTO note_state_history (note_id, captured_at, changes_json)
    VALUES (
        NEW.note_id,
        CAST(strftime('%s', 'now') AS INTEGER) * 1000,
        json_object(
            'source', CASE WHEN OLD.epistemic_source IS NOT NEW.epistemic_source
                          THEN json_object('old', OLD.epistemic_source, 'new', NEW.epistemic_source)
                          ELSE NULL END,
            'content_type', CASE WHEN OLD.epistemic_content_type IS NOT NEW.epistemic_content_type
                                THEN json_object('old', OLD.epistemic_content_type, 'new', NEW.epistemic_content_type)
                                ELSE NULL END
            -- ...
        )
    );
END;
```

(The `WHEN` clause may seem redundant with the per-field CASE inside, but it's the SQLite footgun fix from the cross-check — it prevents trigger evaluation entirely on no-op writes.)

### 2.3 Refinement: bulk-update protocol for §B.3 backfill

Per the cross-check sharp edge: bulk inserts via the trigger are slow + can blow up if a 7,600-note universe fires 7,600 triggers in sequence.

§B.3 backfill protocol:
1. `BEGIN IMMEDIATE`
2. `DROP TRIGGER note_state_history_au` (temporarily)
3. Bulk-insert one `created` event per existing note: `INSERT INTO note_state_history SELECT note_id, modified_at AS captured_at, json_object('source', json_object('old', NULL, 'new', source), ...) FROM note_meta`
4. Re-create the trigger
5. `COMMIT`
6. Tauri progress event during the backfill so the user sees status

### 2.4 Refinement: soft-delete in `note_meta`

The cross-check warns that `ON DELETE CASCADE` to history is the documented anti-pattern (you lose the trail when notes are deleted). Two options:
- **(α)** keep `ON DELETE CASCADE` and accept history loss on delete (simplest; matches today's hard-delete behavior in `note_meta`)
- **(β)** add a `deleted_at INTEGER` column to `note_meta` for soft-delete; history persists; CASCADE never fires

**Decision deferred to §B.1 — surface as Plan-level micro-decision when §B.1 lands.** Most likely (α) for MIG-022 (matches existing behavior); future MIG could move to (β) if recovery becomes a feature.

---

## 3 · Phase order + dependency graph

```
§0 — Cleanup (Cluster F)              ───────► verification: cargo build + test pass
                                                 │
§D — UA partial-frontmatter (PJ-040)  ───────►   │ Boss-Test Gate 1 (small wins land + i18n.1)
                                                 │
§E.1 — Confidence enum i18n (PJ-042)  ───────►───┘
                                       │
§E.2 — Cataloger reasoning i18n       ───────►   Boss-Test Gate 2 (full i18n parity)
                                       │
§E.3 — Taxonomy labels i18n (PJ-043)  ───────►───┘
                                       │
§A.1 — Frontmatter schema             ───────►   │
§A.2 — Typed-link extension (+supersedes) ────►  │ Boss-Test Gate 3 (YAML metadata)
§A.3 — Properties panel UI            ───────►   │
§A.4 — i18n + help docs               ───────►───┘
                                       │
§B.1 — note_state_history schema      ───────►   │
§B.2 — Trigger + WHEN guards          ───────►   │
§B.3 — Backfill (existing notes)      ───────►   │ Boss-Test Gate 4 (temporal axis)
§B.4 — Query API IPC                  ───────►   │
§B.5 — UI surface (per D-B4)          ───────►   │
§B.6 — i18n + help + User Manual      ───────►───┘
                                       │
§N — Final integration audit          ───────►   Boss-Test Gate 5 (close-out)
                                                  + 3-agent audit + MIG-022 ships
```

**Why this order:**
- §0 first because it's pre-work that touches `classifier/` (cluster F) before any other work in `cece/`.
- §D second because surgical (½ day, single test), gives Eisa an early win on the Arabic-frontmatter case.
- §E before §A because §E is breadth-not-depth (i18n volume) and unblocks the i18n-parity invariant before §A adds new fields.
- §A before §B because §B's trigger fires on changes to §A's new fields; cleaner to land §A first so §B's tests cover them.
- §B last (largest, most invasive) before §N close-out.

**Estimated effort by phase:** §0 = ½ day · §D = ½ day · §E.1 = ½ day · §E.2 = 3-5 days · §E.3 = 5-7 days · §A.1 = 1-2 days · §A.2 = 1 day · §A.3 = 2-3 days · §A.4 = 2 days · §B.1 = ½ day · §B.2 = 1 day · §B.3 = 1-2 days · §B.4 = 1 day · §B.5 = 2-4 days (dep on D-B4) · §B.6 = 2 days · §N = 1 day. **Total ≈ 5-7 weeks of agent time.**

---

## 4 · Per-phase steps + verification clauses

### §0 — Cleanup (Cluster F, audit F1)

**Goal:** Verify reachability of `classifier/*` modules; delete what's truly dead.

#### §0.1 — Reachability analysis
- Grep every `pub` symbol exported by `classifier/source_definitions.rs`, `classifier/tier1_embedding.rs`, `classifier/tier1_rules.rs`, `classifier/scan_job.rs`, `classifier/correction_log.rs`, `classifier/mod.rs`.
- For each `pub` symbol: search the codebase for callers OUTSIDE `classifier/`. Build a reachability table.
- If `classifier_suggest_for_note` (the `#[tauri::command]`) has frontend callers, it's reachable — keep it OR rewire its frontend callers to the CECE equivalent first.
- Output: `lab/reports/MIG-022-§0-REACHABILITY.md` with the reachability table.
- **Verification:** the reachability report is a checked artifact. Eisa reviews before §0.2 deletion.

#### §0.2 — Safe deletion
- Delete files / functions confirmed unreachable in §0.1.
- Update `mod.rs` to remove the dead module declarations.
- Run `cargo build` (must succeed) + `cargo test --package constellation-tauri --lib` (must succeed; 92 cece tests + others should still pass).
- **Verification:** `cargo build` clean + `cargo test` count unchanged from V3-§11 baseline (no test deleted accidentally).

**Commit:** `MIG-022 §0 — legacy classifier cleanup (audit F1)`

**Boss-test:** None (internal cleanup; no user-visible change). Verify the next NSIS rebuild still produces a working binary.

---

### §D — PJ-040: partial UA short-circuit

**Goal:** When frontmatter has only one axis populated, UA short-circuits THAT axis and the OTHER axis falls through to normal weighted-vote synthesis.

#### §D.1 — Refactor `synthesis.rs::user_authority_short_circuit`
- Change the function signature to take per-axis short-circuit flags: `user_authority_short_circuit(ua, all_trails, catalogers_voiced, short_circuit_horizontal: bool, short_circuit_vertical: bool)`.
- Inside the function: when `short_circuit_horizontal == false`, build the horizontal AxisDecision via `vote_on_axis(...)` from the OTHER catalogers' trails (filter out UA's vote on horizontal). Same for vertical.
- Synthesis method label becomes `"user_authority_partial_short_circuit"` when only one axis was UA-voiced; `"user_authority_short_circuit"` when both were.
- The caller (`synthesize` in synthesis.rs:92-98) determines per-axis flags by checking UA's trail — `ua.horizontal.is_empty()` means UA didn't voice horizontal, etc.

#### §D.2 — Tests
- Update existing test `user_authority_short_circuits` to use both-axes UA frontmatter (today's behavior).
- Add new test `user_authority_partial_short_circuit_horizontal_only`: UA voices horizontal, Linguistic + Structural voice vertical. Assert `result.horizontal.primary.as_deref() == Some("testimony")` (UA pick) AND `result.vertical.primary.as_deref() == Some("...")` (synthesized from non-UA catalogers) AND `result.synthesis_method == "user_authority_partial_short_circuit"`.
- Add symmetric test `user_authority_partial_short_circuit_vertical_only`.
- Run `cargo test`. Must reach 95 cece tests (92 baseline + 3 new).

**Commit:** `MIG-022 §D — PJ-040 partial UA short-circuit (synthesis) + 3 tests`

**Boss-test Gate 1 (after §D + §E.1 land):** Eisa re-tests on `الخط العربي` (the Gate-2 Stage 6 failing case). Tutorial:
1. Open `الخط العربي` (frontmatter has `sources: testimony/authoritative` only).
2. Re-classify the note (right-click → Classify Sources).
3. The Source Review card now shows BOTH a SOURCES section (= the UA pick) AND a CONTENT TYPE section (= synthesized from non-UA catalogers).
4. The trail shows UA short-circuited horizontal only; vertical was voted by Linguistic / Structural / Semantic.
5. Synthesis method label in the trail says `user_authority_partial_short_circuit`.

---

### §E — Engine-output i18n bundle

#### §E.1 — Confidence enum i18n (PJ-042)
- Add 4 keys to `en.json`: `cece.confidence.high`, `cece.confidence.medium`, `cece.confidence.low`, `cece.confidence.abstain`. Mirror to `ar.json`.
- Add a frontend helper `confidenceLabel(c: string): string` that looks up `cece.confidence.${c}` and falls back to raw enum on missing key.
- Source Review trail renders `[{confidenceLabel(t.self_reported_confidence)}]` instead of `[{t.self_reported_confidence}]`.
- Backfill 13 other locales via Python batch (V3-§10.D pattern). Each locale's translation supplied inline.

**Commit:** `MIG-022 §E.1 — PJ-042 Confidence enum i18n (15 locales)`

**Boss-test (Gate 1 sub-step):** Switch interface language to Arabic (or German). Open any classified note. Check the trail row labels — `[high]` should now read e.g. `[عالٍ]` in Arabic.

#### §E.2 — Cataloger reasoning prose i18n (PJ-041)
- Refactor each cataloger's `build_reasoning(...)` (and synthesis.rs's `user_authority_short_circuit` reasoning) to emit a `(template_key, params)` tuple alongside the existing `reasoning: String`.
- New struct field on `ReasoningTrail`: `reasoning_template: Option<ReasoningTemplate>` where `ReasoningTemplate { key: String, params: serde_json::Value }`. Existing `reasoning: String` field stays as fallback.
- Add ~90 reasoning template keys to `en.json` under `cece.reasoning.*` with placeholder syntax. Mirror to `ar.json`.
- Frontend Source Review trail: when `t.reasoning_template` present, render `$t(t.reasoning_template.key, t.reasoning_template.params)`; otherwise fall back to raw `t.reasoning` (covers legacy composite_json blobs).
- Backfill 13 other locales (~90 keys × 13 ≈ 1,170 translations) via parallel agents.

**Commit:** `MIG-022 §E.2 — PJ-041 cataloger reasoning prose i18n (15 locales)`

**Boss-test (Gate 2):** Switch interface to a non-en/non-ar locale (German). Open any classified note. The per-cataloger reasoning rows render in German. (Plus the §E.1 `[high]` chip.)

#### §E.3 — Taxonomy labels i18n (PJ-043)
- Add `cece.taxonomy.{nodeId}` keys to `en.json` for all ~225 vertical + ~30 horizontal taxonomy IDs. Mirror to `ar.json` (en label → English, ar label → Arabic from existing struct).
- Refactor frontend `labelForId()` (currently reads VerticalNode.en/ar via wrapper) to: `$t('cece.taxonomy.' + id, { default: englishLabel })` where `englishLabel` is the Rust-supplied fallback for missing-key safety.
- Rust side: keep `VerticalNode { id, parent_id, branch }` plus a new `en_fallback: &'static str` for the missing-key safety net. Drop `ar` field from struct (now lives in i18n keys).
- Backfill 13 other locales (~255 nodes × 13 ≈ 3,315 translations) via parallel agents (V3-§10.D scaling pattern).
- Add `cargo test` that loads each locale's `cece.taxonomy.*` block and checks every taxonomy ID has a key (warn-only for non-en/non-ar locales).

**Commit:** `MIG-022 §E.3 — PJ-043 taxonomy node labels i18n (15 locales)`

**Boss-test (Gate 2 conclusion):** German-UI screenshot. Source Review CONTENT TYPE list values render in German (e.g. "Konzept" not "concept"). Sibling Disambig chips render in German.

---

### §A — YAML metadata extensions (without warrant display)

#### §A.1 — Frontmatter schema additions
- Locate the frontmatter parser (Plan-phase verification per Architect §8 #3 — likely `src-tauri/src/notes/frontmatter.rs` or similar; grep confirms before edit).
- Add field handlers for the §6.1 schema MINUS warrant display per D-A2.β:
  - `held_by: String` (default: "user")
  - `domain: Vec<String>`
  - `function: String` (validated against enum: reference / seed / actionable / shipped)
  - `provenance_civilization: String`
  - `updated_at: ISO date string`
  - `ikhtilāf: Vec<{ school: String, position: String }>`
  - `warrant: String` (parsed, stored, NOT surfaced)
  - `warrant_notes: String` (parsed, stored, NOT surfaced)
- These are ALL optional. Existing notes without them work unchanged.
- **No** addition of `supersedes:` or `contradicts:` as YAML scalars per D-A1.β — those go through §A.2.
- **No** addition of `taxonomy_version:` per D-A3.

**Commit:** `MIG-022 §A.1 — frontmatter schema additions (held_by, domain, function, ikhtilāf, etc.)`

#### §A.2 — Typed-link extension: add `supersedes`
- Extend `KNOWN_LINK_TYPES` (in `src/lib/libraries/store.ts` near LINK_TYPE_NAMES) from 7 → 8 by adding `'supersedes'`.
- Mirror in Rust if there's a Rust-side enum (Plan-phase verification — grep `KNOWN_LINK_TYPES`).
- Add `supersedes` to `linkPills.fill` + `linkPills.text` defaults in `DEFAULT_SETTINGS` (pick a color — recommend dark-purple or amber to distinguish from existing 7).
- Add `supersedes` to TYPED_LINK_TYPES (livePreview parser).
- Add `cece.linkType.supersedes` i18n key + 14-locale backfill.
- Update Living Link Architecture docs to reflect 8 typed-link names.
- **Verification:** create a note with `[[supersedes: target-note|reason]]`, confirm the link renders with the new pill, the target note shows backlink with the supersedes label.

**Commit:** `MIG-022 §A.2 — supersedes typed-link added (8th type)`

#### §A.3 — Properties panel UI
- Add field rows to the Properties panel for: `held_by` (text), `domain` (list-of-tags), `function` (dropdown), `provenance_civilization` (text), `updated_at` (date), `ikhtilāf` (depends on D-A4 — see §6 below).
- All rows hidden by default (collapsible "Epistemic" section in Properties panel). User opens the section to fill in.
- `warrant` / `warrant_notes` deliberately NOT surfaced in §A.3 (D-A2.β).
- **Verification:** open a note, expand the Epistemic section, fill in `held_by: "al-Shāfiʿī"`, save; reload note, verify the value persists in frontmatter on disk.

**Commit:** `MIG-022 §A.3 — Properties panel UI for epistemic metadata fields`

#### §A.4 — i18n + help docs + User Manual
- Add labels for all new field rows to `cece.properties.*` in en.json + ar.json. 13-locale backfill.
- New help topic `docs/help.uConstellation.World/Epistemic Metadata/Epistemic Metadata.md` documenting each field with examples.
- New chapter in `docs/User Manual.md` (`## 10c. Epistemic Metadata`).
- 14-locale translations of the help topic and chapter (V3-§10 pattern).

**Commit:** `MIG-022 §A.4 — i18n + help + User Manual chapter for epistemic metadata`

**Boss-test Gate 3 (after §A.1 → §A.4 land):** Eisa creates a note with the new metadata, verifies Properties panel surfaces them, switches locale, verifies labels translate. Help topic discoverable in en + ar + 1 other locale. User Manual chapter present in en + ar.

---

### §B — Temporal axis (the big one)

#### §B.1 — Schema: `note_state_history` table
- Add `note_state_history` table to `init_db` per §2.1 of this Plan (JSON-diff column shape).
- Add the index `idx_note_state_history_note_time`.
- Add the table to the migration sentinel — first-boot users get the table via `init_db`; existing-DB users get it via the next `init_db` migration step.
- **Open micro-decision** — soft-delete vs CASCADE (§2.4): default to (α) `ON DELETE CASCADE` for MIG-022.

**Commit:** `MIG-022 §B.1 — note_state_history table + index (init_db migration)`

#### §B.2 — Trigger: maintain history on note_meta UPDATE
- Add the `note_state_history_au` AFTER UPDATE trigger per §2.2 of this Plan.
- WHEN guard checks every epistemic field tracked.
- changes_json populated via `json_object` per-field with NULL when unchanged.
- Pre-cache trigger creation in `init_db` (once per boot, not per write).
- **Test:** Rust integration test that updates `note_meta.epistemic_source` and asserts a row appears in `note_state_history` with the right `changes_json`.

**Commit:** `MIG-022 §B.2 — note_state_history trigger (write-time derivation)`

#### §B.3 — Backfill: seed `created` events for existing notes
- Run once at boot if `note_state_history` is empty AND `note_meta` has rows.
- Use the cross-check protocol (§2.3): `BEGIN IMMEDIATE` → `DROP TRIGGER` → bulk INSERT → re-create trigger → `COMMIT`.
- Tauri progress event during backfill so the user sees status: `"Building epistemic history for {n} notes..."`.
- Resumable (per CLAUDE.md SO #6 first-time-population rule): if interrupted mid-backfill, the next boot picks up from where it left off (use a sentinel row or partial-progress check).

**Commit:** `MIG-022 §B.3 — first-boot history backfill (resumable, progress events)`

#### §B.4 — Query API IPC
- New IPC `cece_get_note_history(note_path: String) -> Vec<HistoryEvent>` returning all events for that note in chronological order.
- New IPC `cece_query_history(filter: HistoryFilter) -> Vec<HistoryMatch>` for cross-note queries: `{ axis: Option<String>, since: Option<i64>, until: Option<i64>, change_type: Option<"increase"|"decrease"> }`. Returns `{ note_path, captured_at, changes_json }` rows.
- Both IPCs use parameterized queries; no string interpolation.
- **Test:** Rust integration test for both IPCs against a seeded `note_state_history`.

**Commit:** `MIG-022 §B.4 — query API IPC (cece_get_note_history + cece_query_history)`

#### §B.5 — UI surface (per D-B4 — Plan-level decision needed)
- See §6 below — I propose three options for D-B4. Once Eisa picks, this phase has concrete steps. Until then, this phase is a placeholder.

**Commit:** `MIG-022 §B.5 — temporal-axis UI surface ({per-D-B4-pick})`

#### §B.6 — i18n + help + User Manual
- Frontend chrome strings under `cece.history.*`.
- Help topic `docs/help.uConstellation.World/Note History/Note History.md`.
- User Manual chapter `## 10d. Note History (Epistemic Timeline)`.
- 14-locale translations (V3-§10 pattern).

**Commit:** `MIG-022 §B.6 — i18n + help + User Manual chapter for note history`

**Boss-test Gate 4 (after §B.1 → §B.6 land):** Eisa changes a note's `source` field, reloads, verifies the history panel shows the change. Eisa changes it again 30 seconds later, verifies the second event also shows. Boss-test specific stages depend on the §B.5 UI surface choice.

---

### §N — Final integration audit + close-out

- 3 parallel agents (invariants / drift / migration-path), same shape as V3-§11.
- Invariants check: every cross-cutting invariant from MIG-022-ARCHITECT.md §3 still holds (file-over-app, write-time derivation, performance rules 1+8, editor parity, Living Link, CECE 6-cataloger contract, i18n parity, reliability data continuity, additive migration, no boot/typing regression).
- Drift check: any new guards added in MIG-022 that other parts of the codebase don't yet know about.
- Migration-path check: 7 scenarios analogous to V3-§11's (first-boot, MIG-021v3 upgrade, history table empty / partial / populated, mid-backfill interrupt, settings flag back-compat, i18n locale switching, rollback to MIG-021v3).
- Consolidated audit report: `lab/reports/MIG-022-AUDIT.md`.
- AT RISK fixes land in the close-out commit.
- Orientation v-bump (probably v2.00 — first major rev since CECE) marking "MIG-022 ships".
- Pending Jobs bump documenting any new PJs filed during the cascade.
- Final commit + push.

**Boss-test Gate 5:** the 3-agent audit doubles as Eisa's verification — sign-off after audit report + close-out commit lands.

---

## 5 · Boss-test gates summary

| Gate | After phases | Verifies |
|---|---|---|
| 1 | §0 + §D + §E.1 | Cleanup safe + UA partial fix on الخط العربي + Confidence enum i18n |
| 2 | §E.2 + §E.3 | Full engine-output i18n parity (German screenshot test) |
| 3 | §A.1 → §A.4 | YAML metadata fields surface in Properties panel + persist + translate |
| 4 | §B.1 → §B.6 | Temporal axis tracks epistemic field changes + queries work |
| 5 | §N | Final audit + sign-off |

---

## 6 · New Plan-level decisions for Eisa

### D-A4 (NEW) — `ikhtilāf` Properties panel UX

The `ikhtilāf:` field is a list of `{ school, position }` objects. Most Properties panel rows handle scalars or simple list-of-strings. Nested objects need a custom widget. Three options:

| Option | What | Pro | Con |
|---|---|---|---|
| **(α)** Full widget | Custom Properties row: a tabbed list with "Add school" + per-row inputs for school + position | Most polished, no YAML knowledge needed | ~2 days extra UI work |
| **(β)** Raw YAML edit | Properties panel hides nested objects; shows a "Edit raw YAML" button for ikhtilāf only | Simplest, ships in §A.3 | User must know YAML syntax |
| **(γ)** Read-only display | Properties panel shows ikhtilāf rendered nicely (read-only); editing happens by editing the .md file directly | Middle ground | Editing UX poor |

**Recommendation: (β) for MIG-022; (α) as a future polish PJ.** Ikhtilāf is a niche field most users won't touch; raw YAML is honest. The Properties panel UX work for the polished widget can be a focused mini-MIG later.

### D-B4 (NEW) — Temporal-axis UI surface

The §B.5 phase needs a concrete UI surface. Three options:

| Option | What | Pro | Con |
|---|---|---|---|
| **(α)** Side-panel "History" | New right-sidebar panel showing chronological events for the active note. Add to PanelId enum + panelPlacements. | Familiar pattern (matches Backlinks/Outgoing) | One more sidebar panel |
| **(β)** Sight v3 overlay | Timeline-view layer on top of Sight v3 — color stars by "recently changed" or "certainty trajectory" | Visually striking; uses existing Sight infra | Touches Sight v3 (PJ-038 trajectory); risk of slowdown |
| **(γ)** Search filter only | No new panel; expose history via search filter chips ("changed in last 30 days", "certainty dropped") in the existing search/SearchHub | Smallest surface; no new panel; reuses search infra | Less discoverable |

**Recommendation: (α) Side-panel.** Matches existing pattern (Backlinks panel was the precedent). Sight overlay is exciting but couples MIG-022 closure to Sight v3's stability. Search filter alone is too hidden for a new feature.

But this is genuinely Eisa's design call — surface and pacing.

---

## 7 · Cross-cutting invariants tracked through every phase

(Same 10 from MIG-022-ARCHITECT.md §3. Repeated here for completeness — every phase's verification clause checks the relevant subset.)

1. File-over-app — `.md` is source of truth.
2. Performance Rule 1 — zero perceptible keystroke lag.
3. Performance Rule 8 — write-time derivation for every new derived view (history table is the canonical example here).
4. Editor Parity — new metadata UI works in NotePane and any future editor view.
5. Living Link Architecture — typed-link extension in §A.2 respects the 7-now-8 contract.
6. CECE 6-cataloger contract — §D refactor preserves shape.
7. i18n parity — every new string ships en + ar at minimum; 14-locale backfill follows.
8. Reliability data continuity — §E.2 refactor preserves every existing `composite_json` field.
9. Additive migration — every phase lands so existing universes work unchanged.
10. No boot/typing regression on Eisa's primary universe (~7,600 notes).

---

## 8 · What this Plan does NOT cover (future MIGs)

- **Cluster B advanced queries** beyond the §B.4 IPCs (e.g., "stance changes that match a search term" or "evolution of a tag's note set"). Land in a future polish MIG.
- **Cluster C — Warrant Research workstream.** Eisa committed: starts after MIG-022 closes. Separate Concept Paper. Not in MIG-022 scope.
- **Bitemporal extension** (transaction-time vs. valid-time) — possible future MIG if scholarly-audit use case emerges.
- **Soft-delete in `note_meta`** for history preservation across deletes — option (β) of §2.4; not in MIG-022; would be a focused mini-MIG if Eisa wants delete-recovery.
- **Polished `ikhtilāf` Properties panel widget** — D-A4 (α); future PJ.
- **Bookmarks/saved queries for the temporal-axis history** — UX polish; future PJ.

---

## 9 · Risk register

| Risk | Mitigation |
|---|---|
| §0 deletes something that's actually reachable (silent test break) | §0.1 reachability report is a checked artifact; Eisa reviews before §0.2 |
| §E.2 cataloger refactor breaks composite_json shape (reliability data corruption) | New `reasoning_template` field added ALONGSIDE existing `reasoning: String`; legacy data still readable |
| §A.2 typed-link extension breaks LinkLifecycle dedupe (project_link_lifecycle_dedupe_fix.md territory) | §A.2 commit includes a regression test for LinkLifecycle on the new `supersedes` type |
| §B.2 trigger fires on bulk migrations (e.g., importing a vault) and explodes write time | Backfill protocol §2.3 handles known migrations; future bulk imports document the same protocol |
| §B.3 backfill on 7,600-note universe takes minutes (similar to MIG-013 boot-blocking issue) | Tauri progress events; resumable; user sees status |
| 14-locale i18n backfill (§E.1, §E.2, §E.3, §A.4, §B.6) drains agent budget | Per V3-§10 pattern: parallel agents per language family; disclaimer headers; treat as canonical en + AI-translated rest |
| MIG-022 wall-clock (~5-7 weeks) means context resets along the way | SO #6 orientation bumps + session log discipline + every commit self-contained |

---

## 10 · Two questions for Eisa before Phase 3 Build cascade opens

1. **D-A4** — `ikhtilāf` UX: (α) full widget / (β) raw YAML / (γ) read-only display? Recommended (β).
2. **D-B4** — Temporal-axis UI surface: (α) side-panel / (β) Sight overlay / (γ) search filter? Recommended (α).

Once both answered, **Plan-Approval-Equals-Build-Approval** kicks in: I cascade through §0 → §D → §E.1 → §E.2 → §E.3 → §A.1 → §A.4 → §B.1 → §B.6 → §N autonomously, stopping only at the five Boss-test Gates above and on genuine architectural surprise (per the State-the-Function-in-Hand and Stop-on-Correction rules).

---

*Filed at MIG-022 Phase 2 (Plan). Awaiting Eisa's two §10 answers + plan approval to open Phase 3 (Build).*
