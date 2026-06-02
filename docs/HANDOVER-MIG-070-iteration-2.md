# Handover — MIG-070 Constellation Style Setter, iteration 2

**For the next session.** Everything below is committed + pushed on `main`. Read **`docs/Constellation Orientation & Onboarding v2.50.md`** first (the §"What changed in v2.50" + v2.49 preambles cover this feature in full), then this file, then the memory `project_style_setter_feature_requests.md`.

---

## Function in hand
**The Constellation Style Setter (CSS)** — the standalone full-page "design studio" (`Settings → Appearance → "✦ Open Style Setter"`). Iteration 1 + quick wins + iteration-2 #1/#2 are **shipped + Boss-validated**. Iteration 2 continues with **#3 (every Markdown element editable)** and **#4 (faithful plugin previews)**.

## What's shipped (do NOT re-do; protected)
- **The Setter** — `src/lib/components/StyleSetter.svelte` (+ store `src/lib/stores/styleSetter.ts`, entry button in `SettingsModal.svelte`, top-level mount in `+layout.svelte`). Click any preview element → its controls (right) → live update → **Apply to app**. Drives real theme vars: `--interactive-accent` (+ decomposed `--accent-h/s/l`/`--text-accent`), `--background-primary`/`-secondary`, `--text-normal`, `--link-color`, `--font-interface-theme`/`-text-theme`.
- **Themeable note editor** — `NotePane.svelte`: `.e-paper`/`.e-desk`/`.e-breadcrumb` backgrounds + content font wired to theme vars (was hardcoded "paper on a desk"); note now follows dark themes.
- **Content-font fix** — `+layout.svelte` 1694/1762/1767: `.cm-editor .cm-content { font-family: var(--font-text-theme, <stack>) !important }` (was a hardcoded `!important` stack that overrode the Setter).
- **Quick wins** — active-tab tint (`.tab.active` bg + bottom → `var(--tab-active-bg, var(--background-primary,…))`); summary in-page (`NotePane` `summaryHeadline` prop → `.e-summary`, `NoteEditor` passes `activeHeadline`); Setter chrome adapts (`.ss --c-*` → theme vars); tab library label `0.72rem` + `.tab-scroll` padding-top `18px`.
- **Docs** — Style Setter help in `help.uConstellation.World/Appearance and Themes/` + `User Manual.md`, localized to all 15 languages; orientation v2.50; session log `lab/reports/SESSION-LOG-2026-06-02.md`; MoCh `docs/MoCh/MoCh-2026-06-02-1145.md`.

## Hard-won gotchas (read before coding)
1. **The app themes `document.body`, NOT `:root`** (`+layout.svelte:1591`). Apply targets `document.body.style`.
2. **Stay standalone.** The earlier retrofit onto MIG-069's `unifiedStyleList`/`themeToStyle` over `BUILTIN_THEMES` **froze the app 4×** (LL-014). The Setter touches no existing style code. Render **ONE** preview, a small `$state` draft — never a gallery of heavy cards.
3. **Var model** — `deriveThemeVariables` (`store.ts:3100`) is the source of the theme var names. **Style-Settings catalog** for per-element vars (headers etc.) lives in `src/lib/theme/constellationStyleSettings.ts` — read it for #3.
4. **Editor parity** — note content font is `--font-text-theme` (matches FocusPane/CodeMirrorEditor). Per-script fonts come from `bidiPlugin` only when `scriptFonts[script]` is set.

## Next work (both design-first — write the design before code)

### #3 — every Markdown/CSS element editable (the core vision)
Eisa: *"include all Markdown and CSS elements; besides the title, content font, wikilinks, and callout… the Headers, etc. And… change each one's character."* Plus a **text-colour** control.
- **Design first:** enumerate the elements (H1–H6, bold, italic, inline code, code block, blockquote, lists, tables, HR, …) → map each to its CSS variable(s) from `constellationStyleSettings.ts` + the editor's `livePreview.ts` decorations. Produce an element→control table.
- **Then build:** extend `ELEMENTS` in `StyleSetter.svelte`; make the centre preview a **richer mini-note** that actually renders those elements (so they're clickable); add the controls (colour/font/size/weight) mapping to the real vars; verify Apply reaches each (some, like `.cm-content`, may need the same `var(--…, fallback) !important` treatment as the content font).

### #4 — faithful per-plugin surface previews
Eisa: *"Each Core plugin should replicate its form and/or shape."* The Surfaces previews (Sky View, OrgChart, Index, Cataloger, Shell) are generic placeholders now.
- **Design first:** one accurate static sketch per surface, matching the real plugin's form — Sky View = bubble nodes (PIXI), Constellation Map = sunburst arcs (D3, NOT bubbles — they are distinct), Index = the term list, etc. Look at each real component for its shape. Ratified decision: **static** representative samples (not live), but faithful.

### Then (already-ratified roadmap)
Persistence (save a look as a **named Style** + Eisa's reusable/renameable **colour swatches**, export/import) · per-Universe apply scope ("each Universe remembers its own look") · full font list (System/Serif/Mono are placeholders) · **Setter UI i18n** (15 locales — the Setter is English-only; the 15-language help intentionally keeps English button names until this lands) · retire old Appearance theming + MIG-069 Presets at parity.

## Process constraints (non-negotiable)
One location `E:\مشاريع كلاود\Constellation` on `main`; `git pull` first. Build = `npm run tauri build -- --no-bundle` (~1m40s; binary `src-tauri/target/release/constellation.exe`). **Stage 0:** verify binary mtime before any Boss test. Test instructions are tutorials (define → walk through). Plan-approval = build-approval (cascade, stop only at testable steps or surprises). Commit messages end `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`. Push at boundaries. Never "vault". Devtools are DEV-only (test via observable UI). BASIC RULE: don't make things up — read the code, verify (this session caught two wrong hypotheses by verifying: `:root` vs `body`, bidiPlugin vs the `!important` rule).

---

## ▶ Next-session prompt (paste this)

> Continue **MIG-070 — the Constellation Style Setter**, iteration 2. First `git pull origin main`, then read `docs/Constellation Orientation & Onboarding v2.50.md`, `docs/HANDOVER-MIG-070-iteration-2.md`, and the memory `project_style_setter_feature_requests.md`. The standalone Setter (`src/lib/components/StyleSetter.svelte`) ships live click-to-edit + Apply and is Boss-validated; do not touch the frozen MIG-069 `unifiedStyleList` path, and remember the app themes `document.body` not `:root`.
>
> Two design-first features remain: **#3 — make every Markdown/CSS element editable** (Headers H1–H6, bold, italic, code, quotes, lists, tables, each with colour/font/size, plus a text-colour control — map each to the vars in `src/lib/theme/constellationStyleSettings.ts` and the editor's `livePreview.ts`, and make the Setter's centre preview a richer mini-note that renders them so they're clickable); and **#4 — make the surface previews (Sky View, OrgChart, Index, Cataloger, Shell) faithfully resemble each real plugin** (static representative samples). **Start with #3: produce the element→control→variable design table and show it to Eisa before writing code.** Build with `npm run tauri build -- --no-bundle`, verify the binary mtime before any test, and write every test as a step-by-step tutorial.
