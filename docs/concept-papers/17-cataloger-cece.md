# 17 — The Cataloger (CECE) (Concept Paper)

> A satellite of the Editor. It does not write `.md` content; it classifies the notes the Editor has written, and parks every classification behind the user's approval. Serves the root paper's **Connection** Act — it tells you what *kind* of knowledge a note is and where it came from. Verified against real code (MIG-039); items not confirmable from code are marked **unknown — verify in bring-up**.

## 1. Function in hand
The **Cataloger** — `src/lib/components/CatalogerView.svelte`, the full-page left-dock home for CECE (the Constellation Epistemic Content Engine, `src-tauri/src/cece/`). It composes a "Start scan" control, a "Classify a note…" picker, "Build all summaries", a live `ClassifierScanProgressStrip`, and the `SourceReviewPanel` suggestion queue.

## 2. Purpose
Classify each note along two orthogonal axes — its **Source** (kind of proof / where the knowledge comes from) and its **Content Type** — and surface those as suggestions the user accepts or rejects. The ONE job: *"what kind of knowledge is this note, and from what source?"* It serves the **Connection** Act of the Five Acts — placing a note in the epistemic map so it can be related to others. CECE runs a five-cataloger **heuristic ensemble** (the sixth, local-LLM "Reasoning" cataloger is designed-but-not-wired per the component header) and a synthesis layer that, on a split vote, **refuses to assign** and asks the user. It justifies itself: classification is the prerequisite for every source-aware view downstream, and it never auto-mutates a file — every assignment is human-approved.

## 3. What it is NOT
- **Not** an editor — it never writes note body content; it writes only source/content-type frontmatter, and only after the user accepts a suggestion.
- **Not** automatic background AI. Scans are **manual-only** (no boot scan); the on-save path is opt-in via Settings and rides the existing 1.5 s debounced save.
- **Not** a classifier that decides *for* you — on a split-confidence vote it defers to Sibling Disambiguation rather than guessing.
- **Not** Sight, Sky View, or the Index — those are separate epistemic surfaces.

## 4. Wiring
- **Inputs (IPC read):** `classifier_scan_status`, `nsc_backfill_status` (recover in-progress runs on mount); `constellation_search` (lexical, limit 10 — the note-picker); `sources_list_pending_suggestions` (the queue, read by the embedded `SourceReviewPanel`).
- **Inputs (events):** `classifier:scan` and `nsc:backfill` lifecycle (toggle button state); `constellation:classify-and-show` (queue update / flash).
- **Outputs (IPC write):** `classifier_scan_start` (manual universe-wide scan), `classifier_suggest_for_note` (single note), `nsc_backfill_start` (summary pre-compute). Accept/reject writes happen in `SourceReviewPanel` (`sources_*` commands).
- **Outputs (events):** dispatches `constellation:classify-and-show` so both the embedded queue and the right-sidebar SRP instance refresh.
- **Consumers:** `SourceReviewPanel` (both instances), the status-bar scan/backfill strips, and the source/content-type frontmatter that approved suggestions write to disk.
- **Connection to the Editor (the gate):** on save the Editor (`NoteEditor.svelte`, ~line 258) dispatches `constellation:classify-and-show` **only when** `appSettings.cece.backgroundScan === 'on_save'`. CECE therefore learns of a changed note **through the Editor's save event**, never by silently re-reading disk — it attaches downstream of the gate exactly as the root paper requires.

## 5. Right-click / context menu
- `CatalogerView.svelte` itself has **no** `oncontextmenu` handler — verified by grep (no `contextmenu`/`ContextMenu`/`buildContextMenu` in the component or in `SourceReviewPanel.svelte`). Inside the full-page view, classification is reached by the "Classify a note…" picker and "Start scan" buttons, not right-click.
- CECE's right-click entry point lives in the **shared MIG-077 menu** (`src/lib/components/contextMenuBuilder.ts`): the file-tree context menu offers **"Suggest sources"** (`$t('sources.contextMenu.suggest')`, icon ✨), gated to markdown targets (`target.isMarkdown`). This is the **good** path — shared `buildContextMenu`, not hand-rolled.
- Actions reachable **only** by right-click: classifying an arbitrary tree note in place (the picker covers the full-page case, but the per-file tree right-click is the in-context route).
- **Gap to flag (low):** the suggestion **cards** inside the queue have no right-click menu (accept/reject/open are buttons). Whether per-card actions (e.g. "open note", "skip", "re-classify") should be on a shared context menu is **unknown — verify in bring-up**.

