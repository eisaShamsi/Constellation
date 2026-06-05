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

## LL-020: Wall-Clock vs Server-Time Decides Whether the Bottleneck Is in Your Code
**Discovered:** 2026-04-16 (boot-perf cold-start round 3)

`cache_boot_snapshot_core` frontend wall-clock was 27,710 ms. Server-side phase timings (ensure_db, open_reader, read_notes) summed to 8,094 ms. The delta — **19,616 ms of pure queue/contention wait** — proved the bottleneck was not inside the Rust function at all. It was the OS I/O scheduler round-robining 34 concurrent boot IPCs on Windows NTFS and starving the one we needed first.

Without this split attribution I would have optimized Rust code that was already fast enough. The fix wasn't inside the handler; it was **ordering the call sites** (await `refreshLibraryCaches()` before fan-out) so the critical IPC got exclusive I/O until hydration fired.

The same measurement also separated the two real costs: 8 s was genuine Rust work (row-store full scan — fixed with `CREATE INDEX IF NOT EXISTS idx_note_boot_snapshot ON note_meta(name, path, library_name)` — dropped to 5 ms, 1,600×), and the other 19 s was pure queue. Two root causes, one measurement to tell them apart.

**Rule:** For any frontend `invoke(...)` on the critical path, always instrument both:
1. Frontend `performance.now()` wall-clock around the invoke.
2. Rust-side per-phase `Instant::now()` checkpoints returned in the response struct.

If `wall_ms >> sum(server_timings_ms)`, the delta is IPC queue / OS contention — optimize by reordering, de-paralleling fire-and-forget calls, or moving work off the critical path. If `wall_ms ≈ sum(server_timings_ms)`, the bottleneck is genuinely inside the handler — optimize the Rust code or the SQL plan. Never optimize blind; the fix direction flips based on which pattern you have.

**Corollary — covering indexes on row-stores.** SQLite is a row store. A `SELECT a, b, c FROM wide_table` still reads every page to find a/b/c — including columns you never asked for (body_text, JSON blobs). For narrow projections on wide tables, a covering index (`CREATE INDEX ... ON table(a, b, c)`) lets the planner do an index-only scan against a small, dense index instead of a full-page scan against the wide row-store. Cost is trivial disk; speedup is typically 100–1000×.

---

## LL-021: The Five-Stamp IPC Diagnostic — And Why a Measured Queue Is Not a Diagnosed Queue
**Discovered:** 2026-04-19 (boot-perf Criterion 2 closure, Round 1 → Round 3)
**Status:** Revised after Round 3 falsified the original "fix". The rule below is what survives the measurement.

LL-020 identified the wall-vs-server gap and attributed it to OS I/O contention, fixed by reordering call sites. But after ordering was fixed, cold-boot still showed `hydrated_ms ≈ 17.8 s` on the same 7,600-note Universe. Round 1 instrumentation (`Instant::now()` inside the command body, transport-time via Unix timestamps) measured `wall = 23,103 ms`, `sum(server_timings) = 170 ms`, `transport = 19 ms`, `assign = 0 ms` — leaving **22.9 s completely unaccounted**. The gap was not in transport, not in serialization, not in the JS assign cascade, and not in the Rust body as measured.

The blind spot was simple: `Instant::now()` stamped at the command body's first line measures only *in-body elapsed*. It cannot see the time between JS issuing `invoke(...)` and the Rust dispatcher actually running the body — the **pre-body queue**.

### What survives: the five-stamp diagnostic

Always instrument all five. If only some are present, you are blind to one of the failure modes:

1. `invoke_start_unix_ms` (JS, **before** `await invoke(...)`).
2. `server_start_unix_ms` (Rust, **first line of body**, before any work).
3. Per-phase `Instant::now()` checkpoints inside the Rust body.
4. `server_return_unix_ms` (Rust, last line before returning).
5. `client_recv_unix_ms` (JS, **immediately after** `await invoke(...)` resolves).

Derivations:

- `queue_ms = server_start_unix_ms - invoke_start_unix_ms` → **pre-body dispatcher wait**.
- `body_ms = server_return_unix_ms - server_start_unix_ms` → **pure Rust execution** (same as LL-020's `sum(server_timings)`).
- `transport_ms = client_recv_unix_ms - server_return_unix_ms` → **serialization + IPC pipe + JS task-queue wait**.
- `wall_ms = client_recv_unix_ms - invoke_start_unix_ms` → **total elapsed from JS perspective**.

Check: `queue_ms + body_ms + transport_ms ≈ wall_ms` should hold within ±clock-skew noise (<10 ms on the same machine). If it doesn't, the clock drifted or one of the stamps is wrong.

Round 2 applied this model and produced decisive evidence on the trial Universe:

| field | value | interpretation |
|:---|---:|:---|
| `core_wall_ms` | 17,314 ms | JS-side wall |
| `core_queue_ms` | **17,224 ms** | pre-body dispatcher wait |
| `core_body_ms` | **72 ms** | pure SQLite work |
| `graph_queue_ms` | **96 ms** | graph fires via rIC — runtime idle by then |
| `graph_body_ms` | 2,286 ms | graph SQLite + serialization |

**The diagnostic was correct**: 99.5 % of core-phase wall was pre-body queue, not transport, not body, not serialization. Keep this pattern.

### What did NOT survive: the spawn_blocking theory

Round 2's diagnosis jumped from *measured queue* to *hypothesized cause*: "`#[tauri::command] pub fn` dispatches onto Tokio's ~4-thread async-runtime worker pool; 30+ sync fan-out commands saturate it; the core snapshot waits for a worker." The proposed fix was to declare the snapshot commands `async fn` wrapping `tauri::async_runtime::spawn_blocking(move || impl(..)).await` — on the theory that this shifts the sync body off the async pool onto Tokio's 512-thread blocking pool.

That fix landed in commit `5f60448`. Round 3 measurement on the exact same trial Universe, same Rust release binary, same boot sequence:

| field | Round 2 (before fix) | Round 3 (after fix) | Δ |
|:---|---:|---:|---:|
| `hydrated_ms` | 17,800 ms | **20,610 ms** | **+2.8 s worse** |
| `core_queue_ms` | 17,224 ms | **19,880 ms** | **+2.7 s worse** |
| `core_body_ms` | 72 ms | 112 ms | +40 ms |
| `graph_queue_ms` | 96 ms | ~90 ms | ≈flat |

The fix made the queue **slightly worse**, not zero. The root-cause theory was falsified. Commit `5f60448` was reverted in `f5f0b6a`.

**Why the theory failed** (the investigation still owes a definitive answer — see follow-up below — but at least one of these must be true):

- **(a)** Tauri v2's `#[tauri::command]` macro already dispatches sync bodies onto `spawn_blocking` internally. In that case the wrapper I added was a no-op plus one extra async task hop — explaining the +2.7 s regression as pure overhead, not correction.
- **(b)** The queue is not async-worker-pool contention at all. Candidates: per-command receiver serialization, a mutex in the webview IPC drain, the WebView2 IPC message channel itself being single-consumer, or the fan-out occupying a resource the core snapshot also needs that is **not** the worker pool (e.g., the SQLite file's OS-level read contention before WAL warm-up).
- **(c)** Something I haven't thought of.

### The rule that survives

1. **Measurement diagnoses only the stage.** The five-stamp model tells you *where* time is spent (queue vs. body vs. transport). It does **not** tell you *why* time is spent there. Do not skip the second question.
2. **Before proposing a runtime-internals fix, read the runtime's actual source** (or run a targeted experiment that can only succeed if the hypothesis holds). A `spawn_blocking` wrapper should have been preceded by grep'ing `tauri-runtime` / `tauri-macros` for whether sync commands are already wrapped. They may be. I didn't check. That is the mistake LL-021 now exists to prevent.
3. **If a fix regresses the metric it was supposed to improve, revert immediately** — don't stack a "tuning" commit on top. The regression is evidence the model is wrong, not that the fix needs tuning.
4. **Keep the five-stamp diagnostic instrumentation permanently.** It is cheap (two `Date.now()` + two `SystemTime::now()`), works in release, and is the only way to distinguish queue from body from transport. Without it, a 20 s boot looks exactly like a 2 s boot followed by 18 s of mystery JS work — and you will fix the wrong thing.

### Follow-up resolved — Rounds 4 → 7 (added 2026-04-19)

The follow-up investigation closed Criterion 2 on 2026-04-19 at `hydrated_ms = 811 ms` (trial Universe, commit `8a74949`). It took four more rounds to get there. The methodology that worked — and that must be repeated rather than reinvented — is the **escalating-specificity diagnostic stack**:

#### Stage 1 — Queue-time stamps (LL-021's original instrumentation)

The five-stamp model correctly said "the 20 s is pre-body queue, not body, not transport." That narrowed the search to "what happens between JS `postMessage` and Rust body starting." This is table stakes for any IPC regression. Without it you are guessing.

#### Stage 2 — "Broadly plausible" patch rounds (Rounds 4 and 5)

Convert every sync `#[tauri::command]` in the obvious fan-out to `#[tauri::command(async)]` based on the theory "sync commands run inline on the UI thread, so they serialize." The theory is correct as a mechanism but useless as a diagnosis — *which* sync command is the blocker is still unread. Rounds 4 and 5 each converted a cluster (layout fan-out, then DashboardView fan-out) and each left `core_queue_ms` statistically unchanged.

**When this pattern fires twice without moving the metric, LL-014 triggers (three-strike rule). Stop patching.**

#### Stage 3 — Cheap falsifiers (adversarial investigation)

Dispatch an agent with the instruction "try to falsify the UI-thread-contention theory." It will produce two or three specific hypotheses. For each, find a falsifying test that costs one line of code:

- *Hypothesis*: DashboardView mount is the blocker → gate the whole subtree with `{#if false && ...}`. One edit. Measurement: `core_queue_ms` unchanged. Subtree off the critical path. Hypothesis falsified.
- *Hypothesis*: JS itself is blocked (a microtask/promise chain is stuck, not awaiting Rust) → add a `setInterval(…, 100)` from `boot:paint` onward, record `boot_heartbeat_max_gap_ms`, freeze at `boot:hydrated`. Measurement: max gap = 112 ms during an 18,614 ms queue window. JS is fully alive. Hypothesis falsified.

**Cheap falsifiers are the tool of choice the moment a third-hit LL-014 trigger happens.** They cost minutes, not hours, and they eliminate hypotheses definitively. The DashboardView gate was 14 characters. The heartbeat was one `setInterval` + max-gap tracking. Both produced irrefutable verdicts.

#### Stage 4 — Rust-side IPC arrival tracer (the instrument that actually names the culprit)

By elimination after Stage 3: the 18.6 s lives between JS `postMessage` and Rust `invoke_handler` dispatch. That's upstream of every `Instant::now()` we had. The instrument needed is one a per-command stamp **at the dispatcher**, independent of any edits to individual command bodies.

Pattern (Constellation implementation in `src-tauri/src/perf_trace.rs` + `src-tauri/src/lib.rs`):

```rust
// perf_trace.rs
static TRACE_LOG: Mutex<Vec<(String, u64)>> = Mutex::new(Vec::new());

pub fn record(cmd: &str) {
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
    if let Ok(mut log) = TRACE_LOG.lock() { log.push((cmd.to_string(), ts)); }
}

#[tauri::command]
pub fn get_perf_trace_log() -> Vec<(String, u64)> { TRACE_LOG.lock().map(|l| l.clone()).unwrap_or_default() }

// lib.rs — wrap generate_handler!
.invoke_handler({
    let inner: Box<dyn Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static> =
        Box::new(tauri::generate_handler![ /* all commands */ ]);
    move |invoke: tauri::ipc::Invoke<tauri::Wry>| -> bool {
        perf_trace::record(invoke.message.command());
        inner(invoke)
    }
})
```

Two Tauri v2 type-system subtleties worth recording for the next implementer:

1. `generate_handler!` must be bound via `Box<dyn Fn(Invoke<Wry>) -> bool + Send + Sync + 'static>` to pin the macro's `R: Runtime` generic at the binding site. Without the annotation, Rust cannot resolve R in a `let` binding and emits `E0282 cannot infer type`.
2. `invoke.message.command()` returns `&str`; call `perf_trace::record` *before* forwarding to `inner(invoke)` (which consumes `invoke` by value).

**On the first measurement after adding this, the answer fell out in one line of JSON.** Of the 20.6 s queue, the arrival log showed:

| t (ms, relative) | command | what |
|---:|:---|:---|
| +566 | `constellation_map_universe` | first call |
| +17,792 | `constellation_map_universe` (again) | second call, 17.2 s later |
| +21,294 | `cache_boot_snapshot_core` | finally dispatched |

The 17.2 s gap between the two map arrivals was the dispatcher blocked by the first call. The second call cost a further 3.5 s. Core was queued behind both of them for the full 20.7 s.

#### Stage 5 — Named-culprit conversion (Round 7, the actual fix)

One line: `#[tauri::command]` → `#[tauri::command(async)]` on `constellation_map_universe`. Rebuild. Measurement on the next boot: `core_queue_ms = 4`, `hydrated_ms = 811`. A 5,100× reduction in queue, closing Criterion 2 at 7.4× under budget.

### The rule (extended — supersedes "read the runtime's source" from the original)

1. **The five-stamp model is stage 1. Keep it permanently — it rules in/out queue vs. body vs. transport.**
2. **When two consecutive patches (matching the same hypothesis) don't move the metric, stop.** LL-014 triggers. Do not patch a third time on the same theory.
3. **Run cheap falsifiers before writing the next fix.** A single `{#if false}` gate or a `setInterval` heartbeat can eliminate a whole class of hypotheses in one build. Prefer falsification over confirmation — an experiment that *can't fail* teaches nothing.
4. **When the queue stage is named but the cause isn't, instrument the dispatcher, not individual commands.** A per-dispatch arrival log is ~30 lines of Rust and reveals the full timeline with zero per-command edits. In Tauri v2 specifically, wrap `generate_handler!` in a Box-typed closure and stamp `invoke.message.command()`.
5. **Keep every diagnostic instrument in the codebase permanently.** `perf_trace`, the heartbeat, and the five-stamp model are all cheap at runtime (one mutex, one interval, four timestamps per IPC). Together they transform a future boot regression from "where did the time go" into "here is the exact arrival log — go look at entry N."
6. **Fixes that don't move the metric are still sometimes correctness-improving** (Round 5's DashboardView fan-out converts are a real improvement in UI-thread hygiene, even though they didn't close Criterion 2). Keep them; don't revert purely for performance reasons if the commit improves code shape.

### What closed Criterion 2

- `perf_trace` arrival tracer → instrument that named `constellation_map_universe`.
- `#[tauri::command(async)]` on the named command → closed the queue.
- Total code added: ~30 lines Rust, ~15 lines Svelte, one attribute change.

### What the five-diagnostic stack costs to repeat on the next regression

- Queue-time stamps: already permanent.
- Cheap falsifiers: minutes each.
- Heartbeat: already permanent.
- Arrival tracer: already permanent.
- Named-culprit conversion: minutes.

Total: ~1 hour if the methodology is followed. ~4 sessions if it isn't.

---

## LL-022: Always-Mounted UI = Always-Running IPC

Even after Criterion 2 was "closed" by converting `constellation_map_universe` to
`#[tauri::command(async)]` (Round 7), the boot `ipc_arrival_log` still showed **two**
dispatches of that command 17 seconds apart, spending ~17 s of background CPU walking the
Universe filesystem — for a view the user might never open that session. The async
conversion got the work off the UI thread, so Criterion 2 passed at 811 ms — but the work
itself was still wasted.

**Root cause.** Both `<ConstellationMap>` and the fullscreen `<OrgChart>` overlays were
always-mounted in `+layout.svelte`, hidden via CSS (`class:map-visible={showConstellationMap}`)
instead of gated with Svelte `{#if}`. The comment on the ConstellationMap overlay explained
the motivation — "always rendered, hidden with CSS to preserve drill-down state" — and
that motivation was correct, but the cost was invisible until the five-stamp diagnostic
exposed it.

CSS `display: none` hides a component. It does **not** prevent its `onMount` / mount-time
`$effect` from firing. Every always-mounted overlay whose mount performs IPC pays the IPC
cost on every boot. Multiplied across Map, OrgChart, and whatever other panels fall into
the same pattern, the IPC queue stays saturated even after each individual command is
made async.

**Fix.** Gate each overlay with `{#if mapEverOpened}` / `{#if orgChartEverOpened}`, where
`*EverOpened` is a `$state(false)` flag flipped `true` by a one-line reactive effect:

```typescript
let mapEverOpened = $state(false);
$effect(() => { if (showConstellationMap) mapEverOpened = true; });
```

First open mounts the overlay and pays the IPC cost **once, interactively, where the user
expects it**. Subsequent opens reuse the mounted instance, so drill-down state
(`mapFocusNode`, `mapColorMode`) survives exactly as with the CSS-hiding pattern. The
`*EverOpened` flag is sticky within a Universe session — it never flips back on close —
so "has the user ever opened this view" is preserved across every show/hide cycle.

**Reset on context switch.** If the component's IPC only fires from `onMount` (common)
and the context changes (Universe switch, account switch, vault switch), the flag must
be reset, otherwise the user sees stale data from the prior context. In Constellation,
`handleUniverseSwitch` resets `mapEverOpened` / `orgChartEverOpened` alongside the other
in-memory state clears, so the next Map/OrgChart open re-mounts and re-fetches for the
new Universe. Any future overlay using this pattern must do the same.

**Rule.** Any always-mounted component that performs IPC during `onMount` or mount-time
`$effect` must be audited. If the IPC walks the filesystem, opens a DB, or reads anything
larger than O(1), default to lazy-mount with the `*EverOpened` pattern. CSS-hiding is for
components whose mount is cheap and which the user toggles frequently. Lazy-mount is for
components whose mount is expensive and which the user may never open.

**Relationship to Rule 8 (Write-Time Derivation).** Rule 8 is the deeper fix: persist the
map tree via triggers on note save/rename, so first-open is a cheap SQLite read instead of
a filesystem walk. Until that lands, lazy-mount is the cheap win that keeps the boot path
clean. Once the persistent-map refactor ships, the `*EverOpened` pattern becomes
effectively free (DB read on first open, cached afterwards) but still worth keeping — it
also defers the D3 / PIXI component mount itself, which is non-trivial CPU in its own right.

**How to find more of these.** Check the boot `ipc_arrival_log` after `paint_ms`. Any
command that arrives unbidden — i.e., not triggered by a user gesture — and whose
component is currently visible or always-mounted, is a lazy-mount candidate. On the 7,600-
note Universe, the arrival log should be essentially empty after hydration; if it isn't,
start there.

---

## LL-023: Don't Regress Working Features When Adding New Ones

**The rule.** Before shipping any gate, guard, conditional, or refactor around
existing behavior, verify the **baseline path** still executes end-to-end.
"Does X still render?" is not enough — also ask "does X still function?"
Every feature that works today is a hard-won win paid for in a previous
iteration. Losing one costs the time of that iteration all over again.

**The failure mode.** You add a clean new system (placement layer, routing
layer, permission layer, feature flag) that guards an existing surface. You
check the obvious surface: "The button is still visible." You don't check
the invisible machinery around it. An auto-reset `$effect`, a CSS override,
an upstream state derivation, a selector that filters by a property your
new guard doesn't know about — any of these can sit one level below your
audit and silently steal the click, swallow the event, or reset the state.
The user reports regression; you debug your own footprint; the time you
saved by "just adding the guard" is now gone, plus some.

**The case that earned this.** Tier 1 panel placement (2026-04-22). Adding
`panelPlacements` to the right-sidebar tabs introduced visibility guards on
the tab buttons **and** a safety `$effect` that reset `rightSidebarTab`
when the current tab's panel was placed outside `'right-sidebar'`. The
default placement for backlinks was `'left-of-note'`, so the safety effect
evaluated the Backlinks tab as "not visible," and any user click on the
Backlinks tab was silently reset to Properties one tick later. The tab
looked fine and the content path rendered correctly; the **click itself
was being stolen**. Cost: one debugging round after the user caught it.

**The verification protocol.** After any refactor that adds conditional
logic around an existing surface, run through this mental checklist:

1. **Render path** — does the surface still appear in the DOM?
2. **Event path** — does a click / keypress / focus still reach the intended
   handler, and does the handler still do what it used to do?
3. **State path** — does the state change from the event persist, or does
   something downstream revert it? Check `$effect`s, derived stores,
   observers.
4. **Data path** — is the content still populated with the right data, or
   is a filter / selector silently excluding it?

The cost of this checklist is seconds. The cost of skipping it is the time
of the iteration that built the feature you just broke.

**Rule extension (for `$effect` audits).** Any `$effect` that can **overwrite
user intent** (auto-close, auto-reset, auto-navigate) is high-risk and must
be re-read end-to-end whenever a new branch is added to the system it
monitors. A reset effect with a single stale entry in its visibility map is
exactly enough to undo a new feature.

---

## LL-024: Never Scan the Whole Vault on the Read Path — Use the Index

**Symptom (2026-05-21):** every note open lagged ~5 s with stuttering scroll —
*independent of the opened note's size or media*, and it re-lagged on every
reopen.

**Root cause:** the Unlinked Mentions feature (`scan_unlinked_mentions`,
`libraries.rs`) walked the entire library tree and `fs::read_to_string` +
regex-scanned **every** `.md` file (7,646 of them) on **every** note open,
uncached. Reading thousands of files pegged CPU/disk and starved the WebView's
scroll rendering for ~5 s. The "size-independent" tell is the giveaway: the
cost was scanning the *other* notes, not the one being opened.

**Fix:** select candidates from the always-current FTS index (`notes_fts`
phrase MATCH on the Arabic-normalized title → JOIN note_meta → ≤300
candidates), then run the exact original verification (wikilink-strip +
word-boundary regex) on only those few files. ~50× fewer reads → sub-100 ms.

**Rule:** any feature answering a "which notes relate to X?" question must read
the index, never walk the filesystem on the read path. If you find a `read_dir`
+ `read_to_string` loop firing on note open / panel focus / tab change, it's a
bug — route it through FTS, or maintain the answer at write time (Rule 8).
Known remaining offender to migrate: `scan_library_tags` (boot/Dashboard).

**Companion lesson — the WAL high-water mark:** the same session found a
372 MB `search.db-wal` adding ~1.1 s to every boot. Passive auto-checkpoints
reset the WAL's reuse position but **never shrink the FILE** — a past heavy
write (re-index / backfill) leaves a high-water mark forever. Run
`PRAGMA wal_checkpoint(TRUNCATE)` on a background connection (off the boot
path) to reclaim it, and set `synchronous=NORMAL` — safe here because the
search index is ephemeral (rebuilt from the `.md` files), and it makes writes
(note-create / typing) far faster.

## LL-025: Test DB Migrations Under Live App Concurrency — Not Just an Isolated Copy

**Symptom (2026-05-21 → 22):** MIG-041's chunked bigram purge passed an isolated
copy-test, then **stalled at exactly 600k rows for ~7 hours** on the live DB.
MIG-042's copy-test (on a copy of the *real* DB) caught a totally different
blocker — an orphaned trigger (BUG-020) — that no synthetic in-memory fixture
contained.

**Root cause:** an isolated copy has no running app, no WAL-checkpoint daemon, no
concurrent IPC, and none of the field-accumulated schema cruft a real DB carries.
MIG-041 died on a `SQLITE_BUSY` collision with the daemon; MIG-042 would have
died on a leftover trigger from MIG-028. Neither was reproducible without the
real DB and/or live concurrency.

**Rule:** a one-time DB migration MUST be (a) `SQLITE_BUSY`-resilient (retry, not
fatal), (b) coordinated with every other background DB user (pause the WAL daemon
via a shared flag for the duration), and (c) tested on a **copy of the real
production DB**, not a clean synthetic one — the real DB is where orphaned
triggers, partial-migration states, and odd column values live. Run the *exact*
shipped DDL against that copy before touching the live DB.

## LL-026: A `CREATE INDEX` Before Its `CREATE TABLE` Crashes Fresh-DB Init — and `IF NOT EXISTS` Hides It

**Symptom (2026-05-22, BUG-021):** brand-new / rebuilt universes failed to
initialize their search index ("no such table: note_links"); existing universes
were fine. Surfaced first as 4 long-failing `tests_m8c` unit tests, then bit a
real universe ("Eisa Universe") that left it stuck at "0 notes."

**Root cause:** `init_db` ran `CREATE INDEX … idx_link_target_path ON note_links`
(added during MIG-025) ~200 lines **before** `CREATE TABLE note_links`. On an
**existing** DB the table already exists, so `CREATE INDEX IF NOT EXISTS`
no-ops — the bug is invisible. On a **fresh** DB (a new universe, or any
`needs_rebuild` path that deletes + recreates the file) the index statement
aborts the whole init.

**Rule:** in any schema-bootstrap routine, **every `CREATE INDEX`/trigger must
come after the `CREATE TABLE` it references.** `IF NOT EXISTS` is not a safety
net — it actively *masks* ordering bugs on populated DBs, so a fresh-DB unit test
(init_db on an empty file) is mandatory and must stay green. If long-failing
"fresh DB" tests exist, treat them as a live latent bug, not test noise.

## LL-027: Removing an Automatic Maintenance Pass for Performance Requires a *Verified* Recovery Path

**Symptom (2026-05-22, BUG-022):** a universe whose index was empty (BUG-021's
victims, a wiped/restored DB, or files synced in while the app was closed)
displayed "0 notes" forever with **no automatic or manual way to rebuild** —
even after the crash that emptied it was fixed.

**Root cause:** the warm-boot "ZERO BOOT-TIME WALKS" optimization removed the
boot-time index walk (correct — it was thrashing every boot). Its code comment
said the walk was "now triggered by the file watcher / Settings → Rebuild Index /
a first-launch empty-cache modal." But two of those three were **never actually
built**: there is no "Rebuild Index" button, and the empty-cache prompt is
first-launch-only, not per-universe. So an empty index had no recovery.

**Rule:** when you delete an automatic maintenance/recovery pass for performance,
you must ship the replacement recovery path **in the same change** and **verify
it exists** — never trust a comment that says "now handled by X" without grepping
for X's implementation. Prefer a *gated* automatic recovery (here: rebuild only
when the index is empty, so the perf win is preserved for the common case) over a
manual button the user has to know to click.

## LL-028: Windows + Tauri Release Builds Silently Fail to Update `build/` While the App is Running

**Symptom (2026-05-23, MIG-044 Phase 2):** `npm run tauri build` reported `✓ built in 1m 31s` + produced a fresh `.exe` and NSIS installer, but the embedded SvelteKit bundle (`build/_app/immutable/chunks/*.js`) was **stale** — new code (`tooltipHeadline`, `.local-star-tooltip`) literally wasn't in the output. Two consecutive Boss installs shipped stale bundles. Burned an hour chasing "why doesn't my Svelte change appear" before noticing the actual EPERM.

**Root cause:** when Constellation is running (or even just shutting down — `msedgewebview2.exe` instances often persist 30-60s after the parent process exits), Windows file locks prevent SvelteKit's `adapter-static` from deleting the `build/` directory. The Vite compilation runs to completion; the `adapt(): rimraf build/` step throws `EPERM, Permission denied` but the tauri-CLI wrapper does NOT propagate that as a build failure — it logs the error and proceeds to Rust compile + bundle, producing a fresh `.exe` that embeds the OLD `build/` contents.

**Rule:**
- **Before any `npm run tauri build`**: `tasklist | grep -iE "constellation|msedgewebview2"`. If anything matches, fully close the app and wait for the WebView2 processes to exit (or kill them).
- **After any "successful" build that includes a user-visible CSS class or template change**: bundle-grep verify with `grep -rh "<unique-token>" build/_app/immutable/` — if the unique token doesn't appear, the build silently failed to copy. Don't ship a Boss-test install on an unverified build.
- If you see `EPERM, Permission denied: ...build` anywhere in the build output, treat it as a HARD FAILURE even though the CLI doesn't.

## LL-029: Predecessor Lookup Before Editing — Grep the Import Graph, Not the File Name

**Symptom (2026-05-23, MIG-044 Phase 2 §4):** spent two build cycles wiring tooltip headlines into `SkyView.svelte` and then `FullSkyView.svelte` — both shipped Boss-test installs that visibly failed. Bundle-grep eventually revealed neither file was in the bundle: **both are dead code**. The actually-rendered Sky View component is `LocalSkyView.svelte` (despite the "Local" prefix suggesting second-screen-only). A single `grep -r "import.*SkyView" src/` would have answered this in 2 seconds — it was never run.

**Root cause:** file-name authority was assumed. "Sky View bubble inspector" → `SkyView.svelte`. The reasoning sounded right (matches the feature name, the file exists, has the relevant types and hover logic). What wasn't verified: that anything actually *uses* the file. `+layout.svelte` only imports `LocalSkyView`. `SkyView.svelte` and `FullSkyView.svelte` are stale relics of some past refactor that nobody deleted. Vite tree-shakes them; the fixes compiled correctly but were unreachable code.

**Rule:** Before editing any `.svelte` component to wire a user-visible feature, **`grep -r "import.*ComponentName" src/`**. File names are not the source of truth; the import graph is. This applies especially when:
- The repo has multiple components with similar names (`SkyView` / `FullSkyView` / `LocalSkyView`; `MapPane` / `GlobalMap` / etc.).
- A past refactor split or renamed components but left the old files in place.
- The architect doc names a feature (not a file) — never assume the file name matches.

This is the BASIC RULE in the wiring-task domain: don't make up which file is "the right one" — verify it. **Promotes to Predecessor Lookup Rule application:** the predecessor of "the Sky View hover surface" is whatever component is actually mounted in the visible panel, not whatever has the matching filename. Write the Predecessor → Replacement entry against the import graph, not the file tree.

**Follow-up addition (same session, GraphMindView miss):** A name-pattern grep is insufficient — the full-window "Sky View" turned out to be `<GraphMindView>` (filename doesn't contain "SkyView" at all), missed even after the first grep cycle. The fix: when wiring a feature, **enumerate every mounted component for that feature** by grep'ing the layout for `<[A-Z]\w*` template renders and reading the conditional branches around the feature flag (e.g. `showSkyView`, `showMap`). Look for components that LOOK unrelated by name but render under the feature's flag. A 2-second grep for `<[A-Z]\w*` in `+layout.svelte` filtered through the feature-flag block would have caught GraphMindView from the start.

## LL-030: Tauri's WebView Eats HTML5 Drag-and-Drop — Use Pointer Events for In-Page Dragging

**Symptom (2026-05-30, MIG-065 §K):** column drag-to-reorder in the Base table did nothing — the grab cursor appeared on hover, but `ondragstart`/`ondragover`/`ondrop` never fired. A first fix (changing a `<button>` sort label to a `<span>`, on the theory that the interactive button swallowed the parent header's drag) **also failed** — the second "still not working" was the signal to stop patching and find the root cause.

**Root cause:** Tauri v2 enables drag-and-drop interception on the webview by **default** (no `dragDrop` key in `tauri.conf.json` = ON). When on, the native window grabs OS-level drag-and-drop (for file drops) **before the page sees it**, so HTML5 *element* drag-and-drop inside the page silently never starts. The app used no Tauri file-drop anywhere, so nothing surfaced the conflict — the drag just died with no error.

**Rule:** For in-page drag interactions (column/row reorder, drag-to-sort, kanban) in a Tauri app, **do not rely on HTML5 drag-and-drop.** Prefer raw **pointer events**: `onmousedown` on the handle + window `mousemove`/`mouseup`, a small movement threshold (≈5 px) to tell a drag from a click, and `document.elementFromPoint(x,y).closest('[data-col]')` for drop-target hit-testing. This is contained to the component, survives the WebView quirk, gives full control of the drag visual, and — unlike the alternative `dragDropEnabled: false` config — does **not** disable OS file-drop app-wide. Always: remove the window listeners on unmount (Perf Rule 4); set a `suppressNextClick` flag so a drag doesn't also fire the handle's click (e.g. sort); and set `user-select: none` on the draggable elements (and the container during a drag) or the pointer-drag will text-select everything it sweeps across.

## LL-031: The Orientation Bump Rides In the Feature Commit — Never Batched at Handover

**Symptom (2026-06-05, MIG-070 §C):** the Boss asked *"What about the Orientation file?"* during a PCS+O audit. That question is the **documented tell** of an SO #6 violation (memory `feedback_orientation_inline_with_commit`: *"If Eisa has to ask 'Orientation file?' — that's a SO #6 violation"*). The orientation file v2.52 **existed and was committed** — but the *process* was wrong: it had been batched and deferred.

**Root cause:** the orientation v-bump was treated as **end-of-arc PCS housekeeping** instead of a **per-trigger obligation**. The migration-kickoff bump (v2.51, inline at Phase 0/1 `6d4c3e28`) felt "done," so eight subsequent §C commits were allowed to ship with **zero** orientation touch — three of them clear "subsystem ships a major feature" triggers (saved colour swatches `1a743c35`, focused per-element preview `33046ccc`, per-script fonts `e0df6063`). The bump was finally written only at handover, in a trailing docs commit (`4ce37ab2`) — i.e. batched across the very commits that should each have carried it.

**Rule:** SO #6 is **per-trigger, not per-arc.** Every commit that ships a user-facing feature, opens/closes a BUG, adds an LL, rewords a top-principal, fixes a §-drift item, or bumps a version carries its orientation update **in that same commit** — a date-stamped section update at minimum, a version bump on a structural change. **Mid-migration is not an exception:** each phase that ships a major feature triggers, even though the migration as a whole is still open. The self-check: *if you are about to write a trailing "docs/handover" commit that bumps the orientation for features which shipped in earlier commits, you have already violated the rule — the bump belonged upstream.* And the loudest signal that you missed it is the Boss having to ask where the orientation file is.

---

*Last updated: 2026-06-05 (LL-031 added — SO #6 orientation-bump was batched across the
MIG-070 §C feature commits and only written at handover; the tell was the Boss asking
"Orientation file?". The bump must ride in the feature commit, never a trailing docs commit).*
*Earlier: 2026-05-30 (LL-030 — Tauri WebView intercepts HTML5 drag-and-drop by default;
rebuilt on pointer events after a button→span patch failed to address the real cause).*
*Earlier: 2026-05-23 (LL-028/029 — MIG-044 Phase 2 Sky View correction arc:
Windows file-lock silent-build-failure during Tauri release build, and the BASIC
RULE / Predecessor Lookup violation that wired the wrong .svelte file twice before
grep-verifying the import graph).*
*For: Constellation — orientation/SO-#6 discipline + in-page drag in Tauri WebViews + release-build verification + file-name vs. import-graph discipline*
