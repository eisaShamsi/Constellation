# Constellation Pending Jobs

**Version 1.73 | 2026-08-09**

> **What changed in v1.73** (**PJ-228 and PJ-229 both shipped and Boss-passed; two new PJs filed rather than slipped into a tested build. Ultracode**):
>
> **► NEXT ACTION — back to PJ-207 §12, the docs step** — help files ×15 + User Manual ×15 covering §9 (the drift notice), §10 (the shared progress strip) and §11 (the repair door + receipt) together, as the plan sequences them. The §12 pass should now ALSO cover PJ-228's heal strip (a user-visible surface) and PJ-229's language durability. Then **§13 GATED on PJ-224 (Boss ruling still required)** · **§14** flag-off Full re-read + §M1 — **fold the never-yet-performed mid-run Cancel check into that Boss test**, since a Full re-read is by definition long enough to catch · **§15** close (per-cycle whole-app inspection).
>
> ### ✅ PJ-228 — CLOSED, Boss-passed 2026-08-09
> The five-family heal no longer runs inside `init_db` before the database is published (3,143 ms measured, no progress, failures to a discarded `eprintln!`). New `derived_heal.rs`: own connection + FTS tokenizer + 30 s busy timeout, scheduled from `ensure_search_db_ready` after publish, fourth `JobProgressStrip` consumer (`derivedHeal.*` ×15), yields to a repair, and clears the crash markers only on `converged_fully() && gen_stable && !overlapped`. `converge::Ctx.stop` asked between families only; `ConvergeReport::stopped` + `converged_fully()` separate *gave way* from *succeeded* — the distinction the whole design rests on, since `all_ok()` is true for a stopped run. Boss: window usable immediately, strip counted to 5, finished on its own.
>
> ### ✅ PJ-229 — CLOSED, Boss-passed 2026-08-09
> `{app_data_dir}/app-prefs.json` (strict read + atomic write, `style_presets` precedent), `appPrefs.ts` with a read-succeeded latch, `decideLocale` pure + 4 vitest pins, `reconcileLocaleFromDisk()` first in `onMount`, the write in `handleLangChange` (NOT in `setLocale` — the second screen calls that). localStorage demoted to the first-paint cache. Boss: closed in Arabic, reopened in Arabic + RTL. **Verified by me on disk afterwards** (WA#1): the file exists and round-trips. **Fixed in passing (WA#6):** `UniverseSetup` swept every `constellation-*` key including `constellation-wab`, the unsaved-note recovery net.
>
> ### 🆕 PJ-230 — a linked universe's own database no longer gets the derived heal *(needs a Boss ruling)*
> `federation::migrate::run_migrations_on` calls `init_db` on a cUniverse's DB (`federation/migrate.rs:86`), so the heal left that path when it left `init_db`. **Deliberately not re-added there:** the parent would recompute a foreign universe's aggregates using the PARENT's process-global link vocabulary. The child's markers stay armed and heal when that universe is opened as active — so the residual gap is a cUniverse that is *never* opened directly. Options: (a) accept, (b) schedule a heal when a cUniverse is opened, (c) let the parent heal it with the child's own vocabulary loaded. **Boss ruling requested.**
>
> ### 🆕 PJ-231 — the one ungated per-boot member of the same class still discards its result
> `search.rs:5778` — `let _ = mig003_step3_soft_rebackfill(&mut conn, path);`. Unlike the heal it runs on EVERY boot, and it **writes frontmatter into the user's `.md` files**, so a failure is both invisible and consequential. One-line fix (log the outcome via `diag_log`, which is the durable sink — `eprintln!` goes nowhere in a Windows release build). Held back only because the Boss had already tested and passed this exact binary; it must not ride an untested change.
>
> ### 📌 Carried
> The repair's **mid-run Cancel has still never been performed by a human** — the job has outrun the click twice (§10's scan, §11's one-note repair). Fold it into §14's Full re-read.
>
> ### 📌 STILL OPEN, unchanged from v1.72
> PJ-224 gates §13 (Boss ruling required) · PJ-219 design ruling · PJ-225 mtime sweep · PJ-226 walker-classification sweep (20×) · PJ-227 linked-universe phantom rows · PJ-220 (`{name:}` form + args delivery; CRLF proven for scriptPath) · PJ-221 `bases.rs:796` APP-KILLER · PJ-222 · `store.ts loadWorkspaces` APP-KILLER · **PJ-110 localStorage durability — `app-prefs.json` now exists as the natural home for the next tenants; 18 keys remain, and `constellation-wab` (unsaved note content) needs a durability design, not a JSON file** · the 2026-07-30 25 lost candidates · the 38-finding register (`wbxz23bdr`) · PJ-172 Sight flakes (reproduced again under CPU load; pass in isolation). **PJ-223** repaired live at §11 Stage 1; formal close at §15.
>
> **Gates:** Rust 1381/0 · vitest 913/913 · svelte-check 0 · i18n 15/15 ✓ · diff inspection 0 confirmed.
>
> ---

**Version 1.72 | 2026-08-08**

> *(See `Constellation Pending Jobs v1.72.md` — the trail is durable, never overwritten.)*
