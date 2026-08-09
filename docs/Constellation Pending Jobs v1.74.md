# Constellation Pending Jobs

**Version 1.74 | 2026-08-09**

> **What changed in v1.74** (**PJ-230, PJ-231 and PJ-232 all closed — `init_db` stops writing to universes it does not own. Ultracode**):
>
> **► NEXT ACTION — PJ-207 §12, the docs step** — help files ×15 + User Manual ×15 covering §9 (the drift notice), §10 (the shared progress strip) and §11 (the repair door + receipt), plus PJ-228's heal strip and PJ-229's language durability, which are both user-visible. Then **§13 GATED on PJ-224 (Boss ruling still required)** · **§14** flag-off Full re-read + §M1 — **fold the never-yet-performed mid-run Cancel check into that Boss test** · **§15** close (per-cycle whole-app inspection).
>
> ### ✅ PJ-230 — CLOSED as "not reachable; the real fix was already made and unrecorded"
> `run_migrations_on` fires only on `schema_incomplete`, and any database old enough for that predates the crash markers — *old enough to drift ⇒ cannot be armed; new enough to be armed ⇒ cannot drift.* Opening a child as active already heals it (verified end to end). What the investigation DID find: the pre-PJ-228 heal rebuilt a child's aggregates with the parent's vocabulary and then **cleared the child's markers**, making it permanent; PJ-228 ended that while crediting boot latency. Now recorded in three comments, the call site, and a **regression test that fails if a self-heal returns to `init_db`**. Rejected on purpose: loading the child's vocabulary into the global first (a background-thread swap of a process-global every subsystem reads).
>
> ### ✅ PJ-231 — CLOSED
> `let _ = mig003_step3_soft_rebackfill(...)` discarded the early-probe `Err` — the one outcome the function does not log itself. Ungated, every boot, and it writes identity keys into user frontmatter. Now surfaced via `diag_log`.
>
> ### ✅ PJ-232 — FILED AND CLOSED IN THE SAME PASS (found by the inspection, refuting my own new comment)
> `init_db` bakes the ACTIVE universe's link vocabulary into persisted trigger DDL (outgoing aggregates + 3 Sky blocks, all from `link_types::snapshot()`), and `federation::migrate` hands it a FOREIGN database. Then `mig003_step3_soft_rebackfill` fires those triggers on the child's rows and writes frontmatter into the child's `.md` files. Fixed with **`init_db_schema_only`** (`InitScope::ForeignSchemaOnly`): schema only — no vocabulary DDL, no back-fills, no re-backfill, **no Step-4 rename pass (it renames `.md` files)**. `init_db` unchanged for every other caller. Three paired tests.
>
> ### 📌 Carried
> The repair's **mid-run Cancel has still never been performed by a human** — fold into §14's Full re-read. **PJ-110** localStorage durability — `app-prefs.json` now exists as the home for the next tenants; `constellation-wab` (unsaved note content) needs a durability design, not a JSON file.
>
> ### 📌 STILL OPEN, unchanged from v1.73
> PJ-224 gates §13 (Boss ruling required) · PJ-219 design ruling · PJ-225 mtime sweep · PJ-226 walker-classification sweep (20×) · PJ-227 linked-universe phantom rows · PJ-220 (`{name:}` form + args delivery) · PJ-221 `bases.rs:796` APP-KILLER · PJ-222 · `store.ts loadWorkspaces` APP-KILLER · the 2026-07-30 25 lost candidates · the 38-finding register (`wbxz23bdr`) · PJ-172 Sight flakes. **PJ-223** repaired live at §11 Stage 1; formal close at §15.
>
> **Gates:** Rust 1384/0 (15 ignored) · vitest 913/913 · svelte-check 0 · i18n 15/15 ✓.
>
> ---

**Version 1.73 | 2026-08-09**

> *(See `Constellation Pending Jobs v1.73.md` — the trail is durable, never overwritten.)*
