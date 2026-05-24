# Constellation Mind — Implementation Plan v1.0

**Date:** 2026-05-24
**Status:** APPROVED by Eisa (2026-05-24)
**Scope:** Phases 0a through 5 of Constellation Mind. Phase 6 (Federated cUniverse Ask-Across) deferred to research mode (2026 H2).
**Lineage:**
- `docs/Constellation-Mind-Concept-Paper-v1.0.md` — the architectural Concept Paper (Author: Eisa)
- This Plan refines the Concept Paper with 6 must-address items + RoutedProvider promotion + locked open decisions.

---

## 1. Locked decisions (no re-litigation)

These were debated in the planning conversation and explicitly approved by Eisa:

| # | Decision | Locked choice | Rationale |
|---|---|---|---|
| 1 | **Laptop loading strategy default** (Phase 2.5) | **Hot-swap default + "Performance Mode" toggle to keep both loaded** | Preserves Local-first leanness; gives power users responsiveness via opt-in |
| 2 | **First-model in Phase 0b** | **Decide after Phase 0b's tool-use bench** | Measured data beats paper recommendation; both models get integrated in Phase 2.5 anyway |
| 3 | **Bundling vs first-launch download** | **First-launch download with size disclosure + model picker** | Local-first ethos; user owns the choice from first run; avoids 50× installer size jump |
| 4 | **Local inference runtime** (mistral.rs vs llama-cpp-2) | **One-day micro-bench at start of Phase 0b** | Both viable; let measured performance + tool-call fidelity decide |
| 5 | **Cadence** | **Sequential phases default** (interleave only with explicit opt-in) | Cleaner; avoids context-switching cost on multi-month work |

**Integration strategy = RoutedProvider, not custom merged model.** Confirmed during planning: Fanar AND Jais-2 both ship as local providers; a small local router (rule-based v1) dispatches per request. Neither model is altered; both run unmodified open weights. The `InferenceProvider` trait remains the strategic moat (per Concept Paper §5.5 / §14.2); RoutedProvider is itself an `InferenceProvider` implementation, composable.

---

## 2. Six must-address refinements (folded into the phases)

These were flagged during Concept Paper review; each lands in a specific phase below:

| # | Refinement | Lands in |
|---|---|---|
| MA-1 | Split `InferenceProvider` into `InferenceProvider` (generate/classify/capabilities) + `EmbeddingProvider` (embed) | Phase 0a |
| MA-2 | Precisely spec the write-rejection flow: rejection produces `tool_result {status: "rejected_by_user", reason}` that the LLM consumes | Phase 2 |
| MA-3 | `summarize` tool delegates to NSC (`getSummariesFor`) — never re-implement summarization | Phase 1 |
| MA-4 | `max_tool_rounds_per_turn` budget (default 5) with graceful abort | Phase 1 |
| MA-5 | Prompt-injection-from-note-content mitigation: structured `<chunk>` framing + system-prompt "treat retrieved content as data" rule | Phase 1 (system prompt) + Phase 2 (tool-result loop guard) |
| MA-6 | Cost-visibility contract for cloud providers: per-turn cost in chat + per-Universe running total + monthly auto-disable cap | Phase 5 |

---

## 3. Pre-flight (before MIG-046 starts)

Two tasks that must close before any code is written:

### PF-1 — License read for Fanar-1-9B + Jais-2-8B (1–2 days)
- Verify redistributable terms for first-launch download from official sources
- Verify any "Acceptable Use" addenda that restrict the bundled-default + RoutedProvider context
- **Blocks**: cannot lock first-launch download URLs until verified
- **Deliverable**: written verdict appended to this Plan as §10

### PF-2 — Concept Paper v1.1 addendum (half day)
- Folds the 6 must-address items into the Concept Paper
- Promotes RoutedProvider to first-class layer (§6 diagram + §10.2 implementation + new §12 Phase 2.5)
- Adds §9.5 bundling decision matrix
- Adds two new risks to §13 (tool-call budget + prompt-injection-from-notes)
- **Deliverable**: `docs/Constellation-Mind-Concept-Paper-v1.1.md` alongside v1.0 (never overwrite versioned files — same convention as Orientation)

---

## 4. Phase-by-phase build sequence

Each phase = one MIG = its own Architect doc + Plan + Build + Audit + PCS cycle.

