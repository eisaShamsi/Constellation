# Next-session kickoff prompt — paste this to resume

> Ready-to-paste prompt for the session that picks up MIG-079 §C.2b. Copy everything in the box.

---

Working on: **MIG-079 §C.2b — defer the 234k-row `read_links` off the boot critical path** (the actual ~11 s boot win), then **§C.3** (covering index). Continuing the bring-up program.

First read `docs/Constellation Orientation & Onboarding v2.86.md`, then `docs/MIG-079-Plan.md` (§C), then `lab/reports/HANDOVER-2026-06-17-mig079-c2b.md`, then `lab/reports/SESSION-LOG-2026-06-16.md`.

Context (all SHIPPED + Boss-validated, on `main`): §A (single-owner activation) · §B (safeBootMode + the append-only boot-perf history tool `lab/boot-perf/read-boot-history.py`) · **§C.1** (write-time tags) · **§C.2a** (write-time backlink counts — `note_meta.incoming_count`, alias-aware, the badge now shows the true ~413,660 not 47; maintained by a save-path recompute-affected diff, WA#5-validated; VIRTUAL `note_links.target_name_lower` + index `idx_nl_tnl`) · **the save-path link-skip** (text-edit on a link-heavy note is instant — `index_note` skips the rebuild when links are unchanged). **Boot is still ~27 s** because `read_links` (234k rows, ~11.3 s) is still on the boot path — §C.2b removes it.

Task: build **§C.2b** as one proven unit. Locked design (impact analysis `wf_d9a26cf7`): boot keeps tags + aliases but DROPS the 234k-edge array; Sky renders from the persisted `sky_links` + `note_meta.outgoing_*` aggregates; the full edge list lazy-loads via a memoized `ensureFullLinks()` + `linksReady` flag (idle pre-fetch after `boot:graph-ready` + lazy on first Backlinks/Outgoing/Graph open). **P0: guard every `allLibraryLinks` consumer on `linksReady` so none renders empty.** Move `clearLinkTraversalBumps` into the lazy completion. Then **§C.3**: covering index `idx_link_boot ON note_links(status, source_path, target_name, link_type, library_name, weight, traversal_count, last_traversed, confidence)` (annotation excluded); assert `EXPLAIN QUERY PLAN` = `USING COVERING INDEX`. **Verify cold `graph_ready` 33 s → toward the ~1.7 s floor** via the boot-history tool; Backlinks/Outgoing/Sky never-empty after open; Editor-Surface Gate. Crosses the Rust↔Svelte boot contract → full `/migration` care + WA#4 impact agents.

Standing orders that bit this session, honor them: **test instructions must be LITERAL** (exact click/type + expected — no "open X → results appear" assumptions); **measure, don't guess** (boot-history tool; never state a cause without a measurement); **WA#5 cross-check** before any inventive fix.

Git pull first; close the app before `cargo build --release` (the running `.exe` locks cargo). Pure-Rust → no `npm` unless you touch the frontend (§C.2b DOES touch the frontend — `+layout.svelte` loadGraph + the consumers — so `npm run build` THEN `cargo build --release`, and grep `build/` for a new string to confirm the frontend re-embedded). Active universe "Eisa Cognitive Knowledge" (7,653 notes). Do the full closing PCS + handover + next prompt at session end.
