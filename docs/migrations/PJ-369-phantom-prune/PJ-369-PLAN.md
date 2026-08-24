<!-- PJ-369 design+plan, 2026-08-24. Multi-agent design -> 3 adversaries -> synthesis.
NOT APPROVED; no code written. Provenance of the target corrected after the workflow ran
(see the note below and the ledger). Awaiting Boss approval of the plan + the one question. -->

## Provenance correction (verified after the workflow, 2026-08-24)

The workflow called E:\Cognitive Knowledge a "separate demo universe" and I had earlier called it
"pre-MIG-108 residue of the user's own libraries." Both were partly wrong; verified facts:

- E:\Cognitive Knowledge holds a LEGACY-format universe (a bare `universe.json` at its root, no
  `.constellation/` dir, no version field) whose own name field is **"Constellation Discovery."**
- It is NOT registered, and NOT a child of Eisa Universe.
- Its library folders (Arts & Culture / Humanities / Science / العالم العربي) share names and
  structure with the LINKED universe "Eisa Cognitive Knowledge," and **every one of 40 sampled
  phantom filenames has a same-named note living in that linked universe now.** So the 603 rows
  are stale pointers to an EARLIER home of content the user still has.
- How those rows entered Eisa Universe's index is NOT established and is not guessed here.

Safety consequence: the case for pruning is STRONGER than first thought — these are not this
universe's own notes, and their content is preserved in the linked universe.

---

## VERDICT: PLAN — build it. No unclosable hole.

The three adversaries landed three real kills, but every one is closable with a **scoping change to the classifier**, not a redesign. None touches the core mechanism (the mount-aware "is this note truly gone, or just unreachable?" probe), which all three attacks confirmed is sound. Summary of what each kill forces, all folded into the plan below:

- **Data-loss kill (Attack 0):** the classifier must test a row against the **full set of registered libraries from `try_load_libraries` (before the `is_dir()` filter)** — never the `roots_norm` set the boot walk uses. Otherwise, renaming a still-external pre-MIG-108 library folder in Explorer makes every real note in it look like a phantom and destroys its earned link/review state. This is the one change that turns the design from dangerous to safe.
- **Scope kill (Attack 1):** the target folder `E:\Cognitive Knowledge` is a *separate demo universe*, not old residue of your libraries — provenance restated. The 603 rows are safe (zero earned data, bodies survive in the linked universe, archive cleanly). But the classifier must (a) keep the 9 real linked-universe rows, tested; (b) **refuse to run at all if it cannot fully resolve the federation** (an unreadable linked universe would otherwise make the "don't touch federated rows" check silently vacuous); (c) **never prune any row that carries earned data** (weight/traversal/confidence/status/review) — proof the row is not disposable residue, and protection for the 9 lab-repo rows that carry promoted-confidence links.
- **Invariant kill (Attack 2):** do **not** reuse the boot reconcile's 200-row / 10% safety cap — against your ~2,731-row index it computes to 273, and 603 > 273 would make the button silently do nothing (a false door). The **human confirm dialog showing the exact count is the sanity ceiling.** Also: boot only *counts* (writes nothing); pruning happens only from the Settings command; capture the federation "generation" at the start of classification and stop the loop if you switch universes mid-run.

---

## CONCEPT (the horse)
**Give the user an honest, offered way to remove search-index entries that point at notes confirmed gone from a live disk — never touching a note that is merely on a disconnected drive, in another universe, or that still carries earned link/review history.**

---

## THE ONE QUESTION THE BOSS MUST ANSWER

**When we remove a phantom index row, should we still write its history into your note archive (the "time machine"), even though most of these notes' bodies still exist live inside your `Eisa Cognitive Knowledge` linked universe?**

Plain terms: measured on your live database, **597 of the 603** phantom bodies still exist, alive, in the linked universe. Only ~6 exist *solely* as these dead index rows. "Archive" means each removal appends a "deleted" record (body + change history) to Eisa Universe's own archive ledger — up to 603 entries — so the receipt's promise "history was kept" is literally true and the removal is reversible-in-spirit. The cost: for the 597 live-elsewhere ones, that ledger record is slightly redundant (the note isn't really gone from your knowledge, just from this universe's index).

**My recommendation: archive all — do not skip.** It keeps one behavior for "remove a row whose file is gone" (identical to what boot reconcile already does), makes the receipt honest, honors the "archival, not deletion" law, and is cheap (~2 seconds total). The alternative — skip archiving — saves nothing meaningful and creates a second, quieter delete path that will drift from the main one. If you approve "archive all," I add a new provenance tag `PhantomPrune` so the time machine can tell these apart from ordinary deletes. **This is the only decision I need from you; everything else the plan settles.**

---

## THE PLAN — five commits, each landable and verifiable

Migration-sized because it touches the write path, the indexer, the boot reconcile, and cross-surface state. `/simplify` + diff-scoped `safety-inspection` run before every commit; the whole-app inspection runs at close.

