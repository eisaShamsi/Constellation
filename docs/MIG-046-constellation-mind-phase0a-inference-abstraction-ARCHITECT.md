# MIG-046 — Constellation Mind, Phase 0a: Inference Abstraction Skeleton

**Status:** Architect (Phase 1 of the `/migration` workflow). Awaiting Eisa approval of this doc before the Plan is drafted.
**Date:** 2026-05-24
**Lineage:**
- Phase 0a of the Constellation Mind Implementation Plan v1.0 (Plan §4).
- Builds on the Constellation Mind Concept Paper v1.1 (`docs/Constellation-Mind-Concept-Paper-v1.1.md`) — specifically §10.1 (split traits, MA-1), §10.2 (RoutedProvider promotion + LoadStrategy), §10.3 (tool-call budget, MA-4), §10.4 (tool-result framing, MA-5).
- Pre-flight closed: PF-1 license verdict at Plan §10 (Fanar GO-with-conditions, Jais GO-with-conditions / gated). PF-2 v1.1 Concept Paper shipped.

---

## 1. Goal

Lay the **trait surface** of Constellation Mind — the strategic moat (Concept Paper §5.5 / §14.2) — before any real-inference cost is paid:

1. Land the **two traits** (`InferenceProvider` + `EmbeddingProvider`) and the **StreamEvent** enum that gates the entire LLM surface.
2. Land **three stub providers** (`LocalProvider`, `CloudProvider`, `OfflineProvider`) that emit deterministic event sequences, so unit tests can exercise the trait end-to-end without any model download or runtime dependency.
3. Land the **Tauri IPC contract** for streaming (`mind_start_turn` returning a `Channel<StreamEvent>`).
4. Land the **`ChatOrchestrator` skeleton** that consumes the trait via the stubs.
5. Land **in-process telemetry counters** (tokens in/out, latency, provider/model identity) with a snapshot command — never exfiltrated.

What Phase 0a does **NOT** do (those are Phase 0b / MIG-047):

- No real LLM inference (no `mistral.rs`, no `llama-cpp-2`, no real Anthropic HTTP calls).
- No model downloads, no install flow, no first-launch picker.
- No retrieval wiring (chunks remain empty in the prompt envelope).
- No real tool dispatcher (every tool name returns a canned `{"status": "ok"}`).
- No frontend chat UI (Phase 1 / MIG-048).
- No `RoutedProvider` implementation (Phase 2.5 / MIG-050 — the trait surface must admit it cleanly, but it is not built here).

---

## 2. Territory map (verified against current code, 2026-05-24)

### 2.1 What is already there that the Mind subsystem must coexist with

