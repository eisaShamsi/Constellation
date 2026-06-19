# Next-session kickoff prompt — paste this to resume

> Ready-to-paste prompt for the session that picks up MIG-080 §B–§F (the right-rail note-context cascade). Copy everything in the box.

---

Working on: **MIG-080 §B–§F** — the right-sidebar note-context cascade (the right rail becomes the OPEN NOTE's context only; universe functions relocate). The Plan is already approved.

First read `docs/Constellation Orientation & Onboarding v2.91.md`, then `lab/reports/HANDOVER-2026-06-19-mig081-complete.md`, then `lab/reports/SESSION-LOG-2026-06-19.md`, then the Plan `docs/MIG-080-Plan.md` (+ Architect `docs/MIG-080-Architect-Right-Sidebar-Note-Context.md`).

Context (all SHIPPED + on `main`): MIG-079 §C.2a–d (per-note link queries + virtualized panels + Sky deferred; boot freeze fixed). MIG-080 §A/§A.2 (Calendar → left dock launcher + full-page view, validated). **MIG-081 COMPLETE + Boss-validated** — Eisa's astronomical Hijri engine + cultural calendars + a rich CalendarPanel + Month Correction + Calculation Mode + a per-element Style-Setter "Calendar" category + the Daily-Note launcher retired + full ×15 localization + a clean 3-agent migration audit.

THE TASK — MIG-080 §B–§F (approved Plan; cascade, stop at each phase's Boss test):
- **§B** Tags "All tags" → Dashboard (reuse §C.1 `tag_counts`).
- **§C** Tasks split — contextual open-note tasks in the right rail + a left agenda; fix `toggle_task` bypassing the Editor reindex gate.
- **§D** Source Review split — note-scoped right rail + the universe Cataloger on the left.
- **§E** Knowledge Health split — note tensions/health in the right rail | universe → Dashboard.
- **§F** Review Pulse split — this note's review status in the right rail | universe queue → a full-page reviewer; fix the dead `record_note_visit`.
- **§G** reconcile + a 3-agent audit + the **deferred inspector360 Settings-UI bug** (it's missing from the Settings → Panels placement list; needs ×15 i18n) + ×15.

Standing orders that bit last session — honor them: **cargo runs from `src-tauri/`** (root has no Cargo.toml); close the app before `cargo build --release`; frontend change → `npm run build` THEN cargo, grep `build/` for a new string. **Full-localization is a TOP PRINCIPAL** — any new UI label gets added to its i18n block ×15 (Settings AND `styleSetter.labels` if it's a Style-Setter control), not just EN. Staged Boss tests (one stage, wait, next). Stop-On-Correction. Measure-don't-guess. SO #8 cross-check a deferred item against the orientation BODY + session logs before tackling. Plan-approval = build-approval (cascade; stop only at Boss-testable verification points + genuine architectural surprise). Do the full closing PCS + handover + next prompt at session end (commit to `main`; orientation v-bump in the SAME commit).
