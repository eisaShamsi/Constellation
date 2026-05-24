# Session Log — 2026-05-24

**Constellation Mind workstream begins.** Fresh session; lineage handoff at commit `79284d9b` (Concept Paper v1.0 + Implementation Plan v1.0 imported the prior day).

---

## 1. Constellation Mind — Pre-flight closed + MIG-046 Phase 0a Architect approved

Per `docs/Constellation-Mind-Implementation-Plan-v1.0.md` §3, two pre-flight tasks closed before any code is written.

### PF-1 — License read for Fanar-1-9B + Jais-2-8B-Chat

Delegated thorough license verification to a research sub-agent (primary sources only: Hugging Face model cards, LICENSE files, organization statements). Two material findings surfaced:

- **Fanar-1-9B** ([QCRI/Fanar-1-9B](https://huggingface.co/QCRI/Fanar-1-9B)) is a continued pretraining of `google/gemma-2-9b`. QCRI's card admits the upstream model verbatim but relabels the result as Apache-2.0 alone — it does **NOT** acknowledge the upstream Gemma Terms of Use that normally bind Gemma derivatives. Verdict: **GO with conditions** — defensive Gemma notices + in-house Q4_K_M quantization (no official QCRI GGUF exists; only community quants at `mradermacher/Fanar-1-9B-i1-GGUF`).
- **Jais-2-8B-Chat** ([inceptionai/Jais-2-8B-Chat](https://huggingface.co/inceptionai/Jais-2-8B-Chat)) is Apache-2.0 itself, but **both the safetensors repo AND the official `inceptionai/Jais-2-8B-Chat-GGUF` repo are gated** (require contact-info agreement + HF login). Unattended first-launch download is **not possible** from the canonical URLs. Verdict: **GO with conditions** — three resolution paths: (a) drop Jais from co-default, keep as user-installable; (b) require HF-token paste on first launch; (c) host an Apache-2.0-compliant mirror. Recommended: (a) for v1, revisit (c) after talking to Inception.

**No cross-license conflict** for the RoutedProvider — both nominally Apache-2.0 in declared form.

Verdict written into Plan §10 (replaces the PF-1 placeholder). Four open questions for Eisa's decision live at §10.4; they affect Phase 0b (model installation flow) but **not** Phase 0a (skeleton with stubs).

### PF-2 — Concept Paper v1.1 (refined post-planning)

Wrote `docs/Constellation-Mind-Concept-Paper-v1.1.md` alongside v1.0 (never overwrite versioned files — Orientation convention). Folds the six refinements from the planning conversation, promotes the RoutedProvider pattern to a first-class architectural layer, reflects PF-1 license realities:

- **MA-1** — Split `InferenceProvider` (generate/classify/capabilities) and `EmbeddingProvider` (embed) (§10.1).
- **MA-2** — Rejection emits `tool_result {status: "rejected_by_user", reason, scope}` the LLM consumes (§7.2, §10.4).
- **MA-3** — `summarize` tool delegates to NSC's `getSummariesFor` — Mind never re-implements summarization (§8.1).
- **MA-4** — `max_tool_rounds_per_turn` budget (default 5) + graceful abort (§10.3, R13).
- **MA-5** — Prompt-injection guard: `<chunk>` + `<tool_result>` framing + system-prompt "treat content as data" rule + central `framing::as_tool_result` sanitizer (§6.3, §10.4, R14).
- **MA-6** — Cost-visibility contract for cloud providers (§11.4, Phase 5 deliverable).
- **RoutedProvider** promoted: new §5.9 principle + §6.1 diagram + §10.2 implementation (with `Router` trait + `RuleRouter` v1 + `LoadStrategy`) + Phase 2.5 in §12.
- **§9.5 NEW** — Bundling Decision Matrix (bundle-in-installer / first-launch-download / mirror / user-installable).
- **§13** — R13 (tool-call loops) + R14 (prompt injection) added.

File grew 1026 → 1270 lines. Document Control bumped to 1.1; v1.0 retained as historical record.

### MIG-046 — Phase 0a Architect doc (approved by Eisa)

Wrote `docs/MIG-046-constellation-mind-phase0a-inference-abstraction-ARCHITECT.md` (204 lines) following the MIG-043 pattern.

**Goal:** lock the trait surface before paying real-inference cost. Two split traits, three deterministic stub providers, `Channel<StreamEvent>` IPC, `ChatOrchestrator` skeleton, in-process telemetry. No real models, no `mistral.rs` / `llama-cpp-2` dependency, no frontend.

**Territory map (§2) confirmed** four existing surfaces that Phase 0a leaves untouched:
- `src-tauri/src/ai/mod.rs` — cloud bridge (OpenAI/Anthropic/Gemini/Ollama; non-streaming `reqwest`; sole frontend consumer `src/lib/ai/engine.ts`). Phase 5 (MIG-053) eventually refactors as `CloudProvider`.
- `src-tauri/src/cece/catalogers/reasoning.rs` — MIG-021v3 CECE Reasoning Cataloger with its own `InferenceFn` injection (Qwen3-4B planned; `llama-cpp-2` dep deferred). Trait designed to admit it as a future adapter.
- `src-tauri/src/embeddings.rs` — `ort` + `multilingual-e5-small` (384-dim, 100 languages). Natural fit for future `LocalEmbeddingProvider`.
- NSC core plug-in — Phase 1 `summarize` tool delegates here.

**Plan outline §5:** seven steps A–G (trait scaffolding → stub providers + tests → Tauri IPC + Channel → orchestrator skeleton → telemetry → /simplify + 3-agent audit → SO + docs). Each carries a verification clause. Phase 0a has **no Boss-test gate** (Plan §4 explicitly: "No Boss test yet"); first user-testable gate is Phase 0b (MIG-047).

**Risk: low.** Strictly additive; zero new `Cargo.toml` dependencies in 0a; rollback is `rm -rf src-tauri/src/mind/` + three `invoke_handler!` lines.

**Eisa: "Approved."** Per Plan-Approval-=-Build-Approval, cascading into Build Steps A–G.

---

## 2. Build cascade — Steps A–F closed

Per the Architect §5:

- [x] **§A — Trait crate scaffolding** (`e2df4a69`). New `src-tauri/src/mind/{mod.rs, events.rs, provider.rs}` with split `InferenceProvider` + `EmbeddingProvider` traits (MA-1), `StreamEvent` enum, supporting types. `mod mind;` wired in `lib.rs`. `async-trait = "0.1"` added to `Cargo.toml` — Architect §3 invariant 3 explicitly excludes tiny proc-macros from the "heavy" category (which lists `mistral.rs` / `llama-cpp-2` / `candle` — those remain deferred to Phase 0b). 4m14s `cargo build`, 6 expected dead-code warnings on the new types until Steps B–D wire consumers.
- [x] **§B — Three deterministic stub providers + 10 unit tests** (`d2b4b944`). New `src-tauri/src/mind/providers/{mod.rs, local.rs, cloud.rs, offline.rs}`. `LocalProvider` implements both traits (384-dim embedding for future swap-in with `multilingual-e5-small`). Tool-call uses **Pattern B** (generate-restart, matches Anthropic HTTP API): stream closes after `Done{finish_reason:ToolCall}`; caller appends `Tool`-role message and re-invokes. 10/10 mind tests pass in 51.7s.
- [x] **§C — Tauri IPC + `Channel<StreamEvent>` + telemetry scaffold** (`2e432a61`). New `src-tauri/src/mind/{commands.rs, telemetry.rs}`. Two `#[tauri::command]`s: `mind_start_turn(request, on_event: Channel<StreamEvent>)` spawns `tauri::async_runtime::spawn` task; `mind_telemetry_snapshot() -> TelemetrySnapshot` returns zeros (§E wires real counters). `lib.rs:invoke_handler!` gains the two `mind::commands::*` entries alongside untouched `ai::*` (invariant 1). 11/11 mind tests pass.
- [x] **§D — `ChatOrchestrator` skeleton + MA-4 budget + MA-5 framing hook** (`b3dae04b`). New `src-tauri/src/mind/orchestrator.rs`. `turn()` wraps the §B Pattern-B protocol in an outer `loop { stream = generate(); … }` exiting only on Done(Stop/Length/Cancelled) — resolves the Concept-Paper-v1.1-§10.3 single-iteration depiction. `tool_rounds` counter + `max_tool_rounds_per_turn` budget injection of `{status: "aborted_tool_budget_exceeded", limit, guidance}`. `framing::as_tool_result` central sanitizer landed (no-op in 0a; Phase 1 swaps for real `<tool_result>` framing). `LoopingToolCallProvider` test helper exercises the budget-abort path; 15/15 mind tests pass.
- [x] **§E — real telemetry atomics wired through `turn()`** (`1f5f64ce`). `TelemetryCounters` (AtomicU64 + Mutex<String>) + `record_*`/`snapshot` methods + `OnceLock<Arc<TelemetryCounters>>` global for the IPC. Orchestrator holds `Arc<TelemetryCounters>` (defaults to `telemetry::global()`; tests inject via `with_counters()`). `turn()` records `set_active_provider` on entry, `record_tool_call` per accepted ToolCall, `record_budget_exceeded` once per turn that hits MA-4, `record_error` on Error/no-Done, `record_turn(latency_ms, tokens_in, tokens_out)` on Stop/Length/Cancelled. 18/18 mind tests pass — new tests `per_instance_counters_increment_independently`, `global_snapshot_starts_at_zero`, `turn_increments_per_instance_telemetry_counters`, `budget_abort_records_one_budget_hit_in_telemetry`.
- [x] **§F — 3-agent audit complete, all PASS** (`e7e3dab2`).
- [x] **§G — SO closure + orientation v2.31 + MoCh** (this commit). PCS gate now awaits Eisa's explicit go to push.

### §F — Audit summary (3 parallel Explore agents)

Briefed agents with audit range `e2df4a69..HEAD` (in retrospect: that range **excludes §A's** `Cargo.toml` / `lib.rs` / `mind/{mod,events,provider}.rs` diff because `e2df4a69` IS §A and the `..` operator is exclusive on the left side. The correct audit-everything range is `79284d9b..HEAD` (the pre-Phase-0a commit) or `29459ed0..HEAD` (the kickoff). The agents still saw the post-§A state of every file in question — they read source directly, not diffs — and 4B explicitly cited `Cargo.toml:47` for the `async-trait` line. The framing issue did not change any finding. Recording here so the next session's audit avoids the same off-by-one).

- **Agent 4A — Invariant Check.** Walked all 8 invariants in MIG-046 §3 with file:line evidence. **All 8 HOLD.**
  - INV-1 (`ai/` unchanged) — `git diff` confirms zero edits.
  - INV-2 (CECE `InferenceFn` injection) — `cece/catalogers/reasoning.rs` untouched.
  - INV-3 (no heavy deps) — only `async-trait` proc macro added.
  - INV-4 (no schema changes) — no DB-touching files diffed.
  - INV-5 (no boot regression) — `mind/` registers IPCs but spawns no startup task.
  - INV-6 (no hot-path additions) — `src/` frontend zero diff.
  - INV-7 (local-first / no exfiltration) — zero `reqwest`/`HttpClient` use in `mind/`.
  - INV-8 (trait admits Phase 0b+) — all 7 named future implementors (real `LocalProvider`, `RoutedProvider`, real `CloudProvider`, `LocalEmbeddingProvider`, `framing::as_tool_result` real, `max_tool_rounds_per_turn`, `Approval::Rejected{reason, scope}`) are structurally admitted by the trait surface.

- **Agent 4B — Drift Check.** 8 specific drift vectors examined. **All CLEAN.**
  - `invoke_handler!` ordering: two new entries inserted between `ai::ai_list_models` and `libraries::list_libraries`; commas balanced; no name collisions.
  - Symbol collision: `LocalProvider`/`CloudProvider`/`OfflineProvider`/`ChatOrchestrator`/`InferenceProvider`/`EmbeddingProvider`/`StreamEvent`/`TelemetryCounters`/`TelemetrySnapshot`/`framing` not used by any other module.
  - `mod.rs` re-exports: all `pub use` paths resolve.
  - `async-trait` adoption: scoped to `mind/`; doesn't conflict with `#[tauri::command] async fn`.
  - `tauri::ipc::Channel` first usage: import path correct (`tauri::ipc::Channel`); `StreamEvent: Serialize` derive present; channel-close lifecycle matches.
  - `telemetry::global()` lifecycle: `OnceLock<Arc<TelemetryCounters>>` doesn't conflict with the 23 existing `OnceLock` statics elsewhere.
  - Frontend impact: `git diff e2df4a69..HEAD -- src/` empty.
  - Cross-platform: `async-trait` has no platform constraints (unlike `memmap2`).

- **Agent 4C — Migration Path.** 6 scenarios from MIG-046 §6 walked end-to-end. **All PASS.**
  - S1 (fresh install): `mind/` dormant; counters initialize at zero; no model download.
  - S2 (existing universe): zero schema diff; boot path untouched.
  - S3 (mid-turn cancel): `commands.rs:73` graceful `if on_event.send(ev).is_err() { break; }`; spawned task observes Receiver close; no writes to roll back.
  - S4 (rollback): `rm -rf src-tauri/src/mind/` + remove `mod mind;` + remove two `invoke_handler!` lines + drop `async-trait`; nothing outside `mind/` uses `async_trait`, so safe.
  - S5 (Cargo.toml diff): in the §B-§E range only `async-trait` line is the in-scope addition (already from §A; was visible to 4B at `Cargo.toml:47`).
  - S6 (test isolation): every telemetry-touching test uses `.with_counters(Arc::new(TelemetryCounters::new()))` — zero direct writes to `telemetry::global()` from tests.

**Net audit conclusion:** Phase 0a Steps A–E land cleanly. No code fixes needed. Step G can proceed to PCS gate.

---

## 3. Phase 0a closure (§G)

Six Build commits all green, audit clean, trait surface locked. The `src-tauri/src/mind/` module is real and consumable; the strategic moat the Concept Paper described is in place.

### What's in place after Phase 0a

- **Trait surface** (`mind/provider.rs`) — `InferenceProvider` + `EmbeddingProvider` split per MA-1, ready to admit real `mistral.rs` / `llama-cpp-2` (Phase 0b), `RoutedProvider` (Phase 2.5), real `CloudProvider` (Phase 5), `LocalEmbeddingProvider` wrapping `embeddings.rs` (Phase 0b/1).
- **Three stub providers** (`mind/providers/`) — `LocalProvider`, `CloudProvider`, `OfflineProvider`, all deterministic, all exercise the trait surface end-to-end with 10 unit tests in `mind::providers::*`.
- **Tauri IPC contract** (`mind/commands.rs`) — `mind_start_turn(request, Channel<StreamEvent>)` + `mind_telemetry_snapshot()`. Wired in `lib.rs` alongside untouched `ai::*`. Phase 1 refactors `mind_start_turn` to use the orchestrator.
- **ChatOrchestrator skeleton** (`mind/orchestrator.rs`) — `turn()` honors Pattern B (generate-restart) outer loop, MA-4 tool-call budget abort, MA-5 framing-hook placeholder. 4 integration tests exercise the full call graph.
- **Telemetry atomics** (`mind/telemetry.rs`) — `TelemetryCounters` per-instance or via `telemetry::global()` for the IPC. Test isolation via `.with_counters()`. 2 telemetry unit tests + 2 orchestrator-via-counters tests.
- **18/18 tests pass** in the `mind::` module.

### Phase 0a → Phase 0b handoff

Phase 0b (MIG-047) opens with these 0a deliverables in hand. The first 0b tasks:
1. **One-day micro-bench** — `mistral.rs` vs `llama-cpp-2` (Plan §1 Decision #4) on Q4_K_M Fanar / Jais on dev hardware.
2. **PF-1 §10.4 open-question resolution** with Eisa (Gemma notices? Fanar GGUF source? Jais gate? attribution placement?) — these affect 0b's model-install flow and bundled-default identity.
3. **Real `LocalProvider`** — swap the §B stub for the chosen runtime, wired against the trait surface unchanged from §A.
4. **Tool-use reliability benchmark** — 50 representative prompts × Fanar vs Jais × measured success rate on the eventual tool palette schemas (`create_note`, `link_notes`, `search_notes`).
5. **Bench report** + bundled-default recommendation for Eisa's lock-in.
6. **First-launch download flow** + model picker UI (frontend).

Phase 0b is the **first Boss-testable phase**. Plan §4: "Open chat, type `مرحبا، كيف حالك؟`, receive coherent Arabic response within 5s on standard laptop."

### PCS gate

All Phase 0a commits (`29459ed0..` current Step G) live on local `main` only. `origin/ConstellationMain` is 1612 commits behind. Push requires Eisa's explicit go.

Eisa decision queue:
1. **PCS green to push** Phase 0a bundle? (single-PR or push-direct-to-main per project convention.)
2. **PF-1 §10.4 open questions** — settle before MIG-047 Architect.
3. **Next session focus** — proceed straight to MIG-047 (Phase 0b Architect) or pause to address the four 10.4 questions first?

No Boss-test required for Phase 0a (Plan §4: "No Boss test yet"). The 18 unit tests are the verification surface.

---

## 4. PF-1 §10.4 closed — four locks landed in Plan §10.5

After the Phase 0a push, Eisa answered the four open questions from Plan §10.4. The Plan now carries the locked decisions as `§10.5 Locked decisions (PF-1 close + Eisa go, 2026-05-24)`:

| # | Question | Lock |
|---|---|---|
| Q1 | Gemma upstream | **Defensive Gemma notices too.** Combined "Model notices" panel ships Apache + Gemma Terms + Gemma Prohibited Use + Fanar BibTeX. |
| Q2 | Fanar GGUF source | **In-house quantization** from official QCRI safetensors via release pipeline. |
| Q3 | Jais HF gate | **Constellation-hosted mirror** of `Jais-2-8B-Chat-GGUF`. **Unblocks Jais as co-default** — Plan §1 Decision #1 (hot-swap + Performance Mode) now applies to BOTH models from first install. |
| Q4 | Attribution placement | **Settings → About panel.** |

**Q3 is the consequential one.** My original recommendation was "(a) drop from co-default for v1, revisit (c) after talking to Inception." Eisa went straight to (c). This changes the Phase 0b → 2.5 trajectory: both models become real on first install rather than one bundled + one user-installable. RoutedProvider value lands sooner; bench discipline at Phase 0b must measure BOTH models from the start; mirror infrastructure (hosting endpoint, refresh cadence, Apache-2.0 notice file) becomes a MIG-047 land item.

### Cascading work added to MIG-047 (Phase 0b)

The Phase 0b Architect (when written) must add these items beyond the originally-scoped runtime micro-bench + tool-use benchmark + first-launch download flow:

- **In-house Fanar Q4_K_M quantization** — release-pipeline step that runs against `QCRI/Fanar-1-9B-Instruct` safetensors and emits the GGUF Constellation distributes.
- **Jais GGUF mirror** — release-pipeline step that fetches `inceptionai/Jais-2-8B-Chat-GGUF` (Q4_K_M.gguf, 4.8 GiB) via Inception-authenticated step, re-hosts on a Constellation-controlled endpoint with Apache-2.0 notices traveling (LICENSE + citation + "redistributed under Apache-2.0 from inceptionai/Jais-2-8B-Chat-GGUF").
- **Hosting-endpoint choice** — GitHub Releases (lowest-friction; already paid for; LFS or release assets handle 4.8 GiB) vs S3 vs Cloudflare R2 vs custom CDN. Each has cost / latency / DX tradeoffs. **Decision deferred to MIG-047 Architect.**
- **Combined Model Notices block** for Settings → About (lands in MIG-048 Phase 1 frontend, but the Notices text is finalized in MIG-047 once the mirror endpoint is chosen).

### Session close

Phase 0a SHIPPED and pushed. PF-1 §10.4 closed. Next session opens with MIG-047 (Phase 0b Architect) — and now has four extra deliverables baked in that the original Architect outline (`docs/MIG-046-...-ARCHITECT.md` §8 "What Phase 0a explicitly does not decide") flagged as Phase 0b territory.

---

## 5. MIG-047 Phase 0b — Architect through Audit (continuation, same day)

The Phase 0b cascade ran in this same session per Eisa's "Proceed" + "Path A" greenlight.

### Architect (commit `e1c8c4d4`)
- `docs/MIG-047-constellation-mind-phase0b-real-inference-ARCHITECT.md` (298 lines).
- Surfaced the architectural surprise: `mistral.rs` does NOT list Jais support; `llama-cpp-2` inherits Jais via upstream llama.cpp but carries the CECE V3-§7 deferred Windows-MSVC cmake risk. Four design options laid out for Eisa.
- Eisa's four locks:
  - **§4 A:** A4 — `mistralrs` for Fanar in 0b; `llama-cpp-2` added in Phase 2.5 for Jais. Jais is KEPT (per §10.5 Q3 intent), just staged.
  - **§4 D:** "No cloud service at all. Local-first." — overrides §10.5 Q3's "Constellation-hosted mirror"; falls back to **GitHub Releases with file-splitting**. Plan §10.5 Q3 row updated with the Override Note (commit `e1c8c4d4`).
  - **§4 F:** Bench (Step H) runs AFTER real LocalProvider lands (Step G).
  - **Quant for v1:** Q4_K_M only.

### Build cascade Steps A → H (six commits)

- **§A** (`a6e35b5a`) — `.github/workflows/model-pipeline.yml` (on-demand workflow: download QCRI safetensors → `convert_hf_to_gguf.py` → `llama-quantize` → `split -b 1700M` → SHA-256 sidecars → assemble `manifest.json` + `LICENSE.txt` → publish to GH Release `models/<id>-v1`). `src-tauri/resources/models.json` (bundled catalog, placeholder SHA-256 until first workflow run).
- **§D** (`634327fd`) — `mind/providers/local_embedding.rs` wrapping the existing `embeddings.rs` ONNX pipeline (multilingual-e5-small, 384-dim). Zero new ONNX session; HMSE retrieval unaffected. 1 new test; 19/19 mind tests pass.
- **§E** (`f17a4459`) — `mind/model_install/{mod,manifest,download,verify,registry,commands}.rs`. Five new IPC commands: `mind_install_model` / `mind_list_catalog` / `mind_list_installed_models` / `mind_active_model` / `mind_set_active_model`. Chunked download + SHA-256 verify + Arc-cloned forwarder. `sha2 = "0.10"` + `hex = "0.4"` added to Cargo.toml. 14 new tests; 33/33 mind tests pass.
- **§F** (`39fa7258`) — `src/lib/components/MindSettings.svelte` (Svelte 5 component) wired into `SettingsModal.svelte` as new sidebar entry between Intelligence and Security. EN + AR i18n keys added; 13 other locales fall back via `||` pattern (will land alongside Phase 1).
- **§C** (`54c49c43`) — `mind/providers/local.rs` (real `LocalProvider` wrapping `mistralrs 0.8.1`); existing stub moved to `local_stub.rs` via `git mv`. Used a parallel research agent to read the actual `mistralrs` source on GitHub (docs.rs build was broken at fetch time) — confirmed `GgufModelBuilder` API, `Stream<'_>` borrow lifetime, OpenAI-style tool schema, engine-reboot mitigation pattern (mistralrs #2147). `mistralrs = "0.8.1"` + `futures = "0.3"` added. 5 new tests; 38/38 mind tests pass. First-time cargo build with mistralrs: 5m18s.
- **§G** (`62a5a842`) — `mind_start_turn` now loads the user's active model from `mind::model_install::registry`, instantiates real `LocalProvider`, drives a real turn through `mistralrs`. If no active model, emits `StreamEvent::Error` pointing to Settings → Mind.
- **§B + §H** (`c6e075b7`) — Two `[[bin]]` targets in `src-tauri/build_assets/`: `bench_runtime` (model-load + first-token + sustained tokens/sec across 3 prompts; includes Arabic greeting for Boss-test Stage 0 verdict) and `bench_tool_use` (10 starter prompts × 4-tool palette × three pass/fail axes: tool-call validity, argument fidelity, coherent reply). Both compile clean; running needs the Fanar GGUF on disk. `pub mod mind;` widened from `mod mind;` so the bins can reach `constellation_lib::mind::providers::LocalProvider`.

### §I — 3-agent audit summary

Three parallel Explore agents (4A invariants / 4B drift / 4C migration-path) audited the full `e1c8c4d4..HEAD` range.

**4A Invariants — 9 hold, 1 flagged-then-resolved as false positive, 1 cannot-determine-deferred.**
- All 10 from MIG-047 §3 hold EXCEPT:
- **INV-8 FALSE POSITIVE — RESOLVED:** Agent flagged "Tauri HTTP capability missing for `mind_install_model`'s reqwest calls." This is incorrect — backend Rust `reqwest::Client::new()` calls (like the 5 in `src-tauri/src/ai/mod.rs:87,131,177,252,334` for OpenAI/Anthropic/Gemini/Ollama) do NOT need Tauri capability scoping. Capability scoping (`http:default` etc.) applies to the JS-side `@tauri-apps/api/http` plugin in the webview; native backend Rust HTTP is trusted code with full network access. The existing `ai/mod.rs` pattern has been in production without HTTP capability and works fine. The `mind/` outbound-HTTP invariant is satisfied by code-review confirming we only fetch FROM the model URLs (no PUT/POST/exfiltration); no Tauri capability change needed. **No code fix required.**
- **INV-7 cannot-determine:** `docs/models/MODEL-NOTICES.md` (named in Architect §2.3) was deferred — the per-release `LICENSE.txt` (which the workflow assembles inline, see model-pipeline.yml lines 174-243) carries the canonical Apache + Gemma + BibTeX text. Phase 1 (MIG-048) reads from the LICENSE.txt to populate Settings → About — at that point, a separate `MODEL-NOTICES.md` doc may or may not be needed. Deferring the decision is harmless in 0b.

**4B Drift — CLEAN.** 10 drift vectors all clean. Only LOW finding: 13 locales not yet translated for the new `settings.mind.*` keys (known, has `||` fallback pattern, will land alongside Phase 1's chat surface).

**4C Migration path — 7 PASS + 1 NEEDS-FIX (deferred).**
- S1 (fresh install, no model) PASS
- S2 (install Fanar from picker) PASS
- S3 (install both models) PASS (structurally; Jais is 2.5)
- S4 (mid-download interrupt) PASS
- S5 (corrupted cache on load) NEEDS-FIX: the runtime path detects corruption (mistralrs load fails → InferenceError → StreamEvent::Error in chat), but the Settings → Mind UI doesn't proactively re-verify SHA-256 on startup and show "Re-install" affordance. Phase 1 polish; out of 0b scope per Architect §8.
- S6 (rollback to MIG-046) PASS
- S7 (existing 7,600-note universe) PASS — no boot regression
- S8 (Cargo.toml diff bounded) PASS — only `mistralrs` + `futures` + `sha2` + `hex` added

**Net audit verdict:** Phase 0b code is ready for Boss-test Stage 0. No code fixes surfaced. Two follow-up items recorded for Phase 1 / MIG-048 (S5 corrupted-cache UX + 13-locale i18n translation).

### §J — open (awaits Boss-test Stage 0)

Pending Eisa actions for §J completion:
1. **Trigger model-pipeline workflow** at https://github.com/eisaShamsi/Constellation/actions → "Model Pipeline (Constellation Mind)" → Run with `model=fanar-1-9b-q4km, version=v1`. Expected: ~45-60 min on ubuntu-latest.
2. **Send the final SHA-256** from workflow output (logged in the "Compute final SHA-256 + size BEFORE splitting" step). Small commit updates `models.json::final_sha256`.
3. **Install Fanar** via the running app's Settings → Mind → Install button (after a fresh `cargo build` to pick up MIG-047).
4. **Run bench_runtime** + paste output into `lab/reports/MIG-047-bench-runtime-2026-MM-DD.md`.
5. **Run bench_tool_use** + paste output into `lab/reports/MIG-047-bench-tool-use-2026-MM-DD.md`.
6. **Confirm Boss-test Stage 0** — open chat (when Phase 1 ships) OR invoke `mind_start_turn` from a dev script with "مرحبا، كيف حالك؟" → coherent Arabic response within 5s.

Once 1-6 close, §J ships: orientation v2.32 bump documenting Phase 0b shipped + bundled-default lock (Fanar) + bench results; MoCh entry; first MIG-047 help-doc topic in all 15 locales (`docs/help.*/Constellation Mind/`); PCS gate.
