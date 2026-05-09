# MIG-021 — Local LLM Research Summary

**Date:** 2026-05-09
**Purpose:** Inform Eisa's decision on the three open sub-decisions for the Sight Epistemic Classifier (per `project_sight_classifier_local_llm.md`).
**Decision pending:** Eisa picks model + inference engine + bundling strategy → MIG-021 Architect can be drafted.

Three parallel research agents (general-purpose, with WebSearch + WebFetch) ran cover-to-cover on the three sub-decisions. Findings condensed below; agent recommendations preserved verbatim where they constitute a clean conclusion.

---

## Sub-decision 1 — Which LLM?

### Top recommendation: **Qwen3-1.7B (Alibaba, May 2025)**

| Property | Value |
|---|---|
| License | Apache 2.0 — fully redistributable in commercial desktop binary |
| Multilingual | 119 languages incl. **first-class Arabic**; strongest non-English coverage among sub-3B open models |
| Q4_K_M GGUF size | ~1.1 GB on disk |
| CPU speed | 25–45 tok/s on modern 8-core x86 CPU (4–6 threads via llama.cpp) |
| Classification fit | Strong; works with `llama.cpp` GBNF grammar enforcement for the 11-source taxonomy |
| Trade-off | Ships with "thinking mode" — must disable (`/no_think`) for classification or token cost balloons |

### Runner-up: **Gemma 4 E2B (Google DeepMind, April 2026)**
- Apache 2.0 (Gemma 4 dropped the prior custom Gemma ToS)
- 140+ languages; community reports favor Gemma 4 over Qwen 3.5 on Arabic
- Q4_K_M ~1.4–1.6 GB; ~18–30 tok/s CPU
- Pick over Qwen3 only if internal Arabic eval (200–500 hand-labeled Constellation notes) shows measurable win

### Disqualified
- **Llama 3.2 3B**: Arabic NOT in officially-supported 8 languages (EN/DE/FR/IT/PT/HI/ES/TH); Meta disclaims Arabic quality
- **Phi-3.5 / Phi-4-mini**: English-dominant, weak Arabic
- **Gemma 3**: custom license retains Google revocation rights; redistribution risk
- **SmolLM3-3B**: Arabic explicitly listed as "fewer tokens seen" — second-tier

### Sources
- Qwen3 blog + arXiv 2505.09388 · Google InfoQ Gemma 4 Apache 2.0 · Codersera Gemma 4 guide · Llama 3.2 model card · TechCrunch on open-model license restrictions · arXiv 2505.06461 CPU benchmarks · BentoML 2026 SLM survey

---

## Sub-decision 2 — Which inference engine?

### Top recommendation: **llama.cpp via `llama-cpp-2` (utilityai) Rust bindings**

For 1.5–3B classification with constrained JSON output, llama.cpp is the unavoidable best fit. Reasons:
1. **GBNF grammar-constrained decoding** — guarantees valid JSON output without retries (critical for classification; eliminates parsing failures)
2. **Fastest CPU inference** of any candidate (AVX2/AVX-512/NEON tuned)
3. **First-class GGUF support** across every Qwen/Phi/Llama variant Constellation might want

| Dimension | llama.cpp (`llama-cpp-2`) |
|---|---|
| Maturity | Battle-tested, weekly releases |
| Tauri/Rust integration | `llama-cpp-2` crate; bundled C++ via `cmake` build script — no system libs needed; `bindgen` + MSVC works cleanly on Windows |
| Cross-platform | Win/macOS/Linux all first-class |
| Formats | GGUF only |
| GPU (optional) | CUDA, Metal, Vulkan, ROCm — opt-in via cargo features |
| Memory | ~1.5 GB for Qwen2.5-1.5B Q4_K_M; ~2.5 GB for 3B |
| License | MIT |
| Footgun | First compile +30–60s; binary +15–25 MB |

