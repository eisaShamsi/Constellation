# Session Log — 2026-09-14 (continues the PJ-461 sitting)

## §1 — Stage 1 results from the Boss's four screenshots (21:00:21 binary)

| step | surface | his evidence | reading |
|---|---|---|---|
| 1 | Tasks divider | (earlier screenshot, Scratch) | **PASS** — faint grey, not the near-black `#222` |
| 2 | Note graph preview | image 1, Scratch | **"Better."** The diagram is visibly larger; "Note" also reads heavier and larger than the labels around it, which is the step's actual check. Not an unqualified pass — "better" may mean "not yet enough". |
| 3 | Template Studio selected row | image 2, **Eisa Cognitive Knowledge** (he switched universes for it) | **PASS** — 21 kinds available, so the two-row comparison was possible after all. Selected row "86 Movie" carries a pale blue tint; a different row, "66 language publisher", is plain grey. Selected and hovered are now distinguishable, which is the defect this step exists to catch. |
| 4 | Bases list | image 3, ECK | **PASS** — the open Base "EisaTest" carries a tinted fill; its four siblings have none. |
| 5 | Deleted-note text | image 4, ECK | **INCONCLUSIVE from the image.** The saved text renders in a monospaced block, but it rendered in a *generic* monospace face before this change too — that is exactly what made the fault survive unnoticed. A screenshot cannot distinguish Cascadia Code from the browser's default typewriter face at that size. Needs his side-by-side against Settings → Hotkeys. |

**He switched universes for steps 3–5** (Scratch → Eisa Cognitive Knowledge), which also retired the
single-kind limitation that had made my original Step 3 unperformable.

**Method note:** four of the five steps are judged from screenshots by ME, not by him — he supplied
evidence and no verdicts except "Better". Step 5's verdict must come from his eye, and Step 2's
"better" needs a yes/no on whether it is now enough.

## §2 — Steps 2 and 5 resolved by the Boss
- **Step 5 — PASS.** He compared the deleted-note text against the Hotkeys keycaps and ruled:
  *"I like the font of the 2nd image."* The saved text now renders in the app's Code font rather than
  the browser's generic typewriter face. **Stage 1 is 5 of 5.**
- **Step 2 — NOT ENOUGH.** *"I rather it take advantage of the empty space. You could enlarge it
  more."* The viewBox crop recovered only the HORIZONTAL space, exactly as disclosed. The remaining
  gap is vertical and structural: measured, the svg box is 741×418 while the **drawing** (wedges +
  labels, excluding the dotted spine) is ~546×156 — about **3.5:1 against a 1.8:1 box**. No amount of
  cropping fixes that; the fan has to spread vertically, which changes the drawing's own geometry.
  That is a design change to a surface he designed → panel, not my guess.

## §3 — The panel on "enlarge it more" (`wf_2fbc6ea8-916`, 7 agents, 0 failures)
Three approaches, each measured in headless Chrome and then adversarially attacked. **Chair ruled
Approach C with four of its own corrections.**

**The diagnosis all three agreed on:** the nine labels ride radially past their own wedge tips, so
about a fifth of the card's width is text. **The radius cannot grow while the labels compete for the
radial axis.** Free that axis and R goes 148 → **243 (1.642×)**; wedge ink 1.72×; wedge area 2.64×.
Against the ORIGINAL box that is 2.32× (today is 1.41×).

**Why C and not the others.** A spends the same budget on the same axis — its own sweep is flat at
~1.05× and its headline 1.27× exists only near one card shape; at the only panel size with a source
of truth in the code it measures **1.035×**, which is not an enlargement. B buys the largest radius
but with a geometry that cannot be tuned downward and a **0.9u** margin in a 638u box.

**The property that decided it:** C keeps the viewBox **size** (638×360) and only moves its origin, so
the rendered scale is bit-identical to today at every card size. The 1.642× therefore applies at
**every** panel size and viewport, and the 12px label renders at exactly today's pixel size.

