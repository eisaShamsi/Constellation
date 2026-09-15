# Session log — 2026-09-15

## §1 — PJ-466 CLOSED, Boss-passed on his screen (`5e3e6857`)

**Function in hand:** the Mold Repair Door's end-of-run receipt (`MoldRepairDialog.svelte`,
`mold_repair.rs`, `moldRepair.*` ×15).

### His rulings today
- **Run PJ-466 now**, ahead of PJ-456's 107-file wave.
- **File** the two sibling-door defects separately (PJ-489…PJ-492) rather than fold them in.
- **The heading is "What the fix did"**; the noun is **"template file"**, including on the Fix
  button he wrote.
- On the wording: **"I want to hear from the panel."**
- **NEW STANDING ORDER:** *"Bring the tutorial in detailed steps here, and consider it a standing
  order for my future tests and examinations."* → `CLAUDE.md` Testing Instructions Rule **point 5**,
  memory `feedback_tests_in_chat_not_files.md`. The `lab/reports/` copy is the ARCHIVE; the chat is
  the delivery. Write both.
- **Clear the older residue** in Scratch.

### The test — staged, because there was nothing left to fix
The shipping scan against all four real registries returned **0 / 0 / 0 / 0**. PJ-454 closed all 43,
so the door cannot open on real data. Three practice files were staged in Scratch only; every note in
that universe was checked against the rule and none other qualifies.

**Result: PASS, all eight items** (his two screenshots + report). The crown item: `•` Practice Note
Template C.md — *"The file is no longer there."* listed FIRST, with `✓` A and `✓` B after it. The
delicate batch ran, found a file with nothing left to do, called it **Kept**, and **carried on**.
Verified against HEAD that the counterfactual is true: old `repair_one` returned `ok:false` for a
missing file → `failed = outcomes.len() - repaired` counted it → `if (r1.failed > 0)` halted → A and
B would never have been submitted.

**Disk check afterwards (mine):** A and B carry `kind: template` with no root `cid_cn`; C absent; two
backups written 09:36; the scan now returns **0 / 0 / 0**.

### Cleanup, per the promise in the tutorial
This round's practice folder and its two backups removed immediately after his report, unasked. On
his word, the **older residue** was cleared too: `mold-rehearsal/` and `mold-rehearsal-2/` (4 files
each, from 09-11 and 09-13) and **10** old backup files. Nothing named "rehearsal" or "Practice"
remains in Scratch. **Residual, disclosed:** the 8 deleted files still have rows in Scratch's
`search.db`, so Scratch will raise a drift/phantom notice on its next boot — Constellation's own
machinery working as designed. Filed as **PJ-495**.

### §2 — PCS
- Code + i18n + test archive: `5e3e6857` (pushed).
- Ledger **v2.16** — PJ-466 / PJ-469 / PJ-470 / PJ-471 CLOSED; **PJ-493 / PJ-494 / PJ-495** filed;
  the `%APPDATA%`-is-not-readable method note recorded.
