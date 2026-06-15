# 28 — Command Palette (Concept Paper)

> A satellite of the Editor, not the gate. The palette is a keyboard-first launcher: it names every action once and runs it on `Enter`. It owns nothing — it dispatches.

## 1. Function in hand
The **Command Palette** — `src/lib/components/CommandPalette.svelte`, fed the command list by `getCommands()` in `src/routes/+layout.svelte` (line ~1805), opened with **Ctrl+P** (`DEFAULT_SHORTCUTS['command-palette']` in `src/lib/utils.ts`), rendered in `+layout.svelte` at the OVERLAYS block (line ~7094).

## 2. Purpose
One job: **type a few letters, find a command, run it** — a single keyboard-first index of every action in the app so the user never hunts menus. It is *navigational scaffolding*, not a knowledge act in itself; of the Five Acts it most serves **Observation** (it lets the user reach any surface — Index, Sky View, Search, the Editor's fold/property/preview commands — without leaving the keyboard). It justifies its existence as the discoverability layer: ~50 commands are reachable by name in any of 15 languages, which a flat menu bar cannot match. It does no computation and creates no knowledge of its own.

## 3. What it is NOT
- **Not** the Quick Switcher (`QuickSwitcher.svelte`) — that finds *notes*; the palette finds *commands*. They are sibling overlays, mutually exclusive (`showCommandPalette` / `showQuickSwitcher`).
- **Not** a search engine — its filter is a literal substring match over command `name`/`category`, not the FTS5/semantic instrument.
- **Not** a state owner — it holds only `query` + `selectedIndex`; every action lives in `+layout.svelte`.

## 4. Wiring
- **Inputs:** `commands: Command[]` (built fresh by `getCommands()` on each open) and `onClose` — both `$props`. Reads `$t()` for its two chrome strings. The command list itself reads `$appSettings.enabledFeatures` (to include/exclude `constellationMap`, `ccs`), `hasMultipleDisplays`, `$libraries`, and resolved shortcuts via `sc(id)` → `getResolvedShortcut`.
- **Outputs:** none of its own. On `Enter`/click it invokes the selected command's `action()` closure, then `onClose()`. Those closures flip `show*` flags, dispatch `document` CustomEvents (`constellation:fold-all`, `constellation:unfold-all`, `constellation:toggle-live-preview`, `constellation:add-property`), or call `invoke()` (e.g. `get_due_notes`, `list_trails`, `read_trail`) — but all of that lives in `+layout.svelte`, not the component.
- **Consumers:** the same `getCommands()` list also feeds `SettingsModal` (line ~7206) as the hotkey-config source, and the global keydown handler (line ~3447) which matches a pressed combo against each command's shortcut. So `getCommands()` is the single command registry for three surfaces: palette, settings hotkeys, and the keyboard dispatcher.
- **Connection to the Editor (the gate):** indirect. Several palette commands are Editor commands — `toggle-edit`, `add-property`, `fold-all`, `unfold-all`, `toggle-live-preview` — but the palette reaches the Editor only by dispatching a `document` CustomEvent that the active editor pane listens for; it never touches the note model. `toggle-bold` / `toggle-italic` / `insert-link` / `duplicate-line` / `toggle-comment` / `select-next` are currently **no-op `action: () => {}`** stubs (verify + wire or remove in bring-up).

## 5. Right-click / context menu
- **None.** The component has no `oncontextmenu` / `contextmenu` handler and does not import the shared `<ContextMenu>` / `buildContextMenu` (MIG-077). Confirmed by grep — zero matches in `CommandPalette.svelte`.
- **Gap assessment:** a launcher list arguably *could* offer a right-click on a row ("assign hotkey", "pin", "run in new pane"), but none of that exists today and the palette is keyboard-first by design (arrows + Enter). This is a **deliberate-absence-OK** call, not a debt to repay — flagged here so bring-up can confirm the Boss agrees no row context menu is wanted. No action is reachable only by right-click, because there is no right-click.

## 6. Multilingual
- **Chrome strings localized:** the input placeholder (`$t('commandPalette.placeholder')`) and empty state (`$t('commandPalette.noResults')`) exist and are translated in all 15 locales (verified ar/fa/zh).
- **Command names localized:** mostly `$t('commands.*')` — present across locales.
- **Hardcoded-English found (flag):**
  1. The palette's **own** list entry uses `name: $t('settings.plugins.commandPalette')`, and that key is the literal string **"Command Palette"** in ar/fa/zh (and every locale) — verified untranslated. So the row that opens the palette is English in every language.
  2. `{ id: 'knowledge-health', name: 'Knowledge Health' }` — a bare hardcoded English string, no `$t()` at all.
  3. Several commands use `$t('commands.x') || 'English fallback'` (cataloger, review-pulse, open-trail, create-lens, expression-forge, constellation-map, sense-making-canvas, ccs) — fine **if** the key exists in all 15 locales; the `||` fallback masks any missing key. Bring-up must verify each `commands.*` key is present in all 15 files, or the fallback ships English silently.
- **RTL:** the component has **no** `dir` attribute and does **not** call `detectDir()`. Its only logical-direction concession is CSS `text-align: start` on the row and `padding-top: 15vh` centering. Command names in Arabic/Hebrew will render but without per-row `dir="auto"`, so mixed-script command names may mis-align. **Flag:** add `dir="auto"` to `.pi-name` (and the input) in bring-up — cheap, matches the NotePane-badge polish already on the MIG-014 follow-up list.

## 7. Boot behavior
- **Runs at boot?** No. The component mounts only when `showCommandPalette` flips true (Ctrl+P / its own toggle). `getCommands()` runs on demand at open, not at boot. No IPC is issued at boot by the palette.
- **Rule 8 status:** ✅ **compliant — reads, does not recompute.** `getCommands()` returns a static literal array of ~50 command descriptors; it derives nothing from the Universe, walks no notes, and rebuilds nothing. Filtering is an in-memory substring match. The only `invoke()` calls are inside individual command *actions* (e.g. `get_due_notes`), fired by explicit user choice, not at read/boot. No write-time-derivation concern here.
- **Cost:** negligible. Building a ~50-element array of closures on each open is sub-millisecond (estimated — not separately measured; the boot baseline `paint_ms=941 / hydrated_ms=1671` from 2026-06-15 is unaffected because the palette is not in the boot path).

## 8. Flag / gate & bring-up position
- **Gate today:** **none** — the palette is ungated. There is no `enabledFeatures.commandPalette` check guarding `showCommandPalette`; the Ctrl+P binding and the overlay are always live. (A `settings.plugins.commandPalette` label exists in the plugins list, but no code path was found that flips the palette off — verify in bring-up whether a Core Plug-In toggle is intended.)
- **Bring-up phase:** **Phase 2 (satellites of the spine)** — needs the Editor (Phase 1) for its Editor commands to land, and needs whichever surfaces its commands open (Index, Sky View, Search, Settings) to exist or be themselves gated. Depends on: `getCommands()` registry + `DEFAULT_SHORTCUTS` + the global keydown dispatcher. A truly minimal editor-only shell can run with the palette off; it is not core-spine.

## 9. Budget
- **Boot budget:** **0 ms** — not in the boot path; must not register any boot-time IPC.
- **Interaction budget:** open + filter must feel instant (<16 ms per keystroke in the input). The substring filter over ~50 items is trivially within budget; guard against any future command list that issues `invoke()` during `getCommands()` (would violate Rule 3's "no IPC on the open path").
- **Regression guard:** open Ctrl+P, type 10 chars rapidly — list filters with no lag; `Esc` closes; arrow+Enter runs the right command. If `getCommands()` ever grows an `invoke()` or a Universe walk, that is a regression to reject.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** Ctrl+P opens; typing filters; arrows + Enter run the selected command; Esc and overlay-click close.
- [ ] **Serves Constellation's core purpose:** every command routes to a real surface or a real Editor event; no dead/no-op entries ship (wire or remove the 6 `() => {}` stubs).
- [ ] **Wires correctly to the Editor:** the Editor commands (toggle-edit, add-property, fold-all, unfold-all, toggle-live-preview) reach the active pane via CustomEvent and act on it; none touches the note model directly.
- [ ] **Right-click present + correct:** confirmed deliberate-absence-OK with the Boss (keyboard-first launcher, no row context menu) — or a shared `<ContextMenu>` is added, never hand-rolled.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** the palette's own row, `Knowledge Health`, and every `|| 'fallback'` command have a real key present in all 15 locales; `dir="auto"` added to the input and `.pi-name`.
- [ ] **Within budget:** 10-char filter burst shows no lag; no `invoke()` on the open/filter path.
- [ ] **Obeys Rule 8:** `getCommands()` recomputes no Universe-wide derived view; remains a static registry.
- [ ] **Holds its invariants:** palette and Quick Switcher stay mutually exclusive; Esc precedence in the global handler unchanged.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **—** · Notes: Always-on and ungated today; Rule-8 clean (static registry, zero boot IPC). Real debts to clear before re-enable: (1) hardcoded-English on the palette's own row (`settings.plugins.commandPalette` is literal "Command Palette" in every locale) and on `Knowledge Health`; (2) no `dir`/`detectDir` RTL handling on rows; (3) six no-op command stubs (toggle-bold/italic, insert-link, duplicate-line, toggle-comment, select-next). No right-click menu — judged deliberate-absence-OK, pending Boss confirmation. `getCommands()` is the shared registry for palette + settings hotkeys + keyboard dispatcher — changes there ripple to three surfaces.
