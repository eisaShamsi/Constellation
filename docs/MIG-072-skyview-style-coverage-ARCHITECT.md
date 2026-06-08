# MIG-072 — Full Style Setter Coverage of the Sky View Visual Vocabulary (ARCHITECT / Phase 1)

Status: design only, no code. Author: Architect phase. Date: 2026-06-08.

## Problem
The Sky View graph colours are hardcoded in two renderers and are NOT controllable by the Style Setter. Goal: every node/link/overlay style element becomes a Setter control, across BOTH renderers, with no regression when unset. This document maps the territory and lays out bridge options with costs/risks. It does NOT pick one.

## Confirmed facts (verified against files)
- graphEngine.ts (PIXI, full Sky View) calls getComputedStyle 0 times (grep: 0 matches). It reads only isDark (document.body.classList, lines 273/332) via an existing MutationObserver themeObserver (lines 331-335) that already flips isDark + sets needsRedraw. This is the natural style-change signal to extend.
- The --star-* vars in theme.css (141-145) are DEAD for the graph. Only sight/v6/dome.ts and anchor.ts (a separate dome/chrome backdrop subsystem, no Sky View reference) read them via their own getComputedStyle. graphEngine ignores them.
- TYPED_LINK_COLORS (graphEngine 88-97) is a STALE DUPLICATE of the user-editable source linkTypeColor(id) in linkTypeRegistry.ts:189 (reads linkTypesStore, written by Setter -> Links). Recolouring "supports" in the Setter does NOT change the edge; the 8 hex values even diverge from the registry. One-source-of-truth fix: the graph must READ the registry, not gain duplicate controls.
- Node base colour = colorMap[n.libraryName] (graphEngine 390), fed from GraphMindView activeColorMap $derived (GraphMindView:284) via setData(...) (699/760). This is the existing reactive seam.
- +layout.svelte $effect (1558-1587) is the SINGLE body-CSS-var writer (BUG-015). New vars ride it for free.
- --skyview-bg already works as a real CSS background behind the transparent canvas (Setter 326; consumed StyleSetter:1126). Keep intact.

## 1. Territory map (every styleable element)
Renderer: P = PIXI graphEngine, C = Canvas2D LocalSkyView, B = both.

| Element | Hardcoded value | Renderer | Single-source exists? |
|---|---|---|---|
| Node base (library/folder/stratum/maturity) | colorMap / #a78bfa fallback (gE 390; LSV 56,135) | B | yes - activeColorMap / libraryColorMap (each has own list) |
| Default node colour | 0xa78bfa (84) | P | no |
| Typed-link colours (8) | TYPED_LINK_COLORS (88-97) | P (C has none) | YES - linkTypeColor() registry; graph ignores it |
| Highlight edge | 0xf97316 (85,1946) | P | no |
| Normal edge | dark 0x475569 / light 0xbcccdc (1852); LSV rgba(255,255,255,.15) / rgba(0,0,0,.12) (118) | B (separate consts) | no |
| Cluster boundary | 0x7c3aed fallback (1887) | P | partial (clusterColors map) |
| Hover mid-arrow out/in | 0x22c55e / 0xef4444 (1971) | P | no |
| Semantic links | dark 0x818cf8 / light 0x6366f1 (2055) | P | no |
| Orphan ring | dark 0x64748b / light 0x94a3b8 (2145) | P | no |
| Active ring | dark 0xffffff / light 0x333333 (2151); LSV #fff / #333 (144) | B | no |
| Selection/highlight ring | highlightColor 0x7c3aed (161; settable 520-522) | P | no |
| Pinned ring | 0x06b6d4 (2163) | P | no |
| Stratum glow | complementaryColor(n.color) (2169) - derived, not a const | P | derived from node colour |
| Origin glow received/discovered | 0x4A9EFF / 0xFFB347 (2174) | P | no |
| Maturity ring (5) | MATURITY_COLORS (100-106) | P (C buckets via colorMap) | no |
| MOC ring | MOC_RING_COLOR 0xf59e0b (99,2190) | P | no |
| Labels | dark #e2e8f0 / light #1e293b (2336); edge-label = edgeColor (2004); LSV #fff/#000/#bbb/#555 (162-163) | B | no |
| Trail path | 0xFF6B6B (2196) | P | no |
| Search badges (7) | BADGE_COLORS (577-579) | P | no |
| Axis gizmo X/Y/Z | 0xef4444 / 0x22c55e / 0x3b82f6 (2245-2247); centre dot dark/light (2295) | P | no |
| Canvas background | --skyview-bg (shipped) | CSS layer | yes |

Missed-by-brief additions: edge-label fill (2004) tracks edgeColor (free); gizmo CENTRE dot (2295); DIM_ALPHA 0.12 (98) and normalEdgeAlpha are ALPHAS not colours (decide if in scope); LSV LIBRARY_COLORS (56) is a SECOND library palette divergent from GraphMindView's - a parity hazard independent of any bridge.

## 2. Bridge options
All assume back-fill var(--x, <current>) so unset = today. Two-renderers + live-drag assessed per row.

