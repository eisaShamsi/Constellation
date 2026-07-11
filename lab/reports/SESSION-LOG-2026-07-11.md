# Session Log — 2026-07-11 (post-cockpit session — MIG-100 opens)

> Session start state: HEAD `31f64db2`, working tree clean, binary 2026-07-11 14:03 (cockpit lenses complete + Boss-validated — see `HANDOVER-2026-07-11-cockpit-lenses-complete.md`).

## MIG-100 — Auto-Restore Tabs on Relaunch — Phase 1 (Architect) COMPLETE

- **Function in hand:** Auto-restore-tabs-on-relaunch (Boss-wanted 2026-07-09; a Settings toggle, default ON — open tabs are not persisted across restart today, only manual named workspaces).
- **Boss picked this thread** at session start (over the safety-sweep backlog and Orrery polish).
- **Architect workflow `wf_54e79170-db3`** — 15 agents, 0 errors, ~1.48 M tokens: 4 census mappers (boot/close path, workspace machinery, settings plumbing, second-screen/gate surface) · 1 WA#5 prior-art researcher (VS Code / Obsidian / Sublime / Firefox / Chrome, sources cited) · 3 competing designers (D1 max-reuse, D2 clean session file, D3 minimal) · 6 adversarial refuters (2 per design) · 1 completeness critic.
- **Key census facts:** nothing restores tabs at boot today; `restoreWorkspace` has latent defects (unflushed `openTabs.set([])`, newTab-undefined collapse) and must not be reused; +layout has NO beforeunload; Rust `CloseRequested` handler exists but persists nothing; `setActiveUniverse` flips the Rust active pointer BEFORE `handleUniverseSwitch` runs and has 6 call sites incl. the boot loop (the cross-universe landmine).
- **Prior-art verdict:** continuous debounced persist (1–15 s) + best-effort close write; close-only = canonical anti-pattern; atomic write + previous-generation fallback; missing files skip non-fatally; default-ON in every editor-class app; crash-loop breaker (Firefox).
- **Adversarial verdicts:** D1 REJECTED (failure ceiling = named workspaces wiped via the shared whole-array atomic write — 3 independent kill paths); D3 rejected as stated (its `setActiveUniverse` flush hook fires at 5 non-switch sites incl. boot → boot-breaking; accepted silent persist failure = forbidden class); D2 RECOMMENDED — every fatal finding absorbed by a cheap correction (explicit-root IPC signature, ~20-LOC crash-loop breaker).
- **Critic's structural catch (gap all 3 designs shared):** the close path should use the EXISTING Rust `CloseRequested` handler (lib.rs:633-648) for a guaranteed final flush on graceful close — not a new DOM beforeunload; `beforeunload`+IPC survival in Tauri v2/WebView2 is UNKNOWN until tested.
- **Deliverable:** `docs/MIG-100-Auto-Restore-Tabs-Architect.md` (options table, 11 mandatory corrections, invariants, rollback).
- **Next:** Boss option pick → Phase 2 (Plan).
