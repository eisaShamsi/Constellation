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

## ★ A constraint Slice 6 (restore) MUST honour — discovered by the Slice-5 re-test

When a target name is ambiguous the seed correctly **refuses to guess** and keys the record on the
NAME. But two *different* links from one source to two *different* same-named notes then fold to ONE
record (live: `banana` ×2). Both happen to carry `n=1` today, so nothing is wrong yet — but if their
counts differed, a max-fold restore would give the lower link a count it never earned.

> **RULE: a name-keyed record (`to` empty) may restore ONLY when it resolves to exactly ONE
> `note_links` row. If several match, SKIP and report — never distribute one folded count across
> links that may have earned different amounts.** Identity-keyed records restore normally.

Needs its own RED-provable recipe at Slice 6: two same-named targets with DIFFERENT counts → the
restore skips both and says so.

## Slice 7 — the before/after on the live store (Rule 8)

| | bytes |
|---|---|
| `earned.jsonl` before | **6,222** |
| `earned.jsonl` after | **6,222** — unchanged |
| `earned.snapshot.jsonl` | **absent** |

Compaction fires on a **byte threshold (2 MB)**, so the live store — 33 earned links seeded into
6 KB — is ~340× below it and the pass does exactly one `metadata()` call per boot and returns.
That is the intended steady state and the reason this slice is **not Boss-testable**: there is no
user-visible change until a Universe has recorded on the order of 10,000 decisions. Boot timings
are therefore unchanged against Slice 0 by construction, not by measurement noise.

## Recipe registry

