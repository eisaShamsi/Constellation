# Handover — 2026-07-27 session close

**Read FIRST, in this order:** `docs/Constellation Orientation & Onboarding v3.71.md` → `docs/Constellation Pending Jobs v1.53.md` (► Next action) → this file → `docs/migrations/MIG-104-Plan-earned-life-ledger.md` (the approved Plan you are building) → **`docs/LESSONS-LEARNED.md` LL-035 and LL-036 — both written today, both from defects the Boss found that a green test suite did not.**

**Branch:** `main` @ **`16123279`**, pushed. Working tree **clean**.
**Gates:** Rust **1234/0** (11 ignored) · svelte-check **0 errors** · vitest 53 files.
**Binary:** `src-tauri/target/release/constellation.exe` @ **18:11** — Boss-validated.

---

## What shipped today (11 commits, `adc7da42` → `16123279`)

1. **MIG-105 Architect LOCKED** (`adc7da42`) — re-founded on the Boss's **Brain / Core Organizer** concept after two adversarial cycles (69 agents: Architect fan-out → Art Director & Team → Inspectors). All eight design rulings recorded in the doc header.
2. **MIG-105 Stage 0** (`042802c5`) — nine live defects fixed (PJ-149…157, 161) **and the root cause of a 3-week silent failure found: foreign keys ARE enforced** (rusqlite default; the child tables' `ON UPDATE NO ACTION` had been refusing every rename/move of a note owning a summary or history — 1,591 logged).
3. **MIG-104 Plan** (`95390d3a`) — approved by the Boss, all 8 checklist decisions ruled. Its authoring immediately caught a defect in Stage 0 (an archive hook placed at the purge would archive **nothing**, because the CASCADE fires 30 lines earlier at the `note_meta` DELETE).
4. **MIG-104 Slices 0–6** (`d0fbce47` → `16123279`) — baseline+harness · watcher predicate · determinism · appender · write hooks · seed · **the restore, Boss-validated.**

---

## ★ Where MIG-104 stands — you are mid-build on an APPROVED Plan

**Plan approval = build approval.** Cascade the remaining slices; stop only at Boss-testable verification clauses and genuine architectural surprise.

| Slice | State |
|---|---|
| 0 baseline + harness · 1 watcher predicate · 2 determinism · 3 appender · 4 write hooks · 5 seed · 6 restore | ✅ **SHIPPED, 6 Boss-validated** |
| **7 — snapshot + compactor** | **► NEXT.** `earned.snapshot.jsonl` + a 2 MB byte-threshold compaction that renames the tail aside and never deletes. Bounds the fold permanently. |
| **8 + 8b — archive-before-purge, + the note BODY** | **The Boss's time machine.** ⚠ The hook must go **BEFORE** the `DELETE FROM note_meta` — the CASCADE fires there, **proven by `tests_stage0_delete_order_defect`**. A hook at the purge archives nothing. 8b = the body (Boss decision #6), so the machine survives an emptied Recycle Bin. |
| 9 mirror · 10 cascade pre-delete archive · 11 restore rejoin · **12 = PJ-164/C8** · 13 gated overlay · 14 adjacent defects · 15 docs ×15 | pending |

Then **MIG-105 Phase 2 Plan** (Core Organizer + loose content → "Eisa Test"). **MIG-106** (LINK authoring surface, PJ-169) opens now that Slice 6 has validated.

### Constraints that must not be violated (all bought with a live failure)

- **A name-keyed record restores ONLY when it resolves to exactly ONE row.** Never distribute one folded count across links that may have earned different amounts. (The Boss's `banana` pair forced this.)
- **`weight` and the auto-confidence tier are DERIVED, never restored.** The ledger stores only what cannot be derived.
- **Never hold `state.db.lock()` across file I/O** (PJ-066 freeze shape).
- **fsync is per-site:** mandatory for archive-before-purge and user decisions (3,418 µs); a plain append for counters and the mirror (168 µs).
- **The restore is a RECONCILER — it must never carry a stamp.**
- **Any dedicated connection that WRITES a trigger-bearing table must register the FTS tokenizer** (LL-036).

---

## State of standing (SO#5)

### (a) Verified-shipped & protected
MIG-105 Stage 0's nine fixes · MIG-104 Slices 0–6 · **the restore proven on live data**: earned layer wiped (38 → 0 rows), ledger left as the only copy, reboot → **34/34 written, 34 exact matches, 0 mismatches**, an impossible weight healed, **a retired link returned still retired.**

### (b) At-risk / in-flight
**None uncommitted** — tree clean. The Boss's Universe is in its normal state; `EARNED-SNAPSHOT.json` (the test's safety net) lives in the session scratchpad and is no longer needed.

### (c) Known-broken / open
PJ-152 (`custom_stages` still destroyed by rename/attach/detach) · PJ-158 (RTL chevrons, "vault" in 10 locales, cursive captions, English-only const) · PJ-159 (939 MB orphan DB) · PJ-162 (`.base` YAML parsed as JSON) · PJ-163 (review-pulse RMW wipe) · PJ-168 (236 orphan link rows) · PJ-170 (lowercased target names in panels) · PJ-140 (~37).

### (d) Pending, not started
MIG-104 Slices 7–15 · MIG-105 Phase 2 · MIG-106/PJ-169 · PJ-171 (CI runs zero tests) · PJ-167.

### (e) Doc drift / process
- **PJ-172 — PJ-132 is now a GATE, not a nuisance.** The Sight perf tests assert wall-clock budgets inside a *parallel* runner, so they measure the machine. Since the suite became glob-driven it gates every slice, and a load-sensitive test makes that gate **lie in both directions**. Run them as a serial lane until fixed.
- **PJ-166 / PJ-124** — the safety-inspection workflow ignored `args.files` a **third** time, and **23 verifier agents died on a session limit**; those candidates are *unverified*, not cleared.
- PJ-146 — translated help dirs are still a partial subset.

---

## Watch-items for the next session

- **`cargo test`/`build` intermittently hits `LNK1104`** — a transient Windows lock on a freshly-linked test exe. Not a code fault; retry (a 3–4 attempt loop works). `--lib` avoids the bin targets entirely.
- **Never quote a LOAD figure for a REBUILD.** The Slice-0 baseline (~35 s hydrate) is loading an existing index; rebuilding 7,817 notes takes many minutes. Say so before sending the Boss into one.
- **Verify a UI affordance exists before writing it into a test tutorial.** "Retire" does not exist; the label is **Archive link**, on the Backlinks/Outgoing panels — not the editor's right-click menu.
- **Beware `LIKE '%Africa.md'`** — it matches *East Africa.md* and *West Africa.md* too. Check rows by id.
- **Mandatory Boss test before commit** stands (memory `feedback_boss_test_every_build_mandatory`).

---

## ► READY-TO-PASTE NEXT-SESSION PROMPT

```
Continue Constellation on branch main (@16123279, clean). First: git pull; read
docs/Constellation Orientation & Onboarding v3.71.md, then docs/Constellation Pending Jobs v1.53.md
(► Next action), then lab/reports/HANDOVER-2026-07-27-session-close.md, then
docs/migrations/MIG-104-Plan-earned-life-ledger.md (the APPROVED Plan you are mid-build on).
Note LL-035 and LL-036 in docs/LESSONS-LEARNED.md — both written yesterday from defects the Boss
found that a green 52-test suite did not.

Then continue the MIG-104 cascade at Slice 7 (earned.snapshot.jsonl + the 2 MB compactor). Plan
approval = build approval: cascade the slices, stopping only at Boss-testable clauses.

Slice 8 is the Boss's time machine and has one hard constraint proven by test: the archive hook must
go BEFORE the `DELETE FROM note_meta`, because FK enforcement makes the CASCADE fire there — a hook
placed at the later purge archives NOTHING (tests_stage0_delete_order_defect). 8b adds the note body
per Boss decision #6.

Keep the standing constraints: a name-keyed record restores only when it resolves to exactly ONE row
(never distribute a folded count); weight and the auto-tier are DERIVED, never restored; never hold
the DB lock across file I/O; fsync only where the other copy is about to be destroyed; the restore is
a reconciler and must never carry a stamp; any dedicated connection that WRITES a trigger-bearing
table must register the FTS tokenizer.

Run the Sight perf tests as a SERIAL lane (PJ-172) — they measure the machine, not the code, and a
green suite now gates every slice.
```