### Direct answer to "can ONNX Runtime host the LLM (avoid second engine)?"
**Technically possible, practically a poor fit:**
1. No Rust bindings for `onnxruntime-genai`. The `ort` crate wraps ORT itself, not the autoregressive wrapper. Re-implementing generation loop (KV cache, sampling, tokenizer) in raw ORT is significant engineering for a commodity feature.
2. CPU INT4 perf in `onnxruntime-genai` reported slow (open issue #1098).
3. No GBNF grammar constraints — would need post-hoc JSON validation/repair.
4. Quantized GGUF size is comparable to or smaller than equivalent ONNX INT4.

**Recommended split:** ORT stays for embeddings (it's optimized for that and `multilingual-e5-small` is well-supported); llama.cpp handles autoregressive generation. Distinct workloads, distinct tools. Wrap both behind a single `inference::` Rust module so the frontend never sees which engine is used.

### Runner-up: **mistral.rs**
Pure Rust, ~6.7k stars, actively maintained. Pick if NSIS installer size or eliminating C++ build toolchain matters more than raw speed. Trade-off: ~10–25% slower than llama.cpp on CPU; grammar-constrained decoding less mature.

### Don't-pick warnings
- **`candle` directly** — slower kernels than llama.cpp, thinner quantization coverage. Use mistral.rs (built on candle, fixes the issues) instead.
- **`rustformers/llm`** — explicitly unmaintained per its own README.

### Sources
- llama-cpp-2 crate · Tauri local-LLM integration example (dillondesilva/tauri-local-lm) · mistral.rs repo · Hugging Face Candle · onnxruntime-genai issue #1098 · onnx-community/Qwen2.5-1.5B · pykeio/ort · rustformers/llm

---

## Sub-decision 3 — Bundling strategy

### Top recommendation: **Hybrid — bundle small classifier in installer + offer larger optional download in Settings**

This is the **Smart Connections pattern** (Obsidian plugin with 2M+ installs), the only one in the surveyed precedents that delivers zero-friction first-meaningful-use AND lets power users opt up.

### Survey of 8 real precedents (2025–2026)

| App | Installer | Model handling | Choice |
|---|---|---|---|
| **GPT4All** | ~200 MB | In-app downloader; user picks 3–8 GB GGUF on first run | **B (download)** |
| **LM Studio** | 150–350 MB | No bundled model; HuggingFace browser inside app | **B** |
| **Jan.ai** | ~200 MB | Guided GGUF download on first launch via llama.cpp | **B** |
| **Ollama** | ~600 MB | Models pulled via CLI | **C (BYO)** |
| **MacWhisper** | small | Auto-downloads selected Whisper model on first use | **B** |
| **Whisper Desktop** | small | Pick model, app downloads | **B** |
| **Smart Connections (Obsidian)** | KB plugin | **Bundles small embedding model**; optional Ollama for bigger | **HYBRID A-small + B-optional** |
| **Zed** | ~150 MB | Defers entirely to Ollama or cloud APIs | **C** |

### Dominant pattern in 2026
- **For end-user novice-friendly apps**: B (first-run guided download) — GPT4All / Jan / LM Studio / MacWhisper / Whisper Desktop
- **For developer tooling**: C (BYO via Ollama) — Zed
- **The Smart Connections hybrid** is the lowest-friction precedent in the PKM space and most resembles Constellation's audience

### Why the hybrid wins for Constellation
- Bundle a **~100–250 MB Q4 small model** (sufficient for the 11-source classification job) directly in the .exe. Installer grows from ~50 MB to ~200–300 MB — comfortable single-asset GitHub Release, fast download, antivirus-clean.
- Sight works the moment install finishes — no network, no chooser, no "where do I put the GGUF?" The novice never makes a decision.
- In **Settings → AI**: single optional action *"Download larger classifier (better accuracy, ~1.5 GB)"* with progress UI, resumable, cancellable. Power users opt in.
- **Do NOT ship "bring your own GGUF" path in v1** — violates Eisa's "if users don't understand it, its existence is unnecessary" directive.

### Why not pure A (full bundle in installer)?
- ~2 GB installers fail GitHub's 2 GB single-asset limit
- Double bandwidth costs on every patch
- Trigger Windows SmartScreen / AV friction
- Marginal accuracy gain doesn't justify those costs when 80% of classification quality is reachable with a 200 MB model

### Why not pure B (Jan/GPT4All style)?
- The first-run download is the highest-friction moment in those apps' UX
- Sight needs to *just work* the first time the user opens the panel — making them wait for a 1.5 GB download contradicts the "fast software" rule and the novice-first directive

### Sources
- GPT4All (Nomic) · Jan.ai docs · LM Studio docs · MacWhisper / WhisperKit docs · whisper.cpp · Smart Connections (Obsidian) · Zed + Ollama docs · SitePoint UX patterns for large client-AI model downloads

---

## Implication for the small-bundled model (Eisa's call)

The hybrid strategy needs **two model tiers**:

| Tier | Size | Purpose | Candidate |
|---|---|---|---|
| Bundled (default, in installer) | ~100–250 MB | Sight works on Day 1, no network | **Qwen2.5-0.5B-Instruct Q4_K_M (~350 MB)** OR a **Smart-Connections-style embedding-classifier hybrid** OR **a smaller distilled model** — needs choosing |
| Optional download (Settings → AI) | ~1.0–1.5 GB | Power users, better Arabic + accuracy | **Qwen3-1.7B Q4_K_M (~1.1 GB)** ← top recommendation |

The "bundled" tier is a downstream sub-decision that depends on whether 200 MB is a hard ceiling. Three options:
- **(a) Qwen2.5-0.5B Q4_K_M**: 350 MB, 60–80 tok/s CPU, weak Arabic at this size
- **(b) Multilingual-e5-small repurposed via embedding-classifier**: already shipped (113 MB), no additional bundle, classifier-via-embedding-similarity instead of generation. Different architecture; lower accuracy ceiling but free in terms of bundling.
- **(c) Skip the bundled tier, force first-run download (~1.1 GB Qwen3-1.7B)**: simpler architecture, harder first-run UX

---

## Decision matrix for Eisa

Three picks (or "agreed with all three top recommendations"):

1. **LLM**: Qwen3-1.7B (top) / Gemma 4 E2B (runner-up if Arabic eval favors it) / something else
2. **Inference engine**: llama.cpp via `llama-cpp-2` (top) / mistral.rs (runner-up if pure-Rust matters more than speed) / something else
3. **Bundling**: Hybrid (bundle small + optional larger download) / pure B (no bundle, first-run download) / pure A (full bundle in installer)

Once decided, MIG-021 Architect doc is drafted; Sight Concept Paper v2.0 follows in parallel.
