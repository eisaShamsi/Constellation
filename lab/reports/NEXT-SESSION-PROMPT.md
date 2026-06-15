# Next-session kickoff prompt — paste this to resume

> Ready-to-paste prompt for the session that picks up MIG-079 §C.1. Copy everything in the box.

---

Working on: **MIG-079 §C.1 — write-time `tag_counts`** (the boot-graph Write-Time Derivation for tags), the next step of the bring-up program.

First read `docs/Constellation Orientation & Onboarding v2.84.md`, then `docs/concept-papers/00-MASTER-Bring-Up-Charter-and-Checklist.md` (§7 Debt Register), then `docs/MIG-079-Plan.md`, then `lab/reports/HANDOVER-2026-06-15-bringup-mig079.md` (esp. the Addendum), then `lab/reports/SESSION-LOG-2026-06-15.md`.

Context (all SHIPPED + Boss-validated last session, on `main`): MIG-078 §B1 (startup-race fix); the bring-up program + 31 concept papers + the app-wide Debt Register; MIG-079 §A (single-owner activation → `init_db` runs once); MIG-079 §B (`safeBootMode` switch + the append-only **boot-perf history tool** `lab/boot-perf/read-boot-history.py` + the **measured** baseline). MEASURED truth: the editor spine is fast cold+warm (~1.7 s / ~0.6 s); the ENTIRE boot cost (33 s cold / 5.6 s warm) is the graph reads (`note_links` 234k + `tags`). **§C is the whole fix.** Standing order: never state a cause without a measurement (use the boot-history tool).

Task: build **§C.1** as ONE proven unit. Safety already verified: `index_note`'s UPSERT (`search.rs:4266`) is the SOLE writer of `note_meta.tags_json`; delete decrement goes in `reindex_delete_note` (`search.rs:7217`). Design (locked): `tag_counts(tag TEXT PRIMARY KEY, n INTEGER)` in init_db; a **Rust ±delta** (old multiset → new multiset) in those two paths, gated on `schema_versions.tag_counts`; a **non-blocking own-connection backfill** modeled on `src-tauri/src/note_body_backfill.rs`; `read_tags_in_schema` (`cache.rs:1027`) reads the table when stamped, **else falls back to today's live scan**; `reconcile_filesystem` recomputes authoritatively (self-heal). Before it touches the save path: (a) **rehearse on a copy of the live DB** that `tag_counts` == the live `read_tags` aggregate EXACTLY; (b) **adversarially audit the backfill-coexistence race** (a note edited during the ~6 s backfill window — bound it); (c) a unit test for the ±delta multiset math; (d) the Editor-Surface Gate. Then §C.2 (defer the 234k `read_links` off the `graphReady` critical path) + §C.3 (covering index on `note_links`), and verify cold `graph_ready` 33 s → toward the ~1.7 s floor via the boot-history tool.

Git pull first; the active universe is "Eisa Cognitive Knowledge" (`E:\Constellation Universes\Eisa Cognitive Knowledge`, 7,653 notes, 1.83 GB). Binary: `src-tauri/target/release/constellation.exe` (frontend change → `npm run build` THEN `cargo build --release`; the running `.exe` locks cargo — close the app first). Do the full closing PCS + handover + next prompt at session end (standing order).
