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

*Last updated: 2026-03-28*
*For: Constellation — eNotePane development cycle*