| Surface | File | Today's role | Phase 0a treatment |
|---|---|---|---|
| **Cloud AI bridge** | [src-tauri/src/ai/mod.rs](src-tauri/src/ai/mod.rs) — IPC commands `ai_send_message` / `ai_validate_connection` / `ai_list_models` wired in `lib.rs:304-306`; supports OpenAI / Anthropic / Gemini / Ollama; non-streaming `reqwest::Client` calls. Used by `src/lib/ai/engine.ts` (sole frontend consumer) — OpenClaw-style request/response chat. | **Leave untouched.** Phase 5 (MIG-053) eventually refactors this into a `CloudProvider` implementation of the new `InferenceProvider` trait. In 0a, the new code lives in `mind/` parallel to `ai/`; nothing else touches `ai/`. |
| **CECE reasoning cataloger** | [src-tauri/src/cece/catalogers/reasoning.rs](src-tauri/src/cece/catalogers/reasoning.rs) — has an `InferenceFn = Box<dyn Fn(prompt, grammar) -> String>` injection point for a local LLM (planned Qwen3-4B-Instruct-2507 Q5_K_M via llama.cpp); presently unwired (`llama-cpp-2` dep not yet added — abstains gracefully). Drives MIG-021v3 epistemic-content classification. | **Leave untouched.** The new trait surface is designed to *admit* this `InferenceFn` shape as a future adapter (Phase 3 / MIG-051 will rewire CECE's local-LLM call through a `RoutedProvider`); in 0a, CECE keeps its own injection. |
| **ONNX embedding engine** | [src-tauri/src/embeddings.rs](src-tauri/src/embeddings.rs) — `ort` + `tokenizers`, `multilingual-e5-small` (384-dim, 100 languages), 100% offline. Drives the existing HMSE semantic retrieval. | **Leave untouched.** Natural fit for a future `LocalEmbeddingProvider` (Phase 0b / Phase 1). In 0a, the `EmbeddingProvider` trait is declared but `embeddings.rs` is not refactored to implement it yet — a stub `LocalEmbeddingProvider` lives in `mind/providers/local.rs`. |
| **NSC summary engine** | `src-tauri/src/nsc/mod.rs` — MIG-040..MIG-045 NSC Core Plug-in. `summaryStore.ts` shared frontend service. | **Leave untouched.** Phase 1 (MIG-048) wires the `summarize` tool to delegate to NSC (`getSummariesFor`) [MA-3]. In 0a, the canned tool dispatcher returns `{"status": "ok"}` for `summarize` and every other tool name. |
| **Tauri streaming pattern** | Tauri v2 supports typed `tauri::ipc::Channel<T>` for one-direction backend→frontend streaming (the idiomatic streaming primitive in v2). The codebase already uses Tauri events liberally for cross-window sync but no existing IPC streams typed events. | **Adopt `Channel<StreamEvent>`** as the streaming contract — one channel per `mind_start_turn` invocation. (Option C in §4 below.) |

### 2.2 What is genuinely new in Phase 0a

A new top-level module:

```
src-tauri/src/mind/
├── mod.rs                  # re-exports + module wiring
├── provider.rs             # InferenceProvider + EmbeddingProvider traits + supporting types
├── events.rs               # StreamEvent enum + serde-friendly subsidiary types
├── orchestrator.rs         # ChatOrchestrator skeleton
├── telemetry.rs            # in-process counters + snapshot command
├── commands.rs             # Tauri IPC commands (mind_start_turn, mind_telemetry_snapshot)
└── providers/
    ├── mod.rs
    ├── local.rs            # LocalProvider stub (deterministic event sequence)
    ├── cloud.rs            # CloudProvider stub (Anthropic-shaped scaffold, no network)
    └── offline.rs          # OfflineProvider stub (always returns a "no model" message)
```

Wired in `lib.rs` `invoke_handler` alongside the existing `ai::*` commands.

---

## 3. Invariants that MUST NOT break

1. **`ai/` keeps working unchanged.** `src/lib/ai/engine.ts` continues to talk to `ai_send_message` / `ai_validate_connection` / `ai_list_models` exactly as it does today. No frontend code is edited in Phase 0a.
2. **`cece/catalogers/reasoning.rs` keeps its `InferenceFn` injection point.** The new trait surface is designed to *admit* an adapter from `Arc<dyn InferenceProvider>` to `InferenceFn` (Phase 3 work), but no rewiring happens in 0a.
3. **No new heavy dependencies.** `mistral.rs`, `llama-cpp-2`, `candle`, and the like are NOT added to `Cargo.toml` in 0a. The Windows-toolchain risk that CECE V3-§7 deliberately deferred (see `cece/catalogers/reasoning.rs` header comment, ~L23-27) is the same risk we keep deferred. Phase 0b is where that lands.
4. **No schema changes.** `note_summaries`, `note_meta`, `note_links`, `note_embeddings` — all untouched. Telemetry is in-process only.
5. **No boot regression.** Phase 0a code is dormant on startup: the `mind` module registers its IPC commands but loads no model and starts no background task. Measured on the live 7,600-note universe before/after (CLAUDE.md hard constraint).
6. **No hot-path additions.** No CM6 plugin, no `$effect`, no IPC call on the keystroke path (Rule 1).
7. **Local-First / no exfiltration.** Telemetry counters never leave the device. No outbound HTTP from `mind/` in 0a (the CloudProvider stub is a scaffold that returns canned events without contacting any server).
8. **Trait surface must admit Phase 0b+ realities without breaking changes**, specifically:
   - Real `LocalProvider` (mistral.rs or llama-cpp-2 — chosen by Phase 0b bench)
   - Real `CloudProvider` for Anthropic Claude (Phase 5)
   - `RoutedProvider` composing 1..N inner providers (Phase 2.5) — itself implementing `InferenceProvider`
   - `LocalEmbeddingProvider` wrapping `embeddings.rs`'s `ort` session (Phase 0b / 1)
   - `framing::as_tool_result` central sanitizer for prompt-injection guard (MA-5, used in real ToolDispatcher in Phase 1)
   - `max_tool_rounds_per_turn` budget (MA-4, used in real orchestrator in Phase 1)
   - `Approval::Rejected { reason, scope }` shape for write rejections (MA-2, used in real ToolDispatcher in Phase 2)
9. **Concept Paper v1.1 §10 alignment.** Every type signature in `provider.rs` matches the Rust shown in Concept Paper v1.1 §10.1 + §10.2.

---

## 4. Design options

### A. Module layout

- **A1 — `src-tauri/src/mind/` parallel to `ai/` (CHOSEN).** Mirrors the existing pattern (`ai/`, `nsc/`, `cece/`, `classifier/`, `sources/`). Coexistence by namespace. Future `ai/` → `CloudProvider` migration (Phase 5) is a *content* move, not a *location* move.
- A2 — Subsume `ai/` into `mind/` in 0a. Rejected — violates invariant 1 and the "secure what's achieved, don't muddle" principle. Refactor of an in-use surface is not a skeleton-phase activity.

### B. Trait shape

- **B1 — Split `InferenceProvider` + `EmbeddingProvider` (CHOSEN per MA-1, v1.1 §10.1).** Two surfaces, composed orthogonally. A model that is great at generation but indifferent at embedding (or vice versa) can implement just one. Trait objects (`Arc<dyn InferenceProvider>`, `Arc<dyn EmbeddingProvider>`) compose into higher-level structs (RoutedProvider, ChatOrchestrator).
- B2 — One unified trait. Rejected per MA-1; couples two axes that evolve independently.

### C. Streaming mechanism for `StreamEvent`

- **C1 — `tauri::ipc::Channel<StreamEvent>` (CHOSEN).** Tauri v2's first-class typed-event channel. One channel per `mind_start_turn` invoke; the backend command pushes `StreamEvent::Token` / `ToolCall` / `Done` / `Error` events; the frontend awaits via `channel.onmessage`. Lifecycle is bound to the command's `Future` — when the orchestrator returns, the channel closes naturally. Cleanest Svelte 5 integration via a `$state` rune that consumes channel events.
- C2 — Global Tauri events via `app.emit`. Rejected — global event pollution; per-conversation scoping requires manual filtering by id; harder to clean up on conversation abandonment.
- C3 — Polling via repeated `take_next_event` invocations. Rejected — burns IPC; bad for Rule 3 (no `invoke()` on hot paths); the user feels every poll interval as latency.

### D. Stub provider behavior

- **D1 — Deterministic scripted stubs (CHOSEN).** Each stub emits a known event sequence: `LocalProvider` stub emits five `Token("…")` events then `Done`; a `ToolCall` variant of the stub emits one `ToolCall(name="search_notes", args={"query": "x"})` then waits for `push_tool_result()` then resumes with three more tokens and `Done`. Lets unit tests assert exact event sequences and round-trip behavior.
- D2 — Random / echo stubs. Rejected — non-deterministic tests are not tests.
- D3 — Feature-gated real-model stubs (fallback to script). Rejected — Phase 0a explicitly avoids real-model dependency (invariant 3).

### E. Telemetry shape

- **E1 — In-process `TelemetryCounters` struct + snapshot command (CHOSEN).** Fields: `turn_count`, `tokens_in`, `tokens_out`, `latency_ms_p50`, `latency_ms_p99`, `provider_id`, `model_id`, `tool_calls_count`, `tool_call_rounds_exceeded_count` (the MA-4 budget hit), `errors_count`. Exposed via a single `mind_telemetry_snapshot` Tauri command returning a serializable struct. Never written to disk in 0a (Phase 0b/5 decides persistence). Never sent over the network — there are no outbound HTTP calls in `mind/` in 0a.
- E2 — SQLite-backed telemetry from day one. Deferred to Phase 5 (cost-telemetry contract, MA-6) where it has a clear consumer.
- E3 — No telemetry. Rejected — Plan §4 Phase 0a explicitly lists it as a deliverable.

### F. `ChatOrchestrator` scope in Phase 0a

- **F1 — Skeleton that exercises the trait surface end-to-end with stubs (CHOSEN).** Owns one conversation, holds an `Arc<dyn InferenceProvider>`, holds a placeholder `ToolDispatcher` that returns canned `{"status": "ok"}` for any tool name, holds an empty `ConversationHistory`. The `turn()` method runs the same shape as Concept Paper v1.1 §10.3 — including the `tool_rounds` counter (so MA-4 is exercised even with stubs). No retrieval (chunks are empty in 0a; Phase 1 wires HybridRetriever).
- F2 — Trait + stubs only, no orchestrator. Rejected — without an orchestrator the unit tests can't exercise the trait in a realistic call-graph. We need to know now (not in 0b) whether the `Channel<StreamEvent>` + `recv().await` + `tool_rounds` loop composes cleanly.

### G. `framing::as_tool_result` central sanitizer placement

- **G1 — Lives in `mind/orchestrator.rs` as a small `framing` submodule, even though the canned tool dispatcher in 0a doesn't need it (CHOSEN).** Phase 1's real ToolDispatcher will use it (MA-5); pre-allocating its location now means Phase 1 doesn't have to introduce a new location alongside other new logic. The 0a version is a one-liner that wraps the result in `<tool_result name="…">{json}</tool_result>` text framing.

---

## 5. Plan outline (each step = one commit + verification clause)

> **Step A — Trait crate scaffolding.**
> Create `src-tauri/src/mind/{mod.rs, provider.rs, events.rs}`. Declare both traits + `StreamEvent` + supporting types (`ChatMessage`, `GenParams`, `ToolSchema`, `ToolChoice`, `ProviderCapabilities`, `EmbeddingCapabilities`, `InferenceError`). All types `Serialize`/`Deserialize` where they cross the IPC boundary. No impls yet.
> *Verify:* `cargo build` green; `cargo check --all-targets` green; svelte-check 0 new errors (no frontend touched).

> **Step B — Three stub providers.**
> Create `src-tauri/src/mind/providers/{mod.rs, local.rs, cloud.rs, offline.rs}`. Each implements `InferenceProvider` with deterministic event sequences per Option D1. `LocalProvider` stub additionally implements `EmbeddingProvider` returning a fixed 384-dim zero vector (matching `multilingual-e5-small`'s dimensionality, so future swap-in is dimensionally consistent). Unit tests in `mind/providers/local.rs` mod-tests: (1) `generate()` emits the expected 5-token sequence; (2) tool-call round trip works when `push_tool_result()` is called between events; (3) `OfflineProvider.generate()` returns one synthesized message + `Done`.
> *Verify:* `cargo test -p constellation mind::` green (≥6 tests pass).

> **Step C — Tauri IPC + `Channel<StreamEvent>`.**
> Create `src-tauri/src/mind/commands.rs`. Add commands `mind_start_turn(channel: Channel<StreamEvent>, request: StartTurnRequest)` and `mind_telemetry_snapshot() -> TelemetrySnapshot`. Wire both in `lib.rs:invoke_handler!` macro alongside the existing `ai::*` entries (do not touch the `ai::*` entries). `mind_start_turn` spawns a `tauri::async_runtime::spawn` task that drives the provider through one turn and pushes events to the channel.
> *Verify:* `cargo build` green; manual IPC smoke test from a throwaway dev script (or a `#[cfg(test)]` test calling the command via Tauri's test harness, if feasible) — `mind_start_turn` opens a channel, emits Tokens, closes; `mind_telemetry_snapshot` returns a struct with zero counters initially.

> **Step D — `ChatOrchestrator` skeleton.**
> Create `src-tauri/src/mind/orchestrator.rs`. Implement `ChatOrchestrator::new(provider, dispatcher)`, `turn(user_message, ui_tx)`. Honor the `tool_rounds` counter from v1.1 §10.3 (MA-4 budget exercised). Honor the `framing::as_tool_result` central sanitizer hook (a no-op pass-through in 0a, MA-5 placeholder). The placeholder `ToolDispatcher` returns `{"status": "ok"}` for any tool name. No retrieval (chunks empty).
> *Verify:* `cargo test -p constellation mind::orchestrator` green — unit test runs one full turn with `LocalProvider` stub, asserts the sequence of UI events emitted, asserts the tool-rounds counter stays at ≤5 even when the stub repeatedly emits ToolCall (which triggers the budget abort path).

> **Step E — Telemetry counters.**
> Create `src-tauri/src/mind/telemetry.rs`. Implement `TelemetryCounters` (atomics) + `snapshot()` -> `TelemetrySnapshot`. Wire into the orchestrator (count tokens in/out, time the turn, count tool rounds, count budget hits). `mind_telemetry_snapshot` command reads + returns. No persistence.
> *Verify:* After Step D's orchestrator test runs, `snapshot()` reflects the counts the test exercised. No outbound network calls anywhere in `mind/`.

> **Step F — `/simplify` + 3-agent audit.**
> Run `/simplify` on the full diff (Steps A–E). Three parallel agents per the Migration Rule: (4A) invariants §3 hold; (4B) drift — any other module silently picking up `mind::*` symbols, any new `invoke_handler!` ordering risk; (4C) migration path — fresh universe, existing universe, mid-turn interrupt (cancel a `mind_start_turn`), rollback (drop the `mind/` directory + the three `invoke_handler!` lines, verify `ai/` and everything else still works).

> **Step G — SO + docs.**
> Session log (`lab/reports/SESSION-LOG-2026-05-24.md` — Constellation Mind kickoff entry). Orientation v-bump documenting the new `src-tauri/src/mind/` module. **No user-visible surface in 0a → no help-file additions in 0a** (the 15-locale help discipline activates with Phase 1's chat surface). First MoCh of Constellation Mind work (`docs/MoCh/MoCh-2026-05-24-HHMM.md`). PCS gate awaits Eisa's explicit go.

---

## 6. Migration-path matrix

| Scenario | Behavior |
|---|---|
| **Fresh install** | `mind/` module compiles and is dormant. No model is downloaded; no LLM call happens. Telemetry counters initialize at zero. User sees no change. |
| **Existing universe (Eisa's)** | Same — no schema touch, no migration triggered. Boot time on the 7,600-note universe: unchanged (measured before/after at Step F). |
| **Mid-turn interrupt (cancel `mind_start_turn`)** | The Tauri channel drops; the spawned async task observes the channel close and aborts gracefully. No state to roll back (no writes happened — 0a is read-only-stubs). |
| **Rollback to pre-MIG-046** | Delete `src-tauri/src/mind/` + remove the new `invoke_handler!` lines + drop the new dependencies (none in 0a — no `Cargo.toml` change). `ai/` continues working unchanged. Frontend continues working unchanged (no frontend was touched). |
| **Cargo.toml diff in 0a** | Zero new dependencies. Everything used (`serde`, `tokio`, `async_trait`, `tauri`) is already present. |

---

## 7. Risk summary

**Low.** Phase 0a is strictly additive: a new module that compiles, registers two Tauri commands, exposes a trait surface and stubs, and is not yet invoked by any frontend code. The genuinely-new risk is **design risk** — does the trait shape survive the Phase 0b real-inference integration?

Mitigation: the trait shape in `provider.rs` is designed against the documented APIs of all four targets it will eventually serve:

1. `mistral.rs` — `Pipeline::stream_chat_request` returns an async stream of token + tool-call events. Maps cleanly to `Receiver<StreamEvent>`.
2. `llama-cpp-2` — sampling loop emits tokens; tool-call parsing is application-level. Maps cleanly to `Receiver<StreamEvent>` with a small tool-call parser.
3. Anthropic Messages API streaming — `text_delta` / `input_json_delta` / `message_stop` events. Maps cleanly to `StreamEvent::Token` / `ToolCall` / `Done`.
4. OpenAI Chat Completions streaming — `content` deltas + `tool_calls` deltas + `finish_reason`. Maps cleanly to the same.

The trait does NOT assume any one of these specific shapes — it asks each implementation to produce a `Receiver<StreamEvent>`, leaving the per-runtime adaptation inside the implementation. This is what makes `InferenceProvider` the strategic moat the Concept Paper called it.

No schema change. No write-path change. No hot-path addition. Rollback is trivial in both directions.

---

## 8. What Phase 0a explicitly **does not** decide

Surfaced here so they don't get accidentally bundled in:

- **Choice of local runtime (`mistral.rs` vs `llama-cpp-2`).** Decided by Phase 0b's one-day micro-bench (Plan §1 Decision #4).
- **Bundled-default identity (Fanar vs Jais vs other).** Decided by Phase 0b's tool-use reliability benchmark (Plan §1 Decision #2). Plan §10 PF-1 verdict is the license input to that decision.
- **Fanar Gemma-notices question** (Plan §10.4 Q1). Lands at Phase 0b model-install flow at the latest.
- **Jais gate handling** (Plan §10.4 Q3). Lands at Phase 2.5 RoutedProvider when Jais becomes installable.
- **Whether `embeddings.rs` is refactored into `LocalEmbeddingProvider` immediately or kept as a parallel path.** Architectural call deferred to Phase 0b — the trait surface in 0a admits either choice without breaking changes.
- **First-launch download UX.** Phase 0b territory.

These belong to Phase 0b's Architect doc (MIG-047), not this one.

---

*Phase 0a of the Constellation Mind Implementation Plan v1.0 (Plan §4). On Eisa's approval of this Architect doc, I will draft the Plan (commit-level steps A–G with verification clauses, file paths confirmed against the territory map above). Phase 0b (MIG-047) follows as its own `/migration` once 0a's Build + Audit close.*