| Recipe | Slice | What it reproduces | State |
|---|---|---|---|
| C1–C19 | 7 | **The snapshot + compactor**, and the two dead guards it exposed. `compaction_is_lossless`: 16,800 lines + every other record kind → the folded state before == after, and the snapshot is smaller. `every_snapshot_line_folds_back_to_its_own_key` — the property the slice rests on (a re-keyed line would attach the user's count to the wrong link). The tail is **renamed aside, byte-identical, never deleted**, and is NOT read back (which is what bounds the load). A **crash between the snapshot and the rename loses nothing** — both files are read and the duplicated region folds to the same answer, which is why the snapshot is written FIRST. `threshold_is_bytes_not_time`: 100 load cycles on an idle store → zero writes, zero temps, no snapshot, mtime untouched. `note_history_is_never_compacted`: a 40,003-event Stream B over the threshold is untouched to the byte (and `maybe_compact` has **no stream parameter**, so it cannot be pointed at it). Two compactions in one second keep **both** aside copies. PJ-087: temp names unique, same directory. The documented **revert recipe** (`cat` snapshot + tails back into `earned.jsonl`) is tested, not just written in a commit message. A `"seed":1` marker survives; a witnessed record is not relabelled as derived. An absent timestamp round-trips as absent, never as `""`. | **GREEN** — 19/19 |
| ★ C20–C24 | 7 | **TWO DEAD GUARDS AND AN UNREAD RECORD, found by building the compactor.** (a) `refuse_write` was set ONLY inside `link_life::quarantine`, which returns its **own** report — so `link_life_restore`'s *"do NOT write a thing from a store we could not read"* was **structurally unable to fire** while reading as a live protection (the LL-035 shape: "X is inactive" is a runtime claim). The reader now OBSERVES the quarantine on disk (`earned.corrupt-*.jsonl` present = un-acknowledged), so both the restore's and the compactor's guards are real and testable; acknowledging is the user moving the file away. (b) **`priority` records were written and fsync'd since Slice 4 and never read back** — the fold's key function required a target, so every one was dropped, and the Plan's Slice-6 clause *"restores review priority too"* was never true. Losing `search.db` still cost the user every review priority they had set. Now folded (`LedgerState.notes`), snapshotted, and restored — with `-1` mapping back to SQL `NULL`, and the same one-row-or-skip rule that governs an ambiguous link. | **GREEN** — 5/5 |
| LIVE | 6 | **★ BOSS TEST, PASS on the real Universe (2026-07-27).** Earned layer wiped in the DB (38 rows → 0), ledger left as the only copy, app rebooted: **34 of 34 planned writes applied, 34 exact matches, 0 mismatches**, 1 impossible weight healed. The remaining 4 are correct-by-design: 2 whose source note is absent from the index (unkeyable → never recorded) and the 2 `banana` rows the ambiguity rule protects. `France` returned **archived** — a retirement surviving the destruction of the layer that held it. | **PASS** |
| R10–R11 | 6b | **BOSS-FOUND LIVE — the bug the migration exists to prevent.** On a rebuilt index the restore raced the initial indexing, read "no link to attach to yet" as "the links are gone", and **stamped itself complete** — the earned data would never have returned. Root error: a RECONCILER carrying a MIGRATION's stamp. Fixed by (1) removing the stamp entirely — it now runs every boot, silent in the steady state, like `reconcile` — and (2) an `index_not_ready` guard: zero links in `note_links` reports NOT-READY and writes nothing, never "gone". The test pins both halves: empty index → nothing concluded; a LATER pass restores everything, which only works because the pass is unstamped. The seed got the same stamp guard: *"I found nothing" and "there is nothing" are different claims.* | **GREEN** — 2/2 |
| R1–R9 | 6 | **THE RESTORE — the point of the migration.** `db_loss_round_trip`: delete `note_links` entirely, let the links come back as brand-new (`hypothesis / 1.0 / 0`), restore → every earned value returns, a retired link **stays retired**, and `weight` matches `earned_link_weight(n)`. **The Boss's `banana` rule, RED-provable:** a name-keyed record matching SEVERAL rows is SKIPPED and counted — neither link is handed a count it might not have earned; an *unambiguous* name-keyed record still restores. Idempotent (2nd pass writes 0, reports `already_current`). A **newer DB count is never ratcheted down** by an older ledger. The **weight heal** recomputes the off-curve 236-shape and leaves on-curve rows untouched. A record whose link is gone is counted, not fatal. An **unreadable store writes NOTHING** (that is how a restore destroys what it protects). An identity-keyed record **survives a target rename** — the identity is the durable half of the key. | **GREEN** — 9/9 |
| S9–S11 | 5b | **BOSS-FOUND, RED-proven on live data.** (a) The target join **fanned out on duplicate note names** — 3 notes named `السعودية`, 2 `فلسفة`, 2 `banana`, 2 `collision test` turned **38 earned links into 44 records**, 6 of them asserting links the user never walked. Fixed by a correlated subquery that resolves an identity ONLY when exactly one note carries the name, and otherwise **refuses to guess** and keys on the name (what the fold's fallback exists for). Tests: an ambiguous name emits ONE record with an empty `to` and none of the candidate cids; a unique name still resolves, with the note's real title as the label. (b) **Seeded lines are marked `"seed":1`** — a seeded decision's timestamp is borrowed from `last_traversed` and is NOT when the decision happened (a Contested click at 09:21:25 was seeded as 09:13:51). The time cannot be made true, so the record says it is derived; the marker keeps the line valid JSON and does not disturb the fold. | **GREEN** — 3/3 |
| S1–S8 | 5 | **The seed.** The ONE earned predicate matches earned rows three ways (walked / retired / user-judged) and excludes the unearned — including the 236-row **off-curve-weight** shape and the all-rows `last_traversed` shape the two prohibitions exist to keep out. `structural` rows are skipped **in the loop** (the predicate alone cannot exclude them — a structural edge can have `traversal_count>0`) and counted. Orphan-source rows are skipped, counted, and **never written** (a record we cannot key can never be restored). A re-seed converges **by arithmetic, not by the stamp** (run 3×, identical fold). A newer walk already in the store is **never ratcheted down** by a seed from an older DB. An archived row seeds a `retire`, so a rebuild cannot resurrect it. A derivable confidence seeds no `trust`; a manual tier that outranks the count does. | **GREEN** — 8/8 |
| H1–H9 | 4 | **The write hooks.** The line FORMAT with fixed human-readable field order (`v,t,cid,to,tn,n,at`) — which caught a false claim: `serde_json::json!` SORTS keys, so the lines are built by an explicit ordered writer. Q2's type-free key: `supports` + `derives-from` to one target fold to ONE record. An unresolved target still keys (by name) and survives the fold. The auto-tier is never recorded as a decision (`is_derivable_tier`). retire→restore reconstructs in order with the count untouched. A failed append returns Err — the contract the decision path relies on to abort the DB change. | **GREEN** — 30/30 (module total) |
| L1–L13 | 3 | **The appender + union reader.** One LF-terminated line per `write_all`, no CRLF; a torn tail loses ONLY the last line and counts it (every earlier record loads — an append cannot clobber); the fold is commutative + idempotent and `n` never decreases; a later decision wins while the count still maxes; **history never folds** (the `ma`→`mas`→`masadir` shape survives as 3 events, ordered by `hid` with identical `at`, and can never leak into the link-life fold); an unparseable line is skipped+counted with good lines loading on BOTH sides; a structurally-corrupt store is renamed **aside** (never deleted) and refuses a fresh write; `.gitignore` excludes all 8 machine files incl. the orphaned 939 MB `Constellation SV Test.db` (which `search.db*` would MISS) and excludes **no** ledger or config file; `ensure_gitignore` never overwrites the user's edit; a Syncthing conflict copy is folded in then removed; an absent store is a FACT not an error; keys carry no drive letter or backslash. | **GREEN** — 13/13 |
| D1–D4 | 2 | **Determinism + honesty.** D1 (RED-PROVEN): re-indexing a note with unchanged frontmatter manufactured **6 fake history rows in 6 re-indexes** (HashMap iteration order → byte-different `properties_json` → the trigger's `IS NOT` guard fires); with the sorted serializer, **0**. A real property edit is still recorded (the fix must not silence the stream it cleans). D4: the dot-segment guard — `.trash` / `.constellation` paths excluded from Pass 1, a RESTORED note (moved out of `.trash`) still indexable. | **GREEN** — 4/4 |
| W1–W4 | 1 | The `.constellation` watcher predicate: rejects the bare directory event, the tail, a vanished `.tmp`, the rename-aside, and the live D3 `cataloger_reliability.json` persist; ACCEPTS the user's knowledge incl. `.trash/*.md` and a vanished user folder; matches a whole component so `My .constellation notes` survives. Plus the three-key suppression contract (`mark_with_parent` — the bare-directory event is a separate `HashMap<PathBuf>` key). | **GREEN** — 4/4 |
| B1–B12 | 0 | The ledger contracts themselves: LF-only/NFC/relative-key encoding, the corrupt-store contract (one torn line costs one line), the link-life fold being idempotent *by arithmetic*, and the note-history stream **never** folding (`ma`→`mas`→`masadir` must survive as three events). | **GREEN** — 12/12 |

*(Slices 1+ append their rows here. A row may not be marked GREEN until its RED state was
demonstrated first.)*
