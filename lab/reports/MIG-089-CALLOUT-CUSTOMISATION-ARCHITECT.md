# MIG-089 — Callout Customisation — ARCHITECT (2026-06-30)

> Boss asks raised at the MIG-088 §3a callout-colours test: (#1) **add custom callout types**; (#2) **change/add each type's icon** from an icon database. Boss ruling: **full feature now**, via `/migration`. Grounding workflow: `wf_a3a3c145-578` (4 agents: calloutPlugin internals · iconOverrides+picker · Setter+live-react · WA#5 prior-art).

## Concept (the horse)

**The callout vocabulary is the user's, not the app's.** A user can (a) re-icon any of the 10 built-in callout families, and (b) define their **own** callout types — a trigger word, a name, a colour, and an icon — choosing icons from the app's existing **Emoji & Icon Library**. The vocabulary **travels with the Universe** (it is part of *this library's* visual language).

This serves Knowledge Formulation: a scholar's `[!دليل]` (evidence), a researcher's `[!hypothesis]`, a lawyer's `[!ruling]` are first-class parts of how *they* think — Constellation shouldn't fix the vocabulary to 10 English defaults.

## Grounded mechanics (verified against live code)

| Aspect | Finding | Implication |
|---|---|---|
| **Custom triggers** | `[!word]` regex (`calloutPlugin.ts:176`) has **no allow-list** — any word already parses. | A custom type already renders structurally; today it falls back to **note** (blue `#448aff` + `ℹ️`). |
| **Colours** | Pure CSS: `data-callout` attr → `--callout-color` cascade (`calloutTheme`). Built-ins read `var(--callout-<family>-color, hex)` (§3a). | **Fully live** in every open editor + second screen via the single body-var writer (`+layout:1959-1988`) — **zero editor reconfigure**. For a *custom* type (no theme rule), inject `--callout-color: <hex>` **inline in the line-deco `attributes`** (`calloutPlugin.ts:235-238`) — the one point needing no per-type CSS. |
| **Icons** | JS map `CALLOUT_ICONS` (`:43-55`), baked into widget DOM via `textContent` (`:118`). **Not** read from settings. | **Not live.** But the `callout.<type>` **icon slots already exist** in `ICON_SLOTS` (`iconOverrides.ts:89-99`) with a full API (`setOverride`/`peekOverride`/`resolveOverride`) + the `EmojiIconPicker`. Wire calloutPlugin to read `peekOverride('callout.'+type)`. |
| **Live update (icons/types)** | calloutPlugin is a **static** ViewPlugin inside `livePreviewCompartment`; rebuilds fire only on doc/viewport/fold/cursor-cross. | A settings change won't repaint icons/new-types until a rebuild. Add a guarded `$effect` in NotePane keyed on the callout settings → `livePreviewCompartment.reconfigure([... calloutPlugin, calloutTheme ...])` (the existing `:840` array) to force a rebuild; the rebuild reads `peekOverride`/registry fresh. **Mirror in `ConstellationEditor/`.** |
| **Storage** | `settings.json` is **per-Universe** (`universe.rs::read/save_universe_settings`). `styleOverride`, `iconOverrides`, and a new `customCallouts` field are therefore per-Universe automatically. | Matches WA#5's **per-vault** norm + file-over-app: the vocabulary travels with the Universe. **No global store needed.** |
| **Setter UI** | Right rail renders `sel.controls` generically — no picker/form control type. | Per-type icon buttons + the add-form need a **bespoke block** `{#if selected === 'callouts'}` (mirror `{#if selected === 'links'} <LinkTypesEditor embedded />`, `:1452-1454`). **Extract `CalloutTypesEditor.svelte`** (reuse rule, not copy-paste). |

## WA#5 cross-check (settled pattern — do not invent)

Obsidian (closest peer) + Notion + community plugins (Callout Manager / CalloutX) converge on **one** model: a `data-callout` attribute carrying CSS custom props `--callout-color` + `--callout-icon`; a GUI = **list of types, each with an icon picker + colour picker + label**; **per-vault** storage; **case-insensitive parse → lowercase key**; **CSS-variable injection, never per-keystroke rule regeneration** (perf). Constellation already owns every primitive (per-Universe `styleOverride`, `iconOverrides`, `EmojiIconPicker`). The one Constellation-specific (and strictly better) choice: route the icon through the **existing iconOverrides + EmojiIconPicker** rather than Obsidian's `lucide-<id>` string field — friendlier for a non-technical Boss, no parallel icon system. (Keep `lucide-<id>` acceptance as an Obsidian-import affordance only.)

