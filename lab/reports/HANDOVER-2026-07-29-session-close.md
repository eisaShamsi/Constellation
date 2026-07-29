# Handover — 2026-07-29 session close

**Read FIRST, in this order:** `docs/Constellation Orientation & Onboarding v3.74.md` →
`docs/Constellation Pending Jobs v1.56.md` (► Next action) → this file →
`docs/migrations/MIG-107-Architect-Plan-props-single-ownership.md` (§5.6 + §5.7 — the defects and
what was deferred) → **`docs/LESSONS-LEARNED.md` LL-037 and LL-038 (incl. rule 6) — all three
written this session, all three from my own mistakes.**

**Branch:** `main` @ **`11a199e5`**, pushed. Working tree **clean**.
**Gates:** vitest **59 files / 678 tests** · Sight perf **SERIAL lane** (PJ-172) **31/31** ·
svelte-check **0 errors** · Rust **1261/0** (11 ignored).
**Binary:** `src-tauri/target/release/constellation.exe` — rebuild before any Boss test; the last
build predates the Slice-6 `/simplify` edits.

---

## ► THE NEXT JOB IS **PJ-182**, NOT MIG-104

**A reproduced content-loss bug that silently deletes the user's authored data.** It is small, it is
proven, and it outranks feature work.

```
parseFrontmatter('---\ncid_cn: ABCD\ntitle: T\ntags:\n- alpha\n- beta\n---\nbody')
  →  tags: { value: '', listItems: [], type: 'list' }
```

A **zero-indent** YAML block list is valid YAML, is what PyYAML emits, and is common in imported and
hand-authored vaults. All three multi-line branches in `parseFrontmatter` (`store.ts:1900`, `:1981`,
`:2009`) require `/^\s+-\s/` — a **leading space** — so none fire and the key falls through to the
scalar path as an empty list. The panel then shows it as empty, and the next property write takes
`composeFrontmatter`'s ADD branch, splices the block out and appends a fresh one: **every item is
deleted from the `.md`**, with no error, and the output re-parses cleanly so nothing downstream
notices. Same for **`aliases`** — a link-resolution key, so losing it silently breaks every backlink
through that alias. `addTagToNote` is a trigger; the batch tagger multiplies the blast radius.

Then **PJ-181** (APP-KILLER, `store.ts:2448`): the write-ahead net is restored on a `cid_cn` match
with **no freshness check against the disk bytes it just read**, and a net entry is stashed for
merely-VIEWED notes. View → close → the note is edited externally (Syncthing / second device / git
pull; the watcher ignores it because the note is closed) → reopen shows the **stale** content with
the model born dirty → the first tab switch writes it over the newer file. `restoreSessionTabs`
(`store.ts:2930-2940`) already solves exactly this on the sibling path — copy that arbitration.

---

## What shipped this session

| | |
|---|---|
| **MIG-104 Slice 7** | snapshot + 2 MB compactor; the ledger's load is bounded. Boss-validated |
| **PJ-174 #1 (AK-1)** | the rename cascade's mid-walk tab — **four** holes. Boss-validated ×2 |
| **MIG-107** | props single ownership, **all six slices**, five Boss-validated |
| **PJ-178 / PJ-179** | blank-row `"": ""` written to file; stage picker opening on the wrong entry |

