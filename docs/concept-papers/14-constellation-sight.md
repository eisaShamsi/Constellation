# 14 — Constellation Sight (Concept Paper)

> Follows the template in [00-MASTER](00-MASTER-Bring-Up-Charter-and-Checklist.md) §3 and the reference [01-Note-Editor](01-Note-Editor.md). Sight is a **disabled satellite** — detached from core by MIG-038 (2026-05-19), re-categorized as a Constellation Wings external plug-in. This paper is the bring-up gate it must pass to be re-enabled. Where the running code can't tell us something, it says "unknown — verify in bring-up" (BASIC RULE).

## 1. Function in hand
**Constellation Sight** — the whole-universe epistemic-shape dashboard. The canonical render is **v6** (`src/lib/sight/v6/SightV6.svelte`, an anchor dome + Hearst-Flamenco facet sidebar + 4 mini-domes + tradition chip), mounted full-screen from `src/routes/+layout.svelte`. The sidebar register **`SightPanel.svelte`** ("what is the SHAPE of my thinking?") is a separate CNS surface. Internal v-numbers (v2/v6/v7), file names, and IPC names are retained as architectural history per MIG-025 §A.15 — the user-facing name is "Constellation Sight".

## 2. Purpose
Answer ONE question: *"How is my epistemic content shaped / organized?"* (Eisa, 2026-05-09 — the canonical Sight answer; InfraNodus-style metrics dropped). It serves the **Connection** Act primarily and **Synthesis** secondarily — by making the whole universe's structure visible at once (strata × time × tradition-lens), it surfaces where knowledge clusters, where it thins, and how a chosen epistemic grammar (Aristotelian maturity, masādir kinds-of-proof, pramāṇa, PaRDeS, etc.) reframes the same notes. Whether it *earns* re-enablement under Wings is the bring-up's call — MIG-038 detached it precisely because it could not justify staying in the core boot/perf budget.

## 3. What it is NOT
- **Not** single-note scope — Sight is the WHOLE universe. The single-note 360.3D view is a different, mutually-exclusive scope (memory: `project_sight_360_scope_orthogonal`).
- **Not** Sky View (PIXI bubbles) and **not** Constellation Map (D3 sunburst arcs) — Sight is the Canvas-2D anchor dome.
- **Not** a file finder or a metrics report — it is a cognitive-lens instrument, not InfraNodus.
- **Not** core — it is an external Wings plug-in candidate, off by default.

## 4. Wiring
- **Inputs (IPC):** `sight_v6_get_layout` (the layout cache rows), `sight_v6_get_link_set_for_notes` (edges for the loaded notes), `sight_v6_warm_cache` (prewarm), `sight_v6_read_user_traditions` + `sight_v6_read_user_plugins` (user-defined lenses). `SightPanel` separately reads `constellation_knowledge_health_snapshot` (cached Hubs).
- **Inputs (stores/props):** `appSettings` (gate + extended-mode), `$libraryStats`, `locale`/`t`. No file-watcher or note-model read.
- **Outputs:** the `onOpenNote(path, libraryName)` callback → `openNoteTab(...)` in `+layout.svelte`; sets `sightV6ReturnPending`. `SightPanel` dispatches `constellation:open-ccs`. No writes to disk, no `write_note`, no reindex.
- **Consumers:** none downstream — Sight is a terminal read surface. Nothing depends on Sight; Sight depends on the index the Editor maintains.
- **Connection to the Editor (the gate):** Sight is **purely downstream** of the Editor. It reads `note_meta` / `note_links` — the tables the Editor's save→reindex path keeps current — and re-enters the Editor only via `onOpenNote`. It never edits content. It attaches to the gate by *reading what the gate already wrote*; the Editor does not know Sight exists.

## 5. Right-click / context menu
- **None.** Grep of the entire `src/lib/sight/` tree and `SightPanel.svelte` for `contextmenu` / `oncontextmenu` / `ContextMenu` / `buildContextMenu` returns **zero matches**. Interaction is left-click (open note), Shift+click (cross-filter), wheel (zoom), drag (pan), and facet-chip clicks. The only chrome control is the header close button (`×`).
- **Gap flagged.** A whole-universe instrument plausibly *should* offer a right-click on a star — at minimum "Open note", "Open in new surface", "Copy title/path", "Filter to this library/tradition". Today none of that is reachable by right-click. **Bring-up decision:** add a star/chip context menu using the **shared `<ContextMenu>` + `buildContextMenu`** (MIG-077), not a hand-rolled one. No menu items can be enumerated as existing today — they don't exist yet (do not fabricate).

## 6. Multilingual
- `SightV6.svelte` uses `$t(...)` (34 occurrences) and imports `t` + `locale`; the layout title is `$t('sight.v6.title') || 'Constellation Sight'`. `SightPanel.svelte` routes its strings through `$t(...)` with English `|| 'fallback'` literals and handles RTL via `$dir` (`isRTL`, `dir={isRTL ? 'rtl' : 'ltr'}`) and `dir="auto"` on note-name spans.
- **Flag:** the `|| 'English'` fallbacks are pervasive in both files. They are safe only if every key truly exists in all 15 locales (ar de en es fa fr he hi ja ko pt ru tr ur zh) — **unverified here; verify each `sight.*` / `sightPanel.*` / `lens.*` key across all 15 locale files in bring-up.** Per the top-principal full-localization order, tradition chip names, stratum/sector labels, and dome details must all localize (including native equivalents, e.g. مصادر not a transliteration) — **whether every tradition label is localized is unknown — verify in bring-up.**

