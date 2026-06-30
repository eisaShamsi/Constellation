# Next-session kickoff prompt — paste this to resume

> Ready-to-paste prompt for the session after MIG-089 (Callout Customisation) + the Language-First audit completed. Copy everything in the box.

---

Resume Constellation. We just shipped **MIG-088 §3a (callout colours) + §3b-pre (per-element Reset)**, then **MIG-089 "Callout Customisation"** end-to-end (per-type icons, user-defined `[!trigger]` types, a unified centre-zone Callouts manager, saved-colours, edit/remove, in-manager Reset), then a Boss-requested **app-wide Language-First / bidi audit** fixed in Passes 1–4. All Boss-validated, on `main` (pushed, clean). Orientation is **v3.18**.

Before anything:
1. `git pull origin main` (sync).
2. Read **`docs/Constellation Orientation & Onboarding v3.18.md`** — the "What changed in v3.18" preamble carries the full picture.
3. Skim **`docs/handover/Handover-2026-06-30-MIG089-callout-customisation.md`** (state + what's next) and the **CLOSE-OUT block** at the top of **`lab/reports/SESSION-LOG-2026-06-30.md`**.
4. Recall memory `project_stylesetter_add_element_recipe`.

Then **resume the PAUSED MIG-088 §3b–§3d editor colour sweep** (per `lab/reports/MIG-088-STYLESETTER-COMPLETENESS-PLAN.md`, same add-an-element recipe, SO #8 cross-check first):
- **§3b Highlight (unify-on-demand)** — `<mark>` / markdown `==` / toolbar chip onto one shared bg + radius.
- **§3c Syntax tokens** — frontmatter URL (`#0891b2`) + fence/meta (`#888`) wired **inside** `markdownHighlightStyle.define` (NotePane:57-59 — it wins over the theme).
- **§3d Editor badges** — lens-count `#fff` + decoration radii.

Also queued (do when convenient):
- Re-run **`/simplify`** on the MIG-089 diff (deferred — server rate-limit during Phase C).
- The callout-customisation **help-topic folders ×15** (the User Manual ×15 is already done).
- The deferred **A.2 known-issue** — Arabic callout **End/Home caret in the editor**: a CM6 RTL-caret issue on callout lines; needs a **reproduction-driven structural fix** (likely the `Decoration.replace` × RTL interaction). Do NOT re-apply a CSS patch.

Ultracode.

---
