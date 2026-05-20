# Session Log — 2026-05-20

## Function in hand
**The Cataloger** — promoting CECE (the Constellation Epistemic Content Engine) from a right-sidebar "Source Review" tab to a first-class **left-dock Core Plug-in** with its own dock button + full-page view. This is **MIG-039** (handover: `lab/reports/MIG-039-CATALOGER-HANDOVER.md`).

## Pre-build checklist (done)
- `git pull origin main` → already up to date.
- Read CLAUDE.md, the MIG-039 handover, the CECE Concept Paper v1.0, LESSONS-LEARNED.md.
- **SO #8 cross-check**: confirmed MIG-039 is NOT stale. No `showCataloger` exists anywhere in `src/`; orientation v2.18 body + preamble both confirm CECE is still the right-sidebar Source Review panel with the left-dock build pending as MIG-039.
- Studied the OrgChart wiring (the pattern to mirror), the two reuse components, and the `enabledFeatures` type/defaults.

## Architectural impact analysis (Working Agreement #4) — what wires this touches, what it cuts
**Purely additive. Cuts nothing.**
- New `CatalogerView.svelte` — touches no existing code.
- `store.ts` — `+ cece: boolean` in `enabledFeatures` type; `+ cece: true` default. Settings-load spread (`...DEFAULT_SETTINGS.enabledFeatures`) back-fills existing users → default ON. No force-off (unlike Map).
- `+layout.svelte` — `+ showCataloger` state, `+ catalogerEverOpened` lazy-mount flag (LL-022) + effect, universe-switch reset, `fullPageActive` term, `content-hidden` term, dock button (gated `enabledFeatures.cece !== false`), lazy-mounted overlay, command-palette entry, escape branch, and `showCataloger = false` beside every sibling-close site.
- CSS — `.cataloger-overlay` appended to the shared overlay selector (no duplicated rules).
- i18n — `ribbon.cataloger`, `commands.cataloger`, `cataloger.title`, `cataloger.tagline` × 15 locales.
- **No IPC contract change, no schema change, no write-path change.** New `$effect` reads `showCataloger` / writes `catalogerEverOpened` (different vars) → no loop (Rule 2).
- **Right-sidebar Source Review tab untouched** (Concept Paper §10 / handover §5). Two SourceReviewPanel instances coexist safely (each fetches its own queue; both idempotently refresh on the shared `constellation:classify-and-show` window event).

### Deviation from handover §7 (flagged)
The handover assumed `ClassifierScanProgressStrip` carries "scan controls." It does not — it's a passive progress strip (shows progress + Cancel only while a scan runs). The whole-library scan is started via `classifier_scan_start` (today only from SettingsModal). So `CatalogerView` includes its own **"Scan library" button** mirroring `SettingsModal.startClassifierScan`, so the Boss can run a manual universe-wide scan from the dock view. Within the plan's intent (universe-wide home + scan controls).

### Accuracy guardrails (Concept Paper §5/§6, handover §6)
- The local-LLM "Reasoning" cataloger is DESIGNED-but-NOT-WIRED → CECE ships as a 5-cataloger heuristic ensemble. **No UI copy calls The Cataloger "AI" / "LLM".**
- Background auto-scan is not wired → scans are manual-only. **No UI copy implies automatic background classification.**

## Build progress
All code complete; NOT yet committed (per handover: stop at Boss-test verification; commit + orientation v2.19 bump happen "when it ships" = after Boss validates).

