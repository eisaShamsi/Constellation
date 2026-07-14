# PJ-106 — Arabic/RTL Typing & Navigation /migration — Phase 1 ARCHITECT

**Date:** 2026-07-14 · **Workflow:** `wf_cb144677-254` (2 territory mappers + WA#5 research + architect) · **Status:** Boss ruled 2026-07-14 — **Option B (full fix: direction + selection), built A-first**, and **LOGICAL (Word/Windows-style) arrow keys**. → Phase 2 Plan.
**Boss symptoms:** `lab/reports/PJ-106-RTL-Symptoms-BossReported.md` (Rounds 1+2).

**Root cause (one line):** the bidiPlugin renders per-line `dir` DOM attributes but `EditorView.perLineTextDirection` was never enabled, so CM6's caret/selection MOTION engine computes against one whole-editor direction — Home/End/arrows/word-hops disagree with the screen on every mixed line.

---

## Architect deliverable

PJ-106 — PHASE-1 ARCHITECT: Arabic/RTL Typing & Navigation Overhaul

**Function in hand:** the editor's caret/selection engine for RTL and bilingual notes (NotePane + FocusPane).

## 1. Territory (5 lines)

1. We render per-line direction (bidiPlugin DOM `dir` attributes + CSS) but never tell CodeMirror's motion engine — so Home/End, arrows, word hops, and vertical motion all compute against ONE whole-editor direction that disagrees with what the screen shows on every mixed-direction line.
2. That one base direction is itself unstable: hard-coded `dir='auto'` on the content element resolves from the first strong character of the *rendered viewport*, so it can flip while scrolling and ignores our own detectDir heuristic.
3. Paragraph navigation, paragraph selection, and sentence selection do not exist anywhere in CM6 or our code — genuinely missing features, not bidi bugs.
4. Installed CM6 (view 6.39.16, commands 6.10.2) already contains every needed API — `perLineTextDirection`, `bidiIsolatedRanges`, `visualLineSide`, logical-motion variants — verified in node_modules; grep confirms zero uses in src. No upgrade is required for the fix; a hygiene minor bump (6.41.0 / 6.10.3) adds wrapped-line and click-precision fixes.
5. Prior art converges: Marijn (CM author) prescribes exactly "per-line dir decorations + the facet"; Joplin shipped it; Obsidian renders identically. We built half the recipe.

## 2. Option table

| | **A — Official machinery** | **B — A + missing commands** | **C — Keybinding patch only** |
|---|---|---|---|
| What | Enable per-line direction facet; register isolate ranges (incl. callouts); deterministic per-note base direction (kill `dir='auto'` competition); same-transaction dir update on the typed line; symmetric empty-line inheritance; land in shared `$lib/editor/` for both panes; hygiene CM6 minor bump | Everything in A, plus: paragraph next/prev (Ctrl+Up/Down), select-paragraph, select-sentence (Intl.Segmenter, Arabic terminators), per-note direction override (frontmatter/toggle, Windows Ctrl+Shift convention) | Remap Home/End to visual-line-side commands; no facet, no base-dir fix |
| Speed | ~2 sessions | ~3–4 sessions | ~1 session |
| Effort | M | L | S |
| Risk | Perf (facet reads per-line computed style — must measure on 7,600-note Universe); extension-array change rides the save path → full Editor-Surface Gate | A's risks + keybinding conflicts, 15-locale i18n for any UI, Segmenter behavior in WebView2 to spot-check | **Symptom patch** — arrows, selection, word hops, click placement stay broken; burns an LL-014 strike; violates Solve-the-Class |
| Boss defects fixed | Home/End, Shift+Home/End line selection, arrow walks, word select/hop, bilingual chaos, typing-flip lag, likely the deferred callout caret bug | **All reported defects**, including "navigate/select a paragraph, sentence, page" | Home/End partially, at best |
| Left broken | Paragraph/sentence navigation & selection (missing features) | Nothing reported | Nearly everything |

**Boss ruling needed (either option):** arrow-key convention — CM6/Chrome move the caret *visually* (left arrow = moves left); Word on Windows moves *logically* (jumps at Arabic/Latin boundaries). CM6 6.7.0 makes a Word-style setting nearly free. Your call, not mine.

## 3. Invariants

1. **Typing latency (Rule 1):** all bidi work stays per-visible-range; measured before/after on the 7,600+ note Universe; zero perceptible lag or it doesn't ship.
2. **Editor Parity:** fix lands in the shared `$lib/editor/` set; NotePane AND FocusPane both inherit it; FocusPane stays parser-free (these facets are not parsers).
3. **Pure-LTR notes:** zero behavior or performance regression for English-only notes.
4. **`{#key}` lifecycle untouched:** no changes to save/load/teardown; additions are navigation-only extensions.
5. **UI chrome unaffected:** toolbar/panel RTL behavior unchanged; each of the four existing direction heuristics re-pointed deliberately (Predecessor Lookup), never silently.
6. **Content bytes never change from navigation:** motion/selection are read-only; the harness asserts screen === disk after every recipe.

## 4. Migration path / rollback

- **Phase 0 (Reproduce-First, mandatory):** named runtime reproduction recipes in the running app — headless DOM testing false-passes on this bug class (coordinate probes silently fall back to correct behavior). No fix is designed before the defects fire on demand.
- New extensions live in a **compartment behind a settings flag**: rollback = one-line reconfigure to empty, instantly, without touching the save path.
- **CM6 minor bump** is a separate lockfile-pinned commit, revertible independently; blast radius = every editor surface incl. the merge view — covered by the Gate checklist.
- detectDir keeps its never-empty return signature so the existing fallback wiring doesn't silently change meaning.
- Riskiest single edit is the base-direction line pair in NotePane; it gets all 8 Editor-Surface Gate checks plus second-screen verification.
- Deferred Arabic-callout caret bug is subsumed here: its isolate registration is part of A, still gated on Phase-0 reproduction (stop-patching ruling respected — this is structural, not CSS).

## 5. Recommendation

**Option B, built A-first:** land and Boss-validate A (the class fix, Marijn-prescribed and Joplin-proven), then add B's missing commands on top — because A without B still leaves "select a paragraph/sentence" impossible, and C is the kind of patch we've banned three times. Honest sentence: the traced mechanism matches your symptoms exactly, but until Phase 0 reproduces them live in the running app I will not promise the facet alone cures every one — that proof comes before any fix code, not after.

---

## WA#5 prior-art research brief

PJ-106 WA#5 PRIOR-ART BRIEF — RTL/bidi editing in mature systems vs CodeMirror 6's official machinery
(All installed-version claims verified directly against `E:\مشاريع كلاود\Constellation\node_modules` — view 6.39.16, commands 6.10.2, language 6.12.2.)

---

## 1. What CM6 officially provides — and what we get FREE at our installed versions

**The official recipe for per-line RTL exists, and Constellation has built exactly half of it.** Marijn Haverbeke (CM author), asked for a perLineTextDirection example, answered: *"you either compute line directions yourself and explicitly set them per visible line, or just add `dir=auto` to every line (unfortunately doesn't seem to be possible with just CSS, so you need a plugin to add the DOM attributes)"* — i.e. a ViewPlugin adding `Decoration.line` dir attributes **plus** `EditorView.perLineTextDirection.of(true)`. Constellation's bidiPlugin is the first half (the DOM attributes); the facet — the half that tells the *motion engine* — is missing. Source: https://discuss.codemirror.net/t/is-there-an-example-or-perlinetextdirection/5245

Free at installed versions (verified in the installed `.d.ts`):

| API | In our version? | What it does |
|---|---|---|
| `EditorView.perLineTextDirection` (facet, since v6.0.0) | YES (view .d.ts:1257) | "make it read the text direction of every (rendered) line separately" — re-points `textDirectionAt`, and through it Home/End (`moveToLineBoundary`), visual arrows (`bidiSpans`/`moveVisually`), word hops, and vertical motion, to the line's own direction |
| `EditorView.textDirectionAt(pos)` (since 6.0.0) | YES (:1083) | per-line direction "as assigned by CSS… Note that this may trigger a DOM layout" — the Perf-Rule-1 cost to measure |
| `EditorView.bidiIsolatedRanges` (facet, 6.17.0; auto-direction support 6.23.0) | YES (:1357) | decorations that add `unicode-bidi: isolate` "should also include a `bidiIsolate` property… and be exposed through this facet, so that the editor can compute the proper text order"; `bidiIsolate: null` = dir=auto, first-strong resolved (:97-100) |
| `bidiIsolates()` extension in @codemirror/language | YES (language .d.ts:1210) | auto-isolates syntax-tree nodes marked isolating (markdown links/code), with a built-in optimization (only active on lines containing RTL chars) |
| `EditorView.visualLineSide(line, end)` (6.23.0) | YES (:961) | "cursor position visually at the start or end of a line… may differ from the logical position… if text at the start or end goes against the line's base text direction" — the exact tool for correct Home/End on mixed lines |
| `cursorLineBoundaryLeft/Right`, `selectLineBoundaryLeft/Right` (commands 6.1.0) | YES | *directional* (visual) line-boundary motion — free building blocks if we remap Home/End |
| `cursorCharForwardLogical/BackwardLogical` + select variants (commands 6.7.0) | YES (commands .d.ts:164-292) | "logical (non-text-direction-aware) string index order" — a ready-made Word-style logical-arrow mode if the Boss wants it |
| bidi-level preservation in shift-selection (commands 6.1.3) | YES | prevents shift-select getting "stuck in some kinds of bidirectional text" |

Official bidi example (https://codemirror.net/examples/bidi/) confirms the policy: arrows are **visual** ("if you press the left arrow your cursor should move left, regardless of the direction of the text"), Backspace/Delete are **logical**, custom isolate decorations MUST be registered with `bidiIsolatedRanges` "so cursor motion respects the boundaries", and extensions should consult `textDirectionAt` — not `textDirection` — for local direction.

**Version-gap analysis (installed 6.39.16 vs latest 6.41.0, released 2026-04-01):**
- The big RTL click/coords overhaul — "Properly handle bidirectional text in `posAtCoords`" — landed in **view 6.39.0** (2025-12-08), preceded by 6.38.8's "computing a document position from screen coordinates would sometimes go wrong in right-to-left text". **We already have both.** This was the fix arc for the "clicking at end of line in RTL places cursor at wrong location in Chrome" report (https://discuss.codemirror.net/t/clicking-at-end-of-line-in-rtl-text-places-cursor-at-wrong-location-in-chrome/9571). Changelog: https://github.com/codemirror/view/blob/main/CHANGELOG.md
- Post-6.39.16 fixes worth taking as hygiene (not bidi-labeled but relevant to a markdown editor): 6.40.0 "`moveVertically` could move to the wrong place in wrapped lines with a large line height" + shift-mouse selection associativity; 6.41.0 "`posAtCoords` could incorrectly return a position near a higher element on the line, in mixed-font-size lines" (directly relevant to livePreview headings). commands 6.10.2→6.10.3: "selection-extending commands preserve the associativity of the selection head" — relevant to Shift+Home/End at bidi boundaries. **Recommend the minor upgrade as part of the fix, but it is NOT the fix** — the core defect (missing facet) is version-independent since 6.0.0.
- Caveat noted from the fetched GitHub page: codemirror/dev's issue repo is reported archived/read-only as of 2026-04-15 (https://github.com/codemirror/dev/issues/898 — the old Ctrl+Arrow RTL bug, closed); significance unverified, releases continued through 6.41.0.
- Historical confirmation this bug class is real and known: CM5 issues #6531 (End key jumps wrong when RTL line ends in an English word — the Boss's exact symptom), #6117 (start/end of line broken in RTL+numbers), #4878 (visual motion broken in mixed RTL/LTR). https://github.com/codemirror/codemirror5/issues/6531 · /6117 · /4878

## 2. Obsidian (the closest prior art — same product class, same editor)

- **Native since 1.6 (June 2024):** official dev doc — in the editor, *"the `dir` attribute is set to `rtl` or `ltr` per line on `.cm-line` elements by detecting the first strongly directional character"*; **a line with no strong character inherits the PREVIOUS line's direction** (symmetric — unlike Constellation's RTL-only inheritance); reading mode uses `dir="auto"` per block; UI mirrors via `.mod-rtl`; logical CSS properties (`margin-inline-start` etc.) mandated. https://docs.obsidian.md/Plugins/User+interface/Right-to-left · https://obsidian.md/changelog/2024-06-07-desktop-v1.6.2/ — Whether Obsidian also enables the `perLineTextDirection` facet internally is not publicly documented — unconfirmed; the rendering model matches Marijn's recipe exactly.
- **obsidian-rtl plugin (esm7):** adds what native lacks — per-note direction override (LTR/RTL/Auto) remembered per file, frontmatter `direction:` key, vault default, status-bar toggle + hotkey command. Its own changelog admits the Home/End story: it "improved the handling of the Home & End keys (without Shift at this point) in RTL. It's still not perfect (**CodeMirror is not good at this**) but at least the basic functionality works." https://github.com/esm7/obsidian-rtl
- **obsidian-native-rtl (Bip901):** per-line direction forced via **Left-Ctrl+Shift / Right-Ctrl+Shift — the classic Windows per-paragraph direction convention** the Boss's muscle memory knows from Word/Notepad. https://github.com/Bip901/obsidian-native-rtl
- **Joplin PR #10810 — the confirmed shipped example of the full recipe:** enabled `perLineTextDirection.of(true)` AND `dir=auto` on every visible line via decorations, in a CM6 markdown editor, with paste-large-text perf testing. Residual issues were markdown-decoration-specific (checkbox alignment, list-marker side), not motion. https://github.com/laurent22/joplin/pull/10810

## 3. VS Code / Monaco — the anti-pattern

VS Code has **no real RTL support**: #86667 (open umbrella issue for years), Monaco #4809/#4869 (editor breaks when embedded in an RTL page), IBM's Bidi Lab assessed Monaco as deficient; a 2025/26 PR (#255455) only adds decoration-based RTL line *marking*. Lesson: a code editor punting on RTL is not our peer; note-taking editors (Obsidian, Joplin) are — and both converged on the same per-line first-strong pattern. https://github.com/microsoft/vscode/issues/86667 · https://github.com/microsoft/monaco-editor/issues/4809 · https://github.com/microsoft/vscode/pull/255455

## 4. Unicode BiDi essentials → UX policy decisions

- **Logical vs visual order:** Unicode stores text in logical (typing/phonetic) order; display order comes from UAX#9. Paragraph direction by **first-strong character**: "If the first strong character is RTL, reading order is right to left and text is right aligned." (Microsoft Learn: https://learn.microsoft.com/en-us/globalization/fonts-layout/text-directionality · UAX#9: https://www.unicode.org/reports/tr9/)
- **Home/End are direction-agnostic in every mature system:** Home = logical line start = the READING start = the **visual RIGHT edge** on an RTL line; End = reading end = visual left edge. Word and browsers agree on this; they differ only on **arrows**. CM6's `moveToLineBoundary`/`visualLineSide` implement exactly this — *when the line's direction is known correctly*. Our defect is that the direction it consults is the whole-editor one.
- **Arrows — the one genuine convention split:** CM6/Chrome move the caret **visually** ("the arrows… match the direction that the cursor actually moves" — Marijn's design blog, which explicitly contrasts Windows: many programs "move the cursor 'logically'… causing it to move in the opposite direction from the arrow when in right-to-left text"; at direction jumps CM shows a secondary cursor, following Chrome). Word's Windows default is **logical**, user-switchable at File→Options→Advanced — and Microsoft's own i18n veteran Michael Kaplan mocked the logical default ("ironic to call it 'Logical' if no one really knows what to do with it"). **Boss ruling needed:** keep CM's visual default (= what every browser/WebView2 text field on his machine does) or expose a "Word-style logical arrows" setting — commands 6.7.0's `*Logical` commands make the latter nearly free. https://marijnhaverbeke.nl/blog/cursor-in-bidi-text.html · http://archives.miloush.net/michkap/archive/2012/10/09/10357645.html
- **Missing selection granularity is not a bidi problem:** no surveyed system gets paragraph/sentence selection from CM6 free; Word's Ctrl+click=sentence convention and `Intl.Segmenter` (sentence granularity, Arabic terminators ؟ ! ؛) are the standard building blocks for the new commands.

## 5. The proven patterns the fix should be built on

1. **Close the render/motion split with the official facet (Marijn-prescribed, Joplin-shipped, Obsidian-rendering-compatible):** keep bidiPlugin's per-line `dir` decorations as the render layer, add `EditorView.perLineTextDirection.of(true)` in the SHARED `$lib/editor/` set (both panes), and make the touched line's dir decoration update synchronously in the same transaction (Obsidian's model: first-strong per line, previous-line inheritance **both ways** — fix our RTL-only asymmetry to match). Perf gate: `textDirectionAt` "may trigger a DOM layout" — it reads only rendered (viewport) lines, and Joplin shipped it in a notes editor, but Perf Rule 1 requires measuring on the 7,600-note Universe before commit.
2. **Register every isolating/replacing inline decoration with `bidiIsolatedRanges`** (spec property `bidiIsolate`, `null` = dir=auto first-strong) and/or enable `@codemirror/language`'s `bidiIsolates()` for syntax-marked nodes — the official example is explicit that isolate styling without facet registration leaves cursor motion computing the wrong order. This is the leading structural suspect for the deferred Arabic-callout Home/End bug (calloutPlugin adds its own dir attrs + `Decoration.replace` ranges with zero isolate registration) — still subject to Reproduce-First.
3. **Deterministic base direction per note, never viewport-dependent `dir=auto` on the content element** (the obsidian-rtl model): resolve the note's base direction once (auto-detect default + per-note override via frontmatter/toggle, Ctrl+Shift-style switch as an optional Windows-convention nicety), set it explicitly on the editor, and let per-line attrs + the facet handle mixing. This removes the competing-signals defect (NotePane's hard-coded `contentAttributes dir:'auto'` vs the detectDir prop) and the scroll-flip hazard.
4. *(Hygiene, optional but cheap)* **Upgrade view 6.39.16→6.41.0 and commands 6.10.2→6.10.3** for the wrapped-line vertical-motion, mixed-font-size click, and selection-associativity fixes; the major RTL `posAtCoords` overhaul (6.38.8/6.39.0) is already in our installed version.

Sources: https://discuss.codemirror.net/t/is-there-an-example-or-perlinetextdirection/5245 · https://codemirror.net/examples/bidi/ · https://github.com/codemirror/view/blob/main/CHANGELOG.md · https://github.com/codemirror/commands/blob/main/CHANGELOG.md · https://github.com/laurent22/joplin/pull/10810 · https://docs.obsidian.md/Plugins/User+interface/Right-to-left · https://github.com/esm7/obsidian-rtl · https://github.com/Bip901/obsidian-native-rtl · https://obsidian.md/changelog/2024-06-07-desktop-v1.6.2/ · https://github.com/microsoft/vscode/issues/86667 · https://github.com/microsoft/monaco-editor/issues/4809 · https://github.com/microsoft/vscode/pull/255455 · https://marijnhaverbeke.nl/blog/cursor-in-bidi-text.html · http://archives.miloush.net/michkap/archive/2012/10/09/10357645.html · https://learn.microsoft.com/en-us/globalization/fonts-layout/text-directionality · https://discuss.codemirror.net/t/clicking-at-end-of-line-in-rtl-text-places-cursor-at-wrong-location-in-chrome/9571 · https://github.com/codemirror/codemirror5/issues/6531 · https://github.com/codemirror/codemirror5/issues/6117 · https://github.com/codemirror/dev/issues/898 · local verification: E:\مشاريع كلاود\Constellation\node_modules\@codemirror\view\dist\index.d.ts (:961, :1083, :1257, :1357, :97-100), \@codemirror\commands\dist\index.d.ts (:164-292), \@codemirror\language\dist\index.d.ts (:1210), \@codemirror\view\package.json (6.39.16)