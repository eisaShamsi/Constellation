# Session Log — 2026-07-09

## Function in hand

**The G3 second-screen cross-window sync migration** — making the second screen (a separate Tauri webview / JS realm that mounts core `NoteEditor` components as *displays*) safely adopt main-window saves + rename cascades, with a **read-only default** and a Settings toggle for an opt-in **editable** mode. Class: cross-window-integrity (G3). The 2 remaining HIGH from the APP-KILLER #2 sweep (`wf_415a7214-4ad`).

**Concept (the horse):** *A note open in both windows must show and save the same truth. The second screen can never silently undo — or be undone by — the main window.*

Architect + Plan DONE + Boss-approved: `docs/G3-SecondScreen-CrossWindow-Architect.md` + `docs/G3-SecondScreen-CrossWindow-Plan.md`. **Boss ruling (2026-07-09):** read-only by default + a Settings toggle to make it editable; the editable path must be safe (WA#6).

---

## Territory verified (this session, before any edit)

- **SS is a separate JS realm** (`screen-entry.ts` mounts `SecondScreenPage` standalone — `+layout.svelte` never loads there). Every store singleton is per-realm: `cascadingPaths`, `recentWrites`, `openTabs`, noteModel `models` Map.
- **SS mounts 7 `NoteEditor` instances** (all fully editable today): `dashboardNoteTab` (`:1072`), `dashboardSelectedNote` (`:1112`), `indexSelectedNote` term (`:1159`) + compare (`:1215`), `mapCompanionNoteTab` (`:1232`), `peekTab` (`:1588`), `$activeTab` (`:1771`). Store `openTabs` holds only the active-editor tab + tab list; the other 5 are separate `$state` TabLike objects.
- **`externalChange` = `noteSession.externalChange` = `noteModel.adoptDisk`** — freshness-gated (refuses a dirty model, ignores its own echo). The primitive for §2/§3.
- **`reloadTabsFromDisk` FORCE-adopts** (`openNoteModel`, not freshness-gated) — the main uses it only *after* flushing dirty tabs in a cascade. **NOT safe for SS save-adopt** (would clobber a dirty editable SS tab) → §2/§3 must use `externalChange` per the plan.
- **`cascade:rewrote` is `app.emit`** (libraries.rs:1618 / :5483) → reaches the SS realm. `+layout.svelte:3223` is the shape to mirror for §3.
- **Main cascade block** = `handleRenameComplete` (`+layout.svelte:6185-6263`): `cascadeFreeze.set(...)` → `markCascading(t.path)` (`:6209`) → flush + `updateLinksOnRename` + `reloadTabsFromDisk` → `clearCascading` (`:6259`) → `cascadeFreeze.set(new Set())`. §4 emits `screen:cascade-freeze` around the mark/clear.
- **Settings propagation**: `updateSettings(partial)` updates `appSettings` + `saveSettings()` + `emit('screen:settings-changed', get(appSettings))`. SS `onSettingsChanged` (`SecondScreenPage.svelte:766`) merges the full settings → `secondScreenEditable` rides the existing broadcast automatically (no extra wiring).

### DISCOVERED (WA#6 — fix in same pass, not deferred)

**PropertyEditor writes to disk DIRECTLY via `saveTabContent`, bypassing the NoteEditor belt** — two sites: `debouncedSave()` (`PropertyEditor.svelte:806`) and the onDestroy flush (`:473`). So the §1 "belt" (NoteEditor.handleSave/Flush/Title/Promote early-return) is **NOT sufficient** to make the SS read-only: a property edit would still persist. **Fix:** add a `readOnly` prop to PropertyEditor and gate both write sites. Also gating this keeps the read-only model **clean** so §2/§3 `externalChange` always adopts (a stuck-dirty model would silently stop syncing).

---

## Predecessor → Replacement (Predecessor Lookup Rule)

This migration is almost entirely **additive** (new `readOnly` prop, new `secondScreenEditable` setting, new SS listeners) — the Predecessor Lookup Rule does not fire for additions. The one **behavior change** to an existing write path is logged here:

- **PropertyEditor save gate.** *Where it lives now:* `PropertyEditor.svelte` `debouncedSave` (`:806`) + onDestroy flush (`:473`) call `saveTabContent` unconditionally (gated only by `isCascading` inside the store). *Replacement (same place):* both sites get a `readOnly` early-return/guard; the `readOnly` prop is threaded NotePane → PropertyEditor. *Cut:* nothing removed — a guard is added. *Kept:* the editable path is byte-for-byte unchanged when `readOnly` is false (default in the main window).

---

## Build (cascading the approved Plan §1–§5)

### §1 — Read-only NoteEditor + Settings toggle (default read-only) ✅
- **NotePane.svelte**: `readOnly` prop; `readOnlyCompartment` (CM6 `EditorState.readOnly.of(true)` + `EditorView.editable.of(false)`) wired into the extensions + a reconfigure `$effect` (mirrors livePreview/typedLink) so the toggle flips editability live (no remount → cursor/scroll preserved); title `<input readonly={readOnly}>`; passes `{readOnly}` to PropertyEditor.
- **PropertyEditor.svelte** *(discovered fix, WA#6)*: `readOnly` prop; gated BOTH direct `saveTabContent` sites — `debouncedSave()` early-returns, onDestroy flush skips the write (still clears the timer). Closes the hole where a "read-only" SS could still persist a property edit, and keeps the model clean so §2/§3 adopts always fire.
- **NoteEditor.svelte**: `readOnly` prop; belt early-returns in `handleSave`/`handleFlush`/`handleTitleChange`/`handlePromote`; `onDocChange`→`editBody` guarded by `!readOnly` (keeps the read-only model clean); passes `{readOnly}` to NotePane.
- **store.ts**: `AppSettings.secondScreenEditable: boolean` + `DEFAULT_SETTINGS: false`. Propagates to the SS realm via the EXISTING `updateSettings` → `screen:settings-changed` full-settings broadcast (no new wiring).
- **SettingsModal.svelte**: "Make the second screen editable" toggle (Editor section) → `updateSettings({ secondScreenEditable })`.
- **i18n ×15**: `settings.editor.secondScreenEditable` + `...Desc` translated in all 15 locales (targeted insert, all JSON re-validated).
- **SecondScreenPage.svelte**: `ssReadOnly = $derived(!$appSettings.secondScreenEditable)`; `readOnly={ssReadOnly}` on all **7** NoteEditor mounts.

### §2 — SS adopts main→SS saves (freshness-gated, all views) ✅
- `adoptFreshDiskIntoSS(path)` + `adoptCompanionTab()` helpers: re-read disk once, `externalChangeNoteModel` (adoptDisk) into every matching SS view (store openTabs + 5 companion `$state` tabs); bump `reloadVersion` only on the views that actually adopted (clean) → NoteEditor's `{#key}` remounts + reseeds; a dirty editable-mode edit is never clobbered. Wired at the end of `onNoteSaved` (u2).

### §3 — SS reacts to the rename cascade ✅
- `listen('cascade:rewrote')` (app-emitted from Rust, reaches the SS realm) → `adoptFreshDiskIntoSS(p)` per rewritten path (same freshness-gated adopt as §2).

### §4 — Editable-mode cross-window freeze ✅
- **secondScreen.ts**: `CascadeFreezeData {libraryPath, active}` + `emitCascadeFreeze` / `onCascadeFreeze`.
- **+layout.svelte** `handleRenameComplete`: emits `screen:cascade-freeze {libraryPath, active:true}` right after `markCascading`, and `{active:false}` in the `finally` after `clearCascading`.
- **SecondScreenPage.svelte**: `onCascadeFreeze` → raises/clears the SS's OWN realm `markCascading` for its tabs (store + companions) under that library (so its autosave is gated during a main cascade), tracked per-library so clear removes exactly what it marked (refcount), with a **20 s stuck-freeze auto-clear** safety timer; cleaned up in onDestroy (`clearAllSSCascadeFreeze`).

### §5 — Harness + reviews ✅ (in progress)
- **Recipe N** (two-windows-one-path) added to `tests/mig-076/runtimeHarness.test.ts`: main-saves→clean-ss-adopts / main-saves→dirty-ss-refuses / cascade-rewrite→ss-reloads-not-stomps / echo-guard. **26/26 pass** (was 22).
- `svelte-check`: **0 errors** (319 pre-existing +layout CSS-unused warnings).
- Pending: `/simplify` + diff-scoped `safety-inspection` (running) → then Boss two-window tests.

### §5 — /simplify (4 parallel agents: reuse / simplification / efficiency / altitude) — applied
- **Reuse**: no actionable win. Confirmed the deliberate divergence from `reloadTabsFromDisk` (force-adopt → would clobber a dirty tab) is correct; `externalChange`=`adoptDisk` is already the shared primitive both windows use; `emitCascadeFreeze`/`onCascadeFreeze` + the SettingsModal toggle follow the existing patterns.
- **Efficiency (APPLIED)**: `adoptFreshDiskIntoSS` now skips the `read_note` IPC entirely when NO SS view shows the path — a rename cascade can rewrite dozens of backlinks (§3 loops over all), but the SS shows ≤7 notes, so most reads were wasted (Rule 3). Added a `shownHere` pre-check before the read.
- **Simplification (APPLIED)**: merged the two parallel §4 freeze maps (`ssFrozenByLib` + `ssFreezeTimers`) into one `Map<string, {paths, timer}>` — one atomic entry, no hand-sync.
- **Altitude (APPLIED)**: the read-only enforcement was split across layers — body/title non-interactive at the UI layer, but PropertyEditor only write-gated (inputs stayed interactive → a property edit "took then vanished"). Wrapped the PropertyEditor body in `<div style="display:contents" inert={readOnly||undefined}>` so all three editing surfaces are non-interactive at the SAME layer in read-only mode. The write-gate remains the safety belt.
- Re-verified: harness **26/26**, `svelte-check` **0 errors**.

### Safety-inspection — whole-app sweep (`wf_a19eb032-ab4`, 44 agents, 22 confirmed: 6 HIGH · 9 MED · 7 LOW)
Ran whole-app (doubles as the G3-cycle per-cycle sweep). Full register appended to `docs/Constellation-Safety-Audit-CHARTER.md`.
- **G3 CLOSES the two prior-registered G3 HIGHs** (`SecondScreenPage:1771` cascade-blind + `:723` never-adopts-main→SS) — that was the migration's purpose.
- **G3 diff introduces ZERO new app-killers.** The ONE finding in new G3 code — `SecondScreenPage.svelte:877`, the **two-sided-dirty cascade revert** — is the **documented, Boss-approved residual** (plan Residual + Architect §4): only reachable with `secondScreenEditable=ON` (non-default) AND a *simultaneous* edit on the *exact* cascaded note; read-only DEFAULT avoids it; strictly better than pre-G3. Needs the conflict-resolution `/migration` to fully close → **surfaced to Eisa for a ruling** (keep residual / cheap cascade-wins / full conflict migration). NOT symptom-patched (Solve-the-Class: don't trade one silent loss for another).
- **21 pre-existing findings OUTSIDE the G3 diff** → Charter register / G-plan triage. Strongest NEW: the **main-window external-change adopt gaps** (`+layout:3171` file-watcher reload doesn't adopt into model; `:3329` adopt doesn't bump reloadVersion) — literally G3's mirror on the main window; plus `store.ts:659` reloadTabsFromDisk net-wipe + `libraries.rs:1676` move_item DB-cascade (both HIGH). None block G3.

### Release binary build (for the Boss two-window test)
`npm run build` ✅ (toggle string "Make the second screen editable" verified in `build/`), `cargo build --release` running (bg `bs9oohggc`).

**STOP POINT: §1–§4 built + harness green + /simplify applied + safety-swept → release binary building → present staged two-window Boss tests + the residual ruling. Commit is part of the post-validation close-out (PCS).**

---

## PIVOT — Boss re-scopes the Second Screen (state-of-standing, SO #5) — 2026-07-09 (round 2)

During Stage-1 two-window testing the Boss corrected the premise and re-scoped the work. **This is a pivot; recording the standing state before proceeding.**

**Boss decisions (verbatim intent):**
1. **The SS shall be READ-ONLY — always.** Drop the editable toggle (and §4's editable-mode cross-window freeze — it exists only to make SS editing safe). The SS is never an editing domain. *Supersedes the earlier "read-only default + toggle" ruling.*
2. **SS enhancement approved** (typed-link neighborhood / tension surface / follow-the-thread peeking).
3. **The SS complements EVERY surface (function / core plugin), not only the NotePane.** When a note is open on the MS, the SS correctly shows the *panels* (Properties/Backlinks/Tags/Sky View/Tasks) — an extension of the note, NOT a duplicate editor. That is by-design and working (Boss screenshot confirmed).
4. **New direction — re-conceive the SS as a unified, 100%-MS-interactive complement:** *"bring the whole surfaces together when we enable the SS — a Control Dashboard, a General Estimation Map, an Operation Map."* → **This REOPENS + EXPANDS PJ-068** (the parked "SS = contextual companion, never replicate" concept paper).

**Standing state of the G3 build:**
- **Verified-shipped/protected:** nothing committed this session (G3 diff is uncommitted in the working tree).
- **At-risk / in-flight (uncommitted):** the G3 §1–§4 diff — read-only prop (KEEP) + editable toggle & i18n (NOW SUPERSEDED → to remove) + §2/§3 freshness adopt (KEEP — still serves the read-only complement) + §4 freeze (NOW SUPERSEDED → to remove). Harness 26/26, svelte-check 0 errors. Release binary at `src-tauri/target/release/constellation.exe`.
- **Known-broken:** none (diff is clean; the one in-diff item was the editable-mode residual, which vanishes once the toggle is dropped).
- **Pending (the new direction):** SS concept rework = reopened+expanded PJ-068. Sequence per Boss + "Concept before Function": (a) read every SS doc [DONE — #26, PJ-068, help topic, 2026-04-05 7-principles, G3 Architect/Plan]; (b) research how other software use second/companion screens (Lightroom named) [IN PROGRESS]; (c) synthesize the unified-complement concept + Boss rulings → `/migration`. Then re-land the code per the settled concept (keep read-only + freshness; drop toggle + freeze).
- **Doc drift:** the session log §1–§5 above + the Charter G3 register describe the pre-pivot G3 (toggle + freeze). To be reconciled when the rework concept lands. The SS help topic + manual still document full-editor editing on the SS (contradicts decision 1) — fold into the rework.

## PJ-068 v2 — SS Knowledge Cockpit: concept + plan RATIFIED, P0 landed — 2026-07-09

**Research → concept → plan (all Boss-approved):**
- External research: workflow `wf_8b4fdfa4-86d` (10 fronts — Lightroom / Presenter View / DaVinci / DJ-DAW / OBS / mission-control ops / Bloomberg / CAD-GIS / IDE-design / PKM) → synthesis.
- **Concept paper** `docs/concept-papers/PJ-068-v2-Second-Screen-Knowledge-Cockpit-Concept-Paper.md`: the SS = a read-only **"Presenter Display for knowledge formulation"** — ONE cockpit, THREE fixed zones (① Estimation Map ② Control Dashboard ③ Operation Map = the Boss's three metaphors), a **Normal/Live/Locked** coupling dial (Lightroom), ONE action = **click-to-navigate** (100% interactive, 0% mutating). Fixed zones = one SS complements every surface, no per-surface sprawl. + a visual cockpit mockup shown to Boss.
- **Boss rulings (2026-07-09):** zone layout accepted · ship all three dial positions · all seven surfaces at once · retire Navigator(already gone)/OrgChart/fallback-editor (re-validate later) · design Sight's complement now, build later.
- **Estimation Map enrichment (Boss):** it is the **MARQUEE zone — the ONE holistic view of the whole universe across PAST · PRESENT · FUTURE (what's next)**, not a locator (concept §3.1). Gets a dedicated design pass before P4.
- **Build plan** `docs/concept-papers/PJ-068-v2-Knowledge-Cockpit-Build-Plan.md` (Phase-2, APPROVED): P0 reconcile G3→read-only-always · P1 focus-channel+3-zone shell+dial · P2 note complement + retire fallback editor · P3 Sky+Map + retire OrgChart · P4 Estimation+Index+Dashboard+Tasks+Sight · P5 unify+perf+docs. Frontend-only, no schema, `COCKPIT_ENABLED` flag. Territory verified (data all persisted/Rule-8-clean; Navigator already retired MIG-091; G3 reconciliation self-contained).

### P0 — Reconcile G3 → "read-only always" ✅ (committed)
- **KEEP:** `readOnly` threading (NoteEditor/NotePane/PropertyEditor) + `adoptFreshDiskIntoSS`/`adoptCompanionTab` + `onNoteSaved` adopt + `cascade:rewrote` listener (freshness stays).
- **REMOVE:** `secondScreenEditable` (store `AppSettings`+`DEFAULT`) + the SettingsModal toggle + i18n ×15 (restored from HEAD) + all §4 cross-window freeze (secondScreen.ts `CascadeFreezeData`/`emitCascadeFreeze`/`onCascadeFreeze`; +layout emits+import; SecondScreenPage import+helpers+listener+cleanup); `ssReadOnly` → `const true`.
- **The one residual the whole-app sweep flagged (SecondScreenPage:877, two-sided editable conflict) is ELIMINATED** — the editable write path no longer exists.
- **Verify:** harness **26/26**; `svelte-check` **0 errors**; grep confirms zero dangling refs to any removed symbol. Pure removal → covered by the existing sweep (its one in-diff finding removed); no new app-killer surface.

### P1 — Three-zone cockpit shell + Normal/Live/Locked dial (note focus, all zones wired) — built
Boss steer: dial must *use* the space (Style-Setter rule); show ALL zones wired, not an empty skeleton.
- **New `src/lib/cockpitFlag.ts`** (`COCKPIT_ENABLED = true` + `DialMode`) — one-line rollback; old tabbed panels kept behind `{:else}`.
- **New `src/lib/components/SecondScreenCockpit.svelte`** — the read-only three-zone cockpit for a focused note: ① Estimation Map (universe scale + note position + maturity — first cut; holistic past/present/future = P4 dedicated pass) · ② Control Dashboard (link health: dominant confidence, contested/tension count, dormant/decaying, load-bearing, review status via `get_note_review_status`) · ③ Operation Map (outgoing typed links, backlinks, unlinked mentions — click-to-navigate). Plus the **Normal/Live/Locked dial** (full-width segmented, never squeezed). Data via the cheap per-note IPCs (`get_backlink_rows`/`get_outgoing_rows` — MIG-079 index seeks; `scan_unlinked_mentions`; `get_note_review_status`) — Rule-8 clean, gen-guarded, path+nonce-keyed (zero IPC churn on the main window's keystrokes).
- **Wired into `SecondScreenPage`** — renders the cockpit for the note-focus branch (`editorPanelsActive`) behind `COCKPIT_ENABLED`; `cockpitNavigate` → `sendNoteToMain` (read-only nav); `cockpitReload` bumped on the shown note's save/cascade so the link zones re-read.
- **Coupling:** Normal/Live follow the live focus; Locked captures the focus on entry + freezes it. (Live's hover source comes online at P3.)
- **Verify:** `svelte-check` **0 errors**; harness **26/26** (unaffected). Read-only display, no write/index/lifecycle path → per-build write-path safety sweep exempt (verification = svelte-check + harness + gen-guard review). i18n: `$t('cockpit.*') || 'English'` fallbacks (×15 keys land in the P5 i18n close-out once labels stabilize).
- **Next Boss test:** two-window — open a note in the main window (SS open) → the second screen shows the three-zone cockpit; click a backlink → the main window navigates.

### P1 Boss test (round 1) — cockpit WORKS; 2 fixes applied
Boss two-window test: Steps 1 (open SS), 4 (click-to-navigate), 5 (dial Follow/Pin) **PASS**. Screenshot confirmed the three zones filling the space with real typed-link data (derives-from / supports / generalizes chips, backlinks + outgoing). Two issues → fixed (commit `b6419e5f`):
1. **"UX naming issue"** — raw `cockpit.*` keys showed. Root cause: this i18n returns the KEY for a missing entry (truthy) → `$t(k) || 'fallback'` never fell back. Fix: added the real `cockpit.*` label block to `en.json`; confidence shows the value directly (capitalized) instead of an unresolved nested `confidence.*` path; dial labels made `$derived` (locale-reactive). **×14 locales pending Boss label sign-off** (naming was flagged — get wording approved before translating).
2. **Step 6 freshness ("nothing happened")** — cockpit re-read link zones on the save broadcast, before the async reindex updated `note_links`. Fix: immediate fetch on note-open (path change); ~450 ms-delayed refetch on a same-note save/cascade (`reloadNonce` bump) so `note_links` has settled. Leak-safe timer (supersede + `onDestroy`).
- Verify: `svelte-check` 0 errors; labels embedded in the release build. Re-test binary building.

### P1 re-test (round 2) → Boss REDIRECTED the note view to a radial link-graph
- **Naming: approved.** **Step 6 still off** (screenshot showed stale outgoing `a`/`c` vs the note's real links A/C v2/Bauhaus — the timed refetch didn't catch the reindex).
- **Boss's 3rd image + ruling:** the note-focus view should be a **radial graph — the open note at CENTER, its backlinks/outgoing links as surrounding nodes** (with facet tabs). Initially I over-elaborated it as the four Cognitive-Engine questions (Development/Altitude/Origin/Connection) — grounded in `Cognitive-Engine-One-Picture` + `16-inspector360` (the radial IS the existing Inspector-360/360.3D concept) — but **Boss corrected: center = the note, surrounding nodes = backlinks/outgoing links** (simpler). Two mockups shown; corrected one confirmed.
- **Boss design decisions (2026-07-09):** (1) node encoding = colour by typed relationship + size by link weight — **keep**; (2) **backlinks left · outgoing right**; (3) facet tabs = my call, **contextual**; (4) click a node → the **main window** opens that note (read-only nav).

### P2 — the Note Radial Graph (the real note-focus view) — built
- **New `src/lib/components/NoteRadialGraph.svelte`** — the open note centred; **backlinks radiate left, outgoing right**; each node coloured by its typed relationship (the 8 + associative — supports=green / contradicts=red / causes=orange / exemplifies=teal / generalizes=blue / derives-from=violet / part-of / supersedes), sized by lifecycle tier (weight-derived: load-bearing large → emerging small; stale faded); spokes coloured per relationship. Click a node → `onNavigate` → `sendNoteToMain` (read-only). Caps 9/side + "+N more". Pure presentational (host passes the persisted `note_links` rows).
- **`SecondScreenCockpit` restructured** — the note-focus view is now the **dial (Follow/Peek/Pin) + the radial graph**, replacing the P1 list zones (Estimation Map + Control Dashboard + Operation Map). Data still via `get_backlink_rows`/`get_outgoing_rows` (Rule-8 clean); the ~450ms save-refresh timing carried over. The Estimation Map (holistic universe) moves to the separate universe-focus view (later).
- **Verify:** `svelte-check` **0 errors** (fixed 2: SVG `<text>` has no `dir` attr; `prevDial` init). Contextual facet tabs (Boss's call) = the immediate next increment. i18n ×14 for cockpit + radial strings HELD until the note-view design (radial + tabs) stabilizes, then one complete pass (avoid re-translating a moving target).

**Next: P2 radial Boss test (on a fresh binary) → then the contextual facet tabs.**

### P2 radial test → Boss redirect to a designed graph (two SME panels)
- Boss: show ALL links (no cap) + the facet tabs were missing. Fixed the radial to show every node (small dots, hover-reveal, no "+N more") + added the facet tab bar (Links default; others "coming"). Commit `ddfe6d82`+.
- Boss then wanted the note-graph made **beautiful**, with **options**, each **concept-led**, and steered: **the GRAPH is what matters** (not non-graph paradigms), delivered as **pictures**.
- **Panel 1 — aesthetic review** (`wf_67919c1c-4c0`, 6 lenses + creative director): verdict on the plain build = "confetti dots"; direction = a **star-chart / personal orrery**, **Flexoki** dual-theme palette, glow-halos (not flat discs), home-star centre, sorted colour bands, contradicts=ember, one graceful morph, quiet chrome. Concept brief written: `docs/concept-papers/Note-Constellation-Radial-View-Concept.md`.
- **Panel 2 — concept divergence** (`wf_a2af08f5-9a3`, 7 SMEs → 9 curated concepts): graph bets (Aster/Shamsa/Orrery/Deep-Field/Heartwood) + non-graph bets (Tide/Al-Isnad/Self-Writing/Vital-Signs). Shown to Boss as a **visual board** (per the graph steer, graph concepts only).
- **Boss ruling (round 3):** build **THREE switchable lenses** — order **(1) Aster (2) Heartwood (3) Orrery** — toggled in Settings, coloured via the **Style Setter**. Thanked the panel.

### P2 — "The Aster" lens (flagship #1) — built (commit `16b7233e`)
- **`NoteAsterGraph.svelte`** — the relationship ROSE: one petal per typed relationship (split backlinks-left / outgoing-right), petal width ∝ count, filaments = the individual links (radial threads, heaviest longest, sorted to the spine), soft petal glow = the aggregate → density becomes texture at 3 or 200+ links. Hover a thread → name + spoke; click → main navigates. Home-star centre (no rectangle), ecliptic ring, deep-field vignette, contradicts = the ember. **Relationship colours = CSS vars `--rel-*`** (Style-Setter-ready), Flexoki defaults.
- **Pluggable plumbing:** `cockpitFlag.NoteGraphStyle` + `NOTE_GRAPH_STYLES`; `appSettings.noteGraphStyle` (default `aster`); SettingsModal lens selector (Heartwood/Orrery "coming"); cockpit renders the active lens (Aster live; baseline radial for the others).
- **Verify:** svelte-check 0 errors. **Next:** Aster Boss test → Heartwood → Orrery → the Style Setter `--rel-*` category.

### Aster test → PASS + polish/stats + i18n fix (into 2026-07-10)
- Boss: "Good starting point... all pass. But fix the settings localization." → the `settings.editor.noteGraphStyle`/`Desc` keys were missing (raw keys shown). Added ×15 locales ("Second screen · note graph"; "Style Setter" kept as brand). Commit `da909744`.
- Boss: **"Polish the Aster, add the note statistics — every bit available."**
- **Aster polish + stats HUD** (commit `27a32715`): a quiet cognitive HUD in the four corners around the rose, by the four Cognitive-Engine questions — **Development** (stage/maturity/review), **Content·Altitude** (word count/stratum/tags), **Origin** (provenance/source/created), **Connections** (links in/out/dominant confidence/tensions/load-bearing/dormant). Every row graceful (shown only when the datum exists). Data from frontmatter (`parseFrontmatter`) + `get_note_review_status` + the fetched link rows — Rule-8 clean; the graph fetch stays path-keyed (content changes never refetch links). Look: deeper vignette, contradicts ember, lone home-star at 0 links (SVG always renders). Colours stay `--*` CSS vars for the Style Setter.
- **Verify:** svelte-check 0 errors. Binary building for the Boss test.

### Aster RETIRED → Butterfly + Ledger (Boss ruling 2026-07-10)
After Aster v2 the Boss was still unsatisfied ("still a circle, labels collide, gauges tiny") and convened the Art Director + team via the `aster-art-direction` workflow (12 agents, benchmarked glass-cockpit / Bloomberg / Grafana / Lightroom / radial-dataviz / FUI). Team's root-cause: **the primitive was wrong** — one shared polar center filling ~280° reads as a disc by Gestalt closure, halves can't part (shared origin), count-as-area is the worst encoding, conviction (the CE destination) was the faintest pixel. Three directions on ONE shared chassis (twin origins · vertical gutter · length-encoding · framed gauge deck): **The Ledger** (diverging bar balance-sheet), **The Butterfly** (two facing half-blooms, recommended), **The Cockpit** (PFD instrument panel). **Boss ruling: RETIRE the Aster; build The Butterfly + The Ledger, both added to the lens collection ("two for the price of one").**

**Predecessor → Replacement (Predecessor Lookup Rule):**
- Predecessor: `NoteAsterGraph.svelte` (the `aster` lens), rendered by `SecondScreenCockpit.svelte:168` when `$appSettings.noteGraphStyle === 'aster'`; lens registry `NOTE_GRAPH_STYLES` in `cockpitFlag.ts`; setting `noteGraphStyle` in `store.ts:4172/4502`; Settings select `SettingsModal.svelte:1033`. Introduced this session (2026-07-09, commits ddfe6d82→118684bf).
- Replacement: **SAME place** — `noteGraphStyle` union becomes `'butterfly' | 'ledger' | 'heartwood' | 'orrery'` (default `butterfly`); `NoteButterflyGraph.svelte` + `NoteLedgerGraph.svelte` render in the same cockpit `links` facet slot; shared derivation in `cockpitGraphData.ts` + shared `NoteGaugeDeck.svelte`. Retired `aster` value normalizes → `butterfly` on load.
- Cut: `NoteAsterGraph.svelte` deleted; `'aster'` removed from the union + registry. Kept: `NoteRadialGraph.svelte` (baseline fallback for the unbuilt heartwood/orrery).

### Butterfly v2 + lens toggle relocated (Boss remarks 2026-07-10, round 2)
Boss on the shipped pair: **the Ledger is good, no changes.** Butterfly remarks: (1) the note reads as a **handbag** — the conviction arc I drew above the title box was a "handle"; only a title box is wanted. (2) **wings squeezed short**, side space unused — expand them away from the centre box and **separate the nodes**. (3) **type labels overlap**. (4) want a **faint dotted middle divider**. General: **move the lens toggle out of Settings onto the cockpit page.**

Fixes: conviction arc deleted (plain title box, width adapts to the title). Geometry is now **responsive** (`bind:clientWidth/clientHeight` → `viewBox="0 0 W H"`, 1:1 px) and the wings are **elliptical** (semi-axes RX × RY) so they stretch into the full stage width instead of being bounded by the shorter axis — root cause of the squeeze was the fixed `900×560` viewBox letterboxing inside a ~2.5:1 stage. Nodes separated by adapting dot radius to each petal's own arc-length-per-link (no synthetic jitter — Form-Aligns-To-Purpose); filament length now encodes **earned weight** (tier + traversals). Type labels evicted to reserved **outer ledger columns** with a `decollide()` pass (min 22px gap, clamped) + faint leader lines — overlap is now geometrically impossible. Faint dotted seam added top+bottom of the spine.

**Predecessor → Replacement (lens toggle):**
- Predecessor: Settings → Editor → "Second screen · note graph" `<select>` (`SettingsModal.svelte:1028-1039`), writing `noteGraphStyle` via `updateSettings`.
- Replacement: **on-page segmented toggle** in the cockpit's facet-tab row (`SecondScreenCockpit.svelte`, shown only for the `links` facet). **Relocation explicitly approved by the Boss** ("having a toggle switch on the same page is the right place… let's move it from the settings").
- Cross-window safety: the SS **never writes settings** (Display-not-Domain). It emits `screen:set-lens`; MAIN listens (`onLensChangeRequest` → `updateSettings`) and its existing `screen:settings-changed` broadcast re-renders the SS. This avoids the real clobber bug: main's `appSettings` is not refreshed by SS writes, so a later main-window save would have silently reverted the lens.
- Cut: the Settings item + `NOTE_GRAPH_STYLES`/`normalizeGraphStyle` imports in SettingsModal. Kept: the `settings.editor.noteGraphStyle*` locale keys (now unused; harmless).

### Aster v2 — Boss's 4 remarks addressed (superseded by the retire above)
Boss remarks: (1) hard to tell backlinks from outgoing → **PART** them: left bloom (backlinks 110–250°) + right bloom (outgoing −70–70°), wide top/bottom seams + dashed divider + "← backlinks / outgoing →" labels — two facing blooms. (2) shape too uniformly circular → **BREAK** it: dropped the enclosing ecliptic ring; petals reach unequal lengths (0.12–1.0 by weight) → irregular silhouette. (3) prefer charts/gauges → the stats HUD is now **gauges**: stage + maturity step-ladders, review pill, word-count numeral, relationship-mix stacked bar, supports↑/contradicts↓ balance meter, confidence stacked bar (categorical stays as chips). (4) match the app theme → field/aura/text/divider/chips use the app's `--background-*`/`--text-*`/`--interactive-accent` (light+dark correct); `--rel-*` stay for the Style Setter. svelte-check 0 errors.

---

**SS concept sourced (from our docs):** the SS is *"an extension of the mind onto a second monitor — the context around the work in hand, never a second copy of it."* The **PJ-068 razor** (every SS surface must pass): **Contextual** (responds to what the MS is doing now) · **Complementary** (shows what the MS is NOT showing) · **Chosen** (appears only because the user opened the SS; never self-initiates). Display-Not-Domain is settled law. History: born 2026-03-13 as a mode-switcher → found its vocation 2026-03-19 as the Sky View contextual companion → 2026-04-05 the "clean writing space + panels migrate" redesign (7 principles) → double-init fixes. PJ-068's replication audit already grades each mode COMPLEMENTS vs REPLICATES.

</content>
</invoke>