## 7. Boot behavior
- **Runs at boot?** **No.** `SIGHT_V6_ENABLED = false` (MIG-038) — the dock button, mount, and return button are all hidden. `backfill_sight_v6_layout` is **not** wired into the boot path in `lib.rs` (only the IPCs are registered). Sight does work only when the user opens it.
- **Rule 8 status:** **RECOMPUTES-on-read — a violation to fix in bring-up.** The write-path hook is only *half* present: triggers `sight_v6_layout_invalidate_au/_ad` **DELETE** stale rows on `note_meta` update/delete but **do not re-derive** (the comment states the fill needs multi-table joins "not expressible in a trigger body"). To compensate, `sight_v6_get_layout` (fix-3, 2026-05-19) **dropped its WHERE clause and now `INSERT OR REPLACE`-rebuilds the ENTIRE `sight_v6_layout` table from a universe-wide JOIN over `note_meta` on every call** — the code itself calls this "a few hundred ms per call" and justifies it only because "Sight open is rare." That is exactly the read-time-derivation shape Rule 8 forbids: the persisted table exists but is not trustworthy between reads, so the read re-walks the universe. The correct end-state is a trigger/hook that maintains `sight_v6_layout` per-note at write time (or a resumable post-paint backfill), making the read a cheap `SELECT`.
- **Cost:** full-table re-derive JOIN on every open — **"a few hundred ms" on a 7K-note universe (the code's own estimate; not independently measured here — verify in bring-up).** Boot cost today is **zero** (gated off).

## 8. Flag / gate & bring-up position
- **Gate today:** dev flag `SIGHT_V6_ENABLED` (currently `false`, MIG-038) **AND** user setting `appSettings.enabledFeatures?.constellationSightV6 !== false`. (v2/CNS uses `constellationSight`; v7 uses `SIGHT_V7_ENABLED`, also `false`.) Under Wings this likely needs a **new plug-in gate** rather than the in-core dev flag.
- **Bring-up phase:** **late satellite (post-core).** Depends on: a current `note_meta` / `note_links` index (Editor + reindex), `sky_nodes` (stratum/maturity source for the layout JOIN), and the Wings plug-in host. Must not re-enable until the Rule 8 read-time re-derive (§7) is fixed and the right-click gap (§5) is closed.

## 9. Budget
- **Boot budget:** **zero** — Sight must not run at boot. The layout backfill, if any, runs in the background after paint with status-bar progress and must be resumable (Rule 8 first-time-population clause).
- **Interaction budget:** Sight open ≤ ~250 ms to first paint on a 7K-note universe (target; today it pays a full-table re-derive — **must be brought under budget by persisting the derived view**). Zoom/pan/hover must stay at 60 fps on the canvas; no `invoke()` on the pointer-move hot path.
- **Regression guard:** measure open-time and pan/zoom fps before/after on a 7,600+-note universe; assert the `get_layout` call does **not** re-walk the universe once the write-time maintenance lands.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** answers "how is my epistemic content shaped?" — the dome/facets/tradition-lens make universe structure legible, no decorative/filler axes (Form-Aligns-To-Purpose).
- [ ] **Serves Constellation's core purpose:** advances the **Connection / Synthesis** Acts; not a metrics toy.
- [ ] **Wires correctly to the Editor:** reads only Editor-maintained tables; `onOpenNote` re-enters the Editor; writes nothing.
- [ ] **Right-click present + correct:** a star/chip context menu exists, built on the **shared `<ContextMenu>` / `buildContextMenu` (MIG-077)** — not hand-rolled; no action is reachable *only* by right-click without a non-mouse path.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** every `sight.*` / `sightPanel.*` / `lens.*` key plus all tradition/stratum/sector labels exist in all 15 locales with native equivalents; the `|| 'English'` fallbacks never surface.
- [ ] **Within budget:** zero boot cost; open within target; pan/zoom 60 fps; no `invoke()` on pointer hot path.
- [ ] **Obeys Rule 8:** `sight_v6_get_layout` is a cheap `SELECT` from a write-time-maintained `sight_v6_layout`; the universe-wide re-derive is gone; first-time population is a resumable post-paint backfill.
- [ ] **Holds its invariants:** tradition isolation (mini-domes use `starsDefault`, never the anchor's remapped positions); categorical traditions don't jitter within-cell.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** (`SIGHT_V6_ENABLED = false`, MIG-038 — Wings candidate) · Budget met: **— (Rule 8 read-time re-derive outstanding)**
Notes: Two hard blockers before re-enable — (1) **Rule 8 violation**: `sight_v6_get_layout` rebuilds the whole `sight_v6_layout` table from a universe JOIN on every call because the invalidation triggers only DELETE; fix by maintaining the derived rows at write time. (2) **No right-click menu** anywhere in the Sight tree — add the shared `<ContextMenu>`. Open verifications: full 15-locale coverage of every Sight key, the "few hundred ms" open cost (measure on 7.6K notes), and whether the v6 invariants (tradition isolation, no within-cell jitter) still hold. v2/CNS (`ConstellationSight2.svelte`, `SIGHT_V2_ENABLED = true`) and v7 (`SIGHT_V7_ENABLED = false`) remain on disk as fallbacks; this paper covers v6 as the canonical Sight.
