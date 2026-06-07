# Handover — Style Setter Polish (next session)

**Prepared:** 2026-06-07 · **By:** the session that closed MIG-070 §C · **For:** a fresh session to tackle the **Style Setter polish** (MIG-070 follow-ups). **Read the orientation `docs/Constellation Orientation & Onboarding v2.55.md` FIRST** — this brief sits on top of it.

---

## 0. One-line state
**MIG-070 §C is CLOSED.** The **Style Setter** is the single styling home: Style Settings tab retired, Appearance consolidated in, its own Settings nav tab (✦) + a dock inspect crosshair, **143/143 controls wired (0 dead)** by a deterministic audit, dead code removed, and the Appearance & Themes help rewritten in **all 15 languages** (Arabic term = **منسق المظهر**). Milestone `milestone/style-setter-complete` tagged + ZIP backup. Working tree clean, all pushed.

## 1. The task — three polish items
From `project_style_setter_feature_requests` + `project_constellation_style_setter` (memory) and the deferred inspect work.

### A. Real font choices *(the clearest gap)*
- **Now:** `StyleSetter.svelte` `const FONTS` (~L48) is a small curated cross-platform list (~14 stacks: System / Serif / Mono / Segoe UI / Calibri / …). It feeds the Interface / Note / Code font pickers (`type:'select', var:'--font-{interface,text,monospace}-theme'`). Per-script fonts (`AR_FONTS`/`HE_FONTS`/… ~L75) are separate, in the **Global → Per-script fonts** element.
- **Eisa wants:** "many font types in the final version — System/Serif/Mono are placeholders." Orientation notes the deeper follow-up: "a full **installed-fonts list** + per-script fonts + font-theme/numerals."
- **Decisions to make (cross-check proven methods first — WA#5):** curated-but-larger list **vs** enumerate **installed system fonts** (a Tauri/Rust command can query them) **vs** font **categories** (sans/serif/mono/display/handwriting). Whether to bundle web fonts. Whether the picker previews each name **in its own face**. See how Obsidian / VS Code / Figma do font pickers.

### B. Named, reusable colour swatches
- **Now (freshness — verify):** a **Saved-colours palette already exists** — `styleSwatches` (in `appSettings`), `addStyleSwatch`/`removeStyleSwatch` (`store.ts`), `applySwatch` (StyleSetter), rendered (~L875) when the selected element has colour controls. Picks auto-save (unnamed); click a swatch to apply it to the active colour control.
- **Eisa wants:** "**saved + named** colour swatches (reusable palette)." So the gap is **naming** (label a swatch) + richer palette management (rename / remove / order), not the palette itself.
- **Decisions:** the naming UI; keep swatches global (they already are); optional categories.

### C. Diffuse inspect targets
- **Now:** the inspect tool (Option E item D) tagged the main chrome via `data-style-target` — `+layout.svelte` (dock · toolbar · layout bar · tabs · right sidebar · status bar · sidebar shell · universe), `FileTree.svelte` (tree · folders), `TagsPanel.svelte` (tags), `NotePane.svelte` (`.e-desk`→text), `LibrarySwitcher.svelte` (universePanel).
- **Deferred (this item):** **per-library rows, child-universe rows, generic buttons.** Add `data-style-target` to each, mapping to the Setter element it should jump to — the Setter **already has** `library` / `folder` / `cuniverse` / `cButtons` elements, so these are just registry tags, no new elements.

## 2. Hard constraints (do not break)
- **LL-032 — NO themes in the Setter render path.** Never render `BUILTIN_THEMES` (card gallery OR a plain `<select>`) inside StyleSetter — it causes an unreproducible main-thread freeze. Themes live only in Settings → Appearance.
- **BUG-015 — one writer to body CSS vars.** There is a SINGLE shared apply `$effect` in `+layout.svelte` (~L1627); the transient `liveStyleDraft` layer is merged **last** there. Never add a second writer.
- **Form-Aligns-To-Purpose / the wiring audit.** Every Setter control must write a CSS var that something **actually consumes** (or an `appSettings` field via `appnum`/`toggle`/`scriptfont`/`pill*`). A control that paints nothing is a bug.
- **One location:** `E:\مشاريع كلاود\Constellation`, branch `main`.

## 3. Verification (re-run after any Setter change)
**Wiring audit — must report 0 DEAD:**
```powershell
$s = [IO.File]::ReadAllText('src\lib\components\StyleSetter.svelte')
$vars = [regex]::Matches($s, "var:\s*'(--[a-zA-Z0-9-]+)'") | %{ $_.Groups[1].Value } | sort -Unique
$blob = (gci 'src' -Recurse -Include *.svelte,*.ts,*.css | ?{$_.Name -ne 'StyleSetter.svelte'} | %{ [IO.File]::ReadAllText($_.FullName) }) -join "`n"
$dead = $vars | ?{ ([regex]::Matches($blob,'var\(\s*'+[regex]::Escape($_)+'(?![a-zA-Z0-9-])')).Count -eq 0 }
"WIRED $($vars.Count-$dead.Count) / DEAD $($dead.Count): $($dead -join ', ')"
```
- `npm run check` (svelte-check) — pre-existing errors to ignore: `store.ts:2481 'fresh'` + `PropertyEditor.svelte` node-type.
- Build discipline: stop `constellation.exe` (Get-Process | Stop-Process -Force; sleep 2) → `npm run tauri build -- --no-bundle` → verify the binary mtime advanced → relaunch for Boss tests. DevTools are dev-only; Boss tests rely on observable UI.

## 4. Key files
- `src/lib/components/StyleSetter.svelte` — the core (ELEMENTS map, CATEGORIES, control types incl. `appnum`, the FONTS/per-script consts, swatch render, inspect logic).
- `src/lib/libraries/store.ts` — `styleSwatches` (+ add/remove), `liveStyleDraft` (+ set/clear), `mergeStyleOverride`/`clearAllStyleOverride`, `updateSettings`.
- `src/routes/+layout.svelte` — the single apply `$effect`; the `data-style-target` chrome registry; the dock inspect crosshair.
- `src/lib/i18n/*.json` — locale labels (Arabic "Style Setter" = منسق المظهر; the 13 others keep their natural label). Any NEW user-facing string → all 15 files.
- Help/User-Manual: `docs/User Manual.md`, `docs/help.*/Appearance and Themes/Appearance and Themes.md` (15 langs) — keep in sync on PCS.

## 5. Process reminders
- **PCS = Push + Commit + SO** (help files + User Manual all 15 langs + orientation). Orientation v-bump rides **in the same commit** as the trigger (LL-031).
- **Plan approval = build approval** — once Eisa OKs a plan, cascade autonomously; stop only at user-testable verification clauses (articulated as tutorials), genuine architectural surprise, or completion.
- **/migration** if a change crosses subsystem boundaries; **/simplify** for local work.
- **SO #8 — cross-check each item's freshness BEFORE tackling** (against orientation v2.55 §4.x **body** + the session logs). Item B (swatches) especially — the palette already exists; confirm exactly what's missing.

## 6. Suggested opening prompt for the new session
> Resume MIG-070 — **Style Setter polish**. Read `docs/Constellation Orientation & Onboarding v2.55.md` then `docs/handover/Style-Setter-Polish-Handover-2026-06-07.md`. MIG-070 §C is closed (Setter = single styling home, 143/143 wired, 15-lang help). This session does the three polish items: **(A) real font choices** (the curated FONTS list is placeholders — research installed-fonts vs categories vs curated, with live preview), **(B) named colour swatches** (the Saved-colours palette exists but picks are unnamed — add naming/management), **(C) diffuse inspect targets** (per-library / child-universe rows / generic buttons). Cross-check each item's freshness first (SO #8). Honour LL-032 (no themes in the Setter render path), BUG-015 (one apply effect), and re-run the wiring audit (0 dead) after every change. Work in `E:\مشاريع كلاود\Constellation` on `main`. Start by cross-checking A + B against the current code, then propose the plan.
