# Session Log — 2026-06-02

**Function in hand:** the **Constellation Style Setter (CSS)** — MIG-070, a standalone full-page "design studio" where the user clicks any part of the live interface and restyles it, then Applies the look to the whole app. Built **from scratch** on a clean slate; touches none of the old style code.

---

## Context — why a clean slate

The prior approach (retrofit a unified Style list onto the MIG-069 Style-Presets panel) **froze the app** four times running (§A v1 `$derived`, the defensive fix, §A v2 imperative, and the from-scratch `StyleSetter` light-rows attempt). Bisection isolated the freeze to anything calling `unifiedStyleList` / `themeToStyle` over `BUILTIN_THEMES`; root cause never definitively reproduced (release devtools off). Per LL-014 (don't patch a bug past 3 tries — find the root cause or change approach) and Eisa's directive, the retrofit was **abandoned**. The working `StylePresetsPanel` was restored (un-break commit `b561bafe`) so the app stayed usable.

Eisa's directive (verbatim): *"start on a clean slate. Don't even touch any previous style functions or any style that we created before… build a standalone Style Setter… it will be the single source of all styles/themes functions."* Then, after an interactive HTML mockup: *"YES! YES. Of course, it feels right. Proceed."*

---

## §A — Standalone Style Setter, iteration 1 (build)

**Approved design (mockup `docs/Style-Setter-Mockup.html`):** full-page three-zone studio —
- **Top bar:** draft name · Reset · Apply to app · ✕
- **Left rail:** Surfaces (Editor / Sky View / OrgChart / Index / Cataloger / Shell) + My themes (swatch cards)
- **Centre:** a live mini-interface preview; click any part to select it
- **Right rail:** controls for the selected element; edits update the preview instantly

**New files**
- `src/lib/stores/styleSetter.ts` — tiny visibility store (`styleSetterOpen`, `openStyleSetter`, `closeStyleSetter`). Independent of all style code.
- `src/lib/components/StyleSetter.svelte` — the full-page Setter. Renders **one** preview (never a gallery of heavy cards — that was the freeze shape). Edits write to a **draft** (CSS-var overrides scoped to the preview wrapper via `style={draftStyle}`); **Apply** copies the draft onto the real `:root` with direct `setProperty` (no reactive path). Draft is a small `$state` `Record<string,string>` — no big objects, no `unifiedStyleList`.

**Wiring**
- `+layout.svelte` — import + top-level `<StyleSetter />` mount (self-shows via the store).
- `SettingsModal.svelte` — "✦ Open Style Setter" entry button in Appearance, above the (retained) old Styles panel.

**Real app variables driven (iteration 1):** `--interactive-accent`, `--background-primary`, `--background-secondary`, `--text-normal`, `--link-color`, `--font-interface-theme`, `--font-text-theme`. Element→controls map: Accent · Note (bg + note font) · Sidebar (bg + interface font) · Text · Link.

**Mid-build fix:** the `fonts` element had controls but no preview element selected it → fonts were unreachable. Merged the font controls into Sidebar (interface font) and Note (note font) so they're reachable by clicking those parts. Killed the first build, rebuilt once with the fix.

**Verification**
- `svelte-check`: 0 new errors in the Setter / store / wiring. (Pre-existing errors in `store.ts` `fresh` lifecycle + `PropertyEditor.svelte` are unrelated and non-blocking.)
- Release build (`--no-bundle`): _in progress_ (task `bs2qei4pa`). Stage-0 mtime verify + Boss test to follow.

**Scope boundary (iteration 1):** colours + fonts + Apply-to-session + surface previews + theme seeding. **Next:** persistence (save / name / export / import as a real theme), per-Universe apply scope, full localization (15 locales), more controls per surface, retire the old Appearance theming once parity is reached.

**Commit:** _pending build pass._

**Open items**
- Other surfaces (Sky/Org/Index/Cataloger/Shell) are representative previews, not the real components — fine for iteration 1; richer static samples later.
- a11y: clickable preview spans emit Svelte a11y warnings (non-blocking) — convert to role/keyboard handlers in a polish pass.
- Apply currently sets vars on `:root` for the session; it does not yet persist or scope per-Universe (Eisa's ratified "each Universe remembers its own look" lands with persistence).

### Stage 1 test — Boss findings (build `cab575ce`)

**The headline:** ✅ **it opens with no freeze, and live editing works.** The clean-slate rebuild beat the freeze that killed the retrofit 4×. Steps 1, 2, 4 pass; click-to-select + instant recolour pass.

Three polish fixes (commit pending, build `bo6x3wny2`):
1. **Hover affordance invisible** — the preview's `overflow:hidden` clipped the outset `outline` on the edge-touching sidebar/note. Fix: inset `box-shadow` ring (`#9d8dff` hover / `#b9acff` selected) for big elements; outset dashed outline kept for inline elements (title/heading/link/pill), which sit inside the note padding and aren't clipped.
2. **Font dropdown options low-contrast** (Serif/Mono nearly invisible) — `<option>`s had no explicit colours. Fix: `select` + `option` set to opaque `#1d1d2a` bg / `#e8e9f3` text.
3. **Esc closed the whole Settings panel** — the Setter's Escape handler let the event bubble to the Settings modal. Fix: listen in the **capture** phase + `stopImmediatePropagation()` (guarded by `get(styleSetterOpen)`), so Escape closes only the Setter and is a no-op when it's shut.

Stage 2 (full tour: all elements, fonts, theme cards, surface switching, Apply-to-app) to follow on the fixed build.

### Stage 2 test — Boss findings (build `fbfbc9a0`)

All editing passed: hover ring now visible (✓ image), font list readable (✓ image), and styling each element live works — **Text, Accent, Link, Note bg + Note font all pass** (steps 1–6, 9). Esc closes only the Setter (✓).

**Failed: steps 7–8 — "Apply to app" had no effect.** Root cause found by reading the actual theme path (not guessing):
- Constellation applies its theme variables to **`document.body.style`** (`+layout.svelte:1591` `const root = document.body.style`), not `:root`. My Apply wrote to `document.documentElement` (`:root`); since `<body>` is a descendant, its values **shadow** `:root`, so Apply was silently overridden.
- The accent is consumed two ways: `deriveThemeVariables` (`store.ts:3100`) sets `--interactive-accent` **and** `--accent-h/s/l` + `--text-accent`. Writing only `--interactive-accent` misses every control that composes from the HSL parts.

**Fix (build `b8q0ioqoy`):**
1. `apply()` now writes to `document.body.style`.
2. Accent is decomposed (inlined `hexToHSL`, Setter stays standalone) → also sets `--accent-h/s/l`, `--text-accent`, `--interactive-accent-hover`.
3. `curVal()` reads the live value from `<body>` (where the vars live) so swatches seed from the real theme, not grey.
4. Verified my var names already match the app's (`--background-primary/-secondary`, `--text-normal`, `--interactive-accent`, `--font-interface-theme/-text-theme`) — the preview already speaks the app's language; only the Apply *target* was wrong.

Re-test of steps 7–8 to follow.
