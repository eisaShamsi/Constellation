# Handover — 2026-07-29 (session B), PJ-182 closed

**Read FIRST, in this order:** `docs/Constellation Orientation & Onboarding v3.75.md` →
`docs/Constellation Pending Jobs v1.57.md` (► Next action) → this file →
**`docs/LESSONS-LEARNED.md` LL-039** (written this session, from my own mistakes — twice).

**Branch:** `main` @ **`32fcface`**, pushed. Working tree **clean**.
**Gates:** vitest **62 files / 716 tests** · Rust **1280 / 0** (11 ignored) ·
svelte-check **0 errors** · Sight perf **SERIAL lane** (PJ-172) **31/31** ·
`tests/g4/frontmatterRoundtrip.test.ts` still permanently RED by design (4 failed / 1 passed).
**Binary:** rebuilt and Boss-tested at `32fcface`'s code. Rebuild before the next Boss test.

---

## ► THE NEXT JOB IS **PJ-181**

**APP-KILLER, `store.ts:2448`.** The write-ahead net is restored on a `cid_cn` match with
**no freshness check against the disk bytes it just read**, and a net entry is stashed for
merely-VIEWED notes. View a note → close it → it is edited externally (Syncthing / second
device / git pull; the watcher ignores it because the note is closed) → reopen shows the
**stale** content with the model born dirty → the first tab switch writes it over the newer
file. `restoreSessionTabs` (`store.ts:2930-2940`) already solves exactly this on the sibling
path — copy that arbitration.

