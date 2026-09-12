# Handover — 2026-09-01 session CLOSE (the drain cycle's first session)

**Read `docs/Constellation Orientation & Onboarding v4.32.md` first, then this.**
Supersedes `HANDOVER-2026-09-01.md`. Branch `main`, pushed, **tree clean at `82a574e4`**.
No untested code is on `main`.

---

## 1. The standing order — absorb this before anything else

> *"I've noticed you make a lot of mistakes: you assume something, take action, and then discover
> you were wrong. I want you to stop making mistakes. From now on, consult the panel before taking
> any action. Consider this an SO."* — Eisa, 2026-09-01

**The line HE ruled (do not re-interpret):** anything that writes, commits, builds or touches his
data goes to the panel first, with its verdict shown to him before proceeding. Reading and
measuring stay free — but any finding from them reaches him only after an independent check that
could contradict it.

**How it was applied this session, so you apply it the same way:** every code change was panelled
before design; every build had a diff-scoped safety inspection and every confirmed finding was fixed
and re-inspected before commit; every test went auditor → inspector → panel → Boss; every "done"
claim was verified by something that could disagree (disk read, compiled-output read, rerun).
Record writes (session log, ledger, orientation, MoCh) are the PCS obligation itself, not
panel-gated actions.

## 2. What shipped — all four Boss-tested before their commits

| commit | item | Boss test |
|---|---|---|
| `cc507561`, proven `b03ed34d` | **PJ-454 guard** (`canonical.rs` Two-Signal Choke Point) | 3 cases ✅ |
| `b03ed34d` | **PJ-455** tree refresh at the write + outside-universe folder warning | 4 steps ✅ |
| `0a820f96` | **PJ-454 repair door** (`MoldRepairDialog.svelte`, `mold_repair.rs` engine + `preview_mold_repair`) | Stages 1+2 on real files; **موسوعة عيسى's 4 repaired**, disk-confirmed ✅ |
| `82a574e4` | **PJ-460** porous blocked-screen curtain + complete running-state invariant | 4 steps ✅ |

**The count is 43** — 39 Eisa Universe, 4 موسوعة عيسى — reproduced twice. **102, 67, 0 are refuted.
Do not re-derive it; do not widen the rule** (widening is how 102 and 67 happened).

## 3. The repair door — what a fresh session must know to touch it safely

- **Own-libraries only.** A template repair is a WRITE; write sovereignty (MIG-111) keeps a linked
  universe's bookkeeping in ITS database. `scan_stamped_molds` uses `load_libraries` (the write
  path's own resolver); `repair_one` refuses any path outside own libraries. A mold in another
  universe is repaired when THAT universe is active.
- **The running-state invariant, whole:** *while the repair is busy or running, the dialog cannot be
  dismissed, no key reaches the app, and no tab may mount by any enumerated path.* Enforced by:
  `busy` latch + `close()` refusing while busy/running; ✕ hidden while busy/running; a window-capture
  keydown swallow of EVERY key while running; a shared `moldRepairRunning` signal (`store.ts`) read at
  the mount in `openNoteTab` (both mount points) and `loadTabHistoryEntry`; `if (!visible) return`
  after every await. The other `openNoteModel` sites (`reloadTabsFromDisk`, `renameItem`) re-seed
  tabs already in `openTabs` (which block the run); `restoreSessionTabs` is boot-only.
- **Six inspection rounds built that list.** If you change the dialog or `openNoteTab`, run the
  diff-scoped inspection again — it found something real every round until the sixth.
- **Unexercised, disclosed:** halt-on-failure and re-scan-between-batches have NEVER fired and have no
  test; scale (39 vs the tested 4) is untested; the four running-state guards were observed in the
  compiled build but never seen failing-and-recovering on his screen.

## 4. Next actions, in the ledger's order (v2.12 ► NEXT ACTION)

1. **Eisa Universe's 39 molds** through the same door, Eisa Universe active. **His click.** Offer him
   the alternative of running the failure rehearsal first, since scale is untested.
2. **The deliberate-failure rehearsal on copies** — break one file on purpose in a throwaway copy,
   watch the delicate batch halt before the ordinary files. Owed before PJ-456's 107-file wave.
3. **PJ-461** — `--bg-primary` / `--bg-active` / `--bg-modifier-border` do not exist in `theme.css`;
   `CalendarPanel.svelte:287,291`, `+layout.svelte:11553,12291,12309`. Read `theme.css`, rename to
   real tokens, verify each. (This is the class that made the repair dialog render black.)
