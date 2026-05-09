# Session Log — 2026-05-09

**Phase:** Sight v.next concept-validation pass (NO code shipped)
**Result:** v1.75 orientation bump; Sight foundation laid; MIG-021 Architect gated on Eisa's three LLM picks.

## Context inherited from v1.74

The morning v1.74 commit (`547e558`) closed the previous session with: Eisa shelved Sight v4 implementation after the close button failed test, and directed: *"I want to start fresh. I want you to discard what has been developed so far. I want you to go to the basics. Let's validate and confirm the 'Constellation Sight Concept'."* This session opened against that directive.

## What this session did, in order

### Phase 1 — Read everything before opining

Eisa asked: have you read every orientation version + every session log? Honest answer: no, only the v1.74 preamble that came in via CLAUDE.md. He directed exhaustive reading. Spawned 7 parallel agents:

- 5 orientation chunks: v1.0–v1.15, v1.16–v1.30, v1.31–v1.45, v1.46–v1.60, v1.61–v1.74 (each agent read 14–16 versions cover-to-cover)
- 2 session-log chunks: 2026-03-27 → 2026-04-20, 2026-04-21 → 2026-05-08 (each agent read 13–18 daily logs cover-to-cover)

Total: 122,192 lines of orientation + 14,519 lines of session logs distilled into seven structured reports. Surfaced:

- The full Sight identity arc (Lens v0 → Lens v1 → Sight v1 → Sight v2 → Sight v3 → Sight v4 — five identities, not three)
- The "five core functions" canonical rule from 2026-04-13 (*"Each of the five core functions — Search Hub, OrgChart, Sky View, Map, Sight — must complement, not overlap, the others within Cognitive Knowledge / Knowledge Formulation"*)
- The 360.3D Stratification Matrix precedent: 5 failed iterations → LL-014 invoked → Concept Paper first → clean rebuild. Same pattern Eisa wants to apply to Sight now.
- The "what Sight is NOT" line was never written. The 360.3D Concept Paper has it; the three Sight Concept Papers don't.

### Phase 2 — Eisa ratifies the four foundational decisions

Eisa's four answers (verbatim):

1. **Delete `lenses.rs::apply_lens`.**
2. **Sight's one answer: "How are my Epistemic Content shaped and/or organized?" (so, forget about the InfraNodus.)**
3. **The focus of the 360.3D is the Note, while Sight is the whole universe.**
4. **Why was V2 not enough? No. If future Constellation users don't understand it or think it is difficult, then its existence is unnecessary.**

Memory written: `project_sight_canonical_answer.md`, `project_sight_360_scope_orthogonal.md`. Updated: `project_lenses_apply_lens_dead_code.md` (decision flipped to DELETE).

### Phase 3 — Visual mockups

Eisa: *"Provide me first with a mock-up of each option, I will decide later."* Wrote two SVG mockups in `docs/`:

- `Sight-vNext-MockA-Dashboard.svg` — six-panel distribution dashboard (Strata / Maturity / Stages / Confidence / Link Types / Acts), no metaphor, ~3-second first-sight read.
- `Sight-vNext-MockB-Metaphor.svg` — night-sky chart re-anchored: radius = Strata, azimuth = Time, size = Maturity, brightness = Confidence, red = Contested. Same six dimensions encoded as one image.

