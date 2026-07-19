# Session Log — 2026-07-18 (session began 2026-07-17 evening)

**Headline:** PJ-114 started as "add a right-click menu to Focus mode" and became something much bigger — **NotePane was elevated to a top-principal as the GATE to Knowledge Cognition**, audited, and given a vision + phased roadmap with an approved Phase-1 migration now underway. Along the way: 3 user-facing fixes, 2,550 lines of dead code removed, and one hard process lesson re-learned.

---

## Commits (11, all Boss-validated before commit)

| Commit | What |
|---|---|
| `9f1b57cb` | PJ-114 FM+ design brief + migration Plan (Boss-approved direction) |
| `455a7930` | **§0.1 / PJ-116** — a title typed in Focus now renames the note (was silently discarded); also landed the parked sweep-#3 cascade-freeze protection |
| `2772e2be` | **§0.2** — shared parser-free wikilink finder `findWikilinkAtLineOffset` + 11 unit tests |
| `22a9de41` | **§1.1** — `FM_PLUS_ENABLED` build flag + persisted `focusModePlus` setting (default off) |
| `11ace3fe` | **§1.2** — FM+ footer toggle: localized mark ×15, tooltip-above, RTL, fade-on-type |
| `8ee48a86` | ⋯ note-menu **RTL truncation** fix (fixed-position + measure/clamp) |
| `d410f1ea` | **NotePane Cognitive Gate** — capability audit, vision + roadmap, Phase-1 Architect + Plan |
| `e88cafb8` | **§1** — deleted **2,550 lines** of unmounted editor components |
| `ca6ce954` | **Caret continuity** across the NotePane ↔ Focus switch |
| `cc35524d` | ⋯ menu — anchor by the **note's** direction (opens into the note, never over the sidebar) |
| `6c810836` | **§3a** — read-widening: a link's `created` now reaches the UI (Rust → TS → row maps) |

---

## The arc

