# Next-session kickoff prompt — paste this to resume

> Ready-to-paste prompt for the session that picks up MIG-079 §C.2. Copy everything in the box.

---

Working on: **MIG-079 §C.2 — defer the 234k `read_links` off the boot critical path** (the big 11 s of the cold-boot graph cost), continuing the bring-up program.

First read `docs/Constellation Orientation & Onboarding v2.85.md`, then `docs/MIG-079-Plan.md` (§C), then `lab/reports/HANDOVER-2026-06-15-bringup-mig079.md` (esp. the Session-3 Addendum), then `lab/reports/SESSION-LOG-2026-06-15.md` (the §C.1 section).

Context (all SHIPPED + on `main`): MIG-078 §B1 (startup race) · the bring-up program + 31 concept papers + Debt Register · MIG-079 §A (single-owner activation) · §B (`safeBootMode` + the append-only boot-perf history tool `lab/boot-perf/read-boot-history.py` + the MEASURED baseline) · **§C.1 (write-time `tag_counts`)** — boot tag read 5.6 s → ~ms; backfill is ONE atomic `json_each` aggregate (the additive-aggregate race ELIMINATED); rehearsed byte-identical on the live DB; 930/0 tests. **MEASURED boot truth:** editor spine ~1.7 s cold / ~0.6 s warm; the rest of cold-33 s is the graph reads — `read_links` **11.3 s** (234k rows) + `read_tags` 5.7 s (now §C.1) + 12 s queue. **§C.2 removes the largest remaining piece.** Standing order: never state a cause without a measurement (use the boot-history tool).

Task: build **§C.2** as one proven unit. Boss decision (locked): **defer** `cache::read_links_in_schema` (cache.rs:924) off `graphReady` — boot reads the already-write-time-maintained persisted `sky_links` + `note_meta.outgoing_*` aggregates; the full 234k-edge list loads **lazily on first graph/panel open**. This is a bigger change than §C.1 (it changes WHAT boot loads, not just how a view is stored): **map every consumer of the boot `links` payload** (Sky View, backlinks/outgoing panels, graph, link-lifecycle, anything reading `BootSnapshotGraph.links`), confirm each can either use the aggregates at boot or tolerate a lazy edge-load, and prove no graph/Sky/backlink regression. Then **§C.3** (covering index on `note_links` for any residual in-order read). **Verify via the boot-history tool: cold `graph_ready` 33 s → toward the ~1.7 s floor.** Treat it with the full `/migration` care + the impact-review agents (WA#4) since it crosses the Rust↔Svelte boot contract.

Git pull first; close the app before `cargo build --release` (the running `.exe` locks cargo). Active universe "Eisa Cognitive Knowledge" (`E:\Constellation Universes\Eisa Cognitive Knowledge`, 7,653 notes, 1.83 GB). Pure-Rust change → no `npm` unless you touch the frontend. Do the full closing PCS + handover + next prompt at session end.

— Also: if §C.1 has NOT yet been Boss-validated at runtime when you start, validate it FIRST (launch → `[tag_counts_backfill] completed`; 2nd boot `read_tags` ~ms; add/remove a tag → live sidebar count; body intact) before starting §C.2.
