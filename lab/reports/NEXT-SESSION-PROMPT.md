# Next-session kickoff prompt — paste this to resume

> Ready-to-paste prompt for the session that picks up MIG-079 §C.2d (defer the Sky read off boot). Copy everything in the box.

---

Working on: **MIG-079 §C.2d — defer the Sky read off the boot critical path** (the remaining boot win), then the small §C.2c audit follow-ups, then §D bring-up.

First read `docs/Constellation Orientation & Onboarding v2.88.md`, then `lab/reports/HANDOVER-2026-06-17-c2c-done.md`, then `lab/reports/SESSION-LOG-2026-06-17.md`.

Context (all SHIPPED + Boss-validated, on `main`, `6a4d5e20`→`44acd79d`): §C.2b (defer the 234k `read_links` off boot) + §C.3 (covering index) + **§C.2c (kill the in-memory 234k-edge array — Backlinks/Outgoing now query per-note from SQLite + are virtualized via `VirtualList`)** + two summary toggles (default OFF). The freeze/thrashing is fixed.

**THE TASK — §C.2d.** Boot is still ~17–20 s. **Measured** (boot-perf history `lab/boot-perf/read-boot-history.py`): cold `graph_ready` ~17 s, of which `queue≈15 s` is the graph IPC waiting behind **`cache_boot_snapshot_sky`**, which reads the **233,995-row `sky_links`** table cold — the untouched twin of `note_links`. §C.2b already proved the pattern on links; apply the SAME WTD/lazy model to Sky: defer the Sky snapshot off the boot critical path (Sky View loads its graph when opened, not at boot). Cross the Rust↔Svelte boot contract → full `/migration` care + WA#4 impact agents + measure before/after on the boot-history tool (cold via PC restart). Settle in the Architect: can Sky render from `sky_nodes` + a deferred/virtualized edge load (like the panels), or does it need the full edge set?

Then the deferred §C.2c audit items (small): split-pane ×N chips (per-open-tab outgoing map), the `status` predicate nit, the §C.2b cleanup (retire `cache_full_links`/`ensureFullLinks`/`idx_link_boot`), the ×15 i18n for the two new toggles. Then §D bring-up phases.

Standing orders that bit last session — honor them: **measure, don't guess** (boot-history tool; never state a cause without a measurement — the boot pivot came from it); **WA#5 cross-check before any inventive fix** (Eisa enforces it explicitly); **don't patch one symptom >3× — Solve-the-Class / build the whole thing + prove it** (the §C.2c switch-lag saga → "Enough patching" → SME panel); **test instructions LITERAL** (exact click/type + expected). Git pull first; close the app before `cargo build --release`; frontend change → `npm run build` THEN `cargo build --release`, grep `build/` for a new string. Active universe "Eisa Cognitive Knowledge" (7,653 notes). Do the full closing PCS + handover + next prompt at session end.
