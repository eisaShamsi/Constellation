# Session Log — 2026-09-13 (the drain cycle's third session)

**Working on: PJ-461** — the CSS-token class: 17 custom-property names used in 21 files that no
stylesheet or script ever defines, so each site silently renders its literal fallback or nothing
(the class that made the mold-repair dialog black on the Boss's light theme). Its own build and its
own Boss sitting (visual, light + dark). Ledger v2.13 ► NEXT ACTION.

Standing order in force (Eisa, 2026-09-01): consult the panel before any action — anything that
writes code, commits, builds or touches his data goes to the panel first, verdict shown to him;
reads and measurements free; any finding reaches him only after an independent check that could
contradict it; he tests every build before its commit. DRAIN cycle: fix the backlog, no new hunt.

Session opened 11:33 AST (session directory creation time). Read: orientation v4.33 preamble
(v4.33 → v4.29) + §0, §10, §15, §16; HANDOVER-2026-09-13; ledger v2.13 preamble + PJ-461 + PJ-474;
SESSION-LOG-2026-09-12 §3/§3b/§3c; CLAUDE.md; MEMORY.md. Git: pulled, up to date at `b2010378`,
tree clean.

## §1 — The scope re-derived from the tree (the 09-12 synthesis file left with that session)

Method: `var(--NAME[,)]` over `src` + `static` (.svelte/.ts/.css/.js), excluding
`src/lib/components/ConstellationEditor/` (tracked, imported by nothing). **Every per-name count
equals the 09-12 audit's**: font-sans 27 · font-monospace 11 · library-accent 9 · interface-font 6 ·
bg-primary 4 · background-modifier-border-hover 4 · font-text 3 · mono-font 3 ·
background-modifier-active-hover 2 · border-color 2 · text-font 2 · link-tip-font-size 2 ·
background-modifier-active 1 · bg-active 1 · border-faint 1 · font-mono 1 · sidebar-width 1 ·
bg-modifier-border 0. Definitions of any form (`--name:` / quoted name in a `var:` registry or
`setProperty`): **0 for all 18.**

**80 occurrences on 77 DISTINCT source lines across 22 files** (three lines carry two names, so
80 − 3 = 77). **My own first figure of 79 was an arithmetic error**, corrected the moment a
programmatic walk disagreed with my shell arithmetic. **And my charge that the ledger's "75 sites /
21 files" could not be reproduced was itself WRONG** — the panel reconciled it exactly: 75 = 77
minus the two `--link-tip-font-size` lines the ledger itself ruled "keep the literal", and 21 = 22
minus `theme.css`, whose only site is one of those two. The ledger was right and I indicted it
without doing the subtraction. Inventory of every line with its current text: scratchpad
`pj461-edit-inventory.md`.

History (`git log -S'--NAME:'`): none of the 17 was ever defined in `src`/`static` — **except
`--library-accent`**, which the legacy note pane set from the library's appearance
(`if (appearance.accent_color) vars.push('--library-accent: ...')`, commit `64218b50` 2026-03-15;
archived by `fb1e954f` 2026-03-29). The field survives with no reader: `libraries.rs:4580,4597`,
`store.ts:5586` (`accent_color: string | null`); no frontend code reads `.accent_color`.

## §2 — An executed check: what an undefined token inside a `font:` shorthand does

Headless Chrome (`chrome.exe --headless=new --dump-dom`, scratchpad `varfont.html`), a 20px/400
Georgia parent: `font: 600 12px var(--font-sans)` → computed **20px / 400 / Georgia** — the whole
shorthand is discarded, HTML span and SVG `<text>` alike; the same rule with a defined token →
12px / 600 / sans-serif. `background: var(--bg-primary)` with no fallback → transparent.
`var(--border-faint, #222)` → #222 in any theme.

**Consequence the Boss must hear:** every label in the three cockpit lenses (Butterfly, Ledger,
Orrery) and the Style Setter's relationship-graph preview currently renders at the INHERITED size
and weight, not the 9–15 px / 500–600 its author wrote; the rename will make those sizes apply —
a visible change in every lens, light and dark.

## §3 — Reachability of the 21 files (for the sitting's honesty)
- Sight v6 / v7 (11 sites): `SIGHT_V6_ENABLED = false`, `SIGHT_V7_ENABLED = false`
  (`src/lib/sight/engine.ts:146,168`) — unreachable in the app.
- NoteRadialGraph (5 sites): the cockpit's `{:else}` branch; `NOTE_GRAPH_STYLES` lists only
  butterfly/ledger/orrery and normalizes anything else to butterfly (`cockpitFlag.ts:26-36`) —
  unreachable.
- ConstellationMap (1 site): reachable only when `enabledFeatures.constellationMap === true`.
- All other sites are on live surfaces.

## §4 — Panel convened before any code (the SO) — `wf_fed7c5d6-bce`, launched 11:5x AST
Four lenses (theme architect · Boss-sitting/visual regression · concept & scope · verification &
method) → two attackers each (facts; judgements + what was missed) → chair. Ten questions: fix
shape (rename vs Tier-3 aliases) · `--library-accent` and the lost per-library accent · a
`--link-tip-font-size` control or the literal · house name at the Tier-3 sites · OrgChart connector
weight · the Ledger tier dots' scope · the two active-row sites · unreachable surfaces · the
verification and the sitting · any wrong replacement in the table. Verdict recorded below on return.

### §4b — The first panel run FAILED on a usage limit; resumed on Opus 5
`wf_fed7c5d6-bce` returned with **2 of 9 agents completed** — the Fable limit was reached mid-run.
Failed: both remaining proposals (theme-architect, Boss-sitting), all four attackers on the two
completed lenses, and **the chair**. Completed: `propose:concept-scope`, `propose:verification`.
**A partial panel is not a panel** — no ruling was taken from it. The model was switched to Opus 5
and the run RESUMED from the same run id, so the two completed proposals replay from cache and only
the seven failed agents re-run. Recorded because a panel whose agents died must never be read as a
verdict (the standing check: an empty or thin result is a pass only if the agents completed).

### §4c — Independent check running in parallel on the surviving lens's biggest claims
The concept-scope lens asserted, among its reasons, that the per-library appearance loader runs at
boot for every library and that **nothing reads what it stores** — which, if true, also makes a line
of `CLAUDE.md` ("Per-library fonts from `libraryAppearances`") describe a reader that does not
exist. That is a causal/factual claim of exactly the shape the findings-verifier law exists for, so
it went to a verifier (six claims, default verdict REFUTED) before it may be repeated anywhere.

## §5 — The pre-fix BUILD baseline, measured (a check that can fail needs a real "before")
On the existing `build/` (index.html 2026-09-12 20:05), the negative grep over the WHOLE tree for
the 15 renameable names returns **78 occurrences in 4 files** (bundle occurrences, a different count from the 80 source occurrences — the editor theme and the lenses are emitted twice, and the two link-tip sites are not renamed): `build/assets/screen-*.css` 29 ·
`build/_app/immutable/assets/0.*.css` 27 · `build/assets/screen-*.js` 11 ·
`build/_app/immutable/chunks/*.js` 11. Per name: font-sans 27 · **library-accent 18** · font-monospace
9(+3 bare) · background-modifier-border-hover 5 · font-text 3 · text-font 2 · border-faint 2 ·
border-color 2 · bg-primary 2(+1 bare) · sidebar-width 1 · font-mono 1 · bg-active 1 ·
background-modifier-active 1. (Names exceed their source counts because the editor theme and the
lenses are emitted into BOTH the main bundle and the second-screen bundle.)

**Three findings from that measurement, two of which correct the verification lens:**
1. **The baseline is 78, not the 44 the lens reported.** Its own "after must be 0" check is sound;
   its "before" was wrong. Mine is reproducible with the one-line grep above.
2. **`LESSONS-LEARNED` §611's documented bundle-grep command — `build/_app/immutable/` only — sees
   38 of the 78.** The lens's structural point stands and is now measured: the command as written
   would pass while half the sites went unverified. Amend LL in this job's PCS.
3. **`--mono-font` and `--interface-font` return ZERO in the build** — the Sight v6/v7 CSS is
   genuinely not emitted (flags false), confirming register E independently. **But the Radial
   lens's CSS IS emitted** (`rg-hname` present in both screen bundles), so the lens's claim that the
   bundle check "ignores them by construction" is right for Sight and **wrong for Radial** — those
   5 sites must be renamed or the post-build grep cannot reach 0.

## §6 — The BEFORE table, measured from the real built CSS in both themes
Harness (scratchpad `harness.py`): the two built stylesheets concatenated into one page, the theme
class on `<body>` (where the app puts it, `+layout.svelte:2430`), headless Chrome, `getComputedStyle`.

| probe | light | dark |
|---|---|---|
| Ledger name / tick, Orrery label, Butterfly title | **17px / 400, interface stack** (authored 13px·600, 10px, 9px·600, 15px·600 NOT applied) | same |
| Calendar cell background | #fff | **#fff — white boxes on the dark theme** |
| Calendar root font-family | interface stack (Amiri/Cairo NOT applied) | same |
| Tasks row separator | **#222 near-black** | #222 |
| OrgChart connector | **#555** | #555 |
| TemplateStudio selected row vs hover | 240,240,244 vs 240,240,244 — **identical** | 42,42,60 vs 42,42,60 — **identical** |
| Map active row | #e8e8ff | #e8e8ff |
| Welcome card · Digest chevron hover · Sidebar base active | transparent (no-op) | transparent |
| Federation path · Settings KV | inherited UI stack · generic `monospace` | same |
| Link tooltip font-size | 12.8px | 12.8px |

**Two flaws this harness had to survive — both are the "a check that cannot fail is worthless" shape:**
1. **First run: every probe returned the inherited value** and looked like a clean confirmation of
   the shorthand finding. It was measuring nothing — Svelte scopes every component rule with a hash
   class (`.tp-item.svelte-1a9ldsg`) that my bare probe elements did not carry, so no component rule
   applied at all. A harness with no scope class agrees with the hypothesis for the wrong reason.
2. **Second run: `.tp-item` measured as currentColor, not its `#222` fallback** — because TWO
   different components define `.tp-item`, and the lookup took the first hash it found, whose rule
   sets `border:none`. Fixed by selecting the hash of the rule whose BODY contains the token under
   test. Any probe whose class is not unique needs that disambiguation.

## §7 — The independent check on the lens's claims (findings-verifier; default verdict REFUTED)
Six claims, each traced to primary evidence; for three of them the verifier used the compiled
`build/` as a SECOND method that could have disagreed (a TypeScript interface is erased at compile
time, so a runtime reader would have emitted the literal field name — it did not).

| # | claim | verdict |
|---|---|---|
| 1 | nothing in the frontend reads the per-library appearance fields | **CONFIRMED** — `libraryAppearances` (`store.ts:5594`) has four references in all of `src/`: declare, write, import, and a reset on universe switch. Zero readers. Say "zero consumers", not "zero grep hits": the six names DO appear, as the interface declaration. |
| 2 | the loader fans out one IPC per library at boot, fire-and-forget | **CONFIRMED for the main window, REFUTED for the second screen.** `SecondScreenPage.svelte:505` is a `for` loop with `await` inside `loadAllData()`, called from `onMount` and four other triggers — sequential, awaited, **on the critical path of the second screen's first paint**. And it is not only "at boot": `initializeApp()` also runs on every **universe switch**, and N counts the whole federation (the Rust comment records "fires 16× in the boot fan-out"). |
| 3 | `CLAUDE.md:18` describes a reader that does not exist | **CONFIRMED**, with the fair framing: the sentence's *global*-font half IS implemented in both windows; only the per-library clause is unbacked, and it is phrased as a requirement — so it is an **unmet requirement**, not a false statement. |
| 4 | `--library-accent` is orphaned | **second half CONFIRMED, first half REFUTED as worded** — the definition is in `archive/NotePane-legacy.svelte:156` itself (that file IS today's NotePane's predecessor, copied at `fb1e954f`), and "set nowhere" holds for `src/` and the shipped bundle, not repo-wide (two unbuilt `ConstellationEditor/` copies also set it). Write it "set nowhere that ships." Nine live consumers, matching the ledger's 9. |
| 5 | the `* var(--rs-scale, 1)` factor on the link tooltip is inert | **CONFIRMED** — the tip is appended to `document.body` (`linkTip.ts:81`) and `--rs-scale` is set on exactly TWO elements, `.rs-inner` and `.cataloger-queue`, neither an ancestor of `body`; never on `:root`/`html`/`body`, in source or bundle. The lens said "only `.rs-inner`" — incomplete, conclusion unaffected. The behavioural statement worth keeping: **the link tooltip does not follow the right-sidebar text-size slider.** |
| 6 | the Ledger's load-bearing dot borrows the Supports relationship colour | **CONFIRMED**, with three corrections: the precedent comment is at `cockpitGraphData.ts:59-61` (not 52-60); `f.tier` is a **traversal** lifecycle value from `linkLifecycle()`, a pure function of traversal count and staleness — link type is nowhere in it; and **two tiers have no rule at all, `emerging` AND `fresh`**, so a never-traversed link and a lightly-used one are indistinguishable. `.lt-established` borrows the global accent — the same concern one notch milder. |

**What this changes:** claim 2's refutation is a genuine performance finding of its own (a sequential
awaited IPC fan-out on the second screen's first paint, repeated on every universe switch) and does
not belong to PJ-461 — it is a filing. Claim 6's corrections widen the Ledger tier-dot question from
one line to a four-rule block with two tiers missing, which strengthens the case for it being its
own job rather than a rider on a rename.

## §8 — The rename PROVEN before a single file is edited (same rules, real `theme.css`, swapped names)
Four representative rules taken verbatim from source, rendered against the real `theme.css` under a
20px Georgia parent, before and after the rename — headless Chrome, computed values:

| rule | with today's name | with the real token |
|---|---|---|
| `.lg-cn` (Ledger note name) | 20px · 400 · Georgia (inherited) | **13px · 600 · interface stack** |
| `.orr-cn` (Orrery label) | 20px · 400 | **9px · 600** |
| `.cal-root` (Calendar) | Georgia (inherited) | **Amiri, Cairo, then the Note font** |
| `.bp-kv` (Settings mono) | generic `monospace` | **the Code font stack** |

So the job's headline claim is measured, not predicted: the authored sizes and weights have never
rendered, and the rename is what makes them render. **Two consequences the Boss must be told before
he looks:** the Orrery's smallest labels will become 9px, which may be too small on his screen (a
size ruling is a fair follow-up, and must not be pre-empted by quietly changing the authored value);
and the Calendar will change typeface wherever Amiri or Cairo is installed — authored intent from
`CalendarPanel.svelte:225` that has never once rendered.

