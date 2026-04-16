# Constellation — Lessons Learned (LL)

> A living document. Updated after every major development cycle. These are hard-won rules discovered through iterative testing, debugging, and user feedback. They override assumptions and must be consulted before making architectural decisions.

---

## LL-001: Tauri IPC is the #1 Performance Killer
**Discovered:** Phase 2 (13 rounds of testing)

ANY `invoke()` call during or near typing causes perceptible lag. Even "lightweight" async calls accumulate over repeated pauses. The JS-side serialization of `invoke` arguments is synchronous and blocks the main thread.

**Rule:** Zero IPC during typing. Save to memory only. Write to disk on idle (30s), blur, or close.

---

## LL-002: The Layout Reactivity Cascade
**Discovered:** BLOCKING-001, BLOCKING-002, BLOCKING-003

`+layout.svelte` (3873 lines) has 77 `$state` vars, 17 `$effect` blocks, 19 `$derived` blocks, 150+ template reactive reads. Any store mutation (`openTabs.update()`, `activeTabId.set()`, etc.) triggers a full cascade across the entire file — freezing the UI for seconds.

**Rule:** Never call store mutations from inside a component's `onDestroy` or any hot path. Use direct object mutation (`tab.content = ...`) to bypass Svelte's reactivity. Batch related store updates — don't trigger 3 separate cascades when 1 will do.

---

## LL-003: Build Passing ≠ Working App
**Discovered:** Phase 0 testing

`npm run build` and `npm run check` passing tells you nothing about user experience. Phases 0 and 1 compiled perfectly but froze for 3+ seconds on title click due to the layout reactivity storm (LL-002).

**Rule:** Every phase must pass user testing in the actual running Tauri app. Code review and build verification are necessary but not sufficient.

---

## LL-004: CM6 Widget Event Handling Order
**Discovered:** Phase 6 (checkbox toggle), Phase 7 (callout chevron)

`EditorView.domEventHandlers` runs AFTER CM6 processes events internally. For replacement widgets (checkboxes, callout chevrons), CM6 moves the cursor on mousedown, which triggers a decoration rebuild that destroys the widget — BEFORE the click/mousedown handler fires. The handler's `target` no longer exists.

**Rule:** For click handlers on CM6 replacement widgets, use capture-phase `addEventListener` on the editor DOM element. Don't use CM6's `domEventHandlers` or `eventHandlers` for widget clicks.

---

## LL-005: `tauri dev` Rewrites Cargo.toml
**Discovered:** Image preview fix (asset protocol)

The Tauri CLI (`tauri dev`) runs `cargo run --no-default-features --features <list>` and dynamically manages the `[features]` section of `Cargo.toml`. Manual edits to dependency features (`tauri = { features = ["protocol-asset"] }`) are silently overwritten.

**Rule:** To add Cargo features that survive `tauri dev`:
1. Add a `[features]` section to the app's `Cargo.toml` that forwards to the dependency: `protocol-asset = ["tauri/protocol-asset"]`
2. Add the feature name to `build.features` in `tauri.conf.json`
3. Never rely on editing the dependency features line directly.

---

## LL-006: Phase-by-Phase with User GO/NO-GO
**Discovered:** Entire eNotePane development (9 phases)

Phase 2 alone required 13 rounds of testing. If all 8 phases had been built first and then tested, debugging would have been impossible — the bugs compound and mask each other.

**Rule:** Small increments, test each one, never proceed until the user approves. The spec workflow (propose → implement → audit → test → approve) catches issues at the smallest possible scope.

---

## LL-007: Shared Plugins Pay Off
**Discovered:** Phases 6-8

Phases 6-8 were fast because `livePreview.ts`, `calloutPlugin.ts`, `lineDecoPlugin.ts`, `completions.ts`, and `tableUtils.ts` already existed as shared modules. Phase 6 (18 tests, 10 decoration types) was mostly wiring imports, not writing logic.

**Rule:** Build editor extensions in `src/lib/editor/` as shared modules, not inline in components. The Editor Parity Rule (CLAUDE.md) ensures all note views share the same rendering. Duplication between components is a bug.

---

## LL-008: The Session Log is a Lifeline
**Discovered:** Multiple session clears during eNotePane development

Without `lab/reports/SESSION-LOG-{date}.md`, context is completely lost on session clear. The next session has no idea which phases passed, which bugs were fixed, or what approach worked after 13 failed attempts.

**Rule:** Standing Order (SO) — after every phase, commit, or significant step, update the session log with: phase name, commit hash, test results, bugs fixed, open items. This is the safety net.

---

## LL-009: Derive State, Don't Duplicate It
**Discovered:** Simplify review (Phase 7b)

