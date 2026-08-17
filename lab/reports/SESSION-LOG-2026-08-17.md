# Session Log — 2026-08-17

> **Scope note.** No session log was written for **2026-08-12, 08-15 or 08-16**. This file covers
> that whole stretch — one continuous session that crossed several date boundaries — from the
> MIG-111 Architect document through Phase 1.1 and the H1 harness. The gap is stated rather than
> hidden; SO#1 was not met on those days.

---

## Function in hand

**MIG-111 — full cross-universe operations.** Making an operation on a note that lives in a *linked*
universe do its bookkeeping in **that** universe's database, rather than in whichever universe
happens to be active. Phase 0 (the pre-Router safety work) closes here; Phase 1.1 (owner resolution)
lands; Phase 1.2 (the routed context pool) is next and now has its acceptance harness committed.

Alongside it, a remediation backlog: **PJ-278…283**, **PJ-287**, **PJ-294**.

---

## What shipped

| Commit | What |
|---|---|
| `3772c76f` | MIG-111 Architect — territory mapped, Option A recommended |
| `17b52807` | MIG-111 concept panel — validated 5/5, 37 requirements |
| `5973c2da` | MIG-111 Plan — Boss-approved; both adversarial attacks folded in |
| `b913c5c3` | Phase 0.1 — the live-WAL `fs::copy` ban (R11) |
| `f97b1448` | Phase 0.2 — the per-universe OWNER LOCK (R5); WAL probe demoted |
| `c27b53bf` | Phase 0.3 — the `link_life` ledger cross-process lock |
| `21419b7a` | The ①+② remediation — PJ-278…283, Boss-validated |
| `cfcddc3c` | PJ-287 — a write for a model that no longer exists is no longer a success |
| `c5a62d25` | PJ-294 — the Hotkeys screen binds keys for real; New Tab is a command |
| `9e539ab7` | Phase 0.4 — the writers on the federation boundary; **PHASE 0 CLOSES** |
| `660cfda8` | Phase 1.1 — `resolve_owner` (R2): which universe owns this path? |
| *(this commit)* | The H1 harness + the owner.rs verbatim-path fix + LL-047/048 + docs ×15 |

---

## Phase 0.4 — the writers on the federation boundary

Every command writing near a universe boundary now refuses a foreign target. Bases, the sources
rewriters, bulk accept, template creation.

**Four findings in this diff were defects the diff itself introduced.** The first is the one worth
re-reading:

- **The base guard was a DEAD NO-OP.** It compared a **canonicalized** path (`\\?\E:\…` on Windows)
  against the raw registry roots — it could never match, on any input. **Its test passed**, because
  the test asserted the guard's *source position* (a byte-offset comparison of where the call sat)
  rather than its *behaviour*. Fixed by passing the raw `file_path`; the test now drives the decision.
- Fixing that **broke "Save as template"**, because the guard had been applied to the SOURCE note —
  a READ, not a write. Now `validate_path_in_any_library` (read scope).
- The guard sat at the **call sites**, so a caller reaching the rewriter directly bypassed it. Moved
  INTO `rewrite_note_sources_on_disk` / `rewrite_note_content_type_on_disk`.
- The test asserting no command writes around the helpers **counted `gate_rmw` in one file** while
  `bulk_ops.rs` bypassed it. It now walks the directory — and found the bypass the moment it could
  see it.

## Phase 1.1 — `resolve_owner`

`federation/owner.rs`, alone and tested alone. Longest-match wins (H3); unknown is an `Err`, never
the active universe (H2); roots come from the federation tree, never from library lists. 9 tests,
clean on the first inspection round.

**It was not clean.** See below.

## The H1 harness — committed before the code it constrains

`federation/vocab_harness.rs` indexes one note under two vocabularies through the real `init_db` +
`index_note` + `maintain_incoming_after_save`, and diffs **aggregate VALUES, not rows** — because a
vocabulary mismatch leaves every row count correct while changing what the rows *say*. Phase 1.2's
acceptance condition is written into it now, `#[ignore]`d; **removing that attribute is what "1.2 is
done" means.**

Two things it found before any routing code exists:

1. `index_note` alone never reaches `maintain_incoming_after_save` — that runs on the save path. The
   harness drives it explicitly, or it would have observed only edges and none of the incoming
   aggregates H1 is fundamentally about.
2. **The determinism test failed on its first run** with identical inputs. `link_types::REGISTRY` is
   a **process-global** read at call time; a sibling test's `set_active` reached back into an
   already-open database and changed what it produced. That eliminates the design 1.2 was most
   likely to reach for — *open the child's connection, `set_active`, write, restore* — because the
   debounced save, a backfill tick or the watcher lands in that window. **→ LL-047.**

