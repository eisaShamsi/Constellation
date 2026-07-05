# MIG-090 — Plan: the Navigator v1 (the Workbench + Intent Bar)

**Date:** 2026-07-05 · **Phase:** 2 of 4 (Plan) · **Concept:** [RATIFIED paper](concept-papers/MIG-090-Notes-Navigator-v2-Concept-Paper.md) — the horse verbatim, Form C
**Architect delta:** workflow `wf_e6c033e2-d9d` (4 designers + adversarial verifier; spine upheld; corrections applied below). All reads = existing indexes; **v1 writes no note content** — the Editor-Surface Gate is not triggered (and the plan documents why at close-out).

---

## The v1 shape (the verifier's ruthless cut, honoring the horse)

- **The Workbench** — a center surface (mounted exactly like the Reviewer: conditional overlay, joins the full-page exclusion set), entered by a new dock button + command-palette entry, behind a new `enabledFeatures.workbench` flag (**default OFF** — you turn it on to test). **v1 adds, never removes:** the old Navigator's sidebar mode keeps running untouched; its retirement is a separate ruling at the validated-swap moment.
- **The Intent Bar** — the Workbench's front door only (no second global overlay). One input with the QuickSwitcher's proven conventions (IME guard, 300 ms debounce, stale-result guard, `dir=auto`); the typed phrase goes **verbatim** to the existing hybrid search engine (text + semantic fused by rank); when semantic embeddings are off/absent, a quiet "semantic off" hint — honest degradation, never silence.
- **ONE working set**, persisted per-universe in `.constellation/workbench.json` on the bookmarks/workspaces pattern (read/save IPC pair, ensure-on-init, post-paint load — never on the boot path). Members keyed by **cid_cn** (the stable canonical ID → survives renames AND folder moves) with a **non-writing path-keyed fallback** for notes lacking one (adding a note NEVER writes its file), path items riding the existing rename-migration hook. Soft cap 100. Schema nests `sets[]` from day one so multiple named sets (v2) need no file migration. Missing member (note deleted externally) = kept with a "missing" standing until set down — honest state, no silent pruning. Desk opens on demand; boot untouched.
- **Read-only verbs:** open (Ctrl/middle = new tab) · set down · done + sweep-done. Pick-up: from bar results, palette "Add current note," and ONE `addToWorkbench` callback in the shared context-menu builder — auto-surfacing in every wired right-click (the bookmark precedent).
- **The shared primitive:** `NoteRow` + `NoteList` composing the **existing** shared `VirtualList` (already used by 6 panels — the verifier corrected my earlier premise), with the row-height contract exported from one place (defusing an acknowledged keep-in-sync trap in BacklinksPanel). Built to the Workbench's needs; the 9-surface adoption is explicitly **PJ-069 work, not v1**.
- **Four state chips**, client-side intersection over the set's one hydration read — **due** (folding the Reviewer's checkpoint/interval/never-reviewed reasons, documented), **unlinked**, **contested**, **forming** (the note's own declared stage) — zero extra IPC; chips **narrow, never query** (a pinned test guards the verified engine landmine where filters would *expand* a hybrid result).
- **Liveness:** one debounced re-hydrate on the existing mutation events only (note-created, cascade-rewrote, cache-reconciled, note-saved). Membership-only persistence + re-read-on-display is the structural cure for the stale-snapshot hazard that killed the old Navigator.

**Cut to v2 (each with its reason):** writing verbs (add-link / advance-stage / send-to-reviewer — every one drags the full content-integrity harness in; the horse lives without them: hold + act-by-opening is intact) · the stale chip (needs the priciest join; four chips prove the concept) · chips-as-universe-queries · multiple named sets · save-as-Base/Trail exports · any second-screen presence · the old Navigator's retirement + 1,033-line deletion.

## The steps (each = one commit + verification clause)

1. **§1 Rust persistence** — `read/save_universe_workbench` cloned from the bookmarks pair + ensure-on-init + registration. *Verify: cargo build; round-trip test; missing file → empty.*
2. **§2 Hydration read** — new read-only `(async)` `workbench_hydrate(keys)`: ONE indexed SELECT over `note_meta` (name/library/modified/stage/link counts+types) LEFT JOIN `review_schedule`, cid keys via the UNIQUE index, path keys via the PK; review columns honestly empty when that schema is mid-backfill (the Reviewer's own gate reused). *Verify: cargo test with seeded rows incl. mid-backfill.*
3. **§3 Frontend store** — the bookmarks shape (writable + add/remove/toggleDone/sweep + whole-file save + post-paint load); path-keyed items hooked into the rename-migration block. *Verify: svelte-check; rename + folder-move keep membership.*
4. **§4 Surface shell + primitive** — `WorkbenchView` (Reviewer-style mount, flag default-off, dock button, palette entry) + `NoteRow`/`NoteList` over `VirtualList` (+ a small `scrollToIndex` export fixing selection-scroll under virtualization). *Verify: svelte-check; empty-desk state renders; 100-row set scrolls at budget.*
5. **§5 Intent Bar** — QuickSwitcher conventions; verbatim phrase → hybrid search + embeddings vector when enabled; "semantic off" hint; results rendered via NoteList. *Verify: svelte-check; stale-guard test; degraded-mode hint visible.*
6. **§6 Hold + act round-trip** — pick-up wiring (bar/palette/context-menu) + open/set-down/done + save-on-mutation + hydrate-on-open. ***Boss test** (tutorial per the Testing Instructions Rule): ask → hold 5 → restart → desk intact, facts live.*
7. **§7 Chips** — four toggles as in-memory intersection. *Verify: chip toggle does ZERO IPC (traced) and only narrows — the pinned intersection test.*
8. **§8 Liveness** — the debounced event listener (unlisten on destroy). *Verify: 10 s of typing elsewhere → ≤1 refresh after settle; a rename cascade → one refresh, membership intact.*
9. **§9 Discovered defect (fix-what-we-discover)** — `addLinkToNote`'s closed-note branch lacks the cascade guard its open branch has: add the one-line `isCascading` early-return. *Verify: Reviewer connect flow unchanged; connect-during-cascade refused.*
10. **§10 Close-out** — help topic + User Manual ×15; session log + orientation v-bump same commit; /simplify; audit note ("v1 writes no note content — why the Gate doesn't fire"); boot + typing re-measured on the 7,600-note universe vs baseline; staged Boss tutorial.

## Defaults folded in (approved with this plan)
Forming chip = the note's own declared stage (honest absence when unstaged) · missing members kept visible, never auto-pruned · desk on-demand, no boot auto-open · federated (child-universe) members: holdable + openable; the question of mutating verbs on them belongs to v2 with the verbs themselves.

## Cost & risk
~10 commits, one focused session. Risk concentrated in §5's engine wiring (guarded by the QuickSwitcher conventions) and §7's intersection semantics (pinned test). No content writes; no boot-path changes; flag-gated throughout; old surface untouched.
