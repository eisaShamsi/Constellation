# Constellation Pending Jobs

**Version 1.76 | 2026-08-09**

> **What changed in v1.76** (**PJ-207 §14 built and measured; the flag flip is now a Boss decision with a real number behind it. Ultracode**):
>
> **► NEXT ACTION — the §14 FLIP, then §15** — *turn the Full re-read on, or rule that it stays off.* The measurement the Boss's own ruling demanded now exists (below). Flipping needs **both** constants (`repairFlag.ts` **and** `index_repair::FULL_REREAD_ENABLED` — the Rust one is the load-bearing gate), a confirmation dialog quoting the number, `indexRepair.fullReread.*` strings ×15, and a Boss test — **which is where the never-yet-performed mid-run Cancel finally becomes catchable**, because a full re-read runs for tens of seconds rather than outrunning the click. Then **§15** close: the per-cycle whole-app safety inspection, the confirmed register, PJ-223's formal close, the Charter W2-9 close, the Lessons-Learned entry, and the one item no commit can reach — the 2026-05-04 memo `project_index_rebuild_button_decision.md` lives OUTSIDE the repo and must be marked **overturned** by hand. **§13 remains GATED on PJ-224 (Boss ruling still required).**
>
> ### ✅ PJ-207 §14 — CLOSED (built, measured, flag OFF exactly as specified)
> `WalkCtx.force` threaded end to end; `Scope::FullReread` added; `covers()` given an explicit rule in both directions (a re-read subsumes a `Full`; a `Full` does **not** cover a re-read, because it skips the very notes the re-read exists for). **Refused server-side** while the Rust flag is false, per `repairFlag.ts`'s own scope note — a UI-only gate hides a feature, it does not make it unreachable. Release build: `FullReread` appears in zero warnings.
>
> ### 📏 §M1 — the measurement, and a wrong first run caught by its own output
> First run reported 204 s for 799 notes. Invalid: `note_meta` moved 2,721 → 3,541 and `unchanged` was 0 even with the mtime gate on — a path-format mismatch (the app stores backslashed paths) meant every note was INSERTED, not re-read. **The harness now asserts the row count did not move**, so a mismatched run fails instead of publishing a number.
>
> **Valid (Eisa Universe, four own libraries, on the USB drive):** 1,258 notes — ordinary repair **0.1 s**, full re-read **24.7 s** (≈51 notes/s; 40–216/s by note size; PKM measured 14.9 s and 20.1 s on two runs, so treat any figure as a guide). Ordinary repair is ~10,000 notes/s because it stats without opening. **Extrapolated, not measured:** ≈2.5 min for the 7,824-note universe.
>
> ### ⚠️ Correction carried from v1.75
> v1.75 said the mid-run Cancel check would fold into §14's Boss test. **It cannot** — §14 ships flag-off and is not Boss-testable. Cancel belongs to the flip commit.
>
> ### 📌 Carried
> **PJ-110** localStorage durability — `app-prefs.json` is the home for the next tenants; `constellation-wab` (unsaved note content) needs a durability design, not a JSON file. **Doc-drift watch** — the translated manuals are partial (1,592–1,941 lines vs 2,580) and drift in vocabulary; §12 found 67 wrong panel names in `fa`/`ur`. A periodic term-consistency sweep against each locale's own `i18n` JSON is worth a PJ if it recurs.
>
> ### 📌 STILL OPEN, unchanged from v1.75
> PJ-224 gates §13 (Boss ruling required) · PJ-219 design ruling · PJ-225 mtime sweep · PJ-226 walker-classification sweep (20×) · PJ-227 linked-universe phantom rows · PJ-220 (`{name:}` form + args delivery) · PJ-221 `bases.rs:796` APP-KILLER · PJ-222 · `store.ts loadWorkspaces` APP-KILLER · the 2026-07-30 25 lost candidates · the 38-finding register (`wbxz23bdr`) · PJ-172 Sight flakes. **PJ-223** repaired live at §11 Stage 1; formal close at §15.
>
> **Gates:** Rust 1387/0 (16 ignored) · vitest 913/913 · svelte-check 0 · i18n 15/15 ✓.
>
> ---

**Version 1.75 | 2026-08-09**

> *(See `Constellation Pending Jobs v1.75.md` — the trail is durable, never overwritten.)*