Both 1400×900, Suwaidi cream-parchment palette, identical synthetic data (trial Universe's actual link-type distribution: 43.9% derives-from, 42.9% supports, etc.).

WA #2 violation caught: I wrote the SVGs into the worktree at `.claude/worktrees/...` instead of the official tree. Eisa flagged: *"This is the official working Tree (E:\مشاريع كلاود\Constellation)."* Files copied to official tree; workflow corrected for the rest of the session.

### Phase 4 — Eisa picks Mock B; questions multi-mode

Eisa picked Mock B but asked: *"It has to include the other modes, not only the time mode. Or you have a different opinion?"*

I wrote up my opinion: keep all six modes BUT make strata the constant radius across all modes (only azimuth changes per mode). This REVOKES the v3 visual spec's "per-mode (X, Y, Z)" grammar where each mode declared its own radius/azimuth/magnitude. Reasons: spatial memory survives mode switches; "shape vs organize" maps cleanly (radial = shape, wedges = organization); cross-surface coherence with 360.3D Stratification Matrix (which also anchors per-strata).

Eisa requested two more mockups: (a) toggle bar visible + one mode active; (b) two modes side-by-side. First write was a combined mockup (misread); rewrote as two separate files:

- `Sight-vNext-MockB1-Toggle.svg` — single dome (Time mode) + 6-button toggle bar at top
- `Sight-vNext-MockB2-Compare.svg` — two domes side-by-side (Time | Regions), demonstrating wedge re-slicing while strata rings stay constant

Eisa asked: which do I prefer? Recommended Mock B1 for production (single mode at a time matches the diagnostic-instrument metaphor; Mock B2 stays as help-doc figure only). Eisa: confirmed B1.

### Phase 5 — Eisa attaches the Universal Epistemic Content Taxonomy

Eisa attached three documents from `Downloads/`:

- `epistemic-content-EN.md` — comparative civilizational survey of "epistemic content" across five traditions (Greek + Western analytic; Sunni Islamic *kalām* / *uṣūl al-fiqh* / *falsafa*; Indian *pramāṇa-vāda*; classical Chinese Mohist / Confucian / Daoist / Neo-Confucian; Persian-Islamic Ishrāqī) plus Jewish / Tibetan Buddhist / African / Mesoamerican supplementary.
- `epistemic-content-taxonomy.md` — formal two-axis taxonomy: 5 vertical branches (Sensory / Symbolic / Semantic / Epistemic States / Higher-Order Constructs) × 11 horizontal sources (Perception / Inference / Testimony / Mass-transmission / Comparison / Postulation / Non-apprehension / Memory / Innate disposition / Inspiration / Revelation), with bilingual EN+AR labels and cross-civilizational anchors.
- `epistemic-content-taxonomy-chart.html` — interactive 5-level chart implementation, self-contained, bilingual.

Eisa later attached `epistemic-content-AR.md` — the Arabic version of the survey.

Asked for opinion on how this empowers Sight. Three concrete payoffs identified:

1. Replaces InfraNodus as Sight's scholarly spine (cross-civilizational neutrality matches Constellation's RTL-first / language-agnostic-by-design principle).
2. **Strata is already the Constellation projection of the 5-branch taxonomy** condensed by epistemic elevation. The strata-as-radius design is doubly justified.
3. Opens a new dimension Constellation hasn't tracked: **Sources of Knowledge (provenance metadata)** — the horizontal axis of the taxonomy.

Eisa's four-decision response:
1. Adopt taxonomy as scholarly foundation: APPROVED.
2. Keep UI plain: APPROVED.
3. Track sources as future PJ: NO — track them today.
4. Save the three files into `docs/`: APPROVED.

Memory: `project_sight_taxonomy_foundation.md`. Files copied to `docs/`.

### Phase 6 — Six Sources sub-decisions

Sources-today changed scope from "frontmatter field" to a real subsystem. Six questions surfaced; Eisa answered:

1. Which sources ship Day 1 → **All 11.**
2. Single or multi-source per note → **Multi-source.**
3. Default for the 7,636 trial-universe notes → **"Build a tool that reads notes, categorizes them by the 11 sources, and adds frontmatter with the user's permission."** This created an Epistemic Classifier subsystem.
4. UI for setting source → "Based on Q3" → designed as PropertyEditor combobox (manual) + new "Source Review" sidebar panel (queue-based approval) + right-click "Suggest sources for this note."
5. Storage → **Both frontmatter + `note_meta` SQLite column** (matches MIG-014 Strata/Maturity/Stage pattern).
6. Bilingual labels → **Locale-driven across all 15 locales** (not just EN+AR).

Memory: `project_sight_classifier_local_llm.md`.

### Phase 7 — Classifier strategy

Eisa: **(B) LLM-based, locally.** Three sub-decisions opened (model / inference engine / bundling). Per Working Agreement #5, surfaced these as research-first decisions rather than guessing.

### Phase 8 — Three parallel research agents

Spawned in background. All three completed in-session:

- **Model selection** → top recommendation **Qwen3-1.7B Q4_K_M** (~1.1 GB, Apache 2.0, first-class Arabic, 25–45 tok/s CPU). Runner-up: Gemma 4 E2B if Arabic eval favors it. Disqualified: Llama 3.2 (Arabic not in supported 8 languages), Phi-mini (English-dominant), Gemma 3 (license risk).
- **Inference engine** → top recommendation **llama.cpp via `llama-cpp-2` Rust bindings**. Critical reason: GBNF grammar-constrained decoding guarantees valid JSON output for the 11-source classification. Keep ORT for embeddings; the two engines coexist behind a single Rust module. ONNX Runtime alone for LLM is technically possible but practically a poor fit (no Rust bindings for `onnxruntime-genai`, no GBNF, slower CPU INT4).
- **Bundling strategy** → top recommendation **hybrid**: bundle small ~100–250 MB classifier in the .exe (Sight works Day 1, no network) + optional Settings → AI download for the larger ~1.5 GB model (better Arabic accuracy). Smart Connections precedent (2M+ Obsidian installs).

Condensed into single decision-matrix doc: `lab/reports/MIG-021-LOCAL-LLM-RESEARCH.md`.

### Phase 9 — PCS

Eisa interrupted: *"Don't forget to PCS, including the orientation file."*

