# Constellation — Orientation & Onboarding

**Version 2.53 | 2026-06-05**

> **What changed in v2.53** (**MIG-070 §C Phase 5 — link styling folds into the Style Setter: typed-link colours + display toggles + pill shape, all through the existing save paths (shipped; Boss test staged)**):
>
> **A new "Links" category in the Style Setter.** The Setter's left rail gains a **Links** surface with two elements. **Typed-link colours** embeds the existing §G **Link-Types editor** verbatim (`src/lib/components/LinkTypesEditor.svelte`, now with an `embedded` prop that hides its Settings-tab heading/desc) — recolour any of the 8 built-ins, add a custom type, delete a custom, or **reset colours to default**, all written through the one registry save path (`saveLinkTypes` → `.constellation/link-types.json`), so Backlinks, Outgoing, the Knowledge-Health dashboard, and the in-editor links **recolour live** (one colour source — never a second copy). **Link display** carries the two editor toggles — **colour typed links** · **show type labels** — plus the pill **shape** (corner radius · height · label weight), reading/writing `appSettings` (`colourTypedLinks` / `showTypedLinkLabels` / `linkPills.shape`) directly via `updateSettings`, the same immediate-write model the per-script fonts use. The focused centre preview shows the real `LinkTypePill` row (recolours + reshapes live) and an in-note typed link that honours the toggles.
>
> **No new storage model; nothing relocated.** Phase 5 is **additive** — the Setter is a second front-end onto the *existing* registry + appSettings; the old **Settings → Link Types** editor and the **Appearance** pill controls stay live (they retire only at the Phase 9.1 parity gate, per the BUG-015-avoidance discipline). The frozen **MIG-069** link-colours preset path (`applyPreset` merge-by-id) is untouched. Three new Setter control types (`toggle` / `pillrange` / `pillselect`) carry the settings-backed controls. **`note_links.link_type` globally `'relates'`** remains a separate foundational bug — Phase 5 does not assume `link_type` is correctly populated. Per **LL-031**, this orientation bump rides in the same commit as the Phase 5 code (not batched at handover).
>
> **Where the migration stands.** Phase 5 shipped (Boss test staged). Remaining phases: **6** unified Themes+Styles gallery (NEXT) · **7** the 4 no-UI gaps (accent picker · dark/light/system toggle · custom-CSS editor · per-library appearance + its missing apply path) · **8** second-screen full-style sync · Setter localization · **9** retire the old Appearance/Style-Settings tabs at the parity gate (+ deferred Phase-2 catalog parity). Resume from `docs/MIG-070C-HANDOVER.md` → `docs/MIG-070C-PLAN.md`. Session log `lab/reports/SESSION-LOG-2026-06-05.md`.
>
> *(2026-06-05, Stage-1 refinements — Boss-validated colours, then three remarks folded in)* (1) The type **colour boxes are now a fixed pill** — `LinkTypesEditor`'s `.color-input` is self-contained, so it renders identically in Settings and the Setter (the SettingsModal `.color-input` is scoped and never reached the embedded editor → native varying widths; exactly the "self-contained components" lesson). (2) Link colours **save to the universal swatch palette** — the same `styleSwatches` the interface elements use, so a colour you pick is reusable for any element; click a swatch to recolour the **highlighted** type, right-click to remove (auto-saves on pick + on add-type). (3) The focused preview now shows the **in-editor typed-link representation** — the type label stacked above the coloured, underlined link (how a note actually reads), recolouring live next to the sidebar pills and honouring the two display toggles.
>
> *(2026-06-05, Setter layout redesign — Boss test pending)* The Style Setter panel is now **resizable** (drag the bottom-right corner grip; size persists via `localStorage`). The 3-zone layout (left rail · centre preview · right controls) is kept **only for the Editor category** (its rendered note needs the centre); **every other category is now 2-zone** — a left sidebar + one wide right space, no centre. **Links is one integrated surface** (Eisa: *"integrate the two into one, avoid duplication"*): the display controls (colour-typed-links / show-labels toggles + pill radius/height/weight) sit above the §G Link-Types editor, whose **rows each show their LIVE pill** (the control *is* the preview — the pill reflects colour + shape live). This **supersedes the Stage-1 (3) in-editor representation above**: the separate pill-row + in-editor-text blocks are folded away so each colour is shown once (the dead centre branches + their `tlink`/`ltColor`/`previewLinkIds` helpers + `LinkTypePill` import were removed from `StyleSetter`). *(The other non-Editor categories' old chrome/tree/global centre previews remain in code but hidden by the 2-zone grid — pending a `/simplify` sweep once the layout is Boss-confirmed.)*
>
> ---
>
> **What changed in v2.52** (**MIG-070 §C — the styling-unification /migration cascades: Phases 0–4.2 shipped + Boss-validated; handover prepared mid-migration**):
>
> **§C progress (shipped + Boss-validated since v2.51).** The Style Setter is now the working single styling surface for everything built so far. **Persistence spine (Phase 0/1):** a per-Universe **`appSettings.styleOverride`** is merged on top of the theme in the shared `+layout` apply `$effect` (registered in `_lastStyleSettingsKeys` → survives theme switch, clears cleanly); the Setter persists via `mergeStyleOverride` (Apply) / `clearAllStyleOverride` (Reset) and seeds its draft from the saved look on open. **Categories rail** groups the left side into Surfaces (Interface · Components · Editor · Global · Sky View · OrgChart · Index · Cataloger · Shell). **Global category** (backgrounds / text-shades / status / accent shades; type & rhythm; shape & corners) and **Components category** (dock · sidebar toolbar · layout bar · top bar & tabs · right sidebar · buttons · tags & callouts · sidebar shell). **Saved colour swatches** — a per-Universe palette that auto-saves the colour you settle on; click a swatch to apply, right-click to remove. **Focused per-element preview (#4)** — the centre now replicates ONLY the selected element (the rich note for note elements, file-tree rows, the switcher, a status strip, or each chrome widget) instead of an always-on mini-app box. **Fonts (Phase 4.1/4.2)** — 14 curated cross-platform Latin typeface stacks + a Code-font picker, and **language-smart per-script fonts**: an Arabic note renders in its chosen Arabic font even inside an English interface (chrome follows the interface font), via `appSettings.perScriptFonts` feeding the `@font-face` engine through distinct virtual families `CnSetterText`/`CnSetterUI` (the engine is never clobbered by a wholesale `--font-text-theme` override).
>
> **Two decisions locked.** (1) The **interface-language selector stays in Settings → Language** (removed from the Setter — locale is not styling; per-script *fonts* stay in the Setter). (2) **Setter UI localization to all 15 languages happens AFTER the Setter content is final** (translate the ~100-string set once), queued as the penultimate migration step before retire.
>
> **Where the migration stands.** Remaining phases: **5** link colours (NEXT) · **6** unified Themes+Styles gallery · **7** the 4 no-UI gaps (accent picker · dark/light/system toggle · custom-CSS editor · per-library appearance + its missing apply path) · **8** second-screen full-style sync · Setter localization · **9** retire the old Appearance/Style-Settings tabs at a parity gate (+ the deferred Phase-2 catalog parity). **Resume from `docs/MIG-070C-HANDOVER.md`** (self-contained, with a copy-paste prompt) → `docs/MIG-070C-PLAN.md`. State-of-standing: `lab/reports/SESSION-LOG-2026-06-05.md`; MoCh `docs/MoCh/MoCh-2026-06-05-0848.md`. Frozen MIG-069 presets remain untouched; `styleOverride`/`styleSwatches`/`perScriptFonts` are additive + rollback-safe.
>
> *(2026-06-05, post-handover process note) **LL-031** added: the orientation bump rides in the feature commit, **never batched at handover** (SO #6) — recorded after a batching slip across the §C phase commits (v2.51 bumped inline at Phase 0/1, then 8 feature commits shipped with no orientation touch, v2.52 batched at handover). See `docs/LESSONS-LEARNED.md` §LL-031 + `lab/reports/SESSION-LOG-2026-06-05.md`.*
>
> ---
>
> **What changed in v2.51** (**MIG-070 §3 — every interface + Markdown element is editable in the Style Setter (Boss-validated); §C — the "unify all styling into the Style Setter" /migration kicks off**):
>
> **§3 — every element editable (shipped, Boss-validated).** The Style Setter now styles every Markdown element — H1–H6 (own colour + size, shared weight), bold, italic, **strikethrough** (its *line* colour + thickness, not the text), inline code (bg/text/size), blockquote text — and every chrome element: the **file tree** with per-row-type splits (**Library / Folder / cUniverse**, each overriding a File-tree master), **Status bar**, **Universe bar**. The centre preview is a clickable mini-app (sidebar + note + status bar). Two bugs were root-caused in code (not guessed): "only `code()` changed" was NotePane's `markdownHighlightStyle` hardcoding heading/strong/emphasis colours and **winning** over the theme rule (now reads the vars); and note text bled into the chrome because the note fell back to global `--text-normal` — fixed with a note-scoped **`--editor-text-color`** default in `deriveThemeVariables`. The note **tab text + library label + breadcrumb** follow the **interface** text; the note **title + body** follow the note. The left rail is reorganised into **categories (Surfaces)** — Interface · Editor · Sky View · OrgChart · Index · Cataloger · Shell — each grouping its elements.
>
> **§C — unify ALL styling into the Style Setter (formal /migration, underway).** A 3-agent audit (`docs/MIG-070-style-merge-AUDIT.md`) found **THREE** styling surfaces with **three storage models**: the Style Settings catalog (`styleSettingsValues`, per-theme), the Style Setter (session-only body vars), and MIG-069 Style Presets (app-global appSettings bundles). Eisa ratified: persistence = **theme base + per-Universe override**; scope = **everything in one migration**; execution = **formal /migration**. Phase 1 (Architect) + Phase 2 (Plan — 10 phases / ~30 commits, `docs/MIG-070C-PLAN.md`) approved. **Building now:** Phase 0 = a per-Universe **`appSettings.styleOverride`** merged on top of the theme in the shared `+layout` apply `$effect` (registered in `_lastStyleSettingsKeys`, so it survives theme switch + clears cleanly); Phase 1 = the Setter **persists** via that override (`mergeStyleOverride` on Apply, `clearAllStyleOverride` on Reset, seeds its draft from the saved look on open) instead of the old session-only Apply. **Remaining phases:** catalog parity (~17 new vars) · Layout/Components gap controls · fonts · link colours · unified Themes+Styles gallery · the 4 no-UI gaps (accent picker · dark/light toggle · custom-CSS editor · per-library appearance, incl. its missing apply path) · second-screen full-style sync · retire the old Appearance/Style-Settings tabs at parity. Invariants + rollback in AUDIT §5. Session log `lab/reports/SESSION-LOG-2026-06-02.md`; MoCh `docs/MoCh/MoCh-2026-06-03-0900.md`.
>
> ---
>
> **What changed in v2.50** (**MIG-070 iteration-2 quick wins + PCS docs**: tab tint · summary relocation · the Setter wears the theme · tab-label sizing — and the Style Setter help/manual ship in all 15 languages):
>
> **Iteration-1 follow-ups, all Boss-validated.** (a) **Active tab tints to its note's page colour** — `.tab.active` background + bottom edge now `var(--tab-active-bg, var(--background-primary, …))`, so a coloured note gets a matching active tab that "connects" to it (Eisa's choice **(b)**; inactive tabs keep the panel colour; an explicit `--tab-active-bg` override still wins). (b) **Note summary relocated in-page** — the NSC headline moved from the full-width top strip (`NoteEditor`'s `.ne-summary-band`) to **under the note title, above Properties, within the page** (`NotePane` gains a `summaryHeadline` prop → `.e-summary`). (c) **The Style Setter wears the theme** — its chrome `--c-*` vars now map to the app theme (`--background-primary`/`-secondary`, `--text-normal`, `--interactive-accent`, …, dark studio values as fallbacks), and since `.ss` carries the draft, picking *Chocolate* turns the **whole studio** chocolate. (d) **Tab library label** enlarged (`0.55→0.72rem`) + lifted 2px, with `.tab-scroll` top-padding raised `12→18px` so it clears the top border.
>
> **PCS docs.** The **Style Setter help** ships: a new "Style Setter" section in the Appearance help topic + a User Manual entry (open it · click-to-edit · theme cards · surfaces · Apply), localized into **all 15 languages**. (The Setter's *UI* is still English-only; the help keeps English button names in quotes and the explanatory steps localize — only the quoted labels update when the UI is localized.)
>
> **Still pending — MIG-070 (design-first next):** **#3 — every Markdown/CSS element editable** (Headers H1–H6, bold, italic, code, quotes, lists, tables… each colour/font/size, + a text-colour control — the core "change every element" vision; maps to `constellationStyleSettings.ts` vars); **#4 — faithful per-plugin surface previews** (Sky View bubbles, real Index, etc.). Then: persistence (named Styles + reusable/renameable colour swatches, export/import), per-Universe apply scope, full font list, **Setter UI i18n** (15 locales), retire old Appearance theming + MIG-069 Presets at parity. Session log `lab/reports/SESSION-LOG-2026-06-02.md`; MoCh `docs/MoCh/MoCh-2026-06-02-1145.md`; **handover `docs/HANDOVER-MIG-070-iteration-2.md`**.
>
> ---
>
> **What changed in v2.49** (**MIG-070 — the standalone Constellation Style Setter (CSS) ships iteration 1: live click-to-edit + Apply, Boss-validated end-to-end**):
>
> **A full-page "design studio" for the whole interface.** Open it from **Settings → Appearance → "✦ Open Style Setter"** and the screen fills with a three-zone studio (left: surfaces + theme cards · centre: a **live mini-interface preview** · right: contextual controls). **Click any part of the preview** — sidebar, title, heading, link, the note page — and that element's controls appear on the right; edits update the preview **instantly**; **Apply to app** pushes the look onto the real Constellation. Drives the real theme variables: `--interactive-accent` (+ decomposed `--accent-h/s/l` + `--text-accent`), `--background-primary`/`-secondary`, `--text-normal`, `--link-color`, `--font-interface-theme`/`-text-theme`. New files: **`src/lib/components/StyleSetter.svelte`** + **`src/lib/stores/styleSetter.ts`**, mounted top-level in `+layout.svelte`; entry button added to `SettingsModal`. Architect/Plan `docs/MIG-070-constellation-style-setter-{ARCHITECT,PLAN}.md`; approved clickable mockup `docs/Style-Setter-Mockup.html`.
>
> **Clean-slate rebuild after the retrofit froze 4×.** The earlier approach (unify the MIG-069 Style list onto the Presets panel) froze the main thread four times running — anything calling `unifiedStyleList`/`themeToStyle` over `BUILTIN_THEMES`; root cause never reproduced (release devtools off). Per **LL-014** it was **abandoned** and the working `StylePresetsPanel` restored (un-break `b561bafe`). The Setter is a from-scratch, **standalone** world that **touches no existing style code**: ONE preview (never a gallery of heavy cards — that was the freeze shape), a small `$state` draft (CSS-var overrides scoped to the preview via `style={draftStyle}`), Apply = direct `setProperty` on **`document.body`** (the app themes `<body>`, not `:root` — that was why Apply first did nothing). It never froze.
>
> **Two cross-subsystem fixes the editor needed to be styleable** (each Boss-validated, found by reading the code not guessing): **(1)** the note editor was a hardcoded "paper on a desk" — `NotePane.svelte` `.e-paper #fff` / `.e-desk #e8e8ec` / `.e-breadcrumb #fff`, content inheriting the **interface** font — so the Setter's Note-background + Note-font had nothing to grab. Eisa chose "make both themeable": wired to `var(--background-primary/-secondary, …)` + `font-family: var(--font-text-theme, inherit)` (each with the old value as fallback, light look preserved). The note now also follows **dark themes** (was hardcoded white = unreadable light-on-light) and matches FocusPane/CodeMirrorEditor (**Editor Parity**). **(2)** the font effect injected `.cm-editor .cm-content { font-family: <stack> !important }` (`+layout.svelte` 1694/1762/1767) — a deliberate direct CM rule (vars don't cascade into CM's scoped styles) whose `!important` beat the inherited paper font; changed to `var(--font-text-theme, <stack>) !important` — behaviour-identical (the var already holds that stack), but the content now **follows** the Setter. (Verified Eisa's `scriptFonts: {}` first to rule out the bidiPlugin per-script path.)
>
> **Still pending — MIG-070 roadmap (ratified):** persistence — save a look as a **named Style** (+ Eisa's reusable, renameable **colour swatches**), export/import; **per-Universe apply scope** ("each Universe remembers its own look"); Stage-3 breadth (theme cards + the static surface previews); the **full font list** (System/Serif/Mono are placeholders — final pulls installed + bundled + per-script fonts); tab/chrome styling (active tab tints to its note's page colour — Eisa's choice **(b)**); **full i18n** (15 locales — the Setter is English-only so far); then **retire** the old Appearance theming + MIG-069 Style Presets once parity is reached (the ratified "unify Themes + Styles into one 'Style'"). **Apply is currently session-only** (direct DOM, reset by the theme effect on a settings change) — durable per-Universe persistence lands with the save phase. Session log `lab/reports/SESSION-LOG-2026-06-02.md`. Help + User Manual land when persistence ships (the Setter is mid-iteration; body §4 Appearance section to update then).
>
> ---
>
> **What changed in v2.48** (**MIG-067 §E/§G/§H — the Living Vocabulary becomes visible, speaks the note's language, and renders from ONE pill source**; **MIG-069 — Style Presets ship as preview cards**):
>
> **Typed links now SHOW their type, in colour, everywhere.** The 8 typed acts had stored correctly since v2.47 but rendered "all blue" — §E lands the colour: each typed link draws in its **registry colour** (the §G editor is the one source) and carries a small **type LABEL above it** in the editor. Two Settings switches gate the look — **colour typed links** · **show the label above** (both ON by default; `appSettings.colourTypedLinks` / `showTypedLinkLabels`). §G ships the **Settings → Link Types editor** (the headline feature): recolour a type and it reflects **live** everywhere — the editor links AND the Backlinks/Outgoing pills — plus a **Reset colours to default**. The registry is the single colour source; a recolour is one write every surface reads (the `livePreview` plugin subscribes to the vocabulary and dispatches a `linkVocabChanged` effect to rebuild decorations live).
>
> **Labels + pills read in the NOTE's main language, not the UI's** (§H — Eisa's rule). An Arabic note shows `يدعم` / `مشتق من`; an English note shows `supports` / `derives-from` — whatever the interface language. Detection is `dominantLocale(text)` (script→locale) → `tIn(loc, …)` (translate-in-a-specific-locale); the **editor** label reads the note body, the **panel** pills read the note title (`activeNoteName`).
>
> **One self-contained Typed-Link pill serves every surface** (§H.2 → §H.3 — the from-scratch redesign Eisa demanded: "one source serves all"). The Backlinks and Outgoing pills had been *the same component* yet still drifted — because the pill INHERITED font / text-direction / flex-alignment from each host row (two instances diverging purely from their surroundings). Rebuilt **`src/lib/components/LinkTypePill.svelte`** to be IMMUNE to its host: it sets its **own** font (`var(--font-interface-theme)`), direction (`dir="auto"`), size + shape (from `appSettings.linkPills.shape` directly, not host CSS vars), colour (the §G registry + auto-contrast text), and centring (`align-self`/`justify-content`/`vertical-align` + a 1px optical nudge). It now renders **pixel-identically** in Backlinks, Outgoing, and the **Knowledge-Health dashboard** — whose badge previously used a *hardcoded* colour map that had drifted (`supports` green there vs registry blue; now `<LinkTypePill>`, and the distribution bars fill from `linkTypeColor()`). The editor's inline label (CM6) already uses the same registry colour + note-language. **Boss-validated** ("it is clear that they all come from a single source"). **Lesson:** a *shared* component still drifts if it inherits context — make anything that must look identical everywhere fully self-contained, not merely shared.
>
> **MIG-069 — Style Presets (new migration; §C–§F Boss-validated, §G cards pending final test).** Named, **app-global** "Styles" (the VS Code Profiles model): save the current look — ticking which **sections** to include (theme · fonts · link-colours · pill-shape · typed-link-display · Sky-View · layout · behaviour) — **apply** with a click, rename / duplicate / delete, and **export / import** a shareable `<name>.constellation-style.json`, reusable across **every universe**. Storage is app-GLOBAL (`src-tauri/src/style_presets.rs` → `{app_data_dir}/style-presets.json`; frontend `src/lib/libraries/stylePresets.ts`, an 8-section catalogue). **Privacy invariant:** the *behaviour* section excludes security / tokens / folder paths (a vitest guards it). **Apply MERGES link colours** into the current universe by id — the Style wins on a conflict, the universe keeps its own custom types — and **never replaces** (the §F 3-agent audit caught a cross-universe data-loss regression where apply wiped the target's registry; fixed, plus a schema major-version gate on import). §G shows Styles as **preview CARDS** (Eisa, "like Obsidian"): each card is a **generated self-portrait** — theme paper + an `Aa` font sample + the accent pill + the 8 link-type colour dots — so a Style's *look* is visible at a glance, not a bare filename. Architect/Plan: `docs/MIG-069-style-presets-ARCHITECT-PLAN.md`; tests `tests/mig-069/` (5/5).
>
> **Still pending (carried from v2.47):** MIG-067 §F (dynamic per-type sortable Base columns) + §I (3-agent audit); the **search-query grammar** still hardcodes the 7 built-ins (`store.ts:2071`) so `supersedes [[X]]` + custom-type search filters don't work yet. Session log: `lab/reports/SESSION-LOG-2026-06-01.md`. Help topics + User Manual for the Link Types editor, the typed-link colour/label switches, and Styles land with this PCS.
>
> ---
>
> **What changed in v2.47** (**MIG-067 — User-Definable Link Types, "The Living Vocabulary"**: a shared Link-Type Registry ships; the 8 typed acts flow from ONE source to every surface; links are stored **predicate-first** `[[type::target]]` with **type-first authoring**; MIG-066 per-type counts closed; four Boss-found bugs fixed):
>
> **One vocabulary, read everywhere.** A new **Link-Type Registry** — `src-tauri/src/link_types.rs` + the frontend mirror `src/lib/libraries/linkTypeRegistry.ts`, seeded via the boot bundle — is the single source of truth for the 8 typed acts (+ future custom types). Every surface that lists / colours / orders link types now reads it: the editor autocomplete (`completions.ts`) + `livePreview` membership, `store.ts` (`displayLinkType` / `linkTypeNames`), the Backlinks/Outgoing badges, the **360.3D matrix** (`Inspector360`), and the Rust analytics (`strata.rs` / `tension.rs` / `libraries.rs` / `inspector360.rs`, via `LinkTypeRegistry::is_link_type_value`). **The v2.46 drift list is reconciled** — the 360.3D matrix `TYPE_ORDER` (now canonical order + gains `supersedes`), the legacy/broken `CodeMirrorEditor` autocomplete (retired), the `livePreview` / `completions` / `store.ts` lists. **Invariant held: a no-custom universe is byte-identical to today** (default registry = the 8; FNV-1a fingerprint gates re-materialisation only on an actual vocabulary change).
>
> **Links are stored predicate-FIRST** — `[[type::target]]` (was `[[target|display|type]]`). Eisa ratified this as the natural form (the type leads). His ~644k links were converted; `search.rs::extract_typed_links` + every content re-reader read **both** orders. **Authoring is type-first too** (Eisa's "Option A"): typing `[[` offers the **types** (boosted to the top) → pick one → `[[type::` → pick or type the **target** → `[[type::Note]]`.
>
> **MIG-066 §A.2/§B closed** — the Base's **per-type counts** (`supports (358), contradicts (1) …`, canonical-ordered) shipped + Boss-confirmed (EN + AR), plus a machine `note_meta.outgoing_link_types_json` (`{"type":count}`) for the §F per-type sortable columns, all **write-time materialised** (Rule 8). The §A.2 resumable batched back-fill doubles as the §A→§B JSON-column migration via the fingerprint gate.
>
> **Four Boss-found bugs fixed this session:** (1) **typed-link click created a junk "type::target" note** — the §A predicate-first switch updated *rendering* but not the click/HTML *resolvers*; fixed with a shared `stripLinkTypePrefix` (editor `NotePane` + `utils.ts`). (2) **360.3D read every link "untyped"** — the content re-readers parsed predicate-LAST only; fixed with a shared `link_types::resolve_wikilink_type` (both orders; `include_associative` distinguishes the matrix from the analytics). (3) **autocomplete lag** — the dropdown rebuilt + bidi-laid-out 20 mixed-script rows/keystroke (the note search itself is 0.12 ms, measured); cut `maxRenderedOptions` 20→8. (4) **every-boot disk thrash** — a §B-added boot `cache_reconcile()` re-walked all 7 656 files each launch, re-violating **ZERO BOOT-TIME WALKS** (`+layout.svelte:2076`); replaced with the walk-free `cache_mark_search_ready` (ensures DB + fires `cache-reconciled` so link counts / search-ready still load, no walk).
>
> **Status:** **§A–§E shipped + Boss-tested** (registry+parser · materialisation+JSON+change-flow · frontend store · reconcile-all-surfaces · type-first authoring). **Pending: §E inline colours for custom types · §F dynamic per-type sortable Base columns (`note.link.<id>` via `json_extract`) · §G the Settings → Link Types editor (add / nest-under-the-8 / recolour / reorder — the headline feature) · §H i18n + Concept-Paper v1.1 + this orientation · §I 3-agent audit + PCS.** Plan order ratified by Eisa: PCS+Orientation → §E → §F → §G → §H/§I. Architect/Plan: `docs/USER-DEFINABLE-LINK-TYPES-{ARCHITECT,PLAN}.md`; session log `lab/reports/SESSION-LOG-2026-05-31.md`. **Deferred to §I / search-integration:** the search-query grammar (`store.ts:2071` `typedLinkTypes`, `store.ts:1967`, `+layout.svelte:5742`) still hardcodes the 7 — so `supersedes [[X]]` and custom-type search filters don't work yet.
>
> ---
>
> **What changed in v2.46** (the **Living-Link Concept Paper ratified**; **MIG-066** Living-Links columns in flight; MIG-065 follow-ups shipped):
>
> **A Boss-led philosophical dialogue settled *why* the Living Link exists before ordering its types.** Result: **`docs/Living-Link-Concept-Paper-v1.0.md` — RATIFIED 2026-05-30** — now the **single source of truth** for link-type semantics + the **canonical order**: `supports · contradicts · causes · exemplifies · generalizes · derives-from · part-of · supersedes` (`associative` = null/untyped, not in the order). The order is **derived** from the inquiry arc (stance → explanation → abstraction → lineage → composition → succession), not picked — it confirms the spec + backend; the **Living-Links Guide §2 and the 360.3D matrix are drift to reconcile**. Honest repositioning (WA#5 cross-check vs Peirce / Popper / IBIS / Toulmin / Roam Discourse Graph): Constellation is **not the first to *type* a connection — the first to keep it *alive*** (weight + decay + lifecycle = the literal machinery of "without ongoing thought, I will not find truth"); the integrated, local-first, simple-by-default *application* is what stands alone. Two principles baked in: **the untyped link IS the open question** (the live edge of thinking, not a deficiency to upgrade); **facts rest, formulations inquire** (Peirce's genuine-vs-paper doubt; the stratum axis L1 Datum→L8 Worldview encodes where "needs challenge" rises — never nag a fact).
>
> **MIG-066 (Living-Links columns) IN FLIGHT.** The Base gains two opt-in columns — **Outgoing count** + **Link types** (the typed relations a note has as a *source*, canonical-ordered), **write-time materialized** (Rule-8-clean) — plus a reusable **rank-aware sort**. Backlinks **deferred to v2** (they cross library/cUniverse boundaries — no cross-schema trigger; ARCHITECT §4). **§A.1 shipped** (`10d3caf9`): `note_meta` gains `outgoing_count` / `outgoing_link_types` (canonical-ordered) / `outgoing_top_rank` (sort key) via idempotent ALTER + `note_links_outgoing_*` write-time triggers + a shared `outgoing_aggregate_assignments` SQL + test. **Resume at §A.2** — resumable **batched background back-fill** (model `sky_backfill.rs`; must NOT block boot) + **boot/re-index perf measurement on the 7,600-note Universe** (Rule 8 hard constraint) — then §B–§G. **Boss directive: §E reconciles EVERY core Living-Link surface** to the canonical order via one shared source (the 360.3D matrix `TYPE_ORDER`; the **legacy/broken** `CodeMirrorEditor` autocomplete which offers non-Constellation types `related-to`/`see-also`/`extends`; `livePreview`/`completions`/`store.ts` lists; color maps; Guide §2 → v1.1). **Resume map: `docs/HANDOVER-MIG-066-continuation.md`** + `docs/MIG-066-living-links-columns-{ARCHITECT,PLAN}.md`.
>
> **MIG-065 follow-ups shipped (post-v2.45):** `df67e349` reserved-key filter (hide `cid_cn`/`cid`/`kind` from the picker); `82086441` the Bases help topic translated into all **14** other languages; `c6675d49` the Add-column picker no longer shows YAML list-item values as fields (the `parse_frontmatter` colon-split leak, filtered in `discover_keys`).
>
> ---
>
> **What changed in v2.45** (MIG-065 — the **Unified Constellation Base is COMPLETE and Boss-validated end-to-end**; the migration closes):
>
> The "both worlds" Base shipped in full. Everything from §F.2 through §L landed on `main` and passed staged Boss tests. The Base is now **one feature on one engine** (`execute_lens`, SQL, Rule-8-clean): a standalone `.base` (YAML) opens as a **full-tab virtualized table** — Name column + your `prop.*` fields — with **no row limit** (windowed rendering; thousands of notes scroll smoothly).
>
> **Shipped this session (MIG-065 §F.2–§L):** §F.2 `97a8b52d`/`85793fdd` standalone `.base` → full-tab table · §G `eef4d433` tiered **+ Add column** picker (**Your fields** = frontmatter `prop.*`; **Constellation** = Name/Path/Created/Summary) + remove + `.base` save path · §G.2 `57e5fc47` click-header sort (asc→desc→off) + `d4309cc1` multi-sort panel · §H `0cff54de` **edit-in-place** on `prop.*` cells (Name/Created read-only), now with a **rank-ordered dropdown** for list fields (`maturity`: seed→sapling→evergreen→canonical) · §I-a `427cd3df` **`query_base` functionally retired** (orphaned `BaseView*` family removed, commands unregistered; the `fn` body stays dead in `bases.rs` — physical sweep deferred because `dataview.rs` shares helpers) · §I-b `ae28c595` New Base writes LensDefinition YAML · §J `6691fd83` audit fixes (**federated-write leak** closed — writes use the own, non-recursive `load_libraries`; old-JSON calm notice) · column **drag-reorder** `08fee7a2` → **pointer-based** `d0837d4a`/`d082004b` (Tauri's WebView eats HTML5 drag-and-drop; rebuilt on pointer events with a whole-column visual + no text-selection) · **convert old bases** `d13285e6`/`d69bbeaa` (an Obsidian / earlier-Constellation `.base` is left untouched + one-click **Convert to Constellation Base**) · **row virtualization** `9140da88` (removed the 500-row cap) · §L `660075b7` **full 15-locale localization** of the 27 Base UI strings — the brand rendered with each language's native astronomical word (كوكبة / Sternbild / 星座 / Созвездие / 별자리 / Takımyıldız / صورت فلکی / קבוצת כוכבים / तारामंडल / برج …).
>
> **Cognitive model ratified:** `docs/Cognitive-Engine-One-Picture-Concept-Paper-v1.0.md` (`a7b00ebc`/`a5661140`) — the ~10 Cognitive Elements collapse to **FOUR questions** (Development / Altitude / Origin / Connection), in service of the Five Acts toward Conviction. This is the decision rule for placing any element or legacy value; **rank-aware** column sorting lands with the Cognitive-Engine columns (**MIG-068**, where Eisa specifies the canonical orders).
>
> **Docs:** the in-app help **Bases** topic rewritten for the unified Base (English); User Manual §15 reframed as "Constellation Base & Lenses". **Governing principle (unchanged): "Strong yet Simple, by default."**
>
> **Deferred (PJ):** physical dead-Rust sweep (must KEEP the `dataview.rs`-shared `scan_folder`/`apply_filters`/`parse_frontmatter`/`BaseRow`/`FilterRule`/`SortRule`); rank-aware sort orders (MIG-068); 14-language **help-doc** translation of the rewritten Bases topic (the *in-app UI* is fully localized; help bodies pending); faithful list/nested `properties_json` (needs re-index); engine-side LIMIT/COUNT split; broaden the legacy notice to also recognize Obsidian-YAML bases.
>
> ---
>
> **What changed in v2.44** (Constellation Base reborn as the **Unified Progressive Base** — Eisa's "both worlds" direction; **MIG-065 §A–§F shipped + Boss-validated**, §F.2/§G+ in flight):
>
> Eisa: *"have both worlds"* — the familiar Obsidian-style Base **and** the PKF-powered Constellation Base, unified into **one progressive feature**, governed by a new top principle: **"Strong yet Simple, by default."** The default view is uncluttered + familiar; cognitive strength is one column-add away, never crowding the first screen. (Reconciles v1.4's clean-slate refusal of the familiar Base — see Concept Paper **v2.0**, `docs/Constellation-Base-Concept-Paper-v2.0.md`, §5.0 + §3 reframe.)
>
> **Two Bases existed** (accident of history): the old MVP (`bases.rs`, live-scan `query_base`, "Workspace Bases" sidebar) + the new lens system (`lens/`, SQL `execute_lens`, "Five Acts" sidebar). MIG-065 **unifies them onto one engine** — extend `execute_lens`, retire `query_base`.
>
> **Shipped this session (MIG-065 §A–§F, on `main`):** §A `d8af1d5c` Concept Paper v2.0 · §B `5197749e` `properties_json` faithful for **scalar** frontmatter (list/nested deferred) · §C+§D `a89fc1a9` `LensView::Table` + `prop.<key>` columns via `json_extract` + Text filters + federated · §E `3c411031` `discover_base_properties` + materializer fix · §F `76de5ed7` + polish `d3f9f3c3` inline ` ```base view: table ` renders the familiar table (clickable name, `prop.*` columns, RTL per-cell, accent count badge). **97 lens tests pass.**
>
> **Locked:** YAML `.base`; both standalone files + inline blocks; curated picker + finite aggregations (no formula language v1); one engine; `prop.` prefix for frontmatter columns (intentional deviation from the Architect's `property:` key — same outcome, zero churn).
>
> **In flight (handover):** §F.2 standalone `.base`-file → full-tab table · §G "+ Add column" picker · §H edit-in-place · §I retire `query_base` · §J audit · §K staged Boss test · §L PCS. Full resume context: **`docs/HANDOVER-MIG-065-G-continuation.md`** + `docs/MIG-065-constellation-base-unified-{ARCHITECT,PLAN}.md`. Deferred PJs: faithful list/nested `properties_json` (needs re-index); `lensBlock.col*` in 13 locales.
>
> ---
>
> **What changed in v2.43** (the universe-wide **Tag Browser** v2.42 §"New feature queued" promised now ships — that queued section below is superseded):
>
> Boss: *"I want a real universe-wide tag browser."* Constellation already had two tag surfaces — the right-sidebar **per-note** Tags panel (open note's tags only) and the navigator's universe-wide list (a mode Eisa couldn't discover) — but no first-class, discoverable, whole-universe browser. v2.43 adds one in the place Eisa already looks: the right-sidebar **Tags** tab.
>
> - **Where it lives & how it's reached:** right-sidebar **Tags** tab gains a `This note | All tags` toggle. *This note* keeps the old per-note chip list; **All tags** renders the full federated tag tree (the reusable `TagsPanel`, fed by `allLibraryTags` — already federated since MIG-061 §M, so cross-universe data needed no new wiring). The tab now renders **without an open note** (it was gated behind `{#if isHome && sidebarTab}`; pulled to a top-level branch so All-tags works on a blank universe).
> - **The tree:** nested `parent/child` tags, expand/collapse chevrons, click-a-tag → federated Search Hub (`handleTagClick`), `dir="auto"` per tag for RTL/mixed-script, a live filter box (appears > 5 tags).
> - **Sort (Boss-requested):** three modes — **A→Z**, **Z→A**, **by count** (`#`, with an alphabetical tie-break). Recursive across every tree level. Persisted as `sortMode` state.
> - **Polish (Boss remarks):** right sidebar widened 340→380 px for breath; a live **total counter** beside the TAGS header (`{n}` distinct tags — Eisa's universe shows 21 068); the body is a proper scroll region (`.rs-tags-body`, `overflow-y:auto`); the header is padded/centered off the panel edge.
> - **Freeze-on-scroll (final fix):** the TAGS header was pinned but the sort bar + filter box scrolled away. Wrapped them in a `.tp-controls` bar with `position: sticky; top: 0` + full-width opaque background, so header + sort + filter all stay frozen while only the tags scroll. Boss-tested pass.
>
> Commits: `6d6bc2b7` (tree) · `fbda7f86` (render without open note) · `f80956ee` (width / scroll / counter / header) · `44325ad3` (sort modes) · `e5b56c98` (freeze sort+filter bar). Architect: `ae180dbc` / `docs/TAG-BROWSER-ARCHITECT.md`. The 15-locale help-doc batch (#13) and the Federation help topic cover it for end users. **Federation scorecard unchanged at 8/14** — the Tag Browser was already counted closed in v2.42 (navigator path, MIG-062 §A); this is the discoverable front-end surface for it.
>
> ---
>
> **What changed in v2.42** (MIG-062 P3 filesystem-federation shipped + PJ-10/11 federation-scale Sky View / CNS polish; 8 of 14 audit federation surfaces now closed; a real universe-wide Tag Browser queued as a new feature):
>
> Continuation session (2026-05-29, crossed midnight from the 2026-05-28 MIG-060/061 marathon). Two threads: the federation-scale **polish** Eisa flagged when CNS/Sky View first showed the full 8 751-node federation, and **MIG-062** — the P3 "filesystem-walk" federation MIGs from the audit.
>
> ### PJ-10 / PJ-11 — federation-scale rendering polish
>
> When MIG-061 made Sky View + CNS show the full federation, two visual issues surfaced:
> - **PJ-11 (CNS canvas):** the gravity well was sized to `min(width,height)×0.45` — big margins on a wide monitor. Bumped to `×0.58` + fitToScreen zoom `0.85→0.93`. Stays **circular** (no ellipse stretch — that would distort the centrality=radius encoding; Form-Aligns-To-Purpose). Boss-verified pass. Commit `9a2d9890`.
> - **PJ-10 (Sky View node size):** §K's stratum fix finally delivered real `stratum` to graphEngine's strata-sizing (`baseR = 2 + (stratum-1)×2.5`), so foundational nodes rendered large at federation scale. Took 3 rounds of Boss feedback: r1 count-aware damping (sqrt), r2 steeper (exp 0.85), r3 the real fix — the bubble **frames** (stratum/provenance glows at `r+5`/`r+6`, maturity/MOC rings) used fixed pixel offsets that didn't scale with the shrunk fill, so they dominated. r3 halves the frames in dense mode (`>1500` nodes) + fill exp 1.2 → 0.12× at 8 751. Single-universe (≤1500 nodes) **untouched** throughout. Commits `62a9a198`, `f05fe6f9`.
>
> ### MIG-062 — federate the filesystem-walk sidebar surfaces (P3)
>
> The audit listed Tag Browser / Five Acts sidebar / Workspace Bases as P3 "filesystem-walk" gaps. A scoping agent corrected the audit: the **Tag Browser** was never a filesystem problem — `allLibraryTags` is federated (MIG-061 §M); it was a 1-line reactivity bug (`NotebookNavigator` set `tagMap` once on mount, no `$effect` on the federated `initialTags` prop). So MIG-062 = 1 reactivity fix + 2 read-only filesystem-federations.
>
> | § | Commit | What |
> |---|---|---|
> | A | `ca97c38a` | Tag Browser `$effect` (NotebookNavigator re-syncs federated tags) |
> | B | `f3d5cdae` | `resolve_child_universe_roots` → `pub(crate)` + **recursive** variant + 2 cycle-guarded tests |
> | C | `130be036` | Five Acts federation (read-only) + `universe_display_name` helper; `FiveActsNoteEntry.universe_name` |
> | D | `c41d52cf` | Workspace Bases federation (read-only `scan_bases_dir`, no `create_dir_all` into cUniverses) |
> | E | `56cfa153` | Frontend per-universe collapsible grouping; cUniverse bases **open-only** (no delete menu) |
> | E.2 | `5b02f3ca` | Hide the system `Five Acts/` folder from library file trees (recursive) — kills the section↔tree duplication |
> | F | `9fe0b2ef` | Boss-test doc |
>
> **Boss's governing principle (verbatim, from MIG-061 §L, reaffirmed here):** *"the data of Universe A shouldn't be merged/integrated with Universe B… if the user decides to detach… each should keep its data intact… the wheel is already there."* MIG-062 enforces this as **INV-1 (read-only)**: both backend commands only `fs::read_dir` cUniverse paths — never write/move/delete; cUniverse bases have no delete/rename menu; the `Five Acts/` folder hide is display-only (files stay on disk). Detach is lossless.
>
> **The Five Acts refinement (Boss option "A"):** federating surfaced the cUniverse's Observation note in TWO places — the dedicated top "Five Acts" section AND the file tree's `Five Acts/` folder. Boss: *"hide the folder from the tree, but if detached/switched the Five Acts should display by default."* §E.2 hides the `Five Acts/` folder from ALL library trees (recursive — it's top-level in the active universe, nested under a universe-name wrapper in a federated cUniverse). The dedicated top section is the universal access point and always reflects the **active** universe, so switching-to/detaching a cUniverse shows its Five Acts there by default. Boss-verified pass in both contexts.
>
> ### Federation audit scorecard (after MIG-062)
>
> **8 of 14** broken surfaces closed: CNS, Sky View, Backlinks, Outgoing, Tag Browser (the navigator one, via §M+§A), Five Acts sidebar, Workspace Bases — plus Tag Browser counted once. Remaining: Unlinked Mentions, Index entries/mentions, Knowledge Health, right-sidebar previews (all **MIG-063 P2 read-paths**); Cataloger / Classifier / NSC (**MIG-064 P2+P4 write-paths**); Org Chart alias-map (MIG-063).
>
> ### New feature queued — universe-wide Tag Browser
>
> Boss: *"I want a real universe-wide tag browser."* Eisa's only tag surface today is the right-sidebar **per-note** Tags panel (shows the open note's tags). The navigator's universe-wide tag list exists but is in a sidebar mode Eisa doesn't use / can't discover. Queued as a NEW feature (its own Architect: where it lives, how it's reached, click-to-filter, RTL, 15-locale). `allLibraryTags` is already federated, so the data is ready.
>
> ### Docs / PCS note
>
> 15-locale **Federation help-doc** topic (covering MIG-061+062 federation behavior) is queued as a batched task — English canonical written this PCS, 14 translations tracked separately. Session log: `lab/reports/SESSION-LOG-2026-05-29.md`. MoCh: `docs/MoCh/MoCh-2026-05-29-*.md`. Milestone tag `milestone/mig-062-filesystem-federation-shipped`.
>
> ---
>
> *Below: the v2.41 preamble, retained for diff visibility.*

**Version 2.41 | 2026-05-28**

> **What changed in v2.41** (MIG-061 P1 federation fix shipped — CNS / Sky View / Backlinks / Outgoing / Tag Browser all now show federated data; 5 of 14 broken surfaces from the audit are now closed; pre-existing latent bug surfaced and fixed as §K):
>
> Block 3 + Block 4 of 2026-05-28 (continuation of MIG-060 + the federation audit): MIG-061 — federate `cache_boot_snapshot_sky` AND `cache_boot_snapshot_graph` — shipped through 17 commits and 6 Boss-test stages over ~6 hours of debugging and 8 binary rebuilds. The headline win: Eisa's CNS gravity well went from showing 987 of 8 751 federated nodes (showing only the parent universe) to showing all 8 751 nodes with 233 286 links.
>
> ### MIG-061 — Federate the boot-snapshot IPCs (P1)
>
> | § | Commit | What |
> |---|---|---|
> | A | `6b5173fa` | `get_federated_schemas` helper |
> | B | `76f9f826` | `read_sky_nodes_raw_in_schema` |
> | C | `dc43e753` | `read_sky_links_raw_in_schema` |
> | D | `4df77b6d` | `is_federated_sky_ready` (Q4 all-or-nothing) |
> | E | `ade3a010` | `cache_boot_snapshot_sky` federation loop |
> | G | `1d755755` | 8 initial unit tests |
> | H | `e69417af` | Boss-test doc |
> | J | `e05be00a` | `federation:ready` event emit (Rust → frontend) |
> | J.2 | `1a500cf4` | listener-order fix + defensive re-invoke |
> | K | `617f4302` | **stratum column-type read fix — latent pre-MIG-061 bug** |
> | L | `c62f8c53` | **Q3 → Option A: per-schema isolation (Boss principle)** |
> | M | `7f648a55` | federate `cache_boot_snapshot_graph` (Backlinks/Outgoing/Tags/Aliases) |
> | N | `0c6f7661` | listener re-fetches GRAPH too |
> | O | `25562627` | remove §J.3 diagnostic tracing |
> | P+Q | `3b823085` | audit follow-ups (D4 empty-overwrite guard + 2 new unit tests) |
>
> ### The two decisive technical inflection points
>
> **§K — the stratum column-type latent bug.** Boss-test Stage 2 failed three consecutive times with the same "987 nodes" result. The §J.3 `diag_log_line` tracing finally revealed `cache_boot_snapshot_sky` was crashing on `Invalid column type Text at index: 4, name: stratum` — a pre-existing bug, where the column was declared TEXT in the schema but the Rust struct read it as `Option<i64>`. SQLite's loose typing meant some rows stored stratum as INTEGER, some as TEXT; the read crashed on TEXT-class rows. CNS / Sky View / Backlinks have been silently falling back to `buildSkyData` (the legacy non-federated path) all along — for who knows how long. Fix: `row.get_ref(4)` + `match` on `rusqlite::types::ValueRef::{Null, Integer, Text, Real, Blob}` to handle both storage classes.
>
> **§L — the per-schema isolation principle.** Eisa stopped the cascade with a structural insight: *"The Federation should be simple. The app shouldn't reinvent the wheel; the wheel is already there!"* — articulating the principle that each universe's data must stay its own (no merge, no integration; detachment leaves each universe intact). The Architect had locked Q3 = Option B (federated link resolution across the merged set). After §L, Q3 = Option A: each cUniverse's links resolve only within itself. The merge logic in `cache_boot_snapshot_sky` is now a per-schema loop that builds per-schema `path_to_idx` / `name_to_idx` / `alias_to_path` maps — strict isolation. Standalone-A behaves identically to A-as-cUniverse-of-B.
>
> ### Boss-test verdict (6/6 PASS)
>
> | Stage | Result |
> |---|---|
> | 1 — Single-universe regression | ✓ pass |
> | 2 — Federated count (Eisa Universe) | ✓ pass — **8 751 nodes · 233 286 links** (was 987 / 1 178) |
> | 3 — Boot time | 28s on a 25-cUniverse federation (within INV-2's "≤2× single-universe") |
> | 4 — Backlinks for cUniverse note | ✓ pass — **104 linked mentions** with typed-link badges |
> | 5 — Outgoing Links for cUniverse note | ✓ pass — **101 outgoing links** |
> | 6 — Sky View parity | ✓ pass — same 8 751 nodes via the bubble visualization |
>
> ### Audit findings — 8/8 invariants UPHELD
>
> Three parallel agents (invariant-checker, drift-detector, migration-path-validator) audited the shipped code. Verdicts:
> - **All 8 invariants UPHELD** (the 7 documented + 1 new INV-K for the flexible-stratum-read).
> - **D4 (MEDIUM)** — empty-overwrite race in the federation:ready listener. **Fixed in §P.**
> - **D5 (LOW)** — no direct unit tests for §M's graph federation. **Fixed in §Q.**
> - **D6 (MEDIUM-positive)** — §M's graph federation closes **Tag Browser** as a side effect via `allLibraryTags` (so 5 of 14 audit-broken surfaces are now fixed, not 4).
> - **S4 (FAIL)** — pre-MIG-061 binary rolling back after MIG-061 has written would crash on TEXT-stratum reads. But this is a **pre-existing** risk (the bug existed pre-MIG-061; production silently fell back to `buildSkyData`); MIG-061 didn't introduce it.
>
> ### Surfaces now closed by MIG-061
>
> From the 14 broken surfaces in `docs/MIG-061-federation-audit-findings.md`:
>
> | Surface | Closed by MIG-061? | Mechanism |
> |---|---|---|
> | CNS | ✓ | §E federated `cache_boot_snapshot_sky` |
> | Sky View | ✓ | inherits CNS's data |
> | Backlinks | ✓ | §M federated `cache_boot_snapshot_graph` |
> | Outgoing Links | ✓ | §M (same) |
> | Tag Browser | ✓ | §M (side effect via `allLibraryTags`) |
> | Unlinked Mentions | ✗ | uses own Tauri command — MIG-063 (P2) |
> | Five Acts sidebar | ✗ | filesystem walk — MIG-062 (P3) |
> | Workspace Bases | ✗ | filesystem walk — MIG-062 (P3) |
> | The Cataloger | ✗ | uses own Tauri command — MIG-064 (P2+P4) |
> | Classifier (scan + single-note) | ✗ | uses own Tauri command — MIG-064 (P2+P4) |
> | NSC Backfill | ✗ | uses own Tauri command — MIG-064 (P2+P4) |
> | Index entries + mentions | ✗ | own read-only Connection — MIG-063 (P2) |
> | Knowledge Health | ✗ | own Tauri command — MIG-063 (P2) |
> | Right-sidebar previews | ✗ | uses state.db — MIG-063 (P2) |
>
> 5 of 14 closed by MIG-061. Remaining 9 fall under MIG-062 (P3), MIG-063 (P2 read), MIG-064 (P2+P4 write).
>
> ### Two polish items deferred to Pending Jobs
>
> - **PJ-NNN-A — Sky View node size scale.** Bubbles look large on Eisa Universe's 8 751-node federated view; the d3 simulation hasn't been tuned for federation-scale node counts.
> - **PJ-NNN-B — CNS gravity well full canvas.** When Constellation is full-screen, the gravity well doesn't expand to use the whole canvas. Fixed-size layout assumptions in `computeGravityWellLayout`.
>
> ### What's next
>
> Per Eisa's locked Option B from the federation audit: MIG-062 (P3: filesystem walks — narrows to **Five Acts sidebar + Workspace Bases** only since Tag Browser is closed by §M). Then MIG-063, then MIG-064.
>
> Session log: `lab/reports/SESSION-LOG-2026-05-28.md` (Block 4). Boss-test docs: `docs/MIG-061-BOSS-TEST.md`. Audit findings doc: `docs/MIG-061-federation-audit-findings.md`. Architect: `docs/MIG-061-cns-federation-ARCHITECT.md`. Plan: `docs/MIG-061-cns-federation-PLAN.md`.
>
> ---
>
> *Below: the v2.40 preamble, retained for diff visibility.*

**Version 2.40 | 2026-05-28**

> **What changed in v2.40** (MIG-060 Phase 1.5 Threading Gestures shipped; full federation audit completed surfacing 14 broken surfaces across 4 patterns; MIG-061+ scope decisions pending Boss approval):
>
> Block 2 + Block 3 of 2026-05-28 (continuation of the morning MIG-058+059 close): MIG-060 — the Phase 1.5 host-note threading gestures from Constellation Base — shipped through six commits (§A i18n, §B widget, §C listener, §D CSS, §E tests, §F Boss-test) plus three follow-up sub-fixes that came out of Boss-test stages 3-4 (§C-fix CNS focus, §C-fix-2 orphan-hide, §C-fix-3 Cataloger focus). Boss-test verification then surfaced that the test note "Eisa ALSHAMSI" wasn't in CNS's gravity well at all → diagnostic logging via the MIG-058+059 `diag_log_line` infra confirmed CNS shows only **987 of 8 751** federated notes in Eisa Universe. Eisa requested a full audit of how the rest of the system handles cUniverse-bearing Universes.
>
> ### MIG-060 — Threading gestures from lens rows
>
> Each lens row in the Five Acts / Bases / custom `base` blocks now carries three small icon buttons (12 px, 55% opacity at rest, 100% on hover) on its trailing edge: 360.3D / CNS / Cataloger. The buttons dispatch a single `constellation:open-note-in-surface` custom event with a `surface` discriminator; `+layout.svelte`'s listener opens the host note via `await openNoteTab(...)`, `await tick()`, then activates the requested surface.
>
> Per-surface focus mechanism turned out to differ — Plan had treated them uniformly; reality required three custom fixes:
>
> | Surface | Focus mechanism | Fix commit |
> |---|---|---|
> | 360.3D Inspector | Reads from `focusedTab` automatically — no extra wiring needed | (works for free) |
> | CNS | New `focusNoteId?: string` prop on `ConstellationSight2.svelte` — onMount after `fitToScreen()` finds the matching SimNode by path, sets `selectedNode`, pans canvas to center it | `5114ce88` (§C-fix) |
> | Cataloger | Dispatches `constellation:classify-and-show` (existing event) one rAF after `showCataloger = true` so the embedded `SourceReviewPanel` has time to mount | `e7baaadb` (§C-fix-3) |
>
> Orphan-hide gate (§C-fix-2, commit `1ce715ed`): CNS only shows the linked subgraph (typed-link participants). The lens-row CNS icon is hidden when the note isn't in `skyNodePathSet` — a new writable store in `libraries/store.ts` mirrored from `skyNodes` via a `$effect` in `+layout.svelte`. Boot-time edge case (empty set during indexing) permissively shows the icon; the §C-fix listener has a graceful no-match fallback.
>
> Boss-test (`docs/MIG-060-BOSS-TEST.md`) stages:
> - **Stage 1 (visual)** — ✓ pass. Three icons render per row, RTL auto-flips position via `marginInlineStart: auto`.
> - **Stage 2 (360.3D)** — ✓ pass on "Observation — Recent Captures".
> - **Stage 3 (CNS)** — initially failed (note opened, CNS opened, but wrong node centered). Three iterations later (§C-fix → §C-fix-2 with orphan-hide → diagnostic trace → confirmed `NO MATCH` because note isn't in CNS's subgraph) → re-tested on **Eisa Cognitive Knowledge** universe (single-universe, no federation gap) → ✓ pass with Eisa ALSHAMSI bubble correctly centered.
> - **Check A (orphan-hide)** — ✓ pass. Orphan rows show 2 icons (360 + Cataloger); linked-note rows show 3 icons including CNS.
> - **Stage 4 (Cataloger)** — ✓ pass on linked note "Mohammed bin Zayed's Dark Vision of the Middle East's Future" (correctly focused). ✗ for orphan/federated note "Observation — Recent Captures" with `FOREIGN KEY constraint failed: sources_suggestions(note_path) → note_meta(path)` — pre-existing Cataloger federation gap, not a MIG-060 wiring bug.
> - **Stage 5 (RTL)** — ✓ already passed in Stage 1's screenshot (Arabic-named row's icons on visual right).
>
> MIG-060 ships. Threading gestures work end-to-end on all paths that don't hit pre-existing federation gaps.
>
> ### Federation audit — 14 broken surfaces across 4 patterns
>
> Findings doc: **`docs/MIG-061-federation-audit-findings.md`**. Four parallel exploration agents surveyed every core surface. Status:
>
> | Bucket | Count | Surfaces |
> |---|---|---|
> | ✓ Federated (works) | 4 | libraryStats, Search Hub/QuickSwitcher, Lens execution, Federation Warnings popup |
> | N/A by design | 5 | 360.3D, Bookmarks, Global Tasks, Expression Forge / Sense-Making Canvas / Dashboard |
> | ◑ Partial | 1 | Org Chart (`constellation_map_universe`) — tree includes cUniverses but `load_alias_map` is parent-only |
> | ✗ Broken | 14 | CNS, Sky View, Backlinks, Outgoing, Unlinked Mentions, Tag Browser, Five Acts sidebar, Workspace Bases, The Cataloger, Classifier (scan + single-note), NSC Backfill, Index panel (entries + mentions), Knowledge Health, right-sidebar previews |
>
> The 14 broken surfaces collapse to **four root-cause patterns**:
>
> 1. **P1 — `cache_boot_snapshot_sky` not federated** (`cache.rs:382`). One Tauri command, four dependent surfaces (CNS, Sky View, Backlinks, Outgoing). **Highest leverage / lowest risk fix.**
> 2. **P2 — Backend command uses bare `state.db` instead of `state.federated_conn`.** Six surfaces (Cataloger backend, Classifier scan + single-note, NSC, Index entries + mentions, Knowledge Health, right-sidebar previews).
> 3. **P3 — Hardcoded `{active_universe}` filesystem paths.** Three surfaces (Tag Browser, Five Acts sidebar, Workspace Bases). Each needs a per-cUniverse enumeration loop.
> 4. **P4 — FK constraints to parent's `note_meta`.** Compounds P2 for write paths (Cataloger/Classifier INSERT path). Needs schema-design decision: replicate note_meta entries / drop FK / move suggestions to per-cUniverse tables.
>
> Three scope options for the fix MIG(s):
> - **Option A** — one mega-MIG fixes all 14. Big plan, big risk.
> - **Option B (recommended)** — four pattern-grouped MIGs: **MIG-061** = P1 (CNS gravity well); **MIG-062** = P3 (filesystem walks); **MIG-063** = P2 read-paths; **MIG-064** = P2+P4 cataloger/write-paths.
> - **Option C** — just MIG-061 (CNS only); reassess later.
>
> Awaiting Boss decision on scope before drafting the MIG-061 Architect.
>
> ### What's next
>
> Boss picks A/B/C. If B: MIG-061 Architect for `cache_boot_snapshot_sky` federation. Session log captures both blocks (MIG-060 §A-§F + sub-fixes; audit findings) in `lab/reports/SESSION-LOG-2026-05-28.md`. MoCh entry pending.
>
> ---
>
> *Below: the v2.39 preamble, retained for diff visibility.*

**Version 2.39 | 2026-05-28**

> **What changed in v2.39** (MIG-058 + MIG-059 resolved — federated search latency reduced from 15-25s to sub-second; Arabic input truncation resolved as a side effect; the 4-MIG federation detour from the Constellation Base roadmap closes here):
>
> The federation work that started 2026-05-25 with MIG-055 §I Stage 5's discovery of the federation gap, ran through MIG-056 (foundation), MIG-057 (lexicon boundary), and finally MIG-058 + MIG-059 (perf + Arabic input), wrapped on 2026-05-28 morning after 8 iterations of the perf problem.
>
> ### The MIG-058 + MIG-059 resolution
>
> Eight options pursued in sequence, each pruning a hypothesis with empirical evidence. The breakthrough came when Eisa pushed back on "let's accept the limitation" framing: *"It is not in my doctrine to accept any limitation! Think again!"* That forced re-reading the diagnostic data column-by-column instead of speculating about caching / connection state. The cost scaled with **result count (30 rows)** — and the only thing in the SQL doing per-row work via the custom Arabic-normalizing tokenizer was FTS5's native `snippet()`.
>
> | Option | Result | Lesson |
> |---|---|---|
> | §K.1–§K.3 | Architecture correctness (PREPARE failures, scatter-gather + RRF) | bm25/snippet aux funcs can't be schema-qualified in UNION ALL |
> | Diag v2 | Hard evidence (per-branch timings, EXPLAIN, sqlite_stat1) | sqlite_stat1 populated, plan reasonable, no obvious issue |
> | Option C | Per-schema queries on warm federated_conn | 13s — architecture clean but perf unchanged |
> | Option E | mmap_size + cache_size on federated_conn | 18s REGRESSED — mmap on ATTACH bypassed libraryStats-warmed OS cache |
> | Option F | Pre-warm via MATCH query | Stopword filter stripped warm tokens; 0 matches; no warming |
> | Option G | FTS5 `optimize` segment merge | Worked (39s first boot, 0ms idempotent) — improved RESULT QUALITY but not timing; fragmentation wasn't dominant cost |
> | **Option H** | **Bypass FTS5 `snippet()` in federated mode; Rust-side substring snippet** | **< 1 second. Done. MIG-058 resolved as side effect.** |
>
> ### Shipped fix (Options C + G + H combined, commit `c426af7e` final)
>
> - **Option C** — drop the §K.3 per-cUniverse standalone Connection pool. Use the ATTACH-based `federated_conn` for all federation paths (libraryStats / lens / search). Per-schema queries with `FROM cu1.notes_fts` resolve `bm25(notes_fts, ...)` correctly (proven by 4 new option_c_* unit tests).
> - **Option G** — background `federation_prewarm` thread runs `INSERT INTO notes_fts(notes_fts) VALUES('optimize')` per cUniverse after federated_conn is saved. 30-60s first boot per cUniverse; 0ms subsequent boots (idempotent, persistent). Improves BM25 ranking quality but doesn't fix timing alone.
> - **Option H** — `lexical_search_in_schema` takes a `skip_fts5_snippet: bool` parameter. Federated mode (`true`) selects raw `body_text` and synthesizes snippets in Rust via `synth_snippet_for_body` (UTF-8 char-boundary safe, handles bridge terms for cross-language matches). Active mode (`false`) still uses FTS5's native `snippet()`.
>
> ### Boss-test verdict
>
> | Stage | Pre-fix | Post-fix |
> |---|---|---|
> | Federation status bar | 8751 + ⚠ 1 | 8751 + ⚠ 1 (unchanged) |
> | First federated search | 16-25s | **Almost instantly** |
> | Second federated search | 25s | **Under a second** |
> | Arabic slow-typing | Truncated to `الرب`, 30s+ to results | **Full word lands, sub-second results** |
>
> ### Why MIG-058 closed as a side effect
>
> Eisa's MIG-058 truncation was caused by 16-second async `constellationSearch` calls blocking the IPC/event loop. Arabic keystrokes typed at 300-400ms intervals queued in WebView2's event buffer during the block; when the async resolved, all buffered keystrokes fired in rapid succession, and Eisa visually observed only the first 4-5 characters before reactive cascade fully settled. With sub-second search (Option H), no block window exists for keystrokes to buffer in.
>
> ### What's next: back to the Constellation Base roadmap
>
> MIG-058 + MIG-059 close the 4-MIG federation detour. The next trunk MIG per Concept Paper v1.4 is **Phase 1.5 — Host-Note Assemblage + Open-in-360.3D + Open-in-CNS + Open-in-Cataloger gestures**. The full roadmap (Phases 2 / 2.5 / 2.6 / 2.7 / 3 / 4 / 5 / 6 / 7 / 8+) is unchanged from v2.36.
>
> Session log: `lab/reports/SESSION-LOG-2026-05-28.md`. MoCh: pending.
>
> ---
>
> *Below: the v2.38 preamble, retained for diff visibility.*

**Version 2.38 | 2026-05-27**

> **What changed in v2.38** (MIG-057 Lexicon Expansion + Prefix-Wildcard Coexistence shipped — the lemma-vs-prefix collision that disappeared `الرباط` from search results for typed `الربا` is closed; 1 of the 3 follow-up MIGs opened in v2.37 is now done):
>
> Block 2 of 2026-05-27 (continuation of the morning §K.3 federation ship): Eisa picked the clearest of the three open follow-up MIGs to tackle next. MIG-057's fix lives in one function, `src-tauri/src/search.rs::expanded_match_query`. When the lexicon expansion fires for a typed input that IS a corpus lemma, the resulting FTS5 MATCH expression now appends the literal prefix wildcard (`<input>*`) alongside the cross-language OR-list — so a user typing `الربا` (a lemma — "usury") looking for `الرباط` (a longer word starting with the same prefix — the city of Rabat) gets BOTH the translation expansion (`Rabat` / `interest` / `ربا`-rooted notes) AND the prefix match (`الرباط` and other tokens starting with `الربا`). BM25's column-10 weight boost on `name` puts the literal title `الرباط` at rank 2 in Eisa's verified Boss-test (rank 1 was `سورة الأنفال 08` which has the lemma in its body — also a valid match).
>
> Commit `9c1b8603`: the search.rs change + 3 new regression tests in `tests_m12` (English lemma keeps prefix, Arabic lemma keeps prefix, quote-sanitization). All 8 `tests_m12` pass; 836/836 lib tests pass overall — no regression.
>
> Commit `2bf62cbd`: Boss-test tutorial `docs/MIG-057-BOSS-TEST.md`.
>
> Verification screenshot: Eisa's QuickSwitcher with `الرباط` typed in the search box — `الرباط` (lib `جغرافيا`) at rank 2 highlighted, surrounded by geography cluster (`المرابطون` / `الموحدون` / `الدار البيضاء` / `المغرب` / `مراكش` / `نواكشوط` / `فاس` / `منظمة التعاون الإسلامي`). Both halves of the fix working: lexicon expansion still pulls cross-language Rabat-context notes, AND prefix wildcard catches title-prefix matches.
>
> ### Remaining open follow-ups
>
> - **MIG-058 — QuickSwitcher Arabic input truncation.** Pre-existing; not addressed. Stub doc at `docs/MIG-058-quickswitcher-arabic-input-truncation-STUB.md`.
> - **MIG-059 — Slow per-cUniverse search investigation.** Pre-existing; not addressed. Stub doc at `docs/MIG-059-slow-federated-search-investigation-STUB.md`.
>
> No body sections of this orientation needed structural changes for MIG-057 (it's a localized fix inside an existing function). Body-update debt tracked in v2.37 preamble still stands.
>
> ---
>
> *Below: the v2.37 preamble, retained for diff visibility.*

**Version 2.37 | 2026-05-27**

> **What changed in v2.37** (MIG-055 Constellation Base + MIG-056 Cross-Universe Federation shipped — the Five Acts lens system is live, cross-universe search/lens/libraryStats federation is live, and the architectural gap surfaced by MIG-055 §I Stage 5 is closed):
>
> Block 1 of 2026-05-26 reverted MIG-054's SQL-backend rewrite (which was carrying a `parse_frontmatter` upstream bug into corrupted YAML column titles) and rebuilt the Bases system from scratch as **MIG-055 — Constellation Base (clean rebuild)**. The new design ships as a curated-dimension lens system: `note.name`, `note.path`, `note.created_at`, `note.headline` v1 only (no `properties_json` reads in v1 — that's how the MIG-054 bug got in). Lens YAML inside ``` ```base ``` fenced blocks in any markdown file, rendered via a CM6 WidgetType + StateField (after Svelte 5 `mount()` failed for CodeMirror block-replace widgets — Marijn Haverbeke's CM6 author guidance recommended pure DOM for ViewPlugin-supplied block decorations crossing line breaks; §H.3 fix). New `Five Acts` sidebar section above the legacy Workspace Bases. System note `Observation — Recent Captures` lives at `{universe}/Five Acts/*.md` (transfer-on-edit; system never overwrites user edits). 84 lens tests, 13 §G end-to-end fixture tests. Boss-test Stages 1-4 pass; Stage 5 revealed the federation gap (cUniverse counts/notes invisible) that became MIG-056.
>
> ### MIG-055 — Constellation Base (clean rebuild)
>
> | Layer | Module | Status |
> |---|---|---|
> | Dimension registry | `lens::dimensions` | 4 v1 dims, REGISTRY constant, 10 tests |
> | YAML parser + validator | `lens::parser`, `lens::validator` | `parse_lens_yaml`, `validate`, schema:1 mandatory; 24 tests |
> | SQL builder + Tauri command | `lens::sql_builder`, `lens::query` | `build_sql`, `execute_lens`, in-memory SQLite integration tests; 25 tests |
> | LensBlock renderer (CM6 widget) | `src/lib/components/LensBlock.svelte` + `livePreview.ts` | Pure-DOM widget mounted via `baseLensField` StateField; 15-locale i18n for `lensBlock.*` |
> | System note bootstrap | `lens::system_notes` | `init_five_acts_system_notes` idempotent + transfer-on-edit; called from `ensure_search_db_ready`; 10 tests |
> | Sidebar Five Acts section | `+layout.svelte` | New section above Workspace Bases; `list_five_acts_notes` command; 15-locale `sidebar.fiveActs` |
> | End-to-end fixture tests | `lens::tests` | 13 tests driving the full pipeline from canonical YAML; includes drift catch between §E system-note constant and §G fixture |
>
> ### MIG-055 §I → §J PCS → §K hand-off
>
> Stages 1-4 of the Boss-test gate passed on 2026-05-26. Stage 5 (Cross-universe federation) failed predictably — the lens "Observation — Recent Captures" returned 1 row on Eisa Universe + 2 cUniverses because each universe had its own `search.db` and there was no federation layer. Eisa's directive: **"Open MIG-056. MIG-056 > MIG-055."** MIG-055 §J PCS shipped (push, help docs, MoCh) but the §I gate stayed yellow pending MIG-056's federation foundation.
>
> ### MIG-056 — Cross-Universe Federation
>
> Closes MIG-055 §I Stage 5's federation gap. Architect-locked decisions:
>
> - **§5.1 — Four consumers federate:** lens / status bar / libraryStats / global search.
> - **§5.2 — skip_unavailable failure model.** cUniverse failures become warnings, not errors. Federation continues with whatever's available.
> - **§5.3 — Auto-migrate on first attach.** Four safeguards: lock probe (BEGIN EXCLUSIVE), backup to `.pre-mig-056.bak`, atomic txn via backup-restore, audit log to `federation-audit.log`.
> - **§5.4 — ATTACH cap = 25** (Boss-locked; SQLite default is 10).
>
> ### MIG-056 §A–§I (foundation)
>
> | Phase | What landed |
> |---|---|
> | §A | `federation::failure` — `FederationWarning` / `FederationError` / `MigrationError` types |
> | §B + §B.1 | `federation::attach::attach_all` — `ATTACH DATABASE ... mode=ro AS cu{i}`; `verify_schema` on note_meta cols; `SearchState.federated_conn` for libraryStats/lens UNION ALL |
> | §C | `federation::migrate::run_migrations_on` — the 4 safeguards |
> | §D | `federation::query` — `per_schema_select` + `union_all_compose` for the libraryStats/lens path |
> | §E | Lens consumer adoption — `execute_lens` falls through to single-schema if federation isn't ready |
> | §F | libraryStats federation — `aggregate_library_counts` reads from the ATTACH-based `federated_conn` |
> | §G | Federated FTS5 lexical search — original UNION ALL design with `bm25(schema.notes_fts, ...)` |
> | §H | Frontend `FederationWarning` UI — status-bar warning badge + popup; 15-locale i18n for `federation.*` |
> | §I | End-to-end integration tests on synthetic 3-schema federation; 47 federation tests; 42 lens tests still passing |
>
> ### MIG-056 §J audit (3 agents in parallel, PASS-WITH-NOTES)
>
> Invariants / drift / migration-paths agents. 2 P1 findings, both fixed inline as §J.1 before §K Boss-test:
> - **§J.1.A — Federation generation counter** (`federation_generation: AtomicU64`) for the universe-switch-during-background-attach race.
> - **§J.1.B — Lens fallback parity** — `execute_lens` propagated federated-query errors via `?`; fixed via federated-then-fallback closure pattern.
>
> 9 P2/P3 findings catalogued for `§L` PCS hardening pass or future MIG.
>
> ### MIG-056 §K Boss-test gate (2026-05-26 evening → 2026-05-27 afternoon)
>
> Three hotfix rounds during the Boss-test surfaced PREPARE-time SQL constraints + lifecycle issues:
>
> - **§K.1 — FTS5 tokenizer registration on the federation Connection** (commit `0d5c1f8f`). Tokenizer registration is per-Connection in SQLite FTS5 (no global registry in `bundled` builds); without it the federation_conn's MATCH queries silently returned zero results.
>
> - **§K.2 — Drop bm25/snippet from federated SQL + libraryStats lifecycle re-fire** (folded into the §K.3 commit). The §G UNION ALL design used `bm25(schema.notes_fts, ...)` which fails at PREPARE: SQLite's FTS5 aux funcs take a self-referential pseudo-column bound to the unqualified original table name; can't be schema-qualified, can't be aliased. §K.2 dropped the aux funcs and ordered by `modified DESC` — functional but lost BM25 ranking. Also fixed: `loadAllStats()` was called at boot before federation completed; re-fired in the 3s warning re-poll so the status bar updates from 1101 (pre-federation) to 8751 (federated).
>
> - **§K.3 — v2 scatter-gather + RRF merge** (commit `0e094da0`, this PCS). The architecturally correct fix. Per-cUniverse standalone Connections in a new `SearchState.federated_search_conns` pool; background-attach thread opens each, sets PRAGMA setup matching `init_db`, registers the `constellation` tokenizer. `federated_lexical_search_or_fallback` rewritten as scatter-gather: runs single-schema `lexical_search` once per Connection (each with full BM25 + FTS5 snippet because there's only one `notes_fts` in scope per Connection). Per-branch ranked lists merged in Rust via **Reciprocal Rank Fusion (RRF, k=60)** — the Cormack & Clarke (2009) constant adopted by Elasticsearch CCS / Vespa / Lucene MultiSearcher / OpenSearch. Avoids cross-corpus BM25 incomparability. 7 RRF unit tests; 833/833 lib tests pass.
>
> ### MIG-056 §K Boss-test verdict
>
> - ✅ **Stage 1** — Lens federation returns rows from main + cUniverses.
> - ✅ **Stage 2** — Status bar shows 8751 notes (1101 Eisa Universe + 7650 Eisa Cognitive Knowledge); refreshes after the 3s post-boot poll.
> - ✅ **Stage 3** — Sidebar cUniverse library badges show non-zero counts.
> - ✅ **Stage 4** — Federated search returns cUniverse notes with BM25 relevance ranking.
> - ✅ **Stage 5** — Missing cUniverse (`كون عيسى`) gracefully warns via the status-bar badge; popup shows path + reason ("search.db missing"); app keeps working.
>
> Three pre-existing issues surfaced during the Boss-test but were *not caused by* federation; deferred to focused follow-up MIGs (Boss decision):
>
> - **MIG-057 — Lexicon expansion boundary fix.** When a short Arabic input is BOTH a prefix of a longer word AND a lemma in the corpus, `expanded_match_query` replaces the prefix wildcard with multi-language exact-phrase OR — losing the substring match. Pre-existing; federation made it visible.
>
> - **MIG-058 — QuickSwitcher Arabic input truncation.** Search box truncates Arabic when typed at normal pace; only paste / fast-type gets the full word in. Suspected: Svelte `bind:value` + `filtered` `$derived` + async `constellationSearch` debounce racing with IME composition. Pre-existing.
>
> - **MIG-059 — Slow cu1 branch search investigation.** The standalone-Connection `lexical_search` is ~20× slower than the active-mode equivalent on the same data. Doesn't break correctness; perf cost is ~25s for a single federated FTS5 search on Eisa's data.
>
> ### Body-update debt
>
> §4 subsystem records still describe the pre-§K.3 federation design (UNION ALL + bm25, before scatter-gather replaced it). The §K.3 commit (`0e094da0`) is the trustworthy current state. Body sections will be brought into line at the next consolidation pass.
>
> Session logs: `lab/reports/SESSION-LOG-2026-05-26.md` + `lab/reports/SESSION-LOG-2026-05-27.md`. MoCh: pending.
>
> ---
>
> *Below: the v2.36 preamble, retained for diff visibility.*

**Version 2.36 | 2026-05-25**

> **What changed in v2.36** (Cataloger Bridge added to Constellation Base Concept Paper — design now spans all four Constellation deep-read surfaces; MIG-049 renumbered to MIG-054 for audit-trail clarity; Architect doc updated with the 5 Q1-Q5 uniqueness-aligned answers folded in):
>
> Block 4 of today (after v2.35's PCS landed at commit `b659632c` ~40 min earlier) carried two threads. First: Eisa locked the 5 Architect-doc open questions (Q1-Q5) with the instruction "*Answer the questions in a way that achieves the Constellation Base uniqueness*" — re-framing each through the differentiator lens rather than the easiest-to-ship lens. Second: Eisa caught that **The Cataloger (CECE) was treated as a Phase 6 afterthought** in v1.3 §6.4, when it is a **Core Plug-in** with the same architectural status as CNS (promoted from subsystem to dock-mounted feature on 2026-05-19, same day as the Sight + Map disabling per MIG-038). v2.36 captures both threads.
>
> ### Q1-Q5 locked — the uniqueness-aligned answers
>
> 1. **Field-name alignment** (`selectedVaults` → `selectedLibraries`): **Fold into Phase 1 step §D + retire the legacy `"vault"` source type.** Library is part of Constellation's identity (Universe → Library → Folder → Note vocabulary per CLAUDE.md Conventions). Distinguishes Constellation from Obsidian's "Vault" / Notion's "Workspace."
> 2. **`update_note_property` latency window** (1.5s file-watcher debounce): **REJECT — wire explicit `bases:note_updated` Tauri event + immediate `note_meta.properties_json` write.** Dashboard effect (§1) is *"I see what's alive in my work right now"* — a 1.5s gap is eventually-consistent UX, not "alive."
> 3. **Fixture set design:** **75 fixtures designed as the foundation suite for Phase 2 / 2.5 / 2.6 / 2.7**, not just a Phase 1 regression check. Each future phase extends the same harness; the harness IS the regression suite for the entire roadmap.
> 4. **Boss-test Stage 1 universe:** **Cognitive Knowledge universe with the LL-025 copy-test pattern.** Test on a COPY of the real DB first; only swap the live binary after 100% diff-clean.
> 5. **MIG number reuse:** **Jump to MIG-054.** Clean break from the reverted-Mind allocation (MIG-049 was originally Mind Phase 2 — write tools — never built). Audit-trail clarity > minor file-rename cost.
>
> ### The Cataloger Bridge — Concept Paper v1.4 (the §6.12 addition)
>
> CECE (Constellation Epistemic Content Engine) is **"The Cataloger"** in user-facing vocabulary (Arabic: **المُصنِّف**). It classifies each note on two axes drawn from a comparative survey of five civilizations' epistemologies (Greek/European, Arabic-Islamic, Indian *pramāṇa*, Chinese Mohist/Confucian, Persian Illuminationist):
>
> - **Source axis** — where the knowledge came from. 11 parents (perception / inference / testimony / mass-transmission / comparison / postulation / non-apprehension / memory / innate-disposition / inspiration / revelation) → 41 leaves → 53 total IDs.
> - **Content-type axis** — what kind of cognitive object. 5 branches (sensory-inputs / symbolic-entities / semantic-contents / epistemic-states / higher-order-constructs) → ~218 sub-nodes, max depth 4.
>
> v1.4 §6.12 ("CECE Measurements as Bases Columns — the Epistemic Bridge") makes every CECE measurement queryable as a Bases column at **Phase 2.7**:
> - Approved Source × Content-type primary + secondary (from frontmatter mirror in `note_meta.properties_json`)
> - Suggestion regime — unanimous / strong_majority / split
> - Disambiguation flags + candidates (from `sources_suggestions` table)
> - Pending-suggestion boolean
> - Last classified timestamp + classified-by cataloger list
>
> **The Reasoning cataloger is EXPLICITLY OUT OF SCOPE.** It depended on the local-LLM stack reverted in MIG-046/047/048. The 5-cataloger heuristic ensemble (User-Authority / Structural / Linguistic / Graph / Semantic) is what ships and what Bases queries. If Mind ever returns in a different form, Phase 2.7 may be revisited.
>
> **No freshness wrinkle** unlike the CNS Bridge — CECE classifications are persisted on disk (frontmatter) and in SQL (`sources_suggestions`). Bases reads at any time without staleness anxiety.
>
> **CECE's epistemic humility is preserved by Bases.** Notes in Split regime show "(disambiguating)" in the relevant cell, NOT a guessed value. The refusal is data; Bases displays the refusal honestly.
>
> ### The four-surface workflow (was three) — v1.4 §7.5
>
> Phase 1.5 now ships **three** threading gestures from every Bases row (was two):
>
> | Gesture | Routes to | What it shows |
> |---|---|---|
> | Open in **360.3D** | Per-note cognitive standing | Stratification Matrix — Position / Connection Profile / Absence |
> | Open in **CNS** | Per-note network neighborhood | Community, centrality, top-bridge role, blind-spot suggestions |
> | Open in **The Cataloger** | Per-note epistemic classification | Source × Content-type card, per-cataloger reasoning trail, disambiguation chips |
>
> The **four-surface workflow** Constellation uniquely enables:
> - **Bases** = comparison of many notes (surveying)
> - **360.3D** = cognitive standing of one note (cognitive depth)
> - **CNS** = network position of one note (network depth)
> - **The Cataloger** = epistemic classification of one note (epistemic depth)
>
> **No other PKM has all four. No other PKM threads them together with single-click row gestures.**
>
> ### The six Constellation Base differentiators (was five) — v1.4 §9
>
> 1. Living Links as query dimensions.
> 2. Summary headlines visible by default, context-aware rendering.
> 3. Federation across universes — auto by default.
> 4. Cognitive Engine measurements queryable across the collection (the 360.3D Bridge).
> 5. Network topology queryable across the collection (the CNS Bridge).
> 6. **Epistemic classification queryable across the collection** (the CECE Bridge, added v1.4). **The first PKM to make "where this knowledge came from" and "what kind of knowledge this is" filterable across the note collection.**
>
> ### Roadmap updated (Phase 2.7 inserted)
>
> - Phase 0 — Concept ✓
> - **Phase 1 — Rule 8 Migration (MIG-054).** Cheap-lookup `query_base` via SQL against `note_meta.properties_json`. Plus Q1 legacy `"vault"` retirement + Q2 explicit refresh event.
> - Phase 1.5 — Host-Note Assemblage + Open-in-360.3D + Open-in-CNS + **Open-in-Cataloger** gestures.
> - Phase 2 — Living Link Columns.
> - Phase 2.5 — Cognitive Engine Dimensions (360.3D Bridge).
> - Phase 2.6 — CNS Network Measurements (CNS Bridge). Includes freshness-strategy decision.
> - **Phase 2.7 — CECE Epistemic Classifications (the Cataloger Bridge).** §6.12 surface user-facing. 5-cataloger output queryable; no freshness wrinkle.
> - Phase 3 — NSC Headlines as Default Column.
> - Phase 4 — Federation Auto-On.
> - Phase 5 — Five Acts Templates.
> - Phase 6 — Semantic + Index Columns (the former "Cataloger" lump moved to Phase 2.7 per the promotion).
> - Phase 7 — Cell-Edit on Typed Links.
> - Phase 8+ — NL → query, generative lens suggestions, alternative renderers, Bases-driven 360.3D/CNS/Cataloger filtering.
>
> ### Architect doc landed as MIG-054
>
> `docs/MIG-054-bases-rule8-migration-ARCHITECT.md` (v1.1) supersedes the original MIG-049 draft (deleted in this commit). The Q1-Q5 answers are folded into §2-§7 (territory + design + steps). §10 is now "Decisions Locked" with the five resolutions. The Plan doc is the next deliverable in the cascade.
>
> ### Body-update debt
>
> §4582 (current Bases subsystem record) still says "5 commands" — actual is 10. Will be corrected in the §J PCS of the Phase 1 Build cascade (the orientation bump that lands with the build). §8 Migrations table needs a **MIG-054 row** (not MIG-049) added when the Architect doc is approved + Plan doc starts. This preamble is the trustworthy current state (SO #6).
>
> Session log: `lab/reports/SESSION-LOG-2026-05-25.md` (four blocks today — Mind ship AM, Mind revert PM, Bases Concept Paper v1.0→v1.3 evening, **Cataloger Bridge + Q1-Q5 lock late evening**). MoCh: `docs/MoCh/MoCh-2026-05-25-{0700,1500,1800,2100}.md`.
>
> ---
>
> *Below: the v2.35 preamble, retained for diff visibility. Note: v2.35's references to "five differentiators", "three threading gestures", "Concept Paper v1.3", and "MIG-049" reflect the state at the v2.35 PCS moment — superseded by this v2.36 preamble's six / three+Cataloger / v1.4 / MIG-054 values.*

**Version 2.35 | 2026-05-25**

> **What changed in v2.35** (Constellation Base Concept Paper landed — design phase opens; **9 design decisions locked**; Phase 1 Rule 8 migration architecture is the next deliverable):
>
> Today's third block (after the morning Mind ship + the afternoon Mind revert in v2.34) pivoted to a fresh-start design conversation on Constellation Bases. Result: a **four-version Concept Paper progression** and a **9-question decision lock** that sets up the next migration cascade. Design artifacts only — no code change.
>
> ### What was delivered
>
> - **Field research synthesis** — two parallel research agents mapped the PKM/PKF Bases-feature landscape (Obsidian Bases 2025 launch, Dataview, Notion databases, Logseq Datalog, Tana supertags + live search nodes, Anytype Sets/Collections, Capacities, Roam, Coda, AppFlowy, RemNote, Reflect, Mem.ai) plus the intellectual lineage (Steph Ango "files over apps", Zettelkasten / Matuschak / Doto tradition, "everything is a database" thesis from Tana/Capacities, Notion hegemony + local-first reaction, AI integration 2024-2026, Polanyi / Nonaka SECI on the "Formulation" lineage, contrarian voices including Tietze's Collector's Fallacy). Surfaced finding (Boss-relevant): **"Personal Knowledge Formulation" (PKF) appears to be Constellation's coinage** — no prior industry source found.
>
> - **Nine user-stickiness effects model** — 8 effects Constellation honors (dashboard / lens / aggregation / edit-in-place / externalized-self / anxiety-reduction / project-page / assemblage) + 1 explicitly refused: the **structure-invitation effect** — Tietze's Collector's Fallacy in product form. The refused effect is the Bases anti-design north star: *Bases reveals existing structure; never invites the user to invent it.*
>
> - **Constellation Base Concept Paper — four versions in one session, all preserved on disk:**
>   - `docs/Constellation-Base-Concept-Paper-v1.0.md` — pre-decisions draft. 15 sections, 8 open questions in §13.
>   - `docs/Constellation-Base-Concept-Paper-v1.1.md` — 7 of 8 closed. Headlines unconditional + context-aware rendering; all Living Link dimensions queryable; Federation auto-by-default; Wings integration bidirectional; Five Acts templates as both read-only system + editable copies; Cell-edit on typed links → Phase 7; Host-Note Assemblage accelerated to Phase 1.5.
>   - `docs/Constellation-Base-Concept-Paper-v1.2.md` — all 8 closed. **§6.10 added: the 360.3D Bridge.** Ten Cognitive Engine dimensions (Stratum / Maturity / Stage / Provenance / connection geometry / structural flags / review pulse / trail membership / word count) become Bases columns at Phase 2.5. §7.2 added: "Open in 360.3D" row gesture in Phase 1.5. The architectural fact that makes this cheap: *360.3D doesn't compute new metrics — it displays measurements already taken by the CE.*
>   - `docs/Constellation-Base-Concept-Paper-v1.3.md` — all 9 closed (the canonical version). **§6.11 added: the CNS Bridge.** Six CNS measurements (community / centrality / top-bridge flag + breadth / load-bearing / blind-spot participation) become Bases columns at Phase 2.6. §7.3 added: "Open in CNS" row gesture in Phase 1.5. **§7.4 names the three-surface workflow**: *Bases (surveying) → 360.3D (cognitive depth) → CNS (network depth).* Architectural wrinkle acknowledged: CNS metrics are graph-global, not per-note-cheap; freshness strategy (α debounced graph-write recompute / β CNS-open-cached / γ scheduled) deferred to Phase 2.6 Architect doc.
>
> ### The five Constellation Base differentiators (v1.3 §9)
>
> 1. Living Links as query dimensions.
> 2. Summary headlines visible by default, context-aware rendering.
> 3. Federation across universes — auto by default.
> 4. Cognitive Engine measurements queryable across the collection (the 360.3D Bridge).
> 5. Network topology queryable across the collection (the CNS Bridge).
>
> **The first PKM in the world** to make intellectual altitude AND synthesis points filterable across the collection rather than only visualizable in single-note or single-view surfaces.
>
> ### Roadmap locked
>
> - Phase 0 — Concept ✓
> - **Phase 1 — Rule 8 Migration (MIG-049)** — `bases_cache` table + triggers + cheap `query_base`. No new user-visible features beyond instant performance.
> - Phase 1.5 — Host-Note Assemblage + Open-in-360.3D + Open-in-CNS gestures.
> - Phase 2 — Living Link Columns.
> - Phase 2.5 — Cognitive Engine Dimensions (360.3D Bridge).
> - Phase 2.6 — CNS Network Measurements (CNS Bridge) — includes freshness-strategy decision (α/β/γ).
> - Phase 3 — NSC Headlines as Default Column.
> - Phase 4 — Federation Auto-On.
> - Phase 5 — Five Acts Templates.
> - Phase 6 — Semantic + Cataloger + Index Columns.
> - Phase 7 — Cell-Edit on Typed Links.
> - Phase 8+ — NL → query, generative lens suggestions, alternative renderers, Bases-driven 360.3D/CNS filtering.
>
> ### MIG-049 number reuse note
>
> MIG-049 was previously allocated to Mind Phase 2 (write tools + approval modal — designed in `Constellation-Mind-Implementation-Plan-v1.0.md`, never built; planned alongside MIG-050-053). Reused here for Bases Phase 1 since the Mind allocations 049-053 had no shipped code under those numbers (the reverted Mind work is 046-048, in the historical record). Architect doc to follow in a separate commit per `/migration` discipline.
>
> ### Memory updates
>
> - **New memory:** `memory/project_sight_map_disabled_2026_05_19.md` — canonical current-state record for Sight + Map being moved to Constellation Wings per MIG-038, 2026-05-19, commit `57cd7638`. Reason: Eisa's pivot back to founding mission (cultivate wisdom through Living Link system; visualizations are downstream readers, not core mission). All code intact for later detachment; flags off. **Plugin taxonomy:** **Core Plug-in** (Sky View / CNS / Index / CECE) vs **External Plug-in** (Sight / Map → Wings).
> - Older Sight memos preserved as historical record; new memo flags them as superseded.
> - `memory/MEMORY.md` updated.
>
> ### Body-update debt
>
> §4582 (current Bases subsystem record) still says "5 commands" — actual is 10. The Concept Paper §13 (v1.0) and the handover doc both flagged this drift; the Phase 1 (MIG-049) Architect doc will track the correction inline. §8 Migrations table needs a MIG-049 row added when the Architect doc lands. This preamble is the trustworthy current state (SO #6). Session log: `lab/reports/SESSION-LOG-2026-05-25.md` (three blocks today — Mind ship AM, Mind revert PM, Bases design evening). MoCh: `docs/MoCh/MoCh-2026-05-25-{0700,1500,1800}.md`.
>
> ---
>
> *Below: the v2.34 preamble, retained for diff visibility.*

**Version 2.34 | 2026-05-25**

> **What changed in v2.34** (**MIG-046 + MIG-047 + MIG-048 REVERTED — Constellation Mind paused at this stage**; the entire local-LLM stack (Fanar 1.9B + `mind/` module + chat surface + 6-tool dispatcher + citation validator + pre-warm) was removed from `main` after Eisa's Boss-test 2026-05-25 demonstrated that on CPU at 5 tok/s the value proposition didn't justify the speed cost. The cloud `ai/` bridge (Anthropic / OpenAI / OpenRouter) returns to being the only LLM surface; CECE's local-LLM stub returns to its pre-MIG-046 state):
>
> ### What was reverted (the headline)
> All of MIG-046 (inference abstraction skeleton), MIG-047 (real local inference via `llama-cpp-2` + Fanar 1.9B Q4_K_M), and MIG-048 (Phase 1 chat surface) — 30+ commits worth of work — collapsed into one revert commit. The reverted scope:
>
> - `src-tauri/src/mind/` — entire module deleted (provider trait, three stub providers, real `LocalProvider`/`LocalEmbeddingProvider`, model-install command family, `ChatOrchestrator`, telemetry, GBNF grammar, citation validator, history trim, prompt envelope, 6-tool dispatcher).
> - `src-tauri/build_assets/bench_runtime.rs` + `bench_tool_use.rs` — MIG-047 benches.
> - `src-tauri/resources/models.json` — bundled model catalog.
> - `src-tauri/Cargo.toml` — `async-trait`, `sha2`, `hex`, `llama-cpp-2` deps removed; `bench_*` `[[bin]]` entries removed.
> - `src-tauri/tauri.conf.json` — `bundle.resources` restored to `["models/*"]` (the ONNX embedding models; the bundled `resources/*` for Mind's models.json reverted).
> - `src-tauri/src/lib.rs` — `pub mod mind;` removed; 8 mind IPC commands de-registered; `.setup()` pre-warm hook removed.
> - `src-tauri/src/search.rs` — `constellation_search_recent` + `constellation_graph_neighbors` (MIG-048 §B) + `mig048_tests` removed.
> - `src/lib/components/Mind*.svelte` — 5 components removed (Settings + ChatPane + ChatMessage + CitationChip + ToolCallLog).
> - `src/routes/+layout.svelte` — chat sidebar mode + button + mount removed.
> - `src/lib/secondScreen.ts` — `'chat'` removed from `SidebarMode` union.
> - `src/lib/components/SettingsModal.svelte` — Mind section removed.
> - 15 locale `i18n` JSON files — `mind.chat.*`, `settings.mind.*`, `settings.sections.mind`, `navigator.chat` all stripped.
> - `.github/workflows/model-pipeline.yml` — quantization workflow removed.
> - `docs/MIG-046-*.md`, `docs/MIG-047-*.md`, `docs/MIG-048-*.md` — Architect docs.
> - `docs/Constellation-Mind-Concept-Paper-v1.0.md`, `v1.1.md`, `docs/Constellation-Mind-Implementation-Plan-v1.0.md` — design docs.
> - `docs/help.{15 locales}/Constellation Mind/` — help-doc topic deleted in all 15 locales.
> - **Local model file removed:** `%APPDATA%\world.uconstellation.app\models\fanar-1-9b-q4km-v1.gguf` (5.13 GB) + `installed_models.json` registry.
> - **GitHub release removed:** `models-fanar-1-9b-q4km-v1` tag + its 4 split-chunk artifacts (the file-split distribution from MIG-047 §A).
>
> ### What was KEPT (history value)
> - `lab/reports/SESSION-LOG-2026-05-24.md` (MIG-047 §J ship log) and `2026-05-25.md` (MIG-048 cascade + revert).
> - `lab/reports/MIG-046-*`, `MIG-047-*`, `MIG-048-*` bench + audit reports (the trail of what was tried and learned).
> - `docs/MoCh/MoCh-2026-05-2{3,4,5}*.md` (conversation history).
> - `docs/Constellation Orientation & Onboarding v2.30.md` through `v2.33.md` (immutable version history per SO #6).
> - `ai/mod.rs` — the cloud LLM bridge that pre-dated Mind. Untouched throughout MIG-046/047/048.
> - `cece/catalogers/reasoning.rs` — CECE's local-LLM stub. Untouched throughout.
> - `embeddings.rs` — ONNX embedding engine. Untouched throughout.
>
> ### Why
> Boss-test 2026-05-25 sequence:
> 1. NSIS installer built. Chat tab visible. Tool call rendered. Tokens streamed. Citations should have rendered (test universe lacked the expected English-language notes on "Canopus", which would have lived at the Arabic word "سهيل"; the cross-language lexical bridge isn't yet plumbed in `search_notes`).
> 2. **Speed:** first tool-call entry ~1 minute; full response ~2 minutes. CPU-only Fanar 1.9B Q4_K_M at 5 tok/s on the test prompt envelope (~1900 tokens decode at 50 tok/s prompt processing + ~200-token response at 5 tok/s sustained).
> 3. **App crashed on the third question** — likely still a context-window edge despite the `82d2cf91` n_ctx/batch fix and `765751c2` tool-result caps. The exact panic was never captured.
> 4. **Eisa's verdict (verbatim):** *"My general question: If the LLM is using the search_note tool, then what is special about it? I can search if I want to. It is NOT worth it."* — followed by *"Revert back to the prior LLM design and build. Constellation will not have it at this stage. Clean and do the necessary housekeeping."*
>
> Honest reading: the Phase 1 plumbing was correct (chat surface mounted, tool call dispatched, orchestrator drove the turn, citation discipline machinery worked) — but the value-add of an LLM-mediated search on top of the existing SearchHub didn't earn the ~70-second-per-turn cost when the same lookup was milliseconds in SearchHub. The cross-language synthesis case (Canopus ↔ سهيل) would have shown more value but Phase 1's `search_notes` tool didn't yet bridge it.
>
> ### Lessons preserved for any future re-attempt
> 1. **Local LLM at 5 tok/s CPU is too slow for interactive chat.** Any future Mind attempt needs GPU acceleration (CUDA / DirectML / Metal) OR a much smaller model OR a fundamentally different role for the LLM (background classification, batch summarization, async tool calls) rather than turn-by-turn chat.
> 2. **The value proposition needs a clear "what SearchHub can't do" demo.** Cross-language search, multi-tool synthesis, conversational refinement — those are where the LLM earns its cost. A keyword lookup demo undersells.
> 3. **`tauri::AppHandle` as a struct field breaks test-binary DLL load** (`STATUS_ENTRYPOINT_NOT_FOUND`). Use closure-based hooks instead. Caught in MIG-048 §G; documented in the audit report.
> 4. **GBNF eager grammar crashes llama.cpp on `prose | tool-call` alternation.** Use `grammar_lazy` with trigger word. MIG-048 §D.
> 5. **`LlamaBatch::new(N, 1)` capacity must match `n_ctx`.** Default 512 overflows on any real prompt with a system prompt + tool descriptions. MIG-048 fix `82d2cf91`.
> 6. **Tool result payloads can balloon the round-2 prompt past context.** Cap at the tool level (top-K, snippet length) AND at the dispatcher level (defense in depth).
>
> ### What remains intact
> - **`ai/mod.rs`** — the cloud LLM bridge for Settings → Intelligence. Anthropic / OpenAI / OpenRouter remain the LLM surface in Constellation as of v2.34.
> - **The Living Link Architecture, NSC, CECE, Sight, Constellation Map** — all pre-MIG-046 systems unchanged.
> - **Search (FTS5 + structured + semantic + hybrid)** — unchanged.
> - **The Concept Paper's ambition** is on hold, not invalidated. If Mind comes back, it'll need a different runtime story (GPU or cloud-routed) or a different role (background not interactive).
>
> ### Strategic next direction
> Pending Eisa's direction on what the next migration is. The Mind work is behind us; the Architect-Plan-Build-Audit machinery proved itself even though the feature didn't ship — that's the meta-takeaway.

> **What changed in v2.33** (**MIG-048 Phase 1 — SHIPPED end-to-end** — REVERTED in v2.34, preserved below for the history trail; Constellation Mind had a working chat surface in the left dock, a 6-tool dispatcher with GBNF-constrained tool calls, a canonical Arabic-first system prompt with citation discipline + MA-5 data-vs-instructions guard, post-stream citation validation with a 1-retry budget, app-start pre-warm (~10× first-turn latency win), sliding-window history trim, and a 15-locale i18n fill. This was the **first conversational moment** in Constellation's history — you could ask your Universe a question and get an answer with clickable citations back to the source notes):
>
> ### What shipped (the headline)
> A new **speech-bubble button** in the left sidebar mode row, between Digest and the OrgChart/SkyView dock-bar. Click it → the chat panel mounts. Type a question, hit Enter:
> 1. Your message appears as a purple bubble on the right.
> 2. Within ~1–2 seconds (thanks to pre-warm) a collapsed tool-call entry appears: `▸ Tool: search_notes "query":"Canopus"…` (or whatever tool Fanar chose).
> 3. After the dispatcher runs the real tool against `note_meta` / `note_links` / `note_summaries`, the result flows back into Fanar's history (framed as `<tool_result tool="..." id="...">…</tool_result>` per MA-5).
> 4. Fanar emits prose with `[note:<path>]` citations inline. The frontend splits on the citation regex and renders each as a clickable purple pill: 📎 NoteName.
> 5. Citation validator (Rust side, post-stream) scans the assistant text for `[note:<path>]` patterns, resolves each against `note_meta`. If any unresolved AND retry budget remains: re-prompts Fanar with feedback. If still unresolved after retry: prepends `⚠ This response contains N unresolved citations…` warning.
> 6. Click a citation pill → the cited note opens in the editor pane.
>
> ### Architectural surprises absorbed
> Phase 1 hit four unforeseen issues; all resolved in the same cascade:
>
> 1. **GBNF eager grammar crashed llama.cpp's grammar engine** (`GGML_ASSERT(!stacks.empty()) failed`) on the `prose | tool-call` alternation. Swapped to `LlamaSampler::grammar_lazy` with trigger word `{"tool":` — same UX (prose flows freely, grammar locks once trigger appears), no crash.
> 2. **`tauri::AppHandle` as a struct field broke the test binary's DLL load** with `STATUS_ENTRYPOINT_NOT_FOUND`. Switched the citation_validator wiring to a closure-based hook — orchestrator never holds AppHandle. Unit tests no longer need a Tauri runtime.
> 3. **`LlamaBatch::new(512, 1)` overflowed** with §F's system prompt in a real chat turn ("Insufficient Space of 512"). Caught by Eisa's first real Boss test. Bumped `n_ctx` 4096 → 8192 (Fanar's max) and batch capacity to match. Fix shipped as commit `82d2cf91`.
> 4. **Validator fail-CLOSED was a P1 caveat from the §M migration audit.** When the search DB is uninitialized (fresh install before the index opens), the validator marked every citation invalid. Introduced a 3-state `PathVerdict` enum (Exists / Missing / Unverifiable); `Unverifiable` now treats the citation as valid (fail-open). Validator catches only REAL fabrications.
>
> ### Phase 1 shipped state — by commit
> - **Architect** `9cd41e4a` — §9 locks: D1 left dock, C1 1-retry, E2 sliding window.
> - **§A** `a0fe99fe` — RealToolDispatcher + 4 ready tools.
> - **§B** `853dac1b` — constellation_search_recent + constellation_graph_neighbors pub fns.
> - **§C** `a2f28b4f` — list_recent + graph_neighbors tools wired (6 total).
> - **§D** `a282d1e4` — GBNF tool-call extraction via grammar_lazy; bench 7/10.
> - **§E** `cb642111` — mind_start_turn → ChatOrchestrator; UiEvent → StreamEvent bridge.
> - **§F** `294b21ea` — canonical system prompt + `<tool_result>` framing.
> - **§G** `cfa10802` — Citation validator + 1-retry loop (Eisa-locked C1).
> - **§H** `a08a3d75` — 4 Svelte chat components.
> - **§I** `f665e937` — chat sidebar mode (Eisa-locked D1: left dock).
> - **§J** `909bd8aa` — pre-warm on app-start + active-model change.
> - **fix** `82d2cf91` — n_ctx + batch capacity bump (Boss-test §I).
> - **§K** `d71642f0` — sliding-window history trim (Eisa-locked E2).
> - **§L** `c62bb74b` — 13-locale i18n fill.
> - **§M** `4e978076` — 3-agent audit consolidated + validator fail-open fix.
> - **§N** (this commit) — orientation v2.33, MoCh, session log, EN+AR help-doc chat section.
>
> ### Phase 1.x polish queue (not ship-blockers)
> 1. Brand-naming policy for 13 locales — current hybrid vs Arabic-aligned full localization.
> 2. Real tokenizer for history.rs trim budget (currently chars/4).
> 3. Per-Universe conversation persistence (state resets on unmount today).
> 4. m1 two-tool-call edge case (Fanar occasionally emits two adjacent tool calls).
> 5. 13-locale help-doc fill for the chat section (EN + AR shipped this round).
>
> ### Boss-test Stage 1 (Phase 1 ship gate)
> 20-turn Arabic conversation on Eisa Cognitive Knowledge universe; Eisa reads a 50-turn sample; **target ≥90% citation faithfulness**. Pass → Phase 1 closes fully; Phase 2 (MIG-049 — write tools + approval modal + diff preview + undo journal) opens. Fail → §G citation-validator + §F system-prompt iteration before re-running.

> **What changed in v2.32** (**MIG-047 Phase 0b — SHIPPED end-to-end**; Constellation Mind has its first real Arabic-generating model on disk, the Boss-test Stage 0 prompt `مرحبا، كيف حالك؟` round-trips through real `llama-cpp-2` inference, and the runtime story is consolidated to one runtime for Fanar + future Jais. This is the **first user-facing inference moment** in Constellation's history):
>
> ### What shipped (the headline)
> Open a Rust binary on the dev machine, hand it the 5 GiB Fanar Q4_K_M GGUF, and watch real Arabic come back:
>
> ```
> Prompt:   مرحبا، كيف حالك؟
> Response: مرحباً! أنا بخير، شكرًا لك على سؤالك. كيف يمكنني مساعدتك اليوم؟ 😊
> ```
>
> Coherent native Arabic, polite register, emoji acceptable. Warm first-token 1.25s; sustained 5.3 tok/s on CPU-only Q4_K_M. Bench reports at `lab/reports/MIG-047-bench-runtime-2026-05-24.md` and `lab/reports/MIG-047-bench-tool-use-2026-05-24.md`.
>
> ### Two architectural surprises absorbed (Path A pivot)
> The Phase 0b cascade hit two unforeseen issues that the Architect §3 invariants didn't predict — both within the §4 A risk envelope Eisa accepted; both resolved in the same session:
>
> 1. **mistral.rs 0.8.1 panics on `gemma2` GGUF.** §C-v1 shipped a LocalProvider backed by `mistralrs = "0.8.1"`. First real model load died at `mistralrs-core/src/gguf/content.rs:151:22` with "Unknown GGUF architecture `gemma2`". Mistral.rs's README listing of "Gemma 2" is the safetensors path; its GGUF loader has no gemma2 handler. Fanar's quantized GGUF advertises gemma2 (it's a Gemma-2-9B derivative), so it never loaded.
>    - **Eisa's Path A decision (commit `a5035795` precedes; pivot commits at `3af95e0b`):** swap to `llama-cpp-2` — the same llama.cpp release (b6285) that quantized Fanar in our workflow. This eats the CECE V3-§7 deferred Windows-MSVC cmake risk now, but consolidates to one runtime for Fanar (now) AND Jais (Phase 2.5 / MIG-050). The Plan §1 Decision #4 micro-bench between runtimes is collapsed — we ship with the one runtime that actually loads our model.
>
> 2. **llama-cpp-2's bindgen needs libclang on Windows.** The runtime swap built fine on dev machine UNTIL the link step, where bindgen-0.72.1 panicked because `clang.dll` / `libclang.dll` weren't on disk. Windows MSVC alone doesn't ship libclang.
>    - **Resolution:** Eisa ran `winget install LLVM.LLVM` (5 min install). LLVM 22.1.6 went to `C:\Program Files\LLVM\bin\`. The Cargo.toml comment on the `llama-cpp-2` dep now documents this as a one-time per-dev-machine setup. Build went green in 4m28s (dev) / 5m02s (release).
>
> ### Phase 0b shipped state — by commit
> - **§A** (`a6e35b5a`) — `.github/workflows/model-pipeline.yml` + `src-tauri/resources/models.json` (placeholder SHA-256). Took **5 workflow attempts** to land cleanly (huggingface-cli was removed → `hf` CLI; `hf` stalled unauthenticated → `snapshot_download`; that SIGTERM'd on large files → sequential per-file `hf_hub_download` with `max_workers=1` and Xet disabled; `convert_hf_to_gguf.py` needed `mistral_common` dep; attempt #5 shipped clean producing 4 split chunks at 5.01 GiB total). The pipeline is now a working reusable mechanism for any future model.
> - **§D** (`634327fd`) — `LocalEmbeddingProvider` wrapping the existing `embeddings.rs` ONNX pipeline.
> - **§E** (`f17a4459`) — `mind/model_install/` complete: 4 IPC commands (`mind_install_model`, `mind_list_installed_models`, `mind_active_model`, `mind_set_active_model`), 5th added later (`mind_list_catalog`). Chunked download + SHA-256 verify + atomic registry write. 14 new unit tests.
> - **§F** (`39fa7258`) — `MindSettings.svelte` (Settings → Mind sidebar entry between Intelligence and Security). EN + AR i18n shipped; 13 other locales fall back via `||` pattern.
> - **§C-v1** (`54c49c43`) — mistral.rs LocalProvider. **Subsequently superseded by §C-v2 due to gemma2 panic; kept in git history for traceability.**
> - **§G** (`62a5a842`) — `mind_start_turn` IPC loads the active model via the install registry, instantiates real LocalProvider, drives a real turn.
> - **§B+H** (`c6e075b7`) — `bench_runtime` + `bench_tool_use` `[[bin]]` targets; `pub mod mind;` widening so the bins reach into the library.
> - **§I** (`2648a6bd`) — 3-agent audit; INV-8 false-positive resolved (backend reqwest doesn't need Tauri capability scoping; existing `ai/mod.rs` pattern confirms).
> - **§A close** (`a5035795`) — `models.json` SHA-256 populated from workflow run `26364885496`. The Tauri-side install command now passes `is_ready_to_install()`.
> - **§C-v2** (`3af95e0b`) — runtime swap to `llama-cpp-2`. Full `local.rs` rewrite (lazy `OnceCell<Arc<LlamaModel>>`, blocking-pool inference, Gemma-2 chat template, sampler chain). `manifest.rs` test inverted from "TBD-asserter" to "ready-asserter". 38/38 mind unit tests pass.
> - **§J** (this commit) — orientation v2.32, MoCh, session log close, EN + AR help-doc topic. 13 other locale help docs in a background translation agent landing in a follow-up commit; Phase 1 (MIG-048) chat surface is where locales really matter for users.
>
> ### Boss-test Stage 0 verdict
>
> | Reading | Result |
> |---|---|
> | Strict (cold-load) | ⚠️ FAIL — Run 1 first-token 11s > 5s gate |
> | Warm-cache (production UX) | ✅ PASS — Run 2 first-token 1.25s |
> | Sustained Arabic generation | ✅ PASS — 5.3 tok/s; correct Arabic |
> | Tool-call extraction | ⚠️ 0/10 — structural gap; Phase 1 (MIG-048) wires GBNF grammar |
>
> Cold-load mitigation is a Phase 1 UX item: pre-warm the active model in a background tokio task on app start. Doesn't change the runtime.
>
> ### What's installed locally now
> - `src-tauri/target/release/bench_runtime.exe` (25 MB)
> - `src-tauri/target/release/bench_tool_use.exe` (25 MB)
> - `/tmp/fanar-install/fanar-1-9b-q4km.gguf` (5.01 GiB; SHA-256 matches the manifest exactly)
> - LLVM 22.1.6 at `C:\Program Files\LLVM\` (one-time per-dev-machine; needed for bindgen)
> - The Tauri app itself: when next launched in dev mode, Settings → Mind shows Fanar with the "Available" badge and a working Install button.
>
> ### Pending follow-ups (small + clearly scoped)
> 1. **13-locale help-doc translations** — translation agent running in background; lands as a single follow-up commit when done.
> 2. **Pre-warm on app start** — Phase 1 (MIG-048).
> 3. **Tool-call extraction in `local.rs`** — Phase 1 (MIG-048); likely GBNF grammar constraining JSON output.
> 4. **Phase 2.5 jais entry in `models.json`** — when Jais llama.cpp upstream support is verified for the `2-8b` size specifically.
>
> ### Body-update debt
> §4 subsystem map will gain a `mind/providers/local.rs` row (llama-cpp-2-backed); §8 Migrations table gains MIG-047 §A through §J rows. This preamble is the trustworthy current state (SO #6). Session log: `lab/reports/SESSION-LOG-2026-05-24.md` (Phase 0a + Phase 0b shipped in one continuous day). MoCh for the Phase 0b block: `docs/MoCh/MoCh-2026-05-24-1500.md`.
>
> ---
>
> *Below: the v2.31 preamble, retained for diff visibility.*

**Version 2.31 | 2026-05-24**

> **What changed in v2.31** (**MIG-046 Phase 0a — SHIPPED**, all six Build steps (A–F) landed clean; PCS gate awaits Eisa's explicit go before push; the trait surface is now the strategic moat the Concept Paper called it):
>
> ### Phase 0a complete — `src-tauri/src/mind/` is real
> Six commits between `29459ed0` (kickoff) and `e7e3dab2` (§F audit), all green on `cargo build` + `cargo test --lib mind::` (18/18 passing):
> - **§A** (`e2df4a69`) — trait scaffolding: `InferenceProvider` (generate / classify / capabilities) + `EmbeddingProvider` (embed / embed_capabilities) split per MA-1. `StreamEvent` discriminated union (Token / ToolCall / Done / Error). Supporting types: `ChatMessage`, `ChatRole`, `GenParams`, `ToolSchema`, `ToolChoice`, `FinishReason`, `TokenUsage`, `ProviderCapabilities`, `EmbeddingCapabilities`, `InferenceError` (manual `Display`, no `thiserror`). `async-trait = "0.1"` added to `Cargo.toml` — the one new dep, justified inline against Architect §3 invariant 3 ("heavy" means `mistral.rs` / `llama-cpp-2` / `candle`, not a proc macro).
> - **§B** (`d2b4b944`) — three deterministic stub providers (`LocalProvider`, `CloudProvider`, `OfflineProvider`) under `mind/providers/`. `LocalProvider` implements both traits (384-dim zero-vector embedding to match `multilingual-e5-small` for future swap-in). Tool-call uses **Pattern B** (generate-restart, matches Anthropic HTTP API). 10 unit tests.
> - **§C** (`2e432a61`) — Tauri IPC: `mind_start_turn(request, on_event: tauri::ipc::Channel<StreamEvent>)` + `mind_telemetry_snapshot() -> TelemetrySnapshot`. Wired in `lib.rs:invoke_handler!` alongside untouched `ai::*` entries. `mind/telemetry.rs` scaffold (real counters land in §E). 11 tests.
> - **§D** (`b3dae04b`) — `ChatOrchestrator` skeleton. The `turn()` loop wraps the Concept-Paper-v1.1-§10.3 single-iteration snippet in an outer `loop { stream = generate(); … }` (Pattern B). `tool_rounds` counter + MA-4 budget abort path (`max_tool_rounds_per_turn` default 5 — synthetic `aborted_tool_budget_exceeded` tool_result injection). `framing::as_tool_result` central sanitizer (MA-5 placeholder, no-op pass-through in 0a; Phase 1 swaps for real `<tool_result>` framing). `LoopingToolCallProvider` test helper proves the budget terminates. 15 tests.
> - **§E** (`1f5f64ce`) — real telemetry atomics. `TelemetryCounters` (AtomicU64 numerics + Mutex<String> for provider/model id) + `OnceLock<Arc<TelemetryCounters>>` global for the IPC. Orchestrator holds `Arc<TelemetryCounters>` (defaults to global; tests inject via `.with_counters()` for isolation). `turn()` records: `set_active_provider`, `record_tool_call`, `record_budget_exceeded` (once per turn), `record_error`, `record_turn(latency_ms, in, out)`. 18 tests.
> - **§F** (`e7e3dab2`) — 3-agent audit (4A invariants / 4B drift / 4C migration-path), all PASS. All 8 invariants from MIG-046 §3 hold with file:line evidence; 8 drift vectors clean; 6 migration scenarios PASS. One audit-framing self-correction recorded (range `e2df4a69..HEAD` was left-exclusive on §A; agents read post-§A state directly so findings held). No code fixes surfaced.
>
> ### Coexistence with the four existing intelligence surfaces — confirmed clean
> Audit verified: `ai/mod.rs`, `cece/catalogers/reasoning.rs`, `embeddings.rs`, `nsc/` all untouched. `git diff` confirms zero changes outside `src-tauri/src/mind/` + `src-tauri/src/lib.rs` (`mod mind;` declaration + two `invoke_handler!` entries) + `src-tauri/Cargo.toml` (the one `async-trait` line). Frontend `src/` zero diff.
>
> ### What Phase 0a explicitly did NOT do (per Architect §8)
> - No real LLM inference (no `mistral.rs`, no `llama-cpp-2`, no Anthropic HTTP calls)
> - No model download / install flow (Phase 0b)
> - No real retrieval (chunks empty in 0a; Phase 1 wires `HybridRetriever`)
> - No real tool dispatcher (canned `{"status":"ok"}` placeholder; Phase 1 read tools, Phase 2 write tools)
> - No frontend chat UI (Phase 1)
> - No `RoutedProvider` implementation (Phase 2.5 — but trait surface admits it cleanly)
>
> ### PCS gate
> All commits are local on `main` (1612 commits ahead of `origin/ConstellationMain`). **Nothing has been pushed.** Eisa decides when to push the Phase 0a bundle.
>
> ### Boss decisions awaiting before Phase 0b (MIG-047)
> Four open questions from PF-1 §10.4 of the Implementation Plan need to settle before Phase 0b locks bundled-default model identity:
> 1. **Gemma upstream** — accept QCRI's Apache-2.0 relabel or also ship Gemma notices defensively? *Recommended (b): defensive.*
> 2. **Fanar GGUF source** — in-house quantization from official safetensors or depend on `mradermacher/Fanar-1-9B-i1-GGUF`? *Recommended: in-house for bundled-default.*
> 3. **Jais HF gate** — drop from co-default for v1 / require token paste / Constellation-hosted mirror? *Recommended (a): drop from co-default for v1, revisit (c) after talking to Inception.*
> 4. **Attribution placement** — Settings → About panel is conventional.
>
> ### Body-update debt
> §4 subsystem map will gain a `mind/` entry; §8 Migrations table gains MIG-046 row. This preamble is the trustworthy current state (SO #6). Session log: `lab/reports/SESSION-LOG-2026-05-24.md`. MoCh: `docs/MoCh/MoCh-2026-05-24-1200.md`.
>
> ---
>
> *Below: the v2.30 preamble, retained for diff visibility.*

**Version 2.30 | 2026-05-24**

> **What changed in v2.30** (**Constellation Mind workstream begins** — Pre-flight closed + MIG-046 Phase 0a Architect approved; Build cascade now in flight. This is the **second major subsystem** entering active development since NSC Core Plug-in shipped at v2.29):
>
> ### Constellation Mind — Concept Paper v1.1 + Implementation Plan v1.0 are the durable references
> - **Concept Paper v1.1** (`docs/Constellation-Mind-Concept-Paper-v1.1.md`) — refined post-planning from v1.0. Folds six must-address items (MA-1..6), promotes the **`RoutedProvider`** pattern to a first-class architectural layer (new §5.9 principle, §6.1 diagram, §10.2 implementation with `Router` trait + `RuleRouter` v1 + `LoadStrategy`, new Phase 2.5), adds §9.5 Bundling Decision Matrix, adds R13 (tool-call loops) + R14 (prompt injection) to the risk register. v1.0 retained alongside as historical record.
> - **Implementation Plan v1.0** (`docs/Constellation-Mind-Implementation-Plan-v1.0.md`) — locked decisions from the planning conversation (laptop loading = hot-swap default + Performance Mode opt-in; first-launch download with size disclosure; sequential phases; `mistral.rs` vs `llama-cpp-2` decided by Phase 0b micro-bench; first-model bundled-default decided by Phase 0b tool-use bench). Phase 0a (MIG-046, this) → Phase 0b (MIG-047) → Phase 1 (MIG-048) → Phase 2 (MIG-049) → Phase 2.5 (MIG-050) → Phase 3 (MIG-051) → Phase 4 (MIG-052) → Phase 5 (MIG-053). Phase 6 (federated cUniverse Ask-Across) deferred to research mode 2026 H2.
> - **PF-1 license verdict** (Plan §10): **Fanar-1-9B** is a continued pretraining of `google/gemma-2-9b` — QCRI declares Apache-2.0, doesn't acknowledge upstream Gemma Terms. Verdict GO-with-conditions (defensive Gemma notices + in-house quantization). **Jais-2-8B-Chat** is Apache-2.0 itself but **gated on Hugging Face** — both safetensors and official GGUF require login + contact-info agreement. Verdict GO-with-conditions (drop from co-default for v1; user-installable; revisit Constellation-hosted mirror after talking to Inception). No cross-license conflict for RoutedProvider. Four open questions at Plan §10.4 affect Phase 0b, not 0a.
>
> ### MIG-046 — Phase 0a (Inference Abstraction Skeleton) Architect approved
> Architect doc: `docs/MIG-046-constellation-mind-phase0a-inference-abstraction-ARCHITECT.md`. Goal: lock the trait surface before paying real-inference cost. New `src-tauri/src/mind/` module, parallel to the existing `ai/`, `cece/`, `nsc/`, `embeddings.rs`. Split into two traits (`InferenceProvider` + `EmbeddingProvider`, MA-1), three deterministic stub providers (`LocalProvider` / `CloudProvider` Anthropic-shaped scaffold / `OfflineProvider`), `tauri::ipc::Channel<StreamEvent>` IPC contract, `ChatOrchestrator` skeleton, in-process telemetry counters. **No real models, no `mistral.rs` / `llama-cpp-2` dependency in 0a, no frontend** — that's Phase 0b / 1.
>
> Plan outline §5 = seven steps (A–G): trait scaffolding → stub providers + tests → Tauri IPC + Channel → orchestrator skeleton → telemetry → /simplify + 3-agent audit → SO + PCS gate. Phase 0a has **no Boss-test gate** (Plan §4: "No Boss test yet"); first user-testable gate is Phase 0b. Risk: low — strictly additive; zero new `Cargo.toml` deps; rollback is `rm -rf src-tauri/src/mind/` + three `invoke_handler!` lines.
>
> ### Coexistence map for the four existing intelligence surfaces (Phase 0a touches none)
> - `src-tauri/src/ai/mod.rs` — cloud bridge with `ai_send_message` / `ai_validate_connection` / `ai_list_models` (sole frontend consumer: `src/lib/ai/engine.ts`). OpenAI / Anthropic / Gemini / Ollama. Non-streaming. **Phase 5 (MIG-053)** eventually refactors this into a `CloudProvider` impl of the new `InferenceProvider` trait. 0a leaves it untouched.
> - `src-tauri/src/cece/catalogers/reasoning.rs` — MIG-021v3 CECE Reasoning Cataloger with its own `InferenceFn = Box<dyn Fn(prompt, grammar) -> String>` injection point (Qwen3-4B planned via `llama-cpp-2`; presently unwired — abstain path). **Phase 3 (MIG-051)** rewires CECE's local-LLM call through the new `RoutedProvider`. 0a designs the trait surface to admit this adapter without breaking changes.
> - `src-tauri/src/embeddings.rs` — `ort` + `tokenizers` + `multilingual-e5-small` (384-dim, 100 languages). Natural future home for `LocalEmbeddingProvider`. 0a stubs the trait shape only; refactor deferred.
> - `nsc/` (MIG-040..045) — Phase 1 (MIG-048) wires the `summarize` tool to delegate to `getSummariesFor` (MA-3). 0a's canned tool dispatcher returns `{status:"ok"}` for every tool name.
>
> ### Body-update debt
> §4 subsystem map will gain a `mind/` entry as Phase 0a Steps land; §8 Migrations table gains MIG-046 row. This preamble is the trustworthy current state (SO #6). Session log: `lab/reports/SESSION-LOG-2026-05-24.md` (Constellation Mind kickoff).
>
> ---
>
> *Below: the v2.29 preamble, retained for diff visibility.*

**Version 2.29 | 2026-05-24**

> **What changed in v2.29** (MIG-045 Phase 3 — **the Universe Digest left-dock view — SHIPPED + Boss-validated end-to-end**; with this commit, all three pillars of the NSC Core Plug-in roadmap from Concept Paper v2.0 are live):
>
> ### MIG-045 Phase 3 — the Universe Digest pane
> A new left-dock pane (`src/lib/components/DigestPane.svelte`, mounted as the 4th sidebar mode tab at `src/routes/+layout.svelte:4651`) that lets the user skim the WHOLE knowledge base at summary-headline level without opening any notes. Tiered **Library → Folder → Note**, default sort by recency, click chevron or headline to expand inline to the full multi-sentence summary, click the name to open the note. Filter input narrows on `name + headline + full summary` (instant; no IPC). Virtualized via `VirtualList.svelte` so 7,600+-note Universes scroll at 60fps.
>
> **Frontend-only MIG**: zero Rust changes, zero schema deltas, zero new IPC. The Digest reads through the shared `summaryStore` from Phase 1 (`src/lib/nsc/summaryStore.ts`) — bundle-verified `getSummariesFor(` call count is now 4 (NoteEditor + LocalSkyView + GraphMindView + DigestPane), `getSummaryFor(` is 3, plus the 5 list-panels using `getSummariesFor` for batched fetches.
>
> ### Audit caught and fixed BEFORE Boss test (better than Phase 2's saga)
> 2-agent audit (invariants+drift / migration-path) surfaced two real issues that were fixed before shipping the install:
> - **Rule-2 violation** — the headline-fetch `$effect` read `summaryHeadlines` / `fullSummaries` via `new Map(...)` AND wrote to them, with `passesFilter` (used in the `treeRows` derivation) also reading them. Without `untrack`, the writes would re-fire the effect through the `treeRows` dep cycle. Fix: wrap the effect body in `untrack(() => { ... })` (same shape as `IndexPanel.svelte:90-101`). Reads now happen inside `untrack`; the only tracked dep is `treeRows`.
> - **a11y nested-interactive** — the row was a `<div role="button">` containing a `<button>` for the note name. Browsers "repair" nested buttons (Svelte's compiler warned, audit caught it). Fix: row is now a plain layout `<div>` (no role) with TWO sibling `<button>` elements — a chevron-only button (toggles expand) and a name button (opens note). The headline italic line is also a button (clicking it ALSO toggles expand — gives the user a wider expand target). Three real interactives, no nesting, keyboard-accessible.
>
> ### Out of v1 scope (deferred — frontend-only invariant)
> - **cUniverse on/off toggle.** Identifying which library belongs to a child universe vs the current one requires a `universe_id` field on `LibraryInfo` that doesn't exist (and adding it is a Rust change, violating MIG-045's frontend-only invariant). **Federation still works** — child-universe libraries appear inline as peer top-level rows via the existing `resolve_libraries_recursive` flatten — they just can't be hidden via a UI toggle. A follow-up MIG can add `universe_id` and surface the toggle then. Documented inline in `DigestPane.svelte:43-50` + the architect doc §3 / §5.
> - **Right-click context menu, custom groupings, drag-to-reorder.** Per the architect doc §3 — v1 keeps the gestures simple (chevron expands, name opens). Future MIGs can add these.
>
> ### Boss test
> Stage 1 (8 tests) PASS on Eisa Cognitive Knowledge live universe — dock entry, open pane, tiered list renders, headlines fill in, click-to-expand, click name opens, filter narrows correctly (incl. multi-tier collapse of empty headers), sort toggle switches recency↔alphabetical, scroll smooth on 7,600+ note library.
>
> ### Help docs in 15 locales
> - **New help topic:** `docs/help.*/The Digest/The Digest.md` in all 15 locales — what the Digest is, why it exists, all 8 surfaces it complements, full UX walkthrough, common workflows, what's NOT in v1. English written; 14 other-locale translations dispatched in parallel using each locale's established native term (الموجز / 宇宙摘要 / Digesto / etc.).
> - **Note Summaries help** expanded in all 15 locales: 8th surface (Universe Digest) added to the "Where summaries appear" list + intro paragraph + frontmatter description + closing "lazy-fill" paragraph.
> - **i18n strings**: new `digest.*` block + `navigator.digest` key added to `en.json`; 14 other locale `.json` files updated via the same translation agent.
>
> ### The whole NSC Core Plug-in roadmap is now SHIPPED
> Three months of work, three MIGs:
> - **MIG-043** (Phase 1, 2026-05-23): engine `headline` variant + shared frontend `summaryStore` + 2 first surfaces (Cataloger refactor as no-behavior-change + Search results + Editor band).
> - **MIG-044** (Phase 2, 2026-05-23): 5 remaining surfaces wired (BacklinksPanel, OutgoingLinksPanel, IndexPanel, LocalSkyView, GraphMindView). 3 wrong-target wirings caught during Boss test arc → LL-028 + LL-029 filed.
> - **MIG-045** (Phase 3, 2026-05-24): the Universe Digest pane itself. Both audit issues caught + fixed BEFORE Boss test — proof that the LL-028/029 discipline (bundle-grep verify, grep-import-before-edit, untrack-effect-writes) compounds.
>
> ### Body-update debt
> The §4 subsystem map + §8 Migrations table still describe pre-MIG-045 state; this preamble is the trustworthy current state (SO #6). Session log: `lab/reports/SESSION-LOG-2026-05-23.md` (Phase 2 + 3 share the daily log — the work crossed into 2026-05-24 in flight but is one continuous narrative).
>
> ---
>
> *Below: the v2.28 preamble, retained for diff visibility.*

**Version 2.28 | 2026-05-23**

> **What changed in v2.28** (MIG-044 Phase 2 — NSC Core Plug-in **full service reach** — **SHIPPED**; the four remaining note-displaying surfaces now show summary headlines, completing the "service feeding every Constellation function" half of the Concept Paper v2.0 vision):
>
> ### MIG-044 Phase 2 — four surfaces wired (frontend-only MIG)
> - **Backlinks panel** (`src/lib/components/BacklinksPanel.svelte`) — a faint italic `.bl-headline` line under every linked-mention row AND every unlinked-mention row. `$effect` over `filteredBacklinks` + `filteredUnlinked` → batched fetch via the shared store with the Phase-1 `changed`-guard merge pattern.
> - **Outgoing links panel** (`src/lib/components/OutgoingLinksPanel.svelte`) — same shape, target-keyed `summaryHeadlines` Map. The wrinkle: `NoteLink` carries no `target_path` (only the wikilink string), so the panel does a parallel `Promise.all` of `resolveWikilinkCrossLibrary` on visible targets BEFORE the batched summaries fetch. The resolve fires only when `outgoingLinks` ref changes (parent tab switch) — NOT on every render. (A future Rule-8 cleanup could persist `target_path` on `note_links` rows; deferred — outgoing lists are typically small.)
> - **Index panel** (`src/lib/components/IndexPanel.svelte`) — `.gp-ref-headline` under each note-mention row when a term is expanded. `$effect` tracks `mentionsCache.size` (gated by term expansion); body wrapped in `untrack` per the existing Rule-2 discipline. New `ROW_HEIGHT_MENTION_HEADLINE = 16` reserved in the virtualized row-height calc ONLY when a headline is loaded — rows without headlines stay compact. `void summaryHeadlines.size` added inside the `rows` `$derived.by` so VirtualList sees a new prop ref and re-measures.
> - **Sky View hover — wired in TWO components**: `LocalSkyView.svelte` (embedded right-side panel) AND `GraphMindView.svelte` (main full-window mode, mounted at `+layout.svelte:5331` when `showSkyView` is true). The earlier preamble draft + the architect doc both wrongly named `SkyView.svelte`/`FullSkyView.svelte` as the targets; both are dead code (no static importer in `src/`; Vite tree-shakes them). The misdirection was a Predecessor Lookup Rule violation caught across two Boss test cycles:
>   - **Stage 1-B fail:** Eisa hovered in the embedded panel → no tooltip. Grep revealed `LocalSkyView` is the only component actually imported (despite the misleading "Local" prefix). Wired it with namespaced `.local-star-tooltip{,-name,-headline}` classes + edge-aware positioning + 240px max-width + 3-line `line-clamp`.
>   - **Stage 1-D ask:** Eisa asked "what about the main SV?" → grep revealed the full-window mode is `<GraphMindView>` (NOT a `*SkyView*`-named file at all; the original grep missed it because it pattern-matched on filenames). Wired GraphMindView the same way: hooks the existing `onNodeHover` callback for headline fetch, adds container-level `onmousemove` for cursor coords + edge-flip, same two-line tooltip shape with `.gm-tooltip*` classes, max-width 280, 3-line line-clamp.
> - **Bundle verification (mandatory after LL-028):** `getSummaryFor(` call count in bundled JS = **3** — NoteEditor's editor band (MIG-043) + LocalSkyView's tooltip + GraphMindView's tooltip. Exactly the three live UI consumers.
> - See §4 of `lab/reports/SESSION-LOG-2026-05-23.md` for the full incident trail (5 build attempts; 3 wrong-target wirings; LL-028 + LL-029 filed).
>
> ### Out of Phase 2 scope
> - **`SkyView.svelte` and `FullSkyView.svelte`** — dead code; no importer in `src/`. Worth a future cleanup MIG to delete them (would reduce confusion + bundle size).
> - **Second-screen mount of LocalSkyView** — same component but rendered in the second-screen window. The tooltip will work there too automatically (single source of truth), but if Eisa wants the tooltip suppressed on the display screen, that's a separate feature.
> - **Map** — disabled (MIG-038). Skip.
> - **Hover / wikilink-previews** — no such surface exists in the codebase (grep-confirmed). If one is added later, wire it under its own MIG.
>
> ### Audit + tests
> - 3-agent audit (invariants / drift / migration-path) all clean. **All 8 invariants of MIG-044-ARCHITECT §3 HOLD.** No `$effect` loops (IndexPanel uses `untrack` per established pattern; the others don't touch the same `$state` they read). No new IPC, no schema change, no Rust delta. Rollback to MIG-043 reverts cleanly (additive diffs, guarded `{#if summaryHeadlines.get(...)}` everywhere). VirtualList re-measure verified — `getRowHeight` reads `summaryHeadlines` so Svelte 5 tracks it as a `heights` dep automatically; the `rows`-derive `void` is belt-and-suspenders. svelte-check: 3 pre-existing errors, **0 new, warnings dropped by 1**.
>
> ### Help docs in 15 locales
> - English **Note Summaries** help expanded — `description:` frontmatter now enumerates all 7 surfaces (Cataloger, Source Review, Search results, Editor, Backlinks, Outgoing links, Index, Sky View hover); intro paragraph rewrites "in three places" → "across the whole app" with per-surface role-descriptions; `## Where summaries appear, and how they fill in` section grows from 3 bullets to 7 + the "lazily and gently" paragraph updates to include the new gestures (expand-term, hover-bubble). 14 other-locale Note Summaries files translated to match via a background sub-agent using each locale's established native term (المُصنِّف / 分类器 / Klassifikator / etc.).
>
> ### Roadmap after Phase 2
> - **MIG-045 (Phase 3)** — the **Universe Digest** left-dock view itself: tiered Library → Folder → 1-line headline, expandable to full summary, recency-sorted, searchable, virtualized, with cUniverse-children federated in. The biggest of the three NSC Core Plug-in MIGs. Will go through full `/migration` (Architect → Plan → Build → Audit → PCS).
>
> ### Body-update debt
> The §4 subsystem map + §8 Migrations table still describe pre-MIG-044 state; this preamble is the trustworthy current state (SO #6). Session log: `lab/reports/SESSION-LOG-2026-05-23.md` (this MIG appended to the existing day's log).
>
> ---
>
> *Below: the v2.27 preamble, retained for diff visibility.*

**Version 2.27 | 2026-05-23**

> **What changed in v2.27** (MIG-043 Phase 1 — NSC Core Plug-in foundation — **SHIPPED + Boss-validated end-to-end on the live universe**; the cascade Eisa requested at v2.26 closed cleanly across the midnight):
>
> ### MIG-043 Phase 1 ships — the NSC service foundation + first two surfaces
> - **Engine `headline` variant + additive schema** (`src-tauri/src/nsc/mod.rs`). `NoteSummary` and `NoteSummaryEntry` grew a `headline: String` field (the latter `#[serde(default)]` for back-compat). `textrank_top_k` refactored to `textrank_pick` returning **both** top-k-in-doc-order *and* top-1-by-rank — both from one score pass (free). New `first_sentence()` helper (UAX#29-based) used by frontmatter + callout branches so the **author's first sentence wins** as the headline (author authority extended). `note_summaries` gets a nullable `headline TEXT` column via idempotent `ALTER TABLE … ADD COLUMN` for existing DBs; `get_or_compute_cached` treats fresh-content_hash-but-NULL-headline as a cache miss → recomputes both together (lazy fill, no algo-version bump, no rebuild walk).
> - **Shared frontend summary store** (`src/lib/nsc/summaryStore.ts`, NEW). `getSummariesFor(paths)` is the app-wide cache-first + batched + coalesced summary provider. Concurrent callers for the same path share **one** in-flight IPC. A lazy-init `library-changed` watcher invalidates cached entries when their file changes on disk (payload shape verified against `watcher.rs:82-88`).
> - **SourceReviewPanel migrated** to the shared store as a **no-behavior-change refactor**. Identical render / fetch shape / gentle-fill — only the IPC mechanism changed. The Cataloger looks identical to before (Stage-1 Boss validation = pass).
> - **Two first surfaces wired:** (1) **Search results** (`SearchHub.svelte`) — a faint italic `.sh-item-headline` line under each hit in all three result loops (grouped-advanced, flat-advanced, basic-categorized). The existing snippet (why it matched) stays; the new headline (what the note IS) joins it. (2) **Editor header** (`NoteEditor.svelte`) — a thin muted band above `<NotePane>` showing the active note's headline; stale-promise-guarded so tab-switch mid-fetch doesn't write the old note's headline.
>
> ### Audit + tests
> - 3-agent audit (invariants / drift / migration-path) all clean. The 8 architectural invariants of MIG-043-ARCHITECT §3 hold; one HIGH-severity-but-latent finding (SRP's stale local `NoteSummaryEntry` type missing `headline`) was cleaned up inline before commit. Rollback to MIG-040 verified safe both directions (additive schema + serde-default field + `?? ''`-guarded frontend consumers + verified against the `780713b6` INSERT/SELECT shapes).
> - **19/19 NSC unit tests pass** (incl. 3 new `first_sentence` tests + the updated `textrank_pick` tests). svelte-check: 3 pre-existing errors, **0 new**.
>
> ### Help docs in 15 locales
> - English **Note Summaries** help expanded — intro now lists all 3 surfaces (Cataloger / Source Review queue, Search results, Editor); the `## Where summaries appear, and how they fill in` section (renamed from `## When …`) now bullets the surfaces + rewrites the lazy-fill paragraph to mention all 3. 14 other-locale Note Summaries files translated to match, using each locale's established native term (المُصنِّف / 分类器 / Klassifikator / etc.). User Manual already defers to the help topic — no UM change needed.
>
> ### Roadmap after Phase 1
> - **MIG-044 (Phase 2)** — wire the remaining enabled surfaces: Sky View bubbles, backlinks/outgoing panels, the Index panel, hover previews where they exist. Same shared store, same gated/batched discipline. Each surface = a small Architect doc on its own UX placement.
> - **MIG-045 (Phase 3)** — the **Universe Digest** left-dock view itself: tiered **Library → Folder → 1-line headline**, expandable to the full summary, recency-sorted, searchable, virtualized, with cUniverse-children federated in. The biggest Phase by far; will go through full `/migration` discipline.
>
> ### Body-update debt
> The §4 subsystem map + §8 Migrations table still describe pre-MIG-043 state; this preamble is the trustworthy current state (SO #6). Session log: `lab/reports/SESSION-LOG-2026-05-23.md` (today's ship work). Prior day: `SESSION-LOG-2026-05-22.md` (design + design-PCS).
>
> ---
>
> *Below: the v2.26 preamble, retained for diff visibility.*

**Version 2.26 | 2026-05-22**

> **What changed in v2.26** (NSC Core Plug-in workstream **chartered** — Eisa's stated direction "grow the NSC into a standalone Core Plug-in serving every Constellation function" is now design-captured end-to-end, with Phase-1 Build about to cascade):
>
> ### NSC Core Plug-in Concept Paper v2.0 written
> - `docs/Constellation-NSC-Concept-Paper-v2.0.md` — elevates NSC from the single-surface subsystem of v1.0 (MIG-040: engine + Cataloger/SRP card only) into a **Core Plug-in** with two pillars: (1) a shared summary **service** feeding every enabled surface (search results, Sky View bubbles, backlinks/outgoing, Index, editor header, hovers-if-present; Map is currently disabled and out of scope), and (2) a left-dock **Universe Digest** view to skim the whole knowledge base at summary level without opening notes.
> - **5 design decisions locked by Eisa same-session** (Concept Paper §9): dock-view name = **"Digest"**; headline = **stored** as a `headline TEXT` column on `note_summaries`; **cUniverse federation in scope from v1** (the Digest spans linked child universes); **extractive only** for v2.0 (abstractive remains the future LLM-rewrite upgrade); **default sort = recency** within Library → Folder tiering.
> - **Reuse, not rebuild:** the v1.0 engine (TextRank + author-precedence), cache, content-hash invalidation, deferred backfill, `NSC_ALGO_VERSION` self-heal, and the existing batched cache-first delivery (`nsc_get_summaries_for_notes`) are unchanged. The new work is (a) a `headline` (1-line, top-1 TextRank sentence) variant, (b) a shared frontend summary store generalizing what `SourceReviewPanel.svelte:221` already does, and (c) the surfaces + Digest view.
>
> ### MIG-043 Phase 1 Architect+Plan filed
> - `docs/MIG-043-nsc-coreplugin-phase1-ARCHITECT.md` — Phase 1 of the 3-MIG NSC Core Plug-in roadmap (Phase 2 = full service reach across remaining surfaces; Phase 3 = the Digest view itself). Phase 1's scope: (1) engine `headline` variant + additive nullable `headline TEXT` column on `note_summaries` (idempotent `ALTER`, lazy-fill on existing DBs), (2) shared frontend summary store (`src/lib/nsc/summaryStore.ts`) with cache-first + batched + file-watcher invalidation, (3) migrate `SourceReviewPanel` to the store as a no-behavior-change refactor, (4) wire summaries into **search results** + **editor header** (two distinct shapes — list + single-note — that prove the pattern). Six steps A–F; rollback safe both ways (additive schema + serde-default field + reversible refactor).
>
> ### What this commit IS / IS NOT
> - **IS:** the design artifacts only (Concept Paper v2.0 + Architect/Plan + orientation v2.26 + this session log addendum + MoCh addendum). Captures the NSC Core Plug-in direction durably so a fresh session reads it immediately.
> - **IS NOT:** any code change. Phase-1 Build cascade fires next (Eisa direction: "PCS + Orientation > And cascade the Build (Steps A–F)") and lands as its own commit(s) with **orientation v2.27** + help-file additions in 15 locales (search-result summary + editor-header summary are user-visible).
>
> ### Roadmap after Phase 1 ships
> - **MIG-044 (Phase 2)** — wire the remaining enabled surfaces: Sky View bubbles, backlinks/outgoing, Index, hovers-where-present.
> - **MIG-045 (Phase 3)** — the **Universe Digest** left-dock view: tiered Library → Folder → 1-line headline, expandable to full summary, recency-sorted, searchable, virtualized, with cUniverse-children federated in.
>
> Each Phase is cross-subsystem (Rust ↔ Svelte) → full four-phase `/migration` (Architect → Plan → Build → Audit). Hard constraint: no Phase regresses boot, typing latency, or IPC responsiveness (measured before/after on the 7,600+-note universe).
>
> ---
>
> *Below: the v2.25 preamble, retained for diff visibility.*

**Version 2.25 | 2026-05-22**

> **What changed in v2.25** (MIG-042 ships — drop the dead `term_vocab.bridge_concept_id` column — and **three further bugs found & fixed while testing it**, all Boss-validated live on Eisa's real universes):
>
> ### MIG-042 — drop the dead `term_vocab.bridge_concept_id` column
> - The deferred "optional cleanup" from MIG-041 §2.3 / Phase D. The column was dead schema from the abandoned §1C eager-tagging design — no reader anywhere (frontend grep-confirmed); the base `term_vocab` CREATE never defined it (only the per-boot `ensure_term_vocab_bridge_column` add-path did). **Correction to an earlier claim:** it was **not** 100% NULL — ~24,827 / 538,648 rows (~4.6%) carried a stale value. Still dead, still safe to drop.
> - **Design:** drop the index then the column as a one-time **Part 3** of the existing MIG-041 background worker (`run_bigram_purge`), reusing its `MIGRATION_ACTIVE` WAL-daemon-pause + retry-on-busy + self-checkpoint. New `schema_versions` gate `term_vocab_dropcol`=1; `init_db` pre-stamps it when the column is already absent so clean DBs never wake the worker. Deleted `ensure_term_vocab_bridge_column`; `ctse::hooks::apply_delta` INSERT no longer names the column.
> - **Validated:** copy-test on a copy of the real 1.63 GB DB → drop in **0.37 s, 538,648 rows preserved (zero loss), integrity ok**; 3-agent audit clean (incl. proven-safe rollback); live search + note-deletion confirmed. Architect doc: `docs/MIG-042-drop-bridge-concept-id-column-ARCHITECT.md`.
>
> ### BUG-020 — orphaned Sight-v5 trigger silently broke note deletion (folded into MIG-042)
> - The MIG-042 copy-test failed at `DROP COLUMN` (it re-validates the whole schema) on `error in trigger sight_v5_layout_invalidate_ad: no such table: sight_v5_layout`. **MIG-028** (Sight v5 retirement) dropped the table + the `_au` trigger but **missed the `_ad` (after-delete) trigger** on `note_meta` — so **every `DELETE FROM note_meta` failed**, swallowed by `reindex_delete_note`'s `let _ =`, **ghosting deleted notes in the index** since ~2026-05-18. Fix: added `DROP TRIGGER IF EXISTS sight_v5_layout_invalidate_ad;` to the MIG-028 cleanup batch in `init_db` — heals note deletion on boot AND unblocks the column drop.
>
> ### BUG-021 — `CREATE INDEX` before `CREATE TABLE` crashed fresh-DB init
> - Investigating 5 long-failing `tests_m8c`/`m12` unit tests surfaced a real latent bug: `init_db` created `idx_link_target_path ON note_links` (~L1832, added in MIG-025) **before** `CREATE TABLE note_links` (~L2054). On a **fresh** DB this aborts init ("no such table: note_links"); existing DBs mask it (`CREATE INDEX IF NOT EXISTS` no-ops). Because `ensure_search_db_ready` runs `init_db` on a fresh path for any new/rebuilt universe, **new-universe / rebuilt-universe init crashed** — it left "Eisa Universe" stuck at "0 notes." Fix: moved the index into the `note_links` CREATE batch (after the table). (The 5th failure, `m12`, was stale test data — "quasar" is now in the lexicon; retargeted to a nonsense token.)
>
> ### BUG-022 — an empty index had no recovery path → auto-rebuild on open
> - Even with BUG-021 fixed, "Eisa Universe" stayed at "0 notes": schema healed but `note_meta` empty, and **nothing repopulated it.** The warm-boot **"ZERO BOOT-TIME WALKS"** optimization removed the boot index walk; its replacements (a "Settings → Rebuild Index" button + a per-universe empty-cache prompt) were **never built**. Fix (`+layout.svelte`, `initializeApp` post-stats fan-out): if the active universe's indexed note-count is 0 but it has libraries, kick off `initSearchIndex()` (the builder `add_library` uses) in the background. **Gated on empty** → already-indexed universes never walk (ZERO-BOOT-WALKS preserved). Runs on boot AND switch (both go through `initializeApp`). Boss-validated: "Eisa Universe" repopulated on open, and switching to another universe rebuilt it too.
>
> ### Lessons, commits, docs
> - **LESSONS-LEARNED LL-025/026/027** added: (025) test DB migrations under live concurrency on a copy of the *real* DB, not an isolated synthetic one; (026) a `CREATE INDEX` before its `CREATE TABLE` crashes fresh-DB init and `IF NOT EXISTS` masks it on populated DBs; (027) removing an automatic maintenance pass for perf requires a *verified* recovery path (don't trust a "now handled by X" comment without confirming X exists).
> - **Committed as 2 commits** (3 of 4 fixes share `search.rs`; strict per-concern split needs disallowed interactive staging): commit 1 = backend (`search.rs` + `ctse/*` — MIG-042 + BUG-020 + BUG-021 + m12 test fix); commit 2 = frontend (`+layout.svelte` — BUG-022) + docs.
> - **Help files / User Manual: no change** — all four are invisible bug fixes / cleanup (the auto-recover is automatic). Docs-sync satisfied by exception.
> - **Body-update debt:** §8 Migrations table (add MIG-042) + the boot/index sections (note the BUG-022 auto-recover) describe the pre-v2.25 state; this preamble is the trustworthy current state (SO #6). Session log: `lab/reports/SESSION-LOG-2026-05-22.md`.
>
> ---
>
> *Below: the v2.24 preamble, retained for diff visibility.*

**Version 2.24 | 2026-05-22**

> **What changed in v2.24** (MIG-041 — `term_vocab` bigram shrink ships + Boss-validated; a concurrency bug found & fixed in live testing):
>
> ### MIG-041 — remove the dead `term_vocab` bigrams (DB 2.35 → 1.75 GB)
> - The `term_vocab` CTSE shadow table held ~5.19M **bigram rows (90.6%)** that nothing reads — `ctse/search.rs` skips them, and the Index panel / phrase / Arabic matching read the *separate* `notes_fts`/`notes_vocab` store (the key distinction; a stale comment claimed otherwise). MIG-041 stops writing them (`ctse::hooks::token_counts` filter, Phase A) and one-time-purges the existing ones (chunked background DELETE, Phase B), then VACUUMs to reclaim the disk (Phase C). Supersedes + retires the MIG-015 v2-sentinel boot-blocker (`project_mig013`).
> - **Honest payoff: ~0.6 GB / 26%** (the earlier "1.7 GB / 70%" was an overestimate, corrected). Cold-boot disk win + cheaper saves + retired complexity. Schema counter `term_vocab_bridge` 2→3; new `term_vocab_vacuum`=1 gate.
> - Commits: `83a8453c` (A) · `0d8c0cd9` (B) · `e1daf018` (C) · `be24388a` (D, doc cleanup) · concurrency fix (this commit).
>
> ### The concurrency bug (found in live testing — the lesson)
> - First live run **stalled at exactly 600k rows for ~7 hours.** Root cause: the chunked purge worker was **fatal on `SQLITE_BUSY`**, and the **WAL checkpoint daemon's** periodic `TRUNCATE` collided with it (the daemon's checkpoint grew slow as the purge filled the WAL). The **isolated copy-test couldn't reveal it** — no app, no daemon, no concurrency. The Phase-E audit checked daemon-vs-VACUUM but not daemon-vs-purge-worker.
> - **Fix:** pause the WAL daemon during the migration (`MIGRATION_ACTIVE` flag) + retry transient locks instead of dying + worker self-checkpoints (bounds the WAL while the daemon is paused) + full `diagnostics.log` trail. **Re-validated end-to-end on the live 2.35 GB DB** (resumed from 600k → completed → VACUUM → 1.75 GB, integrity ok, Boss-confirmed).
> - **New top lesson (→ migration checklist): test DB migrations under live app concurrency, not just on an isolated copy.** A one-time chunked DB worker MUST be `SQLITE_BUSY`-resilient AND coordinated with any other background DB user.
>
> **What changed in v2.23** (overdue help docs — The Cataloger + Note Summaries ship in 15 languages):
>
> ### Two new help topics
> - **The Cataloger** (`docs/help.uConstellation.World/The Cataloger/The Cataloger.md`) — the MIG-039 left-dock full-page home: the three header buttons (*Classify a note…* note-picker, *Build all summaries*, *Start scan*), the embedded review queue, and the naming trap ("The Cataloger" the room vs. "the catalogers" the six lenses, 5 active). Defers card mechanics to Source Review; states what it does NOT do (no auto-classify default, no cloud, no prose edits).
> - **Note Summaries** (`.../Note Summaries/Note Summaries.md`) — the MIG-040 NSC: author-first precedence (frontmatter `summary`/`description`/`abstract`/`excerpt` → callout `[!summary]`/`[!abstract]`/`[!tldr]` → generated extractive TextRank → opening fallback); read-only / File-Over-App; on-device; lazy fill vs. the *Build all summaries* backfill. **Written to stand alone** — Eisa plans to grow the NSC into a standalone Core Plug-in serving every Constellation function, so it is a sibling topic, not a Cataloger subsection.
> - **Source Review** + **User Manual** §10b updated to cross-reference both (the "two places, one panel" callout + the note-summary section).
>
> ### Translation cascade (all 15 languages) + fidelity
> - 14 parallel agents translated both new topics + patched each language's Source Review + User Manual. Convention matches the newer CNS/Sight files: English folder+filename, translated content, `translation_status`/`language`/`source` frontmatter, English aliases kept + translated aliases added.
> - **Fidelity rule**: feature/button names were pulled from each language's *actual shipped i18n strings* (The Cataloger = ar المُصنِّف · de Klassifikator · zh 分类器 · …) so the help matches the on-screen UI — full-localization rule (everything adapts; native equivalents, مصادر not transliteration). Verified: 28 new files full-length, frontmatter correct, code tokens preserved, zero leftover English headings.
>
> ### Fabrication caught before fan-out (BASIC RULE)
> - The first English draft of Note Summaries claimed a visible per-card label badging each summary's *origin* (frontmatter/callout/extractive/opening) + a "Reading the summary label" table. Checked against `SourceReviewPanel.svelte` (~L1265): the card renders ONLY the `nsc.summary` ("Summary") label + text — the `source` token is fetched but never shown. Removed the false claim + rewrote the section honestly **before** translating, so it was not multiplied ×14.
>
> **What changed in v2.22** (warm-boot cracked — universe + counts now appear at ~0.4 s):
>
> ### Warm boot: sidebar renders instantly
> - **Symptom**: warm boot felt ~3 s — "the universe stays blank for ~3 s, then comes to life all at once." Definitively diagnosed with a `MutationObserver` on `.sidebar-content`: the sidebar DOM was empty (0 nodes) until ~2.45 s with **no main-thread blocking** — Svelte was rendering late because its data arrived late.
> - **Root cause**: the sidebar's library sections derive from `$libraryStats` (`ownLibraries`/`universeNotesStats`), populated only when `loadAllStats()` → `get_all_library_stats` finished — a per-library NOTE-COUNT operation. So the whole universe waited on counts it doesn't need to draw.
> - **Fix** (`+layout.svelte`, after `libraries.set(bundle.libraries)`): seed `libraryStats` from the library list immediately (placeholder counts; the badge hides when 0). `sidebar_populated_ms` **2452 → 423 ms**. Commit `f1ddfa9e`.
>
> ### Warm boot: counts from the index, not a filesystem walk
> - `get_all_library_stats` stat-walked every library tree (~7,600 stat calls cold) + read preview files — the ~1.5–3.5 s "note-counts trail in" cost after the structure painted. Rewrote it to read counts from `note_meta` (the index) via `aggregate_library_counts`: `star_count` exact; `folder_count` = distinct ancestor dirs of notes under the library root (folders containing notes); `recent_stars` dropped (verified unused). Removed 4 dead FS-walk helpers. **Same lesson as LL-024.** Commit `f616ce51`.
> - **Result**: universe structure AND note-counts both appear at **~0.4 s** (Boss-confirmed). A fresh note-write briefly disturbs the OS cache/WAL → the *next* boot is slower, self-healing in 1–2 boots (left as-is by Boss decision — inherent to a multi-GB index).
>
> ### Boot-perf instrumentation (diagnostic, reverted)
> - The `MutationObserver` + longtask recorder + `sidebar_populated_ms`/`sidebar_node_timeline`/`boot_long_tasks` fields in `boot-perf.latest.json` were diagnostic-only and reverted after the fix landed. Re-add if a future boot regression needs pinpointing.
>
> **What changed in v2.21** (boot + write performance; note-open lag fixed; NSC backfill made manual):
>
> ### Note-open lag fixed (~5 s stutter → instant)
> - `scan_unlinked_mentions` (`libraries.rs`) walked the whole library tree and read EVERY `.md` file (all 7,646) on EVERY note open, uncached — ~5 s of scroll stutter, independent of the opened note's size/media (the cost is scanning the OTHER notes). Rewrote to an FTS candidate lookup (`notes_fts` phrase MATCH on the Arabic-normalized title → JOIN note_meta → ≤300 candidates) + the exact original raw-file verify on only those candidates. Identical results, ~50× fewer reads → sub-100 ms. Commit `b7e17603`. See **LL-024**.
>
> ### WAL hygiene — faster boot + instant writes
> - `search.db-wal` had bloated to **372 MB** (passive auto-checkpoints reset the WAL's reuse position but never shrink the FILE), adding ~1.1 s to every boot. `init_db` now sets `synchronous=NORMAL` (safe — the search index is ephemeral, rebuilt from the `.md` files) + `busy_timeout=5000` + `mmap_size=256 MB`; `spawn_wal_checkpoint_daemon` (own connection, 20 s post-boot then every 5 min) runs `PRAGMA wal_checkpoint(TRUNCATE)`. **Boss-measured: WAL 372 MB → 0 MB; boot ~4 s → ~3 s; new-note typing instant for the first time ever.** Commit `a532eaeb`.
>
> ### NSC summary backfill → MANUAL (protects instant boot)
> - The auto-after-paint backfill trigger regressed perceived boot (forced the embedding-model load + a full-Universe embed pass ~8 s in → ~28 s of "still booting"). Removed it. The backfill is now a manual **"Build all summaries"** button in the Cataloger header (same background worker — resumable, gentle, cancellable, progress strip). The Settings toggle + `appSettings.nsc.backfillEnabled` were removed. Summaries still fill lazily on scroll. Commit `a338d9e2`.
> - **Boot goal**: Eisa wants **< 2 s**. The WAL fix got ~4 s → ~3 s; the next lever is fully backgrounding the relationship-graph load so it never gates "ready".
>
> **What changed in v2.20** (MIG-040 — NSC ships; both-axes disambiguation fixed; Cataloger cross-instance sync; "Classify a note…" note-picker):
>
> ### MIG-040 — Note Summary Creator (NSC) ships
> - Each Cataloger card and right-sidebar Source Review card now shows a **summary of the note** under the title (above the reasoning trail). NSC **only generates when the author hasn't written a summary** — precedence: (1) YAML frontmatter `summary:`/`description:`/`abstract:`/`excerpt:`; (2) a body `> [!summary]`/`[!abstract]`/`[!tldr]` callout (read **verbatim from the raw file** — `body_text` is markdown-stripped + Arabic-normalized, which would corrupt the author's wording); (3) else compute **extractive TextRank** over the body (embedding-similarity sentence graph, weighted PageRank d=0.85). 100% offline, all 100 languages via the existing e5-small ONNX model.
> - **Callout precedence (fix, 2026-05-20)**: the first cut checked only frontmatter and generated a TextRank summary that **overrode** author summary callouts — the bug Eisa caught on `الهرم الأكبر` (a `> [!abstract] ملخّص` callout). Now `body_callout_summary()` (mirrors `calloutPlugin.ts`) catches the 📋-family callouts. New source value `callout`.
> - **Sentence segmentation**: Unicode UAX#29 (`unicode-segmentation` crate) + paragraph/opening-sentence fallback for punctuation-light scripts (Thai, Lao, etc.).
> - **Cache**: `note_summaries` table in the search DB (path PK, summary, source, content_hash, updated_at). Content-hash invalidation — stale summaries recompute; unchanged bodies are free reads (Rule 8). The hash is prefixed with `NSC_ALGO_VERSION` (`v2`) so an algorithm change invalidates the whole cache (self-healing, no wipe).
> - **Crash guards** (`nsc/mod.rs`): `MAX_BODY_CHARS = 50_000` body truncation + `MAX_RANK_SENTENCES = 40` downsampling — prevents large notes from aborting the ONNX runtime.
> - **Delivery**: batched IPC `nsc_get_summaries_for_notes` (cache-first, zero per-card IPC); gentle chunked fill (6 notes/batch, 500 ms debounced, paused while scanner runs); NSC never competes with the classifier scan. `nsc_get_summary` for single-note get-or-compute.
> - **UI**: `<div class="srp-summary" dir="auto">` under the card title; accent-color left border; shows in BOTH surfaces (Cataloger + right-sidebar). i18n key `nsc.summary` across 15 locales.
> - **Tests** (10 unit tests pass): UAX#29 en/ar/zh/hi/th split; TextRank cluster-vs-outlier; frontmatter precedence; opening-text fallback; downsample bounds.
> - **Concept paper**: `docs/Constellation-NSC-Concept-Paper-v1.0.md`.
>
> ### MIG-040 — Both-axes disambiguation bug fixed
> - **Bug**: when a card had Split on both axes and the user picked the Source chip, the card vanished before the user could pick Content type.
> - **Fix** (`classifier/mod.rs`): `cece_resolve_disambiguation` returns `Option<SuggestionRecord>`. When the other axis is still Split: re-insert the suggestion with the resolved axis settled and return the updated record. Frontend keeps the card (refreshed); removes it only when both axes are decided (null return). New helpers `other_axis_needs_disambiguation` + `mark_axis_resolved` — 2 unit tests; 7 r7_tests all pass.
>
> ### MIG-039 follow-ups — Cataloger cross-instance sync + note-picker
> - **Cross-instance sync** (`SourceReviewPanel`): `classifyActiveNote()` now dispatches `constellation:classify-and-show` so the Cataloger SRP picks up the new suggestion immediately (was local-only before). Local instance self-guards via `classifying = true`.
> - **Queue reload on Cataloger reopen** (`SourceReviewPanel`): new `visible` prop; `_srp_was_closed` guard reloads queue the first time `visible` flips false→true after mount. Right-sidebar unaffected.
> - **Newest-first queue order (fix, 2026-05-20)**: `sources_list_pending_suggestions` now orders `created_at DESC, note_path ASC` (was `ASC`). The panel renders only the first `RENDER_BATCH = 80` cards; with thousands of pending suggestions (Eisa's Universe: 7203), oldest-first buried a just-classified note at position ~7200, below the cap — the "scan didn't update the list" report. Newest-first puts fresh classifications at the top, matching the live-classify prepend.
> - **"Classify a note…" note-picker** (`CatalogerView`): inline search popover in the header. User searches by note name (`constellation_search`, lexical, limit 10); clicking a result classifies and syncs both SRP instances. ESC closes. i18n `cataloger.classifyNote` / `.searchNotes` / `.noNotesFound` × 15 locales.
>
> ### Body updates in v2.20
> - §3.2 lazy-mount flags: added `catalogerEverOpened`.
> - §4.5 CECE / The Cataloger: new body section (clears body-update debt from v2.19).
> - §3.4 version: corrected to `0.1.0` (body had drifted to `0.3.4`).
>
> ### Verification
> - `svelte-check`: 3 pre-existing, 0 new. `cargo check`: clean. Release build produced for Boss test.
> - Session log: `lab/reports/SESSION-LOG-2026-05-20.md`.
>
> ---
>
> *Below: the v2.19 preamble, retained for diff visibility.*

**Version 2.19 | 2026-05-20**

> **What changed in v2.19** (The Cataloger ships as the first left-dock Core Plug-in; a long-standing CECE memory leak fixed; "Hide reasoning" chevron fixed; two follow-ups opened — NSC + a disambiguation bug):
>
> ### MIG-039 — "The Cataloger" ships (CECE promoted to a left-dock Core Plug-in)
> - CECE's universe-wide home is now a **left-dock button + full-page view** (`src/lib/components/CatalogerView.svelte`), mirroring the OrgChart pattern across ~25 wiring sites in `+layout.svelte` (`showCataloger` state, lazy-mounted `.cataloger-overlay` per LL-022, `fullPageActive` / `content-hidden` terms, command-palette entry, escape branch, close-others lists). Composes the existing `ClassifierScanProgressStrip` + a self-contained **"Scan library"** button (`classifier_scan_start`) + the library-wide `SourceReviewPanel`.
> - **Name**: "The Cataloger" (en) / **المُصنِّف** (ar, *classifier* sense) / classifier-sense in all 13 other locales. Internal engine name stays **CECE**. i18n keys `ribbon.cataloger`, `commands.cataloger`, `cataloger.title`, `cataloger.tagline` across 15 locales (`scripts/add-cataloger-i18n.mjs`).
> - **Feature flag**: `enabledFeatures.cece` (store.ts), default **ON**; gates the dock button; toggleable via a new **Settings → Plug-Ins → Discovery** card (`id: 'cece'`, reuses `cataloger.*` keys). Distinct from the existing `appSettings.cece` engine-settings object.
> - **Accuracy guardrail (Concept Paper §5/§6)**: CECE is a **5-cataloger heuristic ensemble** (the local-LLM "Reasoning" cataloger is designed-but-NOT-wired) and scans are **manual-only**. No user-facing copy says "AI"/"LLM" or implies automatic background classification.
> - Right-sidebar **Source Review tab unchanged** (Concept Paper §10); the two `SourceReviewPanel` instances coexist safely.
>
> ### The CECE memory-leak fix (shared `SourceReviewPanel` — fixed BOTH surfaces)
> - **Root cause**: `SourceReviewPanel` rendered the **entire** pending queue (~4,475 rows on the trial universe) with **no virtualization / cap**, and `filterCounts` + `splitAwareSkipCount` + `filteredQueue` + the per-card render each re-`JSON.parse`d every `composite_json` blob → tens of thousands of DOM nodes + ~18k parses in one synchronous pass → froze the app on Cataloger open, progressively worse during a scan (1.5 s reloads). The CLAUDE.md **Rule 3** violation; Eisa confirmed the same leak pre-existed in the right-sidebar CECE.
> - **Fix**: (a) **memoize** `parseComposite` (component-local `Map` keyed by the immutable `composite_json` string); (b) **render cap** — `RENDER_BATCH = 80`, `visibleQueue = filteredQueue.slice(0, renderCap)`, **"Show more"** footer (+80) with a localized "Showing N of M" note (`cece.queueShowMore` / `cece.queueShowingCount`, 15 locales). Counts + Approve All still operate on the full queue — only the DOM is bounded. Boss-validated: opening on the full universe is instant; scans stay responsive.
>
> ### Two more CECE fixes/finds
> - **"Hide reasoning" chevron fixed** (`SourceReviewPanel`): trails open-by-default (the `reasoningTrailVisibility: 'always'` setting, or trust-cal <50 reviews) couldn't be collapsed — the toggle only had a force-OPEN set (`expandedTrails`). Added a `collapsedTrails` force-CLOSE set; `isTrailOpen` checks it first, so an explicit per-card click overrides any global default. Boss-validated.
> - **Disambiguation bug FOUND (deferred)**: when a card is Split on **both** axes, picking the **Source** chip calls `cece_resolve_disambiguation`, which clears the whole suggestion and only co-writes the other axis if it was *settled* (`extract_other_axis_settled` returns None for a split axis) — so the **Content type is left unclassified and the card vanishes** before the user can pick it. Proper fix is cross-subsystem (backend: keep the suggestion alive until both split axes resolve; frontend: keep the card). **Open** — fix in the NSC increment.
>
> ### Layout
> - `CatalogerView` is **full-width** (removed the 880 px centered column that read as "a window within a window").
>
> ### NSC chartered (Eisa, 2026-05-20) — next subsystem
> - **Note Summary Creator (NSC)**: a new subsystem to show, in each Cataloger card (under the title, above the reasoning), a summary of the note — use the note's own summary if present, else **summarize the whole note**, **language-agnostic**. Eisa frames it as a Constellation differentiator. **To be designed next** (local-first; the existing e5-small ONNX embeddings enable extractive summarization without the unwired LLM — an Architect-pass decision). Build order after this commit: NSC + the disambiguation fix.
>
> ### Verification + ship state
> - `svelte-check`: 3 pre-existing errors (store.ts `fresh` + 2× PropertyEditor node-type), **0 new**. Release build clean (NSIS `.exe` copied as `Constellation_0.1.0_x64-setup.MIG039-cataloger.exe`). All Boss-test steps PASS across two rounds. Version stays **0.1.0**.
> - **Body-update debt**: the §4.x CECE / Source Review subsystem section + the left-dock feature list further below still describe the pre-Cataloger state — update them when the NSC increment lands. This preamble is the trustworthy current state (SO #6).
> - Session log: `lab/reports/SESSION-LOG-2026-05-20.md`.
>
> ---
>
> *Below: the v2.18 preamble, retained for diff visibility.*

**Version 2.18 | 2026-05-19 (evening)**

> **What changed in v2.18** (the Sight strategic pivot + CECE becomes the first Core Plug-in + Constellation is v0.1):
>
> A long evening session that began mid-Sight-v6.3 and ended with a fundamental strategic reframe. Triggered by Eisa's question — *"Why have I created Constellation? … I could easily be satisfied with what Obsidian is offering."* — which led back to the founding mission (`docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`: cultivate **wisdom** through the **living link system**; visualizations are downstream readers, not the core mission) and a decision to externalize the heavy visualizations.
>
> ### The strategic pivot (headline)
>
> - **Sight + Map are DISABLED in core** (MIG-038, `57cd7638`). Sight via `SIGHT_V6_ENABLED=false` in `src/lib/sight/engine.ts`; Map via a `loadSettings` force-off of `enabledFeatures.constellationMap` in `store.ts`. All code intact for later detachment. **⚠️ The Sight + Map descriptions scattered through the body below (subsystem map, dependency tables, the Sight v6.3 delivery sections) describe their shipped state — accurate as implementation records, but neither feature is reachable in the running app as of v2.18.**
> - **"Constellation Wings"** — a NEW sub-project chartered (`docs/Constellation Wings/Charter v0.1.md`) to design the **External Plug-in subsystem**. DEFERRED until Eisa schedules. Captures the Tauri single-binary-Rust constraint, the two-layer (isolate-now / load-later) model, and the hybrid-API decision.
> - **Plugin taxonomy** (Eisa): a **"Core Plug-in"** = a main-LEFT-DOCK feature that stays in the app (Sky View, CNS, Index, and now CECE). An **"External Plugin"** = detached from the app (Sight, Map → Wings).
>
> ### CECE becomes the first Core Plug-in ("The Cataloger")
>
> - **CECE Concept Paper v1.0** written (`docs/Constellation-CECE-Concept-Paper-v1.0.md`, `9ab0d193`) — core concept: classify each note on two axes (content-type × source) to reveal the **epistemic texture** of the universe. **Honest accuracy finding**: the local-LLM "Reasoning" cataloger is **DESIGNED BUT NOT WIRED** — CECE ships as a **5-cataloger heuristic ensemble** (User-Authority / Structural / Linguistic / Graph / Semantic); scans are **manual-only**. Do NOT market CECE as "AI classification."
> - **Named "The Cataloger"** (user-facing; internal engine stays **CECE**), `feec7b12`. **Arabic = المُصنِّف** (classifier sense).
> - **Left-dock build handed off** as **MIG-039** to a fresh session (`lab/reports/MIG-039-CATALOGER-HANDOVER.md`, `96755990`). Right-sidebar Source Review tab stays until the dock view is built.
>
> ### Sight v7 + v6.3 (built, then frozen — both now moot since Sight disabled)
>
> - **MIG-036 (Sight v7)** — a Form-Aligns-To-Purpose ground-up redesign built through P1–P3 (Architect + `density.ts`/`stack.ts` pure primitives + `anchor-v7.ts` universe-view dispatcher + `masadir-v7`), then judged over-engineered by Eisa ("Why reinvent Sight?"). **DORMANT** under `SIGHT_V7_ENABLED=false`; kept on disk as a fallback.
> - **MIG-037 (Sight v6.3)** — the surgical-edits alternative. **P1 shipped** (`bb221fe4`): Time Dome added as a 25th tradition + new "time" family + 15-locale i18n. Phases 2–3 (calendar-rim opt-in, Aristotelian pure-radial reframe, density blobs) **FROZEN** by the pivot.
> - **MIG-029 (per-note frontmatter wiring)** — attempted (6 commits §ν.1–§ν.6 + 3 fixes), failed Boss test, root-caused to the `index_note` cache-hit short-circuit at `search.rs:3004` → carried as **PJ-060** (P1, the single highest-leverage open fix).
>
> ### CLAUDE.md — Form-Aligns-To-Purpose rule (NEW top-principal, `30f1d6a7`)
>
> Added between "Constraint as Design" and "Language-First by Design": every visual element, interaction, or computational layer must serve its core cognitive purpose; do NOT add filler to occupy degrees of freedom the chosen primitive affords but the answer doesn't require. Born from Eisa's rejection of a hash-jitter proposal. Saved to persistent memory.
>
> ### Version → 0.1.0 (`26fe4f43`)
>
> The JS configs (`package.json` + `tauri.conf.json`) had drifted to 0.3.4 while `Cargo.toml` stayed 0.1.0. Aligned to 0.1.0 per Eisa: *"Constellation will be v.0.1."* Installers now stamp `Constellation_0.1.0`.
>
> ### State-of-application audit (SO #5)
>
> Full snapshot run (durable record in `lab/reports/SESSION-LOG-2026-05-19.md`): 18 subsystems verified-shipped + Boss-validated; ~48 open PJs (overwhelmingly P2/P3 polish, no production blockers); PJ-060 surfaced as the P1 leverage fix. Constellation is mature but explicitly **v0.1**.
>
> ### Open PJ tally after v2.18
>
> +PJ-059 (Sight per-note search/finder), +PJ-060 (`index_note` cache short-circuit). Sight-related PJs (PJ-011 Map backlog, PJ-059) effectively dormant while Sight + Map are disabled. CECE-related polish (PJ-041/042/043) remains relevant (CECE is becoming The Cataloger).
>
> ---
>
> *Below: the v2.17 preamble, retained for diff visibility.*

**Version 2.17 | 2026-05-19**

> **What changed in v2.17** (Sight delivery cascade — Tiers 1-3 shipped; Tier 4 + all Sight v5 retired per Eisa decision):
>
> Triggered by Eisa direction: "We will proceed with Tiers 1-3 and drop Tier 4. Whatever is related to Sight v5 shall be abandoned." Single multi-MIG cascade in one session closes 4 of the 8 outstanding Sight items + abandons all Sight v5 footprint.
>
> ### MIGs shipped this turn
>
> - **MIG-028 — Sight v5 retirement.** Deleted `src/lib/sight/v5/` (7 TS/Svelte files) + `src-tauri/src/sight_v5.rs` (36818 bytes); removed `pub mod sight_v5;` + 4 `sight_v5_*` IPC registrations from `lib.rs`; removed `SIGHT_V5_ENABLED` flag from `engine.ts`; removed v5 dock button + v5 modal mount + SightV5 imports from `+layout.svelte` (dead `sightV5Active` state variable retained as harmless no-op — cleanup of remaining mutex-clear references is future polish); replaced 2 v5 init_db calls with idempotent `DROP TRIGGER/TABLE IF EXISTS` cleanup migration in `search.rs`; updated `sight_v6.rs` doc comments + deleted the `dual_mount_v5_and_v6_caches_coexist` test (B2 invariant no longer testable). MIG-024 Plan/Architect docs at `lab/reports/MIG-024-SIGHT-V5-*.md` preserved on disk as historical record.
>
> - **MIG-030 — Sight v6 vitest runner + 2 tests (closes PJ-054).** Installed `vitest@4.1.6` as devDependency; wired `npm run test:sight-v6` / `test:sight-v6:perf` scripts; wrote `vitest.config.ts` with explicit include/exclude scope (skips worktree duplicates + the still-deferred `layout-fidelity.test.ts` which needs playwright). Wrote `tests/sight-v6/tradition-isolation.test.ts` (Plan §14.1 — 28 tests asserting all 24 traditions × channel-isolation invariant) + `tests/sight-v6/tradition-perf.test.ts` (Plan §14.2 — 26 tests asserting per-tradition switch ≤16ms on 7,636-note universe). Combined with the pre-existing `perf.test.ts`: **58 tests across 3 files, all PASS.**
>
> - **MIG-031 — λ-fix-6.b (fa/he/ja/tr canvas deep audit).** 4 parallel polish agents (one per locale). fa.json needed 0 edits (already correctly polished at §λ-fix-3 insertion time — Persian uses Arabic script directly for Sunni Islamic terms). he.json got 17 edits (Hebrew glosses added to bare Arabic-transliteration terms — masadir.* + ibn-rushd-burhan.* + shatibi-maqasid.*). ja.json got 12 edits (Japanese semantic glosses + 5 redundant-duplicate bug fixes in mencian-sprouts/wang-yangming where the gloss matched the term). tr.json got 28 edits (Turkish glosses across pardes/pramana/shatibi/mencian-sprouts/talmudic-middot). Total: **57 keys polished across 3 locales** (fa untouched). Now every Sunni-Islamic / Sanskrit / Hebrew / Greek-Latin technical term carries a target-language gloss matching the ar/zh/ko quality bar across all 15 locales.
>
> - **MIG-032 — Tier 3 housekeeping.** PJ-057.a (Mohist citation): no-op — Concept Paper v4.1 already cites the manifest-canonical "Book IX, Fēi Mìng Shàng 非命上" (v4.0's "ch. 35" is the same content at a different level of detail; v4.0 stays untouched per versioning rule). PJ-057.c (prebuild footnote): added to Concept Paper §9.1 `_manifests.generated.ts` table row + updated §9.3 to reflect MIG-028 v5 retirement. PJ-057.b (fresh 24-tradition SVG mocks for `Sight-vNext-MockB1-Toggle.svg` + `sight-redesign-v0.2-mockE-tradition-registers.svg`) **deferred** — visual design work that overlaps with PJ-051; stays as a polish item for a focused session.
>
> ### Sight v5 final state
>
> Per Eisa "Whatever is related to Sight v5 shall be abandoned": all production code removed. `sight_v5_layout` table + invalidation trigger dropped from existing user databases on first MIG-028-build boot via idempotent `DROP IF EXISTS`. Plan/Architect docs preserved as historical record. **No code path can reach v5 anymore.**
>
> ### Tier 4 final state
>
> Per Eisa "drop Tier 4": the original Sight v5 vision's Layer 2 (diagnostic) / Layer 3 (recommendation) / Layer 4 (coaching) workstreams are formally abandoned. Sight v6's facet sidebar + tradition-aware shape renderers provide a different design direction; the diagnostic/recommendation/coaching layers will not be built under any future MIG. The reserved-but-never-used scope swaps that put MIG-025 = v6 foundation, MIG-026 = v6 traditions, MIG-027 = theme inheritance stand permanently.
>
> ### Sight delivery tally after this cascade
>
> | Tier item | Status |
> |---|---|
> | 1.1 — Per-note frontmatter wiring | **Open** — MIG-029 Architect doc deferred to next session (Rust-side extraction + 8 tradition modules + 15-locale docs is a real cross-subsystem MIG; needs proper /migration discipline) |
> | 1.2 — Sight v6 vitest test runner (PJ-054) | **DONE** in MIG-030 (58/58 tests pass) |
> | 1.3 — fa/he/ja/tr canvas deep audit | **DONE** in MIG-031 (57 keys polished across 3 locales) |
> | 2.4 — Wasm/QuickJS sandbox for TS plugin layer | **Open** — MIG-033 Architect doc deferred to next session (security uplift; large MIG) |
> | 2.5 — v4.1 per-tradition internal-structure polish | **Open** — MIG-034 deferred (aesthetic polish) |
> | 2.6 — Federation cUniverse tradition behavior | **Open** — MIG-035 deferred (design call needed) |
> | 3.7 — PJ-057 (Mohist citation + prebuild footnote) | **DONE** in MIG-032 (citation no-op + prebuild footnote landed) |
> | 3.7 — PJ-057.b SVG mocks (overlaps PJ-051) | **Open** — visual design work; defer |
> | 4 — Layer 2/3/4 (diagnostic/recommendation/coaching) | **ABANDONED** per Eisa decision |
> | v5 codebase | **RETIRED** in MIG-028 |
>
> ### §8 Migrations table additions
>
> MIG-028 row added below; MIG-029/033/034/035 reserved-but-not-started.
>
> ### Open PJ tally after v2.17
>
> Done: 13 → **14** (+1: PJ-054 closes in MIG-030). Open PJs: 48 → 47.
>
> ---
>
> *Below: the v2.16 preamble, retained for diff visibility.*

**Version 2.16 | 2026-05-18**

> **What changed in v2.16** (post-MIG-026 state-of-standing audit closes 4 abandons + 3 status corrections; MIG-022 §N formally closed; orientation §8 Migrations table fully refreshed):
>
> Eisa-requested triage of all remaining work surfaced ledger drift accumulated across 2026-04 → 2026-05 (Sight v3 → v4 → v5 → v6 successive supersessions; stale top-of-queue inheritance; PJ status drift). Single audit + cascade closed it all.
>
> ### Eisa decisions (5 NEEDS-DECISION items locked 2026-05-18)
>
> - **MIG-005** Alias-aware in-memory inbound — **ABANDONED**. Steps 1-3 stay shipped; Steps 4-8 abandoned after the fabrication-catch pause.
> - **PJ-015** 360.3D Stratification Matrix guidance doc — **ABANDONED**. Matrix-UX dependency hasn't moved; low leverage.
> - **PJ-036** Sight layer peeling — **ABANDONED**. Sight v6's facet sidebar substitutes for the v2 §2.2 mechanism.
> - **PJ-056** literal-deletion sub-question — **CLOSED as documentation**. The 24 `name:` + 10 `FAMILIES.label` literals stay as canonical EN source-of-truth + defensive renderer fallback.
> - **MIG-022 §N** — **PROCEEDED + CLOSED** this turn. The §N audit landed 2026-05-12 (`lab/reports/MIG-022-§N-{AUDIT-INVARIANTS,AUDIT-DRIFT,AUDIT-MIGRATION-PATH,FINAL-INTEGRATION-AUDIT}.md`) but Eisa never explicitly locked D-N1/D-N2; the P1 trigger-coverage fix shipped in commit `1240984d` (MIG-024 §0 UPSERT) implicitly chose option (α) + timing (a). Retroactive §8 close-out section appended to `MIG-022-§N-FINAL-INTEGRATION-AUDIT.md` recording the close; P2/P3 polish items (F2-F7) remain in cleanup backlog; F8 (i18n gap) partially resolved by MIG-026 §λ.
>
> ### Status-drift fixes (3 items, ledger now matches reality)
>
> - **PJ-035** body said "Open" → **DONE in MIG-019 §2B** (`16063735`).
> - **PJ-040** body said "Open" → **DONE in MIG-022 §D** (`c072700`).
> - **PJ-038** body said "In-Progress" → **SUPERSEDED by Sight v6**.
>
> ### §8 Migrations table fully refreshed
>
> Previously v2.13's §8 table was stamped 2026-05-07 with MIG-020 → MIG-025 missing rows + MIG-019 marked "🟢 Next-up" (stale — v3 retired). v2.16 §8 below replaces it with the current state: 18 SHIPPED MIGs (001/003/004/008/009/010/011/012/013/014/015/017/021v3/022/025/026/027 + MIG-002 partial), 3 SHIPPED-then-SUPERSEDED (018/019/024 — all Sight v3/v5 work), 2 ABANDONED (016 cancelled, 020 orphaned, +005 added this v-bump), 4 STILL-OPEN-VALID (002 §7-10, 006 §4-11, 007=PJ-005, 023 Warrant Research not started).
>
> ### Top-of-queue rotates (real next-up)
>
> 1. **PJ-005 / MIG-007** Links Settings tab — P1 user-facing; no Architect yet.
> 2. **PJ-002** cid_cn collision scrub utility — P1 mini-MIG.
> 3. **PJ-003** Rename-collision popup — P1 UX.
> 4. **PJ-008 + PJ-009** Typed-link duplication pair — P2 single-file fixes.
> 5. **PJ-016/017/018/019 bundle** — MIG-013 cleanup MIG (4 PJs → 1 MIG).
> 6. **MIG-023** — Constellation Warrant Research workstream (Concept Paper first; reserved since 2026-05-11).
>
> ### Open PJ tally
>
> Done: 7 → **12** (+5 — PJ-035 + PJ-040 status-corrections + PJ-052/053/055/056 this session). Abandoned: 1 → **4** (PJ-015 + PJ-036 + MIG-005 abandoned 2026-05-18). Superseded: implicit → **1** (PJ-038). Rejected: 1 (PJ-037). Open PJs: 56 → **48**.
>
> ### Known doc-drift carried forward
>
> 1. `store.ts:3483` — `TraditionId` literal-union duplicate; should import from `types.ts` for single-source.
> 2. PJ-057 — Concept Paper v4.1 surfaced 3 items (Mohist citation discrepancy, pre-expansion SVG mocks, `_manifests.generated.ts` prebuild footnote).
> 3. MIG-022 §N polish backlog (F2-F7 + PJ-044/046/047/048/049/050) — 12 polish items in dormant queue.
>
> ---
>
> *Below: the v2.15 preamble, retained for diff visibility.*

**Version 2.15 | 2026-05-18**

> **What changed in v2.15** (MIG-026 SHIPPED — Phase μ ship gate closed; milestone tag `milestone/sight-v6.3-traditions-ship` cut):
>
> Phase μ Migration Rule audit (3 parallel agents) returned clean: **zero blockers**. All 10 architectural invariants PASS (mini-domes stay tradition-agnostic, no `$effect` loops, no new IPCs, write-time derivation intact, fallback chain preserved, plugin label passthrough intact). Migration path: 9 of 9 scenarios PASS with 2 advisories (user-plugin label collision with dotted i18n keys → PJ-055; Concept Paper v4.0 vocabulary lag → PJ-052). Drift: 1 high-severity (PJ file v1.11 header → fixed inline by bump to v1.12), 2 low-severity (dome.ts stale comment + dead module name literals → PJ-056). λ-fix-6 translation quality audit found 1 critical + ~95 polish items (de "Geist · Geist" redundancy → fixed inline; the rest → PJ-053).
>
> **Final cascade tally for MIG-026**: 28 phases (γ → θ → ι → κ → λ → μ) over ~36 hours of focused work across 2026-05-17 + 2026-05-18, ~50 commits, 24 curated traditions + 9 shape renderers + user-definable layer (declarative JSON + TS plugin loader with Obsidian-trust consent flow + asset:// dynamic import + CSP add) + full 15-locale chrome+canvas localization + RTL-aware chevron flip + masadir manifest H1 quality fix.
>
> ### Phase μ closeout artifacts
>
> - **Milestone tag**: `milestone/sight-v6.3-traditions-ship` (pointing at `f382a97b`).
> - **ZIP backup**: `E:/Backups/Constellation/Constellation-sight-v6.3-traditions-ship-20260518.zip`.
> - **Boss-test installer (most recent)**: `Constellation_0.3.4_x64-setup.MIG026-phase-lambda-fix4b.exe` (129.6 MB).
> - **Session log**: `lab/reports/SESSION-LOG-2026-05-18.md`.
>
> ### Deferred to PJs (filed in Pending Jobs v1.12)
>
> 1. **PJ-052** — Concept Paper v4.1 (~9,500 words of new scholarly prose covering 24 traditions × 9 shape renderers; 2-day focused single-session task).
> 2. **PJ-053** — λ-fix-6 native-quality translation re-audit (~70 transliteration-without-gloss items in de/ru/fr/es/hi/pt; ~25 wrong-script Latin values in Cyrillic/Devanagari locales; 3 pt-PT vs pt-BR drift items).
> 3. **PJ-054** — Sight v6 vitest test runner (Plan §14.1 channel-isolation + §14.2 perf tests blocked on the deferred runner).
> 4. **PJ-055** — User-plugin schema warning for dotted-path label collision.
> 5. **PJ-056** — MIG-026 drift cleanup (dome.ts stale comment + 24 dead `name:` literals + 10 dead `FAMILIES[*].label` literals + 2 stale doc comments).
>
> ### Known doc-drift (carry-forward from v2.13/v2.14, unchanged by Phase μ)
>
> 1. `store.ts:3483` — `TraditionId` literal-union duplicate; should import from `types.ts` for single-source.
> 2. Concept Paper §4.1.2 (pramāṇa) — NE/SE/SW/NW → E/S/W/N after §δ.2-fix-1.
> 3. Concept Paper §4.1.3 (masādir) — same NE/SE/SW/NW → E/S/W/N after §θ-fix-1.
> 4. §8 Migrations table — MIG-020 through MIG-025 rows still missing.
> 5. (NEW per Phase μ migration-path audit) — `docs/traditions/schema/tradition.v1.schema.json` lacks a warning that literal plugin labels shaped like dotted i18n key paths could collide with the global key namespace → PJ-055.
>
> ---
>
> *Below: the v2.14 preamble, retained for diff visibility.*

**Version 2.14 | 2026-05-18**

> **What changed in v2.14** (MIG-026 Phase λ-fix-3/4/5 — Sight v6 full canvas + chrome localization across all 15 locales):
>
> Boss-test on 2026-05-18 surfaced that after Phase λ-fix-2 shipped chip-dropdown localization, every other on-canvas + chrome string in Sight v6 was still in English when the active locale was non-en. Eisa reiterated the **Full Localization Standing Order** ("When a user switches to their preferred language, the app should fully adapt to it. It means everything") as a day-one principal, then directed "Cascade through now (this session)" — ship λ-fix-3, λ-fix-4, λ-fix-5 in one session.
>
> ### What ships in this cascade
>
> 1. **λ-fix-3 — Dome canvas labels.** `renderAnchorDome` + `renderMiniDome` now accept a `labelize: (key) => string` option (defaults to identity). Module-level `_labelize` state in `anchor.ts` + `miniDome.ts` mirrors the existing `_chrome` pattern, so every `fillText` call across 9 draw helpers (`drawSectorDividers`, `drawRingBoundaries`, `drawLadderSteps`, `drawRelationalGraph`, `drawCyclicFlow`, `drawBinaryFlow*`, `drawHorizontalBands`, `drawGradientFog`) translates via `$t(key)` without changing every signature. 23 of 24 tradition modules refactored to write i18n keys in label arrays (aristotelian has no on-canvas labels). Stratum labels resolve via `STRATUM_LABEL_KEYS` at `sight.v6.stratum.<band>`. Mini-dome titles + provenance sectors now drive through `sight.v6.miniDome.{title,provenance}.<key>`. Extension chips (masadir's istiḥsān/istiṣḥāb/maṣlaḥa-mursalah/ʿurf) localize via `$t` with `dir="auto"` for natural directionality.
>
> 2. **λ-fix-4 — Facet sidebar + header chrome + RTL count-spacing.** `facets.ts` refactored so facet group labels and static category labels (Foundation, Hypothesis, Self, Established, etc.) emit i18n keys; user-domain values (folder paths, library names, custom stage names) stay literal (fallback chain returns unknown keys unchanged). `facetSidebar.svelte` chrome (FACETS title, Filters tooltip, expand/collapse aria-labels) all `$t`-wrapped. RTL count-spacing bug fixed: `padding-right: 6px` → `padding-inline-end: 6px` on `.facet-cat-label` (the "549Biology" mash the Boss screenshot showed — physical-direction padding kept the gap on the right side, so in Arabic the count flushed against the label). `SightV6.svelte` header chrome (title, subtitle, EXTENDED badge + tooltip, filter count "notes" suffix, Reset View label + tooltip) all `$t`-wrapped.
>
> 3. **λ-fix-5 — Arabic masadir manifest H1 fix.** `docs/traditions/ar/masadir.md`: `# مَسَادِر` → `# المصادر`. Per Eisa: "Arabic equivalent for 'masdir' is 'مصادر' not 'مسادر'." The diacritical-marked form was an AI-translation error in Phase ι.1; corrected to the canonical Arabic word for "sources" with definite article matching the manifest's voice.
>
> ### i18n keys added (15 locales)
>
> - 5 stratum labels (FOUNDATION → EDGE OF KNOWING)
> - 10 mini-dome strings (5 titles + 5 provenance sectors)
> - 110 per-tradition canvas labels (23 traditions × 2–15 labels each, ranging from Polanyi's binary tacit/explicit to Talmudic 13 middot)
> - 12 facet sidebar + group labels
> - 4 confidence levels + 12 stage names (Living Link 7 + Concept Paper v4.0 5)
> - 7 header chrome strings
>
> All 15 locales (en + ar curated by Claude; fa/he/ur/zh/ja/ko/es/fr/de/pt/ru/hi/tr backfilled by 4 parallel agents) now have the full `sight.v6.{stratum,miniDome,tradition.canvas,facet,facetSidebar,confidence,stage,header}` subtree. Each locale uses native-equivalent terms where they exist (e.g. zh "古兰经" for Qur'an, fa "قرآن", de "Koran"); tradition-specific transliterations (Sanskrit pramāṇa, Chinese 良知 liángzhī, Akan nokware) preserved with native gloss after `·` where appropriate.
>
> ### Wiring
>
> - `SightV6.svelte` passes `labelize: $t` + `locale: $locale ?? 'en'` to `renderAnchorDome` (was `navigator.language` — leaked the browser locale into calendar-month rendering). New `$effect(() => { void $locale; …paint() })` triggers a repaint on language switch.
> - `MiniDome.svelte` same wiring — `labelize: $t` to `renderMiniDome` + locale-change repaint effect.
> - `facetSidebar.svelte` imports `t` from `$lib/i18n`; every label rendered as `$t(label)`. Folder/library/custom-stage names that aren't i18n keys fall through the fallback chain and render literal (user data preserved).
>
> ### Files touched
>
> 32 files: anchor.ts, miniDome.ts, SightV6.svelte, MiniDome.svelte, facets.ts, facetSidebar.svelte, dome.ts (STRATUM_LABEL_KEYS already shipped in λ-fix-3 step 1), 23 tradition modules, 15 locale JSONs, ar/masadir.md, _manifests.generated.ts (regenerated by prebuild from the tradition .md files).
>
> ### Boss-test artifact
>
> `Constellation_0.3.4_x64-setup.MIG026-phase-lambda-fix345.exe` (129.6 MB) at `src-tauri/target/release/bundle/nsis/`. Build time: ~7 min. Sanity: `vite build` clean; only pre-existing a11y warnings on unrelated components.
>
> ### Remaining MIG-026 work
>
> - **Phase λ-fix-6** — native-quality re-audit of 336 manifest body translations + 15-locale chip canvas translations (deferred to follow-up).
> - **Phase μ** — ship gate + 3-agent audit. Plan §14.
>
> ### Known doc-drift (carry-forward from v2.13)
>
> 1. `store.ts:3483` — `TraditionId` literal-union duplicate; should import from `types.ts` for single-source.
> 2. Concept Paper §4.1.2 (pramāṇa) — NE/SE/SW/NW → E/S/W/N after §δ.2-fix-1.
> 3. Concept Paper §4.1.3 (masādir) — same NE/SE/SW/NW → E/S/W/N after §θ-fix-1.
> 4. §8 Migrations table — MIG-020 through MIG-025 rows still missing.
>
> ---
>
> *Below: the v2.13 preamble, retained for diff visibility.*

**Version 2.13 | 2026-05-18**

> **What changed in v2.13** (MIG-026 baseline foundation 100% COMPLETE — 24 traditions + 9 shape renderers shipped; phases ι/κ/λ/μ + audit remain):
>
> Cascade through Phases γ → θ + 5 fix-iterations across 2026-05-17 evening + 2026-05-18 early morning closes the MIG-026 tradition-module + shape-renderer foundation. ALL 24 curated baseline traditions registered and Boss-tested PASS; ALL 9 TraditionShape renderers implemented; per-shape star-size-boost + per-shape opacity treatment consolidated. Both doc-drift items (pramāṇa NE→E, masādir NE→E from §δ.2-fix-1 + §θ-fix-1 rotations) flagged for ship-gate cleanup.
>
> **Phases shipped today** (in cascade order):
> γ (Polanyi + Mohist) · δ.1 (Peirce + Habermas + §δ.1-fix-1 rotation) · δ.2 (Dewey + Husserl + Longino + §δ.2-fix-1 star size/chevron/pramāṇa rotation) · ε.1 (Ibn Rushd burhān) · ε.2 (Shāṭibī maqāṣid + §ε.2-fix-1 grid star size) · ε.3 (Ibn Khaldūn ʿumrān) · ζ.1 (PaRDeS) · ζ.2 (Maimonidean spiral + new drawLadderSteps) · ζ.3 (Talmudic 13 middot) · η (Mencian + Wang Yangming + Sŏngnihak + new binary-flow vertical layout) · θ (Mignolo + Dussel + Maldonado-Torres + Akan Wiredu + Ibuanyidanda + new drawRelationalGraph + binary-flow concentric layout) · §θ-fix-1 (masādir rotation + Polanyi star size/opacity + Mohist preview removal) · §θ-fix-2 (sectoral +2 boost + BODY_OPACITY_MULT 0.7→1.0).
>
> Session log: `lab/reports/SESSION-LOG-2026-05-18.md` (today) + `lab/reports/SESSION-LOG-2026-05-17.md` (yesterday — Phase γ through ζ.3).
>
> ### Remaining MIG-026 work
>
> - **Phase ι** — 24 tradition manifests (`docs/traditions/<id>.md`) + ⓘ disclosure-layer UI + scope-strip placement. Plan §11.
> - **Phase κ** — user-definable plugin loader (κ.1 declarative JSON + κ.2 TS plugin loader). Plan §12.
> - **Phase λ** — translation cascade for 15 locales. Plan §13.
> - **Phase μ** — ship gate + 3-agent audit. Plan §14.
>
> ### Known doc-drift (for MIG-026 ship-gate cleanup)
>
> 1. `store.ts:3483` — `TraditionId` literal-union duplicate; should import from `types.ts` for single-source.
> 2. Concept Paper §4.1.2 (pramāṇa) — NE/SE/SW/NW → E/S/W/N after §δ.2-fix-1.
> 3. Concept Paper §4.1.3 (masādir) — same NE/SE/SW/NW → E/S/W/N after §θ-fix-1.
> 4. §8 Migrations table — MIG-020 through MIG-025 rows still missing.
>
> ---
>
> *Below: the v2.12 preamble, retained for diff visibility.*

**Version 2.12 | 2026-05-17**

> **What changed in v2.12** (MIG-027 SHIPPED — Sight follows the interface theme; MIG-026 cascade resumes at Phase γ):
>
> Mid-MIG-026 build cascade (Eisa Boss-testing Phase β), Eisa pivoted: "I want Sight to follow the interface theme. Now it is using the Lighter theme." MIG-026 paused at Phase β; **MIG-027** opened as a focused single-subsystem MIG to deliver the theme-inheritance fix before Phases γ–θ ship the 19 new tradition renderers (so all renderers build theme-aware from the start, not retrofit).
>
> ### Why it matters
>
> Constellation already had a full theme system (6 built-in themes — Constellation / Nord / Solarized × Light / Dark — plus `appSettings.activeThemeId` + `deriveThemeVariables` + `theme-light` / `theme-dark` body classes). Sight v6 was the only major subsystem that ignored it: it always painted in its dark "starfield" palette regardless of the user's choice, leaving a dark hole in an otherwise-light app. MIG-027 wires Sight to the theme system **without inventing a new theme infrastructure** — it just subscribes Sight to what already exists.
>
> ### Architecture: chrome vs semantic split
>
> The core decision was a 2-axis classification of Sight colors:
>
> - **Chrome** (theme-aware): bg, strataRing, calendarRimText, stratumLabel, titleText, subtitleText, statusText, starFill, and (post-fix-2) highlightedRing. These read from CSS variables on `document.body` via `readChromePalette(el)` → `getComputedStyle().getPropertyValue('--name')`. Updates flow synchronously when `+layout.svelte`'s theme `$effect` rewrites the vars.
> - **Semantic** (theme-agnostic): stage hues (cyan = spark, orange = birth, violet = growth, etc.) and the 9 typed-link colors. These keep their categorical meaning regardless of theme — making them theme-aware would lose information.
>
> The split lives in `src/lib/sight/v6/dome.ts` as two exports: `ChromePalette` interface + `CHROME_PALETTE_DARK_FALLBACK` + `readChromePalette()` for chrome; `SEMANTIC_COLORS` for the rest. A legacy `PALETTE` const merges both for backwards-compat with consumers that haven't migrated to the new pattern yet.
>
> Canvas-side: `anchor.ts` and `miniDome.ts` each keep a module-level `let _chrome: ChromePalette` that defaults to the dark fallback; the SightV6.svelte parent reads `chromePalette = readChromePalette(canvasHostEl)` at every paint and passes it through; the renderer sets `_chrome = chromePalette` at the top of the call so all chrome refs inside helpers automatically use the live value.
>
> ### Iterations under Boss test
>
> Three rounds against Eisa's Constellation Light reading:
>
> | Round | Commit | What broke (Boss eye) | Fix |
> |---|---|---|---|
> | **Initial** | `686ee58` | Chrome paints theme-correct; chip row, sidebar, mini-domes all invert | Stage 1 PASS — but chip text + 3 dark-only elements leaked through |
> | **§-fix-1** | `2f190dc` | Hover-info bar (the "E:\…" path tooltip) hardcoded dark navy on cream; filter-count badge bg dark; loading boxes dark; non-active chip text using `--text-muted` went too faint on light bg | Theme-vars on the 3 dark-leaks + chip text bumped `--text-muted` → `--text-normal` |
> | **§-fix-2** | `593af51` | Semantic gold `#fbbf24` (bright amber for dark bg) washes out on cream; reads as "pale peach" on hover-linked facets, filter-count badge text, EXTENDED indicator, and canvas hover ring | Introduce theme-conditional `--sight-highlight` family (4 vars: foreground / bg-soft / bg-strong / border-soft); default = bright amber for dark themes; `:global(body.theme-light) .sight-v6-root` overrides to deep amber `#b45309` (Tailwind amber-700, WCAG AA on cream). Promote `highlightedRing` from `SEMANTIC_COLORS` to `ChromePalette` (canvas reads same CSS var); 2 canvas callsites updated |
>
> Boss test trail (all on the .exe rebuilt from fix-2):
>
> - Stage 1 (initial Constellation Light): **PASS** with the chip+leak feedback
> - Stage 1.1 (after fix-1): **PASS** with the gold-washout feedback
> - Stage 1.2 (after fix-2): **PASS** — "Stage 1.2 PASS, deep amber reads clearly on cream"
> - Stage 2 (Constellation Dark regression): **PASS** — original bright amber preserved, no override leak into dark
> - Stage 2.5 (Nord Light + Solarized Light sanity): **PASS** — both light variants inherit the deep-amber treatment via the shared `body.theme-light` hook
>
> ### Files touched in MIG-027 (across initial + fix-1 + fix-2)
>
> - `src/lib/sight/v6/dome.ts` — chrome/semantic split + `ChromePalette` interface + `readChromePalette()` + `--sight-highlight` read
> - `src/lib/sight/v6/anchor.ts` — module-level `_chrome` + `chromePalette` option in `renderAnchorDome` + canvas hover-ring via `_chrome.highlightedRing`
> - `src/lib/sight/v6/miniDome.ts` — same `_chrome` pattern + canvas hover-ring
> - `src/lib/sight/v6/SightV6.svelte` — reads `chromePalette` at every paint; theme-change `$effect` watches `activeThemeId` + `colorScheme`; CSS sweep + theme-conditional `--sight-highlight` vars (default + `:global(body.theme-light)` override)
> - `src/lib/sight/v6/MiniDome.svelte` — same paint + theme-change `$effect` pattern
> - `src/lib/sight/v6/traditionChip.svelte` — CSS sweep + accent-tinted active-chip bg via `hsla(var(--accent-h)...)`
> - `src/lib/sight/v6/facetSidebar.svelte` — CSS sweep + `--sight-highlight` for hover-linked facet rows
>
> ### What's NOT in MIG-027 (deferred — intentional scope cut)
>
> - **CNS (Constellation Nervous System)** — same dark-only assumption; explicitly out of MIG-027 scope. Probable MIG-028 if Eisa requests.
> - **Sight v3 / v4 / v5** — dark-only; intentionally not touched (deprecated / dual-mounted only).
> - **Star color theme-awareness on Solarized Light** — observed that stars look more muted on Solarized's intentionally-low-contrast cream than on Nord Light or Constellation Light. This is theme-inherent (the SEMANTIC stage hues overlay onto Solarized's low-contrast palette). Boss-acknowledged not-a-bug; would require theme-aware stage luminosity to address, which contradicts the chrome/semantic split. Deferred indefinitely.
>
> ### MIG-026 status post-MIG-027
>
> Phase 0 (K1 rename) · Phase α (multi-shape foundation) · Phase β (A3+A6 chip UI) all shipped before the pivot. **Phase γ (Polanyi + Mohist modules) resumes next** — and inherits theme-awareness for free because the chrome plumbing is already in place. Phases γ → μ continue per the existing 21-step Plan.
>
> ### Doc-drift acknowledged (NOT closed in this commit)
>
> §8 Migrations table is dated 2026-05-07 and only enumerates MIG-001 through MIG-019. MIGs 020 through 026 are missing rows; MIG-027 added here. A focused backfill of the 020–026 rows is a pending follow-up — flagging here so the gap is visible. Recording as a doc-drift item to allocate when MIG-026 ships.
>
> ---
>
> *Below: the v2.11 preamble, retained for diff visibility.*

**Version 2.11 | 2026-05-17**

> **What changed in v2.11** (MIG-026 Architect doc APPROVED; 6 remaining §5.5/§8 architectural choices locked; Plan phase opens):
>
> Eisa reviewed `lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-ARCHITECT.md` (522 lines, drafted 2026-05-17) and locked the 6 remaining architectural choices. Architect phase of the Migration Rule closes; Plan phase opens next.
>
> ### 6 Architectural choices locked
>
> | Choice | Locked | Was-Recommended | Scope delta |
> |---|---|---|---|
> | Chip UI redesign (§3.A) | **A3 + A6** (family categorization + 4 favorites + dropdown for rest) | A1 (multi-row inline) | **Heavier** — Phase β proper UI component |
> | Ladder renderer (§3.D) | **D3** (spiral N-step) | D2 (vertical step-list) | **Novel** — needs spiral path math |
> | Relational renderer (§3.E) | **E3** (hub-and-spoke fixed layout) | E3 (same) | Match |
> | Plugin loader (§3.H) | **H1** (dynamic import in v6.3, Obsidian-trust) | H5 (defer to v6.4) | **Materially heavier** — adds Phase κ.2 |
> | Disclosure layer (§3.J) | **J3 + J5** (ⓘ opens manifest + scope strip) | J3 + J5 (same) | Match |
> | Terminology reframe (§3.K) | **K1** (full rename throughout) | K2 (UI-only) | **Substantially heavier** — adds Phase 0 |
>
> **4 of 6 locked picks are heavier than the Architect's Recommendeds.** Eisa is choosing fidelity over speed, consistent with the "Get it right — take the time" priority. The K1 full rename (`register` → `tradition` throughout code + Concept Paper + i18n + help docs) becomes Phase 0 — must come before any new code so Phases α–μ build under the new namespace.
>
> ### Revised MIG-026 scope estimate
>
> | Original (Recommendeds path) | Revised (Eisa's locked path) |
> |---|---|
> | ~6 main phases | **~10 main phases with 11 sub-phases = 21 build-and-test cycles** |
> | ~2 weeks focused work | **~3–4 weeks focused work** |
> | ~5,000 lines code change | **~10,000–15,000 lines code change** |
> | ~10 commits | **~25–30 commits** |
>
> ### Locked phase decomposition
>
> Phase 0 (K1 rename) · Phase α (architecture foundation) · Phase β (A3+A6 chip UI) · Phase γ (Polanyi + Mohist modules) · Phase δ (Modern Western — 5 traditions, δ.1/δ.2) · Phase ε (Arabic Islamic — 3 traditions, ε.1/ε.2/ε.3) · Phase ζ (Jewish — 3 traditions, ζ.1/ζ.2/ζ.3; ζ.2 = D3 spiral ladder renderer) · Phase η (East Asian — 3 traditions, η.1/η.2/η.3) · Phase θ (Latin American + African — 5 traditions, θ.1–θ.5; θ.1+θ.5 = E3 hub-and-spoke relational renderer) · Phase ι (disclosure + manifests) · Phase κ (user-definable: κ.1 declarative JSON + κ.2 TS plugin loader) · Phase λ (translation cascade) · Phase μ (ship gate)
>
> ### Next deliverable
>
> Plan phase doc at `lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md` — step-by-step build sequence with verification clauses per phase. Once Plan approved, Build cascade per Plan-Approval-Equals-Build-Approval.
>
> ### What this commit does NOT do
>
> - No code touched (Architect + Plan are pre-code documents)
> - No build cascade yet
>
> ### Honest note on cascade pace
>
> The 21 build-and-test cycles will take time. Boss-test happens after each user-testable phase (per Testing Instructions Rule). At ~30 minutes per Boss test, that's ~10 hours of Eisa's time just verifying. Plan doc surfaces which phases are Boss-testable and which are pure refactors.

---

**Version 2.10 | 2026-05-17**

> **What changed in v2.10** (Eisa locks the MIG-026 curated-baseline picks — 19 new registers + 5 existing = 24 total; register-shape architectural finding surfaces):
>
> Eisa, after reading the persisted Agent 1 candidate report (`docs/research/MIG-026-candidate-registers.md`), made his selection from the post-religious-lineage-filter menu. The picks are now locked. With the curated set fixed, the register-shape question (sectoral-only / multi-shape / sectoral+chip-overlay) has concrete geometric implications that didn't exist before.
>
> ### 19 new registers selected for MIG-026 curated baseline
>
> | Family | IN (curated for MIG-026) | OUT (per Eisa pick, beyond the religious-lineage rule) |
> |---|---|---|
> | **Arabic / Islamic (Sunni)** | Ibn Rushd's *burhān/jadal/khaṭāba/shiʿr* ladder · Shāṭibī's *maqāṣid* (3 tiers × 5 necessities) · Ibn Khaldūn *ʿumrān* | Muʿtazilī three-proofs · Ikhwān al-Ṣafāʾ tripartite · modern Arab hermeneutics cluster (Abū Zayd, Arkoun) |
> | **African philosophical** | Akan Wiredu (*nokware / ahonyam / adwene*) · Ibuanyidanda (Asouzu) | — (Ifá divination + Bantu Tempels already filtered by religious-lineage rule) |
> | **Latin American decolonial** | Mignolo pluriversal/border · Dussel transmodernity · Maldonado-Torres coloniality-of-being (three-tier) | Quijano's coloniality matrix |
> | **East Asian philosophical** | Mencian four sprouts · Wang Yangming *liángzhī* / *zhī-xíng héyī* · Korean Sŏngnihak Four-Seven debate | — (Huáyán + Tiāntāi already filtered by religious-lineage rule) |
> | **Indigenous** | — | All 4 candidates excluded per Eisa (Māori whakapapa, Inuit QI, Andean sumak kawsay, Medicine Wheel were already filtered by religious-lineage rule; Eisa confirms blanket exclusion of the family) |
> | **Modern Western beyond Polanyi** | Peirce three categories · Dewey pattern of inquiry · Husserl regional ontologies · Habermas three knowledge-interests · Longino CCE four norms | — (all 5 included) |
> | **Jewish (Abrahamic)** | PaRDeS · Maimonidean prophecy hierarchy · Talmudic 13 *middot* | — (all 3 included) |
> | **Hindu beyond Nyāya** | — | All 3 candidates excluded (Mīmāṃsā, Advaita, Sāṃkhya already filtered by religious-lineage rule; Eisa confirms blanket exclusion of the family) |
> | **Buddhist** | — | All 3 candidates excluded (Dharmakīrti, Madhyamaka, Zen sudden/gradual already filtered by religious-lineage rule; Eisa confirms blanket exclusion of the family) |
> | **Feminist / Standpoint** | — | Collins's matrix-of-domination · Harding/Haraway standpoint (both excluded per Eisa) |
>
> **Total**: 19 new + 5 already shipping (Aristotelian / pramāṇa / masādir / Polanyi / Mohist sān biǎo) = **24 registers** in the MIG-026 curated baseline.
>
> ### Geometric-shape audit of the 24-register set
>
> With the curated set locked, the register-shape question (which has been pending since the Agent 1+2 reports landed) becomes concrete. Each of the 19 newcomers wants a particular geometric vocabulary:
>
> | Geometric category | Registers | Count | Current architecture supports? |
> |---|---|---|---|
> | **Sectoral / quadrant / pie** | Mencian 4 sprouts · Habermas 3 · Peirce 3 · Longino 4 | 4 | ✅ Yes — current `sectorDividers` contract handles |
> | **Concentric rings** | Ibn Rushd *burhān* ladder · PaRDeS · Maldonado-Torres · Husserl regional ontologies | 4 | ⚠️ Partial — need to extend renderer to draw multiple concentric rings (currently `sectorDividers` returns angle-sectors, not ring-bands) |
> | **Multi-tier grid** (2D — sectoral × ring) | Shāṭibī *maqāṣid* (3 × 5 = 15-cell) · Korean Sŏngnihak (2 × 2) | 2 | ⚠️ Partial — extend `sectorDividers` to compose sectoral + ring boundaries |
> | **Cyclic ring with arrow flow** | Dewey's pattern of inquiry (5-segment + chronology arrow) | 1 | ⚠️ Partial — add arrow-flow rendering for cyclic sequences |
> | **Two-cell with directional flow** | Dussel transmodernity · Ibn Khaldūn *ʿumrān* · Wang Yangming | 3 | ⚠️ Partial — two-cell is a sectoral special case; "directional flow" arrows are new |
> | **Two-or-three thin cell** | Akan Wiredu | 1 | ✅ Yes (sectoral special case) |
> | **Ladder / hierarchy** (N>4) | Maimonidean prophecy (11 levels) · Talmudic 13 *middot* | 2 | ❌ No — ladder visualization needs new renderer |
> | **Relational / network** (resists sectoral entirely) | Ibuanyidanda · Mignolo pluriversal | 2 | ❌ No — needs node-link / graph-style renderer (Sky View territory) |
>
> **Implication**: 16 of the 19 newcomers fit sectoral or close parametric variants (rings, grids, cyclic-with-flow) — the current `sectorDividers` architecture can be extended modestly to cover them. 3 of the 19 (Maimonidean ladder, Talmudic 13 *middot*, Ibuanyidanda, Mignolo pluriversal) want genuinely different shapes — ladder and network/relational.
>
> This is the **register-shape question** made concrete: does Constellation extend the architecture to support ladder + relational shapes (multi-shape), or are those 3 registers either (a) reshaped to fit sectoral with fidelity loss, (b) dropped from the baseline, or (c) deferred until the user-definable layer ships and lets users author them externally?
>
> ### What's locked for MIG-026
>
> 1. ✅ **Religious-lineage rule** (orientation v2.09) — applies to all future register additions
> 2. ✅ **Hybrid baseline + user-definable architecture** (Eisa's locked choice — matches dominant mature pattern per Agent 2 research)
> 3. ✅ **Curated-baseline picks** — 19 new + 5 existing = 24 registers (this v-bump)
>
> ### What's still pending Eisa
>
> 4. ❓ **Register-shape decision** — sectoral-only (cut Maimonidean/Talmudic/Ibuanyidanda/Mignolo or force-fit them) / multi-shape (add ladder + relational renderers) / sectoral + chip-overlay (chip-overlay doesn't help Maimonidean or Mignolo)
> 5. ❓ **User-definable architecture approach** — declarative JSON / bounded DSL / sandboxed code / TypeScript plugin / hybrid (declarative + plugin per the dominant mature pattern). Agent 2 cost matrix is the reference.
>
> ### What this commit does NOT do
>
> - No code touched — purely a doc-bump locking the curated-baseline picks
> - No register modules added (all 19 newcomers wait for the MIG-026 build cascade)
> - No user-definable architecture work
> - No translation cascade (manifests will batch with the MIG-026 ship)
>
> ### Per-MIG-026 scope estimate (rough)
>
> - **Code**: 19 new register modules (TypeScript files in `src/lib/sight/v6/registers/`); chip UI extension to handle 24 chips (current row-of-N layout won't fit — needs pagination or categorization); renderer extensions per the shape audit above
> - **Spec docs**: 19 new sections in Concept Paper §4.1.x / §4.2.x with citation grounding; 24 register manifests in `docs/registers/<id>.md`
> - **Translations**: 24 manifests × 14 locales = 336 translation files (likely a follow-up commit per §A.15 → v2.05/v2.06 precedent: English first commit, translations cascade in v2.11+)
> - **Architecture**: user-definable register loader (per the architecture decision still pending) + register manifest schema + on-disk storage spec
> - **CARE / disclosure layer**: per Agent 1's cross-cultural-pluralism findings, registers benefit from scope-statement + citation + critique-awareness UI affordances (the ⓘ chip from Plan §C.7 expanded)
>
> Substantial scope. Likely 3–5 phases inside MIG-026 to ship safely.

---

**Version 2.09 | 2026-05-16**

> **What changed in v2.09** (NEW TOP-PRINCIPAL RULE — religious-lineage rule for registers + Suhrawardi Ishrāqī register EXCLUDED + filtered candidate menu locked):
>
> Eisa, after reviewing the Phase 3 §C cascade so far (§C.1 through §C.4 shipped — Aristotelian / pramāṇa / masādir, plus the Polanyi / Ishrāqī / Mohist chip placeholders), surfaced a substantive design concern about the register set's cultural-religious scope. The result is a new top-principal product-design rule, a corresponding code exclusion, and a filtered candidate menu for future expansion.
>
> ### NEW TOP-PRINCIPAL RULE — Religious-Lineage Rule for Registers
>
> Eisa, 2026-05-16, verbatim direction: *"When we are going to deal with religious references, don't include any non-Abrahamic religious references, and when dealing with Islamic ones, don't consider any Shīʿī reference at all. But when dealing with other heritage or culture scholars, the door is open."*
>
> **Interpretation** (locked with Eisa via AskUserQuestion before any code touched):
>
> | Type of register | Rule |
> |---|---|
> | **Religious-source registers** — frames that organize knowledge by religious-scriptural-authority (cite scripture, prophet, canon, divine revelation as the foundation of knowing) | **Abrahamic only** (Jewish, Christian, Sunni Islamic). NOT Hindu religious, NOT Buddhist religious, NOT Yoruba Ifá, NOT indigenous-religious, NOT Daoist-as-religion, NOT Shīʿī Islamic. |
> | **Heritage / culture scholar registers** — frames from philosophical, epistemological, hermeneutic, sociological, ethical, or critical-theoretical traditions, even when those traditions have religious context | **Open door**, any culture. Peirce, Habermas, Polanyi, Longino, Quijano, Dussel, Mignolo, Ibn Khaldūn, Wiredu (Akan), Asouzu (Igbo) etc. all welcome. |
>
> **The dividing line** is the *type of authority* the frame invokes: religious-source frames appeal to scripture / prophet / canon / divine revelation; heritage frames appeal to reasoning / observation / lived experience / argument.
>
> **Strict-lineage application** (Eisa's choice from the AskUserQuestion): if the lineage is religious — even if modern academic treatment is philosophical — the register is OUT. Two exceptions are grandfathered because they shipped before this rule landed:
> - **pramāṇa** (already shipped, §C.3): Nyāya epistemology. Strictly āstika Hindu philosophical lineage, but framed as the epistemological cognitive-act analysis, not the Vedic-authority frame. Stays.
> - **Mohist sān biǎo** (chip placeholder for §D.3): Chinese pragmatist methodology with Heaven-theology context but methodologically secular. Stays.
>
> All other Hindu, Buddhist, Yoruba (Ifá), Daoist-as-religion, indigenous-religious, and Shīʿī Islamic candidates are excluded.
>
> ### Code change shipped in this v-bump
>
> **Suhrawardi Ishrāqī register EXCLUDED entirely** under the new rule (same §C.1-fix-1-style cascade as Dignāga). The Ishrāqī tradition (Suhrawardi, 1154–1191) was overwhelmingly absorbed into Twelver Shīʿī ḥikma (Mulla Sadra, Sabzavari, modern Qom seminary curriculum) — failing the Sunni-only restriction — and is fundamentally religious-mystical theology (ʿilm ḥuḍūrī, mystical presence-knowledge) rather than philosophical-epistemological scholarship. Both clauses of the rule fire.
>
> Seven surfaces updated in lockstep:
>
> | Surface | Change |
> |---|---|
> | `src/lib/sight/v6/registerChip.svelte` | REGISTERS array entry removed; header comment updated to "5 registers"; diacritic-list updated. |
> | `src/lib/sight/v6/types.ts` | `RegisterId` union literal `'ishraqi'` removed; comment block updated with religious-lineage rule citation. |
> | `src/lib/libraries/store.ts` | `activeRegister?:` union literal removed; new idempotent migration block in `applyParsedSettings` rewrites any persisted `activeRegister: 'ishraqi'` → `'aristotelian'`. |
> | `lab/reports/MIG-025-SIGHT-V6-PLAN.md` §D.2 | SUPERSEDED / EXCLUDED with original 3-bullet spec struck through. |
> | `docs/Constellation-Sight-Concept-Paper-v4.0.md` §4.2.2 | Header "Ishrāqī — EXCLUDED in v6 (§C.4-religious-rule)" + status note. Original geometry/citation prose preserved as scholarly background but explicitly labeled "preserved for academic reference, not for build". §4.2 heading updated: "(1 register, originally 3 — Dignāga + Ishrāqī excluded by product decision)". |
> | `docs/Constellation — Universal Orientation.md` | v1-preview register list edited; Ishrāqī removed with a parenthetical noting the religious-lineage rule. |
> | `docs/Constellation Orientation & Onboarding v2.09.md` (this file) | NEW. Documents the rule + the exclusion + the filtered candidate menu. |
>
> ### The filtered candidate menu (post-rule)
>
> Eisa requested two parallel research agents to survey (a) candidate registers across world traditions, and (b) user-definable register architecture. Agent 1 returned ~25 strong candidates; after the religious-lineage rule applies, ~16 remain. Net set for future expansion consideration:
>
> | Family | IN under rule (candidates for future MIG) | OUT under rule |
> |---|---|---|
> | **Arabic / Islamic (Sunni)** | Muʿtazilī three-proofs · Ibn Rushd's *burhān/jadal/khaṭāba/shiʿr* ladder · Shāṭibī's *maqāṣid* (3 tiers × 5 necessities) · Ikhwān al-Ṣafāʾ tripartite · Ibn Khaldūn *ʿumrān* · Abū Zayd / Arkoun modern hermeneutics | Twelver Shīʿī uṣūl |
> | **African (philosophical)** | Wiredu's Akan *nokware/ahonyam* · Asouzu's Ibuanyidanda | Yoruba Ifá (divinatory religion); Bantu Tempels (religious cosmology — also contested as ethnography by Hountondji) |
> | **Latin American decolonial** | Quijano's coloniality matrix · Mignolo's pluriversal/border · Dussel's transmodernity · Maldonado-Torres's three-tier coloniality | — |
> | **East Asian (philosophical)** | Mencian four sprouts · Wang Yangming *liángzhī* · Korean Sŏngnihak Four-Seven debate | Huáyán four dharmadhātu, Tiāntāi three truths (Buddhist religious metaphysics); Daoist *wuwei/ziran* (religion-philosophy) |
> | **Indigenous** | — | Māori whakapapa, Inuit Qaujimajatuqangit, Andean sumak kawsay, Medicine Wheel (all carry religious-cosmological dimensions; OUT under strict reading) |
> | **Modern Western (secular)** | Peirce's three categories · Dewey's pattern of inquiry · Husserl's regional ontologies · Habermas's three knowledge-interests · Longino's CCE four norms · Collins's matrix-of-domination · Harding/Haraway standpoint (as meta-register) | — |
> | **Jewish (Abrahamic)** | PaRDeS · Maimonidean prophecy hierarchy · Talmudic 13 *middot* (or 7 Hillel) | — |
> | **Hindu beyond Nyāya** | — | Mīmāṃsā six pramāṇas, Advaita Vedānta, Sāṃkhya 25 tattvas (all religious-Vedic-authority) |
> | **Buddhist** | — | Dharmakīrti two pramāṇas, Madhyamaka *catuṣkoṭi*, Zen sudden/gradual (all Buddhist religious lineage) |
>
> **Net post-rule**: ~16 candidates for future expansion + 5 currently shipping (4 production: Aristotelian / pramāṇa / masādir / Polanyi + 1 v1-preview: Mohist sān biǎo).
>
> Full candidate scholarly grounding (citations, geometry implications, scope statements) lives in the Agent 1 research transcript (task ID af89a1ab4dc8eba84). Eisa picks which candidates to actually ship in the next MIG.
>
> ### Architecture decisions still pending (post-Agent 1+2 research)
>
> - **Register-shape support**: sectoral-only (current) vs. multi-shape (sectoral + relational-graph + matrix + coordinate-meta + chip-overlay). Question still open while Eisa reviews Agent 1's §3 finding that many strong traditions resist the sectoral format.
> - **Curated-baseline + user-definable architecture relationship**: Eisa chose **Hybrid** (the mature pattern — Obsidian / VS Code / Quarto / Microsoft 365 Copilot all do this). Curated baseline + user-definable architecture ship together in the next MIG.
> - **User-definable approach**: from Agent 2's 4 options (declarative JSON · bounded DSL · sandboxed code · TypeScript plugin) — still open. Likely a hybrid (declarative for simple cases + plugin for arbitrary geometry).
>
> Phase 3 §C cascade is **paused at §C.4** (Aristotelian + pramāṇa + masādir shipping; Polanyi in plan but un-built; Ishrāqī excluded; Mohist sān biǎo as chip placeholder grandfathered). The right next move is a fresh MIG (likely MIG-026) for the register-set expansion + user-definable architecture, with its own /architect → /plan phases, rather than continuing the §C cascade as originally designed.
>
> ### What this commit does NOT do
>
> - No new register modules built. Polanyi (§C.5), Mohist (§D.3), and all candidate registers wait for the MIG-026 plan.
> - No translation cascade (the manifests + 14-locale mirrors wait for the curated baseline to be locked).
> - No user-definable architecture work. Waits for the MIG-026 architect doc.
> - No reframing of "epistemic register" → "scholarly tradition" terminology (Direction D from the original questionnaire). Also waits for MIG-026.
>
> ### Next concrete decision points
>
> 1. Eisa finishes reading Agent 1's full candidate report (the survey is in the agent transcript).
> 2. Eisa decides register-shape question (sectoral-only / multi-shape / sectoral+chip-overlay).
> 3. Eisa picks the curated-baseline candidates to ship (from the ~16 filtered above + the 5 already shipping).
> 4. Eisa picks the user-definable architecture approach.
> 5. A fresh `/architect` document for MIG-026 is drafted with those four decisions locked, then `/plan`, then build cascade.

---

**Version 2.08 | 2026-05-16**

> **What changed in v2.08** (MIG-025 §C.1-fix-1 ships — Dignāga register EXCLUDED + Sight Esc-while-chip-expanded bug fixed):
>
> Two product changes, one commit, surfaced from Eisa's Stage 1 Boss-test of §C.1 (register chip component).
>
> ### Dignāga register permanently excluded
>
> Eisa's direction during §C.1 Stage 2 review of the chip: **"don't include the 'Dignāga' at all in any of Constellation functions."** The register set shrinks from 7 to 6 (4 production-polish + 2 v1-preview, where v1-preview is now just Suhrawardi Ishrāqī and Mohist sān biǎo).
>
> Six surfaces updated in lockstep so doc-drift never opens:
>
> | Surface | Change |
> |---|---|
> | `src/lib/sight/v6/registerChip.svelte` | REGISTERS array: dignaga entry removed (was index 4). Header comment updated: "6 registers in canonical order". |
> | `src/lib/sight/v6/types.ts` | `RegisterId` union: `'dignaga'` literal removed. Comment block updated. |
> | `src/lib/libraries/store.ts` | `activeRegister?:` union: `'dignaga'` removed. New idempotent migration block in `applyParsedSettings` rewrites any persisted `activeRegister: 'dignaga'` → `'aristotelian'` (could exist if Eisa or another tester clicked Dignāga during §C.1 Stage 1 before this fix shipped). |
> | `lab/reports/MIG-025-SIGHT-V6-PLAN.md` §D.1 | Marked SUPERSEDED with EXCLUDED note. The original 3-bullet spec is struck through; the section header now reads "Dignāga register — SUPERSEDED / EXCLUDED (Eisa 2026-05-16, §C.1-fix-1)". |
> | `docs/Constellation-Sight-Concept-Paper-v4.0.md` §4.2.1 | Section header "Dignāga — EXCLUDED in v6". Added a status note explaining the exclusion. Original geometry/citation prose preserved as scholarly background but explicitly labeled "preserved for academic reference, not for build". §4.2 heading updated: "(2 registers, was 3 before §C.1-fix-1)". |
> | `docs/Constellation — Universal Orientation.md` | v1-preview register list edited: now reads "Ishrāqī, Mohist sān-biǎo. *(Dignāga was originally on this list; permanently excluded 2026-05-16 by product decision.)*" |
>
> Phase 4 §D (post-§C work) now has only 2 v1-preview register build steps left: §D.2 (Suhrawardi Ishrāqī) and §D.3 (Mohist sān biǎo).
>
> ### §C.1 Esc-while-chip-expanded bug fixed
>
> Stage 6 Step 4-5 FAIL during Eisa's Boss-test: pressing Esc while the chip was expanded closed Sight instead of just collapsing the chip.
>
> **Root cause**: `+layout.svelte:2335` registers the global "Esc closes Sight overlays" handler on `document` in **capture phase**. The §C.1 chip handler was on `document` in **bubble phase**. Capture-phase handlers fire before bubble-phase handlers, so Layout's close-Sight handler ran first and closed Sight before the chip's `stopPropagation()` could even execute.
>
> **Fix**: chip handler now registers on `window` (which sits outside `document` in the capture chain) in capture phase. `stopPropagation()` + `preventDefault()` kill the event before it propagates down to Layout's document-capture handler. Capture order is `window → document → ...`, so a `window` capture handler fires BEFORE any `document` capture handler. Belt-and-braces in the handler body (both stopPropagation + preventDefault) for defence-in-depth.
>
> ### Cascade resumes from §C.2 after Stage 1 re-test passes
>
> No other regressions surfaced. §C.1 Stages 1–5 and 7 PASSED; Stage 6 Step 4-5 was the only FAIL. Eisa re-tests §C.1 against this fix; on PASS the cascade resumes to §C.2 (Aristotelian register module — the first step that makes the chip actually re-arrange the dome).

---

**Version 2.07 | 2026-05-16**

> **What changed in v2.07** (Universal Orientation doc created — new external-facing briefing surface):
>
> Eisa requested a self-contained orientation document he can paste into Claude Chat conversations when he asks an outside AI to do research on Constellation's behalf. This v-bump documents the new file's existence so future Claude Code sessions know it exists and what it's for.
>
> ### New file
>
> **`docs/Constellation — Universal Orientation.md`** (~3,500 words, 364 lines, no version suffix). Self-contained briefing structured for one-pass reading by an AI assistant who has never seen the repo. 15 sections: identity, the negation list (what Constellation is NOT), formulation-vs-management distinction, the Five Acts, the four-level knowledge hierarchy, the Living Link Architecture (8 link properties + 7 types + 4 confidence levels + 7 lifecycle stages), the surfaces (NotePane, FocusPane, Sight, CNS, Map, Sky View, Index, SearchHub), multilingual-by-design, local-first + file-over-app, performance philosophy, current state (shipped vs in-flight vs refused), constraints that cannot be violated, good-vs-bad research-question shapes, vocabulary cheat-sheet, paste-anywhere one-paragraph summary.
>
> ### Why it lives where it lives
>
> Sits at `docs/` root alongside this versioned series. Filename intentionally has no version suffix — the Universal Orientation is refreshed in place when the project state diverges meaningfully from its content. The versioned `Constellation Orientation & Onboarding vX.Y.md` series remains the canonical internal record (architectural fluency for fresh Claude Code sessions); the Universal Orientation is the external-facing complement.
>
> ### What this commit does NOT do
>
> - No code touched.
> - No feature change.
> - No translation refresh.
> - No new MIG opened. Pure documentation.
>
> ### Next pivot (unchanged from v2.06)
>
> Phase 3 §C — register chip + 4 production-polish registers (Aristotelian default + pramāṇa + masādir + Polanyi) — opens on Eisa's signal.

---

**Version 2.06 | 2026-05-16**

> **What changed in v2.06** (14 language mirrors land for Sight + CNS help docs — translation cascade closes the v2.05 follow-up):
>
> v2.05 shipped the English source for the two new help topics (Sight v6.1 rewrite + the first-ever CNS help doc) and was explicit that the 14 language mirrors were pending the follow-up commit. This is that follow-up.
>
> ### 28 new translation files
>
> For each of the 14 non-English locales — **ar, de, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh** — two new files now exist:
>
> - `docs/help.{lang}/Constellation Sight/Constellation Sight.md`
> - `docs/help.{lang}/Constellation Nervous System/Constellation Nervous System.md`
>
> Every file carries a frontmatter disclosure:
>
> ```yaml
> ---
> translation_status: AI-generated 2026-05-16 — native-speaker review recommended
> language: <code>
> source: docs/help.uConstellation.World/...
> aliases: [...localized...]
> description: <localized one-paragraph summary>
> ---
> ```
>
> The disclosure is structural, not cosmetic — it tells future native-speaker reviewers exactly which file is the source and what review state the translation is in. Reviewers can grep for `translation_status: AI-generated 2026-05-16` to find every file added in this cascade.
>
> ### Translation conventions (consistent across all 14 locales)
>
> - **Brand names kept in English**: Constellation, Sight, CNS, Confidence, Stage, Acts, Provenance, Folder, Library, Stratum, Modularity, Dominance, Entropy, Connectivity, EXTENDED, HEALTHY, CAUTION, IMBALANCED. Same precedent as §A.15 brand convention (`plugins.constellationSight`, `plugins.constellationMap`).
> - **Lifecycle stage code names kept in English**: `spark`, `birth`, `growth`, `maturity`, `dormancy`, `renewal`, `archival`. These map to YAML values on disk; localizing them would break the user's actual notes.
> - **Stratum and facet values kept in English**: Foundation / Roots / Trunk / Branches / Twigs / Edge of Knowing; Hypothesis / Evidence / Established / Contested; Spark / Birth / Growth / Maturity / Dormancy / Renewal / Archival; Self / Read / Heard / Reasoned / Tradition. They correspond to actual schema enums.
> - **Keyboard shortcut tokens kept literal**: Ctrl+D, Cmd+Shift+D, Esc, Shift+click. Localizing these would mislead.
> - **UI button labels kept literal**: "Return to Sight", "Return to CNS", "Reset View", "(×)". They appear in the running app exactly as quoted.
> - **Prose translated**: section headings, body paragraphs, interaction-table cells (gesture + effect), the "When most useful" lists, the "Related Surfaces" footers.
> - **RTL languages (ar, fa, he, ur)**: prose flows naturally right-to-left in the rendered markdown; brand tokens stay LTR inline.
>
> ### Doc-drift item from v2.05 — now CLOSED
>
> v2.05 was explicit that the 14 mirrors were pending the v2.06 follow-up commit. That item is now closed. The English drift was closed in v2.05; the translation drift is closed in v2.06. The §A.15 documentation-drift cycle (carried since the rename) is fully resolved across all 15 locales.
>
> ### What this commit does NOT do
>
> - **No User Manual translation pass.** `docs/help.{lang}/User Manual.md` already exists in 14 locales but they predate this commit; Sections 8/8b/10d in those manuals still describe v5 mode-toggle Sight. Updating those is a separate cascade — not blocking the Sight/CNS help topic which is the in-app reference users actually see when they click the help button.
> - **No native-speaker review.** The translations are AI-generated, disclosed as such, and ship with the explicit `native-speaker review recommended` tag. Eisa will route them through native reviewers per the existing translation-review workflow.
> - **No new feature work.** This is pure documentation; no code touched, no IPC contracts changed, no schema migration.
>
> ### Next pivots (unchanged from v2.05)
>
> - **Phase 3 §C** opens next per the Plan — register chip + 4 production-polish registers (Aristotelian default + pramāṇa + masādir + Polanyi).
> - Open: CECE Source Review queue (4,475 pending suggestions; user-task, populates the Provenance mini for real once approved).
> - Open: future MIGs queued — Sight Settings UI section, Confidence-population, v6.2 d3-hexbin polish, User Manual translation refresh.

---

**Version 2.05 | 2026-05-16**

> **What changed in v2.05** (Help-doc rewrite for Sight v6.1 + new CNS help doc + User Manual update + 14 language mirrors):
>
> The long-deferred help-doc drift (carried since §A.15 rename) is now resolved. Three sets of documentation land in this version:
>
> ### English help docs
>
> - **`docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md`** — full rewrite for v6.1 Coordinated Views. Replaces the v5 six-mode toggle content. Covers anchor dome, 4 mini-channel encodings, dome-swap, bidirectional linked brushing, Shift+click cross-filter, ghost mode, Extended view, density mode, the facet sidebar's 6 facets, and every gesture. ~200 lines, ~1,500 words.
> - **`docs/help.uConstellation.World/Constellation Nervous System/Constellation Nervous System.md`** — NEW. v2 (CNS) had no dedicated help doc until now (Lens.md is about DQL queries, an unrelated feature). Covers Universe Health card + four metrics, gravity well + community detection, Top Bridges (synthesis points), Blind Spots (structural gaps). Documents CNS's single-click-preview / double-click-open pattern that Eisa locked in on 2026-05-16. ~130 lines, ~1,000 words.
>
> ### User Manual sections
>
> - **Section 8 — Constellation Sight** rewritten for v6.1. Concise overview; full reference points to in-app help topic.
> - **Section 8b — Constellation Nervous System (CNS)** NEW. Mirrors the help-doc structure at User-Manual brevity.
> - **Section 10d — Sight v5** marked SUPERSEDED with a header note pointing to Section 8.
>
> ### 14 language mirrors — pending next commit
>
> The English source ships in v2.05. The 14 language mirrors (`docs/help.ar/...` through `docs/help.zh/...`) for both new help docs land in a follow-up commit + v2.06 bump, with each translation carrying a frontmatter `translation_status: AI-generated 2026-05-16 — native-speaker review recommended`. None of the 14 mirror dirs currently have Sight or CNS content; 28 new files will be created across the 14 dirs in the translation cascade.
>
> ### Doc-drift item closed (English) / pending (translations)
>
> The "help docs still describe v5" item carried in v2.04 §12 is now FIXED for the English source. The 14 translation mirrors are pending the follow-up commit (v2.06). Future drift will surface as Sight v6.2 / v6.3 evolution; this commit closes the v6.1-vintage English gap.
>
> ### Next pivots
>
> - **Phase 3 §C** opens next per the Plan — register chip + 4 production-polish registers (Aristotelian default + pramāṇa + masādir + Polanyi).
> - Open: CECE Source Review queue (4,475 pending suggestions; user-task, populates the Provenance mini for real once approved).
> - Open: future MIGs queued — Sight Settings UI section, Confidence-population, v6.2 d3-hexbin polish.

---

**Version 2.04 | 2026-05-16**

> **What changed in v2.04** (Sight v6.1 SHIPS · MIG-025 §B closes · Phase 3 §C opens):
>
> ### Sight v6.1 ships — Coordinated Views interaction model complete
>
> Phase 2 (§B.6 → §B.10) closed across ~14 fix iterations + 4 base commits. The dome-as-instrument model that Concept Paper v4.0 specified is now live end-to-end:
>
> **§B.6 — Bidirectional linked brushing + dome-swap.**
> - Hover any star in any of the 5 surfaces (anchor + 4 minis) → gold ring on the same note in all 5.
> - Click empty area of any mini → that mini promotes into the primary slot; previous primary demotes into the vacated mini slot. Click again to shuffle.
> - Promoted mini gets full zoom/pan support (wheel-zoom, drag-pan, Cmd-0 reset).
> - "Reset View" button in header returns to default in one tap.
> - "Return to Sight" button in note tab-bar after opening a note.
>
> **§B.7 — Cross-filter from any dome.**
> - **Shift+click on a star** in Stage / Confidence / Provenance mini → filter universe to that star's category. All 5 surfaces re-render.
> - **Ghost mode:** non-matching stars stay visible at low opacity (0.15) instead of vanishing. Shift+click a ghost to ADD its category to the filter — multi-select within a facet from the dome.
> - **Zoom-aware hover ring** — ring radius/stroke scale inversely with zoom so screen size stays bounded.
> - **Filter affected-count badge** in header shows `X / Y notes`.
> - **Hover any star → highlight matching sidebar chips.** Closed bidirectional loop. Eisa: "this new feature is making the Sight function smarter, and it will help users better understand their universe."
>
> **§B.8 — Cross-filter perf gate.** Manual verification via Eisa cycle reports. Automated vitest harness deferred to §D.4.
>
> **§B.9 — Density aggregation when matched > hexBinThreshold (5000).** Channel renderers lower per-star alpha for additive-blend perceptual density. Full d3-hexbin → v6.2 polish.
>
> **§B.10 — Extended view persistence** (renamed from "Pro mode" per Eisa: "Pro overpromised"). Cmd-Shift-D toggles `appSettings.sight.extended` and persists. Per-session Cmd-D preserved. "EXTENDED" badge in header. Full schema rename `proMode` → `extended` with migration.
>
> ### Bonus cleanup
>
> The `lastMode` v5 dead-key (Eisa noticed surviving in settings.json despite §A.12 v6MigrationDone=true) is now cleaned up idempotently by a new migration block in `applyParsedSettings`.
>
> ### Phase 2 ship-gate clauses (per Plan §B.11)
>
> | Clause | Status |
> |---|---|
> | Four mini-domes render with isolated channel encoding | ✓ §B.1–§B.5 |
> | Stratum bands visible in each mini | ✓ §B-fix-2 |
> | Linked brushing across all 5 views | ✓ §B.6 |
> | Click in mini-dome filters all 5 views | ✓ §B.7 (Shift+click) |
> | Hex-bin above 5,000 visible | ✓ §B.9 (density mode; true d3-hexbin → v6.2) |
> | Cmd-D toggles diagnostics visibility | ✓ §B.1 |
> | Pro mode persists across sessions | ✓ §B.10 (renamed to Extended) |
> | Cross-filter perf test ≤16 ms | manual ✓ (automated → §D.4) |
> | Register chip area HIDDEN entirely | ✓ Phase 1 baseline |
>
> All clauses met or accounted for. **Sight v6.1 SHIPS.**
>
> ### Phase 3 (§C) opens next
>
> Register chip + 4 production-polish registers (Aristotelian default + pramāṇa + masādir + Polanyi) + register manifests. v1-preview registers (Dignāga + Ishrāqī + Mohist sān-biǎo) labeled but unfinished.
>
> ### Doc-drift item carried forward
>
> Help docs (`docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md`) still describe Sight v5's six-mode toggle architecture. CNS still has no dedicated help doc. Rewrite-from-scratch task; the natural time is **now** (v6.1 ships with stable feature set; help docs can land before Phase 3 opens).

---

> **What changed in v2.03** (Sight rename: v6 → "Constellation Sight" · v2 → "Constellation Nervous System (CNS)" · MIG-025 §A.15):
>
> ### Two visualization surfaces, two clean names
>
> After the §A.14 ship + the §B-preview A/B test where Eisa kept v2 alongside v6 ("Sight v2 = Working. I decided to keep it."), a 5-SME naming panel ran on what to call them. The SMEs offered Atlas + Threads (cleanest), Sight + Threads (pragmatic), Atlas + Web (Web killed by Cross-Civ for Arabic/Persian dignity-loss). Eisa rejected all of them and picked his own pair:
>
> - **v6 → Constellation Sight** — the canonical Sight, the anchor-dome view.
> - **v2 → Constellation Nervous System (CNS)** — the connection-traversal view (Universe Health card + bridges + communities + Blind Spots).
>
> The pairing is anatomical-complementary (Sight = sensory, CNS = neural) and translates cleanly across all working languages: الجهاز العصبي (Arabic), دستگاه عصبی (Persian), 神经系统 (CJK), מערכת העצבים (Hebrew). The CNS acronym already carries scholarly resonance (Central Nervous System), giving "Constellation Nervous System" a recognized referent the SMEs hadn't surfaced.
>
> Eisa-confirmed grammar: **"Nervous System"**, not "Nerve System". The former is the canonical English anatomical term; the latter sounds like a translation artifact.
>
> ### What this commit changes (§A.15)
>
> - **15 i18n locales** updated: `plugins.constellationSight` value → "Constellation Nervous System (CNS)"; `plugins.constellationSightDesc` value → connection-traversal description; `lens.title` value (the v2 dock-button label) → "Constellation Nervous System (CNS)" (English brand kept across all 15 per the existing `constellationMap`/`constellationSight` precedent — the 3 locales that had previously translated `lens.title` to Arabic / Persian / Urdu were re-aligned to the English brand convention); `settings.sight.intro` value updated to describe v6 (drops the stale v3 reference).
> - **`+layout.svelte`** v6 dock-button `aria-label="Constellation Sight v6"` → `"Constellation Sight"`. (No other markup changes; the existing `title={$t('sight.v6.title') || 'Constellation Sight'}` already rendered the right text via fallback.)
> - **`engine.ts`** header comment gains a USER-FACING NAMES block stating the rename + the architectural-history convention that internal v-numbers, file names, IPC names, and engine flags are retained (same precedent as MIG-005 "Lens" → "Sight").
>
> ### What this commit deliberately does NOT change (deferred to a separate commit)
>
> - **Help docs.** `docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md` currently describes Sight **v5** (six cognitive-lens modes — R/L/T/C/S/A/P). v2 has **no dedicated help doc** at all (the existing `Lens/Lens.md` is about DQL queries, an unrelated feature). Both need full prose rewrites — not mechanical edits — and carry through to 14 language mirrors (`docs/help.{lang}/`). **Doc-drift item; tracked in §12 below.**
> - **`docs/User Manual.md`** Sight chapter is similarly stale relative to v6.
> - **Internal symbols** (file names, Rust IPC names, Svelte state vars, engine flags, settings-flag keys, i18n key paths) are unchanged. Same precedent as Lens → Sight: rename labels, keep code-history.
> - **Locale translations of the new descriptions.** Only English values changed in this commit; the 14 other locales already had English values for these brand keys (translation drift predates §A.15). A future translation pass can localize the description prose if Eisa wants.
>
> ### Function in hand from §A.15 onwards
>
> Sight v6 (now just "Sight") sits at the visual-foundation phase complete; CNS (v2) sits beside it as the kept-alive complement. Phase 2 (§B.6 → §B.11 — linked brushing across views, cross-filter from mini-dome category, hex-bin perf gate, Pro-mode persistence, Phase-2 ship gate) opens next, building on the renamed Sight surface.

---

> **What changed in v2.02** (Sight v6.0 SHIPS · MIG-025 §A closes · Phase 2 §B opens):
>
> ### Sight v6.0 ships — `SIGHT_V6_ENABLED = true` on `main`
>
> The v5 → v6 architecture pivot opened in v2.01 has now landed in production. After the §A.14 ship gate (Boss-test cycle, 7 NSIS builds, 16 fixes), Eisa accepted cycle-3.7 with "Ship". `SIGHT_V6_ENABLED` is now `true` on `main`; v5 stays mounted alongside via B2 dual-flag through Phases 1–3, deleted in §D.6 (Phase 4).
>
> **Phase 1 user-facing surface** (the v6.0 deliverable):
>
> - **Anchor dome** with stratum × time positioning, sub-pixel additive-blending density rendering at default zoom, all-circle nodes, wheel-zoom up to 24× for crystal-clear individual inspection (5 px ⌀ at zoom 8×, 15 px ⌀ at zoom 24×).
> - **Hearst Flamenco facet sidebar** — 6 facets in display order: Folder (TOP per §11 invariant 8) · Library · Stratum · Confidence · Stage · Provenance. Click any category to cross-filter; counts in other facets rebalance live.
> - **First-boot orientation tour** — 4-step skippable overlay; gated by `appSettings.sight.tourSeen`.
> - **Gesture grammar** — wheel zoom (toward cursor), drag pan (4-px drag-threshold so click-to-open still works), hover for tooltip + gold ring, click to open in editor, Cmd-0 / Ctrl-0 reset zoom + pan, Esc resets, sidebar tab to expand/collapse.
> - **B2 dual-mount** — v5 dock button still works; mutually exclusive with v6.
> - **Settings migration** — `lastMode` dropped quietly; `lastScope` preserved as dead key for v5 fallback safety; `v6MigrationDone: true` sentinel stamped; 4 new v6 fields defaulted (`proMode`, `activeRegister: 'aristotelian'`, `hexBinThreshold: 5000`, `linkFadeThreshold: 800`).
>
> **Phase 1 commits** (16-step cascade, all on `main`):
>
> ```
> 3e829f6  docs(sight): MIG-025 opens — Concept Paper v4.0 + Architect + Plan
> a0c0af5  §A.1  SIGHT_V6_ENABLED flag (false)
> aa17e10  §A.2  cache schema + 9 unit tests
> baa4a95  §A.3  synchronous backfill skeleton + 8 tests
> a38b256  §A.4  progressive backfill + Tauri events + frontend store + 8 tests
> 970d2bf  §A.5  3 Tauri IPCs (warm_cache, get_layout, get_link_set_for_notes)
> b981129  §A.6  frontend types + module skeleton
> 7a948e9  §A.7  mount in +layout.svelte (B2 dual-mount, dock button)
> 5b3e10b  §A.8  anchor dome chrome (5 strata, calendar rim, labels)
> e5b2334  §A.9  anchor stars + lines + IPC integration + hit-test
> c4b7e7b  §A.10 facet sidebar (Hearst Flamenco)
> c84989b  §A.11 first-boot orientation tour
> 1048ab1  §A.12 settings migration + 4 new v6 fields
> 251d630  §A.13 CI perf-harness skeletons (vitest+playwright deferred)
> ```
>
> Then 16 §A.14 Boss-test fixes across 7 cycles:
>
> ```
> d0e683c  fix-1..4   chrome contrast, jitter widening, smaller stars, hover-title
> 3c70896  fix-5      applyParsedSettings shared helper (boot-bundle drift —
>                     loadSettings had ZERO callers; migration was dead code)
> 6acde74  fix-6..9   brighter chrome, additive density blending, two-pass
>                     render, wheel-zoom + drag-pan + Cmd-0
> 59523f1  fix-10     zoom regression (clear+bg in identity transform)
> f79d26f  fix-11     addEventListener wheel binding (Svelte template
>                     binding silent-failed in cycle-3 multi-edit batch)
> d70ceb1  fix-12     phyllotaxis spiral packing
> 1efadb5  fix-13     node sizing 5 px ⌀ at max zoom (default → density chart)
> 989507a  revert     fix-12 (jitter wins A/B test per Eisa)
> ecabc16  fix-15     all notes render as circles
> f8de004  fix-16     hover-ring screen-padded + ZOOM_MAX 8→24×
> ```
>
> Final ship commit: this commit (engine.ts SIGHT_V6_ENABLED=true permanent, orientation v2.02, session log).
>
> ### Three architectural truths discovered during the cycle (carry forward)
>
> 1. **Sight is a diagnostic, not a navigator.** At 7,650-note scale the dome shows the universe's *shape* (where dense, where empty, where bright); identifying individual notes happens via the facet sidebar (filter down) → zoom-in → click-to-open. Trying to make the dome show individual identities at default zoom is fighting the math; the workflow is filter + zoom.
> 2. **Boot-bundle drift is a real cross-cutting risk.** `+layout.svelte:1880` had drifted from `loadSettings()` and become the de-facto load path with `loadSettings()` having zero callers. My §A.12 migration block in `loadSettings()` was dead code that only fired for theoretical use cases. Fix-5 extracted `applyParsedSettings()` as the single source of truth. Worth a §N audit follow-up to verify no other parsed-settings consumers exist OR settings-mutation paths bypass the load merge.
> 3. **Working Agreement #4 violations during multi-edit batches.** Fix-9's commit `6acde74` claimed it added zoom + drag handlers, but the Edit tool silently failed on the canvas-markup edit mid-batch — handlers existed but were never bound. Fix-11 caught it cycle-3.1 by adding the addEventListener fallback pattern AND completing the missing markup. Lesson: multi-edit batches need explicit verification step after, OR parallel-agent review per Working Agreement #4. Cost: one extra Boss-test cycle + a fresh NSIS build.
>
> ### Phase 2 (§B) opens for next cascade
>
> Mini-domes (Confidence opacity, Stage hue, Acts size, Provenance sectors) + cross-filter brushing + Pro mode (Cmd-Shift-D persistent toggle). ~4 weeks per Plan §A.2. Phase 3 (§C) register chip + 4 production registers; Phase 4 (§D) 3 v1-preview registers + CI hardening + v5 deletion.
>
> ### v4.1 polish targets allocated (pending PJ-NNN)
>
> - **Hex-bin aggregation** for the anchor dome at 50k+-note universes (Concept Paper §3.4 deferred).
> - **Library color recognition aid** — optional low-saturation tint per Concept Paper §6.4 escape hatch.
> - **Three v1-preview registers polish** — Dignāga, Suhrawardi Ishrāqī, Mohist sān biǎo (production-grade rendering when Phase 3 ships).
> - **Pramāṇa internal-structure rendering** — per-quadrant indriya-artha-sannikarṣa loci (deferred from §4.1.2).
> - **Register-aware mini-domes** — masādir relabels Confidence as qaṭʿī/ẓannī etc. (~3 wk).
> - **Color-accessibility variant** — high-contrast / colorblind-safe palette.
>
> ---

> **What changed in v2.01** (Sight architecture pivot v5 → v6 · Concept Paper v3.1 → v4.0 ratified · MIG-025 build cascade opened · §A.1 first commit landed):
>
> ### Sight architecture pivot — v5 → v6 (Concept Paper v3.1 → v4.0)
>
> After Eisa's *"Sight hasn't achieved this goal"* verdict on v5's seven-mode toggle architecture (2026-05-13), a three-round SME-panel-driven redesign converged on **Option D — Coordinated Views** as the architecture for the next implementation. The full design conversation lives in `docs/sight-redesign-design-concept-v0.1.md` → `v0.2.md` → `v0.3.md` (each preserved on disk per SO #6) plus 9 mock SVGs under `docs/sight-redesign-*.svg`.
>
> **`docs/Constellation-Sight-Concept-Paper-v4.0.md`** is the new binding contract. Supersedes `Constellation-Sight-Concept-Paper-v3.1.md` (kept on disk as historical record). The Concept Paper specifies the implementation as **Sight v6** (the next implementation version after v5). Two version axes kept distinct: Concept Paper v3.1 → v4.0, Implementation v5 → v6.
>
> **Architecture in one paragraph**: anchor dome at center-left (stratum × time × library-shape × typed-link lines) + facet sidebar on left edge (Folder · Library · Stratum · Confidence · Stage · Provenance, all Hearst-Flamenco cross-filterable) + four mini-domes on the right (Confidence-opacity, Stage-hue, Acts-size, Provenance-sectors, all linked-brushed) + 7-register chip in title bar (Aristotelian default + pramāṇa + masādir + Polanyi + Dignāga + Ishrāqī + Mohist sān biǎo). **Default-simple chrome**: anchor dome only on first open; sidebar / chip / mini-domes discoverable via single gestures. No persistent toggle bars.
>
> **The 10 architectural invariants** (Concept Paper v4.0 §11) are now the contract floor: channel orthogonality (no two channels share a Bertin variable), Suwaidi-fidelity (anchor ≥80% of canvas in default state), ≤16 ms cross-filter, CIE Delta-E ≥30 between co-rendered hues, pip foveation threshold, register isolation (chip remaps anchor only; mini-domes stay culturally neutral), register manifest version-control, Folder visibility, gesture chrome (no toggle bars), first-boot tour.
>
> ### MIG-025 opens — first build commit landed
>
> `lab/reports/MIG-025-SIGHT-V6-ARCHITECT.md` and `lab/reports/MIG-025-SIGHT-V6-PLAN.md` set the build strategy: **single MIG-025 with §A/§B/§C/§D internal sub-phases over ~21 weeks**, dual-flag dev mount (B2: `SIGHT_V6_ENABLED` flag, v5 reachable Phase 1–3, deleted Phase 4), progressive backfill via Tauri events with status-bar progress (C3 — Standing-Order resumability), continuous CI perf gate from Phase 1 (F3). The Plan locks 43 build steps, every one of the 10 invariants protected by ≥1 step.
>
> **§A.1 lands the `SIGHT_V6_ENABLED` flag in `src/lib/sight/engine.ts`** as the foundation; flag stays `false` until §A.14 ship gate clears and Eisa tests Sight v6.0. Phase 1 (anchor dome + facet sidebar + Default-simple chrome + first-boot tour + CI perf harness) is ~6 weeks of focused engineering.
>
> ### What v5 will be when v6 ships
>
> `src/lib/sight/v5/` and `src-tauri/src/sight_v5.rs` continue to operate through Phases 1–3 (B2 dual-mount). §D.6 (Phase 4) deletes the v5 module set, drops `sight_v5_layout` table + triggers, removes `SIGHT_V5_ENABLED`. Until then, rollback is a one-line edit: flip `SIGHT_V6_ENABLED=false` in engine.ts and v5 dock returns. v5 cache rows survive untouched.
>
> ### MIG-024 §N close-out — obsoleted by MIG-025
>
> The Sight v5 audit doc (`docs/sight-v5-purpose-achievement-audit.md`) and the v5 mode-concepts deep-dive (`docs/sight-v5-mode-concepts.md`) remain on disk as design-conversation history. The MIG-024 §N close-out (Concept Paper v3.2 fold-in, Pending Jobs allocation, etc.) is no longer relevant — v3.x is superseded.
>
> ### Three Plan inferences locked (2026-05-14)
>
> - **Frontmatter convention** for register sector assignment: new fields `pramana_kind` (4 values) and `masadir_source` (8 values) populated by user; default bucket if absent.
> - **Help → Sight tour** is the re-fire affordance for the first-boot orientation overlay.
> - **`enabledFeatures.constellationSightV6`** is the user-settings flag name (not the v3-era `constellationSightV3` quirk).
>
> ### What hasn't changed
>
> The four-level Knowledge Hierarchy (Universe → Library → Folder → Note), the 7+2 typed-link kinds, the Five Acts, the Living Link Architecture, CECE, and every other foundational subsystem. Sight v6 is a Sight redesign, not a Constellation redesign.

> **What changed in v2.00** (Sight v5 Concept Paper canonical · MIG-022 number-collision resolved · MIG-024 reserved · four-layer instrument framing locked):
>
> ### Sight v5 — canonical design contract landed (`docs/Constellation-Sight-Concept-Paper-v3.1.md`)
>
> The canonical Sight v5 specification is **Concept Paper v3.1** (2026-05-12), Eisa-approved on all 6 validation points after one feedback iteration on v3.0:
>
> 1. **MIG-022 number-collision: confirmed** — gap-analysis-response cascade keeps MIG-022 (already shipped through §A; §B.1–§B.4 Rust foundation also shipped); Warrant Research keeps MIG-023; Sight v5 visual foundation = **MIG-024**.
> 2. **§2 canonical question: confirmed** — *"Is my universe healthy? If not, where does it need to be handled?"* (replaces v3.0's *"How is my Epistemic Content shaped and/or organized?"* — that older phrasing survives as the visual question Layer 1 answers en route to the diagnostic).
> 3. **§4.2 strata mapping with CECE's live IDs: confirmed** — third column added showing `epistemic-states/opinion`, `higher-order-constructs/worldview`, etc., as the live taxonomy parents the visual maps against.
> 4. **§12 four-MIG phasing: confirmed** — MIG-024 (Layer 1 visual) → MIG-025 (Layer 2 diagnostic) → MIG-026 (Layer 3 recommendation, V3-§7.b lands here as a sub-phase) → MIG-027 (Layer 4 coaching) → cleanup MIG.
> 5. **MIG-024 commitment: confirmed.**
> 6. **Mock B1 SVG edit: confirmed and shipped** (this commit).
>
> ### Sight v5 = four-layer analytical instrument
>
> Eisa's correction to v3.0's "visualization-only" framing: *"What I want is to be able to analyze, score, recommend, and/or coach. I want Sight to be an analytical instrument that, after identifying the shape of the user's Cognitive Knowledge and Epistemic Content, will help the user enhance their Cognitive and Epistemic Knowledge. It is like having your own local AI."* v3.1 promotes Sight from passive visualization to a **four-layer instrument**:
>
> | Layer | Job | MIG |
> |---|---|---|
> | **1 — Visual foundation** | Show the user the shape and organization of their Epistemic Content as a stable star chart they can learn and remember. ~5-second comprehension threshold. | MIG-024 |
> | **2 — Diagnostic** | Compute health signals (strata distribution, source diversity, confidence balance, growth trajectory, contested resolution, acts coverage); surface plain-language findings. | MIG-025 |
> | **3 — Recommendation** | Convert findings into specific named actions via Qwen3-1.7B + GBNF grammar. | MIG-026 |
> | **4 — Coaching** | Walk the user through executing recommendations conversationally, with Constellation-aware actions. | MIG-027 |
>
> All inference is local (CECE's existing e5-small ONNX + Qwen3-1.7B GGUF via llama.cpp). Privacy guarantee: zero cloud inference path. The coaching is private the way a private tutor is private.
>
> ### Mock B1 — 7-button toggle bar shipped (this commit)
>
> `docs/Sight-vNext-MockB1-Toggle.svg` updated: 7 buttons (R · L · T · C · S · A · **P**), header reads "MOCK B1 · ONE MODE ACTIVE, SEVEN AVAILABLE", group transform shifted from `translate(525, 86)` to `translate(495, 86)` so the bar stays centered at x=700, caption explains P unlocks via Source Review classification. Original 6-button version preserved at `docs/Sight-vNext-MockB1-Toggle-v1.svg` per SO #6.
>
> ### MIG-022 §B status — Rust foundation complete; UI overlay contradicted by Sight v5
>
> | Sub-phase | Status | Notes |
> |---|---|---|
> | §B.1 Schema (`note_state_history` table + index) | ✓ shipped (`c63a2e3`) | |
> | §B.2 Trigger (`note_state_history_au` AFTER UPDATE with WHEN guard, JSON-diff column) | ✓ shipped (`5c4f1e5`) | |
> | §B.3 First-boot backfill (resumable, idempotent via sentinel) | ✓ shipped (`6ecf8ec`) | |
> | §B.4 Query API IPCs (`cece_get_note_history` + `cece_query_history`) | ✓ shipped (`c3c5c66`) | |
> | §B.5 UI surface (Sight v3 overlay per D-B4.β) | **CONTRADICTED** — Sight v3 is retired; the overlay would patch a surface Sight v5 supersedes. Deferred indefinitely; if a temporal-axis surface is wanted, it should target Sight v5's diagnostic Layer 2 within MIG-025. |
> | §B.6 i18n + help + UM chapter | Deferred — only meaningful if §B.5 ships in some form |
>
> The Rust foundation (§B.1–§B.4) is genuinely useful: any future feature that wants to reason about state-changes-over-time (Sight v5 Layer 2's "growth trajectory" health signal is a natural consumer) reads the `note_state_history` table via the shipped query API. The work is not wasted; only the Sight-v3-overlay UI is.
>
> ### MIG-022 §N — final integration audit can fire now
>
> With §B.4 shipped + §B.5/§B.6 deferred-by-design, MIG-022 is effectively complete (§0 + §D + §E + §A all shipped; §B Rust foundation shipped; §B UI contradicted by canonical Sight v5). §N (3-agent integration audit + close-out) can run as the next discrete piece if Eisa wants it before MIG-024 Architect — or it can ride alongside MIG-024 Architect work.
>
> ### Why v2.00 (not v1.100)
>
> Major version bump: this is the first orientation since the Sight v5 canonical design contract landed AND since the MIG-022 number-collision was resolved. Both are foundational changes that downstream sessions need to recognize at first read. v2.00 signals "the Sight target is now stable; build against it."
>
> ### What's next
>
> - **MIG-024 visual-foundation Architect doc** (next-up; the first /migration Phase 1 against Sight v5 v3.1).
> - **MIG-022 §N** — final integration audit + close-out (can ride parallel with MIG-024 Architect or wait until after).
> - **MIG-023 Warrant Research workstream** (per Eisa's D-C1 commitment; opens after MIG-022 ships).
>
> ### Build
>
> NSIS `Constellation_0.3.4_x64-setup.exe` mtime **2026-05-12 11:06** is still the current shipping build (carries §0+D+E+A+B.1–B.4; the v2.00 commit ships docs only — no Rust/Svelte changes).

> **What changed in v1.99** (MIG-022 §A ships — Boss-Test Gate 3 PASS; first-half cascade complete):
>
> ### §A scoreboard — 8 commits + 3 Boss-test catches inline
>
> | Commit | What |
> |---|---|
> | `c94912d` | §A.1 — Frontmatter parser: `nested-object-list` PropertyType; LIST_KEYS/DATE_KEYS extensions for `domain`/`updated_at` |
> | `a03460e` | §A.2 — `supersedes` typed-link (9th cognitive-vocabulary name; 3 Rust + 2 frontend sites + CSS + CLAUDE.md) |
> | `b0305b0` | §A.3 — Properties panel `ikhtilāf` full custom widget (D-A4.α) |
> | `8a0730b` | §A.4.a — 8 i18n keys × 15 locales + en help topic + en User Manual chapter `## 10c.` |
> | `738a35a` | §A.4.b — 14-locale doc translations (28 files via 3 parallel agents) |
> | `9a6a938` | Ar fix — `linkTypes.supersedes` "يَنسَخ" → "يحلّ محلّ" (Boss correction; "ينسخ" primarily means "copies" in Arabic) |
> | `5e9b5ed` | Ur fix — parallel correction "تبدیل کرتا ہے" → "کی جگہ لیتا ہے" |
> | `1042ea6` | §A.4.d — OutgoingLinksPanel + BacklinksPanel `displayAnnotation()` helper translates known link-type names in the annotation slot |
>
> ### Boss-Test Gate 3 — all 5 stages PASS + re-spotcheck
>
> Stage 0 (binary) + Stage 1 (simple §A fields) + Stage 2 (ikhtilāf widget incl. bonus remove) + Stage 3 (supersedes typed-link) + Stage 4 (i18n switch in es/de/ar) + Stage 4.1 (re-spotcheck after Ar/Ur fixes + §A.4.d annotation translation) + Stage 5 (file-existence verified — in-app Help viewer is PJ-049, separate concern).
>
> ### MIG-022 cumulative scoreboard (§0 + §D + §E + §A)
>
> | Cluster | Status | Commits | PJs closed |
> |---|---|---|---|
> | §0 (F) — Legacy classifier cleanup | ✓ shipped | 1 (`d626ae7`) | (audit F1) |
> | §D — PJ-040 partial UA short-circuit | ✓ shipped | 1 (`c072700`) | **PJ-040** |
> | §E — Full engine-output i18n | ✓ shipped | 7 commits | **PJ-041, 042, 043, 045** |
> | §A — YAML metadata + supersedes + ikhtilāf widget + 15-locale docs | ✓ shipped | 8 commits | (none — schema-add work) |
>
> Plus 7 new PJs filed during cascade for §N close-out: PJ-044 / PJ-046 / PJ-047 / PJ-048 / PJ-049 / PJ-050 (all P2-P3 polish; none block §A ships).
>
> Test count: 92 cece tests at MIG-021v3 close → 97 cece tests now (+3 from §D, +3 from §E.2 minus 2 that consolidated).
>
> ### What's next
>
> - **§B — Temporal axis** (the largest remaining piece). Per the Plan: SQLite `note_state_history` table maintained by triggers on `note_meta` writes, JSON-diff column shape per the cross-check refinement, Sight v3 overlay per Eisa's D-B4.β. 6 sub-phases (§B.1 → §B.6); Boss-Test Gate 4 fires after §B.6. ~2-4 weeks agent time.
> - **§N — Final integration audit + close-out** (3-agent like V3-§11; orientation v2.00 marking "MIG-022 ships"; PJ-044/046/047/048/049/050 confirmed in Pending Jobs).
> - **MIG-023 — Constellation Warrant Research workstream** (per Eisa's D-C1 commitment; opens after MIG-022 ships).
>
> ### Build
>
> NSIS `Constellation_0.3.4_x64-setup.exe` mtime **2026-05-12 11:06** is the §A-complete shipping build (carries §0+D+E+A inclusive of §A.4.d + Ar/Ur supersedes corrections).
>
> **What changed in v1.98** (MIG-022 §0 + §D + §E ship — Boss-Test Gates 1 + 2 PASS):
>
> ### MIG-022 Phase 3 Build cascade — first half closed
>
> The first half of the MIG-022 cascade (§0 cleanup, §D PJ-040 fix, and the entire §E i18n cluster — PJ-041 + PJ-042 + PJ-043 + PJ-045) is shipped. Source Review card is now fully localized in all 15 languages. **Boss-Test Gate 1 + Gate 2 both PASS.**
>
> ### Eight commits, two Boss-test catches, four PJs closed
>
> | Phase | Commit | What | Closes |
> |---|---|---|---|
> | §0 | `d626ae7` | Legacy classifier cleanup (-982 LoC; tier1_embedding/tier1_rules/source_definitions deleted) | audit F1 |
> | §D | `c072700` | Per-axis UA short-circuit refactor (+2 tests; 92→94 cece) | **PJ-040** |
> | §E.1 | `6c1c3ae` | Confidence chip i18n (4 keys × 15 locales + confidenceLabel helper) | **PJ-042** |
> | §E.1.1 | `3a250fb` | catalogerLabel + sources.review (Boss-Test Gate 1 Stage 2 catches) | (inline) |
> | §E.2.a | `894b114` | Reasoning prose Rust + en+ar + frontend (+3 tests; 94→97 cece) | (frame) |
> | §E.2.b | `81fba1a` | Reasoning prose 13-locale backfill | **PJ-041** |
> | §E.3.a/b/c | `67c200f` | Taxonomy en+ar seed (277 nodes) + frontend refactor + sources.evidence backfill | — |
> | §E.3.d | `b9f1ab2` | Taxonomy node labels 13-locale backfill | **PJ-043** |
> | §E.3.f | `05b89a5` | Taxonomy ID resolution + dissenter i18n (Boss-Test Gate 2 Stage 2 catch) | (inline) |
>
> **9 commits total** · ~4,775 translations across 15 locales · 4 PJs closed · 2 Boss-test catches landed inline · cece test count 92 → 97.
>
> ### Boss-Test Gate 1 PASS scoreboard
>
> Stage 0 (binary verify) + Stage 1 (PJ-040 partial UA short-circuit on `الخط العربي`) + Stage 2 (Confidence chip in Arabic UI) + Stage 2.1 (catalogerLabel + sources.review re-test on Spanish UI) all PASS.
>
> Stage 2 surfaced two i18n gaps shipped inline as §E.1.1: `catalogerLabel()` was hardcoded EN/AR (bypassed i18n for 13 locales) and `sources.review.*` keys were missing in 13 locales (V3-§10.D's backfill swept `cece.*` only).
>
> ### Boss-Test Gate 2 PASS scoreboard
>
> Stage 1 (Spanish) + Stage 3 (German) PASS first try. Stage 2 (Arabic) surfaced one structural gap fixed inline as §E.3.f: cataloger reasoning template params (`h_id`, `v_id`, `sources`, `content_type`) carried raw English taxonomy IDs that should resolve to localized labels at render time, plus the regime-pill dissenter rendered the raw cataloger name. New `resolveTaxonomyParams()` helper fixes both. Re-run PASSES on all three locales.
>
> ### What V3-§10 + MIG-022 §E together accomplish
>
> Pre-MIG-022 V3-§10 cascade (orientation v1.94) had honest framing: "the chrome translates correctly across all 14 non-en locales for the Source Review panel + Settings section, but reasoning prose stays English (PJ-041), confidence enum bypasses i18n (PJ-042), taxonomy labels are en+ar only (PJ-043)."
>
> Post-MIG-022 §E: those three structural gaps are CLOSED. A non-en/non-ar user looking at a Source Review card today sees: chrome in their language ✓ + reasoning prose in their language ✓ + confidence enum in their language ✓ + taxonomy labels in their language ✓ + dissenter cataloger name in their language ✓ + composite reasoning summary in their language ✓.
>
> Engine output is now **fully** localized.
>
> ### What MIG-022 §E does NOT do (next-up)
>
> - **§A — YAML metadata extensions** (held_by, domain, function, ikhtilāf, etc., with full ikhtilāf widget per D-A4.α). 4-7 days agent time.
> - **§B — Temporal axis** (note_state_history table + Sight v3 overlay per D-B4.β). 2-4 weeks agent time.
> - **§N — Final audit + close-out** (3-agent integration audit, like V3-§11).
>
> Plus filed-but-not-closed:
> - **PJ-044** (P3 polish) — Right-click "Classify Sources" menu entry missing in NotePane (Eisa workaround: "Classify open note" button in Source Review header). To file in next Pending Jobs bump (lands with §N).
>
> ### Build
>
> NSIS `Constellation_0.3.4_x64-setup.exe` mtime **2026-05-11 20:47** is the §E-complete shipping build (carries §0 + §D + §E.1 + §E.1.1 + §E.2 + §E.3 inclusive of §E.3.f).
>
> **What changed in v1.97** (MIG-022 Architect decisions locked + Phase 2 Plan filed):
>
> ### Eight Architect decisions Eisa-locked
>
> | # | Decision | Pick | vs. recommendation |
> |---|---|---|---|
> | D-A1 | `supersedes`/`contradicts` representation | β — typed-link names | matches |
> | D-A2 | Warrant display in Cluster A | **β — defer all warrant UI** | **opposite** of recommendation; warrant fields are parsed-but-inert in MIG-022 |
> | D-A3 | `taxonomy_version` field | implicit | matches |
> | D-B1 | Temporal storage | SQLite + triggers on `note_meta` | matches |
> | D-B2 | Which fields get history | only epistemic fields | matches |
> | D-B3 | Cluster B in MIG-022 | **IN MIG-022** | **opposite** of recommendation; temporal axis is in scope |
> | D-C1 | Warrant classifier | workstream-deferred + **explicitly committed as next workstream after MIG-022** | matches + adds commitment |
> | D-E1 | PJ-043 storage | (c) frontend i18n keys | matches |
> | D-E2 | Cluster E bundling | one MIG | matches |
> | D-F1 | Cluster F scoping | §0 housekeeping | matches |
>
> ### MIG-022 final scope (after locking)
>
> | Cluster | Status | Effort |
> |---|---|---|
> | §0 (F) — Legacy classifier cleanup | IN | ½ day |
> | §D (PJ-040) — Partial UA short-circuit | IN | ½ day |
> | §E (PJ-041/042/043) — Engine-output i18n | IN | 1-2 weeks |
> | §A — YAML metadata extensions (no warrant UI) | IN | 4-7 days |
> | §B — Temporal axis (note_state_history + triggers + UI) | IN | 2-4 weeks |
> | §N — 3-agent audit + close-out | IN | 1 day |
>
> Total ≈ 5-7 weeks of agent time, **5 Boss-test gates**, 14-16 commits.
>
> ### Cross-check applied (Working Agreement #5)
>
> The temporal-axis storage choice (D-B1) was cross-checked against Datomic / XTDB bitemporal, SQL:2011 SYSTEM_TIME, event sourcing, per-file Git versioning. Verdict: SQLite + triggers is the right pattern for local-first SQLite contexts (used by Datasette via Simon Willison's `sqlite-history-json`). Refinement adopted in the Plan: **JSON-diff column shape** (`changes_json`) instead of `(axis_changed, old_value, new_value)` triples — schema-agnostic when MIG-022 §A.1 adds new epistemic fields. Plus three sharp edges: `WHEN OLD.field IS NOT NEW.field` trigger guard, `DROP TRIGGER` + bulk-insert during backfill, soft-delete decision deferred to §B.1.
>
> ### Plan-level decisions (D-A4 + D-B4) surfaced
>
> Two new decisions Eisa needs before Phase 3 Build cascade opens:
>
> 1. **D-A4** — `ikhtilāf` Properties panel UX: (α) full widget / (β) raw YAML / (γ) read-only display? *Rec: (β) — niche field; raw YAML in MIG-022; polished widget as future PJ.*
> 2. **D-B4** — Temporal-axis UI surface: (α) side-panel / (β) Sight v3 overlay / (γ) search filter? *Rec: (α) — matches Backlinks panel pattern; doesn't couple MIG-022 to Sight v3 stability.*
>
> ### Cluster C trajectory: Constellation Warrant Research workstream
>
> Eisa committed (D-C1): the warrant classifier (Sunni uṣūl as test bed) is the **next workstream after MIG-022 closes**. Will be its own Concept Paper (parallel to Sight v3 Concept Paper). Multi-month research project requiring labeled hadith corpus + uṣūlī expert input + classifier ensemble work. Out of MIG-022 scope; in MIG-022 Plan §8 ("what this Plan does NOT cover").
>
> ### Phase progression
>
> ```
> Phase 1 (Architect)  ✓  04080a2 — territory + 8 decisions
> Phase 2 (Plan)       ✓  this commit — 14-16 phases + 5 gates + cross-check
> Phase 3 (Build)      ⏳  awaiting D-A4 + D-B4 + Plan approval
> Phase 4 (Audit)         after §N close-out
> ```
>
> Phase 3 Build cascade opens once Eisa answers D-A4 + D-B4 + approves the Plan. Then Plan-Approval-Equals-Build-Approval kicks in: I cascade through §0 → §D → §E.1 → §E.2 → §E.3 → §A.1-§A.4 → §B.1-§B.6 → §N autonomously, stopping only at the 5 Boss-test gates and on genuine architectural surprise.
>
> **What changed in v1.96** (MIG-022 Architect filed — awaiting Eisa decisions on §6):
>
> ### MIG-022 cascade opens — Architect doc landed
>
> The MIG-022 Architect (`lab/reports/MIG-022-ARCHITECT.md`) maps the territory for the response to:
> - **The gap analysis** (`docs/epistemic-content-gap-analysis.md`) — three structural gaps (temporal/dynamic, justification/warrant, contestation/agent) + five minor extensions
> - **PJ-040** — UA partial-frontmatter discards other catalogers' votes on unfilled axes
> - **PJ-041 / PJ-042 / PJ-043** — engine-output i18n gaps
> - **V3-§11 audit F1** — legacy classifier dead-code cleanup
>
> Six work clusters identified, sized, and risk-assessed:
>
> | Cluster | What | Effort | Risk |
> |---|---|---|---|
> | **A** — §6.1 YAML metadata extensions (held_by, warrant, domain, function, ikhtilāf, supersedes, contradicts, etc.) | 3-5 days | Low-medium |
> | **B** — §6.3 Temporal axis via history layer | 2-4 weeks | Medium-high |
> | **C** — §6.2 Warrant classifier (Sunni uṣūl as test bed) | Multi-month research | High |
> | **D** — PJ-040 fix (partial UA short-circuit) | ½ day | Low |
> | **E** — PJ-041/042/043 i18n bundle | 1-2 weeks | Low-medium |
> | **F** — Legacy classifier cleanup | ½ day | Low |
>
> ### Recommended MIG-022 scope
>
> Architect's §5 recommends MIG-022 = **§0 Cleanup (F) + §A YAML metadata + §D UA fix + §E i18n** = ~3 weeks of agent time, two Boss-test gates. Cluster B (temporal axis) deferred to **MIG-023** as its own Architect cycle. Cluster C (warrant classifier) deferred to a **Constellation Warrant Research** workstream with its own Concept Paper.
>
> ### Eight decisions for Eisa to lock
>
> Phase 2 Plan unblocks once Eisa answers §6 of the Architect:
>
> 1. **D-A1** — `supersedes`/`contradicts` as YAML scalars or typed-link names? *(rec: typed-link names — graph-native consistency)*
> 2. **D-A2** — Warrant *display* in Cluster A or all warrant UI deferred? *(rec: include display now)*
> 3. **D-A3** — Ship `taxonomy_version` field now or stay implicit? *(rec: implicit until breaking change forces it)*
> 4. **D-B3** — Cluster B in MIG-022 or split to MIG-023? *(rec: split — temporal axis deserves own Architect)*
> 5. **D-C1** — Cluster C in MIG-022 or workstream-deferred? *(rec: workstream-deferred — multi-month research)*
> 6. **D-E1** — PJ-043 storage: struct fields / per-locale JSON / frontend i18n keys? *(rec: frontend i18n keys)*
> 7. **D-E2** — Cluster E as one bundled MIG or three separate? *(rec: one MIG with three sub-phases)*
> 8. **D-F1** — Cluster F as MIG-022 §0 or its own mini-MIG? *(rec: §0 housekeeping)*
>
> ### Honest framing
>
> The Architect doc explicitly does NOT pre-decide scope. Recommendations are defensible defaults; every decision is Eisa's. The work that ships with MIG-022 is the work Eisa picks. Cluster B (MIG-023) and Cluster C (Warrant Research workstream) are deferred *because they deserve their own depth*, not because they're being avoided.
>
> Phase 2 (Plan) opens once §6 is locked. Phase 3 (Build) cascades through the Plan. Phase 4 (Audit) runs the same three-agent integration check that just closed V3-§11.
>
> **What changed in v1.95** (V3-§11 final integration audit PASS — **MIG-021v3 ships**):
>
> ### MIG-021v3 ships
>
> The Constellation Epistemic Content Engine (CECE) is **integration-clean and shipping**. V3-§11 ran a three-agent audit (invariants / drift / migration-path) over the entire MIG-021v3 cascade — V3-§1 → V3-§10, every .r/.X follow-up, every Gate. Verdict: **9 invariants HOLD, 0 VIOLATED, 1 AT RISK** — the AT RISK fix landed in this commit (4 lines to `src/lib/libraries/store.ts`). Migration risk: **LOW** across all 7 scenarios. Drift findings: 1 P1 (legacy classifier dead code — candidate cleanup MIG) + 3 P3s. **92 cece tests pass.**
>
> Full audit report: `lab/reports/MIG-021v3-V3-§11-FINAL-INTEGRATION-AUDIT.md`.
>
> ### The AT RISK fix (this commit)
>
> The V3-§10.A Settings flag persistence had a latent gap: `cece` block declared in `AppSettings` interface but missing from both `DEFAULT_SETTINGS` and the `loadSettings` deep-merge. All five current consumers (`+layout.svelte:2088`, `NoteEditor.svelte:182`, `SourceReviewPanel.svelte:256`, `SettingsModal.svelte:1867,1882`) use defensive `?.` + `??` chains, so today's behavior is correct — but a future consumer reading `$appSettings.cece.someFlag` without optional-chain would throw, and any future cece sub-key added by a release would silently overwrite a user's other cece settings on load if their saved settings.json had a non-empty cece block.
>
> Fix: 4 lines added to `src/lib/libraries/store.ts` — `cece: { reasoningTrailVisibility: 'on_disagreement', backgroundScan: 'off' }` in `DEFAULT_SETTINGS`; `cece: { ...DEFAULT_SETTINGS.cece, ...((parsed.cece as Record<string, unknown>) || {}) }` in `loadSettings` spread. No behavior change for current users; contract-strengthening only.
>
> ### MIG-021v3 cumulative scoreboard
>
> | Phase | What | Outcome |
> |---|---|---|
> | V3-§1 → V3-§7 (engine spine) | 6-cataloger architecture, GBNF, synthesis, IPC layer, reasoning prompt | engine internals shipped |
> | V3-§8 (orchestrator + UX polish) | CECE wired into UI; 6 audit fixes; queue composition filter | Gate 1 PASS |
> | V3-§9 (vertical-axis activation) | Vertical lexicon + structural detectors + reliability wiring + GBNF axis-aware | Gate 2 PASS |
> | V3-§10 (user-facing surfaces) | Settings + 15-locale i18n + EN+14 help + EN+14 User Manual | Gate 3 PASS |
> | V3-§11 (final integration audit) | This commit — AT RISK fix + audit report + orientation v1.95 | **MIG-021v3 ships** |
>
> **Tests:** 92 cece tests, all PASS. **Gates:** 1 PASS · 2 PASS · 3 PASS · 4 (V3-§11) PASS.
>
> **PJs filed during cascade:** PJ-040 (UA partial-frontmatter), PJ-041 (cataloger reasoning prose i18n), PJ-042 (confidence enum i18n), PJ-043 (taxonomy node labels en+ar only).
>
> ### Build
>
> NSIS `Constellation_0.3.4_x64-setup.exe` from V3-§10.D.2 (mtime 2026-05-11 13:40:17) is the shipping build. The V3-§11 store.ts fix is contract-strengthening only — no UI/UX-visible difference for users on the existing build; the next NSIS rebuild folds the fix in for safety against future consumer code.
>
> ### What's next: MIG-022
>
> The MIG-022 cascade is the **scope-expansion response** to:
> - **The gap analysis** (`docs/epistemic-content-gap-analysis.md`) — temporal/dynamic axis (§6.3), justification/warrant axis (§6.2), contestation/agent axis, plus the five minor extensions (§6.1).
> - **PJ-040** — UA-short-circuit on partial frontmatter discards other catalogers' votes on unfilled axes.
> - **PJ-041 / PJ-042 / PJ-043** — engine-output i18n gaps that V3-§10 could not address structurally (cataloger reasoning prose hardcoded in Rust, confidence enum bypassing $t(), taxonomy labels en+ar only).
> - **F1 from this audit** — legacy classifier dead-code cleanup (housekeeping §0).
>
> Architect doc + Plan to follow. Boss decides which pieces to pursue when. The CECE engine is the floor; MIG-022 raises the ceiling.
>
> **What changed in v1.94** (Gate 3 PASS close-out + V3-§10.A.1 + V3-§10.D.2 fixes + 3 new PJs filed):
>
> ### Gate 3 closes — V3-§10 user-facing surfaces production-ready
>
> All 7 Gate 3 stages PASS:
>
> | Stage | Verifies | Result |
> |---|---|---|
> | 1 | Settings UI section renders correctly | ✅ PASS |
> | 2 | Trail visibility setting changes behavior across all 3 modes | ✅ PASS |
> | 3 | Background scan doesn't fire on keystroke + queue updates after debounced save | ✅ PASS (after V3-§10.A.1 fix) |
> | 4 | Per-Library calibration view shows real data | ✅ PASS (visible in Stage 1 screenshot — calibration view auto-expanded showing actual V3-§9.C.2 reliability data) |
> | 5 | i18n in 13 other locales | ✅ PASS (after V3-§10.D.2 fix) |
> | 6 | Help topic discoverability in en + at least 1 other locale | ✅ PASS |
> | 7 | User Manual chapter present in en + at least 1 other locale | ✅ PASS |
>
> Stage 5 specifically demonstrated **chrome translates correctly across all 14 non-English locales** for both the Settings section + Source Review panel. The screenshots showed Arabic + German + Chinese all rendering CECE labels in their native scripts.
>
> ### Two Gate-3 catches landed inline
>
> **V3-§10.A.1** (`4ede8ef`) — Stage 3 caught that the on-save background-scan IPC fired correctly but never notified the Source Review panel to refresh. The fix: dispatch the same `constellation:classify-and-show` window event the right-click context menu uses, so the panel's existing `handleClassifyAndShow` handles the IPC + queue prepend + flash highlight in one path. Same lesson as V3-§9.C.2 — when two paths share a dependency on the same flow, centralize the dispatch.
>
> **V3-§10.D.2** (`54276c3`) — Stage 5 caught that the "Sources & content type classifier" section header + 4 setting strings (added by MIG-021v2 §1F' before V3) stayed in English in the 13 non-en/non-ar locales. V3-§10.D's backfill swept `cece.*` keys but missed `settings.classifier.*`. The fix: backfill those 5 keys via Python batch script (translations inline-supplied per language family).
>
> ### Three deeper i18n gaps filed for MIG-022 (PJ-041 / PJ-042 / PJ-043)
>
> Stage 5 surfaced gaps that V3-§10.D's i18n backfill could not have addressed because they're structural — the strings don't go through `$t()` to begin with. Filed in Pending Jobs v1.9:
>
> - **PJ-041 — cataloger reasoning prose hardcoded English in Rust.** Each cataloger generates its `reasoning: String` field via `format!()` with hardcoded English templates ("Structural patterns matched: vertical → ... weight 0.75", etc.). Stored verbatim in `composite_json`, rendered raw by the frontend. Visible on every Source Review card in any non-English locale. Fix is structural: refactor to emit `(template_key, params)` tuples + `$t()` at display time. ~3-5 hrs + ~450 translations.
>
> - **PJ-042 — `self_reported_confidence` enum bypasses i18n.** The `[high]` / `[medium]` / `[low]` labels next to each cataloger's name in the trail render the raw enum string. ~30 min + 60 translations. Smallest of the three.
>
> - **PJ-043 — taxonomy node labels en+ar only.** The vertical (225 nodes) + horizontal (~30 nodes) taxonomy data has `en` + `ar` fields only. Non-en/non-ar users see English/Arabic labels in the Source Review SOURCES + CONTENT TYPE lists + Sibling Disambiguation chips. ~3300 translations across 14 locales. Largest by translation volume.
>
> All three belong in the **MIG-022 cascade** (response to the gap analysis from `docs/epistemic-content-gap-analysis.md`) — they share the theme that the engine output structure has assumptions (prose language, enum-as-string, taxonomy bilingualism) that don't scale to the multi-locale model the user actually wants.
>
> ### What V3-§10 does NOT translate (honest framing)
>
> V3-§10's i18n scope was **"strings the frontend sends through `$t()`"**. That covered:
> - All Source Review panel chrome (count strip, filter chips, dot cluster tooltips, trail toggle, regime pills, rule chips, disambig form labels, queue actions)
> - All Settings UI labels in the new CECE section
> - The new help topic + User Manual chapter (canonical en + AI-translated to 14 other locales with disclaimer headers)
>
> V3-§10 did NOT translate (these are PJ-041/042/043):
> - Per-cataloger reasoning prose generated in Rust at classification time
> - The `[high]` / `[medium]` / `[low]` confidence enum
> - The Source × Content Type taxonomy node labels for non-en/non-ar locales
>
> A non-en/non-ar user looking at a Source Review card today sees: chrome in their language ✓ + reasoning prose in English ✗ + confidence enum in English ✗ + taxonomy labels in English/Arabic (whichever the wrapper falls back to) ✗. Honest signal: V3-§10 is the floor, not the ceiling. MIG-022 raises the ceiling.
>
> ### V3-§10 cumulative scoreboard
>
> | Phase | Commit | What |
> |---|---|---|
> | A | `d44b115` | Settings UI + IPC + appSettings.cece |
> | B | `0054981` | en + ar i18n for cece.settings.* |
> | C | `34a96a9` | EN help topic + EN User Manual chapter |
> | D | `259c333` | 13-locale i18n backfill for cece.* keys |
> | E | `7d6e1a0` | 14-locale help topic translations |
> | F | `50a67b0` | 14-locale User Manual chapter translations |
> | G | `a4438ac` | NSIS + orientation v1.93 + Gate 3 ready |
> | A.1 | `4ede8ef` | On-save IPC dispatches classify event (Boss-test catch) |
> | D.2 | `54276c3` | settings.classifier i18n backfill (Boss-test catch) |
>
> 9 commits total + this close-out commit. Two Boss-test catches fixed inline; three deeper i18n gaps filed for MIG-022.
>
> ### Build
>
> NSIS `Constellation_0.3.4_x64-setup.exe` is at mtime 2026-05-11 13:40:17 (the V3-§10.D.2 build).
>
> ### What's next
>
> - **V3-§11** — final integration audit + close-out of MIG-021v3 entire. Cross-cutting check that V3-§1 → V3-§10 + all the .r/.X follow-ups + all 4 Gates compose correctly. Architect doc + Plan + audit + close-out commit. The CECE engine + UI + Settings + i18n + docs are all live; V3-§11 is the formal MIG close-out.
> - **MIG-022** — Architect doc responding to the gap analysis (`docs/epistemic-content-gap-analysis.md`) + the three new PJs (PJ-041, PJ-042, PJ-043). Maps the structural gaps (temporal/dynamic axis, justification/warrant axis, contestation/agent axis) + the engine-output i18n gaps to specific MIG candidates with effort + risk estimates. Boss decides which pieces to pursue when.
>
> **What changed in v1.93** (V3-§10 — User-Facing Surfaces — Option C cascade A→G shipped; Gate 3 Boss-test ready):
>
> ### V3-§10 done
>
> Boss picked Option C (Settings + en/ar i18n + EN docs + 13-locale i18n backfill + 14-locale help topic + 14-locale User Manual chapter). Cascade landed in 7 commits.
>
> | Round | Commit | What |
> |---|---|---|
> | V3-§10.A — Settings UI + IPC + appSettings.cece | `d44b115` | New "Constellation Epistemic Content Engine" Settings section under Intelligence; 4 setting rows (Reasoning model status, trail visibility radio, background scan radio, per-Library calibration view); new `cece_get_reliability_for_active_library` + `cece_get_active_library_root` IPCs; new `<PerLibraryCalibrationView>` Svelte component; appSettings.cece sub-object; reasoningTrailVisibility wired into SourceReviewPanel's `isTrailOpen()`; backgroundScan wired into NoteEditor's debounced save + +layout.svelte boot |
> | V3-§10.B — en + ar i18n | `0054981` | 28 new `cece.settings.*` keys in en.json + ar.json |
> | V3-§10.C — EN help topic + EN User Manual chapter | `34a96a9` | New `docs/help.uConstellation.World/Source Review/Source Review.md` (~3500 words, 13 sections) + new `## 10b. Source Review (CECE)` chapter in `docs/User Manual.md` + cross-reference added to Cognitive Engine help topic |
> | V3-§10.D — 13-locale i18n backfill | `259c333` | All 13 non-en/non-ar locales got the full `cece` block (~90 keys each) translated. Done via 5 parallel agents per language family. Each block has a `_translation_note` disclaimer in the target language. JSON parse-validity verified for all 13 |
> | V3-§10.E — 14-locale help topic translations | `7e8b66b` (combined commit shown after E commit) | New `docs/help.{locale}/Source Review/Source Review.md` for all 14 non-English locales (ar, de, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh). Each starts with translated disclaimer header. Done via 5 parallel agents |
> | V3-§10.F — 14-locale User Manual chapter translations | `50a67b0` | Each translated User Manual got the new "Source Review (CECE)" chapter inserted at the appropriate position (10b for Latin/CJK, 10ب for fa/ur, 10ב for he, 11ب for ar where ch10 is Second Screen). TOC entries added. Done via 2 parallel agents |
> | V3-§10.G — NSIS + orientation v1.93 + Gate 3 ready | this commit | Build artifact + this orientation bump |
>
> ### Translation honesty
>
> Per Option C's risk register: every translated file carries an AI-translation disclaimer in the target language. Agents flagged specific terms worth native-speaker review:
> - The 11 Source axis values (perception / inference / testimony / mass-transmission / qiyās / arthāpatti / anupalabdhi / memory / fitrah / kashf / al-wahy) carry deep tradition-specific weight; they translate cleanly into ar (matches `ar.json::sources.label` exactly) but renderings for fa/ur/he/ja/ko/zh/hi are plain-language paraphrases. **Highest-priority follow-up for native review.**
> - "Sibling Disambiguation" kept in Latin form alongside the locale translation in most cases — UI feature name worth recognizing across docs/UI.
> - "Living Links" preserved in Latin form across all locales (Constellation product term, like CECE).
> - Confidence regime names (Unanimous / Strong majority / Split) translated consistently per locale's i18n cece block.
>
> ### What's user-visible after V3-§10
>
> Anyone opening Settings → Intelligence on the new build sees:
> - A new "Constellation Epistemic Content Engine" section header with intro paragraph
> - Reasoning Cataloger model status (currently "Not downloaded — deferred to V3-§7.b")
> - Reasoning trail visibility dropdown (Always / On disagreement / Always hide)
> - Background classification dropdown (Off / On note save / On app start)
> - Per-Library calibration collapsible — opens to a read-only table showing per-cataloger per-axis accuracy from the V3-§9.C.2 reliability data, with "(uniform)" labels for catalogers below the 20-correction threshold and an empty-state when the file doesn't exist yet.
>
> Anyone switching the app's interface language to one of the 13 other locales (de, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh) sees the Source Review panel + Settings section in their language.
>
> Anyone opening the help system can read the new "Source Review" topic in 15 languages. Same for the User Manual chapter.
>
> ### Gate 3 Boss-test ready
>
> Per Plan §7, Gate 3 has 7 stages exercising:
>
> 0. Build installation
> 1. Settings UI section renders correctly
> 2. Trail visibility setting changes behavior across all 3 modes
> 3. Background scan doesn't fire on keystroke (perf preserved)
> 4. Per-Library calibration view shows real data
> 5. i18n in 13 other locales (switch to German + 1-2 others)
> 6. Help topic discoverability in en + at least 1 other locale
> 7. User Manual chapter present in en + at least 1 other locale
>
> Boss-test instructions per the Plan §7 will land separately.
>
> ### Build
>
> NSIS `Constellation_0.3.4_x64-setup.exe` rebuilt mtime 2026-05-11 11:49:06 (123 MB). Eisa runs Gate 3 against this build.
>
> ### What V3-§10 does NOT do (gap analysis acknowledgment, 2026-05-11)
>
> Eisa surfaced an analytical addendum mid-Gate-3 — **"Gap Analysis of the Universal Epistemic Content Model"** (`docs/epistemic-content-gap-analysis.md`). The piece identifies three structural gaps and five minor extensions that the two-axis (Source × Content Type) framework cannot represent. Boss directive: close Gate 3 on V3-§10 as scoped; the gap analysis is input for a separate MIG-022 architectural workstream, not a mid-cascade scope expansion.
>
> The three structural gaps the document identifies:
>
> - **Temporal / dynamic axis** — CECE today is static. A note's classification is what it IS, not what it WAS or how it got there. Recommended fix per §6.3 of the analysis: a git-like history layer that tracks change events without polluting the taxonomy. Constellation's existing Living Links already carry partial temporal data (weight, traversal count, last-traversed) for the relational dimension; the per-note epistemic-state-over-time dimension is the new work.
> - **Justification / warrant axis** — *Source* tells you how you came to know; *warrant* tells you why you're entitled to believe. Two notes with identical "testimony" source can have radically different warrant (mutawātir vs anonymous forum post). The Sunni uṣūlī tradition has a thousand years of vocabulary for this (mutawātir / mashhūr / āḥād / ṣaḥīḥ / ḥasan / ḍaʿīf / mawḍūʿ + their conditions). CECE collapses warrant into source — **the most consequential structural gap** the analysis names. Recommended response: defer to MIG-022.v2 as a deliberate scholarly project with uṣūl al-fiqh as the test bed.
> - **Contestation / agent axis** — A note can record the user's stance, a particular scholar's stance, a school's stance, or *ikhtilāf* (structured scholarly disagreement). The current model assumes the user's stance. For serious scholarly use of CECE this is a real limitation.
>
> Five minor extensions noted (domain/subject matter, function/actionability, confidence as probability, linguistic/civilizational provenance, logical relations between notes) — most can land as optional YAML metadata per §6.1 of the analysis without restructuring the engine.
>
> Concrete acknowledgment: CECE v1.0 (V3-§1 → V3-§10) is feature-complete for the two-axis model as designed. The gap analysis identifies where the model itself is bounded. MIG-022 — a focused Architect doc responding to §6.1 (YAML metadata extensions), §6.2 (warrant classifier as a separate workstream), §6.3 (temporal axis via versioning) — lands after Gate 3 PASS + V3-§11 close-out.
>
> ### What's next
>
> If Gate 3 PASSes: **V3-§11** (final integration audit + close-out of MIG-021v3 entire). The CECE engine + UI + Settings + i18n + docs are all live; V3-§11 is the cross-cutting audit + the formal MIG close-out commit. Then **MIG-022** — Architect doc responding to the gap analysis, mapping §6.1/§6.2/§6.3 to specific MIG candidates with effort + risk estimates. Boss decides which pieces to pursue when.
>
> **What changed in v1.92** (Gate 2 PASS close-out + PJ-040 filed; V3-§10 plan next):
>
> ### Gate 2 closes — vertical axis production-ready
>
> All 6 Gate 2 stages PASS:
>
> | Stage | Verifies | Result |
> |---|---|---|
> | 1 | Lexicon entries fire on diverse vertical content | ✅ PASS |
> | 2.1–2.4 | 4 structural detectors (Eng/Ar definition, Eng/Ar worldview, figure, code-density) | ✅ PASS |
> | 3 (re-test) | Per-axis reliability JSON updates correctly on dual-axis Accept | ✅ PASS (after V3-§9.C.2 fix) |
> | 4 | Reasoning Cataloger silent (LLM not wired) | ✅ PASS |
> | 5 | End-to-end vertical diversity across 5 content types | ✅ PASS |
> | 6 | No horizontal regression on UA short-circuit | ✅ PASS (with PJ-040 filed) |
>
> Stage 5 specifically demonstrated **5 distinct vertical primaries across 5 test notes** — concept / worldview (×2 EN+AR) / electromagnetic-signal / sign — proving the vertical axis is no longer collapsing all notes to one or two values. V3-§9.A's lexicon expansion + V3-§9.B's structural detectors did their job.
>
> Stage 3 (re-test) confirmed V3-§9.C.2's dual-axis reliability fix works on real data: the freshly-created `cataloger_reliability.json` now has entries for BOTH `horizontal` AND `vertical` sub-objects per cataloger, with multiple catalogers tracked (linguistic + user_authority + semantic + structural — proves at least one Accept involved a card with frontmatter pre-populated triggering UA on both axes).
>
> ### V3-§9 cumulative scoreboard
>
> | Round | Commit | Tests added | Cumulative cece tests |
> |---|---|---|---|
> | V3-§9.A — Vertical lexicon | `4e0981a` | 4 | 71 |
> | V3-§9.B — Structural detectors | `d9dfa60` | 7 | 78 |
> | V3-§9.C — Reliability wiring | `ec5527e` | 7 | 85 |
> | V3-§9.D — Reasoning interface | `b18a3ee` | 5 | 90 |
> | V3-§9.E — NSIS + close-out | `bf07ae1` | 0 | 90 |
> | V3-§9.C.2 — Dual-axis reliability fix | `75807a3` | 2 | **92** |
>
> +25 tests during V3-§9 (67 → 92). Two Boss-test catches (V3-§9.A's ال-prefix gap, V3-§9.C.2's dual-axis silent gap) found mid-cascade and fixed inline. Net wall-clock ~3hrs of agent time + Eisa's two Boss-test sessions.
>
> ### PJ-040 filed (Pending Jobs v1.7 → v1.8)
>
> Boss-test Stage 6 surfaced an architectural observation worth fixing in a future PR: **UA-short-circuit on partial frontmatter discards the other catalogers' votes on the unfilled axis**.
>
> When `الخط العربي` was re-classified, its frontmatter has `sources: testimony/authoritative` (UA-set) but no `content_type:`. The current `user_authority_short_circuit` produces an `AxisDecision` for BOTH axes hardcoded to Unanimous, taking primary from UA's per-axis vote. For vertical (which UA didn't voice on), `primary: None` → no vertical suggestion entry → the Source Review card renders without a CONTENT TYPE section, even though Linguistic + Structural + Semantic all voted high-confidence on vertical. Their work is silently discarded.
>
> The fix: refactor `user_authority_short_circuit` to short-circuit ONLY the axes UA voiced on; for unfilled axes, fall through to the normal `vote_on_axis` weighted-vote path. Behavior has been the same since V3-§1 — only became visible after V3-§9.A populated meaningful vertical lexicon coverage. Full PJ entry: `docs/Constellation Pending Jobs v1.8.md` PJ-040.
>
> Not blocking V3-§10. Could be a focused mini-MIG between V3-§10 phases or after Gate 3.
>
> ### What's next: V3-§10 — Settings + i18n + Help docs + User Manual
>
> Per the original V3 plan, V3-§10 is the user-facing surfaces layer:
>
> 1. **Settings UI** — new "Constellation Epistemic Content Engine" section under Intelligence: Reasoning model status, reasoning trail visibility toggle, per-Library calibration view (read-only — display the per-cataloger per-axis accuracy from the reliability JSON), background scan toggle.
> 2. **i18n full pass** — ~50 strings of CECE chrome across all 15 locales (currently en + ar populated; others fall back to inline EN defaults).
> 3. **Help docs** — new `docs/help.uConstellation.World/CECE/CECE.md` topic.
> 4. **User Manual** — new chapter in `docs/User Manual.md` + 14 translations.
> 5. Honest accuracy framing per Architect §10 invariant 10.
>
> V3-§10 is mostly UX/content work, not engine internals. Smaller risk surface than V3-§9 — but more breadth (15 locale files, 14 translations, multiple help topics). Architect doc + Plan to follow.
>
> **What changed in v1.91** (V3-§9.C.2 — dual-axis reliability gap caught mid-Gate-2 Stage 3):
>
> ### What Eisa caught
>
> During Gate 2 Stage 3, Eisa opened the freshly-created `cataloger_reliability.json` file expecting four entries (one Linguistic+vertical correct, one Structural+vertical correct, one Semantic+vertical wrong, one Semantic+horizontal correct). The file had only ONE entry: `semantic.horizontal.correct = 1`. The vertical axis hadn't been touched at all.
>
> ### Root cause
>
> The Accept flow makes TWO IPC calls in sequence:
>
> 1. `sources_set_manual` — reads `composite_json` from suggestion row, writes horizontal frontmatter, **clears the suggestion row**, updates horizontal reliability ✓
> 2. `content_type_set_manual` — tries to read `composite_json`, **finds row already cleared**, `prior_composite = None`, reliability update is silently skipped ✗
>
> The bug was at the IPC orchestration layer, not within either IPC. Same pattern affected `cece_resolve_disambiguation`'s auto-write flow — the second axis write also found the row gone.
>
> ### Fix
>
> Refactored reliability updates OUT of the per-axis IPCs into a new dedicated dual-axis IPC `cece_record_correction_for_card(note_path, composite_json, horizontal_pick, vertical_pick)`. The caller snapshots `composite_json` once before the writes and passes it explicitly to the new IPC after the writes complete.
>
> Wired into:
> - **Frontend `acceptSuggestion`** (Source Review Accept button + Edit-mode commit): snapshots `record.composite_json` BEFORE the two writes, calls the new IPC after.
> - **`cece_resolve_disambiguation`** (Sibling Disambig chip pick): snapshots composite at the same time it reads `extract_other_axis_settled`, calls the new IPC after both axis writes.
>
> Single-axis callers (PropertyEditor manual edits etc.) don't have a composite snapshot to pass, so they don't get reliability updates — that's correct behavior since manual edits aren't keyed to a specific suggestion row.
>
> ### Why this matters beyond the test
>
> Without V3-§9.C.2, V3-§9.C's whole value proposition was undermined. Vertical-axis reliability would silently stay uniform forever; the per-axis weighting the synthesis layer was supposed to start using would only have horizontal data to work with. Boss-tests would have reported "horizontal works, vertical doesn't" until someone debugged it. Eisa caught it on the first inspection of the JSON file — exactly the kind of "verify the wired-up state, don't trust the commit message" check that has caught two regressions this week now.
>
> ### Test coverage
>
> 92 cece tests pass (was 90 in V3-§9.E; +2 from V3-§9.C.2):
> - `v3_p9c2_dual_axis_accept_updates_both_axes` — verifies both axes' counters bump from a single composite snapshot
> - `v3_p9c2_horizontal_only_pick_updates_horizontal_only` — verifies empty vertical_pick is a no-op for vertical
>
> The existing 7 V3-§9.C tests still pass (they test `update_reliability_from_correction` directly, which is unchanged — only its callers moved).
>
> ### Build
>
> NSIS `Constellation_0.3.4_x64-setup.exe` rebuilt mtime 2026-05-11 09:38:53 (123 MB). Eisa to re-run Gate 2 Stage 3 (delete the prior `cataloger_reliability.json` so the test starts clean, then Accept a fresh card and verify all 4 expected entries land).
>
> ### Lesson
>
> When two IPCs operate on the same row in sequence, **whichever one needs to read state from that row must snapshot it before the first IPC's side-effects fire**. The cleanest pattern is to lift the read into the orchestration layer (the caller making both IPC calls) and pass the snapshot explicitly to a dedicated handler. Don't rely on each per-axis IPC to re-derive state from a row that may have been mutated by its sibling — that's the silent-failure mode V3-§9.C.2 fixed.
>
> Same lesson as V3-§8.r7 Issue #1 in spirit: when two filters using "the same logic" can disagree on the same data, route them through one helper. Here: when two IPCs need the same row's state, snapshot it once at the orchestration layer.
>
> **What changed in v1.90** (V3-§9 vertical-axis activation — full Option C cascade A→E shipped; Gate 2 Boss-test ready):
>
> ### What V3-§9 set out to do
>
> Per the Architect doc audit, the vertical axis was **mostly already live** through the V3-§8 cascade — catalogers produce vertical output, synthesis computes per-axis regimes, UI renders both axes side-by-side, disambig and auto-write work for vertical, the r8 filter has a vertical-Split bucket. The original V3-§9 plan from `MIG-021v3-EPISTEMIC-CONTENT-ENGINE-PLAN.md` §10 was written before the V3-§8 cascade did most of its work implicitly.
>
> The actual gaps for "vertical axis production-ready" were narrower:
> 1. Lexicon thinness — 7 vertical entries (vs 17 horizontal), all in `epistemic-states`. Other 4 branches had ZERO lexicon coverage.
> 2. Structural detector asymmetry — only 6 vertical regex rules vs 11 horizontal.
> 3. Per-axis reliability dormant — schema was per-axis-aware but `record_correction` was never called from anywhere.
> 4. Reasoning Cataloger interface not yet locked in for V3-§7.b two-pass support.
>
> Boss picked **Option C — Full**. The cascade landed in 4 commits (A → D) plus this Phase E close-out.
>
> ### V3-§9.A — Vertical lexicon expansion (`4e0981a`)
>
> 12 new entries added to `sources_lexicon.json::vertical`, bringing total from 7 → 19 across all 5 branches:
>
> - `epistemic-states/{knowledge/by-content/propositional, knowledge/by-content/acquaintance, illusion}` (3 new)
> - `semantic-contents/{proposition, concept, fact, idea/constructed, information}` (5 new — branch was empty)
> - `higher-order-constructs/{theory, worldview, doctrine}` (3 new — branch was empty)
> - `sensory-inputs/signal` (1 new — branch was empty)
> - `symbolic-entities/sign` (1 new — branch was empty)
>
> Two ID typos caught and corrected during Plan §1's pre-commit grep validation: `semantic-contents/theory` doesn't exist (theory lives under `higher-order-constructs/theory`); `epistemic-states/knowledge/by-acquaintance` is actually `epistemic-states/knowledge/by-content/acquaintance`. Surface-token match is exact substring (not lemma-aware), so all Arabic nouns frequently taking `ال` definite article have both bare and ال-prefixed variants in the tokens list — caught when `الرؤية الكونية` test failed on lexicon's bare `رؤية كونية` until ال-variants added explicitly.
>
> ### V3-§9.B — Structural vertical detectors (`d9dfa60`)
>
> 5 new vertical regex rules + 1 line-pass density rule added to `structural.rs::vertical_rules()`:
>
> - English definition marker → `semantic-contents/concept` (0.80) — "is defined as", "we define", "by definition"
> - Arabic definition marker → `semantic-contents/concept` (0.80) — تُعرَّف, نُعرِّف, التعريف
> - English worldview marker → `higher-order-constructs/worldview` (0.75) — worldview, paradigm, framework
> - Arabic worldview marker → `higher-order-constructs/worldview` (0.75) — رؤية كونية, الرؤية الكونية
> - Figure/diagram reference → `sensory-inputs/signal/physical/electromagnetic` (0.70) — figure N, fig. N
> - Code-block-density (line-pass) → `symbolic-entities/sign` (0.65–0.80 density-driven) — fires at 6+ fence lines (≈3 fenced blocks)
>
> The code-block-density rule lives in a separate `count_code_block_fences()` helper + inline check in `classify()`, not a regex tuple, because threshold gating + density-driven weight don't map to the `(regex, target, weight)` shape used by `vertical_rules()`.
>
> ### V3-§9.C — Wire reliability updates into correction flows (`ec5527e`)
>
> The Architect doc framed Phase C as "per-axis reliability schema migration v1→v2." Re-auditing `reliability.rs` source before implementing revealed the per-axis schema was **already in place** (HashMap<cataloger, HashMap<axis, AccuracyHistogram>>; `weight_for(profile, cataloger, axis)` already takes axis; `synthesis::vote_on_axis` already passes the correct axis). No schema migration was needed.
>
> Real gap caught: `record_correction()` was defined but **called from nowhere**. Reliability tracking was built end-to-end but dormant — accuracy ratios stayed at uniform 1.0 forever because no correction event ever updated the histogram.
>
> Phase C re-scoped to wire the dormant machinery: new `update_reliability_from_correction(library_root, composite_json, axis, user_pick)` helper iterates per-cataloger trails in the composite JSON, finds each voicing cataloger's primary on the axis, marks "correct" if matched user's pick and "wrong" otherwise. Silent catalogers get NO counter bump (silence is neither right nor wrong). Wired into `sources_set_manual` + `content_type_set_manual`, snapshotting `composite_json` BEFORE `clear_suggestions` deletes the row.
>
> Lesson: **when an Architect doc's Phase scope rests on memory of how something is implemented, validate against actual source BEFORE drafting**. The "schema migration" Phase C would have been a no-op had I not re-checked `reliability.rs` first. The real Phase C work (wiring dormant machinery) is more valuable AND smaller scope.
>
> ### V3-§9.D — Reasoning Cataloger axis-aware GBNF (`b18a3ee`)
>
> Same pattern as Phase C: the existing SYSTEM_PROMPT already explicitly distinguishes the two axes (HORIZONTAL = SOURCE; VERTICAL = CONTENT TYPE), and the existing combined GBNF already enforces axis separation at the leaf level (h_value rule contains only horizontal IDs, v_value only vertical). So Phase D's prompt-rewrite scope was already done.
>
> What WAS missing for the V3-§7.b two-pass interface: axis-specific grammar functions that constrain the LLM to ONLY one axis per call. Added:
>
> - `build_gbnf_horizontal_only()` — horizontal-only grammar
> - `build_gbnf_vertical_only()` — vertical-only grammar
> - `build_gbnf_combined()` — backward-compat alias for the existing combined grammar
> - `GRAMMAR_CACHE_HORIZONTAL` + `GRAMMAR_CACHE_VERTICAL` `OnceLock`s (same pattern as r4.6)
>
> No runtime change today — Reasoning Cataloger still abstains because llama.cpp isn't wired (V3-§7.b deferred). When V3-§7.b ships, the wiring layer chooses single-pass (`combined`) or two-pass (`horizontal_only` + `vertical_only`) per benchmark.
>
> ### Test coverage
>
> | Phase | New tests | Cumulative cece tests |
> |---|---|---|
> | V3-§9.A | 4 | 71 (was 67) |
> | V3-§9.B | 7 | 78 |
> | V3-§9.C | 7 | 85 |
> | V3-§9.D | 5 | 90 |
> | **Total** | **+23** | **90 cece tests pass** |
>
> ### Gate 2 Boss-test ready
>
> Per Plan §5, Gate 2 has 6 stages exercising:
>
> 1. **Stage 1**: lexicon entries fire on test notes (e.g. "the concept of constructive proof" → `semantic-contents/concept`)
> 2. **Stage 2**: structural detectors fire on definition / worldview / figure / code notes
> 3. **Stage 3**: per-axis reliability JSON has both `horizontal` and `vertical` sub-objects, axis-specific updates don't cross-pollute
> 4. **Stage 4**: Reasoning Cataloger interface check (still abstains)
> 5. **Stage 5**: end-to-end on 5-10 diverse notes — vertical primaries diverge across content types
> 6. **Stage 6**: no horizontal regression (re-classify الخط العربي, confirm UA short-circuit still produces Unanimous)
>
> Boss-test instructions per the Plan §5 will land separately.
>
> ### Two re-scopings worth carrying forward
>
> Both Phase C and Phase D had their Architect-doc scope reduced after auditing actual source. The pattern: **the V3-§8 cascade did more vertical-axis work than the original Plan accounted for, so two of the four "missing" pieces turned out to already exist**. The Phase C and D commits each include a paragraph in their commit message + this orientation entry calling out what was actually missing vs what was already there.
>
> Net: V3-§9 cascade landed faster than the Plan's 4-6hr estimate (~2hrs of agent time + Eisa's Gate 2 session pending) because two of the four phases were smaller in actuality than the Architect doc claimed. The Plan's pre-commit validation step (grep IDs through the taxonomy + audit current code before drafting) caught the divergence early enough to avoid wasted work.
>
> ### Build
>
> NSIS `Constellation_0.3.4_x64-setup.exe` rebuilt mtime 2026-05-11 08:40:01 (123 MB). Eisa runs Gate 2 against this build.
>
> **What changed in v1.89** (V3-§8.r8 — Source Review queue composition filter; needle-in-haystack solved):
>
> ### What problem this solves
>
> While verifying the r7 fixes Eisa hit the next gap: with a queue of 268 cards (267 Split), trying to find a card with a *specific* composition (e.g. vertical=Split + horizontal=settled, needed for the symmetric-check on the disambig auto-write) was needle-in-haystack scrolling. He noted: *"This led me to think we need a mechanism to search for such a thing to find the right match."*
>
> ### The minimal filter
>
> A single chip row above the count strip slices the queue by what kind of decision each card needs from the user:
>
> - **All** — the full queue (default; preserves the prior behavior)
> - **Both axes need your call** — both horizontal AND vertical have a populated `needs_user_disambiguation_between` array
> - **Source needs your call** — horizontal needs disambig, vertical settled
> - **Content type needs your call** — vertical needs disambig, horizontal settled
> - **Catalogers agreed** — neither axis needs disambig (Unanimous + StrongMajority — quick rubber-stamp candidates)
>
> Each chip carries its bucket count (`Source needs your call (43)`). Empty buckets are dimmed + disabled — clicking them is a no-op. When the active filter has zero matches, the rendered list shows a "No cards match this filter" hint with a "Show all cards" button. The whole filter row hides if the queue has 0 or 1 cards.
>
> ### What stayed unchanged
>
> The filter is purely a render-layer slicer. Operates only on the **rendered** card list via a `$derived` `filteredQueue`. The full queue is preserved everywhere it matters:
>
> - The `splitCount` chip ("267 need your call") still reflects the **true total** across the full queue
> - The Approve All confirm dialog math (`splitAwareSkipCount`) still operates on the full queue — Approve All processes ALL eligible cards regardless of which filter is active
> - Reject All same — clears the entire queue
>
> So the filter doesn't deceive the user about what's in their queue; it just lets them work through it one composition pattern at a time.
>
> ### Implementation
>
> - `axisNeedsUserCall(c, axis)` — per-axis predicate (semantic check on `needs_user_disambiguation_between`, same robustness pattern as r7)
> - `queueFilter` `$state` — one of 'all' / 'both' / 'source' / 'content_type' / 'agreed'
> - `filterCounts` `$derived.by` — single pass over the queue produces all 5 bucket counts
> - `filteredQueue` `$derived.by` — slices `queue` by the active filter
> - `filterChips` `$derived.by` — chip definitions array (declared in script because Svelte 5 only allows `{@const}` inside specific block parents like `{#if}` / `{#each}` / etc.)
> - 5 new `cece.queueFilter.*` i18n keys in en + ar
> - 4 new CSS classes (chip row, chip variants, empty-bucket hint)
>
> ### Build
>
> NSIS `Constellation_0.3.4_x64-setup.exe` rebuilt mtime 2026-05-10 22:31:30 (123 MB). Eisa should now be able to:
> 1. Click "Source needs your call" — see only horizontal-Split cards
> 2. Click "Content type needs your call" — see only vertical-Split cards (the bucket Eisa couldn't find by scrolling)
> 3. Run the symmetric check from r7 verification: pick any card in the "Content type needs your call" bucket, pick a CONTENT TYPE chip, verify both axes land in frontmatter
>
> ### Why this isn't scope-creep
>
> Source Review filter belongs in this V3-§8 cascade because the cascade's whole purpose is making the queue usable on a real Library scale. r5 added the queue-level Split chip; r6 fixed Linguistic for long Arabic notes; r7 fixed Approve All math + disambig data loss; r8 closes the workflow loop by letting the user navigate hundreds of cards by intent rather than scroll position. Without r8 the prior fixes can't be exercised on a queue this large.
>
> A larger search MIG (free-text title search, filter by primary value, filter by dissenting cataloger) is backlog material — separate scope, separate work, can ship later if needed.
>
> **What changed in v1.88** (V3-§8.r7 — two Gate-1 follow-ups closed: Approve All math + disambig auto-write):
>
> ### Two Gate-1 Boss-test follow-ups closed
>
> Gate 1 PASSed on the r6 build with two known-issues filed for triage. Eisa elected to fix both before V3-§9 instead of carrying the debt forward.
>
> - **Issue #1 — Approve All confirm-dialog math wrong on UA-short-circuited cards.** The dialog claimed *"0 agreed / 2 split"* on a queue with one visibly Unanimous card and one Split card. Per-card pill rendering was correct (Unanimous on the UA-short-circuited card; Split on the other). The queue-level `splitCount` filter and `splitAwareSkipCount` derived state both used `regime === 'split'` and both incorrectly counted the Unanimous card as Split.
>
>   **Investigation result:** the synthesis layer is correct. Added a Rust JSON-shape diagnostic test (`ua_short_circuit_serializes_both_regimes_as_unanimous`) that reproduces the exact UA-voiced + others-disagreeing scenario and asserts the JSON has `"regime":"unanimous"` for both axes — test passes. Means the bug is/was somewhere in the data round-trip (DB write, IPC return, or frontend parse) that I couldn't reproduce in isolation.
>
>   **Robust fix:** filter on the SEMANTIC PROPERTY rather than the regime string. `cardNeedsUserCall(record)` returns true iff at least one axis's `needs_user_disambiguation_between` array is populated. The synthesis layer only populates this array when `regime == Split` (`synthesis.rs:268-272`); UA short-circuit explicitly sets it to `None` on both axes (`synthesis.rs:136, 153`). So checking the array is equivalent to checking the regime in correct cases AND robust against any serialization edge case that left the regime field misformatted. Per-card pill (`isSplit`), queue chip (`splitCount`), and bulk-confirm dialog (`splitAwareSkipCount`) all now route through the same helper — guarantees they agree forever.
>
> - **Issue #2 — Disambig chip discarded the settled other-axis value.** When `Auteur theory` had horizontal=Split and vertical=Unanimous (settled on `epistemic-states/illusion`), picking the SOURCE chip wrote only `sources:` to frontmatter — the settled vertical value was silently lost. Architect's intended per-axis surgical behavior, but UX-lossy.
>
>   **Fix:** `cece_resolve_disambiguation` now reads `composite_json` from the suggestion row before writing, calls `extract_other_axis_settled` to find the other axis's primary if and only if that axis is NOT Split, and writes both axes via the existing `sources_set_manual` / `content_type_set_manual` IPCs. Defensive: malformed JSON / missing fields / Split-on-both-axes → fall back to the original surgical behavior (write only the picked axis). 5 unit tests cover the cases.
>
> ### Test coverage
>
> 67 cece module tests pass (was 66; +1 from the synthesis JSON-shape diagnostic). 5 new `classifier::r7_tests` for the `extract_other_axis_settled` helper — auto-write on horizontal pick, auto-write on vertical pick, returns None when both axes are Split, returns None on malformed JSON, returns None when the other axis primary is null. svelte-check: zero new errors.
>
> ### Why both fixes belong in r7 instead of being deferred
>
> Both bugs are user-visible and would be hit by any second Boss-test session. #1 made the bulk-accept confirm dialog give wrong-looking math (would have made Eisa not trust the bulk path). #2 silently discarded data the catalogers had agreed on (would have made the disambig flow feel lossy). Boss directive: don't ship known-broken UX paths. The Plan-Approval-Equals-Build-Approval directive applied to Eisa's "let's work on the two issues before moving on" — same cascade pattern as r1→r5.
>
> ### Build
>
> NSIS `Constellation_0.3.4_x64-setup.exe` rebuilt mtime 2026-05-10 21:41:13 (123 MB). Eisa to verify both fixes against the previous reproduction scenarios:
> - Re-classify الخط العربي (which now has `sources: testimony/authoritative` from Stage 2.1) → Approve All confirm dialog should now correctly count it as 1 agreed.
> - Pick a SOURCE chip on a card with vertical=Unanimous → both `sources:` AND `content_type:` should land in frontmatter.
>
> **What changed in v1.87** (V3-§8.r6 Linguistic timeout regression fix; caught mid-Gate-1 Boss-test):
>
> ### One-line fix from a Boss-test catch
>
> During Stage 1 of the Gate 1 re-run, Eisa noticed that re-classifying the `الخط العربي` note (a ~30K-character Arabic article on Arabic calligraphy) produced a card with the **Linguistic cataloger silently abstaining** — the dot cluster showed it as silent and the trail summary read *"Catalogers voiced: structural, semantic."* Pre-r4 builds had Linguistic voicing on this same note with five CAE roots seen.
>
> Root cause: V3-§8.r4 introduced per-cataloger timeout enforcement via `mpsc::recv_timeout`. Linguistic was grouped with the other cheap-tier catalogers at **500ms** budget. But Linguistic's three-path matching (CAE root extraction → lexicon surface-token scan → Bridge slow-path embedding similarity for unknown Arabic terms) scales with note length. On a 30K-character Arabic note the slow-path legitimately takes 600–1500ms — exceeding 500ms, the orchestrator's `run_one_safe` returns `None`, and Linguistic gets dropped from the trail set entirely.
>
> Fix: move Linguistic from the 500ms cheap tier to the 2s medium tier (alongside Graph + Semantic). One-line edit in `orchestrator.rs::cataloger_timeout`. Regression test added (`linguistic_gets_medium_timeout_not_cheap`) so it doesn't drift back.
>
> ### Why this matters beyond the one card
>
> Constellation's Linguistic cataloger is the lens that's *supposed* to do the heavy lifting on technical Arabic content (Quranic vocabulary, Sufi terminology, *uṣūl al-fiqh* technical terms). The 500ms budget silently disabled it on exactly the use case it was built for. The Boss-test caught it; without the test, the regression would have shipped invisibly.
>
> Lesson worth carrying: per-cataloger timeout budgets need to scale with the cataloger's *worst-case* latency on its target inputs, not its typical latency on tiny test fixtures. The next cataloger added to the ensemble must declare its slow-path latency explicitly and the orchestrator's `cataloger_timeout` mapping must reflect it.
>
> ### Build
>
> NSIS `Constellation_0.3.4_x64-setup.exe` rebuilt (mtime 2026-05-10 20:09:57, 123 MB). Eisa to resume Gate 1 Stage 2 against this build.
>
> **What changed in v1.86** (V3-§8 remediation cascade r1→r5 closed; Gate 1 re-run pending):
>
> ### The remediation cascade is done
>
> All five r-rounds Eisa approved on 2026-05-10 ((A) — stop and remediate ALL P0+P1+P2 before Gate 1 PASS) shipped autonomously per the Plan-Approval-Equals-Build-Approval directive. Composite verdict at the start was ~6/10 ("architecture sound; implementation has specific reproducible gaps"). The cascade addressed every audit finding except those explicitly deferred (out of scope for V3-§8).
>
> - **r1 — P0 critical fixes** (`663e31f`): Arabic comma handling in CAE (`is_arabic_punctuation()` covering ، ؛ ؟ ۔), prompt-injection resistance via random-hex nonce delimiter (replacing the triple-backtick fence + a SYSTEM_PROMPT guard), cross-Library reliability isolation via explicit `/` separator in `library_root_for_note`, the missing **Sibling Disambiguation form** (radio-chip picker + `cece_resolve_disambiguation` IPC), and the `compute_regime` `total_voters >= 3` gate dropped to a ratio-based threshold so 2-voter steady-state can actually reach StrongMajority.
> - **r2 — Synthesis architecture** (`23e50c0`): `MemoizedEmbed` shares the embed call across Linguistic + Semantic instead of computing it twice per note; `OnceLock` lazy fields removed from `CatalogerContext` (dead code); `AxisDecision.secondary` now populated from the second-place vote so the principal/secondary distinction the Architect §3 spec'd actually surfaces.
> - **r3 — Lexicon corrections** (`72629fc`): qiyās → `inference` parent (was `comparison/ratio-legis`); حدثنا → `testimony/reported` (was `mass-transmission/verbal` — opposite!); أعتقد → `belief/occurrent` and أظن → `opinion/probable` (split was conflated); bare متواتر → `mass-transmission` parent (was `mass-transmission/verbal`); anupalabdhi → `non-apprehension` parent (was `non-apprehension/absolute`).
> - **r4 — Robustness** (`84cde6f`): atomic reliability JSON write via `tempfile::Builder::tempfile_in().persist()` (cross-platform safe rename), mutex poison recovery via `lock().unwrap_or_else(|e| e.into_inner())` so one panic doesn't poison all subsequent classifications, `ALTER TABLE` moved out of per-IPC paths into `init_db`, `mpsc::recv_timeout` per cataloger budget enforcement (cheap=500ms / medium DB=2s / Reasoning LLM=5s), NFKC Unicode normalization on every cataloger's input + at lexicon load, GBNF grammar `OnceLock` cache so parent-restricted second pass doesn't recompile per call.
> - **r5 — UX polish** (this commit): seven sub-items addressing the audit's UX/cognitive findings.
>
> ### r5 detail (the UX cluster the Boss-test cards flagged)
>
> - **r5.1 — Badge cluster as 6 tinted dots.** `UA STR LIN GRP SEM RSN` abbreviations replaced with six color-keyed dots (blue/rose/amber/teal/violet/green, one per cataloger lens). Status (voiced+agrees / voiced+dissent / silent) encoded by fill + ring + glyph so color alone is never the channel. Tooltip on each dot names the cataloger + status in plain language. Cluster is now scannable in one glance instead of "what does STR even mean again?".
> - **r5.2 — Reasoning trail render layer.** `rules_fired` strings (`rule_of_authority`, `cae_root_match`, `typed_neighbor_consensus`, `gbnf_constrained`, etc.) translate via a `ruleLabel()` helper to friendly chips ("Your frontmatter is the authority", "Arabic root match (CAE)", "Typed-link neighbor consensus", "AI judgment (grammar-constrained)") rendered as a strip under each cataloger's reasoning sentence. 13 known rule keys mapped — verified against actual `rules_fired.push(...)` call sites in the six cataloger source files; unknown keys de-snake-case as the fallback. Lens-color dot leads each trail entry so the cluster's color vocabulary carries through.
> - **r5.3 — Trust-calibration default.** Reasoning trail auto-expands by default for the first 50 reviews (`localStorage`-backed counter; bumped on every Accept / Reject / Edit-commit / Disambiguation pick of a composite-trail card). After 50, the trail collapses to on-demand. Quiet "Showing reasoning trails until you review N more cards" banner at panel top while still calibrating. Counts only composite-trail cards — legacy v2 cards don't move the counter (no trail to learn from).
> - **r5.4 — Queue-level Split count chip.** Header strip shows `42 pending • 7 need your call` instead of relying on per-card gold borders that become wallpaper. The chip is the at-a-glance "decisions waiting" count.
> - **r5.5 — Approve All Split-aware.** New `skip_split: bool` parameter on `sources_accept_all_pending` (defaults to `true` from the frontend). The bulk-accept worker reads `composite_json`, parses `regime` on either axis, and skips cards where catalogers split — those stay in the queue for the user to decide via the radio-chip form. Confirm dialog now reads "Apply suggestions to N notes whose catalogers reached agreement" + an aside explaining the M skipped cards.
> - **r5.6 — T1/T2 → 'Legacy' pill.** v2-era rows (no `composite_json`) used to render `T1` or `T2` — abbreviations the user was never taught. Replaced with a single italic `Legacy` pill with a tooltip explaining "classified before the cataloger ensemble was added — no per-cataloger trail available."
> - **r5.7 — Blockquote regex weight + attribution rule.** `data/sources_lexicon.json` blockquote rule: bare `(?m)^> +\S` weight dropped from 0.70 → 0.40 (a paragraph-emphasis blockquote in a personal note is too common to be strong testimony evidence on its own). New companion rule matches blockquote followed within 3 lines by attribution markers (em-dash + name, "source:", "author:") at weight 0.85 — that's where the original strong-testimony reading is justified. Three regression tests added.
>
> ### i18n
>
> Every new user-facing string went through `$t()`. New keys under `cece.badge.*` (cluster tooltip, status verbs, legacy pill), `cece.regime.unanimousTooltip`, `cece.trail.*` (added arrows), `cece.rule.*` (13 rule labels — one per actual cataloger rule key), `cece.trustCal.*` (banner + tooltip), `cece.queueSplit.*` (chip label + tooltip). Both `en.json` and `ar.json` populated. Other 13 locales fall back to the inline EN defaults — same pattern as V3-§8.r1.e (translators land them as separate work).
>
> ### Test coverage
>
> 65 cece module tests pass (was 62 in v1.85; +3 from r5.7 blockquote weight regression tests). 11 structural cataloger tests pass — confirms the bare-blockquote 0.40 + attributed-blockquote 0.85 split works as designed.
>
> ### NSIS installer
>
> Rebuilt at `Constellation_0.3.4_x64-setup.exe` (mtime 2026-05-10 19:18, 123 MB). Binary mtime 19:18:37. Two builds were required this round — the first finished cleanly but the rule-key mapping in `ruleLabel()` carried inferred keys that didn't match what the catalogers actually emit; caught + corrected before commit; second build landed the corrected mapping.
>
> ### What's next
>
> Re-run the V3-§8 Gate 1 Boss-test on this build. Eisa should see (a) the new dot cluster instead of abbreviations, (b) reasoning trails auto-expanded with the friendly rule chips, (c) a "47 pending • 8 need your call" count strip, (d) the Approve All confirm dialog mentioning the Split skip, (e) `Legacy` pills on any pre-CECE rows. Sibling Disambiguation radio chips on Split-regime cards should already work from r1.
>
> If Gate 1 PASSes: V3-§9 vertical-axis activation → Gate 2 → V3-§10 Settings + i18n + Help docs → V3-§11 audit + close-out.
>
> ### Body still hasn't moved
>
> Same as v1.74 onward: §3 / §4.x / §13 / §17 of body. Body update lands at V3-§11 close-out (post-remediation, post-Gate-2, post-vertical-axis-active).
>
> **What changed in v1.85** (V3-§8 audit complete; Boss directive (A): stop and remediate before Gate 1 PASS):
>
> ### The audit
>
> Eisa surfaced visible problems in the V3-§8 Gate 1 Boss-test ("looking at the results, they are almost identical!"). Three on-the-fly patches (V3-§8 fix-A+B+C: real ensemble weights / tightened DOI regex / Structural single-hit downgrade) addressed the symptoms partially. Eisa then requested an independent six-cataloger audit before any further patches, mirroring the CECE architectural pattern at the meta-level.
>
> Six independent reviewer agents spawned in parallel through methodologically distinct lenses: **Library Science**, **NLP/ML Engineering**, **Software Architecture**, **UX/Cognitive**, **Epistemology / *uṣūl al-fiqh***, and **Adversarial / Edge-Case**. Each briefed cold, pointed at absolute paths, instructed to disagree.
>
> All six returned with substantive findings. **Composite verdict: ~6/10. Architecture sound; implementation has specific reproducible gaps.** Full report: `lab/reports/MIG-021v3-V3-§8-AUDIT.md`.
>
> ### Critical findings the audit surfaced
>
> **Implementation gaps where commit messages didn't match the code**:
> - Sibling Disambiguation form **does not exist** in the Svelte (commit `daeba00` shipped only the placeholder pill — no radio chips, no `cece_resolve_disambiguation` IPC handler)
> - Top-down decomposition spec'd in Architect §5.1 + Plan V3-§7 + `rules_fired = ["schedule_navigation_top_down"]` but **never implemented** in `reasoning.rs` (single fat grammar, no parent-restricted second pass)
> - **Zero `cece.*` i18n keys exist** — every CECE string is hardcoded English; Boss-test Arabic UI had English bleeding through
> - `AxisDecision.secondary` field hardcoded `Vec::new()` — half-built principal/secondary distinction
> - `OrchestratorState` defined but never used — orchestrator built fresh per IPC call
> - `OnceLock` lazy fields on `CatalogerContext` declared but never read by any cataloger
>
> **P0 reproducible bugs**:
> - **Arabic comma silently kills CAE root path** — `is_ascii_punctuation()` doesn't include `،`/`؛`/`؟`/`۔`, so `هذا قياس،صحيح` → CAE can't extract root → silent abstention. The cataloger documented as "Strong on technical Arabic" is silently weak on Arabic. Explains why `الخط العربي` test showed `LIN✓` (only surface-token fired)
> - **Prompt injection via triple-backtick fence** — user note containing literal ` ``` ` closes the fence early; LLM follows injected instructions toward any *valid* taxonomy ID. Working payload provided
> - **Cross-Library reliability data leakage via path-prefix collision** — `/Universe/Notes` and `/Universe/Notes_old` collide. Direct violation of Architect §10 invariant 9
> - **`compute_regime` cannot reach StrongMajority with typical voter counts** — `total_voters >= 3` gate kicks before majority math; with 2-voter steady-state, every disagreement floors at Split. Eisa observed this on every card
>
> **Convergent findings** (multiple agents flagged independently): mutex poisoning cascade (Software Arch + Adversarial); `ALTER TABLE` per IPC call (Software Arch + Adversarial); kNN per-call cost / no cache (Software Arch + Adversarial); Split-everywhere bug (NLP + LIS + UX); critical lexicon thinness for cold-start (NLP + LIS implied).
>
> **Confirmed neutralized**: regex DoS (Rust `regex` is RE2-style guaranteed linear); GBNF for closed-set classification (small accuracy + large reliability win); `MIN_SAMPLES_FOR_WEIGHTING=20` (defensible against Bayesian credible-interval math); CIP-precedent for User-Authority absolute precedence.
>
> ### Boss directive 2026-05-10: **(A)** Stop and remediate before Gate 1 PASS
>
> Five-phase remediation cascade (~3-5 days):
> - **V3-§8.r1** — P0 critical fixes (Arabic comma; prompt-injection fence; path-prefix collision; `cece.*` i18n keys; **Sibling Disambiguation form**; `compute_regime` threshold)
> - **V3-§8.r2** — Synthesis architecture (`OnceLock` vs injection unification; `OrchestratorState` actually used; `AxisDecision.secondary` populated)
> - **V3-§8.r3** — Lexicon corrections (qiyās → inference; حدثنا → testimony/reported; أظن → ẓann; bare متواتر → parent; anupalabdhi → parent; tradition field)
> - **V3-§8.r4** — Robustness (tempfile rename; mutex poison recovery; ALTER → init_db; timeouts; NFKC normalization)
> - **V3-§8.r5** — UX polish (badge dots; reasoning trail render layer; trust-calibration default; queue-level Split count; Split-aware Approve All)
>
> After r5, re-run Gate 1 Boss-test cleanly.
>
> ### Body still hasn't moved
>
> Same as v1.74 onward: §3 / §4.x / §13 / §17 of body. Body update lands at V3-§11 close-out (post-remediation).
>
> **What changed in v1.84** (MIG-021v3 V3-§1 through V3-§8 shipped — CECE Cataloger Ensemble live in Source Review; Gate 1 Boss-test ready):
>
> ### CECE shipped end-to-end (cargo + UI)
>
> All six catalogers now run as a real ensemble against any classified note:
>
> - **V3-§1 Foundation** (`4afb7d9`) — Cataloger trait, six-cataloger orchestrator with cost-ordered two-pass run + panic isolation, weighted-vote synthesis with three confidence regimes (Unanimous / StrongMajority / Split), per-Library reliability JSON tracking, declarative cataloger rules registry. 11 unit tests.
> - **V3-§2 User-Authority Cataloger** (`6b3d41a`) — frontmatter-only; absolute precedence per invariant 1; synthesis layer short-circuits when this voices.
> - **V3-§3 Structural Cataloger** (`21bd2b8`) — citations (ISBN/DOI/blockquote regex), stance markers (English + Arabic — "I doubt" / أشكّ → epistemic-states/doubt; "I'm certain" / متأكد; "I believe" / أعتقد), mathematical/theorem markers, numerical units.
> - **V3-§4 Linguistic Cataloger** (`03fcaa2`) — three-path matching: (1) CAE root match HIGH confidence when CAE knows the root, (2) lexicon surface-token match MEDIUM, (3) Bridge slow-path embedding similarity LOW for unknown Arabic terms. Honest finding: CAE root coverage is sparse today — most matches go via path 2.
> - **V3-§5 Semantic Cataloger** (`4f735df`) — per-Library kNN-blend over already-classified vault notes via injectable embed_fn + lookup_fn. Cold-start abstain at <3 classified neighbors; min cosine 0.55.
> - **V3-§6 Graph Cataloger** (`d041238`) — Living Links typed-neighbor consensus with link-type-weighted votes (derives-from / part-of +1.0; supports / generalizes +0.7; causes / exemplifies +0.5; contradicts INVERTED -0.7). Cold-start abstain at <2 classified typed neighbors.
> - **V3-§7 Reasoning Cataloger logic + prompt + GBNF** (`8171244`) — system prompt with the five Cataloger Rules + 12 hand-crafted few-shot exemplars spanning the taxonomy, GBNF grammar generator that enumerates every valid taxonomy ID (LLM literally cannot emit OOV labels), JSON response parser with defense-in-depth ID validation. **Local-only per Boss directive** — no cloud track. **The llama-cpp-2 dep + Qwen3-4B GGUF download deferred to V3-§7.b** (Plan §13 Windows-toolchain risk surfaced as its own commit). Cataloger abstains gracefully today; ensemble runs on the five other catalogers.
> - **V3-§8 Orchestrator wiring + classifier IPC swap + SourceReview UI rewire** (this commit) — production wiring (`src-tauri/src/cece/wiring.rs`) instantiates all six catalogers and connects them to the real backends (EmbeddingState for embeddings; search.db for kNN + typed-neighbor lookups). `classifier_suggest_for_note` now runs the CECE ensemble + persists composite reasoning trail in a new `composite_json` column on `sources_suggestions` (additive schema; legacy v2-era rows still readable with `composite_json` NULL). SourceReviewPanel renders per-cataloger badge cluster (UA / STR / LIN / GRP / SEM / RSN with ✓ / ✗ / – status), gold left border on Split cards, "Why this classification?" expandable reasoning trail showing each voicing cataloger's reasoning + confidence.
>
> **55 CECE unit tests pass** across foundation + all six catalogers + synthesis + reliability tracking + orchestrator panic isolation.
>
> ### Architectural decisions visible in v1.84
>
> - Sources of truth respected: CECE depends on CAE (root analysis), Lexical Bridge / bridge_vectors (embedding similarity), Living Links (typed-neighbor graph), e5-small ONNX engine — but modifies none of them. Architect §10 invariant 7.
> - Privacy guarantee absolute: no cloud inference path exists. Architect §10 invariant 4 (strengthened from "local-first by default; cloud opt-in" 2026-05-10).
> - Panic isolation per cataloger via `catch_unwind` in orchestrator. Architect §10 invariant 5.
> - Backward compatibility: v2-era `sources_suggestions` rows render in the v3 UI as legacy single-tier cards (no per-cataloger badge cluster, no reasoning trail expand button) until reclassified.
>
> ### NSIS installer
>
> Rebuilt at `Constellation_0.3.4_x64-setup.exe` (mtime 2026-05-10 16:20).
>
> ### What's next
>
> Boss-test Gate 1 (horizontal axis) on this build. After PASS, V3-§9 vertical-axis activation → Gate 2 → V3-§10 Settings + i18n + Help docs → V3-§11 audit + close-out.
>
> ### Body still hasn't moved
>
> Same as v1.74 onward: §3 / §4.x / §13 / §17 of body. Body update lands at V3-§11 close-out.
>
> **What changed in v1.83** (MIG-021v2 cascade closed; MIG-021v3 reframed as the **Constellation Epistemic Content Engine (CECE)**; Cataloger Ensemble Architect approved):
>
> ### The reframing
>
> Two cumulative Boss-driven architectural reframings stacked on 2026-05-10 changed the v2 cascade direction completely:
>
> - **Reframing 1 — generic classifier → cataloger algorithm.** A classifier asks *"what label is most likely?"* and outputs probabilities. A cataloger asks *"how would a trained DDC-style cataloger navigate the taxonomy and arrive at an assignment?"* — and outputs an **assignment + reasoning trail**. Different optimization target, different output, different failure mode.
>
> - **Reframing 2 — single cataloger → cataloger ensemble.** Multiple methodologically distinct catalogers reading through different lenses produce uncorrelated errors. Disagreement is itself a signal: it identifies the Bayes-irreducible cases and surfaces them to the user with all reasoning trails visible. Library-science precedent (LC, NLM, OCLC second-cataloger review). ML precedent (Snorkel, Mixture of Experts, BioASQ-winner stacking).
>
> Both reframings were preceded by four parallel research-agent reports grounding the architecture in real literature (LSHTC challenges; LCSH/MeSH/Dewey inter-cataloger studies; Qwen3 / Phi / Llama benchmark data; MATCH WWW 2021 + Snorkel + active-learning literature). Eisa's CAE precedent (Constellation Arabic Engine — purpose-fit primitive that broke the off-the-shelf Arabic NLP ceiling) was the analogy that justified the move.
>
> ### What this means for the system
>
> The **CECE** is the system as a whole. The **Cataloger Ensemble Architecture** is the implementation pattern. Six catalogers run per note, each leveraging exactly one Constellation primitive:
>
> | Cataloger | Lens | Constellation primitive |
> |---|---|---|
> | Linguistic | Root-pattern morphology + cross-civilizational equivalence | CAE + Lexical Bridge |
> | Structural | Frontmatter + citations + headings + blockquotes + link types | Existing parsers |
> | Graph | Typed-neighbor consensus | Living Links typed graph |
> | Semantic | kNN-blend over already-classified vault | Tier-2 e5-small embeddings |
> | Reasoning | Schedule + rules in prompt | Local Qwen3-4B Q5_K_M (notes never leave the device — Boss directive 2026-05-10) |
> | User-Authority | Frontmatter + capture-time fields | Frontmatter precedence (absolute) |
>
> A synthesis layer combines the six reasoning trails into one of three **confidence regimes**: Unanimous (silent accept), Strong Majority (accept with dissent surfaced), Split (refuse to assign and ask the user via Sibling Disambiguation UI). The killer property: the ensemble **detects Bayes-irreducible cases automatically** — when methodologically diverse catalogers still disagree, the engine knows the distinction doesn't live in the text, and asks instead of guessing.
>
> ### What's preserved from v2
>
> All §1A'–§1F'.b ship-code is preserved as CECE substrates. Zero rollbacks, zero schema changes. Repositioning:
> - §1A' schema → CECE substrate
> - §1B' tier1_embedding → Semantic Cataloger
> - §1C' Source Review panel → ensemble review UI (cards now render reasoning trails)
> - §1D' PropertyEditor → User-Authority Cataloger input path
> - §1E' right-click action → ensemble-on-one-note trigger
> - §1F' background scan → ensemble-across-vault runner
> - §1F'.b Approve All / Reject All → preserved (Approve All now writes synthesized assignments)
> - §1G' i18n → preserved
> - §1G2' code (Tier 1 rules + lexicon + correction log) → committed today as the v2 close-out commit; reorganizes into Linguistic + Structural Cataloger seeds in v3
>
> ### CECE Architect doc
>
> Full architecture spec at `lab/reports/MIG-021v3-EPISTEMIC-CONTENT-ENGINE-ARCHITECT.md`. Sections:
> - §1 Vocabulary
> - §2 The six Catalogers
> - §3 Synthesis Layer (three confidence regimes; weighted vote vs Snorkel synthesis)
> - §4 Cataloger Rules (five declarative principles)
> - §5 Schedule Navigation (top-down per axis; depth budget; abstain-at-depth)
> - §6 Repositioning what we shipped
> - §7 Net-new components
> - §8 Boss decisions (eight tradeoffs — all approved with defaults 2026-05-10)
> - §9 Performance budget
> - §10 Twelve invariants
> - §11 Migration path
> - §14 Sourced-vs-engineering-inference appendix
>
> ### What's next
>
> MIG-021v3 PLAN drafting authorized. Phases organized around catalogers (per-axis Boss-test gates, per Eisa's §8.7 decision). v3 PLAN lands in a follow-up commit; v3 BUILD cascade follows.
>
> **Body still hasn't moved**: §3 / §4.x / §13 / §17 of v1.74 body — body update lands at v3 close-out (post-CECE ship).
>
> **What changed in v1.82** (MIG-021v2 §1E' PASS, §1F' shipped + Boss-test Stage 1 PASS with one fix, §1F'.b Approve All / Reject All bulk actions added on Eisa request):
>
> - **§1E' (right-click "Suggest sources & content type")** — Boss-test PASS all four stages (menu visibility, action fires, queue de-dupe on re-classify, Arabic label). Commit `0d93753`.
> - **§1F' (background scan)** — shipped end-to-end:
>   - Backend: `src-tauri/src/classifier/scan_job.rs` with three IPCs (`classifier_scan_start` / `_cancel` / `_status`). Cooperative cancel via AtomicBool. Worker thread; chunked progress events every 5 notes. Resumability is implicit — `enumerate_pending` SELECTs from `note_meta` excluding rows already in `sources_suggestions` AND requiring at least one axis empty. Closing mid-scan and restarting picks up where the previous run stopped; no separate cursor.
>   - Frontend: new `ClassifierScanProgressStrip.svelte` mounted in the status-bar center (next to MigrationProgressStrip). Settings → Intelligence → "Sources & content type classifier" section with Start scan button.
>   - Boss-test Stage 1: PASS for setup/start/typing-stays-instant. One bug surfaced: SourceReviewPanel didn't auto-update queue count during scan (only on tab-switch re-mount). Fixed in `1110467` — panel now listens for `classifier:scan` events with debounced 1.5 s queue reload.
>   - Commits: `ff21354` (initial), `1110467` (live-update fix).
> - **§1F'.b NEW (Approve All / Reject All)** — Eisa request after seeing 6,664 pending cards from his first full-Universe scan. Reviewing each by hand was infeasible, so:
>   - Backend: `src-tauri/src/sources/bulk_ops.rs` with four IPCs (`sources_accept_all_pending` + `_bulk_accept_cancel` + `_bulk_accept_status` + `sources_reject_all_pending`). Approve runs on a background thread with `BulkAcceptState`; mirrors per-card Accept semantics (writes ALL suggestions per axis to each note). Reject is a single SQL `DELETE`.
>   - Frontend: two new buttons in the SourceReviewPanel count row + inline confirmation dialog with the count + inline progress bar with Cancel during bulk-accept.
>   - Commit `fb13594`.
> - **NSIS installer**: rebuilt at `Constellation_0.3.4_x64-setup.exe` (mtime 2026-05-10 09:08).
> - **Cascade resumes** with §1F' Stages 2–3 (Cancel + close-and-resume) once Eisa wraps the bulk-action testing, then §1G' i18n full pass.
>
> Body still hasn't moved (§3 / §4.x / §13 / §17 of v1.74 body update lands at §1K' close-out).
>
> **What changed in v1.81** (MIG-021v2 Plan amended — Tier 1 rules + provenance metadata + correction log inserted before Tier-3 LLM):
>
> Eisa surfaced an external SME analysis post-§1D' showing that the current Tier-2-only architecture (e5-small embeddings) caps Source-axis accuracy at ~75-85% at leaf level — a structural limit no model swap removes. The same analysis ranked six recommendations by ROI; three top items are absent from the original Plan: (a) a Tier 1 deterministic rules engine + bilingual lexicon, (b) provenance metadata fields (`source_citation`, `acquisition_method`, `confidence`) surfaced at capture time, (c) an active-learning correction log that turns every user override into ground truth.
>
> Eisa: *"Proceed with your recommendation"* — authorizing **Option B**: insert the missing Tier 1 + provenance phases into MIG-021v2 before §1H', so the system ships as a coherent three-tier deliverable rather than a two-tier deliverable + a deferred follow-up MIG.
>
> **Plan amendment** (lab/reports/MIG-021v2-EPISTEMIC-CLASSIFIER-REDESIGN-PLAN.md):
> - **NEW §1G2'** — Tier 1 rules engine + bilingual lexicon JSON asset + correction log (NDJSON at `<library>/.constellation/classifier_corrections.jsonl`). Boss-test gate.
> - **NEW §1G3'** — provenance metadata schema + QuickCaptureSourceWidget (single dropdown at note-creation time). Boss-test gate.
> - **§1H' renamed** Tier-2 → Tier-3 to reflect the corrected three-tier architecture (deterministic rules → embeddings → LLM, with confidence-routing between).
>
> No prior phase rolled back. §1A'-§1D' shipped foundation survives intact. §1E' (right-click) and §1F' (background scan) cascade resumes pending §1D' Boss-test result.
>
> Architecture clarifications recorded in the Plan's amendment headers:
> - Hierarchical taxonomy gives us **graceful degradation** the SME analysis didn't credit — Sources at the 11-parent level realistically hit 85-92%; tier-aware fallback (§1B') already suggests parent when leaf confidence < 0.55.
> - Eisa-canonical taxonomy carries paired EN/AR/Sanskrit terms baked in as candidate signatures — raises embedding-tier ceiling on cross-civilizational notes meaningfully.
> - Honest framing for downstream marketing/help docs: "up to 95% on what your note is *about*; up to 85% on *how you came to know it*; the system tells you when it isn't sure" — not "90-95% on Sources" (that would erode trust on first contact).
>
> **Body still hasn't moved**: §3 / §4.x / §13 / §17 of the v1.74 body update lands at §1K' close-out.
>
> **What changed in v1.80** (MIG-021v2 §1C' Boss-test PASS; §1D' PropertyEditor inline pickers shipped):
>
> - §1C' Boss-test gate cleared end-to-end. Eisa: *"All pass"* — covering Stage 1 (build + dual-axis classification), Stage 2 (Accept / Edit / tree mechanics / Save / Cancel), Stage 3 (Arabic walkthrough with locale-aware labels). Three §1C' fixes folded in: locale-aware label rendering, pickers always stacked vertically, and TaxonomyTreePicker flat-render rewrite (replaces recursive Svelte 5 `{#snippet}` self-reference with pre-walked `Row[]` derived state).
> - §1D' shipped: PropertyEditor recognizes `sources:` and `content_type:` keys and renders the same TaxonomyTreePicker inline (collapsible, pills + chevron). Storage flows through the existing YAML save path (`saveTabContent` → `index_note` re-extracts on save) — no special IPC, no second source of truth. `sources` and `content_type` added to KEY_SUGGESTIONS (EN + AR).
> - NSIS installer rebuilt: `Constellation_0.3.4_x64-setup.exe`.
> - Cascade resumes — §1E' (right-click context action) next.
>
> **What changed in v1.79** (MIG-021 build cascade paused; Eisa-authored horizontal taxonomy delivered; v2 Redesign Architect drafted):
>
> ### The pivot
>
> Boss tested §1A → §1B → §1C cascade end-to-end (Stages 1, 2, 3 all PASS after fix-1/fix-2/fix-3). At gate-clear, Eisa surfaced: the 11 flat horizontal sources are too abstract for non-expert users to recognize ("Constructed Idea is easier than Semantic Contents"). Directive: include the WHOLE taxonomy, let users pick at any depth, **two parallel fields** (Option B), and at least 2 levels of depth on the horizontal axis.
>
> Per Plan-Approval-Equals-Build-Approval's "architectural surprise" exception, MIG-021 §1D-§1K paused. Eisa explicitly directed me to wait while he authored the horizontal taxonomy himself rather than ratifying my draft (per BASIC RULE — no fabrication of canonical Constellation primitives).
>
> ### Eisa-authored horizontal taxonomy delivered
>
> `docs/sources-of-knowledge-diagram.html` — interactive 3-level diagram, Eisa-canonical:
> - **11 horizontal parents** (S1-S11) with **tier metadata** (NEW design dimension):
>   - Tier 1 (universally accepted, teal `#0f6e56`): S1 Perception, S2 Inference
>   - Tier 2 (broadly accepted, purple `#534ab7`): S3 Testimony, S5 Comparison, S8 Memory, S9 Innate disposition
>   - Tier 3 (school-specific or contested, amber `#854f0b`): S4 Mass-transmission, S6 Postulation, S7 Non-apprehension, S10 Inspiration, S11 Revelation
> - **41 sub-leaves** in scholarly traditional terms — *uṣūl al-fiqh* classifications for S4 Mass-transmission (لَفظي / مَعنوي / عَمَلي), the four classical *qiyās* types for S5 Comparison, *Mīmāṃsā* sub-distinctions for S6 Postulation, traditional *anupalabdhi* sub-types for S7 Non-apprehension (*prāgabhāva* / *pradhvaṃsābhāva* / *anyonyābhāva* / *atyantābhāva*), etc.
> - **Tri-script labels** — EN + AR + (where present) Sanskrit/Pali transliteration on most nodes
>
> ### Fresh Redesign Architect drafted
>
> `lab/reports/MIG-021v2-EPISTEMIC-CLASSIFIER-REDESIGN-ARCHITECT.md` — supersedes the original `MIG-021-EPISTEMIC-CLASSIFIER-ARCHITECT.md`. Key changes from the original:
> - **§2 horizontal taxonomy** is Eisa-canonical (no ratification needed — already his work)
> - **§3 vertical axis** lifted intact from the existing `epistemic-content-taxonomy-chart.html`
> - **§5 tree picker** — new component `TaxonomyTreePicker.svelte`, mirrors the visual language of both diagrams (tier coloring, tri-script labels, search/filter)
> - **§6 classifier extension** — ~271 cached vectors instead of 11; tier-aware confidence fallback (when top-1 is Tier 3 and confidence < 0.55, suggest Tier 1/2 alternative — avoids inappropriately surfacing contested categories on secular notes)
> - **§10 tier UX system** — entirely new design dimension: tree picker tier coloring, Settings → Sources opt-out for Tier 3, classifier fallback
> - **§7 re-sequenced phases** §1A' → §1K' with same 3 Boss-test gates structure
> - **§11 Concept Paper amendments** — §7 of v2.0 needs substantive revision; lands in v2.1 at §1K' close-out
>
> ### Boss review checklist (per Architect §14)
>
> 1. §2 horizontal taxonomy — already Eisa-canonical ✅ no ratification needed
> 2. §5 tree picker UI principles — approve or revise
> 3. §6 classifier extension — approve tier-aware fallback at confidence < 0.55
> 4. §7 phase re-sequence — approve §1A' → §1K'
> 5. §9 Sight mode P design — Option α/β/γ (or defer to MIG-022)
> 6. §10 tier UX — approve tier coloring + Settings opt-out for Tier 3 + classifier fallback
> 7. §12 open questions — agree to all 8 defaults or override per question
>
> ### What's preserved on `main`
>
> All §1A/§1B/§1C foundation work (commits `4d6ef37`, `dcbd40e`, `4e70393` + fixes `c3f3e96`, `4769fbe`, `ec288fe`) survives. The redesign expands vocabulary and replaces the flat picker with a tree picker; ~all backend infrastructure adapts. Substantively nothing on `main` is rolled back; v2 builds forward on the foundation.
>
> ### Body still hasn't moved
>
> Same as v1.75/v1.76/v1.77/v1.78: §3 / §4.x / §13 / §17 of the v1.74 body not yet rewritten. Body update lands at §1K' close-out when Sight v5 implementation completes coherently.

> **What changed in v1.78** (MIG-021 Architect approved; Plan drafted; six open questions locked):
>
> Eisa, after the v1.77 PCS: *"Enough of your never-ending technical questions. Proceed with the MIG-021 Architect Phase-2."* Stop-On-Correction Rule fired — the six open questions in the Architect were treated as gates by me (asking-mode); the directive was to lock defaults inline and proceed.
>
> **MIG-021 Architect approved by directive** (no per-question approval required).
>
> ### File added
>
> - `lab/reports/MIG-021-EPISTEMIC-CLASSIFIER-PLAN.md` — the Phase-2 Plan doc. ~430 lines. Eleven landable phases (§1A–§1K) with three Eisa Boss-test gates (§1C, §1F, §1H). Sequencing diagram. Risk register. Out-of-scope section.
>
> ### Six open questions — locked defaults
>
> | # | Question | Locked default |
> |---|---|---|
> | Q1 | CDN URL for Qwen3-1.7B | GitHub Release asset on `eisaShamsi/Constellation` (zero new infrastructure) |
> | Q2 | Source-definition embedding text length | ~150 words per source from the taxonomy doc, embedded as compile-time constant |
> | Q3 | Classify on title + body, or body only | Title + body concatenated (title carries strong signal) |
> | Q4 | Long-note chunking | Tier 1: first 2,000 chars; Tier 2: full note up to 32k tokens |
> | Q5 | 12th `unclassifiable` token | YES — opt-out value users can set in PropertyEditor |
> | Q6 | Auto-reclassify on Tier-2 acquisition | NO — Settings → AI offers a manual "Re-classify all" button; default off |
>
> All six are reversible; surfaced here so future-me knows which choices were Build-cascade defaults vs explicit Boss approvals.
>
> ### Phase sequencing (11 commits, 3 user-test gates)
>
> ```
> §1A Schema  →  §1B Tier-1 classifier  →  ✅§1C Source Review panel  →  §1D PropertyEditor combobox  →  §1E Right-click action  →  ✅§1F Background scan  →  §1G i18n EN+AR  →  ✅§1H Tier-2 + llama.cpp  →  §1I Help docs  →  §1J /simplify + audit  →  §1K Close-out
> ```
>
> Per Plan-Approval-Equals-Build-Approval (CLAUDE.md), once Eisa approves this Plan, Build cascades through §1A–§1K autonomously, pausing only at the three Boss-test gates (§1C / §1F / §1H) and at any architectural surprise.
>
> ### What this means for code on `main`
>
> Still no code. Plan approval is the gate. On approval, §1A is the first commit (schema migration + frontmatter helpers — non-user-visible foundation).
>
> ### Risk register highlights
>
> Most material risks: (a) Tier-1 e5-small classifier accuracy below 65% threshold (gates §1C); (b) llama-cpp-2 Windows build issues (test before §1H); (c) Tier-2 download URL not yet hosted (upload before §1H verification). All mitigations documented in Plan §3.
>
> ### Body still hasn't moved
>
> Same as v1.75 + v1.76 + v1.77: §3 / §4.x / §13 / §17 of the v1.74 BODY are not yet rewritten. Body update lands at §1K close-out (Plan §1K explicitly schedules it).

> **What changed in v1.77** (Sight v5 design contract + MIG-021 Architect drafted; still no code):
>
> The two canonical Sight v5 documents land. They are the design contract every implementation phase reconciles against. **No code shipped.** The implementation cascade is gated on Eisa's Phase-2-sign-off per the /migration discipline.
>
> ### Files added
>
> - `docs/Constellation-Sight-Concept-Paper-v2.0.md` — **the canonical Sight v5 specification**. ~700 lines. Supersedes `Constellation-Sight-Concept-Paper-v1.1.md`, `Constellation-Sight-v3-Concept-Paper-v1.1.md`, and `SIGHT-V3-VISUAL-SPEC.md`. The three obsolete papers stay on disk per SO #6 versioned-filename rule.
> - `lab/reports/MIG-021-EPISTEMIC-CLASSIFIER-ARCHITECT.md` — the Architect doc for the Sources subsystem (frontmatter + classifier + queue UI + PropertyEditor + i18n + 11 phases §1A–§1K).
>
> ### Concept Paper v2.0 — what it nails down
>
> **§0–§2** — what the paper IS / IS NOT, executive summary, the canonical question ("How is my Epistemic Content shaped and/or organized?").
>
> **§3** — the scholarly foundation (Universal Epistemic Content Taxonomy, 5 branches × 11 sources from five civilizational traditions). §3.3 maps Constellation's existing 8-level strata cleanly onto the 5-branch taxonomy condensed by epistemic elevation — the strata-as-radius design is doubly justified.
>
> **§4–§6** — the visual grammar (Suwaidi cream parchment dome, 8 strata bands, calendar rim, Milky Way wash, faint connector lines), the seven modes (R / L / T / C / S / A / **P**), and the four constants (radius = strata, size = maturity, brightness = confidence, red = contested) that hold across every mode.
>
> **§7** — the Sources subsystem: all 11 sources from the taxonomy, multi-source per note, frontmatter `sources:` + `note_meta.sources` mirror, three setting paths (PropertyEditor combobox / Source Review queue / right-click context).
>
> **§8** — the Epistemic Classifier: two-tier strategy (bundled e5-small embedding-similarity Day 1, optional Qwen3-1.7B + llama.cpp download from Settings → AI). Both tiers feed the same review queue. Boot-perf invariant: zero impact (lazy load on first use).
>
> **§9** — *what Sight v5 IS NOT* — the load-bearing boundary section the v1.x papers never wrote. Explicit table comparing Sight v5 against Sky View, Map, OrgChart, Search Hub, Index, 360.3D, Knowledge Health Dashboard, and the withdrawn Multi-Lens.
>
> **§10–§13** — performance budgets, three-MIG phased rollout (MIG-021 → 022 → 023), acceptance criteria, glossary.
>
> ### MIG-021 Architect — what it scopes
>
> Mission: ship the Sources subsystem so Sight v5's mode P has data to visualize. Eleven landable phases (§1A–§1K): schema migration, Tier-1 classifier, Source Review panel, PropertyEditor combobox, right-click context action, background scan job, EN+AR i18n, Tier-2 download + llama.cpp integration, help docs, /simplify checkpoint, audit + close.
>
> 12 invariants (P1–P12). Three design options surfaced; Option C (Hybrid bundling) approved. Migration-path concerns mapped (first-boot, mid-backfill restart, manual override, downgrade, Tier-2 corruption). Six open questions for Eisa to decide before each relevant phase.
>
> ### What this means for code on `main`
>
> No code change. Sight v4 (v4 Svelte components, currently gated as `SIGHT_V4_ENABLED = true`) remains the user-visible Sight on `main`. The Concept Paper v2.0 + MIG-021 Architect are the contract; Phase-2-sign-off opens the build cascade.
>
> ### What's now obsolete on disk (preserved as historical record)
>
> - `docs/Constellation-Sight-Concept-Paper-v1.1.md` — InfraNodus-spined
> - `docs/Constellation-Sight-v3-Concept-Paper-v1.1.md` — per-mode (X, Y, Z) grammar
> - `docs/SIGHT-V3-VISUAL-SPEC.md` — centrality-on-radius
>
> All three reference InfraNodus heritage, which is dropped. They stay in `docs/` per SO #6 versioned-filename rule (older versions stay as historical record). Future doc-cleanup MIG may move them to `docs/historical/` once Sight v5 is Eisa-confirmed-stable across multiple sessions.
>
> ### What the next session must do
>
> 1. **Phase 2 sign-off on MIG-021 Architect.** Eisa reviews `lab/reports/MIG-021-EPISTEMIC-CLASSIFIER-ARCHITECT.md` and either approves as-is or requests revisions. The Architect's six open questions (CDN URL for Qwen3 download, source-definition embedding text, classify on title+body or body only, long-note chunking strategy, "unclassifiable" 12th token, re-classify on Tier-2-acquisition) get answered.
> 2. **Plan doc drafted** — sequences §1A–§1K into landable steps with per-step verification clauses.
> 3. **Build cascade per /migration discipline** — each phase a single commit, each tied to a verification clause, /simplify after every 2-3 phases.
>
> ### Body still hasn't moved
>
> Same as v1.75 + v1.76: §3 / §4.x / §13 / §17 of the v1.74 BODY are not yet rewritten. Body update deferred to v1.78+ when Sight v5 implementation actually ships and the Sight subsystem section can be coherently rewritten in one pass. The v1.75 + v1.76 + v1.77 preambles are the canonical record of state changes since v1.74.

> **What changed in v1.76** (docs-only: System Requirements section added to User Manual + new help topic):
>
> Triggered by Eisa: *"we need to add the minimum PC requirement to operate Constellation, within the Help file and the user manual."* Driven by the v1.75 commitment to the Sight v5 local-LLM classifier — which adds tier-2 (optional download) hardware expectations users must see *before* installing.
>
> The new section is **tiered**: (a) Minimum to run Constellation core; (b) Recommended for comfortable everyday use, large libraries, second-screen; (c) Sight v5 source classifier — bundled tier (no extra requirements) + optional larger classifier (4-core / 4 GB free / 1.5 GB disk / one-time 1.1 GB download). Plain-language tone, no jargon ("any computer made in 2013 or later" instead of "AVX2"). Internet stated as not required for Constellation core; required only for the optional Sight model download.
>
> ### Files added/changed
>
> - `docs/User Manual.md` — new `### System Requirements` section inserted between `## 1. Getting Started` and `### Installation` (canonical English).
> - `docs/help.ar/User Manual.md` — Arabic translation of the same section, in the same position.
> - `docs/help.uConstellation.World/Getting Started/Getting Started.md` — NEW help topic (canonical English; richer than the Manual section, with a "How to check my computer's specs" footer).
> - `docs/help.ar/Getting Started/Getting Started.md` — Arabic translation of the new help topic.
>
> ### Locale rollout
>
> - **EN + AR shipped this commit** (canonical + Arabic). Per the BASIC RULE, I do not invent translations for languages I cannot verify.
> - **13 other locales (de, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh)** queued as a follow-up PJ — Manual section + help-topic translation across all 13. Allocate `PJ-NNN` at the next Pending Jobs bump. Until translated, those locales' User Manuals show the section heading as a `[needs translation]` marker per the existing manual-localization convention (or whatever convention the locale-translation pass uses; verify before applying).
>
> ### What this is NOT
>
> Not an architectural change. Not a code change. The hardware tiers documented for Sight v5 reflect the v1.75 LLM research findings; **Sight v5 has not yet been built**. The doc is forward-looking: it describes what the user will need when v5 ships. The "built-in classifier runs on the same hardware as Constellation core" line stays honest because the bundled-tier strategy (reusing `multilingual-e5-small` for embedding-classification) does not raise the floor.
>
> ### What still hasn't moved
>
> The body sections of v1.74 (§3 Architecture, §4.x CE phase status, §13 Top Principals, §17 Unread list) — same status as v1.75. Body update deferred to v1.77+ when Sight Concept Paper v2.0 ships and body changes can be made coherently in one pass. The v1.75 + v1.76 preambles are the canonical record of state changes since v1.74.

> **What changed in v1.75** (Boss-directed concept-validation pass — **Sight v.next foundation laid; no code shipped**):
>
> The v1.74 directive was: stop implementation, validate the Sight concept itself before any more coding. This session — same calendar day as v1.74 — completed that validation end-to-end. Eisa ratified eleven decisions in sequence; foundation documents landed; visual mockups landed; classifier strategy chose; research agents condensed the LLM landscape into a one-page decision matrix. **No code touched.** Implementation is gated on: Eisa's pick from the LLM research → MIG-021 Architect → Sight Concept Paper v2.0 → /migration cascade.
>
> ### Decisions ratified by Eisa today (in order)
>
> 1. **Delete `lenses.rs::apply_lens`.** Multi-Lens (CE Phase 9) withdrawn. Cleanup MIG queued. (Closes the open question from `project_lenses_apply_lens_dead_code.md`, 2026-04-27.)
> 2. **Sight's canonical answer is reframed.** Old: "what hidden analytical patterns exist in my thinking?" (InfraNodus heritage — centrality / Louvain / structural gaps / universe health). **New: "How is my Epistemic Content shaped and/or organized?"** Eisa: *"forget about the InfraNodus."*
> 3. **Sight scope vs 360.3D.** Eisa: *"The focus of the 360.3D is the Note, while Sight is the whole universe."* Mutually exclusive. This is the "what Sight is NOT" line the original three Sight Concept Papers never wrote.
> 4. **V2 was insufficient because users couldn't read it without network-science literacy.** Eisa: *"If future Constellation users don't understand it or think it is difficult, then its existence is unnecessary."* Codifies the 2026-04-13 directive (*"simplicity should come from understanding what you see at first sight, NOT to raise more questions"*) as a hard constraint on Sight v.next. First-time-user understanding within ~5 seconds.
> 5. **Visual direction approved: Mock B (night sky, re-anchored to epistemic dimensions).** Specifically **Mock B1 (single-mode + toggle bar)** ships in production. Mock B2 (side-by-side comparison) reserved for help docs only. Mock A (dashboard) rejected.
> 6. **Strata stays the constant radius across all modes.** Only azimuth changes per mode. Star size = maturity, brightness = confidence, red dot = contested — all constant across modes. This REVOKES the v3 Visual Spec §1 "per-mode (X, Y, Z) grammar" where each mode declared its own radius/azimuth/magnitude.
> 7. **Universal Epistemic Content Taxonomy adopted as the scholarly foundation** of the Sight Concept Paper v.next. Eisa-authored taxonomy: 5 branches × 11 sources, distilled from five civilizational epistemological traditions (Greek + Western analytic; Arabic-Islamic Sunni *kalām* / *uṣūl al-fiqh* / *falsafa*; Indian *pramāṇa-vāda*; classical Chinese Mohist / Confucian / Daoist; Persian-Islamic Ishrāqī) plus Jewish / Tibetan Buddhist / African / Mesoamerican supplementary. Bilingual EN+AR; 13 other locales follow the standard $t() fallback chain.
> 8. **Strata IS the Constellation projection of the taxonomy.** The existing 8-level strata field (L1 Datum → L8 Worldview, populated across all 7,636 trial-universe notes) maps cleanly onto the 5-branch taxonomy condensed by epistemic elevation: L1=Branch 1+2.3, L2=Branch 3.5, L3=Branch 4.5, L4=Branch 5.1, L5=Branch 5.2, L6=Branches 5.3-5.5, L7=Branches 5.6-5.7, L8=Branch 5.8. Sight's strata-as-radius design is doubly justified — by Constellation's native taxonomy AND by the cross-civilizational scholarly tradition the new taxonomy synthesizes.
> 9. **UI stays plain.** No civilizational labels in the dock (no "Branch 2 Symbolic Entities" buttons). UI uses Constellation-native vocabulary (Strata / Time / Regions / Link Types / Confidence / Stages / Acts / **Provenance**). Taxonomy is invoked in tooltips, help docs, and the Concept Paper — not in the user-facing chrome.
> 10. **Sources tracked Day 1 — NEW Sight mode P (Provenance).** Six Sources sub-decisions ratified: (a) all 11 sources ship from Day 1; (b) multi-source per note, ranked primary + secondary; (c) build an auto-classifier tool that reads each note and proposes sources for user approval (NOT inferred-and-silently-written; NOT defaulted to "perception"); (d) UI = PropertyEditor combobox (manual setting) + new "Source Review" sidebar panel (queue-based approval) + right-click "Suggest sources for this note" (on-demand single-note classification); (e) frontmatter `sources:` (canonical) + `note_meta.sources` SQLite mirror (matches MIG-014 Strata/Maturity/Stage pattern); (f) all 15 locales, locale-driven via $t().
> 11. **Classifier strategy: local LLM** (NOT rule-based, NOT cloud API). 100% local CPU inference, multilingual, classification-only. Sub-decisions (model / inference engine / bundling) researched in-session via three parallel research agents; results in `lab/reports/MIG-021-LOCAL-LLM-RESEARCH.md`. Top recommendations (still pending Eisa's pick):
>    - **LLM**: Qwen3-1.7B Q4_K_M (~1.1 GB, Apache 2.0, first-class Arabic, 25–45 tok/s CPU). Runner-up: Gemma 4 E2B if Arabic eval favors it. **Disqualified**: Llama 3.2 (Arabic not supported), Phi (English-dominant), Gemma 3 (license risk).
>    - **Inference engine**: llama.cpp via `llama-cpp-2` Rust bindings. Critical reason: **GBNF grammar-constrained decoding** guarantees valid JSON output for the 11-source classification — eliminates parsing failures. Keep ORT for embeddings; the two engines coexist behind a single `inference::` Rust module.
>    - **Bundling**: hybrid — bundle a small ~100–250 MB classifier in the .exe (Sight works Day 1, no network) + optional Settings → AI download for the larger ~1.5 GB model (better Arabic accuracy for power users). Smart Connections precedent (2M+ Obsidian installs).
>
> ### Net visual artifacts (in `docs/`)
>
> - `Sight-vNext-MockA-Dashboard.svg` — Option A dashboard (REJECTED; kept as decision record)
> - `Sight-vNext-MockB-Metaphor.svg` — Option B base, single dome no toggle (kept as decision record)
> - `Sight-vNext-MockB1-Toggle.svg` — **APPROVED for production**: single dome + toggle bar with all 6 mode buttons visible
> - `Sight-vNext-MockB2-Compare.svg` — kept for help docs as the mode-switch teaching diagram (Time vs Regions side-by-side, demonstrating wedge re-slicing)
>
> ### Net foundation references (in `docs/`)
>
> - `epistemic-content-EN.md` — Eisa's comparative civilizational survey (English, 449 lines). The intellectual case.
> - `epistemic-content-AR.md` — Eisa's comparative civilizational survey (Arabic, 165 lines). The bilingual canonical reference.
> - `epistemic-content-taxonomy.md` — formal two-axis taxonomy (5 branches × 11 sources) with bilingual labels and cross-civilizational anchors. The reference Sight is built against.
> - `epistemic-content-taxonomy-chart.html` — interactive 5-level chart implementation, self-contained, bilingual, zero dependencies. Will ship in `docs/help.uConstellation.World/Constellation Sight/` when help docs are written.
>
> ### Research deliverable
>
> - `lab/reports/MIG-021-LOCAL-LLM-RESEARCH.md` — three parallel research agents condensed into one decision-matrix doc. Eisa picks from the three sub-decisions; MIG-021 Architect drafted; Concept Paper v2.0 drafted.
>
> ### What this means for code on `main`
>
> - All three existing Sight Concept Papers (`Constellation-Sight-Concept-Paper-v1.1.md`, `Constellation-Sight-v3-Concept-Paper-v1.1.md`, `SIGHT-V3-VISUAL-SPEC.md`) become **obsolete** the moment the new Concept Paper v2.0 lands. They stay on disk as historical record (per SO #6 versioned-filename rule).
> - `src-tauri/src/sight.rs` (419 lines, Brandes betweenness + Louvain communities + structural gaps + universe health IPCs) is **orphaned** by the InfraNodus drop. Likely deletion candidate alongside `lenses.rs` in the cleanup MIG.
> - `src/lib/sight/v4/SightV4.svelte` and `SightV4SidePanel.svelte` are **paused on disk**. `SIGHT_V4_ENABLED` stays `true` so users currently on `main` see the v4 build until Sight v.next ships and hot-replaces it. v3 helper modules (`modes.ts`, `polar.ts`, `regions.ts`, `library-colors.ts`) are likely partly reusable — to be evaluated in MIG-021 Architect.
>
> ### What previous orientation versions said about Sight that NO LONGER HOLDS
>
> - "InfraNodus heritage as Sight's analytical spine" — REVOKED.
> - "Six modes R / L / T / C / S / A" — now SEVEN (P added for Provenance).
> - "Each mode declares its own (X, Y, Z) per the v3 visual spec" — REVOKED. Strata is the constant radius across all modes; only azimuth changes.
> - "PJ-035 content-similarity TF-IDF Milky Way as InfraNodus's *latent connections*" — REANCHORED. Milky Way still represents content-similarity density, but the framing is "shared themes" not "InfraNodus latent edges."
> - "PJ-036 layer peeling (magnitude slider hides top-centrality stars)" — REVISED. With strata-as-radius, peeling top-magnitude (largest) stars peels by **maturity**, not centrality. Reconcile in Concept Paper v2.0.
> - "PJ-037 Map↔Sight integration" — STAYS REJECTED (Eisa's 2026-05-07 call holds).
> - "v3 Visual Spec §1 per-mode (X, Y, Z) grammar; only color invariant" — REVOKED. New invariants: strata = radius (constant), maturity = size (constant), confidence = brightness (constant), red = contested (constant); only azimuth varies per mode.
>
> ### What the next session must do
>
> 1. Read `lab/reports/MIG-021-LOCAL-LLM-RESEARCH.md` — the three-decision matrix.
> 2. Get Eisa's pick on (a) LLM model, (b) inference engine, (c) bundling strategy. Top recommendations exist; Eisa may "agree with all three" or override per-decision.
> 3. Once decided, draft `docs/Constellation-Sight-Concept-Paper-v2.0.md` (taxonomy-spined, 7 modes, strata-as-radius invariant, "what Sight is NOT" section, sources subsystem) AND `lab/reports/MIG-021-EPISTEMIC-CLASSIFIER-ARCHITECT.md` in parallel.
> 4. Then /migration discipline takes over.
>
> ### Memory written this session (in `~/.claude/projects/.../memory/`)
>
> - `project_lenses_apply_lens_dead_code.md` — UPDATED: decision DELETE
> - `project_sight_canonical_answer.md` — NEW (Sight's reframed canonical answer)
> - `project_sight_360_scope_orthogonal.md` — NEW (Sight = universe, 360.3D = note)
> - `project_sight_taxonomy_foundation.md` — NEW (Universal Epistemic Content Taxonomy as scholarly spine)
> - `project_sight_classifier_local_llm.md` — NEW (six Sources decisions + classifier strategy = local LLM + sub-decisions open)
>
> ### What this orientation file does NOT yet update
>
> The §3 (Architecture surface inventory), §4.x (CE phase status, Sight subsystem section), §13 (Top Principals), and §17 (Unread list) sections of the v1.74 BODY are NOT yet rewritten to reflect today's decisions. **Body update of those sections is deferred to v1.76**, which lands when Sight Concept Paper v2.0 ships and the body changes can be made coherently in one pass. The v1.75 preamble above is the canonical record of what changed today; the body of this file describes the prior state truthfully.

> **What changed in v1.74** (Boss directive — **STOP implementation, validate Sight concept**):
>
> Eisa tested the v4 build (commit `29ce010`). Close button still didn't work (Stage 1 failed). A fix was applied (commit `3a977c8` — raw DOM event handlers) but before it could be tested, Eisa pivoted:
>
> > *"I want to start fresh. I want you to discard what has been developed so far. I want you to go to the basics. Let's validate and confirm the 'Constellation Sight Concept'."*
>
> **This is NOT about fixing the close button again.** This is about questioning whether the Sight concept itself is right before any more implementation work.
>
> **Current state of Sight on `main`**:
> - `SIGHT_V2_ENABLED = false` — disabled, code preserved on disk as known-good fallback
> - `SIGHT_V3_ENABLED = false` — disabled after 13 failed close-button iterations, code preserved
> - `SIGHT_V4_ENABLED = true` — current code on main, but Boss wants concept validation before any more coding
> - Two v4 commits on main: `29ce010` (clean-slate pivot) + `3a977c8` (raw DOM fix, NOT tested by Boss)
>
> **What the next session must do**:
> 1. Read the three Sight concept documents (listed below) — they define what Sight IS
> 2. Present the concept to Eisa for validation in plain language
> 3. Get Eisa's confirmation or revision of each aspect BEFORE any implementation
> 4. Do NOT proceed with any code changes until concept is validated
>
> **The three Sight concept documents**:
> - `docs/Constellation-Sight-Concept-Paper-v1.1.md` — analytical foundation (what Sight computes: centrality, communities, gaps, health, three edge types, six design principles)
> - `docs/Constellation-Sight-v3-Concept-Paper-v1.1.md` — visual + interaction spec (star chart aesthetic, projection math, interactivity, phased rollout)
> - `docs/SIGHT-V3-VISUAL-SPEC.md` — per-mode (X, Y, Z) grammar, color palette, layout invariants
>
> **Reusable assets (pure functions, no DOM/rendering dependencies)**:
> - `src/lib/sight/v3/modes.ts` (413 lines) — per-mode (X, Y, Z) position computation
> - `src/lib/sight/v3/polar.ts` (143 lines) — polar coordinate utilities
> - `src/lib/sight/v3/regions.ts` (244 lines) — region/wedge layout
> - `src/lib/sight/v3/library-colors.ts` (85 lines) — deterministic library color assignment
>
> **Rust analytical pipeline (all working, all reusable regardless of frontend approach)**:
> - `constellation_sight_centrality` — Brandes' betweenness centrality
> - `constellation_sight_communities` — Louvain community detection
> - `constellation_sight_structural_gaps` — inter-community gap identification
> - `constellation_sight_universe_health` — modularity + dominance + entropy + connectivity

> **What changed in v1.73** (MIG-019 — **Sight v3 → v4 clean-slate pivot**):
>
> **13 close-button failures, one root cause**: SightV3.svelte mounted as `position: fixed; inset: 0; z-index: 1000` — a full-screen overlay OUTSIDE normal DOM flow. D3-zoom's event listeners on the viewport-filling canvas consumed all pointer events before any button could receive them. Every z-index escalation, raw addEventListener, Svelte bypass, external button — all failed because the architecture was wrong.
>
> **The fix is architectural, not incremental**: SkyView's close button has worked since day one. Why? It renders inside `.content-area` as a normal flex child (`<div class="star-fullscreen">` → `<div class="star-header">` with close button → `<GraphMindView>`). The close button directly sets `showSkyView = false` in the parent. No callback crossing component boundaries. No `position: fixed`. No z-index wars.
>
> **Sight v4 adopts SkyView's exact mount pattern**:
> - `SightV4.svelte` is a **plain flex child** (NO `position: fixed`, NO z-index overlay)
> - Mounted inside `+layout.svelte`'s `.content-area` → `.star-fullscreen` → `.star-header` + `<SightV4>`
> - Close button lives in `+layout.svelte`'s `.star-header` row: `<button class="star-close" onclick={() => sightV4Active = false}>×</button>`
> - Same Canvas 2D + D3-zoom draw pipeline from v3's §2G.3q migration
> - All v3 helper modules reused: `modes.ts`, `polar.ts`, `regions.ts`, `library-colors.ts`
> - `SightV3.svelte` preserved on disk but disabled (`SIGHT_V3_ENABLED = false`)
>
> **Files changed**: `src/lib/sight/v4/SightV4.svelte` (new), `src/lib/sight/v4/SightV4SidePanel.svelte` (new, copy of v3), `src/lib/sight/engine.ts` (v3 disabled, v4 enabled), `src/routes/+layout.svelte` (v4 mount inside content-area). Concept Paper + Visual Spec updated.

**Version 1.72 | 2026-05-08**

> **What changed in v1.72** (MIG-019 §2G.3q — **Pixi.js v8 → D3 + Canvas 2D migration**):
>
> **Root cause identified**: Pixi.js v8's EventSystem registers `document.addEventListener('pointermove/pointerup', ..., true)` capture-phase listeners that globally intercept pointer events before any DOM element receives them — even buttons completely outside the Pixi component. This was the actual cause of the 11-iteration close-button failure (CSS `:hover` still worked because hover is browser rendering engine, not JS events).
>
> **Solution**: Complete replacement of Pixi.js v8 with Canvas 2D + D3-zoom in `SightV3.svelte`. Canvas 2D has zero global event side effects. D3 v7 was already a dependency (used by SkyView). The migration touches only the rendering pipeline — all data flow, layout computation, and interaction logic stays the same. pixi.js remains in package.json for Constellation Map (graphEngine.ts).
>
> **Architecture**: Immediate-mode draw() pipeline — clear canvas → apply d3-zoom transform → draw layers (Milky Way → territories → edges → stars → focus overlay → rim) → sync HTML overlay CSS transform. d3.zoom() handles wheel zoom, drag pan, touch pinch with passive:false. DPR-aware: canvas attribute size = CSS size × devicePixelRatio.

> **What changed in v1.71** (same day as v1.70; MIG-019 §2G.3o — structural fix: root → flex column with header + body siblings; documented Svelte 5 delegation root cause):
>
> Eisa, after the §2G.3n commit but before testing the build: *"I am really wondering why you are not able to fix a simple task, like a 'Close' function!! Is it that hard? How many attempts so far?"* — and then: *"Go and do your homework. Dig for the simple, proven right solution."*
>
> Iteration #8 (§2G.3n) had **claimed** to "adopt the v2 pattern wholesale" but in fact only copied the markup snippet (a `<div class="sight-v3-header">` with a flex spacer + `<button onclick={() => onClose?.()}>×</button>`). The CSS still had `.sight-v3-header { position: absolute; top: 0; left: 0; right: 0; height: 44px; z-index: 100; pointer-events: none; }` and `.sight-v3-close { pointer-events: auto; }` — meaning the header strip was an absolute overlay over the canvas, with the close button trying to thread the needle via `pointer-events`. v2's actual working pattern is fundamentally different: `.sight2-root { display: flex; flex-direction: column; }` with `.sight2-header` as a real flex row (no positioning at all) and `.sight2-body { flex: 1; position: relative; }` containing the canvas. **Header and canvas are SIBLINGS in two non-overlapping flex rows** — they never share absolute-overlay space.
>
> ### Documented root cause (research agent + GitHub issues, citations below)
>
> This is **Svelte 5 issues #15343 and #13213**: Svelte 5 delegates `onclick={fn}` handlers to `<body>` and relies on the click event bubbling all the way up. When a button is in an absolute overlay sibling of a canvas wrapper that ALSO has `onclick` + `onpointerdown` + `onpointermove` + `onpointerup` + `ondblclick` handlers (all delegated to body), the canvas's handlers and **Pixi v8's document-level capture-phase pointer listener** can interfere with click delivery to the button. Hover (CSS `:hover`) still fires because hover is NOT delegated. That exactly matches the v3 symptom across eight iterations — gold-on-hover but no click.
>
> This explains why every prior defense failed: z-index escalation 100 → 1000 → 9999 didn't matter because the click never reached the button's delegated handler at body. `bind:this` + raw `addEventListener('click', ...)` SHOULD have worked since it bypasses Svelte's delegation entirely — but the Svelte $state ref had subtle timing issues that caused some attaches to silently no-op (the "if (closeBtn)" guard skipped the attach when ref hadn't resolved yet). Defensive `onpointerup` didn't help because pointerup also bubbled through the canvas-event-handler-laden body delegate. The bug was structural, and only structural separation eliminates the trigger.
>
> ### What §2G.3o ships
>
> Single commit, single file (`src/lib/sight/v3/SightV3.svelte`), ~50 LOC delta:
>
> - **`.sight-v3-root` is now `display: flex; flex-direction: column`** — matching v2's `.sight2-root` exactly.
>
> - **`.sight-v3-header` is now a regular flex row** — `flex-shrink: 0; height: 44px; display: flex; align-items: center; padding: 0 16px;` plus a translucent cream background and 1 px bottom border. **No `position: absolute`**, no `z-index`, no `pointer-events`. The close button inside is a plain inline flex item (no `pointer-events: auto`, no manual z-index).
>
> - **NEW `.sight-v3-body` row** — `flex: 1; position: relative; overflow: hidden;` — takes the remaining height below the header. Acts as the positioning ancestor for the canvas + Reset View button + overlays wrapper + tooltip (which used to be parented to root). Canvas is still `position: absolute; inset: 0` but now anchored to body, not root, so it never extends into header territory.
>
> - **Markup restructured** to wrap canvas + reset-view + overlays-wrapper + tooltip in `<div class="sight-v3-body">`. Side panel stays at root level (it's `position: absolute; height: 100vh`, anchors to root, slides over both header and body when active).
>
> - **All `pointer-events: none` and `pointer-events: auto` thread-the-needle CSS removed** from header / header-spacer / close-button. They were trying to compensate for an architectural mistake; with the structural separation, they're unnecessary.
>
> - **`onclick={() => onClose?.()}` retained as plain Svelte 5** (no `on:click` legacy directive, no `onclickcapture`, no manual `addEventListener`). The structural fix removes the trigger condition for Svelte #15343 entirely; plain `onclick` is sufficient. v2 and SkyView both use plain `onclick` and have shipped working in production for months.
>
> ### Sources
>
> - [Svelte #15343 — `stopPropagation` on parent breaks all child `onevent` handlers](https://github.com/sveltejs/svelte/issues/15343)
> - [Svelte #13213 — Event handlers fail with stopPropagation between body and target](https://github.com/sveltejs/svelte/issues/13213)
> - [Pixi #10911 — Events propagating from HTML overlays to Pixi viewport](https://github.com/pixijs/pixijs/discussions/10911)
> - [Svelte legacy `on:` directive docs (still supported)](https://svelte.dev/docs/svelte/legacy-on)
> - [MDN — `pointer-events`](https://developer.mozilla.org/en-US/docs/Web/CSS/pointer-events) (confirms `pointer-events: auto` on child works at the CSS layer — but that's never the bug here; the bug is in JavaScript event delegation, not CSS hit-testing)
> - `src/lib/components/ConstellationSight2.svelte:1041-1064, 1269-1304` — v2's working pattern, line numbers verified against the file
> - `src/lib/components/SkyView.svelte:961-965, 1162-1200` — SkyView's working pattern in `controls-panel`
>
> ### What this means for the rest of v3
>
> The other §2G.3 issues from the §2G.3m Boss test (rim numbers drift, connected-notes invisibility) were correctly addressed in §2G.3n (Pixi `Text` in `calendarRimContainer`; `sidePanelConnectedNotes` $derived). Those fixes stand. §2G.3o is purely the close-button structural fix — additive on the close-button axis, no other behaviors changed.
>
> **Type-checked clean** (only the pre-existing `LinkLifecycle dedupe` error from `project_link_lifecycle_dedupe_fix.md`, deferred until post-CE). orientation v1.70 → v1.71 inline.

**Version 1.70 | 2026-05-08**

> **What changed in v1.70** (same day as v1.69; MIG-019 §2G.3n — adopt the v2 working pattern wholesale):
>
> Eisa's §2G.3m Boss test, three remaining issues:
> 1. **Close button STILL not working** (8th iteration; visual hover fires, click handler never does).
> 2. **Library numbers drift on zoom** ("Fix it, don't patch it" — patches accumulated, root cause persisted).
> 3. **Connected-notes titles invisible** — side panel showed only counts; user wanted to see WHICH notes are linked.
>
> Eisa explicitly directed: *"I want you to check how we manage to do it right in the SV. It is already working there."* and *"Fix it, don't patch it."*
>
> Read `ConstellationSight2.svelte` (the v2 dashboard's working close button) and `SkyView.svelte`. Both follow the same pattern: a thin **flex header bar** at the top of the panel containing the close button as an **inline flex item** — NOT `position: absolute` on the button, NOT bound via `addEventListener` in `$effect`, NOT layered with z-index 9999. Eight rounds of v3 attempts (1000 → 9999 z-index, raw addEventListener, defensive pointerup, $effect-based binding, etc.) all failed because they fought the layout. v2 just used inline flex with `onclick={() => onClose?.()}` and worked the first time.
>
> Shipped in §2G.3n:
>
> - **Close button — v2 pattern adopted wholesale.** New `.sight-v3-header` element: thin 44 px strip at top of `.sight-v3-root` with `pointer-events: none` (clicks pass through to the canvas) holding a flex spacer + the close button. The button itself has `pointer-events: auto` and a simple `onclick={() => onClose?.()}` — the same one-liner v2 has. No bind:this, no addEventListener, no $effect. Removed all prior z-index escalation, defensive pointerup, and $effect-based attachment. The header strip is `position: absolute; top:0; left:0; right:0` because it overlays the canvas, but the BUTTON itself is an ordinary inline flex child, not absolute-positioned. This is the canonical pattern that has shipped in v2 since day one.
>
> - **Library rim numbers — Pixi Text in `calendarRimContainer`** (single transform pipeline). Was: HTML `<span class="sight-v3-rim-number">` overlay scaled via CSS `transform: translate3d() scale3d()` while Pixi rim arcs scaled via `chartContainer.scale.set()`. Two pipelines, sub-pixel divergence accumulating with zoom — patches like `translate3d` reduced but never eliminated drift. Fix: removed the HTML rim-numbers loop entirely; replaced with Pixi `Text` objects rendered inside `drawRegionRim()` and added to `calendarRimContainer` (a child of `chartContainer`). The text now scales / pans / hit-tests in lockstep with the rim arcs themselves, since they share the exact same Pixi transform matrix. Mathematically impossible to drift. CSS `.sight-v3-rim-number` rule removed (dead code). This is the "fix, don't patch" the user demanded — single source of geometric truth.
>
> - **Connected-notes list in side panel.** New `sidePanelConnectedNotes` `$derived.by` in `SightV3.svelte`: scans `resolvedEdges` for the selected note, walks 1-hop neighbours (cap 50, matching the focus-overlay edge cap so hub notes don't overflow), resolves each to `{ path, title, libraryName, colorCss }` using `pathToTitle` / `pathToLibrary` / `regionLayout.pathToWedge` / `libraryColors`. New `connectedNotes` prop + `onConnectedClick` callback on `SightV3SidePanel`. Side panel renders a clickable list (color dot + title + library name) under a "Connected notes (N)" header. Click a row → `onConnectedClick(path)` recentres the side panel on that neighbour (Sight stays open; no editor switch needed for graph exploration). Uses `dir="auto"` on the row + library-name span so RTL note titles flow correctly.
>
> - **Wedge math + library coloring + Louvain communities + Suwaidi cream theme + selection ring + ink edges + 3D transforms** all preserved from §2G.3m — this commit is purely additive on the close-button, rim-numbers, and connected-notes axes.
>
> **Type-checked clean** (only the pre-existing `LinkLifecycle dedupe` error from `project_link_lifecycle_dedupe_fix.md`, deferred until post-CE). orientation v1.69 → v1.70 inline.

**Version 1.69 | 2026-05-08**

> **What changed in v1.69** (same day as v1.68; MIG-019 §2G.3m — fix-up batch after §2G.3l):
>
> Eisa's §2G.3l Boss test: zoom + pan + lens architecture works. Four remaining issues:
> 1. Close button **STILL not firing** (7th iteration).
> 2. Selection ring is too big (doesn't match node size).
> 3. Edge color (dark burnt-amber) still too low contrast on cream.
> 4. Library numbers slightly offset from rim circles when zoomed.
>
> Shipped in §2G.3m:
>
> - **Close button**: added `onpointerup` as a defensive second event path. Both `onclick` and `onpointerup` invoke the same `(e) => { e.stopPropagation(); onClose(); }` handler. After 7 rounds of failing `onclick` despite it being the canonical Svelte 5 pattern, we stop guessing and register on multiple events so AT LEAST one fires. pointerup fires before click in the pointer event sequence and is rarely suppressed.
> - **Esc simplified to single-press always-close**. The cascade (clear → reset → close) felt clever but added friction when the user just wants out. Reset View button is the canonical view-reset path; click empty space clears selection. Esc is now a guaranteed escape hatch since the (×) button has had intermittent failures.
> - **Selection ring matches node size**: extracted a single `actualNodeRadius()` helper used by both `drawStars` and `drawFocusOverlay`. Was: drawStars used one formula (`MIN + sizeNorm * range`, range 1.2-4.0 px), drawFocusOverlay read `focusScreen.r` which is the position pseudo-radius (0.7-8.4 px). Brightest stars had rings 2.35× their visible diameter. Now both call the same function — ring sits exactly 1 px outside the node's drawn edge.
> - **Edges → INK at alpha 0.7**: switched from dark burnt-amber `#6b4f0d` (still too low contrast on cream) to ink `#1a1a1a @ alpha 0.7, width 1.0`. Strong contrast against the parchment.
> - **Rim alignment via translate3d/scale3d**: the CSS overlays-wrapper now uses `translate3d(panX, panY, 0) scale3d(zoom, zoom, 1)` instead of the 2D `translate() scale()`. The 3D versions are GPU-precise (matrix-based) and avoid the slight sub-pixel rounding differences with Pixi's matrix that produced the "library numbers offset on zoom" artifact.
>
> **Type-checked clean.** orientation v1.68 → v1.69 inline.

**Version 1.68 | 2026-05-08**

> **What changed in v1.68** (same day as v1.67; MIG-019 §2G.3l — evidence-backed redesign after a 5-agent audit):
>
> Eisa (after §2G.3k still didn't work): *"Not working. Enough wasting my time. Bring in the audit agents and conduct the necessary research to resolve this for good."*
>
> Spawned three codebase Explore agents (close button, freeze, lens architecture) and two web-research agents (Svelte 5 idioms, Pixi v8 zoom + Tauri/WebView2 quirks). All five converged on the same root causes with citations. The patch loop ended.
>
> ### Audit findings (with citations)
>
> 1. **Close button never had a working handler.** §2G.3i removed `onclick={...}` from the markup when switching to `bind:this` + `addEventListener` via `$effect`. The $effect's handler reference was recreated per cycle; the cleanup-with-stale-ref pattern silently dropped listeners. The Svelte 5 docs explicitly use `<button bind:this={x} onclick={() => x.focus()}>` together — the canonical pattern is `onclick={fn}`. ([Svelte 5 migration guide](https://svelte.dev/docs/svelte/v5-migration-guide), [bind: docs](https://svelte.dev/docs/svelte/bind), closed issue #10435)
>
> 2. **CSS-scaling the `<canvas>` element blurs it.** §2G.3i wrapped the canvas + overlays in a single `.sight-v3-zoom-wrapper` and CSS-transformed it. Per MDN: *"The Canvas is rendering to a bitmap of one size then scaling the bitmap to fit the CSS dimensions."* The canonical Pixi pattern (Steve Ruiz's "Creating a Zoom UI"; pixi-viewport library) is `container.scale.set(zoom)` on a Pixi Container, NOT CSS-scaling the canvas. ([MDN: Optimizing canvas](https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API/Tutorial/Optimizing_canvas), [Steve Ruiz](https://www.steveruiz.me/posts/zoom-ui), [pixi-viewport](https://github.com/pixijs-userland/pixi-viewport))
>
> 3. **`transform` does NOT trigger ResizeObserver** — confirmed by the [W3C spec](https://drafts.csswg.org/resize-observer/): *"Observations will not be triggered by CSS transforms."* The team's earlier "ResizeObserver feedback loop" theory was wrong.
>
> 4. **Wheel `preventDefault` requires explicit `passive: false`** in modern Chromium / WebView2. Already done in §2G.3h.
>
> ### What §2G.3l ships
>
> A single commit reverting the lens-CSS-on-canvas approach and applying the right architecture:
>
> - **Close button** uses `onclick={(e) => { e.stopPropagation(); onClose(); }}` directly. `bind:this`, `closeBtn` $state ref, `$effect`-based addEventListener — all removed.
>
> - **Pixi-native chart zoom restored**. `chartContainer.scale.set(chartZoom)` + `chartContainer.position.set(cx + panX, cy + panY)` + `chartContainer.pivot.set(cx, cy)` is back. Stars, edges, rim circles, Milky Way scale at full Pixi resolution — sharp at any zoom.
>
> - **HTML overlays only** scale via CSS. New `.sight-v3-overlays-wrapper` (separate from the canvas) wraps rim numbers + Universe Health + Universe-name + legend. CSS `transform: translate(panX, panY) scale(chartZoom)` with origin at dome center, in lockstep with the Pixi container's transform. `pointer-events: none` so clicks pass through to the canvas; legend re-enables `pointer-events: auto` for its native title tooltips.
>
> - **Canvas restored to `position: absolute; inset: 0`** as a direct child of `.sight-v3-root`. No flex layout, no nested wrapper. Pixi `resizeTo: canvasContainer` resizes the canvas to fill the screen reliably.
>
> - **Hit-testing** inverse-transform: `internal_x = (mouse_x - cx - panX) / chartZoom + cx`. Pixi-native chart zoom means `getBoundingClientRect` doesn't absorb the scale, so we apply the inverse explicitly. Verified algebraically.
>
> Net: lens-style zoom with sharp Pixi rendering, working close button, accurate hit-testing. Architecture matches the canonical patterns from production canvas tools (tldraw / Excalidraw / pixi-viewport).
>
> **Type-checked clean.** orientation v1.67 → v1.68 inline.

**Version 1.67 | 2026-05-08**

> **What changed in v1.67** (same day as v1.66; MIG-019 §2G.3k — wheel-zoom freeze fix):
>
> Eisa's §2G.3j Boss test: dome rendered correctly again, but *"when I try to zoom in using the mouse wheel, the app freezes. It is non-responsive."*
>
> Two compounding causes:
>
> 1. **A redundant `$effect` watching `chartZoom + chartPanX + chartPanY`.** Added in §2G.3h as belt-and-suspenders, it fired on every wheel event AND every wheel handler also called `syncZoomTransform` directly. With high-DPI mice firing 100+ wheel events per second, this scheduled 200+ Svelte reactive updates per second → main thread saturated.
>
> 2. **No rate limiting on the wheel/drag handlers.** Every wheel notch immediately wrote to multiple `$state` variables and synchronously updated the DOM transform attribute. Smooth-scroll wheels fire at 60-200 Hz; the cumulative DOM work blocked the main thread.
>
> Shipped in §2G.3k:
>
> - **Removed the redundant `$effect`.** Every wheel / drag / reset / Esc / resize handler already calls `syncZoomTransform()` synchronously. The $effect was duplicate work, not a safety net.
>
> - **rAF-throttled both wheel and drag pan.** The state (`chartZoom`, `chartPanX`, `chartPanY`) updates immediately on every event so the math is responsive, but the DOM-write `syncZoomTransform()` is coalesced to at most one per animation frame via `requestAnimationFrame`. Single shared `zoomFrame` token prevents double-scheduling.
>
> Net effect: zoom remains visually smooth (rAF-paced like the browser's compositor) but the main thread is no longer flooded.
>
> **Type-checked clean.** orientation v1.66 → v1.67 inline.

**Version 1.66 | 2026-05-08**

> **What changed in v1.66** (same day as v1.65; MIG-019 §2G.3j — fixes the §2G.3i lens-architecture regressions):
>
> Eisa's §2G.3i Boss test report: *"I will assume that this is a JOKE! What happened to my Sight? Even the mouse wheel doesn't work. Nothing is working, even the close button. Enough patching."*
>
> Two regressions from §2G.3i (the lens-architecture restructure):
>
> 1. **Canvas sizing broke.** The new `.sight-v3-zoom-wrapper` was added with `flex: 1; position: relative` but NOT `display: flex`. The canvas inside still had `flex: 1` from before, but it's a no-op when the parent isn't a flex container. So the canvas sized to its content (Pixi default ~800×800) instead of filling the wrapper. That's why the dome rendered tiny AND why wheel events didn't fire (the canvas only covered a small area of the screen).
>
> 2. **Close button binding still failed.** The §2G.3i fix attached `closeBtn.addEventListener('click', ...)` inside `onMount`, gated by `if (closeBtn)`. But Svelte 5's `bind:this` to a `$state`-typed ref has a subtle timing issue: in some component-mount paths, the ref is still null when `onMount` runs. The `if (closeBtn)` check silently skipped the attach, and the button remained inert despite four rounds of "fixes."
>
> Shipped in §2G.3j:
>
> - **`.sight-v3-zoom-wrapper` is now `display: flex; align-items: stretch`**. The canvas's `flex: 1` now works, and the canvas fills the entire wrapper (which fills the entire screen via its own `flex: 1` on `.sight-v3-root`). Dome renders full-size again.
>
> - **Close-button binding via `$effect`**. Reactive on `closeBtn` (still `$state`). When `bind:this` resolves and assigns the DOM node, the `$effect` re-runs and registers the `click` + `pointerup` handlers. Bulletproof against any timing path. Cleanup function returned for orderly removal on unmount.
>
> - **Close-button z-index 1000 → 9999**. No element in the page should sit above 9999 — guaranteed topmost.
>
> **Type-checked clean.** orientation v1.65 → v1.66 inline.

**Version 1.65 | 2026-05-08**

> **What changed in v1.65** (same day as v1.64; MIG-019 §2G.3i — lens-style zoom architecture + addEventListener close + always-visible reset + darker edges + tooltip outside lens):
>
> Eisa's seventh-round Boss test on §2G.3h surfaced a fundamental architectural mismatch: the close button STILL didn't fire (now four rounds of `onclick={...}` attempts — hover style works, click never does), and his mental model for zoom was different from what was implemented. Direct quote (2026-05-08):
>
> > *"To zoom in and out with the mouse wheel, imagine the Sight page as a regular page; all its components and elements should be locked in place. The mouse wheel will act as a lens. When zooming in, you move the lens closer to the page surface. It means every component/element will be outside the lens's coverage if it is not within it."*
>
> The §2G.3g–3h architecture had the chart zooming while the Universe Health, Universe-name, and legend stayed screen-anchored. Eisa wanted everything-that's-on-the-page to scale together. Lens-style.
>
> Shipped in §2G.3i:
>
> - **Lens-style zoom architecture**. New `.sight-v3-zoom-wrapper` element wraps canvas + rim numbers + Universe Health + Universe-name + legend. A SINGLE CSS transform (`translate(panX, panY) scale(zoom)` with origin at the dome center) applies to the wrapper. Everything inside scales/translates together — true lens behavior. Chrome (close button, reset button, side panel) sits OUTSIDE the wrapper and stays anchored to the window.
> - **Pixi-side `chartContainer.scale` REMOVED**. The chartContainer is now structural only (groups layers); CSS does all the scaling. This avoids dual-transform compounding and simplifies the math.
> - **Hit-test conversion simplified**. With the lens architecture, `(e.clientX - canvasRect.left)` already absorbs the wrapper's translate AND scale into rect.left/top. Inverse simplifies to `internal_x = visual_offset / chartZoom`. Math verified algebraically before commit.
> - **Close button via raw DOM `addEventListener`**. After four rounds of failing Svelte `onclick={...}` bindings (hover styles fired, click handler never did — couldn't reproduce in isolated repros), the close button is now `bind:this={closeBtn}` with `closeBtn.addEventListener('click', ...)` + a defensive `pointerup` listener wired in `onMount`. Bypasses the Svelte event system entirely.
> - **Reset View button always visible**. Was conditional (`{#if zoom !== 1 || pan !== 0}`); Eisa couldn't see it ("I cannot see the reset button"). Now always rendered, with two states: muted (60 % opacity, faded gold border) at default state, prominent (1.5 px gold border, drop shadow, weight 600, color ink) when chartZoom or pan ≠ default. CSS class `.reset-active` toggles. z-index bumped to 999 so it sits above everything except the close button.
> - **Tooltip moved OUTSIDE the zoom wrapper**. CSS `position: fixed` becomes relative to the nearest transformed ancestor — so a tooltip inside the lens wrapper would be positioned relative to the (now-transformed) wrapper, not the viewport. Outside the wrapper, `position: fixed` works correctly relative to the viewport. z-index bumped 50 → 1500 (above everything). Tooltip's first line (the note title) gets `::first-line` 700-weight + 14 px so it reads as a heading.
> - **Edges darkened**. Was light gold (`0xc9a227 @ 0.55`) which Eisa flagged as "not clear with this background." Switched to dark burnt-amber (`0x6b4f0d @ 0.85, width 1.0`) for contrast on the cream BG.
>
> **Type-checked clean.** orientation v1.64 → v1.65 inline.

**Version 1.64 | 2026-05-08**

> **What changed in v1.64** (same day as v1.63; MIG-019 §2G.3h — hit-test inverse-transform, ring sizing, overlay backdrops, edge cap, close button defensive fixes, Esc reset cascade):
>
> Eisa's sixth-round Boss test on §2G.3g found six bugs:
> 1. **Hover/click hit-testing displaced** when zoomed/panned — clicking node A actually selected node B because mouse coords were in transformed space but `pickStar` looked up against canonical positions.
> 2. **Selection ring much bigger than the node** — was `r+3` with a 2 px stroke, ~3× the node diameter.
> 3. **Universe Health card and Universe-name header overlapped the dome** when zooming in — they had no opaque background, so stars bled through their text.
> 4. **Click revealed an overwhelming web of edges** — the prior logic drew ALL edges within the focused node's community on selection, smothering the chart on hub nodes. Selected note's title also wasn't visible (side panel z-index was too low).
> 5. **(×) close button changed color on hover but didn't fire** — likely the `onpointerdown stopPropagation` defensive that I added in §2G.3g was interfering somehow.
> 6. **Reset button discoverability** — Eisa asked for one even though §2G.3g shipped one (faded at the bottom-left, only visible when zoom/pan ≠ default).
>
> Shipped in §2G.3h:
>
> - **`pickStar` inverse-transform** — mouse coords are now inverse-transformed by the chart container's zoom + pan before looking up against `pathToScreen`. The star you point at is now the star you click. Same fix benefits the hover preview.
> - **Selection ring matches node** — radius = `node.r + 1` (was `+3`), stroke = 1.5 px (was 2 px). The ring sits exactly 1 px outside the node's edge, like an outline. Color updated to the §2G palette gold (`#c9a227` from `#d4af37`).
> - **Backdrops on Universe Health + Universe-name** — both overlays now have a 93 % cream background (`rgba(250, 246, 232, 0.93)`) with rounded corners and a soft shadow. Stars zooming under them are properly occluded.
> - **Edge cap and ring-on-neighbours** — clicking a star now draws **incident-only** edges (was: whole community on selection), capped at 50, with lighter stroke (`alpha 0.55, width 0.7`). Each 1-hop neighbour gets a thin gold ring at `r + 0.8` so the user can SEE which stars are connected even through the gold rays.
> - **Side panel z-index 5 → 50** — slides in over the legend (z:7), Universe Health (z:8), Universe-name (z:8), rim wrapper (z:6) so the title is visible. Close button stays above at z:1000. Soft shadow on the panel's leading edge.
> - **Close button defensive** — `onpointerdown stopPropagation` removed (was unnecessary; pointer events on the button don't bubble to the canvas anyway). z-index 100 → 1000. Lambda handler `(e) => { e.stopPropagation(); onClose(); }` invokes the prop explicitly. Hit-area 36 → 38 px, thicker border, `:active` press feedback.
> - **Wheel via `addEventListener`** — Svelte's `onwheel` attribute defaults to passive in modern browsers, silently swallowing `preventDefault()`. Moved to manual `addEventListener('wheel', handleWheel, { passive: false })` so the page no longer scrolls when the user wheels over the chart.
> - **Reactive transform `$effect`** — chartZoom + chartPanX/Y are tracked in an `$effect` that calls `updateChartTransform()` + `syncRimTransform()` synchronously on any change. Belt-and-suspenders: even if a wheel/drag handler forgets the manual call, the rim numbers and the Pixi container always stay in lockstep.
> - **Esc cascade** — keyboard Esc now does: clear selection → reset view → close. One key, three states unwound. Plus the existing Reset View button (now 1.5 px gold border, 13 px font, weight 600, drop shadow) for visual users.
>
> **Type-checked clean.** orientation v1.63 → v1.64 inline.

**Version 1.63 | 2026-05-08**

> **What changed in v1.63** (2026-05-08 morning; MIG-019 §2G.3g — wheel zoom + drag pan + close-button fix + autoDensity):
>
> Eisa's fifth-round Boss test on §2G.3f passed the visual ("Better! I like how it renders"), and asked for three follow-ups:
>
> 1. **Mouse wheel / scroll to zoom in/out** in the chart (browser-level Ctrl+/- broke the layout because Pixi wasn't honoring devicePixelRatio).
> 2. **Pan** — drag the chart with the mouse to move it.
> 3. The (×) **close icon** at top-right wasn't clickable.
>
> Plus a clarifying question: "Does the location of the nodes and their size from the rim towards the center node mean something?" — answered from the visual spec: yes, in Regions mode X = library wedge, Y = centrality rank (center = most-central), Z = degree (size). Same star migrates differently in other modes per the (X, Y, Z) grammar.
>
> Shipped in §2G.3g:
>
> - **Pixi `autoDensity: true` + `resolution: window.devicePixelRatio`** — fixes browser zoom (Ctrl+/-). The canvas backing buffer now scales with DPR so the dome and the HTML overlay stay aligned at any zoom level. Plus a `visualViewport.resize` listener to catch DPR changes that don't trigger a CSS-size resize on the container.
> - **`chartContainer` Pixi parent** — wraps every chart layer (milky way, territories, edges, stars, focus overlay, calendar/region rim). Placeholder text stays directly on the stage. A single transform on `chartContainer` (pivot + position + scale) zooms and pans every chart layer at once.
> - **`.sight-v3-rim-wrapper` HTML wrapper** — wraps the rim numbers and applies the same CSS `transform: translate(...) scale(...)` with `transform-origin` at the dome center. The Pixi container's transform and the HTML wrapper's transform stay in lockstep.
> - **`updateChartTransform()` + `syncRimTransform()`** — called from `fullRedraw()` and from the wheel/drag handlers. Synchronizes the Pixi container with the HTML wrapper using the dome's base center as the pivot.
> - **Wheel zoom**: `onwheel` on the canvas with `preventDefault()`. Multiplicative scaling via `Math.exp(-deltaY * 0.0015)`. Clamped to `[0.4, 5.0]`. Zooms around the dome center.
> - **Drag pan**: `onpointerdown` records the start position. `onpointermove` accumulates movement until `DRAG_THRESHOLD_PX = 4`, then commits to a pan and suppresses hover. `onpointerup` clears the drag. `onclick` checks `panDragMoved` and short-circuits to keep the click-after-drag from triggering a star pick.
> - **Reset view button** — appears in the bottom-left when `chartZoom !== 1 || pan !== 0`. One click returns to the canonical view.
> - **Close button bulletproofing** — z-index 10 → 100 (sits above every overlay), explicit `type="button"` and `pointer-events: auto`, larger hit-area (32 → 36 px), more visible style on cream background, `onpointerdown` stopPropagation so the pan handler never sees the click.
>
> **Type-checked clean.** orientation v1.62 → v1.63 inline.

**Version 1.62 | 2026-05-07**

> **What changed in v1.62** (same day as v1.61; MIG-019 §2G.3f — numbered rim + library legend + library colors + size cap + stronger repulsion + stroke + silent loading):
>
> Eisa's fourth-round Boss test on §2G.3e showed:
> - the de-spoke fix worked (no more spokes), AND
> - stars were still overlapping heavily despite the §2G.3e repulsion (overall density too high for MIN_DIST = 6 px), AND
> - he wanted a major visual redesign:
>   1. Rim labels → **colored numbers**, one per library
>   2. **Library legend panel** on the side (left for LTR, right for RTL) listing Universe root + numbered libraries with color swatches
>   3. **Nodes colored by library** (Louvain communities still computed for cognitive structure — health report, side panel, etc. — but visual coloring uses the library palette)
>   4. **Node size cap** — no node bigger than the center node, sizes proportional below
>   5. **Stroke / contrast frame** around every node
>   6. **Stronger repulsion** so nodes don't touch
>   7. The "Stage 1/4: fetching layout" placeholder removed — silent load
>
> Shipped in §2G.3f (`PENDING-COMMIT`):
>
> - **`src/lib/sight/v3/library-colors.ts` (new)** — deterministic per-library palette using golden-angle hue increment (137.5°) so even N=20 neighbouring libraries on the hue wheel land in distinguishable bands. Saturation 55 %, lightness 42 % — readable on cream parchment. `buildLibraryColorMap(wedges)` keys colors by `library_path` and assigns 1-indexed positions for the rim numbers and legend rows.
> - **Star coloring** moved from uniform `0x1a1a1a` ink to per-library hex via the new map. Louvain community structure is **untouched** — `pathToCommunity`, `communityById`, `clusters`, `healthReport` all still compute. Eisa's clarification: "the Louvain community will remain as it is, but the color scheme will follow its library coloring."
> - **Star sizing capped** at MAX_NODE_RADIUS = 4 px (diameter 8 px), MIN_NODE_RADIUS = 1.2 px. Sizes scale linearly from screen.r so highly-connected nodes are biggest but never exceed the cap. Pairs cleanly with MIN_DIST = 9 px so even adjacent max-size stars keep a 1 px breathing gap.
> - **Each star has a thin contrast stroke** — 0.6 px ink (`#1a1a1a`) at alpha × 0.85. Drawn in the same single-Graphics batch as the fill so the §2E.4 OOM solve still holds (one GL draw call).
> - **Repulsion strengthened** — MIN_DIST 6 → 9 px; MAX_ITER 6 → 12. Same wedge-bounded grid; angular-and-radial clamps after each iteration so a Biology star can't drift into Physics.
> - **Sqrt mapping for radius** — `Math.sqrt(rank)` replaces direct rank percentile, giving uniform AREA density (was uniform RANK density, which packs more stars per unit area near center because angular area scales with r).
> - **Rim labels → colored numbers**. Each wedge displays its 1-indexed number colored from the library palette. Halo via `text-shadow: 0 0 3px #faf6e8` for legibility. No tangent rotation needed (single digit / two-digit number is upright).
> - **Library legend panel** at top-left (LTR) or top-right (RTL via `detectDir(universeName)`). Shows: "UNIVERSE" caps caption → universe name → divider → numbered library rows with circular color swatch + library name + note count. `dir="auto"` per row for native bidi.
> - **Silent loading** — staged `Stage 1/4 / 2/4 / 3/4` placeholder text removed. Detailed timing still flows to `console.log` for diagnostics. The empty-universe case and error states still show placeholder text.
>
> **Type-checked clean.** Same pre-existing `LinkLifecycle.fresh` only. orientation v1.61 → v1.62 inline.

**Version 1.61 | 2026-05-07**

> **What changed in v1.61** (same day as v1.60; MIG-019 §2G.3e — de-spoke + smaller rim font + corrected placeholder text):
>
> Eisa's third-round Boss test on §2G.3d caught three issues:
>
> 1. **"Stage 1/4: layout (MDS embedding)" placeholder** is misleading — the polar layout doesn't use MDS for screen positioning anymore. Placeholder text now reads **"Stage 1/4: fetching layout"**.
>
> 2. **Library names too large** — 15 px serif aggressively triggered ellipsis on long names ("EARTH SCIEN…", "ARCHITECTU…", "RELIGION & C…"). Reduced to **12 px**, letter-spacing tightened. Long names still truncate but trigger the ellipsis less often. Note count caption also reduced 10 → 9 px.
>
> 3. **Stars formed radial spokes within each wedge** instead of spreading. Root cause: `azimuthInWedge` used `atan2(embed_x, embed_y)` from the Rust MDS embedding. Notes from the same MDS cluster (same Louvain community) had nearly identical embed angles → all stacked on a single in-wedge azimuth → radial spoke per cluster. Fixes:
>    - **Hash-based azimuth jitter** replaces `atan2(embed_x, embed_y)`. djb2 hash on the note path → uniform [0, 2π) spread within the wedge. Deterministic per note path so positions are stable across renders.
>    - **Small radius jitter** (±1.5 % of rank) so notes with identical centrality rank don't form perfect concentric rings.
>    - **Wedge-bounded repulsion pass** — Eisa's directive: "Each node shouldn't touch or overlap its neighbors. There should be a node repulsion effect." Within each wedge, push apart any pair of stars closer than 6 px. 6 iterations, spatial-grid neighbor lookup (O(N) per iteration), wedge-bound clamp after each iteration so stars can't drift across wedge boundaries.
>
> **Type-checked clean.** Same `LinkLifecycle.fresh` pre-existing only.
>
> **Boss test pending** — installer build + walkthrough delivered next; if §2G.3e looks right, cascade into §2G.4 (mode toggle UI + 600 ms migration animation).

**Version 1.60 | 2026-05-07**

> **What changed in v1.60** (same day as v1.59; MIG-019 §2G.3c finish + §2G.3d (X, Y, Z) refactor):
>
> **§2G.3c (rim label ellipsis + Universe-name header) and §2G.3d ((X, Y, Z) per-mode dispatch) shipped together** in one commit, with this orientation bump and `docs/SIGHT-V3-VISUAL-SPEC.md` v1.0 → v1.1 inline (per the "orientation in same commit as SO #6 trigger" rule):
>
> - **Rim label ellipsis**: per-wedge tangential chord length emitted as `max-width` per label; long library names truncate with `…` instead of bleeding across adjacent wedges. CSS `text-overflow: ellipsis` on `.sight-v3-rim-label` and `.sight-v3-rim-count`.
>
> - **Universe-name header**: new `universeName` prop on `SightV3.svelte`, passed from `+layout.svelte` as `activeUniverseName`. Renders as a serif italic blue-ink header positioned in the 50 px slot between the Universe Health metrics and the dome top edge. `dir="auto"` so Arabic / Hebrew Universe names render correctly. `getViewport()` TOP_RESERVE bumped 270 → 320 px to accommodate.
>
> - **(X, Y, Z) per-mode grammar** (the architectural elevation Eisa approved earlier this evening — visual spec doc bumps to v1.1):
>   - `src/lib/sight/v3/modes.ts` extended with `ModeContext`, `ModePosition`, `ModeStats`, and a `positionForMode(mode, ctx)` dispatcher.
>   - `positionForRegions`: X = library wedge azimuth, Y = centrality rank percentile, Z = total degree (link count).
>   - `positionForLinkTypes`: Z = outgoingCount works today; X (dominant link type) and Y (type diversity) need `note_links.link_type` piped through, so it currently routes through Regions for X/Y until §2G.4 follow-up.
>   - `positionForTime`: X = creation date wedge (year wedges sized by note count, empty years compressed), Y = recency from `modifiedAt` (today: `createdAt` stand-in until `note_meta.modified_at` is piped), Z = age (oldest = brightest).
>   - C/S/A modes fall back to Regions until their data layers ship per Concept Paper §6.3 P2/P3/P4.
>   - `buildModeStats()` builds universe-wide stats (T mode wedges, time spans) once per fetch.
>   - `recomputeScreenPositions()` in `SightV3.svelte` refactored to use the dispatcher with a reusable `ModeContext` (no per-iteration allocation).
>   - `pathToScreen` map now carries `baseAlpha` so star magnitude is fully mode-aware.
>
> - **Visual spec doc v1.1**:
>   - §1 restructured from "switchable rim axis" to "per-mode (X, Y, Z) grammar" with the full 6-row table per mode (X / Y / Z / cognitive question / data status).
>   - §7 invariants list updated: only **color** is mode-invariant; X/Y/Z are mode-specific. Added invariants 11 (color preserved across mode switches), 12 (rim labels are HTML overlay for native bidi).
>
> **Type-checked clean** (only the pre-existing `LinkLifecycle.fresh` issue in store.ts that's already logged for post-CE follow-up).
>
> **Boss test pending** — installer build + walkthrough delivered next; then §2G.4 cascades (mode toggle UI + 600 ms eased migration + finally lighting up the toggle so the user can switch R ↔ L ↔ T live).
>
> **What's still pending** (post-§2G.3d):
> - §2G.4: mode toggle UI (top-right 6-button bar, R/L/T highlighted "READY", C/S/A dimmed "AVAILABLE LATER") + 600 ms eased migration animation + keyboard shortcuts (R/L/T/C/S/A direct-switch keys + Esc to clear).
> - §2G.5: persist `appSettings.sight.lastMode` per Universe; resolve to `Regions` if a stored mode's data layer isn't ready.
> - §2G.6: 3-agent audit (invariants / drift / migration path) + tag MIG-019 milestone + orientation v1.61 + i18n keys for mode names + close-out.

**Version 1.59 | 2026-05-07**

> **What changed in v1.59** (same day as v1.58; MIG-019 §2A → §2E shipped + §2G in flight):
>
> **MIG-019 §2A → §2E SHIPPED (10 commits today, pushed)** — full v3 §2 surface beyond the §1 foundation:
> - **§2A** TF-IDF compute + similarity IPC (PJ-035 foundation, schema v2)
> - **§2B** Milky Way density wash + Settings toggle
> - **§2A+§2B redesign** density grid replaces edge list (OOM-proof — input-size invariant payload, 256 KB output regardless of universe size). Eisa's directive: "Don't patch it. Solve it."
> - **§2C** calendar rim (Gregorian default + Hijri toggle; Solar Hijri / Hebrew via Settings) + month filter
> - **§2D** universe-health card (modularity / dominance / entropy / connectivity)
> - **§2E** full search integration + always-on labels + Boss-test gate
> - **§2E.1 → §2E.4** four OOM hot-fix commits on Boss-scale 7,636-note universe; root cause was Pixi v8 GPU buffer exhaustion from per-star `new Graphics()` instances. Solve: single Graphics with subpaths + `safeClearContainer` destroying children on remove.
> - origin/main was 36 hours stale; all 10 §2A → §2E commits + §2G work pushed tonight.
>
> **MIG-019 §2G IN FLIGHT (3 commits tonight, pushed)** — visual rewrite from MDS layout to polar layout per Eisa's design directive (`docs/Constellation-Sight-v3-mockup-A2-toggle.svg` + `docs/SIGHT-V3-VISUAL-SPEC.md`):
> - **§2G.1** Visual spec doc + 5 mockup options + Suwaidi cream palette (`bfb8aba`).
> - **§2G.2** Pure helpers: `polar.ts` / `modes.ts` / `regions.ts` (`b1a2477`).
> - **§2G.3 → §2G.3c** SightV3.svelte rewrite cascade (`7d6fcf6`):
>   - Theme: navy → cream parchment.
>   - Polar layout: radius from centrality rank percentile (was direct centrality_norm — distribution skewed packed >90 % at rim); azimuth from library wedge.
>   - Region rim: library wedges sorted by note count desc, empty wedges compressed, blue-ink labels.
>   - Edges: hidden in resting state. Hover/click reveals selected node's links in gold (Concept Paper §4.1).
>   - Universe Health: HTML overlay anchored top-center, four metrics flanking the gold roundel.
>   - Side panel: Universe Health section removed; opens only on star selection.
>   - Calendar-rim hover/click handlers gated on `currentMode === 'time'` (Eisa caught the rim auto-shifting from Regions to calendar months).
>   - Dome margins: 270 px top + 100 px bottom + 100 px sides + 80 px outside-dome reserved.
>   - Region rim labels migrated from Pixi Text to HTML overlay with `dir="auto"` so the browser handles bidi natively for Arabic / Hebrew / mixed-script library names.
>
> **Eisa-approved (X, Y, Z) per-mode grammar** (visual-spec doc bumps to v1.1 in §2G.3d):
> | Mode | X (azimuth) | Y (radius) | Z (magnitude) | Cognitive question |
> |---|---|---|---|---|
> | R · Regions | Library | Centrality rank | Degree | "Where in my cosmos does this idea live, and how central?" |
> | L · Link Types | Dominant outgoing link type | Type diversity | Total outgoing links | "What kind of reasoning, and how versatile?" |
> | T · Time | Creation date wedge | Recency (last edit) | Age | "When did it emerge, and is it still alive?" |
> | C · Confidence | Dominant confidence | Certainty homogeneity | Total link count | "How certain, and how consistent?" |
> | S · Stages | Dominant lifecycle stage | Avg link weight | Total traversal count | "How alive, and how worn the path?" |
> | A · Acts | Which Act produced the note | Synthesis depth | Total connections | "Where in the formulation arc?" |
>
> Color stays invariant across all modes (Louvain community membership). Sight becomes a true multi-instrument cognitive lens — the §5.3 stethoscope / MRI / ECG metaphor finally fully realised.
>
> **Boss tests passed**: §2G.3 visual (cream theme + polar layout + region rim + Universe Health + edges hidden), §2G.3a (territories disabled, side panel cleaned, metric labels fixed), §2G.3b (mode-lock + dome margins + Arabic RTL).
>
> **Standing Order catch-up tonight (~21:50)** — implementation cascade had pulled past doc discipline. Caught up: 3 commits + push tonight; session-log entries for §2G.1 → §2G.3c + state-of-standing record (SO #5) for the X/Y/Z pivot; this orientation v1.59; MoCh due.
>
> **Next**: §2G.3c finish (rim label ellipsis CSS + Universe-name header) → §2G.3d (X/Y/Z refactor) → §2G.4 (mode toggle + 600 ms migration) → §2G.5 (mode persistence) → §2G.6 (audit + close + orientation v1.60).

**Version 1.58 | 2026-05-07**

> **What changed in v1.58** (same day as v1.57; MIG-018 ships v3 projection foundation):
>
> **MIG-018 closes Done** — Sight v3 projection foundation live in production. Six-phase cascade (§1A → §1F) shipped today across 8 commits. The first of three v3-build MIGs per the Concept Paper v1.1 §9 trajectory.
>
> **What's user-visible in v3 §1E**:
> - Star-icon dock button next to where the v2 eye-icon used to be.
> - Dome of stars on Suwaidi-chart deep midnight blue, sized by betweenness centrality (logarithmic 6-magnitude scale).
> - Constellation territories drawn as soft Suwaidi pastel polygons (warm-cream + gold + amber + dusty rose + sandy tan + parchment + antique-white + dark goldenrod cycled by Louvain community id).
> - Faint connector lines visible at rest (Eisa's design call: "we will show it as faint lines until the user hovers over it" — the v3 reframe of v1.1 paper Principle 6).
> - Hover star → tooltip + incident edges brighten; click → constellation lights up + side panel slides in; double-click → opens note in editor.
> - Settings → Sight section: Lambert (default, equal-area) / Stereographic (equal-angle) projection toggle. Switching is free (frontend-only re-projection of the cached MDS embedding).
> - Esc clears selection then closes; deterministic per-snapshot layout means notes return to remembered positions on re-open (spatial-memory grammar working).
>
> **Boss test passed all 11 steps** with `SIGHT_V3_ENABLED = true` flipped locally. Const now committed `true` — production-ready in default config.
>
> **Three-agent audit CLEAN** (0 P0, 0 P1, 0 P2, 0 P3). Audit report at `lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-AUDIT.md`. Verified all 13 invariants from the Architect, the drift map, and the seven migration-path scenarios.
>
> **What's NOT in v3 yet** (deferred per Concept Paper v1.1 §9.2-§9.3):
> - Milky Way density wash (PJ-035) — MIG-019.
> - Calendar rim (Gregorian default + user-add via Settings) — MIG-019.
> - Universe-health card in side panel — MIG-019.
> - Search flares + halo (basic match-highlight wired in §1E; full version in MIG-019).
> - Magnitude slider / layer peeling (PJ-036) — MIG-020.
> - v2 retirement — MIG-020 (after Boss confirms v3 stable across multiple sessions).
> - PJ-037 (Map↔Sight integration) — REJECTED, not in any v3 MIG.
>
> **Pending Jobs v1.7** (`docs/Constellation Pending Jobs v1.7.md`):
> - PJ-038 status: Confirmed → **In-Progress** (1 of 3 MIGs done).
> - MIG-018 trajectory updated: phase 1/3 closed; MIG-019 next-up.
> - Done count after v1.7: 7 (unchanged). Cancelled: 1 (PJ-034). Rejected: 1 (PJ-037).
>
> **Eight commits today on the v3 trajectory**:
> | Commit | Phase / scope |
> |---|---|
> | `1164b08` | PJ-038 Concept Paper v1.0 drafted |
> | `44c37c9` | PJ-038 Concept Paper v1.1 ratified + PJ-037 rejected + PJ v1.6 + orientation v1.57 |
> | `51e270a` | MIG-018 Architect + Plan |
> | `fe85792` | MIG-018 §1A — schema + Rust skeleton |
> | `24aa6bd` | MIG-018 §1B — Landmark-MDS compute (5 unit tests passing) |
> | `dd6759e` | MIG-018 §1C — frontend skeleton + dock button + i18n 15 locales |
> | `4dc6878` | MIG-018 §1D — star rendering + Lambert/stereographic toggle |
> | `26ce36e` | MIG-018 §1E — territories + faint lines + hover/click + Suwaidi palette [Boss-test passed] |
> | (this) | MIG-018 §1F — audit + close-out + orientation v1.58 + Pending Jobs v1.7 + SIGHT_V3_ENABLED=true committed |

**Version 1.57 | 2026-05-07**

> **What changed in v1.57** (same day as v1.56; PJ-038 v3 Concept Paper ratified, PJ-037 rejected, MIG-018 unblocked):
>
> **Sight v3 Concept Paper v1.1 ratified by Eisa**. `docs/Constellation-Sight-v3-Concept-Paper-v1.1.md` is the design contract for the multi-MIG v3 build. v1.0 was the same-day "drafted, awaiting review" state with ten open §11 questions; v1.1 has all ten resolved + two structural revisions: (a) connector lines are now **faint at rest, brighten on hover/select** (replaces v2's "no lines until hover" — Eisa wants the structural pattern visible at rest, just unobtrusive); (b) Map ↔ Sight integration **rejected** — PJ-037 retired in Pending Jobs v1.6.
>
> **Ten §11 design calls (resolved 2026-05-07)**:
> 1. **Embedding**: graph-distance MDS (Landmark variant for memory).
> 2. **Projection**: both Lambert (default) and stereographic ship; user toggle in Settings.
> 3. **Calendar rim**: Gregorian default; users add others (Hijri, Solar Hijri, Hebrew, etc.) via Settings → "Calendar systems."
> 4. **Magnitude slider**: astronomy convention — drag right peels bright stars.
> 5. **Two-up panel**: N/A — Map↔Sight integration rejected (PJ-037).
> 6. **Constellation labels**: hover/select only by default; Settings toggle for always-on.
> 7. **Color scheme**: cycled pastels by Louvain id default; user-overridable via existing Style Settings.
> 8. **Search filter persistence**: Esc + click-background to clear; no persistence across Sight close/reopen.
> 9. **Render layer**: Canvas 2D + D3-zoom (§2G.3q migration from Pixi.js v8 — Pixi's capture-phase EventSystem was the root cause of the 11-iteration close-button failure). Immediate-mode draw() pipeline + DOM layer for UI chrome. SkyView uses the same D3 + Canvas 2D pattern.
> 10. **Accessibility (high-contrast / keyboard nav)**: deferred to a separate post-v3 PJ.
>
> **PJ-037 Rejected**. Sight v3 stays single-view; Map and Sight remain independent surfaces. The "Map diagnoses, Sight prescribes" loop happens in the user's head, not in a shared cursor. Number retired per stable-reference-numbers rule.
>
> **Pending Jobs v1.6** (`docs/Constellation Pending Jobs v1.6.md`):
> - PJ-037 → Rejected.
> - PJ-038 §8 trajectory revised: MIG-020 phase reduced to PJ-036 + v2 retire only (no PJ-037 absorption).
> - Done count: 7 (unchanged). Cancelled: 1 (PJ-034). **Rejected: 1 (PJ-037 — new)**.
>
> **Three-MIG v3 build sequence** (per Concept Paper v1.1 §9):
> - **MIG-018** — Projection foundation. Rust `compute_layout_embedding` (Landmark MDS), `sight_v3_layout` SQLite cache, `src/lib/sight/projection.ts` + `SightV3.svelte`, dock button + Settings entry behind `SIGHT_V3_ENABLED`. Boss-test gate: stars render at correct positions, basic hover/click works.
> - **MIG-019** — Density + time + search. PJ-035 (Milky Way), calendar rim, search integration, universe-health card. Boss-test gate: full visual grammar live.
> - **MIG-020** — Layer peeling + v2 retire. PJ-036 magnitude slider, v2 fallback removal once Boss confirms v3 stable.
>
> **Next-up**: MIG-018 Architect.

**Version 1.56 | 2026-05-07**

> **What changed in v1.56** (same day as v1.55; MIG-017 closes — v2 Sight unreachable in production):
>
> **MIG-017 (PJ-039) shipped — single phase, single commit.** v2 Sight is now unreachable from the running app's user surface in default config. Mechanism: a single code constant `SIGHT_V2_ENABLED = false` in the new `src/lib/sight/engine.ts` module gates four UI surfaces — dock button, modal mount, "Return to Lens" button, Settings → Plugins entry. The v2 component (`ConstellationSight2.svelte`), the `lens*` `$state` fields in `+layout.svelte`, the `toggleLens()` async function, the Rust analytics modules (`lens.rs`, `lenses.rs`), and the `constellation_sight_*` IPCs are **all preserved on disk** as a known-good fallback. Re-enable = flip the const + rebuild.
>
> **Help-doc banner added** at the top of `docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md` — "Constellation Sight is being rebuilt; v3 is in design; here's the link to the v1.1 Concept Paper for context." Original v2 documentation paragraphs untouched beneath.
>
> **Pending Jobs v1.5** (`docs/Constellation Pending Jobs v1.5.md`):
> - PJ-039 → **Done**.
> - Top of queue rotates: **PJ-038 (Sight v3 + own Concept Paper)** → PJ-005 (Links Settings tab) → PJ-002 (cid_cn collision scrub).
> - Done count after v1.5: 7 (PJ-001, PJ-006, PJ-007, PJ-025, PJ-026, PJ-027, PJ-039). Cancelled count: 1 (PJ-034).
>
> **Audit: three-agent** (invariants / drift / migration-path) ran on the diff. Audit report at `lab/reports/MIG-017-DISABLE-V2-SIGHT-AUDIT.md`. 0 P0, 0 P1.
>
> **Next-up — PJ-038**: Sight v3 build with **own dedicated Concept Paper**. Multi-MIG. Star-chart aesthetic per the v1.1 paper §13–§14 vision. v3 inherits the Rust analytics IPCs as-is from v2; rebuilds the visualization layer entirely. PJ-035 (content similarity) / PJ-036 (layer peeling) / PJ-037 (Map↔Sight integration) absorbed as v3 features rather than v2 add-ons.

**Version 1.55 | 2026-05-07**

> **What changed in v1.55** (Boss-directed 2026-05-07; closes the MIG-016 cycle, lands the Sight Concept Paper v1.1, and frames the v3 trajectory):
>
> **MIG-016 closes — Cancelled (partial-shipped).** §1A instrumentation + §1B edges-on-hover gate shipped (commits `a0babbb` → `7e76b17` → `62718f7`). §1C (Web Worker offload), §1D (post-paint prewarm), §1E (SQLite `sight_cache`) **abandoned mid-flight**. Audit close-out at `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md`. PJ-034 retired. The "instant first-toggle" headline goal was not met for v2; designed-in for v3 from the start.
>
> **Decision: secure-don't-muddle.** v2 Sight (`ConstellationSight2.svelte` + the `lens_*` Rust modules + `constellation_sight_*` IPCs) is being **disabled as a known-good fallback** under MIG-017 (PJ-039), while v3 is built fresh under PJ-038. The Rust analytics IPCs and the v2 Svelte component **stay on disk** — they are the proven baseline if v3 fails. v3's visualization layer is rebuilt entirely from the **star-chart aesthetic** (Suwaidi northern-hemisphere chart reference; Sight Concept Paper v1.1 §13).
>
> **Sight Concept Paper v1.1 lands.** `docs/Constellation-Sight-Concept-Paper-v1.1.md` is the markdown port of Eisa's April 2026 v1.0 PDF, refreshed with: (a) "What this paper IS" disclaimer, (b) §12 truth-status matrix (each mechanic mapped to *what's actually shipped*), (c) **Principle 6 — reveal-on-demand** (the edges-on-hover gate as a permanent design principle, not a perf hack), (d) three implementation gaps tracked as PJ-035 / PJ-036 / PJ-037, (e) §13 star-chart vision as the design north star, (f) §14 v3 redesign noted with **its own dedicated Concept Paper to follow**.
>
> **Honest delivery score for v2 Sight**: ~70-80% of the Concept Paper's analytical promise. Centrality / community detection / structural gaps / universe-health all real. **Three notable omissions** — content-similarity TF-IDF edges (PJ-035), layer peeling (PJ-036), Map↔Sight integration (PJ-037) — all inheritable into v3 by design.
>
> **Pending Jobs v1.4** (`docs/Constellation Pending Jobs v1.4.md`):
> - PJ-034 closes as **Cancelled (partial-shipped)** — new terminal status added to status vocabulary.
> - **PJ-035** allocated — Sight content-similarity TF-IDF edges.
> - **PJ-036** allocated — Sight layer peeling.
> - **PJ-037** allocated — Map ↔ Sight integration.
> - **PJ-038** allocated — Sight v3 build with own dedicated Concept Paper (multi-MIG, star-chart aesthetic).
> - **PJ-039** allocated — MIG-017 disable v2 Sight (mini-MIG, single session, **next-up**).
>
> **Top of queue**: PJ-039 (MIG-017 disable v2) → PJ-038 (Sight v3 + own Concept Paper) → PJ-005 (Links Settings tab) → PJ-002 (cid_cn collision scrub).
>
> **Done count after v1.4**: 6 (unchanged). Cancelled count: 1 (PJ-034 partial-shipped).
>
> **§17 update**: `Constellation_Lens_Concept_Paper_Eisa.pdf` no longer in "binary docs not read" — read in full this session via `pypdf`, content folded into `docs/Constellation-Sight-Concept-Paper-v1.1.md` markdown port.

**Version 1.54 | 2026-05-06**

> **What changed in v1.54** (same day as v1.53; MIG-016 §1B lands): edges-on-hover gate in Sight, mirroring Sky View's "nervous system" pattern.
>
> **§1A wrap**: data-collection gate generated three mount-trace paste-throughs (175 / 174-188 / 367 ms total) confirming mount is fast. The toggle trace never fired through Eisa's clipboard because the cache-fast path skips the marks once `lensHealth !== null`. Path 2 chosen (skip toggle-trace data; proceed to §1B based on the verifiable-fast mount + the unmeasured-but-likely first-paint render cost). `alert()` / `clipboard.writeText` calls removed from the §1A code (intrusive in-build); `performance.mark` instrumentation + `console.log` retained for future DevTools sessions.
>
> **§1B implementation** in `src/lib/components/ConstellationSight2.svelte`:
> - **`neighborMap: Map<string, Set<string>>`** populated once per `buildSimData()` call (every link contributes both directions). Mirrors `graphEngine.ts:410-429`.
> - **`needsEdgeDraw` gate** in `draw()`: skips the entire `drawLinks()` call unless one of four conditions holds — hovered node, selected node, active search, or hovered link annotation. On the idle-Sight common path, edge-draw cost drops to **zero**.
> - **Hover/select neighborhood filter** at the top of `drawLinks()`: when one node is hovered or selected (and search isn't active and no link annotation is hovered), iteration drops from `O(E)` to `O(degree)` via the `focusOnly` early-skip.
>
> **Boss-test gate next**: install build, toggle Sight (expect nodes-only paint, no edges), hover a node (its neighborhood lights up), search a term (matched nodes' edges show), Escape / move cursor away (edges hide).

**Version 1.53 | 2026-05-06**

> **What changed in v1.53** (same day as v1.52; MIG-016 §1A fix-up): production binary ships with DevTools disabled, so the §1A `console.table` dump alone wasn't usable for the data-collection gate (Eisa: "the developer console won't open with the binary"). Added a clipboard-write + `alert()` fallback alongside the existing `console.log`: after Sight toggle completes (and after the Sight2 mount), the trace is JSON-formatted as a paste-friendly text block, written to clipboard, and a confirmation alert prompts Eisa to paste it into chat. Both alert dialogs fire in sequence (toggle → mount). Console.log retained for any future session where DevTools is enabled.
>
> **Side-observation worth flagging**: production-build DevTools disabled by default is a Tauri default that may be worth re-evaluating since Eisa is the project's operator + tester (not just an end user). Logged as a candidate PJ for next Pending Jobs bump if Eisa wants persistent DevTools access.

**Version 1.52 | 2026-05-06**

> **What changed in v1.52** (same day as v1.51; MIG-016 §1A lands): instrumentation phase. `performance.mark`s wrapped around every step of `toggleLens()` in `src/routes/+layout.svelte:3332-3460` (rust-centrality / louvain / structural-gaps / universe-health / stratum-weighted / top-bridges / community-profiles / bridge-suggestions / total) AND every step of the cold-mount path in `src/lib/components/ConstellationSight2.svelte::onMount` (buildSimData / layout / fitToScreen / total). Both dumps via `console.table` after lensActive flips / mount completes. No behaviour change.
>
> **Boss data-collection gate next**: build, install, open DevTools console, toggle Sight, send the two console.table outputs to Claude. Trace calibrates §1B (edges-on-hover) / §1C (worker offload) / §1D (post-paint prewarm) / §1E (SQLite cache) per-phase budgets.

**Version 1.51 | 2026-05-06**

> **What changed in v1.51** (same day as v1.50; MIG-016 opens — PJ-034 Sight instant-toggle perf): Eisa-directed perf work on Constellation Sight after a three-pass cross-check (v1.50 latest body, then deeper agent reading §4.x bodies + recent session logs, then full-history scan of all 50 orientation versions + 29 session logs).
>
> **Three findings drove the design choices:**
>
> 1. **No prior B-4-style architecture proposal for Sight has been made.** Boot-prewarm + SQLite cache + dedicated worker + edges-on-hover for Sight is net-new ground. The 2026-04-22 §55 in-memory `lensDataStale` cache is the only prior Sight perf layer; MIG-016 supersedes it as the L1 of a three-tier cache.
> 2. **The 2026-04-13 Sight2 redesign decided "all links solid by default"** — Eisa-confirmed reversed today (2026-05-06). New default: edges hidden until hover or search match. Rationale: the 2026-04-13 decision predated the 2026-04-21 Sky View edges-on-hover work that proved how much render headroom that pattern unlocks.
> 3. **PJ-025 reframe** — PJ-025 was retired as OBSOLETE in Pending Jobs v1.2 because Sight is on-demand (not boot-rebuilt). PJ-034 covers a **different perf axis** — first-toggle latency, which the 2026-04-22 §55 in-memory cache doesn't address across session boundaries. PJ-025 stays retired; PJ-034 is the net-new MIG.
>
> **Architect doc**: `lab/reports/PJ-034-SIGHT-INSTANT-TOGGLE-ARCHITECT.md`. Six-phase plan (instrumentation → edges-on-hover → worker offload → post-paint prewarm → SQLite cache → audit). Three Boss-test gates (Phases 1B, 1D, 1E).
>
> **Awaiting Eisa's "Architect approved"** before writing the Plan.

**Version 1.50 | 2026-05-06**

> **What changed in v1.50** (same day as v1.49; Pending Jobs v1.3 closes the deeper cross-check): the deeper cross-check agent (this time reading orientation §4.x BODIES + session logs per the new SO #8) classified all 27 remaining PJ entries against the latest canonical state.
>
> **Outcome — only 1 entry needed flipping**:
> - **PJ-006 (Living Link Architecture P2–P5) → SHIPPED.** Orientation v1.49 §4.4 confirms `_link_traverse / _link_decay / _link_set_confidence / _link_archive` IPCs (P2/P3), `formulationAnalysis` wrapper (P4), and `KnowledgeHealthDashboard.svelte` mounted in `+layout.svelte:5975` (P5). All four phases live and user-validated.
>
> All 27 other entries verified unchanged from v1.2. No new stale entries surfaced. Scope-rewrites in v1.2 (PJ-010, PJ-014, PJ-021) confirmed correct.
>
> **Done count after v1.3**: 6 (PJ-001, PJ-006, PJ-007, PJ-025, PJ-026, PJ-027).
>
> **Top of queue rotates**: PJ-005 (MIG-007 Links Settings tab) → PJ-002 (cid_cn collision scrub) → PJ-008 (Outgoing Links typed-link dedupe).
>
> **The audit cycle that produced v1.2 → v1.3**: a real demonstration of why the iterative cross-check pattern works. v1.2 caught 5 stale entries (3 OBSOLETE + 2 SHIPPED) but missed PJ-006 because the agent only read preambles. The PJ-006 catch produced SO #8 ("read bodies, not just preambles") which the v1.3 cross-check obeyed and closed cleanly.

**Version 1.49 | 2026-05-06**

> **What changed in v1.49** (same day as v1.48; Standing Order #8 added + PJ-006 catch): Eisa-directed Standing Order: cross-check any PJ before tackling it.
>
> **The catch**: I had just closed Pending Jobs v1.2 (committed `3929bba`) and prepared to cascade into PJ-006 (Living Link Architecture P2–P5). Started by re-reading the PJ-006 entry → discovered orientation §4.4 has been titled *"The Living Link Architecture (P0–P5 all shipped + user-validated)"* since v1.40 (2026-05-05). The v1.2 cross-check agent missed this because my own instructions told it to read only the "What changed in vX.Y" preambles, not orientation bodies. Body trumps preamble for canonical state.
>
> **Eisa's response**: "Don't start tackling any PJs unless you cross-check them with the orientation and session log files." Recorded as **Standing Order #8** in CLAUDE.md + memory feedback note `feedback_pj_crosscheck_before_tackle.md`.
>
> **What's next**: Path 1 — bump to Pending Jobs v1.3 with PJ-006 marked OBSOLETE/SHIPPED, AND re-run a deeper cross-check that reads orientation §4.x BODIES (not just preambles) plus session logs. Find any other stale entries the v1.2 audit missed. After v1.3 closes, work the new top-of-queue.

**Version 1.48 | 2026-05-06**

> **What changed in v1.48** (same day as v1.47; Pending Jobs v1.1 → v1.2 cross-check audit): Eisa-directed cross-check of every Pending Jobs entry against the full orientation timeline (v1.0 → v1.47). Outcome: **2 entries closed (PJ-001, PJ-007 already SHIPPED), 3 entries verified OBSOLETE (PJ-025 Sight, PJ-026 sidebar stars, PJ-027 Map — all already write-time-derived or cache-fast), 3 entries scope-rewritten (PJ-010 alias-bleed, PJ-014 doc-body backfill, PJ-021 narrowed to "verify-then-narrow"), 6 new entries allocated PJ-028 → PJ-033** (MIG-014 §2F audit P2/P3 follow-ups carried from memory).
>
> **Pending Jobs v1.2** (`docs/Constellation Pending Jobs v1.2.md`) is the new canonical backlog. v1.1 stays as iteration record per the doc-versioning convention. Stable reference numbers preserved — PJ-025/026/027 retired but their entries kept with OBSOLETE status; numbers never reused.
>
> **Top of queue** (per v1.2 Quick Reference): PJ-006 (Living Link Architecture P2–P5) → PJ-005 (Links Settings tab) → PJ-002 (cid_cn collision scrub).
>
> **Living Link Architecture P2–P5 (PJ-006)** is unblocked now that PJ-007 closed. It's the multi-MIG that completes the Living Link Architecture as a whole — needs its own Concept Paper before the Migration Rule cascade.

**Version 1.47 | 2026-05-06**

> **What changed in v1.47** (same day as v1.46; MIG-015 closes): §1D three-agent audit complete. Audit report at `lab/reports/MIG-015-CHUNKED-V2-SENTINEL-AUDIT.md`.
>
> **Audit verdict**: 11 of 12 invariants ✅. One P0 found and fixed in the close-out commit: the DB mutex was held across the whole chunked loop, contradicting the §1B design promise + §1C Boss-test claim ("you can edit notes, search, switch tabs while it runs"). Three agents converged on the same finding at three severities (invariant: P1, drift: P0, migration-path: P2). Fix: refactored the chunked helper to a single-chunk `sentinel_bigram_rows_chunk(conn, chunk_size)`, with the worker (`run_v2_sentinel_migration`) doing the lock dance per chunk + 10ms inter-chunk yield. Other DB callers now see ~10ms availability windows between chunks as originally promised.
>
> **Visual Boss test on §1C skipped per Eisa's call.** Boss's library is already at v2 from earlier MIG-013 testing; rolling back to manufacture migration work would touch closed-feature production data. Working Agreement #4 forbids "let's see what happens" on closed-feature data. Static audit verifies behaviour by code-reading; future users with pre-MIG-013 backups will exercise the visible path naturally.
>
> **MIG-015 STATE — CLOSED.** PJ-001 (chunked v2 sentinel migration with progress UI) shipped. The deferred P1-M1 from MIG-013 §1E is now closed.
>
> | Phase | Scope | Status | Commit |
> |---|---|---|---|
> | §1A | Rust helpers (count + chunked-helper) | Done | `0ca7e64` |
> | §1B | init_db defers; async task wired | Done | `df0bf87` |
> | §1C | Frontend strip + 15-locale i18n | Done; visual test skipped per Eisa | `62d3b4a` |
> | §1D | Three-agent audit + P0 fix | Done | (this commit) |
>
> **Next**: PJ-006 — Living Link Architecture P2–P5 (multi-MIG, the link-side lifecycle work that PJ-007 unblocked). Eisa queued for after MIG-015.

**Version 1.46 | 2026-05-06**

> **What changed in v1.46** (same day as v1.45; MIG-015 §1B + §1C land): §1B moves the v2 sentinel migration off the boot critical path (deferred to a worker thread spawned from `ensure_search_db_ready`, mirroring the `sky_backfill::maybe_schedule` pattern). §1C ships the frontend status-bar progress strip (`MigrationProgressStrip.svelte`) in a new `.sb-center` group + i18n keys in all 15 locales (`migrationProgress.termVocabV2.label` and `.done`).
>
> **MIG-015 phase status**:
>
> | Phase | Scope | Status | Commit |
> |---|---|---|---|
> | §1A | Rust helpers (count + chunked) | Done | `0ca7e64` |
> | §1B | init_db defers; async task wired | Done | `df0bf87` |
> | §1C | Frontend strip + 15-locale i18n | Done; **awaiting Boss test** | (this commit) |
> | §1D | Three-agent audit | Pending | — |
>
> The Boss test for §1C verifies: (a) installing the new MSI on a library with a manually-rolled-back schema version produces a fast first paint with the strip visible; (b) the completed counter climbs steadily; (c) the strip self-hides 4 seconds after `done`; (d) crash recovery resumes correctly via the WHERE-clause filter.
>
> **Architect doc**: `lab/reports/PJ-001-CHUNKED-V2-SENTINEL-ARCHITECT.md`. **Plan**: `lab/reports/PJ-001-CHUNKED-V2-SENTINEL-PLAN.md`.

**Version 1.45 | 2026-05-06**

> **What changed in v1.45** (same day as v1.44; MIG-015 opens): MIG-015 (PJ-001) starts — chunked v2 sentinel migration with progress UI. §1A lands the Rust helpers (`count_pending_v2_sentinel_rows`, `sentinel_bigram_rows_chunked`) next to the existing `sentinel_bigram_rows`; no behaviour change yet.
>
> **Why MIG-015 exists.** The MIG-013 v2 sentinel migration converts every `term_vocab` bigram row's `bridge_concept_id` from NULL → `'-'`. On Boss-equivalent libraries (~5.7M rows) the bulk UPDATE blocks boot for 30–90 sec with no UI feedback. Boss is past it; new users with pre-MIG-013 backups would hit it once and see a frozen splash. PJ-001 closes this gap.
>
> **Design (Option C, approved):** defer the v2 step off the boot critical path; spawn a one-shot async task that runs the chunked migration with progress emit. 100k rows per chunk. Tauri event channel `migration:term_vocab_v2`. Frontend status-bar strip in a new `.sb-center` group. All 15 locales updated upfront (no PJ-014 deferral). Crash-recoverable by construction.
>
> **Phase rollout** (mini-MIG):
>
> | Phase | Scope | Status | Commit |
> |---|---|---|---|
> | §1A | Rust helpers (count + chunked) | Done | (this commit) |
> | §1B | init_db defers; async task wired | Pending | — |
> | §1C | Frontend strip + 15-locale i18n | Pending; Boss-test gate | — |
> | §1D | Three-agent audit | Pending | — |
>
> **Architect doc**: `lab/reports/PJ-001-CHUNKED-V2-SENTINEL-ARCHITECT.md`. **Plan**: `lab/reports/PJ-001-CHUNKED-V2-SENTINEL-PLAN.md`.
>
> **Next after MIG-015**: PJ-006 (Living Link Architecture P2–P5 — multi-MIG, the link-side lifecycle work that PJ-007 unblocked). Eisa queued it for after PJ-001 closes.

**Version 1.44 | 2026-05-06**

> **What changed in v1.44** (same day as v1.43; MIG-014 closes): §2F three-agent audit complete. Audit report at `lab/reports/MIG-014-NOTE-STAGE-AUDIT.md`.
>
> **Audit findings:**
> - Invariant agent: PASS — all 10 invariants hold (LIVING_LINK_BASELINE.length === 6, single-arg lookupStageEmoji, Law 2.7 satisfied, M11 zero-diff intact).
> - Drift agent: 3 P0 + 2 P1, all `[pre-existing]` — fixed in close-out commits.
> - Migration-path agent: PASS for P0/P1; 2 P2 + 4 P3 logged as memory follow-ups (`project_mig014_audit_p2_p3_followups.md`).
>
> **Close-out fixes (P0 — write paths still emitting dropped Zettelkasten values):**
> - `src/lib/components/FocusPane.svelte` — Promote button **removed entirely** per Eisa's Option B. `onexit` simplified from `(promote?: string) => void` → `() => void`. Caller in `+layout.svelte` simplified. `focusPane.promote` i18n keys deleted (en + ar).
> - `src/lib/components/ExpressionForge.svelte` — composition note now writes `stage: maturity` (was `synthesis`).
> - `src/lib/components/SenseMakingCanvas.svelte` — canvas-promoted note now writes `stage: growth` (was `permanent`).
>
> **Close-out fixes (P1 — read paths missing `spark` + `archived` typo):**
> - `src/lib/components/KnowledgeHealthDashboard.svelte` — Lifecycle Cards now use all 6 baseline keys; `archived` → `archival`; `spark` added.
> - `src-tauri/src/search.rs` — `lifecycle` aggregation buckets aligned with `LIVING_LINK_BASELINE`. DB enum stays `'archived'` for back-compat; bucket key uses `archival`.
>
> **MIG-014 STATE — CLOSED.** PJ-007 (Note Stage Taxonomy) shipped via the per-note dash-encoded model. §2A → §2F complete. The §1A → §1D commits stay as the iteration record per Eisa's call.

**Version 1.43 | 2026-05-06**

> **What changed in v1.43** (same day as v1.42; MIG-014 §2E ships): the help + User Manual rewrite for the new Stages model lands. Eisa confirmed §2C+§2D Boss test PASSED after the Law 2.7 architectural fix.
>
> **Doc updates:**
>
> - `docs/User Manual.md` §18.6 — "Externalization Engine" rewritten as "Stages — the Living Link lifecycle". Six fixed lifecycle stages (Spark / Birth / Growth / Maturity / Dormancy / Archival) replace the old Zettelkasten 4 (Fleeting / Literature / Permanent / Synthesis). Per-note custom-term suffix model (`spark-concept`, `birth-concept`, …) documented with Mode A / Mode B dropdown explanation.
> - `docs/help.uConstellation.World/Cognitive Engine/Cognitive Engine.md` Feature 6 — same rewrite.
> - `docs/help.ar/User Manual.md` §18.6 — Arabic equivalent (الشرارة / ولادة / نمو / نضج / سُبات / أرشفة).
> - Multi-Lens "By Stage" reference updated in both User Manuals to point at the Living Link lifecycle instead of the old four-stage Externalization Engine.
> - 13 other locales' User Manuals queued via PJ-014 backfill (de / es / fa / fr / he / hi / ja / ko / pt / ru / tr / ur / zh).
>
> **Old Zettelkasten values still display** via `LEGACY_ZETTELKASTEN_EMOJI` for any pre-MIG-014 notes; they aren't promoteable in the new chain.
>
> **MIG-014 §2 status**: §2A → §2E shipped. §2F (three-agent audit) is the only remaining phase before MIG-014 closes.

**Version 1.42 | 2026-05-06**

> **What changed in v1.42** (same day as v1.41, after the §2C+§2D Boss-test fix sequence): adds **Law 2.7 — Single source of truth: properties have one parent** to `Constellation Development Laws v1.4.md` (NEW alongside v1.3).
>
> **Why it landed.** Three patches in a row failed to keep the breadcrumb / Properties / file tree surfaces in sync during the MIG-014 §2C+§2D Boss test. Eisa: "Enough patching." Root cause: three components each held a local `$state` copy of the stage value; each surface updated through a different path. Fixes re-aligned two surfaces while leaving the third drifting. The architectural fix made `currentStage` in NotePane a `$derived` of the prop instead of a local `$state` mirror, removed the local-mutation lines on the promote/demote click handlers, and stripped the `onstagechange` local-setter — every surface now derives from the on-disk content (proxied by `openTabs[id].content`). One source, one update path through `handlePromote → writeNote → openTabs.update → parsed re-derives → stage prop re-passes → derived chain refreshes`.
>
> **Generalisation.** The rule isn't stage-specific. Title, tags, links, body — every first-class property the user can edit through more than one surface — has one canonical owner; UI surfaces are subfunctions that derive. Local `$state` mirrors are forbidden. Edit buffers (input typing), UI-only state (dropdown open/closed), and caches with clear invalidation paths are the named exceptions.
>
> **MIG-014 §2 status update**: §2A → §2D shipped; the §2D fix sequence (commits `bb7a6ef → e3a97a1`) cleared the §2C+§2D Boss-test failures. Awaiting Eisa retest before declaring §2C+§2D passed and moving on to §2E (help + User Manual) and §2F (audit).

**Version 1.41 | 2026-05-06**

> **What changed in v1.41** (next day after v1.40; MIG-014 mid-cascade): the Pending Jobs document, the Constellation Development Laws bumped 1.0 → 1.3, the NotePane Specs distilled from 121 commits, the MoCh convention added as Standing Order #7, two new top-principal feedback memories, MIG-014 opened with the Note-Stage Taxonomy migration: Architect + Plan + §1A → §1D iteration (proven-wrong model) + Stages Concept Paper v1.0 → v1.2 + Plan v2 → v4 + §2A → §2D shipped (correct model). PJ-007 status: in-build, awaiting Boss test on the §2C+§2D combined gate.
>
> **Two key process events** that produced new durable rules:
>
> 1. **MoCh — Minutes of Chating** (Boss-directed 2026-05-06). Every ~3 hours of direct chat, write a fresh file at `docs/MoCh/MoCh-YYYY-MM-DD-HHMM.md` recording Boss ↔ Claude interaction (questions, steers, decisions, outputs) — distinct from the session log (which captures *what shipped*). Recorded as **Standing Order #7** in CLAUDE.md and as feedback memory `feedback_minutes_of_chating.md`. First file: `docs/MoCh/MoCh-2026-05-06-0900.md`.
> 2. **SO #6 inline-with-commit reinforcement** (Boss-directed 2026-05-06). After v1.40 landed, ten hours of triggering changes (Laws v1.1→v1.3, Pending Jobs, NotePane Specs, MoCh convention, MIG-014 Architect/Plan/Build cascade, Stages Concept Paper, §1A→§1D, §2A→§2D) accumulated *without a single orientation bump*. Boss asked "Why do I have to remind you?" Recorded as feedback memory `feedback_orientation_inline_with_commit.md` (top-principal): orientation v-bump lands IN THE SAME COMMIT as any SO #6 trigger; no batching, no waiting for the next commit. v1.41 closes the drift.
>
> **Constellation Development Laws — v1.0 → v1.3.**
>
> - **v1.1** — Law 1.6 added (State the function in hand). [Already in v1.40 preamble.]
> - **v1.2** — Law 2.6 corrected: cUniverse moved out of the required hierarchy layer to a sibling-federation position; structure is `Universe → (Library | Folder | Note)` with cUniverse as an optional federated peer.
> - **v1.3** — Law 2.6 refined: Universe root is itself a default Library (`is_universe_notes` flag in `libraries.json`). The Universe is *not* a higher abstraction over Libraries; it is a Library with an additional federation role. Notes and folders may live directly under the Universe root.
>
> **Constellation Pending Jobs v1.1** (`docs/Constellation Pending Jobs v1.1.md`, NEW alongside v1.0). Durable project backlog. **Stable reference numbers** introduced as a top-of-document rule: each pending job carries a unique sequence number `PJ-NNN` that is never reissued, even after the job is shipped or abandoned. Numbers act as cross-document references (memory notes, session logs, commits, MoCh files all use `PJ-NNN`). Twelve jobs catalogued: PJ-001 through PJ-012. Status vocabulary: Pending / Confirmed / In-Build / Shipped / Abandoned.
>
> **NotePane Specs v1.0** (`docs/NotePane Specs v1.0.md`, NEW). Distilled from the 121-commit `NotePane.svelte` history. Twelve sections, fourteen hard invariants, twelve forbidden anti-patterns. Each statement sourced to a commit hash. **§3.5 corrected post-write** (commit `9973e65`): the breadcrumb is `[← demote] [emoji label badge] [promote →]` — promote/demote arrows + badge, NO dropdown. Earlier draft mistakenly captured the dropdown experiment from commit `6cbe87c` as final state; that experiment was undone at `90c1ea8` (§136, 2026-05-02 redesign). The current code at `NotePane.svelte:918-958` has no dropdown.
>
> **MIG-014 — Note-Stage Taxonomy** (closes PJ-007). The currently-active migration. Two model iterations:
>
> 1. **Iteration 1 (§1A → §1D, kept as iteration record)** — flat extensibility model. `CustomStage { name, emoji }` struct + `custom_stages: Vec<CustomStage>` field on UniverseMeta + 5 IPC commands. PropertyEditor combobox + custom inline dropdown + emoji chain across breadcrumb / file tree / Inspector360. Boss test surfaced multiple model failures: long promote chain doesn't scale; custom emoji adds visual noise; per-Universe scope wrong. **Stays in `main` as the iteration record per Eisa's call (don't rewrite history).**
> 2. **Iteration 2 (§2A → §2D, current)** — per-note dash-encoded model. UniverseMeta restored to pre-§1A shape. The custom term lives only as the dash suffix in each note's frontmatter `stage:` value (e.g. `stage: spark-concept`). The PropertyEditor combobox is a 6-entry mode-flip dropdown: Mode A (input empty / matches a fixed name) → 6 baselines; Mode B (custom word in input or dash suffix) → 6 paired stages. The breadcrumb chain walks the lifecycle and carries the suffix verbatim. No Universe-wide settings, no Settings panel.
>
> **Stages Concept Paper v1.0 → v1.2** captures the model evolution:
>
> | Version | Model                                                | Status     |
> | ------- | ---------------------------------------------------- | ---------- |
> | v1.0    | 2D matrix with multiple custom types                  | Superseded |
> | v1.2    | Per-note custom term, dash-encoded, 6-entry mode-flip dropdown | **Current** |
>
> Both versions kept in `docs/` as a thinking-trace per the orientation-versioning convention.
>
> **MIG-014 §2 Plan v4** (`lab/reports/MIG-014-NOTE-STAGE-PLAN-v4.md`) is the active plan. Six phases:
>
> | Phase | Scope                                                          | Status              | Commit    |
> | ----- | -------------------------------------------------------------- | ------------------- | --------- |
> | §2A   | Rust schema cleanup                                            | Done                | `2f58b8a` |
> | §2B   | Frontend store cleanup + new pure helpers                       | Done                | `59ed95c` |
> | §2C   | PropertyEditor mode-flip combobox                               | Done; Boss-test pending | `432076c` |
> | §2D   | NotePane breadcrumb chain walks within suffix                   | Done; Boss-test pending | `2c58bda` |
> | §2E   | Help + User Manual (en + ar)                                   | Pending             | —         |
> | §2F   | Three-agent audit                                              | Pending             | —         |
>
> §2C + §2D are paired Boss-test gates; combined MSI build in flight at v1.41 commit time.
>
> **Five top-principal feedback rules now in memory** (per `~/.claude/projects/.../memory/MEMORY.md`):
>
> 1. `feedback_dont_make_things_up.md` (BASIC RULE — top of all rules).
> 2. `feedback_secure_dont_muddle.md` (validate every change against full architecture).
> 3. `feedback_tutorial_tests_and_cascade.md` (every test as tutorial; plan = build approval).
> 4. `feedback_minutes_of_chating.md` (MoCh — NEW 2026-05-06).
> 5. `feedback_orientation_inline_with_commit.md` (orientation v-bump in the triggering commit — NEW 2026-05-06, post-Eisa-correction).
>
> **What's open at v1.41 commit time**:
>
> - MIG-014 §2C+§2D MSI build in flight; Boss-test tutorial pending on completion.
> - §2E help + User Manual rewrite (en + ar; PJ-014 backfill).
> - §2F three-agent audit.
> - PJ-014 — 13-locale i18n + User Manual backfill (carried).
> - Carried-over: LinkLifecycle dedupe (Option B approved, deferred until post-CE); pre-MIG-013 backups hit blocking v2 sentinel migration.

**Version 1.40 | 2026-05-05**

> **What changed in v1.40** (same day as v1.39; MIG-013 closes): **MIG-013 §1D-C help + User Manual updates shipped, §1E three-agent audit complete, two cleanup P1s applied, one P1 deferred with documentation. MIG-013 (CTSE Bridge Adapter) is closed.**
>
> **Plus a new Foundational law**: **Law 1.6 — State the function in hand** (Boss-directed 2026-05-05 after the §1D wrong-target post-mortem). At task start, every fresh session, every surface pivot, and every correction, name the function in hand in one line. Predecessor Lookup, Stop-on-Correction, Testing Instructions, and Migration Rule all read against this anchor. Without it they float. Recorded as a top-principal in `CLAUDE.md` and as Law 1.6 in `docs/Constellation Development Laws v1.1.md` (NEW alongside v1.0).
>
> **§1E audit findings** (`lab/reports/MIG-013-CTSE-AUDIT.md`):
>
> | audit | P0 | P1 | result |
> |---|---:|---:|---|
> | Invariant (Rules 1-4 + 8, M11 zero-diff, IPC, tokenizer symmetry, tests) | 0 | 0 | PASS |
> | Drift (16 retired surfaces, dead schema, dead i18n, stale comments) | 0 | 2 | PASS WITH P1 |
> | Migration-Path (4 scenarios: fresh / pre-MIG-013 / interrupt / rollback) | 0 | 1 | PASS WITH P1 |
>
> **Two P1s applied in close-out** (`b66101b → 5a0be3c → 11410dd → b66101b → b5ce03a → close-out commit`):
>
> 1. **P1-D1**: dead `settings.index.semanticSearch` i18n block removed from all 15 locale files. ~300 dead lines cut. The block was the abandoned MIG-012 toggle UI strings; the toggle UI itself was removed in §1D-B but the keys hung on. Cleaned via Python script; verified zero references in `src/`.
> 2. **P1-D2**: four stale Rust doc comments in `src-tauri/src/search.rs` (lines 88+, ~455+, ~483+) referencing the retired `ctse_run_backfill` / "the backfill" as live writers. Rewritten to reflect post-§1D Option B dead-schema status — the column survives as forward-compat; the v2 sentinel migration is now defensive-only.
>
> **One P1 deferred with documentation**:
>
> 3. **P1-M1**: the v2 sentinel migration's bulk UPDATE blocks boot for tens of seconds to minutes on pre-MIG-013 DBs (~5.7M bigram rows on Boss-equivalent libraries) with zero user feedback. Boss has already completed the migration on the §1D-D binary; new pre-MIG-013 backups would hit it once. Memory entry filed: `project_mig013_v2_migration_blocking_boot.md`. Fix is bounded (chunk + Tauri progress events) but ships as a focused mini-MIG before any v1.0 release — out of MIG-013 scope.
>
> **§1D-C help + User Manual updates** (en + ar):
>
> - `docs/help.uConstellation.World/Index/Index.md` — replaces the trailing "planned next step" caveat with a full **Cross-language Filter — `≈ similar`** section. Three-layer filter breakdown (literal substring / bridge `via {lemma}` / concept `≈ similar`), plain-language mechanism for layer 3, expected-misses note.
> - `docs/User Manual.md` — same shape (English).
> - `docs/help.ar/User Manual.md` — same shape (Arabic, with localized variable names: `≈ مشابه`).
> - 13 other locale User Manuals deferred to the existing `project_user_manual_13_locales_backfill.md` queue.
>
> **What MIG-013 shipped (final state):**
>
> - `bridge_vectors` — 30 MB baked asset (20K M11 concepts × 384 f32 vectors), built once at compile time via `cargo run --bin build_concept_vectors --release`. Loaded once at runtime via `OnceLock`.
> - `ctse::hooks` — write-time `term_vocab` ledger maintenance via `on_note_indexed` / `on_note_deleted`. Per-save tokenize → signed delta → upsert. Pure local bookkeeping, no ONNX in the write path.
> - `ctse::search::ctse_search_terms_by_concept` — query-time concept expansion for the IndexPanel filter `≈ similar` row. Embeds query, finds top-K M11 concepts, expands to multilingual lemmas via the `concept_lemmas()` map (built once at boot from `LexiconGraph`), tokenizes each lemma into FTS5-namespace stems, returns the subset that exists in `term_vocab`.
> - `term_vocab.bridge_concept_id` column + index + v1/v2 schema migrations — dead schema, forward-compat preserved.
> - M11 zero-diff invariant intact across the entire migration.
>
> **Architecture aligns with industry best practice** (Lucene `SynonymGraphFilter`, SQLite FTS5 Method 2, CLIR query-translation, Primo controlled-vocabulary expansion). No backfill, no first-fill, no per-library setup cost. Reacts automatically to new M11 releases.
>
> **Boss test passed** Stage 1 (function alive end-to-end on the rebuilt binary) and Stage 2 (coverage + steady-state timing). MIG-013 closes.
>
> **What's next (post-MIG-013)**: there's no immediate follow-up. Open queue items: P1-M1 mini-MIG (chunked bigram-sentinel migration), the long-deferred 13-locale User Manual backfill, and the always-on cross-language `≈ similar` setting may eventually want a kill-switch toggle if Boss wants noise control. None are urgent.

**Version 1.39 | 2026-05-05**

> **What changed in v1.39** (same day as v1.38; §1D wrong-target correction + new top-principals + Constellation Development Laws v1.0): **CTSE concept search lands in the IndexPanel filter — its correct home — replacing the SearchHub `concept` category that v1.38 wired up by mistake. Two new top-principals added to CLAUDE.md (Predecessor Lookup Rule, Stop-On-Correction Rule) so the same drift can't recur. New durable artifact: `docs/Constellation Development Laws v1.0.md` — distilled from CLAUDE.md, every prior orientation version, every session log, every LL entry, and the Boss-feedback record.**
>
> **The §1D wrong-target incident**. After the Option B pivot (v1.38), I shipped the cross-language search wired to **SearchHub** (the global Ctrl+Shift+F modal). MIG-012's predecessor `searchTermsSemantic` had lived in the **IndexPanel filter dropdown** with a `≈ similar` badge. Four explicit pointers (Settings flag named `index.semanticSearchEnabled`, IndexPanel was the actual call site, MIGs 010/011/012 all operated on IndexPanel, the Settings progress strip was under Settings → Index) all said "Index panel"; I read past every one and shipped the wrong target. Boss correction: "SearchHub? But we are working on the Index!" After a follow-up "How can you be confused?" — explicit acknowledgement that the orientation/SO scaffolding exists precisely to prevent this kind of drift, and that I bypassed every layer.
>
> **Two new top-principals in CLAUDE.md** (Boss-approved 2026-05-05):
>
> - **Predecessor Lookup Rule** — before removing/moving/replacing any user-facing feature, IPC, settings entry, or UI wiring, write a `Predecessor → Replacement` entry into the day's session log: *where it lives now* (file path, function name, settings path, predecessor MIG number) → *where its replacement goes* (default: same place; different place ONLY with explicit Boss approval) → *what gets cut and kept*. Verified against the orientation doc, not memory. The entry comes BEFORE any code edit. Now active for §1D-D and onward.
> - **Stop-On-Correction Rule** — when Boss says "wrong target", "you're confused", "no", "unacceptable", or any equivalent course-correction, STOP all in-flight code edits, list everything changed since the last explicit Boss approval, state the corrected understanding, wait for "proceed". No pivot-and-power-through. Overrides Plan-Approval-Equals-Build-Approval (a correction revokes the cascade approval).
>
> **Constellation Development Laws v1.0** (`docs/Constellation Development Laws v1.0.md`, NEW). A durable, higher-order companion to CLAUDE.md. Five Parts:
>
> | Part | Theme | Sample laws |
> |---|---|---|
> | I | Foundational | 1.1 Don't make things up · 1.2 User is Boss not lab assistant · 1.5 Cross-check against proven methods |
> | II | Engineering | 2.1 Fast software is the best software · 2.2 The Migration Rule · 2.3 Architectural-impact review · 2.4 Constraint as design |
> | III | Process | 3.1 Plan = Build approval · 3.2 Predecessor Lookup · 3.3 Stop on Correction · 3.4 Standing Order · 3.5 Tutorial tests · 3.7 Verify binary · 3.8 Walk through writes |
> | IV | Communication | 4.1 Plain language · 4.2 Don't muddle · 4.3 Reuse don't duplicate · 4.4 One-sentence end-of-turn |
> | V | Recovery | 5.1 No more than 3 patches · 5.2 Backup at milestones · 5.3 State-of-standing before pivot · 5.4 Avoid destructive shortcuts |
>
> Plus **Appendix A: dated timeline of canonical violations** — six entries through 2026-05-05. Each violation produced a law. The Laws doc is updated frequently (new top-principal added → new law; recurring failure pattern crystallizes → new law; Boss correction → new law). Each version is a NEW file; older versions stay as historical record. Same convention as the orientation doc.
>
> **§1D-D — IndexPanel restoration (this commit)**. The Predecessor Lookup entry was written into today's session log before any code edit, per the new Law 3.2:
>
> - **Predecessor**: MIG-012 `searchTermsSemantic` per-keystroke effect in `src/lib/components/IndexPanel.svelte`, gated on `$appSettings.index.semanticSearchEnabled`, populating `semanticMatches` and rendering `≈ similar` badge in the filter dropdown.
> - **Replacement**: SAME PLACE (IndexPanel filter dropdown). The badge UX is restored byte-for-byte. The per-keystroke effect now calls `ctseSearchTermsByConcept` over the new `ctse_search_terms_by_concept` Tauri command. Toggle gate omitted (always-on) per Law 2.4 — no per-library setup cost with CTSE, no reason to add a toggle to disable a now-free feature.
> - **Cut**: SearchHub `concept` category integration (reverted). `ctse_search_by_concept` (note-returning) → renamed and reshaped to `ctse_search_terms_by_concept` (term-returning, IndexPanel-shaped). `ctseSearchByConcept` / `CtseConceptHit` frontend types removed. The `searchHub.concept` and `searchBadges.concept` i18n keys stay (no longer used; harmless dead keys, will GC later).
> - **Kept**: §1C hook simplifications. `term_vocab.bridge_concept_id` dead schema (forward-compat). `concept_lemmas()` in-memory map. `bridge_vectors` matrix asset. The renamed `ctse::search` Tauri command.
>
> **The new read path** (Index panel filter, `≈ similar` row):
>
> 1. User types "knowledge" in the IndexPanel filter input.
> 2. Layer 1 (literal substring) — match the user's query against term names. Always on. Existing.
> 3. Layer 2 (cross-language bridge, MIG-010/MIG-011) — query M11 for cross-language equivalents, surface those terms with a `via {lemma}` badge. Existing.
> 4. **Layer 3 (CTSE concept expansion, NEW in §1D-D)** — embed the query, find top-K M11 concepts, expand each to multilingual lemmas, tokenize each lemma through `fts5_tokenizer::tokenize_to_vec` to get FTS5-namespace stems, look up which stems exist in `term_vocab`. Surface those terms with the `≈ similar` badge. Per-keystroke debounced (300 ms; CLAUDE.md Rule 3). Always on (no toggle).
>
> Per-query cost: ~50 ms e5 inference + ~5 ms cosine k-NN + sub-ms in-memory map lookup + sub-ms SQL `term IN (...)` lookup. Reacts automatically to new M11 releases.
>
> **What's pending after this commit**: Boss reinstalls the rebuilt binary and tests Stage 1 of the IndexPanel cross-language flow (open Index panel → type "knowledge" → expect Arabic terms like "معرف" to appear with the `≈ similar` badge in the dropdown). Then §1D-C (help files + User Manual updates) and §1E (three-agent audit).

**Version 1.38 | 2026-05-05**

> **What changed in v1.38** (same day as v1.37; mid-§1D Boss test, second pivot): **the entire CTSE backfill / first-fill pipeline is retired. After two mid-test fixes (bigram-explosion v1.37, then a half-million-stem slow-path projection at 2+ hours), Boss asked `cross-check this with proven methods used by similar coding communities`. Five parallel WebSearches against Lucene, Elasticsearch, SQLite FTS5, CLIR research, and library-platform documentation surfaced an unanimous industry pattern: query-time concept/synonym expansion, NOT index-time term tagging. Lucene retired index-time `SynonymFilter` for query-time `SynonymGraphFilter` in 2017; SQLite FTS5 docs explicitly list query-time expansion as Method 2; CLIR canonical technique is query-translation; Primo / Ex Libris controlled-vocabulary expansion is at search time. CTSE now follows the same pattern.**
>
> **What that means concretely**: the `ctse_search_by_concept` Tauri command is now self-contained. It embeds the user query, finds top-K M11 concepts via cosine k-NN against the baked 20K-concept matrix, expands each concept to its multilingual lemmas via an in-memory `concept_id → [lemmas]` map (built once at boot from `LexiconGraph`, ~5 MB, ~10 ms), unions and deduplicates the lemmas, and runs an FTS5 OR-clause MATCH against `notes_fts`. No `term_vocab.bridge_concept_id` reads, no per-term backfill, no first-fill, no boot-time wait. Boss's library doesn't need to wait through any concept-resolution job to test cross-language search — the rebuilt binary is immediately functional.
>
> **CLAUDE.md Working Agreement #5 added** (this commit): *"Cross-check every non-trivial fix or design against proven methods before applying it."* Before locking in any subsystem-crossing fix or feature, run parallel WebSearch queries against how mature systems and communities solve the same problem (Lucene, Elasticsearch, SQLite, vector DB practice, library science, IR/CLIR research, PKM tools), compare honestly, surface both options to Boss with the tradeoffs that matter, and pick the battle-tested pattern over the inventive one. Canonical violation: §1D-A backfill was an inventive solution to a problem the dominant industry pattern simply doesn't have.
>
> **Code surface deleted by Option B**:
> - `src-tauri/src/ctse/backfill.rs` (entire file: `ctse_run_backfill`, `ctse_cancel_backfill`, `ctse_backfill_status` Tauri commands).
> - `src-tauri/src/ctse/first_fill.rs` (entire file: `ctse_first_fill`, `ctse_first_fill_status`, `ctse_cancel_first_fill` Tauri commands).
> - `ctse::resolve_term_pure`, `ctse::resolve_term_to_concept`, `ctse::resolve_term_multilang`, `ctse::fast_path_concept_id` (the resolver helpers — orphaned).
> - `ctse::hooks::fast_path_resolve_new_terms` (the per-save concept resolution).
> - 7 frontend store wrappers (`ctseFirstFill[Status]`, `ctseRunBackfill`, `ctseBackfillStatus`, `ctseCancel*`) + `CtseFillProgress` / `CtseFillPhase` / `CtseFillStatus` types + `ctseFillStatus` writable.
> - The `+layout.svelte` boot-time auto-fire `$effect` + the bottom-of-viewport status-bar strip + 50 lines of associated CSS.
> - The `ctse.firstFillProgress / firstFillDone / backfillProgress / backfillDone / cancelled` i18n keys (en + ar).
>
> **Code surface kept**:
> - `ctse::hooks::on_note_indexed` and `on_note_deleted` (term_vocab count maintenance — the Index panel still consumes `term_vocab`).
> - `ctse::search::ctse_search_by_concept` Tauri command (the read path).
> - `bridge_vectors` module (the 20K-concept matrix asset and its loader).
> - `term_vocab.bridge_concept_id` schema column + index + the v1/v2 migrations (dead but idempotent — preserved for forward-compat in case a future "deep concept tagging" feature wants to populate it again).
> - SearchHub frontend wiring for the `concept` category, the `searchHub.concept` and `searchBadges.concept` i18n keys.
>
> **Net diff**: ~580 lines removed, ~80 added. Dramatically simpler architecture. Same query latency (~80 ms end-to-end). Same cross-language coverage (in-vocabulary terms, ~20K M11 concepts × 15 languages each). Reacts automatically to new M11 releases — no rebuild-the-concept-index step ever required.
>
> **§1C/§1D-A/§1D-B status under Option B**:
> - §1C (`5aac7fa`) — schema + write-time hook + retired init_term_embeddings: STILL VALID. The schema column stays as dead-but-harmless; the hook stays simplified.
> - §1D-A (`7b52f1d`) — first-fill + concept search backend: SEARCH PATH SUPERSEDED, first-fill module deleted. Tauri search command rewritten.
> - §1D-B (`0ac12eb`) — frontend wiring + Settings cleanup: SearchHub `concept` category + Settings cleanup BOTH STAY. The boot-time auto-fire + status-bar strip + frontend store wrappers are deleted.
> - §1D bigram-explosion fix (`9aba974`) — schema v2 bulk-sentinel: STAYS as dead-but-idempotent migration. The follow-up SQL filter in `ctse::backfill::next_batch` is moot since the file no longer exists.
>
> **What's pending after this commit**: Boss reinstalls the rebuilt binary and runs the combined Stage 1+2 test (now condensed because there's no backfill phase to wait through): open SearchHub, type a cross-language query, expect concept-category hits in the other script. Then §1D-C (help files + User Manual updates) and §1E (three-agent audit).

**Version 1.37 | 2026-05-05**

> **What changed in v1.37** (same day as v1.36; mid-§1D Boss test): **MIG-013 §1D-A and §1D-B shipped + bigram-explosion fix landed. The first §1D Boss-test launch hit a near-freeze: `term_vocab` had 5.73 million NULL `bridge_concept_id` rows on Boss's 7,639-note library — ~50K real stems plus ~5.68M bigrams (every adjacent stem-pair across all notes, joined by `BIGRAM_SEP` = U+001F). The backfill correctly skipped each bigram in microseconds but the sheer volume (11K+ batched UPDATE transactions) saturated the SearchState mutex, hung the WebView, and projected at ~2 hours wall-clock to finish.**
>
> **Working Agreement #4 lesson**: the architect doc (`MIG-013-CTSE-ARCHITECT-v2.md §3.3`) said "long-tail proper nouns, code identifiers" without quantifying the bigram contribution. The previous `init_term_embeddings` flow filtered terms with `total_count >= 20` before any work — implicitly excluding bigram noise. When §1C removed `init_term_embeddings`, that load-bearing filter was removed without realizing it. Should have run `SELECT COUNT(*) FROM term_vocab` against Boss's library before shipping §1C; did not. Logging this and adding pre-ship counter-measure to the migration checklist (see §17 / Lessons-Learned candidate LL-029).
>
> **§1D-A (`7b52f1d`) — first-fill + concept-search backend**:
> - `ctse_first_fill` Tauri command (`src-tauri/src/ctse/first_fill.rs`) — chunked-transaction walk over `note_meta.body_text`, re-fires `on_note_indexed(old=None)` per row inside 50-note transactions. Resumable via the shared `term_embed_cancel` atomic. Companion `ctse_first_fill_status` returns true iff `term_vocab` is empty AND `note_meta` has body content (the frontend gate). Cancellation via `ctse_cancel_first_fill`. Emits `ctse-firstfill-progress` events.
> - `ctse_search_by_concept` Tauri command (`src-tauri/src/ctse/search.rs`) — embeds the query, picks top-K M11 concepts above a tunable cosine threshold (`DEFAULT_MIN_SCORE = 0.55`, `CONCEPT_TOP_K = 10`), expands to every term_vocab row whose `bridge_concept_id` matches, builds an FTS5 OR-clause MATCH (200-term cap, phrase-quoted), returns notes with snippets. Cross-language for free. Bigram terms (containing U+001F) filtered out of the OR clause. Per-call cost: ~50 ms e5 inference + ~5 ms cosine sweep + sub-ms term lookup + FTS5 MATCH.
>
> **§1D-B (`0ac12eb`) — frontend wiring + Settings cleanup**:
> - `store.ts` — adds `ctseSearchByConcept`, `ctseFirstFill[Status]`, `ctseRunBackfill[Status]`, `ctseCancel[FirstFill|Backfill]` IPC wrappers + `CtseConceptHit` / `CtseFillProgress` / `CtseFillPhase` / `CtseFillStatus` types + the module-scoped `ctseFillStatus` writable. Removes `searchTermsSemantic`, `initTermEmbeddings`, `cancelTermEmbeddings`, `termEmbeddingStatus`, `termEmbedProgress`, `TermSimilarity`, `TermEmbedProgress`.
> - `+layout.svelte` — boot-time `$effect` after `graphReady` listens for both progress streams (push payload into `ctseFillStatus`), calls `ctseFirstFillStatus` → `ctseFirstFill` (if needed), then `ctseBackfillStatus` → `ctseRunBackfill` (if NULLs remain). Status-bar strip is a fixed bottom-of-viewport banner subscribed to `$ctseFillStatus`, hides 4s after `done`. Listeners cleaned via `cleanupFns`.
> - `SettingsModal.svelte` — removed the entire MIG-012 semantic-search block (toggle UI, progress strip, Rebuild Term Embeddings button, $effect / listen / UnlistenFn / untrack auto-trigger). The `index.semanticSearchEnabled` flag stays in the schema with no readers (future GC will drop it).
> - `IndexPanel.svelte` — removed the per-keystroke `searchTermsSemantic` effect + `semanticMatches` Map + `≈ similar` badge + `semanticSearchEnabled` prop. Pure literal substring + bridge expansion browsing now.
> - `SearchHub.svelte` — added `concept` category alongside the six existing ones. `triggerSearch` calls `universalSearch` and `ctseSearchByConcept` in parallel; concept hits are mapped to `ConstellationSearchResult` and stuffed into `response.concept` for the existing rendering loop. CTSE failures degrade gracefully (warn-and-continue). Cyan "≈" badge.
> - `i18n` — en/ar add `searchHub.concept`, `searchBadges.concept`, and `ctse.{firstFill,backfill}{Progress,Done}/cancelled` keys. 13 other locales fall back per the established backfill pattern.
>
> **Cargo.toml fix (`46b3675`)** — added `default-run = "constellation"` to disambiguate Cargo's main binary now that §1A introduced a second `[[bin]]` (`build_concept_vectors`). `cargo build --lib` (used during all §1A–§1D verifications) didn't catch it; full Tauri bundling did.
>
> **Bigram-explosion fix (this version)** — bumps `TERM_VOCAB_BRIDGE_SCHEMA_VERSION` 1 → 2. New `sentinel_bigram_rows()` helper in `search.rs::init_db` runs once on the v1→v2 migration: a single bulk `UPDATE term_vocab SET bridge_concept_id = '-' WHERE bridge_concept_id IS NULL AND term LIKE '%' || CHAR(31) || '%'` that turns 5.68M useless backfill candidates into pre-sentinelled tombstones in sub-second wall-clock. `ctse::backfill::next_batch` and `count_null_rows` also gain `AND term NOT LIKE '%' || CHAR(31) || '%'` filters as belt-and-suspenders against future writes. After this fix:
>
> | metric | before fix | after fix |
> |---|--:|--:|
> | term_vocab rows visible to backfill (Boss's library) | 5,729,974 | ~50,000 |
> | wall-clock to drain backfill | ~2 hours, UI hung | ~5–10 min, UI responsive |
> | bigram rows in `term_vocab` | unchanged (~5.68M, kept for FTS5) | unchanged (~5.68M, all sentinelled) |
> | mutation pressure on the SearchState mutex | 11K+ batched UPDATE tx | ~100 batched UPDATE tx |
>
> **What's pending (next session)**: (a) Boss re-runs Stage 1 of the §1D test on the rebuilt binary (the lock-up should be gone, the strip should show "~50,000" not "~5,700,000"). (b) Stage 2 of the test — actual cross-language SearchHub query. (c) Help files + User Manual updates (§1D-C). (d) Three-agent §1E audit (invariants, drift, migration-path).

**Version 1.36 | 2026-05-05**

> **What changed in v1.36** (same day as v1.35; cascade continues from §1B → §1C): **MIG-013 §1C shipped — write-time hooks + slow-path backfill scaffold + retirement of the legacy `init_term_embeddings` Tauri pipeline. Term vocabulary is now maintained on every note save via the same FTS5 tokenizer that backs `notes_fts`; new terms get a fast-path M11 concept lookup (microseconds, no ONNX) immediately, and the slow-path resolution for misses runs in a separate Tauri command (resumable, cancellable, batched). The whole `term_embeddings` table + bulk `populate_term_vocab` bootstrap is gone.**
>
> **What landed in §1C**:
> - **Schema migration** — `term_vocab.bridge_concept_id TEXT` column + supporting index, gated by `schema_versions.term_vocab_bridge = 1`. Idempotent ALTER TABLE; fresh DBs and existing DBs converge.
> - **Write-time hook** (`src-tauri/src/ctse/hooks.rs`, NEW) — `on_note_indexed(conn, path, old_body, new_body)` tokenizes both bodies via `fts5_tokenizer::tokenize_to_vec`, computes signed per-term `(total_delta, doc_delta)`, upserts `term_vocab`, and fast-path-resolves M11 concept ids for newly-introduced terms. `on_note_deleted(conn, path, body)` subtracts contributions; tombstones (zero counts) are kept so revival is free. **No ONNX in the write path** — slow-path is the backfill's job. 1 MiB body cap matches the prior `BODY_CAP_BYTES` precedent. Stopword set cached at module level via `OnceLock`.
> - **Wire-in** — `search.rs::reindex_single_note` and `reindex_delete_note` now read `note_meta.body_text` once before and once after `index_note`, then call the hook. Hook errors are logged but never fail the reindex (term_vocab is a derived view; file + note_meta are the sources of truth).
> - **Slow-path backfill** (`src-tauri/src/ctse/backfill.rs`, NEW) — three Tauri commands: `ctse_run_backfill`, `ctse_cancel_backfill`, `ctse_backfill_status`. Walks `WHERE bridge_concept_id IS NULL ORDER BY total_count ASC LIMIT 500` (TF-IDF descending — rarest first → search becomes useful early). Per-term resolution via new `ctse::resolve_term_multilang(app, term)` (15-language fast-path FST sweep, then e5 inference + cosine k-NN). Sentinel `'-'` for "tried and failed" so re-runs visit only new NULL rows. Resumable per batch transaction. Cancellation reuses `EmbeddingState.term_embed_cancel: AtomicBool` (orphaned by §1C-5). Emits `ctse-backfill-progress` events.
> - **Retired** — `init_term_embeddings` / `cancel_term_embeddings` / `search_terms_semantic` / `term_embedding_status` Tauri commands; `populate_term_vocab` (the Phase 1 rayon bootstrap); `blob_to_vec` (orphan); `TermEmbedProgress` and `TermSimilarity` payload structs; the `term_embeddings` CREATE TABLE. The `term_vocab` comment in `init_db` updated to point at the §1C write-time-derivation maintenance path.
> - **Shared helpers in `ctse/mod.rs`** — new `pub fn fast_path_concept_id(graph, term)` (multi-language lookup; bigrams skipped) and `pub fn resolve_term_multilang(app, term)` (fast-path-then-slow-path; used by backfill).
>
> **Verification — 9 ctse tests + 6 bridge_vectors tests green**. New ctse::hooks tests cover first-time index (insert + fast-path), idempotent resave (zero delta), edit (signed delta), delete (tombstone), and bigram tokens (stay NULL). M11 zero-diff invariant holds: `git diff src-tauri/src/lexicon/` empty.
>
> **Known gap (resolved in §1D)**: the SettingsModal frontend still calls the four removed Tauri commands. Toggling semantic search ON in Settings throws at runtime on the first IPC. **No Boss test in this gap** — §1D follows immediately and removes the call sites + the `termEmbedProgress` writable store + the `confirmDialog` for "Rebuild Term Embeddings".
>
> **Known gap on existing libraries**: `term_vocab` rows from the prior `populate_term_vocab` bootstrap have `doc_count = 0` (the bulk loader skipped that field). Cosmetic only — the backfill cursor is the NULL filter on `bridge_concept_id`, ordering is `total_count` (correct from before).
>
> **What's pending (§1D, next)**: auto-trigger first-fill on boot when `term_vocab` is empty (walks `note_meta.body_text` and re-fires `on_note_indexed`); auto-trigger `ctse_run_backfill` on boot when NULL rows exist; status-bar progress strip subscribed to `ctse-backfill-progress`; new `ctse_search_by_concept` Tauri command + frontend wiring; full Settings UI cleanup (kill the four old IPCs' call sites, `termEmbedProgress` writable, "Rebuild Term Embeddings" confirm dialog); update help files + User Manual (15 languages) for cross-language Constellation Sight. **First Boss-testable gate fires at §1D** (cross-language semantic search). **§1E**: three-agent audit per Migration Rule §4.

**Version 1.35 | 2026-05-05**

> **What changed in v1.35** (next day after v1.34): **MIG-013 §1A + §1B shipped — full architectural pivot of the term-vocabulary semantic-search pipeline. Per-library term-embedding is retired; the 20K M11 controlled-vocabulary concepts are embedded once at build time and shipped with the binary as a 30 MB asset. Library size becomes irrelevant for semantic search.**
>
> **Why the pivot**: MIG-012 fix-1 / fix-2 (v1.34) shipped the auto-trigger + status UI, but on Boss's 7,635-note multi-script library the underlying `init_term_embeddings` flow stalled at note 601/7635 (Phase 1 single-thread bootstrap), then later at total=0 for 7+ minutes (Phase 1.5 batch-ONNX with heartbeat-after-fetch). Three SME audits (parallel-systems, library/IR, application-architecture) independently concluded **the unit being embedded was wrong** — LCSH/MeSH/AAT (the canonical IR pattern since 1909) embed the controlled vocabulary, not the patron's corpus. M11 is exactly that controlled vocabulary. Boss directive 2026-05-05: "Go for A. But don't touch the M11's ~20K concepts."
>
> **Hard constraint** (Boss): `lexicon/` source files have a zero-line diff at every CTSE commit. Verified mechanically by `git diff src-tauri/src/lexicon/` returning empty. CTSE reads M11; never writes to it.
>
> **§1A shipped (`5e1c0f1`) — build-time concept-vector pipeline**:
> - New offline `[[bin]]` target `build_concept_vectors`: reads M11's seed TSV (read-only via `lexicon::parse`), picks one canonical surface form per concept (en > zh > es > fr > de > ... fallback chain), embeds with multilingual-e5-small in batches of 128 on `available_parallelism()` threads, validates per-vector L2-norms, writes asset.
> - New `src-tauri/src/bridge_vectors/` module (stub at this phase): `ASSET_MAGIC = b"CTSEBV01"`, `VECTOR_DIM = 384`. Layout: 8-byte magic + u32 count + u32 dim + concept-id table (u16 LE byte_len + UTF-8) + 4-byte-aligned f32 LE row-major matrix.
> - New `pub fn embeddings::embed_passages_standalone(model_path, tokenizer_path, texts, intra_threads, batch_size)` — builds its own ONNX session without an `AppHandle`; chunks through the existing `run_embedding_batch` pipeline. Runtime engine path is unchanged.
> - Visibility flips in `lib.rs`: `arabic`, `embeddings`, `lexicon` → `pub mod`. Purely additive; in-crate access paths unchanged. Required so the build helper can name `lexicon::ConceptRecord`, `arabic::Lang`, and `embeddings::embed_passages_standalone`.
> - **Numbers from the build run**: 20,000 concepts parsed, 100% English coverage (fallback chain never fired), 1,008.5 passages/sec on 24 threads, 19.8 sec total embed, 29.6 MB asset (committed to repo per Boss directive — changes only when `lexicon_v1.tsv` does).
>
> **§1B shipped (`909e381`) — runtime loader + Bridge Adapter**:
> - `bridge_vectors/asset.rs` — `parse()` over `include_bytes!("data/concept_vectors_v1.bin")`. Copies into owned `Box<[f32]>` to avoid the f32-alignment hazard of reinterpreting `include_bytes!` data as `&[f32]`.
> - `bridge_vectors/store.rs` — `ConceptVectorStore` with `nearest_concept` (top-1) and `nearest_concepts_k` (small-k via sorted Vec, beats BinaryHeap for k≤32). Cosine over flat row-major matrix.
> - `bridge_vectors/mod.rs` — `pub fn get() -> &'static ConceptVectorStore` singleton via `OnceLock`.
> - New `src-tauri/src/ctse/` module — Bridge Adapter:
>   - `resolve_term_pure(graph, store, embed_query, term, lang, threshold)` — pure dependency-injected core; closure invoked **only when M11 fast path misses**.
>   - `resolve_term_to_concept(app, term, lang)` — Tauri-context wrapper; pulls singletons + delegates query embed to `embeddings::constellation_embed_text`.
>   - `DEFAULT_THRESHOLD = 0.78` (initial guess from e5 model card; tunable in §1D).
> - **Fast path**: `LexiconGraph::find_nodes(lang, lemma)` (M11 already-public method) → direct `graph.nodes[idx as usize].concept_id` (M11 already-public field). Microseconds; no ONNX. Hits ~80% of expected terms.
> - **Slow path**: e5 embedding of the query term + cosine k-NN against the 20K matrix. Used only when fast path misses.
> - **10 tests, all green**: 5 store unit (synthetic basis vectors), 1 baked-asset round-trip (real 30 MB asset), 4 adapter (real M11). The fast-path test uses a panicking-on-call closure to verify the slow path is never invoked when M11 has the lemma.
>
> **What's pending (next session)**: §1C (Rust-side only — re-scoped from approved Plan): schema migration `term_vocab.bridge_concept_id`, write-time hook in `reindex_single_note` (fast-path-only resolution; no ONNX in write path), Tauri `ctse_run_backfill` command (NULL-row walker, batched, resumable, sentinel-marked failures), removal of `init_term_embeddings` + `term_embeddings` table + `populate_term_vocab`. **§1D**: auto-trigger backfill, status-bar progress UI, `ctse_search_by_concept` query path, full Settings UI cleanup. **First Boss-test gate fires at §1D** (cross-language semantic search). **§1E**: three-agent audit per Migration Rule §4. Resume checklist in `lab/reports/SESSION-LOG-2026-05-05.md` "State-of-standing" section.
>
> **What gets retired in §1C**: `init_term_embeddings` Tauri command + the entire per-library term-embedding loop (the source of every Phase 1.5 freeze). The `term_embeddings` table is dropped (note: existing DBs leave the table dangling — harmless but worth a future GC). The Settings modal "Rebuild Term Embeddings" button + `termEmbedProgress` writable store + the v1.34 status line `✓ N terms indexed` all go with it. Replaced by silent write-time derivation (Rule 8) and a single status-bar progress strip when the backfill is running.
>
> **Where to read the design**: `lab/reports/MIG-013-CTSE-ARCHITECT-v2.md` (architecture; supersedes v1 which is preserved as historical record), `lab/reports/MIG-013-CTSE-PLAN.md` (phase-by-phase commits with verification clauses), `lab/reports/SESSION-LOG-2026-05-05.md` (full session record + state-of-standing).
>
> **Standing migration checklist update (LL-027 candidate)**: when an audit returns three independent SME reports converging on "the unit of work is wrong, not the implementation", **stop iterating on the implementation** — ship a fresh Architect doc that pivots, not another fix-N. Five fix-3 → fix-9 attempts to scale `init_term_embeddings` would have continued indefinitely without the cross-discipline audit. The audit pattern (X-ray view + Library/IR view + App-architecture view) is the right shape for any "we keep band-aiding the same hot path" symptom. To be confirmed and added to LL after §1E.

**Version 1.34 | 2026-05-04**

> **What changed in v1.34** (same day as v1.33, post-MIG-012 polish): **MIG-012 Build.7-fix-1 + Build.7-fix-2 shipped — auto-trigger semantic-init when toggle flips ON, plus visible status line and manual Rebuild button.** Closes the "deferred follow-up" logged in `MIG-012-AUDIT.md`.
>
> **fix-1 (`91356b1`)**: SettingsModal `$effect` watches `$appSettings.index.semanticSearchEnabled`; on ON-flip calls `termEmbeddingStatus()` and, if count is 0, attaches a `term-embedding-progress` Tauri event listener and fires `init_term_embeddings(false)`. Progress UI (live counter + accent-fill bar + Cancel button) renders inline below the toggle, driven by a new module-scoped `termEmbedProgress` writable store so the job survives Settings modal mount/unmount. On OFF-flip while running: cancels via `cancel_term_embeddings`, waits for the cancelled-event flush, then clears UI after 4 sec. Resumable: re-firing skips already-embedded terms via the existence check.
>
> **fix-2 (`dd3b2e5`)**: when toggle is ON and no job is in progress, a status line shows the current state — **`✓ {N} terms indexed`** (when count > 0; ready to use) or **"Index not built yet"** (when count == 0; either freshly toggled or a real bug). Rebuild / Build now button on the same row gives users a manual escape valve, useful when models change in the future and especially as the only way to verify state in production builds (Tauri disables DevTools at release-build time, so the user can't `invoke('term_embedding_status')` from console). Lesson: **production-Tauri-builds-disable-DevTools means every state the user might want to verify needs visible UI affordance.** Logged for future MIGs introducing background jobs.
>
> Boss verified PASS — status line correctly shows **`✓ 18,200 terms indexed`** on a Universe where yesterday's MIG-012 G2 session embedded the table. Confirms fix-1 correctly skips re-init when populated AND fix-2 surfaces the truth visibly.
>
> **i18n**: 9 new keys × 15 locales (3 from fix-1 + 6 from fix-2). Full ar+en, English placeholders in 13 others per established backfill pattern.
>
> **Standing migration checklist update (LL-026)**: when introducing a long-running background job that affects user-visible feature state, the migration MUST also include (a) a UI status indicator visible without DevTools and (b) a manual trigger affordance (Rebuild / Force / Run-now button). Mandatory for ship.

**Version 1.33 | 2026-05-04**

> **What changed in v1.33** (same day as v1.32; Boss "Proceed all" cascade): **THREE more MIGs closed back-to-back — MIG-011, MIG-012, plus a pre-existing script-filter bug fix and the note-stage-taxonomy-decision queue.** The Index function went from "mentions-side cross-language" (v1.32) to a full vocabulary search engine across all three retrieval layers: literal substring (always-on), lexical-bridge (M11 corpus, 20K concepts × 15 langs), semantic (multilingual-e5-small ONNX embeddings).
>
> **Pre-existing script-filter bug fix** (`5dbb43f`): typing Arabic in the Index filter while script-tab "All" was active returned 0 results until the user bounced through "عربي" once. Two layers — substring-direction-mismatch (FTS5 stores stems shorter than typed surface forms; the bidirectional `query.includes(term)` check was gated on comma-mode-only) and stale-letter-filter persistence (clicking a Latin letter then typing Arabic dropped Arabic terms via the active letter filter). Both fixed; bidirectional substring is now always active and the letter filter auto-clears when filtered entries don't match it.
>
> **MIG-011 closed — cross-language Index *filter* expansion.** Mirror of MIG-010 applied to the search box: typing "knowledge" surfaces Arabic terms `معرف` / `علم` with `via knowledge` badges; typing `معرفة` surfaces English `knowledg` / `cognit` with `via معرفة` badges. New Tauri command `lexicon_expand_for_filter`; frontend per-keystroke debounce 300ms + cancel-token + per-session cache; same Settings toggle drives both surfaces (one mental model, two behaviors). 5 build commits + simplify + audit. Boss verified PASS at G2.
>
> **Side-discovery during MIG-011 G2 testing** (`c95a0e6`): two i18n keys (`indexPanel.returnToIndex` + 6 Living Link lifecycle stages under `notePane.stage.*`) were rendering as raw literals in the Arabic interface — and audit showed they were missing in **all 15 locales**. Backfilled with full ar+en + English placeholders in 13 others. The deeper question — should Notes use Living Link lifecycle stages (`spark/birth/growth/maturity/dormancy/archival`) or Zettelkasten stages (`fleeting/literature/permanent/synthesis`)? — queued as `project_note_stage_taxonomy_decision.md` for Boss design call.
>
> **MIG-012 closed — Index Search Engine: search history + semantic search.** Boss-approved Q1.A + Q2.C + Q3.B (term-level embeddings, lazy-on-first-semantic-query bootstrap, SQLite-per-Universe history). Two new tables (`term_embeddings`, `index_search_history`) with idempotent `CREATE TABLE IF NOT EXISTS` for transparent migration. 4 new Rust IPCs for embeddings (`init_term_embeddings` with progress events, `cancel_term_embeddings`, `search_terms_semantic`, `term_embedding_status`) + 3 for history (`read_index_history`, `write_index_history_entry`, `clear_index_history`). Frontend: 2 new Settings toggles + Clear button, per-keystroke debounced semantic search (mirrors MIG-011 pattern), filter loop now matches across direct → bridge → semantic with priority, `≈ similar` cyan badge for semantic matches, history dropdown on filter focus, full Arabic translation. 8 build commits + simplify + audit + confirm-dialog fix. Boss verified PASS at all three G2 stages.
>
> **§Build.8 simplify caught 3 Tier 1 issues** that would have shipped to users: (1) `init_term_embeddings` held `EmbeddingState.engine` and `SearchState.db` for the entire ~10–20 min embed-all loop, freezing every concurrent IPC during the job — fixed via lock-per-iteration. (2) f32 LE BLOB encode/decode duplicated between note + term + read paths — extracted `vec_to_blob` / `blob_to_vec` helpers; existing `constellation_embed_notes` migrated to use them too. (3) `TERM_EMBED_CANCEL` was a process-global static; moved to `EmbeddingState` for per-app-instance scope. The simplify methodology earned its keep on this MIG.
>
> **§Build.8-fix (`8d98a3a`)**: Boss G2 stage 1 step 6 surfaced that the browser-native `confirm()` dialog couldn't honor app i18n — both message text and OK/Cancel buttons stayed English even on the Arabic interface. Replaced with the existing `ConfirmDialog.svelte` component for the Clear-history button; Arabic users now see fully-localized "حذف نهائي... / مسح / إلغاء". Pattern for any future confirmation surface.
>
> **Boss-approved follow-on workstreams (logged 2026-05-04, NOT yet started)**:
> - Note-stage taxonomy decision (Living Link lifecycle vs Zettelkasten) — `project_note_stage_taxonomy_decision.md`. Quick i18n fix shipped today; deeper architecture decision deferred.
> - Auto-trigger semantic-init when toggle flips on — Plan-promised but currently the init must be invoked explicitly. Manual trigger via DevTools available (`init_term_embeddings`). Logged for Build.7-fix-1.
> - Search history toggle: track this with the rest of the deferred items in the existing backfill workstream.
>
> **Lessons logged this round (LL-025)**: simplify pass with parallel review agents earns its keep on cross-subsystem migrations. The lock-per-iteration find on MIG-012 §Build.8 would have shipped a real ~20-min freeze to Stage 2 testers without the simplify check — caught before binary release. Lesson: **for any migration that adds a new long-running background job, `/simplify` is mandatory before the Boss G test.** Adding to the standing migration checklist.

> **What changed in v1.32**: **MIG-010 closed — Lexical Bridge integration into the Index panel.** Boss directive: "finish and implement the Index function." Build cascade ran §A (Phase A bug fix — register `read_cooccurring_terms` in `tauri::generate_handler!`, the chip-strip cooccurrence panel was silently broken pre-MIG-010) → Architect doc → Plan doc → §Build.1 (`pub(crate)` bridge helpers + parameterize `find_match_via_marked` for STX/ETX vs `<mark>` delimiter regimes) → §Build.2 (`read_term_mentions` extended with `expand_cross_language: Option<bool>`; new `via_lemma: Option<String>` on IndexMention; `build_term_match_clause` helper with 4 unit tests) → §Build.3 (Settings: new "Index" section + `indexExpandCrossLanguage: bool` toggle in 15 locales) → §Build.4 (IndexPanel reads setting, renders `via_lemma` badge with `dir="auto"`) → §Build.4-fix (G2 cosmetics: off-state visual contrast + RTL toggle slider mirror; latent G3 fix attempted) → §Build.4-fix2 (defensive expansion fallback + frontend error catch — diagnostic infrastructure) → §Build.4-fix3 (the actual G3 root cause: `$effect` in IndexPanel read `mentionsCache.size` making the cache its own dependency → Rule 2 violation — wrapped cache reads in `untrack()`) → §Build.5 (`/simplify` three-agent pass — fixed Tier 1 prop-coupling via `cacheKey?: unknown` rename, Tier 2 `LexicalExpansion::into_parts()` accessor + `fts_quote_phrase` extraction + flatten `match` block + `prepare_cached` + gated `eprintln`, Tier 3 magic-pixel comment) + Phase 4 Audit doc.
>
> **Boss verified PASS at G2 + G3** — screenshot showed Arabic notes ("2007", "2010", "428 هـ") with **`via علم`** badges + Spanish-language reference ("Ada Lovelace") with **`via conocimiento`** badge. The 7,600-note mixed Arabic/English library is now searchable by *concept* across languages, not just by literal lemma. Audit at `lab/reports/MIG-010-AUDIT.md` confirms all 11 invariants hold.
>
> **Phase D (boot perf)**: deferred `readIndexEntries()` from `graphReady` to first Index-panel open. ~tens of ms saved on every boot for users who don't open the Index that session. Cost paid on demand.
>
> **Phase E (docs)**: dedicated Index help page at `docs/help.uConstellation.World/Index/Index.md`. User Manual §7 + Arabic User Manual §8 updated with cross-language toggle subsection. 13 other locale User Manuals queued in existing `project_user_manual_13_locales_backfill.md`.
>
> **Phase G (guidance)**: teaching doc `docs/help.uConstellation.World/Index/Index Guidance — How to Read Your Vocabulary.md` — three reads (frequency profile, language-pair balance, cognitive adjacency), five common patterns + readings, weekly-practice ritual. Boss-pattern teaching doc, modeled after the queued 360.3D Stratification Matrix guidance.
>
> **Lesson logged (LL-024)**: `$effect` body must declare its dependencies explicitly. The §Build.4-fix3 root cause (cache-invalidation effect tracked the cache it managed → infinite-clear loop) is a CLAUDE.md Rule 2 violation that I shipped without an end-to-end IPC trace. New rule: for any cross-subsystem `$effect` work, run a console-level trace BEFORE the Boss test cycle. Working Agreement #4 self-correction.
>
> **Boss-approved follow-on workstreams** (logged 2026-05-04, NOT yet started):
> - **MIG-011** — cross-language Index *filter* (mirror of mentions expansion, applied to the search box). Today the filter does substring matching only; bridge-aware filtering is the next step.
> - **MIG-012** (eventually) — Index search engine: search history + semantic search powered by existing `embeds.rs` ONNX pipeline. Memory: `project_index_search_engine_history_semantic.md`.
> - Pre-existing Index script-filter bug ("All" hides Arabic terms until "عربي" bounce). Memory: `project_index_script_filter_all_hides_arabic.md`.
> - "Rebuild Index" button — explicitly **deferred** per Rule 8 (no `rebuild_*` commands; FTS5 triggers maintain the index at write-time). Memory: `project_index_rebuild_button_decision.md`.
>
> **Phase C status**: Settings → Boot-perf scorecard turned out to be ALREADY shipped (5-criterion view in `SettingsModal.svelte`); STATUS.md was stale on this. Rebuild Index button deferred per above.

> **What changed in v1.31**: **MIG-008 closed.** Build cascade ran §145 (CreateItemDialog component + i18n en+ar) → §146 (wire New Folder) → §147 (wire New Note) → §148 (wire New Base, replace NewBaseDialog) → §149 (wire New Library + new `create_new_library_at` Rust IPC) → §150 (orphan sweep — five state vars + two functions + `NewBaseDialog.svelte` deleted) → §151 (Boss-flagged context-menu gaps: folder right-click missing "New Base" + library-row right-click falling through to browser-default menu — both fixed) → §152 (Build.7 /simplify checkpoint: i18n backfill 13 locales, `create_new_library_at` async, IME composition guard, KIND_LABELS lookup, `parseFrontmatter` instead of hand-rolled regex, dropped `defaultName` prop + `lastOpenState` $effect, `baseSelectedSet` for O(1) lookup, plus four Boss-approved adds — right-click "New note" now applies folder templates the same way the toolbar does, `/libraries` route migrated to the dialog, path-traversal hardening on Rust create IPCs via `sanitize_name`) + docs commit (User Manual + 2 help articles + Arabic User Manual) + audit doc. Boss verified PASS across all 8 create scenarios on the §151 binary plus the four §152-specific verifications (templates, route migration, path traversal, IME). Audit at `lab/reports/MIG-008-AUDIT.md` confirms all 11 invariants (I1–I11) hold. Project memory `project_create_dialog_standardize.md` marked SHIPPED.
>
> **Logged for follow-up**: 13 User Manual translations (`project_user_manual_13_locales_backfill.md`); reserved-Windows-name + trailing-dot/space hardening on Rust create IPCs (pre-existing gap, not MIG-008-introduced); collision popup (`project_rename_collision_popup_wanted.md`) — pre-existing, will compose with the dialog when shipped.

> **What changed in v1.30** (MIG-008 starts; §142–§144 closed MIG-006 §4):
>
> **MIG-008 — Create-Dialog Standardization (Phase 1 Architect committed at `22839d4`)**. Boss directive 2026-05-03: "Whenever I created a folder it is created in the respective location under the name 'New Folder'. It shouldn't work this way. What I want it to do is to follow the standard way of any file system. A popup dialog box should emerge to name the new folder and to choose the location. Same thing should happen when creating new note, base or library." Architect plan at `lab/reports/MIG-008-CREATE-DIALOG-ARCHITECT.md`. Inventory found four inconsistent create flows (Folder rejects collisions / Note auto-increments / Base has its own `NewBaseDialog` / Library has folder picker only); 11 invariants (I1–I11) defined; three options enumerated (A: shared modal, B: inline tree-row input, C: rich modal with templates); **Option A approved by Boss**. Phase 2 Build cascade kicks off in 8 steps (§Build.1–.8): build shared `<CreateItemDialog>` component → wire each of the four affordances → drop orphaned auto-create handlers → /simplify → audit. Each step pauses for Boss-testable verification clause.
>
> **MIG-006 §4 closed (§142 + §144)**. Original gap from §3-redo Stage 1 testing: Outgoing Links / Backlinks panels stayed stale after wikilink rename cascade because the SQLite index wasn't reindexed and the frontend's `allLibraryLinks` `$state` was loaded once at boot and never refreshed. **§142** plugged the Rust side (cascade walker calls `reindex_single_note` for each rewritten path; new `library_name` parameter on the `update_links_on_rename` IPC). **§143** attempted a frontend-side targeted update of `allLibraryLinks` but only matched entries whose `target` equaled the rename's `oldName` exactly — after several renames in a session (Hub v4 → v5 → … → v8) the in-memory state had drifted further than any single rename's `oldName`, so the targeted match never fired. **§144** superseded §143 with the simpler drift-resistant fix: re-fetch `cache_boot_snapshot_graph` post-cascade and replace `allLibraryLinks` + `notePathToAliases` wholesale. Catches not just the just-rewritten target but any drift accumulated in the session. Boss tested PASS — Outgoing Links panel updates immediately after rename, no app restart, no manual rebuild.
>
> **Side discoveries during §144 testing**: (1) Pre-§140 cid_cn collision found in Boss's SourceA test note (title: Hub v6, cid_cn matching Hub v8) — §140's check prevents NEW path-reuse contamination but can't retroactively heal already-corrupted files. Boss self-healed via delete + recreate. Logged for future sessions: a one-time scrub utility for existing libraries is queued. (2) Unlinked Mentions panel matches frontmatter alias entries — the scanner reads full file content (frontmatter + body) so YAML alias entries (`- "Hub v6"` from rename history) surface as "unlinked mentions". Logged in project memory `project_unlinked_mentions_alias_bleed.md`; pair with the existing `project_unlinked_mentions_double_count.md` in a single Unlinked Mentions cleanup MIG.
>
> **Boss agenda items added today** (queued, not in scope of any in-flight MIG):
> - Standard OS-style create dialog (greenlit → became MIG-008).
> - One-time scrub utility for pre-§140 cid_cn collisions in existing libraries.
> - Outgoing Links display case fix (`hub v8` → `Hub v8`; cosmetic).
> - Unlinked Mentions / frontmatter alias bleed (project memory above).
> - NSIS bundling lock investigation — recurring `os error 32` when Constellation is running during build; not a tooling bug per se but worth a workaround.

> **What changed in v1.29** (§135 + §136, same calendar day as v1.28):
>
> **§135 — `/simplify` checkpoint over §128-§134** (commit `fe9bf9e`). Three review agents (reuse / quality / efficiency) walked the MIG-006 §3 redo arc with Boss-supplied focus areas. Real-bug fixes shipped: refcounted `cascadingPaths` (Set → `Map<string, number>` so spam-renames in the same library don't pop each other's marks); killed the 1-second magic-timeout settle (orchestrator now `await`s `reloadTabsFromDisk(result.rewritten)` directly — real completion signal, no listener race, no wall-clock penalty on single-file renames); extracted `tabsInLibrary(libraryPath)` helper with separator-bounded prefix check (`/Foo/Bar` no longer falsely matches `/Foo/Bar2`). Efficiency wins: `reloadTabsFromDisk` batched + idempotent (parallel reads, single `openTabs.update`, skips bump when content matches); `watcher_suppress::was_recent` cheap-path lookup with opportunistic 256-threshold sweep (was O(N) `retain` on every watcher event); `CascadeResult.failed` capped at 100 entries with a `failed_truncated: usize` counter (defensive against pathological cascades bloating the IPC payload); consolidated `isCascading` WHY-comments at the three gate sites into one canonical docstring on `isCascading()` itself.
>
> **§142–§144 — MIG-006 §4 closed (write-time index propagation, both Rust + frontend halves)**. Boss surfaced the original gap in §3-redo Stage 1 testing: after rename, Outgoing Links panel kept showing the OLD target name (`foo`, lowercased) — the body cascaded but `note_meta.outgoing_links_json` and `note_links` weren't updated, so panels reading the index served stale data. **§142** plugged the Rust side: `update_links_on_rename` now calls `reindex_single_note` for each rewritten path after the cascade walk, with a new `library_name` parameter on the IPC. SQLite caught up. **§143** attempted a frontend-side targeted update of `allLibraryLinks` (the boot-snapshot `$state` the panels actually read from), but only matched entries where the in-memory `target` equaled the rename's `oldName` exactly — and after several renames in a session (Hub v4 → v5 → … → v8), the in-memory state had drifted further than any single rename's `oldName`, so the targeted match never fired. **§144** superseded §143 with the simpler drift-resistant fix: re-fetch `cache_boot_snapshot_graph` post-cascade and replace `allLibraryLinks` + `notePathToAliases` wholesale. Boss tested PASS on the §144 binary — Outgoing Links panel now updates immediately after rename. Closes the original Stage 1 observation. (§143's targeted update is left in the commit history as an "almost-fix" anchor — useful context for the next person who wonders why we don't do incremental updates.)
>
> **Tab/title corruption discovered + recovered during §144 testing**: a SourceA test file from earlier sessions had `title: Hub v6` AND a duplicate `cid_cn` matching Hub v8's identity — pre-§140 corruption that survived in the disk file. §140's `cid_cn` check prevents NEW path-reuse contamination but can't retroactively heal already-corrupted files. Boss self-healed by delete + recreate. Post-§140 the bug shouldn't reproduce on fresh notes. Logged for future sessions: existing libraries may carry pre-§140 cid_cn collisions; those need manual recovery (delete + recreate) or a one-time scrub utility.
>
> **Side discovery during §144 testing — Unlinked Mentions panel matches frontmatter alias entries** (logged: `project_unlinked_mentions_alias_bleed.md`). The scanner reads the full file content (frontmatter + body) when looking for the active note's name as a plain-text occurrence, so frontmatter `aliases:` entries surface as "unlinked mentions" of unrelated notes. Should split on the closing `---` fence. Pair with `project_unlinked_mentions_double_count.md` in a single Unlinked Mentions cleanup MIG.
>
> **§141 — `/simplify` checkpoint over §137-§140**. Three review agents (reuse / quality / efficiency) walked the §137-§140 diff. Real cleanups shipped: **(a)** new `normalizePathKey(p)` exported from `src/lib/utils.ts` — the `(p) => p.replace(/\\/g, '/').toLowerCase()` function was duplicated 7+ times across utils, store, and +layout. Single source of truth so a future filesystem-rule change (case-sensitive volumes, NFC normalisation) is one edit, not eleven. Every path-keyed Map operation now goes through this. **(b)** `WAB_LS_KEY = 'constellation-wab'` constant in store.ts — the localStorage key was hard-coded in five places. **(c)** Single `walkAuxStatePaths` walker shared by `migratePathKeyedAuxStateOnRename` and `clearPathKeyedAuxStateOnDelete` — both used to walk the same three structures (in-memory wab, in-memory recentWrites, localStorage wab) with identical norm-and-prefix matching. The walker passes the ORIGINAL key to the decide callback so folder-rename suffix preservation works on case-mixed Windows paths. **(d)** `openNoteTab`'s wab/disk choice extracted to `resolveNoteContent(filePath)` helper — the §140 inline check was three levels deep with three duplicated `clearWriteAhead` calls. The helper returns `{content, cursorPos, scrollTop}`: when wab is stale (cid_cn mismatch), drops the wab cursor/scroll too — they were for the OLD note, a subtle correctness improvement the inline §140 code missed. **(e)** `handleStageChanged(path, stage)` hoisted in +layout.svelte — the 3-line callback was inlined twice (main editor + split/second-screen path). **(f)** `extractCidCn` regex bounded to the first `---…---` frontmatter block — prior code matched against the full content, so a 10MB note made the lazy regex walk the whole body. **(g)** Stripped `// §139:` / `// §140:` inline anchor comments where they narrated what the code obviously does; kept multi-line docstrings on function declarations.
>
> **§140 — Cross-note content corruption via stale `writeAheadBuffer` (Rule 8 + the BUG-015 corruption class)**. Boss reported a **serious data corruption bug**: "Sometimes, when switching between notes after renaming or creating notes, I discover that a note replicates its contents, title, and cid_cn into another note. The victim note keeps its title in the file tree, but when I click it, it shows the culprit note (title, content, and properties)." Investigation pinpointed `writeAheadBuffer` (in-memory `Map<filePath, V>` + `localStorage` backup that survives app restarts). When a note is flushed, the editor's content is stashed under its file path so a later `openNoteTab` can substitute it for a disk read. **`renameItem` / `moveItem` / `deleteItem` migrate `openTabs.path` correctly but never touched the buffer** — so when a path was reused after a rename or delete (trivial with human-named notes: rename Foo → Bar, create new Foo, the new Foo lands at the old `…/Foo.md` path), `openNoteTab` hit the stale buffer entry and loaded the OLD note's content (cid_cn / title / body) into the new tab. The file tree kept showing the new note's title (driven by `display_title` from disk frontmatter — disk was correct) while the tab held the old note's content (in-memory only, until the user typed and triggered a `handleSave` that committed the corruption to disk too). Same Rule 8 / write-time-derivation gap §137 closed for `stageMap` / `maturityMap` — except corruption-class severity. §140 closes it three ways: **(1)** new helpers `migratePathKeyedAuxStateOnRename` and `clearPathKeyedAuxStateOnDelete` migrate / drop `writeAheadBuffer` + `recentWrites` entries (in-memory + localStorage backup), with folder-prefix support for folder rename / delete; **(2)** wired into `renameItem`, `moveItem`, `deleteItem`; **(3)** defense-in-depth in `openNoteTab` — when a wab entry hits, also read disk and compare the `cid_cn` signature; on mismatch, prefer disk and clear the stale buffer (handles historical localStorage entries from before §140). Self-healing via (3) for any user with stale localStorage from prior sessions.
>
> **§139 — Three production-binary bugs Boss caught (RTL arrows, recursive FileTree, SvelteMap reactivity)**. Boss installed the §138 production binary and reported three bugs from real-world testing: (1) Promote → / ← demote arrows inverted in RTL note context — the visual reading direction is right-to-left so `→` reads as "backward" in RTL; fix is to swap arrow characters when `dir === 'rtl'`. (2) Folder children in the file tree never receive `stageMap` / `maturityMap` — `<svelte:self>` recursion at `FileTree.svelte:102` was missing those two props from its prop list, so notes inside any folder rendered with default empty maps. (3) Promote/demote and "add Stage via property panel" updated the breadcrumb badge but **not** the file-tree emoji — the chain (handlePromote → onStageChanged → stageMap reassign) looked correct but the file tree didn't re-render. Root cause: the `$state(new Map())` + reassign-to-fresh-Map pattern has a Svelte 5 prop-propagation quirk visible specifically through this child-reads-via-prop path. Fix: switch `stageMap` and `maturityMap` to `SvelteMap` (Svelte 5's explicitly-reactive Map) — mutations are reactive at the operation level, no reassign-to-force-identity needed. Updated all six call sites (enrichNodesBackground, §138 toggleLibrary scans, §137 handleRenameComplete migrations, both onStageChanged callbacks) to use direct `.set()` / `.delete()`. New `migratePathKeyedMapInPlace<V>` helper in `src/lib/utils.ts` for SvelteMap targets in §137. `notePathToAliases` and `searchLinkCounts` stay on the original `$state(new Map())` pattern for now — narrow scope, only the user-visible drift surfaces converted.
>
> **§138 — Stage + maturity load on library expand (deeper Rule 8 fix)**. Boss tested §137 and reported: "the emoji is not visible, not before renaming or after it." The §137 path migration was correct but lit nothing because the upstream `stageMap` and `maturityMap` were both **empty on boot**. Audit found the cause: `enrichNodesBackground` (the only path populating these maps) was deliberately removed from the boot flow for boot-perf — comment at `+layout.svelte:2744-2757` explains "ZERO BOOT-TIME WALKS." Before §138, the only triggers were the Sky View legend's `onRequestEnrichment` button, the Settings → Rebuild Index path, and the first-ever-launch modal. None of those fire on a normal boot, so the file tree never showed stage emojis or maturity dots. §138 adds a third trigger: when the user expands a library in the sidebar (`toggleLibrary`, first-expand only), fire `scan_note_stages` + `compute_note_maturity` for that library and merge results into `stageMap` / `maturityMap`. Fire-and-forget so the expand isn't blocked; maps are reactive `$state` so the file tree re-renders when each scan returns. This respects the boot-perf discipline (no walks on boot) while restoring the Rule 8 expectation (every derived view present at the moment the user looks at it). Mutation guard: the merge only writes a fresh Map when at least one entry actually changed — Svelte doesn't fire spurious reactivity on no-op merges.
>
> **§137 — Rename propagates to path-keyed reactive state (Rule 8 reinforcement)**. Boss observation during Stage 5 testing: "we used to have the stage icon attached to the note title as a prefix — and we want Constellation to do it instantly when the user promotes, demotes, renames, or re-renames. That's why Constellation is unique and has its own prediction engine." Audit revealed: file-tree stage emoji + maturity dot + alias index + search-hub link counts are all `Map<path, V>` reactive `$state` in `+layout.svelte` (`stageMap`, `maturityMap`, `notePathToAliases`, `searchLinkCounts`). Promote/demote already kept them in sync via the `onStageChanged` callback chain; **rename did not**. After a rename, the renamed file's old path stayed in every map as an orphan, and the new path had no entry — so the file-tree showed the renamed note without its stage emoji until the next library scan. Direct violation of Rule 8 (Write-Time Derivation: "every computed view in Constellation is maintained at write time, not read time"). §137 adds `migratePathKeyedMap<V>(map, oldPath, newPath)` in `src/lib/utils.ts` (handles file rename, folder-prefix rename, and no-op canonical-file renames where the disk path stays the same; returns `null` to skip spurious reactivity when nothing migrated) and calls it from `handleRenameComplete` for all four affected maps. The renamed file's stage emoji, maturity dot, and aliases now follow the path the moment the rename lands.
>
> **§136 — Stage breadcrumb redesign + `handlePromote` cascade gate**. Boss observation: the breadcrumb Stage dropdown duplicated the property panel — same control, two surfaces. Homework on commit history showed why: the predecessor commit (`87d21d7`, CE Phase 6) added Stage to the breadcrumb as a one-click `Promote →` *verb* per `docs/CE-spec.md` Phase 6, then commit `6cbe87c` (40 minutes later) silently refactored the verb into a property-selector dropdown. Boss's "not LOGICAL" critique was reading the post-refactor state correctly. §136 restores the verb-distinct design: the breadcrumb now renders `[← demote] [stage badge] [Promote →]` with visual asymmetry — Promote prominent (accent border), demote subdued (faint arrow, no border, tooltip-only label). Demote is permitted (CE-spec one-way line was an oversimplification — knowledge revision is real research practice), but visually subdued to encode the frequency asymmetry. Removal of the stage property entirely stays in the property panel (verbs vs administration). Side fix: `NoteEditor.handlePromote` was the *other* drift surface the §134 audit missed — it bypassed the `isCascading` cascade gate the same way `PropertyEditor.saveTabContent` did. Added the gate at the top of `handlePromote`. Both stage-edit paths (breadcrumb verb + property panel) and both body-edit paths (`handleSave` + `handleFlush`) now share one consistent cascade gate. CE-spec Phase 6 updated to match (the "one-way" line is now historically annotated). i18n: added `notePane.demote` to all 15 locales; `notePane.promote` already existed from CE Phase 6.
>
> Stage 1-4 of the §3 redo Boss test cycle have all PASSED (basic cascade ✓, open-editor coherence ✓ — the headline win, pre-cascade-staleness ✓, multi-source watcher-loop ✓). Stage 5 (PropertyEditor / handlePromote cascade gate verification) and Stage 6 (spam-rename refcount) remain.

> **What changed in v1.28**: MIG-006 §3 redo lands clean (commits §128-§133). After the §115 attempt at §3-expanded ("open-editor coherence") burned BUG-015, MIG-006 §3 sat in `REVERTED` status for a week. Boss directed (via the 360.3D pattern) that a Concept Paper come first; that landed as §127 (`docs/Rename-Function-Concept-Paper-v1.0.md` + `lab/reports/MIG-006-3-REDO-ARCHITECT.md`). The redo itself shipped across §128-§133 as six landable steps + Phase 4 audit closure, all anchored to the eight P1-P8 invariants and Principle D6 (no `$effect` reads/writes value/editBody — that's BUG-015's class).
>
> **The redo (Concept Paper Option A — recreate via `{#key}` bump):**
>
> - **§128 (§3-redo.1)** — `flushAllTabsInLibrary(libraryPath)` helper in `store.ts`. Iterates open tabs in the affected library, writes any in-flight `writeAheadBuffer` content to disk via `writeNote`, marks each path as a recent write so the watcher's external-edit emit skips it. Closes F2-pre-cascade-staleness.
> - **§129 (§3-redo.2)** — new `src-tauri/src/watcher_suppress.rs` module: `mark(path)` / `was_recent(path)` with 2.5 s TTL. Cascade walker calls `mark` before each `fs::write`; the file watcher's emit path filters out recent writes. Closes F3-watcher-loop.
> - **§130 (§3-redo.3)** — `CascadeResult { rewritten, failed }` struct + `cascade:rewrote { paths }` Tauri event. Per Concept Paper D3, the cascade is per-file atomic but not transactional across files; failures collect into `result.failed` rather than rolling back successes.
> - **§131 (§3-redo.4)** — `OpenTab.reloadVersion?: number` field + `reloadTabFromDisk(path)` helper + `cascade:rewrote` listener in `+layout.svelte`. The listener re-reads each affected file from disk, updates `tab.content`, bumps `reloadVersion`. NoteEditor's `{#key}` includes `reloadVersion` so NotePane destroys + remounts with fresh content. Per Principle D6, this is the safe primitive — never an `$effect`-driven `view.dispatch`.
> - **§132 (§3-redo.5)** — `handleRenameComplete` orchestration: markCascading → flushAllTabsInLibrary → updateLinksOnRename → settle → clearCascading. NoteEditor's `handleSave` and `handleFlush` both gate on `isCascading(filePath)` and bail out for the duration. Closes F2-post-cascade-stomp.
> - **§133 (§3-redo.6)** — `/simplify` checkpoint cleanups: path normalisation in `cascadingPaths` Set + `flushAllTabsInLibrary` (Windows backslash vs forward-slash), parallelised `cascade:rewrote` listener (Promise.all), conditional 1 s settle (skip when `result.rewritten.length === 0`), opportunistic full-map GC in `watcher_suppress::was_recent`.
> - **§134 (§3-redo.7) — Phase 4 audit closure (this commit).** Three review agents found two HIGH/MEDIUM drift items shipped as fixes here:
>   - **PropertyEditor bypass (HIGH)** — `PropertyEditor.svelte` calls `saveTabContent` directly when the user edits a frontmatter property. Without an `isCascading` gate inside `saveTabContent`, a property edit during the cascade window would stomp the cascade's wikilink rewrite. Fixed by adding `if (isCascading(filePath)) return` at the top of `saveTabContent`. NoteEditor's gates on `handleSave`/`handleFlush` cover the body-save path; this gate covers the property-save path. Both routes now share the same protection.
>   - **Universe-switch leak (MEDIUM)** — `cascadingPaths` Set entries persisted across Universe switches. New `clearAllCascading()` helper called from `handleUniverseSwitch` so the new Universe starts with a clean slate.
>   - Concurrent renames + typing-during-cascade keystroke loss documented as known limitations; fixes deferred (concurrent renames need a `rename_id` serialization layer; keystroke loss is the input-block step that Concept Paper P4 explicitly accepts as out-of-scope for v1).
>
> **What MIG-006 §3 redo does NOT cover** (queued for §3-redo.8 onward, mapped to the original §4-§11 plan in `MIG-006-WIKILINK-CASCADE.md`):
> - Reindex via `index_note` (P7 — `note_links.target_name` reflecting disk).
> - Sync/async dispatch + progress events (P6 — hub-rename UX).
> - Atomic per-file writes via tempfile (P5 — kill-mid-cascade integrity).
> - Pre-MIG-006 backfill command for stale wikilinks.
> - Phase 4 audit (FULL — per-step audits ran inline; the cross-cutting audit happens at MIG-006 closure).
>
> **Migration table updated**: MIG-006 row now shows §1-§3 ✅ + §4-§11 ⏸.

> **What changed in v1.27**: Inline warning icons in matrix column headers (commit §125). Boss tested §124 on Abu Bakr and reported: "It is easy to identify the blind spot, but not the tensions. Is it in the Causes?" The §124 brown top border on Contradicts was being clipped by the matrix's `border-radius: 12px` + `overflow: hidden`. Boss's fix: "Maybe if we add the warning icons in their place, it will be easier."
>
> **§125 adds the same icon as the corresponding HUD chip directly above the column name** in the column header:
>
> - Blind spot column → ⚠ in red (alongside the existing full-red §122 treatment)
> - Fragile column (Derives From) → ⚠ in yellow
> - Tensions column (Contradicts) → ⚡ in brown (`#8b4513` light theme, `#c89875` dark theme)
>
> The icon is the primary signal; the §124 top border stays as a secondary cue (visible on middle columns even when the rounded corners clip the leftmost / rightmost). Visual continuity from HUD chip to column is direct: see ⚡ at the bottom, find ⚡ at the top of Contradicts.
>
> **No backend change in §125** — frontend template + CSS only.

> **What changed in v1.26**: Per-warning HUD chip colours + matching column-header overlays for fragile and tensions (commit §124). Boss confirmed §122 (red blind-spot column highlighting) on دمشق, then asked: "I want to have the same for the other warnings, like Orphan. But we have to choose a different color for each one."
>
> **Colour assignments**:
> - **Blind spots** (typed columns with 0 connections) — **red** (`var(--text-error)`). Existing §122 treatment; unchanged.
> - **Orphan** (no inbound links) — **orange** (`var(--color-orange)`). HUD chip only — no natural matrix counterpart, since "no one points at me" isn't a column-level signal.
> - **Fragile** (load-bearing on thin foundation) — **yellow** (`var(--color-yellow)`). HUD chip + 3 px yellow top border on the Derives From column header (the column whose under-population is what `single_point_of_failure` measures).
> - **Tensions** (active Contradicts links pointing at this note) — **brown** (Boss directive; brown isn't in the theme palette so hardcoded `#8b4513` for light theme and `#c89875` for dark theme). HUD chip + 3 px brown top border on the Contradicts column header.
>
> **Stacking precedence**: when a column is both a blind-spot and a fragile/tensions overlay candidate, blind-spot wins (red replaces everything). The `tensions-flag` and `fragile-flag` classes are only applied when `!isBlindSpot`. In practice tensions and blind-spot on Contradicts are mutually exclusive (tensions = inbound contradicts, which would make column count > 0); fragile + blind-spot on Derives From overlap only when the note has zero outbound derives-from while still being load-bearing-via-inbound — the red treatment is more important there.
>
> **No backend change in §124** — frontend CSS + classes only.

> **What changed in v1.25**: Stage 3.2 follow-up — blind-spot column highlighting (commit §122). Boss tested S3.2 on note دمشق, confirmed the column-totals row delivers the §4.2 Connection-Profile signal cleanly, then asked: "since the matrix identified the blind spots, it should highlight them within the matrix to help the user undertake the right measures."
>
> **Shipped in §122**: when a typed column's total is 0, the column header gets a warning treatment in addition to its normal type-coding:
>
> - Background gradient swaps from the soft type-colour tint (5%) to a `var(--text-error)`-mixed warning tint (14%).
> - Bottom border switches from the type colour to `var(--text-error)`.
> - The column name and the `0` count both render in `var(--text-error)`.
>
> Untyped is excluded from blind-spot detection — its 0 means "no plain wikilinks", not a typed-direction gap.
>
> Theme-aware via `var(--text-error)` (defined in `theme.css` as `--color-red`). With four-plus blind-spot columns, the visual is intentionally loud — the matrix is telling you which directions of reasoning haven't been declared for this note. The bottom HUD's `⚠ N blind spots` chip stays as a corroborating count.
>
> **No backend change in §122** — frontend CSS-only.

> **What changed in v1.24**: Three §120 retest follow-ups (commit §121). Boss flagged on the Arabic locale: (a) the `Untyped` column header still rendered in English because `typeLabels` derived skipped untyped via `if (lt === 'untyped') continue` — leftover from the §113 hardcode workaround; (b) the stage value `spark` (used in Boss's library) wasn't in the i18n stage map; (c) Arabic stratum-name terminology corrections — Boss's preferred terms: L3 رأي (vs قضية), L7 منظور (vs نموذج), L8 رؤية شاملة (vs رؤية كونية).
>
> **Three fixes shipped in §121**:
>
> 1. **Untyped column localized**. `typeLabels` in `Inspector360.svelte` no longer skips untyped — the loop now treats it uniformly, looking up `inspector360.untyped` (which §120 added to en + ar). With the §120 fallback chain, locales without that key fall through to en. Hardcoded English values stay as the final defensive fallback.
> 2. **`stage_spark` added** to en.json + ar.json. English: "spark"; Arabic: شرارة. Stage values are user-defined free-text (read directly from the YAML frontmatter `stage:` field by `extract_stage()` in `inspector360.rs`), so Boss's library uses lifecycle terminology beyond the four canonical Zettelkasten stages. Other lifecycle terms (birth/growth/maturity/dormancy/renewal) can be added on-demand if encountered.
> 3. **Arabic stratum corrections**: `stratum_name_3` قضية → رأي, `stratum_name_7` نموذج → منظور, `stratum_name_8` رؤية كونية → رؤية شاملة. Updated dependent help strings (`help_stratum_3/7/8`, `help_axis_stratum`, `help_dim_stratum`) to use the new terminology consistently.
>
> **No backend change in §121** — frontend + i18n only.

> **What changed in v1.23**: Three §119 follow-ups bundled (commit §120). Boss flagged on the §119 binary: (a) tooltip text for the dimension-strip `?` icons rendered ALL CAPS — inheriting `text-transform: uppercase` from the parent strip label; (b) tooltips near the right edge of the matrix were clipped because `transform: translate(-50%)` pushed half the tooltip off-screen; (c) "everything fully localized, like the Stratum, and the top row" — non-typed text in the matrix (stratum names, dim labels, maturity/origin/stage values, "Due", "Untyped") still rendered in English even on the Arabic locale, plus the new help text needed translations.
>
> **Three fixes shipped in §120**:
>
> 1. **HelpTip uppercase + edge-clip**. `.help-tooltip` now sets `text-transform: none` to override any uppercase ancestor; `font-weight: 400; letter-spacing: normal` for safety. `computeCoords()` clamps the tooltip's `x` coordinate to viewport bounds (190 px conservative half-width + 12 px margin), so triggers near the left or right edge no longer clip the tooltip.
> 2. **i18n fallback chain**. `t` derived in [`src/lib/i18n/index.ts:108`](src/lib/i18n/index.ts:108) now falls back to `en.json` when the active locale's lookup returns the literal key path (i.e. the key isn't in the active locale). Previously, missing keys in non-en locales returned the key string verbatim — a bug that forced the §104/§113 Untyped-label hardcode. With the fallback chain, missing keys display English instead, and partial translation stays graceful while translators backfill. Loaders cast each non-en locale through `unknown as typeof en` to bypass strict structural matching (the runtime fallback handles missing keys cleanly).
> 3. **Full Arabic + English localization of the matrix**. New i18n keys in `inspector360.*`:
>    - `untyped`, `stratum_name_1..8`, `dim_stratum/maturity/origin/stage/review/trails/lenses` (10)
>    - `maturity_seed/sapling/evergreen/canonical/wilting`, `origin_received/discovered/mixed/none`, `stage_fleeting/literature/permanent/synthesis/none`, `review_due/none` (16)
>    - `axis_stratum_label`, `axis_type_label` (2)
>    - `help_axis_stratum/type`, `help_stratum_1..8`, `help_type_*` (8), `help_dim_*` (7), `help_grand_total`, `help_hud_orphan/fragile/blind_spots/tensions` (4) — total 30 help strings
>    - All keys added to en.json (English source-of-truth) and ar.json (full Arabic translation, native-quality terminology). Other 13 locales fall back to English via the new chain — to be backfilled later.
>
> `Inspector360.svelte` updated: every previously-hardcoded label uses `tr($t(key), key, fallback)` where `tr()` is a small helper that returns the translation when present and the English fallback when `$t` returns the literal key. Static `STRATUM_NAMES`, `HELP_STRATUM`, `HELP_TYPE`, `HELP_DIM`, `HELP_GRAND`, `HELP_HUD`, `HELP_AXIS_*` constants removed; only `STRATUM_FALLBACK` retained as the in-component English fallback.
>
> **No backend change in §120** — frontend + i18n only.

> **What changed in v1.22**: Stage 3.1 follow-up — first-time-user `(?)` help affordances on the 360.3D matrix (commit §119). Boss S3.1 finding: "for the first-time user, we need to help them figure out what this matrix is all about. We need to explain each stratum, type, and/or every bit of detail within the 360.3D. By adding a (?) with each one of those elements."
>
> **Shipped in §119**:
>
> 1. **New reusable component** [`src/lib/components/HelpTip.svelte`](src/lib/components/HelpTip.svelte) — small `?` button that surfaces a styled tooltip on hover, and pins-on-click for accessibility / touch (outside-click dismisses). Tooltip uses `position: fixed` driven by `getBoundingClientRect()` so it escapes overflow boundaries. Theme-aware via `--background-secondary` / `--text-normal` / `--text-accent`.
> 2. **30 help markers wired** across the full-window matrix in [Inspector360.svelte](src/lib/components/Inspector360.svelte). Coverage:
>    - Corner cell: 2 (`▲ Stratum` axis legend, `Type →` axis legend)
>    - Column headers: 8 (one per typed direction + Untyped)
>    - Stratum row labels: 8 (L1 Datum → L8 Worldview)
>    - Dimension strip cells: 5 base + 2 conditional (Stratum, Maturity, Origin, Stage, Review, Trails, Lenses)
>    - Grand total Σ in the corner cell: 1
>    - HUD warnings: 4 (Orphan, Fragile, Blind spots, Tensions)
> 3. **Explanation text** authored as one-paragraph descriptions per element. Stratum text covers what kind of note lives at that altitude. Type text covers what the typed link asserts and shows the wikilink syntax. Dimension text covers the source-of-truth + how it's computed. HUD text covers when the warning fires and what it means cognitively. Axis-legend text in the corner cell explains how to read the matrix overall.
>
> **Compact scorecard untouched** — the sidebar widget is too narrow for `?` icons. First-time learning happens in the full-window matrix; once Boss is fluent, the scorecard reads at a glance.
>
> **No backend change in §119** — frontend-only.

> **What changed in v1.21**: Sky View inspect-mode lockout fix (commit §118). Bug Boss reported on 2026-05-01: in Sky View, click a node → app opens that note as a tab → close that tab via its own × (rather than via the "Return to Sky View" dismiss pill) → app locks; both sidebars refuse to open from their toggle buttons; only recovery is restarting the app.
>
> **Root cause**: clicking a Sky View node calls `handleSkyNodeClick` which (1) snapshots the current sidebar state to `sidebarSnapshots.get('skyInspect')`, (2) hides both sidebars, (3) sets `skyViewInspectMode = true`. The intended exit is a pill rendered next to the active tab — clicking its body returns to Sky View, clicking its `×` dismisses inspect mode and pops the snapshot. **But the pill only renders while `$activeTab?.path` is truthy** ([+layout.svelte:4439](src/routes/+layout.svelte:4439)), and the sidebar toggle handlers are guarded by `!skyViewInspectMode` ([+layout.svelte:1660-1661](src/routes/+layout.svelte:1660)). Closing the tab via its own × clears `$activeTabId` to `null` → pill disappears with the tab → flag stays `true` → toggles refuse to fire. Locked.
>
> **Fix shipped in §118**: a `$effect` in [+layout.svelte:586-590](src/routes/+layout.svelte:586) watches `skyViewInspectMode` and `$activeTabId`. When the tab goes null mid-inspect, it runs the same cleanup the dismiss × button runs — `popSidebars('skyInspect')` to restore the pre-SV sidebar layout, then sets `skyViewInspectMode = false`. Tab-close-via-X now exits inspect mode cleanly. Frontend-only fix; the dismiss pill itself is unchanged for users who use the intended path.

> **What changed in v1.20**: Verification B Check-2 follow-up (commit §117). Boss accepted §115's column-header text colour change but flagged the background tint as still too strong: "lower the tinted background more." §117 reduced the tint from 10 % type-colour mix to 5 %. Text colour and bottom-border colour kept the §115 values. One-liner CSS change.

> **What changed in v1.19**: Verification A retest fixes (commit §116). Boss tested the §115 list-of-titles and surfaced two issues:
>
> 1. **Cell expansion persisted across navigation.** Click a list item → matrix moves to new note → previously-expanded `(stratum, type)` cell stayed expanded on the new note. Boss: "It should collapse by default when we move to another node." Same on back-bar return: "When we are back, it should collapse automatically."
> 2. **Untyped should be expandable too.** Boss originally directed (S1.3.5 in §114) to exclude Untyped because dot-grid expansion at 800+ would balloon the matrix. §115 reworked expansion as a scrollable title list, which contains the size cleanly. Boss: "Let's have the 'untyped' expandable like the other type."
>
> **Fix shipped in §116** (frontend-only):
>
> 1. **Auto-reset on navigation**: a `$effect` watches `data?.note_path` and resets `expandedCells = new Set()` whenever it changes. Covers both forward (title-click → onNoteClick fires → parent updates `data` → effect runs → state clears) and backward (back-bar → onBack restores prior `data` → same path).
> 2. **Untyped exclusion removed** from `toggleCellExpand` and the template branch. The `+N` chip on Untyped is now a clickable button just like the seven typed columns. The list view caps at 240 px with internal scroll regardless of count, so Untyped's typically-large overflow is contained.
>
> **No backend change in §116** — frontend-only.

> **What changed in v1.18**: Stage 1 + Stage 2 retest follow-up bump (commit §115) — six refinements bundled into one rebuild after Boss walked all 6 + 6 sub-stages of the matrix tutorial.
>
> **Six fixes shipped in §115** (frontend-only):
>
> 1. **Expanded typed-cell renders as a list of note titles, not more dots.** S1.3.5 surfaced this: when the user clicked `+N` on a typed cell, §114's design just showed all the hidden dots — visually overwhelming for cells with 30+ connections, and the user still had to hover each dot to learn the name. New design: clicking `+N` switches the cell into a **vertical list of note titles**, each clickable to navigate. Dot bullet shows the type colour beside each name.
> 2. **Always-visible `×` collapse button** at the top-right of the expanded list. Replaces §114's `−` button which was at the *end* of the dots and easy to miss when the cell scrolled. Now positioned absolutely so it stays visible regardless of scroll.
> 3. **Max-height + internal scroll** (240 px) on the expanded list so very large typed cells (e.g. Abu Bakr's L7-Supports with 49 connections) don't balloon the row past the canvas. List scrolls inside the cell.
> 4. **Active-note name chip removed** from the row label. The note's name is already visible in the matrix header at the top; repeating it on the active stratum row was redundant. Active row is still highlighted in the theme accent (purple band + accented row number) — that signal is preserved.
> 5. **Column-header text contrast.** §113's gradient used 22 % type-colour tint with text in the same hue, which read as colour-on-same-colour. Reduced tint to 10 % and switched text colour to `color-mix(var(--col-color) 55 %, var(--text-normal))` so text stays type-coded but lifts off the background. Bottom border keeps the full-strength type colour for the visual signal.
> 6. **Grand total visible** in the top-right corner cell (the row-totals header). New layout stacks `Σ` symbol over the matrix-wide grand total of all (deduped per cell) connections. Confirms at a glance that column-totals sum equals row-totals sum equals this number.
>
> **No backend change in §115** — frontend-only. The §112 backend (`stratum: u8` on `LinkedNote` + `precompute_all_strata`) stays as-is.

> **What changed in v1.17**: a Stage 1.2 retest fix bump (commit §114). The §113 "2× sizes" directive overshot for the full-window matrix — Boss confirmed S1.1 (compact scorecard) but flagged S1.2 (full-window matrix) with two findings: "Minimize by 1" (sizes too big) and "L1 missing, L2 cut" (the bottom of the matrix was clipped by `overflow: hidden` because 8 rows × 110 px row-min exceeded the canvas height).
>
> **Fix shipped in §114** (frontend-only, full-window only — compact scorecard untouched):
>
> 1. **Full-window matrix scaled down ~25 %.** `360.3D` label 32 px → 24 px, brain icon 56 px → 40 px, header name 44 px → 32 px, strip label 22 px → 16 px, strip value 30 px → 22 px, column name 18 px → 14 px, column count 26 px → 20 px, row num 26 px → 20 px, row name 24 px → 18 px, active chip 20 px → 15 px, HUD font 28 px → 21 px, dot 16 px → 13 px. Padding tightened to match.
> 2. **Cell row min reduced from 110 px → 78 px** (and column min 120 px → 96 px, row-label column 280 px → 220 px, row-total column 100 px → 76 px). All 8 stratum rows now fit in a typical 1080p viewport without clipping.
> 3. **`min-height: 0`** on `.i360-matrix-wrap` so the matrix can shrink in tight viewports rather than getting clipped.
>
> **Compact scorecard unchanged**: Boss explicitly passed S1.1 at the §113 sizes (1.85rem name, 1.4rem pills, 14 px bar height), so those stayed.

> **What changed in v1.16**: a Stage-1-tutorial fix bump for the §112 Stratification Matrix (commit §113). Boss walked S1.1 → S1.6 in sequence and recorded seven refinements; all of them landed in one rebuild rather than commit-per-fix.
>
> **Fixes shipped in §113**:
>
> 1. **`Untyped` label hardcoded** in both the compact bar chart label and the matrix column header. The §104 fix had been preserved across the spherical line until §112 reverted it, and the i18n-key leak (`inspector360.unty…`) returned in Stage 1.1. The fix is the same as §104's: `$t('inspector360.untyped')` returns the literal key string when the translation is missing, which is truthy, so the OR fallback never fires; hardcode `'Untyped'` for that one type, keep `$t()` for the seven typed directions where the keys exist in en.json.
> 2. **Compact bars switched from max-normalised to percent-of-total.** Boss's "Abu Bakr" test note had Untyped=6,107 vs Supports=101 — max normalisation collapsed every typed bar to ~1.6% width and made them invisible. Each bar now fills its share of total connections and the right-hand number reads `X.X%` (or `—` for zero). The shape of the share, not the absolute count, carries the cognitive signal.
> 3. **Compact scorecard text and figures roughly doubled.** Card name 0.95rem → 1.85rem, pills 0.72rem → 1.4rem, bar height 8 px → 14 px, label column 90 px → 130 px, count column 28 px → 60 px to fit `100.0%`.
> 4. **Full-window background and chrome are now theme-aware.** Hardcoded `#060612` / `#0a0a1c` / `#060614` and `rgba(255,255,255,0.X)` greys replaced with `var(--background-primary)`, `var(--background-primary-alt)`, `var(--background-secondary)`, `var(--text-normal)`, `var(--text-muted)`, `var(--text-faint)`, `var(--text-accent)`, `var(--background-modifier-border)`. Active-row purple now derives from `--text-accent` via `color-mix`, so it follows the theme accent instead of locking to a single hex.
> 5. **Full-window `360.3D` header label doubled** (16 px → 32 px). Brain icon 28 px → 56 px. Active-note name 26 px → 44 px.
> 6. **Full-window matrix text and figures doubled.** Strip labels 11 px → 22 px, strip values 16 px → 30 px. Column headers 10 px → 18 px. Column counts 14 px → 26 px. Row labels 13 px → 24-26 px. Active chip 11 px → 20 px. HUD text 16 px → 28 px. Dot size 11 px → 16 px (subset; doubling fully would break 16-dot density per cell). Cell row height 72 px → 110 px. Row-label column 200 px → 280 px; row-total column 64 px → 100 px; column min 80 px → 120 px.
> 7. **Hover label moved from the fixed top-right of the matrix to a floating tooltip that sits directly above the hovered dot.** The previous placement (which I'd justified as "doesn't follow mouse, doesn't pop chrome on dense rows") forced the user to look away from the dot they were hovering. New placement uses `position: fixed` driven by the dot's `getBoundingClientRect()` so it escapes `overflow: hidden` on the matrix and works regardless of cell layout.
>
> **No backend change in §113** — frontend-only. The §112 backend (`stratum: u8` on `LinkedNote` + `precompute_all_strata`) stays as-is.
>
> **Process note**: I bundled S1.1 through S1.6 into one tutorial message and Boss flagged the staging violation early. The remaining sub-stages were sent one at a time (S1.2 alone, then S1.3, then S1.4, etc.). `feedback_staged_tests.md` interpreted strictly going forward — one focused test per turn, never a numbered list of tests in a single message.

> **What changed in v1.15**: the 360.3D Inspector redesign lands as code (commit §112). The concept paper (v1.0) was approved; the clean-slate redesign is the **Stratification Matrix**.
>
> **The matrix in one sentence**: an 8 × 8 grid where the **vertical axis is stratum** (L8 Worldview at the top → L1 Datum at the bottom) and the **horizontal axis is link direction** (the 7 typed directions + Untyped). Each connected note becomes a small dot in the cell at the intersection of its own stratum and the typed direction it shares with the active note. The active note's row is highlighted; **empty cells are visually present** (diagonal stripes) so absence reads as readily as presence — Concept Paper §4.3 "Absence is first-class."
>
> **Why this is the right shape (vs spheres / sectors)**: stratum is the dimension Constellation alone measures, and the matrix puts it on the dominant visual axis (vertical position = altitude in the knowledge hierarchy). Typed direction now has its own dedicated lane instead of competing with stratum on a polar layout. Counts read at a glance: column totals tell you which directions you over- or under-use; row totals tell you which strata your thinking spans. Gaps (empty rows = strata you haven't reached; empty cells = directions you don't use at this stratum) are part of the geometry, not afterthoughts.
>
> **Backend addition** ([`inspector360.rs`](src-tauri/src/inspector360.rs)): `LinkedNote` now carries `stratum: u8`. A new `precompute_all_strata()` helper computes every note's stratum once at the top of `get_360_view`, building an inbound-count + sources-of map up front so each `LinkedNote` can be stamped in O(1). Total cost stays O(N + total_links) — same big-O as before. The same rule set used for the active note (`compute_stratum_for_note`) is reused for connections.
>
> **Frontend rewrite** ([`Inspector360.svelte`](src/lib/components/Inspector360.svelte)): the spherical line — `SECTOR_MAP`, `polarToXY`, the three viz-mode toggle (Atmospheric / Neural / Cosmic), `ringsLayout`, `layoutMode`, `allNodes`, `vizMode` — is gone. Full-window mode is the matrix on an HTML/CSS grid (no SVG polar coordinates). Compact sidebar is now a **scorecard**: note name + stratum pill + maturity pill + ↑outbound/↓inbound/word counts + a per-type bar chart with explicit "—" markers for blind spots + a flags row. The matrix is too dense for a 280 px-wide sidebar; the scorecard is the right read at that scale.
>
> **Preserved from §107 / §109**: hover-only labels (no always-on names cluttering pattern reading), per-render `uniqueId` keying so empty-path collisions don't multi-highlight, multi-hop back-stack for click-to-navigate. Universe switch still resets the back-stack to `[]`.
>
> **Dropped permanently**: `vizMode` dropdown, polar / angular layout primitives, `SECTOR_THRESHOLD` hybrid logic, depth-based ring assignment, count-based ring assignment. The §110 binary (the previous "final iteration" of the spherical line) is no longer the latest runnable Inspector — the §112 binary is.

**Author of facts: Eisa ALSHAMSI (project owner, designer, IT Boss).**
**Maintainer: Claude (consultant / engineer / SME).**

---

## 0. How to use this document

**This is the first document any new Claude session reads.** It exists so a fresh AI can get to architectural fluency in one read instead of rediscovering the project from `git log` + screenshots over several frustrating turns.

**Maintenance is a Standing Order** (`CLAUDE.md` Standing Order #6). Whenever a fact below changes — a phase ships, a rule is added, a doc-drift item is fixed, a migration closes — update this file in the same commit that lands the change. Bump the version when the structure changes; date-stamp every section that updates. **The filename always carries its version suffix**: `Constellation Orientation & Onboarding v1.0.md`, `... v1.1.md`, `... v1.2.md`, etc. **Each new version is written as a NEW file alongside the existing ones — older versions are NEVER deleted.** They remain in `docs/` as a historical record the project owner uses to track how orientation evolved. A new session reads only the highest-version file, but the trail behind it is durable.

**This document is grounded.** Every claim cites the authoritative source (file:line, commit hash, or session log section). When two project documents disagree, I name both and don't pick a winner unless code-reading resolves it. When I don't know something, I say so explicitly in §17.

**Hard rule for every reader (human or AI) of this file**: if you find this document contradicts the actual codebase or a more recent session log, **trust the code and the session log first**, then update this file in the same session.

### v1.14 changelog (vs v1.13)

v1.14 was a clean-slate reset for the 360.3D Inspector (commit §111) on 2026-04-30. After five attempts (§104, §106, §107, §109, §110) at the spherical / orbital / compass-position layout — exceeding LL-014's three-attempts rule — Boss invoked the rule and directed a return to first principles.

Two artefacts shipped in §111 (no code change):

1. **Concept Paper v1.0** — `docs/360.3D-Concept-Paper-v1.0.md`. Defines what 360.3D is, why it exists, what cognitive dimensions it encodes, the three outputs the user should leave with (Position / Connection Profile / Absence), the eight design principles any 360.3D visualisation must satisfy, and what 360.3D is NOT (vs Sky View, Map, Sight, Index, OrgChart). Recommended starting axis: **stratum**.

2. **Orientation v1.14** — captured the reset and the pending clean-slate redesign.

The redesign itself shipped in §112 — see v1.15 above.

### v1.13 changelog (vs v1.12)

v1.13 was a sector-layout fix (commit §110) on 2026-04-30. The §109 depth-based rings didn't help "1902"-class data because `inspector360.rs::get_360_view` stamps every outbound and inbound link with `depth = 1`. §110 replaced depth-based with count-based ring assignment: typed groups sorted by count, distributed across the inner two rings (smallest typed → inner 160, largest typed → middle 270); untyped always on the outer ring 380. Three reliably distinct rings, no typed/untyped collision. **§110 is the final iteration of the spherical layout line — see v1.14 for the clean-slate reset.**

### v1.12 changelog (vs v1.11)

v1.12 was a sector-layout course-correction (commit §109) on 2026-04-30. **Restored depth-based sector rings** `[160, 270, 380]` (matching the compact widget). Each typed group's nodes cluster at their SECTOR_MAP compass angle with the widget's 8°-per-node spread; ring radius determined by note depth. **The §109 fix was insufficient for "1902"-class data** because the IPC always stamps typed links with depth=1, so every typed node piled onto the inner ring 160 and untyped depth-1 collided with them. §110 (v1.13) corrected this with count-based ring assignment.

### v1.11 changelog (vs v1.10)

v1.11 was a Stage 2B retest follow-up (commit §107) on 2026-04-30. Boss reported two findings on the v1.10 binary.

Two changes in §107:

1. **Single-ring sector layout** (interpreting "Distribute all nodes in one circle"): replaced §106's three depth-based rings with a single ring at `SECTOR_RADIUS = 290`. **This was an over-correction; §109 restored depth-based rings.**
2. **Hover label leak fix**: each rendered node now carries a `uniqueId`; hover state renamed `hoveredNode → hoveredId` keying on it instead of `node.path`. Fixes the empty-path collision (`inspector360.rs::get_360_view` returns `path: ""` for outbound links to notes outside the library). **This fix is preserved post-§109.**

### v1.10 changelog (vs v1.9)

v1.10 was a tuning bump for the Stage 2B sector layout (commit §106) on 2026-04-30. Boss reported during Stage 2B retest that the §104 sector mode rendered the test note "1902" too sparsely on the full-window canvas. Boss directive: "It has to be similar to the widget."

Two changes in §106:

1. **Sector spread formula switched** from §100's normalised cap to **the compact widget's exact formula** `(i - (n-1)/2) * 8`. Trade-off: large sectors bleed past their 50° semantic slot into adjacent compass directions. The widget shows this; Boss accepted.
2. **`SECTOR_THRESHOLD` raised** from 8 → **30**. Notes with up to 30 typed-link connections per group now use sector layout; Abu Bakr-class hubs still trigger ring-per-group.

### v1.9 changelog (vs v1.8)

v1.9 was a **CE Phase 12 hardening / refinement bump** (commits §96–§104, ten commits since v1.8 closed) on 2026-04-30. Phase 12 became user-testable on 2026-04-29; Boss tutorial-tested it across Stage 1 and Stage 2 over two days, and every iteration rolled into a fix-and-rebuild loop. Net result: the 360° Inspector surface that v1.8 announced as "enabled" is now the surface the Boss is actually using.

Highlights:

1. **Stage 1 hotfix (§96)** — clicking the new right-sidebar 360° tab routed the user back to Properties because a safety `$effect` (`+layout.svelte:1255`) was force-resetting `rightSidebarTab` to the first known visible tab. The `tabVisible` map and fallback `order` array missed `inspector360`. Fixed; tab now sticks.
2. **rs-tabs strip overflow fix (§97)** — adding the 11th tab pushed past the default 340 px sidebar width; the new tab clipped at the right edge. Pure CSS: replaced default `<button>` padding with explicit `padding: 0; flex: 1 1 28px; min-width: 24px; flex-wrap: wrap;`. Tabs now wrap to a second row instead of clipping.
3. **Compact-mode back-nav (§98 → §99)** — Boss requested a "back to source note" affordance inside the compact widget. Started as single-step (§98) then upgraded to a **multi-hop stack** (§99) per Boss directive: walks all the way back through any chain. State: `inspector360BackStack: $state<Array<{path, name}>>`. Universe switch resets the stack to `[]`.
4. **Stage 2 omnibus (§100)** — five Stage 2 findings: dock-button tooltip i18n leak (`ribbon.inspector360` key returned verbatim because `$t()` returns the key on miss); viz didn't fill canvas (removed `max-width: 1400px; max-height: 900px;` from `.i360-viz`); side panels + HUD doubled in size; tighter sector grouping `(i / (n-1) - 0.5) * 50`; full-window auto-close removed in favour of "Return to {previous}" header button.
5. **Sector → ring-per-group → hybrid (§101 → §102 → §104)** — three iterations on visualisation layout. §104 made the choice automatic: sector layout when max typed-group count ≤ `SECTOR_THRESHOLD = 8`, ring-per-group when above.
6. **Minimised nodes + hover-only labels (§103)** — node radii reduced 10/7/4 → 6/4/3. Always-on labels removed; hover-only with 13 px font + 3 px black SVG stroke. 6 px invisible hit-area expansion.
7. **Dedupe by path + Untyped label fix (§104)** — frontend dedup per-group in `ringsLayout` (the IPC returns the same note from outbound + inbound + second-order). Untyped label hardcoded `'Untyped'` to skip the broken i18n fallback.

**Boss's perf verdict on Phase 12**: first-fetch "almost instantly". **MIG-010 priority dropped to LOW** based on lived experience.

**Process violations recorded for the day**: (a) the over-long Stage 2 tutorial bundled 2.1–2.7 in one message — `feedback_staged_tests.md` rule. (b) Standing Order #6 violation: §96–§104 shipped without bumping the orientation in the same commit. **v1.9 was the catch-up bump.**

### v1.8 changelog (vs v1.7)

v1.8 captured three landings on 2026-04-29:

1. **MIG-003 integrated to main** via fast-forward of `claude/frosty-stonebraker-75c9bf` (the side branch that closed MIG-003 on 2026-04-28 but was never merged). `origin/main` moved from `6545b3e` (MIG-008/009 tip) to `8cb80ac` (MIG-003 handover). Three byte-identical "stranded" closure docs in main's working tree (the v1.7 file, SESSION-LOG-2026-04-28.md §85–§89, CANONICAL-FILENAME-ARCHITECTURE.md updates) became tracked. Source ↔ binary parity restored at main by copying the post-MIG-003 release artifacts from the frosty worktree.
2. **CE Phase 12 360° Inspector re-enabled** (§93 + §94 + §95). Backend `get_360_view` IPC was already shipped from earlier work; only the import + UI wiring was gated at `+layout.svelte:84`. Re-enable shipped both surfaces: a compact right-sidebar tab and a full-window overlay reachable from a new ribbon-dock button. IPC fetch debounced 200 ms with sequence-guard + last-fetched-key dedup; lazy-mount via `inspector360EverOpened`. The `get_360_view` IPC walks the full library on every call (acknowledged Rule-8 violation); MIG-010-scale work to cache `note_360_view` was queued, contingent on Boss's perf verdict.
3. **CE Phase 9 Multi-Lens approved for re-wire on Path B** (Rule-8 compliant) — queued after MIG-006 §3 redo. `lenses.rs::apply_lens` stays dead until that future MIG-010-scale migration.

### v1.7 changelog (vs v1.6)

v1.7 captured MIG-003 closure (Human-name Filenames) on the side branch `claude/frosty-stonebraker-75c9bf`. § 6 fully rewritten to reflect the inverted architecture: `cid_cn` is the immutable internal id (frontmatter only), filenames are human-readable. § 8 migration table updated to mark MIG-003 closed. The Canonical Filename Architecture design doc was given a Post-MIG-003 historical banner. Visible behavior change: every `.md` file on disk now has a human title as its filename; renames cascade through every dependent table (`note_meta`, `note_links`, `sky_nodes`, `note_aliases`, `note_embeddings`).

**Important context for any reader of v1.7**: at the time v1.7 was written, the seven MIG-003 commits + this v1.7 file itself + the closure session-log entries + the CANONICAL-FILENAME-ARCHITECTURE.md updates **only existed on the `claude/frosty-stonebraker-75c9bf` branch and as uncommitted/untracked files in `main`'s working tree**. They were not on `origin/main`. The stranded state was discovered and resolved at the start of the 2026-04-29 session via `git merge --ff-only` (see v1.8 note above). v1.7's "MIG-003 closed" claim was correct — but only on the side branch; the main-line integration arrived a day later.

### v1.6 changelog (vs v1.5)

v1.6 captures two cleanup migrations shipped on 2026-04-27 / 28:

**MIG-008 — Canonical Naming Cleanup** ✅ closed.

- Added shared helper `note_display_name(path, content_opt)` in [`libraries.rs`](src-tauri/src/libraries.rs) — smart enough to skip the file read for human-named files (file_stem IS the title) and only pay the I/O cost for canonical-named files.
- Patched ~14 sites across `map.rs`, `inspector360.rs`, `strata.rs`, `maturity.rs`, `provenance.rs`, `review.rs`, `lenses.rs`, `tasks.rs`, `tension.rs`, `libraries.rs::scan_index_words_recursive`, `trails.rs::find_note_recursive`, `universe.rs::collect_templates_recursive` — all switched from `path.file_stem()` to the helper so user-visible labels show frontmatter title instead of canonical filenames.
- Two of those changes are **correctness fixes**, not just label fixes: `inspector360.rs:88` (now matches incoming wikilinks for canonical notes) and `trails.rs::find_note_recursive` (canonical notes were unfindable by name lookup).
- User-verified across Stages 1, 3, 4a/4b, 5 (Constellation Map, Strata + Maturity + Provenance, Tasks, Review Pulse, Tension via Health). Stages 2 (Inspector 360) and 4c (Multi-Lens) skipped — surfaces are deliberately disabled or dead in current builds (see below).
- Phase 4 audit clean: invariant check / drift check / migration-path check all PASS.

**MIG-009 — Lens-to-Sight Naming Cleanup** ✅ closed.

- Renamed `src-tauri/src/lens.rs` → `src-tauri/src/sight.rs` to align the analytics module's filename with its UI surface (Constellation Sight, formerly Constellation Lens).
- Renamed Tauri commands: `constellation_lens_centrality` → `constellation_sight_centrality`, `constellation_lens_tag_edges` → `constellation_sight_tag_edges`. Frontend `+layout.svelte:3235` invoke updated atomically.
- Frontend JS variable names (`lensActive`, `toggleLens`, `lensCentrality`, `lensCommunities`, `lensCommunityAssignments`, `lensGaps`, `lensHealth`, `lensLoading`, `lensDataStale`, `availableLenses`, `activeLensId` — ~60 occurrences) intentionally **not** renamed; deferred as bookkeeping with no architectural payoff.
- `src-tauri/src/lenses.rs` (plural — CE Phase 9 Multi-Lens) **NOT renamed** — separate concern, deferred to whenever CE Phase 9 is resumed (see "dead-code finding" below).
- User-verified: Constellation Sight still renders centrality + community + gaps after rebuild.

**Dead-code finding** (catalogued, not fixed in this bump):

- `lenses.rs::apply_lens` has **zero frontend callers**. Verified by exhaustive grep on 2026-04-27. The Settings UI can still create + save lens definitions via `list_lenses` / `save_lenses`, but those definitions are never applied to anything. The Multi-Lens (CE Phase 9) IPC pipeline is dead-on-arrival.
- Decision deferred: either delete `lenses.rs` + the Settings lens-definition UI, or re-wire `apply_lens` into a real surface (Sight or a separate panel). Tracked in `project_lenses_apply_lens_dead_code.md` memory.
- MIG-008's patches to `lenses.rs::scan_property_recursive` and `scan_tags_lens_recursive` ship harmlessly but don't run today. Don't revert; the code is correct should the wiring be restored.

**UI / surface notes locked into memory this session:**

- Constellation Lens / Multi-Lens UI surface was renamed to **Constellation Sight** earlier (`feedback_lens_renamed_to_sight.md`). Internal Rust file was just renamed to match (MIG-009).
- 360° Inspector frontend component is deliberately disabled at [`+layout.svelte:84`](src/routes/+layout.svelte:84) — Rust backend (`inspector360.rs`) ships ready, but no UI surface mounts it today.

**New backlog items**:

- Decide fate of CE Phase 9 Multi-Lens (delete vs re-wire). Tracked.
- Decide fate of CE Phase 12 360° Inspector (re-enable vs withdraw).
- `docs/IPC-CONTRACT.md` is now even staler — missing the `constellation_sight_*` rename. Doc-drift item.

### v1.5 changelog (vs v1.4)

v1.5 is a focused-fix bump for the Unlinked Mentions panel (item 6 from the option-(e) backlog). User-verified 2026-04-27 ~18:00.

**§90 — Unlinked Mentions panel: scanner fix + frontmatter-title label**

Two bugs in `scan_unlinked_mentions` ([`libraries.rs:1665-1759`](src-tauri/src/libraries.rs:1665)) closed in one commit:

1. **Scanner false-positive on typed/aliased wikilinks.** The previous "skip source if `[[NoteName]]` substring is present" check was too narrow — every typed-link form `[[NoteName|supports]]`, every alias form `[[OldTitle]]`, and every embed `![[NoteName]]` slipped past it. The active note's title would then be matched as plain text *inside the wikilink markup* and counted as an unlinked mention. Fix: strip ALL wikilinks (regular + embed forms) from content before plain-text scanning. The regex `!?\[\[[^\]]*\]\]` removes them all in one pass.
2. **Source-row label was canonical filename, not human title.** Filename for canonical notes (`20260426T140940Z_NOTE_11B4`) is unreadable; users couldn't tell which note was being shown. Fix: prefer `extract_frontmatter_title()` (already used by the rename path), fall back to `path.file_stem()` only when title is missing.

**Side benefit.** Both fixes are upstream in Rust, so any future caller of `scan_unlinked_mentions` automatically gets correct behavior. No frontend changes needed; the existing `BacklinksPanel.svelte` Unlinked-Mentions section renders the corrected data unmodified.

**What this closes from §12 / §13 / backlog**:
- Item 6 (Unlinked Mentions double-count + canonical filename label) — both bugs fixed.
- The "(e) didn't fully cover item 6" gap I owned in v1.4 — now closed.

**Open items still in the backlog** (unchanged from v1.4 plus the snapshot-path mystery and second-screen alias):
- MIG-007 — Links Settings tab consolidation.
- Constellation Map: tooltip canonical-filename + search highlight + suspected memory leak (the canonical-filename label fix in §90 does NOT propagate to the Map — Map uses a different code path; that's still pending in `project_constellation_map_backlog.md`).
- SecondScreenPage.svelte buildSkyData calls still alias-blind.
- Architectural mystery: why is `cache_boot_snapshot_sky` bypassed at boot in builds that contain MIG-001/MIG-004 §8.

### v1.4 changelog (vs v1.3)

v1.4 captures the 2026-04-27 work session: MIG-005 Tutorial #1 testing, the Sky View edge regression fix (§88), the panel-dedupe fix (§89), and a basket of new backlog items the testing surfaced.

**Architecture / fixes shipped:**

- **§88** — `buildSkyData` fallback now alias-aware. The legacy graph-population path that runs when `cache_boot_snapshot_sky` is bypassed had no alias resolution; renamed-target wikilinks were silently dropped, leaving renamed notes as bubble-without-edges in Sky View. Fix at [`store.ts`](src/lib/libraries/store.ts) buildSkyData now accepts an optional `notePathToAliases` map and applies the same 3-tier resolution as `cache.rs::read_sky_links_raw`. User-verified.
- **§89** — Backlinks / Outgoing Links panel dedupe. A source note with both `[[Note]]` (regular) and `[[Note|supports]]` (typed) targeting the same active note used to render twice — once with no badge, once with the type badge. Now grouped by source path (Backlinks) / target name (Outgoing) into ONE row carrying a `linkTypes[]` array of all distinct typed-link badges. Helper `dedupeBySource` in `store.ts`. Same change includes annotation-redundancy suppression: when a typed-link annotation IS the typed-link keyword (e.g. `[[Note|supports]]` stores "supports" in both slots), the redundant italic prose underneath the badge is now suppressed.
- **Badge taxonomy update**: **M = Mutual link** confirmed by project owner 2026-04-27. Moved out of Unresolved into the link-relationship table in `Badge-Taxonomy.md`. **No more pending badge letters.** §13.1 here updated to match.

**New backlog items surfaced this session:**

- **Auto-update Links toggle is misplaced** under "Sky View & Links". Decision 2026-04-27: a new "Links" Settings tab will consolidate every link-related control. Will be **MIG-007** when greenlit. *(Reverses the v1.2 §12 entry that wrongly "corrected" v1.0's right call.)*
- **Constellation Map UX bugs**: tooltips show canonical filename instead of human title; search doesn't highlight matched arc; suspected memory leak / slowness. All filed in `project_constellation_map_backlog.md`.
- **Unlinked Mentions panel** double-counts wikilink occurrences as unlinked mentions (the scanner doesn't strip wikilink syntax before matching) AND shows source label as canonical filename instead of human title.
- **SecondScreenPage.svelte buildSkyData calls** still use the 2-arg form (alias-blind). Same rename-drops-edges symptom there until threaded.
- **Architectural mystery**: even with MIG-005/MIG-004 §8 in the binary, the alias-aware sky snapshot path (`cache_boot_snapshot_sky`) appears to be bypassed at boot — the legacy `buildSkyData` runs instead. The §88 defensive fix neutralizes user-visible impact, but the underlying "why" is unresolved. Filed for follow-up forensics.

**New top-principal rules / Standing Orders saved this session:**

- **Standing Order — staged tests**: split test tutorials into stages. Send Stage 1, wait for findings, then Stage 2. Never dump 6 tests at once. (Memory: `feedback_staged_tests.md`.)
- **Stage 0 — verify the running binary's mtime** before any test tutorial. The user runs an installed `.exe`, not the source on disk — confirm the binary contains the feature being tested. (Memory: `feedback_verify_binary_before_testing.md`. Earned by the 2026-04-27 incident where I burned hours testing against a binary that pre-dated the feature.)
- **Sky View vs Constellation Map vocabulary** — Sky View has bubbles (PIXI nodes); Constellation Map has sunburst arcs (D3). NOT interchangeable. Same correction had to be made twice. (Memory: `feedback_skyview_vs_map_vocabulary.md`.)

**§17 unknowns reduced:**

- **M = Mutual link** — resolved (see above). Removed from §17.
- Sidebar active-item highlight ~10 s lag — still unresolved.
- 2026-04-16 untracked-backup vs tracked log diff — still unresolved.

### v1.3 changelog (vs v1.2)

v1.3 is a focused correction round driven by [`docs/Badge-Taxonomy.md`](docs/Badge-Taxonomy.md), the canonical badge reference dated 2026-04-15 (predates v1.0). I missed it on every prior orientation pass. Corrections folded in:

- **§13.1** badge table rewritten:
  - **W** = Wikilink (`[[target]]`), grey `#94a3b8` — was "unresolved" in v1.2.
  - **LT** = Link **Target** (this note links *to* the queried note), green — was "Link Type" in v1.2 (wrong).
  - **G** = deprecated, superseded by **#** — added to the table for posterity.
  - The badge set ships in **two** components per the source-of-truth invariant: [`ConstellationMap.svelte:80-84`](src/lib/components/ConstellationMap.svelte:80) **and** [`ConstellationSight2.svelte:79-83`](src/lib/components/ConstellationSight2.svelte:79). Both must agree letter→color.
  - Semantic clarification: badges indicate **where in the note the search query matched** (or what link relationship), not arbitrary note categories.
- **§14** "Where to read what" — new row pointing to `docs/Badge-Taxonomy.md`.
- **§17** unknowns — **W removed** (now resolved). M still pending owner clarification.

### v1.2 changelog (vs v1.1)

v1.2 closes the §17 unread list. Significant additions:

- **§3.2** corrected: `+layout.svelte` reactive declarations are now **155 $state, 29 $effect, 1 $derived** (was 77/17/19 in LL-002 / 2026-03-27 — file has roughly doubled).
- **§3.3** corrected: 32 Rust modules; ~120 commands.
- **§3.5** (NEW): full Rust module sizes — `search.rs` 4790, `libraries.rs` 3978, `universe.rs` 1472, `canonical.rs` 1401, `cache.rs` 824.
- **§4.2** enriched per-phase with the Rust file path, the actual aggregator details for Phase 12, and corrected Phase 9 lenses status.
- **§5** Arabic Engine: confirmed mmap is wired through ([`fst_bake.rs:323`](src-tauri/src/arabic/fst_bake.rs:323)), via `Arc<Mmap>` shared by both stripped + folded FSTs.
- **§5.5** (NEW): ai/, embeds/, embeddings/, tasks/, lens.rs (Brandes betweenness), inspector360.rs, sky_backfill.rs (BATCH_SIZE=1000, INTER_BATCH_SLEEP_MS=50), boot_bundle.rs.
- **§7.1** editor stack now described per-plugin from full reads. Added the LL-014 RULE A / RULE B in `calloutPlugin.ts`.
- **§7.4** (NEW): `store.ts` write-ahead buffer (memory + localStorage), navigation supersede tokens, `recentWrites` 2 s gate, save coalescing.
- **§7.5** (NEW): `secondScreen.ts` event API (12 main→screen, 4 screen→main, 1 bidirectional).
- **§9.3** (NEW): boot-bundle (10 IPCs → 1 round-trip) for early-boot data.
- **§11** LL list now grounded in verbatim text.
- **§12** drift list refreshed: `autoUpdateLinks` toggle is **correctly under "Sky View & Links"** (v1.0 misclaimed it as misplaced); `IPC-CONTRACT.md` still 4 weeks stale.
- **§13** badge taxonomy resolved: **T/C/P/S confirmed**; **#, ∅, W, M and LT/LF/⇄/LB/LA also defined** in `ConstellationMap.svelte:80-84`. **W and M letter meanings remain unresolved** (no doc found; honest).
- **§13** auto-update-links toggle confirmed at Settings → **Sky View & Links** (not "Files" as v1.0 wrongly suggested).
- **§14** corrected `lib.rs:233-432` line range.
- **§15.3** (NEW): collision tiebreak — name wins over alias; identical-alias multi-target is **first-write-wins, undefined order**.
- **§17** dramatically reduced — every Rust module read; every CM6 plugin read; every major Svelte component surveyed; `store.ts`, `secondScreen.ts`, `universe/store.ts` read; user manual + 24 help topics + BASES_MVP_SPEC + Concept Paper + Editor-Spec + eNotePane-development-record indexed; 14 translated User Manuals confirmed (ar = 1328 lines, others = 1120, parity confirmed); 20 session logs digested chronologically.
- **§17 remaining unknowns**: badge letters W and M (defined in code but undocumented); sidebar active-item highlight ~10 s lag origin (no reactive source isolated).

---

## 1. What Constellation IS

**Constellation is a Personal Knowledge Formulation desktop application.**

The distinction is fundamental — it is **not** PKM (Personal Knowledge Management):

> Knowledge Management asks: "Where did I put that?"
> Knowledge Formulation asks: "What can I BUILD from what I know?"
> *(`docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md:13-17`)*

It is built on **standard Markdown files** (`.md` + YAML frontmatter) on the user's local filesystem, with a portable Universe-config layer above. Local-first, no telemetry, no cloud, no account.

- **Author**: Eisa ALSHAMSI
- **License**: MIT
- **Repository**: `github.com/eisaShamsi/Constellation`
- **Stack**: Tauri v2 (Rust backend) + SvelteKit + Svelte 5 + SQLite (rusqlite, bundled) + ONNX Runtime (`ort`) + CodeMirror 6 + PIXI v8 + D3 v7
- **Languages supported at launch**: 15 — `ar / de / en / es / fa / fr / he / hi / ja / ko / pt / ru / tr / ur / zh`
- **RTL languages first-class**: 4 — Arabic, Hebrew, Persian, Urdu
- **Platforms**: Windows, macOS, Linux desktop. CI ships Windows builds today.
- **Mobile**: iOS/Android excluded via `cfg(not(any(target_os="ios", target_os="android")))` for `memmap2`. Not shipping mobile apps.

---

## 2. Universe / Library / Note hierarchy

Constellation has a **five-level knowledge hierarchy**:

```
Universe (root, named by user, contains universe.json)
  └── cUniverse (child universe — federation of libraries)
       └── Library (self-contained knowledge base, like Obsidian vault)
            └── Folder (subdirectory inside a Library)
                 └── Note (single .md file with optional YAML frontmatter)
```

- **Universe** = portable directory. Contains `.constellation/` subfolder with `universe.json`, `libraries.json`, `settings.json`, `bookmarks.json`, `workspaces.json`, `property-types.json`, `bases/`, `templates/`. Move it to another machine and the entire workspace follows.
- **Library** = first-class citizen with its own color/appearance/tags/links/index. Registered in `libraries.json`. Multiple libraries coexist in one Universe. Constellation reads them in place — never copies.
- **Folder ≠ Library**. Folders are organizational only.
- **Terminology**: use "Library" everywhere, **never** "vault" (except for Obsidian import compatibility).

### 2.1 Universe migration (legacy → current)

[`universe.rs::migrate_legacy_data`](src-tauri/src/universe.rs:1306) moves a v1 layout to v2:

- **From**: flat `universe.json` / `vaults.json` / `settings.json` at universe root; registry stored at `app_data_dir/vaults.json`; nested `name/name/` notes layout.
- **To**: `.constellation/` subdirectory; `vaults.json` renamed to `libraries.json`; registry moved to `app_data_dir/universes.json` (UniverseRegistry with `entries` and `active_id`); flat notes layout (Universe root IS the library, Obsidian-style).

`migrate_to_constellation` (line 133), `ensure_universe_notes_folder` (line 195), `set_active_universe` (line 545 — also consolidates same-name nesting `C:\Name\Name\` → `C:\Name\`).

### 2.2 Child-universe federation

[`universe.rs:425`](src-tauri/src/universe.rs:425) `resolve_child_universe_roots(parent)` reads `universe.json::children[]`, canonicalizes, filters directories. `resolve_libraries_recursive` (line 353) collects own + all child libraries, prevents circular refs, deduplicates by path. Frontend command: `resolve_universe_libraries`.

---

## 3. Architecture (one-page view)

```
┌─────────────────────────────────────────────────────────────────┐
│  Frontend (SvelteKit / Svelte 5)                                │
│  src/routes/                                                    │
│    +layout.svelte (6872 lines — orchestrator, see §3.2)         │
│    +page.svelte (1 line — note viewing handled by layout)       │
│    libraries/+page.svelte (704 lines — library management)      │
│    skills/+page.svelte (219 lines — skills/onboarding)          │
│  Second window: static/screen.html (separate Tauri webview)     │
│  Editors: NotePane.svelte (388) / FocusPane.svelte (213)        │
│  Panels: Sky View (PIXI), Constellation Map (D3 sunburst),      │
│    Inspector 360, Tension, Sight, Lens, Bases, Tasks, Calendar, │
│    Backlinks, OutgoingLinks, IndexPanel, OrgChart, SearchHub    │
├─────────────────────────────────────────────────────────────────┤
│  Tauri IPC (~120 commands, 32 Rust modules)                     │
│  - perf_trace (LL-021): every dispatch stamped at the boundary  │
│    via Box-typed closure wrapping generate_handler!             │
│  - 3 plugins: opener / process / updater                        │
│  - panic hook in run() writes constellation-crash.log           │
│    (NO panic-handler plugin — just std::panic::set_hook)        │
├─────────────────────────────────────────────────────────────────┤
│  Backend (Rust, src-tauri/src/, 32 modules — see §3.5)          │
│  - libraries.rs (3978) — file I/O, link extraction, cascade     │
│  - search.rs (4790) — SQLite, FTS5, Living Link triggers,       │
│    sky_nodes/sky_links triggers (Rule 8)                        │
│  - cache.rs (824) — boot snapshot, alias resolution             │
│  - canonical.rs (1401) — YYYYMMDDTHHMMSSZ_KIND_XXXX             │
│  - universe.rs (1472) — universe/cUniverse + legacy migration   │
│  - arabic/ (15 files) — 5-layer morphological engine, mmap'd    │
│  - lexicon/ (6 modules) — Lexical Bridge polylingual lemma graph│
│  - CE Layer 1: strata.rs / maturity.rs / tension.rs /           │
│    provenance.rs / inspector360.rs / lens.rs / lenses.rs /      │
│    review.rs / trails.rs / canvas.rs                            │
│  - bases.rs — .base file CRUD (read-time)                       │
│  - dataview.rs — DQL queries (read-time)                        │
│  - importers.rs — 7 source formats (one-off, async)             │
│  - watcher.rs — notify-rs file watch (must be async)            │
│  - boot_bundle.rs — 10 IPCs collapsed into 1                    │
│  - sky_backfill.rs — resumable populator, BATCH_SIZE=1000       │
│  - embeddings.rs — ONNX multilingual-e5-small (write-time)      │
│  - embeds.rs / fts5_tokenizer.rs                                │
│  - perf_trace.rs — IPC arrival tracer                           │
│  - ai/mod.rs — OpenAI/Anthropic/Gemini/Ollama                   │
├─────────────────────────────────────────────────────────────────┤
│  Storage                                                         │
│  - .md files on disk (source of truth)                          │
│  - SQLite DB at <universe>/.constellation/search.db              │
│    Tables: schema_versions, note_meta, note_embeddings,         │
│    note_links, note_aliases, sky_nodes, sky_links, notes_fts,   │
│    notes_vocab (fts5vocab), sky_backfill_cursor,                │
│    term_vocab [+ bridge_concept_id col post §1C, MIG-013]       │
│    (term_embeddings table retired in MIG-013 §1C)               │
│  - boot-perf.latest.json — per-boot scorecard                   │
│  - .meta.json sidecars for non-markdown files (canonical)       │
│  - .constellation/review-pulse.json — Phase 7 schedule state    │
│  - .constellation/arabic-overrides.json — L5 user overrides     │
│  - kind_registry.json — auto-generated KIND codes (file_kinds)  │
└─────────────────────────────────────────────────────────────────┘
```

### 3.1 Key dependencies (versions)

| Layer | Package | Version | Purpose |
|---|---|---|---|
| Rust | `tauri` | 2.x with `protocol-asset` feature | App runtime |
| Rust | `rusqlite` | bundled | SQLite |
| Rust | `ort` | ONNX Runtime | Semantic embeddings |
| Rust | `tokenizers` | HuggingFace (with `onig`) | Tokenizers |
| Rust | `fst` | BurntSushi | Arabic generative index |
| Rust | `memmap2` | 0.9 (desktop only) | mmap baked Arabic FST — **wired through** [`fst_bake.rs:323`](src-tauri/src/arabic/fst_bake.rs:323) |
| Rust | `notify` | File watcher | |
| JS | `svelte` | ^5.0 | UI framework (runes mode) |
| JS | `@sveltejs/kit` | ^2.9 | Routing |
| JS | `@codemirror/*` | 6.x (full set) | Editor |
| JS | `pixi.js` | ^8.17 | Sky View force graph (LL-019: `pixi.js/unsafe-eval` first) |
| JS | `d3` | ^7.9 | Constellation Map sunburst |
| JS | `@xenova/transformers` | ^2.17 | Frontend ONNX |
| JS | `katex` / `mermaid` / `marked` / `dompurify` | latest | Math / diagrams / markdown / XSS |

Plugins: `tauri-plugin-opener`, `tauri-plugin-process`, `tauri-plugin-updater`. **No panic-handler plugin** — the crash log path uses `std::panic::set_hook` in [`lib.rs:212-222`](src-tauri/src/lib.rs:212).

### 3.2 The `+layout.svelte` reactivity load (corrected counts)

`+layout.svelte` is the orchestrator. **6872 lines as of 2026-04-26.** Reactive declaration counts (verified by Grep this round):

| Kind | Count | LL-002 baseline (2026-03-27) | Change |
|---|---|---|---|
| `$state` | **155** | 77 | +78 |
| `$effect` | **29** | 17 | +12 |
| `$derived` | **1** (`allTagsList`) | 19 | −18 |

Growth drivers: multi-phase graph boot, second-screen sync effects, Tier 1 panel-placement state, child-universe sidebar expansion, lazy-mount flags. The drop in `$derived` count reflects intentional consolidation — derivations now live inside `$state`-bearing handlers or were promoted to module-level helpers.

`+page.svelte` is **a single-line comment** — the entire note-viewing UI is composed inside `+layout.svelte`. The `libraries/` (704 lines) and `skills/` (219 lines) routes are real pages.

**Lazy-mount flags** ([`+layout.svelte:569-572`](src/routes/+layout.svelte:569)): `mapEverOpened`, `orgChartEverOpened`, **`catalogerEverOpened`** (MIG-039). All are sticky $state(false), set true via $effect on their respective `show*` state, **reset in `handleUniverseSwitch`**. Used to gate `{#if *EverOpened}` ... `{#if show*}` two-tier rendering (LL-022 compliance).

**$effect violation candidates flagged** (audit-pending): line 498 (`lastSavedContent` async-race risk per LL-023), lines 781 / 837 / 1235 / 1353 / 1449 / 3480 (always-mounted IPC fan-out — index/sky scans run regardless of visibility).

### 3.3 Tauri command surface

[`lib.rs:233-432`](src-tauri/src/lib.rs:233) registers ~120 commands across 32 modules. The `invoke_handler` is wrapped in a Box-typed closure that records each dispatch via `perf_trace::record(invoke.message.command())` — the LL-021 IPC arrival tracer.

Two Tauri v2 type-system subtleties (from LL-021):

1. `generate_handler!` must be bound via `Box<dyn Fn(Invoke<Wry>) -> bool + Send + Sync + 'static>` to pin the macro's `R: Runtime` generic at the binding site.
2. `invoke.message.command()` returns `&str`; call `perf_trace::record` *before* forwarding to `inner(invoke)`.

**[`docs/IPC-CONTRACT.md`](docs/IPC-CONTRACT.md) is significantly stale** (last updated 2026-03-31; lists ~80 commands of ~120). Until refreshed, [`lib.rs:233-432`](src-tauri/src/lib.rs:233) is the authoritative command registry.

### 3.4 Build / Release / CSP / Windows / Capabilities

**Versions** (aligned at **0.1.0** since v2.18 — Eisa decision: "Constellation will be v.0.1"):
- [`package.json`](package.json) — `"version": "0.1.0"`
- [`src-tauri/tauri.conf.json:4`](src-tauri/tauri.conf.json:4) — `"version": "0.1.0"`
- `src-tauri/Cargo.toml` — `0.1.0`

**`tauri.conf.json` highlights**:
- `productName: "Constellation"`, `identifier: "world.uconstellation.app"`
- Two windows: `main` (1200×800) and `second-screen` (1200×800, `url: "screen.html"`, `visible: false` at startup).
- CSP: `default-src 'self'`; `script-src 'self' 'unsafe-inline'`; **no `unsafe-eval`** → LL-019 still applies (PIXI must use `pixi.js/unsafe-eval` side-effect import).
- Asset protocol enabled, `allow: ["**/*"]`, `requireLiteralLeadingDot: false`.
- Updater enabled, endpoint = public Gist (`gist.githubusercontent.com/.../latest.json`); minisign pubkey embedded.

**Capabilities** ([`src-tauri/capabilities/default.json`](src-tauri/capabilities/default.json)) — applies to both `main` and `second-screen` windows. Permissions: `core:default`, window controls, `core:webview:allow-create-webview-window`, `core:webview:allow-set-webview-zoom`, `opener:default`, `updater:default`, `process:allow-restart`.

**Second-window file**: [`static/screen.html`](static/screen.html) (built copy at `build/screen.html`).

**CI / release** ([`.github/workflows/release.yml`](.github/workflows/release.yml)) — `windows-latest` runner. Tag push `v*` or manual `workflow_dispatch` (bump `patch|minor|major` or `custom_version`). Bumps `package.json` + `tauri.conf.json` + `Cargo.toml` in lock-step, commits, tags, runs `tauri-action`. Post-release, downloads `latest.json` from release assets and `gh gist edit` updates the public Gist that the in-app updater polls.

**No frontend test harness** (no vitest / playwright / `*.test.ts` / `*.spec.ts` under `src/`). Rust unit tests only.

### 3.5 Rust module sizes (full census)

| File | LOC | Role |
|---|---|---|
| `search.rs` | 4790 | SQLite schema + FTS5 + Living Link triggers + search commands |
| `libraries.rs` | 3978 | File I/O + cascade walker + link extraction + 11 cascade tests |
| `universe.rs` | 1472 | Universe registry + child federation + legacy migration |
| `canonical.rs` | 1401 | Canonical filename generation + cid_cn migration + repair |
| `cache.rs` | 824 | Boot snapshots (core/graph/sky) + perf instrumentation |
| `embeds.rs` | 708 | Living embed resolver (`![[target]]`) — 7 resolution tiers |
| `inspector360.rs` | 517 | Aggregates 9 phase data per note (read-only); §112 added per-note `stratum` + `precompute_all_strata` |
| `lens.rs` | 419 | Brandes' betweenness centrality + tag-shared edges |
| `sky_backfill.rs` | 470 | Resumable populator (BATCH=1000, sleep=50ms) |
| `tasks.rs` | 495 | Task scanning (Tasks plugin emoji syntax) |
| `boot_bundle.rs` | 138 | 10 IPCs collapsed into 1 round-trip |
| `tension.rs` | — | CE Phase 4 |
| `provenance.rs` | — | CE Phase 5 (isnad-inspired) |
| `review.rs` | — | CE Phase 7 |
| `trails.rs` | — | CE Phase 8 |
| `canvas.rs` | — | CE Phase 10/11 (Cynefin) |
| `lenses.rs` | — | CE Phase 9 (Multi-Lens) — Rule 8 hybrid violation |
| `bases.rs` | — | .base file CRUD — Rule 8 read-time violation |
| `dataview.rs` | — | DQL queries — Rule 8 read-time violation |
| `importers.rs` | — | 7 source formats (Obsidian / Bear / Notion / Evernote / Markdown / HTML / Constellation backup) |
| `embeddings.rs` | — | ONNX e5-small (384-dim, 100 langs) |
| `watcher.rs` | — | Must be `async` (else Boot Criterion 2 dies) |
| `file_kinds.rs` | — | 3-layer kind classification |
| `fts5_tokenizer.rs` | 479 | Custom 'constellation' tokenizer (stemming + bigrams) |
| `perf_trace.rs` | 71 | TRACE_LOG mutex; record/get/clear |
| `strata.rs` | — | CE Phase 2 (8-level hierarchy) |
| `maturity.rs` | — | CE Phase 3 (5 states) |
| `map.rs` | — | Constellation Map (D3 sunburst data) — Rule 8 read-time |
| `arabic/mod.rs` + 14 files | — | 5-layer morphological engine |
| `lexicon/` | 6 files | Polylingual lemma graph |
| `ai/mod.rs` | 406 | 4-provider AI abstraction |

---

## 4. The Cognitive Engine (CE)

`docs/CE-spec.md` + `docs/cognitive-engine-roadmap.md` are the canonical specs. Two-layer architecture.

### 4.1 Seven epistemological foundations (`CE-spec.md:22-29`)

1. Knowledge is not information — value is in connections, not storage.
2. Knowledge has a vertical dimension — 8-level hierarchy (Datum → Worldview).
3. Knowledge has a certainty dimension — `ilm al-yaqin → haqq al-yaqin`.
4. Knowledge is organized by immutable principles — non-contradiction, causality, hierarchy.
5. Knowledge has diverse sources — sensory, rational, transmitted, experimental, intuitive.
6. Knowledge exists on a spectrum — received vs discovered.
7. The essence of knowledge is understanding-generative apprehension.

### 4.2 Layer 1 — Structural Cognition (zero AI). All shipped.

| # | Name | File | Rule 8 |
|---|---|---|---|
| 1 | Typed Links | `libraries.rs` + `search.rs` (note_links + triggers) | ✅ Write-time |
| 2 | Knowledge Strata (8-level) | [`strata.rs`](src-tauri/src/strata.rs) | ✅ Write-time (sky_nodes triggers, [`search.rs:1137`](src-tauri/src/search.rs:1137)) |
| 3 | Maturity Lifecycle | [`maturity.rs`](src-tauri/src/maturity.rs) | ✅ Write-time (sky_nodes triggers, [`search.rs:1215`](src-tauri/src/search.rs:1215)) |
| 4 | Tension Detector | `tension.rs` | ⚠️ Partial — contradictions cached, structural gaps on read |
| 5 | Provenance Chain (isnad-inspired) | `provenance.rs` | ⚠️ Partial — frontmatter sources cached, traversals on read |
| 6 | Externalization | within `strata.rs` (word_count signal) | ✅ Write-time |
| 7 | Review Pulse | `review.rs` | Hybrid — `.constellation/review-pulse.json` |
| 8 | Trails | `trails.rs` | ✅ Write-time |
| 9 | **Multi-Lens Views** | `lenses.rs` | ❌ **Hybrid violation** — definitions write-time (`lenses.json`), results recomputed on read (`apply_lens` walks the tree) |
| 10/11 | Expression Forge / Sense-Making Canvas | `canvas.rs` | ✅ Write-time (JSON persisted) |
| 12 | 360° Inspector ✅ enabled v1.8 §93, hardened v1.9 §96–§104, **redesigned v1.15 §112 (Stratification Matrix)** | `inspector360.rs` (517 lines) | ⚠️ **Read-time aggregation, but actual perf is fine** — the per-fetch cost was theorised as 1–3 s but Boss's lived experience is "almost instantly". MIG-010 (cache `note_360_view` write-time) priority dropped to LOW. Frontend mitigations still in place: debounce 200 ms, sequence guard, last-fetched-key dedup, lazy mount, dedupe-by-path in the matrix. |

**Inspector 360° aggregator** ([`inspector360.rs:1`](src-tauri/src/inspector360.rs:1)): aggregates `Note360View` from typed/untyped links (7 types) + active-note stratum + maturity + contradictions + orphan/SPOF flags + provenance + stage + review + trail membership + lens groups + missing-link-types gap analysis. **Post-§112**: every `LinkedNote` (outbound, inbound, second-order) also carries `stratum: u8`, populated by `precompute_all_strata()` — a single pass that builds an inbound-count + sources-of map for the library, then runs the existing `compute_stratum_for_note` rule set against each note. O(N + total_links). Same big-O as before; constants higher but sub-second on the 7,600-note Universe per Boss's lived experience.

**Frontend Inspector 360 surface** (post-§112 — **Stratification Matrix**):

- Two display modes via the `compact` prop. Compact = right-sidebar tab (scorecard glance widget). Full-window = ribbon-dock button (deliberate-study matrix, replaces editor area).
- **Full-window = the matrix.** HTML/CSS Grid (no SVG polar coordinates). 8 rows (stratum L8 → L1, top-down) × 8 columns (`supports`, `contradicts`, `causes`, `derives-from`, `generalizes`, `exemplifies`, `part-of`, `untyped`) + a 200 px row-label column on the left + a 64 px row-totals column on the right. Each `(stratum, type)` cell holds the connected notes whose stratum matches the row, drawn as 11 px coloured dots (max 16 per cell, then `+N` overflow chip). Active note's row is highlighted (purple background gradient + bold `L{n}` chip showing the note's truncated name). Empty cells render diagonal stripes — gaps as first-class signal.
- **Compact = a scorecard.** Stratum pill (`L4 Concept`), maturity pill, ↑outbound/↓inbound/word-count line, per-type bar chart (label + filled track + count, with explicit `—` for blind spots and 50 % opacity to mark zero rows), and a flags row (orphan, fragile, gap count, due for review). No matrix — 280 px is too narrow.
- **Multi-hop back stack** shared between compact and full-window. State: `inspector360BackStack: $state<Array<{path, name}>>` in `+layout.svelte`. Forward node-click pushes current; back click pops one entry; bar shows `← {previous}` until empty. Universe switch resets to `[]`.
- **Hover-only labels** (preserved from §107). Hovering a dot reveals the connected note's name in a fixed top-right tooltip on the matrix canvas — does not follow the mouse, doesn't pop chrome on dense rows. The dot itself enlarges (`scale(1.6)`) and gains a colored glow (`box-shadow: 0 0 10px var(--dot-color)`) on hover.
- **Per-cell dedup** on path so the same note returned from outbound + inbound + second-order sources renders once per `(type, stratum)` cell.
- **Dimension strip** below the header surfaces the non-spatial dimensions: Stratum (with name), Maturity (color dot), Origin + trust depth (color dot), Stage (icon + name), Review (date or "Due"), Trails / Lenses (count) — only shown if non-empty.
- **Bottom HUD** keeps the existing `total_outbound` / `total_inbound` / `word_count` summary plus warning chips for orphan / fragile / blind-spots / tensions.
- **Dropped permanently**: `vizMode` dropdown (Atmospheric / Neural / Cosmic), `SECTOR_MAP`, `polarToXY`, `ringsLayout`, `layoutMode`, `allNodes`, `SECTOR_THRESHOLD`. Polar geometry is gone from the file; the design space the matrix occupies is grid + axis semantics.

### 4.3 Layer 2 — AI Discovery (5 phases, 🔲 all not started)

12. Hidden Pattern Discovery (ghost links via semantic engine).
13. Blind Spot Detection.
14. Cross-Domain Insight Generation.
15. Socratic Challenger.
16. Worldview Synthesis.

Local-LLM-first; cloud opt-in only. Existing infrastructure: `ai_send_message` Tauri command across 4 providers (OpenAI / Anthropic / Google Gemini / Ollama — [`ai/mod.rs:1-406`](src-tauri/src/ai/mod.rs:1)); embeddings via ONNX multilingual-e5-small (384-dim, 100 languages — `embeddings.rs`).

### 4.4 The Living Link Architecture (P0–P5 all shipped + user-validated)

`docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md` is the philosophy doc.

**8 link properties**: Type · Direction · Annotation · Weight · Confidence · Created · Last Traversed · Traversal Count.

**8 typed link types** (the cognitive vocabulary; default = untyped `associative`), read from ONE **Link-Type Registry** (`link_types.rs` + `linkTypeRegistry.ts`, MIG-067) by every surface — canonical order:
`supports` (blue #4A9EFF) · `contradicts` (red #FF4A4A) · `causes` (orange #FF8C42) · `exemplifies` (green #4AFF88) · `generalizes` (purple #A44AFF) · `derives-from` (gold #FFD700) · `part-of` (gray #AAAAAA) · `supersedes` (slate #5B7A8A). Users can add **custom types** (top-level or nested under one of the 8) — §G.

**Syntax** (MIG-067): predicate-FIRST `[[type::target]]` / `[[type::target|display]]` — the type leads. The parser + every reader also accept the legacy `[[target|type]]` / `[[target|alias|type]]` (backward compat). Authoring is **type-first**: `[[` → pick a type → `[[type::` → pick/type the target.

**4 confidence levels**: `hypothesis` → `evidence` → `established` → `contested`. Auto-promote at traversal_count ≥3 → evidence, ≥10 → established. Manual override via right-click.

**Decay formula** (display-only — `weight` raw column never modified):
```
effectiveWeight = rawWeight × exp(−ln(2) × daysSinceTraversal / halfLifeDays)
```
Default half-life: 60 days.

**Storage**: dual-layer design (LINK files on disk + SQLite). **The on-disk LINK files layer was deliberately deferred** — implementation lives only in `note_links` SQLite table.

**Archive = soft-delete.** Reversible via Link Dashboard's Archived tab.

**Lifecycle commands** ([`search.rs:2330-2938`](src-tauri/src/search.rs:2330)): `_link_stats`, `_link_traverse` (updates weight via `1.0 + ln(1 + traversal_count)`), `_link_dormant`, `_link_decay`, `_link_set_confidence`, `_link_backfill_confidence`, `_link_archive` / `_unarchive` / `_archived`.

### 4.5 CECE — The Cataloger (MIG-039 / MIG-040, shipped 2026-05-20)

**User-facing name**: "The Cataloger" (en) / **المُصنِّف** (ar). Internal engine: **CECE** (Constellation Epistemic Content Engine). Feature flag: `enabledFeatures.cece` (default ON).

**Purpose**: classify each note on two independent axes — *Source* (where the knowledge came from: perception, testimony, reason, revelation, experiment, tradition, intuition, consensus; 11 categories in the horizontal taxonomy) × *Content Type* (the epistemic form of the note: argument, case study, data, definition, hypothesis, narrative, etc.; ~40 nodes in the vertical taxonomy). Together they reveal the epistemic texture of the universe.

**Shipped architecture** (5 catalogers, manual scan only):

| Cataloger | File | Axis | Mechanism |
|---|---|---|---|
| User Authority | `cece/user_authority.rs` | Both | frontmatter `sources:` / `content_type:` override — first and final word |
| Structural | `cece/structural.rs` | Both | title/filename/heading patterns (date → log; hypothesis → hypothesis card; etc.) |
| Linguistic | `cece/linguistic.rs` | Both | Arabic root matches (CAE), surface keywords, Bridge cross-lingual similarity |
| Graph | `cece/graph.rs` | Both | typed-link neighbor consensus |
| Semantic | `cece/semantic.rs` | Both | e5-small kNN over classified neighbors |
| Reasoning (designed, NOT wired) | `cece/reasoning.rs` | Both | local-LLM GBNF-constrained taxonomy navigation — **scaffold only** |

**Orchestrator** (`cece/synthesis.rs`): `SynthesisResult` with per-axis `AxisDecision` (regime: `unanimous` / `strong_majority` / `split`; `primary`, `secondary`, `needs_user_disambiguation_between`). A `Split` regime on an axis means catalogers disagreed and the user must pick; the card shows disambiguation chips for that axis.

**IPC surface** (registered in `lib.rs`):
- `classifier_suggest_for_note` — on-demand single-note classification (creates/upserts a `sources_suggestions` row)
- `classifier_scan_start / cancel / status` — universe-wide background scan (MIG-021v2 §1F')
- `cece_resolve_disambiguation` — user picks from Split candidates; returns updated record (both axes) or null (both settled) — **MIG-040 fix** (was void; now keeps card alive when other axis is still Split)
- `cece_record_correction_for_card` — updates per-cataloger reliability after Accept/Edit
- `cece_get_reliability_for_active_library` — calibration view data
- `cece_get_note_history / cece_query_history` — note state history

**Frontend surfaces**:
- **The Cataloger** (left-dock, `src/lib/components/CatalogerView.svelte`): full-page view. Header with "Scan library" button + **"Classify a note…" note-picker** (inline search popover using `constellation_search`, dispatches `constellation:classify-and-show` to both SRP instances) + close button. `ClassifierScanProgressStrip`. Library-wide `SourceReviewPanel` (no `activeNotePath`, `visible={showCataloger}` for reload-on-reopen). Lazy-mounted per LL-022 (`catalogerEverOpened`).
- **Right-sidebar Source Review tab** (still present; `SourceReviewPanel` with `activeNotePath`). "Classify open note" button now dispatches `constellation:classify-and-show` after classifying so the Cataloger SRP also updates (MIG-039 follow-up fix).
- **SourceReviewPanel** (`src/lib/components/SourceReviewPanel.svelte`): shared between both surfaces. Per-card: note title + **NSC summary** (see §4.6) + disambiguation chips + Accept/Reject/Edit + reasoning trail (expandable/collapsible per-card, respects `reasoningTrailVisibility` setting). Filter chips (All / Both axes need call / Source needs call / Content type needs call / Catalogers agreed). Render cap: 80 cards (`RENDER_BATCH`), Show More button (+80). `parseComposite` memoized (component-local Map). Trust-calibration banner (first 50 reviews).

**Settings**: Settings → Intelligence → CECE: `reasoningTrailVisibility` ('always' / 'on_disagreement' / 'never').  Settings → Plug-Ins → Discovery: toggle `enabledFeatures.cece` ON/OFF.

**Known open items**: PJ-041 (reasoning prose EN-only), PJ-042 (confidence enum i18n), PJ-043 (taxonomy labels en+ar only), PJ-044 (right-click classify from NotePane).

### 4.6 NSC — Note Summary Creator (MIG-040, shipped 2026-05-20)

**Purpose**: each Cataloger / Source Review card shows a **note summary** under the title (above the reasoning trail) so the user can decide whether to review the note without opening it.

**Algorithm** (fully offline) — NSC **only generates when the author has not written a summary** (precedence order):
1. **Frontmatter** — `summary:` / `description:` / `abstract:` / `excerpt:` → use first match, verbatim (source = `frontmatter`).
2. **Body summary callout** — a `> [!summary]` / `[!abstract]` / `[!tldr]` callout in the body (the 📋-icon family from `calloutPlugin.ts`). Returns the callout's body lines verbatim (source = `callout`). **Read from the RAW note file**, not `note_meta.body_text`: `body_text` is markdown-stripped AND Arabic-normalized (tashkeel/tatweel removed, ة→ه etc.), so it would corrupt an author's exact wording. The file read happens only on a cache miss for a note with no frontmatter summary (notes with a frontmatter summary never touch the file). `body_callout_summary()` mirrors the `calloutPlugin.ts` block syntax.
3. **Extractive (generated)** — segment the body into sentences (Unicode UAX#29 + paragraph/opening-sentence fallback), truncate to `MAX_BODY_CHARS = 50_000` and `MAX_RANK_SENTENCES = 40` to prevent ONNX crashes on large notes, embed all sentences via e5-small (384-dim, multilingual, L2-normalized), build a sentence-similarity graph (cosine edges), run weighted PageRank (d=0.85), return the top-3 highest-scoring sentences in document order (source = `extractive`; pure-opening fallback = `opening`).

> **Why callout precedence matters**: most imported Wikipedia-style Arabic notes carry their lede as a `> [!abstract] ملخّص` callout, not a frontmatter field. Before MIG-040's callout step, NSC ignored these and generated a TextRank summary that overrode the author's own — the bug Eisa caught 2026-05-20 on `الهرم الأكبر`.

**Cache**: `note_summaries` table (path PK, summary, source, content_hash, updated_at) in the search DB. Created by `ensure_note_summaries_table` called from `init_db`. The cached `content_hash` is prefixed with `NSC_ALGO_VERSION` (currently `v2`) so an algorithm change (e.g. adding the callout step) invalidates every cached summary — they recompute lazily on next view, no wipe needed.

**IPCs**: `nsc_get_summaries_for_notes` (batched, cache-first, async) · `nsc_get_summary` (single-note get-or-compute, async).

**Frontend delivery**: `scheduleSummaryFill` → `fillNextSummaryChunk` (6 notes/batch, 500 ms debounce, paused while classifier scan runs). `summaryRequested` is a plain (non-reactive) Set so the `$effect` only tracks `visibleQueue`/`summaryScanRunning` without a Rule-2 violation.

**Backfill (MANUAL, v2.21)**: `nsc::backfill` pre-computes summaries for all notes lacking a current (`NSC_ALGO_VERSION`) one. Background thread, resumable (re-enumerates pending each run), gentle (engine lock released per note + 30 ms inter-note sleep, pauses while a classifier scan runs), cancellable. Commands `nsc_backfill_start`/`_status`/`_cancel`; `nsc:backfill` progress events → `NscBackfillProgressStrip` in the footer. **Triggered ONLY by the "Build all summaries" button in the Cataloger header — never on boot** (the earlier auto-after-paint trigger regressed startup ~4 s → ~28 s; removed v2.21).

---

## 5. The Arabic Engine + Lexical Bridge

A native 5-layer morphological engine. Built from scratch, license-clean. **Not a port.**

### 5.1 Engine architecture (verbatim from [`arabic/mod.rs:16-37`](src-tauri/src/arabic/mod.rs:16))

```
[L1 normalizer]        — tashkeel / tatweel removal, hamza variants,
                          language detection; preserves surface form
   ↓
[L2 protected list]    — ~20K proper nouns + loanwords (hash lookup)
   ↓
[L3 generative FST]    — rolling-hash + FST over all (root × pattern)
                          combinations
   ↓
[L4 disambiguator]     — ranks multiple analyses by corpus frequency
   ↓
[L5 user overrides]    — per-Universe learning layer
```

**5 logical layers, 15 physical Rust files** in `src-tauri/src/arabic/`:

- `normalizer.rs` (484 lines) — L1: tashkeel/tatweel strip, aggressive folding (alif/ya/ta-marbuta), script detection (Arabic/PersianFamily/Hebrew/Latin/Other). Core test: `وائل` survives stripping (Light10 bug fix).
- `protected.rs` (551 lines) — L2: TSV-backed `HashMap<stripped, ProtectedEntry>` (~1196 entries). Categories: ProperNoun / Place / Loanword / Function. First-write-wins on dupes. M1e flagship: `وائل`, `محمد`, `إنترنت` return verbatim with confidence=1.0.
- `fst_index.rs` (598 lines) — L3: `GenerativeFst` wraps **two `fst::Map<FstBytes>`** (stripped + folded). Packing: FST value = `(offset u32 << 32 | count u32)`. ~300K distinct keys, ~1.1M forms at 7K-root scale, single-digit MB via prefix sharing.
- `fst_bake.rs` (991 lines) — M3-baker on-disk cache. **mmap wired through line 323**: `Mmap::map(&file)?` → `Arc<Mmap>` shared by both stripped + folded FSTs (single syscall + VMA). Cache filename: `arabic-fst-v{djb2(SEED_TSV) XOR CACHE_FORMAT_VERSION:016x}.bin`. Mobile fallback: heap `Vec<u8>`.
- `generator.rs` — Template substitution `(Root, Pattern) → surface`. Placeholders ف/ع/ل. Phonology passes: gemination fusion, hamza carrier picking, weak-radical rewrites (M2.c).
- `patterns.rs` — ~158 morphological patterns (verbal 50, verbal nouns 20, participles 22, broken plurals 27, etc.). All patterns carry full tashkeel.
- `roots.rs` — Root inventory (595 seed → 7K corpus). Classification: Hamzated / Geminated / Assimilated / Hollow / Defective / Sound (triliterals); Sound / Weak (quadriliterals).
- `affixes.rs` — Affix-peeling cascade (e.g., ال + كاتب).
- `disambiguate.rs` — L4 deterministic ranking (confidence → origin priority → POS → fewer affixes → alphabetic).
- `overrides.rs` — L5 per-Universe JSON store at `<universe>/.constellation/arabic-overrides.json`. Tauri commands: `read_arabic_overrides`, `add_arabic_override`, `remove_arabic_override`, `reindex_arabic_overrides`.
- `types.rs` — `Analysis`, `Root`, `Pattern`, `PartOfSpeech`, etc.
- `regression.rs`, `bench.rs`, `rss.rs` — test/bench harness (cfg-gated).

**Entry points** ([`arabic/mod.rs:129-564`](src-tauri/src/arabic/mod.rs:129)): `analyze`, `analyze_with_overrides`, `analyze_best`, `analyze_with_overrides_best`.

### 5.2 M-numbered milestones (NOT module boundaries)

The "M3-M14" series in session logs are **project milestones**. Engine is 5 layers (above). All M-milestones shipped:

- M3 FST-backed generative index + M3-baker cache.
- M5 502-case regression corpus, 100% pass.
- M6 FTS5 routes Arabic stemming through `analyze_best`. Closes flagship `وائل → "ائل"` mangle.
- M7 deterministic disambiguator.
- M8 + M8b + M8c — L5 user overrides + ACTIVE_STORE registry + Settings UI.
- M9 bench — ~130k words/sec, ~7.6 MiB cache.
- M10 Lexical Bridge architecture (15-concept seed).
- M11-infra Lexical Bridge baker.
- M11-data v1 (49-concept seed).
- **M11-data v2 Producer ✅ complete** — **20,000 concepts** across **499 thematic shards** in `lab/m11-data/concepts/` (verified by `wc -l lexicon_v1.tsv` = 20,015 lines incl. header).
- M12 query expansion plumbing (`escape_fts_term`, `build_match_expr`, `expand_to_match_expr`).
- M12-detect language detection (15-language classifier).
- M12-bench (mean 5.2 µs, p99 15.8 µs — 60–600× under 1 ms budget).
- M13 multilingual result badge (`match_via`).
- M14 lexical_search end-to-end bench gate.

### 5.3 Lexical Bridge (`src-tauri/src/lexicon/`, 6 modules)

**Polylingual lemma graph**, not a morphological tool: every lemma in any of the 15 languages can be looked up and yields its equivalents in any other.

- **graph.rs** — Node identity: `(lang, lemma, sense_id)`. Edge types: Equivalent / Synonym / Hypernym / Hyponym / UserLink. Storage: FST `{lang_code}:{normalized_lemma} → (first_node_idx u32 << 32 | sense_count u32)`. Core tier: ~20K concepts × 10 langs ≈ 200K nodes, ~800K edges.
- **expansion.rs** — Query expansion. `SynonymLevel`: None / Synonym / SynonymAndHypernyms (±1 hop). Pipeline: lemmatize → fetch equivalents → add synonyms/hypernyms → build FTS5 MATCH across selected languages. Cap 8 per language by default.
- **bake.rs** — TSV ingestion + binary cache (content-addressed, version-hash gated).
- **detect.rs** — Language detection (15-language Unicode classifier).
- **fts.rs** — FTS5 integration (escape, match expression assembly).
- **parse.rs** — TSV format parsing.

Source: `src-tauri/src/lexicon/data/lexicon_v1.tsv`. Built deterministically by [`lab/m11-data/build.py`](lab/m11-data/build.py) (Python 3) from 499 JSON shards.

**Coverage policy**: `en` + `ar` required per concept; target ≥8 of 15 languages. **No third-party sources** — all content original (WordNet / Wiktionary explicitly rejected per project policy in `lab/m11-data/README.md`).

### 5.4 Custom FTS5 tokenizer ('constellation')

[`src-tauri/src/fts5_tokenizer.rs`](src-tauri/src/fts5_tokenizer.rs) (479 lines). Wraps the Rust stemming pipeline: Arabic Light10 + Hebrew prefix stripping + Persian / Cyrillic / Devanagari / German / Spanish / Portuguese / French / Turkish / English stemmers + bigrams. Symmetric across `FTS5_TOKENIZE_DOCUMENT` (write) and `FTS5_TOKENIZE_QUERY` (read).

**Token emission**:
1. Primary token: stemmed form.
2. Bigram (colocated): `prev_stem \x1f cur_stem` (separator `0x1f` unmatchable in user text).
3. Stopwords/length-filtered: emit nothing, break bigram chain.
4. Bigrams form **only between tokens in the same script** (prevents Arabic↔English bigram noise).

All Arabic-side morphology delegates to `crate::libraries::process_word_for_fts` → `analyze_best()`.

### 5.5 Other Rust modules (read this round)

- **inspector360.rs** (517, post-§112) — see §4.2 row 12.
- **lens.rs** (419) — Brandes' betweenness O(VE), weighted by link_type (supports=1.0, causes=0.9, contradicts=0.8). **At >500 nodes**: approximate sampling (200 sources). Tag-shared edges command: weight 0.6 × shared_tag_count, top 500.
- **boot_bundle.rs** (138) — `BootBundle`: libraries + settings + bookmarks + workspaces + property_types + workspace_bases + child_universes + child_universe_lib_paths + per-step `timings_ms`. Replaces ~10 serialized IPCs.
- **sky_backfill.rs** (470) — MIG-001 §5 resumable populator. `sky_backfill_cursor` table stores `last_path`. `BATCH_SIZE=1000`, `INTER_BATCH_SLEEP_MS=50`. Per-batch phases: A (insert sky_nodes/links under lock) → B (read note files, compute word_count + created_at + aliases, no lock) → C (UPDATE note_meta) → D (UPDATE sky_nodes stratum/maturity). Idempotent via `INSERT OR IGNORE`. Final stamp: `schema_versions.sky = SKY_SCHEMA_VERSION`.
- **tasks.rs** (495) — `[- | * | +] [ ] | [x] | [X] text` pattern. Extracts: due_date (📅 YYYY-MM-DD or `[due:: …]`), priority (⏫🔼🔽), tags (#tag), created_date (➕), done_date (✅). Commands: `scan_library_tasks`, `scan_note_tasks`, `toggle_task`, `scan_library_note_dates`.
- **embeds.rs** (708) — Living embed resolver. 7-tier search order: relative-to-note → absolute-in-vault → explicit-attachment-folder (`.obsidian/app.json`) → fallback (attachments/ images/ assets/) → vault-wide index → vault root. `EmbedKind`: image / audio / video / pdf / canvas / excalidraw / note / generic / missing. URLs: data: if ≤4 MB, else `asset://localhost/{encoded_path}`. Digit normalization: Arabic-Indic (٠–٩) + Extended (۰–۹) → ASCII.
- **embeddings.rs** — ONNX runtime + multilingual-e5-small (384-dim, 100 langs), 100% offline. `constellation_init_embeddings`, `_embed_text`, `_embed_notes`, `_embedding_status`. Vectors persisted to SQLite. **MIG-013 §1A**: added `pub fn embed_passages_standalone(model_path, tokenizer_path, texts, intra_threads, batch_size)` for the offline `build_concept_vectors` `[[bin]]` (builds its own ONNX session without an `AppHandle`). **MIG-013 §1C (pending)** removes the per-library term-embedding loop (`init_term_embeddings`, `populate_term_vocab`, `term_embeddings` table) — superseded by the M11 Bridge Adapter below.
- **bridge_vectors/** (MIG-013 §1B) — CTSE Bridge Vector Store. `asset.rs` parses the baked `concept_vectors_v1.bin` (30 MB, 20K × 384 f32, magic `CTSEBV01`, L2-normalized) via `include_bytes!` into an owned `Box<[f32]>`. `store.rs` does cosine k-NN over the flat row-major matrix (`nearest_concept`, `nearest_concepts_k`). `mod.rs::get()` is the `OnceLock` singleton. **Constant-time semantic search regardless of library size** — the asset is fixed at compile time.
- **ctse/** (MIG-013 §1B) — Constellation Terms Scanning Engine, Bridge Adapter. `resolve_term_pure(graph, store, embed_query, term, lang, threshold)` — pure DI core for tests. `resolve_term_to_concept(app, term, lang)` — Tauri-context wrapper. **Fast path**: `LexiconGraph::find_nodes` + `graph.nodes[idx].concept_id` (microseconds, no ONNX, ~80% hit rate on M11-covered terms). **Slow path**: e5 query embed + cosine k-NN, gated by `DEFAULT_THRESHOLD = 0.78`. M11 zero-touch invariant: every CTSE commit verifies `git diff src-tauri/src/lexicon/` returns empty. §1C wires `resolve_term_to_concept` into `reindex_single_note`; §1D wires it into the search query path.
- **build_assets/build_concept_vectors.rs** (MIG-013 §1A) — offline `[[bin]]` target run once per release: `cd src-tauri && cargo run --bin build_concept_vectors --release`. Reads M11 TSV (read-only via `lexicon::parse`), picks one canonical surface form per concept (`en > zh > es > fr > de > ja > ru > pt > ar > ko > hi > tr > fa > he > ur` priority), embeds with multilingual-e5-small in batches of 128, validates per-vector L2-norms, writes `bridge_vectors/data/concept_vectors_v1.bin`. Boss-approved policy: the `.bin` is committed to the repo (changes only when `lexicon_v1.tsv` does).
- **importers.rs** — 7 formats async. `import_pick_source`, `_preview`, `_execute`, `_with_canonical`.
- **watcher.rs** — `notify` crate. **MUST be `#[tauri::command(async)]`** (recursive watch is blocking I/O; sync command runs on WebView2 UI thread → Boot Criterion 2 fails). Inline note at lines 19-38 explains the constraint.
- **dataview.rs** — DQL TABLE / LIST / TASK / CALENDAR + FROM + WHERE + SORT + LIMIT. Reuses bases.rs scan primitives. Read-time recompute on every `execute_dataview_query`.
- **bases.rs** — `.base` YAML CRUD + `update_note_property` (§H edit-in-place) + `convert_base` (old-JSON→new-YAML). **`query_base` functionally retired (MIG-065 §I)** — orphaned `BaseView*` UI removed, commands unregistered; the `fn` body is dead-present (physical sweep deferred — `dataview.rs` shares `scan_folder`/`parse_frontmatter`/etc.). The live Base read is now **`execute_lens`** (SQL, Rule-8-clean).
- **perf_trace.rs** (71) — `static TRACE_LOG: Mutex<Vec<(String, u64)>>`. `record(cmd)` / `get_perf_trace_log` / `clear_perf_trace_log`.
- **file_kinds.rs** (454) — 3-layer kind classifier. Layer 1: extension map. Layer 2 (markdown): explicit frontmatter `kind:` / `type:`, then heuristics (LINK = from+to fields; TMPL = `<%…%>` / `{{…}}` ≥3 occurrences or `template: true`; MARK = `url:` + body <500 chars; CLIP = `source:` + blockquotes; BASE = `schema:` / `dataview` blocks; default = NOTE). Layer 3: unknown extension → `auto_generate(ext)` → persist in `kind_registry.json`. 4 unit tests.

---

## 6. Filename + Identity Architecture (post-MIG-003, 2026-04-28)

> **Architecture inverted by MIG-003 (commits §85–§89). The legacy "canonical filename = primary key" design is preserved as historical record in `docs/CANONICAL-FILENAME-ARCHITECTURE.md` § 0 banner; the rest of that doc describes the pre-MIG-003 design.**

### 6.1 Two ids, two purposes

| | What it is | Where it lives | Mutability |
|---|---|---|---|
| **`cid_cn`** | Immutable internal id, namespace-safe ("Constellation Node id") | Frontmatter `cid_cn:` field + `note_meta.cid_cn` column + every dependent-table `_cid_cn` column | **Never changes** for the life of the note |
| **Filename** | Human-readable representation of the title | The on-disk `.md` filename + `note_meta.path` column | Changes when the user renames the note |

`cid_cn` format is still the canonical pattern (`YYYYMMDDTHHMMSSZ_KIND_XXXX`), but it is no longer used as a filename — only as an internal correlation key.

### 6.2 Frontmatter contract

```yaml
---
title: Agriculture System
cid_cn: 20260410T153045Z_NOTE_7F3A
kind: note
created: 2026-04-10T15:30:45Z
aliases:
  - Old Title (preserved on rename)
---
```

`title` is user-mutable and equals the filename stem in the steady state. `aliases:` accumulates old titles automatically on rename (so wikilinks targeting the old name still resolve). `cid_cn:` is the load-bearing internal id and is never edited by the user.

### 6.3 12 file kinds — unchanged

`NOTE` · `BASE` · `TMPL` · `LINK` · `MARK` · `CLIP` · `IMG` · `AUD` · `VID` · `ATT` · `CANVAS` · `DRAW` ([`file_kinds.rs:25-45`](src-tauri/src/file_kinds.rs:25)). Auto-generated for unknown extensions (e.g. `.blend` → `BLEND`). The kind is recorded in `cid_cn` itself (the `_KIND_` segment) and in frontmatter; classification logic is unchanged.

### 6.4 `cid_cn` generator

[`canonical.rs:49-93`](src-tauri/src/canonical.rs:49) — timestamp source priority: frontmatter `created:` → filesystem creation → modification → `Utc::now()`; XXXX is 4-char uppercase hex; collision avoidance tries 10 hex suffixes, fallback +1 second. Output is the cid_cn string written to frontmatter at note creation.

### 6.5 Rename flow (post-MIG-003 §89)

`rename_item` ([`libraries.rs:rename_item`](src-tauri/src/libraries.rs)) — unified single path for `.md` files:
1. Read current frontmatter title (for alias preservation).
2. Update frontmatter title + append old title to `aliases:`.
3. `fs::rename` old_path → new_path.
4. Cascade DB: `UPDATE note_meta.path` (fires `note_meta_sky_au` → propagates to sky_nodes/sky_links) + explicit UPDATE on `note_links.source_path/.target_path`, `note_aliases.path`, `note_embeddings.path`.
5. Stamp 'rename' alias row keyed to the new path (durable safety net independent of frontmatter edits).
6. Reindex the note at new path.
7. Frontend cascades `[[OldTitle]]` → `[[NewTitle]]` body rewrite via existing `update_links_on_rename`.

The legacy "canonical-detection special case" that updated frontmatter without renaming the file is **removed**. Folder rename keeps the legacy fs::rename-only flow (folder DB cascade is its own concern, deferred).

### 6.6 New-note creation flow (post-MIG-003 §89)

`create_note` ([`libraries.rs:create_note`](src-tauri/src/libraries.rs)) — single unified path:
1. Sanitize the user-supplied title via `note_display_filename()` (strips reserved chars, falls back to "Untitled" if empty).
2. Resolve filename collision via `resolve_filename_collision()` — auto-suffixes "Untitled" → "Untitled 1.md" → "Untitled 2.md".
3. Generate fresh cid_cn via `canonical::generate_canonical()`.
4. Write frontmatter with `title`, `cid_cn`, `kind`, `created`.

The previous `native` / `compatible` mode branching is removed. Every library creates human-named files; cid_cn lives only in frontmatter.

### 6.7 Wikilink resolution — unchanged shape, alias-aware

Wikilinks target **titles**, never cid_cn. Resolution order: `title exact → aliases → original_filename → broken (red)`. The alias table (`note_aliases`) is populated from frontmatter `aliases:` lists by the indexer plus explicit 'rename' rows stamped by `rename_item`.

### 6.8 The MIG-003 commit trail

| § | What landed |
|---|---|
| §85 (Step 1) | `cid_cn` column on `note_meta` + UNIQUE index `idx_note_meta_cid_cn` + backfill from frontmatter (7,610 rows; 38 + 4 collisions auto-resolved). Schema-versions module `note_meta` stamped to 1. |
| §86 (Step 2) | `cid_cn` columns on `note_links` (source + target) / `sky_nodes` / `note_aliases` / `note_embeddings` + per-table backfill via JOIN on existing path columns. Schema-versions module `dependent_tables_mig003` stamped to 1. |
| §87 (Step 3) | All 7 INSERT writers stamp cid_cn at write time. `note_meta_sky_ai` trigger updated to copy cid_cn. Boot-time soft re-backfill (cheap, 0 rows in steady state). The `target_cid_cn` bulk re-backfill was caught + omitted (would have hung the app at boot — Working Agreement #4 violation). |
| §88 (Step 4) | New module `mig003_step4.rs`. Walked 17 libraries, found 19 canonical-named .md files (only the user's "inbox" Universe Notes folder used canonical mode; the 16 declared libraries already had human filenames). Per-library transaction; audit log to `.constellation/mig003-step4-renames.tsv`. Schema-versions module `mig003_step4` stamped to 1. |
| §89 (Step 5) | Unified `create_note` + `rename_item` flows. Canonical-detection special case removed (dead code post-Step-4). |

### 6.9 What was deliberately skipped

- **Step 6** (promote `cid_cn` to formal PRIMARY KEY of `note_meta`, drop redundant path columns from dependent tables) — the dual-keyed schema is not a defect; path columns are still load-bearing for fs operations; the rebuild risk was judged not worth the cleanliness gain.
- **§89 alias-append** (preserve old canonical stem in frontmatter aliases of the 19 renamed files) — those files are all dev/test notes from this week's work, no external references existed; saved as wanted-feature memory if future external integration ever needs it.
- **User Manual + 14 i18n translations update** — the user-visible behavior change is small (filenames are now intuitive); separate doc-only commit when convenient, not a blocker.

### 6.10 Legacy commands still in the tree

- `canonicalize_preview` / `canonicalize_execute` / `auto_canonicalize_all` / `inject_cid_library` / `de_canonicalize_library` / `repair_external_libraries_on_startup` — these were the original architecture's tooling. Post-MIG-003 they are mostly dead code. `inject_cid_library` is harmless (just stamps cid_cn into frontmatter); `de_canonicalize_library` is a no-op in the new world (filenames are already human). Deletion candidates for a future cleanup migration; not urgent.

---

## 7. Editor (NotePane / FocusPane)

**Two editors**:

- **[`FocusPane.svelte`](src/lib/components/FocusPane.svelte)** (213 lines) — quick capture, plain text. Imports **only** `bidiPlugin` + base CM6. No markdown parser, no syntax highlighting, no decorations. Comment at line 201 codifies: "Tab switches destroy/recreate FocusPane with new value prop" — no $effect for value sync.
- **[`NotePane.svelte`](src/lib/components/NotePane.svelte)** (388 lines) — full WYSIWYG-like CodeMirror 6. Live preview decorations, callouts, code blocks, images, wikilinks, tables.

### 7.1 The shared editor stack — full per-plugin

`src/lib/editor/` — 11 plugins per the **Editor Parity Rule**.

- **activeEditor.ts** (24) — Singleton `lastView` registry; queried by emoji/icon picker.
- **bidiPlugin.ts** (209) — Per-line script detection (Arabic, Hebrew, Devanagari, CJK split into Hiragana/Katakana → Japanese, Hangul → Korean, else Chinese, Cyrillic, Latin). Theme rule `unicodeBidi:isolate` on `[dir]` lines. Empty-line RTL inheritance from preceding non-empty line. Viewport-only scan; debounced 300 ms.
- **calloutPlugin.ts** (420) — **LL-014 freeze-proof architecture** (lines 5-23 doc):
  - **RULE A**: `Decoration.replace` only when cursor on **different line**. Provably safe — cursor on line N cannot be inside replace covering line M (M ≠ N).
  - **RULE B**: Collapsed body lines use zero-length `Decoration.line({class})` at `line.from === line.from`. CSS `display:none` on `.cm-callout-body-collapsed` does the hiding; Decoration.replace never spans the collapsed region. Cursor never gets "inside" a replace → no CM6 nudge loop.
  - Fold state: `StateField<Set<number>>`. Line numbers remapped via `tr.changes.mapPos()` on docChanged so fold persists across edits.
- **completions.ts** — Wikilink + **type-first** typed-link authoring (MIG-067 Option A: `[[` lists the registry types boosted + notes; pick a type → `[[type::` → target notes; legacy `[[note|type]]` menu kept), tag (Unicode `\p{L}` regex, RTL-aware), slash (14 commands incl. `/table 3x4`). `maxRenderedOptions` 8 (dropdown render perf).
- **iconSets.ts** (173) — 4 libraries: Lucide (~1500), Phosphor (~1500), Heroicons (~300), Feather (~290). Lazy-load via single shared promise; cached afterwards. `wrapForInsertion` namespaces icon ids.
- **lineDecoPlugin.ts** (131) — Blockquote + fenced-code line-level borders/background. Syntax tree resolved once at viewport start (replaces O(N) forward scan). Callout detection: upward scan max 50 lines.
- **livePreview.ts** (1271) — Core inline-render plugin.
  - **Pre-cached Decoration objects** at lines 138-181: `headingDecos[0..5]`, `boldDeco`, `italicDeco`, `strikeDeco`, `codeDeco`, `linkDeco`, `replaceDeco`, 8 typed-link decos, 2 checkbox states (CR Rule 1).
  - **ViewPlugin update guard** (LL-002, lines 1046-1098): `contextChanged` branch detects path/attachment-folder/traversal-map state effects; `selectionSet` guard rebuilds **only when cursor crosses line boundary** (CR Rule 1); `docChanged` fast path maps decorations + debounces full rebuild 300 ms.
  - Image/embed resolution: 7-tier search; cached (`_imageCache`, `_embedCache`); circular-transclusion guard (`_transcludeStack`).
  - Widgets: ImageWidget, UniversalEmbedWidget (image/audio/video/pdf/canvas/excalidraw/note-transclusion/generic/missing), IconShortcodeWidget, CheckboxWidget, InlineHtmlWidget, AlignmentWidget, CodeBlockLabelWidget, DataviewLabelWidget. All implement `eq()` for memoization.
  - Living Link traversal chip (P4.2, lines 967-988): keyed on `sourcePathLower|targetNameLower`; emits `×N` widget on high-count links.
- **markdownHighlight.ts** (49) — Lezer extension for `==highlight==`. Adds `Highlight` and `HighlightMark` syntax-tree nodes.
- **shortcodeAutocomplete.ts** (167) — Loads 23 emojibase locale datasets in parallel. Combined emoji + icon ranking; per-set boosts (lucide 0, feather −1, heroicons −2, phosphor −3). Lazy-load on first `:` keystroke.
- **tableFormulas.ts** (163) — `=SUM/AVG/COUNT/MIN/MAX(A1:A5)`. A1 syntax with column-letter → 0-based index. Numeric-aware, fallback to `localeCompare` (Arabic-aware).
- **tableUtils.ts** (363) — `parseTable`, `formatTable`, `generateTable`, `detectTabularText` (TSV-first then CSV, ≥50% row consistency required), add/delete/move row/col, `setAlignment`, `sortByColumn` (numeric-aware).

### 7.2 Key NotePane spec rules (top-principal)

- **§2.1 — The Editor Owns Its Content.** After mount, CM6 owns the document. One-way: Editor → onchange(text) → Parent stores → Debounced save. Never Parent → Editor.
- **§2.6 — No `$effect` for Editor State.** No `$effect` reads or writes `value` / `editBody`. Only allowed: dir change (guarded by `prevDir`), font change (guarded by `prevFontKey`). **Violating §2.6 caused BUG-015** (see §8.1).
- **PaperOnDesk (PoD) layout**: gray desk `#e8e8ec`, white paper `max-width: 1200px`, `padding: 48px`.
- **Auto-title format**: code generates canonical `YYYYMMDDTHHMMSSZ_NOTE_XXXX` filename + `title:` field.

### 7.3 Audit-agent count (clarification)

Three sets exist; the umbrella is "14 audit agents":

- **[`lab/audit-agents.md`](lab/audit-agents.md) — 7**: PA / AA / MA / SCA / RA / UXA / CQA.
- **NotePane spec — 8**: above + **EA** (Environment Auditor), added 2026-03-27.
- **[`docs/AUDIT-SYSTEM.md`](docs/AUDIT-SYSTEM.md) — 14**: 8 above + LA / SIA / SA / DIA / CFS / OGA.

Migrations use a different cohort: Phase 4 of `/migration` runs three parallel agents (Invariant Check / Drift Check / Migration Path).

### 7.4 `src/lib/libraries/store.ts` (write-ahead buffer, navigation)

**Stores**: `libraries`, `editingTabIds`, `openTabs`, `activeTabId`, `splitActive`, `focusedTabId`, `bookmarks`. Derived: `activeTab`, `universeNotesLibrary`, `selectedNote`, `focusedTab`, `libraryCount`, `totalStars`.

**Save discipline**:
- `saveLocks` map prevents concurrent writes per tab.
- `recentWrites` map (2 s TTL) gates the file watcher to ignore notes the app just wrote (prevents echo loops).
- **Write-Ahead Buffer**: in-memory + localStorage. `getWriteAhead()` checks memory first, falls back to localStorage (crash-safe). Cleared on tab close.
- `saveTabContent()`:
  - Auto-stamps "updated" / "حُدث" date if the property type === `date`.
  - Emits `screen:note-saved` for the second-screen window.
  - Async reindex via `constellation_search_reindex`.
  - Async semantic embed via `constellation_embed_notes`.
  - Tracks recent-edited in localStorage (20-deep) for second-screen dashboard.
  - **Does not dispatch to openTabs during autosave** — editor owns content, store re-syncs on tab switch.

**Navigation**: per-tab `_navTokens` prevent races on rapid Alt+Left/Alt+Right (newer click supersedes in-flight handler). 200-entry `_navTrace` ring-buffer exposed as `window.__navTrace`. Cross-library nav handled in `loadTabHistoryEntry`.

**Frontmatter parser**: multi-line YAML lists + inline `[a, b, c]`; type detection (list / link / checkbox / datetime / date / number / text); **Arabic property keys** recognized (`الوسم`, `وسوم`, `المجموعة`, ...); date normalization DD/MM/YYYY → YYYY-MM-DD.

### 7.5 `src/lib/secondScreen.ts` (12 events main→screen, 4 screen→main, 1 bidi)

**Window mgmt**: `openSecondScreen`, `openSecondScreenSmart` (auto-positions on secondary monitor at 80% size), `closeSecondScreen`, `isSecondScreenOpen`, `listMonitors`.

**Events**:
- **Main → Screen**: `screen:open-note`, `:universe-switched`, `:settings-changed`, `:context-changed` (editor/skyview), `:skyview-hover`, `:skyview-click`, `:sidebar-mode-changed`, `:split-mode-changed`, `:dashboard-open-note`, `:dashboard-tag-selected`, `:index-search`, plus workspace state restore.
- **Screen → Main**: `screen:open-in-main` (reverse-open), `:closed`, `:state-request` (workspace save), `:state-response` (restore).
- **Bidirectional**: `screen:note-saved` (both windows listen).

**Workspace State**: `ScreenState { mode: 'grid'|'star'|'detail'|'skyview'; linkedBrowsing; tabs; activeTabPath }`.

`src/lib/universe/store.ts` — 18 async invocation wrappers. **No local Svelte stores.** Pure IPC pass-through; Rust holds state.

---

## 8. Migrations (active state, 2026-05-18 — v2.16 full refresh)

`/migration` — four-phase workflow: **Architect → Plan → Build → Audit**.

| ID | Plan | Status |
|---|---|---|
| **MIG-001** Sky View Write-Time Derivation | `lab/reports/MIG-001-SKYVIEW-WTD.md` | ✅ Closed. |
| **MIG-002** Enrichment Persistence | `lab/reports/MIG-002-ENRICHMENT-PERSISTENCE.md` | ⏳ §1–§6 shipped + tested. §7–§10 pending; deprioritized indefinitely (no PJ-NNN tracks the remainder). |
| **MIG-003** Human-name Filenames | `lab/reports/MIG-003-HUMAN-NAME-FILENAMES.md` | ✅ Closed (2026-04-28). Steps 1–5 + 7–9 shipped; Step 6 (PK promotion) skipped by Boss decision. |
| **MIG-004** Alias-Aware Resolution | `lab/reports/MIG-004-ALIAS-AWARE-RESOLUTION.md` | ✅ Closed. 9/12 invariants verified. |
| **MIG-005** Alias-aware in-memory inbound | `lab/reports/MIG-005-ALIAS-AWARE-INMEMORY.md` | 🟡 **ABANDONED 2026-05-18** (Eisa decision during state-of-standing triage). Steps 1–3 stay shipped (§121/§122/§123 — `map.rs` / `strata.rs` / `maturity.rs`); Steps 4–8 abandoned after the fabrication-catch pause. |
| **MIG-006** Wikilink Rename Cascade | `lab/reports/MIG-006-WIKILINK-CASCADE.md` | ⏳ §1 ✅. §2 ✅ + 11 cascade tests. §3 expanded shipped at `3c4732d`, REVERTED at `5afe0c2` (BUG-015). §3 redo shipped per v1.28; §4–§11 still pending. |
| **MIG-007** Links Settings tab | (planned; no Architect yet) | ⏳ STILL-OPEN-VALID. Same as PJ-005. Top of queue. |
| **MIG-008** Create-Dialog standardization | `lab/reports/MIG-008-CREATE-DIALOG-*.md` | ✅ Closed. |
| **MIG-009** Lens → Sight rename | `lab/reports/MIG-009-LENS-TO-SIGHT-NAMING.md` | ✅ Closed. User-facing rename complete; internal Rust names (`lens.rs` / `lenses.rs` / `apply_lens`) keep old names per Boss memory. |
| **MIG-010** Index lexical bridge | (Architect+Plan+Audit on disk) | ✅ Closed. |
| **MIG-011** Index filter bridge | (Architect+Plan+Audit on disk) | ✅ Closed. |
| **MIG-012** Index search engine | (Architect+Plan+Audit on disk) | ✅ Closed (v1.34 preamble). |
| **MIG-013** CTSE (Constellation Terms Scanning Engine) | (Architect+Plan v1+v2+Audit on disk) | ✅ Closed (v1.40 preamble). PJ-016/017/018/019 cleanup bundle still open. |
| **MIG-014** Note-stage taxonomy (per-note dash-encoded) | `lab/reports/MIG-014-NOTE-STAGE-AUDIT.md` | ✅ Closed (2026-05-06). PJ-007 done. |
| **MIG-015** Chunked v2 sentinel migration + status-bar UI | `lab/reports/MIG-015-CHUNKED-V2-SENTINEL-AUDIT.md` | ✅ Closed (2026-05-06). PJ-001 done. |
| **MIG-016** Sight instant-toggle perf | `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md` | 🟡 Closed — Cancelled (partial-shipped) (2026-05-07). §1A + §1B shipped; §1C / §1D cancelled; §1E was deferred to PJ-038 which itself superseded. PJ-034 retired. |
| **MIG-017** Disable v2 Sight | `lab/reports/MIG-017-DISABLE-V2-SIGHT-*.md` | ✅ Closed (2026-05-07). PJ-039 done. `SIGHT_V2_ENABLED = false`. |
| **MIG-018** Sight v3 projection foundation | `lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-*.md` | 🟡 **SHIPPED-then-SUPERSEDED**. Shipped 2026-05-07 (`SIGHT_V3_ENABLED = true`); v3 retired by Concept Paper v3.1 → Sight v6 pivot. v3 codebase preserved on disk but unreachable (`SIGHT_V3_ENABLED = false`). |
| **MIG-019** Sight v3 density + calendar + search + universe-health | (mid-flight pivot) | 🟡 **SUPERSEDED**. Started as v3 work; pivoted at commit `29ce0101` "v3 → v4 clean-slate pivot"; v4 then superseded into v5 then v6. v2.13 §8 incorrectly carried "🟢 Next-up" status; corrected here. §2B (Milky Way density wash) shipped at `16063735` and incidentally closed PJ-035. |
| **MIG-020** Layer peeling + v2 retire | (never opened) | ❌ **ABANDONED**. v2 retire ostensibly happened under MIG-017; layer peeling (PJ-036) abandoned 2026-05-18; v3 itself retired before MIG-020 opened. Number orphaned. |
| **MIG-021** Epistemic Classifier (CECE) | (v1+v2+v3 iterations on disk) | ✅ Closed under MIG-021v3 (commit `407d79b5`, "MIG-021v3 V3-§11 close-out — MIG-021v3 ships"). Three redesign iterations preserved as historical record. |
| **MIG-022** Gap-analysis response (YAML metadata + i18n + history Rust foundation) | `lab/reports/MIG-022-{ARCHITECT,PLAN}.md` + §N audit quartet | ✅ **Closed 2026-05-18**. §0 + §D + §E + §A + §B.1–§B.4 all shipped + tested + audited. §B.5 + §B.6 contradicted-and-deferred-by-design (Sight v3 was the planned consumer, retired). §N P1 fix landed in MIG-024 §0 (UPSERT `1240984d`); F2-F7 polish backlog persists. Retroactive close-out section in `lab/reports/MIG-022-§N-FINAL-INTEGRATION-AUDIT.md` §8. |
| **MIG-023** Constellation Warrant Research workstream | (reserved; not started) | ⏳ STILL-OPEN-VALID. Reserved per Eisa commitment 2026-05-11; no Architect doc yet. Multi-month research project per scope estimate. |
| **MIG-024** Sight v5 Layer 1 visual foundation | `lab/reports/MIG-024-*.md` | 🟡 **SHIPPED-then-SUPERSEDED**. Steps §1–§6 shipped (commits `7caf56d9` → `e1836bb3`); v5 went live (`V5→true`); §N close-out partial (commit `a106580d`). v2.01 preamble: "obsoleted by MIG-025" — v5 superseded by v6 before §N formally closed. |
| **MIG-025** Sight v6 foundation (reallocated from "v5 Layer 2 diagnostic") | (50+ commits `MIG-025 §A.x → §C.4`) | ✅ Closed. v6.0 ship `8cdb73cd`; v6.1 ship `fc392b46`. Reallocation per v2.01 preamble — v5 pivoted to v6 mid-cascade; MIG-025 number kept but scope swapped to v6 foundation. Original v1.11 PJ reservation for "v5 Layer 2 diagnostic" never built. |
| **MIG-026** Sight tradition expansion (24 baseline + 9 shape renderers + user-definable + 15-locale full localization) | `lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-{ARCHITECT,PLAN}.md` | ✅ **Closed 2026-05-18**. Phases 0 / α / β / γ / δ / ε / ζ / η / θ / ι / κ / λ / μ all shipped across 2026-05-17 → 2026-05-18. Milestone tag `milestone/sight-v6.3-traditions-ship` at commit `99e4ed37`. PJ-052 (Concept Paper v4.1) + PJ-053 (λ-fix-6 native-quality polish) + PJ-055 (schema warning) + PJ-056 (drift cleanup) all closed same-day. PJ-054 (vitest runner) + PJ-057 (3 doc-drift) remain open. |
| **MIG-027** Sight theme inheritance | (focused single-subsystem MIG; opened mid-MIG-026 pivot) | ✅ Closed (2026-05-17). Initial + §-fix-1 + §-fix-2 (commits `686ee58` / `2f190dc` / `593af51`). Sight follows interface theme via chrome/semantic split + `--sight-highlight` CSS var family + theme-conditional override. |
| **MIG-028** Sight v5 retirement | (focused abandonment MIG; opened + closed same-turn 2026-05-19 per Eisa direction "whatever is related to Sight v5 shall be abandoned") | ✅ **Closed 2026-05-19**. Deleted `src/lib/sight/v5/` (7 files) + `src-tauri/src/sight_v5.rs`; removed `pub mod sight_v5;` + 4 IPC registrations + `SIGHT_V5_ENABLED` flag + v5 dock button + v5 modal mount; replaced 2 init_db v5 calls with idempotent `DROP TRIGGER/TABLE IF EXISTS` cleanup migration. MIG-024 Plan/Architect docs preserved as historical record. NSIS artifact: `Constellation_0.3.4_x64-setup.MIG028-sight-v5-retirement.exe`. |
| **MIG-029** Per-note frontmatter wiring for tradition-kind fields | (Architect doc pending; deferred to next session) | 🟡 **Architect-deferred 2026-05-19**. Tier 1 #1 of the Sight delivery cascade. Rust-side extraction of `pramana_kind` / `masadir_source` / `burhan_kind` / `mencian_sprout` / `mohist_zone` / `pardes_level` / `peirce_category` / `habermas_interest` / `songnihak_cell` into `LayoutCacheRow`; 8 tradition modules read the field via `XXXOf(row)` lookup; 15-locale User Manual chapter. Closes 8 TODO comments across the tradition modules. |
| **MIG-030** Sight v6 vitest runner + 2 tests (closes PJ-054) | (focused single-subsystem MIG; opened + closed 2026-05-19) | ✅ **Closed 2026-05-19**. Installed `vitest@4.1.6`; wired `npm run test:sight-v6` / `test:sight-v6:perf`; wrote `vitest.config.ts` (excludes worktree duplicates + the playwright-needing layout-fidelity test); added `tests/sight-v6/tradition-isolation.test.ts` (Plan §14.1, channel-isolation invariant) + `tests/sight-v6/tradition-perf.test.ts` (Plan §14.2, ≤16ms switch). **58/58 tests pass** across 3 files. |
| **MIG-031** λ-fix-6.b fa/he/ja/tr canvas deep audit | (focused single-subsystem MIG; opened + closed 2026-05-19) | ✅ **Closed 2026-05-19**. 4 parallel polish agents (one per locale). fa.json: 0 edits (already polished). he.json: 17 edits. ja.json: 12 edits (incl. 5 redundant-duplicate bug fixes in mencian-sprouts/wang-yangming). tr.json: 28 edits. Total: 57 keys polished. Now every borrowed Sunni-Islamic / Sanskrit / Hebrew / Greek-Latin technical term carries a target-language gloss matching the ar/zh/ko quality bar across all 15 locales. |
| **MIG-032** Tier 3 housekeeping (PJ-057 closures + Concept Paper §9 updates) | (focused docs MIG; opened + closed 2026-05-19) | ✅ **Closed 2026-05-19** (partial). PJ-057.a Mohist citation: no-op (v4.1 already cites manifest-canonical form). PJ-057.c prebuild footnote: added to Concept Paper §9.1 + §9.3 updated for MIG-028. PJ-057.b fresh 24-tradition SVG mocks: **deferred** (visual design work; overlaps with PJ-051; defer to focused session). |
| **MIG-033** Wasm/QuickJS sandbox for TS plugin layer | (Architect doc pending; deferred to next session) | 🟡 **Architect-deferred 2026-05-19**. Tier 2 #4. Replace Obsidian-trust H1 (current model: user consents to run arbitrary plugin code at app privilege) with sandboxed execution. Large MIG; security uplift. |
| **MIG-034** v4.1 per-tradition internal-structure polish | (reserved; not started) | 🟢 **Reserved**. Tier 2 #5. Per-quadrant radial-internal structure in pramāṇa; sub-sector annotations (naṣṣ / ijtihādī / qaṭʿī / ẓannī) in masādir; etc. Per-tradition aesthetic polish. |
| **MIG-035** Federation cUniverse tradition behavior | (reserved; not started; design call needed) | 🟢 **Reserved**. Tier 2 #6. Should federated cUniverse plugin pins surface in the chip dropdown? Design decision + implementation. |

### 8.1 The MIG-006 §3 / BUG-015 incident

- **§115** (`3c4732d`, 2026-04-25) shipped MIG-006 §3 expanded "open-editor coherence" — included a **value-prop → CM6 doc sync `$effect`** in NotePane that dispatched a doc-replace transaction on parent body-prop change.
- The `$effect` raced with `{#key tab.id+'|'+tab.path}` `onDestroy` on tab navigation. Click source → click target → reactivity propagated `tab.content` to target's body → OLD source NotePane's `value` prop changed → `$effect` replaced its own CM6 doc with target's body BEFORE `{#key}` ran destroy → destroy's `doFlush()` read the swapped doc → `handleFlush` wrote that swapped content to the OLD pane's `mountedFilePath`. Result: target file body overwritten with source body.
- **NotePane spec §2.6 explicitly forbade this pattern.** Spec wasn't read before commit.
- §116 (`5afe0c2`) reverted §115. §117 + §118 cleaned docs + recovered disk. BUG-014 closed as collateral.
- **Lesson**: per BASIC RULE + Working Agreement #4, every change touching write paths / lifecycle / reactivity / IPC contract MUST validate against the architecture before shipping. The MIG-006 §3 plan even documented a **fictional** "existing prop-change handler" that didn't exist — the plan misled itself.

---

## 9. Boot performance — 5 ship-gate criteria

`lab/boot-perf/BOOT-BUDGET.md`. Test corpus: **trial Universe (7,600 notes, 16 libraries, 656k typed links, 4k images on Windows 11 NTFS)**.

| # | Criterion | Status |
|---|---|---|
| 1 | UI visible ≤ 2.5 s | ✅ ~870 ms production (verified 2026-04-19) |
| 2 | Fully responsive (`hydrated_ms`) ≤ 6 s | ✅ closed at **811 ms** after Round 7 (LL-021) |
| 3 | Idle RSS ≤ 350 MB | 🔲 Not measured |
| 4 | Stat-sweep 50 externally-modified files ≤ 3 s, non-blocking | 🔲 Not implemented |
| 5 | Kill-mid-index recovery (no duplicate notes, no WAL corruption) | 🔲 Not implemented |

**Permanent diagnostic instrumentation** (kept after Criterion 2):
- **Five-stamp IPC diagnostic** (LL-021): `invoke_start_unix_ms` → `server_start_unix_ms` → per-phase `Instant::now()` → `server_return_unix_ms` → `client_recv_unix_ms`.
- **`perf_trace::TRACE_LOG`** at [`src-tauri/src/perf_trace.rs`](src-tauri/src/perf_trace.rs) — wraps `generate_handler!` to stamp every IPC dispatch arrival.
- **JS heartbeat** (max-gap from `boot:paint` to `boot:hydrated`).

### 9.1 What closed Criterion 2

`perf_trace` arrival tracer (Round 6) showed `constellation_map_universe` dispatched twice (~17.2 s gap), blocking `cache_boot_snapshot_core`. Round-7 fix: single attribute change `#[tauri::command]` → `#[tauri::command(async)]` on `constellation_map_universe`. `core_queue_ms` ~19.9 s → 4 ms; `hydrated_ms` 811 ms. **5,100× reduction.**

### 9.2 Other boot-perf primitives

- **Covering index** `idx_note_boot_snapshot ON note_meta(name, path, library_name)` — 100–1000× speedup (LL-020 corollary).
- **Paint-first UI** (LL-018): `appReady = true` synchronously; data hydrates after.
- **`LIBRARIES_CACHE`** (LL-016): in-memory cache for `load_all_libraries` invalidated by `save_libraries` + `set_active_universe`.
- **Always-mounted lazy-mount** (LL-022): `*EverOpened` flags for Map / OrgChart.
- **Watcher async** ([`watcher.rs:19-38`](src-tauri/src/watcher.rs:19) inline note): recursive watch is blocking I/O; sync command runs on WebView2 UI thread → Boot Criterion 2 fails.

### 9.3 Boot bundle — 10 IPCs into 1

[`boot_bundle.rs`](src-tauri/src/boot_bundle.rs) returns a single `BootBundle { libraries, settings, bookmarks, workspaces, property_types, workspace_bases, child_universes, child_universe_lib_paths, timings_ms[per step] }`. Replaces ~10 serialized invokes during `initializeApp`.

---

## 10. Standing rules (top-principal hierarchy)

### 10.1 BASIC RULE — Don't Make Things Up *(top of all rules)*

If I don't have a clue or information, I say **"I don't know."** No invented file paths, line numbers, function names, badge taxonomies, prior-art summaries, or any factual claim. **Fabrication is the worst class of error** — bugs are recoverable; trust isn't.

When tempted to add a "side note" — every claim in it must be sourced. If any claim isn't, the entire side note is cut.

Canonical violation prevented: 2026-04-26 tutorial fabricated T/C/P badge meanings as "Theory/Concept/Proposition." Actual: T = Title, C = Content, P = Property, with S = Semantic.

### 10.2 Working Agreement #1–#4

1. **Do the work yourself.** SQL, log greps, file inspection, build verification — Claude's job.
2. **One location: `E:\مشاريع كلاود\Constellation` on `main`.**
3. **The user is a non-technical IT Boss.** Plain language; tutorials per §10.4.
4. **Validate every change against the entire architecture before shipping.** Spawn parallel agents for any change touching write paths / lifecycle / reactivity / IPC. (BUG-015 is the canonical violation this rule prevents.)

### 10.3 Standing Orders

1. Update `lab/reports/SESSION-LOG-YYYY-MM-DD.md` after every phase / step / significant commit.
2. Update help files + User Manual + 14 translations on user-facing changes.
3. Session log = safety net for context loss.
4. `/simplify` (code review) after each phase.
5. **State-of-standing record before any pivot or major triage** — `§STATE-OF-STANDING` in the day's session log.
6. **Maintain `docs/Constellation Orientation & Onboarding vX.Y.md`** — filename always carries version suffix; rename in same commit on bump.

### 10.4 Tutorial Rule (top principal)

Every test instruction is a tutorial. Define the feature first (what / why / why it matters). Click-by-click walkthrough. Pre-state, action, post-state per step. Failure modes spelled out. Plain language only.

### 10.5 Plan Approval = Build Approval (top principal)

Once user approves a plan, Claude cascades through build steps autonomously. Stops only at: user-testable verification clauses, genuine architectural surprise, plan completion.

### 10.6 Migration Rule

Subsystem-crossing changes go through `/migration` four-phase workflow before any code is written. Single-file refactors → `/simplify`.

### 10.7 Performance Rules (8)

1. Every keystroke instant. Line-change guard for `selectionSet`. Pre-cache module-level Decorations.
2. No `$effect` loops. `$derived` for computed values.
3. No heavy work on the main thread. Vault indexing / search / file I/O → Rust. Debounce saves ≥1500 ms. **Zero `invoke()` on the keystroke hot path.**
4. No memory leaks. Every `setTimeout` / `setInterval` / `addEventListener` / `EditorView` / `listen()` / `requestAnimationFrame` → cleanup in `onDestroy`.
5. Minimal DOM. `display: none` not removal. No `:global()` cross-tree CSS.
6. No unnecessary imports. No `@codemirror/language-data` in FocusPane (500 KB+).
7. Test before commit. 10-char rapid type in NotePane + FocusPane after every change.
8. **Write-Time Derivation.** Every computed view maintained at write time. Persist + trigger on source-of-truth write path. Reads = cheap lookups. **No new feature may regress boot / typing / IPC** on the 7,600-note Universe.

### 10.8 Architecture principles

- **File Over App.** `.md` on disk = source of truth.
- **Local-First.** No telemetry, no cloud dependency.
- **Knowledge Formulation, not Management.**
- **The Living Link Architecture.**
- **Constraint as Design.** FocusPane has no toolbar — that IS the design.
- **Language-First by Design.** Bidi is architectural.
- **Constellation Knowledge Hierarchy** (5 levels).

### 10.9 Don't (hard "no" list)

- Don't use preview/screenshot tools unless essential.
- Don't add unnecessary abstractions.
- Don't use "vault" terminology in new code.
- Don't add a feature that makes the app slower.
- Don't commit `$effect` loops.
- Don't import heavy libraries in FocusPane.
- Don't use `position: absolute` for layout.
- Don't write CSS magic numbers without comment.
- **Don't patch the same bug more than three times** (LL-014).
- Don't create `Decoration.mark/replace/widget` inside builders — pre-cache.
- Don't call `invoke()` from a CM6 ViewPlugin or input event handler.
- **Don't duplicate working code by copy-paste-and-adapt** — extract.
- **Additional screens are displays, not domains.**

### 10.10 PCS Protocol

Push + Commit + Standing Order. Every milestone: verify build → commit → push → milestone tag → ZIP → session log → help files → 14 translations → SO.

### 10.11 Backup routine

`git tag milestone/<name> <commit>` + `git push origin --tags`. ZIP: `git archive --format=zip --prefix=Constellation-<name>/ HEAD -o "E:/Backups/Constellation/Constellation-<name>-YYYYMMDD.zip"`.

### 10.12 Versioned filename for this orientation doc — preserve every version

This file's name **always** carries its version suffix: `Constellation Orientation & Onboarding vX.Y.md`.

**Rule (corrected 2026-04-26):** when bumping the version, **write the new version as a NEW file**. Do NOT delete or overwrite the previous version. Older versions stay in `docs/` as a historical record — the project owner uses the trail to track how the project's architectural understanding evolved.

A new session reads only the highest-version file. But the trail behind it is durable.

---

## 11. Lessons Learned (LL-001 → LL-023, summary)

[`docs/LESSONS-LEARNED.md`](docs/LESSONS-LEARNED.md) is canonical.

- **LL-001** Tauri IPC = #1 perf killer. Zero IPC during typing.
- **LL-002** `+layout.svelte` reactivity cascade. Direct mutation bypasses Svelte; never store-mutate from `onDestroy` or hot path. *(2026-03-27, file 3873 lines / 77/17/19. Today: 6872 / 155/29/1.)*
- **LL-003** Build passing ≠ working app.
- **LL-004** CM6 widget event handling — capture-phase `addEventListener` on editor DOM.
- **LL-005** `tauri dev` rewrites Cargo.toml. Use forwarding feature pattern.
- **LL-006** Phase-by-phase with user GO/NO-GO.
- **LL-007** Shared plugins in `src/lib/editor/` pay off.
- **LL-008** Session log = lifeline.
- **LL-009** Derive state, don't duplicate.
- **LL-010** Merge iteration loops over visible ranges.
- **LL-011** Tauri v2 asset protocol — 4 things: protocol-asset Cargo feature; assetProtocol enable+scope in tauri.conf.json; `http://asset.localhost` in CSP `img-src` AND `connect-src`; `https:` in `img-src`.
- **LL-012** `posAtDOM` unreliable for replacement widgets. Use `posAtCoords({x, y})`.
- **LL-013** `getCursorColumn` pipe-counting bug.
- **LL-014** **Three Strikes** — fix from root after 3 failed patches.
- **LL-015** Always test production before chasing dev-mode performance (~37 s/IPC dev overhead in Tauri v2 + Vite + DevTools).
- **LL-016** Cache at the call site when callers are unknown.
- **LL-017** When patching fails, spawn adversarial expert agents.
- **LL-018** **Paint-First UI** — never gate first paint on IPC.
- **LL-019** PIXI v8 + Tauri CSP — `import 'pixi.js/unsafe-eval'` as side-effect before any PIXI class. Never relax app-wide CSP.
- **LL-020** Wall-vs-server-time diagnostics. Plus covering-index corollary.
- **LL-021** Five-stamp IPC diagnostic + `perf_trace` arrival tracer. Methodology: Stage 1 stamps → Stage 2 plausible patches (stop after 2 fail) → Stage 3 cheap falsifiers → Stage 4 dispatcher tracer → Stage 5 named-culprit conversion.
- **LL-022** Always-mounted UI = always-running IPC. `*EverOpened` lazy-mount. Reset flags on context switch.
- **LL-023** Don't regress working features. 4-step verification: render → event → state → data path.

---

## 12. Documentation drift log

| Doc | Drift |
|---|---|
| [`docs/IPC-CONTRACT.md`](docs/IPC-CONTRACT.md) | Last 2026-03-31. Lists ~80 commands; actual ~120. |
| [`docs/CE-spec.md`](docs/CE-spec.md) | Body progress table at line 862-878 stale (says Phases 4 + 7 + 12-16 not started; roadmap and code show 1–11 done). |
| [`docs/CANONICAL-FILENAME-ARCHITECTURE.md`](docs/CANONICAL-FILENAME-ARCHITECTURE.md) | Says `cid`; code uses `cid_cn` namespace — see §6.1. |
| [`docs/Constellation-Editor-Spec.md`](docs/Constellation-Editor-Spec.md) | Describes a custom-built editor never built. CodeMirror 6 was used. **Aspirational.** |
| `lab/reports/MIG-006-WIKILINK-CASCADE.md:165-167` | The §3 plan claimed an existing prop-change handler that didn't exist. |
| Audit-agent count | `lab/audit-agents.md` = 7; NotePane spec = 8 (adds EA); `docs/AUDIT-SYSTEM.md` = 14. `lab/audit-agents.md` not updated to umbrella. |
| **CE Rule 8 audit-pending** | ~~`bases.rs` (read-time `query_base`)~~ **resolved MIG-065 §I** — the Base read is now `execute_lens` (Rule-8-clean); `dataview.rs` (read-time); `lenses.rs` (hybrid violation: definitions write-time, results read-time on `apply_lens`); **Constellation Map** (`map.rs::constellation_map_universe` walks filesystem on every open). Sky View now write-time post-MIG-001. |
| **No frontend test harness** | No vitest / playwright / `*.test.ts` / `*.spec.ts` under `src/`. Rust unit tests only: 11 in `cascade_walker_tests`, 6 in `canonical.rs`, 4 in `file_kinds.rs`. |
| **No help topic for Constellation Map** | Sky View has [`docs/help.uConstellation.World/Sky View/Sky View.md`](docs/help.uConstellation.World/Sky%20View/Sky%20View.md). |
| Versioning | All three (`package.json`, `tauri.conf.json`, `Cargo.toml`) at 0.3.4 today. |
| Orientation v1.0 — auto-update toggle placement | v1.0 bug §13 said the toggle was wrongly placed under "Sky View & Links" and should be elsewhere. **The actual UI section is "Sky View & Links" and that's correct** (it's a links-cascade behavior, not a files-management one). v1.2 corrects: toggle is **correctly placed**. |

---

## 13. Outstanding bugs / cosmetic issues

| ID | Status |
|---|---|
| **BUG-013** open-editor cascade race | Open. Documented limitation: switch tabs before renaming a target whose source is visible. |
| **BUG-014** orphan `cid_cn` (collateral from BUG-012) | Closed §118 (2026-04-25). |
| **BUG-015** target-body corruption from §115 value-sync `$effect` | Vector removed at §116 (`5afe0c2`). Forensics in `lab/forensics/`. |
| Title-heading rename gap | **CONFIRMED**: [`NoteEditor.svelte:179-204`](src/lib/components/NoteEditor.svelte:179) handler calls `renameItem(filePath, newPath)` only — does **NOT** call `updateLinksOnRename`. The cascade is gated only by file-tree rename ([+layout.svelte:3807-3808](src/routes/+layout.svelte:3807) — conditional on `$appSettings.autoUpdateLinks && !isDir`). |
| Sidebar active-item highlight ~10 s lag | **Origin unresolved.** No reactive source / debounce / async refresh found that accounts for the 10 s; further forensics needed when it next reproduces. |

### 13.1 Badge taxonomy

Canonical reference: [`docs/Badge-Taxonomy.md`](docs/Badge-Taxonomy.md). Render sites (must stay in sync per the source-of-truth invariant):
- [`ConstellationMap.svelte:80-84`](src/lib/components/ConstellationMap.svelte:80) — `CAT_COLORS` map; rendered at line 660 (current result) and line 711 (result list).
- [`ConstellationSight2.svelte:79-83`](src/lib/components/ConstellationSight2.svelte:79) — `CAT_COLORS` map.

**What badges mean.** A badge tells the user **where in the note the search query matched** (or what kind of link relationship the result represents). One result can carry multiple badges.

**Content / structural matches** (where in the note the match occurred):

| Badge | Meaning | Color | Hex |
|---|---|---|---|
| **T** | Title | Blue | `#3b82f6` |
| **C** | Content (body text) | Green | `#16a34a` |
| **P** | Property (frontmatter key/value) | Amber | `#f59e0b` |
| **S** | Semantic (embedding similarity) | Purple | `#7c3aed` |
| **W** | Wikilink (`[[target]]`) | Grey | `#94a3b8` |
| **#** | Tag / Hashtag (`#tag` or YAML `tags:`) | Pink | `#f472b6` |
| **∅** | Empty / Null result | Slate | `#64748b` |

**Link-relationship badges** (matched by virtue of how the result links to/from the queried note):

| Badge | Meaning | Color | Hex |
|---|---|---|---|
| **LT** | Link **Target** (this note links *to* the queried note) | Green | `#16a34a` |
| **LF** | Link From (this note is linked *from* the queried note) | Red | `#ef4444` |
| **⇄** | Bidirectional (mutual link in both directions) | Violet | `#8b5cf6` |
| **LB** | Link Back (backlink hit) | Light blue | `#0ea5e9` |
| **LA** | Link Alias (matched via the link's display alias rather than its target) | Pink | `#d946ef` |
| **M** | Mutual link (the queried note links *to* the source AND the source links *back*) | Cyan | `#06b6d4` |

**Deprecated**:

| Badge | Status |
|---|---|
| **G** | Earlier identifier for Tag/Hashtag. Superseded by **#**. Not present in current code. |

**Unresolved**: none. M was the last pending letter; resolved 2026-04-27 as Mutual link.

**Adding a new badge**: see `docs/Badge-Taxonomy.md` § "Adding a new badge" — must update both `CAT_COLORS` maps in lock-step + this section + Badge-Taxonomy.md.

### 13.2 Filter chips on Constellation Map ([`ConstellationMap.svelte:114-125`](src/lib/components/ConstellationMap.svelte:114))

These are **search-syntax helpers**, not letter badges:
`linksTo` (`links to [[`) · `linksFrom` (`links from [[`) · `orphans` · `tag` (`#`) · `supports` (`supports [[`) · `contradicts` (`contradicts [[`).

### 13.3 Auto-update-links toggle path

**[`SettingsModal.svelte:1395-1428`](src/lib/components/SettingsModal.svelte:1395)** — under section `activeSection === 'skyview'` (display label "Sky View & Links"). Toggle binds to `$appSettings.autoUpdateLinks`. Cascade trigger ([`+layout.svelte:3807`](src/routes/+layout.svelte:3807)):

```
if ($appSettings.autoUpdateLinks && !isDir) {
  await updateLinksOnRename(lib.path, oldName, newName);
}
```

---

## 14. Where to read what (index)

| Topic | Source |
|---|---|
| Why Constellation exists / vision | [`docs/Constellation — Concept Paper.md`](docs/Constellation%20—%20Concept%20Paper.md) |
| **Sight — what it's for + analytical foundation + truth-status + v3 north star** | [`docs/Constellation-Sight-Concept-Paper-v1.1.md`](docs/Constellation-Sight-Concept-Paper-v1.1.md) (v1.1 markdown port + v3 forward-look) · v1.0 source: `docs/Constellation_Lens_Concept_Paper_Eisa.pdf` |
| **Sight v3 — visual + interaction specification (ratified 2026-05-07)** | [`docs/Constellation-Sight-v3-Concept-Paper-v1.1.md`](docs/Constellation-Sight-v3-Concept-Paper-v1.1.md) (v1.0 = draft; v1.1 = post-Eisa-design-review) |
| Map (radial sunburst) | [`docs/Constellation_Map_Concept_Paper_Eisa.pdf`](docs/Constellation_Map_Concept_Paper_Eisa.pdf) |
| Living Link philosophy + 8 properties + 7 types + 6 lifecycle stages | [`docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`](docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md) |
| Cognitive Engine 16-phase spec | [`docs/CE-spec.md`](docs/CE-spec.md) + [`docs/cognitive-engine-roadmap.md`](docs/cognitive-engine-roadmap.md) |
| Canonical filename + 12 kinds + import pipeline | [`docs/CANONICAL-FILENAME-ARCHITECTURE.md`](docs/CANONICAL-FILENAME-ARCHITECTURE.md) |
| NotePane editor rules | [`docs/NotePane-spec.md`](docs/NotePane-spec.md) |
| Audit system (7 / 8 / 14) | [`docs/AUDIT-SYSTEM.md`](docs/AUDIT-SYSTEM.md) + [`lab/audit-agents.md`](lab/audit-agents.md) |
| Migration four-phase workflow | [`.claude/skills/migration.md`](.claude/skills/migration.md) |
| PCS protocol | [`docs/PCS-PROTOCOL.md`](docs/PCS-PROTOCOL.md) |
| Working protocols / Tutorial Rule | [`docs/WORK-BEHAVIOR.md`](docs/WORK-BEHAVIOR.md) |
| Hard-won rules from real bugs | [`docs/LESSONS-LEARNED.md`](docs/LESSONS-LEARNED.md) (LL-001 → LL-023) |
| Migration plans | `lab/reports/MIG-NNN-*.md` |
| Active boot-perf budget | [`lab/boot-perf/BOOT-BUDGET.md`](lab/boot-perf/BOOT-BUDGET.md) |
| What's in flight today | `lab/reports/SESSION-LOG-{latest-date}.md` |
| Subsystem status snapshot | [`lab/reports/STATUS.md`](lab/reports/STATUS.md) |
| User-facing feature docs | `docs/help.uConstellation.World/<topic>/<topic>.md` (24 topics) |
| Master User Manual (English, 25 chapters) | [`docs/User Manual.md`](docs/User%20Manual.md) |
| 14 translated User Manuals | `docs/help.{ar,de,es,fa,fr,he,hi,ja,ko,pt,ru,tr,ur,zh}/User Manual.md` (ar = 1328 lines, others = 1120) |
| **Tauri command registry (authoritative)** | [`src-tauri/src/lib.rs:233-432`](src-tauri/src/lib.rs:233) |
| Tauri config / windows / CSP | [`src-tauri/tauri.conf.json`](src-tauri/tauri.conf.json) |
| Window permissions | [`src-tauri/capabilities/default.json`](src-tauri/capabilities/default.json) |
| Release workflow (CI) | [`.github/workflows/release.yml`](.github/workflows/release.yml) |
| Bases MVP | [`docs/BASES_MVP_SPEC.md`](docs/BASES_MVP_SPEC.md) |
| Badge taxonomy (canonical reference) | [`docs/Badge-Taxonomy.md`](docs/Badge-Taxonomy.md) |
| eNotePane build history | `docs/eNotePane-development-record.md` + `lab/experiments/phase-N-*.md` |
| Forensic snapshots | `lab/forensics/` |

---

## 15. Session-start protocol

1. **`git pull origin main`** to sync.
2. **`git log --oneline -10`** for recent work.
3. **Read `lab/reports/SESSION-LOG-{latest-date}.md`**. Look for `§STATE-OF-STANDING`.
4. **Read THIS document** (`docs/Constellation Orientation & Onboarding vX.Y.md`).
5. **Read [`docs/LESSONS-LEARNED.md`](docs/LESSONS-LEARNED.md)** — every rule was earned by a real bug.
6. **Read [`CLAUDE.md`](CLAUDE.md)** — top-principal rules + Working Agreement + Standing Orders.
7. **Read [`lab/reports/STATUS.md`](lab/reports/STATUS.md)** — one-page subsystem status index.
8. **Read memory files** at `C:\Users\ealsh\.claude\projects\E---------------Constellation\memory\MEMORY.md` and linked entries.

If any contradict each other, ground in the code (`grep`) and update the stale doc in the same session.

### 15.1 Tools you'll need

- `gh` — GitHub CLI (release pipeline, PR ops).
- `git`, `npm`, `cargo`, `tauri` (`npm run tauri`).
- `sqlite3` for direct DB inspection (Rust side ships `rusqlite` bundled — no external sqlite3 required at runtime, but useful at dev time).

### 15.2 Boot pipeline summary

1. `paint:start` → `paint` (target ≤ 870 ms — Criterion 1) → app shell visible.
2. `cache_boot_snapshot_core` (note list, libraries, settings) — awaited.
3. `cache_boot_snapshot_graph` (links, tags, aliases) — deferred via `requestIdleCallback`.
4. `cache_boot_snapshot_sky` (pre-shaped sky_nodes + sky_links from triggers) — parallel with graph.
5. `boot:hydrated` (target ≤ 6 s — Criterion 2; achieved 811 ms).

### 15.3 Wikilink resolution + collision tiebreak

Three-tier resolution ([`cache.rs:553-588`](src-tauri/src/cache.rs:553)):
1. **`name_to_idx` hit** → use canonical id.
2. **`alias_to_path` hit** → resolve to canonical path → bump on canonical row.
3. **Unresolved** → fall back to lowercase comparison; orphan edge skipped.

**Tiebreak under collisions**:
- Two notes with identical title (case-insensitive): **Unresolved** — both match, no deterministic winner.
- Title equals another note's alias: **Name wins** (tier 1 precedes tier 2).
- Two notes share an alias: **First-write-wins** — `alias_to_path` is single-valued; insertion order undefined. Practical advice: avoid shared aliases.

---

## 16. Standing Order #6 (this document's maintenance contract)

Update this document in the same commit when:

- A migration starts, ships a step, or closes.
- A top-principal rule is added or reworded.
- A BUG-NNN opens or closes.
- A drift item from §12 is fixed (remove the row).
- A new LL-NNN is added.
- A boot-perf criterion changes or closes.
- A version bumps (`Cargo.toml`, `package.json`, `tauri.conf.json`).
- A subsystem ships a major feature.
- A help topic ships or restructures.

**Bump version (1.x → 1.y)** on structural changes. **Write the new version as a NEW file** in the same commit (filename always carries version suffix per §10.12). **Do NOT delete the previous version** — every version stays as a historical record. Date-stamp every section that updates.

The document **must remain readable in one pass.** If it grows past ~1500 lines, split into linked sub-documents in `docs/orientation/`.

---

## 17. What I (Claude) have NOT read in detail (v1.2 — significantly reduced)

This list is mandated by the BASIC RULE. If you need certainty on a claim that touches an "unread" file, **read it before acting**.

**Source code I have NOT read in full**:
- Some sections of `search.rs` (4790 lines), `libraries.rs` (3978) — read at section level, not line-by-line. Function signatures, schema, triggers, command surface confirmed.
- `+layout.svelte` (6872 lines) — structural map only (region table + $effect inventory + IPC list + component mount list). Not line-by-line.
- `libraries/+page.svelte` (704), `skills/+page.svelte` (219) — listed and counted, not read.

**Docs I have NOT read in full**:
- 14 translated User Manuals (parity confirmed: ar = 1328 lines, others = 1120; same chapter structure).
- `docs/User Manual.md` chapters beyond TOC + opening paragraphs.
- Binary docs (`docs/GraphMind*.docx`, `docs/constellation_cognitive_engine_v2.1.pdf`) — text tools cannot extract reliably.

**Resolved this session (2026-05-07)**:
- `docs/Constellation_Lens_Concept_Paper_Eisa.pdf` — read in full via `pypdf` extraction; content folded into the markdown port at `docs/Constellation-Sight-Concept-Paper-v1.1.md`. Removed from the "binary docs not read" list above.

**Session logs partially read**:
- 2026-04-18 (1.46 MB): structural digest + sampled headlines (Arabic Engine M3-M14 milestone day).
- 2026-04-19 (99 KB): structural digest.
- All 20 logs digested chronologically (see §11 / §15 / §16 references throughout this doc).

**Specifics I do NOT know**:
- **Sidebar active-item highlight ~10 s lag origin** — no reactive source / debounce / async refresh isolates the lag. Reproduce-and-instrument needed.
- **Why the alias-aware sky snapshot path (`cache_boot_snapshot_sky`) is bypassed at boot** in builds that contain MIG-001 / MIG-004 §8 / MIG-005. The §88 defensive fix neutralizes user impact, but the underlying "why" is unresolved.
- **Whether `2026-04-16.UNTRACKED-BACKUP.md` (3.8 KB) and the tracked `2026-04-16.md` (13 KB) diverge in content** — sizes differ; backup may be checkpoint or partial draft. No content-level diff performed.
- **Whether the SECTOR_THRESHOLD = 8 cut-off feels right at the boundary** (v1.9 §104). The hybrid layout flips from sector to ring-per-group when the largest typed-link group exceeds 8 notes. Below 8 the sector layout looks balanced; above, the rings layout. The threshold itself is arbitrary; if Boss reports flips happening at the wrong moment for their data, the constant is one edit. Right now no data point either way.
- **Visualisation-mode distinctness (Stage 2E, deferred)** — at v1.9 commit time Boss had not yet flagged the three modes (Atmospheric / Neural / Cosmic) as too similar after the §103/§104 changes. The mode-specific decorations were redesigned to differentiate (rotating ellipses vs faint dashed rings vs solid coloured rings + sector lines + rim labels), but it's not Boss-confirmed. Triage only if flagged in 2E retest.

**Resolved during v1.9** (folded into §4.2 row 12 above, removed from §17):
- *Actual `get_360_view` latency on 7,600-note Universe.* Boss reports "almost instantly". MIG-010 priority dropped to LOW.
- *Inspector 360 first-fetch empty-state UX.* Confirmed not jarring in practice — the IPC is fast enough that the empty state barely shows.

**Resolved this session (2026-04-27):**
- **M = Mutual link** (was unresolved badge letter through v1.3). Confirmed by project owner; folded into §13.1 + Badge-Taxonomy.md.
- **W = Wikilink** (was unresolved through v1.1). Resolved earlier via Badge-Taxonomy.md.

**Future maintainers**: when you read one of the above and confirm a fact, update §17 to remove it AND fold the verified fact into the relevant section above. Keep §17 honest.

---

*End of v1.58 (preserves v1.14 footer cadence: each version is a NEW file alongside its predecessors). Maintained per Standing Order #6.*
