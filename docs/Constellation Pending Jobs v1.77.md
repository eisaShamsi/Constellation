# Constellation Pending Jobs

**Version 1.77 | 2026-08-09**

> **What changed in v1.77** (**PJ-207 §14 FLIP Boss-passed — the Full re-read is live and the mid-run Cancel is finally exercised. Only §13 and §15 remain. Ultracode**):
>
> **► NEXT ACTION — PJ-207 §15, the migration close** — because **§13 is still GATED on PJ-224 (Boss ruling required)**. §15 is: the per-cycle **whole-app** safety inspection (`Workflow({ name: 'safety-inspection' })` — a `/migration` close IS a cycle boundary, and every confirmed finding is fixed before the cycle is declared closed); the confirmed register appended to the Charter; **Charter W2-9 → closed** (evidence §8/§13); **PJ-223 formally closed** (repaired live at §11 Stage 1); the Lessons-Learned entry — *a repair pass with no door is indistinguishable from no repair pass at all, and a localised string naming a nonexistent control is a promise the app breaks in 15 languages*; and **the one item no commit can reach** — the 2026-05-04 memo `project_index_rebuild_button_decision.md` lives OUTSIDE the repo in the session memory and must be marked **overturned** by hand (its own reopen condition — "reopen if Boss flags index desync" — fired and was measured at 60/7,824).
>
> ### ✅ PJ-207 §14 FLIP — CLOSED, Boss-passed 2026-08-09 ("All pass")
> One door two scopes (`constellation_search_init`'s optional `full_reread`), a second Settings row with a confirmation that **quotes the measured numbers**, ×15 locales, both gates flipped. The OFF-pin was **inverted rather than deleted** — it now asserts the two gates agree. Boss ran Step 0 → Step 6 including **the mid-run Cancel, finally caught** after the job outran him twice (§10's scan, §11's one-note repair), and the next-launch "Finishing an interrupted index repair…" net.
>
> ### ☠️ Five inspection findings pre-commit — four mine, one a live regression
> The serious one: `index_note`'s save-during-read guard was `if !force`, premised on force callers always being "this file just changed" contexts. §14 made the **bulk walk** a force caller and falsified it — a note saved during a tens-of-seconds run could be written back with pre-save bytes, counted `Indexed`, silently. Fixed by splitting intent (`index_note` / `index_note_bulk`). Three more shared one root cause — `matches!(scope, Scope::Full)` in three gates, all missed by the new variant → `Scope::is_whole_universe()`. Plus a dead §11 placeholder the flip turned into a visible broken duplicate row. **Re-inspection: 0 confirmed.**
>
> ### 🆕 PJ-233 — the app runs a Universe that its own registry does not list *(MED, filed not chased)*
> `C:\Users\ealsh\AppData\Roaming\world.uconstellation.app\universes.json` lists only `كون عيسى` and points `active_id` at it (registry mtime 2026-08-07), while the app has demonstrably been running **Eisa Universe** — proven independently by the app-generated `"timestamp"` inside its own `boot-perf.latest.json` (2026-08-09 09:04), its logged boot sequence, its federation manifest, and direct file counts (1,260 / 2,102 vs `كون عيسى`'s 6). The ui-inspector could not reconstruct from source how `set_active_universe` — which requires the id to exist in `registry.entries` — reached this state, and **said so rather than inventing a mechanism**. Nothing depends on it today, but a registry that disagrees with reality is precisely the "unverified reachability claim" class this whole migration exists for. Consider at §15 or as its own job.
>
> ### 📌 Carried
> **PJ-110** localStorage durability — `app-prefs.json` is the home for the next tenants; `constellation-wab` (unsaved note content) needs a durability design. **Doc-drift watch** — the translated manuals are partial and drift in vocabulary (§12 found 67 wrong panel names in `fa`/`ur`); a periodic term-consistency sweep against each locale's own `i18n` JSON is worth a PJ if it recurs.
>
> ### 📌 STILL OPEN, unchanged from v1.76
> PJ-224 gates §13 (Boss ruling required) · PJ-219 design ruling · PJ-225 mtime sweep · PJ-226 walker-classification sweep (20×) · PJ-227 linked-universe phantom rows · PJ-220 (`{name:}` form + args delivery) · PJ-221 `bases.rs:796` APP-KILLER · PJ-222 · `store.ts loadWorkspaces` APP-KILLER · the 2026-07-30 25 lost candidates · the 38-finding register (`wbxz23bdr`) · PJ-172 Sight flakes.
>
> **Gates:** Rust 1389/0 (16 ignored) · vitest 913/913 · svelte-check 0 · i18n 15/15 ✓.
>
> ---

**Version 1.76 | 2026-08-09**

> *(See `Constellation Pending Jobs v1.76.md` — the trail is durable, never overwritten.)*
