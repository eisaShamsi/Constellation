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

### Apply re-test — Boss findings (build `6ea21aae`)

✅ **Apply works** for accent, sidebar background, and text. ❌ Two hold-outs: **editor note background** and **editor note font**. Investigated (Explore agent + direct reads), root cause confirmed — both are baked into the editor, not theme-wired:

- **Note background**: the editor is a deliberate "paper on a desk" (NotePane spec 3.1) — `.e-desk` is hardcoded `#e8e8ec` (NotePane:1183), `.e-paper` hardcoded `#ffffff` (NotePane:1275), `.e-breadcrumb` `#ffffff` (NotePane:1191). No CSS variable, no `.theme-dark` override. So `--background-primary` can't touch it.
- **Note font**: `:global(body)` font-family is `var(--font-interface-theme)` (+layout:7161). NotePane's editor content is `.cm-scroller { fontFamily: 'inherit' }` (NotePane:451) → it inherits the **interface** font, and NotePane never sets `--font-text-theme`. (FocusPane:295 and CodeMirrorEditor:1195 *do* use `--font-text-theme` — NotePane is the parity outlier.) So the Setter's "Note font" (`--font-text-theme`) is ignored by the main editor.

**This is a design decision, not a bug fix** — making the paper/desk/note-font theme-wired touches the core editor surface (Migration-rule territory) and changes a deliberate design. Surfaced to Eisa with a recommendation (make both themeable, aligns with the "style every element" vision + fixes the dark-theme paper gap + the font-parity outlier) vs. keep the paper fixed. Awaiting his call before touching NotePane.

### Decision + fix — note paper/desk/font made themeable (build `bpwsvpafn`)

Eisa chose **"Make both themeable."** Three NotePane.svelte changes, each with the current value as a fallback so the light-theme look is preserved:
- `.e-paper` background `#ffffff` → `var(--background-primary, #ffffff)`, + `font-family: var(--font-text-theme, inherit)` (note content + title now follow the Text font — parity with FocusPane/CodeMirrorEditor).
- `.e-desk` background `#e8e8ec` → `var(--background-secondary, #e8e8ec)`.
- `.e-breadcrumb` background `#ffffff` → `var(--background-primary, #ffffff)`.

Result: the Setter's existing "Note background" (`--background-primary`) and "Note font" (`--font-text-theme`) controls now drive the real editor; the note also finally follows dark themes (was hardcoded white = unreadable light-on-light in dark before). No Setter-side change needed — the var names already matched. Second screen mounts the same NotePane → inherits it (falls back to white paper if it doesn't set the vars → no regression). Behaviour change to note: content/title now use the **Text** font setting, not the Interface font (the correct, consistent behaviour).

Known follow-ups: desk shade in light theme is now `--background-secondary` (a touch lighter than the old `#e8e8ec`); a fuller dark-theme audit of editor decorations is a separate pass.

### Note paper/font re-test — Boss findings (build `bd0a264a`)

- **Note background: ✅ PASS** — the page turns the chosen colour.
- **Note font: partial** — the chosen font reaches the **title + Properties** but NOT the **CodeMirror note content**.
  - First hypothesis (bidiPlugin per-script fonts) **ruled out** by reading Eisa's actual settings (`.constellation/settings.json`): `scriptFonts: {}`, `textFont: ""`, `interfaceFont: ""` — all empty, so bidiPlugin sets no per-line font. (Good thing I verified instead of patching the wrong layer.)
  - **Real root cause**: the font effect in `+layout.svelte` injects a global, **`!important`**, *direct* CM rule `.cm-editor .cm-content { font-family: <resolved-stack> !important }` (lines 1694/1762/1767). The comment at 1760 explains the intent — CSS vars "don't cascade into CodeMirror's scoped styles" so they direct-target with the resolved stack. That `!important` rule beats the inherited `.e-paper` font, so the content ignored `--font-text-theme`. Title + Properties aren't `.cm-content`, so they followed it.
  - **Fix** (build `b847jkxpe`): change those 3 rules to `font-family: var(--font-text-theme, <same-stack>) !important`. Behaviour-identical for normal use (the effect already sets `--font-text-theme` to that exact stack, which becomes the fallback), but the content now **follows** `--font-text-theme`, so the Setter's Note-font control drives it. No regression for per-script `@font-face` users (the var still holds the `"ConstellationText", …` stack).
