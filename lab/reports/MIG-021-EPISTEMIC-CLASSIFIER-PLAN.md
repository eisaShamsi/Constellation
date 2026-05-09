# MIG-021 — Epistemic Classifier (Sources Subsystem) — Plan

**Date**: 2026-05-09
**Architect**: [`MIG-021-EPISTEMIC-CLASSIFIER-ARCHITECT.md`](MIG-021-EPISTEMIC-CLASSIFIER-ARCHITECT.md) — approved by Eisa 2026-05-09 ("Enough of your never-ending technical questions. Proceed with the MIG-021 Architect Phase-2.")
**Anchored against**: [`docs/Constellation-Sight-Concept-Paper-v2.0.md`](../../docs/Constellation-Sight-Concept-Paper-v2.0.md) §7 + §8.
**Hard constraint**: zero impact on boot-perf budget (≤ 6 s hydrated; ≤ 1 s on Eisa's machine). Verified at the end of every commit.

---

## §0 · Open questions (locked with defaults)

The Architect §7 surfaced six questions. Eisa's directive is to proceed; defaults locked:

| # | Question | Locked default | Reversible? |
|---|---|---|---|
| Q1 | CDN URL for the Qwen3-1.7B Q4_K_M GGUF download | **GitHub Release asset on `eisaShamsi/Constellation`** — same distribution model as the installer. Zero new infrastructure. | Yes — change URL in Settings → AI manifest before Tier-2 ships in any release. |
| Q2 | Source-definition embedding text length | **~150 words per source**, drawn from the bilingual taxonomy doc (`docs/epistemic-content-taxonomy.md`) + Concept Paper v2.0 §7.1. Embedded as compile-time constant in `src-tauri/src/classifier/source_definitions.rs`. | Yes — regenerate at app build; old cached vectors shipped as constant. |
| Q3 | Classify on title + body, or body only? | **Title + body concatenated** (title carries strong signal — titles often name the source explicitly). Configurable in Settings → AI for advanced users in a future PJ if accuracy issues emerge. | Yes — single-flag swap. |
| Q4 | Long-note chunking strategy | **Tier 1**: first 2,000 characters (e5-small has 512-token window; covers most knowledge notes). **Tier 2**: full note up to 32k tokens (Qwen3 native context). | Yes — chunk size is a constant, easily tuned. |
| Q5 | 12th "unclassifiable" canonical token to opt-out specific notes from future classifier suggestions | **YES** — adds `unclassifiable` as a 12th canonical value in `sources:`. Suppresses `sources_suggested:` writes for that note. | Yes — clear from PropertyEditor. |
| Q6 | When Tier 2 is downloaded, auto-reclassify all notes? | **No automatic re-classification.** Settings → AI offers a one-click "Re-classify all with larger model" button; default = off. | N/A — user choice. |

These defaults are committed to in this Plan. The Build cascade does not pause to re-ask. If a default proves wrong mid-build, surface as a Stop-On-Architectural-Surprise per the Plan-Approval-Equals-Build-Approval rule (CLAUDE.md).

---

## §1 · Phased build (11 commits, 5 user-testable gates)

Each phase is one landable commit with its own verification clause. **Eisa Boss-tests at the gates marked ✅.** Other phases self-verify (type-check, /simplify, lint, schema-migration idempotency).

### Phase §1A — Schema migration + frontmatter read/write

**Goal**: ship the data substrate. `note_meta.sources` column added. `sources_suggestions` table created. Frontmatter `sources:` read/write helpers in Rust.

**Files touched**:
- **NEW** `src-tauri/src/sources/mod.rs` — public API: `read_sources(path)`, `write_sources(path, list)`, `read_suggestions(path)`, `write_suggestions(path, list)`, `clear_suggestions(path)`.
- **NEW** `src-tauri/src/sources/schema.rs` — `migrate_to_v1_sources()`: idempotent `ALTER TABLE note_meta ADD COLUMN sources TEXT DEFAULT NULL` + `CREATE TABLE IF NOT EXISTS sources_suggestions (...)` + `CREATE INDEX IF NOT EXISTS ...`.
- `src-tauri/src/lib.rs` — register the `sources` module + 3 IPCs (`sources_get_for_note`, `sources_set_manual`, `sources_clear`).
- `src-tauri/src/note_meta.rs` (or wherever the existing write-time triggers live) — extend the `scan_note_*` pipeline to re-parse `sources:` on note save and update `note_meta.sources`.

**Self-verification** (no Boss test):
- Schema migration runs idempotently: run twice in test, second run is no-op.
- `cargo check` passes.
- Manual: write `sources: [testimony]` to one trial-universe note's frontmatter by hand; restart Constellation; verify `note_meta.sources` mirror is populated within 5 seconds via `sqlite3 search.db "SELECT sources FROM note_meta WHERE path = '...';"`.

---

### Phase §1B — Tier 1 classifier (e5-small embedding-similarity)

**Goal**: the bundled classifier works. No UI yet. Callable via Tauri command.

**Files touched**:
- **NEW** `src-tauri/src/classifier/mod.rs` — public API: `classify_note(path) -> Vec<Suggestion>`, `classify_universe(library_set, opts)`.
- **NEW** `src-tauri/src/classifier/source_definitions.rs` — 11 source definitions as `const &str` arrays (EN canonical text, ~150 words each, drawn from taxonomy doc). Compile-time constant.
- **NEW** `src-tauri/src/classifier/tier1_embedding.rs` — at app startup (lazy, on first classifier call): embed the 11 definitions via the existing e5-small ONNX runtime (one-time cost ~200 ms). Cache the 11 × 384-dim vectors in `Once<Vec<[f32; 384]>>`. For each note: take title + first 2000 chars of body, embed, compute cosine similarity to each of 11 cached vectors, return top-3 sorted descending.
- **NEW** `src-tauri/src/classifier/queue.rs` — write/read `sources_suggested:` frontmatter + `sources_suggestions` SQLite table.
- `src-tauri/src/lib.rs` — register IPC `classifier_suggest_for_note(path)`.

**Self-verification**:
- `cargo check` passes.
- Manual: invoke `classifier_suggest_for_note` on 10 hand-picked trial-universe notes (mix: 3 quote-heavy, 3 logical-derivation, 2 observational-data, 2 mystical/inspirational). Inspect suggestions. **Acceptable: ≥7/10 top-1 suggestions are intuitively reasonable.** Below 7/10 = Q2 (source-definition text quality) needs revising before §1C.

---

### Phase §1C — Source Review sidebar panel ✅ Eisa Boss-test gate

**Goal**: the queue-based approval workflow surfaces. User can Accept / Edit / Reject suggestions.

**Files touched**:
- **NEW** `src/lib/sources/SourceReviewPanel.svelte` — sidebar panel component. Lists pending suggestions one note at a time (or paginated). Per-note: shows note title + current `sources:` (if any) + the suggested list with confidences + Accept / Edit / Reject buttons.
- **NEW** `src/lib/sources/sourcesStore.ts` — Svelte 5 store wrapping the Tauri IPCs (`sources_get_suggestions`, `sources_accept_suggestion`, `sources_reject_suggestion`).
- `src/routes/+layout.svelte` — register the new panel as one of the right-sidebar tabs (mirror the Review Pulse tab registration). Tab icon: a small annotated tag/check glyph.
- `src/lib/i18n/en.json` — add `sources.review.*` keys (~12 strings: panel title, button labels, empty-state, confidence-display, "no suggestions" hint).
- `src/lib/i18n/ar.json` — same keys, Arabic translations.
- New IPCs: `sources_get_suggestions(path)`, `sources_accept_suggestion(path, sources)`, `sources_reject_suggestion(path)`.

**Eisa Boss-test gate**:
1. After §1B, classifier has populated suggestions for ≥10 trial-universe notes (run from §1B verification).
2. Open Constellation; click the new **Source Review** sidebar tab.
3. Confirm: panel lists ≥10 notes with their suggestions. Each row shows note title + suggested sources with confidence percentages.
4. Click a note → Accept the top suggestion → confirm `sources:` field appears in that note's frontmatter (open the .md file in any editor) AND the suggestion is consumed (no longer in the queue).
5. Click another note → Edit (e.g., remove the second suggestion, keep only the first) → Accept → confirm only the kept source lands in frontmatter.
6. Click another note → Reject → confirm suggestion cleared, no `sources:` written.
7. Switch UI to Arabic → confirm all labels render correctly RTL.

If any step fails or is unclear, surface immediately and pause cascade.

---

### Phase §1D — PropertyEditor combobox (manual setting)

**Goal**: the user can set sources directly without going through the classifier queue.

**Files touched**:
- **NEW** `src/lib/sources/PropertyEditorSourcesField.svelte` — multi-select combobox component. Lists all 11 sources (locale-driven labels). Multi-select with primary-source ranking (drag-to-reorder or arrow buttons).
- `src/lib/components/PropertyEditor.svelte` — integrate the new field between Maturity and Stage rows.
- `src/lib/i18n/en.json` — add `sources.label.{source_id}` keys (11 strings) + `sources.description.{source_id}` (11 strings, used as combobox tooltips) + `sources.propertyEditor.*` (~6 strings).
- `src/lib/i18n/ar.json` — same keys, Arabic translations.

**Self-verification**:
- Type-check passes.
- Manual: select a note in the editor; PropertyEditor opens; Sources field appears between Maturity and Stage; multi-select 3 sources; reorder; save; confirm frontmatter and `note_meta.sources` mirror update.

---

### Phase §1E — Right-click "Suggest sources for this note" context action

**Goal**: on-demand single-note classification surfaces from any context where a note is selectable.

**Files touched**:
- `src/lib/components/contextMenus/NoteContextMenu.svelte` (or wherever the existing right-click menu is wired) — add "Suggest sources for this note" item. Action: invoke `classifier_suggest_for_note(path)` then open the Source Review panel scrolled to this note's entry.
- Wire the same action into Sky View node right-click and any other note-selectable surface.
- `src/lib/i18n/en.json` + `ar.json` — add `sources.contextMenu.suggest` key.

**Self-verification**:
- Right-click any note in the file tree → "Suggest sources for this note" appears in menu → click → Source Review panel opens with the new suggestion within ~3 seconds.

---

### Phase §1F — Background scan job ✅ Eisa Boss-test gate

**Goal**: the universe-scale resumable scan ships. Status-bar progress, opt-in per Universe, cancel-able.

**Files touched**:
- `src-tauri/src/classifier/queue.rs` — extend with scan-job orchestration mirroring `sky_backfill::maybe_schedule`. Chunks of 50 notes per batch with `requestIdleCallback`-style yielding between chunks.
- **NEW** `src-tauri/src/classifier/scan_job.rs` — `start_scan(library_set)`, `cancel_scan(scan_id)`, emits `classifier:progress` events.
- New IPCs: `classifier_scan_universe(opts)`, `classifier_cancel_scan(scan_id)`, `classifier_scan_status()`.
- `src/lib/components/StatusBar.svelte` — new status-bar group `.sb-classifier-progress` mirroring the MIG-015 `.sb-center` pattern. Shows "Classifying sources… 1,247 / 7,636 (~16%)".
- `src/lib/components/SettingsModal.svelte` (or wherever Settings tabs live) — new "AI" section with toggle: "Auto-classify sources for new and changed notes" (default OFF — user must opt in).
- `src/lib/i18n/en.json` + `ar.json` — add `classifier.progress.*` keys (~6 strings) + `sources.settings.autoScan.*` (~4 strings).

**Eisa Boss-test gate**:
1. Open Settings → AI; toggle on "Auto-classify sources for new and changed notes."
2. Confirm status-bar shows "Classifying sources… 0 / 7,636" and the count starts climbing.
3. Switch to a different tab; type 10 characters in a note; confirm typing is NOT lagged by the background scan (Performance Rule 1).
4. Close Constellation while scan is mid-flight; reopen; confirm scan resumes from where it left off (status shows e.g. "Classifying sources… 2,800 / 7,636" within 5 seconds of boot, NOT restarting from 0).
5. Click the status-bar group → cancel button → confirm scan stops and status clears.
6. Re-enable scan → run to completion → confirm Source Review panel now has hundreds of suggestions queued for review.

---

### Phase §1G — i18n EN + AR full pass

**Goal**: every new UI string + the 11 source labels translated across EN and AR. The other 13 locales fall back to EN via the existing $t() chain.

**Files touched**:
- `src/lib/i18n/en.json` — verify all keys from §1B–§1F are present + add `sources.unclassifiable` (the 12th canonical token Q5).
- `src/lib/i18n/ar.json` — same keys, Arabic translations.

**Self-verification**:
- Switch UI to Arabic; click through every new surface (Source Review panel, PropertyEditor sources field, right-click menu, Settings → AI, status-bar progress); confirm RTL renders correctly and no key-strings leak (literal `sources.foo.bar` showing instead of translated text).
- Switch to a non-EN-non-AR locale (e.g. Spanish); confirm Source Review panel renders English fallback (no broken keys).

---

### Phase §1H — Tier 2 download + llama.cpp integration ✅ Eisa Boss-test gate

**Goal**: the optional larger classifier path works end-to-end.

**Files touched**:
- `src-tauri/Cargo.toml` — add `llama-cpp-2 = "0.1"` (or current pinned version) crate dependency.
- **NEW** `src-tauri/src/classifier/tier2_llm.rs` — llama.cpp wrapper. `init_tier2(model_path)` lazy-loads the model. `classify_with_tier2(note_text) -> Vec<Suggestion>` runs few-shot prompt + GBNF-grammar-constrained generation. Output is guaranteed JSON matching schema `{ sources: [{name, confidence, evidence}] }`.
- **NEW** `src-tauri/src/classifier/prompt.rs` — few-shot prompt template + GBNF grammar definition for the 11-source enum.
- **NEW** `src-tauri/src/classifier/tier2_download.rs` — resumable HTTP download from `https://github.com/eisaShamsi/Constellation/releases/download/sight-v5-classifier/qwen3-1.7b-q4_k_m.gguf` (Q1 default; verify URL exists in the release before §1H ships).
- New IPCs: `tier2_download_model(opts)`, `tier2_status()`, `tier2_unload()`, `classifier_reclassify_all_with_tier2()`.
- **NEW** `src/lib/sources/SettingsAIPanel.svelte` — Settings → AI section: Tier-2 download button (with progress, resume, cancel) + classifier-status indicator + "Re-classify all with larger model" action button + "Use larger classifier when available" toggle.
- `src/lib/i18n/en.json` + `ar.json` — add `sources.settings.tier2.*` keys (~15 strings).

**Eisa Boss-test gate**:
1. Settings → AI → confirm "Larger classifier" status reads "Not downloaded".
2. Click "Download larger classifier" → confirm progress bar appears, completes ~2-10 min depending on connection.
3. Confirm status flips to "Downloaded — ready" with model size + version info.
4. Right-click a note in Arabic → "Suggest sources for this note" → confirm Source Review now shows a Tier-2 suggestion (visibly different from Tier-1 — likely better quality + more confident).
5. Settings → AI → click "Re-classify all with larger model" → confirm queue refills with Tier-2 suggestions for all already-`sources:`-set notes.
6. Confirm Tier-2 model can be unloaded (Settings → AI → "Unload model from memory") to free RAM.

---

### Phase §1I — Help docs + User Manual EN + AR

**Goal**: the user can read about Sources end-to-end before deciding to opt in.

**Files touched**:
- `docs/User Manual.md` — new section under §3 Creating and Editing Notes: "Source Tags". Explains: what sources are, why they matter (link to `docs/help.uConstellation.World/Sources/Sources.md` for the full taxonomy), how to set them manually, how the classifier proposes them, how to review, opt-out via "unclassifiable", optional larger classifier.
- `docs/help.ar/User Manual.md` — Arabic translation of the same section.
- **NEW** `docs/help.uConstellation.World/Sources/Sources.md` — full help topic with the 11-source vocabulary (linking to `docs/epistemic-content-taxonomy.md` for the scholarly foundation), worked examples, FAQ.
- **NEW** `docs/help.ar/Sources/Sources.md` — Arabic translation of the help topic.
- 13 other locales: queued as a follow-up PJ. Per the BASIC RULE, no inventing translations.

**Self-verification**:
- Read both EN and AR sections from a first-time-user perspective; confirm the 11 sources are explained without civilizational jargon dominating; confirm the link from Manual → help topic works.

---

### Phase §1J — `/simplify` checkpoint + 3-agent audit

**Goal**: the standard /migration audit. Spawn three agents in parallel: Invariant (do P1–P12 hold?), Drift (any new guards the system doesn't know about?), Migration-path (first-boot, mid-backfill, downgrade, Tier-2 corruption).

**Files touched**:
- /simplify pass over the full diff §1A–§1I. Address any Tier-1 findings (real bugs, perf regressions, dead code) before audit.
- Audit findings written to `lab/reports/MIG-021-EPISTEMIC-CLASSIFIER-AUDIT.md`.
- P0/P1 findings fixed in close-out commit; P2/P3 deferred to follow-up PJs.

**Self-verification**:
- /simplify produces zero unaddressed Tier-1 findings.
- All three audit agents report PASS or PASS-WITH-FIXED-P0/P1.

---

### Phase §1K — Close-out

**Goal**: MIG-021 marked Done. Orientation bumped. Session log appended. PCS.

**Files touched**:
- `docs/Constellation Orientation & Onboarding v1.78.md` (or whichever version is next at close-out) — preamble bump per SO #6 (subsystem ships major feature is the trigger). Body §3 / §4.x updated to reflect the new Sources subsystem + Source Review panel + Settings → AI section.
- `lab/reports/SESSION-LOG-YYYY-MM-DD.md` — append phase log.
- `docs/Constellation Pending Jobs vN.md` — mark MIG-021 / PJ-NNN as Done. Allocate PJ for 13-locale Sources translation.

**Self-verification**:
- `git push origin main` succeeds.
- Boss confirmation: "Sight v5 Sources foundation is shipped" — no further regressions surface in the next 24-48 hours of use.

---

## §2 · Sequencing diagram

```
§1A  Schema  ───────────►
§1B  Tier 1 classifier  ──────►
                              ✅ §1C  Source Review panel  ──────►
                                                              §1D  PropertyEditor combobox  ──►
                                                                                        §1E  Right-click action  ──►
                                                                                                        ✅ §1F  Background scan  ──►
                                                                                                                          §1G  i18n EN+AR  ──►
                                                                                                                                       ✅ §1H  Tier 2 + llama.cpp  ──►
                                                                                                                                                                §1I  Help docs  ──►
                                                                                                                                                                              §1J  /simplify + audit  ──►
                                                                                                                                                                                              §1K  Close-out + PCS
```

3 user-testable Boss-test gates: §1C, §1F, §1H. Other phases self-verify and cascade autonomously.

---

## §3 · Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Tier-1 e5-small classifier accuracy < 65% top-1 (below threshold for usable suggestions) | Medium | §1B verification gates further phases. If <65%, revisit Q2 source-definition text quality before §1C. |
| llama-cpp-2 crate has Windows-build issues (cmake / MSVC toolchain) | Medium | Test build locally on Windows before §1H lands; document any extra setup in CONTRIBUTING.md. |
| Tier-2 download URL not yet hosted on GitHub Releases | Low | Upload Q4_K_M GGUF to a test release before §1H verification; commit URL constant only after upload confirmed. |
| Background scan thrashes on low-RAM machines (≤8 GB) | Low | Configurable chunk size (default 50; user can lower in Settings → AI advanced if needed). Background scan is opt-in, not on by default. |
| `sources_suggested:` frontmatter writes pollute git diffs on shared universes | Low | Document in help that `sources_suggested:` is a transient queue; users can `.gitignore` if it bothers them. Not a blocker. |
| Boot-perf regression from new schema migration on first launch | Low | Schema migration is idempotent + completes in <100 ms on Eisa's library. Verify in §1A. |
| 12th `unclassifiable` token confuses Sight v5 mode P (not a real source) | Low | Sight v5 mode P treats `unclassifiable` as a separate "User opted out" wedge alongside the "Unsourced" wedge — visible but distinct. Documented in Concept Paper v2.0 §7.4 follow-up. |

---

## §4 · Out of scope (this MIG)

- **Sight v5 visual** (the dome, the modes, the rendering). That's MIG-022.
- **Sight v5 mode P (Provenance) wiring** — the visual side. That's MIG-023.
- **Cleanup of `lenses.rs::apply_lens` and orphaned `constellation_sight_*` IPCs** — separate cleanup MIG.
- **Tier-1 / Tier-2 accuracy comparison eval set** — useful for tuning but not blocking; can ship as a follow-up.
- **Cross-platform validation beyond Windows** — Mac and Linux validation are blocking before any release ships; tracked separately in the release checklist, not in MIG-021.
- **AI-assisted translation of the 13 non-EN-non-AR locales** — queued as a separate follow-up PJ.

---

## §5 · Cross-references

- [`docs/Constellation-Sight-Concept-Paper-v2.0.md`](../../docs/Constellation-Sight-Concept-Paper-v2.0.md) — the design contract this Plan implements
- [`lab/reports/MIG-021-EPISTEMIC-CLASSIFIER-ARCHITECT.md`](MIG-021-EPISTEMIC-CLASSIFIER-ARCHITECT.md) — the Phase-1 Architect doc
- [`lab/reports/MIG-021-LOCAL-LLM-RESEARCH.md`](MIG-021-LOCAL-LLM-RESEARCH.md) — the LLM stack research
- [`docs/epistemic-content-taxonomy.md`](../../docs/epistemic-content-taxonomy.md) — the 11-source vocabulary
- Precedent Plans: `MIG-013-CTSE-PLAN.md`, `MIG-014-NOTE-STAGE-PLAN.md` (frontmatter + note_meta mirror pattern), `MIG-015 chunked job pattern`
- CLAUDE.md — Performance Rules 3 + 8; Working Agreements 4 + 5; Plan-Approval-Equals-Build-Approval

---

**End of MIG-021 Plan.** Awaiting Boss Phase-2 sign-off ("approved" or revision request). On approval, the Build cascade follows §1A → §1K autonomously, pausing only at the three Eisa Boss-test gates (§1C, §1F, §1H) and at any architectural surprise.
