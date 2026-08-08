# Constellation Pending Jobs

**Version 1.69 | 2026-08-08**

> **What changed in v1.69** (**PJ-207 §9 built, Boss-passed live (19→20) — the drift notice ships, and the walk that produces it got 14× faster. Ultracode**):
>
> **► NEXT ACTION — PJ-207 §10**, *One progress strip instead of three copies* (extract `JobProgressStrip.svelte` from the two byte-equivalent strips; the repair reuses it). Then **§11 THE DOOR** — Repair index in Settings + "Repair now" on the notice, every surface refreshes. Plan: `docs/PJ-207-Index-Repair-Plan.md`. **§13 remains gated on PJ-224 (Boss ruling required).**
>
> ### ✅ PJ-207 §9 — BUILT, Boss-passed 2026-08-08
> **No new walker.** The boot reconcile already walked every own library each launch and already computed two of §9's four numbers — reporting them only to `diagnostics.log` (*"825 orphan files (> cap 200) — skipping re-adopt"*, 4× in one day on `Eisa Universe`). **PJ-223 was detected-but-unreported.** §9 teaches that pass the mtime comparison (`search::mtime_secs`, one function shared with `index_note`'s gate) and surfaces a four-counter `DriftReport` as an amber, dismissible, never-all-clear notice band — `indexDrift.*` ×15 locales. `missing_from_index` added because the plan's three counters could not represent PJ-223 (RED-proven: the plan's own shape reported 825 never-read notes as "changed"). `foreign_rows` counted by PATH (9 real) never by name (621 — 603 are pre-MIG-108 ghosts).
> - **§M6 inverted:** drift check +4–10 ms on 7,964 files; the measurement exposed `path.is_dir()` as ~95% of the walk → `entry.file_type()`: **252–260 → 17–19 ms warm** (cold on the Boss's USB-HDD `E:` was 3.5–8.7 s). §9's net cost ≈ **−235 ms/launch**.
> - **Boss round 1 caught a layout break:** the unplaced notice auto-placed into the saturated `.app` grid — workspace displaced. `.tpl-err`/`.store-err` shipped with the SAME latent defect (and a red-on-red palette: both error tokens = `--color-red`). One `.notice-band` grid row now hosts all three; every `.app` child explicitly placed; palette fixed.
> - **Inspection (diff-scoped — PJ-220's CRLF blocker diagnosed and bypassed): 5 confirmed, 4 fixed pre-commit** — "could not look" now counts as a finding on both sides of the wire; drift cleared on universe switch; the §7 drain busy-spin closed; `resurrected` rows no longer over-reported. 1 LOW filed → PJ-227.
> - **The live round validated §8 by accident:** the Boss's first test note was inside linked `كون عيسى` — the count correctly did NOT move (own-scope boundary), and the in-app creation live-reproduced PJ-219 (row in the parent's index; child's own index has nothing).
> - **Gates:** Rust 1370/0 · vitest 900/900 · svelte-check 0 · i18n 15/15 ✓.
>
> ### 🆕 FILED
> - **PJ-225 — the `mtime_secs` sweep.** §9 extracted the one mtime definition and converted the two `search.rs` sites; **nine** hand-rolled copies of the same three-line expression remain: `search.rs:3284` (cid-collision tie-break), `write_gate.rs:350` (folds unreadable→0, the exact class the helper's doc forbids), `libraries.rs:7456/:3364/:3886`, `bases.rs:663`, `map.rs:626`, `inspector360.rs:397`, `tasks.rs:314`. If `note_meta.modified` ever changes shape, §9's two sites move and nine silently disagree.
> - **PJ-226 — the walker-classification sweep.** `path.is_dir()` = `fs::metadata` = a handle-open per directory entry on Windows; ≈25 walkers across the crate share the shape (`collect_md_paths`, mig108, canonical, …). Measured on the reconcile walk: **20×**. The biggest single cold-boot lever on a mechanical drive.
> - **PJ-227 — linked-universe phantom rows are unhealable post-§8.** Scoping dead-row removal to own roots means a `note_meta` row pointing at a *deleted* linked-universe file is never removed and (being `foreign_rows`) never user-reported — 9 live rows today. Belongs with §13/PJ-219's design ruling, not §9.
> - **PJ-220 — UPDATED, cause half-proven.** The "script contains control characters" rejection is **CRLF**: the failing copy carried 130 CR bytes (Python `write_text` newline translation on Windows); the repo file has zero; written LF-only, `Workflow({scriptPath})` launches and runs genuinely diff-scoped. STILL OPEN: whether `{name:'safety-inspection'}` fails for the same reason (a loader re-encoding?), and whether `args` reach the script (untested this round — the file list was hardcoded into the script copy).
> - **§11 PREP NOTES (not new PJs — they are §11's own build concerns):** (a) §11 needs a *recompute* path for the report after a repair — clearing the notice on `ok:true` would be a success **claim**, the class this migration exists to end; the altitude review's `scan()`/`heal()` split of `reconcile::run` is the shape. (b) Extract the drift listener + race guard into a `.svelte.ts` store **before** the second screen duplicates it. (c) The plan's §11 text anchors "Repair now" on the `storeHealthError` bar (`+layout.svelte:7442-7446`, stale line numbers) — §9 deliberately did not put drift there; the natural host is the new drift band. Reconcile the plan text before building §11.
>
> ### 📌 STILL OPEN, unchanged
> **PJ-224 gates §13** (ordinary search box does not federate — Boss ruling required before any removal offer). **PJ-223** is now *reported* by §9; *fixed* by §11. PJ-219 (user-action write class) awaits its design ruling — note the live reproduction above, and the new asymmetry sentence for that discussion: an external edit to a linked note is invisible to the parent's drift check (own-scope) AND to the child's until the child universe is opened. PJ-221 (`bases.rs:796` APP-KILLER), PJ-222 (`collect_md_paths` boundary), `store.ts loadWorkspaces` (APP-KILLER), the 2026-07-30 inspection's 25 lost candidates, the 38-finding register (`wbxz23bdr`), PJ-172 flaky Sight timings — all as v1.68 left them.
>
> **Marker words burned for future tests:** `plarnwick` (now on disk ×2, indexed nowhere), `zarquon`, `blorptide`, `vandrasil`.
>
> ---

**Version 1.68 | 2026-08-07**

> *(See `Constellation Pending Jobs v1.68.md` — the trail is durable, never overwritten.)*
