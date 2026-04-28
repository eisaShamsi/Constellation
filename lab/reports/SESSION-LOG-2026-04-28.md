# Session Log — 2026-04-28

---

## §91 — MIG-008 Canonical Naming Cleanup — closed

**Lab plan**: `lab/reports/MIG-008-CANONICAL-NAMING-CLEANUP.md`.

**What shipped**:

- Shared helper `note_display_name(path, content_opt)` in libraries.rs.
- Patched ~14 user-visible label sites across map.rs, inspector360.rs,
  strata.rs, maturity.rs, provenance.rs, review.rs, lenses.rs (dead-
  code-on-arrival per the wiring investigation), tasks.rs, tension.rs,
  libraries.rs::scan_index_words_recursive, trails.rs::find_note_
  recursive, universe.rs::collect_templates_recursive.
- Two correctness fixes (not just label fixes): inspector360.rs:88 now
  matches incoming wikilinks for canonical notes; trails.rs::find_note_
  recursive can now resolve canonical notes by display name (was
  deterministically broken before).

**User testing** (Stages 1, 3, 4a, 4b, 5): all PASS.

- Stage 1 (Constellation Map): tooltips show human titles.
- Stage 3 (Strata + Maturity + Provenance): all three correct.
- Stage 4a (Tasks panel): "Lunch Plan" displayed correctly.
- Stage 4b (Review Pulse): clean.
- Stage 5 (Tension via Health panel): clean.

**Stages skipped**:

- Stage 2 (360° Inspector): component deliberately disabled in
  +layout.svelte:84 — no UI surface to test.
- Stage 4c (Multi-Lens): apply_lens has zero frontend callers (dead
  code, see §92's investigation).

**Phase 4 audit** (3 parallel agents): all PASS.

- 4A Invariants: wikilink resolution unchanged, sky/backlinks/search
  unaffected, snapshot IPCs untouched, helper logic correct, command
  set intact, helpers properly scoped.
- 4B Drift: zero remaining `path.file_stem()` Category-B leaks; zero
  remaining old IPC names; zero remaining `lens.rs` source references.
- 4C Migration path: zero-note Universe safe; schema unchanged; no
  partial-state risk; IPC rename graceful; perf impact negligible.

---

## §92 — MIG-009 Lens-to-Sight Naming Cleanup — closed

**Lab plan**: `lab/reports/MIG-009-LENS-TO-SIGHT-NAMING.md`.

**What shipped**:

- `git mv src-tauri/src/lens.rs src-tauri/src/sight.rs` (file rename
  preserves git history).
- Function rename: `constellation_lens_centrality` →
  `constellation_sight_centrality`; `constellation_lens_tag_edges` →
  `constellation_sight_tag_edges`.
- `lib.rs:16` `mod lens;` → `mod sight;` and `lib.rs:310-311` handler
  list updated atomically.
- `+layout.svelte:3235` invoke renamed atomically with the backend.
- Frontend JS variables (`lensActive`, `toggleLens`, etc., ~60
  occurrences) intentionally NOT renamed — internal naming lag
  accepted.
- `lenses.rs` (plural — CE Phase 9 Multi-Lens) NOT renamed; separate
  concern.

**User testing**: Constellation Sight loads and renders centrality +
community clusters + structural gaps + universe-health header
correctly after rebuild. PASS.

**Audit**: same Phase 4 pass as MIG-008.

**Side-finding catalogued** (memory: `project_lenses_apply_lens_dead_
code.md`): `lenses.rs::apply_lens` is dead code (zero frontend
callers). Settings can still create + save lens definitions but
they're never applied. Decision deferred — delete or re-wire CE
Phase 9 Multi-Lens.

---

## Doc bump

`docs/Constellation Orientation & Onboarding v1.6.md` written
alongside v1.0..v1.5 per SO #6 versions-stack rule. Changelog notes
MIG-008 + MIG-009 closure, dead-code findings, and new backlog items
(CE Phase 9 fate, CE Phase 12 360° Inspector fate, IPC-CONTRACT.md
drift now also missing constellation_sight_* rename).

**Backlog status after this PCS**:

- MIG-007 — Links Settings tab consolidation (queued).
- CE Phase 9 Multi-Lens dead-code: delete or re-wire (decide).
- CE Phase 12 360° Inspector: re-enable or withdraw (decide).
- SecondScreenPage.svelte buildSkyData alias-blind (cosmetic).
- Architectural mystery: cache_boot_snapshot_sky bypass at boot.
- MIG-005 Steps 4-8 (alias-aware tension/inspector360/LinkDashboard).
- Constellation Map perf / search-highlight (separate from canonical-
  naming label which IS now fixed).
- IPC-CONTRACT.md ~5 weeks stale.

