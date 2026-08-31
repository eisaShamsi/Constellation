# PJ-433 — Silent Boot Fallback — Architect (Phase 1)

**Date:** 2026-08-31 · **Status:** Phase 1 complete, panel review appended below · **Workflow:** `wf_c64b6c15-e8b` (5 territory maps + synthesis), panel `wf_38ca0d52-dc7`

**Concept (the horse):** at boot, the app must never substitute a different universe for the one
the user chose without telling them — and must never record its own substitution as the user's
choice. The function (the carriage): the boot-activation flow and its notice.

**Ledger entry:** `docs/Constellation Pending Jobs v2.08.md` §PJ-433 (HIGH — Group 1). SO#8
cross-check 2026-08-31: entry NOT stale; line numbers drifted (ledger's 3644-3647 → 3712-3725;
1245 → 1424), mechanism verified live.

---

## 1. Current state (all file:line read this session by the mapping agents)

Boot loads the registry (`+layout.svelte:3691`) and walks entries in active-first order
(`universe.rs:880-893`), calling `setActiveUniverse` per entry with a silent `catch { continue; }`
that discards the error string (`+layout.svelte:3712-3725`). An unreachable path errors at
`universe.rs:1277` before any durable mutation, but a **successful fallback** activation persists
`active_id` LAST (`universe.rs:1424-1425`) — recording the substitution as choice. If nothing
activates, boot shows `UniverseSetup` (`+layout.svelte:3726-3729`), a first-run wizard that never
lists registry entries (no `listUniverses()` call in the component; its "Open Existing Universe"
is only a folder picker). No boot-error surface exists (before `appReady` only the setup overlay
and a bare spinner render; every notice band renders inside the post-activation `.app` shell),
and no "why did activation fail" IPC exists (`set_active_universe` returns `Result<(), String>`;
the boot loop discards the message). A second silent-substitution writer exists:
`remove_universe_from_registry` repoints `active_id` to `entries.first()` (`universe.rs:1443-1444`).

### Side-effect order of `set_active_universe` (universe.rs:1212-1428)

1. `switch_lock` acquire :1219-1223 (no mutation)
2. Idempotency check (read-only) :1235-1248 — early `Ok` if already active
3. `load_registry_for_update` :1250 — on a CORRUPT registry calls `set_aside_corrupt` (:129), a disk mutation
4. Path check :1260; healing branch :1263-1275 (`save_registry` at :1272 only when a parent heal succeeds); otherwise **Err :1277**
5. `migrate_to_constellation` :1282 → `ensure_universe_notes_folder` :1285 → same-name consolidation :1287-1354
6. `heal_paths_after_move` :1359 (PJ-435 relocation arming — AFTER the reachability check; unreachable never arms/disarms)
7. In-memory `active_path` :1363-1365; `universe_lock::activate` :1369; cache + search invalidation :1373/:1383; Arabic overrides :1398-1420
8. **`active_id` persist LAST** :1424-1425

**On Err for a plain unreachable path, nothing durable has mutated** (`active_path` stays
None/previous, so the MIG-079 guard falls through on retry and a retry fully re-attempts).

### Every writer of `registry.active_id`

- `:986` `create_universe` — only if `active_id.is_none()`
- `:1424` `set_active_universe` — **the PJ-433 persist**
- `:1443-1444` `remove_universe_from_registry` — silent repoint to `entries.first()`
- `:1586, :1615, :1633` `open_existing_universe` (register-only doors; PJ-310/PJ-435 repoints)
- `:1813` `link_library_as_universe` — unconditional write; activation left to caller
- `:2505-2508` `migrate_legacy_data` — also sets in-memory `active_path` directly :2512-2515 WITHOUT invalidations (pre-existing half-activation)

### Consumers of `list_universes`' active-first sort

- The boot loop (`+layout.svelte:3715-3725`) — order = try-order
- Second screen title: `SecondScreenPage.svelte:960-966` + `:735-738` take `universes[0].name`
  (Display-Not-Domain: the second screen never activates)