## Invariants (must not break)

1. **BUG-015 single writer** — all callout colour flows through `styleOverride`/`$liveStyleDraft` into the one `+layout` effect; never a second `setProperty` on root.
2. **CM6 freeze RULE A/B** (`calloutPlugin.ts:5-31`) — any new custom-type decoration keeps the cursor-safe `replace` guard + zero-length line decos.
3. **Editor Parity Rule** — callouts render identically in NotePane and every note view; FocusPane exempt.
4. **LL-032** — the Setter render path never touches `BUILTIN_THEMES`.
5. **Cross-window** — mirror the calloutPlugin + NotePane hook in `ConstellationEditor/` (the second app) or rendering drifts.
6. **`CalloutTitleWidget.eq()`** (`:140-148`) must compare **icon** too, else a changed icon reuses stale DOM.
7. **Perf (Rule 3 / Rule 8)** — compute the type→{colour,icon} table at write-time; CSS reads variables; no per-render CSS generation. No `invoke()` on the keystroke path.
8. **Sanitisation** — the trigger word becomes a `data-callout` attr; sanitise to a safe slug (lowercase, hyphen, strip quotes/brackets/whitespace) before it touches the DOM.

## Design decisions (recommendations — Boss confirms at Plan)

- **D1 — Storage scope:** **per-Universe** (falls out of per-Universe `settings.json`; matches WA#5 + file-over-app). *Recommended; this was the Boss's flagged call — grounding resolves it to per-Universe.*
- **D2 — Icon fidelity:** **full picker** (emoji **and** Lucide/Phosphor/Heroicons/Feather SVG), via a pre-warmed sync icon cache + a `startsWith('<svg')` render branch in the widget. (Emoji-only would feel broken when the user picks an SVG icon.)
- **D3 — Custom-type colour home:** stored **in the `customCallouts` registry entry** (injected inline). The per-element Reset (§3b-pre) extends to clear a custom type's overrides too.
- **D4 — Built-in collision:** **block** reserved slugs (the 10 families + their aliases) in the *Add custom type* form (warn: "that's a built-in — recolour/re-icon it above instead"). Built-ins are customised via their existing colour controls (§3a) + the new icon buttons; *new* types must use new names.
- **D5 — Aliases:** an icon/colour override on a canonical family (e.g. `tip`) applies to its aliases (`hint`/`important`) — the lookup keys the canonical type.

## Plan (phase-by-phase — each landable + Boss-testable)

- **Phase A — Built-in callout icons (ask #2).** Wire calloutPlugin to read the existing `callout.<type>` icon overrides (emoji + SVG branch + `eq()` icon fix + sync icon cache); add the NotePane live-reconfigure hook (+ ConstellationEditor mirror); add per-type **icon picker buttons** to a new `CalloutTypesEditor.svelte` mounted in the Callouts bespoke block. **Test:** change Warning's icon in the Setter → it updates live in an open note.
- **Phase B — Custom callout types (ask #1).** Per-Universe `customCallouts` registry `{slug,name,color,icon}`; calloutPlugin injects inline `--callout-color` + reads the icon for a recognised custom slug; `CalloutTypesEditor` gains **Add / Edit / Remove** (trigger + name + colour + icon) with slug sanitisation + built-in-collision block; per-element Reset covers custom types. **Test:** add `[!decision]` (colour + icon) → type `> [!decision] …` in a note → renders styled, live; remove it → reverts to note look.
- **Phase C — i18n ×15 + Audit.** Localise all new labels ×15; 3-agent audit (invariants 1-8, cross-window mirror, perf on a large note); `/simplify`; orientation bump + help/manual ×15.

## Rollback

Additive + per-Universe. Phase A: revert the calloutPlugin icon-read + the NotePane hook → icons fall back to `CALLOUT_ICONS` (the override rows in settings.json are inert). Phase B: `customCallouts` is a new optional field; dropping the read makes custom types render as `note` again (their frontmatter `[!slug]` text is untouched). No schema/Rust/IPC change beyond reusing `save_universe_settings`.
