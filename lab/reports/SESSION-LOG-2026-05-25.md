# Session log — 2026-05-25

## MIG-048 Phase 1 — Read-only conversational RAG SHIPPED (§A–§N)

Phase 1 of the Constellation Mind Implementation Plan v1.0 landed in
one cascade. 12 commits between `a0fe99fe` and `[§N]` cover the full
chat surface end-to-end: real tool dispatcher, GBNF-constrained tool-
call emission, ChatOrchestrator-driven turns, canonical system prompt
with MA-5 framing, citation validator with 1-retry, four Svelte chat
components, sidebar mode integration, app-start pre-warm, sliding-
window history trim, 15-locale i18n, 3-agent audit + P1 fix.

Eisa-locked §9 decisions before cascade started:
- **D1** — chat is a sidebar mode in the left dock (not a right-sidebar
  panel, not a floating window).
- **C1** — citation validator gets 1 retry, then a warning prefix.
- **E2** — sliding-window history trim (overrode E1 Fanar-summarize).
- **Boss-test Stage 1 universe** — "Eisa Cognitive Knowledge".

## Step-by-step commits

| Step | Commit | What |
|---|---|---|
| Architect | `9cd41e4a` | MIG-048 Phase 1 Architect doc, §9 locks recorded |
| §A | `a0fe99fe` | RealToolDispatcher + 4 ready tools (search_notes / read_note / find_similar / summarize). Orchestrator promoted to a module dir. 41 mind tests pass. |
| §B | `853dac1b` | constellation_search_recent + constellation_graph_neighbors pub fns in search.rs. 3 new tests. |
| §C | `a2f28b4f` | list_recent + graph_neighbors tools wired. Dispatcher covers all 6 read tools. 9 orchestrator tests. |
| §D | `a282d1e4` | GBNF tool-call extraction via LlamaSampler::grammar_lazy. Closes the §H 0/10 gap from MIG-047. Bench: 7/10 tool-call validity. 49 mind tests. |
| §E | `cb642111` | mind_start_turn refactored to drive through ChatOrchestrator. UiEvent gains args + finish_reason + Error variant. |
| §F | `294b21ea` | Canonical system prompt + tool_result framing (`<tool_result>` tags). 56 mind tests. |
| §G | `cfa10802` | Citation validator + 1-retry loop. Closure-based hook avoids tauri::AppHandle in orchestrator struct (DLL-load gotcha discovered the hard way). 62 mind tests. |
| §H | `a08a3d75` | Four Svelte chat components: MindChatPane / MindChatMessage / MindCitationChip / MindToolCallLog. en + ar i18n. |
| §I | `f665e937` | Chat sidebar mode wired into +layout.svelte (Eisa-locked D1: left dock). |
| §J | `909bd8aa` | Pre-warm active model on app start (via .setup hook) + on active-model change. First-turn latency: ~9-11s → ~1-1.5s. |
| §K | `d71642f0` | Sliding-window history trim (E2). chars/4 heuristic, 6500-token budget. 68 mind tests. |
| **Boss-test §I** | `82d2cf91` | **FIX** — `LlamaBatch::new(512, 1)` overflowed on real prompts with §F's system prompt. Bumped n_ctx 4096→8192 + batch capacity to 8192. Hit during Eisa's first real Boss test. |
| §L | `c62bb74b` | 13-locale i18n fill (de/es/fa/fr/he/hi/ja/ko/pt/ru/tr/ur/zh). Brand-naming policy flagged for Phase 1.x. |
| §M | `4e978076` | Consolidated 3-agent audit + P1 caveat fix (validator now fails OPEN when DB unavailable). |
| §N | `[N commit]` | This file + orientation v2.33 + 15-locale help-doc update + PCS gate. |

## Architectural surprises absorbed during cascade

1. **mistral.rs gemma2 panic** — actually inherited from MIG-047 §C-v2;
   not a §A–§L surprise. Mentioned for completeness in §F's commit.
2. **GBNF eager grammar crashed llama.cpp's grammar engine**
   (`GGML_ASSERT(!stacks.empty())`). Swapped to `grammar_lazy` with
   trigger word `{"tool":` — same outcome, no crash.
3. **`tauri::AppHandle` as struct field broke test binary DLL load**
   (STATUS_ENTRYPOINT_NOT_FOUND). Switched the citation_validator
   wiring to a closure-based hook (orchestrator never holds AppHandle).
4. **LlamaBatch capacity of 512 overflowed** with §F's system prompt
   in a real chat turn. Caught by Eisa's first Boss test §I; fixed
   inline.

## Bench result (§D)

`bench_tool_use` against installed Fanar:
- Tool-call validity: **7/10 (70%)** ✓ ≥ Architect's ≥7/10 target
- Argument keyword present: 6/10
- Coherent reply (round 2): 0/10 — Phase 1.x will measure this in
  Boss-test Stage 1 with the full ChatOrchestrator path.

Report: [MIG-048-D-bench-tool-use-2026-05-25.md](MIG-048-D-bench-tool-use-2026-05-25.md).

## Audit result (§M)

- Invariants: **11/11 PASS**
- Drift: **9/9 CLEAN**
- Migration paths: **15/15 covered**, 1 P1 (fail-CLOSED validator) **fixed inline**.

Reports: [MIG-048-M-audit-consolidated-2026-05-25.md](MIG-048-M-audit-consolidated-2026-05-25.md)
+ three per-agent reports in the same directory.

## Outstanding Phase 1.x polish items (not ship blockers)

1. Brand-naming policy for 13 locales — current "hybrid" pattern vs the
   stricter Arabic-aligned full localization.