- **Step 1 — `src/lib/components/CatalogerView.svelte`** (new): full-page view = "The Cataloger" header + tagline + a self-contained "Scan library" button (`classifier_scan_start`, mirrors `SettingsModal.startClassifierScan`, syncs disabled-state off the `classifier:scan` event lifecycle) + `ClassifierScanProgressStrip` + library-wide `SourceReviewPanel` (no `activeNotePath`). Bounded flex-column layout so SRP's `height:100%` internal scroll works. `onNoteClick` + `onClose` props.
- **Step 2 — `store.ts`**: `+ cece: boolean` in `enabledFeatures` type + `+ cece: true` default. Distinct from the existing `appSettings.cece` engine-settings object. Settings-load spread back-fills existing users → ON.
- **Step 3 — `+layout.svelte`** (25 sites, mirrors `showOrgChart`): import, `showCataloger` state, `catalogerEverOpened` lazy-mount flag + effect, universe-switch reset (LL-022), `fullPageActive` term, `content-hidden` term, dock button (after Index, gated `enabledFeatures.cece !== false`, layers SVG icon), lazy-mounted `.cataloger-overlay`, command-palette entry, escape branch, shared `.cataloger-overlay` CSS selector, and `showCataloger = false` added to all 12 sibling close-lists (one `replace_all`) + the 3 dock buttons that don't close OrgChart (Sky View / Index / Map).
- **Step 4 — i18n (15 locales)** via `scripts/add-cataloger-i18n.mjs`: `ribbon.cataloger`, `commands.cataloger`, `cataloger.title`, `cataloger.tagline`. en="The Cataloger", ar="المُصنِّف", other 13 = classifier-sense per Concept Paper §10. Clean diffs (8 lines/file, no reformat).
- **Step 5 — verify**: `svelte-check` → **3 errors, all pre-existing** (store.ts `fresh` + 2× PropertyEditor node-type); **0 new errors**. Release build running.

### Settings → Plug-Ins card (added on Eisa's request, 2026-05-20)
- Added a `{ id: 'cece', ... }` card to the **Discovery** group in `SettingsModal.featureGroups` (right after Index). `id: 'cece'` → `toggleFeature` writes `enabledFeatures.cece`; `getFeatureEnabled` defaults it ON. Reuses the already-localized `cataloger.title` / `cataloger.tagline` keys (no new i18n). Toggling it OFF hides the dock button (gate `enabledFeatures?.cece !== false`). Requires a rebuild so the Boss-test binary includes the card.

## Boss test round 1 (2026-05-20) — Steps 1-4 PASS; Step 5 blocked by a memory leak; layout wrong
Eisa's findings:
1. Steps 1-4 (dock button, tooltip, open full-page, Esc/reopen, Settings card toggle) **PASS**.
2. **Severe memory leak / unresponsiveness** on Cataloger boot + operation (had to force-close). Eisa adds: *the same leak already existed in the right-sidebar CECE* — so it's a pre-existing `SourceReviewPanel` bug, not the new wiring.
3. **Layout wrong**: the view rendered as a narrow centered column ("a window within a window") instead of using the full dock width.

### Root cause (diagnosed, not guessed)
`SourceReviewPanel` renders the **entire** pending queue with **no virtualization / no cap** (`{#each filteredQueue}` over all ~4,475 rows). Each card runs `parseComposite` (a `JSON.parse`) ≥2× (template + `cardNeedsUserCall`), and `filterCounts` + `splitAwareSkipCount` + `filteredQueue` each iterate the **full** queue parsing every blob. Opening the dock view (which loads the whole universe queue) → tens of thousands of DOM nodes + ~18k JSON parses in one synchronous pass → freeze. During a scan, `scheduleQueueReload` reloads every 1.5s on a growing queue → progressive slowdown. This is the CLAUDE.md **Rule 3** violation ("virtualize every list that can exceed 50 items"). It lived in the shared panel, hence the right-sidebar leak too.

### Fixes (in the shared `SourceReviewPanel` — cures both surfaces)
- **Memoize `parseComposite`** — component-local `Map` keyed by the immutable `composite_json` string; each blob parsed once, reused across counts/skip/filter/render and across scan reloads.
- **Cap the rendered cards** — `RENDER_BATCH = 80`; `visibleQueue = filteredQueue.slice(0, renderCap)`; render `visibleQueue`; **"Show more"** footer (+80) with a localized "Showing N of M" note; reset to one batch on filter change. Filter counts + Approve All still operate on the full queue — only the DOM is bounded.
- **i18n** — `cece.queueShowMore` + `cece.queueShowingCount` added to all 15 locales (via the script).
- **Layout (`CatalogerView`)** — removed the 880px centered column; the view now fills the full dock width.

