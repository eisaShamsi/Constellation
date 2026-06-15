# Handover — next session: MIG-079 §B (safeBootMode + editor baseline) → §C (the 30 s graph WTD)

**Date prepared:** 2026-06-15 (evening) · **Branch:** main (all pushed through `256d7c5f`) · **Active universe:** "Eisa Cognitive Knowledge" (`E:\Constellation Universes\Eisa Cognitive Knowledge`, 7,653 notes, 1.83 GB).

**Read first:** the highest orientation version (`docs/Constellation Orientation & Onboarding v2.83.md`), then `docs/concept-papers/00-MASTER-Bring-Up-Charter-and-Checklist.md` (esp. **§7 Debt Register**), then `docs/MIG-079-Plan.md`, then this file.

---

## Where we are (all SHIPPED + Boss-validated this session)
- **MIG-078 §B1** (`fe01e3db`, `44a230fd`, `a7590837`) — thundering-herd `init_db` race fixed (init-mutex); frontmatter parser root-cause fixed (line-anchored `\n---` via shared `split_frontmatter`); the 26–41 s `cid_cn` sweep eliminated (`cid_cn=''` probe + force-reindex). Data-proven; 4-lens audit PASS_WITH_NOTES.
- **The bring-up program OPENED** — `docs/concept-papers/`: core Constellation paper + charter/template (Right-click §5 + Multilingual §6) + **31 per-function papers** + the **app-wide Debt Register** (§7). The concept papers are the canonical per-function reference + the bring-up checklist.
- **MIG-079 §A** (`256d7c5f`) — single-owner activation: idempotency guard in `set_active_universe` + second-screen display-only. **Verified: `init_db` runs ONCE** (was twice); Boss: "way better, no freezes."
- **Tasks reframed** (Boss ruling) — "open epistemic loops" (Tension→Synthesis→Conviction); kept, not removed.

**Binary:** `src-tauri/target/release/constellation.exe` (mtime 2026-06-15 20:39; rebuild after any change — `npm run build` THEN `cargo build --release` for any frontend change, verify `build/` has the new string).

---

## THE TASK: continue MIG-079 (Plan: `docs/MIG-079-Plan.md`)

### §B — `safeBootMode` + missing gates (next)
- Add `appSettings.safeBootMode` (default false), read once at boot; gate the **4 unconditional boot IPCs** behind it: `cache_boot_snapshot_graph`, the `federation:ready` listener, `listFiveActsNotes`, `getFederationWarnings` (`src/routes/+layout.svelte`).
- Add the **missing `enabledFeatures` gates** the audit found (Search Hub has NONE; also Quick Switcher, Knowledge Health, Tension/Provenance, Forge/Canvas, Second Screen, Style Setter).
- **Goal:** with `safeBootMode` on → an **editor-only boot**; capture the clean baseline (paint/hydrated/graph_ready) as the regression harness. This is the minimal-mode baseline the whole bring-up measures against.

### §C — the boot-graph Write-Time Derivation (the 30 s; Boss decisions locked)
- **Tags:** maintain a `tag_counts(tag, n)` summary **in `index_note` (Rust ±delta)** + resumable backfill (gate `schema_versions.tag_counts`) + flip `read_tags` (cache.rs:1027) to read it. 5.6 s → <1 ms.
- **Links:** **defer** the 234k-row `read_links` (cache.rs:924) off the `graphReady` critical path — boot reads persisted `sky_links` + `note_meta.outgoing_*`; the full edge list loads lazily on first graph/panel open.
- **+ covering index** on `note_links`. **Verify:** `graph_ready_ms` 32.5 s → <5 s; search-identity preserved; per-keystroke save latency unchanged.

### §D — phase-by-phase bring-up
Re-enable each function in dependency order (concept-papers §5) only when its paper's §10 checklist passes: the write-time cure for its Rule-8 violation + the shared right-click menu + i18n fixes + its gate. The **confirmed defects** (Calendar day-click always opens TODAY = `get_daily_note_path` ignores the date; Tasks `toggle_task` bypasses the Editor reindex; Review-Pulse dead `record_note_visit`; 6 Command-Palette no-op stubs) get fixed in their function's phase.

