# Handover — 2026-07-12 — MIG-100 CLOSED · backlog re-prioritized (PJ v1.18) · SO#9 process live

**Read `docs/Constellation Orientation & Onboarding v3.39.md` first** (highest version). Then this. Then `git pull origin main` and `git log --oneline -10`.

---

## 1. What shipped this arc (verified + protected)

- **MIG-100 Auto-Restore Tabs — SHIPPED, Boss-validated ×3, migration CLOSED.** Open tabs + active tab + split come back on relaunch; per-Universe `.constellation/session.json`; Settings toggle default ON; 15 locales. Phase-4 close audit PASSED (7/7 invariants, 6/6 migration-path); 3 deferred-cid lifecycle leaks fixed pre-close; **10 pre-existing app-killer-class defects fixed in passing**. Tag `milestone/mig-100-auto-restore-tabs`; backup zip in `E:/Backups/Constellation/`. Full detail: `SESSION-LOG-2026-07-11.md`.
- **The whole outstanding backlog re-verified + re-prioritized → `docs/Constellation Pending Jobs v1.18.md`.** 16 stale PJs closed, 13 new filed (PJ-070→082), 5 priority groups, a standing "► Next action" line.
- **Standing Order #9 (new, in CLAUDE.md):** the PJ ledger is the living backlog, reconciled at the close of EVERY job (first file opened). End-guard pairing SO#8.

## 2. State of standing

- **Verified-shipped + protected:** everything in §1. Working tree clean, all pushed (HEAD `9fbc17d4`). Latest release binary 2026-07-12 10:31 (MIG-100 complete). vitest 318/318, cargo 1047, svelte-check 0.
- **At-risk / uncommitted:** none.
- **Known-broken:** none new. Two pre-existing APP-KILLERs now tracked as **PJ-070** (watcher external-change clobber) + **PJ-071** (bulk Accept-All race) — not yet fixed.
- **Open investigation:** **PJ-072** — the running instance boots universes ("Eisa Cognitive Knowledge" 7,751 notes, "Scratch") absent from every on-disk `universes.json` findable on the machine. Needs a diagnostic build logging the resolved `app_data_dir` + registry path. Does not block anything; genuinely not understood yet.
- **Doc drift:** none introduced. `PJ-081` tracks the pre-existing §12 drift batch (incl. the now-stale "no test harness" row) + the orientation BODY refresh (§3/§4.x/§13/§17 still ~v1.58 — current truth lives in stacked preambles).

## 3. NEXT — Boss-ruled: start Group 1, top-down. First item = PJ-070.

**PJ-070 — the watcher external-change APP-KILLER.** An external `.md` edit on an OPEN note (git-pull / Syncthing / Obsidian) updates only `tab.content`, never the single-ownership note model / `reloadVersion`, so the mounted editor + model keep the stale body and the user's next keystroke's debounced save durably overwrites the external edit (then reindexes, so search agrees with the stomp). It is the **main-window mirror of the closed G3 cross-window class** — the fix reuses the proven `SecondScreenPage.adoptFreshDiskIntoSS` pattern (adopt disk into the model + bump `reloadVersion` for the `{#key}` remount). Site: `+layout.svelte:3172` (the watcher external-change flush). It is a write-path / lifecycle change → **Reproduce-First → `/migration`**.

Group 1 order after PJ-070: PJ-071 (bulk-accept race) → PJ-073 (G4 YAML parser) → PJ-074 (G5 durable rename) → PJ-072 (registry diagnostic) → PJ-075/076 (G6/G7 residuals). Full grouping in Pending Jobs v1.18.

## 4. Standing rules that bite here

- **SO#9 (new):** at the close of PJ-070 (and every job), reconcile the PJ ledger FIRST — close PJ-070 with evidence, file anything it surfaces, re-rank, bump to v1.19. In the same commit as the work.
- **SO#8:** cross-check PJ-070 against the orientation BODY + session logs before starting (it's fresh — filed 2026-07-12 — so low staleness risk, but confirm the `+layout.svelte:3172` line + the `adoptFreshDiskIntoSS` pattern still exist).
- **Reproduce-First (top principal):** no defect-targeting change ships before the defect is reproduced on demand. Instrument the write journal (`write-journal.jsonl`) + reproduce the external-edit clobber before designing the fix. `svelte-check`/`vitest` are NOT runtime verification for editor-lifecycle bugs.
- **Every build:** diff-scoped `safety-inspection` (this touches a write/lifecycle path → NOT exempt); `/simplify`; `svelte-check` 0/0; `npm run build` BEFORE `cargo build --release`; verify binary mtime newer than source before any Boss test.
- **Art Director & Team own UX/UI** (not relevant to PJ-070 — it's backend/lifecycle, no visual design).
- **Test tutorials:** staged, one stage at a time, tutorial-style.

## 5. Where to resume

`lab/reports/NEXT-SESSION-PROMPT.md` is the ready-to-paste kickoff. Full narrative: `SESSION-LOG-2026-07-11.md` (§ Post-close 2026-07-12). Conversational trace: `docs/MoCh/MoCh-2026-07-12-0900.md`. The backlog: `docs/Constellation Pending Jobs v1.18.md`.
