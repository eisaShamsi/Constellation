# MIG-061 — Boss Test: CNS Gravity Well Federation

**Date:** 2026-05-28
**Architect:** `docs/MIG-061-cns-federation-ARCHITECT.md`
**Plan:** `docs/MIG-061-cns-federation-PLAN.md`

---

## What just shipped

The Sky View / CNS / Backlinks / Outgoing data source — a single Tauri command called `cache_boot_snapshot_sky` — was reading only from the active universe's `sky_nodes` / `sky_links` tables. In Eisa Universe (24 cUniverses + 8 751 notes), that meant CNS showed only **987 nodes** instead of the full federation.

MIG-061 federates that command. Per the Architect §8 locks:

| Decision | Lock |
|---|---|
| Q1 — Approach | Rust per-schema loop + merge |
| Q2 — Node IDs | Keep `id = lower(name)`; `path` for disambiguation |
| Q3 — Link resolution | Federated across the merged node set |
| Q4 — Readiness | All-or-nothing (every schema must be ready, or fallback to legacy `buildSkyData`) |

Seven commits landed in `main`:

| § | Commit | What |
|---|---|---|
| §A | `6b5173fa` | `get_federated_schemas` helper |
| §B | `76f9f826` | `read_sky_nodes_raw_in_schema` |
| §C | `dc43e753` | `read_sky_links_raw_in_schema` |
| §D | `4df77b6d` | `is_federated_sky_ready` (Q4 all-or-nothing) |
| §E | `ade3a010` | Integration: federation loop in `cache_boot_snapshot_sky` |
| §G | `1d755755` | 8 unit tests (848/848 lib tests pass) |
| §H | (this doc) | Boss-test |

§F was rolled into §E inline. §I (PCS) lands after Boss-test passes.

---

## Test stages

### Stage 1 — Single-universe regression (no cUniverses)

**Pre-state:** Launch the fresh binary. Switch to **Eisa Cognitive Knowledge** universe (the one without cUniverses).

**Action:**
1. Open CNS via the dock button (eye icon).
2. Look at the header.

**Expected:**
- Header shows roughly the same node count as before MIG-061 (the universe has its own notes; that count should be unchanged).
- Gravity well renders.
- Single-universe boot time feels identical to pre-MIG-061 (no perceptible regression).

**Why this matters:** INV-1 — single-universe Universes must not regress. The federation code path detects this case (`schemas.len() == 1`) and falls through to the same bare-Connection path used pre-MIG-061.

**Failure modes:**
- *Node count changed significantly.* → bug in the single-universe code path; the new `schemas.len() == 1` branch is misbehaving.
- *Boot feels noticeably slower.* → unexpected overhead in the federation-detection or readiness-check logic.

---

### Stage 2 — Federated count (THE headline test)

**Pre-state:** Switch to **Eisa Universe** (the one with 24 cUniverses + 8 751 notes — federation root). Close CNS if it's already open.

**Action:**
1. Open CNS via the dock button.
2. Look at the header — `Constellation Nervous System (CNS) NNN nodes · MMM links`.

**Expected:**
- Header shows **a node count close to 8 751** (or whatever your Universe's actual federated total is — much greater than the 987 you saw before MIG-061).
- Link count is dramatically higher than 1 178 (the federated link total).
- Gravity well renders all the federated nodes — multiple library colors visible in significant clusters, not just one cluster of pink.

**Why this matters:** The whole point of MIG-061. If this test passes, CNS now sees the full federation.

**Failure modes:**
- *Still shows 987 nodes.* → §E's federation detection didn't fire OR `federated_conn` is None at query time. Check `get_federated_schemas` return value via diag-log (if needed, I'll add tracing).
- *Header shows `is_ready=false` / empty gravity well.* → Q4 Option A's all-or-nothing readiness gate detected an unstamped cUniverse. One of your cUniverses hasn't completed its `sky_schema_version` back-fill yet. Wait a minute for back-fill, retry. If persistent, we surface which cUniverse is unstamped via diag-log.
- *Node count >> 8 751.* → Cross-schema deduplication issue (probably duplicate paths from a misconfigured cUniverse — fixable via universe.json review).

---

### Stage 3 — Boot time check

**Pre-state:** Eisa Universe is active. Restart Constellation (close + relaunch).

**Action:**
1. Observe boot time (eyeball — seconds from window open to CNS dock-button responsive).
2. Open CNS once boot is complete.

**Expected:**
- Boot time on Eisa Universe is within roughly **2× of single-universe boot time** (INV-2). With 24 cUniverses, each adds a tiny per-schema query overhead; the warm `federated_conn` keeps prepare-plan cost low.

**Failure modes:**
- *Boot is dramatically slower than before* (e.g., > 5× single-universe) → unexpected per-schema query bottleneck. We'd profile via `timings_ms` (returned in the snapshot response) to find which scan phase is the cost.

---

### Stage 4 — Backlinks panel for a cUniverse note

**Pre-state:** Eisa Universe active. Stage 2 passed (CNS shows full federation).

**Action:**
1. Open a note that lives in a cUniverse — pick any note from one of your 24 child-libraries.
2. Open the right-sidebar **Backlinks** tab.

**Expected:**
- Backlinks panel shows ALL notes linking to this cUniverse note — including notes from OTHER universes (cross-universe wikilinks resolve per Q3 Option B).
- Previously: if the note was in cu0 and was linked from main, the link would be missing.

**Failure modes:**
- *Backlinks panel is empty / fewer entries than expected.* → frontend `getBacklinks` filter on `allLibraryLinks` may not be reading the merged data. We'd add a log/breakpoint.

---

### Stage 5 — Outgoing Links panel for a cUniverse note

**Same pre-state as Stage 4.**

**Action:** Switch to the right-sidebar **Outgoing Links** tab on the same cUniverse note.

**Expected:** All wikilinks the note has — including those pointing into OTHER universes — appear here.

**Failure modes:** Same as Stage 4.

---

### Stage 6 — Sky View parity

**Pre-state:** Eisa Universe active.

**Action:** Switch to the **Sky View** dock button (different visualization of the same data — bubbles instead of gravity well).

**Expected:**
- Same node count as CNS (Stage 2's headline number).
- Same library coloring spread (multiple clusters from different libraries).

**Failure modes:**
- *Sky View shows different node count than CNS.* → the two surfaces should share the same `skyNodes` state; if they diverge, something else is filtering. Investigate.

---

## After all stages pass

Reply with **"All pass"** and Claude will:

1. Cascade to **§I (PCS)** — orientation v2.41, MoCh entry for today's marathon, 15-locale help-doc updates (CNS / Backlinks help docs gain a "shows your full federation" line), milestone tag `milestone/mig-061-cns-federation-shipped`, ZIP backup.
2. Then auto-spawn the three Audit-phase agents (invariant-checker, drift-detector, migration-path-validator) per the /migration workflow.
3. Then propose MIG-062 (P3 — filesystem walks) as the next step.

If any stage fails, paste the failure mode and Claude will triage before §I.

---

## Build status

Build command: `npm run tauri build`. After completion, the binary at `src-tauri/target/release/constellation.exe` (mtime should be **post 17:00 today / post commit `1d755755`**) is the test binary. Close Constellation fully (Task Manager check) before launching the new binary.