| Option | Mechanism | Speed | Effort | Risk class | Failure modes |
|---|---|---|---|---|---|
| A - engine reads CSS vars | graphEngine gains refreshPalette() that runs getComputedStyle(document.body) ONCE, caches hex->int into a palette field; called from the EXISTING themeObserver callback (extend its attributeFilter to ['class','style'] so Setter writes to body.style trigger it) + once on init. Never per frame. | Med | Med | Med | Reading a var inside draw() -> Perf Rule 3 violation at 7,600 nodes (must code-review the cache boundary). themeObserver now fires on every body.style write (accent HSL, fonts) -> must early-out if palette unchanged. hex->PIXI-int parse cost/edge cases. Covers ONLY P; LSV (C) still needs its own getComputedStyle path -> two impls, parity drift. Live drag: works (liveStyleDraft writes body.style -> observer -> refresh). |
| B - palette as config (Law-1 "told its data") | A skyPalette $derived in the Svelte layer resolves every var AND the typed-link colours from linkTypeColor(); passed to each engine via new setPalette(p) (mirrors setData). Engine stores it, sets needsRedraw. | Med-Slow | Med-High | Low | Largest surface: new engine field + threading through GraphMindView AND LocalSkyView. But fits the engine model (already "told" colorMap/config), keeps ZERO getComputedStyle in graphEngine (Perf Rule 3 by construction), and a SINGLE skyPalette feeds BOTH renderers -> true parity. Typed links reactive for free (derived re-runs on linkTypesStore). Live drag: $derived recomputes on liveStyleDraft/store change -> setPalette -> redraw. Failure modes: forgetting to pass palette to LSV; a $derived that reads getComputedStyle won't auto-track CSS writes unless keyed on the draft/settings (must depend on $liveStyleDraft / $appSettings). |
| C - hybrid | Typed links via registry REACTIVELY (mandatory regardless - fixes the stale duplicate); everything else via CSS vars read by the engine (Option A style). | Med | Med | Med | Two mental models in one engine (some colours pushed, some pulled) -> maintenance ambiguity. Still needs an LSV path. Inherits A themeObserver-fires-too-often risk. Upside: smallest correct fix for the headline bug (typed links) while deferring the long tail. |

Cross-cutting (all options): the typed-link duplicate MUST be deleted in favour of linkTypeColor() - not optional, independent of A/B/C. LocalSkyView needs the same palette regardless; only B gives it from one source.

## 3. Invariants that must not break
1. Perf Rule 3 - zero per-frame getComputedStyle/IPC. Graph runs 7,600+ nodes; draw() is hot. Palette reads cached, refreshed only on a signal. (B by construction; A/C need a guarded cache.)
2. BUG-015 - +layout.svelte:1558 stays the ONLY body-CSS-var writer. New vars flow through styleOverride/liveStyleDraft; no engine writes to body.style.
3. Typed-link single-source - graph reads linkTypeColor() (linkTypeRegistry.ts:189); delete TYPED_LINK_COLORS; add NO duplicate Setter control (Links category already owns them).
4. --skyview-bg keeps working unchanged (real CSS bg behind transparent canvas).
5. Dark/light still resolves - every element today has dark+light variants gated on isDark. The bridge must preserve a dark/light answer (two vars, or var defaults to today theme-correct value with isDark picking the fallback when unset).
6. Second-screen parity - LocalSkyView (Canvas2D) MUST receive the same palette; both renderers identical ("additional screens are displays, not domains"). Watch divergent LIBRARY_COLORS (LSV:56) vs activeColorMap.
7. 144/144 wiring-audit - every new var:'--X' needs a real var(--X) consumer in src (0 dead). With Option B the consumer is the skyPalette derived (document so the audit recognises engine-config vars, not just CSS var() sites).
8. Unset var = today exact look - pixel-identical to current constants; no regression.

## 4. Back-fill / rollback
- Back-fill: each control reads var(--skyview-X, <current-hardcoded>) (A/C) or skyPalette defaults to the current constant when the var is empty (B). Unset Universe = today look. No data migration: appSettings.styleOverride is free-form JSON (+layout.svelte:1573 Object.assign); new keys appear only on edit.
- Rollback: delete the new ELEMENTS/vars and the setPalette/refreshPalette calls; the engine falls back to its in-file constants (keep them as defaults). No disk/schema change to revert - stale styleOverride keys are inert (cleared by the _lastStyleSettingsKeys reset, +layout.svelte:1580).

## Critical Files for Implementation
- E:\مشاريع كلاود\Constellation\src\lib\graph\graphEngine.ts
- E:\مشاريع كلاود\Constellation\src\lib\components\LocalSkyView.svelte
- E:\مشاريع كلاود\Constellation\src\lib\components\GraphMindView.svelte
- E:\مشاريع كلاود\Constellation\src\lib\components\StyleSetter.svelte
- E:\مشاريع كلاود\Constellation\src\lib\libraries\linkTypeRegistry.ts