**1. FM+ (Focus Mode Plus) — shipped the foundation, then deliberately paused the menu.**
Boss directed a right-click menu for Focus mode. Concept locked: FM+ = an opt-in switch (footer token, single master, default off) that unlocks extra Focus affordances *without* Focus becoming "another NotePane" (Boss's guardrail). Design pass (Art Director & Team, `wf_12bd4169-78c`) → Architect → Plan → build. §0.1/§0.2/§1.1/§1.2 shipped and validated.

Then **v1 of the menu was Boss-REJECTED**: Copy-link-text/Copy-path + Cut/Copy/Paste is strictly *poorer* than the native webview menu. Boss: *"If we only have these limited choices while there is a richer native one, then we don't need it. Do some digging and give me more smart choices."* A research dig (`wf_b5c67f60-646`) reframed it: **the native menu owns the clipboard; FM+ owns knowledge formulation.** Then the decisive Boss ruling: *"it is the NotePane that should have the full living-link, so we are not going to replicate it in the FM"* → **audit NotePane first, cross-check FM+ after.** FM+ menu code was reverted; the shipped toggle/flag/finder stay.

**2. NotePane elevated to a top-principal.**
> *"NotePane is the GATE to the knowledge cognition, it is the key to one's knowledge. If Constellation is going to be a unique, powerful, and smart PKM/PKF system, it will be through the NotePane capabilities. It is the starting point of one's cognitive journey, so we have to make sure it is well-equipped and well-instrumented."* — Eisa, 2026-07-18

**The audit (`wf_baf417ca-22a`) verdict:** NotePane is a competent *Markdown* editor with rich *diagnostic* panels, but **not a living-link editor**. Of the 8 living-link properties, **only 1 (Confidence) is truly settable** on the note surface; Type + Annotation are authorable only via raw `[[type::Target|reason]]` syntax; right-clicking a real `[[link]]` offers only open/copy/edit/remove. **~90% of the gap is wiring engines that already exist.**

**The vision (`wf_cfffaf11-0b3`)** organizes NotePane's instrumentation by the Five Acts and phases the build-out (Phase 0 housekeeping → Phase 1 own the living link → Phase 2 tension first-class → Phase 3 co-visibility → Phase 4 discovery → Phase 5 distillation/synthesis).

**Boss rulings:** Phase 1 = "stretch it" (the four picks + housekeeping + `supersedes` + confidence badge); controls at **three depths**; tension **early** (Phase 2); at-a-glance density **minimal by default, richer by choice via Settings** *(a better answer than either option offered)*; delete the dead components; **one continuous migration**.

**3. Phase-1 Architect + Plan — which overturned assumptions and found live bugs.**
- **The documented dual-layer link storage does not exist.** No LINK file is ever written. For Type + Annotation the **body text is the source of truth**; `note_links` is a derived index rebuilt on every save. A DB-only `setLinkAnnotation` would be silently reverted → item 1.2 was the wrong shape. Boss ruled: **body-text is the write** (File-Over-App).
- **Link identity = `(source_path, target_name, link_type)`** (UNIQUE). No occurrence index — repeated identical tokens collapse; **the 2nd token's annotation is already silently dropped today**.
- **LIVE data-loss bug:** the indexer's preserve gate ignores `confidence`, and the re-INSERT hardcodes `'hypothesis'`, `created = now`, `status = 'active'` → editing an annotation **wipes a user-set confidence**, **resets `created`**, and **resurrects archived links** (so `archiveLink` doesn't survive a save — breaking `supersedes` at its root). Blocking prerequisite, scheduled as §5.
- **"Remove link" strips EVERY link on the line** (5 first-match-on-the-line regexes); right-clicking the 2nd link acts on the 1st. Fixed as part of §7.
- **Backlinks rows cannot be a write depth** — their `sourcePath` is another, usually closed note; rewriting there is the forbidden disk read-modify-write. Write depths = editor right-click · Outgoing rows · inspector. *(Adjusts the Boss's "three depths" ruling.)*

**4. Build progress:** §1 (delete 4 unmounted components, 2,550 lines) ✅ · §2 (align-buttons reproduce) — buttons *work*, but a different defect surfaced (filed) · §3a (read-widening) ✅.

---

## Fixes shipped for Boss-reported issues

- **⋯ menu RTL truncation** — clipped by `.e-desk`'s `overflow-x: hidden`; made fixed-position with measure-and-clamp. Then a **second** report (Latin UI + Arabic note) disproved the viewport-overflow rule: with the sidebar open the menu spilled over the file tree. Correct rule = **anchor by the NOTE's direction, always open into the note**. Verified across all four UI×note-direction combinations.
- **Caret continuity** NotePane ↔ Focus (see the process lesson below).

---

## PROCESS LESSON — Reproduce-First, re-learned the hard way

I shipped **two** caret fixes built on *plausible mechanisms* (first "onDestroy runs in time," then "it's an ordering race") without ever observing the failure. Both failed live; the Boss caught both and finally said **"Enough guessing."**

The rule I broke is a project top-principal: *a plausible mechanism is not a diagnosis; if the bug cannot be reproduced, the ONLY shippable work is the instrumentation that makes it reproducible.* Because release builds disable devtools, the instrumentation had to be **on-screen** — a temporary trace panel. **One** run produced the answer: the hand-off worked perfectly (`NP APPLIED @38`) and was then **overwritten** by the app's own per-tab cursor memory (`tab.cursorPos` → `initialCursorPos`, `NotePane.svelte:848-851`). I had built a **second, competing cursor memory**. The fix deleted mine and fed Focus's cursor into the existing one. Instrumentation removed after confirmation.

**Cost:** two wasted Boss test cycles. **Takeaway:** go to instrumentation *first*, not after the Boss stops you.

Also logged: **§0.2's "proven against the main editor" claim was false** — it refactored `CodeMirrorEditor.svelte`, which nothing mounts. The helper is sound (11 tests, used by FocusPane), but the validation claim wasn't. Caught by §1's dead-code discovery.

---

## New jobs filed (for the PJ ledger)

1. **Markdown tables render as raw pipes** — live preview has *no* table renderer (only the Bases/Lens one). Boss wants real tables. Own concept paper + migration.
2. **Text-align inserts raw HTML** — `<div style="text-align: center">…</div>` visible on the active line, renders only after the cursor leaves, line hard to re-click. Same family as (1).
3. **`BacklinksPanel` "Link it" clobber risk** — raw `readNote` → `write_note` on a possibly-open-dirty note, wrapped in `catch {}`. Same class as the `b6310479`/`baae4533` sweeps. Out of scope for this migration; fix separately.
4. **FM+ menu (PJ-114 §1.3+)** — paused pending the NotePane cross-check.

---

## State at close

- **On `main`, pushed:** everything through `cc35524d`. `6c810836` (§3a) committed — push at session end.
- **Working tree:** clean apart from the runtime `.claude/scheduled_tasks.lock` deletion (leave it).
- **Next:** Phase-1 **§3b** — extract the shared *localized* traversal helper (`fmtTraversed` is byte-identical in both panels **and** hardcoded English; the chip tooltip is a hardcoded English template). ~8 keys ×15 locales. Then §4 (confidence badge + density setting, folding in the `LinkStateChips` extraction), §5 (the blocking indexer fix), §6–§10.
- **Help/manual:** deliberately NOT updated this session — the only new user-facing surface (the FM+ toggle) is default-off and its feature set is paused pending the cross-check. Documenting it now would document something incomplete. **Due when FM+ resumes.**

---

# §3b — the living-link state chip speaks the user's language (AWAITING BOSS TEST)

*Function in hand: the `×N` traversal chip beside the type pills on Backlinks / Outgoing Links rows — and, as it turned out, inside NotePane itself.*

**Concept (the horse):** a link's accumulated traversal is evidence of how much a connection has actually carried thought. The chip is where the user reads that evidence — so it must speak the user's language, from one place, and never leak an internal token.

## What changed

New `src/lib/links/linkDisplay.ts` — one source of truth for how a living link's *state* is put into words. Both link panels and the editor's inline chip now read from it. Net **-35 lines** in the two panels while *adding* localization.

## The design deviation worth recording: ZERO new i18n keys

The plan estimated "~8 keys ×15 locales." The verification pass found the vocabulary **already existed, fully translated in all 15 locales, and already on screen**:

- `plurals.walks` — the CLDR plural-aware traversal count. "Walk" is already this app's user-facing word for a traversal (`CCSView.svelte:268/281` builds a tooltip of this exact shape).
- `ccs.tier.*` — fresh / emerging / established / loadBearing / stale.

Minting `linkState.tier.*` + `plurals.traversedTimes` would have created a **second translation of the same two concepts across 15 files** — guaranteed drift, with the CCS panel calling a traversal "عبور" while the chip called it something else for the identical event. Verified programmatically that both namespaces are complete in all 15 locales before relying on them.

Relative time uses `Intl.RelativeTimeFormat` rather than hand-written strings — the same house precedent as `Intl.PluralRules` in the i18n core (WA#5).

## Measured, not guessed

Ran the full 15-locale × bucket matrix in node before choosing:

- **`numberingSystem` must go in the locale TAG (`-u-nu-latn`), not the options bag** — the options form is a **TS2353 error** under this repo's `strict` (verified against the repo's own tsc) even though it works at runtime. It matters because **`fa` defaults to `arabext`** (۳ روز پیش) while the app interpolates counts with plain `String(count)` — one tooltip would have mixed ۳ and 3.
- **`numeric:'auto'` is right for days, wrong for months/years.** At magnitude 1 it switches from elapsed duration to a *calendar claim*: "last month" for 44 elapsed days is a different and false statement, and ru/ja emit a prepositional phrase ("в прошлом месяце") rather than a label.
- **Bucket boundary = `LINK_STALE_DAYS` (90), not 30.** Days up to the stale threshold, then months, then years. The boundary carries meaning (inside the living window → day resolution) **and** steps over the only remaining CLDR artifact in the whole matrix: Hebrew months 1–2 render as "לפני חודש (1)". At months 3+ every locale is clean.

## Three pre-existing defects fixed in-pass (WA#6)

1. **A third copy, inside NotePane itself.** Caught by grepping the *built bundle* for the old English string after the panels were clean — `livePreview.ts`'s `WikilinkTraversalChipWidget` carried the same hardcoded `'Traversed N times'`. The source grep had missed it (the literal is assembled by concatenation). Now shares `walkCountLabel`. It knows only the count (`linkTraversalMapField` is `Map<string, number>`), so it says *less* — it must never default a tier it was not told. `loc` added to `eq()` so a stale-language chip cannot survive a document language change.
2. **Custom link types rendered their raw slug in the annotation** while the pill on the *same row* rendered the user's label — the panels' private `typeName` was a two-branch resolver where `relLabelIn` (already shared by the three cockpit lenses and `LinkTypePill`) is three-branch. Now uses the existing shared resolver; no fourth copy minted.
3. **Future timestamps rendered "-1d ago."** `Math.floor` of a negative fraction rounds toward −∞. Reachable via clock skew across synced devices and hand-edited files. Clamped to "today" — without the clamp the *localized* form would have upgraded visible garbage into the confident falsehood **"tomorrow."**

**One review finding rejected on evidence:** a reviewer called for `.bl-tier-fresh` / `.ol-tier-fresh` CSS. Traced it out — the chip renders only when `count > 0`, `linkLifecycle` returns `fresh` only at `count === 0`, and the dedupe ranks `fresh` below `emerging`, so the state is unreachable; the base `.bl-traversal-chip` rule already styles it regardless. Adding a rule for an unreachable state is the filler Form-Aligns-To-Purpose forbids.

## Verification

`svelte-check` **0 errors** · full suite **495 tests / 36 files green** (55 new, first-run pass — every predicted localized string matched: ar `اليوم`, he `אתמול`, ja `14 日前`, ru `1 год назад`, ar `عبوران`/`ركيزة`) · frontend + release binary rebuilt (17:47) · **the old English template grep-verified GONE from the built bundle** — the check that found the third copy.

New tests lock: bucket boundaries, the future clamp, the unknown-tier key-path guard, Latin digits in every locale, the Hebrew-artifact-free assertion at every bucket, and **i18n parity across all 15 locales** — nothing in this repo enforces locale parity (no CI check, no lint rule, TS structural check deliberately disabled).

## Status

**Built, not committed — awaiting the Boss test** (standing order: Boss tests every build before commit).

## Separate matter needing a ruling

The per-build inspection ran **whole-app** rather than diff-scoped (the skill takes its `files` arg in a different form than passed). **No finding is in this diff.** It returned a full per-cycle sweep: **61 confirmed — 3 APP-KILLER, 17 HIGH, 5 LOW, 36 MED**, all pre-existing:

- `store.ts:3539` — `moveItem`'s folder-move tab repath drops the moved folder's own name, repointing open tabs one directory too high; later saves create a phantom note that absorbs every edit.
- `+layout.svelte:6356` — every cascade guard is built from a pre-walk snapshot while `reloadTabsFromDisk` force-adopts against live `openTabs`; a tab opened during the cascade window is outside all four guards but inside the force-adopt.
- `store.ts:2277` — the display-only second screen is born dirty from the shared write-ahead net and durably writes that stale snapshot on its next navigation.

Not folded into §3b. Needs a sequencing ruling — they are Group-1-class and PJ-114 Phase 1 has §5–§10 still ahead.

## §3b — correction round 1 (Boss test findings)

Boss tested the 17:47 binary and returned three findings. Binary rebuilt **18:40**.

**Finding 3 — editor chip stayed English under an Arabic UI. RULING, applied.**
I had shipped the editor's `×N` chip tooltip in the NOTE's language, reasoning from the §E.2 note-language principle that governs the typed-link *labels* it sits beside. The test note is titled "Collision Test", so `dominantLocale` resolved to English and the tooltip stayed English under an Arabic interface. Boss ruled it follows the **interface** — consistent with the standing order *"when the user switches language, EVERYTHING adapts."*

The distinction that survives: the typed-link **label** is authored vocabulary and keeps the note's language; the chip tooltip is a **diagnostic about** the link and follows the UI. Implemented via `get(locale)` in `buildDecorations` plus a `locale.subscribe` in `LivePreviewPlugin`'s constructor that dispatches the existing `linkVocabChanged` effect — mirroring the `subscribeLinkTypes` pattern already there, with the first (synchronous, mid-construction) store emission skipped to avoid a re-entrant dispatch, and `unsubLocale()` added to `destroy()`. Without that subscription a CM6 widget cannot observe the language store at all and the chip would hold its build-time language until the note was reopened.

**Findings 1 + 2 — one cause, and it was mine: the bidi isolates.**
Reported as "the tooltip should be right-aligned in the English UI" and "the tooltip is truncated within the box" (Arabic). Read together: in English the box was measured **wider than the text it painted**; in Arabic the RTL text ran past a box measured as if LTR. Single mechanism — I had wrapped the tooltip in FSI…PDI (U+2068/U+2069), so the box width was computed for a string containing two characters the paint pass does not draw.

Removed. They were never justified: the reviewers who proposed them stated they could not verify them in WebView2/WKWebView, and the module's own comment already established the hazard is **absent here** — `traversalTooltip` composes all three segments in ONE locale, so the string is directionally homogeneous with no internal reordering to isolate. A test now asserts the tooltip carries **no invisible bidi control characters** in en/ar/he/ja, so a well-meaning re-add fails CI rather than the Boss test.

**Reproduce-First — attempted, blocked, and NOT papered over.**
I did not want to fix 1+2 on reasoning alone, so I launched the 17:47 build against the Boss's *Eisa Cognitive Knowledge* universe under computer-control to hover the real chip in the real WebView2. The machine locked before I reached the note, and a lock screen is not something to work around. So the isolate removal ships as a **well-motivated hypothesis, not an observation** — stated as such to the Boss rather than claimed as verified. If the symptoms survive, the honest conclusion is that a native `title` gives no control over box width, alignment, or direction, and the fix is a custom tooltip element — a design decision for the Art Director, not an improvisation.

Also corrected: the Chrome-extension route for observing this was unavailable, and the first app launch (via `nohup` from Git Bash) did not survive the shell exiting — relaunched detached via `Start-Process`. The instance was stopped before the rebuild; nothing was edited in it.

**Verification after the correction:** `svelte-check` 0 errors · 495 tests / 36 files green · isolates grep-confirmed absent from the built bundle · frontend + release binary rebuilt (18:40). Still **not committed** — awaiting the re-test.

## §3b — correction round 2: Constellation now draws the chip's tooltip

Boss re-tested the 18:40 binary. Localization **passed** (image 3 showed `عبوران`). The two
remaining asks resolved the open question decisively:

> "I want the tooltip to shift left relative to the hand" · "enlarge the tooltip box to accommodate the text"

**Neither is reachable through a native `title`** — the browser owns its placement and no CSS
reaches its box. So the diagnosis from the previous round was right about the direction but
incomplete about the remedy: removing the bidi isolates was necessary (the box was measured
with two characters it never painted) but not sufficient. The tooltip had to become ours.

**This is the SECOND time this exact native-tooltip failure has been reported in this app.**
`StructuralOutlinePanel.svelte:125` carries a comment dated 2026-06-28: *"Bleeding-tip fix
(Boss): replace the native `title` (WebView2 renders it as a wide box that bleeds off the panel
edge) with a position:fixed tooltip whose x is clamped to the viewport."* Same cause, same
answer, one panel over. Worth recording so the third report doesn't start from scratch.

**Boss rulings this round:** fold the tooltip into §3b (one commit, one test) · scope it to the
**living-link chip only**, not an app-wide sweep.

### Design — Art Director & Team (`wf_8b148465-32b`), 3 proposals + judge

Winner 47/50: a **window-singleton tooltip driven by a `data-linktip` attribute contract**
(`src/lib/links/linkTip.ts`). Two delegated `mouseover`/`mouseout` listeners per window; one
reused `<div>`; chips declare text via an attribute and own nothing else.

**Why not a Svelte component** — the deciding constraint: one of the three chips is a raw-DOM
CodeMirror `WidgetType`. A component would serve the two panels and fail the editor, forcing a
third implementation — the same triplication `linkDisplay.ts` was created to end, one layer up.

**Why not per-row handlers** (the `.toc-tip` precedent's shape): `VirtualList` keys its
each-block by SLOT index, so every visible row's snippet re-runs on every scroll tick (~45–55
rows). Delegation adds **zero** per-row work — the panels already computed this string for
`title`; it now goes to a differently-named attribute.

**Placement.** Anchored to the chip element, its trailing edge pinned to the chip's trailing
edge — so in LTR the box lies to the LEFT of the pointer (the ask), and in RTL it mirrors
rather than sitting over the file tree. Vertically above, since the chip is on a row's first
line with context/annotation/headline stacked beneath. Side comes from **direction, never
viewport proximity** — proximity was tried, Boss-reported and reverted at `cc35524d`.

**Two direction signals, deliberately separate.** Placement reads the *anchor's* computed
direction; text direction reads `document.documentElement.dir` (the interface). They disagree
in exactly one real case — an Arabic UI with an English note — where the words must lay out RTL
while the box must still open into the LTR note. Collapsing them breaks one or the other.

**The Arabic fix is four properties**, each answering a distinct failure: `line-height: 1.75`
+ 9px padding (the actual answer — an OS line box is sized for Latin ascenders, so the tanwin in
*يومًا* met the frame); `white-space: normal` so the text wraps and the box grows taller rather
than truncating; `max-width` with **no min-width** (a min-width is what leaves a short English
string in a box wider than its text — the other half of the report); and **measure-then-place**,
so nothing assumes a width. That last point improves on both existing tooltips, which guess
(`HelpTip` halfWidth=200, `.toc-tip` halfW=150).

**`title=""` on the chip is load-bearing, not residue.** The row button carries its own native
tooltip; `title` resolves by walking to the nearest ancestor that has one, so without the empty
string the Windows-drawn box would appear *underneath* ours. Commented at both call sites.

**Listener exception, named.** The two delegated listeners are installed once per window for the
window's lifetime — a deliberate, documented exception to "remove every listener on destroy",
because install/uninstall per panel mount would have the editor and both panels racing to own a
shared resource, and the first unmount would silently kill the tooltip for the other two.

### Verification
`svelte-check` **0 errors** · **495 tests / 36 files green** · `link-tip` CSS **and**
`data-linktip` grep-confirmed in BOTH bundles (main + second screen) · binary rebuilt **19:44**,
verified newer than every touched source.

**Static checks are NOT runtime verification for hover/placement** — that is the Boss test.

### Inspection
Ran three times; the saved workflow ignored `args` in every form tried (object, string,
`mode`/`scope` keys) and swept **whole-app** each time. Stopped trying to scope it — worth a
separate fix to the workflow's arg handling. **Zero findings in the §3b changes.** The only hits
in a touched file are two at `BacklinksPanel.svelte:178` (`linkMention`) — both are **PJ-123**,
already filed and explicitly out of PJ-114's scope.

The two sweeps did surface pre-existing app-killers, including one the first sweep missed:
`+layout.svelte:4816` — template-insert's `.note-pane` selector matches nothing in `src/`
(CLAUDE.md itself records the class as `.pane`), so the "fallback" branch is the ONLY reachable
one, and it writes the stale store copy `tab.content` over disk. Awaiting the Boss's sequencing
ruling along with the rest.

---

# §3b-SS — the link tooltip becomes a Style-Setter element (AWAITING BOSS TEST)

Boss passed the §3b tooltip test, then: *"Add it to the Style Setter."* §3b committed first
(`5b618bf4`, pushed) so the validated work was secured before new work began.

**Placement:** a new `pLinkTip` element in the **Panels** category, sitting directly beside
`pLinkTiers`. That pairing is the point — `pLinkTiers` styles the chip, `pLinkTip` styles the
chip's explanation. (My stored recipe warned that `panels` is a two-zone category whose centre
preview would be dead; **that note was stale** — `panels`, `cognitive` and `relgraph` have since
been added to the two-zone exception list at `StyleSetter.svelte:732`, so the centre preview is live.)

