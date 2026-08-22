# HANDOVER — MIG-111 Stage B (start at B4)

**Written 2026-08-22 at session close. Head: `a088226b`, pushed, working tree clean.**
**Gates at handover: Rust 1532 / 0 / 19 ignored · vitest 997 / 997 · svelte-check 0 errors · 15/15 locales in parity.**

---

## 1. Read these first, in this order

1. `docs/Constellation Orientation & Onboarding v4.3.md` — **the highest version. Read only this one.**
2. `docs/Constellation Pending Jobs v1.92.md` — the ledger. Its **► Next action** line names Stage B.
3. `docs/migrations/PJ-235-federation-boundary/MIG-111-PLAN-1.2.md` — the plan, **including the
   "A5–A7 — the measured surface" section**, which replaced the Architect's estimate with counts.
4. `docs/LESSONS-LEARNED.md` — **LL-047, LL-048, LL-049, LL-050**.
5. `lab/reports/SESSION-LOG-2026-08-20-mig111-stage-A.md` — the whole arc, including every error.

---

## 2. Where MIG-111 stands

**Phase 1.2 Stage A is COMPLETE and its acceptance test passes with `#[ignore]` removed** — which was
the definition of done set before the work began.

An operation on a note in a **Linked Universe** now does its bookkeeping using *that universe's* link
vocabulary, read from that universe's own disk. Not by swapping the process-global — that is read at
CALL time, and a debounced save, a backfill tick or the watcher landing inside the swap window
classifies a note under another universe's vocabulary **with every row count still correct** (LL-047).
The vocabulary is carried as a value.

**The acceptance test is mutation-proved.** Point the routed scope at the active global instead of the
owner's disk and it fails, showing the corruption itself:

| | `link_rows` | edge | incoming |
|---|---|---|---|
| correct | **1** | `("source.md", "target", "refutes")` | `1` · `"refutes (1)"` |
| corrupted | **1** | `("source.md", "refutes::target", "associative")` | `0` · `""` |

The type collapses to `associative`, `refutes::` is absorbed **into the target's name**, the backlink
vanishes — and the row count is 1 in both. **A check that counted rows would report perfect health.**

---

## 3. Stage B — the work, measured not estimated

### B4 — START HERE. Read-side analytics.

**Seven sites, all the identical shape** — `active_universe_vocabulary().structural_not_in_clause("link_type")`:

| file | line |
|---|---|
| `cache.rs` | 516, 548, 1288 |
| `sight.rs` | 77 |
| `tension.rs` | 277 |
| `search.rs` | 189, 267 |

The plan's B4 clause: each takes `schema: &str` over ONE connection, so this is a **threaded registry
per schema**, not a bundle — a federated read over two universes must report each universe's own
types. **Also correct `tension.rs:88-92`**, which falsely claims `validate_path_in_any_library`
refuses Linked-Universe paths; `libraries.rs:727-728`'s own doc says the opposite.

**The census will go red when you touch these.** That is the point. `link_types.rs`'s
`the_ambient_vocabulary_reads_are_census_ed` pins every remaining ambient read by file and count.
**Answer its question at each new site — *whose vocabulary is this, and why is the active one right
here?* — before updating the map.** Updating the map without answering is the failure it exists to
prevent, and it has already caught this session's own additions twice.

### B3 — ALREADY DONE

The two filesystem re-walkers were threaded during A5; deleting the ambient readers forced them.
**A real defect fell out**: `strata.rs` and `inspector360.rs` re-read the global **once per
directory**, so a walk spanning a vocabulary change could classify half a library one way and half
another. They now take one value from the top.

### B5 / B6 — the rename path. **TWO COMMITS. NEVER ONE.**

> **B5:** `rewrite_wikilinks_in_text` (`libraries.rs:7478`) and `update_links_recursive` take the
> **OWNER's** registry. **The fences stay up in this commit.**
> **B6:** *then* the fences come down — the SEEK-branch refusal at `libraries.rs:6969` and the
> `&foreign` exclusion at `:6992`, for the owner's own universe only. A rename still refuses to
> cross into a *third* universe (Phase 3 / R23).

