# Constellation Pending Jobs

**Version 1.71 | 2026-08-08**

> **What changed in v1.71** (**PJ-207 §11 built and Boss-passed (Stage 1) — THE DOOR is open; PJ-223's backlog was its first customer. Ultracode**):
>
> **► NEXT ACTION — PJ-207 §11 Stage 2, then §12** — *the Arabic RTL pass + the mid-run Cancel, carrying the review-count fix.* Stage 2 verifies: (a) the drift band, Repair now, the §10 strip and the Settings receipt all render correctly in Arabic RTL; (b) Cancel mid-run — cancelled is honest, already-repaired stays repaired, the band does NOT re-derive on a cancelled run (only a completed one earns the fresh look); (c) the review-count fix — the receipt's "Review schedule — updated 0" was a hard-coded placeholder (`converge.rs` maps `review::recompute_all_in`'s countless `Ok(())` to `Converged(0)`, unlike its count-returning siblings); fix = return the pass-2 rebuilt-row count; it ships ONLY behind Stage 2's Boss pass per the tested-build-before-commit rule. Then **§12** docs (help ×15 + User Manual ×15 for §9/§10/§11 together, by design) · **§13 gated on PJ-224 (Boss ruling)** · **§14** flag-off re-read + §M1 · **§15** close (per-cycle whole-app inspection).
>
> ### ✅ PJ-207 §11 — BUILT, Boss-passed Stage 1 2026-08-08
> One `submitRepair()` behind both buttons (drift band **Repair now** + Settings → Index → **Repair**) into the pre-existing typed submit door — my duplicate `index_repair_start` command was deleted the same hour it was written. `ConvergeReport` un-discarded and threaded to the frontend as `RepairReport` (stored Full-only; done event Full-only — the ColdStart empty-receipt corner); Settings renders **Last repair** (walk + 5 families). Band clears only by **re-derivation** (`reconcile::maybe_schedule` after an ok Full run), never by `ok:true`. Progress on §10's strip (`index-repair:progress`, ×25 throttle, terminal unconditional). Done-refreshes gated on "changed anything". 9 inspection findings fixed pre-commit, 1 HIGH (D2 clear now path-gated). **Boss Stage 1: six steps, all passed, digit-exact** — band 22 / 830 (predicted 21+1 / 830); strip 2,100, 90 s; `brimsloe` FOUND; band gone via fresh look; receipt 852 re-read · 1,248 unchanged · 0 failed (predicted 851+1 / 1,249−1); Sky View live, no restart. Pipeline round 3 APPROVED, 18 claims; a separator finding fixed in the CODE (`toLocaleString`), not the prose.
>
> ### ⚠️ SURFACED at the pass — being fixed in-pass (WA#6), no new PJ
> The review family's placeholder count (above). Not deferred: the fix is built immediately after this commit and rides Stage 2's Boss verification. `PJ-223` note: the 798-missing `Constellation PKM` class is now REPAIRED on the Boss's live universe (830 → 0 missing-from-index, band cleared) — PJ-223 closes formally at §15 with the register.
>
> ### 📌 STILL OPEN, unchanged from v1.70
> PJ-224 gates §13 (Boss ruling required) · PJ-219 design ruling (+ the federated-drift asymmetry note) · PJ-225 mtime sweep · PJ-226 walker-classification sweep (20×) · PJ-227 linked-universe phantom rows · PJ-220 (`{name:}` form + args delivery still open; CRLF proven for scriptPath) · PJ-221 `bases.rs:796` APP-KILLER · PJ-222 · `store.ts loadWorkspaces` APP-KILLER · the 2026-07-30 25 lost candidates · the 38-finding register (`wbxz23bdr`) · PJ-172 Sight flakes.
>
> **Gates:** Rust 1372/0 · vitest 909/909 · svelte-check 0 · i18n 15/15 ✓. Committed only after the Boss pass.
>
> ---

**Version 1.70 | 2026-08-08**

> *(See `Constellation Pending Jobs v1.70.md` — the trail is durable, never overwritten.)*
