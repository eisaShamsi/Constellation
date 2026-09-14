# PJ-461 — the panel's verdict (`wf_fed7c5d6-bce`, 13 agents, 0 failures)

Four lenses (theme architect · Boss sitting / visual regression · concept & scope · verification &
method) → two adversarial attackers each → chair. The first run died on a usage limit with no
chair and no ruling was taken from it; this is the resumed run, completed in full.

## Rulings

### Q1 FIX SHAPE — rename every site, add 17 Tier-3 aliases, or hybrid
**RULING.** RENAME. Zero new Tier-3 aliases. Of the 77 distinct lines: 65 renamed, 9 stripped (--library-accent, Q2), 1 given a literal (--sidebar-width, Q10), 2 left untouched (--link-tip-font-size, Q3). --bg-modifier-border is struck — measured 0 uses. Two of the 65 leave this pass for a Boss ruling (CalendarPanel:225,323 — Q10).

**Why.** I read theme.css:156 myself: the tier is headed 'TIER 3: Backward-Compatibility Aliases' and its twelve members are declared on `.theme-light, .theme-dark` — it exists to keep names that ONCE resolved resolving. Minting aliases for names that never resolved would promote seven spellings of three fonts (measured: theme.css:8-10 defines exactly three font tokens) into permanent public API. It also destroys the only gate that can fail: with an alias, `--font-sans` still ships in the bundle and the negative grep — the one instrument in this stack that catches a mistyped custom property — becomes unavailable.

**Dissent.** theme-architect's own stated reason for rejecting aliases ('--font-sans and --font-text sit on the SAME declaration meaning different things') is REFUTED and I do not adopt it: NoteButterflyGraph:410 is `var(--font-text, var(--font-sans))`, a nested fallback, which an alias tier would resolve correctly. The ruling stands on the closed-tier and verifiability grounds, not that one. Second dissent, unresolved in principle: the 'complete Tier 2' option (five of the 17 are Tier-2-shaped names Tier 2 never defined) was never put on the table by any lens and my ruling does not reach it — I decline it on drain-scope grounds, not on merit.

### Q2 --library-accent — strip, keep, or re-feed from accent_color
**RULING.** STRIP at all 9 sites. livePreview.ts:1930 is DELETED, not renamed. Do not re-feed. File the orphaned accent_color as an inert-data tidy-up, not as a false-success.

**Why.** Read all nine verbatim: eight place the name before --interactive-accent (defined, theme.css:108) so the strip is byte-identical; :1930 is `var(--interactive-accent, var(--library-accent, #6c5ce7))` — the name is unreachable text there, and renaming it would emit `var(--interactive-accent, var(--interactive-accent, #6c5ce7))`. The 'check whether a UI still offers the colour' worry is SETTLED, not deferrable: there is no write_library_appearance command anywhere in src-tauri, the value's origin is an imported Obsidian appearance.json, and grep finds zero frontend readers. No control is silently broken.

**Dissent.** theme-architect wanted the filed PJ to open with 'check first whether any UI still sets it.' Both its attackers closed that question by measurement in-session; leaving it open would file an unknown that is already known.

### Q3 --link-tip-font-size — add a control, which unit, or drop the var
**RULING.** LEAVE BOTH LINES EXACTLY AS AUTHORED (theme.css:228, StyleSetter.svelte:2204). No control, no literal substitution. Strike the name from the job — it is not one of the defects. If it is ever built, the unit is rem.

**Why.** Replacing it with the literal would REGRESS a working family: the eight sibling --link-tip-* names are the app's declared 'unify on demand' pattern and theme.css:209-215 says so in its own words. The concept for a dial is sounder than the drain cycle can pay for — I verified the tooltip is `document.body.appendChild(el)` (src/lib/links/linkTip.ts:81), so the `* var(--rs-scale, 1)` factor in that very declaration is permanently 1 and NO existing control can move the tooltip's size. That makes the dial genuinely unanswered by Line height or the panel scale — and a feature the drain cycle does not build.

