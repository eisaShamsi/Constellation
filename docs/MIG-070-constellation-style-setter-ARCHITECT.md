# MIG-070 — Constellation Style Setter (CSS) — Architect

**Status:** Architect (Phase 1 of /migration). Provisional number MIG-070. Date 2026-06-01.
**Author:** Claude (Opus 4.8), from a Boss-articulated vision + 3 parallel research threads.

---

## 0. The vision (Eisa, verbatim)

> "Visualize our **Constellation Style Setter (CSS)**: when a user clicks it, it opens a **full-page view** that mimics the real interface. Surrounded by every element that the user can change. As they change any element, they can see it adapt instantly. The user can navigate through each core plugin — such as **SV [Sky View], OC [OrgChart], Index, and Cataloger** — and from there change any element. When they are done, they can **save it as a theme**, name/rename it, export/import it. When they save it, a **card will be generated that replicates the interface**."

This grew out of MIG-069 (Style Presets / cards). The cards Eisa rejected because they were an *abstraction*, not the *real interface*. The card is the **output** (a saved-state thumbnail); the **Setter** is the full-page editor that produces it.

---

## 1. The headline finding: the engine already exists

Constellation already implements the field-standard theming substrate. We are **not** building a theming engine — we are building a **visual, preview-driven front-end** on top of the one that ships today. Three pillars already exist:

| Pillar | Where | What it already does |
|---|---|---|
| **Typed token registry** | `src/lib/theme/constellationStyleSettings.ts:17–195` (5 core blocks: colors · typography · layout · components · editor) + the parser/types in `src/lib/theme/styleSettings.ts:17–28` (10 control types) | A declarative catalog where each setting's `id` becomes the CSS variable `--{id}`. Covers ribbon/dock, sidebar, tabs, status bar, right sidebar, editor, links, code, callouts, typography, radii, shadows, colours. |
| **Live-apply path** | `src/routes/+layout.svelte:1553–1646` (`$effect` → `deriveThemeVariables()` `store.ts:3100` + `generateStyleSettingsCSS()` `styleSettings.ts:404`) → `setProperty()` on `document.body`/`:root`; fonts at `1655–1753` | Turns a token change into one `setProperty` call; the CSS cascade does the rest. Instant, no rebuild. |
| **Controls UI** | `src/lib/components/StyleSettingsPanel.svelte` (renders all 10 control types) wired in `SettingsModal.svelte:2519` | Already renders colour pickers / sliders / selects / toggles for the whole catalog and writes values back to the active theme. |
| **Persistence + theme model** | `ConstellationTheme` `store.ts:3054–3072` (5-colour model + `styleSettingsValues` + `styleSettingsBlocks`); save via `updateSettings()` `store.ts:4054`; theme export/import already exists (Appearance tab) | Save as a (named) theme, export/import — exactly the "save as a theme, name/rename, export/import" Eisa asked for. |

