# Handover — 2026-07-29 (session C close): three APP-KILLERs closed

**Supersedes** `HANDOVER-2026-07-29-B-pj182-close.md`.

**Read FIRST, in this order:** `docs/Constellation Orientation & Onboarding v3.77.md` →
`docs/Constellation Pending Jobs v1.59.md` (► Next action) → this file →
**`docs/LESSONS-LEARNED.md` LL-039, LL-040 and LL-041** — three laws written today, all three
from defects found *inside my own fixes*.

**Branch:** `main` @ **`450123d1`**, pushed. Working tree **clean**.
**Gates:** vitest **64 files / 724 tests** (started the day at 59/678) · Rust **1280 / 0**
(11 ignored) · svelte-check **0 errors** · Sight perf **SERIAL lane** (PJ-172) 31/31 ·
`tests/g4/frontmatterRoundtrip.test.ts` still permanently RED by design.
**Binary:** rebuilt at 18:10 and Boss-tested. Rebuild before the next Boss test.

---

## ► THE NEXT JOB IS **MIG-104 Slice 8 + 8b**

The archive hook MUST go **BEFORE** the `DELETE FROM note_meta` at `search.rs:9845`, because
FK enforcement fires the CASCADE there — a hook at the later explicit purge archives
**NOTHING** (`tests_stage0_delete_order_defect`). **8b** adds the note BODY (Boss decision #6).
Then Slices 9–15, then **MIG-105 Phase 2**.

**Owed to the Boss first: a triage of PJ-187's remaining 49 sites** (the whole-app register,
`lab/reports/SAFETY-INSPECTION-2026-07-29-pj182.md`). Its headline was closed today; four
more APP-KILLERs in it have had no ruling.

---

## What shipped — three APP-KILLERs, all Boss-validated

| | |
|---|---|
| **PJ-182** | The zero-indent YAML block list. Filed as one function; it was **20 surfaces across two languages**. One shared predicate + extractor per language (`isYamlSeqItem`/`yamlSeqItemValue`, and the new `src-tauri/src/yaml_lines.rs`), routed through every surface. Three further shapes closed in the same pass |
| **PJ-181** | A merely-VIEWED note's cached copy overwriting a newer external edit. The write-ahead net entry now records **why** it exists (`snapshot?: boolean`) |
| **PJ-187 headline** | Cross-note property bleed — note A's properties written onto note B. The rows now carry **provenance** (`rowsBelongToTarget`) |

---

## ⚠ THE PATTERN THAT MATTERS MORE THAN ANY OF THE THREE FIXES

**Every one of today's fixes contained a defect that a later pass found.** Not one was caught
by the suite; all were caught by `/simplify` or the safety inspection, and every one was
*measured* before being believed.

1. **PJ-182** — `/simplify` found **eight sites still hand-rolling** the predicate the change
   had just created (including the function whose comment the new module quotes as the rule's
   origin), and the inspection found **one half-routed site** where I had converted the
   block-skip half and left the key-match half. → **LL-039**.
2. **PJ-181** — the inspection found an **APP-KILLER in the fix**: the snapshot flag was
   derived from `needsDiskSave`, which is NotePane's view-level `dirty`, cleared at
   save-REQUEST time and never restored on failure. After a failed save the fix would have
   **deleted the user's only copy of unsaved work.** → **LL-040**.
3. **PJ-187** — every identity guard in the chain was present and passing; none of them asked
   whether the *payload* belonged to the destination. → **LL-041**.

**And two of my tests were worthless when written** — one passed because I handed
`flushIfDirty` a path where a save-ENV was expected, the other passed with the flag
hard-coded to `true`. Both were caught only by replacing the assertion with the old value and
watching it fail. **Do that to every guard you write.**

---

## Owed to the Boss

**PJ-187 register triage** — 49 remaining sites, 4 unruled APP-KILLERs. Notably
**`yamlDoc.ts:311`**: `composeFrontmatter`'s malformed-YAML passthrough discards *every*
property edit on such a note and reports the save as **successful** (`hasErrors` has zero
consumers outside `yamlDoc.ts`).

**PJ-166 — eighth strike.** The inspection returns `mode: "whole-app"` every time it is
invoked diff-scoped: ~30 min and ~10 M tokens for what should be a 13-file gate. For PJ-187 I
deliberately wrote a **focused 4-lens hunt over the diff instead** — 20 agents, ~3 M tokens —
which is what diff-scoped was meant to be, and it found the PJ-181 app-killer. That is a
usable pattern until PJ-166 is fixed.

**The docs-sweep gap (Boss-found today).** The PJ-181 commit shipped code + orientation +
ledger with **no manual and no help file**. The audit that followed: **only 2 of the last 20
commits carried manual/help, and none of the three named "session-close PCS" did.** Most of
the rest are internal slices with nothing user-facing — but the session-close PCS is exactly
where SO#2's sweep belongs and it does not happen there. **Proposed and not yet ruled on:**
make a close incomplete until the docs sweep is either done or the log records "no
user-facing change, because…". PJ-187 was committed with its docs ×15 *in the same commit* to
set the precedent.

---

## State of standing (SO#5)