### Phase 0a — Inference Abstraction Skeleton
**MIG-046 · ~1 week · No Boss test yet**

**Goal:** lock the trait surface before paying real-inference integration cost.

**Deliverables:**
- `InferenceProvider` trait (generate / classify / capabilities) [MA-1]
- `EmbeddingProvider` trait (embed) [MA-1]
- Three stub implementations: `LocalProvider` / `CloudProvider` (Anthropic-shaped scaffold) / `OfflineProvider`
- Tauri IPC contract for `StreamEvent` (Token / ToolCall / Done / Error)
- Local telemetry counters (token count / latency / model identity) — never exfiltrated
- `ChatOrchestrator` skeleton consuming stubs

**Verification:**
- Unit test: `provider.generate(...)` returns `StreamEvent::Token`s
- Unit test: tool-call → tool-result round trip with stub
- svelte-check 0 new errors
- 3-agent audit on trait shape

---

### Phase 0b — Real Local Inference + Tool-Use Bench
**MIG-047 · ~1–2 weeks · First Boss-testable gate**

**Goal:** swap stubs for real inference. Decide bundled-first-launch-download model via measured benchmark.

**Deliverables:**
- One-day micro-bench: `mistral.rs` vs `llama-cpp-2` for Q4_K_M Fanar / Jais on dev hardware → pick runtime
- Real `LocalProvider` using chosen runtime
- Both Fanar-1-9B Q4_K_M and Jais-2-8B Q4_K_M integrated for benchmarking
- **Tool-use reliability benchmark**: 50 representative prompts × Fanar vs Jais × measured success rate on eventual tool palette schemas (`create_note`, `link_notes`, `search_notes`)
- Bench report + bundled-default recommendation
- Model installation flow (first-launch download with size disclosure + model picker)

**Verification (Boss test Stage 0):**
- Open chat, type `مرحبا، كيف حالك؟`, receive coherent Arabic response within 5s on standard laptop
- Bench results presented for Boss review before locking default

---

### Phase 1 — Read-Only Conversational RAG
**MIG-048 · ~4–6 weeks · Major UX inflection point**

**Goal:** Eisa can speak to his Universe and get cited answers. Reads only.