- **Tab not restyled** — the note's tab in the tab bar didn't follow the look (it's chrome, likely `--background-secondary`); noted for a "chrome/tab" styling pass.

### Style Setter feature requests (Eisa, 2026-06-02) — for the persistence/palette iteration
1. **Saved colour swatches** — when the user picks a colour, save it as a reusable swatch usable on other elements.
2. **Rename swatches** — each saved swatch can be named/renamed.
3. **Many font types** — the final version offers a full font list (current System/Serif/Mono are placeholders), incl. the user's installed + bundled fonts and (per Language-First) per-script fonts.

### Tab styling preference (Eisa, 2026-06-02)
For the **chrome pass**: option **(b)** — the **active tab tints to its note's page colour** (`--background-primary`), so a yellow note gets a yellow-ish active tab; inactive tabs stay with the panel/bar colour.

**Implemented** (build `b31oqxv29`, "Quick wins" — Eisa chose this next): `+layout.svelte` `.tab.active` `background` + `border-bottom` changed from `var(--tab-active-bg, #ffffff)` → `var(--tab-active-bg, var(--background-primary, #ffffff))`. The active tab now defaults to the note page colour and "connects" to it; an explicit `--tab-active-bg` Style-Setting override still wins (non-breaking). Pending Boss test alongside Stage 3 (theme cards + surface previews).

### Quick-wins test — Boss findings (build `04ac4f73`): **All Pass** ✅
Tab tint, theme cards (Midnight/Daylight/Chocolate/Nord), and surface switching all confirmed.

## Style Setter — iteration-2 remarks (Eisa, 2026-06-02)

Four directions captured for the next phase (alongside the already-ratified persistence/swatches/i18n/per-Universe roadmap):

1. **Relocate the note summary** *(editor-layout fix, not strictly Setter)* — the note's summary/excerpt currently renders in the full-width **top strip** (above the breadcrumb, outside the page). Move it to render **under the note title, above the Properties block, within the page borders.** Touches `NotePane` rendering.
2. **Setter chrome adapts to the theme** — the Style Setter's own dark frame (left rail / top bar / right rail; fixed `--c-*` chrome vars in `StyleSetter.svelte`) should **follow the selected theme**, not stay dark around a light preview (image 2: Chocolate draft, dark Setter). Need: decide adapt-to-draft vs adapt-to-app-theme.
3. **Editor: every Markdown/CSS element editable, each with its own character** *(the core "change every element" vision)* — add a **font/text colour** control, and expand beyond title/content-font/wikilink/callout to **all** elements: **Headers (H1–H6)**, bold, italic, inline code, code blocks, blockquotes, lists, tables, HR, etc. — each with colour/font/size/weight controls. Maps to the app's existing Style-Settings vars (`constellationStyleSettings.ts`); **design-first** (element→var taxonomy + preview rows).
4. **Each core plugin replicates its real form/shape** — the surface previews (Sky View, OrgChart, Index, Cataloger, Shell) should **look like the actual plugin**, not generic placeholders (Sky View = real bubble layout, etc.). Ratified "static representative samples", now made faithful. **Design-first** per surface.

Sizing: #1 focused editor fix · #2 moderate Setter-chrome · #3 large (design-first, the vision centrepiece) · #4 large (design-first, per-surface). Everything to date is committed/pushed (`04ac4f73`) + documented (orientation v2.49) — clean handoff point.

### Iteration-2 #1 + #2 implemented (Eisa chose "Quick fixes first"; build `b0inwq4wp`)

**#1 — note summary relocated.** The NSC headline (MIG-043) moved from the full-width `.ne-summary-band` strip (in `NoteEditor`, above NotePane, outside the page) to **inside the page, under the title, above Properties**. `NotePane` gains a `summaryHeadline` prop → renders `<div class="e-summary" dir="auto">` after `.e-title`, before the Properties block (muted italic, pulled up under the title, respects the `.e-paper` 48px padding so it's within the page). `NoteEditor` passes `summaryHeadline={activeHeadline}` and the old band + its CSS are removed. (Second screen passes nothing → prop defaults `''` → no summary, no regression.)

**#2 — Setter chrome adapts to the theme.** `StyleSetter.svelte`'s fixed dark chrome vars now map to the theme: `--c-bg→var(--background-primary)`, `--c-surface→var(--background-secondary)`, `--c-surface2→var(--background-modifier-hover)`, `--c-text→var(--text-normal)`, `--c-muted→var(--text-muted)`, `--c-border→var(--background-modifier-border)`, `--c-accent→var(--interactive-accent)` (each with the original dark value as fallback); `.ss-center` backdrop → `var(--background-secondary)`. Because `.ss` carries the **draft** (`style={draftStyle}`) and otherwise inherits the app theme, the whole studio now follows the look being edited (pick *Chocolate* → the frame goes chocolate), starting from the app's current theme. Pending Boss test.

---

## ✅ MILESTONE — Style Setter iteration 1 complete end-to-end (Boss-validated 2026-06-02)

The note body font re-test **passed** (whole note — title, body, callouts, typed-link badges, wikilinks — consistently Mono on the cream page). **Every element the user clicks in the Style Setter now restyles the real Constellation on Apply:** accent, sidebar background, text, note background, note font. The standalone Setter opens with no freeze, edits live, and applies to the real app. Orientation bumped to **v2.49** in the same pass.

### State-of-standing (SO #5)

**(a) Verified-shipped & protected** (committed + pushed + Boss-validated):
- Standalone Style Setter iteration 1 — `StyleSetter.svelte` + `styleSetter.ts`, Settings entry, top-level mount. Click-to-edit → live preview → Apply. Commits `cab575ce` → `11605aeb`.
- Note editor made themeable — `NotePane` paper/desk/breadcrumb + content font; `+layout` content-font var fix. Boss-validated.
- Old `StylePresetsPanel` retained + working (un-break `b561bafe`); freeze-prone retrofit abandoned per LL-014.

**(b) In-flight / by-design-incomplete:** Apply is **session-only** (direct DOM; reset by the theme effect on a settings change) — durable persistence is the next phase, not a bug. Nothing uncommitted.

**(c) Known-broken:** none in the Setter.

**(d) Pending — not started:** persistence (save named Style + reusable/renameable colour swatches, export/import) · per-Universe apply scope · Stage-3 breadth verify (theme cards + surfaces) · tab tint (b) + chrome · full font list · full i18n (15 locales — Setter is English-only) · retire old Appearance theming + MIG-069 Presets at parity.

**(e) Doc drift:** help files + User Manual not yet updated (deferred to persistence ship, per Testing-Instructions/SO #2 — the Setter is mid-iteration); orientation **body** §4 Appearance section to update at ship (the v2.49 **preamble** covers it for now).

---

### Iter2 #1 + #2 Boss test — **All Passed** ✅ (build `118ff93b`)

### Tab library-label tweak (Eisa, build `b3hchhqme`)
The library name above each tab title (`.tab-lib-name`, `+layout.svelte`) **enlarged** (`font-size 0.55rem → 0.72rem`) and **lifted 2px** (`bottom: calc(100% + 4px) → calc(100% + 6px)`). "Enlarged by 1x" read as a clear, tasteful bump (still ≤ the tab title's 0.8rem); offered to Eisa to calibrate bigger/smaller. Pending test.

**Follow-up (build `bfy5ybwz6`):** the enlargement + lift made the label **touch the top border** (the `.tab-scroll` `overflow` clips above its `12px` top padding). Fix: raised `.tab-scroll` `padding-top 12px → 18px` to make headroom — keeping Eisa's enlargement (`0.72rem`) and lift (`6px`) intact rather than shrinking them. Strip is ~6px taller to hold the bigger labels; offered to trim size/lift if the height feels off. **Boss test: Pass.**

---

## PCS + Orientation + Handover — session close-out

Eisa: *"Prepare the handover files and prompt, but after PCS + Orientation."*

- **PCS** — code committed/pushed throughout (`cab575ce` … `06c7af97`). **Style Setter help shipped:** new "Style Setter" section in the Appearance help topic (`help.uConstellation.World`) + a User Manual entry; **localized to all 15 languages** (English written here; 14 translations added by 4 parallel background agents grouped RTL / European / RU-TR-HI / CJK — each reads the English source + matches the target file's conventions + keeps English UI button names in quotes since the Setter UI is still English).
- **Orientation** — bumped **v2.49 → v2.50** (quick wins + iter-2 #1/#2 + label tweak + the PCS docs).
- **Handover** — `docs/HANDOVER-MIG-070-iteration-2.md` (function-in-hand, what's shipped + protected, hard-won gotchas, **#3/#4 design-first notes**, process constraints, and a ready-to-paste next-session prompt). MoCh `docs/MoCh/MoCh-2026-06-02-1145.md`.

**Final state:** Style Setter iteration 1 + quick wins (tab tint, Stage-3) + iteration-2 #1 (summary) / #2 (chrome adapts) + tab-label tweak — **all shipped, Boss-validated, committed, pushed, documented (v2.50)**. **Next (design-first):** #3 every-Markdown-element editing · #4 faithful per-plugin previews · then persistence (named Styles + swatches) · per-Universe scope · full font list · Setter UI i18n · retire old at parity.

---

## MIG-070 §3 — every Markdown element editable (iteration 2)

**Design-first (shown to Eisa before code).** Verified the ground truth against `livePreview.ts`'s `livePreviewTheme` (the editor element styling, wired into the real note editor at `NotePane.svelte:383/801`): which catalog vars are actually *consumed* vs dead. Findings: heading sizes (`--h1..h6-size`), `--heading-weight`, inline-code bg/text/font/size, link colour/decoration are **live**; bold/italic/strike are **hardcoded** (no colour var); H6 colour = `--text-muted`; `--blockquote-border-color` is **dead** (consumed nowhere — callouts own their bar via `calloutPlugin.ts` line decos, not `.cm-md-blockquote`, which is an inline mark = colour only); markdown pipe-tables are **not** rendered as styled tables (plain text + toolbar). Eisa decisions: **Full** per-element colour; **build** the extra table/list rendering too.

### §3A — per-element colour/size + clickable preview *(this build; pending Boss test)*
- **`livePreview.ts` `livePreviewTheme`** — additive, fallback-preserving (unset = today's look, zero regression):
  - `.cm-md-heading1..5` gain `color: var(--hN-color, inherit)`; `.cm-md-heading6` → `var(--h6-color, var(--text-muted))`.
  - `.cm-md-bold` → `font-weight: var(--bold-weight, 700); color: var(--bold-color, inherit)`.
  - `.cm-md-italic` / `.cm-md-strikethrough` gain `color: var(--italic-color|--strikethrough-color, inherit)`.
  - `.cm-md-blockquote` → `color: var(--blockquote-text-color, var(--text-muted))`.
- **`StyleSetter.svelte`** — new `range` control type (size px / weight); `ELEMENTS` expanded to H1–H6 (own colour+size, shared weight), Bold (colour+weight), Italic, Strikethrough, Inline code (bg/text/size), Blockquote (text colour), Body text (+ size); centre preview rebuilt as a richer **mini-note** rendering all of them, each clickable. Apply still writes to `<body>` (per-element vars flow through automatically).
- **Build** `npm run tauri build -- --no-bundle` → OK (frontend clean; only pre-existing Rust dead-code warnings). Binary mtime `2026-06-02 22:00:23` (Stage-0 fresh). Commit: pending.
- **Deferred to §3C (needs new editor decorations — cross-check Obsidian/CM6 first per WA#5):** blockquote coloured **left bar** (line deco, exclude callouts), **list-marker** styling, **markdown table rendering** + its controls. Docs (help + User Manual, 15 langs) to land with §3 PCS once §3A+§3C validated.

### §3A-fix — "only code() changed; Apply doesn't work" *(Boss test 1 → root-caused → rebuilt)*
**Symptom (Eisa):** in the real note, only inline `code()` (and callouts) responded; headings/bold/italic didn't, and Apply seemed dead. **Root cause (diagnosed, not guessed):** a *second* styling layer I'd missed — `NotePane.svelte`'s `markdownHighlightStyle` (a CodeMirror `HighlightStyle`) **hardcodes** `tags.heading1..6 #d73a49` + weight, `tags.strong #e36209`, `tags.emphasis #7c3aed`, `tags.monospace #16a34a`, `tags.link #2563eb`. That highlight layer is applied to the syntax token and **wins** over the `livePreviewTheme` rule that read my new vars — so the colour vars were silently overridden. Inline-code **background** lives in `.cm-md-code` (livePreviewTheme, no highlight competitor) → it alone responded, which is exactly the reported tell. Same class as the content-font `!important` override the handover flagged.
**Fix:** `markdownHighlightStyle` now reads the vars (`var(--h1-color, #d73a49)`, `var(--heading-weight, 700/600)`, `var(--bold-color, #e36209)`, `var(--italic-color, #7c3aed)`, `var(--strikethrough-color, inherit)`, `var(--code-normal, #16a34a)`, `var(--link-color, #2563eb)`) — original values as fallbacks ⇒ **zero regression when unset**. Removed the now-dead colour/weight rules from `livePreviewTheme` (headings keep SIZE; `.cm-md-bold` keeps the WEIGHT var since the highlight `strong` sets none; blockquote text colour stays — no highlight competitor). Setter preview fallbacks updated to the same defaults so the mini-note looks like a real note pre-edit. Rebuilt OK; binary mtime `2026-06-03 09:39:25`. **Parity follow-up:** `CodeMirrorEditor.svelte` uses `defaultHighlightStyle` (not the markdown one) — separate view, pre-existing; revisit for editor-parity.
**Boss test 1 → Pass.** ✅ Headings/bold/italic/code all respond + Apply reaches the real note.

### §3A — "Interface" core element + note↔chrome decouple *(Boss directive after test 1)*
Eisa: *"focus should be on the Note itself, avoid the interface — the File Tree changes its file colour to match the note body colour. Include the Interface as a core element, at the top of the list."*
**Root of the bleed (verified):** note title (`.e-title`) + body both resolve to the global `--text-normal`, and the file tree reads `var(--ft-master-color, var(--text-normal))` (`FileTree.svelte:191`) — so the note "Text colour" (writing `--text-normal`) recoloured the tree.
**Fix:**
- **Note-scoped text colour** — `NotePane.svelte` `.cm-content` + `.e-title` now read `var(--editor-text-color, var(--text-normal, #1a1a1a))` (new var; fallback = current ⇒ no regression). The Setter "Body text → Text colour" writes `--editor-text-color` → no longer touches the chrome.
- **"Interface" element** added as the **first** `ELEMENTS` entry: Interface text colour (→ global `--text-normal`), Interface font (`--font-interface-theme`), Panel background (`--background-secondary`). Absorbs the old "Sidebar" element; the preview's left sidebar now selects **Interface**.
- **Visible element list** (left rail "Elements" section) with `ELEMENT_ORDER` = Interface first, then Note bg, Body text, Accent, Link, H1–H6, Bold, Italic, Strike, Code, Blockquote — clicking a row selects it (parity with clicking the preview). Discoverability for the now-16 elements.
- Setter preview note text (`.ss-main`/`.ss-title`/`.ss-body`) reads `--editor-text-color`; `.ss-side` stays `--text-normal` — so the preview demonstrates the decoupling live.
- Cascade model: `--text-normal` is the global default; the note overrides via `--editor-text-color`. Set note colour → note only; set Interface colour → chrome (+ note only if its own colour unset). Rebuilt OK; binary mtime `2026-06-03 10:09:51`.
**Boss test 2 → Pass.** ✅ (screenshot: headings recoloured, file tree no longer follows note text.)

## MIG-070 §3B — full interface styling (9 elements, full property set)

After test 2, Eisa marked the 10 interface regions on a screenshot and asked to **name each** + **identify the sub-elements** to cover. Named (verified in code): **#1** Window title bar (OS-native, `tauri.conf` decorations — NOT CSS-styleable), **#2** Ribbon dock (`.dock-btn`, `dock-*`), **#3** Sidebar "New" toolbar (`.sidebar-toolbar > .toolbar-actions .tb-btn`, `sidebar-toolbar-*`), **#4** Top bar = tab strip + pane toggles (`topbar-*`/`tab-*`/`layout-*`), **#5** Sidebar view & sort tabs (`.toolbar-modes .mode-tab` — no vars yet), **#6** File tree (`FileTree.svelte`, `ft-master-*`), **#7** Editor/note (done §3A), **#8** Settings button (a `.dock-btn`, part of #2), **#9** Universe switcher (`.sidebar-footer` → LibrarySwitcher — no vars yet), **#10** Status bar (`.status-bar/.sb-*`, `statusbar-*`).
**Eisa decisions:** **Full set** per element (background · font family/size/weight/colour · border width/style/colour · radius · icon size/colour · spacing); **leave #1 out** (OS chrome). Build in **milestones** (A Sidebar · B Top/bottom bars · C Dock), preview grows into a faithful mini-app (also delivers #4). New props (border-style, per-element fonts) additive w/ current fallbacks.

### §3B Milestone A.1 — File tree (#6) full set *(pending Boss test)*
- **`FileTree.svelte`** — `.folder`/`.note` gain `font-family: var(--ft-master-font-family, inherit)`, `border-radius: var(--ft-row-radius, 3px)` (was hardcoded), opt-in separators `border-bottom: var(--ft-border-width, 0px) var(--ft-border-style, solid) var(--ft-border-color, var(--background-modifier-border))`. Existing `--ft-master-color/-font-size/-weight/-row-padding-y` confirmed live. Unset = unchanged.
- **`StyleSetter.svelte`** — `BORDER_STYLE` select; new **File tree** element (text colour, font, size, weight, row spacing, row radius, separator width/style/colour); added to `ELEMENT_ORDER` after Interface; preview file rows now clickable → File tree + read the `--ft-*` vars (live). Background noted as the Interface panel bg (shared, not duplicated — Form-Aligns-To-Purpose).
- **Build:** lock conflict first (running test instance held the exe — `Access denied`; stopped it, rebuilt). Binary mtime `2026-06-03 11:13:16`. Next: #3 toolbar, #5 view tabs, #9 universe footer → then Milestone A Boss test.
- **Boss test → Pass.** ✅ File-tree styling applies live + on Apply; note area untouched.

### §3B refinement — Strikethrough styles the LINE, not the text *(Eisa)*
Eisa: *"Strikethrough should affect only the line — change the line colour or thickness, not the font."* Repointed `--strikethrough-color` from the text `color` to **`text-decoration-color`**, and added **`--strikethrough-thickness`** (→ `text-decoration-thickness`).
- `NotePane.svelte` `tags.strikethrough`: `textDecorationColor: var(--strikethrough-color, currentColor)` + `textDecorationThickness: var(--strikethrough-thickness, auto)` (removed the `color` override).
- `livePreview.ts` `.cm-md-strikethrough`: same two decoration vars added (keeps `line-through` + `opacity .6`).
- `StyleSetter.svelte`: Strikethrough element now has **Line colour** + **Line thickness** (1–6px); preview `.ss-strike` strikes the line via `text-decoration-color`/`-thickness`, text colour untouched. Fallbacks (currentColor / auto) = no regression. Binary mtime `2026-06-03 11:33:58`.
**Boss test → Pass.** ✅

### §3B fix — note text colour now fully independent of the chrome *(Eisa: "changed file tree colour, note body changed too")*
**Investigation (verified, not guessed):** grepped every consumer of `--ft-master-color` (FileTree `.folder/.note`, StyleSetter, `+layout` `.library-header/.universe-notes-item/.child-universe-item`) — **all sidebar**, none in the note. The note body (`.cm-content`) reads `var(--editor-text-color, var(--text-normal))`. So there is **no direct file-tree→note link in code**. The real fragility: `--editor-text-color` was *unset* at runtime, so the note **fell back to the global `--text-normal`** — meaning any chrome text-colour control that writes `--text-normal` (the Interface element, or any global text change) bled into the note body.
**Fix:** `deriveThemeVariables` (store.ts:3100) now emits **`--editor-text-color: text`** (= the theme's text colour) into the body var map (`+layout:1592` applies it). The note now has its **own** value, not a fallback — so chrome text-colour changes can no longer reach it. The Setter "Body text → Text colour" overrides it; theme switches reset it (consistent with every other Setter var, which is session-only). Zero regression (default = theme text = what `--text-normal` gave). Binary mtime `2026-06-03 12:04:16`.
**Boss test (3-way):** Step 2 (file tree) Pass ✅ · Step 3 (note body) Pass ✅ · Step 1 (interface) **"Not exactly — check the note tab + breadcrumb."**

### §3B fix-2 — note tab + breadcrumb follow the NOTE, not the chrome *(Eisa, Step 1)*
Setting **Interface** text colour (→ `--text-normal`) turned the **active note tab** and the **breadcrumb note name** green too, because both read the global `--text-normal`. They identify the open note, so they should track the note.
- `NotePane.svelte` `.e-bc-note` → `color: var(--editor-text-color, var(--text-normal))`.
- `+layout.svelte` `.tab.active/.focused` → `color: var(--tab-active-color, var(--editor-text-color, var(--text)))` — a dedicated tab colour (Milestone B #4) still wins; default now = the note's colour.
Now: Interface text → chrome only (tab + breadcrumb stay); Note text → note body + its tab + breadcrumb. Binary mtime `2026-06-03 12:28:15`.

### §3B fix-3 — REVERSED: tab + breadcrumb + library-label follow the INTERFACE (Eisa clarified)
fix-2 tied them to the note; Eisa clarified the opposite: *"The note tab text, the library title text above it, and the breadcrumb should follow the universal interface text styles."* Also clarified: *"'handle the note' = the note content (minus the summary and properties); I want to handle them [summary, properties] separately, with their own elements within the editor."*
- `+layout` `.tab.active/.focused` → `color: var(--tab-active-color, var(--text-normal, var(--text)))` (interface; dedicated tab colour still wins).
- `+layout` `.tab-lib-name` → `color: var(--text-normal, var(--text))` (was `var(--text)` → it had no real colour and **inherited the tab's**, which is why it moved with the note once the tab was note-tied; now explicit interface).
- `NotePane` `.e-bc-note` → `color: var(--text-normal)` (reverted from editor-text-color).
- Note **title + body** stay on `--editor-text-color` (the note). Binary mtime `2026-06-03 13:54:13`.
- **Queued (Eisa's refined scope):** Editor sub-elements **Note summary** + **Properties** (own styles, separate from content); Interface sub-elements **cUniverse / Library / Folder** rows (own styles, split from the single File-tree element).
**Boss test:** breadcrumb note name + tab + lib-label now follow interface ✅, BUT (a) breadcrumb **library name + "/" separator** still didn't follow; (b) **"missing the Libraries and folders title color"** (no per-type control). Point 2 Pass.

### §3B G1 — breadcrumb completion + per-row-type file-tree elements (Library / Folder / cUniverse)
- **Breadcrumb completed:** `NotePane` `.e-bc-lib` + `.e-bc-sep` → `var(--text-normal)` (were `--text-muted` / `--background-modifier-border-focus`). Whole breadcrumb now follows the Interface text colour.
- **Per-row-type split (G1):** each sidebar row type now reads `--ft-{type}-*` overriding the File-tree master `--ft-master-*` (→ today's default), wired:
  - **Folder** (`FileTree .folder`) → `--ft-folder-color/-font-family/-font-size/-weight`.
  - **Library** (`+layout .library-header` + `.universe-notes-item`) → `--ft-library-*`.
  - **cUniverse** (`+layout .child-universe-item`) → `--ft-cuniverse-*`.
- **Setter:** new **Library**, **Folder**, **cUniverse** elements (colour / font / size / weight) after File tree in `ELEMENT_ORDER`; preview sidebar gained a 📚 library header, 📁 folder, and ✦ cUniverse row (clickable, reading their vars). Unset = follows the File-tree master = no regression. Binary mtime `2026-06-03 16:49:57`.
- **Still queued:** Editor **Note summary** + **Properties** elements (G2).
**Boss test → All pass.** ✅ (breadcrumb whole + Library/Folder/cUniverse per-type colours.)

### §3B Milestone B (partial) — Status bar (#10) + Universe switcher (#9) *(Eisa: "what about the Status bar and the Universe management text?")*
Both already wired; exposed them + closed two gaps:
- **Status bar** element — `--statusbar-bg/-color/-font-size/-height` (all live in `.status-bar`). Fixed `.sb-universe` (status-bar universe name) `color: var(--text-secondary)` → `var(--statusbar-color, var(--text-secondary))` so the whole bar's text (incl. the universe name) follows the control.
- **Universe bar** element — the "◊ Universe" switcher at the sidebar foot (`.sidebar-footer`). Gave it `color: var(--universe-bar-color, var(--text-normal))`, `background: var(--universe-bar-bg, none)`, `font-family: var(--universe-bar-font-family, inherit)` (colour cascades to the chevron + label). New element: text colour + background + font.
- **Preview:** the mini-window grew a **status-bar strip** (grid row, bottom, full-width) + a **◇ Universe footer** at the sidebar foot — both clickable, reading their vars. `.ss-prev` now `grid-template-areas: "side main" / "status status"`.
- `ELEMENT_ORDER`: + `universe`, `statusbar` after `cuniverse`. Binary mtime `2026-06-03 17:08:03`. Remaining: #2 dock, #3 sidebar "New" toolbar, #4 top bar/tabs (Milestone B/C), + Editor **summary**/**properties** (G2).
