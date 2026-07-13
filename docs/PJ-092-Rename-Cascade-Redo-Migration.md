# PJ-092 Redo — Rename-Cascade Edit-Loss/Freeze — `/migration` Record

**2026-07-13.** The four-phase `/migration` (+ a new design-stage safety inspection) that fixed PJ-092 *properly* after the focused-fix band-aid froze the app and was reverted (`cfdb75a3`).

## Concept (the horse)
*A rename must never destroy a linked note's unsaved edits — and must never freeze doing so.* The cascade rewrites other notes' files while they may be open and dirty; the fix must keep every open editor's unsaved work safe when its file can't be written, without leaving the editor in a model↔disk state the reactive layer can't converge.

## The bug
`handleRenameComplete` → `flushAllTabsInLibrary` (swallowed the flush outcome) → Rust walker `update_links_on_rename` (rewrote `[[Old]]`→`[[New]]` in every backlink's disk) → `reloadTabsFromDisk` (force-reseeded each rewritten open tab's model *clean*). If a backlink note was open + dirty and its `.md` was momentarily locked (Syncthing/OneDrive/Defender), its flush failed silently; the walker still rewrote its stale disk; the reload force-reseeded → **edits lost, net wiped, save-health banner self-healed green.** The prior dirty-guard band-aid (`0a605f02`) stopped the loss but FROZE the app (a dirty + disk-diverged model behind a live editor with no `{#key}` remount to settle the reactive layer).

## Phase 1 — Architect
Territory-map + 4 design options, 2 dropped as freeze-unsafe:
- **flush-gate-exclude** ★ (picked) — the walker excludes not-durably-flushed notes by identity → their disk is never rewritten → no divergence.
- flush-gate-abort — skip the whole cascade if any flush fails (coarser, no Rust).
- ~~reuse-adopt~~ — routes dirty notes through PJ-070 conflict sidecar; inherits the divergence/freeze surface. DROPPED.
- ~~remount-converge~~ — remount the dirty model without reseeding; rests on the uncharacterized freeze cause. DROPPED.
**Boss picked flush-gate-exclude** (the only option whose no-loss guarantee is *structural* — never write the file).

## Phase 2 — Plan
9 steps, each landable as one commit, each with a verification clause; the sharp edge pinned: the JS↔Rust **path-normalization contract**.

## Phase 2.5 — Design-stage safety inspection (NEW standing step)
Adversarial inspection of the *Plan* before any code — 5 hazards caught for free:
- **H1 (app-killer):** the exclude match was a string compare, not file identity — the Arabic universe root is a live NFC/NFD surface; a mismatch silently reintroduces the loss. → **file-identity match** (`canonicalize` + NFC) + a frontend fail-closed belt.
- **H2 (app-killer):** `flushAllTabsInLibrary` single-shot `save` → a keystroke during the write leaves the model dirty at `ok:true`. → the **bounded re-flush loop**.
- **H3 (content-integrity):** `reloadTabsFromDisk` is focus-blind — a clean backlink in FocusPane during a rename silently reverts. → **focus-aware reseed**.
- **H4 (app-killer class):** the 4 sibling flush-then-reload callers share the loss primitive. → **fix all 4 in-scope** (Boss ruling) via `flushOpenTabOrAbort`.
- **H5 (low):** alias-map refresh skipped when all backlinks excluded. → refresh whenever there were backlinks.

## Phase 3 — Build
All amendments folded in. `/simplify` caught 2 more contract gaps (the belt wasn't NFC-folding — missing the very hazard it backstops; the 3 sibling gates re-opened the H2 race) → both fixed, siblings unified on the bounded-loop `flushOpenTabOrAbort`. Per-build safety-inspection: only the temporary LOCKTEST harness + pre-existing backlog. **Reproduce-First on the running app** via a dev LOCKTEST harness (content-based, removed before commit) — a real OS lock would also block the walker, so the failure must be frontend-only.

## Phase 4 — Audit
- **Invariant: 11/11 HOLD**, 0 regressed (concrete evidence, loss/freeze class + FocusPane/second-screen/restart).
- **Migration-path: PASS** — pure behavior change; `exclude_paths` is required (a missing arg fails closed → aborts → nothing rewritten); camelCase mapping proven; rollback clean (`walker_empty_exclude_rewrites_all` pins the empty case to pre-fix behavior).
- **Drift: one found + fixed** — the `cascade:rewrote` listener reloaded the raw rewritten paths, bypassing the belt; now both reload sites share the excluded set (`renameCascadeExcludedKeys`).

## Verification
`tests/mig-076/renameCascadeExclude.test.ts` (3) + Rust `cascade_walker_tests` (NFC/NFD identity, deliberate separator-mismatch exclude, empty-exclude rollback). svelte-check 0 · vitest 338 · cargo walker 16. **Boss live-test: A1 (normal rename), A2 (locked-note-protected + others-update, on the real Arabic-root universe), B1 (Focus mode), B2 (restart recovery), + clean-binary sanity — all PASS.**

## Follow-up
**PJ-097** — FocusPane isn't covered by the `CascadeFreezeOverlay` during a cascade (pre-existing; a contrived re-type-during-cascade race; PJ-092's H3 reseed is an improvement over the prior silent stale-revert).

## Lesson
A rename-cascade / reactive-lifecycle change is a `/migration`, not a focused fix — the reverted band-aid is the counter-example. And the design-stage safety inspection is now standing: catch the design flaw before it's built.