- Orientation **v4.35** — the full account, including the four defects of mine and the new SO.
- MoCh `docs/MoCh/MoCh-2026-09-14-1730.md`.
- Handover `lab/reports/HANDOVER-2026-09-15.md`.
- **SO#2 (help + User Manual): 10 documents updated. My first answer here was WRONG and is corrected
  in place, because the correction is the lesson.**
  I wrote "no change — the door has no help topic and no manual passage", then ran the check. It
  disagreed: `docs/help.uConstellation.World/Templates/Templates.md` and `docs/User Manual.md` both
  describe this door in detail, and **both carried the sentence PJ-466 had just proved false** —
  *"Every file is backed up first."* A file refused before the backup step gets no backup.
  - **Then I mis-measured the translations.** I grepped the 14 translated manuals for the ENGLISH
    button label and got 0/14, and nearly recorded "none carry it". Of course they do not contain an
    English string. Re-probing with **each locale's own shipped `moldRepair.recheck` value** gave the
    true answer, and it matched the wrap panel's figure exactly: **8 carry the passage**
    (ar de es fa ja ko ru zh), **6 do not** (fr he hi pt tr ur).
  - **Updated:** the English help topic + English manual (corrected sentence + a new "What the
    receipt tells you" section), and all **8** translated manuals — each correcting its own
    over-promise in place and gaining the receipt paragraph, quoting **its own** shipped
    `moldRepair.summaryTitle` / `sum*` strings rather than a re-translation. Verified after the fact:
    all 8 now contain their own app heading.
  - **Filed PJ-496** for the 6 manuals that have no passage at all — a pre-existing gap this work
    widens.
  - **The method point:** a probe in the wrong language returns a confident zero. The right probe was
    the app's own translated string. Same shape as the `%APPDATA%` finding on the same day: *a check
    that cannot disagree is worthless, and a check aimed at the wrong artefact is worse — it
    disagrees for the wrong reason.*

### §3 — What is next
**► PJ-475** — switch a Saved Style from the command palette. **His own request**, timed by him for
after PJ-461; PJ-466 was taken first on his ruling. `loadStylePresets()` and `applyPreset()` already
exist and are standalone; only app-level wiring is missing. Three open questions are in the ledger
entry — and under the 2026-09-14 law, each must be checked against what the Style Setter already
offers before any of them reaches him.

---

## §4 — CORRECTION: I nearly replaced a TRUE mechanism with a FALSE one, in four documents

Recorded at length because the near-miss is the lesson, and because the same trap caught two
different actors in one day.

**What I had recorded (correctly, in mechanism):** `%APPDATA%/world.uconstellation.app/universes.json`
is a frozen 7-August copy; a copy also sits under the Claude package's `LocalCache/Roaming/`; treat
it as a virtualized read. A wrap-panel lens had moved to BLOCK the whole PJ-466 Boss test on the
strength of that file, and its adversary refuted it with `E:`-resident witnesses.

**What I then talked myself into.** Preparing PJ-475 I read `style-presets.json` from that same
folder and it worked. I then noticed `write-journal.jsonl` there was written **today at 09:36** by
the app — its last two entries are this session's own mold repair, matching the file mtime to the
millisecond. I concluded: the folder is live, this session reads it fine, the "virtualized shadow"
story is false, and the truth is merely the known §17 mystery. **I said so to the Boss and was about
to rewrite the orientation, the ledger, the session log and the handover.**

**The inference does not hold, and a `findings-verifier` refuted it.** A pass-through file always
looks live whether or not its neighbour is shadowed — a cross-check that cannot disagree. The
decisive test is file IDENTITY, which I did not run: `fsutil file queryfileid` returns the **same
NTFS file ID** for both paths. They are not two byte-identical copies; they are **one file object
reached by two paths**. `fsutil hardlink list` on the `%APPDATA%` path returns ONLY the container
name — which a genuine hardlink never does (proved against a positive control on `notepad.exe`,
which lists all three of its names). **I reproduced both results myself before accepting them.**

**And the shadowed file IS stale, provably, without reading it.** `save_registry` sits on
`set_active_universe`'s unconditional success path (`universe.rs:1490-1494`); boot calls
`setActiveUniverse` (`+layout.svelte:4356`); the idempotent guard cannot suppress it on a cold
start. The live journal carries **37** `session_restore_begin` markers after 2026-08-07 09:56, and
`diagnostics.log` records **zero** registry-save failures. `save_registry` uses `atomic_write`
(temp + rename), which mints a new file ID each write — which is exactly why the container's
snapshot stays pinned to the old one.

**The correct statement, which is what now stands in all four documents:** the trap is **PER-FILE**.
`fsutil hardlink list <path>` — `Error 50` = genuine; a `LocalCache` path = shadowed. In that one
folder, `universes.json` is shadowed and `style-presets.json`, `app-prefs.json` and
`write-journal.jsonl` are genuine. So my original rule was right in mechanism and **overstated in
scope**; my "correction" would have been a second false mechanism. The registry's real CONTENTS
remain unverifiable from this session.

**A third instrument failed too, and it was mine.** My own quick per-file loop reported
`universes.json` as "pass-through (genuine)" — the opposite of what the raw command says. The
matcher, not the evidence, was wrong. Three checks in one hour, and the only ones that held were the
two that could disagree with me.

**Two documents corrected beyond the note itself:**
- The orientation contradicted ITSELF: a v4.x passage concluded from the same redirect that *"the
  registry really does hold one entry, and Eisa Cognitive Knowledge is not in it"* — the trap, one
  paragraph after correctly naming it. Corrected in place.
- §17's 2026-08-07 entry has the same contaminated premises (*"has not been written since 09:56"*,
  *"two `universes.json` files exist"*). Its ADVICE stands; its premises are amended. The mystery it
  describes may not exist at all.