**Corrections the chair made to its own winner:** a spine clamp (a Hindi chip crossed 57.5u into the
other wing's half); the row balance re-cut to 4 top / 5 bottom; the proposed "◀ incoming / outgoing ▶"
headers **dropped** as new chrome he did not ask for; and a raw i18n key removed —
`$t('cockpit.noteWord')` has **zero** hits in `src/`, so the centre box would have read as the key.

**Three corrections to MY brief**, all measured: Russian "поддерживает" is **82.6u**, not the 49.5u I
told them, and the widest label overall is Hindi's supersedes at **91.6u**; my "90% width / 56% height"
fill is really **85.4% / 43.5%** in English; and **today's layout is NOT collision-free in English** —
one label sits on a wedge, plus 11 label-label overlaps across the locale set. The proposal measures
**zero** of all of those, in all 15 languages.

**NOT APPLIED.** It changes the drawing's character, not only its size, so it goes to him as a picture
first: `lab/reports/PJ-461-relgraph-proposal.png` (today above, proposal below, same card size).
**My first render of that picture was wrong** — the colour chips landed on the far side of their
words on the right wing — and was rebuilt with measured label widths before he saw it.

## §4 — He approved the layout; built to the panel's spec
*"I like the proposed layout. Go for it."* — Eisa, 2026-09-14.

Four hunks applied to `StyleSetter.svelte`, each asserting its anchor before touching anything:
1. **`relTextW()`** — a canvas text measurer placed after `curVal`, so cell widths follow the DRAFT
   interface font the user is editing in that same panel (agreement with `getComputedTextLength`
   measured at 0.014u worst case over 15 locales × 9 labels).
2. **`relWedges` → `relModel`** — R 148 → **243**, and the two-row callout packer. Every cell is
   clamped inside its own half (`[6, 311]` / `[327, 632]`) so nothing can cross the spine, which is
   the diagram's only directional encoding. Cells are ordered by rim x, so leaders cannot cross each
   other. `k` condenses glyphs as a last resort rather than letting a future translation spill.
3. **The markup** — leaders painted FIRST so a wedge always covers a leader, never the reverse.
4. **CSS** — a `.ss-relleader` rule, and the previous comment replaced (it described the 138-origin
   crop and claimed a ±46u margin; the real margin in that build was 13.4u).

`relWedges` now has **zero** references — the swap is complete, and `svelte-check` is
**0 errors / 268 warnings**, identical to the pre-edit baseline.

**A discrepancy I caught between what he approved and what the panel specified.** My comparison
picture drew the colour chip before the word on BOTH wings. The panel's rule mirrors it: the chip
sits on the side of the word FACING its wedge, so the leader reaches it from outside and can never
cross the word (measured 0 crossings in all 15 locales). That is a real geometric reason, so I built
the panel's rule — and re-rendered **the shipped algorithm itself**, transcribed line-for-line, so
the picture he gets now matches the app rather than my approximation:
`lab/reports/PJ-461-relgraph-shipped.png`. The visible difference is small: on the right wing the
coloured square follows the word instead of leading it. The panel flagged this as a taste call he
may overrule.

## §5 — Built and verified (binary 2026-09-14 11:25:15)
Ordering holds: HEAD 09-13 11:31 → frontend 09-14 11:21:33 → binary 11:25:15. The exe contains this
build's stylesheet filename, and the emitted CSS carries `.ss-relleader`. **Every earlier fix
survived the rebuild**, checked individually rather than assumed: `.ss-center { min-height: 0 }` is
present (my first regex matched the two-zone rule again and reported False — listing all matching
rules settled it), the retired names are still zero but for the two deferred Calendar lines, the
positive literal reads 1, and the Template Studio selected-row rule still sits after hover.
`vitest` 1008 passing, `svelte-check` 0 errors / 268 warnings.

## §6 — STAGE 1 COMPLETE: 5 of 5, Boss-passed
*"Step 2: Pass."* — Eisa, 2026-09-14, on the 11:25:15 binary. His screenshot shows the redesigned
preview: wedges reaching the card edges, the nine names in two rows with their colour chips and
connectors, no overlaps, and the mirrored chip rule visible on the right wing exactly as built.

| step | verdict |
|---|---|
| 1 Tasks divider · 3 Template Studio · 4 Bases row · 5 Deleted-note text | PASSED 2026-09-13 |
| 2 Note graph preview (redesigned, R 148→243) | **PASSED 2026-09-14** |

## §7 — Facts measured in HIS daily universe before writing Stage 2
Read from `Eisa Cognitive Knowledge/.constellation/settings.json` rather than assumed:
- `--background-modifier-hover: #cdcbcb` **and** `--background-modifier-border: #cdcbcb` — both
  pinned, both applied inline on `document.body`, so **neither follows the theme**.
- `--interactive-accent: #215de8` (blue, not the default purple).
- `--cal-cell-bg` **not set**, `--background-primary` **not set** — which is exactly why the calendar
  cells fall through to the fix and why the dark-theme change will be visible.
- **25** `--cal-*` overrides tuned in the light theme, untouched by this job.
- `colorScheme: light` — so switching to dark is a deliberate step, not his normal state.

**Consequence that must be disclosed in Stage 2, or it will read as a bug:** in the dark theme the
HOVERED row will be a light grey (his pin) while the SELECTED row is an 18% blue tint over a dark
ground — so the hovered row will look *brighter* than the selected one, the reverse of light mode.
That is his own pinned colour meeting the dark theme, not this change.

## §8 — My Stage 2 instruction was WRONG, and it exposed a real discoverability defect
He typed **"theme"** into the command palette and got **"No results"**, then asked: *"Do we have a
Theme control?"*

**My instruction was wrong, and the gate could not have caught it.** The `ui-inspector` verified the
command exists (`+layout.svelte:2742`) and that its label resolves to "Toggle dark/light mode"
(`en.json` `commands.toggleTheme`). Both true. **What nobody checked was whether the search word I
told him to type would MATCH.** `CommandPalette.svelte:27-31` filters on
`c.name.toLowerCase().includes(query)` **or** `c.category.toLowerCase().includes(query)`. The name is
"Toggle dark/light mode", the category is "Appearance" — **neither contains the string "theme"**.
This is the same class as the Organization Chart step: a verified string, a false instruction.
Working search words: **dark · light · toggle · mode · appearance**.

**And the answer to his actual question is worse than a typo.** Measured:
- The **only** writer of `colorScheme` in the entire app is `handleToggleTheme`
  (`+layout.svelte:6100-6108`), reachable from **one** place: that command-palette entry.
- **No Settings control exists** — `grep` for a `bind:`/`onclick`/`select`/`input` touching
  `colorScheme` in every component returns nothing.
- **No default keyboard shortcut** — `toggle-theme` is absent from `DEFAULT_SHORTCUTS`
  (`utils.ts`), so the Hotkeys screen shows it as "Not set".

So a top-level appearance setting has exactly one route, that route has no shortcut, and the route
cannot be found by searching its own concept word. **Filed for the ledger as a discoverability
defect; it is not part of PJ-461 and I will not widen this job to fix it without his ruling.**

## §9 — BOSS RULING, recorded so it cannot be lost: the Style Setter IS the theme control
> *"When I decided to create the 'Style Setter', it was to serve as Constellation's Theme control.
> The 'Saved Styles' are the user themes that they can switch to as they wish. What is missing is a
> quick toggle to those saved styles through Ctrl + P."* — Eisa, 2026-09-14

**This corrects my framing in §8.** I filed "no Settings control for `colorScheme`" as a
discoverability defect, reasoning from the code outward. That was the wrong frame: light/dark is one
switch inside the theme system, not the theme control. **The Style Setter is the theme control and
Saved Styles are the themes** — a concept statement, and the kind that gets lost when it lives only
in a chat (see the Linked Universe naming ruling, which was taken once, written nowhere, and later
recommended back to him by a review panel). It goes into the ledger and CLAUDE.md at the close.

**The gap he named, verified in the source — the parts already exist and are cleanly separated:**
- `loadStylePresets()` and `applyPreset(preset)` are both exported from
  `src/lib/libraries/stylePresets.ts`, independent of the Style Setter component.
- `applyStyle()` (`StyleSetter.svelte:1030`) is a four-line wrapper over `applyPreset`.
- **What is missing** is only the wiring: `savedStyles` is component-local state
  (`StyleSetter.svelte:727`), loaded when the Setter opens, while `getCommands()`
  (`+layout.svelte:2711+`) builds its list synchronously. So a palette entry per saved style needs the
  presets available at app level — a store loaded at boot or on first palette open — and then one
  command per style calling `applyPreset`.

**Not started, and NOT folded into PJ-461** — it is a new capability, not a CSS-token repair. Filed
for the ledger; his call on timing.

## §10 — FILED: PJ-475 and PJ-476 — his ruling, "after PJ-461 closes"
Highest number in ledger v2.13 is PJ-474, so these take **475** and **476**. Both land in the v2.14
bump at the PJ-461 close (SO#9), not before — he ruled the timing himself.

### PJ-475 — Switch a Saved Style from the command palette *(his request, 2026-09-14)*
**Concept (the horse):** *"Let me put on a different theme without going anywhere."* The Style Setter
IS Constellation's theme control and Saved Styles ARE the user's themes — his words. Choosing one
should not require opening a full-page editor, any more than switching a note should require opening
Settings.
**What exists already** (verified, not assumed): `loadStylePresets()` and `applyPreset(preset)` are
exported from `src/lib/libraries/stylePresets.ts` and are independent of the Style Setter component;
`StyleSetter.svelte:1030 applyStyle()` is a four-line wrapper over `applyPreset`.
**The only missing piece:** `savedStyles` is component-local (`StyleSetter.svelte:727`), loaded when
the Setter opens, while `getCommands()` (`+layout.svelte:2711+`) builds synchronously. Needs the
presets at app level — a store loaded at boot or on first palette open — then one command per style
calling `applyPreset`, category "Appearance".
**Open questions for the build:** does the list refresh when a style is saved/renamed/deleted while
the palette is closed? Should the ACTIVE style be marked in the list? Do the commands need stable ids
for the Hotkeys screen, so a user can bind a favourite theme to a key?

### PJ-476 — "Toggle dark/light mode" is unreachable by its own concept word *(found 2026-09-14)*
**Concept:** a control the user cannot find does not exist for them.
**Measured:** `CommandPalette.svelte:27-31` matches only `name` and `category`; the command is named
"Toggle dark/light mode" in category "Appearance", so **typing "theme" returns No results** — which is
what he hit. It also has **no default shortcut** (absent from `DEFAULT_SHORTCUTS` in `utils.ts`), and
**no Settings control writes `colorScheme`** — `handleToggleTheme` (`+layout.svelte:6100-6108`) is its
only writer.
**Smallest honest fix:** give the palette an alias/keyword field, or rename the command so its own
concept word matches. **Note the frame correction:** this is NOT "the theme control is missing" — per
PJ-475 the Style Setter is that. This is one switch inside it being hard to reach.

## §11 — The exemption re-verified against the GROWN diff, and four defects fixed under WA#6
The earlier EXEMPT verdict had been earned against a 73-line CSS-only diff. The redesign added ~100
lines of real logic, so the verdict was **re-checked rather than carried forward** — the stale-
verification pattern that has already bitten twice today.

**Verdict: still EXEMPT**, on evidence re-derived from scratch: every changed line in all 21 files
sits inside a `<style>` block or inside `livePreview.ts`'s single static `EditorView.theme({…})`
object (boundaries measured: `</script>` at :1301, `<style>` at :1893). Whole-diff negative control —
both sides of all 754 lines contain **zero** `invoke(`, `emit(`, `listen(`, `localStorage`,
`updateSettings`, `onMount`, `onDestroy`, `addEventListener`, `setTimeout`, `noteModel`, `saveNote`,
`frontmatter`, `db.`. The component is not in the second-screen graph. The new code writes nothing:
`relModel` writes no state and cannot loop; the packer's objects are created fresh each run and
escape only to the template; `curVal` is a pure read. SSR is off (`+layout.ts:5 export const ssr =
false`) and there is no prerender directive, so the canvas can never run without a DOM.

**Four defects it surfaced — none an inspection trigger, all fixed in the same pass (WA#6):**
1. **A canvas leak on a host without 2D support.** `relCtx ??= …getContext('2d')` re-allocates a
   fresh `<canvas>` on **every call** — nine per recompute — if `getContext` ever returns `null`.
   Fixed with a `relCtxTried` flag so the attempt happens once.
2. **The backing store was never released.** `relCtx = null` added to `onDestroy`.
3. **Stale font measurement.** `curVal` falls through to `getComputedStyle(body)` for anything not in
   the draft, which is not a reactive read — so changing the interface font OUTSIDE the panel left
   the labels measured against the old font. Fixed with a `void $appSettings` dependency.
4. **Nine forced style reads per recompute.** The font is now resolved **once** into `relFont` and
   passed in, instead of `curVal` being called per label inside the derived.

`svelte-check` and `vitest` re-run after the fixes. **The binary is deliberately NOT rebuilt yet** —
he is mid-Stage-2 on the 11:25:15 build, and none of these four touches anything Stage 2 looks at.

## §12 — A verifier pass refuted two of the exemption's own claims, and caught a FALSE COMMENT I shipped
The exemption agent launched an independent `findings-verifier` over its eleven load-bearing claims.
It **upheld the EXEMPT verdict** — nine claims confirmed, several by two methods that could have
disagreed (it walked the second-screen import graph to 95 modules AND grepped the shipped screen
bundle; the raw string "Relationship colours" DOES appear there, and it correctly resolved that to
the i18n payload rather than reporting a false failure). But it refuted two claims and found three
defects in my own durable record.

**Its own method note is the lesson of the day, again:** its first pass on claim 10 searched the
registries for quoted `--name` strings and found none — a CONFIRM. The registries store keys
**without** the `--` prefix (`constellationStyleSettings.ts:11` says so in its own doc comment). The
check could not have disagreed. Re-run against the real key shape, it flipped.

**REFUTED 1 — `--sidebar-width` IS a registry key** (`constellationStyleSettings.ts:94`), so "not a
key anywhere" was wrong. The consequence does not follow, for a reason worth recording: that registry
is **dead** — `generateStyleSettingsCSS` has zero call sites and `StyleSettingsPanel` is never
mounted (`+layout.svelte:2490` — "MIG-071: the theme subsystem was removed"). `--sidebar-width` had
exactly one consumer in `src/` at HEAD, the Style Setter's own sidebar mock, and none after this
diff. Behaviourally a no-op; the ledger must say **that**, not "not a key anywhere."

**REFUTED 2 — "every custom-property reference on an added line is a `var()` consumption"** is false
at exactly one line: `StyleSetter.svelte:934`, `curVal('--font-interface-theme')`, which reaches the
CSSOM from JavaScript. Still a read, not a declaration — but it is the one added line a reviewer
would most want named, and my sentence hid it.

**THREE DEFECTS IN MY OWN COMMENTS — all fixed:**
1. **A false causal claim in shipped code.** `TemplateStudioRow.svelte` said the new selector
   "must out-rank `.ks-row:hover` (0,2,0)". **`.ks-row.ks-row-sel` is ALSO (0,2,0)** — it does not
   out-rank, it **ties and wins on source order**. The behaviour is right and the diagnosis of the old
   bug is right, but the stated mechanism was wrong, and the difference matters: an order-dependent
   win breaks silently if anyone reorders the block. Rewritten to say exactly that, including the
   warning. (I had told him the correct version in chat and the wrong one in the code — worse than
   either alone.)
2. **Comments narrating a state that is not HEAD.** They described the box as "keeping its 638x360
   size, only the origin moves to 0" — true of an intermediate step inside this session. Against the
   **committed baseline** the box goes **900x360 -> 638x360** and its origin was already 0. A durable
   comment that describes a state no commit contains is a trap for the next reader. Rewritten.
3. **An overstated invariant.** "The rendered scale is unchanged at every panel size" holds only in
   the **height-limited** regime: between the old aspect (2.500) and the new (1.772) the fit switches
   from width- to height-limited and the 12px label grows by up to **1.41x**. Now stated with its
   regime and the stage-height window where it applies.

**And one honesty fix:** the harness that produced the measured figures lives in the scratchpad, not
the repo, so those numbers would land as durable assertions on my say-so. The comment now says so,
and names the one figure re-derivable from the file itself (243/148).

## §13 — Final build for Stage 2 (binary 2026-09-14 13:53:34)
Rebuilt BEFORE he runs Stage 2, deliberately: his standing order is that he tests every build before
its commit, so had he tested on 11:25 and I then rebuilt for the audit fixes, the committed artefact
would be one he never saw.

Ordering: HEAD 09-13 11:31 → frontend 13:47:17 → binary 13:53:34. Verified the exe embeds THIS build
by two artefacts, not one: the current stylesheet `0.oiBu_2E-.css` **and** the chunk carrying the new
measurer (`CAVFSnPA.js`). **The stylesheet filename is unchanged from the previous build** — expected,
since CSS comments are stripped in production so the emitted CSS is byte-identical, which is exactly
why the stylesheet alone would NOT have proven the script fixes shipped. The emitted code shows the
font resolved once (`500 12px ${ot(…)}`) rather than per label.

Gates: retired names zero but for the two deferred Calendar lines; the selected-row rule still later
than hover; the enlarged viewBox present; `svelte-check` 0 errors / 268 warnings; `vitest` 1008
passing.

## §14 — BOSS RULING: the built-in dark theme is not his environment, and Stage 2 was mostly wasted
> *"This is the case when using Ctrl + P = dark, to choose the dedicated dark theme, which I never
> use, and I don't think it is suitable for the app. That's why I prefer to use the Style Setter
> instead."* — Eisa, 2026-09-14

**Stage 2 Step 1 PASSED** — image 1 shows the Calendar's day cells taking the dark panel colour with
the dates readable, where they had painted solid white. The headline fix works.

**"Worst theme I've ever seen" is his verdict on the BUILT-IN dark theme, not on this work.** Steps 2
and 3 of Stage 2 asked him to re-check three surfaces inside that theme; that is of no value to him
and it is dropped.

**The miss is mine, and the evidence was in front of me.** I measured `colorScheme: light` in **all
nine** of his universes and wrote it up as *"switching to dark is a deliberate step, not his normal
state."* The correct reading of nine-for-nine is **"dark is not his environment at all"** — and the
right question, which nobody in the auditor → inspector → panel chain asked, is *does he use this
theme?* Instead I built a whole stage inside it, and its headline fix — the white calendar cells — is
a fault he would never have met. **The fix is still correct** (it is a real defect for anyone who does
use dark, and it was live on disk), but I should have said so and moved on, not staged a sitting
around it.

**Consistent with his 2026-09-14 ruling on PJ-475:** the Style Setter is Constellation's theme
control and Saved Styles are the themes. The light/dark toggle is a separate, older layer — and his
own style pins **34 background/text colours** (measured: `--background-secondary #e1e3e5`,
`--background-secondary-alt #ffffff`, `--topbar-bg`/`--dock-bg`/`--statusbar-bg`/`--center-zone-bg`
all `#e1e3e5`, against `--background-primary`/`--text-*` left free), so toggling dark flips only what
he has NOT pinned and produces a hybrid. **Filed as PJ-477** — see below; not part of PJ-461.

### PJ-477 — the light/dark toggle and Saved Styles are two theme layers that collide
**Concept:** a user has ONE theme. Today the app has two independent layers — the built-in
light/dark scheme, and the per-Universe `styleOverride` a Saved Style writes — and the toggle changes
only the first. A user whose style pins light colours gets a half-dark result and reasonably calls it
broken. **Measured on his daily universe:** 174 override keys, 81 pinned hex colours, 36 of them very
light (luminance ≥ 170) and 13 very dark. **His own words:** the built-in dark theme is *"not
suitable for the app."* Options range from a dark variant per Saved Style, to making the toggle
select between two Saved Styles, to retiring the built-in toggle in favour of styles. **His design
call, not mine** — it needs the same panel treatment PJ-475 will get.

## §15 — Stage 3: the three lenses, on his second screen (Eisa Cognitive Knowledge, note الحضارة الإسلامية, 136 links)
He sent one screenshot per lens, no verdict — the same pattern as Stage 1, evidence for me to read.

**The observable the fix produces is DIFFERENTIATED label sizes**, where every label previously drew
at the single inherited 14px/400 (`SecondScreenCockpit.svelte:186 .ck { font-size: 14px }`). All three
show it:
- **Orrery** — three tiers visible at once: the ring pills ("never walked / older / this quarter /
  this month / this week / today"), the rim labels colour-coded with their counts
  (`part-of· 9`, `supports· 20`, `generalizes· 1`, `derives-from· 106`), and the small cluster badges
  (`+38`, `+4`). The centre pill carries the note name over "136 links".
- **Butterfly** — the centre pill's title reads heavier and larger than the flank labels beside the
  wedges.
- **Ledger** — the type names are bold and colour-coded, the bar counts sit beside them, and the axis
  ticks (106 / 53) are distinctly smaller. Three sizes in one view.

**What I can and cannot judge from these.** I can see the differentiation is present and that nothing
collides, overflows or renders unreadably. I **cannot** verify from a screenshot that a given label is
now 9px rather than 14px — Arabic and Latin at different weights on a high-DPI second screen defeat
that. The verdict that matters is his eye on whether anything is now too small.

**One action outstanding and easy to miss:** the lens choice **persists** to
`settings.json.noteGraphStyle` via `requestLensChange` → `updateSettings`. He started on **The Orrery**
(his ECK setting) and his last screenshot is **The Ledger**, so ECK is now set to Ledger until he
clicks back.

## §16 — NEW LAW: Check What Is Already Built BEFORE You Ask (Boss-dictated 2026-09-14)
> *"This kind of mistakes is unacceptable. Don't ASSUME at all. GO and CHECK what have been built
> before construct a question for something already done, or solved."* — Eisa, 2026-09-14

Written into `CLAUDE.md` above the Test-Pipeline law, and into memory
(`feedback_check_built_before_asking.md`, `project_style_setter_is_theme_control.md`, both indexed).

**It is the fourth statement of one principle, and the first that costs HIM rather than the record.**
Don't-Make-Things-Up forbids fabricating claims in output; No-Guessing forbids fabricating them in
reasoning; Never-Describe-The-App forbids fabricating the app. **This one forbids fabricating the OPEN
QUESTION** — presenting as undecided something the app has already decided.

**Three violations, all in one sitting today:**
1. **The Calendar typeface.** I put a three-option design question to him **without opening the Style
   Setter's Calendar category**, which he built and which carries **33 controls including "Calendar
   font"**. Only the *unset default* was ever open — a fraction of what I asked.
2. **"Do we have a Theme control?"** I filed "no Settings control writes `colorScheme`" as a defect,
   reasoning outward from code, **without checking the Style Setter** — which IS the theme control.
3. **"Nothing was written."** I told him an edit had not landed, from a tool's rejection message,
   **without opening the file.** It had landed, on both lines. I corrected it in the next message.

## §17 — The outstanding question WITHDRAWN by applying the law
I had ended §15's exchange asking him whether "Reset this element" should land on Amiri or on his Note
font. **That question is already answered by the code**, and asking it would have repeated the
violation the law was just written for.

`CalendarPanel.svelte:225,323` reads, and has always read,
`var(--cal-font, 'Amiri', 'Cairo', var(--…))` — **Amiri is FIRST in the author's own fallback chain**,
ahead of Cairo and ahead of the note font. The unset case landing on Amiri is not a side effect of the
repair; it is the declaration's stated intent, which has simply never been reachable because the tail
of the chain made the whole declaration invalid. **No ruling needed. Option (a) stands as he ruled it.**

**Verified by measurement, not reasoning** (headless Chrome, real `theme.css`, the three states of his
control):

| state | before | after |
|---|---|---|
| control set to "Dubai" | Dubai | **Dubai — identical** |
| control set to "System" | the system stack | **identical** |
| unset (reset, or a universe that never chose) | the interface font | **Amiri → Cairo → Note font** |

**His condition — "make sure it doesn't affect the Style Setter control the way I designed it" — is
met:** whenever the control holds a value the fallback is never consulted, so every state his control
can produce renders exactly as before.

## §18 — PJ-461 CLOSED — Boss-passed on every stage
| stage | verdict |
|---|---|
| Stage 1 (5 steps, light, main window) | **PASSED 5/5** |
| Stage 2 Step 1 (Calendar cells in dark) | **PASSED**; the rest of Stage 2 retired — he never uses that theme |
| Stage 3 (the three second-screen lenses) | **PASSED**; he restored his Orrery setting |
| Stage 4 (Calendar typeface) | **RULED (a)**, then verified live: ECK unchanged (*"First: Pass"*), Eisa Universe now renders Amiri (*"good as a starting point. Consider it passed"*) |

**Final:** 17 names — **16 corrected, 1 kept by design**, `--bg-modifier-border` struck. 21 files,
`225 insertions / 98 deletions`. Zero `var()` consumers of any retired name left in `src/`.
Binary **2026-09-14 17:35:51**, proven to embed this build by its stylesheet filename, with the
repaired Calendar declaration present in the emitted CSS at both sites. `vitest` 1008 passing;
`svelte-check` 0 errors / 268 warnings — unchanged from the pre-edit baseline throughout.

**The safety inspection was claimed EXEMPT and the claim was re-earned** after the diff grew from 73
CSS lines to ~100 lines of added logic: every changed line sits inside a `<style>` block or inside
`livePreview.ts`'s single static theme object; zero `invoke(`/`emit(`/`listen(`/`updateSettings`/
`onMount`/`onDestroy` on either side of the whole diff; no added line declares a custom property; and
his three real `settings.json` files contain none of the retired names.

## §19 — PCS
Commit carries: 21 source files · ledger **v2.14** (PJ-461 closed, PJ-475…PJ-488 filed) · orientation
**v4.34** · `CLAUDE.md` (the new law) · both session logs · the panel verdict · the Boss test document ·
two renders · the MoCh · this handover. **SO#2 (help + User Manual): no change owed** — verified, no
user-facing document asserts anything this build makes untrue. Memory updated with the new law and
the theme ruling.

## §20 — Post-commit addendum from the findings-verifier, and the method point in it
Its background sweep returned after the close. Two things, both acted on.

**1. A claim strengthened, and re-verified by me.** No retired name is written via
`element.style.setProperty(...)` anywhere in `src/` or `static/` — so **no retired name is declared by
ANY mechanism, static or runtime.** Checked myself: the only `setProperty` hit matching that pattern
is `--font-monospace-theme`, a *replacement* name. The only occurrences of retired names anywhere in
the repo outside `src/` are in the stale `.claude/worktrees/*` checkouts — the very parallel trees
filed as PJ-479.

**2. The sharper point, and it is about me, not the code.** My claim that "none of the 15 retired names
is a Style Setter / style-settings registry key" was **already refuted by a record sitting in my own
working tree**: `PJ-461-PANEL-VERDICT-2026-09-13.md:201` had filed the orphaned `--sidebar-width`
control, naming the same registry line, the same zero-caller generator and the same `CORE_BLOCK_IDS`
strip. Ledger v2.13:4075 carried it too. **This is the same failure shape as the law written today** —
a claim drafted against a record that already contained the answer — and had it been confirmed, a PJ
the panel correctly opened would have been lost.

**Disposition:** the substance already survived into ledger v2.14 inside PJ-482 (the dead parallel
registry), but it was folded into prose. It is now a **named sub-item** of PJ-482 so that closing that
PJ cannot quietly drop it. The verifier's own recommendation — *"the sentence needs the exception
written in, not removed"* — is honoured: `--sidebar-width` IS a registry key, it is safe to retire
because the generator has no callers, and the orphaned control remains filed.

---

## §21 — PJ-466 opened: the panel ruled, and the ledger's own entry was wrong in two ways

**Function in hand:** the Mold Repair Door's end-of-run screen (`MoldRepairDialog.svelte`) — the
receipt a run gives of itself. Ledger v2.14 has PJ-466 as the ► NEXT ACTION.

**Nothing was written to the app.** This section records reads, one read-only design panel
(`wf_6d74a307-646`, 13 agents) and one `findings-verifier` pass. Full verdict:
`lab/reports/PJ-466-PANEL-VERDICT-2026-09-14.md`.

### What the ledger's entry got wrong
1. **"three exits" is FOUR.** `MoldRepairDialog.svelte:173` (delicate halt), `:182` (between-batches
   blocker), **`:194` (normal completion)** and `:198` (the catch). The omitted one is where the Boss
   has **already** seen a genuine write failure worded "skipped" — the rehearsal's Recovery 1,
   `SESSION-LOG-2026-09-12.md:444`: *"3 fixed, 1 skipped."* over a `✕ … ReplaceFileW error 5` row. A
   fix scoped to the ledger's three would have left a confirmed-wrong screen shipping. There is also
   a **fifth honesty exit** nobody had scoped: a thrown `scan_stamped_molds` (`:99-106`) dismisses
   the door in silence and `+layout.svelte:624`'s `catch { moldRepairCount = 0; }` retracts the
   banner — a scan that could not run is reported as a universe with zero molds.
2. **"his question" was not open, and asking it would have broken the law written today.** The entry
   carries *"should a stale-file refusal halt the cascade at all?"* as a question for the Boss. The
   app already answers it: `phantom_prune.rs:590-592` defines that exact class — *"the file
   reappeared, or the row no longer classified as a phantom. **Never an error.**"* — and the door's
   own header (`MoldRepairDialog.svelte:25-26`) says the halt exists *"so a systematic problem never
   reaches the ordinary files"*, which an already-fixed file is not. It is a coding defect, not a
   design choice. **Withdrawn.** The sibling question ("should a genuine write failure still halt?")
   is likewise answered: `mig108.rs:1044-1046` — a consented, journaled, batched file operation
   halts on the first genuine failure.

### The finding that set the scope — the house already ships this receipt
Three of the four lenses independently proposed **inventing** an outcome taxonomy while telling the
panel no precedent existed. An attacker found one, and I verified it myself:
`phantom_prune.rs:585-604` `PruneReceipt { removed, skipped, failed, unknown, stopped_early,
refused }`, whose doc comments draw exactly the line this job needs. It is **rendered** at
`SettingsModal.svelte:2753-2790` — neutral title ("Last removal"), one sentence per non-zero class, a
refusal replacing the lot, every count through `$tn('plurals.entries', n)` — **in the same Settings
page as the mold door's own entry point**. My own check found it is not alone: `repairReport`
(`:2791+`) is a second shipped receipt with the same grammar and its own `stoppedEarly` line. The
chair ruled for the shipped shape and against all three inventions. Under the Whole-Ecosystem Fix
Law, a thirteenth vocabulary beside a working twelfth is the drift the law exists to prevent.

### The root cause, stated once
`MoldRepairOutcome` carries only `ok: bool` and free text (`mold_repair.rs:56-62`). There is no
sentence in any language that separates *"left alone, nothing owed"* from *"still broken"*. That one
missing field is why a benign refusal halts the run, why PJ-469's refusal causes a needless write,
why `relinked_sources` over-counts, and why the receipt speaks English in fifteen languages. **One
field, four ledger entries.**

### Two checks disagreed, and reading settled it
My register claimed `report` can be `null` at the summary via the between-batches blocker with zero
delicate files. The `findings-verifier` **confirmed** it; the panel **refuted** it. I read the code:
the only `await` between the blocker check at `:155-156` and the one at `:179` sits **inside**
`if (delicate.length > 0)`, so with no delicate files the two checks cannot disagree and that exit is
unreachable. **The panel is right; my verifier was wrong.** `report === null` reaches the summary by
exactly one path — the catch at `:198`. And the real Exit-2 screen is *worse* than the empty one I
imagined: a success headline with a green count over a run whose second half never happened.

The verifier earned its keep elsewhere, correcting three of my claims: there are **ten** non-success
sites in `repair_one`, not nine (`:507` "Could not read it" was in nobody's taxonomy); `:537` leaves
the file **changed** on disk while reporting `ok:false`, so "skipped" is false about a file that was
written; and `:540`'s rollback is `let _ = std::fs::write(...)` while `:541` asserts *"so it was
undone"* regardless. The panel added that `:517` is provably **unreachable** and that
`mold_repair.rs:443-447` is dead code.

### Where it now stands
Scope, build plan (6 steps), the full proposed copy, the risks and the dissent are in the panel
verdict. The job is **a small `/migration`** by CLAUDE.md's own boundary test — it changes the IPC
report shape, Rust ↔ Svelte — and it closes **PJ-466 + PJ-469 + PJ-470 + PJ-471** together, for about
24 new keys and one new plural noun across 15 locales.

**Two questions go to the Boss, and only two** — the copy (his vocabulary, by standing ruling) and
the priority + the WA#6 filing of two defects found on other doors (the importer's green tick over a
failed import; a progress strip that can never say it failed). Eleven other candidate questions were
answered by the app and are recorded in the verdict's *Already decided* section rather than spent on
his time.

**Orientation:** NOT bumped in this commit. No SO#6 trigger has fired — no migration has opened, no
PJ has closed, no rule changed. It bumps when PJ-466 closes. Stated so the skip is not silent.
