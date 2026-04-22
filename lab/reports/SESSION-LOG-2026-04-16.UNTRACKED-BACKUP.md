# Session Log — 2026-04-16

## Phase: Daily Automated Code Audit

**Triggered by:** Scheduled task `constellation-code-audit`  
**Scope:** `src/` (Svelte + TypeScript) and `src-tauri/src/` (Rust)

---

## Audit Results

### Critical Issues: None

### High Issues: None

### Medium Issues

| File | Line | Rule | Finding |
|------|------|------|---------|
| `src/lib/components/CodeMirrorEditor.svelte` | 1506 | Rule 4 (No Memory Leaks) | `onDestroy` destroys the EditorView but does not clear `toolbarTimeout`. If the component is destroyed while a 400ms toolbar debounce is pending, the callback fires post-destroy. Safe in practice (callback guards `if (!containerEl || !view) return`) but technically violates Rule 4. |

### Low Issues

| File | Line | Rule | Finding |
|------|------|------|---------|
| `src/lib/components/CodeMirrorEditor.svelte` | 73 | Rule 7 (Dead Code) | `onchangeTimer` is declared but never assigned or read anywhere in the file — dead variable. |
| `src/routes/+layout.svelte` | 1894 | Rule 7 (Debug Logs) | `console.log('[boot-perf]', report)` — legitimate perf diagnostic but adds noise in production. |
| `src/routes/+layout.svelte` | 1353 | Rule 7 (TODO) | `// TODO: trail picker` — known deferred item, not urgent. |
| `src/lib/components/GraphMindView.svelte` | 464, 546 | Rule 4 | Fire-and-forget `setTimeout(() => engine?.renderSearchBadges(), 50/100)` without stored handles — cannot be cancelled on destroy. Minor: callbacks guard `engine?.` and these are short-lived. |

---

## Areas Checked (Clean)

- **Rule 1 — ViewPlugin update() guards:** All 4 plugins checked (`livePreview.ts`, `lineDecoPlugin.ts`, `calloutPlugin.ts`, `bidiPlugin.ts`) implement correct docChanged / selectionSet / viewportChanged guards with line-change guards on cursor moves and debounced fast-path rebuilds. ✅
- **Rule 2 — $effect loops:** All `$effect` blocks reviewed in `+layout.svelte` and `GraphMindView.svelte` (most recently modified). No read+write to same `$state`, no echo loops. ✅
- **Rule 3 — Heavy work on main thread:** No synchronous large-data processing in Svelte components found. ✅
- **Rule 4 — Memory leaks:**
  - `DashboardView.svelte` `setInterval` → cleaned up via `onMount` return callback ✅
  - `SecondScreenPage.svelte` `setInterval` → pushed to `unlisteners` array ✅
  - All `requestAnimationFrame` handles (`animFrame`, `centerRaf`, `layoutRaf`, `focusRaf`, `rafHandle`) — properly cancelled in `onDestroy` ✅
  - `FocusPane.svelte` `saveTimer` and `pauseTimer` → both cleared in `onDestroy` ✅
- **Rule 5 — Minimal DOM:** No JS-based positioning or unnecessary wrappers found. ✅
- **Rule 6 — No unnecessary imports:** `FocusPane.svelte` imports only `@codemirror/view`, `@codemirror/state`, `@codemirror/commands`, and minimal app stores — no markdown parser or language-data. ✅
- **Security — innerHTML:** All `innerHTML` assignments in `utils.ts` and `livePreview.ts` use `DOMPurify.sanitize()` on user-controlled strings. Numeric values (`rows.length`, `query_time_ms`) are injected directly but are typed numbers from Rust backend. ✅
- **Security — Secrets:** No API keys, tokens, or passwords found in source. GitHub token is stored in user settings (not source) and rendered as `type="password"` input. ✅
- **New file `src/lib/libraries/skyGraphStore.ts`:** Clean implementation using `writable()` store. Correct reasoning documented in header. ✅

---

## Action Taken

No auto-fixes applied (policy: only fix Critical/High issues automatically).

Medium and Low findings are recorded above for manual review in a future session.

---

## Open Items (carry forward)

- Medium: Clear `toolbarTimeout` in `CodeMirrorEditor.svelte` `onDestroy`
- Low: Remove dead `onchangeTimer` variable from `CodeMirrorEditor.svelte`