**Eight controls:** Background · Text · Border · Radius · **Line height** · Max width ·
Padding (vertical) · Padding (horizontal). Line height is the one that matters — it is the dial
for the exact complaint that produced this box.

**No shadow control — deliberately.** `theme.css` wires `box-shadow: var(--tooltip-shadow,
var(--shadow-l))`, reusing the **existing** shared "Tooltip shadow" token (Global → Shadows)
rather than minting `--link-tip-shadow`. One dial should move every tooltip's elevation
together; that is the app's own unify-on-demand pattern.

**Byte-identical until edited.** Every value is `var(--link-tip-*, <today's value>)`, so nothing
changes appearance until a control is actually moved.

**The preview composes its samples with the REAL `traversalTooltip`**, in the interface language
— so what the Boss styles cannot drift from what the app draws. The second sample row is fixed
to **Arabic regardless of interface language**: the Line-height control exists *for* scripts with
above-base marks, and the tanwin over يومًا is the mark that met the frame in the OS-drawn
tooltip this box replaced. Styling that blind would be styling the wrong thing. The preview CSS
mirrors `.link-tip` property-for-property with the same vars and fallbacks, with a comment on
both sides to keep them in step.

### i18n — 3 new labels, not 8
`ssSlug` reuse meant only **`link_tooltip`, `max_width`, `room_for_marks_above_the_line`** were
genuinely new; Background/Text/Border/Radius/**Line height**/Padding (vertical)/(horizontal) all
already existed. Translated ×14 by one native-grammar agent per locale (`wf_91d1d543-b9d`).