This commit:

- 4 SVG mockups in `docs/`: MockA-Dashboard, MockB-Metaphor, MockB1-Toggle (PRODUCTION), MockB2-Compare (help-doc).
- 4 foundation docs in `docs/`: `epistemic-content-EN.md`, `epistemic-content-AR.md`, `epistemic-content-taxonomy.md`, `epistemic-content-taxonomy-chart.html`.
- 1 research summary in `lab/reports/`: `MIG-021-LOCAL-LLM-RESEARCH.md`.
- 1 orientation bump: `Constellation Orientation & Onboarding v1.75.md` (preamble-only update; body sections §3 / §4.x / §13 / §17 deferred to v1.76 when Concept Paper v2.0 lands).
- 1 session log: this file.

5 memory files written/updated (live in `~/.claude/projects/.../memory/`, not in repo).

## Help docs / User Manual

**No user-facing changes shipped.** Sight v4 is still the user-visible build on `main`. The new direction is documented but not implemented. Help docs and User Manual updates land WITH Sight v.next implementation, not before. This is honest: nothing user-facing changed today, only the project's internal direction.

## What's open for next session

1. Eisa picks from the three LLM sub-decisions (`lab/reports/MIG-021-LOCAL-LLM-RESEARCH.md`) — top recommendations exist; Eisa may agree-with-all-three or override per-decision.
2. Draft `docs/Constellation-Sight-Concept-Paper-v2.0.md` (taxonomy-spined, 7 modes, strata-as-radius invariant, "what Sight is NOT" section, sources subsystem).
3. Draft `lab/reports/MIG-021-EPISTEMIC-CLASSIFIER-ARCHITECT.md`.
4. /migration discipline takes over.

## Verbatim Eisa quotes captured in v1.75 preamble

