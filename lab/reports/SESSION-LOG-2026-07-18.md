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