Every agent independently found and reused **this app's own existing word for "tooltip"** from
`styleSetter.labels.tooltip_shadow` — تلميح · راهنما · הסבר כלי · ipucu · подсказка · 提示框 ·
ツールチップ · Infobulle · Dica — rather than introducing a synonym. Several explicitly rejected the
Microsoft-style calque in their language. Arabic reached further and used **التشكيل**, the app's
own word from `scriptToolbar.tashkeel`, alongside the generic "العلامات" so the hint still covers
nikkud and Persian/Urdu marks.

### A real bug caught by verifying rather than trusting the diff
The first insertion pass anchored on `raw.index('"styleSetter"')` — but `"styleSetter"` also
appears **as a value** (`"styleSetter": "منسق المظهر"`) ~3,250 lines earlier, so the search landed
on an unrelated `"labels"` block and wrote all three keys into the wrong namespace in **all 14
files**. `git diff --numstat` showed a clean `3 0` per file and looked perfectly healthy. Only
reading the keys back through `d['styleSetter']['labels']` exposed it. Reverted, re-anchored on
`"styleSetter"\s*:\s*\{` (the object, not the string), and added a **post-write assertion** that
reads each key back from the parsed object before the file is kept.

### Verification
`svelte-check` **0 errors** · **495 tests / 36 files green** · all 15 locales confirmed to carry
the 3 keys **in `styleSetter.labels`** · diffs are exactly 3 added lines per locale · new CSS var
and new label grep-confirmed in both bundles · binary rebuilt **09:40**.

