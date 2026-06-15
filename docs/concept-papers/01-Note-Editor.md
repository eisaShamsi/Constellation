# 01 — Note Editor (Concept Paper)

> The reference concept paper — demonstrates the template in [00-MASTER](00-MASTER-Bring-Up-Charter-and-Checklist.md) §3. The Editor is the **gate**: every other function attaches to it. It is the one function that stays ON in minimal mode.

## 1. Function in hand
The **Note Editor** — `NoteEditor.svelte` wrapping one of two surfaces per the note's mode: **NotePane** (`src/lib/components/NotePane.svelte`, full markdown) or **FocusPane** (`src/lib/components/FocusPane.svelte`, plain capture).

## 2. Purpose
Edit the content of one `.md` note, instantly and without data loss, and be the **single authority** for that note's in-memory state. It answers: *"what does this note say, and let me change it."* Everything else in Constellation (search, backlinks, graph, tags, second screen) is downstream of edits the Editor makes — so it is the entry point through which knowledge enters the system. Justified beyond doubt: without it nothing can be written; *File Over App* makes the `.md` file the source of truth and the Editor the window onto it.

## 3. What it is NOT
- **Not** a renderer-only viewer — it owns content, not just display.
- **Not** the place that computes derived views (backlinks, graph, tags). It *emits the change*; the derivation happens write-time downstream.
- **FocusPane is NOT a markdown editor** — plain text only, no parser, no decorations (capture-fast). That absence *is* its design.

## 4. Wiring
- **Inputs:** tab switch (`{#key tab.id}` remount); external file-watcher change (adopt-disk if the model is clean); revert/reload (`tab.reloadVersion` bump); universe switch (`closeAll()` clears models). Seeds body from the **model** (`seedBody`), never a stale prop copy.
- **Outputs (events):** `constellation:open-note` (wikilink/image/query click), `constellation:open-note-in-surface` (shift/alt-click), `constellation:open-external` (URL), `constellation:classify-and-show` (on save, if CECE on-save).
- **Outputs (IPC):** `write_note(path, content, origin)`; then `constellation_search_reindex` (FTS5/tags/links/word_count) + `constellation_embed_notes` (semantic, if enabled); `broadcastNoteSaved` to the second screen.
- **Consumers:** the search index, Backlinks/Outgoing/Tags (via the reindex it triggers), the file tree (active-tab sync), the Property editor (reads/writes `model.props`), the rename cascade.
- **Connection to the gate:** every consumer learns of a change **only** because the Editor dispatched an event / fired the reindex — there are no silent reads. That mandatory-dispatch is what makes the Editor *the* gate.

## 5. Right-click / context menu
- The Editor has a **specialized** in-editor context menu — MIG-077 classified "Editor" as a *specialized* menu, distinct from the shared `<ContextMenu>` action-menu used by the file tree / tabs / panels. Its exact items (text-selection formatting? link/wikilink actions? cut/copy/paste?) must be **enumerated and verified during the Editor's bring-up**, recorded here once confirmed — **not assumed** (BASIC RULE).
- Wikilink / image / embed interactions are handled by *click* (livePreview capture-phase mousedown), not the context menu.
- Bring-up action: confirm right-click inside the editor offers the expected actions; if it duplicates shared-menu logic, fold it (one source of truth — the MIG-077 intent).

## 6. Multilingual (by default)
- **Per-line bidirectional text** (`bidiPlugin`) + per-script font selection (Arabic, Hebrew, CJK, Devanagari, Cyrillic) is a *core* editor feature — mixed-script content in any view, in **both** NotePane and FocusPane; RTL per line via `detectDir()`.
- Chrome strings (breadcrumb, stage-lifecycle controls, floating + table toolbars, word-count footer, autocomplete labels) flow through `$t()` in all 15 locales.
- Note **content** is stored original (not normalized) so it round-trips faithfully in any language; Arabic search-normalization happens at index time + name-side at query time — never lossily in the file.
- No layout / cursor / input assumption is single-language.

## 7. Boot behavior
- **Runs at boot?** Only the **first tab-open** mounts a pane (`EditorState.create()` once per tab); the app shell paints before any editor exists.
- **Rule 8 status:** ✅ compliant — the Editor reads one note's content on open; it recomputes nothing universe-wide.
- **Cost (measured 2026-06-15):** `paint_ms=941`, `hydrated_ms=1671` — the editor mounts inside that and is cheap. Per-tab open ≈ 1–3 ms disk read (estimated). The Editor is **not** a boot bottleneck.

## 8. Flag / gate & bring-up position
- **Gate today:** none — the Editor is unconditional core. Minimal mode needs a **new** core-spine guard only if we ever want a truly editor-only shell; in practice it simply *stays on* while satellites flip off.
- **Bring-up phase:** **1 (Core spine).** Depends on: the app shell + `cache_boot_snapshot_core` (note list for the tree) only.

## 9. Budget
- **Boot budget:** mount within the `hydrated_ms` envelope (<2 s) — already met.
- **Interaction budget:** **every keystroke instant (Rule 1)** — zero perceptible lag; no `invoke()` on the keystroke path; debounced save ≥1500 ms; `toString()` paid once per save, never per keystroke.
- **Regression guard:** type 10 chars rapidly in *both* NotePane and FocusPane (Rule 7); open a 5,000-word note and scroll (no stutter). Measure before/after any change touching `$lib/editor/` or the save path.

## 10. Acceptance checklist (the gate to "re-enabled" — here, "baseline proven")
- [ ] **Purpose:** edits persist; FocusPane↔NotePane round-trip on the same file loses nothing; no markdown features leak into FocusPane.
- [ ] **Serves Constellation's core purpose:** the Editor is Observation + Synthesis made concrete — it's where Acts become text (see [00-Constellation](00-Constellation-Core-Concept-Paper.md)).
- [ ] **Wiring:** a wikilink click opens the target; a save fires exactly one `write_note` + one reindex; the second screen receives `broadcastNoteSaved`.
- [ ] **Right-click:** the in-editor context menu's items are enumerated, work, and don't duplicate shared-menu logic.
- [ ] **Multilingual:** per-line bidi + per-script fonts render mixed Arabic/Hebrew/CJK; chrome strings localize ×15; content round-trips unmodified.
- [ ] **Budget:** 10-char burst in both surfaces shows no lag; 5,000-word scroll is smooth; no `invoke()` on the keystroke path.
- [ ] **Rule 8:** the editor recomputes no universe-wide derived view on open.
- [ ] **Content-integrity invariants (the top-principal class):** passes the full Editor-Surface Gate checklist — NotePane type-burst save; Focus enter→type→exit (no spurious write at enter); tab switch-away+return in both surfaces; tab switch *while in Focus*; PropertyEditor + stage promote; rename cycle with a linked probe pair; second-screen edit + sync; restart/workspace restore. On-screen === on-disk after every transition.
- [ ] **i18n / RTL:** per-line bidirectional text + per-script fonts work (Arabic/Hebrew/CJK); chrome strings localized ×15.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft (reference example)** · Enabled in bring-up: **yes (core, always)** · Budget met: **✓ (boot baseline measured: paint 0.9 s / hydrated 1.7 s)**
Notes: The Editor is the proven-fast baseline. The MIG-076 single-content-ownership ("buffer pattern") closed the content-integrity disease (BUG-012/015/019/023); the Editor-Surface Gate (§8) is its standing regression set. Sub-elements folded here (no separate paper): the floating toolbar, table toolbar, autocomplete (wikilink/tag/slash), live-preview decorations, callouts, the emoji/icon picker, the page-preview hover card.
