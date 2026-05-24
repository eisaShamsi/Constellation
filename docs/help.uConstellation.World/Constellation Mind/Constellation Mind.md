---
aliases:
  - Constellation Mind
  - Mind
  - Local LLM
  - Fanar
  - AI Chat
  - Personal AI
description: Constellation Mind is Constellation's local Large Language Model layer — an AI you can chat with about your own notes, running entirely on your device. Phase 0b shipped 2026-05-24 with the Fanar-1-9B Arabic-first model installable from Settings → Mind. The chat surface lands in Phase 1.
---

# Constellation Mind (عقل Constellation)

## What Is It?

Constellation Mind is the local Large Language Model (LLM) layer of Constellation — an AI assistant that knows your Universe and can talk with you about your notes, **without sending any of them to the cloud**.

Three things make it distinct from every other "AI for notes" tool:

1. **Local-first.** The model runs on your device. Your notes never leave. There is no cloud round-trip — the chat is local and offline-capable.
2. **Arabic-first.** The bundled-default model is **Fanar-1-9B**, Qatar Computing Research Institute's Arabic-centric Sunni-aware model. Native MSA + Gulf-dialect competence; English is the second language, not the only one.
3. **Citation-bound.** Every factual claim the AI makes about your notes must cite the source note. Hallucinated citations are caught by a post-generation validator (Phase 1).

## What ships today (Phase 0b — 2026-05-24)

- **Settings → Mind panel** — lists installable models (currently just Fanar 1.9B Q4_K_M, ~5 GiB), with an Install button that downloads + verifies the model.
- **Model installation** — chunked download from a GitHub Release (no third-party cloud), SHA-256 verified per chunk and on the assembled whole.
- **Real inference runtime** — `llama-cpp-2` (CPU-only in v1) loads the Q4_K_M GGUF and streams tokens.
- **No chat surface yet** — that's Phase 1 (the next milestone). Today you can install the model and verify it; the conversational UI ships in MIG-048.

## How to install Fanar

1. Open **Settings → Mind**.
2. Find **Fanar 1.9B (Q4_K_M)** in the catalog. The card shows the size (5.01 GiB), the license (Apache-2.0 with defensive Gemma notices), and a "Set active" or "Install" button.
3. Click **Install**. A progress bar shows download + SHA verify + assemble in three phases.
4. When the badge flips to **Installed** + **Active**, the model is ready. Fanar lives at `<app-data>/Constellation/models/fanar-1-9b-q4km-v1.gguf` and is mmap-backed (no copy into RAM).

That's it. Until Phase 1 ships the chat surface, the installed model is on standby.

## What's coming in Phase 1 (next milestone)

- **Chat surface** — a Constellation panel where you talk to Fanar about your Universe in Arabic or English (RTL-aware per message).
- **Read tools** — Mind can call `search_notes`, `read_note`, `find_similar`, `list_recent` to ground its answers in your actual notes.
- **Citation validator** — every claim cites a real note; fabricated `note:UUID` references are rejected before they reach you.
- **Pre-warm on app start** — Mind loads in the background so your first chat doesn't pay the 10-second cold-load.
- **Conversation history** — saved per Universe; promotable to a Note.

See `docs/Constellation-Mind-Concept-Paper-v1.1.md` for the full architecture and `docs/Constellation-Mind-Implementation-Plan-v1.0.md` for the phase-by-phase roadmap.

## What's coming later

- **Phase 2 — Write tools** (Mind proposes edits / new notes / links under your explicit approval).
- **Phase 2.5 — RoutedProvider + Jais** (a second model, Jais-2-8B from G42/MBZUAI, joins Fanar as a co-default; Mind routes between them based on the request).
- **Phase 3 — Auto-classification + smart-linking** (Mind proposes facets and links on note save).
- **Phase 4 — Capability tools** (voice → note, OCR → note, translation).
- **Phase 5 — Cloud opt-in** (your own Anthropic / OpenAI key, with per-Universe cost cap and per-turn egress log).

## Privacy & data flow

- **Outbound HTTP only when installing a model** — Constellation downloads model files from the [`models/*` GitHub Releases](https://github.com/eisaShamsi/Constellation/releases) of this repo. No telemetry. No cloud inference (yet — that's Phase 5, and only with your explicit opt-in).
- **On-disk:** the model GGUF + an `installed_models.json` registry tracking which models you have and which is active.
- **At runtime:** the loaded model file is memory-mapped; your prompts + responses live in RAM only.

## Licenses

Each model carries its own LICENSE.txt alongside it in the GitHub Release. For Fanar:

- **Apache License 2.0** (QCRI's declared license on the Fanar-1-9B-Instruct repo).
- **Gemma Terms of Use** — Fanar is a continued pretraining of `google/gemma-2-9b`; Constellation ships the Gemma notices defensively even though QCRI relabels the result as Apache-2.0 alone.
- **Fanar citation** (Fanar Team 2025, arXiv:2501.13944).
- **Constellation redistribution notice** — the GGUF on Constellation's GitHub Release is a quantization of QCRI's upstream safetensors, produced by `.github/workflows/model-pipeline.yml` and distributed under Apache-2.0 with the original LICENSE traveling.

The full LICENSE.txt lives alongside each model in its release: <https://github.com/eisaShamsi/Constellation/releases/tag/models/fanar-1-9b-q4km-v1>.

## Troubleshooting

**"Not yet ready" badge instead of Install button.** The bundled catalog has a placeholder SHA-256 for that model. This shouldn't happen on a normal Constellation install; if you see it, the catalog hasn't been updated for that model version. Open an issue.

**Install hangs at "Downloading part X/Y".** Network issue. Cancel from Settings → Mind, re-trigger Install — the partial chunks are cleaned up automatically.

**Install succeeds, file SHA-256 doesn't match.** A bit-flip on download. Re-install will fetch fresh.

**Chat surface missing.** Phase 1 (MIG-048) hasn't shipped yet. The model can be installed + verified today; the conversation UI lands in the next release.

---

*Sub-topics will join this folder as Phase 1 ships: chat-UI walkthrough, citation-chip tap behavior, multi-model picker, second-screen rendering of long chats.*
