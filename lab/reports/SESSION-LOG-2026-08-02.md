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

---

# 2026-08-03 — the register campaign, and four half-sweeps of my own

Boss ruling: **"All 31."** The consolidated triage's whole register, not a batch. This log covers
the first tranche — the four app-killers — and the audit that found my fixes wanting.

---

## §8 — Concern #1: "couldn't read it" ≠ "you have none"

**Ranked APP-KILLER, 13 register entries across four files.**

A loader that maps *I could not read this* onto *you have none*, after which an ordinary save
writes that emptiness over the user's real file. One second of a sync tool or antivirus holding
a file is an everyday event on Windows.

**The shared primitive** — `universe::read_persisted_json`, deliberately placed beside
`atomic_write`, whose own comment has described this defect *in writing* since the G6 audit:
*"every loader here swallows the parse error and falls back to empty … and the next save writes
that emptiness back."* That audit hardened the WRITE and left the READ exactly as described —
which made the destructive overwrite atomic.

The distinction it draws, and the whole point:

| State | Verdict | Why |
|---|---|---|
| Absent | `Ok(None)` | genuinely "none yet" — safe to write over |
| Present, readable | `Ok(Some(v))` | — |
| Unreadable / corrupt / truncated | `Err` | **the data is on disk and we failed to see it** |

**Applied to:** `universe.rs` (6 registry write-back sites via `load_registry_for_update`),
`review.rs` (3 sites via `load_pulse_data_for_update`), `link_types.rs`, `style_presets.rs`.
`libraries.rs` was found **already correct** — all four mutating sites use the strict
`try_load_libraries`, every lenient caller is read-only. I had drafted a comment calling it a
live defect and corrected it to say what is actually true.

**The lenient twin is kept everywhere,** documented as read-only. A panel showing nothing for
one boot is a display problem; a save writing nothing is data loss. The two callers genuinely
differ, so the split is the fix — not blanket strictness.

## §9 — Concern #4: the fallback editor that never saved

**Ranked APP-KILLER.** If `new EditorView(...)` threw, NotePane built a simpler fallback from
its own extension list — which omitted `EditorView.updateListener`. That listener appeared
**exactly once in the file**, inside the primary state. A note opened through that path looked
and behaved completely normally and **nothing typed into it ever reached the save path**: no
dirty flag, no debounced save, no idle save, no push to the note model.

Fixed by extracting `changeListener()` and using it in both states. Sharing it — rather than
copying it into the fallback — is the point: a copy is what drifts.

## §10 — Concern #2: "Overwrite" destroying the note you KEPT

**Ranked APP-KILLER. Boss-validated live.**

`deleteWithSetting` has always closed tabs and disposed models. `moveToTrash` never did — and
`moveToTrash` is what all three displacement paths call: Overwrite-on-create,
Overwrite-on-rename, and the PJ-088 conflict sidecar. So Overwrite trashed the existing note's
file and left its tab open, model live, still owning that path. The kept note was then renamed
onto it, and the stale model wrote over the survivor at its next flush.

PJ-187 had already unified those paths on *where the displaced file goes* and left *what happens
to its open tab* in one of them. Half a sweep inside the fix meant to make them agree.

Cured by the shared `releaseTabsForVacatedPath`. **Boss test passed**: after a full relaunch the
survivor held the correct body with the old title preserved as an alias.

## §11 — Concern #3: REFUTED, and the refutation is the useful part

The register claimed a restart discards the app's own rescue copy. I began fixing it, then found
the PJ-102 Recipe S test header stating that exact path had been **checked and refuted**. I
verified the reasoning against source and it holds: a path enters `pendingCidEnsure` only when
its content has **no** `cid_cn`, while net-recovered content is `identityProven` by construction
and therefore always carries one. A tab cannot be in that queue AND hold recovered work.

I kept a genuine hardening from the attempt — `drainCidEnsure` now adopts through
`externalChange` rather than hand-rolling `openNoteModel`, inheriting the identity, echo and
baseline guards across its four awaits — and **rewrote my comment**, which had asserted a
mechanism that cannot occur. A confident wrong comment is how the next reader is misled.

---

## §12 — The audit on my own work, and what it found

37 agents, every finding refuted twice before acceptance. **Three of my four fixes were wrong or
incomplete.**