**(a) Verified-shipped & protected.** MIG-104 Slices 0–7 · PJ-174 AK-1/2/3 · MIG-107 §0–6 ·
PJ-178/179 · **PJ-182** · **PJ-181** · **PJ-187 headline**. Suite 678 → 724; Rust 1261 → 1280.

**(b) At-risk / in-flight.** Nothing uncommitted. `PROPS_SINGLE_OWNERSHIP` still **retained**
per Boss ruling — remove when MIG-104 closes, not before.

**(c) Known-broken.** **PJ-187** (49 sites, 4 unruled APP-KILLERs) · PJ-176 · PJ-177 · PJ-152 ·
PJ-158 · PJ-159 · PJ-162 · PJ-163 · PJ-168 · PJ-170 · PJ-140 (~37) · the earlier PJ-174 register.

**(d) Pending, not started.** MIG-104 Slices 8–15 · MIG-105 Phase 2 · MIG-106/PJ-169 · PJ-180 ·
PJ-183 (the Rust block WALKER — eleven sites still own a private state machine) · PJ-184
(a block scalar borrows `nested-map` and is labelled "Nested map" in 15 languages) · PJ-185 ·
PJ-186 · PJ-188 (the WAB localStorage blob is unbounded, uncapped, and fails silently on
quota) · PJ-189 · PJ-190 · PJ-191 · **PJ-137 — strike SIX**, and worth recording:
`projectProps` / `parseFrontmatterDoc` are **test-only today**, so a spec-compliant projector
already sits beside the hand-rolled one, dark. That is the cheapest possible starting point.

---

## Watch-items

- **Transient cargo failures now hit `cargo test` too.** Twice today a bogus *"could not
  compile … due to 1 previous error"* with no error line anywhere; both times a clean re-run
  passed. Match **both** `LNK1104` and `build failed`, and re-run once before believing it.
- **The Boss's test Universe is `E:\Constellation Universes\Eisa Cognitive Knowledge`**
  (7,820 notes). **It is NOT in the app's registry** — `universes.json` lists only `كون عيسى`
  and points `active_id` at it. **ASK; do not infer it from the registry.**
- **Bases cells: only `prop.<key>` columns are editable, by DOUBLE-CLICK.** `note.*` columns
  are read-only, and **`prop.tags` is blank for every note by design** (tags live in
  `tags_json`, never `properties_json`).
- **The Boss is on Windows PowerShell.** A `bash`-tagged command block with `printf` fails for
  him. Give PowerShell, or run it yourself. Three environment-unchecked instructions today.
- **There is NO component-test harness** (no jsdom, no testing-library, no component tests).
  Design guards as pure predicates in a module, or they cannot be tested at all.
- **Test artefacts left in the Boss's Universe** (his to keep or delete): `PJ-182 Test\`
  (7 notes) and `.constellation\bases\PJ-182 Test.base`.

---

## ► READY-TO-PASTE NEXT-SESSION PROMPT

```
Continue Constellation on branch main (@450123d1, clean). First: git pull; read
docs/Constellation Orientation & Onboarding v3.77.md, then docs/Constellation Pending Jobs
v1.59.md (► Next action), then lab/reports/HANDOVER-2026-07-29-C-three-appkillers.md. Note
LL-039, LL-040 and LL-041 in docs/LESSONS-LEARNED.md — three laws written last session, all
three from defects found INSIDE my own fixes, none caught by the suite.

The next job is MIG-104 Slice 8 + 8b: the archive hook MUST go BEFORE the
DELETE FROM note_meta at search.rs:9845, because FK enforcement fires the CASCADE there — a
hook at the later explicit purge archives NOTHING (tests_stage0_delete_order_defect). 8b adds
the note body (Boss decision #6). Then Slices 9-15, then MIG-105 Phase 2.

Owed to the Boss FIRST: a triage of PJ-187's remaining 49 confirmed sites
(lab/reports/SAFETY-INSPECTION-2026-07-29-pj182.md) — 4 unruled APP-KILLERs, notably
yamlDoc.ts:311, where composeFrontmatter's malformed-YAML passthrough discards every property
edit on such a note and reports the save as successful. Also unruled: the docs-sweep gap —
only 2 of the last 20 commits carried manual/help and none of the three named "session-close
PCS" did; the proposal is that a close is incomplete until the sweep is done or the log
records "no user-facing change, because...".

Keep the PROPS_SINGLE_OWNERSHIP toggle until MIG-104 closes (Boss-ruled).

Working rules that cost real time last session: ASK which Universe before placing test notes
(it is E:\Constellation Universes\Eisa Cognitive Knowledge, which is NOT in the app registry).
The Boss is on Windows PowerShell — never hand him a bash command. There is NO component-test
harness, so design guards as pure predicates in a module or they cannot be tested. Prove every
new assertion load-bearing by replacing it with the OLD value and watching it fail — two tests
last session were worthless until that was done. Run the Sight perf tests as a SERIAL lane
(--no-file-parallelism --maxWorkers=1, PJ-172), rebuild the binary before any Boss test and
verify sources → build/index.html → constellation.exe by mtime, and re-run once before
believing a cargo failure.

PJ-166 is at eight strikes (the inspection returns whole-app when invoked diff-scoped). Until
it is fixed, use the focused per-diff hunt pattern from last session's PJ-187 work — ~20
agents over the changed files, which is what diff-scoped was meant to be.
```
