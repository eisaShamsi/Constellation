# Sky View Production Bug — Virtual Lab

## Mission (as directed by user 2026-04-16)

> "Create a lab, then examine why the Dev mode is working perfectly, to
> understand its mechanism. From there (on your own virtual lab), conduct as
> many trials/attempts to fix it. Then you come back with a proven solution."

## Ground truth (falsified prior belief)

- HEAD `ef45c17` in **production** (the release `.exe`) boots with Sky View
  showing `0 nodes · 0 edges`. Screenshot captured 2026-04-16 06:47 local.
- The prior session's Agent 3 audit claimed the Sky View 0-nodes bug was
  caused by the two-phase IPC split I introduced. **That audit was wrong.**
  The bug pre-exists at HEAD with zero modifications.
- **Dev mode (`npm run tauri dev`) works perfectly** — user has confirmed this
  many times. This means: the code itself is capable of populating Sky View.
  Something about the **production build** breaks the population path.
- Therefore: the bug is in the **dev↔prod divergence**, not in the business
  logic of Sky View. Candidates include Tauri IPC, Vite minification, Rust
  release optimizations, worker URL bundling, SQLite behavior in bundled exe,
  startup timing/race.

## Rules for this lab

1. **No code changes to the main tree** until a proven solution exists.
2. **No builds for the user to run** until that proven solution has citable
   evidence from agents / docs / traced source.
3. Each hypothesis gets a numbered markdown file in this directory with:
   - The mechanism proposed.
   - Falsifiable predictions.
   - Evidence found (file:line, doc URL, or agent report).
   - Verdict: confirmed / refuted / inconclusive.
4. Attempts budget for the final proposed fix: 1 shot under the new process
   (we have 2 remaining after the failed `$state.raw` attempt).

## Hypothesis axes to investigate

- **H1 — IPC payload truncation/encoding:** Tauri v2 prod vs dev IPC transport
  differences; 232k-link JSON payload behavior.
- **H2 — Vite/esbuild release minification:** variable mangling, dead-code
  elimination, hoisting differences altering execution order.
- **H3 — Rust release-mode optimization:** rusqlite iterator/serde divergence
  under `-O` vs debug; panic semantics; WAL under bundled layout.
- **H4 — SQLite file layout in bundled vs dev:** different `app_data_dir`
  resolution; cache path resolves to different DB in prod.
- **H5 — Startup timing/race:** prod is faster at paint, so
  `refreshLibraryCaches` may execute before a dependency is ready.
- **H6 — Silent error:** an exception is thrown in prod but swallowed;
  `snapshot.links` arrives but assignment fails.
- **H7 — Web Worker URL resolution:** `GraphEngine` worker URL resolves
  differently under Tauri custom protocol.
- **H8 — Svelte 5 prod reactivity:** runes behavior differs under
  `NODE_ENV=production` (dev-only guards stripped).

See `01-dev-vs-prod-source-trace.md` onwards.
