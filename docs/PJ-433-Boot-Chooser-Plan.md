# PJ-433 — Boot Chooser — Plan (Phase 2)

**Date:** 2026-08-31 · **Status:** FINAL, presented to the Boss for approval · **Workflow:**
`wf_77362636-844` (planner + WA#5 proven-methods cross-check + 3 adversarial reviews + finalizer)
· **Architect:** `docs/PJ-433-Silent-Boot-Fallback-Architect.md` (panel verdict in its §6)

**Concept (the horse):** at boot, the app must never substitute a different universe for the one
the user chose without telling them — and must never record its own substitution as the user's
choice.

**The Boss's rulings (2026-08-31, final):** (1) fix shape = the **Boot Chooser** (panel A-LEAN +
mount-watch + A′); (2) **no "Remove from list"** on the boot screen — removal stays in the
Universe Manager; (3) drive-returns behavior = **wait for the click** ("It's back — Open" lights;
no auto-open).

**⚠ Verification-method warning (carried from HANDOVER-2026-08-31):** sandbox reads/writes of
`%APPDATA%\world.uconstellation.app\universes.json` hit **MSIX virtualization** — the file the
sandbox sees can be a stale ghost of the file the installed app uses. Every verification clause
below that inspects or hand-edits the registry must go through the fsutil hardlink route or be
confirmed by **on-screen evidence in the running app**, never by a naive `%APPDATA%` read.
(Dev builds run un-virtualized; the trap is the installed/MSIX context.)

---

All load-bearing anchors re-verified against source by the finalizer (`universe.rs:880-893` — no
`active_id` in the `list_universes` payload, confirming the CRITICAL finding; `+layout.svelte:3232-3244`,
`:3689-3729`, `:7995-8003`; `tauri.conf.json:21-29` — the second-screen window is pre-created at
launch, `visible:false`).

**Function in hand:** the boot-activation flow — replacing the silent fallback loop
(`+layout.svelte:3712-3725`) with a must-answer Boot Chooser.

## Step 1 — Rust: registry status + reachability commands
**Commit: "PJ-433 §1 — get_registry_status + check_universe_reachability"**

- `src-tauri/src/universe.rs`, beside `list_universes` (:879-893), **two** commands sharing one
  internal helper:
  - `get_registry_status` — registry read only, **zero fs probes**:
    `{ active_id: Option<String>, entries: Vec<UniverseEntry> }`. Called once per boot
    **[absorbs Attack1-F1 / Attack2-1 / Attack3-F1 — the CRITICAL: boot must know `active_id`
    before attempting; the draft's "report only on failure" was self-contradictory]**. The "zero
    new IPC on happy path" claim restated honestly: one registry-read IPC per boot, replacing the
    existing `listUniverses()` call at :3691 (net zero).
  - `check_universe_reachability` — **`async fn`** (fs probes can block seconds on dead UNC
    paths — the PJ-066 class) **[absorbs Attack1-F5]**: per-entry `Path::is_dir()`, reasons as
    machine keys `not-found` / `not-a-directory` only. The `healable-parent` variant is
    **dropped** — the heal at :1263-1275 runs inside `set_active_universe` at attempt time, so
    the key would be dead vocabulary ×15 **[absorbs Attack1-F6]**. Chooser renders the raw
    `set_active_universe` error string as fallback for reasons the probe can't classify (`:1256`
    not-in-registry, `:1282-1285` migrate/ensure errors) **[absorbs Attack3-F6]**.
- Called only while the chooser is open or at boot-failure time — never on a healthy boot.
  Register both in `main.rs`; wrappers in `src/lib/universe/store.ts` beside `listUniverses` (:20-22).
- **Verification:** `cargo build` green; healthy boot makes zero `check_universe_reachability`
  calls (grep call sites); real vs renamed folder returns correct reachable/reason in dev test.

## Step 2 — A′: remove the second silent-substitution writer
**Commit: "PJ-433 §2 — remove_universe_from_registry stops guessing"**

- `universe.rs:1439-1447`: removed-was-active → `active_id = None` (never `entries.first()`).
  `create_universe:986` tolerates `None` — legal state.
- `UniverseManager.svelte` `confirmRemove` (:81-99): dialog names the successor; switch at
  :94-97 stays the explicit act. New i18n key (§6).