Then **MIG-104 Slice 8 + 8b**: the archive hook MUST go **BEFORE** the
`DELETE FROM note_meta` at `search.rs:9845`, because FK enforcement fires the CASCADE there —
a hook at the later explicit purge archives **NOTHING** (`tests_stage0_delete_order_defect`).
8b adds the note BODY (Boss decision #6). Then Slices 9–15, then **MIG-105 Phase 2**.

---

## What shipped

**PJ-182 — CLOSED, Boss-validated 3/3.** Filed as one function; it was **20 surfaces across
two languages**, six of them APP-KILLERs not previously known. One shared predicate **and
extractor** per language — `isYamlSeqItem` / `yamlSeqItemValue` (`store.ts`) and the new
**`src-tauri/src/yaml_lines.rs`** — routed through every surface. `parseFrontmatter`'s three
divergent branch probes are now ONE block-extent scan + one classification. Three further
shapes closed in the same pass (block scalars · flow sequences · comments in a block), plus
an indentation-independent one found in-pass: **every ikhtilāf edit was a silent no-op**.

---

## ⚠ READ THIS BEFORE TOUCHING THE FRONTMATTER LAYER AGAIN

**Three defects were found IN the fix, by the two passes that ran after it.** Every one was
reproduced by running before it was fixed. If you change this layer, run both passes again:

1. **`/simplify`'s reuse review found EIGHT sites still hand-rolling the predicate** the
   change had just created — including `search.rs::parse_frontmatter`, *the function whose
   comment the new module quotes as the rule's origin*, and the direct siblings of two
   functions that WERE routed.
2. **The safety inspection found one HALF-routed site** — `sources/mod.rs` had its
   block-SKIP half converted and its key-MATCH half left testing the trimmed line, so an
   indented `sources:` was **deleted**. Half a routed site reads as fixed and is not.
3. **The Rust and TS twins diverged on comments** — only the Rust half was wrong, and my own
   widening turned that into orphaned items: a NEW instance of the class being fixed.

That is **LL-039**, and it is the thing to internalise: *extracting a shared helper does not
end the drift — ROUTING every site does, and a site has more than one half. The sweep finds
where the bug FIRES; it does not find where the concept LIVES.*

---

## Owed to the Boss — two rulings

**PJ-187 — the whole-app inspection register.** 50 unique confirmed sites
(5 APP-KILLER · 10 HIGH · 29 MED · 7 LOW): `lab/reports/SAFETY-INSPECTION-2026-07-29-pj182.md`.
Headline **APP-KILLER `PropertyEditor.svelte:974`** — the right-sidebar Properties panel is
never `{#key}`-remounted, so a pending 800 ms debounce that survives an in-place navigation
(click a wikilink within 800 ms of a property edit) writes note **A**'s properties onto note
**B**, durably and silently. Its NotePane twin IS protected by a `{#key}`; the sidebar one is
not. Also inside it: **`yamlDoc.ts:311`** — the malformed-YAML passthrough discards every
property edit on such a note and reports the save as **successful** (`hasErrors` has zero
consumers outside `yamlDoc.ts`), and **`yamlDoc.ts:362`** — the CST splice+append deletes
YAML comments attached to the edited key, on the app's single most common frontmatter edit.

**PJ-166 — EIGHTH strike.** This build's inspection was invoked diff-scoped with
`args.files` and returned `mode: "whole-app"`: 88 agents, ~10.8 M tokens, ~30 minutes for a
13-file gate. It earned its cost *again* (it caught the half-routed `sources/mod.rs`), but
the per-build gate the standing order requires still does not exist, eight attempts in.

---

## State of standing (SO#5)

**(a) Verified-shipped & protected.** MIG-104 Slices 0–7 · PJ-174 AK-1/2/3 · MIG-107 §0–6 ·
PJ-178/179 · **PJ-182 (all five slices, Boss-validated 3/3)**. Suite 678 → 716; Rust 1261 → 1280.

**(b) At-risk / in-flight.** Nothing uncommitted. `PROPS_SINGLE_OWNERSHIP` still **retained**
per Boss ruling — remove when MIG-104 closes, not before; its legacy branches have not been
exercised since MIG-107 Slice 3.

**(c) Known-broken.** **PJ-181** (► next) · **PJ-187** (the new register, 5 APP-KILLERs) ·
PJ-176 · PJ-177 · PJ-152 · PJ-158 · PJ-159 · PJ-162 · PJ-163 · PJ-168 · PJ-170 · PJ-140 (~37) ·
plus the earlier PJ-174 sweep register.

**(d) Pending, not started.** MIG-104 Slices 8–15 · MIG-105 Phase 2 · MIG-106/PJ-169 ·
PJ-180 · PJ-183 (the Rust block WALKER — eleven sites still own a private state machine) ·
PJ-184 (a block scalar borrows `nested-map` and is labelled "Nested map" to the user in 15
languages) · PJ-185 · PJ-186 · **PJ-137 — now at strike SIX**, and worth recording:
`projectProps` / `parseFrontmatterDoc` are **test-only today**, so a spec-compliant projector
already sits beside the hand-rolled one, dark. That is the cheapest starting point.

**(e) Process.** **PJ-146** — the Properties help topic is still English-only; the User
Manual's Properties section WAS updated in all 15 languages this pass. **PJ-172** — the Sight
perf serial lane still has no permanent home; run it manually.

---

## Watch-items

- **Transient cargo failures now hit `cargo test` too**, not just `cargo build --release`.
  Two clean re-runs this session followed a bogus *"could not compile … due to 1 previous
  error"* with no error line anywhere in the output. Match **both** `LNK1104` and
  `build failed`, and re-run once before believing a failure.
- **The Boss's test Universe is `E:\Constellation Universes\Eisa Cognitive Knowledge`**
  (7,820 notes; it federates all of `E:\Cognitive Knowledge`). **It is NOT in the app's
  registry** — `%APPDATA%\world.uconstellation.app\universes.json` lists only `كون عيسى` and
  points `active_id` at it. Ask before assuming which Universe; I got this wrong.
- **Bases cells: only `prop.<key>` columns are editable, by DOUBLE-CLICK.** `note.*` columns
  are read-only, and **`prop.tags` is blank for every note by design** (tags live in
  `tags_json`, never `properties_json` — see the test `tags_are_excluded_from_properties`).
  I wrote a test step against each of those before checking. Read the component first.
- **Test artefacts left in the Boss's Universe** (his to keep or delete):
  `PJ-182 Test\` (4 notes) and `.constellation\bases\PJ-182 Test.base`.

---

## ► READY-TO-PASTE NEXT-SESSION PROMPT

```
Continue Constellation on branch main (@32fcface, clean). First: git pull; read
docs/Constellation Orientation & Onboarding v3.75.md, then docs/Constellation Pending Jobs
v1.57.md (► Next action), then lab/reports/HANDOVER-2026-07-29-B-pj182-close.md. Note LL-039
in docs/LESSONS-LEARNED.md — written last session after /simplify and the safety inspection
each found a defect in the fix that had just been made to obey the Whole-Ecosystem Fix Law.

The next job is PJ-181 (APP-KILLER, store.ts:2448): the write-ahead net is restored on a
cid_cn match with NO freshness check against the disk bytes it just read, and a net entry is
stashed for merely-VIEWED notes. View → close → the note is edited externally (Syncthing /
second device / git pull; the watcher ignores it because the note is closed) → reopen shows
the STALE content with the model born dirty → the first tab switch writes it over the newer
file. restoreSessionTabs (store.ts:2930-2940) already solves exactly this on the sibling
path — copy that arbitration. Reproduce it first.

Two rulings are owed to the Boss before or alongside that work:
 - PJ-187, the 2026-07-29 whole-app inspection register: 50 confirmed sites, 5 APP-KILLERs
   (lab/reports/SAFETY-INSPECTION-2026-07-29-pj182.md). Headline PropertyEditor.svelte:974 —
   the right-sidebar Properties panel is never {#key}-remounted, so a pending 800ms debounce
   surviving an in-place navigation writes note A's properties onto note B.
 - PJ-166, now EIGHT strikes: the inspection returns mode "whole-app" every time it is
   invoked diff-scoped (~30 min, ~10M tokens per build).

When MIG-104 resumes it is at Slice 8 + 8b: the archive hook MUST go BEFORE the
DELETE FROM note_meta at search.rs:9845, because FK enforcement fires the CASCADE there — a
hook at the later explicit purge archives NOTHING (tests_stage0_delete_order_defect). 8b adds
the note body. Keep the PROPS_SINGLE_OWNERSHIP toggle until MIG-104 closes (Boss-ruled).

The Boss's test Universe is E:\Constellation Universes\Eisa Cognitive Knowledge — ASK, don't
infer it from the app registry, which lists only كون عيسى. Bases cells: only prop.<key>
columns are editable and only by double-click; prop.tags is blank for every note by design.
Run the Sight perf tests as a SERIAL lane (--no-file-parallelism --maxWorkers=1, PJ-172),
rebuild the binary before any Boss test and verify sources → build/index.html →
constellation.exe by mtime, and re-run once before believing a cargo failure — transient
"could not compile" with no error line now hits cargo test as well as cargo build.
```