## The inspection finding: Phase 1.1 had the §0.4 defect, one layer up

The per-build inspection over this commit returned **one CONFIRMED HIGH** — in `owner.rs`, the phase
I had just called clean.

`resolve_child_universe_roots_recursive` builds its list with `fs::canonicalize` (verbatim
`\\?\E:\…`); the active root arrives raw. `norm()` knew nothing of verbatim prefixes, so:

- a **nested** linked universe (the DEFAULT shape under MIG-108) resolved to the **ACTIVE PARENT**
  with `is_active: true` — attack H3, defeated in the pure function and reintroduced by the wrapper;
- a linked **sibling** became permanently unroutable;
- `Owner.root` came back raw from one branch and verbatim from the other — **one universe, two
  identities**, which for a lock key means no lock at all.

**All nine tests stayed green** because every one drove the pure function with hand-built raw paths.
Reproduced with real directories and real `fs::canonicalize` (3 new tests, red → green), then fixed
in `norm` — stripping the verbatim prefix so the comparison is **total over path forms**, rather than
by promising callers will pass the right form. That promise has now been broken twice in one
migration. **→ LL-048.** The module header's claim that the roots "are raw" was false on the
federation side on the day it was written, and is corrected in place.

Whole-Ecosystem sweep of every `canonicalize` comparison: the rest canonicalize **both** sides, which
is safe. `universe_lock::canon` deliberately keeps the verbatim form because it also *builds* the
lock file's path and verbatim is what bypasses Windows' 260-character limit — left alone, with the
reconciliation contract now documented on `Owner::root`. Its raw-fallback ambiguity is filed as
**PJ-301** rather than changed under a MAX_PATH risk I have not measured.

## PJ-287 · PJ-294

- **PJ-287** — `saveUnchained` ratified a write whose model lineage had broken: a retry landing after
  the user clicked *"Discard my changes"* wrote the discarded bytes to disk. Now `lineageHolds` at
  the mutator; superseded writes return `{ ok: false, reason: 'superseded' }`. Two of the fixes in
  this diff were themselves cross-note defects, caught by the gate.
- **PJ-294** — rebinding a shortcut previously did nothing. **Twelve** inspection rounds (LL-046).
  The reserved-key list is now **derived** from every keymap in `src/lib/editor/` rather than
  hand-listed. **New Tab** became a real command (`Ctrl+Shift+T`).

## Two corrections from the Boss, both about the gate's blind spot

- *"There is NO tabs in Focus mode. You should know that."* — the step was auditor-written and
  inspector-APPROVED on a real effect no user can reach (`.focus-pane` is `position: fixed; inset: 0`
  and covers the window). Existence was verified; **reachability never was**. **→ LL-045.** When I
  then hunted for another route to force it, the Boss stopped that too: *"We design it this way, and
  that's why we name it Focus mode."* A surface's constraint IS its design.
- *"Holding Ctrl and clicking any note in the sidebar opens Move / Add tag / Delete… and this is true
  by design."* — approved on a real `Ctrl+click` handler that a capture-phase `document` listener
  reaches first. **The gate itself produced the wrong correction. → LL-045b.**

---

## Documentation

- **User Manual §17, all 15 languages** — the `Ctrl+Shift+T` row and a *Customising your shortcuts*
  section. Each translation quotes **its own** locale's labels (Settings → Hotkeys, *Filter
  commands*, *Press keys…*, *Not set*, *Reset*, *Clear*), so the manual names each control exactly as
  that reader sees it on screen.
- **New help topic** — `docs/help.uConstellation.World/Hotkeys/Hotkeys.md`.
- **Orientation v3.98**, **LL-047**, **LL-048**, **PJ ledger v1.91**.

---

## Gates

- Rust **1500 / 0** (20 ignored, incl. the 1.2 acceptance test)
- H1 harness: 3 passing + 1 `#[ignore]`d acceptance condition
- Diff-scoped `safety-inspection` on every commit; the run over Phase 1.1 is what found the HIGH
  above, and the fix was re-inspected before commit
- The Boss test for PJ-278…283 was **rejected twice** by `ui-inspector` before approval

## Open

- **Phase 1.2** — the routed context pool. Acceptance = removing the `#[ignore]`.
- **PJ-300** — federation cache degraded-resolve.
- **PJ-301** — `universe_lock::canon`'s raw fallback (filed today).
- **PJ-288** — awaiting a Boss ruling.
- **PJ-284** — freeze-and-leaks scope still unswept.