- **PJ-233 rests on the same read and is still carried as OPEN.** PJ-321 was CLOSED for exactly this
  trap (*"the app was never at fault; I was reading a shadow copy"*). PJ-233 is now flagged to be
  re-verified before any code is written for it.

---

## §5 — PJ-475 PREPARED for a fresh session (no code written)

**Function in hand:** the command palette's command list — adding one row per Saved Style.

**His instruction:** *"Prepare PJ-475 for a new session, after PCS + Orientation."*

### Under the 2026-09-14 law, I opened the surfaces before framing anything — and two of the three filed questions were never questions
- **Stable ids?** Already exist. `StylePreset.id` is a `crypto.randomUUID()`, and Hotkeys binds by
  arbitrary command id. Binding comes free.
- **Refresh on rename/delete?** Dissolved by the build: one list, one owner, nothing to refresh.
- **Mark the active style?** **KILLED on measurement** — and this is the one worth remembering. Three
  of four lenses wanted it; one claimed to have proved it derivable on his real data, and its proof
  covered five of six sections, omitting the one that breaks it. `applyPreset` MERGES `linkColors` by
  design; his registry resolves to 11 link types and "Eisa Default" carries 9, so the predicate is
  FALSE for that style one second after he applies it — permanently dark for one row in three, correct
  for the other two, so any test written against the other two confirms a broken predicate.

### THE ENTRY'S OWN PREMISE WAS FALSE, and I had written it
PJ-475 said the presets load "only when the Setter opens". `<StyleSetter />` is mounted
unconditionally (`+layout.svelte:10613`, one tab of indent); the `{#if $styleSetterOpen}` gates the
markup, not the lifecycle. They already load at first paint on every boot. **I handed that false
"measured fact" to the panel in its brief**; a lens caught it by opening the file. Building on it
would have added a second boot-time IPC. Corrected in the ledger entry itself.

### The finding that changes the job — measured by me on his live data
Applying a Saved Style **REPLACES** `styleOverride`; the main window then removes vanished variables.
`Eisa Cognitive Knowledge` (174 tuned values): "Eisa Default" → 120, **losing 54**; "Eisa Default 02"
→ 157, losing 17; "تنسيق عيسى الرئيس" → 174, losing none. Reproduced in Scratch (173 → 120). None of
the lost variables has a `:root` fallback. **No settings undo exists anywhere.** The Setter's Keep
MERGES while apply REPLACES — an asymmetry nothing documents. PJ-475 therefore ships an Undo, and the
underlying decision is **PJ-497 (Group 1)**.

### Filed: PJ-497…PJ-503 — seven defects, none introduced by this feature
Two Group 1: the lossy replace, and a failed link-type write making the whole apply a **silent
no-op** (the named app-killer class — `applyPreset` awaits `saveLinkTypes` first, which throws on its
latch, and neither door catches it).

### Deliverables
`lab/reports/PJ-475-BRIEF-2026-09-15.md` (build-ready, written for a session that has never seen this
code) · ledger **v2.17** · orientation **v4.36**.