## 6. Multilingual
- `CatalogerView.svelte` routes every user-facing string through `$t()` — `cataloger.title`, `cataloger.tagline`, `cataloger.classifyNote`, `cataloger.searchNotes`, `cataloger.noNotesFound`, `nscBackfill.*`, `settings.classifier.scan*`, `common.close`. The `cataloger.*` keys are confirmed present in all **15** locale files (ar de en es fa fr he hi ja ko pt ru tr ur zh).
- Each `$t()` call carries an inline **English literal fallback** (`|| 'The Cataloger'`). These are safety nets, not hardcoded copy — the locale keys exist, so they should not surface; **verify in bring-up** that no fallback renders for a real locale.
- RTL: the root `<div>` sets `dir={$dir}`; the picker popover uses `inset-inline-end` and `text-align: start` (logical properties) — RTL-aware. The `pickerError` string renders `String(e)` (a raw backend error) — not localized, but it's a diagnostic, not chrome.
- Queue copy (filter chips, per-cataloger badges) lives in `SourceReviewPanel` and is also `$t()`-driven. Whether the **cataloger names / source taxonomy labels** are fully localized (per the standing "everything adapts" order) is **unknown — verify in bring-up**.

## 7. Boot behavior
- **Runs at boot?** **No.** No scan or classification is triggered on startup. `classifier_scan_start` spawns a worker thread only on explicit user action; `CatalogerView` runs IPC only `onMount` (when the user opens the dock), and even then only **status** reads (`classifier_scan_status`, `nsc_backfill_status`) to recover an already-running job.
- **Rule 8 status:** ✅ **reads-stored.** The queue is `SELECT … FROM sources_suggestions ORDER BY created_at DESC` (`src-tauri/src/sources/mod.rs::sources_list_pending_suggestions`) — a persisted table, not a re-walk of the Universe. Classifications are computed at write-time (manual scan / on-save / right-click) and stored; reads are cheap lookups. No `scan_*`/`rebuild_*` on read.
- **Cost:** boot cost ≈ **0** (nothing runs until the user opens the Cataloger). Opening it pays a few status IPCs + one `sources_list_pending_suggestions` query (renders first ~80 cards). A manual scan over a large Universe is the heavy path; per-note classify cost is **unknown — verify in bring-up** (measure on the 7,600-note Universe).

## 8. Flag / gate & bring-up position
- **Gate today:** `appSettings.enabledFeatures.cece` — the left-dock button renders under `{#if $appSettings.enabledFeatures?.cece !== false}` (`+layout.svelte` ~line 5250); default-on. The on-save behavior is separately gated by `appSettings.cece.backgroundScan`.
- **Bring-up phase:** **after the Editor (Phase 1)** — CECE is a satellite that attaches to the Editor's save event and reads the search DB. Depends on: the Editor + save path; the search/`sources_suggestions` schema (`ensure_search_db_ready`); the shared MIG-077 context menu; `SourceReviewPanel` + `ClassifierScanProgressStrip`. Re-enable only once those are green.

## 9. Budget
- **Boot budget:** **0 added** — it must not run at boot (already true); regression = any startup-time scan trigger.
- **Interaction budget:** opening the dock < one frame after status IPCs; picker search debounced 300 ms; on-save classify must **never** fire per-keystroke (rides the ≥1500 ms debounced save — Rule 1/3). No `invoke()` on the keystroke path.
- **Regression guard:** confirm no boot scan; type a 10-char burst with `backgroundScan: 'on_save'` and confirm classify fires once per save, not per key; open the dock on a large Universe and confirm the queue renders capped (80 cards) without stutter.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** Source + Content Type suggestions appear; split votes defer to the user; accepted suggestions write the right frontmatter; rejects don't.
- [ ] **Serves Constellation's core purpose:** classification advances the **Connection** Act (root paper) — places notes in the epistemic map; never auto-mutates a file.
- [ ] **Wires to the Editor:** on-save dispatch fires `constellation:classify-and-show` only when `cece.backgroundScan === 'on_save'`; the queue updates; the right-sidebar SRP stays in sync.
- [ ] **Right-click present + correct:** "Suggest sources" comes from the shared MIG-077 `buildContextMenu` (not hand-rolled), gated to markdown; per-card menu need is decided.
- [ ] **Multilingual ×15 + RTL:** all `cataloger.*` keys resolve in every locale (no English fallback surfaces); `dir={$dir}` + logical properties flip correctly; cataloger/source labels localize.
- [ ] **Within budget:** zero boot cost; on-save classify never per-keystroke; large-Universe queue renders capped without stutter.
- [ ] **Obeys Rule 8:** queue read = `SELECT` from `sources_suggestions`; no scan/rebuild on read or boot.
- [ ] **Holds its invariants:** no file content written without explicit accept; manual-only scan unless on-save opted in; split-vote refusal honored.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **—** · Notes: CECE ships as a **5-cataloger heuristic ensemble** (the local-LLM Reasoning cataloger is designed-but-not-wired). MIG-039 promoted it to a full-page left-dock home; MIG-040 made the queue newest-first. Open verifications: per-card right-click need; full localization of cataloger/source-taxonomy labels; per-note classify cost on a large Universe; that no English `$t()` fallback ever renders for a real locale.
