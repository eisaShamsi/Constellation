# Next-session kickoff prompt — paste this to resume

> Ready-to-paste prompt for the session after PJ-106 Part-A core shipped. Copy everything in the box.

---

Read `docs/Constellation Orientation & Onboarding v3.51.md` first (highest version — PJ-106 Part-A core is in its "What changed in v3.51" preamble). Then read the handover `lab/reports/HANDOVER-2026-07-15-pj106-partA.md`. Then `git pull origin main` and skim `git log --oneline -12`.

State: last session shipped and Boss-validated **PJ-106 Part-A CORE** — the Arabic/RTL typing engine. Four increments, each committed only after Eisa's live PASS: §A1 per-line direction → caret motion (killed the `dir='auto'` competitor), §B0 triple-click selects the TEXT not the trailing newline, §A3+§A2 Enter on an RTL line puts the caret on the RIGHT, §A5 logical (Word-style) arrows across Arabic↔Latin seams. Symptoms ①②③ confirmed resolved. New shared editor files: `src/lib/editor/{rtlFlag,tripleClickLine,rtlMotion}.ts`; `bidiPlugin.ts` reworked. `main` clean.

**Standing rules — do NOT regress:**
1. **The Boss Test is MANDATORY on every build** — commit is the LAST step, gated on Eisa's live-test pass on the running app. No "backend-only"/"proven-by-tests" exceptions.
2. Tests are **staged, one stage at a time, tutorial-style** (define the feature, then walk click-by-click). The **Art Director & Team own UX/UI**. **Reproduce-First** is a top principal — for editor-lifecycle / content-integrity bugs the running app is the verification, not svelte-check/vitest.
3. **`safety-inspection` workflow is at its WEEKLY agent limit until Jul 18, 4am** — work solo / hand-review until then.

**► Next action = PJ-106 Part B** (Eisa is actively working RTL). Plan: `docs/PJ-106-RTL-Typing-PLAN.md` §B; symptoms `lab/reports/PJ-106-RTL-Symptoms-BossReported.md` (Rounds 1–6). Remaining:
- **§A4** isolate ranges — but symptoms ②/③ are Boss-confirmed PASS, so §A4 may reduce to the deferred callout-caret repro ONLY. Reproduce-First-gate it: do NOT build it unless a live repro of the callout-caret glitch fires first.
- **Part B** — select **sentence** (Ctrl+click, `Intl.Segmenter` Arabic terminators `؟ ۔ !`, not `؛`), select **paragraph/line/page**, and the **Right-Ctrl+Shift → paragraph 100% RTL / Left-Ctrl+Shift → 100% LTR** override persisted as an invisible **RLM/LRM** mark (custom keydown reading `KeyboardEvent.code` for Left-vs-Right Ctrl; WebView2-eats-key spot-check + fallback). **B4 touches the SAVE path** → diff-scoped safety-inspection (once the limit resets) + the 8-point Editor-Surface Gate.

Before starting: cross-check PJ-106 Part B against the orientation §4.x BODY + this session's log (SO#8). Build with the mandatory gates (svelte-check 0, `npm run build` BEFORE `cargo build --release`, verify binary mtime), staged tutorial-style Boss live-test, **commit ONLY after Eisa passes**. At close (SO#9): reconcile the ledger FIRST → bump Pending Jobs v1.31, then Orientation v-bump + session log + MoCh + handover + this prompt.

Two items also open (don't lose): **PJ-107** (imported-note Home-caret invisible — PARKED by Boss; polish; needs an instrumented build to name the pixel mechanism — the trigger is the rich 16-field frontmatter, body proven innocent) and **PJ-108** (NEW APP-KILLER — a second-screen `[[wikilink]]` click destroys an unsaved save-failed note's crash-recovery net because the read-only SS consumes it via `openNoteTab → resolveNoteContent`/`clearWriteAhead` and never re-stashes; fix = read-only hosts open with `preserveNet`). Group-1 queue after PJ-106: PJ-108 · PJ-103 · PJ-104 · PJ-105.