- `+layout.svelte:3417-3420` and `UniverseManager.svelte:47-53` match by
  `get_active_universe_path` — NOT sort-dependent; `LibrarySwitcher.svelte:56-57` count only

### Deliberate-switch surfaces (Whole-Ecosystem enumeration)

`setActiveUniverse` wrapper `src/lib/universe/store.ts:30-32`. Callers: boot loop
(`+layout.svelte:3717`); `handleUniverseCreated` (`:3240`); `UniverseManager.svelte:68`
(switch click; also `confirmRemove` :96 auto-switches after removing the active),
`:167` (Open Existing footer); `UniverseSetup.svelte:114/:131/:349`. The UniverseManager modal
(status bar `sb-universe` button, `+layout.svelte:10819`) lists name + full path + ACTIVE badge +
Switch/Remove — **no reachability indicator** (its `refresh()` never checks the filesystem).
No retry-unreachable UI exists anywhere.

### Reusable notice vocabulary

- **Notice band** (post-activation): `.notice-band` `+layout.svelte:8015`; PJ-435 moved row
  :8092-8100 (`drift-note` + primary button + dismiss ✕; message `$derived.by` :742-751)
- **Federation warning badge**: `.sb-federation-warning` :10812 + popup :10827-10848 listing
  path + reason per warning
- **Boot-gate precedent — CORRECTED BY THE PANEL (Claim 5, PARTLY)**: the `Mig108UnifyDialog`
  gate (`+layout.svelte:3046-3068`) is the CRASH-RESUME JOURNAL gate, and it parks only the
  background boot fan-out — `appReady` flips at :2811 and the full UI paints first; every dialog
  exit (including "Not now") releases it, plus an unconditional 30s failsafe. **No surface in
  Constellation today truly wedges boot.** The real reusable precedent for a must-answer boot
  surface is the `showUniverseSetup` blocking mount (`+layout.svelte:7995-8003`)
- i18n: 15 locales confirmed (`src/lib/i18n/{ar,de,en,es,fa,fr,he,hi,ja,ko,pt,ru,tr,ur,zh}.json`);
  `indexDrift` / `federation` / `mig108` are the section precedents; `tOr()` fallback pattern

### Side effects of opening the WRONG universe (measured)

- **MIG-100 session: the intended universe's session.json is SAFE** — both IPCs take an explicit
  root (`universe.rs:2194-2198, 2224, 2246`); the tracker arms for the actually-activated root
  only (`session.ts:419-427`); restore origin-check discards foreign payloads (`session.ts:365-368`).
  Minor bleed: the crash sentinel `constellation-session-restoring` is a GLOBAL localStorage key
  (`session.ts:228, 352-359`).
- **localStorage is global, not universe-keyed**: `constellation-recent-opened` /
  `constellation-recent-edited` (`recentNotes.ts:20-21`), `constellation-search-history`
  (`searchHistory.ts:11`), `index-excluded-terms` (`IndexPanel.svelte:361,366`) — working in the
  fallback pollutes these; no corruption path into the intended universe found.
- **PJ-435 relocation records cannot interact with an unreachable universe** (armed at :1359,
  after the reachability check; foreign/unreadable records deleted at activation).
- **Switch-back when the drive returns is clean** (`UniverseManager.handleSwitch` → departure
  flush → activation; a failed switch leaves the desk untouched, error shown).

## 2. Design options

