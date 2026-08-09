# Constellation Pending Jobs

**Version 1.75 | 2026-08-09**

> **What changed in v1.75** (**PJ-207 §12 closed — the docs describe the repair door, and writing them found an app bug plus 67 wrong panel names. Ultracode**):
>
> **► NEXT ACTION — PJ-207 §14, the flag-off Full re-read + §M1 measurement** — because **§13 remains GATED on PJ-224 (Boss ruling still required)**. §14 turns on the Full re-read (`FULL_REREAD_ENABLED`, currently `false`) and measures §M1. **Fold the never-yet-performed mid-run Cancel check into §14's Boss test** — a Full re-read is by definition long enough to catch, which the one-note repair twice was not. Then **§15** close (per-cycle whole-app safety inspection + the confirmed register + the formal close of PJ-223).
>
> ### ✅ PJ-207 §12 — CLOSED
> Corrections (all four plan targets verified still present) AND the new documentation §11 made necessary: a User Manual subsection *"If your notes changed while Constellation was closed"* in §2's external-changes section, which until now covered only changes arriving while the app is open. `IPC-CONTRACT.md` gains the repair commands, both progress events and the `cache_reconcile` note; four stale comments now name the control that exists. Plan's verification grep returns only intentional history. English gated by the ui-inspector (29 claims, 1 finding — mine: I wrote "looked frozen" about a defect I had measured the day before and explicitly called "a ~3 s pause, not a freeze"). Translated into 14 locales by one agent each, labels pasted verbatim from each locale's own i18n JSON, then verified.
>
> **Two plan assumptions corrected by evidence:** the semantic-search sentence exists in **Arabic only**, not "the 13 other locales"; and while every locale has the external-changes section, the heading after it differs and several lack "Universe Notes Folder", so insertion points were located per file.
>
> ### 🐛 FIXED IN PASS — a Korean UI string, found by writing the documentation
> `indexDrift.changed` and `.missingFromIndex` in `ko.json` used the generic `이(가)` subject-particle placeholder after a noun that is ALWAYS `노트 {count}개` — always vowel-final — so Korean users read `개이(가)`, which is not grammatical Korean. The placeholder exists for an unpredictable preceding word; here it never is. Now `개가`. **User-visible in Korean; not Boss-testable in English or Arabic.**
>
> ### 🐛 FIXED IN PASS — 67 wrong panel names in Persian and Urdu
> `fa` and `ur` called Sky View **"Star View"** throughout — 22 and 20 occurrences in their User Manuals plus 25 across four `fa` topic pages — while their own app strings say `نمای آسمان` / `آسمانی منظر`. A reader would look for a panel that does not exist, and it contradicts a standing Boss correction. All corrected, headings and TOC anchors together. Persian's remaining "star" references are legitimate (notes ARE stars). Also: `ur` named `Ctrl+O` "Star Jump" — not a feature; the app calls it the Quick Switcher.
>
> ### 📌 Carried
> The repair's **mid-run Cancel has still never been performed by a human** — fold into §14. **PJ-110** localStorage durability — `app-prefs.json` exists now as the home for the next tenants; `constellation-wab` (unsaved note content) needs a durability design, not a JSON file. **Doc-drift watch:** the translated manuals are partial (1,592–1,941 lines vs the English 2,580) and drift in vocabulary as well as coverage — the Sky View finding suggests a periodic term-consistency sweep against each locale's own `i18n` JSON would be worth a PJ if it recurs.
>
> ### 📌 STILL OPEN, unchanged from v1.74
> PJ-224 gates §13 (Boss ruling required) · PJ-219 design ruling · PJ-225 mtime sweep · PJ-226 walker-classification sweep (20×) · PJ-227 linked-universe phantom rows · PJ-220 (`{name:}` form + args delivery) · PJ-221 `bases.rs:796` APP-KILLER · PJ-222 · `store.ts loadWorkspaces` APP-KILLER · the 2026-07-30 25 lost candidates · the 38-finding register (`wbxz23bdr`) · PJ-172 Sight flakes. **PJ-223** repaired live at §11 Stage 1; formal close at §15.
>
> **Gates:** i18n parity 15/15 ✓ · Rust 1385/0 · vitest 913/913 · svelte-check 0.
>
> ---

**Version 1.74 | 2026-08-09**

> *(See `Constellation Pending Jobs v1.74.md` — the trail is durable, never overwritten.)*
