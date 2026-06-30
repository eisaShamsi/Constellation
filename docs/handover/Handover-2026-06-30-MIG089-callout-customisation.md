# Handover — 2026-06-30 — MIG-089 Callout Customisation + Language-First audit

**Branch:** `main` (pushed, clean). **Orientation:** **v3.18** is the file to read. **Session log:** `lab/reports/SESSION-LOG-2026-06-30.md` (has a CLOSE-OUT / state-of-standing block at the top).

## What shipped this session (all Boss-validated, on `main`)
- **MIG-088 §3a** — callout COLOURS: 10 families restyleable (`--callout-<family>-color`, byte-identical fallback). `d395673f`.
- **MIG-088 §3b-pre** — per-element **Reset** (Boss-discovered footgun: the universal Reset nuked the whole theme). `9c9d412f`.
- **MIG-089 Callout Customisation** (Architect→Plan→Build):
  - **§A** per-type **icons** — reuse the Emoji & Icon Library (`iconOverrides` `callout.<family>` slots, `EmojiIconPicker`, `SlotIcon`).
  - **§B** custom **`[!trigger]` types** — per-Universe `appSettings.customCallouts: {slug,name,color,icon}[]`; `src/lib/theme/customCallouts.ts`; family data in `src/lib/editor/calloutFamilies.ts` (dependency-free). Unicode triggers (`[!فكرة]`), inline `--callout-color` on the CM6 line deco, name-as-bold-header. New `ColorField.svelte` (saved-colours popover) + `IconRef.svelte`.
  - **Unified Callouts manager** (Boss redesign) — one box in the **centre zone**; right rail hidden for the Callouts category (`.ss--norail`); working in-manager Reset (built-in colours + icons).
- **Language-First / bidi audit** (Boss-requested) — fixed app-wide: Pass 1 global `theme.css` input rule (`unicode-bidi: plaintext`) + `dir="auto"` on ~15 inputs (Boss PASS); Pass 2 display `dir="auto"`; Pass 3 `isEditableTarget` keyboard guard; Pass 4 RTL logical CSS + Inspector360 arrow flip.

## Key canonical facts (mental model)
- Callout colours = per-Universe Style Setter vars; built-in icons = per-Universe `iconOverrides`; custom types = per-Universe `appSettings.customCallouts` (colour injected **inline on the CM6 line deco**, NOT body — so BUG-015's single body-var writer is intact).
- Family data: `src/lib/editor/calloutFamilies.ts`. Manager: `CalloutTypesEditor.svelte` (rendered in the Setter centre for `pk==='callouts'`).
- **Language-First input pattern:** the global `theme.css` rule gives every `<input>`/`<textarea>` `unicode-bidi: plaintext`; add `dir="auto"` on user-content fields. Keyboard handlers that `preventDefault` bare keys must guard with `isEditableTarget(e)` (`utils.ts`).

## Deferred / open (honest — not silently parked)
1. **A.2 KNOWN-ISSUE (Boss "stop patching", ruled defer):** the Arabic-callout **End/Home caret *in the editor*** is still wrong. It's a CM6 RTL-caret problem on callout lines; the speculative `isolate→plaintext` override was **reverted**. Plausible (unverified) cause: the callout's `Decoration.replace` ranges (the `>`-prefix hide + title widget) interacting with RTL caret movement → a real fix is **structural**, and needs a **reproduction** (no GUI from the agent side). Do NOT re-apply a CSS patch.
2. **`/simplify`** on the MIG-089 diff — deferred (transient server rate-limit hit during Phase C). Re-run.
3. **Help-TOPIC folders ×15** for callout customisation — the **User Manual ×15 is done**; the topic folders (`docs/help.<lang>/…/`) are a secondary surface, optional.

## Paused before this detour (resume here)
- **MIG-088 §3b** — Highlight (unify-on-demand: `<mark>` / markdown `==` / toolbar chip onto one shared bg+radius).
- **MIG-088 §3c** — Syntax tokens: frontmatter URL (`#0891b2`) + fence/meta (`#888`) wired **inside** `markdownHighlightStyle.define` (NotePane:57-59 — it wins over the theme).
- **MIG-088 §3d** — Editor badges: lens-count `#fff` + decoration radii.
- Then MIG-088 Phases 4–10. Plan: `lab/reports/MIG-088-STYLESETTER-COMPLETENESS-PLAN.md`. Recipe memory: `project_stylesetter_add_element_recipe`.