| | Speed | Effort | Risk class | Failure modes | Concept honor |
|---|---|---|---|---|---|
| **A. Boot-blocking chooser** | One extra click when original unreachable; instant otherwise | **M-L**: new pre-`appReady` surface (reuse UniverseManager list pattern; the `showUniverseSetup` blocking mount is the real precedent — the mig108 gate is NOT, see §1 correction), Rust error-detail plumbing, i18n ×15 | Boot-path regression; chooser must not orphan the all-unreachable→setup path | Chooser itself unreachable state; Retry loop UX on flaky drives | **Full** — nothing opens, nothing persists until explicit pick |
| **B. Fallback-with-banner, no persist** | Unblocked immediately | **M**: skip-persist flag in `set_active_universe`, notice-band row, i18n ×15 | **In-memory active ≠ persisted `active_id`**: the active-first sort then misleads its consumers — second-screen title would name the unreachable universe; next-boot loop order changes | localStorage bleed while working in fallback; banner dismissed and forgotten | Substitution visible, not recorded — but substitution still happens |
| **C. Minimal: no-persist + status badge** | Unblocked immediately | **S**: persist change + federation-badge pattern | Same sort-divergence as B | Badge easily missed — weakest honesty | Not recorded, but arguably still "silent" — borderline fail |

## 3. Invariants that must not break

- Empty-registry first-launch → setup (`+layout.svelte:3699-3709`)
- MIG-079 idempotency guard: a failed try leaves `active_path` unset → retry fully re-attempts (`universe.rs:1224-1248`)
- `switch_lock` serialization (`universe.rs:1219-1223`)
- PJ-435: `heal_paths_after_move` runs only after the reachability check — unreachable never arms/disarms relocation
- MIG-100 write-authority: tracker arms for the actually-activated root only — intended universe's session.json safe under all options
- All-unreachable path must still reach a create/open door
- Second-screen read-only title semantics (depends on the active-first sort; B/C break its meaning, A does not)

## 4. Migration / rollback

No `universes.json` schema change is required for any option (A/B/C only *withhold* a write).
If B persists a "fallback" marker, downgrade behavior of unknown fields is NOT VERIFIED.
**First-boot-after-update on a machine already living on a persisted fallback:** `active_id`
already records the old substitution and no record of original intent exists — unrecoverable;
the fix is forward-only, honestly. Downgrade simply resumes persisting fallbacks; no data damage.

## 5. Open questions for the Boss

1. Is being auto-opened into a *different* universe ever acceptable, even with a banner (A vs B)? Product call.
2. Should "Remove from list" appear on the boot screen, when the drive may merely be unplugged?
3. `remove_universe_from_registry`'s silent repoint (`universe.rs:1444`) is the same class — fix in this pass (Whole-Ecosystem Fix Law) or separate PJ?
4. When all entries are unreachable, should the chooser list them (with reasons) *alongside* the setup doors, replacing today's amnesiac wizard?

---

## 6. Panel verdict (appended 2026-08-31 — workflow `wf_38ca0d52-dc7`: 6 default-REFUTED verifiers + 4 conflicting lenses + synthesis chair)

### Verifier verdicts on the load-bearing claims