- Crash window (`active_id: None`, entries remain) → Step 3 routes to the chooser's **pick-one
  state** — genuinely reachable because `get_registry_status` runs every boot **[fixes the
  draft's broken verification, per all three attacks]**.
- **Verification:** remove non-active → no change; remove active → dialog names successor,
  registry matches switched-to id; hand-edit `active_id: null` (⚠ fsutil route) → next boot
  shows chooser pick-one state, nothing activates, nothing persists.

## Step 3 — Boot-loop rewrite + extracted continuation
**Commit: "PJ-433 §3 — boot activates only the recorded choice"**

- **Extract `finishBoot()`** — the full post-activation tail: federation:ready listener
  registration (:3731-3812, MIG-061 §J.2 ordering preserved), `initializeApp()`, post-init
  refreshes (:3820-3848), watcher/later onMount steps (:3814+), and `notifyUniverseSwitch` for
  the pre-created second-screen window **[absorbs Attack1-F2 HIGH — `handleUniverseCreated`
  alone is a partial resume: no federation listener, no watcher → silent index drift; and
  Attack3-F4 — the SS window exists from process launch and needs the switch notify]**. Normal
  boot, chooser pick, and wizard create all run it (idempotency guard for the once-per-process
  listener).
- Rewrite :3689-3729: `getRegistryStatus()` replaces `listUniverses()`. Empty registry /
  migration (:3699-3709) unchanged → wizard. `active_id` set + entry present → attempt **that
  entry only**; success → `finishBoot()`. `active_id: null` or dangling (∉ entries —
  **[absorbs Attack3-F2]**) with entries present → chooser pick-one state, no false
  "last-active" banner. Failure → capture error, `await checkUniverseReachability()` **inside
  try/catch — any throw still sets `showBootChooser` with degraded props (raw error, no
  per-entry status)**; never an unhandled throw to the bare spinner **[absorbs Attack3-F5,
  RF1]**. No other entry tried; nothing persists.
- Honest-by-design lines stated in code comments: (a) already-persisted historical fallback =
  forward-only per Architect §4; (b) the write-then-activate doors (`open_existing_universe`
  :1586/:1615/:1633, `link_library_as_universe` :1813, wizard :114/:131) and
  `migrate_legacy_data` (:2505-2515) record genuine explicit picks — out of scope **[absorbs
  Attack2-5/6]**. Corrupt-registry lenient-load (`:154-157` → empty vec → wizard →
  `set_aside_corrupt`) is **known-out-of-scope: PJ filed at ledger reconcile** **[absorbs
  Attack3-F3 honestly]**.
- **Verification:** healthy boot unchanged (daily universe opens, boot time unregressed,
  idempotent re-persist at :1424 harmless); renamed folder → chooser, `universes.json`
  byte-unchanged after close (⚠ on-screen / fsutil evidence); `active_id: null` → pick-one
  state; empty registry → wizard.

## Step 4 — BootChooser component + gate (RF1, RF2)
**Commit: "PJ-433 §4 — the Boot Chooser"**

- New `src/lib/components/BootChooser.svelte` — sibling of the wizard, never a mode flag inside
  it. Gate: `{#if showBootChooser}` → BootChooser · `{:else if showUniverseSetup}` → wizard ·
  `{:else if !appReady}` → spinner · `{:else}` → app.
- **Pick path** = wrapper, not bare `handleUniverseCreated` **[absorbs Attack1-F3 HIGH +
  Attack2-4]**: try { stop-tracking → flush/clear (no-op on empty desk) → `setActiveUniverse` →
  **only on success** `showBootChooser = false` → `finishBoot()` } catch { refresh reachability,
  render inline error, chooser stays mounted }. Buttons single-flight; `switch_lock`
  (:1219-1223) serializes anyway. A failed pick never lands on the bare spinner.
- **Exits (RF1 — every state terminates live):** Retry (full idempotent re-attempt; MIG-079
  guard verified — failed try leaves `active_path` unset); per-reachable-entry **Open**;
  **"Open from folder…"** — routes to the wizard's existing Open-Existing door
  (`open_existing_universe` PJ-310/PJ-435 repoint path), honestly labeled — the
  Lightroom/Obsidian "Locate" affordance **[absorbs Attack2-2 + cross-check adoption 1]**;
  **Create new** → wizard. All-unreachable state = full list + reasons + both doors (the
  amnesiac-wizard cure). **No Remove button** (ruling 2 — deliberately declining Obsidian's
  pattern; Boss ruling overrides industry).
- **Wizard return door [absorbs Attack1-F4 / Attack2-3]:** one additive optional prop `onBack`
  on `UniverseSetup`, rendered only when provided (chooser context); wizard behavior elsewhere
  byte-identical. **Flagged for explicit Boss approval with this plan**, not discovered mid-build.
- **Mount-watch (ruling 3):** `setInterval` ~3s, in-flight guard, **epoch/mounted guard so a
  stale in-flight result resolving after activation is discarded** **[absorbs Attack1-F5]**,
  cleared in `onDestroy`. Reappearance lights **"It's back — Open"** — state change only;
  polling never calls `setActiveUniverse` (cannot race `switch_lock`/MIG-079). Wait for the click.
- **Verification:** all exits terminate live; failed pick returns to chooser with inline error;
  pick runs `finishBoot()` (federation listener + watcher armed — verify watcher fires on an
  external file edit after pick); rename-back lights button, no auto-open; timers cleared
  post-activation; wizard-back returns to chooser with list intact.

## Step 5 — RF3: second-screen consumers + stale comment
**Commit: "PJ-433 §5 — second screen titles from actual activation"**

- `SecondScreenPage.svelte` :735-739, :960-968: title from the entry matching
  `get_active_universe_path`, fallback "Constellation" — never `universes[0]` blind. The window
  pre-exists (`tauri.conf.json:21-29`) and mounts during boot — the draft's "unreachable while
  chooser is up" reasoning was wrong; the title fix + Step-3 `notifyUniverseSwitch` in
  `finishBoot()` are the actual cures **[absorbs Attack3-F4]**.
- Belt-and-braces `appReady` guard inside `handleToggleSecondScreen`/`handleSendToSecondScreen`
  at the `openSecondScreenSmart` call sites **:6097/:6122** (draft's `:7889` was not a spawn
  affordance — corrected) **[absorbs Attack1-F7]**.
- Fix stale `UniverseManager.svelte:48` comment. **No edit near `universe.rs:1359`**; audit
  clause asserts heal still sits after the :1277 return.
- **Verification:** with chooser up, SS shows no wrong universe name; after pick, SS titles the
  opened universe; grep confirms :1359 untouched.

## Step 6 — i18n ×15
**Commit: "PJ-433 §6 — locale keys"** — `universe.bootChooser.*` (en.json `universe` :2349):
`title, couldNotOpen, pathLabel, reasonNotFound, reasonNotDirectory, retry, open, openFromFolder,
itsBack, createNew, back, allUnreachable, noActiveRecorded, reachable, unreachable, pickFailed` +
`universe.removeConfirmSuccessor`. (No `reasonHealable` — dropped per Step 1.) All 15 locales;
root `dir={$dir}`; paths in `dir="ltr"` spans. **Verification:** `scripts/i18n-parity.mjs` +
vitest green; ar RTL renders unbroken.

## Step 7 — Help + User Manual ×15
**Commit: "PJ-433 §7 — docs"** — help topic "When your universe can't be found at startup"
(covering chooser, It's-back, Open-from-folder, remove-successor dialog) in
`docs/help.uConstellation.World/` + `docs/User Manual.md` + 14 translations. **Verification:**
present in all 15.

## Step 8 — /simplify + diff-scoped safety-inspection
On the full diff; `files:` = all changed files. Every confirmed finding fixed pre-commit.

## Step 9 — Boss test (tutorial-auditor → ui-inspector → panel → Boss), staged
Never touches the daily universe; recipe below.

**Skipped as false-positive:** none — every review finding was real and absorbed. Two
cross-check suggestions explicitly declined: per-entry Remove (Boss ruling 2) and
remember-the-skip (no source showed it; nothing to adopt).

---

## Risk → mitigating step

| Risk | Step |
|---|---|
| Silent-guess reintroduced via blind `universes[0]` (CRITICAL) | 1 + 3 (`get_registry_status` before any attempt) |
| RF1 boot wedge (throw pre-mount / failed pick → spinner) | 3 (degraded-props catch) + 4 (wrapper, chooser stays mounted) |
| RF2 partial resume (no federation listener / watcher) | 3 (`finishBoot()` shared continuation) |
| Poll races a click / stale result mutates state | 4 (poll never activates; epoch guard; `switch_lock`) |
| RF3 second screen titles from unreachable entry | 5 + 3 (notify) |
| Dead-UNC probe blocks | 1 (async fn) |
| Create-new one-way strand | 4 (`onBack` prop, Boss-approved with plan) |
| Moved universe has no door | 4 (Open from folder…) |
| Boot-perf regression | 3 verification (net-zero IPC, measured on daily universe) |

**Rollback.** Behavior-only, zero schema change, zero new persisted state. Reverting the commits
restores the old loop wholesale; `active_id: None` (Step 2) is already legal for old code
(`create_universe:986` tolerates it), and old boots tolerate None/dangling `active_id` without
wedge or data loss (Attack-3 downgrade check). Forward-only honesty per Architect §4 stands — no
attempt to un-persist historical silent fallbacks.

**Boss test (staged, after §8).** Recipe: Claude creates a `PJ433-Test` universe, activates it,
closes the app, renames its folder. **Stage 1:** launch → chooser names it + path + reason; the
daily universe listed reachable; nothing opened; registry unchanged (on-screen evidence).
**Stage 2:** Claude renames the folder back while the chooser is open → "It's back — Open"
lights; Boss clicks; full boot (tabs, federation, watcher live). **Stage 3:** re-break; Boss
picks the daily universe from the list. **Stage 4:** Boss removes the active test universe in
Universe Manager → dialog names successor. **Stage 5:** Boss clicks Create new, then Back →
chooser returns intact. All-unreachable + pick-one (`active_id: null`) states: tested by Claude
pre-Boss via backed-up/restored config swap through the fsutil route, his data untouched.

**WA#5 verdict:** the Boot Chooser matches the strongest proven pattern (Lightroom's blocking
must-answer dialog + Obsidian's picker-on-launch, now with their Locate affordance adopted as
"Open from folder…") and avoids both documented anti-patterns (VS Code's silent dead-restore,
Logseq's in-place disabled editor) — battle-tested, not inventive.
