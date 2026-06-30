# Session Log — 2026-06-30

**Focus:** MIG-088 Phase 3 (Editor specifics) — the Style Setter completeness sweep, editor surface. Ultracode.

---

## Resume + SO #8 cross-check (before any code)

- `git pull` → already up to date (main @ `f32acbb4`).
- Read: Orientation v3.17 "What changed", `MIG-088-STYLESETTER-COMPLETENESS-PLAN.md`, `MIG-088-STYLESETTER-AUDIT.md`, `MIG-088-PHASE2-COLOR-MAP.md`; recalled `project_stylesetter_add_element_recipe`.
- **Discovery Workflow** (`wf_7808a9cc-b8a`, 5 parallel readers) mapped the live state of every Phase 3 surface vs the (possibly stale) audit. Key SO #8 finding, verified against live code:
  - **§2e already shipped** the editor search-match highlights (`NotePane.svelte:505-510` → `var(--match-category-*, …)`). The plan line *"search-highlight term badges"* and the audit's *"Search highlight term badges (6 types)"* are **DONE** → **excluded** from Phase 3 (no re-wire).
  - The `editor` category is **already 3-zone** (twoZone exception list, line 614) with the `pk==='note'` mock-note preview — no twoZone change needed to add editor elements.
  - Callout colours: `calloutPlugin.ts:393-418` set `--callout-color` per `[data-callout]` rule (hardcoded hex); consumers (border/title-tint/body-tint/icon, lines 356/362/367/381) already read `var(--callout-color)`. 10 colour **families** (note/abstract/tip/success/question/warning/failure/danger/example/quote) — `question` & `warning` share `#ff9100` but are distinct families.
  - i18n: `success`/`warning` slugs already in `styleSetter.labels`; the other callout names are MISSING (English-fallback works, but localize for the Boss Arabic test).

## Phase 3 sub-step plan (each landable + Boss-testable)

- **§3a Callouts** — 10 family colours (this entry).
- **§3b Highlight** (unify on demand) — `<mark>` / markdown `==` / toolbar chip onto one shared bg+radius.
- **§3c Syntax tokens** — frontmatter URL (`#0891b2`) + fence/meta (`#888`), wired inside `markdownHighlightStyle.define`.
- **§3d Editor badges** — lens-count `#fff` (the last hardcoded colour) + bounded decoration radii.

---

## CLOSE-OUT (state of standing) — 2026-06-30

**Shipped + Boss-validated (protected):** MIG-088 §3a (callout colours `d395673f`) · §3b-pre (per-element reset `9c9d412f`) · MIG-089 §A (icons) + §B (custom types + Unicode/bold/saved-colours/edit fixes + unified centre-zone manager + rail-drop + in-manager reset) · Language-First audit Pass 1 (global input rule + dir, Boss PASS) + Passes 2–4 (display dir / keyboard guard / RTL layout). svelte-check 0 throughout; invariant audit clean (inline).
**Docs done:** Orientation **v3.18** (committed `7118e35b`) · User Manual EN + ×14 (`8429cdf7`,`042dda42`) · MoCh `MoCh-2026-06-30-0600`.
**Deferred / flagged (honest, not parked):**
- **A.2 known-issue** — Arabic callout **End/Home caret in the editor** still wrong (CM6 RTL-caret on callout lines; speculative fix reverted; Boss ruled "defer"). Likely the `Decoration.replace` ranges × RTL; needs a structural fix + a real reproduction.
- **Help TOPIC ×15** — the callout-customisation help *topic folders* (the User Manual ×15 IS done; the topic folders are a secondary surface) — optional follow-up.
- **`/simplify`** on the MIG-089 diff — deferred (transient server rate-limit during Phase C; re-run next session).
**Paused (pre-detour):** **MIG-088 §3b** (Highlight, unify-on-demand) · **§3c** (Syntax tokens: frontmatter URL `#0891b2` + fence/meta `#888` inside `markdownHighlightStyle.define`) · **§3d** (Editor badges: lens-count `#fff` + decoration radii). Then MIG-088 Phases 4–10.
**Unpushed:** 15 commits on `main` (offer push at session close).

---

## §3a — Callout colours — BUILT (awaiting Boss test)

**Concept (horse):** recolour the 10 callout families — the coloured note boxes — from the Style Setter. Today their hues are hardcoded in `calloutPlugin.ts`; a user can't reach them.

