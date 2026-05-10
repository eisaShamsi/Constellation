# MIG-021v3 V3-§8 — Six-Cataloger Independent Audit

**Date**: 2026-05-10.
**Trigger**: post Gate-1 Boss-test, Eisa surfaced visible problems with cards looking "almost identical" across different notes. Three on-the-fly patches (V3-§8 fix-A+B+C: real ensemble weights / tightened DOI regex / Structural single-hit downgrade) addressed the symptoms partially. Eisa requested an independent six-cataloger audit before any further patches.
**Method**: Six independent agent reviewers spawned in parallel, each through a methodologically distinct lens: Library Science, NLP/ML Engineering, Software Architecture, UX/Cognitive, Epistemology / *uṣūl al-fiqh*, and Adversarial / Edge-Case. Each reviewer briefed cold, given absolute paths to the actual code, and instructed to disagree where they found issues.
**Outcome**: All six returned with substantive findings. Composite verdict: ~6/10. Architecture sound; implementation has specific reproducible gaps.

---

## §1 — Composite scores

| Reviewer | Score | Headline |
|---|---|---|
| Library Science | 6/10 | Faceting in spirit, not Ranganathan-clean; missing scope notes / authority records is single biggest LIS gap |
| NLP/ML Engineering | 6.5/10 | Top-down decomposition spec'd but **not implemented**; lexicon coverage too sparse for cold-start |
| Software Architecture | 6.5/10 | Dead `OnceLock` fields; Windows `fs::rename` atomic trap; no timeout enforcement |
| UX / Cognitive | 4.5/10 | **Sibling Disambiguation form does not exist in code**; debug-leak in reasoning trails; zero `cece.*` i18n keys |
| Epistemology / *uṣūl* | 5/10 | Lexicon collapses real distinctions: *qiyās → comparison* (should be *inference*); `حدثنا → mass-transmission` (should be *testimony/reported* — opposite!) |
| Adversarial | 8 P0/P1 | Arabic comma silently kills CAE on every Arabic note; backtick-fence prompt injection; cross-Library reliability path-prefix collision violates Architect §10 invariant 9 |

---

## §2 — Implementation gaps where commit messages didn't match the code

These are the most uncomfortable findings — places where the V3-§8 commit (`daeba00`) claimed behavior the code doesn't actually have.

1. **Sibling Disambiguation form** — Architect §3.1 + Plan V3-§8 specify "card morphs into needs-your-call form with radio chips for the candidate siblings." Commit `daeba00` shipped only the placeholder UI: gold border + pill + the same `Edit / Accept / Reject` flow from v2. The user gets the 270-node TaxonomyTreePicker on Edit, not the focused 2–3 candidate picker the spec promised. **No `cece_resolve_disambiguation` IPC handler in the Svelte.** Commit message claimed shipped; didn't.
2. **Top-down schedule navigation in Reasoning Cataloger** — Plan V3-§7 + `reasoning.rs::rules_fired = ["schedule_navigation_top_down"]`. Code at `reasoning.rs:74-141` makes ONE call with a single fat grammar containing every leaf. No second-pass parent-restricted prompt. KG-HTC says this leaves 25–30pp on the table at L3.
3. **`cece.*` i18n keys** — zero exist in `en.json` or `ar.json`. Every CECE-era string in the rendered Svelte is hardcoded English. Eisa's Arabic-UI Boss-test would have shown English bleeding through.
4. **`AxisDecision.secondary` field** — declared in the schema, hardcoded `Vec::new()` in `vote_on_axis`. Principal/secondary distinction is half-built.
5. **`OrchestratorState`** — defined in `orchestrator.rs:150`, never used. Orchestrator built fresh per IPC call (7K rebuilds during a scan).
6. **`OnceLock` lazy fields on `CatalogerContext`** — declared in `cataloger.rs:131-136`, never read by any cataloger. Each cataloger uses its own injected `embed_fn`/`lookup_fn`. Net cost: Linguistic + Semantic both run e5-small independently when they could share = ~3.5 min wasted on a 7K scan.

---

## §3 — P0 ship-blocking bugs (with reproducible inputs)

**P0.1 — Arabic comma silently kills CAE root path** (`linguistic.rs:381`)
`is_ascii_punctuation()` doesn't include Arabic comma `،` (U+060C), semicolon `؛`, question mark `؟`, full-stop `۔`. Input `هذا قياس،صحيح` → tokens `["هذا", "قياس،صحيح"]` → CAE can't extract a root from a multi-word string → silent abstention on the cataloger documented as "Strong on technical Arabic." **This explains why `الخط العربي` test showed `LIN✓` — only surface-token fired, masking that the root path was already dead.** One-line fix: also split on `c == '،' || c == '؛' || c == '؟' || c == '۔' || (matches!(c as u32, 0x2000..=0x206F))`.