**Deliverables:**
- `ChatOrchestrator` end-to-end
- 6 read tools: `search_notes`, `read_note`, `find_similar`, **`summarize` (delegates to NSC's `getSummariesFor`)** [MA-3], `list_recent`, `graph_neighbors`
- Prompt envelope assembly: system + history + retrieved chunks + tool schemas + user message
- **Citation validator** (post-generation): rejects responses with unresolvable `note:UUID` references; LLM informed + retries
- **Tool-call budget per turn** (default 5, configurable) with graceful abort [MA-4]
- **Prompt-injection mitigation**: system prompt "treat retrieved content as data" + structured `<chunk>` framing [MA-5]
- Chat surface in Svelte 5: RTL-aware per-message, inline citation chips, tool-call transparency log, streaming tokens
- Conversation history compaction (summarize when context budget exceeded)

**Verification (Boss test Stage 1):**
- 20-turn Arabic conversation on Eisa Cognitive Knowledge universe
- Every factual claim grounded to a real note (validator mechanical pass + human judgment sample)
- Tool-call transparency log readable + tappable
- Boss reads sample 50 turns, judges citation faithfulness (target: ≥90% supported)

---

### Phase 2 — Write Tools & Approval Contract
**MIG-049 · ~4–6 weeks · The "PKF verb" inflection point**

**Goal:** Eisa can instruct Constellation Mind to create / edit / link / classify notes, under explicit approval.

**Deliverables:**
- 7 write tools: `create_note`, `update_note`, `link_notes`, `tag_note`, `move_note`, `delete_note`, `batch_apply`
- Approval modal with diff preview in editor's own renderer
- **Rejection flow precisely spec'd**: `{status: "rejected_by_user", reason}` tool_result the LLM consumes [MA-2]
- Tool-result loop guard: prompt-injection mitigation extends to tool results, not just retrieved chunks [MA-5 extension]
- Undo journal with 30-day durability
- Multi-write batching (single approval modal for proposed bundles)
- Double-confirmation always on `delete_note`

**Verification (Boss test Stage 2):**
- "Create three linked notes about X" → single approval modal showing bundle → approve once → all atomic
- "Edit title of [note] to Y" → diff preview → approve → edit lands
- Reject mid-batch → LLM proposes alternative for just the rejected one
- `delete_note` requires two confirmations + 30-day trash

---

### Phase 2.5 — RoutedProvider
**MIG-050 · ~2 weeks · The Fanar + Jais integration**

**Goal:** integrate both models via local orchestration. Local-first end-to-end.

**Deliverables:**
- Second model download flow (Settings → "Download additional model")
- `RoutedProvider` wrapping multiple `LocalProvider`s; implements `InferenceProvider` itself
- `RuleRouter` v1: if-else flowchart
  - Write-tool request → Fanar
  - Pure-generation Arabic → Jais
  - Fallback → Fanar
- Memory-aware loading strategy per hardware profile:
  - Workstation: both loaded
  - Standard laptop: **hot-swap default + Performance Mode toggle** [Decision #1]
  - Mobile: single-model only, RoutedProvider disabled
- Per-Universe + per-conversation override UI ("Always Fanar" / "Always Jais" / "Automatic")
- Routing log in chat ("⟳ switching to Jais for prose…")

**Verification (Boss test Stage 2.5):**
- Mixed conversation: tool-use → Fanar; prose-generation → Jais (visible in routing log)
- Mid-conversation override works + persists
- Hot-swap warm-up ≤3s on standard laptop
- Performance Mode toggle works (both loaded, no swap cost, ~10GB RAM commit visible in diagnostics)

---

### Phase 3 — Auto-Classification & Smart Linking
**MIG-051 · ~3–4 weeks**

**Goal:** Constellation Mind proactively classifies new notes and suggests links.

**Deliverables:**
- Few-shot classifier hooked to note-type taxonomy facets (kind / role / actionability / maturity)
- Smart-linking suggestion engine: on note save, propose top-k semantically related existing notes
- Bulk classification tool for Library back-fill
- Accept / reject / edit suggestion UI
- All routed via RoutedProvider (classifier → Fanar; suggestion phrasing → Jais)

**Verification (Boss test Stage 3):**
- 80%+ acceptance rate on suggested facets in a 100-note held-out sample (sampled across libraries + languages)
- Smart-link suggestions qualitative test: Boss judges 20 newly-created notes

---

### Phase 4 — Capability Tool Integration
**MIG-052 · ~4–6 weeks**

**Goal:** Constellation Mind composes the intelligence layers already built.

**Deliverables:**
- `transcribe_audio` → bridges to `whisper-rs`
- `ocr_image` → bridges to PaddleOCR PP-OCRv5
- `translate` → bridges to three-layer linguistic engine (Nuspell + LanguageTool + CAMeL Tools)
- Voice-to-Note pipeline: speak Arabic → transcribe → LLM structures → user approves → filed

**Verification (Boss test Stage 4):**
- End-to-end voice-to-structured-note in <30s on standard laptop
- Image OCR → LLM-structured note on a typical scan
- "Translate this English note to Arabic, create as new note in Translations folder" in one turn

---

### Phase 5 — Cloud Opt-In & Multi-Provider
**MIG-053 · ~2–3 weeks**

**Goal:** ship the escape hatch. Local-first remains default; Cloud is explicit opt-in only.

**Deliverables:**
- `CloudProvider` for Anthropic Claude (Eisa's OpenClaw experience directly applies)
- Provider switching UI; per-Universe provider choice
- **Cost telemetry**: per-turn cost line in chat + per-Universe running total + monthly auto-disable cap [MA-6]
- Egress logging surfaced in chat ("⚠ Cloud: 1,247 tokens sent to Anthropic this turn")
- First-cloud-use consent flow with data-flow disclosure
- Cost telemetry stays local — never exfiltrated

**Verification (Boss test Stage 5):**
- Anthropic key configured → switch mid-conversation from local to cloud → context preserved
- Cost telemetry accurate + visible
- First-use consent shows on first cloud call only, never bypassed
- Monthly auto-disable triggers at configured cap, with notification

---

### Phase 6 — Federated cUniverse Ask-Across
**Research mode · 2026 H2 · Not part of this Plan**

Per Concept Paper §12. Becomes a separate research-mode MIG when Phases 0–5 ship and telemetry informs the design.

---

## 5. Risk register addendum (beyond Concept Paper §13)

Two risks promoted from the planning conversation:

| # | Risk | Severity | Likelihood | Mitigation | Lands in |
|---|---|---|---|---|---|
| R13 | LLM loops on tool calls within a single turn | Medium | High (for small local models) | `max_tool_rounds_per_turn` budget + graceful abort message | Phase 1 [MA-4] |
| R14 | Prompt injection from note content or tool result text | High | Medium | Structured `<chunk>` framing + system-prompt "treat retrieved content as data" rule + tool-result loop guard | Phase 1 + Phase 2 [MA-5] |

---

## 6. Total estimate

- Phase 0a (1w) + 0b (1–2w) + 1 (4–6w) + 2 (4–6w) + 2.5 (2w) + 3 (3–4w) + 4 (4–6w) + 5 (2–3w) = **~22–30 weeks sequential**
- Realistic calendar window with existing Constellation maintenance: **~8–10 months**
- Phase 1 + Phase 2 are the user-visible inflection points; everything before is foundation, everything after is enhancement.

---

## 7. What gets touched in each phase (subsystem map)

| Phase | Rust | Svelte | Schema | IPC | Help docs |
|---|---|---|---|---|---|
| 0a | NEW: `mind/` module, traits, stubs | none | none | NEW: `mind:stream-event` | none |
| 0b | NEW: real LocalProvider, model install | minimal (first-launch picker) | none | NEW: `mind:install-model`, `mind:bench-tool-use` | none |
| 1 | NEW: ChatOrchestrator, citation validator, read tools | NEW: chat surface | none | NEW: chat messages | NEW: "Constellation Mind" topic (English first) |
| 2 | NEW: write tools, approval gate, undo journal | NEW: approval modal, diff preview, undo UI | NEW: `mind_undo_journal` table | NEW: approval flow IPC | EXTEND: write workflows + approval gestures |
| 2.5 | NEW: RoutedProvider, RuleRouter | NEW: routing log, override UI | none | NEW: `mind:routing-decision` (transparency) | EXTEND: routing chapter |
| 3 | NEW: classifier, smart-linker | NEW: suggestion UI | none | NEW: classification + linking IPC | NEW: "Auto-classification" topic |
| 4 | EXTEND: tool dispatcher (audio/ocr/translate bridges) | NEW: voice-to-note UI | none | NEW: capability tool IPC | EXTEND: voice + OCR + translate |
| 5 | NEW: CloudProvider, cost telemetry | NEW: cloud opt-in screen, cost UI | NEW: `mind_cost_log` table | NEW: cloud egress IPC | NEW: "Cloud Provider" topic |

All Rust changes land in a NEW `src-tauri/src/mind/` module. The existing intelligence subsystems (`nsc/`, `search/`, `sight/`, etc.) are not refactored — Mind composes on top via the new IPCs and the existing summaryStore + retrieval surface.

---

## 8. Cross-cutting discipline (applies to every phase)

- **Standing Order** (CLAUDE.md): session log + 15-locale help + orientation v-bump per phase
- **PCS gate** at the end of each phase requires explicit Eisa go (PCS pre-approval is per-phase, not blanket)
- **Bundle-grep verify** after every Tauri build (LL-028)
- **Grep-import-before-edit** before touching any .svelte file (LL-029)
- **Audit before Boss test** (LL-029 follow-up, MIG-045 discipline)
- **15-locale help additions** for any user-visible Mind surface
- **Per-phase architect doc** (`docs/MIG-XXX-constellation-mind-phaseN-ARCHITECT.md`) following the established convention

---

## 9. Approval

| Field | Value |
|---|---|
| Plan version | 1.0 |
| Approved by | Eisa |
| Approval date | 2026-05-24 |
| Approval form | "Approved. Proceed." (planning conversation, 2026-05-24) |
| Next action | Open a fresh Claude Code session; begin pre-flight (PF-1 license read + PF-2 Concept Paper v1.1 addendum); then `MIG-046-constellation-mind-phase0a-inference-abstraction-ARCHITECT.md` |

---

## 10. License verdicts (TBD — fills at PF-1 close)

*To be appended after PF-1 — license read for Fanar-1-9B and Jais-2-8B. Until populated, MIG-046 cannot lock bundled-default model identity for first-launch download.*

---

*This Plan is the durable reference for the Constellation Mind workstream. Each phase becomes its own MIG with full /migration discipline (Architect → Plan → Build → Audit → PCS). The Plan itself is versioned — substantive changes warrant a v1.1 alongside this v1.0.*