**Why the order is absolute.** Drop a fence before the vocabulary reaches the rewriter and a rename
**silently corrupts a Linked Universe's files on disk**: `[[refutes::Old]]` in a child, where
`refutes` is a child-only type — the parent does not know `refutes`, so the whole `refutes::Old`
reads as the target name, the rewrite does not fire, and the link is left broken. **No error, no
wrong count.**

B6 also needs the **Editor-Surface Gate checklist run on a FEDERATED note**, including the
linked-probe-pair shape (item 6).

### B1 / B2 / B7 — the remainder

- **B1** — the backfills' `recompute_*` FUNCTIONS, not merely the generators.
- **B2** — DDL generation takes the vocabulary explicitly.
- **B7** — the watcher fence is a single identifier at `search.rs`'s `reindex_changed_paths`
  (`try_load_libraries`, the OWN set). Swapping it for `load_all_libraries` would enable routed child
  writes **with zero compile errors**. It stays as-is in 1.2; **pin it with a test so the swap cannot
  happen silently.**

---

## 4. Standing orders that bit hardest this session — read these twice

- **SO#10 (new, 2026-08-21):** PCS and orientation **before** any ruling request. A ruling asked
  against a stale record *launders the staleness into a recommendation* — a nine-agent panel read
  three stale canonical documents and formally recommended a retired name back to the Boss.
- **The test pipeline is not optional and it is not ceremony.** `tutorial-auditor` → `ui-inspector` →
  panel → Boss. It **held** two tutorials this session and each time found something no suite could:
  a receipt read that fired 126 lines before the database opened, and the fact that the Boss's
  **installed** binary is from June and lacks the command entirely.
- **The per-build diff-scoped `safety-inspection` before every commit.** It found four defects on one
  diff, **three of them mine, written that same day.**
- **Reproduce-First.** Every fix that shipped this session was red on demand first. Where a fix made
  the original reproduction *inexpressible* (PJ-332), that is recorded as the outcome to aim for.
- **Measure live data before shipping a guard.** Two of this session's own fixes were regressions
  caught only by querying the Boss's real databases — one would have re-read 8,031 notes **on every
  boot, forever**.

---

## 5. Traps this session paid for — do not re-pay them

1. **`cargo clean -p constellation` removes ~26 GB**, not the crate. ~18-minute cold rebuild.
2. **After ANY `LNK1104`, delete the exe and force a relink.** Cargo's fingerprint will serve a
   **corrupt** binary as up-to-date, and every test result after that is a claim about code you are
   not running (LL-050).
3. **Never grep the exe for FRONTEND strings** — Tauri compresses the embedded frontend, so *no* UI
   string appears, including months-old ones. Grep `build/` and check the binary is newer.
   **Rust strings DO appear** in the exe — that check is valid.
4. **Bash heredocs choke on apostrophes/non-ASCII here.** Write the script with the `Write` tool and
   run it, rather than fighting `<<'PY'`.
5. **`stratum` is stored as TEXT.** That is why every reader `CAST`s it, and why a missing value
   becomes rank 0 via `.unwrap_or(0)` rather than erroring.

---

## 6. Open, with what each needs

| item | state |
|---|---|
| **PJ-326..331** | the job the Boss scheduled for after Stage A — no unlink control for a Linked Universe; a warning badge that can silently never appear; the reason string as an IPC-contract change; a dead universe's libraries leaking into the active list *(severity not established)*; a legacy-layout universe that may never warn *(not adjudicated)*; the **Linked Universe** rename (10 translated values + 2 labels never passed through `$t()`). |
| **PJ-334's ORIGIN** | established for the defect's **permanence**, **not** for the first event. Do not close it as understood. |
| **PJ-336** | 22 of 43 help topics missing from all 14 translated sets. |
| **PJ-321** | the registry contradiction. **Carries an explicit STOP.** It has produced two confident wrong explanations. Reproduce under instrumentation or leave it alone. |
| **PJ-332b residual** | a panic in the sky back-fill thread leaks its run-slot for the process lifetime. Matches every sibling; revisit for all of them together **or not at all**. |

---

## 7. The through-line worth carrying

Four separate defects this session were **one shape**: *an operation that moves or derives from the
user's files treating "I could not read that" as "there is nothing there."* PJ-322, PJ-332, PJ-334,
PJ-335. When you meet a reader that returns an empty list, ask what it returns when it **fails** —
and whether the caller can tell the difference.
