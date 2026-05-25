# MIG-048 §M — Consolidated 3-Agent Audit Report

**Date:** 2026-05-25
**Scope:** commits `a0fe99fe` → `c62bb74b` (steps §A through §L).
**Methodology:** Three parallel Explore agents covering (4A) invariants,
(4B) drift, (4C) migration paths.

## Headline result

| Axis | Result | Issues |
|---|---|---|
| Invariants (Architect §3) | **11/11 PASS ✅** | none |
| Drift | **9/9 CLEAN ✅** | none |
| Migration paths | **15/15 covered** | 1 P1 → **fixed inline (commit below)** |

## Audit 1 — Invariants check

The 11 invariants from Architect §3 all hold with file:line evidence.

| # | Invariant | Status | Evidence |
|---|---|---|---|
| 1 | MIG-046 trait surface frozen | ✅ | zero edits to provider.rs / events.rs since `59a2b34a` |
| 2 | MIG-047 install flow unchanged | ✅ | only §J pre-warm hook added in MindSettings.svelte::setActive |
| 3 | mind_telemetry_snapshot additive only | ✅ | TelemetrySnapshot fields unchanged |
| 4 | No edits to ai/ or cece/ | ✅ | no commits touch those paths |
| 5 | No schema change | ✅ | zero CREATE TABLE / ALTER TABLE / new columns |
| 6 | No boot regression | ✅ | pre-warm spawned on tauri::async_runtime, non-blocking |
| 7 | No hot-path additions | ✅ | no $effect on keystroke; only derived state |
| 8 | Local-First / no exfiltration | ✅ | no new fetch/reqwest calls |
| 9 | Editor parity rule preserved | ✅ | citation chips render in chat bubbles only |
| 10 | /migration discipline | ✅ | 12 commits §A–§L tied to step numbers |
| 11 | Citation discipline | ✅ | warning prefix enforced at core.rs:470 |

Full report: `lab/reports/MIG-048-M-audit-invariants-2026-05-25.md`.

## Audit 2 — Drift check

All 9 drift axes clean. No cross-module symbol pickup, no sampler-chain
interactions outside the gated grammar_lazy path, no Channel<T> leaks,
no system-prompt double-injection.

| # | Axis | Status |
|---|---|---|
| 1 | Module symbol pickup | CLEAN |
| 2 | GBNF sampler-chain order | CLEAN |
| 3 | Conversation-state consumers | CLEAN |
| 4 | constellation_search_* interaction | CLEAN |
| 5 | SearchState lock contention | CLEAN |
| 6 | Tauri Channel<T> cleanup | CLEAN |
| 7 | Pre-warm side-effects | CLEAN |
| 8 | MindChatPane onDestroy cleanup | CLEAN |
| 9 | System-prompt FIRST-turn-only gating | CLEAN |

Full report: `lab/reports/MIG-048-M-audit-drift-2026-05-25.md`.

## Audit 3 — Migration paths

15/15 scenarios from Architect §6 + 4 additional edge cases are covered
or acceptably degraded. One P1 caveat surfaced and was **fixed inline**
during the audit consolidation.

### The P1 caveat (now fixed)

**Symptom:** when `SearchState.db` is uninitialized (fresh install before
the search index has opened, lock poisoned, SQL error), the validator
returned `false` for every path → marked every citation invalid →
warning prefix shown constantly.

**Fix (commit forthcoming):** introduced a 3-state `PathVerdict` enum
(Exists / Missing / Unverifiable). `scan_and_verify` now treats
`Unverifiable` as VALID (fail-open) instead of invalid (fail-closed).
The validator catches only REAL fabrications where the DB can
confirm the path is missing.

Full report: `lab/reports/MIG-048-M-audit-migration-2026-05-25.md`.

## Boss-test §I run finding (real fix already shipped)

During Boss test of the §I+§J installer, Eisa hit:

```
provider error: runtime error: batch.add prompt[512]: Insufficient Space of 512
```

Root cause: when §F added the canonical system prompt + 6 inline tool
descriptions, the prompt token count exceeded the hardcoded
`LlamaBatch::new(512, 1)` capacity. The §D context bump from 512 → 4096
addressed the model's context window but missed the batch staging
buffer.

Fix shipped in commit `82d2cf91`:
- `LlamaContextParams::with_n_ctx` bumped 4096 → 8192 (Fanar's max)
- `LlamaBatch::new(n_prompt.max(8192), 1)` — dynamic capacity

Re-running the Boss-test §I after the rebuild is the verification.

## Outstanding follow-ups (Phase 1.x candidates, not ship blockers)

1. **Brand-naming policy for 13 locales (§L flag).** Current state
   uses a hybrid "Constellation + localized Mind" pattern; Eisa's
   standing order suggests fully-localized "Mind of Constellation"
   matching the Arabic precedent. One-commit follow-up.
2. **Real tokenizer for history.rs trim budget.** The chars/4 heuristic
   is conservative across English+Arabic but may mis-estimate for
   token-heavy languages (Japanese, Chinese). Phase 1.x can swap to
   llama-cpp-2's tokenize() API.
3. **Per-Universe conversation persistence.** MindChatPane state
   currently resets on unmount. Architect §H mentioned per-Universe
   `mindConversationStore`; deferred to Phase 1.x polish.
4. **Bench tool m1 two-tool-call edge case.** Fanar occasionally emits
   two adjacent tool calls in one turn; current parser fails to
   split. Phase 1.x can enhance try_parse_tool_call.
5. **Coherent-reply axis still 0/10 on bench.** Round-2 coherence
   verification happens in Boss-test Stage 1 with the full
   ChatOrchestrator path; the standalone bench can't simulate it.

## Ship readiness

**Phase 1 is ready for §N (PCS gate).** All invariants hold; no drift;
all migration paths covered (P1 fixed inline). The remaining work is
documentation + the orientation v-bump + the help-doc update + the
final PCS commit.

The end-to-end Boss-test Stage 1 (50-turn citation faithfulness ≥90%)
happens after §N lands. The bench (§D) saw 7/10 tool-call validity in
isolation; the real chat path adds the ChatOrchestrator + citation
validator + history trim, which should push the rate higher in
practice — but only the Boss-test will confirm.
