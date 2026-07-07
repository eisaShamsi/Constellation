# Constellation Safety & Integrity Audit — Charter

**Opened:** 2026-07-07 · **Mandate (Eisa):** *"Stop everything and put the app under inspection to find and fix those app-killing bugs. I don't care how long it will take or how much effort — what matters is declaring the app safe and secure."*

**Trigger:** the rename→index durability bug (MIG-098) — a silent, source-of-truth-corrupting failure that hid for ~9 days and surfaced only by accident during unrelated tests. That is the signature of the most dangerous defect class, and the mandate is to hunt the whole class across the app, not just this instance.

## Definition — an "app-killer"

A defect that **silently** damages the user's knowledge or the app's ability to serve it, WITHOUT surfacing an error the user or a test would notice. Recoverability is irrelevant; *silence* is the defining trait. Ranked above all else.

## The taxonomy (hunt targets)

1. **Silent data loss / durability gaps** *(the trigger class)* — a source-of-truth write (`.md` file, `note_meta`, FTS/sky/links index) performed via a fire-and-forget / best-effort / unawaited task that can be lost (uninitialized DB, lock contention, app-close) with **no retry and no error**. Markers: `spawn`/`spawn_blocking` whose handle is dropped; `let _ =` on a fallible DB/FS write; `tokio::spawn` mutating the source of truth.
2. **Content integrity** — a note's on-screen or on-disk content acquiring ANOTHER note's data, or losing its own (the BUG-012/015/019/023 / LL-014 three-strike class). Editor lifecycle, save-composition, `{#key}` teardown, cross-note transitions, second-screen writes.
3. **Error swallowing / false success** — a function that reports success (`Ok(())`, resolved promise) when it actually **skipped** the work (e.g. `reindex_single_note` returning `Ok(())` on a `None` connection); `.catch(() => {})` on a write; `unwrap_or` that hides a failure. These MASK class 1 & 2.
4. **Index ↔ disk divergence** — any derived surface (`note_meta`, `notes_fts`, `sky_links`, backlinks, tag counts, aliases, `review_schedule`, embeddings) that can silently drift from the `.md` source (Rule 8 write-time-derivation gaps + missing reconcile).
5. **Init / ordering races** — an operation running before the DB/state is ready (the conn-`None` case), or a boot sequence that lets writes escape before the index is live.
6. **Concurrency / lifecycle races** — `$effect` read/write loops, cross-window sync, concurrent writers on the same file/row (TOCTOU), write-gate escapes.
7. **Freeze / hang killers** — an unbounded lock wait on the awaited IPC path (the §B2-4 freeze class), `invoke()` on the keystroke hot path.
8. **Resource leaks** — unclosed `listen`/timer/`EditorView`/`addEventListener` (the slow-death memory class).

Priority: **1 → 3 → 2 → 4** (the silent + hiding classes first), then 5–8.

## Method (no reinvention — WA#5)

1. **Anti-pattern sweep** — mechanically grep the syntactic markers of each class → candidate-site register (fast, mechanical, high-recall).
2. **Multi-agent semantic audit** — agents read each candidate + its data-flow and construct a CONCRETE failure scenario (inputs → silent damage). Fan-out by (subsystem × class).
3. **Adversarial verification** — every candidate independently attacked by a skeptic prompted to REFUTE it; only findings with a concrete, defensible repro survive → **Confirmed Register**. (No crying wolf.)
4. **Reproduce-First** — a confirmed app-killer is reproduced on the running app / via a test BEFORE any fix.
5. **Prior-art fixes (WA#5)** — fixes follow proven patterns (transactional outbox / WAL / translog / disk-reconcile for durability; single-ownership for content-integrity), cross-checked against mature systems. No inventive fixes where a battle-tested one exists.
6. **Verify each fix** — every fix proven against its repro (red→green) before the next.

## Phases

- **P0 — Charter + state-of-standing** (this doc + the session log snapshot). *DONE.*
- **P1 — Recon + anti-pattern sweep** — map every source-of-truth write path; grep the class-1/3 markers; build the candidate register.
- **P2 — Wave hunts** (fan-out + adversarial verify), in priority order:
  - Wave 1: durability / silent-loss / false-success (classes 1, 3) — the trigger family.
  - Wave 2: content-integrity + index↔disk divergence (classes 2, 4).
  - Wave 3: init/ordering + concurrency/lifecycle (classes 5, 6).
  - Wave 4: freeze + leaks (classes 7, 8).
- **P3 — Confirmed Register + severity ranking** (app-killer > integrity > freeze/leak).
- **P4 — Remediation** — fix confirmed app-killers with proven patterns, each Reproduce-First + verified. The MIG-098 rename-durability fix is remediation item #1.
- **P5 — Safe & Secure declaration** — a signed-off register with every app-killer confirmed-fixed + verified; regression tests where feasible.

## Findings Register

*(populated by the waves; each entry: id · class · file:line · failure scenario · verdict CONFIRMED/REFUTED · severity · fix status)*

_P1/Wave-1 in progress._
