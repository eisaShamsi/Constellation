# Session Log — 2026-08-02

Continues 2026-08-01 (Stage-B ran and passed; the Search Hub's concept was stated; MIG-104
Slice 8 + 8b shipped — see that log's §8–§10, appended retroactively).

Today: the Boss reported a ~4-minute boot and a deleted note that stayed in the file tree.
Both were fixed. The boot investigation then found a **third** problem neither fix was aimed
at — a cost the Boss had been paying for a month.

---

## §1 — Boot: the probe had an index for the exact opposite predicate

**Commit `1b29e2b4`.**

**Boss report:** *"When I open the app, it takes about 4 minutes to populate the notes. I
thought we fixed it before!"* Measured on his universe: **122.7 s**, of which **109.7 s** was
one query.

**The defect.** The boot probe counts how many notes still lack a canonical id:

```sql
SELECT COUNT(*) FROM note_meta WHERE cid_cn = ''
```

`note_meta` carried exactly one partial index on that column — `WHERE cid_cn != ''`. That is
the **precise complement** of the predicate being run. SQLite cannot serve a query from the
negation of a partial index's own `WHERE` clause, so the probe full-scanned a 7,827-row table
and dragged `body_text` through memory on every boot.

**The fix.** One mirror index, added in `ensure_note_meta_mig003_unique_index`:

```sql
CREATE INDEX IF NOT EXISTS idx_note_meta_cid_empty
  ON note_meta(cid_cn) WHERE cid_cn = '';
```

**Measured:** 109.7 s → effectively zero. Boot 122.7 s → **~16 s**, which exposed §2.

**The transferable lesson**, now in Orientation §9.2: *a column being "indexed" says nothing
about whether your query can use it.* Read `EXPLAIN QUERY PLAN` **for the predicate as
written**. A partial index is a promise about one specific subset, and its complement is not
covered — it is the one case guaranteed *not* to be.

**Method note.** The Boss's instruction here changed the outcome: *"Instead of exploring for a
solution, try to see how we overcame the same issue before. We reach almost instant boot
time."* That redirected the work from open-ended profiling to reading MIG-079's boot-perf
record — where the covering-index primitive (§9.2, LL-020 corollary) is written down. The
answer was in our own history, not in a new investigation.

---

## §2 — The standing defrag rule: the database compacts itself

**Commits `99769c2e` (the rule), `c3b7bdc1` (visibility + interruption cost).**

With the index fixed, boot was still ~16 s. MIG-108 had rewritten every path in 13 tables and
left the file fragmented: **106,155 free pages of 494,728 — 21.5%** of a 1.9 GB database.

The Boss's instruction was not "vacuum it." It was **"Take the standing rule."** So the fix is
a rule, not an action:

> **After any mass rewrite, Constellation defragments its own database.**

- **`maybe_schedule_defrag(app)`**, gated by a pure, unit-tested predicate
  `defrag_wanted(free, total, min_pages, min_ratio)` — fires only when the freelist exceeds
  **both** `MIN_FREE_PAGES = 25_000` **and** `MIN_FREE_RATIO = 0.10`. Requiring both stops a
  small-but-proportionally-messy file and a huge-but-tidy one from each triggering a needless
  multi-minute `VACUUM`.
- Runs in the background, off the boot path.

**Result:** boot **122.7 s → 4.7 s**. Freelist after: **0**.

### §2.1 The follow-up the Boss forced — "I didn't see the [defrag]"

The first hand-run vacuum froze the app with no indication of why. The Boss closed it. That was
the **correct** response: an invisible multi-minute freeze is indistinguishable from a hang.
`VACUUM` is atomic, so nothing was damaged — but a routine that gets closed every time it runs
is a routine that never completes.

Two changes, both in `c3b7bdc1`:

1. **It says so.** `vacuum_start` / `vacuum_done` are emitted on the existing
   `migration:term_vocab_v2` channel, which drives **MIG-041's `MigrationProgressStrip`**.
   *That surface already existed and I had simply never wired it.* Before building a progress
   indicator, check whether the app already has one.
2. **An interruption must not cost a day.** A genuine failure earns a 24 h cooldown
   (`FAILED_COOLDOWN_SECS = 86_400`); being interrupted earns **10 minutes**
   (`INTERRUPTED_COOLDOWN_SECS = 600`). Treating "the user closed their laptop" the same as
   "this operation failed" is how a self-healing routine quietly stops healing.

---

## §3 — The finding neither fix was aimed at: the boot was slow for a month

**No commit — analysis.** Tool installed at `lab/boot-perf/cold_vs_warm.py`.

The Boss's phrasing — *"I thought we fixed it before!"* — was the useful part. It implied a
number he had lived with. Bucketing all **896** recorded boots in `boot-perf.history.jsonl` by
**gap since the previous boot** (a proxy for OS page-cache eviction) separated warm from cold:

| Bucket | Before the vacuum | After |
|---|---|---|
| **COLD** (> 30 min idle) | n=304, **median 26.7 s** | n=1, **1.2 s** |
| cool (5–30 min) | n=117, 3.7 s | — |
| warm (< 5 min) | n=31, 0.4 s | — |

So the "27–34 s boot that predates today" was **the same disease** — the cost of reading a
fragmented file cold — and the standing rule cured it as a side effect.

**Two lessons.** *(1)* A boot-perf number without a cold/warm label is not a measurement: the
same build honestly reports 0.4 s and 26.7 s. *(2)* A cost can hide for a month **behind an
average**, because every boot a developer runs while iterating is warm. Only the user pays the
cold price, and only once a day — which is exactly often enough to be infuriating and exactly
rare enough to never appear in a dev loop.

**Honest caveat, recorded in Orientation §17:** the post-vacuum cold sample is **n=1**. The
direction is unambiguous; the confirmation will come from ordinary use.

---

## §4 — The deleted note that stayed in the tree: one resolver for six sites

**Commit `6f22ff47`.**

**Boss report (with screenshot):** *"I deleted the note, but it is still listed in the file
tree. When I try to select it, nothing happens."*

**What was actually true.** The note was gone — from disk, from `note_meta`, from every index
table. The delete had fully succeeded. **Only the tree was never told.** Nothing surfaced an
error because nothing had failed. That is the silent class: a correct operation with an
un-notified observer.

**The mechanism.** The delete handler resolved the note's owning library by **first match**:

```ts
const lib = $libraryStats.find(v => confirmDelete!.path.startsWith(v.path));
await deleteWithSetting(confirmDelete.path);
if (lib) await refreshLibraryTree(lib.library_id);   // ← and nothing if it missed
```

**MIG-108 is what made this reachable.** Now that every library lives *inside* the universe
root, the root library's own path is a **prefix of all of them** — so `find()` returns the root
for a note that belongs to a nested library, and the wrong tree is refreshed. Before MIG-108,
libraries could live anywhere and the prefixes rarely overlapped.

**The Whole-Ecosystem Fix Law applied.** Grepping the concern — "which library owns this path"
— found **six call sites answering it three different ways**: first-match (4 sites),
longest-match-but-unbounded (1), and one bespoke variant. All six now share one resolver:

```ts
export function owningLibrary<T extends { path: string }>(libs: readonly T[], path: string): T | null
```

Longest-root-wins · **separator-bounded** (so the library `…/Research` never claims a note under
`…/Research Notes`) · case- and separator-insensitive (Windows). The delete path now refreshes
**unconditionally** — `else await refreshAllLoadedTrees()` — because "I could not identify the
library" is never a reason to leave a deleted note on screen.

**Tests:** 14 in `tests/pj-196/owningLibrary.test.ts`, including an **in-suite RED proof** that
runs both legacy implementations against the same cases. If someone later "simplifies"
`owningLibrary` back to either shape, the suite fails and says why.

---

## §5 — Verification

| Gate | Result |
|---|---|
| vitest | **875 / 875** (74 files) ✅ |
| svelte-check | **0** ✅ |
| Rust suite | green ✅ |
| i18n parity | **15 / 15** ✅ |

**Correction worth recording:** I nearly wrote "791 tests / 69 files" into the Orientation doc
from memory of an earlier point in the session. Running the gate showed **875 / 74**. Gate
numbers get stale within a single day's work — re-run, never recall.

---

## §6 — Standing Order compliance

- **SO#1** — this log. **It was missing for seven commits across two days**; §8–§10 were
  appended retroactively to the 08-01 log in the same pass. Recorded as a real lapse, not
  backfilled silently.
- **SO#6** — Orientation **v3.81** written (new file; v3.80 untouched). Body updated, not just
  the preamble: new **§9.4** (the standing defrag rule) and **§9.5** (cold-vs-warm measurement),
  two new §9.2 primitives, five migration rows added to §8 (MIG-104/105/107/108/109 — the table
  had stopped at MIG-093), the tree-refresh bug closed in §13, two drift entries corrected in
  §12, and two honest unknowns added to §17.
- **SO#9** — PJ ledger reconciliation → **v1.65** (below).
- **SO#2** — help files / User Manual: **no user-facing change today.** The boot fix, the defrag
  rule and the tree refresh are all invisible-when-working; the defrag progress strip reuses
  MIG-041's existing, already-documented surface. No manual edit required.

---

## §7 — Documentation drift found while writing v3.81

Two entries added to Orientation §12, both found by checking rather than assuming:

1. **"No frontend test harness"** — stale. Vitest is now the frontend gate (875 tests, 74
   files) plus `npm run i18n:parity`. Marked RESOLVED.
2. **`CLAUDE.md`'s Living-Link Storage note** — **partially drifted, and I did not amend it.**
   It states *"TODAY ONLY THE SECOND LAYER EXISTS"* (nothing earned persisted on disk) and
   instructs that it be amended *"when it ships, and only then."* **MIG-104 is that migration,
   and Slices 0–8b have shipped** — earned state now persists to `.constellation` and is
   archived before a delete. But Slices 9–15 are open, so the gap is **narrowed, not closed**:
   the sentence is no longer literally true and not yet safely rewritable. Flagged for a Boss
   ruling rather than edited unilaterally — CLAUDE.md is the project's instruction file.