- **Wiring** (`src/lib/editor/calloutPlugin.ts`): each `[data-callout="x"]` rule now sets `--callout-color: var(--callout-<family>-color, <today's hex>)`. Aliases share their family var (info→note, summary/tldr→abstract, hint/important→tip, check/done→success, help/faq→question, caution/attention→warning, fail/missing→failure, error/bug→danger, cite→quote). `question` and `warning` keep **separate** vars (both default `#ff9100`). Byte-identical until edited.
- **Style Setter** (`src/lib/components/StyleSetter.svelte`): new `callouts` ELEMENTS entry (10 colour controls) + added to the `editor` category element list + a `pk==='callouts'` legend preview (mirrors the Phase-2 `cogMatch` legend; each swatch reads `var(--callout-X-color, hex)` so it re-colours live).
- **i18n** (`wf_5962d918-6d4`, 14 native localizers): 9 new `styleSetter.labels` slugs (callouts/note/abstract/tip/question/failure/danger/example/quote) added ×15 (en + 14), each matched to its locale's existing `success`/`warning` siblings + the app's established callout term. Byte-identical JSON round-trip (verified en/ar/zh); +9 keys/file, clean diff.
- **Verify:** `svelte-check` 0 errors (318 pre-existing CSS warnings, none from §3a). Frontend build ✓ (40.8s); embed confirmed (`callout-note-color`, ar `التنويهات`, fr `Encadrés` all in `build/`). Apply path confirmed: `+layout.svelte:1985` writes the Setter draft to `document.body.style` (single BUG-015 writer) → CM6 callout DOM inherits `--callout-*-color`.
- **Commit:** `d395673f` — **Boss PASS** (2026-06-30). Release binary `06:34:53` (mtime > source ✓).
- **Boss follow-ups raised at the §3a test (3):**
  1. **Custom callouts** — let a user ADD their own callout type (beyond the built-in 10).
  2. **Per-type icons** — change/add each callout type's icon, from an icon database the user browses (→ reuse the existing `EmojiIconPicker`).
  3. **DISCOVERY (safety gap):** the Setter has ONLY a universal Reset → resetting one element nukes the whole theme ("disaster"). Need a **per-element Reset** button. → Fix-what-you-discover: build FIRST (universal, protects every element).

## §3b-pre — Per-element Reset (the discovery fix) — BUILT (awaiting Boss test)

**Concept (horse):** a user tweaking ONE element must be able to revert *just that element* without nuking their whole theme. The lone universal Reset is a footgun (Eisa: "disaster").

- **store.ts:** new `clearStyleOverrideKeys(keys[])` — batch-removes named vars from the per-Universe `styleOverride` (one update + save + emit). Sibling to the existing single-key `clearStyleOverride`.
- **StyleSetter.svelte:** `selVars` ($derived) = the selected element's CSS-var keys, expanding `--interactive-accent` → its 5 decomposed keys (the Keep-time `mergedDraft` derivation). `selHasOverride` = any selVar set in draft OR saved `styleOverride`. `resetElement()` drops those keys from `draft` + calls `clearStyleOverrideKeys` → the +layout apply effect (styleOverride) and the live $effect (draft) both re-apply, reverting **only** that element. New **"↺ Reset this element"** button in the right-rail header (disabled when nothing's overridden; tooltip explains). Universal Reset untouched (still resets all). Settings-backed controls (appnum/toggle/scriptfont/pill*) are out of scope (they write appSettings, not the styleOverride layer).
- **i18n ×15:** 3 new slugs (`reset_this_element` + 2 tooltips), each reusing its locale's existing `reset` term for consistency. Byte-identical round-trip, +3 keys/file.
- **Verify:** `svelte-check` 0 errors. Frontend build ✓ (49.3s); embed confirmed (`clearStyleOverrideKeys`, ar `إعادة تعيين هذا العنصر`). Commit `9c9d412f`. Release binary `10:39:57` (mtime > source ✓). **Boss PASS** (2026-06-30).

## Callout customisation (Boss asks #1 + #2) — DESIGN PENDING (sequencing question put to Boss)

- **Discovery:** the app ALREADY has an **Emoji & Icon Library** core plug-in — `appSettings.iconOverrides: Map<slot, ref>` (ref = emoji char or `"lucide:heart"`-style id) + the `EmojiIconPicker.svelte` component. → ask #2 (per-type icons) reuses this: add `callout:<type>` slots, `calloutPlugin` reads the override (fallback = built-in `CALLOUT_ICONS`).
- **ask #1 (custom callout types)** = a persisted registry `{trigger, name, color, icon}` + `calloutPlugin` recognising the trigger (sets `--callout-color` inline from the registry) + a Setter "Add callout type" UI. Cross-subsystem (editor ↔ settings ↔ Setter ↔ icon picker) → **/migration-class**; storage-scope (per-Universe vs global) is the open design Q. WA#5 cross-check: Obsidian does custom callouts via user CSS `--callout-color`; we give a GUI over the same var.
- **Put to Boss:** sequencing — full callout feature now vs after the §3b–§3d colour sweep.
- **Boss ruling:** **"Full callout feature now"** (AskUserQuestion) → MIG-089 via /migration before §3b–§3d. **Architect + Plan A/B/C presented + Boss APPROVED.**