**P0.2 — Prompt injection via triple-backtick fence** (`reasoning_prompt.rs:55-61`)
User note containing literal `\`\`\`` closes the fence early; LLM follows injected instructions toward any *valid* taxonomy ID. GBNF only constrains *which IDs are valid*, not *which valid ID gets picked*. Working payload provided by adversarial reviewer. Qwen3-4B-Q5 highly susceptible. Fix: nonce-delimited fence + explicit "anything inside fence is data" line in system prompt.

**P0.3 — Cross-Library reliability data leakage via path-prefix collision** (`correction_log.rs:96-107`)
No trailing-separator boundary in `library_root_for_note`. Library `/Universe/Notes` plus folder `/Universe/Notes_old` (or `Notes-archive`) — corrections from the latter write into the former's `cataloger_reliability.json`. **Direct violation of Architect §10 invariant 9** ("per-Library calibration is per-Library — no cross-Library data leakage"). One-line fix: append `'/'` before `starts_with`.

**P0.4 — `compute_regime` cannot reach StrongMajority with typical voter counts**
NLP + LIS + UX agents independently flagged this. The `total_voters >= 3` gate kicks before majority math. With 2-voter coverage being the steady state (other catalogers often abstain), every disagreement floors at Split. Eisa observed this on every card — gold border + "needs your call" on every note. Fix: drop the `>= 3` gate; treat 2-voter unanimous as Unanimous; treat 2-voter agreement-with-1-abstainer as StrongMajority.