**Dissent.** theme-architect ruled ADD IT as owed work, on the strength of theme.css's 'every value below is var(--link-tip-*, <today's value>) with a Style Setter control' comment. That is the strongest argument on the table and I reject it only on cycle scope: a registry row is a feature, it is the one change in the job that writes persisted state, and it would void the pure-UI inspection exemption. concept-scope's reason for the same outcome ('the dial has no horse') is REFUTED by the dead --rs-scale measurement — the horse exists; the cycle does not.

### Q4 HOUSE NAME at +layout 12293, 12311, 11555
**RULING.** --bg at 12293 and 12311 (Tier-3, matching the block). 11555 gets the accent tint `color-mix(in srgb, var(--interactive-accent) 18%, transparent)` — NOT --accent-bg.

**Why.** I read theme.css: --accent-bg → --background-secondary-alt → --color-base-25, and --bg-hover → --background-modifier-hover → --color-base-25. The same colour. And +layout:11554 `.ws-base-item:hover` already sets `color: var(--text-normal)`, which :11555 sets too — so --accent-bg would make the active row COMPLETELY identical to hover, background and text. That is a check that cannot fail: you could never see it work. 11555 shares one concern with Q7's two sites — 'a selected row with no distinct background' — and the Whole-Ecosystem Law gives one concern one answer.

**Dissent.** theme-architect and concept-scope both ruled --accent-bg on local-idiom grounds and disclosed the collision as pre-existing house behaviour (citing .s-result.active:11425). The precedent is real but it is a precedent for a defect, and at 11555 the collision would be NEW — verified: `.ws-base-item.active` has no fallback, so it is transparent today and distinguishable. I file the .s-result:11424/11425 collision separately. Also note: both lenses' Tier-3-throughout citation is partly false — :12304 uses --text-muted and :12316 --text-on-accent, both Tier 2 — so the --bg pick rests on the block's dominant vocabulary, not on purity.

### Q5 OrgChart connectors — --background-modifier-border vs -border-focus
**RULING.** --background-modifier-border at OrgChart.svelte:1326, 1338, 1355. Disclose the contrast change in BOTH directions before he sits; escalation to -border-focus is his call at the sitting, not mine here.

**Why.** Measured myself: --background-modifier-border appears on 18 other lines in that file and `border-focus` appears ZERO times. A tree connector shares the concern of the node borders it joins; -border-focus means the border of a FOCUSED element, which a static connector is not — choosing it would be picking a token for its pixel value. #555 is a literal painting in both themes today, so it is already wrong in one of them.

**Dissent.** Every lens said 'seven other borders'; the real count is 18 — the register undercounted and three members repeated it. More important, every lens framed the change one-directionally. Measured: light #555 → #e0e0e4 (lighter), dark #555 → #313244 against a #1e1e2e panel (markedly fainter). His colorScheme is 'light' in all nine universes, so the light direction is what he will see by default, but the dark stage is where a rejection is most likely.

### Q6 Ledger tier dots — fix :157, fix 157-159, or out of scope
**RULING.** OUT OF SCOPE for PJ-461. File it whole: FOUR renderings (NoteLedgerGraph.svelte:156, 157, 158, 159) onto the traversal-tier chain, against THREE already-correct references. Never 'fix :157 only'.

**Why.** None of the four is one of the 17. I checked --rel-supports myself: its only non-var() occurrence in the tree is a Style Setter colour control (StyleSetter:470) — it is the designed undefined-until-set --rel-* family the register's own KEEP-AS-IS list exempts; --interactive-accent and --text-faint are defined; :156 uses --text-muted. So this is a wrong-token concern, not an undefined-name one, and folding it in would put colour changes on his second screen inside a commit framed as a rename. The filing must carry three corrections the register got wrong: :156 `.lg-ltier` is the base dot every tier wears and there is NO `.lt-emerging` rule (verified at the markup, :118 `class="lg-ltier lt-{f.tier}"`), so emerging falls through; the concern has three correct references (BacklinksPanel:421-438, OutgoingLinksPanel:277-303, StyleSetter:2184-2187) which need NO edit; and the token is spelled --link-tier-loadbearing, one word (StyleSetter:497).

**Dissent.** verification ruled fix-now, on the strength of cockpitGraphData.ts:59-61 — the codebase's own prior diagnosis of a --rel-supports borrow. I checked it: that comment is about CONFIDENCE levels, not traversal tiers. Same anti-pattern, different vocabulary. It raises the filed item's priority; it does not pull it into this job.

### Q7 the two 'active row' sites — accent tint vs hover vs leave
**RULING.** BOTH get `background: color-mix(in srgb, var(--interactive-accent) 18%, transparent)` — and so does +layout:11555 (Q4). BACKGROUND ONLY: do not copy MoveDialog's `color: var(--interactive-accent)`. Reject 'leave TemplateStudioRow'.

**Why.** TemplateStudioRow.svelte:88/:89 read verbatim: the selected row's fallback IS the hover token, with no font-weight or other differentiator — the selected row cannot be told from the hovered one. That is the bug, not graceful degradation, and WA#6 forbids noting-and-shipping it. ConstellationMap:924's own literal #e8e8ff is a pale accent tint, so the tint is the author's measured intent, not an invention. I decline the `color` half because the concern being repaired is the missing BACKGROUND; at ConstellationMap:920 the row already sets its own `color`, and importing a second change neither site asked for is exactly the scope drift this job is cleaning up.

**Dissent.** Two attackers showed ConstellationMap:924 already carries `font-weight: 600`, so 'the selected state would stop being visible' is FALSE there — I do not use that argument. And boss-sitting's split (tint the Map, collapse TemplateStudio to hover) applies its own test inconsistently: it calls the collision unacceptable at the site that does NOT have it and acceptable at the site that does. Counter-dissent I accept: copying half of MoveDialog's idiom is itself a duplication, and a reviewer may reasonably say the colour belongs with it.

### Q8 unreachable surfaces — fix in the same pass or leave
**RULING.** FIX ALL, DISCLOSE with a number and a per-surface reason. Corrected counts: Sight = 9 (not 11), Radial = 5. Add three more unshowable live-code sites the register missed: +layout:12293, +layout:12311, ConstellationMap:924 — total 17 unshowable. The builder verifies Radial's 5 himself; he does NOT ask the Boss.

**Why.** Leaving them is how PJ-461 was born, and the render risk of touching dead code is nil. But the tiers must not be blurred. Sight: engine.ts:146/:168 both false, and I measured that --interface-font and --mono-font have ZERO occurrences anywhere under build/ — the grep reads 0 before AND after, so it is a check that cannot fail. ConstellationMap: store.ts:7429 appends `constellationMap: false` AFTER the user spread on every settings load, commented 'force-disables… even for users who previously enabled it… Reversible: delete the override' — the Settings→Plugins toggle flips it in memory only. Telling him to switch it on is an instruction he cannot follow. +layout:12293 renders only under `{:else if $libraries.length === 0}` (verified at :10036) and every universe auto-registers a library; +layout:12311's `.w-option-input` has NO markup user at all — I grepped: the class appears only in its own two CSS rules, so Svelte prunes it and the site is dead code.

**Dissent.** Register E's stated Radial mechanism is REFUTED by all six attackers and by my own read: cockpitFlag.ts lists FOUR styles, 'heartwood' at built:false, and normalizeGraphStyle KEEPS any listed id — the gate is the picker's `.filter((s) => s.built)`. Measured on live data, his noteGraphStyle is orrery/butterfly, never heartwood. So Radial is unreachable in practice but reachable from a settings value — which is why WA#1 makes those 5 the builder's job, not a disclosure.

### Q9 VERIFICATION + SITTING
**RULING.** Gate on: (a) a RECURSIVE, BOUNDARY-ANCHORED grep over the whole build tree, (b) one positive literal, (c) a narrow headless-Chrome shape probe, (d) /simplify. Claim the inspection exemption explicitly in the commit. Sit in four stages: light main window, dark main window, second screen, then the Calendar question. Route the whole sitting through tutorial-auditor → ui-inspector → panel before it reaches him.

**Why.** Nothing else in this stack can disagree with me. A var() naming a nonexistent property is valid CSS at parse time — the Svelte compiler, svelte-check, vitest and the Rust suite all pass it; there is no stylelint in devDependencies. Three measured traps make the naive gate worthless, and I reproduced all three: the lens CSS ships to a SECOND bundle (25 of 27 --font-sans occurrences are in build/assets/screen-B8k-aTeL.css, 2 in build/_app/immutable/assets/0.C0ITB83c.css, and livePreview's declarations land in JS chunks); four of the 17 names are substrings of their own replacements (plain `--font-monospace` = 26 hits in the main bundle vs 10 boundary-anchored — a 16-hit phantom from --font-monospace-theme); and `grep -c` counts LINES on minified CSS that is one line. Add a precondition nobody stated: build/ must post-date HEAD — measured, today's build is 2026-09-12 20:05 against HEAD at 2026-09-13 11:31, so every bundle figure in this panel was read from a different tree than the one being edited.

**Dissent.** verification's positive literal `13px var(--font-text-theme)` FAILS on a correct fix (that line renames both names, so the closing paren never follows). boss-sitting's `font: 600 9px …` fails on the space — I measured the emitted bytes as `font:600 9px var(--font-sans)`, unminified spacing appears nowhere. concept-scope's 'zero hits for all 17' is unachievable for the four substring names and vacuous for the two Sight names that already read zero. Use `grep -oE -- "--<name>[^-A-Za-z0-9_]" | wc -l`, recursively, with `600 9px var(--font-interface-theme` as the single positive.

### Q10 THE TABLE — replacements wrong for the site's intent or for dark/light
**RULING.** Six substantive corrections, itemized in register_corrections and applied in final_table. The two that would have shipped a defect: --border-faint must go to --border (not --border-light), and --bg-active must NOT go to --accent-bg. The two Calendar font lines LEAVE this pass for a Boss ruling. The count is 80 occurrences on 77 lines across 22 files — and the ledger's 75/21 is CORRECT, not stale.

**Why.** I measured the border tokens myself: --border-light aliases --background-modifier-border-focus = base-35 = #d4d4d8 light / #3a3a4e dark, against --border = base-30 = #e0e0e4 light / #313244 dark. Base-35 is the HIGHER-CONTRAST line in both themes — the alias named 'light' is the heavier one — so mapping a separator the author called 'faint' onto it inverts his word. And TasksPanel already uses --border-light at :188 and :214 for its box borders while reserving a different, fainter name for the row divider at :229: a deliberate two-weight scheme that --border preserves. On the Calendar: I confirmed --cal-font is SET (to 'Dubai') in seven of his nine universes and UNSET in two, including Eisa Universe — so the typeface flip is real and reachable for him, and no edit at that site is both name-correct and behaviour-neutral. That makes it a design decision, not a rename.

**Dissent.** boss-sitting proposed `var(--cal-font, var(--font-text-theme))` as 'byte-identical'; an attacker MEASURED it is not — today's invalid declaration inherits the body font, which is --font-interface-theme (+layout:11172), so the note-font remedy ships the very unannounced typeface change it was written to prevent. concept-scope and verification both ruled the Calendar rename safe ('the Arabic-first priority is untouched'); three independent headless-Chrome runs say the opposite — the bare `inherit` in a family list voids the declaration today and the rename turns Amiri ON. theme-architect's --border-light finding is the single best catch in the panel and I adopt it.

## Corrections to the evidence register (including my own errors)
- COUNT — Register A's 'distinct source lines = 79' is WRONG and its indictment of the ledger is wrong too. I measured with a boundary-anchored grep over src/: 80 occurrences on 77 DISTINCT lines across 22 files. Per-name counts reproduce the register's exactly (27/3/11/1/3/2/6/9/4/4/2/2/1/1/1/2/1/0 = 80); three lines carry two names (NoteButterflyGraph:410, NoteLedgerGraph:134, NoteOrreryGraph:824). And the ledger's '75 sites / 21 files' is EXACTLY right for what it was counting: 77 minus theme.css:228 and StyleSetter:2204 — the two --link-tip-font-size lines the ledger itself ruled 'keep the literal fallback' — and 22 files minus theme.css, whose ONLY site is :228 (verified). Open point J1 is settled this way. Only 79 was ever wrong.
- REGISTER E, Sight — '11 sites' is WRONG; it is 9, and E contradicts A's own per-name table (6 --interface-font + 3 --mono-font). The nine: sight/v6/SightV6.svelte:1361,1684,1764,1799,1829,1836; sight/v6/facetSidebar.svelte:156; sight/v6/tour.svelte:134; sight/v7/SightV7.svelte:215.
- REGISTER E, NoteRadialGraph — the mechanism is WRONG. NOTE_GRAPH_STYLES (cockpitFlag.ts:26-31) lists FOUR ids, 'heartwood' at built:false, and normalizeGraphStyle (:36) returns any id present in the list — so 'heartwood' is NOT normalized to butterfly and SecondScreenCockpit.svelte:174's {:else} mounts NoteRadialGraph. The real gate is the picker's `.filter((s) => s.built)` at :155. Consequence: those 5 lines are builder-verifiable via a settings value, not untestable.
- REGISTER E, ConstellationMap — 'reachable only when enabledFeatures.constellationMap === true' is materially misleading. src/lib/libraries/store.ts:7429 appends `constellationMap: false` AFTER the parsed settings on every load, commented 'MIG-038… force-disables Constellation Map even for users who previously enabled it… Reversible: delete the override.' The Settings→Plugins toggle flips it in memory only; it never survives a reload. The site is unshowable.
- REGISTER E, missing unreachable sites — +layout.svelte:12293 renders only under `{:else if $libraries.length === 0}` (:10036), unreachable on any configured universe; +layout.svelte:12311 is DEAD CSS — `.w-option-input` and `.w-option-form` appear nowhere in markup (I grepped: only their own CSS rules), so Svelte prunes them. Unshowable total is 17 lines, not 16.
- REGISTER E, emit locations — the record must name THREE, not one: build/_app/immutable/assets/*.css (main window), build/_app/immutable/chunks/*.js (livePreview's CM6 theme objects), and build/assets/screen-*.{css,js} (the second-screen entry from vite.screen.config.js). Measured: --font-sans = 2 in the main CSS bundle and 25 in the screen bundle.
- REGISTER A, --border-faint — the proposed --border-light INVERTS the author's intent. theme.css: --border-light → --background-modifier-border-focus → base-35 = #d4d4d8 light / #3a3a4e dark; --border → --background-modifier-border → base-30 = #e0e0e4 light / #313244 dark. Base-35 is the higher-contrast line in BOTH themes. TasksPanel already uses --border-light at :188 and :214 for box borders and reserved the fainter name for the row divider at :229.
- REGISTER A, --text-font — the Calendar row is recorded as a plain rename and is not one. The current declaration `var(--cal-font, 'Amiri','Cairo', var(--text-font, inherit))` is INVALID at computed-value time (a bare `inherit` cannot sit in a family list), so the panel inherits the interface font and Amiri has never rendered. Three independent headless-Chrome runs agree. Amiri is bundled with no unicode-range. MEASURED on his machine: --cal-font is set to 'Dubai' in seven of nine universes and UNSET in 'Eisa Universe' and 'Constellation Test' — so the flip is reachable for him.
- REGISTER A, --font-monospace — the row flattens two before-states. Nine sites carry `, monospace` and render the generic face today; TWO have no fallback at all (livePreview.ts:1908 `.cm-lens-error-msg`, +layout.svelte:12527 `.federation-popup-path`) and therefore render the INHERITED PROPORTIONAL font. Their delta is proportional → monospace, a much larger visible change.
- REGISTER D's consequence line — '26 of the 27 --font-sans lines are the font: shorthand; .orr-ring-lbl uses font-family only' is right but its implication is backwards. A `font-family: var(--font-sans)` longhand with an undefined var and no fallback is ALSO invalid at computed-value time and inherits — and the inherited value IS the interface font. So NoteOrreryGraph:821 does not render correctly today and its rename changes NOTHING. What survives at that site is its separate `font-weight: 600`.
- REGISTER D — the family does not change at the 27 --font-sans sites. The main window's body carries `font-family: var(--font-interface-theme)` (+layout.svelte:11172) and the second screen carries it on `.second-screen` (SecondScreenPage.svelte:1480) — which is the rule that actually governs the lenses, because the second screen is a standalone app mounted by src/screen-entry.ts and never loads +layout. The visible delta is SIZE and WEIGHT. Family changes at only three lines (410/134/824), and only if his Note font differs from his Interface font.
- REGISTER D — the labels' today-size is measurable, not merely 'inherited': SecondScreenCockpit.svelte:186 `.ck { font-size: 14px }` is a literal, so every cockpit lens label renders 14px/400 today. The two Style Setter relgraph sites (:1982, :1984) inherit a DIFFERENT, unmeasured main-window baseline.
- REGISTER F — mis-scoped twice. The Ledger diverges at FOUR renderings: :156 `.lg-ltier` is the base dot every tier wears and there is no `.lt-emerging` rule (verified at the markup, :118). And the chain has three ALREADY-CORRECT references, not one: BacklinksPanel.svelte:421-438, OutgoingLinksPanel.svelte:277-303, StyleSetter.svelte:2184-2187. The token is --link-tier-loadbearing, one word (StyleSetter:497).
- REGISTER F — should record that --rel-supports is itself undefined-until-set: its only non-var() occurrence is the Style Setter control at StyleSetter.svelte:470. It is the designed --rel-* family the register's own KEEP-AS-IS list exempts, which is precisely why NoteLedgerGraph:157 is outside PJ-461.
- REGISTER G — undercounts the OrgChart idiom. Measured 18 other --background-modifier-border lines (1258,1279,1291,1422,1437,1445,1461,1462,1464,1489,1523,1583,1610,1617,1631,1638,1644,1674), not seven. border-focus = 0 in that file.
- REGISTER B — '(no root font-size => 1rem = 16px)' is WRONG. Both windows set it from the user's setting (+layout.svelte:2544, SecondScreenPage.svelte:530, `(s.interfaceFontSize || 14) + 'px'`), and his interfaceFontSize is 17, so 0.8rem = 13.6px on his machine. Register H's '12.8px' inherits the same error.
- REGISTER B — 'the --font-*-theme tokens are SET by JS on the root' is WRONG; they are set on document.body.style (+layout.svelte:2535-2536, whose own comment says 'Set font variables on <body> to override .theme-light/.theme-dark class rules'). The local variable is named `root` and is body.
- REGISTER C — the definedness method has a blind spot it cannot see past: styleOverride is applied through a GENERIC writer (`for (const [k,v] of Object.entries(s.styleOverride ?? {})) root.setProperty(k,v)`, SecondScreenPage.svelte:595 and its +layout twin), so a grep for literal setProperty calls proves nothing. The sound closure is the Style Setter registry — 362 `var:` entries, none of them among the 17 — which two attackers ran and which I accept. The conclusion survives; the stated method did not establish it.
- REGISTER C — the accent_color question is SETTLED, not open: no write_library_appearance command exists in src-tauri, the value originates from an imported Obsidian appearance.json (libraries.rs:4597), and grep finds zero frontend readers. Inert data, not a silent false-success.
- REGISTER D's harness file (varfont.html) DOES NOT EXIST in the repo — it was a throwaway from another session's scratchpad. Budget the probe as work to be WRITTEN.
- REGISTER — add the freshness fact that every bundle figure in this panel depends on: build/ on disk is from 2026-09-12 20:05 and HEAD is 2026-09-13 11:31. The counts happen to be unaffected, but they were read from a different tree than the one being edited.
- REGISTER — add the grep traps: four of the 17 names are substrings of their own replacements (--font-monospace, --font-mono, --font-text, --background-modifier-active), measured as a 16-hit phantom in one file; `grep -c` counts lines and both built stylesheets are exactly one line; and --interface-font/--mono-font already read ZERO in build/, so a zero-hit condition gives no signal for nine of the sites.
- REGISTER — add the dead multiplier inside the very rule the job inspects: theme.css:228 and StyleSetter:2204 read `calc(var(--link-tip-font-size, 0.8rem) * var(--rs-scale, 1))`, but the tooltip is `document.body.appendChild(el)` (src/lib/links/linkTip.ts:81) while --rs-scale is set INLINE on `.rs-inner` (+layout.svelte:10125). The multiplier is permanently 1. Two names are dead on that line.

## Declined — the Boss's to decide, not the panel's
- Whether the Calendar should render Amiri. Three edits are available at CalendarPanel:225/:323 — a plain rename (Amiri turns on, first in the stack, for any user with --cal-font unset), `var(--cal-font, var(--font-text-theme))` (the author's literal intent, the note font), or `var(--cal-font, var(--font-interface-theme))` (byte-identical to today). Amiri was authored deliberately and has never once rendered. Which of the three ships is a typeface decision for two live surfaces, and it is his.
- Whether the lens label sizes are right now that they apply. Restoring the authored 9-15px shrinks most labels from a uniform inherited 14px. 'Correct per the authored design' and 'too small on his second screen' can both be true. If he rejects it, the answer is to adjust the SIZES in a follow-up, never to revert the rename — but the judgement is his.
- Whether the OrgChart connectors should escalate to --background-modifier-border-focus. The correct token by meaning reduces contrast in dark. Picking the heavier token for its pixel value is a taste call, and it belongs to him at the sitting.
- Whether the two note-name labels (.bf-pname NoteButterflyGraph:414, .orr-pn NoteOrreryGraph:828) should take the note font rather than the interface font. One attacker showed .orr-pn renders `plate.lines[0]`, which is '+N' for a cluster — so the 'it is a note name' premise is false there. This is a lens-design question for the art direction, not a rename.
- Whether per-library accent colour should come back as a feature. It was archived 2026-03-29 and nothing has asked for it since.
- Whether --accent-bg should be re-pointed (it aliases a background tier, not an accent tint) and whether .s-result.active should stop rendering identically to .s-result:hover. Correcting the token moves every consumer at once — his call, with its own sitting.
- The risk appetite on shipping 65 hand-edits behind a grep and a shape probe rather than a full computed-style regression suite. The panel names the exposure; the tolerance is his.

## Build plan

Function in hand: PJ-461 — the 17 CSS custom-property names nothing defines, across 22 files. Concept (the horse): every CSS name a component asks for must resolve to the theme the user controls. Not "tidy the variables" — 75 authored lines silently render something the theme cannot reach and the Style Setter cannot move.

STEP 0 — READ THE 45 UNREAD LINES (no commit). Every lens confessed it read ~30 of the 77 and accepted the rest from the register's per-name counts. Between them the attackers found errors in 7 register items and in 14 proposal claims, including three "verified" sentences that were not. Open the remaining lines — SettingsModal's mono cluster, LinkTypesEditor, Mig108UnifyDialog, BaseTab, the Sight files, NoteRadialGraph — and confirm each site's intent against final_table BEFORE the first edit. The base rate on unread assertions in this stack is too high to build on.

STEP 1 — BASELINE (no commit). Run `npm run build` (BOTH vite passes; package.json:8 chains them with &&). PRECONDITION, newly imposed: every artifact under build/ must post-date `git log -1 --format=%ci`. Today's tree fails this (build 2026-09-12 20:05 vs HEAD 2026-09-13 11:31), so the panel's own bundle figures were read from a different tree. Then record the per-name occurrence table with `grep -roE -- "--<name>[^-A-Za-z0-9_]" build/ | wc -l` — recursive, boundary-anchored, across all three emit locations. Save it to the scratchpad; it is the only thing STEP 6 can diff against.

STEP 2 — COMMIT 1: the invisible half (25 lines). All 9 --library-accent strips (with :1930 DELETED, not renamed), livePreview:1884, NoteOrreryGraph:821, StyleSetter:2211 (+ clamp deleted), and all 14 unshowable-and-dead lines (Sight 9, +layout 12293 and 12311, ConstellationMap:924, Mig108UnifyDialog:450). Verification clause: the recursive grep shows those names at zero and every other count unchanged; nothing on screen moves. This commit's claim IS that nothing changed.

STEP 3 — COMMIT 2: the visible renames, main window (18 lines). OrgChart 1326/1338/1355; TasksPanel:229 -> --border; SettingsModal 4001/4030/4034/4072/4078/4135/4162/4183; LinkTypesEditor 264/297; BaseTab:1117; +layout:12527; DigestPane:538; StyleSetter 1982/1984. Verification clause: this is the first commit the Boss can see — but he does not sit yet.

STEP 4 — COMMIT 3: the second screen and the selected-row idiom (27 lines). All remaining --font-sans/--font-text sites in Butterfly, Ledger, Orrery, Radial; livePreview:1908; and the three selected-row sites (+layout:11555, TemplateStudioRow:89, ConstellationMap:924) under ONE color-mix idiom, background only. Named check before the sitting, which no lens raised until an attacker did: the lenses size their text plates from hard-coded character-width constants calibrated against the wrong (inherited 14px) size — NoteButterflyGraph:76 `title.length * 8.4 + 34` and :260 `* 7 + 16`, NoteOrreryGraph:493 `* 3.2`, :522 `* 7.4 + 26`, :572 `* 6.4`. Restoring the authored 9-15px changes every one of those fits. Verify each label still sits inside its plate, box and pill before the Boss sees it.

STEP 5 — COMMIT 4 (contingent, after the sitting): the two Calendar font lines, whichever of the three candidates he rules. Held out of every earlier commit so nothing visible rides in unannounced, and split from the Calendar CELL fix (STEP 3's sibling in the same file) so the fix is not held hostage to the question.

I18N OBLIGATION: NONE. No control is added (Q3 adds no Style Setter row), so no new string enters the app and the 15 locale files are untouched. State this explicitly in the commit — the absence of an i18n change is itself a claim, and it is the tell that Q3 was honoured. If the Boss later orders the tooltip dial, THAT build carries the full obligation: a registry row, `styleSetter.labels.font_size` (already present in all 15 locales — verified), and the pure-UI inspection exemption LAPSES because setVar writes persisted state.

RECORD OBLIGATIONS (SO#11, in the same commits, unprompted): session log; PJ ledger v2.13 -> v2.14 closing PJ-461 with evidence and CONFIRMING its 75/21 figure rather than correcting it; the 13 new filings; orientation v-bump; MoCh. Help files and User Manual: reviewed — no change, because nothing user-facing ships unless the Calendar ruling lands a visible one.

## Verification plan

THE STANDING QUESTION, applied: "if my method were wrong, would this result look different?" Three of the four checks below can come back wrong. The fourth cannot, and is named as such.

1. BUILD ORDER AND FRESHNESS. `npm run build` (both vite passes) BEFORE `cargo build --release` — cargo alone re-embeds a stale build/. Then Stage 0: the installed binary's mtime must post-date the source. New precondition: every artifact under build/ must post-date `git log -1 --format=%ci`.

2. THE NEGATIVE GREP — recursive, boundary-anchored, three locations. `grep -roE -- "--<name>[^-A-Za-z0-9_]" build/ | wc -l` for each of the 17. Pass = 0. THREE traps, all measured in-session, each of which alone makes the naive form worthless:
   (a) SCOPE. 25 of the 27 --font-sans occurrences are in build/assets/screen-B8k-aTeL.css (the second-screen bundle from vite.screen.config.js); only 2 are in build/_app/immutable/assets/. livePreview's declarations are inlined as JS strings in build/_app/immutable/chunks/*.js. A grep scoped to _app/immutable/assets finds 2, reports success, and misses the job.
   (b) SUBSTRINGS. --font-monospace, --font-mono, --font-text and --background-modifier-active are substrings of their own replacements. Measured: plain `grep -o -- "--font-monospace"` returns 26 in the main bundle where the anchored form returns 10 — a 16-hit phantom from --font-monospace-theme. Unanchored, those four can NEVER reach zero on a correct build, and the executor would "fix" a correct build by reverting it.
   (c) LINE COUNTING. `grep -c` counts lines; both built stylesheets are exactly one line. It works as a zero-check and CANNOT produce the companion positive count.
   TWO ZEROS THAT PROVE NOTHING: --interface-font and --mono-font already read 0 across the whole build tree today (measured — Sight's CSS is never emitted). Those nine sites carry no grep signal either way. Say so; do not let them sit inside a sentence implying verification.

3. THE POSITIVE LITERAL — exactly one, chosen to survive minification. `grep -roE -- "600 9px var\(--font-interface-theme" build/ | wc -l` must return 1. Measured today the bundle carries `font:600 9px var(--font-sans)` with NO space after the colon (unspaced `font:` is the only form anywhere in build/), and `600 9px` occurs exactly once in src (NoteOrreryGraph:814) and once in build. Three panel members each proposed a positive literal that FAILS on a correct build: two included a space after `font:`, one anchored a closing paren on a line that renames both of its names. Do not use a paren-anchored form on the 410/134/824 lines. Pair every zero with this positive, or deleting a rule passes the gate.

4. THE HEADLESS-CHROME SHAPE PROBE — YES, ~30 lines, and it is the only instrument here that can disagree with a reading of the source. It already did, twice, in this panel: it overturned "the Calendar rename is a no-op" and it measured that the proposed "byte-identical" Calendar remedy was not. Chrome is installed; there is no puppeteer/playwright in node_modules/.bin, so drive `chrome.exe --headless=new --dump-dom` over a hand-written fixture. Probe the SHAPES, not the shipped stylesheet: (a) a `font:` shorthand with an undefined var inherits family/size/weight — in HTML and inside <svg><text>; (b) a `font-family` LONGHAND with an undefined var and no fallback also inherits (this is what makes NoteOrreryGraph:821 a no-op); (c) the Calendar's bare-`inherit` tail, before and after, with all three candidate replacements; (d) the before/after size+weight table for every lens label, to hand the Boss at Stage 3; (e) two roots, .theme-light and .theme-dark, asserting the colour tokens DIFFER between them — proving theme-awareness rather than a re-frozen literal. REFUSED: running a computed-style harness against the extracted 466KB minified bundle. That requires authoring a synthetic DOM to apply it to — measuring my own fixture, the "test that is a copy of the code under test" shape the Verify-the-Finding law names.

5. WHAT PROVES NOTHING, stated so it is never cited as evidence. svelte-check, vitest and the Rust suite are green on all 77 lines TODAY. There is no stylelint in devDependencies. CSS custom-property names are untyped by spec: a typo renders as silence, which is literally what this job is made of. Run svelte-check; cite it as nothing.

6. /simplify — YES, on the final diff (SO#4). It is also what catches the duplicated-tail error if livePreview:1930 is renamed rather than deleted.

7. SAFETY INSPECTION — EXEMPT, and the exemption is CONDITIONAL and must be stated in the commit rather than silently taken. The diff is CSS declarations inside <style> blocks plus livePreview's baseTheme style object: no write path, no index, no lifecycle, no IPC, no persisted JSON, no frontmatter. The condition: the exemption lapses the instant the diff touches a .ts/.rs contract line or adds a Style Setter registry row (setVar writes the persisted appearance store). Because Q2 FILES the accent_color removal rather than building it, and Q3 adds no control, the condition holds. Whoever builds this checks the diff's file list against that boundary BEFORE invoking the exemption, not after.

8. THE TEST PIPELINE IS NOT OPTIONAL. The sitting is test material: tutorial-auditor -> ui-inspector -> panel -> Boss. Two of this panel's own sitting drafts named surfaces that do not exist ("file-tree connector lines" — FileTree.svelte has no connectors; "Settings -> Backup" — carries none of the 77 lines). Both would have been caught at the inspector. Every surface named in the sitting below was verified in the markup in this session; the inspector verifies it again.

## Sitting plan

CONSTRAINT MEASURED BEFORE PLANNING: `colorScheme` is "light" in ALL NINE of his universes. Every dark-theme claim requires him to switch the theme deliberately — that is a STEP, not an assumption. His daily universe is Eisa Cognitive Knowledge (noteGraphStyle: orrery). `--cal-cell-bg` is unset everywhere; `--cal-font` is set to "Dubai" in seven universes and UNSET in Eisa Universe and Constellation Test.

FRAME BEFORE HE LOOKS AT ANYTHING (one paragraph, plain language): "Seventeen colour and font names in the code were spelled in a way nothing in the theme answers to. Where a line had a spare value written beside it, you have been seeing that spare value — frozen, the same in light and dark. Where it had none, you have been seeing nothing at all. In one case a whole size-and-weight instruction was being thrown away, so labels that were meant to be 9 to 15 pixels have all been drawing at one size. Nothing is being redesigned. Things start obeying the settings you already chose."

STAGE 1 — LIGHT THEME, MAIN WINDOW (his default; nothing to change first).
 1. Organization Chart — the left-dock button whose tooltip reads "Organization Chart", in its Tree view. The connector lines joining the boxes are a flat mid-grey today, the same in both themes. After: a normal pale border, matching the boxes they join. NOT the sidebar file tree — that has no connector lines at all; if you look there you will see nothing change.
 2. Tasks panel — the thin line between rows is nearly black today. After: a normal faint separator. Look in BOTH windows, main and second screen.
 3. Style Setter -> Note graph category, centre preview — the relationship labels and the title take their intended sizes (12px and 14px) and weights for the first time. This is the same repair as the second screen's lenses, on a screen you can see now.
 4. Template Studio — click a row. Today the selected row is exactly the same shade as whatever row your pointer is over, so you cannot tell them apart. After: a soft accent tint.
 5. Bases list — the selected Base gains the same accent tint; today it has no background at all.
 6. Digest panel, the chevron button — hover it. Today nothing happens to its background. After: a hover background appears.
 7. Linked-Universe path popup — the path switches from your normal reading font to your chosen Code font. It is NOT monospaced today, though it was meant to be.
 8. Settings -> Deleted notes — the note text switches from Windows' default typewriter face to YOUR chosen Code font. Do NOT expect the row borders to change: measured, they move by under five units per colour channel and you will not see it.
 (Settings -> Debug -> Boot Performance shows the same Code-font change, but only if a boot report exists; skip it if the panel says "no report yet".)

STAGE 2 — SWITCH TO THE DARK THEME, then repeat in this order.
 1. THE CALENDAR — the headline of the whole job. Today every day cell paints solid white while its text takes the dark theme's near-white — near-white on white. After: the cells take the dark panel colour and the dates are readable. This defect is live on your machine right now and nothing masks it.
 2. Organization Chart connectors — here the change runs the OTHER way: the lines get FAINTER in dark, not lighter. This is the one place in the job to expect "too faint". If you say so, the answer is a heavier border token, and that is your call, not ours.
 3. Tasks panel, Template Studio, Bases, Digest, the path popup — same as Stage 1, confirming each works in both themes.

STAGE 3 — SECOND SCREEN, both themes. Open the Cockpit.
 FRAME THIS BEFORE HE LOOKS, or it reads as a regression: "Every text label in these lenses has been drawing at one size — 14 pixels — because the instruction that set its size was being thrown away whole. The sizes the lenses were designed with, 9 to 15 pixels with some bold, have never once appeared on screen. They do now. Most labels get SMALLER. If any of them is now too small to read, that is a size to adjust, not a fault in the fix."
 Start with the Orrery (your daily universe is set to it), then the Butterfly and the Ledger. Watch for: tick and count labels shrinking and becoming tidier; titles and type names gaining weight; and — the named check — whether every label still fits inside its plate, box and pill, since those boxes were sized for the wrong font size. Also: each lens's centre note title switches to your NOTE font, if you have set a Note font different from your Interface font.

STAGE 4 — THE CALENDAR FONT QUESTION (open Eisa Universe for this; your daily universe has a Calendar font set, so it cannot show the difference).
 The Calendar was written to prefer a bundled Arabic calligraphic face called Amiri. Because of the same spelling fault, that instruction has been invalid since it was written and the Calendar has always borrowed the interface font instead. Fixing the name makes the instruction valid — and Amiri turns on for the first time. Three choices, and it is yours: (a) let Amiri render as originally intended; (b) use your Note font; (c) keep exactly what you see today, the interface font. We will not ship any of the three without your word.

WHAT CANNOT BE SHOWN — disclosed with a number and a reason, never inside a sentence implying it was verified.
 • 9 lines in Sight v6/v7. Both feature flags are off and, measured, that CSS is not even in the shipped app — so neither your eyes nor our automated check can confirm them.
 • 5 lines in the unbuilt "Heartwood" lens. Not offered by any picker. The builder verifies these himself by setting the value directly; you are not asked to.
 • 1 line in the Constellation Map. The app force-disables the Map on every start; the Settings toggle does not survive a restart. There is no way to show you this one.
 • 2 lines on the first-run welcome screen. One renders only when you have zero libraries; the other belongs to a form that no longer exists in the app at all.
 • 12 more lines where the whole point is that NOTHING changes — the dead library-accent layer, a border that already fell through, one Orrery label that already inherits the right font, and a sidebar-width placeholder. These are proven by the automated check, not by your eyes. If you see a change in any of them, something is wrong.

## Filings for the ledger
- PJ-NNN — LibraryAppearance.accent_color is read and discarded: libraries.rs:4580/4597 parses it from an imported Obsidian appearance.json, store.ts:5586 parks it, no frontend reader and no write command exists. Inert data, not a false-success — file as a tidy-up, not an investigation.
- PJ-NNN — Ledger lens traversal-tier colours: NoteLedgerGraph.svelte:156,157,158,159 (FOUR renderings, including the base dot that emerging falls through to — there is no .lt-emerging rule) onto the --link-tier-* chain, against three already-correct references (BacklinksPanel:421-438, OutgoingLinksPanel:277-303, StyleSetter:2184-2187). Token is --link-tier-loadbearing, one word. His Style Setter tier dials are set in two of three universes, so the lens is visibly diverging from his own settings today.
- PJ-NNN — Link-tooltip font-size control: NOT OWED. Reopen only on a Boss complaint naming tooltip text size. If built: unit rem, def 0.8, step 0.05, and the pure-UI inspection exemption lapses.
- PJ-NNN — Dead multiplier in the link-tip font-size declaration: theme.css:228 and StyleSetter:2204 both read `calc(var(--link-tip-font-size, 0.8rem) * var(--rs-scale, 1))`, but the tooltip is appended to document.body (src/lib/links/linkTip.ts:81) while --rs-scale is set inline on .rs-inner (+layout:10125). The multiplier is permanently 1 — no existing control can scale the tooltip.
- PJ-NNN — --accent-bg is misnamed (it aliases a background tier, not an accent tint) AND .s-result.active (+layout:11425) renders identically to .s-result:hover (:11424) because both resolve to --color-base-25. A live pre-existing collision; fixing the token moves every consumer, so it needs its own sitting.
- PJ-NNN — Dead CSS: .w-option-input and .w-option-form (+layout:12305-12311) have no markup user anywhere and are pruned from the shipped bundle. Two rules to delete.
- PJ-NNN — CalendarPanel font chain (CalendarPanel:225,323): the Boss ruling from Stage 4 lands as its own commit. Whichever of the three candidates he picks, the current declaration is invalid and must not be left as it is.
- PJ-NNN — Orphaned sidebar-width control: src/lib/theme/constellationStyleSettings.ts:94 still declares it, generateStyleSettingsCSS (styleSettings.ts:404) has zero callers, and +layout:2459 strips CORE_BLOCK_IDS from stored themes. A shipped-then-unwired control.
- PJ-NNN — THE FOSSIL-KEY CLASS (PJ-461's mirror image, and arguably worse): --font-interface-size is present in all three of his styleOverride maps, is in no current Style Setter catalog, and has ZERO var() readers in src/ — yet it is re-applied to body on every boot forever. Nothing filters styleOverride against the current catalog. PJ-461 is a SITE reading a name nothing writes; this is a CONTROL writing a name nothing reads — a dial the user can touch that does nothing.
- PJ-NNN — Orphaned i18n keys settings.noteGraphStyle and settings.noteGraphStyleDesc exist in all 15 locale files with no component reference; leftovers from the SettingsModal lens picker removed at e8b392d0.
- PJ-NNN — Should normalizeGraphStyle fold UNBUILT ids back to butterfly? cockpitFlag.ts:36 keeps any listed id, so a user who once picked 'Heartwood (living tree) — coming' from the pre-e8b392d0 Settings select has it persisted and their second screen silently mounts a lens no picker offers, permanently, with no notice. A behaviour change, hence its own item.
- PJ-NNN — Lens label box-fit constants (NoteButterflyGraph:76,260; NoteOrreryGraph:493,522,572) size text plates from character counts calibrated against the wrong, inherited font size. Restoring the authored sizes changes every fit; re-derive them once the real sizes apply.
- LESSONS-LEARNED entry — bundle verification method: the app emits to THREE locations (build/_app/immutable/assets/*.css, build/_app/immutable/chunks/*.js, build/assets/screen-*), grep patterns for custom properties must be boundary-anchored because retired names are substrings of their replacements, `grep -c` counts lines on one-line minified CSS, and build/ must be proven to post-date HEAD before any baseline is recorded. Three of four panel members proposed a build proof that would have failed on a correct build.
- LESSONS-LEARNED entry — a register is not evidence. Seven items in this job's own evidence register were wrong (a line count, the Sight tally, two reachability mechanisms, the rem baseline, where the font JS writes, and a tier-chain citation), and every lens that trusted one inherited its error. The panel's value came entirely from attackers re-measuring what the register asserted.

## The complete replacement table

```
COMPLETE REPLACEMENT LIST — 77 distinct lines, 22 files. Legend: [R]=rename, [S]=strip, [L]=literal, [U]=unchanged, [D]=deferred to Boss ruling. "no-op" = provably byte-identical; "unshowable" = fixed but cannot be verified by eye.

--- --font-sans -> var(--font-interface-theme)  [27 occurrences; family does NOT change — body/.second-screen already carry that token; SIZE+WEIGHT begin applying] ---
[R] src/lib/components/NoteButterflyGraph.svelte:405 -> font: 500 11px var(--font-interface-theme)
[R] src/lib/components/NoteButterflyGraph.svelte:406 -> font: 12px var(--font-interface-theme)
[R] src/lib/components/NoteButterflyGraph.svelte:410 -> font: 600 15px var(--font-text-theme)   [BARE — see --font-text below; this line carries TWO names]
[R] src/lib/components/NoteButterflyGraph.svelte:411 -> var(--font-interface-theme)
[R] src/lib/components/NoteButterflyGraph.svelte:412 -> var(--font-interface-theme)
[R] src/lib/components/NoteButterflyGraph.svelte:414 -> var(--font-interface-theme)   [.bf-pname; note-font question DECLINED to Boss, mapping unchanged for now]
[R] src/lib/components/NoteLedgerGraph.svelte:134 -> font: 600 13px var(--font-text-theme)   [BARE; two names on this line]
[R] src/lib/components/NoteLedgerGraph.svelte:135 -> font: 500 12px var(--font-interface-theme)
[R] src/lib/components/NoteLedgerGraph.svelte:136 -> font: 500 12px var(--font-interface-theme)
[R] src/lib/components/NoteLedgerGraph.svelte:137 -> font: 11px var(--font-interface-theme)
[R] src/lib/components/NoteLedgerGraph.svelte:138 -> font: 10px var(--font-interface-theme)
[R] src/lib/components/NoteLedgerGraph.svelte:139 -> font: 14px var(--font-interface-theme)
[R] src/lib/components/NoteOrreryGraph.svelte:814 -> font: 600 9px var(--font-interface-theme)   [.orr-cn — THE positive build literal]
[R] src/lib/components/NoteOrreryGraph.svelte:815 -> var(--font-interface-theme)
[R] src/lib/components/NoteOrreryGraph.svelte:821 -> var(--font-interface-theme)   [NO-OP: font-family longhand, undefined var, no fallback => already inherits the interface font. Its separate font-weight:600 applies today and is untouched.]
[R] src/lib/components/NoteOrreryGraph.svelte:824 -> font: 600 12px var(--font-text-theme)   [BARE; two names on this line]
[R] src/lib/components/NoteOrreryGraph.svelte:825 -> var(--font-interface-theme)
[R] src/lib/components/NoteOrreryGraph.svelte:826 -> var(--font-interface-theme)
[R] src/lib/components/NoteOrreryGraph.svelte:828 -> var(--font-interface-theme)   [.orr-pn renders '+N' for clusters — NOT always a note name; note-font proposal REJECTED]
[R] src/lib/components/NoteOrreryGraph.svelte:829 -> var(--font-interface-theme)
[R] src/lib/components/NoteRadialGraph.svelte:135 -> var(--font-interface-theme)   [unshowable — heartwood lens; builder-verified via a settings value]
[R] src/lib/components/NoteRadialGraph.svelte:139 -> var(--font-interface-theme)   [unshowable]
[R] src/lib/components/NoteRadialGraph.svelte:140 -> var(--font-interface-theme)   [unshowable]
[R] src/lib/components/NoteRadialGraph.svelte:141 -> var(--font-interface-theme)   [unshowable]
[R] src/lib/components/NoteRadialGraph.svelte:142 -> var(--font-interface-theme)   [unshowable]
[R] src/lib/components/StyleSetter.svelte:1982 -> font: 500 12px var(--font-interface-theme)   [.ss-rellabel — MAIN WINDOW, Style Setter "Note graph" preview; IN THE SITTING]
[R] src/lib/components/StyleSetter.svelte:1984 -> font: 600 14px var(--font-interface-theme)   [.ss-reltitle — MAIN WINDOW; IN THE SITTING]

--- --font-text -> var(--font-text-theme), BARE, no inner fallback [3; theme.css:9 defines it unconditionally on :root, so a nested fallback is unreachable] ---
(the three occurrences are on lines 410 / 134 / 824 above — the ONLY three lines in the job where the FAMILY changes, and only if his Note font differs from his Interface font)

--- --font-monospace / --font-mono / --mono-font -> var(--font-monospace-theme) [15] ---
[R] src/lib/components/LinkTypesEditor.svelte:264 -> var(--font-monospace-theme, monospace)   [generic monospace -> his Code font]
[R] src/lib/components/LinkTypesEditor.svelte:297 -> var(--font-monospace-theme, monospace)
[R] src/lib/components/Mig108UnifyDialog.svelte:450 -> var(--font-monospace-theme, ui-monospace, monospace)   [unshowable unless a pre-MIG-108 universe raises the dialog]
[R] src/lib/components/SettingsModal.svelte:4001 -> var(--font-monospace-theme, monospace)   [Boot Performance scorecard, Settings -> Debug]
[R] src/lib/components/SettingsModal.svelte:4030 -> var(--font-monospace-theme, monospace)   [Boot Performance]
[R] src/lib/components/SettingsModal.svelte:4034 -> var(--font-monospace-theme, monospace)   [Boot Performance]
[R] src/lib/components/SettingsModal.svelte:4072 -> var(--font-monospace-theme, monospace)   [Boot Performance]
[R] src/lib/components/SettingsModal.svelte:4078 -> var(--font-monospace-theme, monospace)   [Boot Performance]
[R] src/lib/components/SettingsModal.svelte:4183 -> var(--font-monospace-theme, monospace)   [.deleted-text — Settings -> Deleted notes; the only mono site in that area]
[R] src/lib/lens/BaseTab.svelte:1117 -> var(--font-monospace-theme, monospace)   [path is src/lib/lens/, NOT src/lib/components/ — register A is wrong]
[R] src/lib/editor/livePreview.ts:1908 -> 'var(--font-monospace-theme, monospace)'   [NO fallback today => renders the inherited PROPORTIONAL note font; becomes monospace. LARGER change than the nine above.]
[R] src/routes/+layout.svelte:12527 -> var(--font-monospace-theme, monospace)   [.federation-popup-path; NO fallback today => proportional interface font; becomes monospace. IN THE SITTING.]
[R] src/lib/sight/v6/SightV6.svelte:1764 -> var(--font-monospace-theme, ...)   [unshowable — flag off, CSS not even emitted]
[R] src/lib/sight/v6/SightV6.svelte:1829 -> var(--font-monospace-theme, ...)   [unshowable]
[R] src/lib/sight/v6/SightV6.svelte:1836 -> var(--font-monospace-theme, ...)   [unshowable]

--- --interface-font -> var(--font-interface-theme) [6, ALL unshowable: SIGHT_V6_ENABLED/V7_ENABLED false, CSS absent from build/] ---
[R] src/lib/sight/v6/facetSidebar.svelte:156
[R] src/lib/sight/v6/SightV6.svelte:1361
[R] src/lib/sight/v6/SightV6.svelte:1684
[R] src/lib/sight/v6/SightV6.svelte:1799
[R] src/lib/sight/v6/tour.svelte:134
[R] src/lib/sight/v7/SightV7.svelte:215

--- --library-accent -> STRIP the layer [9, ALL no-op] ---
[S] src/lib/editor/livePreview.ts:1756 -> 'var(--link-color, var(--interactive-accent))'
[S] src/lib/editor/livePreview.ts:1758 -> 'var(--link-color, var(--interactive-accent))'
[S] src/lib/editor/livePreview.ts:1804 -> 'var(--link-color, var(--interactive-accent))'
[S] src/lib/editor/livePreview.ts:1807 -> 'var(--link-color, var(--interactive-accent))'
[S] src/lib/editor/livePreview.ts:1810 -> 'var(--link-color, var(--interactive-accent))'
[S] src/lib/editor/livePreview.ts:1841 -> 'var(--interactive-accent)'
[S] src/lib/editor/livePreview.ts:1842 -> 'var(--interactive-accent)'
[S] src/lib/editor/livePreview.ts:1851 -> 'var(--interactive-accent)'
[S] src/lib/editor/livePreview.ts:1930 -> 'var(--interactive-accent, #6c5ce7)'   [INVERSE SHAPE — DELETE, do not rename; a rename yields a duplicated tail. Not part of the other eight's find/replace.]

--- --bg-primary -> var(--bg)  [4; Tier-3, matching each block's dominant vocabulary. The ledger's "or" is resolved.] ---
[R] src/lib/components/CalendarPanel.svelte:287 -> var(--cal-cell-bg, var(--bg, #fff))   [THE DARK-THEME HEADLINE: --cal-cell-bg is unset in all nine of his universes (measured), so cells paint #fff while the text takes --text (#cdd6f4 dark) — near-white on white. Requires a deliberate theme switch; his colorScheme is 'light' everywhere.]
[R] src/lib/components/CalendarPanel.svelte:291 -> same chain inside the :hover color-mix
[R] src/routes/+layout.svelte:12293 -> var(--bg)   [unshowable: .w-option-card renders only under {:else if $libraries.length === 0}]
[R] src/routes/+layout.svelte:12311 -> var(--bg)   [unshowable AND dead: .w-option-input has no markup user; Svelte prunes the rule]

--- --background-modifier-border-hover -> var(--background-modifier-border) [4] ---
[R] src/lib/components/OrgChart.svelte:1326 -> var(--background-modifier-border)   [Organization Chart tree connectors — light: #555 -> #e0e0e4 (lighter); dark: #555 -> #313244 on #1e1e2e (fainter). Disclose BOTH.]
[R] src/lib/components/OrgChart.svelte:1338 -> var(--background-modifier-border)
[R] src/lib/components/OrgChart.svelte:1355 -> var(--background-modifier-border)
[R] src/lib/editor/livePreview.ts:1884 -> '1px solid var(--background-modifier-border)'   [NO-OP: already falls through. NOT a sitting item — do not group it with OrgChart's three.]

--- --border-color -> var(--border) [2; Tier-3, matching SettingsModal's deleted-notes block which uses --bg-hover at :4152 and --bg-secondary at :4183] ---
[R] src/lib/components/SettingsModal.svelte:4135 -> var(--border)   [VISUALLY ~INVISIBLE: rgba(128,128,128,.25) composited resolves within ~5 units/channel of the target in both themes. Do NOT promise a visible border change.]
[R] src/lib/components/SettingsModal.svelte:4162 -> var(--border)   [same — invisible]

--- SELECTED-ROW CONCERN — one idiom, three surfaces [3] ---
[R] src/lib/components/ConstellationMap.svelte:924 -> background: color-mix(in srgb, var(--interactive-accent) 18%, transparent)   [keep its existing font-weight:600. UNSHOWABLE: store.ts:7429 force-disables the Map on every settings load.]
[R] src/lib/components/TemplateStudioRow.svelte:89 -> background: color-mix(in srgb, var(--interactive-accent) 18%, transparent)   [LIVE + DEFECTIVE today: byte-identical to .ks-row:hover with no other differentiator. IN THE SITTING.]
[R] src/routes/+layout.svelte:11555 -> background: color-mix(in srgb, var(--interactive-accent) 18%, transparent)   [NOT --accent-bg: that resolves to base-25, the same colour as --bg-hover on :11554, whose `color: var(--text-normal)` is identical too. IN THE SITTING (Bases list).]
(background only at all three — MoveDialog's `color: var(--interactive-accent)` is NOT copied)

--- remaining singles ---
[R] src/lib/components/DigestPane.svelte:538 -> var(--background-modifier-hover)   [no fallback today => no hover background at all; one appears. IN THE SITTING.]
[R] src/lib/components/TasksPanel.svelte:229 -> 1px solid var(--border)   [NOT --border-light: base-35 is the HIGHER-CONTRAST line in both themes and the file already uses --border-light at :188/:214 for box borders. Today renders a near-black #222 divider in light. IN THE SITTING.]
[L] src/lib/components/StyleSetter.svelte:2211 -> width: 260px   [DELETE the clamp: clamp(120px,260px,320px) is dead arithmetic. NO-OP.]

--- UNCHANGED, with the reason ---
[U] src/lib/theme.css:228 -> unchanged. --link-tip-font-size is the app's designed undefined-until-set pattern; its eight siblings have controls, it does not. Replacing it with the literal would regress a working family. (The ledger already excluded this line — this is agreement, not a correction.)
[U] src/lib/components/StyleSetter.svelte:2204 -> unchanged, same reason (the mirror of theme.css:228).
[D] src/lib/components/CalendarPanel.svelte:225 -> DEFERRED. `var(--cal-font, 'Amiri','Cairo', var(--text-font, inherit))` is invalid today (bare `inherit` in a family list) so the panel inherits the interface font; ANY valid replacement is a typeface decision. Three candidates, Boss rules: plain rename (Amiri ON), var(--cal-font, var(--font-text-theme)) (note font), var(--cal-font, var(--font-interface-theme)) (byte-identical). No-op in the seven universes where --cal-font is set to 'Dubai'; live in 'Eisa Universe' and 'Constellation Test'.
[D] src/lib/components/CalendarPanel.svelte:323 -> DEFERRED, same declaration on the day popup. Both lines land together in a follow-up commit after his ruling.
[STRUCK] --bg-modifier-border -> 0 live uses anywhere in src/ or static/. Nothing to rename; remove it from the ledger's list of 17 so it does not imply work that does not exist.
[OUT OF SCOPE] src/lib/components/NoteLedgerGraph.svelte:156,157,158,159 -> filed separately (Q6). None is one of the 17: --text-muted, --rel-supports (a Style Setter control var), --interactive-accent and --text-faint all resolve today.

TOTALS: 65 renamed + 9 stripped + 1 literal = 75 edited lines across 21 files (theme.css has only :228 and is untouched) — which is EXACTLY the ledger's figure. 2 lines unchanged, 2 deferred. Of the 75, 17 are unshowable (Sight 9, Radial 5, ConstellationMap 1, +layout 12293/12311) and a further 12 are provable no-ops (the 9 strips, livePreview:1884, NoteOrreryGraph:821, StyleSetter:2211).
```