`svelte-check`: still 3 pre-existing errors, 0 new. Rebuilding for Boss test round 2.

## Boss test round 2 (2026-05-20) — ALL 5 STEPS PASS
Eisa: Steps 1-5 all PASS; *"you nailed the memory leak."* Leak gone, full-width layout correct, scan runs responsively, approve writes frontmatter, right-sidebar Source Review unchanged.

### New finding (round 2): "Hide reasoning" chevron does nothing
- **Root cause (pre-existing, not introduced by MIG-039):** the trail toggle only tracked an `expandedTrails` ("force-open") set. Reasoning trails are **open by default** for a card when either (a) the user set `reasoningTrailVisibility: 'always'` in Settings → Intelligence → CECE, or (b) trust-cal is active (<50 reviews). On a default-open card, clicking the chevron added the path to the *expanded* set (a no-op — already open) with no way to force it *closed*. **Eisa correctly identified the 'always' Settings choice as the likely cause** (I had wrongly assumed trust-cal without checking — corrected).
- **Fix (`SourceReviewPanel`):** added a `collapsedTrails` ("force-closed") set; `toggleTrail(notePath, currentlyOpen)` now flips based on the card's current visible state; `isTrailOpen` checks `collapsedTrails` **first**, so an explicit per-card click overrides any global default-open ('always' or trust-cal). Correct for all 3 prefs + trust-cal. `svelte-check`: 3 pre-existing, 0 new. Rebuilding for a quick chevron re-verify.

