# Session Log — 2026-08-31 (new session, post-PJ-435)

Previous session: `lab/reports/SESSION-LOG-2026-08-29.md` (§1–§29, closed with the PJ-435 close
and the Linked-Universe doc rename). Handover: `lab/reports/HANDOVER-2026-08-31.md`. Entry
commit: `323f000f`, branch `main`, synced (git pull: already up to date).

---

## §1 — Session start: PJ-433 taken up; SO#8 cross-check PASSED (entry live, line numbers drifted)

**Working on: PJ-433 — the silent boot fallback** (the boot loop in `+layout.svelte` that
silently opens a different universe when the last-active one is unreachable at boot, and
`set_active_universe` then persisting that fallback as if the user chose it).

**Concept (the horse):** at boot, the app must never substitute a different universe for the one
the user chose without telling them — and must never record its own substitution as the user's
choice. The function (the carriage): the boot-activation flow and its notice.

**SO#8 cross-check — verdict: NOT STALE, scope accurate, line numbers drifted.** Verified in the
current tree (read, this session):

- The boot loop is now at `+layout.svelte:3712-3729` (ledger v2.08 says 3644-3647 — drift from
  the PJ-435 commits). Mechanism intact and quoted verbatim: try each entry of `listUniverses()`
  in order, `catch { continue; }` on failure, `showUniverseSetup = true` only when NOTHING
  activates.
- `list_universes` (`universe.rs:878-893`) sorts **active-first**, so the loop tries the user's
  universe first and silently falls through to the others in registry order.
- The persist is now at `universe.rs:1424-1425` (ledger says 1245 — drift): at the END of a
  successful `set_active_universe`, unconditionally `registry.active_id = Some(id)` +
  `save_registry`. So a successful FALLBACK activation is recorded as the user's choice.
- The remedy pattern the ledger names (`federation.warningBadge` popup) exists, now at
  `+layout.svelte:10812` (`sb-federation-warning`).
- Session logs + handover confirm PJ-433 filed 2026-08-30, never started
  (`SESSION-LOG-2026-08-29.md:782`, handover §4).
- Adjacent finding while cross-checking: `remove_universe_from_registry`
  (`universe.rs:1443-1444`) also auto-picks `entries.first()` as the new active when the removed
  one was active — same "app decides, silently" family; handed to the Architect map as part of
  the whole-ecosystem enumeration.

**Migration Rule applies** (Rust ↔ Svelte, boot path, persisted registry state) → the four-phase
`/migration` workflow. **Phase 1 (Architect) launched** as workflow `wf_c64b6c15-e8b`
(`pj433-architect`): five parallel territory maps (boot flow + setup screen; Rust activation +
every `active_id` writer; every switch surface, Whole-Ecosystem; existing honest-notice
patterns to reuse; wrong-universe side effects — MIG-100 session write-authority, PJ-435
relocation records, MIG-079 idempotent guard) → one synthesis into the <600-word option paper
(options A boot-blocking chooser / B fallback-with-banner / C minimal no-persist+badge, each
with speed/effort/risk).

Next in the pipeline per the standing laws: Architect doc → **panel** (The Panel Speaks First)
→ options to the Boss for the Phase-2 pick. No code before the Boss's pick.

---

## §2 — PJ-433 Phase 1 complete: Architect mapped, panel ruled UNANIMOUSLY for Option A (A-LEAN + mount-watch + A′)

**Architect** (workflow `wf_c64b6c15-e8b`, 6 agents, 5 territory maps + synthesis) filed at
`docs/PJ-433-Silent-Boot-Fallback-Architect.md`. Three options: A boot-blocking honest chooser /
B fallback-with-banner-no-persist / C minimal no-persist + badge. Full territory evidence in the
doc: `set_active_universe` side-effect order (nothing durable mutates before the Err at :1277),
all six writers of `active_id`, every switch surface, the notice vocabulary, and the measured
wrong-universe side effects (MIG-100 session file SAFE; bleed limited to global localStorage
conveniences: recents, search history, index-excluded-terms).

**Panel** (workflow `wf_38ca0d52-dc7`, 11 agents: 6 default-REFUTED verifiers + 4 conflicting
lenses + synthesis chair): five claims CONFIRMED, one PARTLY — the Architect's "mig108 boot-gate
= blocking precedent" was mischaracterized (that gate parks only the background fan-out, paints
the UI first, 30s failsafe — **no surface in Constellation today truly wedges boot**; the real
precedent is the `showUniverseSetup` blocking mount). Architect doc corrected in place (§1, §2)
— the stale-precedent error was caught BEFORE it could brief anyone downstream.

**The ruling — no dissent on the option, all four lenses chose A independently:** Option A in
the **A-LEAN** shape (Boot Chooser as a SIBLING of the wizard under the existing
`showUniverseSetup` gate — names the unreachable universe + path + reason, lists the registry
with reachability, Retry / explicit pick / Create-new; nothing activates or persists until the
user clicks) + **mount-watch** (poll the missing path while the chooser is open; light "It's
back — Open" on reappearance) + **A′** (close the second silent writer:
`remove_universe_from_registry:1443-1444` stops guessing a successor). Panel also ruled: fold
the amnesiac-wizard fix in (the chooser IS that fix); fix the remove-repoint in this pass
(Whole-Ecosystem Fix Law). Declined to the Boss: Architect Q1 + Q2 verbatim, plus one taste
call (drive reappears while chooser open: auto-open vs. one click). Three Phase-2 red flags
recorded in the doc §6 (first-ever boot wedge; re-entry must be the whole boot; entry[0]
consumers while nothing is active).