**MIG-104 is PAUSED CLEANLY at the Slice 7/8 boundary.** Slice 8 has one hard constraint already
proven by test: the archive hook must go **BEFORE `DELETE FROM note_meta`** (`search.rs:9845`) —
FK enforcement fires the CASCADE there, so a hook at the later explicit purge archives **NOTHING**
(`tests_stage0_delete_order_defect`). 8b adds the note BODY (Boss decision #6).

---

## ⚠ TWO CLAIMS IN THE SLICE-5 COMMIT ARE FALSE — DO NOT CITE THEM

Both verified false by grep, corrected in the session log and MIG-107 §5.7. History is not rewritten.

1. *"No whole-array property write remains in the app."* — **`addTagToNote` (`+layout.svelte:6536`)
   still does one**, and it is the exact writer `propsCommit.ts`'s own header names as the canonical
   foreign key.
2. *"There is no callback left to omit."* — **the `onStageChanged` push channel is still fully
   wired** (`+layout:8533`, `:8742`). A pull mechanism was added BESIDE it, not instead of it, so
   that display fact now has two owners.

Both are folded into **PJ-180** (MIG-107's altitude follow-ups), along with: a by-name
`setPropByName` intent; a generic `noteProp(id, key)` read facade — which is what lets the push
channel be **deleted** rather than shadowed; splitting `saveTabContent`'s two modes
(`propsAlreadyInModel` deletes the function's first half rather than tuning it); a `propsCommit`
draft handle; and `buildFullContent` being a **lossier** composer than the `compose()` every other
writer uses.

---

## State of standing (SO#5)

**(a) Verified-shipped & protected.** MIG-104 Slices 0–7 · PJ-174 AK-1/2/3 · MIG-107 §0–6 ·
PJ-178/179. Suite **607 → 678**.

**(b) At-risk / in-flight.** Nothing uncommitted. The **`PROPS_SINGLE_OWNERSHIP` toggle is
deliberately RETAINED** (Boss ruling) — remove it when MIG-104 closes, not before. Its legacy
branches have not been exercised since Slice 3, so if it is ever flipped, test before trusting it.

**(c) Known-broken.** **PJ-182** (► next) · **PJ-181** · PJ-176 · PJ-177 · PJ-152 · PJ-158 · PJ-159 ·
PJ-162 · PJ-163 · PJ-168 · PJ-170 · PJ-140 (~37) · plus the un-triaged sweep registers (PJ-174).

**(d) Pending, not started.** MIG-104 Slices 8–15 · MIG-105 Phase 2 · MIG-106/PJ-169 · PJ-180 ·
PJ-171 · PJ-172.

**(e) Process.**
- **PJ-166 struck SEVEN times this session.** Every inspection invoked diff-scoped returned
  `mode: "whole-app"` — ~32 min and ~10 M tokens each. They earned their cost (they caught the
  Slice-7 TOCTOU, the `renameItem` APP-KILLER, PJ-181 and PJ-182), but **the per-build gate the
  standing order requires still does not exist.** This is now the biggest process gap.
- **PJ-146** — the Properties help topic exists in **English only**; none of the 14 translated help
  dirs carry it. Not fabricated to claim ×15 coverage.
- **PJ-172** — the Sight perf tests still need a permanent serial lane; run manually as one all session.

---

## Watch-items

- **`cargo build --release` intermittently fails transiently** — usually `LNK1104`, but once with
  only *"build failed, waiting for other jobs to finish"* and no error line. A retry loop that
  matches **only** `LNK1104` will bail on that variant; match both.
- **Rebuild the binary before every Boss test**, and verify the chain `sources → build/index.html →
  constellation.exe` by mtime. `cargo build --release` alone re-embeds a stale `build/`.
- **The Boss found six defects the 678-test suite did not** — three of them mine, in the same
  session. The mandatory Boss-test-before-commit gate is doing real work; do not shortcut it.
- **Every test instruction is a tutorial.** A one-line step ("enter Focus, type, exit") drew a fair
  *"What do you mean?"* — define the feature, then walk the clicks.

---

## ► READY-TO-PASTE NEXT-SESSION PROMPT

```
Continue Constellation on branch main (@11a199e5, clean). First: git pull; read
docs/Constellation Orientation & Onboarding v3.74.md, then docs/Constellation Pending Jobs v1.56.md
(► Next action), then lab/reports/HANDOVER-2026-07-29-session-close.md. Note LL-037 and LL-038
(including rule 6) in docs/LESSONS-LEARNED.md — all three written last session from defects the Boss
found that a 678-test suite did not.

The next job is PJ-182, NOT MIG-104. It is a REPRODUCED content-loss bug that silently deletes the
user's data: a zero-indent YAML block list (`tags:` newline `- alpha`, no leading space — valid YAML,
what PyYAML emits, common in imported vaults) projects as an EMPTY list, because all three multi-line
branches in parseFrontmatter (store.ts:1900/:1981/:2009) require /^\s+-\s/. The panel shows it empty,
and the next property write replaces the whole block, deleting every item — silently, re-parsing
cleanly afterwards. Same for `aliases`, which breaks every backlink through that alias.
Verify it first with parseFrontmatter, then fix it Reproduce-First, then do PJ-181.

Do NOT cite two claims from the Slice-5 commit (3519e19d): "no whole-array property write remains"
(addTagToNote at +layout.svelte:6536 still does one) and "there is no callback left to omit" (the
onStageChanged push channel is still wired at +layout:8533/:8742). Both are false; MIG-107 §5.7 is
the correction of record, and both are folded into PJ-180.

Keep the PROPS_SINGLE_OWNERSHIP toggle — Boss-ruled, removed only when MIG-104 closes.

When MIG-104 resumes it is at Slice 8 + 8b: the archive hook MUST go BEFORE the
`DELETE FROM note_meta` at search.rs:9845, because FK enforcement fires the CASCADE there — a hook at
the later explicit purge archives NOTHING (tests_stage0_delete_order_defect). 8b adds the note body.

Run the Sight perf tests as a SERIAL lane (--no-file-parallelism --maxWorkers=1, PJ-172), rebuild the
binary before any Boss test and verify sources → build/index.html → constellation.exe by mtime, and
match BOTH "LNK1104" and "build failed" when retrying a transient cargo failure.
```