4. Then the queue: **PJ-456** (approve by ENUMERATION in folder batches — no safe rule exists; the
   Boss's unfinished concrete-formwork note defeats every heuristic) · **PJ-457** (the 18 GB duplicate:
   link the original as a Linked Universe, verify 13 links, bin the copy — his call) · **PJ-458** ·
   **PJ-459** · **PJ-462** (after the drain, his ruling) · **PJ-463** · the multi-folder templates
   setting (Boss-ruled for this cycle) · unpack **PJ-264 / PJ-378** · PJ-434, PJ-438.

## 5. Method lessons paid for this session (each cost a round trip)

- **Verify a compiled fix by printing and READING the emitted code around a literal unique to it.**
  Never tighten or loosen a regex on a theory: two "strict" checks returned 0 because `)` inside a
  compiled getter broke the character class — the code was there.
- **Locate a build artefact by a symbol unique to the code under test** (a command name, a trace
  literal, a scoped class) — never by a translated string or a shared class name. A same-hash chunk
  filename means "content unchanged" — that is a signal, not noise.
- **The mount is the tab-store write, not the model call.** A gate before `openNoteModel` guards
  nothing; the editor mounts from `tab.content`.
- **State the invariant whole before fixing its first edge.** Six rounds of one-sibling-at-a-time
  came from never writing "while busy or running, nothing may…" in full at the start.
- **A check that returned 1 for the wrong file and a check that returned 0 for the wrong regex are
  the same error.** Ask: if my method were wrong, would this look different?
- **"Tested" means HE tested it.** The suite passing is "suite green", never "tested".
- **An empty findings list is a pass only if the agents completed.** Read the failure count.
- **Warnings carried:** `%APPDATA%` reads hit MSIX virtualization — on-screen evidence only;
  `LNK1104` linker locks are transient — retry serially; `cargo build --release` does NOT rebuild the
  frontend — `npm run build` first, then grep `build/` for the change.

## 6. Ready-to-paste prompt for the next session

```
Read docs/Constellation Orientation & Onboarding v4.32.md first, then
lab/reports/HANDOVER-2026-09-01-close.md. This is a DRAIN cycle — fix the backlog, run NO new
whole-app hunt (Boss ruling; it also suspends the per-cycle whole-app sweep until the drain is done).

STANDING ORDER, issued 2026-09-01 and binding: consult the panel before taking any action. Anything
that writes, commits, builds or touches his data goes to the panel first, with its verdict shown to
him before proceeding. Reading and measuring are free, but any finding from them reaches him only
after an independent check that could contradict it. Boss tests every build before its commit —
"tested" means HE tested it.

STATE: main is clean at 82a574e4. Four Boss-validated closes this cycle: the PJ-454 guard, PJ-455,
the PJ-454 repair door (موسوعة عيسى's 4 of the 43 molds repaired, disk-confirmed), and PJ-460 (the
door's porous blocked-screen curtain + the complete running-state invariant, six inspection rounds).
The count is 43 (39 Eisa Universe, 4 موسوعة عيسى) — do NOT re-derive it or widen the rule; 102/67/0
are refuted.

NEXT ACTION: Eisa Universe's 39 molds through the same door with Eisa Universe active — the Boss's
click; offer him the failure rehearsal first, since scale (39 vs 4) is untested. Then the
deliberate-failure rehearsal on copies (halt-on-failure has NEVER fired; no test). Then PJ-461
(non-existent CSS variable names in CalendarPanel + layout — read theme.css, rename to real tokens).

QUEUED with briefs in ledger v2.12: PJ-456 (107-file second wave — approve BY ENUMERATION in folder
batches, never by predicate), PJ-457 (the 18 GB duplicate inside Eisa Universe — link the original,
verify 13 links, bin the copy; his call), PJ-458, PJ-459, PJ-462 (after the drain, his ruling),
PJ-463, the multi-folder templates setting, then unpack PJ-264 and PJ-378, then PJ-434, PJ-438.

METHOD: verify a compiled fix by printing and READING the emitted code around a literal unique to
it — never by tightening a regex; locate a chunk by a symbol unique to the code under test, never a
translated string; the mount is the tab-store write, not the model call; state an invariant whole
before fixing its first edge; an empty findings list is a pass only if the agents completed.
```
