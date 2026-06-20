# Next-session kickoff prompt — paste this to resume

> Ready-to-paste prompt for the session after MIG-082 (the clickable, 8-calendar Calendar) completed. Copy everything in the box.

---

We just completed **MIG-082 — the clickable, 8-calendar Calendar** (Gregorian, Hijri, Solar-Hijri, Hebrew, Indian, Buddhist, Chinese, Korean): interactive day cells, per-calendar year + native/phonetic month-name settings, the daily-note Hijri stamp, and a Gregorian→cultural date converter in any note's Properties. It's fully committed (HEAD `5925e783`), Phase-4 **audited** (3 P1s found + fixed, including a real BUG-015 F2 fix to already-shipped §A.3 code), and `/simplify`-clean.

Before anything, please:
1. `git pull origin main` (sync).
2. Read **`docs/Constellation Orientation & Onboarding v2.93.md`** FIRST — the preamble carries the full MIG-082 picture.
3. Skim **`lab/reports/HANDOVER-2026-06-20-mig082-complete.md`** (state + what's next) and **`lab/reports/SESSION-LOG-2026-06-19.md`** (the shipped record).

Then we pick the next move (your call):
- **Docs translation catch-up** — the in-app i18n is complete ×15, but the **User Manual §14** + the **help-site** calendar topic were updated **EN-only**; the ×14 translations are pending (a Workflow fan-out would clear them).
- The deferred **inspector360 Settings-UI bug**.
- **Boot performance** — the remaining boot cost is the Sky read (~234k sky_links); defer it off boot / move to write-time derivation (per `project_mig079_boot_wtd`).
- A new feature you have in mind.

What would you like to tackle?

---