1. **The link-types fix landed on a dead command.** `read_universe_link_types` is registered in
   `lib.rs` with **zero callers**; the editor reads through `list_link_types` → `load_active` →
   the lenient `read_deltas`. My fix protected nothing. Moved to the command actually in use.
2. **The style-presets fix was cancelled at the UI layer.** `loadStylePresets` did
   `catch { return [] }`, so a file the backend had just refused to treat as empty arrived at
   the screen as empty anyway. Added the read-succeeded latch + a surfaced error.
3. **My registry fix locked the user out of the app.** With a corrupt `universes.json`, all four
   routes in — Create, Open Existing, Link Library, Import — returned the same error naming a
   hidden file. Refusing to overwrite had become refusing to continue. Now `PersistedError`
   distinguishes **Unreadable** (transient — refuse, a retry works) from **Corrupt** (set the
   file aside, preserving it, and proceed from a *true* empty).
4. **My tab teardown destroyed unsaved work.** It erased the write-ahead recovery buffer and
   disposed the model **without saving it**, so a note whose last save had failed lost its only
   copy — while the "your edit is safe" banner stayed on screen with dead buttons. Now
   `preserveWorkBeforeVacating` flushes first; if the flush cannot be proven, the net is KEPT.

Plus four siblings I had missed: `closeTab` never repaired `focusedTabId`; nothing turned split
view off; the save-health banner named deleted notes forever; and **`cece/reliability.rs`** — a
fifth file with the identical load→mutate→save shape, where a corrupt profile replaced every
cataloger correction the user has ever made with a single datapoint.

---

## §13 — The Close arc: four rounds, three of them my own misses

Boss Test C could not be performed: **there was no way to close a note in split view at all.**
The tab bar — the only close affordance — is hidden entirely while split is on
(`{#if !$splitActive}`), and neither the ⋯ menu nor the file-tree right-click offered Close.

| Round | What the Boss found | Fix |
|---|---|---|
| 1 | No Close anywhere in split view | **Close** in the ⋯ menu, handled in `NoteEditor`'s *always* group (per the 2026-07-18 precedent, so it works on the second screen too) |
| 2 | Pane went empty after Overwrite | `releaseTabsForVacatedPath` now repairs `activeTabId`/`focusedTabId` — a **pre-existing gap in Delete**, surfaced by sharing the teardown |
| 3 | **No Close in the file tree** — I wired 1 of 6 note-action builders, and not the tree | One `wireCloseNote` helper called from every builder (the `wireCollectionPickup` precedent) |
| 4 | **⋯ → Close dead in the Index preview** — I hid the × there and left the item live | Derived `isClosableTab` from `openTabs` membership; gates BOTH controls |

**Boss rulings absorbed:** the × is present in every view; split view collapses to normal when
**fewer than two** notes remain (not zero — a lone pane in a split layout has no tab bar, so the
survivor was stranded); and after an SME discussion, **the page × shows only when the note has no
visible tab** — closing is an act on the *open set*, so it belongs to the tab strip, and the
header × is a bridge over views that lack one.

**The lesson, stated plainly:** rounds 3 and 4 were the Whole-Ecosystem Fix Law failing *inside*
the work meant to honour it. Both were cured the same way — replace the promise a caller must
remember with a structure that cannot forget: a shared helper, then a derived value.

---

## §14 — Verification

| Gate | Result |
|---|---|
| vitest (main lane) | **812 / 812** (71 files) ✅ |
| vitest (Sight lane, serial per PJ-172) | **84 / 84** ✅ |
| svelte-check | **0 errors** ✅ |
| Rust | **1339 / 0** ✅ |
| i18n parity | **15 / 15** ✅ |

**Boss-validated live:** Overwrite (data correct across a relaunch) · Delete-active-note ·
Close from ⋯, file tree, split view · the collapse rule · the conditional × · the Index preview.

New tests: 6 (`read_persisted_json` classification, Rust) + 21 (`tests/pj-200`), including
in-suite RED proofs for the vacate teardown, the activation repair, and the collapse threshold.

## §15 — Standing Orders

- **SO#1** — this log.
- **SO#6** — Orientation **v3.82**.
- **SO#7** — MoCh for the day.
- **SO#9** — PJ ledger **v1.66**; **MIG-110** allocated with concept paper
  `docs/concept-papers/34-tabs-in-every-view.md`; PJ-208/209/210 filed.
- **SO#2** — help/User Manual: **owed**. Close is a new user-facing command in four surfaces;
  filed as PJ-211 rather than claimed done.
