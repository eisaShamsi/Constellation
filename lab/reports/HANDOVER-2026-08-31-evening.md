# Handover — 2026-08-31 evening / 2026-09-01 (after PJ-433, PJ-446, and the PJ-454 panel)

**Read `docs/Constellation Orientation & Onboarding v4.28.md` first, then this.**

> **UPDATED 2026-09-01 after the session continued past the PJ-433 close.** Everything below
> section 1 still stands; sections 4 and 6 are superseded by this box.
>
> **Shipped and Boss-passed since:** **PJ-446** — `ensure_cid_cn_cmd` gains `(async)`; PJ-431 had
> put a full note re-index on the IPC dispatch thread six days earlier, re-introducing the PJ-066
> freeze class. Committed with PJ-433 at **`5e56c00a`**.
>
> **► THE NEXT CYCLE IS A DRAIN CYCLE (Boss-ruled), AND ITS FIRST ITEM IS PJ-454.**
> The per-cycle whole-app sweep found the ledger is a working NET and a failing QUEUE — ~158
> confirmed findings sit invisible inside PJ-264 (≈100) and PJ-378 (58), and the sweep spent most
> of its budget re-proving known bugs. **Fix the backlog; run NO new hunt.** Unpacking those two
> umbrella entries into numbered, visible, ranked items is the drain's second act.
>
> **PJ-454 — the drain's FIRST act, already panelled and Boss-ruled.** A template outside the
> templates folder gets stamped with an identity on first open (Rust tests `kind` OR location; the
> frontend guard that gates stamping tests location ONLY), so a cast would inherit the mold's birth
> date. **Measured on his disk: 102 stamped molds** — 82 Eisa Universe, 19 موسوعة عيسى, 1 daily.
> **No cast inherited one** (`create_note` strips identity keys, `libraries.rs:1666-1686`,
> regression-tested). Ten paths can stamp; the guard covers two; the primitive
> (`canonical.rs:1449`) has none.
> **Approved fix: the Two-Signal Choke Point** — the test moves into the engine that writes the
> stamp and asks BOTH questions, strip-at-creation kept as an independent second layer.
> **His rulings: repair the 102 — YES; show him the exact file list FIRST (never automatic);
> build in the drain cycle.** The brief for producing that list safely is in PJ-454's ledger entry
> — follow it literally, because misidentifying a real note silently severs its earned history.
>
> **Then:** PJ-434 (unreachable Linked Universe reported present-and-empty), PJ-438; PJ-437
> (identity-relative addressing) remains the direction that outlives them all.
(The morning's handover, `HANDOVER-2026-08-31.md`, opened this session — it stays as written.)

Branch `main`, pushed. This session ran the **entire PJ-433 `/migration` end to end**: Architect →
adversarial panel → three Boss rulings → plan → build → `/simplify` → safety inspection → the
auditor/inspector/panel test pipeline → **seven live Boss stages, all passed** → close.

---

## 1. Where things stand in one paragraph

**PJ-433 is CLOSED.** When the recorded universe cannot be activated at boot, Constellation now
opens **nothing** and persists **nothing**: the **Boot Chooser** names the universe, its path and
the reason, lists every registered universe with live reachability, and waits for a click. The
Boss ruled its three open questions (the chooser shape; **no Remove** on the boot screen; **wait
for the click** when the drive returns) and passed all seven stages on his screen — including
**Stage 1b**, the panel's own addition, which is the only observation that proves *nothing is
remembered*. `A′` rode along: removing the active universe no longer guesses a successor, and the
dialog **names** the one it will open. Two pre-existing defects were cured in passing (a
wizard-created universe used to run with no file-watcher and no federation listener until restart;
the "Welcome" wizard no longer appears when universes are merely unreachable). **Next: PJ-434.**

## 2. The single most important thing to absorb

**A sweep that removes a lie is only half the job — check that the truth was carried.**

The ×14 manual pass provoked a truth sweep when one agent flagged, *outside its own task*, that
the Hindi manual still carried the PJ-435-era false auto-repair promise. Checking all fourteen:
the false promise survived in **Hindi only**. But the **Full-re-read warning was missing from ALL
FOURTEEN translations** — *do not reach for a Full re-read to "fix" a move; it rebuilds from
scratch and resets every link's birth date to today* existed in English and nowhere else. The
PJ-435 pass had deleted the false sentence everywhere and never verified the true one had been
carried, so **every non-English reader had the move procedure without the one warning that
protects the link graph's age.** Now in all fifteen, each anchored to its own locale's
`movedRepairNow` + `fullReread` strings, verified programmatically.

Corollary, equally cheap to remember: **let an agent's out-of-scope flag interrupt you.** The most
valuable finding of the whole close-out arrived as "outside what you asked me…".

## 3. What is verified-shipped and protected

- **PJ-433**, all nine plan steps + 14 `/simplify` fixes + 2 inspection fixes. Boss-passed
  7/7 stages on the 15:15:37 release binary. Suite **1,616/0**; svelte-check **0 errors**; i18n
  parity **15/15**.
- **Phase-4 audit PASS:** 4A — all nine invariants hold, zero regressions (PJ-435's heal still
  strictly after the reachability check; second-screen title semantics *strengthened*). 4B — no
  new bypass; **no `listUniverses()[0]`-as-active reader remains anywhere in the repo**. 4C —
  all seven upgrade/downgrade scenarios pass; downgrade tolerates `active_id: None`.
- **Docs ×15** — the Boot Chooser documented in the English manual + help topic and all 14
  translations, plus the truth-sweep repairs above.

## 4. Known-broken / at-risk / open

- **PJ-434 (► next)** — an unreachable Linked Universe reported present-and-empty; two walkers of
  one concern disagree, and a dead link cannot be retracted without hand-editing `universe.json`.
  Same honesty family as PJ-433, one level up.
- **PJ-440…PJ-445, filed this session.** PJ-440 (MED) is the notable one: PJ-433 generalized the
  **enter** half of a universe change but not the **leave** half — the remove-last→create door
  runs none of `handleUniverseSwitch`'s ~50-line residue sweep (pre-existing; each line there is
  a measured leak). PJ-445 is this feature's own narrow seam: a *failed* "Open from folder…" still
  moves the recorded choice, because `open_existing_universe` writes `active_id` before
  activation (bounded, self-announcing; the write order belongs to PJ-310/PJ-435's shared path).
- **PJ-331 (UI half)** still the most visible naming drift: the running app shows "Child
  Universes"/"cUniverse" on 7+ label keys ×15 locales.
- **Four PJ-433 states ship code-verified but NEVER SEEN LIVE**: all-unreachable,
  no-recorded-choice, the inline pick-failure path, the Unreachable chip on a list row. Desktop
  control was declined and sandbox registry manipulation is unreliable.

## 5. Measured facts worth not re-deriving

- The Boss's live registry holds **nine** universes (his screen), including `موسوعة عيسى` at
  `E:\موسوعة عيسى`. **The sandbox's `%APPDATA%\world.uconstellation.app\universes.json` shows ONE
  entry with a three-week-stale mtime — the MSIX ghost, re-confirmed. On-screen evidence only.**
- `list_universes` hides `active_id` behind its sort — **that is where the silent fallback was
  born**. `get_registry_status` exists so a caller can tell a choice from a guess.
- An unconditional boot notify makes the hidden second screen re-run `collect_library_notes`
  across every library (~8,000 note files on the daily universe). Guarded by a same-universe check
  in `SecondScreenPage`'s switch handler — do not remove it.
- Overlapping `cargo` invocations in one target dir produce `LNK1104`; run them serially.

## 6. Ready-to-paste prompt for the next session

```
Read docs/Constellation Orientation & Onboarding v4.28.md first, then
lab/reports/HANDOVER-2026-08-31-evening.md. PJ-433 (the Boot Chooser) and PJ-446 (the
ensure_cid_cn_cmd freeze) are both CLOSED — Boss-passed, committed at 5e56c00a. The ledger is
docs/Constellation Pending Jobs v2.09.md.

THIS IS A DRAIN CYCLE — Boss-ruled. Fix the backlog; run NO new whole-app hunt. The last sweep
found the ledger works as a net and fails as a queue: ~158 confirmed findings sit invisible
inside PJ-264 (~100) and PJ-378 (58), and the sweep spent most of its budget re-proving known
bugs.

FIRST ITEM: PJ-454, already panelled and Boss-ruled — do not re-panel it. A template outside the
templates folder is stamped with an identity on first open (Rust tests kind OR location; the
frontend guard that gates stamping tests location ONLY), so a cast would inherit the mold's birth
date. 102 stamped molds are measured on his disk (82 Eisa Universe, 19 موسوعة عيسى, 1 daily); no
cast inherited one. Approved fix: the Two-Signal Choke Point — move the test INTO the engine that
writes the stamp (canonical.rs ensure_cid_cn) and ask BOTH questions, keeping strip-at-creation as
an independent second layer. His rulings: repair the 102 = YES; show him the EXACT FILE LIST for
approval FIRST, never an automatic pass; snapshot-first. Follow PJ-454's ledger brief literally
when producing that list — a "mold" carrying earned records or inbound identity links is NOT a
mold, and stripping identity from a real note silently severs its earned history.

Then: unpack PJ-264 and PJ-378 into numbered visible entries; then PJ-434, PJ-438; PJ-437
(identity-relative addressing) remains the direction that outlives them all.

Standing orders: panel before any ruling request, tutorial-auditor → ui-inspector → panel before
any test reaches the Boss, findings-verifier before any factual claim lands, Boss tests every
build before commit. Three warnings carried: sandbox reads of %APPDATA% hit MSIX virtualization
for this app's registry (his live registry holds NINE universes; the file the sandbox sees shows
one) — trust on-screen evidence only; a workflow returning an empty findings list may have had
every agent die on a rate limit — read the failure count before believing it; and reject a test
that cannot fail (the panel's own PJ-446 test would have passed whether or not the fix worked).
```