**Not committed — awaiting the Boss test.**

## §3b-SS — regression found by the Boss, fixed: the tooltip stopped working entirely

Boss: *"The Style Setter is fine, but the ×N badge no longer has any effect."* His screenshot
showed the row's **native** "Right-click to set confidence" box where ours should have been.

**Cause — mine, and entirely self-inflicted.** Parameterizing `.link-tip` for the Style Setter,
I extended the comment above it and left a stray `*/` mid-block: the original comment closed at
its old line, then seven lines of prose sat **outside any comment**, ending in a second `*/`.

**Why that is worse than a syntax error:** CSS does not fail on it. The stray text is absorbed
**into the next selector**, so the rule silently became `stray text… */ .link-tip` — a selector
that matches nothing. The element was still created, still had its text, and carried **zero
styling**: no `position: fixed` (so the JS-set left/top did nothing), no background, no
`visibility` control. Proven, not assumed — parsing the exact broken text with postcss returns
the selector `"stray text…*/\n.link-tip"`, versus `".link-tip"` when fixed.

**Why nothing caught it — the part worth remembering.** `svelte-check` does not read `.css`
files. Vitest had no CSS assertion. And **my bundle grep passed**: I searched the built output
for `link-tip-line-height` and found it, because the text was still in the file even though it
no longer parsed as a rule. *Presence in the bundle is not proof that the CSS parses.* That
grep gave me false confidence and I shipped on it.

