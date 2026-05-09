# MIG-021 — Epistemic Classifier (Sources Subsystem) — Architect

**Status**: Phase 1 (Architect) — awaiting Boss Phase 2 sign-off.
**Anchored against**: [`docs/Constellation-Sight-Concept-Paper-v2.0.md`](../../docs/Constellation-Sight-Concept-Paper-v2.0.md) §7 + §8.
**Companion**: [`lab/reports/MIG-021-LOCAL-LLM-RESEARCH.md`](MIG-021-LOCAL-LLM-RESEARCH.md) — the LLM stack research this Architect builds on.
**Decisions ratified**: Eisa 2026-05-09 (six Sources sub-decisions + classifier strategy = local LLM + Concept Paper v2.0 ratification).

---

## §1 · Mission

Ship the **Sources subsystem** end-to-end so that Sight v5's Provenance mode (P) has data to visualize:

1. New per-note frontmatter field `sources:` (multi-source, ranked, all 11 from the Universal Epistemic Content Taxonomy).
2. Mirror in `note_meta.sources` SQLite column (matches MIG-014 Strata/Maturity/Stage pattern).
3. **Epistemic Classifier** (two tiers): bundled e5-small embedding-similarity (Day 1, no extra requirements) + optional Qwen3-1.7B GGUF download (Settings → AI, llama.cpp inference).
4. **Source Review sidebar panel** — queue-based approval workflow (mirrors Review Pulse pattern).
5. **PropertyEditor combobox** — manual setting (mirrors Strata / Maturity / Stage exactly).
6. **Right-click context action** — "Suggest sources for this note" → on-demand single-note classification.
7. **Background scan job** — opt-in per Universe, runs on idle, resumable, status-bar progress.
8. **i18n** — 11 source labels × 15 locales + UI chrome strings (panel labels, button labels, tooltips, settings).
9. **Help docs + User Manual** — EN canonical + AR translation; 13-locale follow-up queued as PJ.

Sight v5 itself is **not** built in MIG-021. This MIG produces the *data* Sight v5 will visualize. MIG-022 builds the visual; MIG-023 wires modes and ships the optional Tier-2 classifier path.

---

## §2 · Invariants (must hold at every commit)

| # | Invariant | Why |
|---|---|---|
| P1 | `sources:` frontmatter is canonical; `note_meta.sources` is the read mirror | Law 2.7 (single source of truth). Frontmatter wins on disagreement. |
| P2 | Classifier writes ONLY `sources_suggested:` — never `sources:` | The user is the only path to canonical assignment (Eisa Q3, 2026-05-09). |
| P3 | Multi-source ordered list; primary = first | Eisa Q2. List order is semantically meaningful. |
| P4 | All 11 sources from the taxonomy ship from Day 1 | Eisa Q1. No subset cuts. |
| P5 | Locale-driven labels across all 15 locales via `$t()` | Eisa Q6. EN + AR primary; 13 others follow standard fallback chain. |
| P6 | Boot-perf zero impact | Per CLAUDE.md Performance Rule 8 + the 2026-04-15 boot discipline. Classifier loads lazy on first use. |
| P7 | No `invoke()` on the keystroke hot path | Per CLAUDE.md Performance Rule 3. Classification IPCs are background-only. |
| P8 | All classifier output is reversible — clearing `sources:` from PropertyEditor restores the unsourced state with no side-effects | Per Eisa's "every link operation must be reversible" pattern from CE Layer 1. |
| P9 | Tier 1 (bundled e5-small) requires NO additional bundle, NO additional system dependency | Reuses the existing 113 MB ONNX model already shipped for semantic search. Zero installer growth. |
| P10 | Tier 2 (optional Qwen3-1.7B + llama.cpp) is gated behind a Settings → AI download — never bundled in the installer | Per the bundling-research recommendation: novice never sees the friction; power user opts in. |
| P11 | The Source Review panel's Accept/Edit/Reject actions are atomic — no partial state where `sources:` is half-written | Mirrors MIG-006 §3's atomic-rename discipline. |
| P12 | Schema migration is idempotent — running on a database that already has `note_meta.sources` is a no-op | Standard Constellation schema-migration pattern. |