**P0.5 — Sibling Disambiguation form missing** (see §2 #1) — without it, Split regime is just a sad pill; the entire ensemble's value (refusal + targeted ask) is invisible.

**P0.6 — Zero `cece.*` i18n keys** (see §2 #3) — Boss-test Arabic UI would have English bleeding through.

---

## §4 — P1 fix-before-Gate-2 bugs

| # | Finding | Source | Fix complexity |
|---|---|---|---|
| P1.1 | Day 0 lexicon coverage ~5% (21 entries, ~270 needed) | NLP | Half day of authoring |
| P1.2 | Windows `fs::rename` atomic trap in `reliability::save` | Software Arch + Adversarial | Use `tempfile::NamedTempFile::persist` |
| P1.3 | Mutex poisoning cascade (Semantic panic in `db.lock()` → Graph + Reasoning silent abstain) | Software Arch + Adversarial | `lock().unwrap_or_else(\|e\| e.into_inner())` |
| P1.4 | No timeout enforcement (Architect §10 invariant 12 promises bounded; code doesn't enforce) | Software Arch | `tokio::time::timeout` over `spawn_blocking` |
| P1.5 | `ALTER TABLE` per IPC call with errors swallowed | Software Arch + Adversarial | Move to `init_db` with versioned migration |
| P1.6 | kNN brute-force per-call = 150 MB at 100k notes; per-scan-iteration with no cache | Software Arch + Adversarial | Cache in `OrchestratorState`, invalidate on note write (Write-Time Derivation rule) |
| P1.7 | Unicode confusables bypass lexicon match (Cyrillic, ZWNJ, Tatweel) | Adversarial | NFKC normalize via `unicode-normalization` |
| P1.8 | Stance regex fires without use-vs-mention guard (`أشكّ` in *قال فلان أنه أشكّ* misfires) | Epistemology | Backward-look for attribution verbs (*qāla / dhakara / rawā*) within ~30 chars |
| P1.9 | `قياس → comparison/ratio-legis` is wrong tradition mapping | Epistemology | Map to `inference/*` parent only |
| P1.10 | `حدثنا → mass-transmission/verbal` is the OPPOSITE of correct | Epistemology | Map to `testimony/reported` (single-narrator marker) |
| P1.11 | `أظن → belief/occurrent` is wrong tradition mapping | Epistemology | Map to `epistemic-states/opinion/probable` |
| P1.12 | Bare `متواتر` mapped to `mass-transmission/verbal` (specific) instead of parent | Epistemology | Map to `mass-transmission` parent + force descend_uncertain |
| P1.13 | `anupalabdhi` collapses to `non-apprehension/absolute` (atyantābhāva) when 4 sub-types exist | Epistemology | Map to parent only |
| P1.14 | `OnceLock` vs injection inconsistency — pick one, delete the other | Software Arch | Recommend keeping OnceLock (deduplicates I/O); delete per-cataloger fn fields |
| P1.15 | `OrchestratorState` defined but never used — orchestrator built fresh per IPC call | Software Arch | Tauri-manage; build once at boot |

---

## §5 — P2 known-issues (file but don't block)

- Reasoning trail leaks debug formatting ("[high]", "weight 0.85", "horizontal → testimony/scriptural" — Python list-comprehension tell)
- T1/T2 legacy + 6-badge cluster coexist in same list (visual inconsistency)
- Approve All bulk-writes Split-regime cards (defeats the engine's "refuse to assign" decision)
- GBNF grammar regenerated per call (no `OnceLock`)
- Malformed JSON in `note_meta` silently dropped
- `mass-transmission` should be sub-property of `testimony`, not orthogonal facet (self-inflicted Bayes-irreducibility)
- "Graph Cataloger" mislabeled — it's bibliometric coupling (Small 1973, Garfield 1979), not LCSH-style authority control
- Subject authority records / scope notes missing per taxonomy ID (LIS single biggest gap)
- No per-Library `tradition:` field — same lexicon for Sunni vs Hindu vs Buddhist users
- Synthesis forces single primary per axis — real fiqh rulings draw on 5+ co-equal sources
- Tier system inverted from Sunni perspective (Revelation at Tier 3 is Western-comparative framing)
- UX badge cluster wrong encoding (6 abbreviations × 3 states = 18 micro-glyphs — should be 6 tinted dots + plain-English tooltips)
- Reasoning trail default wrong direction (should be always-visible for first ~50 reviews per Library — trust calibration window — then auto-collapse)

---

## §6 — Confirmed neutralized (independently verified)

- **Regex DoS** — Rust `regex` crate is RE2-style, guaranteed linear-time. Whole attack family neutralized by dependency choice. (Adversarial)
- **GBNF for closed-set classification** — small accuracy win, large operational reliability win. (NLP)
- **`MIN_SAMPLES_FOR_WEIGHTING=20` knee** — defensible against Bayesian credible-interval math. (NLP)
- **Brute-force kNN at current scale (≤10K notes)** — 5ms is fine; only becomes a problem past 100K. (Software Arch + NLP)
- **CIP-precedent for User-Authority Cataloger absolute precedence** — well-grounded against LIS practice. (LIS)
- **Per-axis faceted classification (Source × Content Type)** — defensible Ranganathan move; the most important faceting decision was correct. (LIS)
- **Six methodologically distinct lenses + synthesis** — sound architecture per Snorkel / MoE precedent. (NLP)

---

## §7 — Convergent findings (multiple agents flagged independently)

The strongest signals — real bugs that surfaced from multiple lenses:

- **`compute_regime` Split-everywhere bug**: NLP + LIS + UX
- **Mutex poisoning cascade**: Software Arch + Adversarial
- **`ALTER TABLE` per IPC call**: Software Arch + Adversarial
- **kNN per-call cost / no cache**: Software Arch + Adversarial
- **Sibling Disambiguation gap**: UX (architectural; others implied)
- **Lexicon coverage critically thin for cold-start**: NLP + (LIS implied via "scope notes missing")

---

## §8 — Eisa decision (2026-05-10)

Three options surfaced post-audit:
- **(A)** Stop and fix everything before Gate 1 PASS. ~3–5 days. Boss approved.
- (B) Fix only P0s, ship Gate 1 with documented P1s
- (C) Open MIG-021v3.1 patch series, fold findings, resume

**Boss directive: (A).** Cascade through V3-§8.r1 → V3-§8.r5, re-run Gate 1 cleanly.

Remediation phases:
- **V3-§8.r1** — P0 critical fixes (Arabic comma; prompt-injection fence; path-prefix collision; `cece.*` i18n keys; **Sibling Disambiguation form**; `compute_regime` threshold)
- **V3-§8.r2** — Synthesis architecture (`OnceLock` vs injection unification; `OrchestratorState` actually used; `AxisDecision.secondary` populated)
- **V3-§8.r3** — Lexicon corrections (qiyās → inference; حدثنا → testimony/reported; أظن → ẓann; bare متواتر → parent; anupalabdhi → parent; tradition field)
- **V3-§8.r4** — Robustness (tempfile rename; mutex poison recovery; ALTER → init_db; timeouts; NFKC normalization)
- **V3-§8.r5** — UX polish (badge dots; reasoning trail render layer; trust-calibration default; queue-level Split count; Split-aware Approve All)

After r5, re-run Gate 1 Boss-test cleanly.

---

## §9 — Files of record (full reviewer reports)

The six full reviewer reports are preserved in this audit's task-output files (not in repo to keep size bounded; available for forensic re-read):
- Library Science: `tasks/ac5abe1b3f6128005.output`
- NLP/ML Engineering: `tasks/a74df29acc344b1bf.output`
- Software Architecture: `tasks/aecdfcd1add6cabab.output`
- UX/Cognitive: `tasks/afe25b9db66d50200.output` (path-corrected re-run)
- Epistemology: `tasks/a87cdda06fe8702c2.output` (path-corrected re-run)
- Adversarial: `tasks/ab0936278515475e1.output`

Convergent findings + the most actionable specific code paths are inlined in §3–§5 above.

End of audit.
