# MIG-009 — Lens-to-Sight Naming Cleanup

**Owner**: Eisa ALSHAMSI
**Migration lead**: Claude
**Opened**: 2026-04-27
**Scope**: β (Architectural) — file rename + Tauri command rename + frontend IPC invoke. Frontend JS variable names (`lensActive`, `toggleLens`, etc.) remain unchanged for now.
**Status**: Phase 3 — Build (cascade)

---

## Phase 1 — Architect

### Why

`lens.rs` is the analytics module that powers the **Constellation Sight** UI surface (centrality / community / structural-gaps / universe-health). The user-facing UI was renamed from "Constellation Lens" to "Constellation Sight" earlier; the backing Rust file's name was left lagging. Project owner's principle (2026-04-27): file names should reflect what they actually power. Cleanup brings the architecture in line with the UX.

### Distinction from `lenses.rs`

The two files (singular/plural) serve different purposes and stay distinct:

- **`lens.rs` (singular)** — Sight analytics. Brandes betweenness centrality, tag-shared edges. Live, drives Constellation Sight via `toggleLens()` in `+layout.svelte`. **THIS file is what we rename to `sight.rs`.**
- **`lenses.rs` (plural)** — CE Phase 9 Multi-Lens definitions + scanners. `apply_lens` is dead code (verified 2026-04-27, see `project_lenses_apply_lens_dead_code.md`). **STAYS as `lenses.rs`** — to be tackled when CE Phase 9 is resumed.

### Invariants to preserve

1. Tauri IPC contract — frontend invokes by exact command name. Rename must update both sides atomically.
2. `lib.rs::generate_handler!` block must enumerate the new handler names; release build will fail to compile otherwise.
3. No schema, no triggers, no user data touched. Pure code rename.
4. The `lens.rs` file does NOT export anything to `lenses.rs` or vice versa — confirmed by grep (no cross-references between the two). Rename is independently safe.

---

## Phase 2 — Plan (already approved)

| Step | File | Change |
|---|---|---|
| 1 | `src-tauri/src/lens.rs` | Move to `src-tauri/src/sight.rs` |
| 2 | `sight.rs` (the moved file) | `pub fn constellation_lens_centrality` → `pub fn constellation_sight_centrality`; `pub fn constellation_lens_tag_edges` → `pub fn constellation_sight_tag_edges` |
| 3 | `src-tauri/src/lib.rs:16` | `mod lens;` → `mod sight;` |
| 4 | `src-tauri/src/lib.rs:310-311` | `lens::constellation_lens_centrality` → `sight::constellation_sight_centrality`; `lens::constellation_lens_tag_edges` → `sight::constellation_sight_tag_edges` |
| 5 | `src/routes/+layout.svelte:3235` | `invoke('constellation_lens_centrality', ...)` → `invoke('constellation_sight_centrality', ...)` |

### Verification per step

- After steps 1-4: `cargo check --release` clean (no orphan references).
- After step 5: `npx vite build` clean (no orphan IPC name).
- After Tauri build + user reopens: Constellation Sight still renders with centrality / community / gap data populated. (Same code path that worked before; we just renamed the symbols carrying it.)

### Out of scope

- Frontend JS variable names (`lensActive`, `toggleLens`, `lensCentrality`, `lensCommunities`, `lensCommunityAssignments`, `lensGaps`, `lensHealth`, `lensLoading`, `lensDataStale`, `availableLenses`, `activeLensId`) — these are internal JS naming. ~60 occurrences; renaming them is bookkeeping with no architectural payoff, so deferred.
- `lenses.rs` — separate file, separate concern, deferred to CE Phase 9 work.
- i18n strings — already user-visible as "Sight"; nothing to change here.

---

## Phase 3 — Build (cascade in progress)

Steps execute in order; each verified by `cargo check` / `vite build` before proceeding.

## Phase 4 — Audit

After build: re-grep `constellation_lens_` across the entire repo. Should return zero matches outside session logs / orientation docs. Re-grep `lens.rs` — should return only the deprecated-mention rows in `lenses.rs` neighbour files (orientation, etc., which get updated in the v1.6 bump).
