# Constellation Audit System

**Version 1.0 | April 2026**

## Why

Constellation is a complex multi-language desktop app (Rust + SvelteKit + ONNX) serving users who depend on it for their knowledge work. Every code change can introduce regressions in performance, memory, RTL support, search correctness, security, or cross-function integration. Manual review misses edge cases. The Audit System provides systematic, repeatable quality gates.

## When

Run the full 14-agent audit:
- **After every major phase** (new feature, significant refactor, or multi-file change)
- **Before every release** (version bump, production build)
- **After importing external dependencies** (new crates, npm packages, model files)
- **On demand** when the user requests `/simplify` or a targeted audit

Run individual agents:
- **PA** after any editor extension, CM6 plugin, or rendering change
- **RA** after any UI change touching text display or layout
- **SIA** after any search.rs or store.ts search-related change
- **OGA** after any dependency or network-related change

## How

Each agent is launched as a parallel Explore subagent. All agents run simultaneously for speed. Each agent returns a findings report with severity levels:

- **CRITICAL** — must fix before commit (crashes, data loss, security holes)
- **HIGH** — must fix before release (memory leaks, broken features, wrong results)
- **MEDIUM** — should fix soon (dead code, missing i18n, UX friction)
- **LOW** — nice to fix (naming, minor style, optimization opportunities)

After all agents complete, findings are consolidated into an action list. Critical and High issues are fixed immediately. Medium issues are tracked. Low issues are noted.

---

## The 14 Audit Agents

### Core Quality (1-8)

| # | Agent | Code | Scope |
|---|-------|------|-------|
| 1 | **Performance** | PA | $effect loops, hot-path invoke() calls, debounce compliance (>=300ms), unnecessary re-renders, large decoration rebuilds, virtualization of lists >50 items |
| 2 | **Architecture** | AA | Dead code, unused imports, duplicated logic, component boundaries, separation of concerns, shared vs duplicated functionality |
| 3 | **Memory** | MA | setTimeout/setInterval cleanup in onDestroy, event listener removal, EditorView.destroy(), Tauri listen() unlisten, Mutex lock release, localStorage bounds |
| 4 | **Spec Compliance** | SCA | CLAUDE.md rules adherence: Svelte 5 runes only, no legacy patterns, i18n $t() for all user strings, CSS conventions, editor parity rule, IPC boundary rules |
| 5 | **RTL/Bidi** | RA | dir="auto" on user text, dir={$dir} on containers, chevron/arrow flipping, text-align: start (not left), padding-inline (not padding-left), RTL layout testing |
| 6 | **UX** | UXA | Interaction flow completeness, keyboard accessibility, error states, loading states, empty states, clear feedback for user actions, discoverable features |
| 7 | **Code Quality** | CQA | Type safety, error handling, naming consistency, function length, unused variables, console.log cleanup, TODO comments, code style |
| 8 | **Environment** | EA | Cargo.toml dependency versions, package.json consistency, tauri.conf.json correctness, build scripts, Git LFS tracking, .gitignore completeness |

### Domain-Specific (9-14)

| # | Agent | Code | Scope |
|---|-------|------|-------|
| 9 | **Localization** | LA | All 15 locale files: missing keys, extra keys, broken JSON, placeholder consistency, RTL string handling, Arabic translation quality |
| 10 | **Search Integrity** | SIA | All 6 link operators return correct results, FTS5 queries don't silently fail, tag/property/wikilink LIKE queries match stored data, Arabic normalization consistency between FTS (normalized) and LIKE (raw), multi-term comma splitting, universal search category correctness |
| 11 | **Security** | SA | {@html} XSS vectors from user note content, SQL injection in search.rs (parameterized queries check), unsafe Rust blocks, ONNX model file integrity, CSP compliance |
| 12 | **Data Integrity** | DIA | SQLite schema consistency (note_meta ↔ note_embeddings), schema version migration, BLOB embedding roundtrip (f32 → bytes → f32), orphan detection query correctness, reindex idempotency |
| 13 | **Cross-Function Sync** | CFS | SearchHub matchIds → Sky View/Sight propagation, note save → reindex + re-embed chain, settings toggle → feature activation, second screen event sync, sidebar state preservation on overlay open/close |
| 14 | **Offline Guarantee** | OGA | Zero network calls at runtime, ONNX model loads from local disk only, no CDN/fetch in any code path, @xenova/transformers not called at runtime, CSP doesn't block local WASM, no telemetry |

---

## Agent Prompt Templates

Each agent receives a prompt structured as:
```
{AGENT_NAME} AUDIT for Constellation at {PROJECT_PATH}.
Focus on: {CHANGED_FILES_OR_SCOPE}
Check: {SPECIFIC_ITEMS_FROM_SCOPE}
Report ONLY actual issues with severity (CRITICAL/HIGH/MEDIUM/LOW).
```

## Output Format

Each agent returns:
```
## {AGENT_CODE} — {AGENT_NAME} Audit Results

### CRITICAL
- [file:line] Description of issue

### HIGH
- [file:line] Description of issue

### MEDIUM
- [file:line] Description of issue

### LOW
- [file:line] Description of issue

### PASS
- Items checked that passed
```

## Consolidated Report

After all 14 agents complete, a single consolidated report is produced:
- Total issues by severity
- Issues grouped by file (which file has the most problems)
- Action items with assignee (fix now vs fix later)
- Sign-off: PASS (0 critical, 0 high) or FAIL (must fix before commit)
