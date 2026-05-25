---
aliases:
  - Constellation Mind
  - Mind
  - Local LLM
  - Fanar
  - AI Chat
  - Personal AI
description: Constellation Mind is Constellation's local Large Language Model layer — an AI you can chat with about your own notes, running entirely on your device. Phase 1 shipped 2026-05-25 with the chat surface in the left dock, citation-bound answers, and a 6-tool read dispatcher over your Universe.
---

# Constellation Mind (عقل Constellation)

## What Is It?

Constellation Mind is the local Large Language Model (LLM) layer of Constellation — an AI assistant that knows your Universe and can talk with you about your notes, **without sending any of them to the cloud**.

Three things make it distinct from every other "AI for notes" tool:

1. **Local-first.** The model runs on your device. Your notes never leave. There is no cloud round-trip — the chat is local and offline-capable.
2. **Arabic-first.** The bundled-default model is **Fanar-1-9B**, Qatar Computing Research Institute's Arabic-centric Sunni-aware model. Native MSA + Gulf-dialect competence; English is the second language, not the only one.
3. **Citation-bound.** Every factual claim the AI makes about your notes must cite the source note. Hallucinated citations are caught by a post-generation validator (Phase 1).

## What ships today (Phase 1 — 2026-05-25)

- **Chat surface in the left sidebar.** A new speech-bubble mode button between Digest and the OrgChart/SkyView dock-bar buttons. Click it; the chat pane mounts. Type a question, press Enter, the model reads your notes and answers in your language.
- **6 read tools.** `search_notes`, `read_note`, `find_similar`, `summarize`, `list_recent`, `graph_neighbors`. Mind picks the right one and calls it; you see the call as a collapsed `▸ Tool: <name>` entry in the chat (expandable to inspect the JSON args).
- **Citation pills.** Every claim Mind makes about your notes appears as a clickable purple pill: 📎 NoteName. Click it to open the cited note in a new editor tab.
- **Citation validator.** Post-stream check: every `[note:<path>]` Mind cites is verified against `note_meta`. Unresolved citations trigger a single retry with feedback; if still unresolved, the response gets a `⚠ Verify before trusting.` prefix.
- **Pre-warm on app start.** First chat turn pays warm latency (~1–1.5s) instead of cold (~9–11s). Mind loads in the background within ~10s of app launch.
- **Sliding-window history trim.** Long conversations get the oldest turn pairs dropped from what Fanar sees; the UI keeps the full conversation visible.
- **Settings → Mind panel** (unchanged from Phase 0b) — install models, set the active one.
- **Model installation** (unchanged) — chunked download from GitHub Releases, SHA-256 verified.
- **Real inference runtime** (unchanged) — `llama-cpp-2` (CPU-only in v1) loads the Q4_K_M GGUF.

### Chat surface walkthrough

1. **Open the chat tab.** Left sidebar → speech-bubble icon. The chat pane mounts with an empty hint.
2. **Type a question.** "What did I write recently about Canopus?" Press **Enter**. (Shift+Enter inserts a newline for multi-line composer input.)
3. **Watch the turn unfold.** Your message appears as a purple bubble on the right. An assistant bubble below has a blinking cursor while Mind thinks. A collapsed tool-call entry appears within 1–2 seconds: `▸ Tool: search_notes`. Click it to expand the JSON args.
4. **Read the answer.** Mind streams tokens into the assistant bubble. Citations render as clickable purple pills wherever Mind references a note.
5. **Click a citation** to open the cited note in a new editor tab.
6. **Clear the conversation** with the 🗑 button in the chat header. (Persistence per-Universe lands in a Phase 1.x polish.)

### Arabic conversations

The chat surface is Arabic-aware per-message. Type a question in Arabic; Fanar replies in Arabic; the bubble auto-flips to RTL. Mixed-script messages flow correctly in either direction.

## How to install Fanar

1. Open **Settings → Mind**.
2. Find **Fanar 1.9B (Q4_K_M)** in the catalog. The card shows the size (5.01 GiB), the license (Apache-2.0 with defensive Gemma notices), and a "Set active" or "Install" button.
3. Click **Install**. A progress bar shows download + SHA verify + assemble in three phases.
4. When the badge flips to **Installed** + **Active**, the model is ready. Fanar lives at `<app-data>/Constellation/models/fanar-1-9b-q4km-v1.gguf` and is mmap-backed (no copy into RAM).

That's it. Until Phase 1 ships the chat surface, the installed model is on standby.

## What's coming in Phase 1.x polish + Phase 2

**Phase 1.x polish queue** (additive on top of what shipped in Phase 1):
- Per-Universe conversation persistence (today the chat resets when you switch sidebar modes).
- Real tokenizer in the history trim budget (today uses a chars/4 heuristic).
- 13-locale brand-name policy review — current state uses a hybrid pattern ("Constellation" + localized "Mind"); Arabic uses fully-localized "عقل المجرّة".

**Phase 2** — Write tools: Mind proposes edits / new notes / new typed links under your explicit approval. Diff modal + undo journal. Tracked as MIG-049.

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

**Chat tab doesn't appear in the sidebar.** Either you're on a build older than 2026-05-25 (Phase 1 hadn't shipped), or the build's frontend bundle didn't include the layout change. Re-run the installer.

**"No Constellation Mind model is active" banner in the chat pane.** Settings → Mind doesn't show Active for any model. Install Fanar (the only model in v1) and click Set active.

**First chat turn takes 9+ seconds before tokens appear.** Pre-warm hasn't finished. Either wait ~10 seconds after app launch before sending the first message, OR run a second turn — by then the model is warm and first-token is ~1–1.5 seconds.

**Citation pill clicks open the wrong note (or nothing).** The library-resolver in `MindCitationChip.svelte` does a best-effort prefix match against your registered libraries. If your cited path is outside any registered library root, the chip's openNoteTab call may fail silently. Report the path + expected library so the resolver can be tuned.

**"⚠ unresolved citations" warning appears on every response.** The citation validator failed-open as a Phase 1.x P1 fix (commit `4e978076`), so this should NOT happen when the search DB is unavailable. If it does happen with a fully-indexed Universe, the model is fabricating note paths. Send the response + the actual note titles you have, and the system prompt / validator can be tuned.

---

*Sub-topics for Phase 1.x and beyond: multi-model picker (Phase 2.5 / MIG-050 adds Jais alongside Fanar); per-Universe conversation persistence; second-screen rendering of long chats; write-tool approval modal (Phase 2 / MIG-049).*