## Chevron re-verify (round 3) — PASS
Eisa: "All pass." Collapse/expand works per-card with "Always show" set; cards collapse independently. Eisa correctly noted the open-by-default came from his `reasoningTrailVisibility: 'always'` Settings choice (I'd wrongly assumed trust-cal — corrected); the fix covers both because the explicit-collapse check runs first.

## Eisa direction (2026-05-20, end of session)
1. **Question answered — disambiguation bug.** Card split on BOTH axes: picking the Source chip (`cece_resolve_disambiguation`) writes the source, clears the whole suggestion, and co-writes the other axis only if it was *settled* (`extract_other_axis_settled` → None for a split axis). So **Content type is left unclassified + the card vanishes** before the user picks it. Real bug. Fix is cross-subsystem (backend keeps the suggestion until both split axes resolve + frontend keeps the card). **Deferred to the next increment.**
2. **NSC chartered.** Eisa wants a **"Note Summary Creator (NSC)"** subsystem (NOT a plain excerpt): use the note's own summary if present, else **summarize the whole note**, **language-agnostic**. Framed as a Constellation differentiator. Design next (Architect pass — local-first; e5-small embeddings enable extractive summarization without the unwired LLM).
3. **Sequencing**: **secure the Cataloger now** (this commit), then build NSC + the disambiguation fix.

## Commit (secure the Cataloger) — 2026-05-20
ONE commit: `CatalogerView.svelte` (new) + `+layout.svelte` (dock wiring) + `store.ts` (`enabledFeatures.cece`) + `SettingsModal.svelte` (Plug-Ins card) + `SourceReviewPanel.svelte` (leak fix: memoize + render cap; chevron fix) + 15 i18n locales + `scripts/add-cataloger-i18n.mjs` + orientation **v2.19** (new file) + this session log.

## MIG-040 — NSC (Note Summary Creator) + disambiguation fix (2026-05-20, post-Cataloger-commit)
Eisa direction: build NSC (extractive; **proven standard method**; all languages) + fix the disambiguation bug. Both verified at code level; bundled into one build for Boss test.

### Method (Eisa-approved): extractive, embedding-based **TextRank**
Cross-checked (WebSearch): TextRank/LexRank (graph PageRank over a sentence-similarity graph) is the canonical extractive standard; the modern variant uses sentence-embedding cosine similarity as edges. **Sentence segmentation = Unicode UAX#29** (`unicode-segmentation`), the cross-language standard, + paragraph/opening fallback for punctuation-less scripts (Thai/Lao). Multilingual via the existing e5-small model. Abstractive (LLM rewrite) deferred to when the local LLM is wired.

### NSC build (Phases 0–4)
- **Phase 0** — `docs/Constellation-NSC-Concept-Paper-v1.0.md`.
- **Phase 1** — `src-tauri/src/nsc/mod.rs`: `split_sentences` (UAX#29 + fallback), `textrank_top_k` (weighted PageRank, d=0.85, power iteration), `frontmatter_summary` (precedence: summary/description/abstract/excerpt), `summarize_body`/`compute_summary_for_note`. **10 unit tests pass** (en/ar/zh/hi/th split, ranker drops outlier, frontmatter precedence, opening-text). `mod nsc;` in lib.rs; `run_embedding_batch` made `pub(crate)`; Cargo: `unicode-segmentation`.
- **Phase 2** — `note_summaries` cache table (path PK, summary, source, content_hash, updated_at; created in `init_db`). `nsc_get_summaries_for_notes` (batched, cache-first, `#[command(async)]`) + `nsc_get_summary`. Registered in lib.rs. Rule 8 (cached derived view; content_hash invalidation) + Rule 3 (off the hot path; bounded by render cap; **zero per-card IPC**).
- **Phase 3** — folded into the batch IPC (no `SuggestionRecord` change / no JOIN needed).
- **Phase 4** — `SourceReviewPanel`: a Rule-2-compliant `$effect` (reads `visibleQueue`, writes `summaries`; plain `summaryRequested` Set) batch-fetches summaries for visible cards; displays under the title (above the reasoning), `dir="auto"`, accent-bordered. i18n `nsc.summary` ×15 (`scripts/add-nsc-i18n.mjs`). Shows in both the Cataloger and the right-sidebar card.

### Disambiguation both-axes-split bug — FIXED
`cece_resolve_disambiguation` rewritten: when the OTHER axis is still Split, re-insert the suggestion with the resolved axis marked settled (`mark_axis_resolved`) preserving `created_at`, and return the updated `SuggestionRecord`; clears the card (returns `None`) only when both axes are decided. New helpers `other_axis_needs_disambiguation` + `mark_axis_resolved` with **2 unit tests** (7 r7_tests pass). Frontend `resolveDisambiguation` keeps the card (refreshed) on a returned record, removes it on null.

### Verification
- `cargo test --lib`: nsc 10/10, r7_tests 7/7. svelte-check: 3 pre-existing, 0 new. Full release build running.

## Open / next (after the NSC Boss test + commit)
- Boss test (staged): (A) disambiguation — filter "Both axes need your call", resolve one axis → card stays for the other; (B) NSC — each card shows a summary (frontmatter wins; else extractive; works on an Arabic note).
- On PASS: commit MIG-040 (NSC + disambiguation) + orientation **v2.20** + this log + help-doc sync (Cataloger + NSC, 15 langs).
- **Deferred (NSC v.next, in Concept Paper §4/§8)**: a true background backfill worker for ALL notes (currently summaries compute lazily per visible card, cached) + abstractive (LLM) upgrade. Allocate PJ-NNNs.
- Orientation **body** (§4.x CECE + left-dock list) still pre-Cataloger — update with the v2.20 bump.

## NSC Boss test result + Cataloger cross-instance sync (2026-05-20, continued)

### NSC test PASS / Eisa's finding
Eisa tested NSC (Stage A all-pass). Then reported: **Arabic notes classified from the right-sidebar "Classify open note" button do NOT appear in the Cataloger queue.** A scan did not fix it either. Question: how to add "Classify open note" capability inside the full-page Cataloger (which has no open-note context).

### Root cause analysis
**Two root causes**, both in `SourceReviewPanel`:

1. **Cross-instance gap (classifyActiveNote)** — the right-sidebar's "Classify open note" button calls `classifyActiveNote()`, which updates the LOCAL SRP instance's `queue` state directly via `queue = [record, ...queue.filter(...)]`. It does NOT dispatch the `constellation:classify-and-show` window event (which both instances listen to). So the Cataloger's SRP never hears about it. Fix: one line — dispatch the event after the local queue update. Local instance self-guards via `classifying = true` in `handleClassifyAndShow`.

2. **Stale queue on Cataloger reopen** — the Cataloger's SRP is lazy-mounted once; it only calls `loadQueue()` in `onMount`. Reopening the Cataloger just un-hides the overlay (CSS); the SRP doesn't reload. Fix: add `visible` prop (passed from +layout as `showCataloger`); add a `_srp_was_closed` guard + `$effect` that calls `loadQueue()` when `visible` goes false → true (skips the first mount — `onMount` already handles it).

### Feature: "Classify a note…" note-picker in CatalogerView
The Cataloger is full-screen with no open-note context. Eisa asked: *"how are we going to do it, since the cataloger takes the whole space?"* The answer: a **note-picker** — a "Classify a note…" button in the header that opens a compact inline search popover. The user types a note name, results appear from `constellation_search` (lexical, limit 10), clicking a result calls `classifier_suggest_for_note` and dispatches `constellation:classify-and-show` to both SRP instances.

### Changes landed
- **`SourceReviewPanel.svelte`**:
  - `classifyActiveNote()`: after `queue = [record, ...]`, dispatch `constellation:classify-and-show` event. Local instance guards via `classifying = true`.
  - Add `visible` prop (default `true`); add `_srp_was_closed` flag + `$effect` for reload-on-reopen.
- **`CatalogerView.svelte`**: add `visible` prop (forwarded to SourceReviewPanel); add note-picker (debounced `constellation_search` + `classifyPickedNote` + event dispatch; ESC closes; timer cleared in `onDestroy`). Three new i18n keys: `cataloger.classifyNote`, `cataloger.searchNotes`, `cataloger.noNotesFound`.
- **`+layout.svelte`**: pass `visible={showCataloger}` to CatalogerView.
- **i18n (15 locales)**: `cataloger.classifyNote/searchNotes/noNotesFound` via `scripts/add-cataloger-picker-i18n.mjs`.

### Verification
`svelte-check`: 3 pre-existing, 0 new. `cargo check`: clean. Release build running.

## NSC override bug + queue-visibility root cause (2026-05-20, continued)

### Eisa's finding (the real bug)
After installing the cataloger-sync build, Eisa reported: **NSC is OVERRIDING notes that already have a summary** — generating its own TextRank summary even though the note already has one. He attached two images of the pyramid note `الهرم الأكبر`: image 1 = the note's own embedded summary callout; image 2 = the different TextRank summary NSC put on the Cataloger card. Design intent (agreed at NSC build time): *NSC generates only when the note lacks an author summary.*

### Root cause A — NSC ignored body summary callouts
`nsc::summarize_from_parts` checked only the **frontmatter** summary fields (`summary`/`description`/`abstract`/`excerpt`), then fell straight to extractive TextRank. The author's summary in the pyramid note is a **`> [!abstract] ملخّص` callout in the body**, not a frontmatter field — so NSC never saw it and generated its own. Verified against the real file: `E:\Cognitive Knowledge\العالم العربي\libraries\جغرافيا\معالم ومواقع\الهرم الأكبر.md` (federated cUniverse note) has no frontmatter summary key and a `[!abstract]` callout whose body matches image 1 verbatim.

**Fix** (`src-tauri/src/nsc/mod.rs`): new precedence — frontmatter → **body summary callout** → extractive. Added `body_callout_summary()` (+ `parse_callout_header`/`is_quote_line`/`strip_quote_prefix`/`collapse_ws`), mirroring `calloutPlugin.ts` syntax; summary-family types = `summary`/`abstract`/`tldr` (the 📋-icon family). New `SOURCE_CALLOUT = "callout"`.
- **Reads the RAW file, not `note_meta.body_text`.** `body_text` is markdown-stripped AND Arabic-normalized — the pyramid note's `body_text` reads `الهرم الاكبر او` (hamzas stripped, ة→ه folded) vs. the raw file's verbatim `الهرم الأكبر أو`. A user-authored summary must display verbatim, so we read the file (one read, only on a cache miss for a note with no frontmatter summary). 6 new unit tests, all pass (16 NSC tests total).
- **Cache invalidation** — the algorithm changed but the body didn't, so the cached TextRank summaries would persist. Added `NSC_ALGO_VERSION = "v2"` embedded in the cached `content_hash` (`body_hash`). A version bump makes every pre-v2 cached summary stale → recompute on next view (self-healing, no wipe). Proven cache-versioning pattern.

### Root cause B — newly-classified notes buried below the render cap
DB check (read-only on `Eisa Cognitive Knowledge`, 2.35 GB): the Arabic notes Eisa classified ARE persisted — `sources_suggestions` has **7203** pending rows incl. 1262 Arabic-script notes (pyramid note is the newest). So classify-from-sidebar persists fine. But `sources_list_pending_suggestions` ordered `created_at ASC` (oldest-first) and the panel renders only `RENDER_BATCH = 80` cards (`visibleQueue = filteredQueue.slice(0, 80)`). With 7203 pending, a just-classified note sorts to position ~7200 — **beyond the render cap, invisible** — even after a reload. This is why "the scan didn't update the list."

**Fix** (`src-tauri/src/sources/mod.rs`): `sources_list_pending_suggestions` now orders `created_at DESC, note_path ASC` (newest-first). A freshly-classified note lands at the TOP, in the first render batch — matching the live-classify path, which already prepends. Only consumer is `SourceReviewPanel` (both instances); newest-first is the correct review order for both.

### Why both fixes are needed together
Reload-on-reopen (root cause B from the previous section, already fixed) re-reads the queue; newest-first ordering puts the new notes where the 80-card cap can show them; callout precedence makes their summaries correct. All three combine so Eisa's classified Arabic notes appear at the top of the Cataloger with their own summaries.

### Verification (this increment)
`cargo test --lib nsc::` → 16 passed, 0 failed (incl. 6 new callout tests w/ verbatim Arabic). `cargo check --lib` → clean (42 pre-existing warnings, 0 errors). Release build running.

## Boss test: callout/ordering fixes (2026-05-20)
Build `Constellation_0.1.0_x64-setup.MIG040-callout-fix.exe`.
- **Stage 1 PASS** — pyramid card shows the author's `[!abstract] ملخّص` callout verbatim (not TextRank); newest-first ordering confirmed (`الهرم الأكبر` at top).
- **Stage 2 PASS** — classify from right sidebar (Cataloger closed) → reopen → note at top, no manual refresh.
- **Stage 3 PASS except #7** — note-picker search/classify/top-of-queue all work; **but pressing Escape closed the whole Cataloger** instead of just the picker popover.

### Stage-3 #7 bug + fix
**Root cause**: `+layout.svelte` registers `handleGlobalKeydown` on `document` in the **CAPTURE** phase (`addEventListener('keydown', …, true)`, ~line 2350) and closes the Cataloger on Escape (~line 2963). Capture runs outermost→innermost, so it fires before the picker's bubble-phase `onPickerKeydown` — `stopPropagation` in the picker's bubble handler is too late.
**Fix** (`CatalogerView.svelte`, frontend-only): register a **`window`-capture** keydown listener (window is outside document in the capture path → runs first). While `showPicker` is true it consumes Escape (`stopPropagation` + `closePicker`); when closed it's a no-op and Escape flows to the global handler as before. Removed the redundant bubble `onPickerKeydown` + its a11y-ignore. `onMount` adds the listener, `onDestroy` removes it. `svelte-check`: same 3 pre-existing errors, 0 new. Rebuilding for Stage-3 #7 re-test.

## Pending commit (after Boss test)
All MIG-039 (sync + picker) + MIG-040 (NSC + disambiguation + callout precedence + newest-first queue + picker Escape fix) changes uncommitted. Will commit together as one "MIG-040" commit after Boss test passes; orientation v2.20 in the same commit.
