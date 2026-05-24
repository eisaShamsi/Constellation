# MIG-047 Phase 0b — Tool-Use Reliability Bench (Fanar-1-9B Q4_K_M)

**Date:** 2026-05-24 19:07 UTC
**Model:** `fanar-1-9b-q4km` (SHA-256 `7d18ceee…`)
**Runtime:** llama-cpp-2 0.1.146 (Path A pivot from §C-v1's mistral.rs)
**Prompt count:** 10 (Architect §5 Step H called for 50; starter set demonstrates the methodology)

---

## Headline result

| Axis | Pass rate | Reason |
|---|---|---|
| Tool-call validity | **0/10 (0%)** | **Structural gap, not a Fanar weakness** — see below |
| Argument keyword | 0/10 (0%) | (gated on tool-call validity) |
| Coherent reply | 0/10 (0%) | (gated on round-2 dispatch) |

Every row of the bench table reads `❌ ❌ ❌ — (no tool call)`. **This is the bench correctly surfacing that `mind/providers/local.rs` does not extract tool calls from generated text in Phase 0b**, not a measurement of Fanar's instruction-following on its own.

---

## Why every row is (no tool call)

In the §C-v1 mistral.rs implementation, `Response::Chunk.delta.tool_calls` was a first-class field — mistral.rs parsed model output for tool-formatted text and emitted typed `ToolCallResponse`s. The §G code already mapped those to `StreamEvent::ToolCall`.

When Path A pivoted to **llama-cpp-2** (§C-v2), the runtime swap consciously dropped the high-level tool-parsing layer. `llama-cpp-2` is a thinner wrapper — it produces raw tokens and leaves tool-call extraction to the application. Phase 0b's `local.rs::run_inference` therefore emits `StreamEvent::Token` for every model output and never produces a `ToolCall` event.

This is **documented intent**, not regression. From `local.rs::classify()` and the §C-v2 commit message:

> classify() is not implemented in Phase 0b; coming in Phase 3 (MIG-051) via constrained generation
> tool-call extraction — Phase 1 (MIG-048) ToolDispatcher parses model output for tool calls (likely via GBNF grammar)

The bench harness itself is correct: it sets `params.tools = [...]`, calls `provider.generate()`, watches for `StreamEvent::ToolCall`. It just receives plain tokens because the runtime doesn't synthesize ToolCall events yet.

---

## What Fanar actually emitted

The bench captures the model's raw text response to each prompt (round-1 only, since round 2 fires only after a tool call). Spot-checking the bench output:

- **s1 ("Search my notes for everything about Canopus.")** — Fanar generated prose describing Canopus, not a tool-formatted JSON. With prompt-engineering or system-prompt tool instructions, this would change.
- **s2 (Arabic: "ابحث في ملاحظاتي عن سهيل.")** — Same: prose response in Arabic, no JSON tool emission.
- **lr1 ("What did I write in the last 7 days?")** — Conversational answer; no `list_recent` JSON.
- **multi-step prompts** — Coherent multi-paragraph responses; no tool-call planning.

This matches expectations for Gemma-2-class instruction-tuned models without an explicit tool-call system-prompt or constrained-generation harness. Phase 1 closes the gap with one or more of:

1. **GBNF grammar constraint** — `llama-cpp-2` exposes llama.cpp's grammar feature; we constrain the model to emit valid JSON matching a tool schema when tools are present.
2. **Prompt-template tool block** — inject a Gemma-2-friendly system prompt that tells the model how to format tool calls (e.g., the Hermes format), then regex-extract from output.
3. **Output stream sniffer** — watch the token stream for `{"name":"…"` patterns and split tokens between Text and ToolCall events on the fly.

Plan §1 Decision #4 + §10.5 Q3's RoutedProvider direction both bake on tool-call discipline; MIG-048 makes this real.

---

## Per-prompt detail

| ID | Category | Prompt (truncated) |
|---|---|---|
| s1 | search | Search my notes for everything about Canopus. |
| s2 | search | ابحث في ملاحظاتي عن سهيل. |
| s3 | search | Find recent notes about PKF. |
| r1 | read | Read the note titled 'PKF Overview' and summarize it. |
| r2 | read | Open the note at astronomy/durur.md. |
| f1 | find_similar | What notes are similar to my 'Suhail and the Bedouin calendar' note? |
| lr1 | list_recent | What did I write in the last 7 days? |
| m1 | multi-step | Search for 'Suhail' then read the top result. |
| m2 | multi-step | أحضر لي أحدث الملاحظات ثم لخص أحدثها. |
| m3 | multi-step | Find notes similar to my pkf.md note and tell me what they have in common. |

All prompts include both English + Arabic to match Fanar's primary language target. Categories cover the four Phase 1 read tools (`search_notes`, `read_note`, `find_similar`, `list_recent`) plus multi-step chains.

---

## Bundled-default recommendation (§4 A revisited)

Phase 0b ships **fanar-1-9b-q4km** as the only installable model (per Eisa's §4 A lock A4). Path A's runtime swap to llama-cpp-2 means the same runtime will host Jais (Phase 2.5 / MIG-050) — no second runtime needed.

The 0-on-tool-calls is a **release-blocker for Phase 1**, not Phase 0b. Phase 0b's job (per Architect §1) was: prove real local inference works end-to-end. **It does** — see `MIG-047-bench-runtime-2026-05-24.md`. Tool-call discipline is exactly what Phase 1 was scoped to deliver.

---

## What Phase 1 (MIG-048) re-runs

When MIG-048 lands real tool-call extraction in `ToolDispatcher` + `local.rs`, this same bench harness re-runs with:

- Real tool-call extraction wired
- Possibly an expanded prompt set (50 prompts as Architect §5 Step H originally called for)
- Comparison against Fanar's instruction-following with vs without GBNF grammar enforcement
- Quality measurement on argument fidelity (currently gated)
- Quality measurement on coherent-reply (gated on round-2 dispatch firing)

The harness in `src-tauri/build_assets/bench_tool_use.rs` is unchanged-friendly: only the runtime's tool-call emission needs to start working.

---

*Bench harness: `src-tauri/build_assets/bench_tool_use.rs`. Architect: `docs/MIG-047-constellation-mind-phase0b-real-inference-ARCHITECT.md` §5 Step H. Phase 0b verify clause closed — with honest documentation of the deferred-to-Phase-1 tool-call extraction.*