### Step 1 — The hardened classifier (Rust, pure function, writes nothing) + its test battery
Build the discriminator as a standalone predicate with **all** the adversary-forced guards, exercised only by a Rust test battery against a copy of your live database:
- Condition 3 tested against the **full registered-library set** (pre-`is_dir`) — the Attack-0 fix.
- Condition 4 tested against **strictly-loaded** linked-universe roots; the predicate **returns "cannot classify" if the federation can't be fully resolved.**
- **Earned-data gate:** KEEP any candidate whose outgoing links show weight > 1, traversal_count > 0, confidence ≠ hypothesis, or status ≠ active, or whose row carries a review priority or review history.
- **Mount-liveness probe** exactly as designed: recognized-absolute-prefix pre-gate, long-path fail-closed, `try_exists` file check, ancestor climb, `read_dir` readability, case+Unicode-normalization absence cross-check, and the governing **fail-closed law** (any doubt → KEEP). Ancestor-readability memoized (≈5 distinct ancestors) so it's cheap.

**Verification clause:** the test battery is red→green for every case the three attacks named — the 603 demo phantoms → PRUNE; the 9 real linked-universe `.trash` rows → KEEP; the 9 lab-repo earned rows → KEEP; a simulated ejected `E:` → all 603 KEEP; a renamed still-registered external library → all its notes KEEP; an unresolvable linked universe → classifier refuses (0 candidates). No production code path calls it yet; zero rows change.

### Step 2 — Count it at boot, tell the user (write-free), route to Settings
Add a `stalePhantoms` field to the drift report and its frontend mirror; at boot reconcile step 3, run the Step-1 classifier in **count-only mode** (no deletes). Add **one honest sentence** to the drift notice band — "*N entries in the search index point at notes that no longer exist on disk. You can remove these in Settings → Index.*" — gated on the count, and **keep "Repair now" tied only to repairable findings** so it never appears as the answer to a finding it can't fix. Do **not** add the phantom count to `has_findings()`.

**Verification clause (first Boss-testable step):** on boot, the notice band shows the count; the existing reconcile numbers are unchanged; the database row count is identical before and after boot (nothing deleted). Ejecting `E:` → the sentence does not appear (honest silence).

### Step 3 — The prune executor (backend command), driven by the harness only
A user-offered command that: captures the federation generation **at the start of classification**; classifies via Step 1; loops the confirmed list through the **single delete funnel `reindex_delete_note`** with a new `DeleteReason::PhantomPrune`; re-stats each path immediately before deleting (drive-came-back guard); stops the loop on any universe switch; and returns a receipt (removed / skipped / failed). **No safety-cap silent-abort** — the human confirm is the ceiling. Archive behavior follows your answer to The One Question.

**Verification clause:** run against a *copy* of your live DB — all 603 removed and archived across every path-bearing table (links, aliases, body, sky, review, tags, FTS, term vocabulary), the 9 foreign and 9 earned rows untouched; a second run removes 0 (idempotent/resumable); a simulated mid-run universe switch stops cleanly with no rows written to the wrong database.

### Step 4 — Settings → Index control, danger-confirm, receipt (Boss-testable)
Add "**Remove stale index entries**" to the existing Index-repair block: shows the count, opens the shared confirm dialog with `danger: true` (like Clear-history), runs the Step-3 command, and renders a recover-on-mount receipt beneath it ("*Removed N stale entries. Their history was kept in the note archive.*"), with an honest partial-run line if a drive drops mid-clean. All new strings added to **all 15 locales** (en is source; add `plurals.entries` per-language if absent).

**Verification clause:** the test tutorial goes `tutorial-auditor → ui-inspector → panel → Boss` (per the Test Pipeline law). Boss removes the 603, sees the receipt, reboots, confirms the notice band is now silent and search/backlinks/Sky View no longer surface the dead entries.

### Step 5 — /migration Phase-4 audit (three agents in parallel)
Invariants, drift (any new guard the system doesn't know about), and migration path (first boot, drive offline at prune time, interrupt mid-prune, universe switch mid-prune). Whole-app `safety-inspection` sweep. Orientation + Pending-Jobs ledger + help/manual updated in the closing commit.

---

## INVARIANTS THAT MUST NOT BREAK
1. Never delete a real note's row.
2. Never touch a linked-universe row (write-sovereignty).
3. Never delete anything on an unmounted/offline drive (WA#4).
4. **Never prune a row under any registered own-library path — even when that library's root currently fails `is_dir()`** (the moved/renamed-folder case; Attack 0).
5. **Never prune a row carrying earned data** — weight/traversal/confidence/status/review (Attack 1).
6. **Never prune when the federation can't be fully resolved** (Attack 1).
7. Boot only counts; all deletion is user-offered from Settings (Attack 2).
8. Every removal is offered-with-receipt, never silent (project law).
9. All removal goes through the single funnel `reindex_delete_note`, archive-first — no hand-rolled bulk DELETE.
10. No boot-time or hot-path regression (ancestor probe memoized; classification lock-free, post-paint).

---

## WHAT THE BOSS MUST APPROVE
1. **This plan** (approval = build approval; I cascade Steps 1→5, stopping only at the Step-2 and Step-4 test gates and Phase-4).
2. **The One Question above** — archive all (my recommendation) vs. skip archiving for phantoms. This is the single decision that blocks Step 3.

Two settled points, stated so they aren't mistaken for open questions: pruning is a **permanent count at boot + a one-off user-offered command** (not an automatic boot deletion), and the target scope is **only truly-orphan rows that pass every guard above** — the general "prune any outside-root row" framing the design started with is explicitly narrowed by Steps 1's guards. A minor build-time reconciliation (the 2,731 vs 8,031 total-rows discrepancy between two design notes) doesn't affect any decision here — the count is computed live at runtime.