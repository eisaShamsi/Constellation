# Handover — 2026-07-01 — MIG-088 §3b–§3d (editor sweep) + Phase 4 §4a/§4b (chrome)

**Branch:** `main` (pushed, clean, tip `c543046c`). **Orientation:** **v3.19** is the file to read. **Session log:** `lab/reports/SESSION-LOG-2026-06-30.md` (this session's entries are appended after the MIG-089 CLOSE-OUT). **MoCh:** `docs/MoCh/MoCh-2026-07-01-0900.md`.

## What shipped this session (all Boss-validated, pushed)
- **MIG-088 §3b Highlight** (`fc18fa00`,`9b1c484b`) — `<mark>`/`==`/toolbar-chip share `--highlight-bg`/`--highlight-text`/`--highlight-radius`. Text control added at Boss test (dark-bg legibility).
- **MIG-088 §3c Syntax tokens** (`963cfd15`) — `--url-color` (URL) + `--syntax-meta-color` (control **"Markup marks"** — the in-editor markdown marks, NOT frontmatter).
- **MIG-088 §3d Link chip** (`97d6d6ce`,`c7f185b8`) — the ×N traversal chip: `--link-chip-bg` (solid) / `--link-chip-text` / `--link-chip-radius`. Boss ruled "Link chip only".
- **/simplify** on the §3b–§3d diff (`07436595`).
- **MIG-088 Phase 4 §4a Tab bar extras** (`b2e7f487`) — new-tab `+` button + bulb + scroll arrows → `cTabExtras` element (Components). Boss PASS.
- **MIG-088 Phase 4 §4b Shadows** (`dd08026a`,`c543046c`) — 3 tokens (`--modal-shadow`/`--popover-shadow`/`--tooltip-shadow`) unify ~11 live-surface box-shadows; `gShadows` element (Global, select presets). Boss PASS. **The §4b fix (`c543046c`) reverted wirings to STALE components + dropped the dead `--dropdown-shadow`.**

## Canonical facts (mental model)
- Style Setter now: **Editor** += Highlight/Syntax tokens/Link chip; **Components** += Tab bar extras; **Global** += Shadows.
- **STALE / unmounted — never style these:** `CodeMirrorEditor.svelte` (0 importers), `EditorContextMenu`, `FormattingToolbar`. The live editor is `NotePane` (`.e-tb` toolbar); the note-body right-click is the shared `ContextMenu` (`--shadow-l`).
- Apply path: `+layout.svelte:~1985` writes ALL `styleOverride`/`liveStyleDraft` vars to `document.body.style` generically. New vars need no allow-listing.
- Add-a-styleable-element recipe (memory `project_stylesetter_add_element_recipe`) + **NEW rule: verify the component is actually MOUNTED before treating a hardcoded value as a Phase target.**

## Deferred / open (honest, not parked)
1. **§4c dialog-scrim consolidation** — the 4 inconsistent backdrop opacities (0.3/0.35/0.4/0.6). Needs an **opacity** control (a colour picker can't express a see-through scrim). Its own micro-step.
2. **Disabled-Wing shadows** — Sight (OFF), Map, GraphMind, SenseMakingCanvas, Inspector360 panel, tour — wire when those surfaces ship.
3. **8 typed-link type colours** (`livePreview.ts:181-182`) — a cross-surface cognitive colour set (mini Phase-2 "unify on demand"); its own step.
4. **Help-topic folders ×15** for callout customisation (User Manual ×15 done; topic folders optional).
5. **Arabic callout End/Home caret** known-issue (CM6 RTL-caret on callout lines; from v3.18; needs reproduction + structural fix — do NOT re-apply a CSS patch).
6. **User Manual / help** — the new Style-Setter controls are self-descriptive (UI labels localized ×15); manual has a high-level Style-Setter section, not per-control. Confirm whether Boss wants each new control enumerated.

## NEXT (resume here)
- MIG-088 **§4c** (scrim opacity control) OR **Phase 5** (right-sidebar panels: KH/Provenance/Tasks/Review/Inspector360 badges) — Boss's pick.
- Then the typed-link colours, then Phases 6–10.

---

## Ready-to-paste next-session prompt

Resume Constellation. Last session shipped MIG-088 §3b–§3d (the editor colour sweep — Highlight+Text, Syntax tokens/URL + "Markup marks", Link chip) all Boss-validated, then opened Phase 4 (Chrome): §4a Tab-bar extras + §4b Shadow consolidation, both Boss-validated. All pushed to `main` (tip `c543046c`). Orientation is v3.19.

Before anything:
1. `git pull origin main`.
2. Read `docs/Constellation Orientation & Onboarding v3.19.md` — the "What changed in v3.19" preamble carries the full picture (esp. the CANONICAL LESSON: `CodeMirrorEditor`/`EditorContextMenu`/`FormattingToolbar` are STALE/unmounted — never style them).
3. Skim `docs/handover/Handover-2026-07-01-MIG088-phase3-4.md` + the tail of `lab/reports/SESSION-LOG-2026-06-30.md`.
4. Recall memory `project_stylesetter_add_element_recipe`.

Then pick up MIG-088 (SO #8 cross-check first): either **§4c** — the dialog-scrim opacity control (unify the 4 inconsistent backdrops 0.3/0.35/0.4/0.6 via a `--modal-overlay-alpha`-style OPACITY control, since a colour picker can't express a see-through scrim), or **Phase 5** — the right-sidebar panel badges (KH stage/total cards, Provenance external-source tag, Tasks due badges, Review stale badge, Inspector360 cards/tensions borders, link traversal-chip tiers). Plan: `lab/reports/MIG-088-STYLESETTER-COMPLETENESS-PLAN.md`. Also queued: the 8 typed-link type colours (cross-surface cognitive set); the deferred Arabic callout-caret known-issue (reproduction-driven, no CSS patch). Ultracode.