| Claim | Verdict |
|---|---|
| 1. Unreachable path → Err at :1277 with zero durable mutation (incl. `load_registry_for_update` write-free when registry parses; the corrupt-branch `set_aside_corrupt` cannot reach the :1277 Err) | **CONFIRMED** |
| 2. `UniverseSetup` never lists registry entries; "Open Existing" is only a folder picker | **CONFIRMED** |
| 3. Second screen titles from `universes[0]` (persisted-`active_id` sort) at TWO sites (`SecondScreenPage.svelte:735-739, 960-968`); it is the ONLY consumer equating entry[0] with the open universe. (`UniverseManager.svelte:48` carries a stale comment about the mechanism; its code doesn't depend on position) | **CONFIRMED** |
| 4. MIG-100 write-authority arms for the actually-activated root only; intended universe's session.json safe (file is `src/lib/libraries/session.ts`; :365-368 is restore-origin discard, arming is :419-427 + call-site root) | **CONFIRMED** (citation nit) |
| 5. No boot-error surface (true) + "mig108 gate = boot-blocking precedent" | **PARTLY** — see §1 correction: no surface truly wedges boot today; the chooser would be the codebase's FIRST must-answer boot surface |
| 6. `remove_universe_from_registry` silently repoints to `entries.first()`; sole caller `UniverseManager.confirmRemove` already performs the explicit successor switch itself, making the backend repoint redundant as well as silent (it also never touches in-memory `active_path` — the already-filed PJ-322 gap) | **CONFIRMED** |

### The ruling (unanimous on the option — all four lenses chose A independently)

**Option A in the A-LEAN shape, with two adopted refinements:**

- **A-LEAN** (boot-and-simplicity lens): render the **Boot Chooser** under the EXISTING
  pre-`appReady` gate (`showUniverseSetup`, `+layout.svelte:7995-8003`) as a **sibling component
  of the wizard, never a mode flag inside it**. It names the unreachable universe, its path, and
  the reason; lists the registry with reachability (cheap fs metadata, computed only when this
  screen opens — never on a healthy boot); offers Retry, an explicit pick, and Create-new (the
  demoted wizard). Nothing activates and nothing persists until the user clicks. This also cures
  the amnesiac wizard (verified Claim 2) in the same stroke — same gate, same concern, one
  component; the wizard component itself stays untouched.
- **mount-watch** (real-world-usage lens): while the chooser is open, poll the missing path; on
  reappearance light "It's back — Open" — makes the commonest portable-drive case (drive mounts
  20s after auto-start) near zero-cost.
- **A′** (data-safety lens): close the SECOND silent-substitution writer in the same pass —
  `remove_universe_from_registry:1443-1444` stops guessing a successor; the confirm dialog names
  it (frontend already switches explicitly, `UniverseManager.svelte:94-97`).

**Two decisive reasons.** (1) Which universe is open is the identity decision of the whole app —
it arms session write-authority and addresses every save and every `earned.jsonl` record, and no
tooling exists to unmix two universes' earned records; B and C substitute first and explain
after — the rulings forbid the substitution itself, not merely its silence, and only A makes
wrong-universe authorship impossible. (2) B/C create a standing invariant (in-memory active ≠
persisted `active_id`) that verified Claim 3 shows would make the second screen title itself with
a universe that is not open — a sustained display lie of the PJ-435 class — and in the dead-drive
case B/C nag forever with no move-on affordance, while A's chooser terminates it in one
persisted, genuine pick. A costs nothing on a healthy boot: all new code sits on the failure
branch. **Dissent: none on the option**; the split was only among refinements, resolved by
adopting all three as one shape.

### Panel rulings on the scope questions

- **Q3 — fix the remove-repoint in this pass** (the concept is "never record a substitution as
  the user's choice"; the Whole-Ecosystem Fix Law forbids fixing one writer and leaving the
  other; it is small).
- **Q4 — yes, fold the amnesiac-wizard fix in** (the chooser IS the fix for Claim 2; leaving a
  registry-blind wizard behind the same door would ship the defect A exists to cure).

### Declined — the Boss's calls

- Architect open questions **1 and 2** go to the Boss verbatim.
- One taste call the panel surfaced and declines: **when the drive reappears while the chooser is
  open** — auto-open his persisted choice (honoring a decision already made) or wait for the one
  click (Constellation Way's letter). Both are honest; which is Constellation is his.

### Red flags for Phase 2

1. **First-ever boot wedge.** Every chooser exit must terminate somewhere live (Retry / pick /
   create / quit); empty-registry and all-unreachable must still reach the wizard
   (`+layout.svelte:3726-3729`). A chooser that throws pre-mount bricks every boot.
2. **Re-entry must be the whole boot.** A pick must run the full `handleUniverseCreated` path
   (`+layout.svelte:3232-3244`: stop tracking → flush/clear tabs → `setActiveUniverse` →
   `initializeApp()`), never a partial resume; and mount-watch polling must not race a user click
   past `switch_lock` / the MIG-079 idempotency guard.
3. **entry[0] consumers while nothing is active.** While the chooser is up, `universes[0]` is the
   unreachable universe — gate second-screen spawn on actual activation; fix the stale
   `UniverseManager.svelte:48` comment in passing. Keep `heal_paths_after_move` strictly after
   the reachability check (`universe.rs:1359` after the :1277 return) — do not reorder it in the
   refactor.
