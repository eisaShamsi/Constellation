# MIG-098 — Rename → Index Sync Reliability — Architect

**Date:** 2026-07-07 · **Status:** BUILD (Step 1 instrumentation shipped; root fix pending the trace). Boss said "Proceed" to the instrument-first approach after MIG-096 §2 surfaced the drift.

## The horse (concept)

> **A rename writes the file reliably (gated) — the search-index must follow it just as reliably.** After a rename settles, `note_meta` (name + path) must match disk, so every reader (Reviewer, Search, Index, Sky) shows the note's *current* name at its *current* path — never a dead path or a stale name, and never silently dropped from the index.

## The defect (reproduced against the Boss's live data, 2026-07-07)

- **Symptom:** renaming a note leaves `note_meta` pointing at the OLD (now-dead) path with the OLD name; disk is correct. Readers show stale name → opening hits a dead path → empty Dashboard.
- **Frequency:** NOT rare. `diagnostics.log` shows the boot reconcile removing 1–2 stale rows on ~every launch for ~9 days — i.e. renames drift routinely on the 2 GB / 7,713-note universe, and the MIG-078 reconcile has been silently *deleting* the drifted notes from the index each boot. Lines up with the **2026-07-03 §B2-4 change** that detached the rename's DB tail (`rename_item_db_tail`) to a best-effort `spawn_blocking` to stop a freeze — relying on "the watcher / next reindex heals any miss," but gated renames suppress the watcher.
- **Confirmed cases:** `التجربة الثانية ن2` (cid 8878) — dead row already *removed* by an earlier boot's reconcile → now an orphan file with no index row (missing from Reviewer). `§D test 1 v2` (cid 6C47) — renamed mid-session → `note_meta` still at dead `§D test 1.md`.
- **Why MIG-097 (relocate-on-boot) is insufficient:** (a) runs only at boot → mid-session renames stay stale; (b) can't recover notes whose dead row a prior boot already deleted (needs orphan *re-adopt*, not just relocate).

## Step 1 — Instrumentation (shipped; Reproduce-First)

The symptom is reproduced; the **mechanism** (does the detached tail run? does the `UPDATE note_meta … WHERE path=old` match 0 rows? does it error? does reindex skip on a library miss?) is not yet pinned. Added `diag_log` traces (release-safe, → `diagnostics.log`):
- `rename_item`: `[rename-tail] scheduling tail old=… new=…` at the spawn point.
- `rename_item_db_tail`: `START`; the `note_meta` path `UPDATE affected N row(s)` (or ERROR); canonical (old==new) branch; reindex `OK`/`ERROR`/`NO LIBRARY matched … SKIPPED`; `END`.

**Decision tree from the trace:**
- `scheduling` logs but no `START` → the `spawn_blocking` task was starved/dropped (blocking-pool saturation on boot backfills) → fix = don't rely on the detached task for the critical cheap update.
- `START` + `UPDATE affected 0 row(s)` → path mismatch (separator/normalization) between the passed `old_path` and stored `note_meta.path` → fix = normalize / match on cid.
- `START` + `affected 1` but note_meta still stale → a later write reverts it (concurrent reindex / watcher) → fix = ordering.
- `NO LIBRARY matched` → `new_path` prefix mismatch → fix = matching.

## Step 2 — Root fix (pending the trace)

Chosen once the mechanism is known. Candidate shapes (do NOT reintroduce the §B2-4 freeze — the awaited IPC must never hold an unbounded writer-lock wait):
- Make the **cheap** `note_meta` path+name update reliable (it's a 1–2 row write; the freeze came from the heavy reindex, not this) — e.g. a bounded/try acquisition, or match-by-cid so it can't miss.
- OR a reliable post-cascade verify-and-reindex of the single renamed note (targeted, retryable) that can't be starved.

## Step 3 — Complete the self-heal (extends MIG-097)

- **Re-adopt orphans** — index `.md` files that have NO `note_meta` row (recovers notes a prior reconcile deleted, e.g. `التجربة الثانية ن2`).
- Keep the relocate-by-cid (preserves aux) + the WA#4 safety caps.
- Consider running the reconcile after a rename cascade (targeted, not the full stat-all-paths walk) to close the mid-session window if Step 2 can't fully close it.

## Invariants

1. Disk stays the source of truth; the index follows it (File-Over-App).
2. No freeze regression — the awaited rename IPC never holds an unbounded writer-lock wait (the §B2-4 rule stands).
3. No note silently dropped from the index on rename (the MIG-078 remove-on-drift must not fire for a note whose file merely moved).
4. Boot time + typing latency + IPC responsiveness unregressed on the 7,600-note universe (measure before/after).
5. Aux data (review history, links) preserved across a rename/relocate.

## Migration path

Instrument (done) → **Boss renames one note → read `diagnostics.log`** → design Step 2 from the trace → build Step 2 + Step 3 → audit (invariants + perf on the large universe) → remove the instrumentation → PCS.
