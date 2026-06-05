# MIG-070 §C — Live-Preview for the Style Setter's Non-Editor Categories — RESEARCH

**Written:** 2026-06-05 · **For:** Eisa's request *"Non-Editor categories live preview: do some audit and research, explore other design options."* · **Status:** research only — no code, awaiting a direction choice.

After the layout redesign (`3eab826b`), only the **Editor** category has a live preview (the centre note). The other categories — **Interface, Components, Global, Sky View, OrgChart, Index, Cataloger, Shell** — style app chrome + plugin surfaces and currently show controls only (you Apply to see the effect). This doc audits how mature tools solve "preview what you're styling when it isn't a document," and proposes options.

## Key architectural fact (grounds every option)
The Style Setter already uses a **draft-scoped CSS-variable model**: edits write CSS vars into a `draft` scoped to a preview wrapper; **Apply** copies them onto `document.body`. All chrome reads those same `--*` vars off `body`. So "preview" is purely *what surface the draft vars are scoped to* — mock, real app, or wrapper — the rendering machinery is identical. Cheapest possible substrate for any option below.

## Tool-by-tool findings

| Tool | Preview target | Shape | How it handles "too big for a box" | Source |
|---|---|---|---|---|
| **Obsidian "Style Settings"** (closest analog) | Real app, applied live (injects CSS vars / classes on `body`) | Settings modal over the live workspace; the app restyles behind it | **Not solved** — relies on seeing chrome around the modal edges (Constellation's exact problem) | [README](https://github.com/obsidian-community/obsidian-style-settings), [Obsidian CSS vars](https://docs.obsidian.md/Reference/CSS+variables/About+styling) |
| **VS Code** `workbench.colorCustomizations` | Real app, live on save | Edit JSON in one pane; whole window recolors | **Names every region** (~600 keys) so you target by name, not by pointing | [Themes](https://code.visualstudio.com/docs/configure/themes), [Color reference](https://code.visualstudio.com/api/references/theme-color) |
| **Chrome/Firefox DevTools** | Real page, live (ephemeral) | Inspect cursor → hover-highlights real element → side Styles pane edits it | **Point at the full-size element in situ**; never boxed | [DevTools CSS](https://developer.chrome.com/docs/devtools/css/) |
| **Figma / Material Theme Builder** | Your mock frames, live | Property panel beside a pan/zoom canvas | **Infinite zoomable canvas** holds palettes + samples; scroll to the surface | [Material Theme Builder](https://m3.material.io/blog/material-theme-builder) |
| **Storybook** | Isolated component (mock) | Dedicated canvas + Controls table; sidebar switches component | One component at a time, full-size | [Controls](https://storybook.js.org/docs/essentials/controls) |
| **tweakcn / shadcn theme editors** | Sample components, live | Controls sidebar + large preview pane **with scene tabs** (Dashboard / Mail / Charts / Components) | **Representative mock scenes + tabs** — a "Charts" tab stands in for graph styling | [tweakcn](https://tweakcn.com/), [shadcn generator](https://www.shadcndesign.com/theme-generator) |
| **Windows Terminal / macOS Appearance** | Tiny sample / thumbnail | Inline swatches (WT); thumbnail mockups (macOS) | Keep the previewable surface tiny; **apply-on-confirm** for the rest | [WT color schemes](https://learn.microsoft.com/en-us/windows/terminal/customize-settings/color-schemes) |

**Cross-cutting finding:** there is **no single pattern** — mature tools *match the pattern to the surface*. Whole-live-app (Obsidian/VS Code), inspect-to-highlight (DevTools), mock-pane-with-scene-tabs (tweakcn/Storybook), pan-zoom-canvas (Figma), inline-swatch (OS pickers).

## Five concrete options for Constellation

| # | Option | What it is | Effort | Best for |
|---|---|---|---|---|
| **A** | **The real app is the preview** | Side-dock the Setter as a translucent panel; draft vars go live on `body` immediately; "Apply"→"Keep", close→Discard reverts. The real chrome (tree, dock, tabs, status bar, **and the live Sky View/OrgChart**) restyles as you drag. *(Obsidian/VS Code model.)* | **Low** | Interface, Components, Shell — and the **only honest** preview for Sky View/OrgChart |
| **B** | **Mock scene per category** | A centre pane renders a static HTML/SVG **mock** of that surface (fake tree + status bar; fake dock + tabs; swatch grid; mini node-cluster) reading the same draft vars. *(tweakcn/Storybook model.)* | **Med** | Interface, Components, Global. **Weak for Sky View** (a static mock can't honestly stand in for a live PIXI graph) |
| **C** | **Inline mini-previews in each control** | Each control shows its own effect inline — exactly like the **Links pills already do**. A row-colour control shows a sample row; a radius slider shows a corner; a shade shows a chip. | **Low–Med** | **Global** (shades/radii/borders that *are* their own preview) + atomic controls everywhere |
| **D** | **Inspect-to-style** | An "inspect" toggle: hover the real chrome → it highlights + names itself → click → jumps to that element's control. The reverse of "pick from a list." *(DevTools model.)* | **Med** | Interface, Components, Shell — dense chrome where *finding* the right control is the real pain |
| **E** | **Hybrid (recommended)** | Route each surface to its best fit: **A** as the substrate (live app + docked translucent shell + Keep/Discard), **D** layered for chrome discovery, **C** for Global atoms, Editor's note-preview unchanged. | **Med–High** (but each piece independently shippable) | the whole non-Editor set |

## Recommendation — Option E, built incrementally (A → D → C-for-Global)

1. **The real chrome already responds to CSS-var changes** and Apply already writes `body` → **Option A is the smallest change from today** and reuses that path (lowest effort/risk).
2. **Mock panes (B) carry the one risk the repo explicitly warns against** — `feedback_self_contained_components.md` ("a shared component can still drift per host"). A fake sidebar/dock/graph that doesn't match the real markup is a latent "looked right in preview, wrong in app" bug. Previewing on the **real surface eliminates drift by definition.**
3. **Sky View & OrgChart cannot be honestly mocked.** A static SVG cluster standing in for a live PIXI force-graph is exactly the "visual filler the geometry affords but the content doesn't require" that **Form-Aligns-To-Purpose** forbids. Only the live surface (A) previews them truthfully.
4. **Discovery is the real pain** for dense chrome (dozens of stylable parts). **D (inspect-to-style)** is the proven answer and layers cleanly on A.
5. **Global's atoms** are best served by **C (inline previews)** — the link-pill pattern already in the codebase.

**Suggested build order (each independently landable, per the Migration Rule):** A (live substrate + docked translucent shell + Keep/Discard revert) → D (inspect toggle + a `data-style-target` registry on chrome elements) → C-for-Global. Reserve **B** only if a specific category proves un-previewable live — and make any such mock self-contained.

## Relevant files
- `src/lib/components/StyleSetter.svelte` — the Setter; draft model + Apply-to-`body` path; element/control taxonomy.
- `src/lib/components/LinkTypesEditor.svelte` — the existing **inline-preview (pill)** precedent for Option C.
- `src/lib/libraries/store` — `mergeStyleOverride` / `clearAllStyleOverride` / `updateSettings` (the Apply/persist path Option A would also drive live).