---

## §3 · Design options considered

### Option A — Single classifier (Tier 2 only, mandatory download)

Skip the bundled tier; force users to download Qwen3-1.7B on first Sight v5 launch.

- **Pro**: one classifier path to maintain, higher accuracy from Day 1.
- **Con**: violates Eisa's "Sight works first time, no chooser, no download" novice constraint. First-meaningful-use blocked behind a 1.1 GB download.
- **Verdict**: REJECTED.

### Option B — Single classifier (Tier 1 only, no Tier 2 ever)

Ship just the e5-small bundled tier; never offer the larger model.

- **Pro**: simplest architecture; no second inference engine; no optional-download UX.
- **Con**: Tier 1's ~65–75% accuracy is acceptable for many users but inadequate for Eisa's Arabic-heavy use case. The Source Review queue compensates partially but classifier precision matters.
- **Verdict**: REJECTED. Tier 2 is the path to ~85–90% accuracy on Arabic; not shipping it caps the feature's quality ceiling.

### Option C — Both tiers, hybrid bundling (RECOMMENDED)

Tier 1 (bundled e5-small) ships in every installer; Tier 2 (Qwen3-1.7B + llama.cpp) gated behind Settings → AI download.

- **Pro**: First-time user experience is instant (Tier 1 works on install). Power user opts up with one click. Installer stays at ~50 MB. Smart Connections precedent (2M+ Obsidian installs validates the pattern).
- **Con**: Two inference engines coexist (ORT for embeddings/Tier 1, llama.cpp for Tier 2). Two classifier code paths to maintain.
- **Verdict**: APPROVED.

---

## §4 · Architecture

### §4.1 Rust modules (new)

```
src-tauri/src/
├── classifier/
│   ├── mod.rs                  — public API: classify_note(), classify_universe()
│   ├── tier1_embedding.rs      — e5-small embedding-similarity classifier
│   ├── tier2_llm.rs            — Qwen3-1.7B llama.cpp classifier
│   ├── source_definitions.rs   — the 11 source definitions, embedded at build time
│   ├── prompt.rs               — Tier 2 few-shot prompt + GBNF grammar
│   └── queue.rs                — sources_suggested write/read; review-queue management
└── sources/
    ├── mod.rs                  — frontmatter read/write for `sources:` and `sources_suggested:`
    ├── schema.rs               — note_meta.sources column migration + write-time triggers
    └── ipc.rs                  — Tauri commands (see §4.4)
```

ORT crate (`ort`) stays for embeddings (Tier 1). New crate dependency: `llama-cpp-2` (utilityai) for Tier 2. Both wrapped behind a single `classifier::classify_note()` public API so the frontend never sees which tier runs.

### §4.2 Schema (SQLite)

```sql
-- New column on existing note_meta table
ALTER TABLE note_meta ADD COLUMN sources TEXT DEFAULT NULL;
-- JSON-encoded list of source IDs, e.g. '["testimony","mass-transmission"]'
-- NULL = unsourced (the user hasn't classified yet)

-- New table for the suggestion queue (avoids polluting note_meta with transient suggestions)
CREATE TABLE IF NOT EXISTS sources_suggestions (
  note_path TEXT PRIMARY KEY,
  suggestions_json TEXT NOT NULL, -- ordered list of {source, confidence, evidence}
  classifier_tier INTEGER NOT NULL, -- 1 = embedding, 2 = LLM
  created_at INTEGER NOT NULL,
  FOREIGN KEY (note_path) REFERENCES note_meta(path) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_sources_suggestions_created ON sources_suggestions(created_at);
```

Write-time trigger on `note_meta` (mirrors the existing Strata/Maturity/Stage triggers): when a note's frontmatter changes, re-parse `sources:` and update `note_meta.sources`. Idempotent on schema-already-migrated DBs.

### §4.3 Frontmatter contract