`tableToolbarVisible` was a `$state(false)` that was manually set to `true`/`false` in sync with `currentTable`. It was pure redundancy — `$derived(currentTable !== null)` replaces it with zero risk of desync.

**Rule:** If a value can be derived from existing state, use `$derived`. Don't create `$state` that mirrors other state — it will eventually desync and cause bugs.

---

## LL-010: Merge Iteration Loops
**Discovered:** Progressive lag investigation

`livePreview.ts` had 3 separate line-by-line loops over visible ranges (wikilinks, image embeds, tags). Each loop called `doc.lineAt()`, allocated regex objects, and iterated the same lines. Merging into a single pass significantly reduced progressive lag.

**Rule:** When iterating visible lines for decorations, do all checks in a single pass. Multiple passes over the same data multiply GC pressure and cache misses.

---

## LL-011: Tauri v2 Asset Protocol Configuration
**Discovered:** Image preview fix (multiple failed attempts)

In Tauri v2, `convertFileSrc()` generates `http://asset.localhost/path` URLs. Getting these to work requires ALL of:
1. `protocol-asset` Cargo feature (via LL-005 method)
2. `assetProtocol: { enable: true, scope: { allow: ["**/*"] } }` in `tauri.conf.json` security
3. `http://asset.localhost` in CSP `img-src` AND `connect-src`
4. `https:` in CSP `img-src` for external images

Missing any one of these results in silent failures (fallback icon, ERR_CONNECTION_REFUSED).

**Rule:** When configuring Tauri protocols, test with DevTools console open. Check for CSP violations, connection errors, and scope denials separately.

---

## LL-012: `posAtDOM` Unreliable for Replacement Widgets
**Discovered:** Phase 6 (checkbox toggle)

`view.posAtDOM(widgetElement)` returns unreliable positions for CM6 replacement widgets (`Decoration.replace`). The widget's DOM element doesn't correspond to document text, so the position mapping is imprecise.

**Rule:** For mapping widget clicks to document positions, use `view.posAtCoords({ x: event.clientX, y: event.clientY })` or `view.coordsAtPos()` for reverse mapping. Don't use `posAtDOM` on widget elements.

---

## LL-013: `getCursorColumn` Pipe-Counting Bug
**Discovered:** Phase 7b (Shift+Tab in tables)

The original `getCursorColumn` in `tableUtils.ts` used a stateful `inCell` flag to count pipe separators. For tables with leading `|`, the first pipe set `inCell=true` but the adjustment `col - 1` gave wrong results for columns > 0.

**Rule:** For pipe-counting in markdown tables, simply count pipes before the cursor offset. If the line starts with `|`, subtract 1 (leading delimiter). Avoid stateful tracking — it's error-prone.

---

## LL-014: Three Strikes — Fix From the Root
**Discovered:** 2026-03-31 (callout plugin freeze — 7+ patch iterations)

Applying a patch when a fix fails is reasonable once, maybe twice. By the third failure, patching is the wrong strategy — the root cause has not been found. Every additional patch builds on a broken foundation and compounds the debt.

In the callout freeze case: seven rounds of patches addressed symptoms (cursor guard, batch collapse, replace-range trimming) while the true cause — `Decoration.replace` creating a cursor-exclusion range that triggers an infinite selectionSet→rebuild loop — went undiagnosed. One correct diagnosis → one correct fix (`Decoration.line` zero-length + CSS) → problem eliminated permanently.

**Rule:** If a bug is not resolved after three distinct attempts, **stop patching**. Step back, identify the actual root cause, and redesign from that level. A correct fix at the root is always shorter, cleaner, and more permanent than accumulated workarounds.

---

## LL-015: Always Test Production Before Chasing Dev-Mode Performance
**Discovered:** 2026-04-15 (boot-perf saga on 7,600-note Universe)

Tauri v2 on Windows in dev mode (`npm run tauri dev` via Vite + WebView2 + DevTools attachment) introduces massive per-IPC latency — measured at ~37 seconds per command dispatch on the test hardware. The *same code* compiled as a production `.exe` (via `npm run tauri build`) has no such delay — the production app boots in 1s UI / 8s fully responsive where dev mode was 25s / 136s.

Hours of debugging went into chasing a phantom that only exists in dev mode. Dev-mode timings are NOT representative of user experience and must never be used as the signal for "is this fast enough to ship."

**Rule:** When performance is the question, always measure against a production build. Dev-mode numbers are only useful for *relative* comparisons within the same run, never as an absolute ship-gate.

---

## LL-016: Cache at the Call Site When Callers Are Unknown
**Discovered:** 2026-04-15 (load_all_libraries 50-calls-per-boot)