## §9 — Found while measuring, OUT of PJ-461's scope — five checkouts, and one file that escaped the ignore
`git worktree list` reports **five** working trees: this one, three under `.claude/worktrees/`
(dated 2026-05-09, 2026-05-18, 2026-08-07 — **887 MB** together), and a fifth at
`E:\مشاريع كلاود\Constellation-wtSC` (2026-06-12, detached HEAD), which no document in `docs/`,
`lab/` or `CLAUDE.md` mentions. Working Agreement #2 is explicit: one location, no worktrees,
no alternate checkouts, no parallel paths.

Two consequences, one of them not cosmetic:
- **Each holds a full copy of `src/`.** Any audit that greps from the repo root without excluding
  them silently double-counts or reads months-old code. My own `find .` hit the 2026-08-07 copy
  while checking a test file today.
- **`.claude/worktrees/` is gitignored** (`.gitignore:37`, added by `261e0c89`) **but one file
  inside it is still TRACKED** — `.claude/worktrees/upbeat-proskuriakova/.claude/settings.local.json`
  — because an ignore rule never untracks what is already in the index. Its contents are a Claude
  permission allowlist carrying absolute paths under `C:\Users\ealsh\` and a record of an unrelated
  local task. It is committed to a repository that is pushed to GitHub.

Filed, not fixed: removing checkouts and rewriting the index touches his disk and his repository,
which the standing order sends to the panel, and none of it belongs inside a CSS rename.

## §10 — The candidate tokens resolved in both themes (measured, for the open decisions)

| token / expression | light | dark |
|---|---|---|
| `color-mix(--interactive-accent 18%, transparent)` (the proposed selected-row tint) | purple @ 18% | lavender @ 18% |
| `--background-modifier-hover` | 240,240,244 | 42,42,60 |
| `--accent-bg` | **240,240,244** | **42,42,60** |
| `--bg` = `--background-primary` | 255,255,255 | 30,30,46 |
| `--border` = `--background-modifier-border` | 224,224,228 | 49,50,68 |
| `--border-light` = `--background-modifier-border-focus` | **212,212,216** | **58,58,78** |

**Two consequences, one of which contradicts a proposed replacement:**
1. **`--accent-bg` is byte-identical to `--background-modifier-hover` in both themes.** So mapping
   the sidebar's active base row (`+layout.svelte:11555`) to `--accent-bg` would make the ACTIVE row
   render exactly as a HOVERED row — which is precisely the defect being fixed at the Template
   Studio in the same pass (selected == hover). Local consistency argues for `--accent-bg`; the
   measurement says it reproduces the bug one surface over. This goes to the chair as a dissent, not
   into the build silently.
2. **`--border-light` is the MORE prominent line in both themes**, not the fainter one — so
   `--border-faint` → `--border-light` moves the Tasks separator the wrong way for a name that says
   "faint". `--border` (224,224,228 light / 49,50,68 dark) is the fainter of the two and matches the
   file's own Tier-3 dialect. Confirms the verification lens's correction independently.

## §11 — THE PANEL HAS RULED (`wf_fed7c5d6-bce` resumed: 13 agents, 0 failures)
Full verdict saved to `lab/reports/PJ-461-PANEL-VERDICT-2026-09-13.md`. Ten rulings, 25 register
corrections, 7 declines to the Boss, 13 filings, a 4-commit build plan, a 4-check verification plan
and a 4-stage sitting.

**The decisions:** rename at every site, **zero new Tier-3 aliases** (the tier is for names that once
resolved; minting them would also destroy the only gate that can fail) · strip `--library-accent` at
all 9 sites, deleting `livePreview.ts:1930` rather than renaming it · **no link-tooltip control** —
the two lines stay exactly as authored and the name leaves the job · the three selected-row sites
(sidebar Bases, Template Studio, Map) all take one accent-tint idiom, background only · OrgChart
takes `--background-modifier-border` · the Ledger tier dots are **out of scope**, filed whole as four
renderings · the unreachable surfaces are fixed and disclosed, 17 lines that cannot be shown · the
two Calendar font lines are **held out of the build for his ruling**.

**It corrected me five times, and the corrections matter:**
1. **The ledger's 75/21 was right and I was wrong** (corrected in §1 above).
2. **`1rem ≠ 16px` on his machine.** Both windows set the root size from `interfaceFontSize`, which
   is **17** — so `0.8rem` is **13.6px**, not the 12.8px my §6 table records.
3. **The cockpit labels are 14px today, not "inherited and unmeasured"** — `SecondScreenCockpit.svelte:186`
   sets `.ck { font-size: 14px }`. My harness measured 17px because it had no `.ck` ancestor; that
   number was a fixture artefact, not the app's before-state.
4. **The font FAMILY does not change at the 27 `--font-sans` sites** — body and `.second-screen`
   already carry `--font-interface-theme`. Only size and weight begin to apply. My §8 table implied
   a family change that will not happen except on three lines, and only if his Note font differs.
5. **The `--font-*-theme` tokens are set on `document.body`, not the root** (the local variable is
   named `root` and is body) — so my register's description of the write site was wrong.

**It also reached my §10 dissent independently:** `--accent-bg` is byte-identical to the hover token,
so the sidebar's active Base row would have rendered exactly as a hovered row — and it went further
than I did, noting that `:11554` already sets the same `color`, making the collision total. Its
ruling puts all three selected-row sites on one tint instead.

**What it declined, as his:** the Calendar typeface (three candidates, all measured); whether the
restored 9–15px lens labels are too small; whether the OrgChart connectors should escalate in dark;
the per-library accent as a feature; and the risk appetite of 65 hand-edits behind a grep and a
shape probe.

## §12 — STEP 0 done (read-only): the chair's corrections re-verified in the source by me
Not taken on the panel's word — each read at the line:
- `document.documentElement.style.fontSize = (s.interfaceFontSize || 14) + 'px'` in **both** windows
  (`+layout.svelte:2544`, `SecondScreenPage.svelte:530`). So `1rem` is his interface font size, not
  16px, and my §6 figure of 12.8px for the tooltip is wrong in principle. The chair measured his
  setting at 17, giving 13.6px; I have not re-read his settings file myself, so that number is
  theirs, not mine.
- `body { font-family: var(--font-interface-theme) }` (`+layout.svelte:11172`) and
  `.second-screen { font-family: var(--font-interface-theme, …) }` (`SecondScreenPage.svelte:1480`)
  — **confirmed**, so the 27 `--font-sans` sites change size and weight only, never family.
- `.ck { … font-size: 14px }` (`SecondScreenCockpit.svelte:186`) — **confirmed**. Every cockpit lens
  label renders 14px today; my harness's 17px was a fixture artefact.
- `livePreview.ts:1930` is `var(--interactive-accent, var(--library-accent, #6c5ce7))` — the
  **inverse** shape; renaming it would emit a duplicated tail, so it is a deletion. Confirmed.
- `livePreview.ts:1884` already falls through to `--background-modifier-border`; a no-op. Confirmed.
- `StyleSetter.svelte:2211` is `clamp(120px, var(--sidebar-width, 260px), 320px)` — dead arithmetic
  that always yields 260px. Confirmed.
- `NoteOrreryGraph.svelte:821` is a `font-family` **longhand** with no fallback, so it already
  inherits the interface font; only its separate `font-weight: 600` is live. Confirmed no-op.

**The table reconciles exactly:** 65 renamed + 9 stripped + 1 literal = **75 edited lines across 21
files**, 2 unchanged, 2 deferred to his ruling — which is the ledger's own 75/21, arrived at from
the other direction.

**STOP POINT.** Everything up to the first source edit is done. The next step is `npm run build`
plus 75 line edits — a build and a write, which the standing order sends to him with the panel's
verdict before it proceeds. Nothing in the working tree is modified: `git status` shows only this
log and the verdict record.

---

# BUILD — the Boss said "Proceed" (2026-09-13)

## §13 — Step 1: the baseline, on a build proven to post-date HEAD
`npm run build` (both vite passes) at **13:13:32**, HEAD at **11:31:32** — the freshness precondition
the panel imposed, because every bundle figure it quoted had been read from a tree a day older.
Per-name occurrence counts recorded to the scratchpad (`baseline-counts.txt`); the positive literal
`600 9px var(--font-interface-theme)` read **0**, and the bundle carried `font:600 9px var(--font-sans)`
unspaced, exactly as the panel measured.

## §14 — Step 2–4: the 73 edits, each with a precheck that could have failed
One line-targeted pass: for all 73 edits the script first asserted the expected old text is present
at that exact line and **aborts the whole run on any mismatch**. All 73 matched. Result:
`73 insertions(+), 73 deletions(-)` across **21 files** — no stray edit, no reflow, no line-ending
churn. Source now reports **zero** occurrences of 15 of the 17 names; the two that remain are the
deliberate ones (`--text-font` ×2, the deferred Calendar lines; `--link-tip-font-size` ×2, kept).

**The panel's named pre-sitting risk, measured and cleared.** The lenses size their text plates from
character-count constants (`title.length * 8.4 + 34` and four siblings) calibrated against a size
that never applied. Measured in headless Chrome against the real strings, English and Arabic:

| plate | before | after | verdict |
|---|---|---|---|
| Butterfly title box | 121–221px in a 177–300px box | 134–237px | **fits, every sample** |
| Orrery centre pill | 121–221px, one sample within 9px of the edge | 107–190px | roomier |
| Butterfly hover plate | 121–221px | 113–205px | roomier |
| Orrery hover plate | one sample **TIGHT** at 121.5 vs 124.8 | 107.1 | **tight case cleared** |

Only the Butterfly title grows (14px/400 → 15px/600); everything else shrinks, and the two fits
that were tight before are now comfortable. **The constants were calibrated for the authored sizes
all along** — restoring the sizes brings them into alignment rather than out of it.

**A third fixture bug, caught by the same question.** The first fit-measurement returned *identical*
widths for 14px and 15px — impossible. The font stack contains `'Segoe UI'`, and I had written it
into a double-quoted HTML attribute, so the quotes closed the attribute and both cases silently fell
back to the page default. Before/after being identical was the tell. Re-run from a stylesheet.

## §15 — The verification gates (the panel's four, plus the two suites)

**1. Negative grep — recursive, boundary-anchored, whole build tree.** The panel's three measured
traps were all honoured: the lens CSS ships to a SECOND bundle (`build/assets/screen-*`), four of the
names are substrings of their own replacements, and `grep -c` counts lines on one-line minified CSS.

| name | before | after |
|---|---|---|
| `--font-sans` | 27 | **0** |
| `--library-accent` | 18 | **0** |
| `--font-monospace` | 12 | **0** |
| `--background-modifier-border-hover` | 5 | **0** |
| `--font-text` · `--bg-primary` | 3 · 3 | **0 · 0** |
| `--border-color` · `--border-faint` · `--background-modifier-active-hover` | 2 each | **0** |
| `--font-mono` · `--background-modifier-active` · `--bg-active` · `--sidebar-width` | 1 each | **0** |
| `--text-font` | 2 | **2 — the deferred Calendar lines, by design** |
| `--mono-font` · `--interface-font` · `--bg-modifier-border` | 0 | 0 — **no signal either way** (Sight's CSS is never emitted; these nine sites carry no grep evidence and must not be described as verified) |

**2. Positive literal: 0 before → 1 after.** Without it, deleting a rule would pass the negative gate.

**3. Computed-style probe, both themes, against the real built stylesheets** — every claim the sitting
will make, measured rather than predicted:

| surface | before | after |
|---|---|---|
| Ledger name · tick | 17px/400 · 17px | **13px/600 · 10px** |
| Orrery label · Butterfly title | 17px/400 | **9px/600 · 15px/600** |
| Calendar cell, DARK | **#fff** (white boxes) | **30,30,46** |
| Tasks separator | #222 near-black | 224,224,228 · 49,50,68 |
| OrgChart connector | #555 both themes | 224,224,228 · 49,50,68 |
| Template Studio selected vs hover | **identical** | accent tint vs 240,240,244 — **distinct** |
| Sidebar Base active · Digest chevron hover | transparent (no-ops) | accent tint · hover colour |
| Federation path · Settings mono | inherited / generic `monospace` | **the Code font stack** |
| Calendar font · link tooltip | unchanged | **unchanged — deferred and kept, as ruled** |

**A fourth fixture bug, same shape as the other three.** The first after-run reported the Digest
chevron and the sidebar Base row still transparent. The code was right; my probe still injected the
*retired* token inline. Fixed, re-measured, both now paint.

**4. `svelte-check`: 1634 files, 0 errors, 268 warnings — byte-identical to the pre-edit tree**,
measured by stashing the diff and re-running. No warning was introduced.

**5. `vitest`: 87 files, 1008 tests, all passing** — including `tests/pj-114/linkTipCss.test.ts`,
whose "wires every Style-Setter control" case asserts the `--link-tip-font-size` line this job
deliberately did not touch.

## §16 — `/simplify`, consistency angle: seven findings, three verified by me, ONE changes the sitting
The reviewer first re-verified the claim the whole fix rests on: **all nine replacement tokens are
genuinely defined in `theme.css`** (`--font-*-theme` on `:root` at :8-10; the rest on
`.theme-light, .theme-dark` at :81, :85, :87, :108, :159, :163). It also noted Tier 2 and Tier 3 are
declared **on the same selector at the same specificity**, so `--bg` versus `--background-primary`
is a pure vocabulary choice with zero cascade difference — which retires the panel's Q4 as a matter
of taste, exactly as it ruled.

**F3 — VERIFIED BY ME, and it makes one line of the sitting false.** `DigestPane.svelte:538`: the
chevron's hover now paints `--background-modifier-hover`, but `.dg-chev-btn` is a **child** of
`.dg-note` (markup :334-336) and `.dg-note:hover` paints **the same token** (:519-521). Hovering the
chevron always hovers the parent, so it is the same colour over the same colour — **no visible
feedback**. The panel's sitting plan says "hover it … a hover background appears." That claim is
wrong and must not reach him. **Disposition:** keep the rename (it retires an undefined name and is
correct), **reclassify the site as a no-op, strike it from the sitting**, and file the invisible
chevron hover for a ruling — the block's own grammar for child feedback is non-background (the name
underlines, the headline shifts colour).

**F2 — VERIFIED BY ME, disclosed rather than fixed.** `StyleSetter.svelte:1984` `.ss-reltitle` is
the preview mimicry of the Butterfly's `.bf-title` (its sibling pair `.ss-rellabel`/`.bf-flank` both
took the interface font). After the rename the title pair diverges: preview on the interface font,
live lens on the note font. **Not fixed**, because it is the same "which typeface does this label
take" question the panel explicitly declined as his — and because the two already differ in size
(14px vs 15px), so the mimicry was never exact. **Filed, and disclosed in the sitting**, since he
sees this preview at Stage 1.

**F1 — filed, not fixed.** `NoteRadialGraph.svelte:141` renders the hub note name on the interface
font while its three sibling lenses render the same datum on the note font. Faithful to what the
source said, and invisible today because that lens is unreachable. Same declined question.

**F4/F5/F6 — SKIPPED, with a reason that is the job's own purpose.** Five renamed lines are now bare
`var()` where their file's local style carries a literal fallback. The fallbacks those lines used to
carry were `#222`, `rgba(128,128,128,.25)` and `#7c3aed` — **frozen, theme-blind literals, which are
precisely what this job exists to remove**. Re-adding them to satisfy a local style would walk the
fix backwards. The bare form is also the repo-wide norm (OrgChart alone has 112 bare uses).
**And the exposure I cited for them was wrong**, refuted independently by two reviewers: there is no
unthemed paint window at all — `class="theme-light"` is baked statically into BOTH entry documents
(`src/app.html:11`, `static/screen.html:13`) and the runtime swap is a synchronous remove-then-add,
so a bare `var()` on a theme token cannot render unresolved. The skip stands on the frozen-literal
ground alone.

**F7 — pre-existing variance**, not introduced here. No change.

**Clean, per the reviewer:** the sweep leaves **no file speaking two dialects** of a renamed token;
the font vocabulary is now exactly three names everywhere; the `--library-accent` strip is a genuine
no-op (0 definitions, 0 `setProperty` references); `OrgChart` matches all 24 of its neighbours; the
Calendar and welcome-card `--bg` picks match their blocks' own grammar; and `StyleSetter:2211`'s
`clamp(120px, 260px, 320px)` → `260px` is byte-identical.

## §17 — `/simplify`, the other three angles: one fix applied, the rest filed
**APPLIED — the only code change from the review.** The diff had *added* an unreachable `, monospace`
fallback at two sites that were bare before (`livePreview.ts:1908`, `+layout.svelte:12527`), which
also made `livePreview.ts` contradict itself 160 lines apart (`:1748` bare, `:1908` with a tail).
Both are now bare, matching their own pre-diff shape and the file's house form. **This is the one
finding that was cleaning up churn the diff itself introduced**, rather than pre-existing style.
Diff unchanged at 73/73 across 21 files.

**A claim about that line, corrected before it could spread.** One reviewer described
`livePreview.ts:1908` as inline code spans in the editor rendering in the wrong font. **It is not** —
`:1908` is `.cm-lens-error-msg`, the error text inside a Base/lens block. Inline code spans are
`.cm-md-code` at `:1748`, which already used `--font-monospace-theme` and was never affected. Read at
the line before repeating.

**Filed, not fixed** (each is real, none belongs in a rename commit):
- **`ConstellationMap.svelte:862`** — two lines above the row I fixed, a sibling "accent-tinted active
  state" is a frozen `rgba(124,58,237,0.1)`, the app-default accent hard-coded. Same file, same
  concept, one theme-aware and one not. A frozen literal is a different defect class from an
  undefined name, and this surface is force-disabled, so it is a filing rather than a rider.
- **Row selection now has four strengths app-wide** — 12% (`LinkTypesEditor:317`), 15%
  (`+layout:11328`), 18% (`MoveDialog:185` + this job's three), 20% (`FileTree:243`). The suggested
  mitigation was one comment at the MoveDialog origin; **skipped deliberately**, because it would add
  a 22nd file and the exemption verdict below was verified against exactly these 21.
- **Five coexisting spellings of the monospace token's fallback tail** (bare ×6, `, monospace` ×19,
  and three others). Normalizing is its own pass.
- **A dead parallel style registry** — `constellationStyleSettings.ts` (~120 controls),
  `StyleSettingsPanel.svelte` and `generateStyleSettingsCSS` have no live consumer; only
  `CORE_BLOCK_IDS` is imported, to filter blocks out. It still advertises the `sidebar-width` control
  the live registry deliberately removed. **It matters to this job's method**: it is a
  plausible-looking source of custom-property names, exactly the input a "which names are defined?"
  audit reads.
- **`livePreview.ts` repeats `var(--link-color, var(--interactive-accent))` five times**; the file's
  own house pattern is module-level constants.
- **24 near-identical SVG text rules across the four lens files**, no shared stylesheet — this job is
  the evidence, since one token change cost 24 edits.

**Two reviewers disagreed, and I kept the panel's pick.** On `SettingsModal.svelte:4135/4162`, one
reads the file (45 uses of `--background-modifier-border` vs 2 of `--border`) and calls Tier-3 wrong;
the other reads the block (the `.deleted-*` block is self-consistently Tier-3: `--bg-hover`,
`--bg-secondary`) and calls it right. A later reader arrives at the block, not the file census, and
both tiers resolve identically. Kept, with the disagreement recorded rather than hidden.

**DOC DRIFT the review caught, and it is mine to fix at the close:** ledger v2.13 still prescribes
`--border-faint` → `--border-light` and `--bg-active` → `--accent-bg`, neither of which shipped. The
SO#9 v2.14 bump must carry the panel's corrections or the ledger contradicts the code.

## §18 — The Safety-Inspection exemption: claimed, and EARNED on evidence
An adversarial agent was told to assume the exemption FAILS and make the diff prove otherwise. It
returned **EXEMPT**, and its method could have disagreed at every step:
- All 62 Svelte-side changed lines fall inside each file's single `<style>` block, verified
  boundary-by-boundary; all 11 `livePreview.ts` lines fall inside the object literal passed to
  `EditorView.theme(...)` (`:1721`–`:2068`).
- Grepping every ADDED line for `invoke(`, `emit(`, `listen(`, `await`, `function`, `=>`, `$effect`,
  `$state`, `onDestroy`, `localStorage`, `JSON.` returns **zero**.
- **Editor lifecycle closed:** `livePreviewTheme` is a static `export const` (0 arrow functions or
  `function` in that range), referenced by three consumers and never rebuilt — the compartment
  reconfigure at `NotePane.svelte:1096` is guarded so it fires only on the Live-Preview toggle.
  **Zero allocation on the keystroke path.** And **no selector key changed** — only right-hand
  values — so the decoration↔theme class pairing is intact.
- **Persisted state closed four ways:** the three `StyleSetter` lines are inside the `<style>` block
  (1808–2220), not the control registry (which lives above 1808); none of the 15 retired names is a
  registry key; `setStyleVar`/`setStyleVars` have no caller outside their own module; and the one
  read-back (`getComputedStyle(document.body).getPropertyValue`) reads *declared* properties, while
  **no changed line declares a custom property** — every one consumes via `var()` on a right-hand
  side.
- **Measured against the Boss's real data**, not argued: his three `settings.json` files carry 174,
  174 and 92 `styleOverride` keys, and **none of the retired names appears in any of them**.

**Verdict: a diff-scoped safety inspection is not required before this commit**, and that claim goes
in the commit message with its reason, per the standing order.

**One labelling correction it earned:** `StyleSetter.svelte:2211` is **not** a rename — it is a var
*removal* (`clamp(120px, var(--sidebar-width, 260px), 320px)` → `260px`), provably computed-value
identical. **Its own replacement count was wrong too** — it said "72 renames + 1 constant-fold",
which folds the nine `--library-accent` strips into "renames". The accurate breakdown, and the one
the commit message carries: **63 renames + 9 strips + 1 constant-fold = 73 edited lines.**

## §19 — Final state of the build, after the review's one fix
Rebuilt at **13:57:49**, HEAD **11:31:32** — freshness precondition holds. Re-verified end to end:

| gate | result |
|---|---|
| negative grep, whole build tree, boundary-anchored | **only `--text-font` = 2** (the two deferred Calendar lines). All 15 other names **0**. |
| positive literal `600 9px var(--font-interface-theme)` | **1** (was 0) |
| `svelte-check` | 1634 files, **0 errors**, 268 warnings — identical to the pre-edit tree |
| `vitest` | 87 files, **1008 tests, all passing** |
| computed styles, both themes, real built CSS | every after-value as ruled; the Calendar typeface and link tooltip correctly **unchanged** |

Diff stands at **73 insertions, 73 deletions, 21 files** — 63 renames, 9 strips, 1 constant-fold.
The release binary is building for his sitting (the frontend was rebuilt first, so `cargo` embeds
the new bundle rather than re-embedding a stale one).

**Not committed.** His pass is what gates the commit, and the sitting is in the auditor → inspector →
panel pipeline now.

## §20 — Stage 0: the binary he will test on, and the limit of what I can prove about it
**Release binary: `src-tauri/target/release/constellation.exe`, 2026-09-13 14:08:15, 96,330,752 bytes.**
Built on the first attempt (no `LNK1104`), with the frontend rebuilt FIRST so cargo embeds the new
bundle rather than re-embedding a stale one.

**A check that had no power, discarded rather than reported as a pass.** Grepping the exe for
`font-interface-theme` returns 0 — but so does `moldRepair`, a string that certainly exists in the
frontend. Tauri embeds `../build` compressed, so **no plain-text grep of the binary can distinguish a
fresh embed from a stale one**, and reporting that zero as "clean" would have been exactly the
worthless check this project keeps paying to learn about.

**What I can actually prove, and it is a chain, not a timestamp:**
1. The **bundle** in `build/` is verified to carry the fix — positive literal 1, negative grep clean.
2. Cargo's fingerprint for the app crate (`constellation-8ad3fe57a396e9af`) was **invalidated and
   regenerated after 13:57**, i.e. it saw the changed `frontendDist` assets — it is the only crate
   fingerprint touched in that window.
3. The exe was relinked at **14:08:15**, after both.

**What I cannot prove:** I cannot read inside the compressed embed. The final confirmation is his
own screen — which is what the sitting is for, and the lens labels are an unmissable tell.

## §21 — The sitting, through its gates: the auditor corrected the brief I gave it
The `tutorial-auditor` verified every surface in the source AND cross-checked the claims about HIS
setup against his live universe files, which is the part a brief cannot assert for him. It changed
two things I had handed it:

1. **It dropped a step I had asked for.** The Linked-Universe path popup renders only when a Linked
   Universe FAILS to attach — and it found that **none of his universes has one attached at all**
   (no `universe.json` with children anywhere under `E:\Constellation Universes` or `E:\`). Asking
   him to open it would have been an instruction he could not follow. Moved to the disclosure list.
2. **It confirmed three data preconditions** rather than assuming the panels would have content: his
   daily universe has 6 notes with unchecked task lines, 5 `.base` files, and at least two
   deleted-note entries carrying retained text — so every step it wrote has something to click.

**One finding of its own that changes a disclosure:** his **Interface font and Note font are both
empty (System Default), so they are identical today.** That makes the Stage-3 typeface disclosure
moot — and it also means the `/simplify` F2 divergence (the Style Setter preview title on a
different font from the live lens) is **invisible on his machine right now**. It stays filed, but it
is not something he can see, and the sitting says so instead of implying he should look for it.

It also flagged one of its own claims as **inherited rather than newly verified** (the Radial lens's
picker gate) — an honest self-report, and the inspector was told to verify it or reject it.
Sent to `ui-inspector` with that instruction plus one suspicious citation (a single `en.json` line
cited for two different labels).

## §22 — The inspector REJECTED the draft: five findings, four accepted, one contested and refuted
This is the gate earning its keep for the third time in this session.

**Accepted and fixed:**
1. **Stage 4 was inventing a cause.** It said a *misspelling* had disabled the Calendar's Arabic face
   and that this build fixes it. Both halves wrong: `'Amiri'` is spelled correctly and is genuinely
   bundled, and **this build deliberately does not touch those two lines**. The real mechanism is
   that the declaration is invalid at computed-value time (a bare `inherit` cannot sit inside a
   font-family list), so the whole line is discarded. Rewritten as what it is: a decision, with
   nothing in this build changing it.
2. **The count.** "Seventeen names, this build corrects the seventeen" — it corrects **15**; one is
   deferred for his ruling, one is deliberately kept.
3. **A citation** that used one `en.json` line for two different keys. Evidence trail only.
4. **"None of your universes has a Linked Universe attached" was FALSE** — the auditor's own
   correction to my brief was wrong. `Eisa Universe` links to two; `كون عيسى` links to one.

**And the inspector's own correction was wrong too, which I found by checking the disk.** It proposed
"the ones attached currently resolve without error." They do not: `كون عيسى` links to
`…\Two universe UNIVERSE\Two Universe UNIVERSE`, **which does not exist**. So the warning path is
*reachable*, and the step the auditor deleted returns as a conditional Step 7 — a real visible change
(the path row moves to the Code font) with an honest "if no badge appears, skip and say so."

**CONTESTED, with evidence — the Digest chevron.** The inspector ruled it "a real, visible, reachable
fix" because the old rule named an undefined property with no fallback, so the button painted nothing.
That reads the rule in isolation. `.dg-chev-btn` is a **child** of `.dg-note` (markup :334-336), and
`.dg-note:hover` paints **the same token** the chevron's new rule paints (:519-521 vs :537-540) — so
hovering the chevron always hovers the row, and the user saw that grey before and sees the same grey
now. Sent back for an explicit ruling rather than quietly overridden.

**The completeness finding was accepted and answered with an exact reconciliation** — every one of
the 73 lines assigned to a bucket, arithmetic checked programmatically, zero unaccounted:

| disposition | lines |
|---|---|
| in the sitting, visible and tested | **30** |
| visible but NOT tested, each disclosed with its condition | 9 |
| measured no-ops | 15 |
| unreachable surfaces | 18 |
| conditional (the Linked-Universe path) | 1 |
| **total** | **73** |

## §23 — SO#2 (help files + User Manual): checked, and the answer is "no change", stated not skipped
Searched the User Manual and all of `docs/help.uConstellation.World/` for any statement of a colour,
font or contrast fact about the surfaces this job touches. Three hits, none invalidated:
`User Manual.md:736` is file-tree multi-select, `:1935` describes the Style Setter's **Panels**
category (a different family of tokens), and `Notes Management.md:238` describes the file tree's
accent bar. **No user-facing document asserts anything this build makes untrue**, and no document
describes the old frozen values as intended behaviour.

PJ-461 therefore carries **no help or manual obligation** — recorded explicitly, because a silently
skipped SO#2 is indistinguishable from a forgotten one.

## §24 — Inspection round 2: one finding, my contest UPHELD, and a claim of the inspector's that is false
**It rejected again, on exactly one finding — and the finding is real.** Stage 1's header offered the
Boot Performance timestamp as something to look at, while the accounting placed all five Boot
Performance lines in the "visible but NOT tested" bucket. It proved the contradiction with
arithmetic: it reconstructed the 30-line "tested" bucket from the diff itself
(OrgChart 3 + Tasks 1 + StyleSetter 2 + TemplateStudio 1 + Bases 1 + Deleted-notes 1 + Calendar 2 +
Butterfly 6 + Ledger 6 + Orrery 7 = 30) and showed it only closes if the timestamp is NOT tested.
One sentence said "look at this now" and another said "you cannot reliably look at this." **Fixed by
deleting it from Stage 1** — Boot Performance now appears once, in the disclosure.

**My contested point was upheld.** It re-ruled the Digest chevron a **no-op, confirmed empirically**,
and dropped its round-1 claim that the hover was a real visible fix.

**It verified the two hardest things independently, and its method could disagree:** it reproduced the
Calendar mechanism in headless Chrome (that declaration computes to the inherited family; swapping the
inner fallback for a real token computes to "Amiri, Cairo, …"), and it reproduced the shorthand
collapse (an undefined `var()` in a `font:` shorthand takes size, weight AND family from the parent —
it used a deliberately absurd parent font to make the failure unmistakable). It also re-walked all 21
files and confirmed the 73-line accounting file by file.

**And it asserted one thing that is false.** In its verified list it wrote that `--library-accent`
"IS legitimately set/used in `NotePane.svelte`". I checked: `grep` for that name in `NotePane.svelte`
returns **nothing at all**; no definition of any form exists anywhere in `src/`; and after this build
there are **zero** remaining `var()` uses. The only assignment in the repository is
`archive/NotePane-legacy.svelte:156`, imported by nothing. **This is not bookkeeping** — if the token
really were set in NotePane, stripping it from the editor theme could change the editor's link
colour, and the "nine proven no-ops" line in the Boss's disclosure would be false. Sent back for an
explicit ruling rather than left as a contradiction in the record.

**That is the third time in this session an agent asserted something checkable and wrong** (the
panel's "75 is not reproducible", the auditor's "no Linked Universe attached", and now this) — each
caught by going to the source. The gates are worth their cost precisely because I do not take their
output on faith either.

## §25 — Inspection round 3: APPROVED, and it retracted its own false claim
**`--library-accent` — ruled INCORRECT, retracted in its own words.** Its round-2 justification was
fabricated: the token appears nowhere in `NotePane.svelte`, has no definition anywhere in `src/`, and
the only assignment in the repository is the archived legacy pane (`fb1e954f`, March), imported by
nothing. **The disclosure the Boss receives — "9 proven no-ops" — is TRUE**, but for the right reason
(the token never resolved in any shipping scope), not the one it had given.

**A weakness in MY process, which it disclosed rather than papered over.** It could not read the
tutorial: the draft existed only in my context, not on disk, so its round-3 approval of the
Boot-Performance fix was **triangulated from the diff and my session log, not a direct read**. It said
so plainly instead of claiming verification it did not have. **Fixed:** the tutorial is now written to
`lab/reports/PJ-461-BOSS-TEST-2026-09-13.md`, and the panel wrap reads that file — the first direct
read of the actual document.

## §26 — The panel wrap (`wf_f9ed10a9-a90`), the last gate before he sees it
Three lenses, each adversarially refuted, then a chair: **can he follow it** (every pre-state
reachable from the previous step, no ambiguity, no step that sends him away from a screen a later one
needs) · **is anything in it false** (every tooltip, label, route, every "until now" claim checked
against `git show HEAD:`, every "after" claim against the working tree, the 73-line arithmetic, and
the claims about his own data) · **what is missing or misleading by omission** (any visible change
absent from the accounting, anything called a no-op that he could see, any risk understated).

The lenses were handed **the two false claims the gates already produced** — so they know to treat
every confident assertion, mine included, as checkable.

## §27 — THE PANEL WRAP RULED **FIX-FIRST**, and it caught what three inspection rounds missed
Seven blocking items. **I verified every one myself before acting; all seven are real.** This is the
single most valuable gate of the session, and it is the one CLAUDE.md added last.

**1. A REAL CODE DEFECT IN MY OWN DIFF — Template Studio's fix did not work.**
`.ks-row:hover` is specificity (0,2,0); `.ks-row-sel` was (0,1,0). **Hover wins.** So the selected row
still repainted to plain hover grey under the pointer — *the exact symptom the fix exists to remove*,
and true the instant he clicks, because his pointer is on the row he just clicked. The bug was latent
before: the old fallback resolved to the SAME colour as hover, so losing the cascade was invisible.
**My change made the colours differ and thereby exposed it.** Fixed: `.ks-row.ks-row-sel` (0,2,0,
later in source, wins), with the reason in a comment. Checked the two sibling surfaces — `.ws-base-item.active`
and `.cmap-result-row.active` are both (0,2,0) and tie-break correctly; only this one used a bare
single-class selector.
**Why my own harness could not catch it:** it measured the selected row and the hover row as separate
elements. It never applied both states to one element — so it could not disagree with me. That is the
same flaw as the four fixture bugs earlier, in its most expensive form.

**2. Step 1 tested NOTHING — and the code says so itself.** The three changed OrgChart rules
(`.tree-vline`, `.tree-hline`, `.tree-children::before`) have their markup at `:1138`, `:1139`, `:1225`
— inside the branch gated on `sidebarMode === 'skyview'`. Every assignment to `sidebarMode` in the app
is `'tree'` or `'digest'`; **nothing ever assigns `'skyview'`**, and `OrgChart.svelte:259` states it in
its own comment: *"is presently unreachable … nothing assigns that mode."* The dock button mounts
`fullscreen={true}`, whose connector is `.oc-org-vline` — **not in this diff at all.** He would have
looked at an unchanged screen and passed a step that verified nothing, and the stated failure mode
could never fire. **Step 1 and Stage 2 item 2 are cut; those 3 lines move to the unreachable bucket.**

**3. Step 2 sent him to the wrong panel.** `TasksPanel` mounts in the RIGHT sidebar (`+layout:10345`)
and on the second screen. The dock button "Open global tasks" opens `GlobalTasksView` — a different
component, untouched by this diff. Retargeted, and the definition corrected: that panel lists the
unfinished checkboxes **in the note you have open**, not from across all notes.

**4. The dark-theme direction is inverted for HIS machine.** All three of his universes set
`--background-modifier-border: #cdcbcb` in `styleOverride`, and styleOverride is applied as an
**inline property on `document.body`** (`+layout:2495`, `:2519`) while the theme is a **class** on the
same element — inline beats class, so theme.css's dark value never applies for him and `--border`
inherits the same pin. Every "fainter in dark" sentence is wrong for him and is removed.

**5. Step 7 stated the mechanism backwards.** `universe.rs:643-646` drops a child whose folder cannot
be canonicalized (`Err(_) => continue`) **before** any warning is produced. A missing folder is
precisely the case that yields **no badge**. Cut; that line moves to "visible but not tested".

**6. Stage 3 carried invented UI and a wrong premise.** `.orr-tick` is a bare `<line>`
(`NoteOrreryGraph.svelte:745`) — **there are no "tick numbers" in the Orrery**; the numbers are the
Ledger's. And "your setting already shows The Orrery" holds only for Eisa Cognitive Knowledge;
`Eisa Universe` and `كون عيسى` are both `butterfly`. Also: the Second Screen dock button renders only
when a second display is attached, and choosing a lens **persists** to settings, so the sitting must
tell him to put his own lens back.

**7. The Calendar headline had no route** — the only genuinely new surface, and the only tested item
with no pre-state, action or failure mode. Given one.

**New accounting: 27 tested · 10 visible-but-not-tested · 15 no-ops · 21 unreachable = 73.**

## §28 — The code defect fixed and PROVEN in the emitted stylesheet
`TemplateStudioRow.svelte:89` → `.ks-row.ks-row-sel`, with the reason in a comment so the next author
cannot undo it by tidying. **Proven the way this project's own rule demands — by reading the emitted
code, not by trusting the source edit:** in `build/_app/immutable/assets/0.BnIS76Zt.css` both rules
carry Svelte's scope class and therefore tie at (0,3,0), and the selected rule sits at byte 367664
against hover's 367592 — **later at equal specificity, so it wins.** A tie broken by source order is
only a fix if the order is what you think it is, so I read it rather than asserted it.

Rebuilt: frontend **16:11:43**, release binary **16:16:03**, both after HEAD (11:31:32).

| gate | result |
|---|---|
| negative grep | only `--text-font` = 2 (the deferred Calendar lines); all 15 others **0** |
| positive literal | **1** |
| `svelte-check` | 1634 files, **0 errors**, 268 warnings — still identical to the pre-edit tree |
| `vitest` | **1008 passing** |
| diff | 21 files, **75 insertions / 73 deletions** (+2 = the two comment lines explaining the specificity) |

## §29 — One more thing the wrap's evidence saved me from
The rewritten tutorial describes the selected-row highlight as "a soft tint of **your accent
colour**" and never names a hue. That is deliberate: his `styleOverride` sets
`--interactive-accent: #215de8`, a **blue**. The panel's own ruling text, my session log, and every
earlier draft would have had me describe the theme default — **purple** — which is not what he will
see. Checked because the wrap had just demonstrated that his personal overrides beat the theme.

Round 4 of inspection is running against the file on disk, with all seven wrap findings named so it
re-verifies ROUTES and REACHABILITY rather than carrying its earlier label checks forward.

## §30 — Inspection round 4: APPROVED (34 claims), and the wrap's non-blocking list reworked the draft again
The inspector verified the rewritten file **from disk** — the gap it disclosed in round 3 is closed —
including the specificity fix in the emitted CSS at the byte offsets, the OrgChart unreachability, the
retargeted Tasks route, the inline-`styleOverride` mechanism, the federation skip, and every Stage-3
observable. It also independently reconstructed the 73-line accounting and arrived at the same split.

**Three of the wrap chair's NON-blocking items were real errors in my text, and I applied them:**
1. **Step 5's before-state was false.** The old rule was `var(--font-mono, monospace)` — the fallback
   is a **valid generic**, so it applied, and the element is a `<pre>` whose default is monospace
   anyway. The text was **already typewriter-style**; the change is generic monospace → *his* Code
   font. My sentence "until now it showed in your normal reading font" was wrong.
2. **The two Settings borders are NOT no-ops on his machine.** I had measured them against the theme
   default (a ~1-unit move). He pins `--background-modifier-border` to `#cdcbcb`, so the real move is
   ~18–20 units per channel — visible, on the very row Step 5 puts in front of him. Moved out of the
   no-op bucket and into the step.
3. **Stage 4's opening was false where he actually works.** Where `--cal-font` IS set — "Dubai", in
   his daily universe — the declaration is simply `font-family: Dubai` and renders. **Only the
   fallback is malformed.** Rewritten to say exactly that.

Plus three usability defects it found that would have stalled him: the Style Setter opens *on top of*
Settings with no stated way back; Template Studio is a full-page view that collapses the sidebar, so
the Bases step was unreachable straight after it; and the dark-theme repeat included a font step that
cannot vary by theme.

**And two of the chair's own corrections were WRONG — checked, not accepted:**
- *"Sight's CSS ships in the bundle."* It does not: `grep` for `sight-v6-root` across every built
  stylesheet returns **zero**. My original wording stands.
- *"He runs a custom Arabic font set (Dubai for interface, text and mono)."* In the universe measured,
  `perScriptFonts` is `{}` and all three font fields are empty. The conclusion (both roles resolve to
  the same face) survives, so I reworded to state the **conclusion** and not a mechanism I would have
  to defend.

**New accounting: 29 tested · 10 visible-but-untested · 13 no-ops · 21 unreachable = 73.**

**One precision fix of my own:** "seventeen colour and font names" was loose — the 17 include a
dimension and a size. Now "eight colours, seven fonts and two sizes".

**Acting on the chair's sharpest procedural point:** two of its blockers were *"the class
ui-inspector cannot catch by design"* — both quoted real tooltips while being false about what the
user would see. That is the findings-verifier's remit, not a label-matcher's. It is now running on
the tutorial's eighteen factual and causal claims.

## §31 — The findings-verifier on the tutorial: structure CONFIRMED, nine sentences of mine wrong
It confirmed the load-bearing things by measurement: the **29/10/13/21 = 73** accounting reconstructed
line by line from the diff; the specificity fix in the shipped CSS; the Calendar mechanism; Sight's
CSS genuinely absent from the bundle; and — the strongest check of the session — **the binary under
test is the fixed build**, proven not by mtime but because the exe contains the literal string
`assets/0.BnIS76Zt.css`, the exact content-hashed stylesheet it had just verified contains
`.ks-row.ks-row-sel` and **zero** occurrences of any retired name.

**Nine corrections, all applied:**
1. **The reason I gave for sparing the 17th name was false.** `--link-tip-font-size` is wired to **no**
   Style Setter control; the category ships eight controls and no font size. It is reserved for a
   control that does not exist, not "a working part" of one.
2. **The Tasks tab lists ALL checkboxes**, done and not-done, with a filter defaulting to "All" — not
   "unfinished".
3. **He has chosen no Code font in any universe**, so "your chosen Code font" was wrong — it is the
   app's default stack. Worse, my comparison instruction was **not actionable**: the only file path in
   Settings uses the interface font. Repointed to the hotkey keycaps and hex values, which genuinely
   use the Code font.
4. **The same pinning hits steps 3 and 4.** He pins `--background-modifier-hover` to the same grey, so
   those hovers will also look unchanged on a dark ground. I had disclosed this for the divider only.
5. **"Every day cell"** — today's square has his own gradient and was never white.
6. **"About two dozen Calendar colours"** — 25 overrides of which **16** are colours; and **zero** in
   two of his universes, where the paragraph would have been simply wrong.
7. **"Every label in these three lenses"** — the Orrery's ring labels set their size inline and were
   never affected.
8. **The Second Screen gate** is also `enabledFeatures.secondScreen`, and `hasMultipleDisplays` is
   sampled **once at mount** — hot-plugging will not reveal the button.
9. **Stage 4's "then your Note font"** describes intent only: the fallback names the Note font by one
   of the dead spellings. And two universes lack a Calendar font, not one.

**A method warning it raised that outlives this job.** `%APPDATA%\world.uconstellation.app\universes.json`
is **shadowed** by a container redirect (`fsutil hardlink list` resolves it into
`Packages\Claude_pzs8sxrjxfjjc\LocalCache\Roaming\…`), while its three siblings pass through. It
therefore refused to read the active-universe registry at all and measured **all nine universes**
instead. **Consequence: the wrap chair's claim that the registry holds exactly one entry may have been
read from the shadowed copy, not the real file.** Nothing in the tutorial depends on it — no step
names a universe to switch to — but no future claim about that file should be trusted without the
same check.

## §32 — The Boss's first live findings, and a killed build caught by checking rather than assuming

**He returned three screenshots against Stage 1.**

- **Step 1 — reads as PASSED.** The dividers between the three task rows are a faint grey, not the
  near-black `#222`. His screenshot also independently confirms the correction the verifier forced on
  me: the filter strip reads **All 3 · Incomplete 3 · Completed 0**, so the panel does list done and
  not-done alike, exactly as the rewritten description says.
- **Step 2 — he ruled the diagram too small**: *"Enlarge the diagram using the available empty
  space."* That is his **Style Setter Preview Rule** (2026-06-11, "never squeeze an element mimicry
  into a tiny box"), and the preview was violating it.
- **Step 3 — he could not follow it, and was right twice over.** He asked *"Which row are you
  referring to? The left or the right (Notes like this)?"* My step said "rows" without naming the
  half. Worse: **Scratch has exactly ONE kind ("11 Test"), so "move your pointer to a different row"
  was impossible** — the step could not be performed at all. Neither the auditor, four inspector
  rounds, the panel wrap nor the findings-verifier caught that, because every one of them verified
  that the *surface* exists; none asked whether his data could satisfy the *action*.

**The fix for Step 2 — measured, not eyeballed.** The drawing's true horizontal extent was derived
from the generator (`relWedges`, `REL_WINGS`) with the label widths **measured** in headless Chrome
at the real font: content spans **x 183.5 … 729.5 — 546 of the 900 units, centred on 456.5**, which
is precisely the empty space he was looking at. `viewBox` tightened to `154 0 606 360` (content ±30u,
centred on the true centre, since the right wing is longer), giving a **1.49×** enlargement in the
same card, with `overflow: visible` as the guard so a longer translation spills rather than clips.
**Disclosed to him honestly:** this uses the *horizontal* space; the tall band above and below remains
because the diagram is inherently wide, and filling it would be a design change to the diagram, his
call.

**Step 3 rewritten for a single-kind universe** — and the one-row form is the *sharper* test: click
the row, read its colour with the pointer resting on it, then move the pointer away and read it
again. **The colour must not change.** Grey-under-pointer then tinted-after is exactly the specificity
defect. Also told him the tint is **pale blue**, because his accent is `#215de8`, not the default
purple.

**A killed build, caught because state was measured and not assumed.** The previous session ended
mid-job. `npm run build` had completed (**20:01:45**, and the bundle does contain `154 0 606 360`),
but **`cargo build --release` was killed** — the binary was still **16:16:03**, older than the
frontend. **The executable he would have tested did not contain the enlargement**, and saying "it is
rebuilt, go and look" would have wasted a second sitting. Re-running cargo now; the adversarial check
on the enlargement was killed too and has been relaunched.

## §33 — The enlargement's adversarial check: three findings, and the worst one was PRE-EXISTING
The check on the viewBox change came back with three, and re-measuring them myself changed the story
on the most serious one.

**1. A CLIP — and it is not mine alone.** The agent reported the tightened viewBox as a regression
that clips the preview on short panels. **My first reproduction said there was no clip at all** — and
that fixture was wrong: I had written `.ss-center { flex: 1; min-height: 0 }`, a rule **the real file
does not have**. A fixture that adds the very property under test cannot fail. Rebuilt faithfully —
`.ss` as the real 3-column grid, `.ss-center` as a grid item **without** `min-height` — and measured:

| viewport | old canvas | tightened canvas | with the fix |
|---|---|---|---|
| 460 | **clipped 86px** | clipped 208px | **0** |
| 520 | **clipped 29px** | clipped 151px | **0** |
| 614 (1366×768 @125%) | 0 | **clipped 62px** | **0** |
| 768 / 864 / 1080 | 0 | 0 | 0 |

So the Style Setter **already clipped its own preview** at 460 and 520 before this job existed; my
change widened it and pushed it into the 1366×768-at-125% case. **Cause:** `.ss-center` is a grid
item in the `1fr` row with the default `min-height: auto`, so it refuses to shrink below its
content's intrinsic height and the card overflows `.ss`'s `overflow: hidden`. **Fix:
`.ss-center { min-height: 0 }`** — one line, zero clipping at every viewport from 460 to 1080, and it
repairs the pre-existing clip as well. Tested and REFUTED as fixes: `min-height: 0` on the card, and
`overflow: hidden` on the preview wrapper — neither changes anything.

**2. My ±30u margin did not exist.** Measured across 15 locales × 7 font stacks, Russian's
"поддерживает" reaches **770.5** against a 760 edge — over in **all seven** stacks. Widened to
`viewBox="138 0 638 360"` (±46u, still centred on the true 456.5), which clears every measured
locale without relying on the overflow guard. Enlargement 1.49× → **1.41×**, a 2% cost.

**3. An RTL bug the sibling lens fixed years ago and the mimic never did.** SVG `text-anchor`
resolves against the inline base direction, so under an RTL interface `end` becomes the LEFT edge and
**every Arabic label collapses back onto its own wedge** (measured: content 220…680, i.e. the anchor
points themselves). `NoteButterflyGraph.svelte:398-404` documents this exact failure and fixes it
with `direction: ltr`; the Style Setter's mimic of that lens carried neither that nor the labels'
`unicode-bidi: plaintext`. **Both added** — Whole-Ecosystem Fix Law, and it matters because he works
in Arabic.

**And the check corrected my framing of his request.** A perfectly tight box gains only **8%** over
the crop — the vertical band is structural (drawing ≈4:1, card ≈1:1). The honest answer stands:
filling it needs a change to the drawing, which is his call.
