# Session Log — 2026-06-16 → 06-17 (continuous session)

> MIG-079 §C.2 arc. Carries on from 2026-06-15 (which shipped §C.1 write-time `tag_counts`). This session: §C.1 Boss-validated; the **right-sidebar relocation design** decided; **§C.2a write-time backlink counts** built + Boss-validated through a multi-round perf saga; a **pre-existing save-freeze** uncovered and fixed. Branch `main`. Active universe "Eisa Cognitive Knowledge" (7,653 notes).

## §C.1 — Boss-validated (Steps 1–5 PASS)
The write-time `tag_counts` (committed 2026-06-15) was runtime-validated this session: tags total **19,542** correct; backfill stamped; boot `read_tags` **5,658 ms → 4/36 ms** (real boot-perf history tool); live add/remove a tag maintained the count (incl. the de-quote cleanup of importer-mangled tags — add/decrement/prune all exercised); content intact. §C.1 done.

## Right Sidebar → note-context-only — design decided (deferred to its own /migration, after §C)
Boss observation: the right sidebar should be a live extension of the OPEN NOTE, not mixed with universe-wide functions. Two workflows: a **scope audit** (`wf_29860fe0`) classified all 11 tabs (note vs universe) and a **PKM-placement research** pass (`wf_e09ede1a`, web-cited) found where the field homes each evicted function. Boss dispositions: keep note-scoped tabs (Properties, Backlinks, Sky-local [Boss-confirmed], Tasks, Provenance, 360.3D, Tags-this-note) + **note-scoped REDESIGNS** of Knowledge Health, Review Pulse, Source Review (Knowledge-Health and 360.3D stay **distinct**); relocate universe functions (All-tags → Search Hub + Dashboard; Calendar → left launcher; Tasks → with the left Calendar; Cataloger → left; universe Health → Dashboard; universe Review → full-page reviewer). **Decision recorded: `docs/Right-Sidebar-Note-Context-Design-Decision.md`** (commit `bfa66f0a`). Sequencing: its own four-phase `/migration` AFTER §C.2/§C.3. Fixes the Calendar wrong-library + Review `record_note_visit` defects by relocation.

## §C.2 impact analysis (`wf_d9a26cf7`) — the incoming-links verdict
Mapped every consumer of the boot `links` payload. Pivotal finding: **no write-time incoming aggregate existed**; the backlink-count badge inverted the legacy `outgoing_links_json`, which CANNOT parse the 100%-typed `[[type::target]]` wikilinks → it counted **47 backlinks in the whole library** (vs ~413,660 real). So §C.2 needs a write-time incoming aggregate (the §C.2a enabler) before the 234k `read_links` can be deferred off boot (§C.2b). Boss decisions: full mirror; idle pre-fetch; re-point the badge now.

## §C.2a — write-time backlink counts: BUILT + Boss-validated (a 5-round perf saga)
The goal: a `note_meta.incoming_count` (+ types/rank, full MIG-066 mirror) matching the Backlinks panel exactly (`COUNT(DISTINCT source_path)`, alias-aware, status-filtered), maintained write-time, re-pointing the badge. Commits + the arc:

- **`16f03ccb` foundation** — inert incoming columns + the rehearsal harness (`lab/tag-counts/analyze-incoming-links.py`). Reproduce-First: all 7,653 `tags_json`... (incoming) — distinct-source dedupe matters for **1,249** notes, an alias adds a source for **399**; the legacy badge disagrees with the truth on **7,508 / 7,653** notes (~98%).
- **`83cceb90` functional (triggers)** — first build used per-edge triggers. Index discovery: the `OR LOWER(target_name)=…` match **full-scanned** 234k rows PER note (33 CPU-min, killed). Fix: a **VIRTUAL `target_name_lower` generated column** + plain index `idx_nl_tnl` so the name+alias match (a UNION of two branches) SEEKS. Rehearsal: `incoming_count == getBacklinks` for all 7,653, 0 mismatches; recompute ~60 s (background backfill).
- **`7cd192cc` 2nd-boot fix** — Boss-caught: the app showed **0 notes** on the 2nd launch. Root cause: the virtual-column idempotency guard used `column_exists()`/`PRAGMA table_info`, which **hides VIRTUAL columns** → the ALTER re-ran → "duplicate column" → `init_db` looped → 0 notes. Fix: detect via `PRAGMA table_xinfo`. + regression test. (DB never corrupted; the failed ALTER is a clean no-op.)
- **`e5b4eee1` redesign (recompute-affected)** — Boss-caught: saving a 119-link note froze **5–7 s** + typing lag. The per-edge triggers recomputed EVERY target's full aggregate on every save (index_note rebuilds all links even on a text edit). Redesign: move maintenance OFF triggers into a **save-path Rust diff** — capture the note's incoming signature BEFORE index_note, `symmetric_difference` AFTER, recompute ONLY changed targets (text edit → empty diff → zero work). Delete path recomputes ex-targets. reconcile keeps `recompute_all_incoming` as the convergent self-heal.
- **WA#5 cross-check (`wgm336n25`)** — VALIDATED recompute-affected as the **textbook-correct** strategy for a `COUNT(DISTINCT)`+alias+status (non-self-maintainable) aggregate; ±delta would be WRONG (can't honor dedupe; alias/status changes move the count with no edge insert/delete; SQL Server bans `COUNT(DISTINCT)` in incremental views). Keep recompute-affected; the hub-fan-in tail mitigation (durable dirty-set + async) is DEFERRED until measured.
- **`fca3f194` link-skip (pre-existing freeze)** — the redesign fixed §C.2a's own freeze, but a residual lag remained. **Measured** on the copy: rebuilding a 531-link note's links fires the **pre-existing** Sky/stratum/maturity (MIG-001) + outgoing (MIG-066) per-edge triggers → **~40 s** (76 ms without triggers). Independent of §C.2a — `index_note` does DELETE-all+INSERT-all on every save. Fix (textbook dirty-check): **skip the note_links rebuild when the would-be edge set is byte-identical** to what's stored (conservative: rebuild unless annotation/target_cid_cn/source/library all match + all active). Text edit → no link change → no rebuild → no trigger cascade → instant.

### Boss validation (final binary 2026-06-17 05:12)
- Counts correct: A Thousand Ships `incoming_count=4` == its 4-row Backlinks panel; ISBN (identifier) = 5,358 with a full type breakdown. `init_db` runs once (loop gone); 7,653 notes load.
- **Step A** (text-edit on 119-link Monotheism): **instant** (was 5–7 s). PASS.
- **Step B** (remove a link): edge gone + A Thousand Ships `incoming_count` 5→4. PASS.
- **Step C** (content-integrity gate): Focus/tab-switch, body intact; DB `note_meta==note_body==disk`, no contamination. PASS.

## Key lessons (Boss corrections this session)
- **Test instructions must be literal.** Twice I implied Search Hub shows results on open / omitted "type a query." Tighten to exact click/type + expected, no assumptions (Testing Instructions Rule).
- **Measure, don't guess.** The UTC-vs-local timestamp misread (called the logger broken when it wasn't); the pre-existing cascade was *measured* (40 s), not assumed. Every cause stated had a measurement.
- **WA#5 before locking an inventive fix.** Boss's "have you cross-checked?" → the recompute-affected validation (it was the proven pattern, not invention).

## State / remaining
- **Shipped + validated this session:** §C.1, §C.2a, the save-path link-skip, the right-sidebar design decision. All on `main`, pushed.
- **Remaining in §C.2:** **§C.2b** (defer the 234k-row `read_links` off boot — the actual ~11 s boot win; the §C.2a incoming aggregate is the enabler so the badge survives deferral) + **§C.3** (covering index on `note_links`). Boot is still ~27 s until §C.2b lands.
- **Deferred:** the hub-fan-in async dirty-set (only if a measured hub edit shows save latency); the right-sidebar relocation `/migration` (after §C); §C.2a vocab-change re-materialize of `incoming_link_types` (reconcile heals; types are future-use).
- **No user-facing help/manual topic** changed (perf + a correctness fix to backlink counts; no new feature/string).