```yaml
# User-canonical (set via PropertyEditor or by approving a suggestion)
sources:
  - testimony
  - mass-transmission

# Classifier output (read by Source Review panel only; cleared on user action)
sources_suggested:
  - { source: testimony, confidence: 0.82, evidence: "Quoted from Ibn Sīnā..." }
  - { source: inference, confidence: 0.41, evidence: "Therefore X follows from Y..." }
```

Both fields optional. Empty list means unsourced. Classifier-suggested entries carry a confidence float (0–1) and an evidence string (the textual cue that triggered the suggestion). User-set entries are bare source IDs without confidence/evidence (the user's assignment is canonical regardless of textual evidence).

### §4.4 Tauri commands (new)

| Command | Purpose | Tier |
|---|---|---|
| `classifier_suggest_for_note(path)` | On-demand single-note classification; writes to `sources_suggested:` and `sources_suggestions` table | 1 by default; 2 if Tier 2 available |
| `classifier_scan_universe(library_set, opts)` | Background scan; chunks; emits `classifier:progress` events | 1 by default; 2 if Tier 2 available |
| `classifier_cancel_scan(scan_id)` | Cancel an in-flight scan | — |
| `sources_get_suggestions(path)` | Read suggestion record for a note (for Source Review panel) | — |
| `sources_accept_suggestion(path, sources)` | Accept (or edited) suggestion → write `sources:` to frontmatter + clear suggestion | — |
| `sources_reject_suggestion(path)` | Reject → clear suggestion without writing `sources:` | — |
| `sources_set_manual(path, sources)` | PropertyEditor / direct manual setting → write `sources:` to frontmatter | — |
| `tier2_download_model(opts)` | Download Qwen3-1.7B GGUF from CDN; emits progress events | — |
| `tier2_status()` | Returns whether Tier 2 model is downloaded and ready | — |
| `tier2_unload()` | Free Tier 2 model from RAM | — |

### §4.5 Frontend surfaces (new)

- `src/lib/sources/SourceReviewPanel.svelte` — new sidebar panel; one of the existing right-sidebar tabs (mirrors Review Pulse / Backlinks / Outgoing structure).
- `src/lib/sources/PropertyEditorSourcesField.svelte` — multi-select combobox; integrated into the existing PropertyEditor between Maturity and Stage.
- `src/lib/sources/SettingsAIPanel.svelte` — new Settings → AI section; Tier-2 download button + classifier-status indicator + background-scan toggle.
- Right-click context action wired into the existing context-menu system (FileTree, Sky View, Sight, etc.).

### §4.6 i18n keys (new)

| Key prefix | Count | Notes |
|---|---|---|
| `sources.label.{source_id}` | 11 | Source names (Perception, Inference, ...) |
| `sources.description.{source_id}` | 11 | Tooltip definitions (used in PropertyEditor + Source Review) |
| `sources.review.*` | ~12 | Source Review panel chrome (title, accept/edit/reject buttons, empty-state, hints) |
| `sources.propertyEditor.*` | ~6 | PropertyEditor field label + placeholder + helper text |
| `sources.settings.*` | ~15 | Settings → AI panel (download button, status, background-scan toggle, model-info, errors) |
| `classifier.progress.*` | ~6 | Status-bar strip during background scan |

Total: ~75 new strings × 15 locales = 1,125 translation entries. EN + AR shipped this MIG; 13 other locales queued as a follow-up PJ (per the existing 15-locale convention in MIG-014).

---

## §5 · Phased build plan

Each step is one landable commit with a verification clause.

### §1A — Schema migration + frontmatter read/write
Files: `src-tauri/src/sources/schema.rs`, `src-tauri/src/sources/mod.rs`, integration into existing `note_meta` write-time triggers.
Verify: schema migration runs idempotently on Eisa's trial Universe. Write a sample frontmatter `sources:` field by hand to a single note; confirm `note_meta.sources` mirror updates within seconds.

### §1B — Tier 1 classifier (e5-small embedding-similarity)
Files: `src-tauri/src/classifier/mod.rs`, `tier1_embedding.rs`, `source_definitions.rs`, `prompt.rs` (Tier 2 stub).
The 11 source definitions are embedded at app build time; the resulting 11 × 384-dim cached vectors ship inside the binary as a compile-time constant.
Verify: run `classifier_suggest_for_note(path)` on 10 hand-picked notes from the trial Universe; inspect the suggestions; spot-check that intuitive cases (a quoted-source note → testimony top-1; a logical derivation note → inference top-1) classify correctly. Acceptable: ~7/10 top-1 correct.

### §1C — Source Review sidebar panel + Accept/Edit/Reject flow
Files: `src/lib/sources/SourceReviewPanel.svelte`, panel registration in the existing right-sidebar tab system, frontend wrappers for `sources_get_suggestions` / `sources_accept_suggestion` / `sources_reject_suggestion` / `sources_set_manual`.
Verify: Eisa-test — open Source Review panel; classifier-pre-populated suggestions for 5 notes; approve 2 / edit 1 / reject 2; confirm `sources:` frontmatter updates correctly for the approved 3.

### §1D — PropertyEditor combobox (manual setting)
Files: `src/lib/sources/PropertyEditorSourcesField.svelte`, integration into existing PropertyEditor component.
Verify: select a note in the editor; open PropertyEditor; multi-select 3 sources via the combobox; confirm frontmatter writes; confirm Sight (when MIG-022 ships) reads them. (For now: confirm `note_meta.sources` mirror updates.)

### §1E — Right-click "Suggest sources for this note" context action
Files: extension of the existing context-menu wiring in FileTree / Sky View / wherever notes are selectable.
Verify: right-click any note → "Suggest sources" → Source Review panel surfaces the suggestion within ~3 seconds.

### §1F — Background scan job
Files: `src-tauri/src/classifier/queue.rs`, scan-job orchestration mirroring `sky_backfill::maybe_schedule`. New status-bar strip group (`MigrationProgressStrip` pattern from MIG-015 §1C, reusable).
Verify: enable background scan in Settings; observe status-bar progress; let it complete on the trial Universe (~10–15 min on Tier 1); inspect the queue; confirm resumable across app restart.

### §1G — i18n EN + AR (UI strings + 11 source labels)
Files: `src/lib/i18n/en.json`, `src/lib/i18n/ar.json` (the two locales I can responsibly translate per the BASIC RULE).
Verify: switch UI to Arabic; confirm Source Review panel + PropertyEditor combobox + Settings → AI all render correctly with RTL.

### §1H — Tier 2 download + llama.cpp integration (optional path)
Files: Add `llama-cpp-2` Cargo dependency; `src-tauri/src/classifier/tier2_llm.rs`; `src/lib/sources/SettingsAIPanel.svelte`; `tier2_download_model` IPC with resumable HTTP download from a CDN URL TBD (Hugging Face mirror or self-hosted).
Verify: Eisa downloads Qwen3-1.7B from Settings → AI → "Download larger classifier"; download completes; runs on a sample of 20 notes; compare suggestions vs Tier 1; spot-check Arabic notes specifically.

### §1I — Help docs + User Manual EN + AR
Files: `docs/User Manual.md` (new section under §3 Creating and Editing Notes — "Source Tags"), new help topic `docs/help.uConstellation.World/Sources/Sources.md` + Arabic mirror.
Verify: read through the new sections from a first-time-user perspective; confirm the 11 sources are explained without civilizational jargon (the taxonomy backs the explanations but doesn't dominate them).

### §1J — `/simplify` checkpoint + audit
Run `/simplify` on the full diff. Spawn three audit agents (Invariant, Drift, Migration-path) per the standard /migration discipline. Address P0/P1 findings; defer P2/P3 to follow-up PJs.

### §1K — MIG-021 close-out
Mark MIG-021 Done. Bump orientation. Append session log. PCS.

---

## §6 · Migration-path concerns

| Scenario | Handling |
|---|---|
| First-boot on a pre-MIG-021 DB | Schema migration runs (idempotent). `note_meta.sources` column added with all NULL. `sources_suggestions` table created empty. No user-visible delay. |
| Mid-backfill app restart | Background scan resumes from the last-classified note. Suggestion records persist in SQLite across restarts. |
| User manually sets `sources:` on a note that already has a `sources_suggested:` entry | The suggestion is consumed (cleared) automatically. PropertyEditor write wins; classifier respects user authority. |
| User downgrades Constellation to a pre-MIG-021 version | The `sources:` and `sources_suggested:` frontmatter fields remain in `.md` files (harmless extra fields). The `note_meta.sources` column is preserved (unused). The `sources_suggestions` table is preserved (unused). On re-upgrade, everything resumes. |
| User deletes Tier 2 model from disk | `tier2_status()` reports unavailable; classifier silently falls back to Tier 1; user sees no error. |
| Tier 2 download interrupted | Resumable from last byte. If aborted, partial file deleted. |
| Tier 2 model file corrupted | llama.cpp returns load error; surface to user with "Re-download" button in Settings → AI. |

---

## §7 · Open questions / risks

| # | Question | Path |
|---|---|---|
| Q1 | What's the CDN URL for the Qwen3-1.7B Q4_K_M GGUF download? | Eisa to decide: Hugging Face direct, GitHub Releases asset, or self-hosted on uConstellation.World infrastructure. Resolved before §1H. |
| Q2 | The 11-source definitions need *embedding-quality canonical text* — long enough that e5-small can distinguish them, short enough not to bloat the build. | First draft: ~100-word definition per source from the taxonomy doc. Iterate against a 50-note hand-labeled eval set in §1B. |
| Q3 | Should the classifier run on note title + body, or body only? | Default to title + body; title carries strong signal (titles often name the source explicitly). Configurable in Settings if accuracy issues emerge. |
| Q4 | What's the chunking strategy for very long notes (>10k tokens)? | Tier 1 (e5-small) has a 512-token window. Tier 2 (Qwen3-1.7B) has 32k. For Tier 1: classify on the first 2k chars (heuristic; covers most knowledge-note lengths). For Tier 2: classify on full note up to context limit. |
| Q5 | Can the user mark a note as "not classifiable" to exclude it from the queue forever? | Yes — a separate frontmatter field `sources: ["unclassifiable"]` (a 12th canonical token outside the 11 sources). Suppresses future classifier suggestions on that note. |
| Q6 | When the user downloads Tier 2, should the existing Tier-1 suggestions be re-classified? | Settings → AI offers a one-click "Re-classify with larger model" action. Default is no — Tier 1 suggestions stay until the user actively re-runs. |

---

## §8 · Cross-references

- [`docs/Constellation-Sight-Concept-Paper-v2.0.md`](../../docs/Constellation-Sight-Concept-Paper-v2.0.md) — the canonical specification this Architect implements
- [`docs/epistemic-content-taxonomy.md`](../../docs/epistemic-content-taxonomy.md) — the 11-source taxonomy with bilingual labels and definitions
- [`lab/reports/MIG-021-LOCAL-LLM-RESEARCH.md`](MIG-021-LOCAL-LLM-RESEARCH.md) — model + inference + bundling research
- [`lab/reports/MIG-014-...`](.) — precedent for frontmatter + note_meta mirror pattern (Stage taxonomy)
- [`lab/reports/MIG-015-...`](.) — precedent for resumable chunked background job + status-bar progress strip
- CLAUDE.md — Performance Rules 3 + 8; Working Agreements 4 + 5; Standing Order 6 (orientation bump in same commit as ship)
- Memory: `project_sight_classifier_local_llm.md` (the six Sources sub-decisions); `project_sight_taxonomy_foundation.md`; `project_sight_canonical_answer.md`; `project_sight_360_scope_orthogonal.md`

---

**End of MIG-021 Architect.** Awaiting Boss Phase 2 sign-off. On approval, the Plan doc will sequence §1A–§1K into landable steps with per-step verification clauses; on Plan approval, the Build cascade follows the standard /migration discipline.
