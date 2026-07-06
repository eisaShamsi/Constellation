# MIG-094 — The Orphan/Fragile Connectivity Vocabulary — Plan

**Date:** 2026-07-06 · **Status:** **PLAN — awaiting Boss approval (Plan approval = build approval).** Follows the ratified Architect (`docs/MIG-094-Orphan-Fragile-Vocabulary-Architect.md`). PJ-069 Step 1.

## Rulings locked (Boss, 2026-07-06)

1. **Two named concepts.** `UNREFERENCED` (no incoming — `incoming_count==0`) is distinct from `ISOLATED` (no links at all — `incoming_count==0 && outgoing_count==0`). Search + Collections keep ISOLATED; Reviewer/360/Tension/Sky/CNS use UNREFERENCED.
2. **Per-surface stub filter.** The connectivity predicate is one shared definition; each surface AND-s in its own `word_count` floor if it wants one (Reviewer/360/Tension keep `>20`; Sky/CNS don't gate). Substance is an orthogonal axis, never baked into the shared predicate.
3. **Verdict changes approved, with before/after tests.** The four correctness fixes (360's `word_count` source; Tension → alias-aware; Search → alias-aware; Sky counts re-sourced from `note_meta`) ship each with a tutorial-framed before/after from the P0 harness.

`FRAGILE` (`incoming_count>=5 && derives-from-support<=1`) is a separate named concept — one implementation replacing the current four. The false members (cataloger `degree<2`, livePreview membership gate) are renamed out of the orphan family; `weak_foundations` and community-gaps stay out of class.

## The end-state (one home per named concept)

- **Rust** `note_meta`-backed helpers (WHERE-fragment **and** `fn(&row)->bool`): `is_unreferenced` = `incoming_count==0` · `is_isolated` = `incoming_count==0 && outgoing_count==0` · `is_fragile` = `incoming_count>=5 && COALESCE(json_extract(outgoing_link_types_json,'$."derives-from"'),0)<=1`.
- **Frontend** predicate module mirroring `collectionChips` (`isUnreferenced`/`isIsolated`/`isFragile`) over `note_meta` facts.
- Every surface reads a helper + AND-s its own substance filter; zero read-time re-walks remain.

---

## Steps (each one landable commit + verification clause)

**§1 — P0 · verdict-parity harness (no behavior change).** A read-only diagnostic that, for the active universe, computes each surface's *current* orphan/fragile verdict and logs the disagreement set (which notes are orphan in Sky but not Reviewer due to the floor; which flip on alias-awareness; the 360 `word_count`-source diff). *Verify:* runs on the 7,600-note universe and prints concrete before-numbers; no surface behavior altered. **This is the Reproduce-First baseline every later verdict change is measured against.**

**§2 — P1 · land the shared helpers (dormant).** Add the 3 Rust helpers + the frontend module; no call site switched. *Verify (build-gate):* a Rust test asserts `is_fragile` via the JSON map == via the old `note_links COUNT(*)` subquery across a real `note_meta` snapshot — **zero mismatch, or the mismatch is characterized and surfaced for a ruling before proceeding** (Invariant 4). If the JSON semantics differ, stop and report — a dedicated column may be needed.

**§3 — P2 · no-change swaps (byte-parity).** Re-point `review.rs` (orphan + fragile lenses + note-tab badge) and `collectionChips.ts` (Unlinked → `is_isolated`) to the helpers. *Verify:* the harness shows Reviewer queue + note-tab badge + Collections chip produce **identical** verdicts before/after (zero diff). No Boss test needed (nothing changes).

**§4 — P3 · 360 + Tension (VERDICT CHANGE — Boss-testable).** Re-point `inspector360.rs` (`word_count`→`note_meta.word_count`, `out_derives`→JSON map) and `tension.rs` (inbound→alias-aware `incoming_count`, `out_derives`→JSON map). Help + User Manual updated in the same commit. *Verify:* the harness diff vs §1 == exactly the approved flip set (alias-linked notes; `word_count`-source-diff notes), nothing else; 360 and Tension now agree with Reviewer on the shared question. **Boss test:** a tutorial-framed before/after on 360 + Tension.

**§5 — P4 · Search filter (VERDICT CHANGE — Boss-testable).** Replace `search.rs:6675`'s `_incoming_targets` temp-table pass with `is_isolated` over `note_meta` (ISOLATED stays per Ruling 1). Search help updated. *Verify:* result set changes only by the approved alias-awareness flip; **perf measured before/after on 7,600 notes — must be ≤ prior** (the temp-table rebuild is gone). **Boss test:** before/after on the Search "Orphans" chip.

**§6 — P5 · Sky View internal + payload re-source (Boss-testable).** (a) Make the `graphEngine` ring (`:2193`) and filter (`:365`) read one UNREFERENCED source so Sky stops contradicting itself; (b) JOIN `note_meta.incoming_count/outgoing_count` into the sky snapshot in `cache.rs` so Sky ring + CNS stat read the canonical column, not a `sky_links` re-derivation. *Verify:* a note linking out with no backlinks no longer both passes the filter and wears the ring; Sky orphan-ring count == CNS orphan stat == `incoming_count==0` count. **No boot/frame regression** (measure against the ~17s SKY read, MIG-079 §C.2d — if it regresses, fall back to reconciling only the two internal sites and document the residual). **Boss test:** Sky View + CNS.

**§7 — P6 · rename false members + close-out.** Rename `cece/graph.rs:97` → a classifier-abstain concept (`min_classified_neighbors`) and `livePreview.ts:1044` → a note-in-graph render-eligibility check (behavior identical, no "orphan" label). Delete the dead `buildSkyData` total-degree fallback if confirmed unreachable. *Verify:* grep confirms no user surface labels a non-degree threshold "orphan"; the harness reports each named concept computed once; the §1 disagreement set is reduced to zero except the intentional UNREFERENCED-vs-ISOLATED distinction. **SO#6 orientation v-bump + session log + help/manual in this commit.** Then `/simplify` on the full diff; then the `/migration` Audit trio.

## Boss-test checkpoints (Testing Instructions Rule — staged, one at a time)

Steps §4, §5, §6 each pause for a tutorial-framed Boss test (define the surface, walk through it, before→after→expected). §1–§3 and §7 are internal/parity-verified. Per the staged-tests standing order, I send one test at a time and wait for findings.

## What this does NOT do (scope fences)

- Does **not** widen the fragile predicate to count `supports` as a foundation link (latent design question → future concept paper; preserve current `derives-from`-only behavior).
- Does **not** merge UNREFERENCED and ISOLATED (Ruling 1).
- Does **not** touch `weak_foundations` (link-confidence lens) or CNS community-gaps (both out of class).
- Does **not** change `note_meta.incoming_count` semantics (canonical, untouched).

---

**On approval I cascade §1→§7**, pausing only at the three Boss-test checkpoints and at §2's build-gate if the fragile JSON parity fails. The old surfaces keep running until each validated swap.
