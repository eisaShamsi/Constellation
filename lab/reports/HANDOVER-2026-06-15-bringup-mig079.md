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
