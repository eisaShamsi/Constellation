# PLAN — MIG-088: Style Setter Completeness (the full 149)

**Opened:** 2026-06-29 (Boss-ruled "everything"). **Architect/discovery:** the audit (`MIG-088-STYLESETTER-AUDIT.md`, wf_703dfdca-2e4) — 149 worth-restyling elements (50 high) across all surfaces; the Setter today controls 262 vars / 13 categories.

**Concept (horse):** every user-visible element in Constellation should be restyleable from the Style Setter — no hardcoded colour/shape a user can't reach. (Boss found the gap via the new "Style…" RC → opened the Setter → frontmatter tags weren't styleable.)

**Pattern that shapes the build:** ~half the findings are the SAME *cognitive-vocabulary* colour sets hardcoded per-surface (Maturity, Confidence, Origin, Stage, Search-match/category). These are consolidated into ONE shared control each, wired everywhere — better architecture + fixes ~60 findings at once. The other half are per-surface specifics.

**Standing invariants (every phase):** LL-032 — the Setter render path NEVER touches `BUILTIN_THEMES` (only per-Universe CSS-var overrides). New element → its control set + its preview case + the component wired to the var (with a fallback to today's value so nothing changes until the user edits). i18n: every new control label ×15. Boot/typing perf untouched (CSS vars only). svelte-check 0 + a Boss test per phase (Testing Instructions Rule).

---

## Phase 1 — Mechanism + Frontmatter (the trigger; establishes the pattern) **[GATE]**
- Confirm the Setter's add-an-element recipe: ELEMENTS entry `{ name, controls:[{label,type,var,...}] }` + add to a CATEGORY's `elements` + a PREVIEW case + the apply path writes the var.
- **Property tags** (`.pe-tag`) → new element: Background `--pe-tag-bg`, Text `--pe-tag-text-color`, Radius `--pe-tag-radius`, Height `--pe-tag-height`. Wire `.pe-tag` (fallbacks = today's `--background-modifier-border-focus`/#fff/--pill-*).
- **Taxonomy pills** (`.pe-taxo-pill`) → element: Background `--pe-taxo-pill-bg`, Text `--pe-taxo-pill-color`, tier colours `--pe-taxo-tier1/2/3-color`.
- Category: a new **"Frontmatter"** group (or extend Components). Preview: a mimic property row with tags + taxonomy pill.

## Phase 2 — Shared semantic colour sets ("Cognitive colours") **[GATE]**
Colour map: `lab/reports/MIG-088-PHASE2-COLOR-MAP.md` (wf_1c77386c-019). **KEY FINDING:** every semantic state is drawn in DIFFERENT colours on different surfaces today (e.g. wilting = green@40% in file-tree, lime `#a3e635` on the Map, `#16a34a` in Sky). So "one shared colour" can't be byte-identical unless each surface keeps its own value as the fallback.

**Boss ruling 2026-06-29 — "unify on demand" (per-surface fallback):** each surface wires `var(--<set>-<state>, <THIS surface's exact current value>)`. Until the user sets the shared var in the Setter → byte-identical (today's per-surface colours). Once set → ALL wired surfaces snap to it. New Setter category **"Cognitive colours"** (key `cognitive`, 3-zone, legend preview), one element per set.

**Scope split (canvas vs CSS):** CSS/inline-style surfaces wire directly. **Sky** (PIXI, `skyPalette.ts`) already has `--skyview-maturity-*` + its own Setter control → KEEP as-is (separate). **Map / OrgChart** (D3/JS hex) read via `getComputedStyle` → deferred to **Phase 7**. So Phase 2 wires the CSS surfaces only.

Sub-steps (each landable + Boss-testable):
- **§2a Maturity** (`--maturity-{seed,sapling,evergreen,canonical,wilting}`) → file-tree (`.note.mat-*`), tabs (`.tab-maturity.mat-*`), Inspector360 (`MATURITY_COLORS`). ✅ SHIPPED `ea68a565` (Boss Pass).
- **§2b Confidence** (`--confidence-{hypothesis,evidence,established,contested}`) → ConfidencePicker (`.conf-dot`, color-mix), KH confidence bars, backlinks/outgoing traversal chips. NOTE: ConfidencePicker uses `color-mix(accent N%, transparent)` not flat hex — fallback must preserve the color-mix expression.
- **§2c Origin** (`--origin-{received,discovered,mixed,none}`) → Provenance, Inspector360 (`ORIGIN_COLORS`).
- **§2d Stage** (`--stage-{spark,birth,growth,maturity,dormancy,archival}`) → KH stage cards, note stage badge.
- **§2b Confidence** → ConfidencePicker, KH confidenceColors. ✅ SHIPPED `2d7f1ac3` (Boss Pass). (Excluded BacklinksPanel traversal-chip tiers — link-weight, not confidence.)
- **§2c Origin** → Inspector360 ORIGIN_COLORS, ProvenancePanel originColor(). ✅ SHIPPED `2d7f1ac3`.
- **§2d Stage** → KH stageColors. ✅ SHIPPED `2d7f1ac3`. (Excluded note stage badge — `--text-muted` today.)
- **§2e Match-category** (`--match-category-{title,content,tag,wikilink,property,semantic,structured}`) → editor search highlight (NotePane CM6), SearchHub. ✅ SHIPPED `605642d8` (awaiting test). (OrgChart/Map/Sight D3 = Phase 7.)
- i18n fix (Boss Arabic review `372e1b29`): added missing `maturity` slug ×15 (was English-fallback in ALL locales) + ar `mixed` gender (مختلطة→مختلط, incl. source provenancePanel.mixed).

**Phase 2 CSS surfaces complete.** Remaining for these sets: the D3/canvas surfaces (Map/OrgChart/Sight badges + Map maturity/stratum) in Phase 7; Sky already has its own controls.

## Phase 3 — Editor specifics **[GATE]**
Callout type colours (9), search-highlight term badges, toolbar highlight, HTML mark, wikilink ×N chip, code-block language label, lens count badge, data-view block, frontmatter URL/fence syntax, typed-link label, image fallback.

## Phase 4 — Chrome (tabs/dock/layout) **[GATE]**
Tab bar/inactive/active-border/new-button, tab library-name label, tab-scroll arrows, sight close, modal scrim, dropdown/window shadows, flank-handle highlight.

## Phase 5 — Right-sidebar panels **[GATE]**
KH stage/total/annotated cards, Provenance external-source tag, Tasks due badges (overdue/today/upcoming) + tag pill, Review stale badge, Inspector360 cards/tensions/fragile borders, link traversal-chip tiers.

## Phase 6 — Search / Index / switcher **[GATE]**
Zero-link badge, direction arrows (in/out), index letter header, semantic/via/compound badges, cooccurrence chip, search-in-tags highlight, category badge contrast.

## Phase 7 — Sky / OrgChart / Map **[GATE]**
OrgChart search category badges (5), close-hover, cUniverse icon, folder expand/collapse icons; Map category/maturity/stratum/depth colours, arc-highlight strokes, badge text.

## Phase 8 — Calendar **[GATE]**
Popover bg/shadow/radius/rows/badge, header buttons/text/hover, cell borders/hover, today sub-text, weekday row, pill radius.

## Phase 9 — Dialogs / Settings / Global primitives **[GATE]**
ConfirmDialog/CollisionDialog buttons + overlay + radius, ContextMenu/EditorContextMenu radius+shadow+separator, EmojiIconPicker, FormattingToolbar, input focus/border, Dashboard stat value, SettingsModal status badges, toggle knob, scrollbar, color-input, FocusPane bg, child-universe stat cards, Structure panel accents.

## Phase 10 — Audit + close
3-agent audit (every new control writes its var + a preview + the component consumes it + i18n ×15; no `BUILTIN_THEMES` touch; perf clean) + `/simplify` + orientation bump + milestone tag.

---

## Rollback
Every var is additive with a fallback to today's value → the app is byte-identical until the user edits a control. Reverting a phase's commit removes its controls + restores the component fallbacks. No schema/IPC/data change — pure theming layer.
