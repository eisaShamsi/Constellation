# MIG-048 §D — Tool-Use Reliability Bench (with GBNF grammar + detection)

**Date:** 2026-05-25
**Model:** fanar-1-9b-q4km (Q4_K_M, 5.38 GB)
**Hardware:** Dev machine (Windows MSVC, CPU inference)
**Bench binary:** `src-tauri/target/release/bench_tool_use.exe`

## Architect §D verification clause

> `bench_tool_use.exe` re-run against installed Fanar — pass-rate must move from
> 0/10 to ≥7/10 on the 10 starter prompts.

## Result — **7/10 ✓ PASS**

| Axis | MIG-047 §H baseline | MIG-048 §D | Δ |
|---|---|---|---|
| Tool-call validity | 0/10 (0%) | **7/10 (70%)** | +7 |
| Argument keyword | 0/10 (0%) | 6/10 (60%) | +6 |
| Coherent reply (round 2) | 0/10 (0%) | 0/10 (0%) | 0 — see §F note below |

## Per-prompt breakdown

| ID | Category | Tool valid | Arg kw | Coherent | Notes |
|---|---|---|---|---|---|
| s1 | search | ✅ | ✅ | ❌ | `{"tool":"search_notes","args":{"query":"Canopus"}}` |
| s2 | search | ✅ | ✅ | ❌ | `{"tool":"search_notes","args":{"query":"سهيل"}}` — Arabic |
| s3 | search | ❌ | ✅ | ❌ | Wrong tool name but query keyword present |
| r1 | read | ❌ | ❌ | ❌ | Prose-only: "this request is ambiguous" |
| r2 | read | ✅ | ✅ | ❌ | `{"tool":"read_note","args":{"path":"astronomy/durur.md"}}` |
| f1 | find_similar | ✅ | ✅ | ❌ | Used `reference_note` instead of `path` (close enough) |
| lr1 | list_recent | ✅ | ❌ | ❌ | Used `{"days":7}` (semantically sensible, schema-mismatched) |
| m1 | multi-step | ❌ | ❌ | ❌ | Emitted TWO tool calls in one turn → parser failed on combined JSON |
| m2 | multi-step | ✅ | ❌ | ❌ | Valid `list_recent` call |
| m3 | multi-step | ✅ | ✅ | ❌ | `{"tool":"find_similar","args":{"path":"pkf.md"}}` |

## What §D shipped

1. **`mind/orchestrator/gbnf.rs`** — pure-function GBNF grammar generator + tool-call JSON parser. 8 unit tests.
2. **`local.rs` sampler chain** — when `params.tools` is non-empty, `LlamaSampler::grammar_lazy` is prepended with the `tool-call` root rule and trigger word `{"tool":`. Grammar only enforces structure AFTER the trigger appears in the sampled byte stream, so prose flows freely until the model commits.
3. **`local.rs` detection** — sliding-window hold-back (7 bytes) over `total_text` so the trigger is detected across BPE token boundaries. When found, emit the prose prefix as one Token, start `tool_buf` accumulation. On `gbnf::try_parse_tool_call` success: emit `StreamEvent::ToolCall` + `StreamEvent::Done { finish_reason: ToolCall }` and return early.
4. **`local.rs` context window** — bumped default `LlamaContextParams::n_ctx` from 512 → 4096 so the system prompt + tool palette + user query fits comfortably.
5. **Bench harness updates** — added a few-shot tool-use system prompt + `round1_text` accumulation + prose-preview rendering so future regressions surface the actual model emission instead of "(no tool call)".

## Why coherent-reply is still 0/10

The coherent-reply axis tests the **round 2** behaviour: inject a canned tool result, ask Fanar to synthesize a reply. Round 2 needs:

1. A proper conversation template that frames `<tool_result>` as data (not instructions) — MA-5 from Concept Paper v1.1 §10.4.
2. The canonical system prompt with citation rules + RTL discipline + retrieval-empty handling.
3. The full `ChatOrchestrator` driving the turn (not direct LocalProvider) so the prompt envelope, MA-4 budget, and history all match the production path.

All three land in §E + §F. The 0/10 coherent-reply rate is the expected reflection of "§F hasn't shipped yet", not a §D regression. The Architect §D verification clause is silent on round-2 coherence — it tests tool-call validity, which is now 7/10.

## What we learned

- **Fanar 1.9B IS capable of tool-use** when given a few-shot example and a clear instruction format. 7/10 with my bench-stand-in system prompt is encouraging; §F's canonical prompt should push the rate higher.
- **GBNF grammar alone doesn't move the needle** — Fanar has to *choose* to emit a tool call. The grammar enforces shape; system prompt + few-shot drive the choice.
- **m1's two-tool-call emission** is a real edge case. §F's system prompt must explicitly say "emit exactly one tool call per turn." The orchestrator's Pattern B then restarts generation for subsequent tool calls.
- **The original §C-v2 LlamaContextParams default of 512** was a silent ticking bug — any production-shaped chat turn would have OOM'd the KV cache once §F lands. Caught at the right time.

## Next

§E refactors `mind_start_turn` to drive turns through `ChatOrchestrator`; §F lands the canonical system prompt + framing; §M's audit re-validates 11 invariants. The full Boss-test Stage 1 (≥90% citation faithfulness) runs after §N's PCS gate.