- *"I want to start fresh. I want you to discard what has been developed so far. I want you to go to the basics. Let's validate and confirm the 'Constellation Sight Concept'."* (2026-05-09)
- *"forget about the InfraNodus."* (2026-05-09, on Sight's reframed answer)
- *"The focus of the 360.3D is the Note, while Sight is the whole universe."* (2026-05-09)
- *"If future Constellation users don't understand it or think it is difficult, then its existence is unnecessary."* (2026-05-09)

---

## Phase 10 — Recommendation accepted; Sight v.next renamed v5; LLM picks confirmed

After the v1.75 PCS, Eisa asked for the recommendation across the three LLM sub-decisions (he was unfamiliar with the LLM landscape). Recommended:

- **LLM**: Qwen3-1.7B Q4_K_M (Apache 2.0, first-class Arabic, ~1.1 GB)
- **Inference**: llama.cpp via `llama-cpp-2` (GBNF grammar-constrained decoding is the killer feature for guaranteed-valid JSON)
- **Bundling**: hybrid — but with a twist: **reuse the existing `multilingual-e5-small` (113 MB, already shipping) as the bundled "starter classifier"** via embedding-similarity classification, and offer Qwen3-1.7B as the optional Settings → AI download. Saves a separate bundled-model decision; installer stays at ~50 MB.

Eisa asked about hardware/software requirements. Surfaced:
- End-user minimum: 64-bit machine from 2013 onward, 4 GB RAM, 200 MB disk, no internet required.
- Sight v5 bundled tier: same as Constellation core (no extra requirements).
- Sight v5 optional larger classifier: 4-core CPU, 4 GB free RAM during run, 1.5 GB disk, one-time 1.1 GB download.

**Eisa: "The next Sight will carry v.5."** Naming locked. v.next → v5 going forward.

## Phase 11 — Second PCS (System Requirements docs)

Eisa: *"Then we need to add the minimum PC requirement to operate Constellation, within the Help file and the user manual."*

Drafted EN system-requirements section, three-section format (Minimum / Recommended / Sight v5), plain-language tone (Eisa: "Your call"). Eisa approved and directed to include the Sight v5 sub-section now (with "when it ships" framing).

Shipped this commit:
- New `### System Requirements` section in `docs/User Manual.md` between `## 1. Getting Started` and `### Installation`
- Same section translated into Arabic in `docs/help.ar/User Manual.md` (same position)
- New help topic `docs/help.uConstellation.World/Getting Started/Getting Started.md` (richer than the Manual section, with a "How to check my computer's specs" footer)
- Arabic version of the new help topic at `docs/help.ar/Getting Started/Getting Started.md`
- Orientation v1.75 → v1.76 with brief preamble noting the docs addition
- 13 other locales (de, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh) NOT translated this commit — queued as PJ-NNN follow-up. Per the BASIC RULE, no inventing translations for languages I cannot verify.

## Verbatim Eisa quotes captured in v1.76 preamble

- *"we need to add the minimum PC requirement to operate Constellation, within the Help file and the user manual."* (2026-05-09)
- *"The next Sight will carry v.5."* (2026-05-09 — naming lock)

---

## Phase 12 — Sight Concept Paper v2.0 + MIG-021 Architect drafted (third PCS)

Eisa: *"Proceed."*

Drafted both canonical Sight v5 documents in one pass:

### `docs/Constellation-Sight-Concept-Paper-v2.0.md` (~700 lines)

The **canonical specification of Sight v5**. Supersedes the three obsolete v1.x papers (Sight Concept Paper v1.1, Sight v3 Concept Paper v1.1, SIGHT-V3-VISUAL-SPEC v1.1). Structure: 14 sections.

Notable sections:
- **§3.3** — the strata-as-radius design is doubly justified (by Constellation's native taxonomy + the cross-civilizational scholarly tradition). Includes the explicit L1→Branch 1+2.3 ... L8→Branch 5.8 mapping table.
- **§5** — the seven modes (R / L / T / C / S / A / **P**) — each with wedge basis, cognitive question, data source. P (Provenance) is the new mode.
- **§6** — the four constants (radius / size / brightness / red) that hold across every mode. P0 invariants.
- **§7** — Sources subsystem with all 11 sources from the taxonomy, frontmatter contract, three setting paths.
- **§8** — Epistemic Classifier two-tier architecture (e5-small bundled / Qwen3-1.7B optional).
- **§9** — *what Sight v5 IS NOT* — the load-bearing boundary section the v1.x papers never wrote. Explicit comparison vs every adjacent surface (Sky View, Map, OrgChart, Search Hub, Index, 360.3D, Knowledge Health Dashboard, Multi-Lens).
- **§11** — three-MIG phased rollout (MIG-021 → 022 → 023) + cleanup MIG.

### `lab/reports/MIG-021-EPISTEMIC-CLASSIFIER-ARCHITECT.md` (~390 lines)

The Architect doc for the Sources subsystem. Eleven landable phases (§1A–§1K). Twelve invariants (P1–P12). Three design options surfaced + rejection reasoning. Architecture (Rust modules, schema, frontmatter contract, Tauri commands, frontend surfaces, i18n keys). Migration-path concerns (first-boot, mid-backfill restart, manual override, downgrade, Tier-2 corruption). Six open questions for Eisa to decide before relevant phases.

### Orientation v1.76 → v1.77

Preamble bump per SO #6 (major doc ship is the trigger). Three orientation bumps in one calendar day (v1.74 morning + v1.75 + v1.76 + v1.77 evening). Body sections still deferred to v1.78+ when Sight v5 actually ships.

### What's now obsolete on disk (preserved per SO #6)

- `docs/Constellation-Sight-Concept-Paper-v1.1.md` — InfraNodus-spined
- `docs/Constellation-Sight-v3-Concept-Paper-v1.1.md` — per-mode (X, Y, Z) grammar
- `docs/SIGHT-V3-VISUAL-SPEC.md` — centrality-on-radius

### What's open for next session

1. Eisa Phase-2-sign-off on MIG-021 Architect (or revisions).
2. Six open questions in Architect §7 to be decided.
3. Plan doc drafted; Build cascade per /migration discipline.

## Verbatim Eisa quotes captured in v1.77 preamble

- *"Proceed."* (2026-05-09 — directive to draft Sight Concept Paper v2.0 + MIG-021 Architect in parallel)

---

## Phase 13 — Stop-on-correction; MIG-021 Architect approved; Plan drafted (fourth PCS)

After v1.77, I pitched Eisa three open items including the six MIG-021 Architect open questions. Eisa fired the Stop-On-Correction Rule:

> *"Enough of your never-ending technical questions. Proceed with the MIG-021 Architect Phase-2."*

Reading: I was treating those questions as gates ("await your answer before proceeding"). Eisa was clear: **the Architect is approved by directive; the questions are mine to lock with sensible defaults; cascade to the next deliverable.** Same lesson as 2026-04-13 ("simplicity from understanding at first sight, NOT raising more questions").

Locked the six open questions inline in the Plan doc:
1. CDN URL → GitHub Release asset (zero new infra)
2. Source-definition text → ~150 words per source from taxonomy
3. Classify scope → title + body
4. Long-note chunking → Tier 1 first 2k chars; Tier 2 full to 32k
5. 12th `unclassifiable` token → YES
6. Auto-reclassify on Tier-2 → NO (manual button only)

All six are reversible. Documented in v1.78 preamble + Plan §0 so future-me knows which calls were Build-cascade defaults vs explicit Boss approvals.

### Plan drafted

`lab/reports/MIG-021-EPISTEMIC-CLASSIFIER-PLAN.md` — ~430 lines. Eleven landable phases (§1A–§1K). Three user-testable Boss-test gates (§1C Source Review panel, §1F Background scan, §1H Tier-2 download). Other phases self-verify (type-check, /simplify, schema-migration idempotency).

Phase sequencing diagram + risk register + out-of-scope section. Tier-1 accuracy gate at §1B (must reach ≥7/10 top-1 reasonable suggestions on hand-picked sample, else revisit Q2 source-definition text quality).

### Orientation v1.77 → v1.78

Per SO #6, Plan ship is the trigger. Bump preamble notes the Architect approval + Plan ship + six locked defaults + sequencing diagram + risk highlights.

### What's next

Plan-Approval-Equals-Build-Approval is in force. On Eisa's nod ("approved" or revision request), Build cascade begins with §1A (schema migration). Pauses only at §1C / §1F / §1H Boss-test gates, or at architectural surprise.

## Verbatim Eisa quotes captured in v1.78 preamble

- *"Enough of your never-ending technical questions. Proceed with the MIG-021 Architect Phase-2."* (2026-05-09 — Stop-On-Correction trigger; Architect approved by directive)

---

## Phase 14 — Build cascade: §1A → §1B → §1C (first Boss-test gate reached)

After v1.78 PCS I went off-thread investigating a Claude Code UI question (`Ctrl+O` toggles tool-output verbosity). Eisa course-corrected: "I was referring to this" — pointing to the four-document Sight v5 specification block. The "Proceed" was the Plan-Approval-Equals-Build-Approval signal, not the Claude Code investigation. Stop-On-Correction; Build cascade started.

### Build environment caveat (architectural surprise, not blocking)

I cannot run `cargo check` or `cargo build` from this agent sandbox — no Rust toolchain installed, and Tauri builds require system libraries the sandbox lacks. Per Plan-Approval-Equals-Build-Approval's "architectural surprise" exception I surfaced this in commit messages: each phase ships with explicit "build verification pending Eisa's local cargo build" notes. If anything fails to compile, fix forward in a follow-up commit.

### §1A — Schema migration + frontmatter parser + 3 IPCs (commit `4d6ef37`)

**NEW src-tauri/src/sources.rs** (~520 LOC including 11 unit tests):
- SOURCE_IDS constant: 11 canonical sources from the Universal Epistemic Content Taxonomy + 12th `unclassifiable` opt-out token (Plan §0 Q5)
- `extract_sources()` — frontmatter parser handling all three YAML shapes (scalar / inline array / block list); mirrors `search::extract_aliases` (MIG-004 §2)
- `rewrite_frontmatter_sources()` — frontmatter rewriter preserving all other fields and body
- DB read/write helpers + 3 IPCs: `sources_get_for_note`, `sources_set_manual`, `sources_clear`

**EDIT src-tauri/src/search.rs**:
- `init_db`: wired `ensure_note_meta_sources_column` + `ensure_sources_suggestions_table` after MIG-003 block (idempotent)
- `index_note`: extracts sources from frontmatter on every save, stamps into the new `note_meta.sources` column (extended INSERT 12 → 13 columns)

**EDIT src-tauri/src/lib.rs**: `mod sources;` + 3 IPCs registered.

### §1B — Tier 1 classifier (commit `dcbd40e`)

**NEW src-tauri/src/classifier/** (directory module, 3 files, ~480 LOC total):
- `mod.rs` — `classifier_suggest_for_note(path)` IPC: reads note → extracts title+body (Plan §0 Q3) → truncates to 2000 chars (Plan §0 Q4) → runs Tier-1 → writes top-3 to queue → returns SuggestionRecord
- `source_definitions.rs` — 11 source definitions, ~150 words each, drawn from the taxonomy doc with rich semantic cues for e5-small distinguishability (textual phrases, examples, contrasts with adjacent sources)
- `tier1_embedding.rs` — embed source defs once at first call (cached in OnceLock), embed note text, cosine similarity, top-3 sorted descending. Math helpers: l2_normalize, dot, clamp01, char-boundary truncate

**EDIT src-tauri/src/embeddings.rs**: `run_embedding` and `ensure_engine` made `pub(crate)` so the classifier can reuse the cached e5-small ONNX engine without duplicating the model load.

**EDIT src-tauri/src/lib.rs**: `mod classifier;` + `classifier_suggest_for_note` IPC.

Hardware impact: zero. e5-small is already shipped (113 MB ONNX for semantic search); reused at no additional bundle cost.

### §1C — Source Review sidebar panel + 3 review IPCs + i18n (commit `4e70393`) — ✅ Boss-test gate

**NEW src/lib/components/SourceReviewPanel.svelte** (~430 LOC incl Suwaidi-aligned styling):
- Lists pending suggestions FIFO across active Universe
- Per record: title (clickable → openNoteTab), tier badge, top-3 with confidence + evidence
- Three actions: Accept (writes via `sources_set_manual`, clears suggestion), Edit (multi-select 11 sources + unclassifiable opt-out, save), Reject (clears without writing)
- "Classify open note" button when a note is open — small scope expansion to make the §1C test self-contained without yet shipping the §1E right-click action (production builds disable dev-tools console; needed an in-app trigger)
- Empty / loading / error states with plain-language messages
- `dir="auto"` for native bidi (RTL works without per-locale switching)

**EDIT src-tauri/src/sources.rs** — three new Tauri commands:
- `sources_get_suggestions(path)` → Option<SuggestionRecord>
- `sources_list_pending_suggestions()` → Vec<SuggestionRecord> (FIFO by created_at)
- `sources_reject_suggestion(path)` → clear queue entry without writing

**EDIT src/routes/+layout.svelte**:
- Imported SourceReviewPanel
- Extended `rightSidebarTab` type union with `'sourceReview'`
- Added `tabVisible[sourceReview] = true` (force-visible until panelPlacements wiring ships)
- New tab button (checkmark-in-square icon)
- Two render branches (with-active-tab + without-active-tab) so panel works regardless of editor state — mirrors ReviewPulse pattern
- Both branches pass `activeNotePath={sidebarTab?.path ?? null}`

**EDIT src/lib/i18n/en.json + ar.json** (~30 strings each):
- panels.sourceReview
- sources.label.{11_sources_+_unclassifiable} — full bilingual label set
- sources.review.{title|refresh|loading|error|empty|pending|tier|accept|edit|reject|save|cancel|classifyActive|classifying}

13 other locales NOT translated this commit — fall back to EN via standard $t() chain. Queued as PJ for human translation.

### Commit hashes

- `4d6ef37` — §1A
- `dcbd40e` — §1B
- `4e70393` — §1C (✅ Boss-test gate reached)

### What's now testable

Eisa builds locally (cargo build / npm run tauri dev), then verifies the Source Review panel works end-to-end on the trial Universe per the test tutorial below.

### Next session

Either: (a) fix any build errors Eisa reports, or (b) cascade §1D (PropertyEditor combobox) → §1E (right-click context action — replaces the temp "Classify open note" button) → §1F (background scan + status bar — second Boss-test gate) → §1G (i18n full pass) → §1H (Tier 2 download — third Boss-test gate) → §1I (help docs) → §1J (/simplify + audit) → §1K (close-out + orientation v1.79 bump).

## Verbatim Eisa quotes captured in this phase

- *"I was referring to this."* (2026-05-09 — Stop-On-Correction; "Proceed" meant Build cascade, not the Claude Code UI investigation)

---

## Phase 15 — MIG-021 cascade pause; v2 Redesign Architect drafted

After Stage 3 RTL fixes landed (`ec288fe`), Eisa surfaced the usability concern: 11 flat horizontal sources are too abstract for non-expert recognition. ("Constructed Idea is easier than Semantic Contents.") Asked whether the code captured the whole taxonomy.

### Honest gap-disclosure

Acknowledged: only the 11 horizontal sources were embedded; the 5-branch vertical tree (~218 nodes) was projected onto the existing strata field per Concept Paper v2.0 §3.3 but not separately addressable; the 218 deeper sub-nodes were not in code at all.

### The pivot

Eisa picked Option B (two parallel fields per note: horizontal `sources:` + new vertical `content_type:`) + at least 2 levels of depth on the horizontal axis.

I drafted a 51-sub-leaf decomposition for the 11 horizontal parents (§3 of an in-flight Architect). Eisa challenged: "Where did I mention the 11 source? Which document?" — a sharp BASIC-RULE check. I conceded honestly: the "11" came from his taxonomy doc Section III + my question framing, not from his personal directive. Eisa then directed me to wait while he developed the horizontal taxonomy himself rather than ratifying my draft.

### Eisa-authored horizontal taxonomy delivered

Eisa shared `sources-of-knowledge-diagram.html` — 3-level interactive diagram, EN+AR+transliteration, 11 parents with TIER METADATA (Tier 1/2/3 acceptance), 41 sub-leaves in scholarly traditional terms (uṣūl al-fiqh classifications, classical pramāṇa sub-distinctions, Mīmāṃsā arthāpatti types, etc.). Substantively different from my generic-modern draft.

Saved to `docs/sources-of-knowledge-diagram.html` as canonical.

### Tier dimension is new

Tier metadata was not in my earlier work. Implications: tier-coloring in the tree picker (teal/purple/amber per the diagram), Settings → Sources opt-out for Tier 3 (school-specific sources users may not endorse), classifier confidence-fallback (when top-1 is Tier 3 and confidence is borderline, suggest Tier 1/2 alternative to avoid surfacing contested categories on secular notes).

### Fresh v2 Redesign Architect drafted

`lab/reports/MIG-021v2-EPISTEMIC-CLASSIFIER-REDESIGN-ARCHITECT.md` — supersedes the original. Built against Eisa's canonical horizontal taxonomy. New sections:
- §2 horizontal taxonomy (Eisa-canonical, no ratification needed)
- §3 vertical axis (lifted from existing chart)
- §5 tree picker UI (mirrors both diagrams)
- §6 classifier extension with tier-aware fallback
- §10 tier system UX
- §11 Concept Paper v2.1 amendments (deferred to §1K' close-out)
- §12 8 open questions locked with defaults

### Orientation v1.79 bumped (per SO #6 — Architect ship trigger)

Preamble notes the pivot, the Eisa-authored taxonomy, the redesign Architect, the preserved foundation on `main`.

### What's preserved on `main`

All commits from §1A through §1C (incl. 3 fix commits) — `4d6ef37`, `dcbd40e`, `4e70393`, `c3f3e96`, `4769fbe`, `ec288fe`. Substantively nothing rolled back. The v2 redesign expands vocabulary + replaces flat picker with tree picker; foundation adapts.

### Boss review pending

Architect §14 has the checklist — 7 items for Eisa to ratify or revise. On approval, new Plan doc → Build cascade resumes per Plan-Approval-Equals-Build-Approval.

## Verbatim Eisa quotes captured in v1.79 preamble

- *"It would be easier for regular users to select the right source if we include the whole taxonomy. ... We will give the user the choice to choose the right level."* (2026-05-09 — Option B + 2-level depth directive)
- *"Where did I mention the 11 source? Which document?"* (2026-05-09 — BASIC RULE check; surfaced that "the 11" came from his taxonomy doc + my framing, not a separate personal directive)
- *"I want you to wait until I develop the Horizontal Axis: Sources / Means of Knowledge taxonomy."* (2026-05-09 — pause directive; honored before drafting v2 Architect)

---

## Phase 16 — MIG-021v2 Build cascade §1A' → §1C' (first Boss-test gate of v2)

Eisa: "Approved." Cascade started.

### §1A' — Schema + extracted taxonomies + content_type field + 5 new IPCs (commit `7b4db70`)

**NEW src-tauri/src/sources/** (was sources.rs single-file; converted to directory module):
- `horizontal_taxonomy.rs` (~620 LOC) — extracted from `docs/sources-of-knowledge-diagram.html`. 53 nodes: 11 parents + 41 leaves + 1 unclassifiable. Tri-script labels. Tier metadata. 8 lookup helpers + 7 unit tests.
- `vertical_taxonomy.rs` (~390 LOC) — extracted from `docs/epistemic-content-taxonomy-chart.html`. ~218 nodes across 5 branches. Full path-slug IDs. 5 unit tests.
- `mod.rs` (was sources.rs) — adapted: SOURCE_IDS const → source_ids() function reading from horizontal_taxonomy; new content_type subsystem (mirrors sources); 2 NEW taxonomy IPCs (sources_get_horizontal_taxonomy + sources_get_vertical_taxonomy) so frontend fetches once + caches.

**NEW src/lib/sources/horizontalTaxonomy.ts + verticalTaxonomy.ts** — frontend wrappers with cached fetch + lookup helpers + tier/branch color mapping.

5 new Tauri IPCs registered: `content_type_get_for_note`, `content_type_set_manual`, `content_type_clear`, `sources_get_horizontal_taxonomy`, `sources_get_vertical_taxonomy`.

Backward-compat invariant: 11 §1A parent IDs (perception, inference, etc.) byte-identical in v2 horizontal_taxonomy. Legacy `sources:` data on disk validates without migration.

### §1B' — Classifier expand to ~275 candidates + tier-aware fallback + axis tagging (commit `7ea86db`)

**REWRITE src-tauri/src/classifier/source_definitions.rs**:
- 11 parent SOURCE_DEFINITIONS preserved (~150 words each; battle-tested)
- NEW HORIZONTAL_LEAF_HINTS: 41 short scholarly sentences from the diagram + standard literature
- NEW build_classifier_candidates() runtime builder: 52 horizontal (excludes opt-out) + 222 vertical = ~274 entries
- For vertical nodes: mechanical embedding text "{en} ({ar}). Branch X — {branch}. Parent: {parent}." per Plan §3 risk mitigation (no fabrication of philosophical content where chart provides only labels)

**REWRITE src-tauri/src/classifier/tier1_embedding.rs**:
- Two cached vector pools: HORIZONTAL_VECTORS (52) + VERTICAL_VECTORS (~218)
- classify() returns combined Vec<Suggestion>: top-3 horizontal + top-3 vertical
- Tier-aware fallback (Plan §0 Q7): when top-1 is Tier-3-effective and confidence < 0.55, promote highest-scoring Tier-1/2 candidate
- Leaf-vs-parent strategy (Plan §0 Q5): when leaf confidence < 0.55, replace with its parent (deduped); user can drill down manually
- CONFIDENCE_FALLBACK_THRESHOLD = 0.55 locked

**EDIT src-tauri/src/sources/mod.rs**: Suggestion struct adds `axis: String` field with #[serde(default)] -> "horizontal" so legacy §1A/§1B suggestions deserialize.

### §1C' — TaxonomyTreePicker.svelte + dual-axis Source Review + i18n (commit `751c036`) — ✅ Boss-test gate

**NEW src/lib/sources/TaxonomyTreePicker.svelte** (~300 LOC):
- Reusable hierarchical tree picker (works for both axes via props)
- Multi-select via checkbox per node
- Tier-based color coding via border-inline-start (auto-flips RTL)
- Search/filter input — auto-expands ancestors of matches
- Tri-script labels (EN + AR + transliteration)
- Recursive Svelte 5 snippet-based rendering

**REWRITE src/lib/components/SourceReviewPanel.svelte**:
- Card body: TWO suggestion sublists per record (Sources / Content type), tier badges on horizontal entries
- Edit mode: TWO TaxonomyTreePicker instances side-by-side ≥1200px (per Plan Q4)
- Accept commits BOTH axes (sources_set_manual + content_type_set_manual)
- Taxonomies fetched once on mount + cached

**i18n EN + AR**: sources.review.axis.{horizontal|vertical} + taxonomyTreePicker.{search|expandAll|collapseAll|tier1Legend|tier2Legend|tier3Legend}

### Build verification

cargo check clean throughout (verified after §1A', §1B', and §1C'). 37 warnings, all pre-existing or "never used" for §1D'+ consumption.

### What's now testable

✅ §1C' Boss-test gate. Eisa builds locally + verifies dual-axis classification, tree picker, tier badges, RTL.

### Verbatim Eisa quotes

- *"Approved."* (2026-05-09 — Plan-Approval-Equals-Build-Approval signal)

---

## §1C' Boss-test gate — PASS (2026-05-09)

Eisa: *"All pass"* — covering Stage 1 (build + dual-axis classification), Stage 2 (Accept / Edit / tree mechanics / Save / Cancel), Stage 3 (Arabic walkthrough with locale-aware labels). Closed all three §1C' fixes:

- fix-1: locale-aware label rendering (`labelForId` reads `$locale`)
- fix-2: pickers always stacked vertically (removed `@media (min-width: 1200px)` row layout)
- fix-2 (cont.): TaxonomyTreePicker flat-render rewrite (replaces recursive Svelte 5 `{#snippet treeNode(node, depth)}` with pre-walked `Row[]` derived state)

Commits: `4769fbe` (gold border) → `ec288fe` (RTL `border-inline-start`) → `609b2d8` (sub-leaf evidence i18n) → `3582f1d` (stacked + locale-aware) — all under §1C'.

---

## §1D' — PropertyEditor inline taxonomy pickers

**Goal**: per-note manual editing of `sources:` and `content_type:` directly from the Note properties panel.

**EDIT src/lib/components/PropertyEditor.svelte**:
- Added `isTaxonomyKey(key)` → `'horizontal' | 'vertical' | null` matching `sources` / `content_type`
- Lazy-load both taxonomies on first expand (`ensureTaxonomiesLoaded`); cached in module-level state
- New value-rendering branch placed before `stage`: pills row + chevron toggle + inline TaxonomyTreePicker (height-capped 320px)
- `applyTaxonomySelection(idx, set)` writes `listItems` + `value`; saves via existing `debouncedSave` → `saveTabContent` (frontmatter is single source of truth; `index_note` re-extracts on save so SQLite mirror updates)
- Pills carry tier-color borders (horizontal axis) via `border-inline-start: 3px solid` (auto-flips RTL)
- `removeTaxonomyValue` removes individual pill without opening the picker
- Added `sources` and `content_type` to KEY_SUGGESTIONS (EN + AR)

**Storage path**: PropertyEditor writes the YAML list through the standard frontmatter save; no special IPC. The `sources_set_manual` / `content_type_set_manual` IPCs remain reserved for the Source Review accept-flow.

**i18n**: reused existing `propertyEditor.empty`, `propertyEditor.delete`, `taxonomyTreePicker.expandAll` — no new strings required for §1D' chrome.

**Verification**:
- `npx svelte-check`: zero new errors (pre-existing LinkLifecycle dedupe untouched — Option B deferred per memory)
- `npm run build`: clean
- `npm run tauri build -- --bundles nsis`: produced `Constellation_0.3.4_x64-setup.exe`

**Awaiting §1D' Boss-test**.
