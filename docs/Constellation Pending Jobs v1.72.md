# Constellation Pending Jobs

**Version 1.72 | 2026-08-08**

> **What changed in v1.72** (**PJ-207 §11 fully Boss-passed (Stages 1+2); five pre-existing defects fixed in-pass, two new PJs filed. Ultracode**):
>
> **► NEXT ACTION — PJ-207 §12, the docs step** — help files ×15 + User Manual ×15 for §9 (the drift notice), §10 (the shared progress strip) and §11 (the repair door + receipt) together, as the plan sequences them. Then **§13 GATED on PJ-224 (Boss ruling still required)** · **§14** flag-off Full re-read + the §M1 measurement · **§15** close (per-cycle whole-app safety inspection + the confirmed register).
>
> ### ✅ PJ-207 §11 — CLOSED, Boss-passed Stages 1 and 2 (2026-08-08)
> Stage 1: six steps, digit-exact (22/830 band · 2,100 walk in 90 s · `brimsloe` found · band cleared by re-derivation · 852 re-read · Sky View live, no restart). Stage 2: Arabic RTL pass, and the receipt line that had always read "updated 0" now reads **جدول المراجعة حُدّث 2,721** — matching outgoing/backlinks and the `note_meta` count measured beforehand. Test gated in three inspector rounds (58 + 13 + 9 claims, 0 findings outstanding).
>
> ### ✅ The review-family fixes (WA#6, all in-pass, none deferred)
> Inspecting the two-line count fix surfaced 2 defects; fixing those and re-inspecting surfaced 4 more. **All six pre-existing.** Fixed: the degrading-loader false-success (a transient unreadable pulse rewrote every row as never-reviewed and reported success); the mid-repair silent revert of a ✓ Reviewed (closed, not narrowed, by re-reading the pulse inside each window's transaction); **four COMMIT-without-ROLLBACK sites** (one of which could leave the app's own connection transacted for a whole session, discarding every `search.db` write at exit) now behind one shared `converge::commit_or_rollback`; the first-time back-fill stamping a divergence authoritative; and `set_review_priority` accepting a decision it stored nowhere. `load_pulse_data` is now `#[cfg(test)]` so the compiler enforces the write-back contract.
>
> ### 🆕 PJ-228 — the boot heal runs inside `init_db`, synchronously, with no progress *(MED — measured, not argued)*
> `after_interrupted_walk_at_boot` converges all five families before `state.db` is published; `init_db` has no `AppHandle`, so there is no event, no status-bar progress and no surfaced error. §11's Cancel made the marker-armed state reachable by an ordinary gesture (a cancelled walk keeps `derived_tail_pending`). **Measured with `converge_boot_heal_cost` (`#[ignore]`d, against a copy of the live universe): 3,143 ms for all five on 2,721 notes** — the hunt's "~90 s" was a misread comment. So: a ~3 s silent pause on the launch after a cancelled repair, not a freeze. **Fix = move it off the init path, background after paint with progress in the status bar, resumable — exactly Rule 8.** Deliberately not bolted onto a build under test; **Boss ruling requested** on whether it lands now or as its own step after §11.
>
> ### 🆕 PJ-229 — the interface language does not survive a restart *(MED, Boss-found at Stage 2)*
> Closed the app in Arabic, relaunched in English. The locale persists ONLY to `localStorage['constellation-locale']` (`src/lib/i18n/index.ts:64-91`), read back by `getInitialLocale()`; there is no durable on-disk setting for it, and **localStorage is the store PJ-110 already proved non-durable** (leveldb orphan-wipe, 2026-07-17). Pre-existing, unrelated to the §11 build. Fix = persist alongside the other settings on disk and treat localStorage as a cache. **Related:** the same non-durable store backs 19 `localStorage.setItem` call sites across 12 files — worth a sweep under PJ-110 rather than one-off fixes.
>
> ### 📌 Carried — the repair's mid-run Cancel is still unexercised by a human
> Twice now the job has outrun the click (§10's scan, §11's one-note repair). The shared strip's Cancel IS Boss-validated (§10 Stage 2, summary build) and the repair's stopped-early handling is source-verified + unit-pinned, but the specific gesture has never been performed on a repair. **Cheapest honest opportunity: §14's Full re-read**, which by definition gives a minutes-long run — fold the Cancel check into that Boss test rather than manufacturing work by touching file timestamps in the Boss's real universe.
>
> ### 📌 STILL OPEN, unchanged from v1.71
> PJ-224 gates §13 (Boss ruling required) · PJ-219 design ruling · PJ-225 mtime sweep · PJ-226 walker-classification sweep (20×) · PJ-227 linked-universe phantom rows · PJ-220 (`{name:}` form + args delivery; CRLF proven for scriptPath) · PJ-221 `bases.rs:796` APP-KILLER · PJ-222 · `store.ts loadWorkspaces` APP-KILLER · PJ-110 localStorage durability · the 2026-07-30 25 lost candidates · the 38-finding register (`wbxz23bdr`) · PJ-172 Sight flakes. **PJ-223** repaired live at Stage 1 (830 → 0 missing); formal close at §15.
>
> **Gates:** Rust 1377/0 (15 ignored) · vitest 909/909 · svelte-check 0 · i18n 15/15 ✓.
>
> ---

**Version 1.71 | 2026-08-08**

> *(See `Constellation Pending Jobs v1.71.md` — the trail is durable, never overwritten.)*