**Surfaced during mapping — to file at the next ledger reconciliation (SO#9.2), NOT lost:**
- `migrate_legacy_data` (`universe.rs:2505-2515`) writes `active_id` AND sets in-memory
  `active_path` directly WITHOUT the invalidation chain — a pre-existing half-activation on the
  legacy-migration path. Outside PJ-433 scope; needs its own PJ number at the next bump.
- `UniverseManager.svelte:48` stale comment (mechanism wrong: reorder happens read-time in
  `list_universes`, not in `set_active_universe`) — panel says fix in passing during Phase 3.
- Verifier cross-reference: the remove-flow's in-memory `active_path` gap is ALREADY filed as
  PJ-322 (no new filing needed).

**Records landed before the ruling request (SO#10):** Architect doc (with panel verdict §6),
this log, orientation v4.26. Ruling request to the Boss follows the commit. Committed
`3a81bf5d`, pushed.

---

## §3 — THE BOSS RULED (2026-08-31): Boot Chooser; no Remove on the boot screen; wait for the click

Three rulings, taken via the options dialog after the panel's voice reached him:

1. **Fix shape: the Boot Chooser** (Option A / A-LEAN + mount-watch + A′, the panel's unanimous
   recommendation). The app never opens a substitute universe; an honest screen names the
   unreachable universe, path, and reason; lists the registry with reachability; Retry /
   explicit pick / Create-new; nothing activates or persists until the user clicks.
2. **"Remove from list" stays OFF the boot screen** — removal remains a deliberate act in the
   Universe Manager only (Architect open question 2 → answered NO).
3. **Drive-returns behavior: wait for the click** — the "It's back — Open" button lights up when
   the missing path reappears; no auto-open (the taste call the panel declined → the
   Constellation Way's letter).

Phase 2 (Plan) launched next: planner + WA#5 proven-methods cross-check + adversarial plan
attacks (red-flag compliance, whole-ecosystem completeness, migration path) + final synthesis.
Plan goes to the Boss for approval before any code (Plan Approval = Build Approval).

---

## §4 — Phase 2 (Plan) FINAL: nine steps, all attack findings absorbed, filed for Boss approval

**Workflow `wf_77362636-844`** (first launch `wnnv68uxg` had a script bug — the reviewers would
have received a literal `${draftPlan}` placeholder instead of the plan; caught before the attack
phase ran, stopped, fixed, relaunched as `wnlh1eus8`). Final plan filed at
`docs/PJ-433-Boot-Chooser-Plan.md`.

**Shape:** §1 Rust `get_registry_status` (registry-only, once per boot, replaces `listUniverses`
— net-zero IPC) + `check_universe_reachability` (async — dead-UNC probes must not block) · §2 A′
(`remove_universe_from_registry` → `active_id = None`, dialog names the successor) · §3
boot-loop rewrite + extracted `finishBoot()` (the full post-activation tail incl. the
federation:ready listener, watcher arming, and the second-screen `notifyUniverseSwitch` — the
draft's bare-`handleUniverseCreated` pick path was a PARTIAL RESUME, caught by Attack 1) · §4
`BootChooser.svelte` sibling gate + pick wrapper + mount-watch with epoch guard + "Open from
folder…" (the Lightroom/Obsidian Locate affordance, adopted from the WA#5 cross-check) + one
additive `onBack` prop on the wizard (flagged for explicit Boss approval) · §5 second-screen
title from `get_active_universe_path` (never `universes[0]` blind) + `appReady` spawn guards ·
§6 i18n ×15 · §7 help + manual ×15 · §8 /simplify + diff-scoped safety-inspection · §9 staged
Boss test (auditor → inspector → panel first; never touches the daily universe).

**Notable attack catches absorbed:** the draft's "report only on failure" self-contradiction
(boot must know `active_id` BEFORE attempting — `list_universes` doesn't return it); the partial
resume; a chooser throw falling to the bare spinner (now degraded-props catch); the dangling
`active_id` state; the pre-created (`visible:false`) second-screen window invalidating the
draft's spawn reasoning; the draft's `:7889` mis-anchor (real spawn affordances at :6097/:6122).

**Deliberately out of scope, to file as a PJ at the ledger reconcile:** corrupt-registry
lenient-load (`universe.rs:154-157` → empty vec → wizard while `set_aside_corrupt` shunts the
file) — pre-existing, unrelated mechanism, same honesty family.

**WA#5 verdict:** matches Lightroom's blocking dialog + Obsidian's picker; avoids VS Code's
silent dead-restore and Logseq's disabled-editor anti-patterns. Battle-tested, not inventive.

**Added by me to the plan doc:** the MSIX-virtualization warning stamped on every
registry-touching verification clause (fsutil route / on-screen evidence only — carried from
HANDOVER-2026-08-31).

---

## §5 — THE BOSS APPROVED THE PLAN (2026-08-31): "Approve — build it", full plan INCLUDING the wizard Back button

Phase 3 (Build) cascade begins. Per the Boss-test standing order: **all code stays uncommitted
until his test passes**; the §-structure of the plan orders the work, the code commits land
after the pass. Step progress logged below as each verification clause completes.

---

## §6 — Build progress: plan §1–§7(en) implemented; svelte-check 0 errors; i18n parity 15/15

- **§1 Rust commands** — `get_registry_status` (registry-only, `RegistryStatus { active_id,
  entries }`) + `check_universe_reachability` (async; machine keys `not-found` /
  `not-a-directory` via the extracted `classify_reachability`) in `universe.rs` after
  `list_universes`; registered in `lib.rs`; TS wrappers + types in `universe/store.ts`.
  `cargo check` green.
- **§2 A′** — `remove_universe_from_registry` → extracted `remove_entry_from_registry`:
  removing the active sets `active_id = None`, never `entries.first()`. UniverseManager confirm
  dialog names the successor (`removeSuccessorName` derived = exactly what `confirmRemove`'s
  `handleSwitch(universes[0].id)` will open); stale `refresh()` comment fixed (the §5
  in-passing item); en key `universe.manager.removeConfirmSuccessor`.
- **§3 boot rewrite + `finishBoot()`** — the 474-line post-activation tail (federation:ready
  listener → `initializeApp` → §N refreshes → watcher + every boot listener) extracted from
  `onMount` to a component-scope function with a `bootTailRan` first-entry guard and an
  unconditional `notifyUniverseSwitch()` in `finally` (the SS window pre-exists). PowerShell
  surgery with 12 boundary assertions, all passed; no multi-line template literals in the moved
  region (verified before re-indent). The boot section now: `getRegistryStatus()` → empty/
  migration branches unchanged → attempt ONLY the recorded active entry → success `finishBoot()`
  / failure or no-active → chooser state set (pure synchronous sets — nothing can throw past it,
  RF1) → return. `handleUniverseCreated` now runs `finishBoot()` — closing the PRE-EXISTING
  first-run partial resume (wizard sessions had no watcher + no federation listener until
  restart).
- **§4 BootChooser** — `src/lib/components/BootChooser.svelte`: sibling gate
  (`{#if showBootChooser}` before `{:else if showUniverseSetup}`); banner (name + path `dir=ltr`
  + reason + `nothingChanged` reassurance); one retry button that BECOMES "It's back — Open"
  (accent) when the 3s mount-watch poll (single-flight, `alive` epoch guard, cleared in
  onDestroy; never activates) sees the path return; entries list with Reachable/Unreachable
  chips; degraded mode (probe failure → no chips, all Opens enabled, pick fails cleanly inline);
  `handleBootChooserPick` = stop-tracking → flush → activate → only-on-success unmount →
  `finishBoot()`, error returned as string and rendered inline (RF1); "Open from folder…" =
  pick_folder → `openExistingUniverse` → same pick path; "Create new" → wizard with additive
  `onBack` prop (Boss-approved; renders `us-back` button on step 0 only in chooser context).
- **§5 second screen** — `resolveActiveUniverseName()` (match by `get_active_universe_path`,
  never `universes[0]`) at both title sites; `appReady` guards in `handleToggleSecondScreen` /
  `handleSendToSecondScreen`; verified `heal_paths_after_move` untouched (Err :1345 still
  precedes the activation heal :1427).
- **§6 i18n** — 17 en keys (`universe.bootChooser.*` + `removeConfirmSuccessor`); 14-locale
  workflow `wf_5a623f78-e02` (first run died whole on the session usage limit — 0 files
  touched, verified by git status + 15/15 JSON parse check — relaunched clean): 14/14 DONE,
  each reusing its file's established universe term; `i18n-parity.mjs`: **15/15 in parity**.
  Deviations from the plan's key sketch, deliberate: `removeConfirmSuccessor` placed in
  `universe.manager` beside its sibling `removeConfirm` (not top-level `universe.`); no
  `bootChooser.back` (wizard reuses its own `universe.setup.back`); added `nothingChanged`
  (reassurance line, PJ-435 banner precedent); no `reasonHealable` (dropped per plan §1).
- **§7 English docs** — help `Universe/Universe.md`: new section "When your universe can't be
  found at startup" + the Portable-Universes steps and aliases updated (the old text documented
  the silent fallback as a known shortcoming); `User Manual.md`: "Auto-Reopen" section rewritten
  as "Auto-Reopen — and the chooser when your universe can't be found" + move-steps 3-4 updated.
  **The ×14 translations run after the Boss pass, before the final commit.**
- svelte-check: **1,633 files, 0 errors** (268 pre-existing warnings, none in the new code).
  Rust suite: **1,616 passed / 0 failed** (the 3 new PJ-433 pin tests included; a first
  background run exited 101 with output swallowed by my own Select-String filter — the identical
  re-run with full capture passed clean; cause of the transient not established, result verified).

---

## §7 — /simplify: 4 review agents, 14 findings applied, 4 skipped with reasons

**Applied (deduped across the reuse / simplification / efficiency / altitude agents):**
1. `RegistryStatus` mirror struct DELETED — `get_registry_status` returns `UniverseRegistry`
   itself (now `pub`, fields private; wire shape = disk shape, one struct).
2. Remove-successor SINGLE-SOURCED — one `removeSuccessor` derived; the dialog names it and
   `confirmRemove` opens the captured same entry (was: two independent computations agreeing
   only by a comment across a Rust-side sort).
3. `enterUniverse(entry, flushReason)` kernel — the wizard's create and the chooser's pick now
   share one enter path (stop-tracking → flush → activate → name → close gates → `finishBoot`);
   pick wraps it in catch-to-string + console.error (a post-activation throw is never silent).
4. `finishBoot` guards COMPLETION-keyed (`federationListenerArmed` / `bootListenersArmed` set
   after their blocks complete) — an `initializeApp` throw on the first entry no longer demotes
   every later entry to listener-less for the life of the process (altitude F1). Header comment
   re-scoped to the ENTER tail (the leave half stays per-door, noted).
5. **Efficiency HIGH:** the unconditional boot notify made the hidden second screen re-run its
   FULL `loadAllData` (per its own header, `collect_library_notes` opens every canonical note —
   ~8,000 files on the daily universe) on every normal boot. Fixed with a same-universe guard:
   `loadedUniversePath` recorded at load; the `onUniverseSwitch` handler refreshes the cheap
   title always and runs the dispose-and-rewalk only when the active path actually changed.
6. `check_universe_reachability` body moved into `tauri::async_runtime::spawn_blocking` (the
   dead-UNC hang the command exists for must pin a blocking-pool thread, not an async worker).
7. Store-level `getActiveUniverse()` (in-memory pointer ⋈ registry — deliberately NOT
   `active_id`, which is the persisted choice) converging the SS title helper and the
   `handleUniverseSwitch` title site; UniverseManager.refresh keeps its inline join (needs the
   sorted list for display) with a pointer comment.
8. BootChooser: `alive` flag dropped (post-destroy writes to local `$state` are inert in
   Svelte 5; interval still cleared in onDestroy), shared `runPick` bracket for both doors,
   `{@const r}` row lookup, button metrics aligned to the `.um-btn` kit.
9. Boot section: shared `armBootChooser()` (two copy-pasted arm sites folded), bare `find` (the
   null-ternary was redundant), `appReady`-guard comments corrected (the real exposure is the
   pre-existing mid-switch window, not the chooser), pick-comment's unreachable-path
   justification removed, `onBack` prop de-noised, `listUniverses` import dropped from +layout.

**Skipped, with reasons (filed for the ledger reconcile per SO#9.2):**
- **Shared universe-row component** (chooser row kit is a copy-adapt of UniverseManager's) —
  extracting mid-PJ restyles a Boss-validated surface right before the Boss test; metrics
  aligned now so the measured drift is zero; the extraction is a follow-up PJ.
- **Dispatcher-level `appReady` gate** (every palette command is dispatchable mid-switch;
  per-handler guards don't scale) — pre-existing exposure beyond this diff; follow-up PJ.
- **Leave-half generalization** (the remove-last→create door runs NONE of
  `handleUniverseSwitch`'s ~50-line residue sweep — pre-existing gap the altitude agent
  surfaced) — large refactor, its own job; follow-up PJ. `finishBoot`'s header no longer
  overclaims.
- **Per-tick full-list probe** (only the failed entry can flip "It's back") — chooser-scoped,
  negligible for local paths; simplicity wins.

Re-verification: **svelte-check 0 errors; cargo test 1,616 / 0** on the post-simplify tree.
(The two transient exit-101s were `LNK1104` — cargo invocations overlapping in one target dir
lock the test binary; the same run passes serial. Don't parallel-run cargo here.)

Diff-scoped safety inspection launched on the eight changed source files (`wf_cd4e1f1c-013`).

---

## §8 — Safety inspection: 2 confirmed LOW findings, both FIXED in-pass (WA#6)

**The inspection's two confirmed findings** (both chained on one mechanism — `finishBoot` has no
catch, and the one realistic unguarded throw site in the tail was the single-library
auto-expand's `await toggleLibrary` → `read_library_tree`, a fallible fs IPC):

- **F1 (false-success, LOW):** a post-activation `finishBoot` rejection returned its error into
  the ALREADY-UNMOUNTED chooser — an inert `$state` write; console-only, and devtools are off in
  release. The painted shell looked normal while everything after the throw silently didn't run.
- **F2 (resource-leak, LOW):** the tail's watcher/note-created/second-screen unlistens joined
  `cleanupFns` only in an end-of-block batch, and `bootListenersArmed` was completion-keyed — a
  mid-block throw left live untracked listeners AND a later re-entry re-registered the whole
  block (two 'library-changed' handlers sharing one debounce handle; the loser's pending Sets
  grow unboundedly; announces adopted twice).

**Fixes (all four applied, frontend-only):**
1. The auto-expand is guarded (`try/catch` + console.warn) — the realistic trigger is gone.
2. Every unlisten (and the keydown remover) joins `cleanupFns` AT its registration; the
   end-of-block batch deleted — a partial registration is always tracked and cleaned.
3. The listener block is now attempt-keyed AT ITS TOP (duplicates are worse than tracked
   partials, and with #2 partials are cleaned); `initializeApp` sits OUTSIDE the block, so its
   failure still lets a later entry register properly — the §7 completion-keying's purpose is
   preserved where it matters.
4. A post-activation pick failure RE-OPENS the chooser carrying the error
   (`bootChooserPickError` → `initialError` prop seeding `pickError`; the pick-one subheading is
   suppressed there — a choice IS recorded); Retry through the same door re-runs the tail, whose
   guards make re-entry safe. Never console-only again.

Re-verified: svelte-check **0 errors** (the intentional initial-capture seed carries a
`svelte-ignore` with its reason); Rust untouched since the 1,616/0 run.

---

## §9 — Release build; the test pipeline; a plan deviation stated honestly

- **Release binary built**: frontend first (46.4s; `bootChooser` confirmed embedded in `build/`,
  3 hits), then `cargo build --release` (2m18s, exit 0) →
  `src-tauri\target\release\constellation.exe`, **2026-08-31 15:15:37**, 96 MB. No Constellation
  instance running at build time.
- **The MSIX ghost, re-confirmed**: the sandbox's read of
  `%APPDATA%\world.uconstellation.app\universes.json` shows 277 bytes, ONE entry (كون عيسى),
  mtime 2026-08-07 — three weeks stale against the PJ-435 test nights, exactly the handover's
  warning. SESSION-LOG-2026-08-29 (§24) confirms the Boss tests the raw `target\release` exe BY
  PATH and those tests exercised multiple universes — so the raw exe's live registry ≠ what the
  sandbox reads. **No registry-file manipulation from here; on-screen evidence only.**
- **Desktop-control request DENIED by the Boss** (`request_access` → user_denied). Respected, not
  retried. Consequence — **a plan deviation, stated openly**: the plan's "Claude pre-tests the
  all-unreachable and pick-one states live via config swap" cannot be executed (the swap is
  registry manipulation, unreliable from the sandbox; and I cannot drive the app). Those two
  states ship covered by: the Rust pin test (`active_id = None` backend half), the boot-flow
  code's review by four simplify agents + the safety inspection + verifiers — but NOT seen live.
  The Boss-facing tutorial says so in its "does not cover" section, verbatim honest.
- **Test setup restructured for the denial**: a 2-minute SETUP stage the Boss performs in-app
  (launch by path → Universe Manager → Open Existing Universe → `E:\Constellation
  Universes\PJ433-Test`), with Claude doing only filesystem renames between stages. The fixture
  universe was created on disk (valid `.constellation/` structure + 2 notes, JSON-validated).
  Stage 5 extended to cover **Open from folder…** live (5B), with the expected PJ-435
  moved-universe bar called out as expected-and-safe.
- **tutorial-auditor** drafted the staged tutorial (its own verification caught that the
  Universe Manager CLOSES ITSELF on Switch — the recipe's Stage 4 gained the reopen step); I
  adjusted the draft for the setup-by-Boss structure; **ui-inspector** verified it (three
  rounds, 34+ claims: round 1 REJECTED on "the red Remove button" — the confirm button is red
  only on hover; round 2 APPROVED; a final scoped round after the panel's edits REJECTED once
  more on the chip-lag transient and supplied its own correction verbatim, applied). The
  **panel** ruled FIX-FIRST with six edits — the decisive one: **Stage 1b** (close at the
  chooser without clicking, relaunch, same screen returns), the ONLY observation proving
  "nothing is REMEMBERED until you click"; without it a build that silently persisted a
  substitute would have passed all five stages. All six applied.

---

## §10 — BOSS TEST: ALL STAGES PASSED (1, 1b, 2, 3, 4, 5A, 5B) — the gate is cleared

Run on the 15:15:37 release binary, launched by path. Every stage on the Boss's screen, with
two screenshots in evidence:

- **Setup** — the Boss registered + activated PJ433-Test through the Universe Manager's Open
  Existing door; Claude's only actions all test long were literal-path folder renames.
- **Stage 1 PASS** — the chooser, exactly as specified: banner ("Constellation could not open
  \"PJ433-Test\"" + Location + reason + the nothing-changed reassurance + Try again), and NINE
  registered universes listed Reachable — including موسوعة عيسى at `E:\موسوعة عيسى` (the
  may-live-anywhere ruling visible on screen) and the RTL rows rendering correctly. The
  screenshot also settled the environment question on-screen: the live registry ≠ the sandbox's
  277-byte ghost.
- **Stage 1b PASS** — close at the chooser without clicking, relaunch: the same screen returned
  still naming PJ433-Test. **Nothing was remembered** — the headline claim, proven live.
- **Stage 2 PASS** — folder renamed back while the chooser watched (16:51:13); "It's back —
  Open" lit on its own; his click opened PJ433-Test fully; typing normal.
- **Stage 3 PASS** — deliberate pick of Eisa Cognitive Knowledge from the chooser opened it
  fully; the relaunch opened it DIRECTLY (the pick persisted as his genuine choice).
- **Stage 4 PASS** — Switch to PJ433-Test, Remove-the-active: the confirm showed **"Constellation
  will then open: Eisa Cognitive Knowledge"** (screenshot) and confirming opened exactly that.
  Post-check from here: the folder + both notes + config INTACT on disk.
- **Stage 5A PASS** — the wizard reached from the chooser wears the context-only **Back**
  button (screenshot); Back returned to the chooser intact.
- **Stage 5B PASS** — "Open from folder…" at the MOVED location opened the universe with both
  notes AND armed the PJ-435 moved-universe banner ("Repair the index — safe, keeps
  everything") — **PJ-433 and PJ-435 interlocking live on one screen**, exactly as the manual
  now documents the move flow.

Honest coverage stance (stated in the tutorial and standing): the all-unreachable and
no-recorded-choice screens, the inline pick-failure path, the Unreachable chip on list rows,
and the not-a-directory wording were NOT seen live (desktop control declined; registry
manipulation unreliable from the sandbox) — covered by the Rust pin tests, four review agents,
the safety inspection, and 37 inspector-verified UI claims.

Close-out in flight: ×14 manuals/help translations (workflow `wf_2b422ed9-a7b`), fixture
cleanup pending the Boss's removal confirmation, then commit + PCS.

---

## §11 — Close-out: docs ×15, the Phase-4 audit (PASS), and a truth-sweep the Hindi file provoked

**Fixture cleanup:** the Boss removed PJ433-Test from his list; all three test folders
(`PJ433-Test`, `-HIDDEN`, `-MOVED`) deleted — zero PJ433 artifacts remain under
`E:\Constellation Universes`.

**Docs ×14 (`wf_2b422ed9-a7b`), 14/14 DONE.** Each locale's manual got the rewritten Auto-Reopen
section and the corrected move-steps, with every on-screen button name taken **byte-true from its
own locale's `bootChooser` strings** (أعد المحاولة / Wieder da — Öffnen / « Il est de retour —
Ouvrir » / 見つかりました — 開く / Снова на месте — Открыть …). Verified independently: 15 manual
files changed, and **no translated help set has a Universe topic folder at all** — so the agents'
uniform "help SKIPPED" was accurate, not a dodge (the Universe topic exists only in
`help.uConstellation.World`).

**The Hindi file provoked a real find.** Its agent flagged, outside its own task, that hi's
Portable-Universes section **still carried the false auto-repair promise** in its opening
paragraph ("Constellation will automatically detect and fix all internal paths") — the exact claim
the PJ-435 pass corrected everywhere else — and that hi lacks the **Full-re-read warning**, the
load-bearing safety point of that section. Per WA#6 that became a **14-locale truth sweep**
(`wf_4d5f0122-91f`): every manual checked for both defects, fixed only where actually present,
each repair-button string verified programmatically against its locale's
`indexDrift.movedRepairNow`.

**The sweep's result is the most consequential thing this close-out found — 14/14 agents,
verdicts recorded individually:**
- **CHECK 1 (the false auto-repair promise): 13 clean, 1 FIXED — Hindi only.** The PJ-435
  correction HAD reached the other twelve translations; hi was the single straggler, and it was
  found only because one agent looked outside its own task and said so.
- **CHECK 2 (the Full-re-read warning): ADDED in ALL FOURTEEN.** The load-bearing safety point of
  that whole section — *do not reach for a Full re-read to "fix" a move: it rebuilds from scratch
  and resets every link's birth date to today* — **existed in English and in no other language.**
  Every non-English reader of the manual had the move procedure without the one warning that
  protects the link graph's age. Now present in all fifteen, each with its own locale's
  `fullReread` command label and `movedRepairNow` button string byte-verified with node, not by
  eye. 14 files, +145/−61; no truncation, encodings and line endings preserved.

**Why this matters beyond the fix:** the PJ-435 pass corrected the *false* sentence everywhere but
never checked whether the *true* one had been carried. A correction sweep that only removes the
lie leaves the manual honest and incomplete — the warning is the half that actually protects the
user. Worth remembering as a method, not just a fix.

**Phase 4 (`/migration` audit) — PASS, `wf_d19f775a-17e`** (first run died whole on the model rate
limit; re-run clean on Opus 5):
- **4A invariants — all nine STILL HOLD, zero regressions, zero cannot-determine.** Including the
  two that most needed proving: PJ-435's heal still sits strictly after the reachability check
  (Err :1344 precedes heal :1426, untouched by the refactor), and the second-screen title
  semantics are *strengthened* — both `universes[0]` readers are gone.
- **4B drift — no new bypass.** Every `setActiveUniverse` caller accounted for; **no
  `listUniverses()[0]`-as-active reader remains anywhere in the repo**. One pre-existing gap
  (the leave-half → filed PJ-440) and one watch item (the wizard persists `active_id` mid-flow for
  a universe never entered — quitting mid-wizard makes it the recorded choice, but it then opens
  *honestly*, so it is not the silent-substitution shape).
- **4C migration path — PASS across all seven scenarios**, no data loss, no wedge. Downgrade
  tolerates `active_id: None` (it simply resumes the old silent fallback). Four notes: **M1** the
  corrupt-registry wizard (pre-existing → PJ-444), **M2** a failed "Open from folder…" still moves
  the recorded choice (→ **PJ-445**, a narrow contradiction of this feature's own promise, bounded
  and self-announcing — not fixed in-pass because the write order belongs to PJ-310/PJ-435's
  shared repoint path), **M3** the deliberate documented listener tradeoff, **M4** an imprecise
  code comment — **corrected in this commit** (the claim "errs before any write" holds for the
  unreachable path; what universally holds is that the RECORDED CHOICE is untouched, since
  `active_id` is written last).

**Ledger v2.09** (PJ-433 CLOSED with the seven-stage evidence; PJ-440…PJ-445 filed) and
**orientation v4.27** written.

---

## §12 — The per-cycle whole-app sweep: 19 confirmed, de-duplicated to 17, and a verdict on the LEDGER itself

**The non-result first.** The initial run (`wf_c2f63c5b-dea`) returned `confirmed_findings: []`
with **all 14 hunters dead on a model rate limit**. That is not a pass and was never recorded as
one. Re-run on Opus 5 as `wf_c684def0-3fa`: **65 agents, 0 errors, 19 confirmed.**

**De-duplication before filing** (`wf_de58824b-fbf`, one verifier per finding against the ledger +
Charter + both prior sweep registers, each also re-reading the cited code in the CURRENT tree):
**8 NEW → PJ-446…PJ-453** · **8 ALREADY-FILED** (PJ-396, PJ-378 ×2, PJ-348, PJ-347, PJ-264,
PJ-248 item 13, PJ-346) · **1 REFUTED-STALE** — the sweep's own claim withdrawn on re-reading
current code. Filing all nineteen would have inflated the backlog with the appearance of new work.

**The two HIGHs that are genuinely new:**
- **PJ-446** — `canonical.rs:1477` `ensure_cid_cn_cmd` is a bare `#[tauri::command]` that, since
  **PJ-431's fix at `4aee6ea2` (2026-08-29)**, runs `reindex_single_note` on the IPC dispatch
  thread, awaited on the note-open path. **PJ-431 re-introduced the exact class PJ-066 was opened
  to kill — six days ago**, which is why no earlier sweep could have caught it. Three independent
  hunters found it. Four sibling commands all carry `(async)`; this one does not. **One token.**
- **PJ-447** — `propsCommit.ts:110` emits a colliding property key as a SET on the existing key,
  silently overwriting it, defeating a **Boss-approved ruling** that a collision is reported and
  never resolved last-wins. Reachable by ordinary typing; the refusal guard `renamePropKeyIn` has
  **zero call sites in `src/`**. Measured by running the shipping code, not reasoned about.

**The structural finding — recorded because it indicts the METHOD, not the code.** Eight of
seventeen were already filed, some since 2026-08-11. **The ledger works as a NET (nothing was lost;
every one was findable) and fails as a QUEUE (nothing comes out).** The mechanism is two umbrella
entries — **PJ-264 (~100 unnumbered findings) and PJ-378 (58)** — inside which a defect is filed
but invisible to any human reader. The panel: *"This sweep spent most of its budget re-proving
known bugs. The cure is not a better sweep."*

**Panel ruling on what to do tonight:** FIX-NOW **PJ-446 only** (one token; a stall not corruption;
~2-minute Boss test: open three never-opened notes, confirm no stall), FILE the other sixteen —
PJ-447 explicitly deferred because its fix spans three files AND needs a decision about what the
user is *told* on a collision, and "a multi-file content-integrity change at the end of a long
session is how regressions ship."

**Two questions the panel declined and put to the Boss:** (1) ship PJ-446's one-word fix in
tonight's build for one short test, or file all seventeen and close the cycle with nothing to test?
(2) should the next cycle be a **DRAIN** cycle — fix the ~158 backlogged confirmed findings, run no
new hunt? Register appended to the Charter; all eight new PJs in ledger v2.09.

---

## §13 — THE BOSS RULED (2026-08-31): ship PJ-446 tonight, and the next cycle is a DRAIN cycle

1. **"Ship it — I'll test."** PJ-446's fix applied: `#[tauri::command]` → `#[tauri::command(async)]`
   at `canonical.rs:1477`, with the full reasoning recorded in a doc-comment AT THE SITE — the
   defect's whole history is that a later change (PJ-431) added heavy work to a command whose
   threading was invisible at the call site, so the comment is the guard against a third round.
   No contract change: the promise still resolves on completion; awaited callers stay correct.
2. **"A DRAIN cycle."** The next cycle fixes the backlog and runs **no new hunt**. Its first act is
   to unpack **PJ-264 (~100 unnumbered findings)** and **PJ-378 (58)** into individually numbered,
   visible, ranked entries — the panel's point being that a defect no human can see is not filed,
   it is buried, and another sweep would simply re-find it. PJ-434 and PJ-438 wait behind the
   drain. Ledger's ► Next action updated accordingly.

**Suite on the fixed tree: 1,616 / 0.** (Two intervening `LNK1104`s — no process held the binary;
transient AV lock on the freshly linked exe. Retried clean.)

**THE PANEL'S PROPOSED TEST WAS REJECTED BY ME — it could not have failed.** The panel asked for
"open three never-opened notes in Eisa Cognitive Knowledge." But the handover records his daily
universe at **27/8,033 with no `cid_cn` — 14 exempt templates + 13 real candidates** — so three
random notes would reach the fixed branch with probability ≈ 13/8,033 each. **The test would have
passed whether or not the fix works**, which fails the standing question *can this check disagree
with me?* Replaced with a fixture that reproduces the actual exposed population.

**The fixture, and why it is the right one — read from the code, not assumed.** A NEW universe
would NOT reproduce it: `mig003_backfill_cid_cn` is gated on
`stored_note_meta_version < NOTE_META_SCHEMA_VERSION` (`search.rs:4987`), and a fresh universe
runs it at first boot, stamping everything. The exposed population is specifically **a folder
brought into an ALREADY-migrated universe** — and the app's own comment confirms this is by
design: `+layout.svelte:6614-6618` — *"No bulk writes to the vault on import — the Living Link
identifier (cid_cn) is injected lazily on a per-note basis the first time Constellation actually
opens a note."* So: `E:\Constellation Universes\PJ446-Import-Test`, three unstamped notes, one
41 KB with 400 wikilinked sections, brought into one of his TEST universes.

**Test pipeline (auditor step folded — the panel authored the test's substance):** ui-inspector
**REJECTED** the first draft with 5 findings, one of which was **a number I got wrong** (I wrote
"13 of 8,033 unstamped"; the source says 27, of which 14 are exempt templates). Also corrected:
the door (verified click path — sidebar-footer **Universe** → **Own Libraries** → **+ Bring in a
library**, distinguished from the status-bar button), the dialog's real labels (**"Bring this
folder into your universe"** / **Copy in** / **Move in**, not "Copy"/"Move"), the removal route
(**Manage libraries...** → trash icon, not "the sidebar's library controls"), and an unverified
sibling-count softened to no number. It also independently confirmed **no guard refuses the
fixture**: `ensure_under_active_root` is only called by `add_library`, never on the Bring-In path.
Re-inspection in flight.

**Nothing commits until his pass** — his standing order, and the fix touches the note-open path of
every library that lacks identity stamps.

**BOSS TEST PASSED (2026-09-01)** on the 19:16:19 release binary: the unstamped import fixture was
brought into a test universe and its three notes opened — including the 41 KB, 400-section,
heavily-wikilinked one — with no stall and responsive typing. **PJ-446 CLOSED.** The commit gate
is cleared for both features of this session (PJ-433's seven stages + PJ-446).

---

## §14 — PJ-454 filed from the BOSS'S OWN REMARK, and the panel convened on it

Before running the PJ-446 test he wrote: *"Templates shouldn't have a constellation cid_cn stamp,
never, when created by a user. Because if Constellation assigns a cid_cn stamp to a template at the
time and date of creation, it will carry it all the time, and if the user tries to use any template
to create a new note out of it, Constellation will assume it was created at the time and date of
the cid_cn stamp."*

That is his own 2026-07-19 ruling (MIG-TPL §1) restated with its WHY. **Verified rather than
assumed — and the ruling turns out to be honoured on one side of the app and broken on the other:**

| | test for "is this a template" |
|---|---|
| **Rust** (`search.rs:4327-4332`) | `kind == "template"` **OR** under `templates_dir` — TWO arms, with a test fixture literally named **`stray`** for the outside-the-folder mold (`search.rs:17098-17100`) |
| **Frontend** (`store.ts:4875-4888` `isTemplatePath`) | **location only** |

The frontend predicate is the one gating the stamping call (`store.ts:3369`, `:3713` →
`ensure_cid_cn_cmd`), so **a `kind: template` file outside the templates folder IS stamped on first
open** — the mold acquires an identity, and every cast then inherits its birth date. Exactly his
scenario. Reachable by ordinary use: moving a template out of the folder, **changing the
`templateFolder` setting** (every existing template becomes a stray at once), setting
`kind: template` on a note in the Properties panel, or importing templates into another folder.
`rebrandCopyFrontmatter` (`store.ts:566`) strips `cid_cn` on the recovered-copy path ONLY.

**Filed PJ-454 (HIGH — Group 1).** On his instruction ("convene the panel to study and research
this issue thoroughly; I'll wait for their recommendations"), panel `wf_77c3801f-411`: four
parallel investigations (every stamping site + every template predicate; the cast path end to end;
blast radius INCLUDING a read-only check of his real data for already-stamped molds and for two
notes sharing one identity; WA#5 prior art — Obsidian/Templater, Logseq, Notion, Zettelkasten UID
practice, and the general prototype-vs-instance pattern), then three adversarial lenses
(correctness/Whole-Ecosystem, existing damage/migration, ruling-fidelity under File-Over-App),
then a chair verdict with ONE recommendation, a yes/no on repairing history, and the test that
would prove it. **No code written until he rules on their recommendation.**
