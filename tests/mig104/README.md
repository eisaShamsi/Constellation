# MIG-104 — the Earned-Life Ledger reproduction registry

Every slice of MIG-104 appends its **named failure recipe** here before its fix lands, and the
recipe must be RED first. This file is the index; `harness.ts` is the shared machinery.

Glob-driven vitest (PJ-157) collects everything under `tests/` automatically — nothing to register.

## The measured baseline (Slice 0, 2026-07-27 — the real Universe, 7,817 notes / 234,233 links / 2.03 GB search.db)

Recorded from the app's own scorecard (`.constellation/boot-perf.history.jsonl`, 814 full-universe
boots) and from a release-mode measurement on the Boss's E: drive. **No later slice may commit
without a before/after against these numbers** (Rule 8 hard constraint).

| Metric | Median | p90/p95 | Source |
|---|---|---|---|
| `paint_ms` | **672 ms** | 906 ms | last 20 boots |
| `libraries_loaded_ms` | **752 ms** | 1,012 ms | last 20 boots |
| `hydrated_ms` | **34,871 ms** | 37,462 ms | last 20 boots |
| `graph_ready_ms` | **35,546 ms** | 37,839 ms | last 10 recorded |
| append 200 B, **no fsync** | **168 µs** | 333 µs (p95) | `mig104_measure_append_fsync_cost` |
| append 200 B **+ fsync** | **3,418 µs** | 4,922 µs (p95) | same |

### ★ What the fsync number changes about the design

**fsync costs 20× a plain append (3.4 ms vs 168 µs).** That single measurement decides where
durability is bought and where it is wasted, and it must be stated per write site rather than
applied uniformly:

- **Archive-before-purge (Slice 8) — fsync is MANDATORY.** The purge destroys the only other copy
  microseconds later; an un-synced buffer is not an archive. 3.4 ms on a delete is invisible.
- **Decisions (archive / unarchive / confidence / priority) — fsync.** Rare, user-initiated,
  irreplaceable. The proven `review.rs` file-first order.
- **Walk counters (Slice 4) — plain append, NO fsync.** Boss decision #1 rejected coalescing, and
  the measurement shows why that is affordable: an un-synced append is 168 µs, and the OS flushes
  it. A walk count lost only to a power cut is the honest trade; paying 3.4 ms per link click to
  protect a number that feeds a logarithm is not.
- **The continuous note-history mirror (Slice 9) — plain append, no fsync, off the save path.**
  At 3.4 ms per save it would be the most expensive thing on a debounced save; at 168 µs it is
  noise. Measure again at the slice.

## Recipe registry

| Recipe | Slice | What it reproduces | State |
|---|---|---|---|
| W1–W4 | 1 | The `.constellation` watcher predicate: rejects the bare directory event, the tail, a vanished `.tmp`, the rename-aside, and the live D3 `cataloger_reliability.json` persist; ACCEPTS the user's knowledge incl. `.trash/*.md` and a vanished user folder; matches a whole component so `My .constellation notes` survives. Plus the three-key suppression contract (`mark_with_parent` — the bare-directory event is a separate `HashMap<PathBuf>` key). | **GREEN** — 4/4 |
| B1–B12 | 0 | The ledger contracts themselves: LF-only/NFC/relative-key encoding, the corrupt-store contract (one torn line costs one line), the link-life fold being idempotent *by arithmetic*, and the note-history stream **never** folding (`ma`→`mas`→`masadir` must survive as three events). | **GREEN** — 12/12 |

*(Slices 1+ append their rows here. A row may not be marked GREEN until its RED state was
demonstrated first.)*
