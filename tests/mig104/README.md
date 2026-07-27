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
| H1–H9 | 4 | **The write hooks.** The line FORMAT with fixed human-readable field order (`v,t,cid,to,tn,n,at`) — which caught a false claim: `serde_json::json!` SORTS keys, so the lines are built by an explicit ordered writer. Q2's type-free key: `supports` + `derives-from` to one target fold to ONE record. An unresolved target still keys (by name) and survives the fold. The auto-tier is never recorded as a decision (`is_derivable_tier`). retire→restore reconstructs in order with the count untouched. A failed append returns Err — the contract the decision path relies on to abort the DB change. | **GREEN** — 30/30 (module total) |
| L1–L13 | 3 | **The appender + union reader.** One LF-terminated line per `write_all`, no CRLF; a torn tail loses ONLY the last line and counts it (every earlier record loads — an append cannot clobber); the fold is commutative + idempotent and `n` never decreases; a later decision wins while the count still maxes; **history never folds** (the `ma`→`mas`→`masadir` shape survives as 3 events, ordered by `hid` with identical `at`, and can never leak into the link-life fold); an unparseable line is skipped+counted with good lines loading on BOTH sides; a structurally-corrupt store is renamed **aside** (never deleted) and refuses a fresh write; `.gitignore` excludes all 8 machine files incl. the orphaned 939 MB `Constellation SV Test.db` (which `search.db*` would MISS) and excludes **no** ledger or config file; `ensure_gitignore` never overwrites the user's edit; a Syncthing conflict copy is folded in then removed; an absent store is a FACT not an error; keys carry no drive letter or backslash. | **GREEN** — 13/13 |
| D1–D4 | 2 | **Determinism + honesty.** D1 (RED-PROVEN): re-indexing a note with unchanged frontmatter manufactured **6 fake history rows in 6 re-indexes** (HashMap iteration order → byte-different `properties_json` → the trigger's `IS NOT` guard fires); with the sorted serializer, **0**. A real property edit is still recorded (the fix must not silence the stream it cleans). D4: the dot-segment guard — `.trash` / `.constellation` paths excluded from Pass 1, a RESTORED note (moved out of `.trash`) still indexable. | **GREEN** — 4/4 |
| W1–W4 | 1 | The `.constellation` watcher predicate: rejects the bare directory event, the tail, a vanished `.tmp`, the rename-aside, and the live D3 `cataloger_reliability.json` persist; ACCEPTS the user's knowledge incl. `.trash/*.md` and a vanished user folder; matches a whole component so `My .constellation notes` survives. Plus the three-key suppression contract (`mark_with_parent` — the bare-directory event is a separate `HashMap<PathBuf>` key). | **GREEN** — 4/4 |
| B1–B12 | 0 | The ledger contracts themselves: LF-only/NFC/relative-key encoding, the corrupt-store contract (one torn line costs one line), the link-life fold being idempotent *by arithmetic*, and the note-history stream **never** folding (`ma`→`mas`→`masadir` must survive as three events). | **GREEN** — 12/12 |

*(Slices 1+ append their rows here. A row may not be marked GREEN until its RED state was
demonstrated first.)*