**Implication:** the Style Setter reuses all four (per our "reuse, don't copy-paste" rule and WA#5 "don't ship an inventive solution when a battle-tested pattern exists"). The genuinely **new** parts are: (a) a **full-page composition**, (b) a **live preview of the real interface** the controls drive, (c) **surface navigation** (Editor / SV / OC / Index / Cataloger / chrome), (d) **draft-vs-live** scoping, (e) the generated **card thumbnail**.

---

## 2. Territory map

### 2.1 The CSS-variable system (Agent 1 — full inventory in the session notes)
Three tiers in `src/lib/theme.css`: base colour ramp (`--color-base-*`, per `.theme-light`/`.theme-dark`) → semantic vars (`--background-primary`, `--text-normal`, `--interactive-accent`, …) → back-compat aliases. Plus Style-Settings-generated vars (`--h1-size`, `--sidebar-bg`, `--tab-active-bg`, `--link-color`, …) and JS-set fonts (`--font-interface-theme`, `--font-text-theme`, `--font-monospace-theme`, sizes). Every interface part already reads a named variable.

### 2.2 Surface rendering feasibility (Agent 2)
The preview must show each surface. Feasibility of a **live** miniature vs a **static** representative mock:

| Surface | Component (file) | Live-mini feasibility |
|---|---|---|
| Ribbon / dock | inline `+layout.svelte:4796` | **Easy** — pure SVG buttons + flags |
| Top bar / tabs | inline `+layout.svelte:5396` | **Easy** — mock tab array |
| Status bar | inline `+layout.svelte:6981` | **Easy** — mock counts |
| File sidebar (tree) | `FileTree.svelte` | **Easy** — stub `FileEntry[]` |
| Right sidebar (Backlinks / Outgoing) | `BacklinksPanel.svelte` / `OutgoingLinksPanel.svelte` | **Easy** — mock rows (already self-contained after MIG-067 §H.3) |
| Note editor | `NotePane.svelte` / CodeMirror 6 | **Hard** — CM6 is a stateful machine → render a static markdown sample (reuse `PagePreview.svelte` pattern) |
| Index | `IndexPanel.svelte` | **Hard** — virtualized + lazy IPC → static sample term list |
| Cataloger | `CatalogerView.svelte` | **Better-as-static** — needs live classifier IPC |
| Sky View | `GraphMindView.svelte` (PIXI + force sim) | **Better-as-static** — force layout is CPU-heavy → static snapshot/SVG + legend |
| OrgChart | `OrgChart.svelte` | **Better-as-static** — needs full library enumeration |

**Reusable prior art:** `PagePreview.svelte` (note render preview) and `MiniDome.svelte` (small PIXI dome) are patterns to reuse for the editor sample and any small canvas.

### 2.3 What this means for the preview
The preview is a **hybrid**: live mini-instances of the *chrome* (ribbon, tabs, status bar, sidebar, right panels — all Easy) + *static representative* renders of the *content surfaces* (editor sample, Sky View snapshot, OrgChart/Index/Cataloger mocks). Both react to the same draft variables, so theming them is identical — only the data is mocked, not the styling.

---

## 3. How the field does it (Agent 3 — the proven pattern)

The universal architecture is one loop: **typed controls → flat named tokens → CSS custom properties on a root → serialized text file.** Nobody re-renders or recompiles for a colour change.

- **Live preview — three approaches:** (A) theme the *real app in place* (VS Code, Obsidian Style Settings, DevTools); (B) a *purpose-built sampler* of components (daisyUI, shadcn); (C) an *iframe* of real components (Storybook). **Field recommendation: B-primary + A-confirm; avoid the iframe.**
- **Controls→tokens:** a typed registry where the control `id` is the CSS var name (Obsidian Style Settings — which Constellation already mirrors). Apply live via `setProperty` on the hot path + an injected `<style>` block for whole-theme swaps.
- **The scoping trick (no iframe needed):** scope draft variables to a **wrapper class** (`.css-draft { --x: … }`) so the preview resolves the draft while the rest of the app keeps `:root`. "Apply" copies the draft onto `:root`. This is Storybook-grade isolation at zero postMessage cost — and Constellation already scopes light/dark this way.
- **Persist/export:** flat JSON `{id:value}` for live state (reapply on boot) + a `.css` custom-property block for share/export (daisyUI-style bidirectional import). DTCG/Style-Dictionary is overkill.
- **Don't reinvent / don't regress:** ride the cascade + `setProperty`; no Rust on the interaction path (debounced save only); reuse components.

---

## 4. Design options (speed / effort / risk)

### 4.1 Preview rendering — **DECISION NEEDED**
- **Option A — Edit the live app in place (no separate preview).** Cheapest; the running app *is* the preview. **Risk:** the user edits the surface they're standing on; needs a clean revert. **Effort: Low.**
- **Option B — Purpose-built sampler scoped to a draft wrapper (RECOMMENDED).** A full-page mock of the interface (chrome live, content static) under `.css-draft`; "Apply" promotes to `:root`. **Matches the field + safest.** **Effort: Medium-High** (build the sampler + surface nav).
- **Option C — iframe of real components.** Hard isolation; heaviest; postMessage plumbing. **Not justified** for a global-CSS-variable app. **Effort: High.**
→ **Recommend B**, with an "Apply to app" that uses A to confirm against the real workspace.

### 4.2 Token registry — reuse vs new
- **Reuse the existing Style-Settings catalog** (`constellationStyleSettings.ts`) as the token registry, and the existing `StyleSettingsPanel` control renderers, grouped/filtered **by surface** for the navigation. **Strongly recommended** (zero new engine; one source of truth). The only additions are tokens not yet catalogued (e.g. Sky-View canvas colours) — see §8 Q4.

### 4.3 Surface coverage — live vs static (per §2.2)
- **Live mini:** ribbon, tabs, status bar, file sidebar, right-sidebar panels.
- **Static representative:** editor sample, Sky View, OrgChart, Index, Cataloger.
→ Both styled by the same draft vars; only data is mocked.

### 4.4 Persistence / export
- **Save as theme:** reuse `ConstellationTheme` + `updateSettings` — the draft's `styleSettingsValues` (+ 5 colours + fonts) become a named theme. Name/rename/export/import already exist on the Appearance tab.
- **Card thumbnail:** the MIG-069 card becomes a small render of the same sampler for the saved theme (replaces the abstract `Aa`+dots).
- **Relationship to MIG-069 "Styles":** **DECISION NEEDED** (§8 Q5) — do Themes and app-global Styles unify, or coexist?

---

## 5. Recommended architecture (the shape to ratify)

A new full-page surface — **Constellation Style Setter** — composed of:

1. **A draft token-state store** (`$state {[id]: value}`) seeded from the theme being edited; controls write to it; one `$effect` applies it via `setProperty` **to a `.css-draft` wrapper** (not `:root`).
2. **The control panel** = the existing `StyleSettingsPanel` catalog, **re-grouped by surface** (a left or right rail), so picking "Sky View" shows Sky-View tokens, "Editor" shows editor tokens, etc.
3. **The live preview** = a center pane: a **surface switcher** (Editor · Sky View · OrgChart · Index · Cataloger · Full chrome) rendering live-mini chrome + static content, all under `.css-draft`.
4. **Apply / Save** = promote the draft to `:root` (live confirm) and/or save as a named theme (reuse the theme model + export/import); generate the card thumbnail.
5. **Persistence** = reuse `updateSettings` (debounced; Rust only persists). No `invoke()` on the edit path.

This is **low-risk on the engine** (100% reuse) and **medium-high effort on the preview** (the sampler + surface nav + static content renders).

---

## 6. Invariants that must not break

1. **The existing Themes + Style Settings tabs keep working** (the Setter reuses, doesn't replace, the catalog + apply path) — unless §8 Q3 says otherwise.
2. **A no-edit boot is byte-identical** — applying a saved theme stays N `setProperty` calls at startup; **no boot-time regression** (hard constraint), no new walk.
3. **appSettings fonts + per-library `libraryAppearances` keep their cascade.** The Setter's tokens must layer deliberately (theme-default < global < per-library, or as ratified), not collide (Agent 3's caution).
4. **Second screen stays a display** — it re-applies the same variables via `notifySettingsChanged()`; it does not own the edit.
5. **Editing happens in a draft scope** — the user's live workspace is not mutated until "Apply"/"Save" (if Option B is ratified).
6. **Zero `invoke()` on the keystroke/drag path** (IPC contract); persistence is debounced.
7. **i18n:** every new label through `$t()` in all 15 locales; RTL-correct (mixed-script preview sample).

---

## 7. Proposed phases (for the Plan — to be approved)

- **§A — Shell + draft scope.** The full-page Setter view (route/overlay), the `.css-draft` wrapper, the draft token-state store, and the apply-to-wrapper `$effect` (reusing `generateStyleSettingsCSS`). Controls reused from `StyleSettingsPanel`, grouped by surface. Apply-to-`:root` + revert.
- **§B — Live preview: chrome.** The sampler with live-mini ribbon / tabs / status bar / sidebar / right panels under `.css-draft`.
- **§C — Surface navigation + content.** The surface switcher; static representative renders for editor (PagePreview-style), Sky View, OrgChart, Index, Cataloger.
- **§D — Save / theme / card.** Save the draft as a named theme; reuse export/import; generate the card thumbnail (replaces the MIG-069 abstract card).
- **§E — i18n (15 locales) + help/User-Manual + orientation.**
- **§F — 3-agent audit + PCS.**

(Phase boundaries are landable commits with Boss-test gates, per the Migration Rule.)

---

## 8. Open questions for Eisa (decide before the Plan)

1. **Draft vs live editing** — edit a **draft** (preview only) and then **Apply** (safe; recommended), or edit **live** so the real app changes instantly as you drag (Obsidian-style, no preview pane needed)?
2. **Preview fidelity for heavy surfaces** — is a **static representative** mock of Sky View / OrgChart / Index / Cataloger / the editor acceptable (recommended; live is very costly), or must any of them be **truly live**?
3. **Relationship to the current Appearance tab** — does the Style Setter **replace** today's Themes + Style Settings tabs (become THE way to theme), or **complement** them as an advanced visual mode?
4. **Scope of "every element"** — the existing catalog covers chrome + editor + typography + colours. Sky View / Cataloger have their **own canvas colours** not yet in the catalog. Add those (more work, true "every element") or start with the catalogued set?
5. **Themes vs Styles** — Eisa said "save as a **theme**." Do the MIG-069 app-global **Styles** and the per-Universe **Themes** now **unify** into one concept the Setter produces, or remain two things (a Style bundles a Theme + more)?

---

*Research provenance: 3 parallel agents (theming-engine map · surface-rendering feasibility · field-pattern research with sources incl. VS Code, Obsidian Style Settings, shadcn/daisyUI, Figma, Storybook). Full agent outputs in the 2026-06-01 session notes.*
