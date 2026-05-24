# MIG-047 — Constellation Mind, Phase 0b: Real Local Inference + Tool-Use Bench + Mirror Infrastructure

**Status:** Architect (Phase 1 of the `/migration` workflow). Awaiting Eisa approval of this doc before the Plan is drafted.
**Date:** 2026-05-24
**Lineage:**
- Phase 0b of the Constellation Mind Implementation Plan v1.0 (Plan §4 Phase 0b + §10.5 locked decisions).
- Builds on MIG-046 Phase 0a (`docs/MIG-046-constellation-mind-phase0a-inference-abstraction-ARCHITECT.md` — trait surface + stubs + IPC + orchestrator + telemetry, all shipped at `1d91e5cd`).
- Inherits four cascading deliverables from PF-1 §10.5: defensive Gemma notices (Q1), in-house Fanar quantization (Q2), Constellation-hosted Jais GGUF mirror (Q3), Settings → About attribution placement (Q4).

---

## 1. Goal

Phase 0b swaps Phase 0a's stub providers for real local inference and **delivers the first Boss-testable Constellation Mind gate** (Plan §4: "Open chat, type `مرحبا، كيف حالك؟`, receive coherent Arabic response within 5s on standard laptop").

Five distinct deliverables compose Phase 0b:

1. **Runtime choice + real `LocalProvider`** — pick the inference runtime (`mistral.rs` vs `llama-cpp-2`) via a measured micro-bench, then ship a real `InferenceProvider` impl wrapping the chosen runtime, replacing the §B Phase 0a stub.
2. **Real `LocalEmbeddingProvider`** — wrap the existing `src-tauri/src/embeddings.rs` ONNX session as an `EmbeddingProvider` impl, so the new trait surface gets a real backing for vector retrieval.
3. **Tool-use reliability benchmark** — 50 representative prompts × {Fanar-1-9B-Q4_K_M, Jais-2-8B-Q4_K_M} × measured success rate on Concept Paper §8 tool palette schemas (read tools: `search_notes`, `read_note`, `find_similar`, `list_recent`). Output: bench report + bundled-default recommendation.
4. **Distribution infrastructure** (NEW per §10.5 Q2 + Eisa's "no cloud service" lock at §4 D) — release-pipeline steps that quantize `QCRI/Fanar-1-9B-Instruct` safetensors → Q4_K_M GGUF in-house, split into ~1.7 GiB chunks via POSIX `split`, publish to a dedicated GitHub Release tagged `models/fanar-q4km-v1` with a manifest JSON + SHA-256 sidecars + Apache-2.0 + Gemma LICENSE file. Phase 2.5 adds the equivalent for Jais (mirror infra is unchanged; same workflow with different inputs).
5. **Model installation UX** — first-launch download flow + Settings → Mind model picker, fetching from the mirror endpoint with size disclosure and progress UI.

What Phase 0b does **NOT** do (those are later phases):

- No frontend chat surface (Phase 1 / MIG-048).
- No real retrieval wiring through to the orchestrator (Phase 1 / MIG-048 — the orchestrator still uses canned dispatcher in 0b).
- No write tools, no approval modal (Phase 2 / MIG-049).
- No `RoutedProvider` (Phase 2.5 / MIG-050 — both models are *installable* in 0b, but the orchestrator still uses one at a time).
- No real `Settings → About` panel display (Phase 1 / MIG-048 — but the canonical Model Notices text is finalized HERE so MIG-048 can paste it).
- No cost telemetry / Cloud opt-in (Phase 5 / MIG-053).

---

## 2. Territory map (verified against current code, 2026-05-24)

### 2.1 What MIG-046 left in place that Phase 0b extends

| Surface | Status after MIG-046 | Phase 0b treatment |
|---|---|---|
| `src-tauri/src/mind/provider.rs` — `InferenceProvider` + `EmbeddingProvider` traits | Stable, frozen | **Unchanged.** Phase 0b just adds new implementations. |
| `src-tauri/src/mind/providers/local.rs` — `LocalProvider` stub | Deterministic event-sequence stub | **Replaced** by a real impl wrapping the chosen runtime. The stub moves to `src-tauri/src/mind/providers/local_stub.rs` (for unit testing that doesn't load a model). |
| `src-tauri/src/mind/providers/cloud.rs` — `CloudProvider` Anthropic-shaped scaffold | Stub | **Unchanged in 0b.** Phase 5 (MIG-053) makes it real. |
| `src-tauri/src/mind/providers/offline.rs` — `OfflineProvider` | Stub | **Unchanged.** Stays as the "no model configured" safe fallback. |
| `src-tauri/src/mind/commands.rs` — `mind_start_turn` IPC | Hardcoded LocalProvider stub | **Updated** to instantiate the real `LocalProvider` against the user's currently-installed model (read from Settings). Still does NOT use the orchestrator (that's Phase 1's refactor). |
| `src-tauri/src/mind/orchestrator.rs` — `ChatOrchestrator` skeleton | Tested with stubs | **Unchanged in 0b.** Phase 1 swaps the canned `ToolDispatcher` for real read tools. |
| `src-tauri/src/mind/telemetry.rs` — counters | Wired into orchestrator's `turn()` | **Unchanged.** Phase 1 plumbs them into `mind_start_turn`. |
| `src-tauri/src/embeddings.rs` — existing `ort` + `multilingual-e5-small` ONNX pipeline | Drives HMSE semantic retrieval | **Wrapped** by a new `mind/providers/local_embedding.rs::LocalEmbeddingProvider` impl. The existing `embeddings.rs` API stays callable directly (HMSE retrieval doesn't change paths). |

### 2.2 What's in place outside `mind/` that Phase 0b touches

| Surface | File / Location | Phase 0b treatment |
|---|---|---|
| Release CI pipeline | `.github/workflows/release.yml` — single Windows-latest job, uses `tauri-apps/tauri-action@v0`, signs with `TAURI_SIGNING_PRIVATE_KEY`, publishes to GitHub Releases + updates a Gist with `latest.json` for the updater. | **Untouched.** Phase 0b adds a *separate* workflow `.github/workflows/model-pipeline.yml` that runs on-demand or on a model-version tag (`models/v*`), produces the GGUFs, uploads to the mirror endpoint. The main `release.yml` doesn't change. |
| Tauri updater (Gist-backed) | `latest.json` on `gist.github.com/d78713095f2bddb26698c9f04a79b2d8` | **Untouched.** App-version updates stay on this path; model-version updates are a parallel mechanism (model URL + SHA-256 in a `models.json` manifest in the repo). |
| `tauri.conf.json` (resource_dir / capabilities) | `src-tauri/tauri.conf.json` | **Extended** with HTTP-fetch capability scoped to the mirror endpoint domain so Tauri's HTTP client can download models. (The existing `protocol-asset` feature stays.) |
| Settings UI infrastructure | `src/lib/components/Settings/*` (existing settings sections per topic) | **Extended** with a new "Mind" section (model picker, download UI, model status). UX lands in Phase 1 polish; 0b ships the functional minimum so the install flow works. |

### 2.3 What's genuinely new in Phase 0b

A new `mind/providers/` family:
```
src-tauri/src/mind/providers/
├── local.rs                # REAL — wraps the chosen runtime; replaces §B stub
├── local_stub.rs           # MOVED from local.rs — unit-test stub
├── local_embedding.rs      # NEW — wraps embeddings.rs's ort session
├── cloud.rs                # unchanged (Phase 5 makes it real)
└── offline.rs              # unchanged
```

A new `mind/model_install/` family for the first-launch download UX (Rust side):
```
src-tauri/src/mind/model_install/
├── mod.rs                  # public API (install_model, list_available, …)
├── manifest.rs             # parse models.json (URLs + SHA-256 per model)
├── download.rs             # streamed HTTP fetch with progress events
├── verify.rs               # SHA-256 + size verification
└── registry.rs             # per-user installed-model registry (in Library settings)
```

A new release-pipeline workflow:
```
.github/workflows/model-pipeline.yml
```

A new in-repo manifest:
```
docs/models.json             # canonical list of installable models (URLs, SHA-256, sizes, licenses)
docs/models/MODEL-NOTICES.md # the combined Model Notices text (consumed by Phase 1 Settings → About)
```

A new GitHub Release per model (no cloud infrastructure operated by Constellation):
```
https://github.com/eisaShamsi/Constellation/releases/tag/models/fanar-q4km-v1
├── fanar-1-9b-q4km.gguf.part-aa     # ~1.7 GiB chunk 1
├── fanar-1-9b-q4km.gguf.part-ab     # ~1.7 GiB chunk 2
├── fanar-1-9b-q4km.gguf.part-ac     # ~1.6 GiB chunk 3
├── manifest.json                    # parts list + per-part SHA-256 + final SHA-256
└── LICENSE.txt                      # Apache-2.0 + Gemma Terms + BibTeX

(Phase 2.5 adds:)
https://github.com/eisaShamsi/Constellation/releases/tag/models/jais-2-8b-chat-q4km-v1
├── jais-2-8b-chat-q4km.gguf.part-aa
├── ... (split per the same scheme)
├── manifest.json
└── LICENSE.txt                      # Apache-2.0 + redistribution notice + Jais citation
```

---

## 3. Invariants that MUST NOT break

1. **Phase 0a trait surface is frozen.** No edits to `provider.rs` or `events.rs`. All new implementations consume the existing traits unchanged.
2. **`mind/` IPC contract is additive.** The two existing commands (`mind_start_turn`, `mind_telemetry_snapshot`) keep their wire shape. New commands (`mind_install_model`, `mind_list_installed_models`, `mind_active_model`, `mind_download_progress`) are added; nothing is removed or renamed.
3. **`ai/` + `cece/` still untouched.** The cloud bridge stays as it is; CECE's `InferenceFn` injection point stays; `embeddings.rs` keeps its existing `pub(crate) run_embedding` callers (HMSE retrieval), and the new `LocalEmbeddingProvider` is an additional consumer, not a replacement.
4. **No regression in the 18 mind:: unit tests** from MIG-046. New tests pile on top. `local_stub.rs` carries the deterministic stubs the existing tests reference.
5. **Boot time on a 7,600-note universe (Eisa's): unchanged.** No model loading on startup — models load lazily on first chat turn. The `ChatOrchestrator` skeleton initialized at startup must NOT instantiate a real provider; the real provider materializes inside `mind_start_turn` only after the user actually invokes a turn.
6. **First-launch experience without a model installed must work.** A fresh install where no model is downloaded yet should: (a) show the model picker / "Install a model" prompt in Settings → Mind, (b) the chat surface (when Phase 1 ships) routes to `OfflineProvider` until a model is installed, (c) installing a model is one click + a download progress bar + a verify step.
7. **Apache-2.0 + Gemma notices travel with every redistributed weight.** The R2 mirror serves `*-LICENSE.txt` alongside each GGUF; the in-repo `docs/models/MODEL-NOTICES.md` carries the canonical text Phase 1 displays.
8. **No outbound HTTP from `mind/` to anywhere except the configured model-mirror endpoint.** Telemetry stays local (invariant 7 of MIG-046). The HTTP-fetch capability in `tauri.conf.json` is scoped narrowly — no general internet access.
9. **CI build time stays bounded.** The model-pipeline workflow runs on-demand, not on every release; the main `release.yml` does NOT pull or quantize models (the GGUFs are pre-uploaded to R2 with stable URLs; app installs fetch on first launch, not on app install).
10. **Windows MSVC toolchain remains green** for `cargo build` on every Constellation dev machine. The runtime-choice decision in §4 A weighs this heavily — adding a cmake/C++ chain has been a CECE V3-§7 deferred risk and must not be silently accepted.

---

## 4. Design options

### A. Inference runtime choice — the architectural pivot

The research (parallel agent A, sources: crates.io, GitHub repos, llama.cpp build docs, Tauri sidecar bugs) returned a tradeoff matrix:

| Criterion | `mistral.rs` 0.8.1 | `llama-cpp-2` 0.1.146 |
|---|---|---|
| Maturity / cadence | Quarterly releases | Weekly, tracks upstream llama.cpp |
| Fanar (Gemma-2-9B) load | **Listed in README** | Upstream proven |
| **Jais-2-8B load** | **NOT listed** | Upstream Jais since b3282; "2-8B" novelty TBM |
| Streaming API | Async + Tokio Stream (ergonomic) | Manual decode loop (caller wraps) |
| Tool calls | **First-class** + JSON-schema enforcement | Raw GBNF grammar plumbing (caller writes Hermes adapter) |
| Embeddings | First-class `EmbeddingModelBuilder` | Low-level only |
| **Windows MSVC build** | **Pure Rust, no cmake** | cmake + C++ chain (CECE V3-§7 deferred risk) |
| Mobile (iOS/Android) | No support documented | Possible via NDK/XCFramework, no Rust-side recipe |
| License | MIT | MIT/Apache-2.0 |
| GPU breadth | CUDA + Metal | CUDA + Metal + Vulkan + ROCm + DirectML |

**The architectural surprise:** `mistral.rs` does NOT list Jais among its supported architectures. `llama-cpp-2` inherits Jais via upstream llama.cpp. **This affects Plan §10.5 Q3 directly** — if we pick `mistral.rs` and it can't load Jais, the §10.5 Q3 mirror infrastructure ships Jais bytes that Constellation can't actually run for v1. Three options follow:

- **A1 — `mistral.rs` ALONE, drop Jais from v1.** Reverts Q3 to "Jais user-installable for v1 once mistral.rs adds support." Q3's mirror still gets built, but Jais isn't downloaded by default; the §10.5 Q3 "co-default" status is deferred to Phase 2.5 when mistral.rs adds Jais OR we swap to llama-cpp-2 then. Trade: cleanest Windows-MSVC story now, but the bolder Phase 1 trajectory (both models from day one) is delayed.
- **A2 — `llama-cpp-2` ALONE, accept the cmake risk.** Both models work day one (proven upstream). CECE V3-§7's Windows-toolchain risk lands now rather than deferred. Mitigation: invest one Plan step in a Windows-MSVC CI matrix to catch build breaks early; pin `llama-cpp-2` to a specific upstream llama.cpp commit hash so weekly upstream churn doesn't surprise us. Trade: heavier build, but matches Eisa's §10.5 Q3 intent directly.
- **A3 — BOTH, runtime-selectable per model.** A trait extension `LocalProvider::with_runtime(Runtime::MistralRs | Runtime::LlamaCpp2)` lets the user pick. Fanar can use either; Jais routes to llama-cpp-2. Trade: doubles dep weight + maintenance + test surface. Likely overkill for v1.
- **A4 — Phase the choice in: `mistral.rs` for Fanar in 0b, add `llama-cpp-2` for Jais in Phase 2.5.** The micro-bench in §5 Step B measures Fanar performance on both runtimes to validate the choice; Jais arrives with RoutedProvider. Trade: two-stage; Phase 2.5 carries the cmake risk introduction instead of 0b.

**Recommendation: A4** — keeps 0b Windows-MSVC-friendly via `mistral.rs`, validates Eisa's §10.5 Q3 lock via Phase 2.5 once routed-provider scaffolding is real. But Eisa picks; A2 is the bolder match to his intent and worth choosing if the cmake-on-Windows pain is judged acceptable now rather than deferred. **A1 is rejected** unless we want to walk back §10.5 Q3 — which I don't recommend.

### B. `LocalEmbeddingProvider` implementation

- **B1 — Wrap `embeddings.rs`'s existing `EmbeddingEngine` (CHOSEN).** New file `mind/providers/local_embedding.rs` holds a `LocalEmbeddingProvider` that borrows the existing `EmbeddingState` via Tauri State, calls `run_embedding` for each input text, and returns the resulting f32 vectors. `embed_capabilities` returns `{model_id: "multilingual-e5-small", dimension: 384, max_input_tokens: 512}` — already what `embeddings.rs` advertises. Zero duplication, zero new ONNX session. The HMSE retrieval path keeps calling `embeddings::run_embedding` directly; the new `LocalEmbeddingProvider` is a parallel consumer.
- B2 — Have `mistral.rs` (if A1/A3/A4) own the embeddings via its `EmbeddingModelBuilder`. Rejected: we already have a working embedding pipeline; adding a second one is dep + perf cost for no gain.

### C. Quantization strategy (Fanar) — per §10.5 Q2 lock

- **C1 — In-house GitHub Actions workflow runs `quantize` from llama.cpp prebuilt binaries (CHOSEN).** The model-pipeline workflow downloads `QCRI/Fanar-1-9B-Instruct` safetensors (via the public HF API; no auth needed for Fanar), runs `convert_hf_to_gguf.py` + `quantize ... q4_k_m`, uploads the resulting `fanar-1-9b-q4km.gguf` to the mirror endpoint along with a SHA-256 sidecar. The workflow runs **on-demand** (workflow_dispatch) or on a model-version tag (`models/fanar-1-9b-v1`).
- C2 — Quantize on developer machine, commit the GGUF to the repo. Rejected: ~5 GiB binary in git is wrong; rebuild-from-scratch determinism lost.
- C3 — Use `mradermacher/Fanar-1-9B-i1-GGUF` community quant. Rejected per Eisa's §10.5 Q2 lock — in-house quantization.

### D. Distribution endpoint — per Eisa's "No cloud service at all" lock (overrides §10.5 Q3)

Eisa's lock (2026-05-24, after this Architect was first presented): **"No cloud service at all. Local-first."** This explicitly overrides §10.5 Q3's "Constellation-hosted mirror" — Constellation operates zero cloud infrastructure for model distribution.

Per the parallel agent B's research, with the "no Constellation-operated cloud" filter applied:

| Option | Constellation operates cloud? | Per-file limit | Cost | Honors lock? |
|---|---|---|---|---|
| ~~Cloudflare R2~~ | **Yes** (operated bucket) | none | $0-2/yr | **NO** |
| ~~AWS S3+CloudFront~~ | **Yes** | none | ~$870/yr | **NO** |
| ~~Constellation-owned HF Datasets~~ | **Borderline** (we own the HF repo) | 200 GiB | $0 | NO (still adds a vendor) |
| **GitHub Releases (split files)** | **No** (same infrastructure as the installer itself) | 2 GiB per file → split | $0 | **YES** |
| Bundle in installer | No | n/a | n/a (size) | NO — installer hits the same 2 GiB GH Releases limit |
| User manual install | No | n/a | n/a | YES but UX-hostile |

- **D1 — GitHub Releases with file-splitting (CHOSEN).** The repo already publishes the installer via GitHub Releases (`release.yml`); model assets become *separate* Releases tagged `models/<name>-q4km-v1` (decoupled from app version tags like `v1.0.0`). The 2 GiB per-file limit is handled by splitting each GGUF into ~1.7 GiB chunks via POSIX `split` (`split -b 1700M file.gguf file.gguf.part-`), uploading parts + a manifest JSON + a SHA-256 sidecar + a LICENSE file. Tauri downloader fetches the manifest, downloads each part with progress events, concatenates to the final file, verifies the final SHA-256. Pure bytes, no archive format, deterministic. **Zero new vendor relationships; zero new cloud infrastructure; the user downloads from the same place the app itself came from.**

- D2 — User manual install. Rejected for v1: bad UX. Worth keeping as a future "advanced users" path (drop a `.gguf` into a known folder and Constellation picks it up).

**The §10.5 Q3 lock is recorded as overridden by this newer §4 D lock** — see Plan §10.5 update in the same commit as this Architect.

**Tauri-side implementation cost:** ~200 lines of Rust for the chunked-download + verify + concatenate flow in `mind/model_install/download.rs`. Failure modes (partial chunk, network drop mid-chunk, hash mismatch on a part, hash mismatch on the concatenated whole) all have well-defined recovery paths: discard partial, retry the chunk that failed, or restart the whole install. The chunking is transparent to the user — they see one "Installing Fanar" progress bar reflecting bytes-fetched-so-far across all chunks.

**No subdomain decision needed.** URLs are GitHub-native: `https://github.com/eisaShamsi/Constellation/releases/download/models/fanar-q4km-v1/fanar-1-9b-q4km.gguf.part-aa` etc. `tauri.conf.json` scopes HTTP-fetch capability to `https://github.com` + `https://objects.githubusercontent.com` (where the actual release-asset bytes are served from GitHub's CDN).

### E. Model installation UX

- **E1 — Settings → Mind → Models section with picker + download button (CHOSEN).** Lists each installable model from `docs/models.json` with: name, size, license summary, "Install" button. Clicking Install opens a Tauri-side modal with progress bar (% + MiB/total) backed by `mind_download_progress` event stream. Verify hash on completion; mark as installed in per-user registry. **No first-launch wizard**: Settings is where models live; the user discovers them when they want chat. The chat surface (Phase 1) shows "No model installed — open Settings → Mind to install one" if none present.
- E2 — First-launch onboarding wizard that forces a model choice. Rejected: hostile to users who don't want LLM features. Local-first means LLM is opt-in, not push.
- E3 — Background auto-download Fanar on first launch silently. Rejected: 5 GiB silent download violates user consent (Plan §1 Decision #3 — "first-launch download with size disclosure").

### F. Bench harness — the 50-prompt tool-use reliability benchmark

- **F1 — Rust-side bench harness as a separate `[[bin]]` target (CHOSEN).** `src-tauri/build_assets/bench_tool_use.rs` (alongside the existing `build_concept_vectors` bin from MIG-013 §1A). Loads each runtime+model combination, plays 50 fixed prompts through `provider.generate()` with a `params.tools` palette matching the Phase 1 tool list (`search_notes`, `read_note`, `find_similar`, `list_recent`), scores each result on three axes:
  - **Tool-call validity** — did the model emit a structurally-valid `ToolCall` with parseable JSON args for the right tool name?
  - **Argument fidelity** — do the args match a manually-graded "correct" args structure for the prompt?
  - **Coherent reply after tool result** — does the round-2 response synthesize the (canned) tool result meaningfully?
  Outputs a CSV + markdown report (`lab/reports/MIG-047-bench-tool-use-{date}.md`) per Eisa-reviewable bench discipline.
- F2 — Hand-graded ad-hoc bench. Rejected: not reproducible.
- F3 — Reuse an existing benchmark suite (Helm Arabic / ArabicMMLU). Rejected: those measure knowledge, not tool-use. Tool-use is what we actually need to predict.

**The 50 prompts list** is finalized in the Plan (not the Architect). Mix: 20 read-only (search + read), 10 conceptually-write-but-receive-canned-rejection (since 0b has no real writes), 10 multi-step (model needs to chain 2-3 tool calls), 10 Arabic-language equivalents of the English prompts (the actual delivery is Arabic-first).

---

## 5. Plan outline (each step = one commit + verification clause)

> **Step A — Model-pipeline workflow + first GitHub Release for Fanar (no Rust code yet).**
> Write `.github/workflows/model-pipeline.yml` with one job `quantize-fanar`: download `QCRI/Fanar-1-9B-Instruct` safetensors (no auth needed; public HF repo), run `convert_hf_to_gguf.py` + `quantize ... q4_k_m` from llama.cpp prebuilt tools, split the resulting GGUF with `split -b 1700M`, compute SHA-256 for each part + the final concatenated whole, emit a `manifest.json` listing parts + hashes + final-size, gather a `LICENSE.txt` (Apache-2.0 + Gemma Terms + Fanar BibTeX), publish all artifacts to a GitHub Release tagged `models/fanar-q4km-v1` via `softprops/action-gh-release`. Workflow trigger: `workflow_dispatch` only (manual; not on every release). Create `src-tauri/resources/models.json` listing the model entries (name, version, release-tag URL pattern, final SHA-256, total size). Phase 2.5 adds a second job `quantize-jais` (HF-token-authenticated; Inception org access needed for the GGUF gate).
> *Verify:* Workflow runs successfully on a manual trigger; all part files downloadable from the GitHub Release URL without auth (curl works); SHA-256 of `cat fanar-1-9b-q4km.gguf.part-* | sha256sum` matches `manifest.json.final_sha256`; LICENSE file served alongside; `models.json` schema-validates.

> **Step B — Runtime micro-bench (one focused day).**
> Per §4 A's chosen runtime path (Eisa's pick). If A1/A4 — add `mistralrs = "0.8"` to Cargo.toml; if A2 — add `llama-cpp-2 = "0.1"` + verify cmake/C++ chain on the Windows dev machine + adjust GitHub Actions Windows runner if needed; if A3 — both, behind feature flags. Write `src-tauri/build_assets/bench_runtime.rs` that loads `fanar-1-9b-q4km.gguf` (downloaded from the §A mirror to a temp dir) and times: model-load latency, first-token latency, sustained tokens/sec, memory footprint. Run on dev hardware; record results in `lab/reports/MIG-047-bench-runtime-{date}.md`.
> *Verify:* Numbers measured + Eisa-reviewable; build green on Windows MSVC (CECE V3-§7 risk validated or escalated).

> **Step C — Real `LocalProvider` impl.**
> Move existing `src-tauri/src/mind/providers/local.rs` → `local_stub.rs` (kept for tests); write new `local.rs` wrapping the chosen runtime. Implements `InferenceProvider`: real `generate()` streams real tokens, real `classify()` (if needed for Phase 3 future-proofing or stubbed for now), `capabilities()` returns real model_id/runtime/context-window. Existing 10 unit tests on `local_stub.rs` keep passing; new integration test in `local.rs` loads a small test GGUF (cached via §A) and asserts a generation completes.
> *Verify:* `cargo test --lib mind::` — old tests pass; new test runs in <30s on dev hardware; svelte-check unchanged (no frontend).

> **Step D — Real `LocalEmbeddingProvider` impl.**
> New `src-tauri/src/mind/providers/local_embedding.rs`. Wraps `embeddings::EmbeddingState`; implements `EmbeddingProvider`. Unit test confirms a 3-text batch produces 3 × 384-dim vectors. The existing HMSE retrieval path (`embeddings::run_embedding` callers) is **unchanged** — the new provider is an additional consumer.
> *Verify:* Test passes; HMSE search still works on Eisa's universe (smoke test); `cargo test --lib` green.

> **Step E — Mind model-install Rust side.**
> New `src-tauri/src/mind/model_install/`. Implements: `mind_install_model(model_id, on_progress: Channel<DownloadProgress>)`, `mind_list_installed_models()`, `mind_active_model()`, `mind_set_active_model(model_id)`. Reads `docs/models.json` at startup (bundled as Tauri resource); downloads to `app_data_dir/models/`; verifies SHA-256; writes to per-Library settings.
> *Verify:* From a `#[cfg(test)]` test or a one-off dev command, install Fanar from R2 → file lands at the expected path with the right hash + size.

> **Step F — Settings → Mind UI (minimum viable).**
> New `src/lib/components/Settings/MindSettings.svelte`. Lists models from `mind_list_installed_models`, shows install button (calls `mind_install_model`), progress bar (consumes the `Channel<DownloadProgress>`), active-model picker (calls `mind_set_active_model`). RTL-aware; 15-locale i18n added inline.
> *Verify:* Boss-test Stage 1 — open Settings → Mind, click Install Fanar, see progress bar, see "Installed" status when done. (Stage 2 is the actual chat — Step H below.)

> **Step G — `mind_start_turn` refactor (now uses real LocalProvider).**
> Update `src-tauri/src/mind/commands.rs::mind_start_turn` to load the user's active model via the install registry, instantiate the real `LocalProvider`, drive it through one turn. Still bypasses the orchestrator (Phase 1 swaps that).
> *Verify:* Manual IPC test from a throwaway dev script: invoke `mind_start_turn` with "مرحبا، كيف حالك؟" → stream Arabic Token events back. **First Boss-test Stage 0 verification.**

> **Step H — Tool-use reliability benchmark.**
> Write `src-tauri/build_assets/bench_tool_use.rs` per §4 F1. Run the 50-prompt × {Fanar, Jais} matrix (Jais only if §4 A choice supports it). Score the three axes. Output: `lab/reports/MIG-047-bench-tool-use-{date}.md` with per-prompt scores + aggregate pass-rate per model. Recommend bundled-default identity to Eisa.
> *Verify:* Bench runs to completion; report renders; Eisa reviews and locks bundled-default in a Plan §1 Decision #2 closure note.

> **Step I — `/simplify` + 3-agent audit.**
> Same shape as MIG-046 §F. Three parallel agents: (4A) invariants §3 hold; (4B) drift — invoke_handler ordering, new dep landscape, mirror infrastructure not leaking auth tokens; (4C) migration-path — fresh install / model not installed / mid-download interrupt / corrupted-cache recovery / rollback to MIG-046.

> **Step J — SO + 15-locale help + PCS gate.**
> Session log final entry; orientation v-bump documenting Phase 0b shipped + bundled-default decision; **15-locale help additions** — `docs/help.*/Constellation Mind/` topic in all locales (what Mind is, how to install models, the bundled-default, what's coming next); MoCh; PCS gate awaits Eisa's go.

---

## 6. Migration-path matrix

| Scenario | Behavior |
|---|---|
| **Fresh install, user never installs a model** | OfflineProvider returns the "install a model" message on any chat invocation; Settings → Mind shows the model picker; no model files exist on disk. |
| **Fresh install, user installs Fanar from the picker** | Download from R2 progresses with UI feedback; SHA-256 verifies on completion; registered as active model; chat turns now route to real LocalProvider; first-turn latency includes one-time model-load (~2-5s on warm SSD, mmap-backed). |
| **User installs both Fanar AND Jais** | Both files on disk; the active-model selector picks one at a time in 0b (RoutedProvider lands in Phase 2.5); user can switch in Settings. |
| **Mid-download interrupt (network drop, app quit)** | Partial file is discarded on next launch; user can re-trigger install from Settings; download resumes from scratch (range-resume is Phase 1 polish, not 0b). |
| **Corrupted cache (SHA-256 mismatch on load)** | Provider initialization fails with `InferenceError::NotConfigured` + a clear message; Settings → Mind shows "Re-install Fanar"; user clicks → download repeats. |
| **Rollback to MIG-046** | Delete `src-tauri/src/mind/{providers/local.rs (real), providers/local_embedding.rs, model_install/}`, restore `local_stub.rs` → `local.rs`, remove the new commands from `lib.rs::invoke_handler!`, drop the chosen runtime from Cargo.toml, delete `.github/workflows/model-pipeline.yml`. R2 bucket stays (it's the mirror; deletion is operational, not code). Frontend Settings → Mind component deleted. |
| **Existing universe (7,600 notes)** | Boot time unchanged (no model loading on startup); HMSE retrieval unchanged (embeddings.rs untouched); first chat turn pays the model-load cost once per process. |
| **Cargo.toml diff in 0b** | One of: `mistralrs = "0.8"`, OR `llama-cpp-2 = "0.1"`, OR both (per §4 A). Plus possibly `sha2` (for SHA-256 verification) if not already present — verify before adding. |

---

## 7. Risk summary

**Medium.** Phase 0b is genuinely larger than Phase 0a: real models (multi-GB download), real network infrastructure (R2 mirror), real CI pipeline (quantization step), real first-user-visible UX (model picker). Five distinct risks beyond the baseline:

- **R-0b-1: Windows MSVC build risk if §4 A=A2 chosen.** CECE V3-§7's deferred concern materializes. Mitigation: dedicated Windows MSVC CI matrix in Step B; pin `llama-cpp-2` to a specific upstream commit; have a fallback plan (revert to mistral.rs + drop Jais) ready if the build proves intractable.
- **R-0b-2: Jais "custom architecture" novelty.** Even on `llama-cpp-2`, `Jais-2-8B` may diverge from `Jais-13B` enough to require an upstream llama.cpp PR. Mitigation: Step B's micro-bench LOADS Jais as the first test; if load fails, we either upstream a fix or drop Jais from v1 (revert Plan §10.5 Q3 outcome with Eisa's go).
- **R-0b-3: GitHub Releases bandwidth throttling.** GH Releases doesn't publish bandwidth limits but reserves the right to throttle "abusive" traffic. At hundreds-of-installs-per-year × 5 GiB = ~tens of TiB/yr, this is well within normal open-source-project traffic and unlikely to trigger throttling. Mitigation: chunked downloads use the standard GitHub asset URL pattern (no novel access pattern that would look anomalous to GH's monitoring); if throttling appears at scale, we can introduce an alternative endpoint without changing the Tauri-side download code (just the URLs in `models.json`).
- **R-0b-4: Model-file integrity attack.** A compromised mirror could serve malicious GGUFs. Mitigation: SHA-256 sidecar verified before load; sidecar URL pinned in `docs/models.json` which is bundled as a Tauri resource (not fetched at runtime); the manifest changes only via a signed Constellation release.
- **R-0b-5: Download UX friction on slow connections.** A 5 GiB download is real time. Mitigation: progress UI is mandatory (Step F); resume support is Phase 1 polish; the install flow blocks chat features only — the rest of Constellation works fine without Mind.

No schema change. The mind/ trait surface from MIG-046 is unchanged. Rollback is well-defined.

---

## 8. What Phase 0b explicitly does NOT decide

Surfaced here so they don't get accidentally bundled in:

- **`RoutedProvider`** (Phase 2.5 / MIG-050) — 0b installs both models but the orchestrator uses one at a time. Routing logic + LoadStrategy come later.
- **Real `ToolDispatcher`** with actual `search_notes` / `read_note` / etc. wired to NSC + HMSE — Phase 1 (MIG-048).
- **Chat UI surface** — Phase 1 (MIG-048).
- **Citation validator + post-generation `note:UUID` resolution** — Phase 1.
- **Write tools + approval modal + undo journal** — Phase 2 (MIG-049).
- **Cost telemetry / cloud opt-in** — Phase 5 (MIG-053).
- **Q5_K_M or larger quantizations** for the workstation profile (Concept Paper §9.3) — defer; 0b ships Q4_K_M only. Workstation users can manually install other quants later if we expose the URL pattern.
- **Range-resume on interrupted downloads** — Phase 1 polish; 0b restarts on interrupt.
- **Real Settings → About panel display of the Model Notices block** — 0b finalizes the *text* (`docs/models/MODEL-NOTICES.md`); MIG-048 displays it.
- **Mobile (iOS/Android) inference** — both runtimes lack a documented mobile recipe; mobile builds fall back to `OfflineProvider` via target-specific Cargo block (similar to the existing `memmap2` pattern).

---

## 9. Decisions locked by Eisa (2026-05-24)

| # | Question | Lock | Status |
|---|---|---|---|
| §4 A | Runtime choice | **A4** — `mistral.rs` in 0b for Fanar; `llama-cpp-2` added in Phase 2.5 for Jais. Jais is kept (per §10.5 Q3 intent); the staging splits the cmake risk introduction off into Phase 2.5 rather than 0b. | LOCKED |
| §4 D | Distribution endpoint | **GitHub Releases with file-splitting** — no Constellation-operated cloud. Each model lives in its own `models/<name>-q4km-v1` GH Release. Tauri downloader concatenates ~1.7 GiB chunks + verifies SHA-256. **Overrides Plan §10.5 Q3** (the original "Constellation-hosted mirror" lock). | LOCKED |
| §4 F | Bench order | Bench (Step H) runs **after** real `LocalProvider` lands (Step G). | LOCKED |
| Quant level | What Q-level for v1 | **Q4_K_M only.** Q5_K_M deferred to a later phase (likely MIG-050 once the install registry handles multi-quant variants per model). | LOCKED |

The four locks above are durable inputs to the Build cascade. Plan §10.5 will be updated in the same commit as this Architect to record the §4 D override on Q3.

---

*Phase 0b of the Constellation Mind Implementation Plan v1.0 (Plan §4 + §10.5). Locked. Build cascade begins per Plan-Approval-=-Build-Approval. Phase 1 (MIG-048) follows as its own `/migration` once 0b's Build + Audit + Boss-test Stage 1 close.*