Diagnostics showed `load_all_libraries` was being invoked 50+ times per boot from many code paths (both known and mystery callers in reactive cascades, file watchers, map commands, and validation paths). Each call re-read `libraries.json` from disk and re-parsed JSON. Under Tauri's IPC queue this produced a 60-second boot hang.

Auditing every call site to remove redundant calls would have taken hours and missed future callers. The better fix: add an in-memory cache at the callee side, keyed by a stable invariant (active universe path), invalidated by the two functions that mutate the underlying state (`save_libraries` and `set_active_universe`). All 50+ callers — present and future, known and unknown — now get instant reads automatically.

**Rule:** If a function is called from many places across the codebase and the data rarely changes, cache the result inside the function rather than auditing every call site. Invalidation points should be the few places that mutate the underlying data, not the many places that read it.

---

## LL-017: When Patching Fails, Spawn Adversarial Expert Agents
**Discovered:** 2026-04-15 (boot-perf saga)

After 4 patch attempts on the boot-perf issue failed, continuing to guess was clearly wrong (per LL-014). The breakthrough came from spawning three AI agent personas in parallel — an Obsidian internals expert, a Tauri/Rust systems expert, and a PKM architecture generalist — each instructed to review the proposed fix adversarially and produce a structured memo with verdict, risks, and acceptance criteria.

Their convergent findings identified architectural errors I had missed (especially: "Constellation awaits backend work before the window is shown; Obsidian doesn't") and produced 5 concrete ship-gate criteria that became `lab/boot-perf/BOOT-BUDGET.md`. The objective criteria transformed further work from blind patching into measurable progress.

**Rule:** When LL-014 triggers ("stop patching"), don't just stop — actively escalate to adversarial expert review. Spawn 2–3 independent AI reviewers with distinct perspectives, produce concrete numerical acceptance criteria, and only then resume implementation. The first move out of a patching loop should be a lab harness and a referee panel, not another patch.

---

## LL-018: Paint-First UI — Never Gate First Paint on IPC
**Discovered:** 2026-04-15 (boot-perf expert panel)

Constellation's boot originally awaited 7+ serialized IPC calls before setting `appReady = true`, meaning the loading spinner could not be replaced until the slowest Rust handler returned. On large Universes with Tauri's command-queue contention, this gated first paint on tens of seconds of backend work.

Obsidian does not do this — it shows its shell immediately and hydrates data asynchronously. Every competitor that paints faster than Constellation does this. The fix is structural: in `initializeApp`, call `appReady = true` synchronously at the top; let every data load run afterward as a fire-and-forget that populates reactive stores progressively.

**Rule:** First paint MUST NOT await backend work. The UI shell shows immediately; data arrives asynchronously and fills in. This applies at every level — layout mount, tab open, settings panel, any dialog. If a component's first render is gated on an `await invoke(...)`, refactor it.

---

## LL-019: PIXI v8 + Tauri CSP — Import `pixi.js/unsafe-eval` as a Side-Effect
**Discovered:** 2026-04-16 (Sky View rendering bug)

Sky View was returning data, pushing it into `skyNodes`/`skyLinks`, reaching the PIXI `Application.init()` call — and then rendering an empty black canvas with "0 nodes · 0 edges" in the status bar. No visible error in the DevTools console.

Root cause: PIXI v8 generates WebGL shader programs at runtime using `new Function(...)`. Tauri's default CSP does not allow `unsafe-eval`, so every shader compile throws `"Current environment does not allow unsafe-eval, please use pixi.js/unsafe-eval module to enable support."` PIXI catches the throw internally and leaves the renderer half-constructed — no crash, no red border, just a silent empty canvas.

The throw never surfaced in the normal error paths; only after a 13-component disassembly with progressive probes (A–I inside `init()`) did the caught error get re-logged and identified. Once seen, the fix is one line:

```ts
// MUST be the first PIXI-related import — pure side-effect
import 'pixi.js/unsafe-eval';
import { Application, Container, Graphics, Text } from 'pixi.js';
```

`pixi.js/unsafe-eval` ships a pre-compiled shader generator that does not use `new Function()`. It must be imported **before** any PIXI class is constructed. Relaxing the app-wide CSP was rejected — weakening `unsafe-eval` for the whole app to accommodate one library's default build is a security regression across the entire frontend.

**Rule:** When a WebGL / Canvas / GPU library fails silently on a Tauri target, suspect CSP — the default is strict and any library using `new Function()` or `eval()` for runtime codegen will fail without surfacing an error. Prefer the library's pre-compiled / no-eval variant via a side-effect import before any usage. Never relax the app-wide CSP to work around a single library's default build.

---

*Last updated: 2026-04-16*
*For: Constellation — boot-perf + Sky View fix cycle*
