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