---

## Don't forget
- **Reproduce-First + the rules** ("commit to auditing and research" — Boss's standing instruction this session): measure before optimizing; WA#5 research before inventive fixes; `/migration` four phases with the Phase-4 audit; the Editor-Surface Gate where the write path is touched (§C).
- The **19 cleaned files** (~145 MB) still await a reflect into the DB — boot is walk-free (MIG-067), so they reflect via Settings → Rebuild Index or the §BL.2/§BL.3 controlled reconcile, NOT a normal boot. (MIG-078 §BL.2/§BL.3 remain after MIG-079.)
- The §B1 deferred PJs: active-index FTS5 optimize; the 4 naive `find("---")` copies in `libraries.rs` (preview/tag surfaces).

---

## ADDENDUM (session 2 — §A + §B SHIPPED+validated; §C.1 is the precise pickup)

### Measured ground truth (the boot-perf history TOOL — `boot-perf.history.jsonl`, append-only, never overwritten; reader `lab/boot-perf/read-boot-history.py`)
Built because `boot-perf.latest.json` kept only the last boot, so every cold diagnosis was inference. Measured on the live universe (3 boots captured):

| boot | mode | tree ready | fully ready | where the time is (MEASURED) |
|---|---|---|---|---|
| cold-minimal | editor only | 1.7 s | **1.7 s** | DB read only 0.65 s (ensure_db 172 + read_notes 295) |
| warm-full | everything | 0.5 s | 5.6 s | graph: read_links 0.4 s + read_tags 0.06 s + queue 1.2 s |
| cold-full | everything | 1.5 s | **33 s** | graph: read_links **11.3 s** + read_tags **5.7 s** + queue **12 s** |

**Conclusion (measured, not guessed):** the editor spine is fast cold AND warm (~1.7 s / ~0.6 s). The **entire** boot cost — cold 33 s / warm 5.6 s — is the **graph snapshot reads** (`note_links` 234k rows + `tags`). The DB open + note-list read is ~0.5 s regardless. A prior "13 s cold (minimal)" claim was on the pre-tool binary, UNMEASURED, and did **not** reproduce — do not chase it. **§C is the whole fix, cold + warm.** (Correction logged: an earlier "the 13 s is the cold 1.83 GB file read" was a GUESS and WRONG — the measurement disproved it. Eisa's standing order: never state a cause without a measurement behind it.)

### §A SHIPPED+validated (`256d7c5f`): single-owner activation — `init_db` now runs ONCE (was twice). Boss: "no freezes."
### §B SHIPPED+validated (`3d115ef0`/`bb8ab1bf`/`c8f7ff9a`): `safeBootMode` switch (gates the 4 satellite boot IPCs) + the editor baseline measured + the append-only boot-history tool.

### §C.1 — write-time `tag_counts` (THE PICKUP). VERIFIED, designed, NOT yet built:
- **SAFETY VERIFIED:** `index_note`'s UPSERT (search.rs:4266) is the SOLE writer of `note_meta.tags_json` (grep-confirmed; rename moves path not tags; Add-tag routes through reindex). So a Rust ±delta is COMPLETE. The delete decrement goes in `reindex_delete_note` (search.rs:7217, which already reads old state before deleting — mirror it for tags).
- **DESIGN:** `tag_counts(tag TEXT PRIMARY KEY, n INTEGER)` in init_db; Rust ±delta (old multiset → new multiset) in those 2 paths, **gated on `schema_versions.tag_counts`**; a **non-blocking own-connection backfill** (aggregate `note_meta.tags_json`, stamp) modeled on `note_body_backfill.rs`; `read_tags_in_schema` (cache.rs:1027) reads the table when stamped, **else falls back to the live scan** (zero-risk rollout); `reconcile_filesystem` recomputes authoritatively (the periodic self-heal).
- **MUST DO before it touches the save path:** (a) **rehearse on a copy of the live DB** that `tag_counts` == the live `read_tags` aggregate EXACTLY (§BL.1 discipline); (b) **adversarial audit the backfill-coexistence race** (a note edited during the ~6 s backfill window — bound it; it's the additive-aggregate variant of the race `links_backfill` accepts); (c) a unit test for the ±delta multiset math; (d) Editor-Surface Gate (the write path is touched).
- **Then §C.2** (defer the 234k `read_links` off the graphReady critical path — read persisted `sky_links` + `note_meta.outgoing_*` at boot, lazy full-edge load on first graph open) + **§C.3** (covering index on note_links). **Verify §C end-to-end via the boot-history tool: cold graph_ready 33 s → toward the ~1.7 s floor.**

### Then §D — phase-by-phase bring-up per the concept-papers §7 Debt Register (each function: write-time cure + shared right-click + i18n + gate). Confirmed defects to fix in-phase: Calendar day-click opens TODAY; Tasks toggle bypasses the reindex gate; Review-Pulse dead code; Command-Palette stubs. Tasks REFRAMED (kept) as "open epistemic loops".

---

## ADDENDUM (session 3 — §C.1 BUILT + proven; §C.2 is the precise pickup)

**Orientation now v2.85** · all on `main` (binary rebuilt 2026-06-15 23:21, pure-Rust → no `npm`).

### §C.1 SHIPPED (Claude-verified; pending Boss runtime validation)
Write-time `tag_counts` replaces the 5.6 s boot tag scan. Files: new `src-tauri/src/tag_counts.rs` (+`mod tag_counts;`); `tag_counts(tag,n)` in `init_db`; ±delta in `index_note` (capture old tags before the UPSERT, apply after — inside the existing `BEGIN IMMEDIATE`) + `reindex_delete_note` (old→[]); read-flip in `cache::read_tags_in_schema` (table when `{schema}` stamped, else legacy scan); backfill scheduled in `ensure_search_db_ready`; authoritative recompute in `reconcile_filesystem`.

- **Backfill = ONE atomic `json_each` aggregate** (refinement of the "batched like note_body" handover line — atomicity ELIMINATES the additive-aggregate race, item b; proof-by-cases in SESSION-LOG-2026-06-15 §C.1 + orientation v2.85). Dedicated connection, `IMMEDIATE`, aggregate+stamp in one txn; ~6 s one-time lock hold, post-paint, WAL keeps readers unblocked.
- **Gates met:** (a) live-DB rehearsal byte-identical (`tag_counts::tests::rehearse_against_live_copy` vs `lab/tag-counts/live-read-tags-target.json`, gitignored — regen via `lab/tag-counts/analyze-live-tags.py`); (b) race audit; (c) 4 ±delta unit tests; `cargo test --lib` **930/0**; (d) Editor-Surface reasoning (tags = derived view, content untouched).
- **Boss validation pending:** launch → `diagnostics.log` `[tag_counts_backfill] completed: 19542 distinct tags`; 2nd boot `read_tags` 5.7 s → ~ms (boot-history tool); add/remove a tag → sidebar count moves live; body intact across the gate.

### §C.2 — defer the 234k `read_links` (THE PICKUP — the BIG win, 11.3 s cold)
- `cache::read_links_in_schema` (cache.rs:924) reads ALL 234k `note_links` rows on the `graphReady` critical path. Boss decision (locked): **defer it** — boot reads the persisted `sky_links` + `note_meta.outgoing_*` aggregates (already write-time maintained), and the full edge list loads **lazily on first graph/panel open**. Then **§C.3** (covering index on `note_links` for any residual in-order read). **Verify cold `graph_ready` 33 s → toward the ~1.7 s floor via the boot-history tool.** This is a bigger architectural change than §C.1 (it changes WHAT boot loads, not just how a view is stored) — give it the same full-care treatment: map every consumer of the boot `links` payload, confirm the lazy-load path, prove no graph/Sky/backlink regression.
