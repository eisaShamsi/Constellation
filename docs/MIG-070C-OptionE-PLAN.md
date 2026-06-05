# MIG-070 §C — Option E (Hybrid Live Preview for non-Editor categories) — ARCHITECT + PLAN

**Written:** 2026-06-05 · **Chosen by Eisa:** Option E (hybrid), from `docs/MIG-070C-non-editor-preview-RESEARCH.md`. · **Status:** plan — awaiting approval before build.

Goal: the non-Editor Style Setter categories (Interface, Components, Global, Sky View, OrgChart, Index, Cataloger, Shell) get a live preview. Build incrementally: **A** (the real app *is* the preview) → **D** (inspect-to-style) → **C** (inline atoms for Global). The **Editor** category's centre note-preview is unchanged throughout.

---

## The key architectural decision (the safe answer to the live-apply/revert + BUG-015 risk)

Today the Setter `draft` is scoped to the `.ss` overlay wrapper (`style={draftStyle}`) — it previews **only inside the overlay**; `apply()` persists it to `appSettings.styleOverride`, and the **one** shared apply `$effect` in `+layout.svelte` (L1554) writes `theme → Style-Settings → styleOverride` (last writer) onto `document.body`, tracking keys in `_lastStyleSettingsKeys` for clean clear-down.

**Option A needs the draft to restyle the REAL app live, then revert cleanly on cancel** — and the canonical BUG-015 was a second `$effect` racing the apply path. So we do NOT write `body` from a second place. Instead:

> **Add a transient in-memory `liveStyleDraft` layer, merged LAST in the existing apply `$effect` (above `styleOverride`), tracked in `_lastStyleSettingsKeys`.** The Setter feeds it while open; **Keep** promotes it to `styleOverride`; **Discard/close** clears it and the same `$effect` reverts.

Why this is correct:
- **One apply path, no race.** `liveStyleDraft` is just another tracked source in the *same* `$effect`; when it changes the effect re-runs and re-applies `theme → Style-Settings → styleOverride → liveStyleDraft`; when cleared, the existing stale-key cleanup (L1631-1633) removes its vars → reverts. No competing writer, no BUG-015 shape.
- **No persistence churn / performance-safe.** `liveStyleDraft` is an in-memory `writable` — dragging a slider does **zero** `invoke`/disk writes (Rule 3 / IPC contract). Persistence happens once, on **Keep** (`mergeStyleOverride`).
- **Discard never mutates saved state.** Cancelling clears the in-memory layer only; `styleOverride` on disk is untouched.
- **Survives theme switch + clears cleanly** — inherits the proven `_lastStyleSettingsKeys` machinery.

`liveStyleDraft` precedence: `theme < Style-Settings < styleOverride < liveStyleDraft` (the live edit wins while open, exactly like the centre preview wins today).

---

## Phases (each independently landable; one `§E-x` commit; Boss test where marked)

### Phase E-A — "The real app is the preview" (the substrate)
- **A1** — Add `liveStyleDraft` (`writable<Record<string,string>>`, in-memory) + `setLiveStyleDraft`/`clearLiveStyleDraft` in `store.ts`; merge it LAST in the apply `$effect` (after L1627), into `trackedVars` (so clean clear-down already covers it). *Verify:* set a var via devtools → real chrome restyles; clear → reverts; switch theme back/forth → live layer still on top, clears cleanly. **(No UI; the safety-critical step — written + reviewed first.)**
- **A2** — Setter "live mode" for non-Editor categories: the `draft` writes through to `liveStyleDraft` reactively (while open + category ≠ editor); the panel **docks to one side + goes translucent** so the real app shows; **"Apply to app" → Keep** (`mergeStyleOverride(draft)` + `clearLiveStyleDraft`), **close / a new "Discard" → `clearLiveStyleDraft`** (revert). The seed-on-open + Reset paths updated to match. **[BOSS TEST]** open Interface/Components → drag a chrome colour/size → the **real** sidebar/dock/tabs restyle live; **Keep** persists (survives relaunch); **Discard** reverts with nothing saved.
- **A3** — Editor category keeps its centred 3-zone note preview (no live-on-real-app needed — it previews its own note). *Verify:* Editor unchanged; switching Editor↔non-Editor flips centred-modal ↔ docked-translucent cleanly, with no stuck `liveStyleDraft`.

### Phase E-D — Inspect-to-style (chrome discovery)
- **D1** — Tag chrome elements with `data-style-target="<elementKey>"` (FileTree rows, dock, sidebar toolbar, layout bar, tabs/top bar, right sidebar, status bar, universe bar, buttons, tags) — a small registry mapping DOM → the Setter's element keys. *Verify:* every Components/Interface element key has ≥1 real target; no key orphaned.
- **D2** — An **"Inspect" toggle** in the docked Setter: while on, the panel is click-through except its own controls; hovering the real app outlines + names the nearest `data-style-target`; **clicking it jumps the Setter to that element** (category + element selected). **[BOSS TEST]** turn on Inspect → hover the dock → it highlights "Ribbon dock" → click → the Setter opens the dock's controls.

### Phase E-C — Inline atoms for Global
- **C1** — Extend the inline-preview (Links-pill) pattern to the **Global** category's atomic controls: each shade/colour shows a sample chip, each radius shows a corner sample, each border shows a sample edge — so Global needs no central preview. *Verify:* every Global control shows its own live effect; matches the pill precedent.

---

## Invariants (must not break)
- **Single apply path** — `liveStyleDraft` lives only inside the existing `+layout` `$effect`; no second writer to `body` (BUG-015 guard).
- **Performance** — `liveStyleDraft` in-memory; zero IPC/disk on edit; persist once on Keep; measure typing/boot on the 7,600-note Universe unaffected.
- **Discard safety** — cancelling never mutates `styleOverride` on disk.
- **Theme switch + clean clear-down** — `liveStyleDraft` keys ride `_lastStyleSettingsKeys`.
- **Editor unchanged** — its centre note-preview + 3-zone stay.
- **Second screen** — out of scope here (Phase 8 syncs `styleOverride`; `liveStyleDraft` is main-window-only by design — a transient edit layer, not persisted state).
- **Frozen MIG-069 presets** — untouched.

## Rollback
- `liveStyleDraft` is additive + in-memory. Revert the A1 merge line → the Setter falls back to today's Apply-only model (draft scoped to the overlay); no schema change, no data touched. Each `§E-x` is one commit; revert any phase independently.

## Risk → mitigation
| Risk | Step | Mitigation |
|---|---|---|
| Race / BUG-015 on the apply path | A1 | Same single `$effect`, one writer; no parallel effect |
| IPC/disk churn on drag | A1/A2 | In-memory layer; persist only on Keep |
| Cancel corrupts saved look | A2 | Discard clears in-memory only |
| Stuck live layer on category/window switch | A3 | Clear `liveStyleDraft` on close + on Editor switch |
| Inspect can't find a control | D1 | Orphan-check every element key has a `data-style-target` |
| Mock drift | — | Avoided by design — A previews the real app (no mocks) |

**Commit estimate ≈ 6:** A1 · A2 · A3 · D1 · D2 · C1.