## MIG-089 Phase A — Built-in callout icons — BUILT (awaiting Boss test)

**Concept (horse):** change the icon of any built-in callout family, from the app's existing Emoji & Icon Library.

- **`iconOverrides.ts`:** aligned the `callout.<family>` slots to the 10 §3a families (removed `info`, added `failure`); new **`resolveOverrideSync(slot)`** (emoji as-is; SVG only if the icon cache is warm) + **`prewarmIcons()`**.
- **`calloutPlugin.ts`:** `CALLOUT_FAMILY` map (type→family, aliases inherit) + exported `CALLOUT_FAMILIES`, `calloutDefaultIcon()`; build-time icon read = `resolveOverrideSync('callout.'+family) ?? CALLOUT_ICONS[type] ?? ℹ️`; widget `toDOM` branches emoji (textContent) vs SVG (innerHTML); **`eq()` now compares icon** (else stale icon on reuse); new exported **`refreshCallouts`** StateEffect honored in `update()`; `calloutTheme` sizes `.cm-callout-icon svg` to 1em (currentColor = callout colour).
- **`NotePane.svelte`:** a guarded `$effect` keyed on the 10 callout icon-slot signature → `prewarmIcons()` → `view.dispatch(refreshCallouts)` so an open editor (incl. the second screen, which mounts NotePane) repaints live. (Colours already ride CSS live.)
- **`CalloutTypesEditor.svelte` (new):** the bespoke `{#if selected==='callouts'}` block in the Setter (mirrors `{#if selected==='links'} <LinkTypesEditor>`); 10 family rows = SlotIcon preview + Change-icon (EmojiIconPicker, shortcode→ref normalize) + Reset-icon. **Reused** SlotIcon + EmojiIconPicker + setOverride (no parallel icon system).
- **Scope:** `ConstellationEditor/` is a SEPARATE app with a different (HTML) callout renderer — NOT the live second screen (which is the main app's SecondScreenPage → NoteEditor → NotePane) → no mirror needed. z-index: picker (1000) is a descendant of the Setter overlay (9000) with no transform/filter clipping ancestor → paints above the panel (verified, no change).
- **i18n:** 3 new labels (callout_icons/change_icon/reset_icon) ×15 (localizer `wf_c6abe57a-329`).
- **Verify:** `svelte-check` 0 errors. Frontend build ✓; embeds confirmed. Commit `6dee5887`. **Boss PASS.**

## MIG-089 Phase B — Custom callout types — BUILT (awaiting Boss test)

**Concept (horse):** a user defines their own `[!trigger]` callout type — trigger word + name + colour + icon — that travels with the Universe.

- **`calloutFamilies.ts` (new):** extracted the dependency-free family data (CALLOUT_ICONS / CALLOUT_FAMILIES / CALLOUT_FAMILY / CALLOUT_BUILTIN_TYPES / calloutDefaultIcon / isBuiltinCalloutType) — shared by calloutPlugin + customCallouts (avoids a circular import). calloutPlugin re-exports CALLOUT_FAMILIES/calloutDefaultIcon for back-compat.
- **`customCallouts.ts` (new):** the per-Universe registry — `CustomCallout {slug,name,color,icon}`; sync `peekCustomCallouts`/`peekCustomCallout`; `sanitizeCalloutSlug` (lowercase/hyphen/DOM-safe); `slugStatus` (empty/builtin/duplicate/ok); add/update/remove via `updateSettings` (cross-window). Storage: `appSettings.customCallouts` (inline-typed in store.ts; default `[]`).
- **`iconOverrides.ts`:** `resolveRefSync(ref)` (resolve a raw emoji/SVG ref — for custom-type icons).
- **`calloutPlugin.ts`:** build now branches builtin vs custom — a custom type takes its icon from the registry (`resolveRefSync`) and its colour **injected inline** as `--callout-color` on the line decorations (validated hex/rgb; built-ins still use the CSS theme var). The `[!word]` parser already accepts any word → a custom type renders styled; removing it reverts to the note look.
- **`CalloutTypesEditor.svelte`:** Phase-B section — list of custom types (colour swatch [onchange, no save-storm] + IconRef preview + `[!slug]` + remove) + an **Add form** (Name + Trigger + colour + icon picker + Add) with live slug preview, built-in/duplicate collision warnings, and a unified picker target. New **`IconRef.svelte`** renders a raw icon ref (emoji/SVG) — SlotIcon untouched.
- **`NotePane.svelte`:** the live-refresh hook now also watches `customCallouts` (signature) + an **empty-skip guard** (no prewarm/dispatch on first mount when nothing is customised).
- **i18n ×15:** 6 new labels (custom_callouts/name/trigger/add + 2 collision warnings); reused `colour`/`remove`. svelte-check 0.
- **§B edit-mode (Boss ask — "edit a custom callout"):** each custom row gets an **✎ Edit** → inline **Name + Trigger** inputs + **✓/✗**; colour/icon already inline. `slugStatus(slug, editingSlug)` excludes self; a changed trigger shows a **hint** ("won't restyle callouts you've already typed"). i18n: +edit/save/cancel/callout_trigger_edit_hint ×15.
- **§B fix-3 (Boss finding — Arabic input misbehaving: cursor/Home-End/double-click).** Diagnosed via `wf_b4e3ef38-7f3`: **pure bidi**, no event interception. Root cause: the Trigger inputs had **no `dir`** (LTR-locked) and **no input in the entire app** has `unicode-bidi: plaintext` — the caret-repair the editor uses on `.cm-line` (NotePane:1635). Fix: `dir="auto"` on both Trigger inputs + `unicode-bidi: plaintext; text-align: start` on `.cte-in`. (Secondary finding: the global `handleGlobalKeydown` has no input-focus guard — a latent landmine, doesn't bite default config; folded into the audit.)
- **§B redesign — unified Callouts manager (Boss suggestion, confirmed).** Per the Full-Center-Zone rule, the Callouts editing moved into ONE box in the **centre zone**: every callout (built-in + custom) is a uniform row — `[colour] [icon] [name · aliases / [!trigger]]` with a left border in its own colour (a live mini-preview) — built-ins, a **divider**, then custom callouts, then the Add row. CalloutTypesEditor rewritten; built-in colours write the draft vars via `getDraftColor/setDraftColor` (= StyleSetter's `curVal/setVar`) passed as props; the right rail now skips the 10 generic colour controls (kept in ELEMENTS for the per-element reset) + the redundant saved-colours. Centre `pk==='callouts'` renders `<CalloutTypesEditor>` filling the zone (scrolls). Supersedes the A.1 rail-width fix.
- **A.2 — REVERTED, NOT FIXED (Boss: "still weird, stop patching").** The `isolate→plaintext` hypothesis did NOT fix the End-key behaviour in Arabic callouts. Per Reproduce-First + LL-014 (don't keep patching), the speculative `.cm-line.cm-callout-line[dir]{plaintext}` override was **reverted**. **Unresolved known issue:** a CM6 RTL-caret problem specific to callout lines; not verified by reproduction (no GUI from my side). The plausible-but-unverified area is the callout's `Decoration.replace` ranges (the `>`-prefix hide + title widget) interacting with RTL caret movement — a known-hard CM6 area; a real fix would be structural, not a CSS patch. **Awaiting Boss ruling: defer as documented known-issue vs a dedicated reproduction-driven investigation.**
- **§B refinements (Boss: "why is the right rail still there? why is Reset not active?").** (1) The right rail is now **hidden for the Callouts category** (`.ss--norail` → `210px 1fr`, right column dropped) — the manager is self-contained; the centre fills full-width. (2) A working **Reset** moved into the manager header: it reverts the built-in families' **colours + icons** (not custom callouts — they keep their own ✕), and is **active when any built-in colour OR icon is overridden** (the old greyed state only checked colours). `resetColours` + `coloursOverridden` passed from StyleSetter; the manager also clears the 10 `callout.<family>` icon overrides. svelte-check 0.
- **AUDIT LAUNCHED (Boss-requested — "comprehensive app audit so no future issues like this"):** `wf_463a3f8c-047`, 4 dimensions — (1) every editable input's bidi state, (2) display surfaces rendering user text without `dir`, (3) keyboard handlers without input-focus guards, (4) hardcoded LTR/physical layout assumptions. Solve-the-Class: the callout `.cte-in` is now the reference pattern. Findings + fix plan pending.
## Language-First audit fixes — Pass 1 (inputs) — BUILT (awaiting Boss test)

**Boss ruling:** A.2 = **defer** (documented known issue); B = **Pass**. Proceed with the audit fixes.

- **Pass 1a — global input rule (the class fix):** `src/lib/theme.css` now has a zero-specificity default `input:where([type=text],search,url,email,tel,password,:not([type])), textarea { unicode-bidi: plaintext; text-align: start; }` — every text field app-wide gets the proven caret-repair (the property the editor uses on `.cm-line`, which no plain input had). Any component rule still wins (`:where` = 0 specificity).
- **Pass 1b — `dir="auto"` on the high-severity user-content inputs** (`wf_a1d74cd9-06c`, 10 parallel agents): FileTree rename ×2, CreateItemDialog name, PropertyEditor (pe-key/val/tag-input/link-input/stage-input), UniverseManager/UniverseSetup ×2/WorkspaceManager names, TemplatePrompt value, ImporterModal subfolder, StyleSetter draft-name, ArabicOverridesPanel note. ~15 inputs; the detectDir-using inputs (RenameDialog/MoveDialog/Collision/LinkTypes/Arabic surface-lemma-root) keep their dir + now get the caret fix from the global rule. svelte-check 0.
- **Pass 1 — Boss PASS** ("checked out").
- **Passes 2–4 (`wf_3cd4bae7-bd1`, 12 agents + inline) — BUILT.**
  - **Pass 2 (display `dir="auto"`):** FileTree folder/note names, +layout tab title + lib-chip + status-bar names, SearchHub snippet/lib, Backlinks/Outgoing context+annotation, IndexPanel ref-name, StructuralOutline crumbs — each user-text span now resolves its own direction (no inheriting the row's).
  - **Pass 3 (keyboard guard):** new `isEditableTarget(e)` in `utils.ts` (INPUT/TEXTAREA/contentEditable/`.cm-editor`); the global `handleGlobalKeydown` now early-returns for bare caret/edit keys when an editable is focused (Escape still closes; modifier combos still fire); GraphMindView's 3 fly-through guards switched to `isEditableTarget`.
  - **Pass 4 (RTL layout):** Inspector360 back-arrow flips via `:global([dir="rtl"]) .i360-back-arrow{scaleX(-1)}` (the agent's extra `.i360-corner-type` flip was reverted — it mirrored the whole "Type" label); ContextMenu submenu → logical `inset-inline-*`; DashboardView + SecondScreenPage tags-col + close → logical props; SourceReviewPanel accent → logical. svelte-check 0.

- **§B fix-2 (Boss finding — "can't use saved colours"):** the custom-callout colour inputs were bare `<input type=color>` (OS picker), so the user's **saved-colours palette** (`styleSwatches`) wasn't reachable there (built-in family colours get it via the right rail). New **`ColorField.svelte`** — a swatch button → popover with the saved colours (click to apply) + a native picker (a freshly-picked colour also joins the palette via `addStyleSwatch`). Used in the custom-row + add-form. The popover is **`position:fixed`** (anchored via `getBoundingClientRect`, viewport-clamped) because `.ss-right` is `overflow:auto` and would clip an absolute child. Removed the now-unused `.cte-swatch`. i18n: reused `saved_colours`; +`custom_colour` ×15. svelte-check 0; Boss PASS pending.
- **§B fix-1 (Boss finding at test — Language-First violation + header request):** the trigger rejected non-Latin (`[!فكرة]`) because `sanitizeCalloutSlug` used `[a-z0-9]` and the parser used `\w` — both Latin-only. **Fixed:** sanitize now keeps Unicode `\p{L}\p{N}\p{M}` (Arabic/CJK/Cyrillic… stay; only spaces→hyphen, punctuation dropped); the 3 parser regexes (`findCalloutsInRange` match + body-break + `rawTitle` strip) use `[^\]\n]+` (any non-`]` run) + `.trim().toLowerCase()`. Custom Arabic types need no CSS selector (colour is inline), so `data-callout="فكرة"` is safe. PLUS: the callout header now shows the **type's display name** when there's no explicit title (custom type → its `name`, e.g. «فكرة»), matching Obsidian's default-title; `.cm-callout-title-text` weight 600→**700** (true bold). Built-ins unchanged (displayName=''). Form: slug derives from Trigger OR Name. svelte-check 0.
- **Verify:** `svelte-check` 0. Frontend build ✓ (embeds: peekCustomCallout, ar تنويهات مخصصة, zh 自定义标注). Commit + binary next.