**Fixed + guarded.** One comment block. New `tests/pj-114/linkTipCss.test.ts` parses `theme.css`
with postcss and asserts: `.link-tip` exists as an exact selector · **no selector anywhere in the
file has swallowed a comment delimiter** (guards the whole file, not just this rule) · the three
declarations the box needs to be visible at all (`position: fixed`, `visibility: hidden`,
`pointer-events: none`) · all eight Style-Setter vars are wired **with fallbacks** · the shadow
still reuses the shared `--tooltip-shadow`. **Verified red→green**: re-introduced the exact break
and all 5 assertions failed; restored and all 5 pass.

**Build verification upgraded.** The bundle check no longer greps for text — it parses every
built `.css` with postcss and asserts `.link-tip` resolves as a rule with `position` present.
Both stylesheets (main + second screen) pass with 21 declarations each.

`svelte-check` 0 errors (one caught in my own new test — a concise arrow returning a Map where
postcss's walker wants `void | false`) · **500 tests / 37 files green** · binary **10:46**.

**Not committed — awaiting the Boss re-test.**

## §3b-SS2 — the link panels stop using OS tooltips entirely (AWAITING BOSS TEST)

Boss passed the regression fix, then: *"Fix the badge tooltip style, as you did for the xN chip,
and include it in the Style Setter."* Binary **11:07**.

**The row hint is now app-drawn.** `linkConfidence.rightClickHint` on both row buttons moves from
`title` to `data-linktip`. **No second Style-Setter element was added** — the row hint renders in
the *same* `.link-tip` box, so the existing "Link tooltip" controls already govern it. The preview
now shows a third sample row carrying the hint, so the element does not under-report its reach.
(`Backlinks` was already a label slug — zero new i18n for that.)

**`title=""` relocated to the row button**, the outermost element of the row: `title` resolves by
walking to the nearest ancestor that has one, so ONE empty title there terminates the native walk
for everything inside, including the chip. The chips' own `title=""` was then redundant and removed.

### The change surfaced a defect it would itself have caused (WA#6)
Putting `data-linktip` on the ROW meant every child that still carried a native `title` would draw
**two boxes at once** — ours (resolved from the row via `closest()`) plus the OS one. Four such
children existed: the annotation and NSC-headline spans in both panels. All converted to
`data-linktip`. Nearest-ancestor-or-self resolution means hovering the annotation shows the
annotation and hovering elsewhere on the row shows the hint, with no ambiguity.

**A pre-existing localization bug found in passing:** the unlinked-mention "Link it" button carried
a **hardcoded English `title="Link it"`** — never localized, in violation of the standing order.
New key `backlinksPanel.linkIt` ×15 (`wf_b32b9254-7d2`, one native agent per locale). Every agent
grounded the verb in the app's own existing root for "link" and most matched
`reviewer.suggestLinkBtn` verbatim; several explicitly rejected the bare noun because a noun on an
icon button reads as "a link" rather than "link it". Chinese and Hebrew independently chose the
*convert-into-a-link* form (转换为链接 / המר לקישור) as truer to what the button does.

### An accessibility regression I introduced, caught by svelte-check
Removing `title` from the **icon-only** "Link it" button silently removed its **accessible name** —
`title` had been doing double duty as tooltip *and* label, so a screen reader would have announced
an unlabelled button. `aria-label` added, commented as required-not-optional for any icon-only
control converted to the app-drawn tooltip. The warning count is back to its 264/38 baseline; a
count that rises by one is a real signal, not noise.

### Verification
`svelte-check` **0 errors, 264 warnings (baseline)** · **500 tests / 37 files green** · `.link-tip`
**parses as a rule** in both built stylesheets (not merely present as text — the lesson from the
previous round) · localized `linkIt` confirmed in both bundles · all 15 locales verified to carry
`backlinksPanel.linkIt` **in the right block**, diffs a clean 5 lines each · binary **11:07** newer
than every touched source.

*Two Sight-v6 perf budgets failed on one full-suite run and passed in isolation — they are
wall-clock budgets and were starved by the parallel translation workflows, not regressed.
Re-verified green on a quiet run rather than assumed.*

**Not committed — awaiting the Boss test.**

## §3b-SS3 — summaries are language-aware, by DOMINANCE (AWAITING BOSS TEST)

Boss: *"Make sure that the summary are language aware. In this case (the image), it should be
RTL."* His screenshot showed an Arabic NSC summary in the app-drawn tooltip laid out **LTR** —
the short final line hugging the left edge, the sentence period on the wrong side. Binary **11:24**.

**Cause — my own design decision.** `linkTip.ts` set the box's direction from
`document.documentElement.dir`, i.e. the INTERFACE language. That is correct for what the box
originally carried (the ×N diagnostic, composed in the UI language) but wrong now that the same
box also carries **note content**: a link's annotation and an NSC summary headline, which are in
the *note's* language. An English interface therefore forced an Arabic summary to lay out LTR.

**The fix is `detectDir(text)`, NOT `dir="auto"` — and that distinction is the point.**
My first instinct was `dir="auto"`. The repo has already litigated and REJECTED that:
`tests/pj-106/rtlDirection.test.ts` documents §A1 replacing `dir="auto"` with `detectDir`
because auto resolves from the **first strong character**, and *"first-char heuristics get
bilingual content wrong."* That is not hypothetical here — **the very screenshot reporting this
bug contains the failing shape**: a row titled "Arabic music" (Latin-first) whose summary is
Arabic (Arabic-dominant). `auto` lays that out LTR; dominance gets it right. Checking the repo's
own prior ruling turned a plausible fix into a correct one.

**Extended to the inline rows too.** The Boss's instruction was general ("the summaries"), and the
same summary renders both in the tooltip and inline in the row — with the identical weakness. The
seven content-bearing spans in both panels (summary headline ×3, annotation ×2, context excerpt
×2) move from `dir="auto"` to `dir={detectDir(…)}`. Deliberately NOT changed: the row *buttons*
(their `dir` governs row layout and the tooltip's opening side, a different question) and the
filter *input* (a UI control, where auto on typed text is correct).

**Alignment pinned.** `text-align` is an inherited property, so a resolved-RTL box could still sit
flush left if any ancestor set `text-align: left`. `.link-tip` now pins `text-align: start`, which
follows whatever direction was resolved. Verified as `start` in **both** built stylesheets.

**Two direction signals remain deliberately separate** and the comment now says why: TEXT
direction from the content's own dominance; PLACEMENT side from the ANCHOR's resolved direction
("which way does this surface open?"). They disagree in a real case — an Arabic UI with an English
note — where the words must follow their script but the box must still open into the note.

### Verification
`svelte-check` **0 errors, 264 warnings (baseline)** · **504 tests / 37 files green** (4 new,
locking the exact screenshot shape: Arabic summary → RTL; **Latin-first Arabic-dominant → RTL,
which `dir="auto"` would fail**; Arabic-first English-dominant → LTR; and each composed chip
tooltip resolving to its own interface language) · `text-align: start` confirmed in both built
stylesheets · binary **11:24**.

*The two Sight-v6 wall-clock perf budgets failed again on a loaded full-suite run and passed on a
quiet one — same transient as before, re-verified rather than assumed.*

**Not committed — awaiting the Boss test.**

## §3b-SS4 — Suggested Connections joins the app-drawn tooltip (AWAITING BOSS TEST)

Boss reported the Suggested Connections panel: the summary tooltip "not working", the Link button
still showing a Windows box — while Outgoing Links "working perfectly". Binary **11:42**.

**Honest classification: NOT a regression I introduced.** `RelatedCandidates.svelte` is untouched
by this session — no working-tree change, last commit `19448e1c` (MIG-086). It never had
`data-linktip`; it still used native `title`. What changed is the *contrast*: now that the two link
panels draw their own tooltips, the un-converted surface beside them reads as broken. The
user-visible complaint is entirely valid; the cause is coverage, not breakage.

**What was actually there, read off the CSS rather than assumed:**
- `.rc-name` — `overflow:hidden; text-overflow:ellipsis; white-space:nowrap` → genuinely truncates,
  so its tooltip earns its place. Converted to `data-linktip`.
- `.rc-snippet` — `max-height: 2.7em; overflow: hidden` → clipped to ~2 lines, and carried **no
  tooltip at all**. That is precisely the "summary tooltip not working": there was never one to
  work. Added.
- `.rc-link` — `white-space:nowrap; flex-shrink:0` → never truncates, and its visible label already
  reads "🔗 Link". Its `title` only ever repeated the text on the button. **Removed rather than
  converted** — a tooltip that restates a visible label is noise (Constraint as Design). The button
  keeps its accessible name from its own visible text, so no `aria-label` is needed here (contrast
  the icon-only "Link it" button, which did need one).

Both content lines also move `dir="auto"` → `dir={detectDir(…)}`, carrying forward the Boss's
language-awareness ruling to this surface.

**svelte-check earned its keep again:** my added `import { detectDir }` was a DUPLICATE — the file
already imported it three lines below. Three errors, caught before the build.

### Coverage — surfaced rather than left for the Boss to find
This is the second surface the Boss has had to report. A survey found **295 native `title=` sites
across the app** (`+layout.svelte` 64, `NotePane` 22, `SourceReviewPanel` 17, `TableToolbar` 16,
`GraphMindView` 10 …). Most are legitimately chrome (toolbar buttons, icon controls) where a native
tooltip is fine. The ones that matter are those showing **note content** — names, snippets,
summaries, annotations — because those are the ones that truncate, and that need language-aware
direction. Rather than converting 295 sites unilaterally or waiting to be told one at a time, the
list goes to the Boss with a recommendation to convert the *content-bearing* subset only.

### Verification
`svelte-check` **0 errors, 264 warnings (baseline 264/38)** · **504 tests / 37 files green** ·
`RelatedCandidates` wiring confirmed in the bundle · binary **11:42**.

**Not committed — awaiting the Boss test.**

---

# APP-KILLER remediation — Boss ruling: "Fix the app-killers first, then the PJ-126"

*Function in hand: the five confirmed silent content-loss defects from the whole-app sweeps.*

## Phase-1 Architect (`wf_2f120304-01e`): 5 re-verified at HEAD, adversarially refuted

**Verdicts:** moveItem repath **CONFIRMED** (worse than filed) · cascade window **CONFIRMED**
(refuter attacked on 5 vectors; all held) · second-screen dirty-birth **CONFIRMED** (5 vectors
held; the net is mirrored through localStorage, which is how the SS sees the main window's
snapshot) · template-insert stale write **CONFIRMED** · gate_rename clobber **DOWNGRADED** —
the code fact is true (no dest-exists check in the locked region) but the consequence is refuted:
`move_item` checks `dest.exists()` at `libraries.rs:1713` BEFORE calling gate_rename, and the
rename path's collision handling sits upstream too. Real hole (a race between check and rename,
and any future caller), not an app-killer. Re-file at lower severity.

**Workflow caveat, recorded honestly:** the sequencing agent received only the first 2 of 5
analyses — my script sliced its input at a character budget. Its "2 confirmed" framing is an
artefact of MY truncation, not a finding. The per-bug verdicts above are each from a dedicated
agent with full context and are what I acted on.

**moveItem got WORSE under scrutiny:** the slice starts after the moved folder's own name, so the
segment is dropped for DIRECT children too — wrong 100% of the time, not just for deep nesting.
And when the target folder already holds a note with the same basename, the phantom write lands ON
that unrelated note (gate verdict `WouldRefuseIdentity`, but `WRITE_GATE_ENFORCE = false` means
journal-and-proceed) — destroying a second note, not just forking the first. A second symptom
surfaced: clicking the real note in the tree after the move opens a SECOND tab on the same file
(the wrong-path tab can't match), two tabs with divergent models.

## FIX #1 SHIPPED (build 12:42) — moveItem derives from the Rust-returned path

**Reproduce-First compliance:** this one is deterministic (no timing window), so the red test IS
the reproduction of the repath contract, and the Boss's live recipe proves the on-disk half.
**Red first:** `tests/pj-127/moveItemRepath.test.ts` written BEFORE the fix — **5 of 7 failed**
against the bug (direct child, deep descendant, Windows separators, Rust-chosen collision name,
same-prefix sibling). Notably the "tab and model never disagree" case PASSES with the bug present
— both get the same wrong value; that agreement is exactly what defeats `compose`'s identity
guard, and the test file says so.

**The fix** (store.ts:3552-3555): `newPath + relative` instead of `targetFolder + relative` —
matching `renameItem`'s always-correct sibling branch (`effectivePath + relative`). Also the only
correct source when Rust chooses a collision-suffixed final name, which `targetFolder` cannot know.

**Blast radius mapped (WA#4):** both UI callers (single `+layout.svelte:6027`, batch `:6017`)
route through `moveItem` — one fix covers both. Repo-wide grep: no other `+ relative` path
derivation exists. **PJ-098 verified STILL LIVE and adjacent:** `OrgChart.svelte:254` calls raw
`invoke('move_item')`, bypassing the wrapper — an OrgChart drag-move repaths NOTHING (a different,
already-filed defect). Boss test must use the file tree's Move, not the OrgChart, or it will look
like the fix failed.

**Verification:** red→green 7/7 · full suite **511 / 38 green** · svelte-check 0 errors ·
binary **12:42**. **Not committed — awaiting the Boss test.**

**Queue after this:** #2 template-insert (deterministic, dead selector — design question: what
SHOULD "insert template" do, given the reachable branch never worked as designed?) · #3 cascade
window + #4 second-screen (both need the on-screen instrumentation strip + the artificial cascade
delay before any fix, per Reproduce-First — they are races) · gate_rename re-filed at lower
severity · PJ-126 after.

## ⋯-menu dead commands — Boss-reported, THREE distinct mechanisms, all fixed (build 13:32)

Boss passed the moveItem fix, then reported five ⋯-menu commands dead: Copy path, Copy name,
Show in system explorer, Open in default app, Delete. Investigation found the five split across
THREE failure mechanisms, plus a sixth dead item the Boss had not hit yet:

1. **The four file-ops — shadowed by the host's existence.** `NoteEditor.handleMoreAction` was
   `if (onmoreaction) { delegate } else { built-in }`. The main window's handler knows only its
   own five actions (rename/revealInTree/delete/addProperty/switchToFocus), so the four file-ops
   fell into its switch, matched nothing, and died silently. **Fix (structural):** the four pure
   file-ops are handled in NoteEditor UNCONDITIONALLY — they depend only on the tab and must be
   identical in every host (the second screen passes no handler at all and always relied on the
   built-ins) — and only host-owned actions delegate. The `.catch(() => {})` swallows became
   `console.error`s while in there.
2. **Delete — a phantom event.** The host dispatched `constellation:delete-note`; grep proves NO
   listener has ever existed. **Fix:** the menu joins the tree's EXACT delete flow —
   `confirmDelete = {path, name}` → the same confirm dialog → gated `deleteWithSetting`, which
   itself closes the open tab (verified at store.ts:3598-3603). No new delete path invented.
3. **addProperty — dispatched at `window`, listener on `document`** (PropertyEditor.svelte:474).
   Events dispatched AT window never reach document listeners (propagation runs document→window,
   not back down). Dead since wiring; unreported — found while fixing #2, fixed in-pass (WA#6).
   **Fix:** dispatch on `document`.

The phantom-event class has PRECEDENT the codebase itself records: the comment at
+layout.svelte:5891 documents `reveal-in-tree` being "dispatched but nothing listened" and fixed.
Third instance of the class (delete, addProperty, reveal-in-tree). Worth a lint/test someday:
every `CustomEvent('constellation:*')` dispatch should have a greppable listener.

Verification: svelte-check 0 errors / 264 baseline · 511 tests green · binary **13:32**.
**Not committed — awaiting Boss test (bundled with the moveItem fix test already passed).**