2. Real tokenizer for history.rs trim budget (currently chars/4).
3. Per-Universe conversation persistence (MindChatPane state resets
   on unmount today).
4. m1 two-tool-call edge case (Fanar occasionally emits two adjacent
   tool calls in one turn).
5. Round-2 coherence rate (measured in Boss-test Stage 1).

## What's next after PCS

**Boss-test Stage 1** (Plan §4 Phase 1 ship gate): 20-turn Arabic
conversation on Eisa Cognitive Knowledge universe; Eisa reads a 50-turn
sample; target ≥90% citation faithfulness. If passed, Phase 1 closes
fully and Phase 2 (MIG-049 — write tools + approval modal) can begin.

---

## SAME-DAY REVERT — Constellation Mind REMOVED at Eisa's call (a9cf4d62)

After the §N Phase 1 SHIPPED commit landed and Eisa Boss-tested the NSIS
installer (the rebuilt version that included the n_ctx + batch fix +
tool-result caps), the verdict landed:

> *"My general question: If the LLM is using the search_note tool, then
> what is special about it? I can search if I want to. It is NOT worth it."*
> *"Revert back to the prior LLM design and build. Constellation will
> not have it at this stage. Clean and do the necessary housekeeping."*

The Phase 1 plumbing WAS correct (chat tab visible, tool call dispatched
via the real RealToolDispatcher, tokens streamed through the
ChatOrchestrator → bridge → Channel<StreamEvent>, citation discipline
machinery operated). But Fanar 1.9B at 5 tok/s CPU made every turn
~70 seconds; an LLM-mediated search on top of the existing SearchHub
didn't earn that cost. The cross-language synthesis case
(Canopus ↔ سهيل) — where an LLM would have visibly outperformed
lexical search — wasn't yet plumbed into `search_notes`'s tool wrapper.
The app also crashed on the third Boss-test turn (context-window edge
despite the §N follow-up fixes).

### What the revert deleted

85 files changed, −13,442 / +5,194 lines (the lopsided ratio: ~30
MIG-046/047/048 commits collapsing into one revert). Scope:
- `src-tauri/src/mind/` — entire module
- `src-tauri/build_assets/bench_runtime.rs` + `bench_tool_use.rs`
- `src-tauri/resources/models.json`
- `src-tauri/Cargo.toml` — async-trait / sha2 / hex / llama-cpp-2 deps;
  `bench_*` `[[bin]]` entries
- `src-tauri/tauri.conf.json` — `bundle.resources` reverted to
  `["models/*"]`
- `src-tauri/src/lib.rs` — `pub mod mind`, 8 mind IPC commands,
  `.setup()` pre-warm hook
- `src-tauri/src/search.rs` — `constellation_search_recent` +
  `constellation_graph_neighbors` + `mig048_tests`
- 5 `Mind*.svelte` components
- `src/routes/+layout.svelte` — chat sidebar mode + button + mount
- `src/lib/secondScreen.ts` — `'chat'` from `SidebarMode`
- `src/lib/components/SettingsModal.svelte` — Mind section
- 15 i18n JSON files — all `mind.chat.*` / `settings.mind.*` keys
- `.github/workflows/model-pipeline.yml`
- MIG-046 / 047 / 048 Architect docs + Concept Paper + Plan
- 15-locale help-doc Constellation Mind topic

### Out-of-tree cleanup (Eisa-authorized)

- `%APPDATA%\world.uconstellation.app\models\fanar-1-9b-q4km-v1.gguf`
  (5.13 GB) — removed.
- `installed_models.json` registry — removed.
- GitHub Release `models-fanar-1-9b-q4km-v1` + git tag + 4 split-chunk
  artifacts — deleted via `gh release delete ... --cleanup-tag`.

### What was KEPT (history trail)

- This session log + all MIG-046/047/048 bench + audit reports.
- `docs/MoCh/MoCh-2026-05-2{3,4,5}*.md` — conversation history.
- `docs/Constellation Orientation & Onboarding v2.30.md` through
  `v2.33.md` — immutable per SO #6.
- `ai/mod.rs`, `cece/`, `embeddings.rs`, `nsc/` — pre-Mind surfaces,
  untouched throughout the MIG-046/047/048 cascade by design.

### What's new

- `docs/Constellation Orientation & Onboarding v2.34.md` — preamble
  documents the revert + 6 lessons preserved for any future Mind
  re-attempt:
  1. GPU acceleration is the gating issue (5 tok/s CPU isn't viable
     for interactive chat).
  2. Cross-language synthesis is the value-add to lead with — not
     plain keyword lookup.
  3. `tauri::AppHandle` as struct field breaks test-binary DLL load
     (`STATUS_ENTRYPOINT_NOT_FOUND`).
  4. GBNF eager grammar crashes llama.cpp on `prose | tool-call`
     alternation — use `grammar_lazy` with trigger word.
  5. `LlamaBatch::new(N, 1)` capacity must track `n_ctx`; default 512
     overflows on any real prompt with a system message + tools.
  6. Tool result payloads need defense-in-depth size caps (per-tool
     AND dispatcher-level) to avoid round-2 context overflow.

### Verified clean

- `cargo check --lib` — 0 errors, 42 warnings (all pre-existing
  dead-code in non-Mind code).
- `svelte-check` — 3 errors (all pre-existing: `LinkLifecycle.fresh`
  in store.ts, 2 PropertyEditor union-type issues); 0 new.

### Strategic next direction

Mind is shelved. The Architect-Plan-Build-Audit machinery proved
itself even though the feature didn't ship — that's the meta-takeaway.
Next migration target pending Eisa's direction.
