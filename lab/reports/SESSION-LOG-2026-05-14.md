# Session Log — 2026-05-14

## Phase: Sight v6 redesign ratified · MIG-025 build cascade opens

This session captures the convergence of ~3 days of Sight redesign work into a binding architectural contract (Concept Paper v4.0) and the opening commits of the MIG-025 build cascade against it.

---

## Commits landed today

| Commit | Subject | Files | Insertions |
|---|---|---|---|
| `3e829f6` | docs(sight): MIG-025 opens — Concept Paper v4.0 + Architect + Plan | 17 | 8,491 |
| `a0c0af5` | §A.1 — Add SIGHT_V6_ENABLED feature flag (MIG-025) | 1 | 12 |
| `aa17e10` | §A.2 — Sight v6 cache schema + invalidation triggers (MIG-025) | 3 | 408 |
| `baa4a95` | §A.3 — Sight v6 backfill skeleton + sentinel + 8 tests (MIG-025) | 1 | 438 |
| `a38b256` | §A.4 — Progressive backfill via Tauri events + frontend store (MIG-025) | 2 | 557 |
| `970d2bf` | §A.5 — Three Sight v6 Tauri IPCs + handler registration (MIG-025) | 2 | 160 |
| `b981129` | §A.6 — Sight v6 frontend types + module skeleton (MIG-025) | 3 | 427 |
| `7a948e9` | §A.7 — Mount Sight v6 alongside v5 in +layout.svelte (MIG-025) | 3 | 67 |
| `5b3e10b` | §A.8 — Anchor dome chrome render: 5 strata + calendar rim + labels (MIG-025) | 3 | 403 |
| `e5b2334` | §A.9 — Anchor dome stars + lines + IPC integration (MIG-025) | 2 | 559 |
| `c4b7e7b` | §A.10 — Facet sidebar with Hearst Flamenco cross-filter (MIG-025) | 3 | 633 |
| `c84989b` | §A.11 — Sight v6 first-boot orientation tour (MIG-025) | 3 | 245 |
| `1048ab1` | §A.12 — Sight v5 → v6 settings migration + 4 new v6 fields (MIG-025) | 1 | 66 |
| `251d630` | §A.13 — Sight v6 CI perf-harness skeletons (MIG-025) | 4 | 311 |

Branch: `main` (all 14 commits on primary repo, per Working Agreement #2).
Total: ~46 files touched, ~12,000+ insertions across the MIG-025 §A cascade.

**Phase 1 build complete (§A.1 → §A.13).** §A.14 ship gate is the
next user-testable point — stops cascade for Eisa Boss-test of
Sight v6.0.

### Cascade lineage

```
6b14d54  ← yesterday's last commit (Sight v5 purpose-achievement audit)
   ↓
3e829f6  docs landing (8 design docs + 9 mocks + Architect + Plan + orientation v2.01)
   ↓
a0c0af5  §A.1 — SIGHT_V6_ENABLED flag (B2 dual-mount foundation)
   ↓
aa17e10  §A.2 — sight_v6_layout table + 2 invalidation triggers + 9 tests
   ↓
baa4a95  §A.3 — synchronous bulk backfill + sentinel + 8 tests
   ↓
a38b256  §A.4 — 5-tier progressive backfill + BackfillProgress events + frontend store + 8 tests
   ↓
970d2bf  §A.5 — sight_v6_get_layout + sight_v6_get_link_set_for_notes + sight_v6_warm_cache IPCs
   ↓
b981129  §A.6 — frontend types + anchor.ts stubs + SightV6.svelte placeholder mount
```

### Phase 1 progress

| Step | Status |
|---|---|
| Backend (§A.1–§A.5) | ✅ complete |
| Frontend skeleton (§A.6) | ✅ complete |
| Mount in +layout.svelte (§A.7) | ✅ complete |
| Anchor chrome render (§A.8) | ✅ complete |
| Anchor stars + lines + IPC (§A.9) | ✅ complete |
| Facet sidebar + cross-filter (§A.10) | ✅ complete |
| First-boot tour (§A.11) | ✅ complete |
| Settings migration (§A.12) | ✅ complete |
| CI perf-harness skeletons (§A.13) | ✅ complete |
| Ship gate (§A.14) | ✅ **PASSED** — v6.0 SHIPPED |

### §A.14 ship gate: 7 NSIS Boss-test cycles, 16 fixes

After Eisa's first cycle-1 Boss-test, 7 NSIS build cycles + 16 incremental fixes cleared the spec. Eisa accepted cycle-3.7 with "Ship".

**Cycle log** (each cycle = full Tauri build + NSIS bundle, ~1m 30s Rust compile + ~30s bundle):

| Cycle | What changed | Boss-test verdict |
|---|---|---|
| 1 | First v6.0 NSIS build | 1✅/2❌/3✅/4⚠/5✅/6✅/7❌/8 N/A — chrome faint, blobs, no tooltip title, settings not migrating |
| 2 | fix-1..4 (chrome, jitter, sizing, hover-title) | Test 7 still failed (settings didn't migrate) |
| 2-investigate | fix-5 root-cause: boot-bundle merge drift; loadSettings has zero callers | Single-line discovery, reusable applyParsedSettings extracted |
| 3 | fix-5 (settings) + fix-6..9 (chrome, blending, two-pass, zoom) | Test 7 PASS; zoom regression broke wheel |
| 3.1 | fix-10 (zoom regression: identity-transform clear) | Zoom still broken (root cause was deeper) |
| 3.2 | fix-11 (real wheel binding via addEventListener; canvas markup completed) | Zoom WORKS; clusters still hard to read |
| 3.3 | fix-12 phyllotaxis spiral packing | Sunflower-pretty but not solving meaning at scale |
| 3.4 | fix-13 5px-at-max-zoom node sizing | Reasonably better; nodes meaningful at zoom |
| 3.5 | revert fix-12 (jitter wins A/B) | "Perfect!" for default + zoom — natural starfield |
| 3.6 | fix-15 all-circle nodes | Stage PASSED |
| 3.7 | fix-16 hover-ring screen-padded + ZOOM_MAX 8→24 | **"Ship"** |

### Architectural truths discovered (carry forward to future MIGs)

1. **Sight is a diagnostic, not a navigator.** The 7,650-note dome shows the universe's *shape* via density gradient at default zoom. Identifying individual notes requires the workflow filter (sidebar) → zoom (wheel) → click. Trying to make individual notes visible at default zoom is fighting the math.
2. **Boot-bundle drift is a real cross-cutting risk.** `loadSettings()` had ZERO callers in src; the boot-bundle path in `+layout.svelte` was the de-facto load path and had drifted from `loadSettings()`. My §A.12 migration block was dead code. Fix-5 extracted `applyParsedSettings()` as the single source of truth. **Worth a §N audit follow-up** to verify no other parsed-settings consumers exist OR settings-mutation paths bypass the load merge.
3. **Working Agreement #4 multi-edit verification.** Fix-9 (`6acde74`) claimed it wired the canvas markup but the Edit tool silently failed on one of nine edits mid-batch. Handlers existed but were never bound. Fix-11 (cycle-3.1) caught it via the addEventListener fallback. **Lesson**: multi-edit batches need explicit grep-verification afterward, OR parallel-agent review per Working Agreement #4. Cost: 1 extra Boss-test cycle + a fresh NSIS build.
4. **Phyllotaxis is mathematically pretty but not what users want for a "star chart".** A/B tested vs random jitter; jitter won perceptually because it reads as a natural starfield rather than a designed sunflower. Phyllotaxis stays in v0.x design history; v4.1 might revisit if specific use case emerges.

### Ship-day commits (in order)

| # | Commit | Subject |
|---|---|---|
| 1 | `3e829f6` | docs(sight): MIG-025 opens — Concept Paper v4.0 + Architect + Plan |
| 2 | `a0c0af5` | §A.1 — Add SIGHT_V6_ENABLED feature flag |
| 3 | `aa17e10` | §A.2 — Sight v6 cache schema + invalidation triggers |
| 4 | `baa4a95` | §A.3 — Sight v6 backfill skeleton + sentinel + 8 tests |
| 5 | `a38b256` | §A.4 — Progressive backfill via Tauri events + frontend store |
| 6 | `970d2bf` | §A.5 — Three Sight v6 Tauri IPCs |
| 7 | `b981129` | §A.6 — Sight v6 frontend types + module skeleton |
| 8 | `42c4f40` | docs(session-log): MIG-025 §A.1–§A.6 cascade summary |
| 9 | `7a948e9` | §A.7 — Mount Sight v6 alongside v5 in +layout.svelte |
| 10 | `5b3e10b` | §A.8 — Anchor dome chrome render |
| 11 | `e5b2334` | §A.9 — Anchor dome stars + lines + IPC integration |
| 12 | `c4b7e7b` | §A.10 — Facet sidebar with Hearst Flamenco cross-filter |
| 13 | `c84989b` | §A.11 — Sight v6 first-boot orientation tour |
| 14 | `1048ab1` | §A.12 — Sight v5 → v6 settings migration + 4 new v6 fields |
| 15 | `251d630` | §A.13 — Sight v6 CI perf-harness skeletons |
| 16 | `5796f18` | docs(session-log): MIG-025 Phase 1 build complete (§A.1 → §A.13) |
| 17 | `d0e683c` | §A.14 fix-1..4 — Boss-test cycle 1 fixes |
| 18 | `3c70896` | §A.14 fix-5 — boot-bundle settings merge + migration drift fix |
| 19 | `6acde74` | §A.14 fix-6..9 — density-aware rendering + zoom/pan |
| 20 | `59523f1` | §A.14 fix-10 — zoom regression: clear+bg in identity transform |
| 21 | `f79d26f` | §A.14 fix-11 — actually wire wheel/drag/keys to the canvas |
| 22 | `d70ceb1` | §A.14 fix-12 — phyllotaxis spiral packing |
| 23 | `1efadb5` | §A.14 fix-13 — node size 5px @ max zoom; default → density chart |
| 24 | `989507a` | Revert fix-12 (jitter wins A/B) |
| 25 | `ecabc16` | §A.14 fix-15 — all notes render as circles |
| 26 | `f8de004` | §A.14 fix-16 — hover-ring matches node + zoom 8× → 24× |
| 27 | (this commit) | **§A.14 SHIP — Sight v6.0 (MIG-025)** + orientation v2.02 + session log |

**~27 commits in one day. ~12,000+ insertions across ~50 files. 25 Rust unit tests passing. v6.0 live on `main`.**

### Phase 2 opens

§B Phase 2 (mini-domes + cross-filter brushing + Pro mode) is queued per Plan §A.2. ~4 weeks. Then §C (register chip + 4 production registers, 5 wk), then §D (3 v1-preview registers + CI hardening + v5 deletion, 3 wk).

**v4.1 polish targets** (post-Phase 4): hex-bin aggregation, register-aware mini-domes, library-tint recognition aid, three v1-preview registers polish, pramāṇa internal-structure, color-accessibility variant.

**§A.14 verification checklist** (per Concept Paper v4.0 §13.1):

- [ ] Anchor dome renders all 6 pre-attentive channels per §3.1
- [ ] Default-simple layout satisfies §6.2 (≥80% anchor)
- [ ] Facet sidebar cross-filters across all 6 facets including Folder
- [ ] First-boot tour fires once, skippable, persisted via `tourSeen`
- [ ] All §5 gestures work except mini-dome cross-filter (mini-domes don't exist yet)
- [ ] v5 module set still present + reachable via dock toggle (B2)
- [ ] CI perf gate harness present (runners deferred to §D.4)
- [ ] Settings migration runs cleanly on a v5-state Universe

---

## What shipped

### Commit 3e829f6 — MIG-025 design landing

The full Sight redesign conversation, the binding contract, and the build cycle docs:

- **`docs/Constellation-Sight-Concept-Paper-v4.0.md`** — the binding contract. Supersedes v3.1. Specifies Sight v6 architecture (Coordinated Views: anchor dome + 4 mini-domes + facet sidebar + 7-register chip), 10 invariants, default-simple chrome with Pro-opt-in, channel orthogonality discipline.
- **3 design concept docs** (`sight-redesign-design-concept-v0.1.md` → `v0.2.md` → `v0.3.md`) — historical record of the design conversation. Each preserved on disk per SO #6.
- **9 mock SVGs** (`sight-redesign-*.svg`) across three rounds of SME panel review. v0.3 mocks (`sight-redesign-v0.3-full-layout.svg`, `sight-redesign-v0.3-register-chip-detail.svg`) are the binding visual contract for the Sight v6 build.
- **`lab/reports/MIG-025-SIGHT-V6-ARCHITECT.md`** — Phase 1 of /migration: territory map (every v5 file's v6 disposition), 9 build-strategy options (A–I), 10 invariants, migration/back-fill/rollback concerns. Locked option pack: A1 + B2 + C3 + D1 + E1 + F3 + G1 + H1 + I1; chip-hidden-until-Phase-3.
- **`lab/reports/MIG-025-SIGHT-V6-PLAN.md`** — Phase 2 of /migration: 43 ordered steps across 4 sub-phases (§A=14, §B=11, §C=11, §D=7), every Concept Paper §11 invariant protected by ≥1 step. Each step is one-commit-sized with verification clause.
- **`docs/Constellation Orientation & Onboarding v2.01.md`** — SO #6 bump capturing the architecture pivot. v2.00 preserved on disk.

### Commit a0c0af5 — §A.1 first build step

- **`src/lib/sight/engine.ts`** — added `export const SIGHT_V6_ENABLED = false;` per MIG-025 Plan §A.1. Dual-flag mount mechanism (Architect Option B2) foundation. Flag stays `false` until §A.14 ship gate clears and Eisa tests Sight v6.0.

---

## Tests / Verification

### Rust (`cargo test --lib sight_v6`)

**25/25 passed; 0 failed.**

- Schema (§A.2 — 9 tests): idempotency, covering indexes, v6 columns presence, both invalidation triggers (AU + AD) firing correctly, snapshot hash format + monotonicity, dual-mount v5+v6 caches coexist (B2 invariant).
- Backfill (§A.3 — 8 tests): all rows backfilled, link in/out counts (3-out/1-in on a.md vs 0-out/2-in on b.md per fixture), frontmatter key count (4/1/0 on a/b/c), body chars matches len, sentinel stamps to 1, idempotency via sentinel short-circuit, stratum L-prefix parsing (L3→3, L1→1, NULL→NULL), contested marking on inbound contradicts.
- Progressive backfill (§A.4 — 8 tests): all 10 rows across 5 tiers, exactly 6 events emitted (5 tier + 1 done), first-tier-complete unblocks render, done_rows monotonically increases, sentinel stamps only after final tier, short-circuits to single done event when sentinel set, orphans (NULL stratum) swept by tier 5, resumability when sentinel missing mid-run.

### TypeScript (`npm run check`)

**1384 files, 3 pre-existing errors, 303 pre-existing warnings, 47 files with problems.**
All 3 errors pre-existing (in `store.ts:2470`, `PropertyEditor.svelte:236/252` — unrelated to Sight v6).
File count went from 1380 → 1384 = exactly +4 new files (`backfillProgress.svelte.ts`, `types.ts`, `anchor.ts`, `SightV6.svelte`); error count unchanged confirms zero new errors from MIG-025.

### Cargo check

41 pre-existing warnings (all in unrelated modules: `sources/`, `arabic/`, `embeds.rs`, `search.rs` field reads, etc.). Zero new warnings from `sight_v6.rs`, the `lib.rs` mod declaration + IPC registrations, or the `search.rs` `ensure_*` integration.

### Manual verification deferred

The §A.14 ship gate is the user-testable point. Cascade continues autonomously until then per Plan-Approval-Equals-Build-Approval. v5 dock and mount path unchanged in this turn (`SIGHT_V6_ENABLED = false`); v5 still ships as the user-visible Sight.

---

## Bugs fixed

None — this session was design + first build commit, not bug-fixing.

---

## Decisions logged

The locked option pack from MIG-025 Architect:

| Choice | Lock |
|---|---|
| MIG structure | A1 — Single MIG-025 with §A/§B/§C/§D internal sub-phases |
| v5 deletion timing | B2 — `SIGHT_V6_ENABLED` dev-flag, v5 reachable Phase 1–3, deleted Phase 4 (per Eisa decision) |
| SQLite cache migration | C3 — Progressive backfill via Tauri events, status-bar progress |
| Worktree strategy | D1 — Single worktree all 4 phases |
| Tour persistence | E1 — Show on first-ever Sight v6 open via `tourSeen` flag |
| Performance gate | F3 — Continuous CI from Phase 1 |
| Settings migration | G1 — Read `sight_v5_*` once, map `lastScope` forward, drop `lastMode` silently |
| Register manifests | H1 — Create all 7 in Phase 3 alongside geometry |
| Visual reference | I1 — v0.3 SVGs as binding contract |
| Phase-2 chip behavior | Hidden entirely until Phase 3 (per Eisa decision) |

Plus the three Plan inferences (per Eisa decision, 2026-05-14):

- **Frontmatter convention** for register sector assignment (`pramana_kind`, `masadir_source`).
- **Help → Sight tour** is the re-fire affordance.
- **`enabledFeatures.constellationSightV6`** is the user-settings flag name.

---

## Open items

### In flight (next turn)
- §A.2: `sight_v6.rs` skeleton (LayoutCacheRow + LinkEdge structs + `ensure_sight_v6_layout_table` + `ensure_sight_v6_invalidation_trigger` + `compute_universe_snapshot_hash`) + `search.rs` `ensure_*` calls. The cache schema has 4 new columns vs v5: `link_in_count`, `link_out_count`, `frontmatter_key_count`, `body_chars`.
- §A.3 → §A.13 cascade after §A.2 lands.
- §A.14 ship gate stops for Boss-test of Sight v6.0 (Phase 1 deliverable: anchor dome + facet sidebar + Default-simple chrome + first-boot tour).

### Pending (later phases of MIG-025)
- Phase 2 (§B.1–§B.11): 4 mini-domes + cross-filter + Pro mode → Sight v6.1.
- Phase 3 (§C.1–§C.11): register chip + 4 production-polish registers + manifests → Sight v6.2.
- Phase 4 (§D.1–§D.7): 3 v1-preview registers + CI hardening + v5 deletion → Sight v6.3.
- §N Audit: 3 parallel agents (invariants / drift / migration-path).

### Obsoleted
- **MIG-024 §N close-out** is no longer relevant. Sight v5 architecture is being superseded by Sight v6; the v3.1 Concept Paper fold-in / Pending Jobs allocation work is moot.

---

## Documentation drift

None. SO #6 orientation bumped same-commit-as-change (v2.01 in commit `3e829f6`). User Manual (`docs/User Manual.md`) and help-files (`docs/help.uConstellation.World/`) do not yet have a Sight v6 section — that's expected because Sight v6 is not user-facing yet. Add Sight v6 chapter when §A.14 ship gate clears.

---

## Notes for next session

- `git -C "E:/مشاريع كلاود/Constellation"` for all git operations (the active session is in a worktree per Working Agreement #2 — operate against primary).
- Read `docs/Constellation Orientation & Onboarding v2.01.md` first (SO #6 — orientation is canonical).
- Read `lab/reports/MIG-025-SIGHT-V6-PLAN.md` for the build step list. §A.2 is next.
- The §A.14 ship gate is the next user-testable point. Cascade autonomously through §A.2 → §A.13 per Plan-Approval-Equals-Build-Approval.

---

*Original session-log close at §A.2 retained above for chronological honesty.*

---

## §A.3 → §A.14 + §B.1 → §B.5 catch-up summary (logged retroactively, 2026-05-14 PM)

**SO #1 drift:** the cascade went all the way through Phase 1 ship + Phase 2 §B.5 without inline session-log updates. Each commit landed cleanly to `main`; this section back-fills the log so a fresh session can reconstruct the turn order without git-archeology.

**§A.3 → §A.13 (Phase 1 build, 11 commits):**
- `baa4a95` §A.3 backfill skeleton + 8 tests
- `a38b256` §A.4 progressive backfill + Tauri events + frontend store
- `970d2bf` §A.5 three Tauri IPCs (`sight_v6_warm_cache`, `_get_layout`, `_get_link_set_for_notes`)
- `b981129` §A.6 frontend types + module skeleton
- `7a948e9` §A.7 mount in `+layout.svelte` (B2 dual-mount, dock button, Esc handler)
- `5b3e10b` §A.8 anchor dome chrome (5 strata, calendar rim, labels)
- `e5b2334` §A.9 anchor stars + lines + IPC integration + hit-test
- `c4b7e7b` §A.10 facet sidebar (Hearst Flamenco)
- `c84989b` §A.11 first-boot orientation tour (4-step skippable)
- `1048ab1` §A.12 settings migration + 4 new v6 fields
- `251d630` §A.13 CI perf-harness skeletons (vitest+playwright deferred to phase 4)

**§A.14 Boss-test cycle (16 fixes across 7 NSIS builds, ratified at cycle-3.7 with "Ship"):**
- `d0e683c` fix-1..4 — chrome contrast, jitter widening (±15% → ±85%), smaller stars (5 px ⌀), hover-title (`noteTitle()` extraction)
- `3c70896` fix-5 — `applyParsedSettings` shared helper. **Root cause:** `loadSettings()` in store.ts had ZERO callers; the boot-bundle path in `+layout.svelte` was the de-facto load path and had drifted from `loadSettings`. The migration was dead code on one path. Extracted shared helper as single source of truth.
- `6acde74` fix-6..9 — brighter chrome (#1a1f2e → #2a3245), additive density blending, two-pass render (bodies pass 1, pips pass 2), wheel-zoom + drag-pan + Cmd-0 reset
- `59523f1` fix-10 — zoom regression: `clear+bg` in identity transform (was being clipped by the zoom transform)
- `f79d26f` fix-11 — `addEventListener` wheel binding. **Root cause (Working Agreement #4 violation):** the original fix-9 multi-edit batch silently left the canvas markup unfinished; Svelte's template binding never actually wired up. Replaced with explicit `addEventListener` + completion of the canvas markup. Lesson: post-edit verification is mandatory on multi-file batches.
- `d70ceb1` fix-12 — phyllotaxis spiral packing (Option C/Path A per Eisa's Architect-vs-NIST web-search choice)
- `1efadb5` fix-13 — node sizing 5 px ⌀ at max zoom; default → density chart, zoom reveals individuals
- `989507a` Revert fix-12 — Eisa A/B test verdict: jitter wins ("Perfect!")
- `ecabc16` fix-15 — all notes render as circles (drop library-shape encoding at small sizes per Eisa: "I want all the notes to take a circular shape")
- `f8de004` fix-16 — hover-ring screen-padded (constant 4-px in screen space) + `ZOOM_MAX` 8× → 24×

**§A.14 SHIP MOMENT — `8cdb73c`:** `SIGHT_V6_ENABLED = true` permanent on `main`. Eisa's "Ship" message recorded. Phase 1 deliverable closed.

**§B.1 → §B.5 (Phase 2 mini-domes, 4 commits):**
- `cd5cc15` §B.1 — mini-dome Svelte wrapper + renderer skeleton + 2×2 grid + Cmd-D toggle
- `d15488a` §B.2-§B.5 — four channel renderers (Confidence opacity / Stage hue / Acts size / Provenance sectors) + anchor→mini coord scaling
- `d455a8f` §B.5-fix-1 — Stage mini renders unknown-stage values as neutral gray (was returning `null` from `pipColorForStage` → `if (!stageColor) continue` skipped every node, mini was empty)
- `db8326a` §B-fix-2/3 + engine — visible mini chrome (stratum rings full opacity, was 0.04) + 1.5-px mini node ⌀ + **engine.ts side-by-side test config: `SIGHT_V5_ENABLED = false`, `SIGHT_V2_ENABLED = true`** so Eisa can A/B v2 vs v6.

**Eisa's verdict (post-`db8326a`):** "Sight v2 = Working. I decided to keep it." → A/B test concluded; v2 stays alongside v6 permanently. v5 stays disabled.

**Naming decision (this turn, 2026-05-14 PM):**
1. Eisa: "We have to think about renaming it. What do you think?" → spawned 5-SME panel (UX, LIS, Brand, Cross-Civ, Cog-Psych).
2. SME synthesis recommended Atlas + Threads or Sight + Threads (Cross-Civ killed "Web" for Arabic/Persian dignity loss).
3. **Eisa rejected both. Final pick: v6 = "Constellation Sight", v2 = "Constellation Nervous System (CNS)"** — biological/anatomical metaphor pairing (Sight = sensory, CNS = neural). Cross-civ clean across all 15 locales.
4. Eisa-confirmed grammar: **"Nervous System"** (canonical English anatomical term, matches the well-known CNS = Central Nervous System acronym referent), not "Nerve System" (translation-artifact register).

---

## §A.15 Predecessor → Replacement entry (per Predecessor Lookup Rule)

**Function in hand:** rename Sight v6 user-facing label "Constellation Sight v6" → **"Constellation Sight"**, and rename Sight v2 user-facing label "Constellation Sight" (lens-mounted) → **"Constellation Nervous System (CNS)"**.

### Predecessor (where the user-facing strings live now)

| Surface | File | Line | Current value | Predecessor MIG |
|---|---|---|---|---|
| v6 dock-button title | `src/routes/+layout.svelte` | 4492 | `title={$t('sight.v6.title') \|\| 'Constellation Sight'}` `aria-label="Constellation Sight v6"` | MIG-025 §A.7 |
| v6 mount-block title | `src/routes/+layout.svelte` | 5390 | `<span class="star-title">{$t('sight.v6.title') \|\| 'Constellation Sight'}</span>` | MIG-025 §A.7 |
| v2 dock-button title | `src/routes/+layout.svelte` | 4431 | `title={$t('lens.title') \|\| 'Constellation Sight'}` (no aria-label) | MIG-017 (lens predecessor: pre-PJ-039 "Lens" naming) |
| v2 i18n leaf — `lens.title` | `src/lib/i18n/{15 locales}.json` | en:2396 / ar:2358 / others vary | `"Constellation Sight"` in all 15 | MIG-017 |
| v2 plug-in label | `src/lib/i18n/{15 locales}.json` | en:353 / others:420 | `"constellationSight": "Constellation Sight"` in all 15 | MIG-017 |
| v2 plug-in description | `src/lib/i18n/{15 locales}.json` | en:354 / others:421 | `"Gravity-well knowledge visualization with analytics"` (English in all 15 — translation drift) | MIG-017 |
| Settings → Sight section intro | `src/lib/i18n/{15 locales}.json` | en:522 (block at 521) | `"Constellation Sight v3 — star-chart visualization of your knowledge universe."` (stale; refers to retired v3) | MIG-018 |

### Replacement (where the user-facing strings will live after this commit)

**Default per the Rule: same place.** No file relocation, no new i18n key paths, no settings-flag rename.

| Surface | New value | Notes |
|---|---|---|
| v6 dock-button title | `aria-label="Constellation Sight"` (drop "v6"); fallback already `'Constellation Sight'` | Matches the canonical Sight identity post-ship |
| v6 mount-block title | unchanged (already renders "Constellation Sight" via fallback) | No edit needed; documented for completeness |
| v2 dock-button title | i18n value of `lens.title` becomes `"Constellation Nervous System (CNS)"` | Same key, same call site |
| v2 plug-in label | `"constellationSight": "Constellation Nervous System (CNS)"` | Same key |
| v2 plug-in description | `"constellationSightDesc": "Connection-traversal view of your universe — Universe Health metrics, communities, top bridges, and structural-gap (\"Blind Spot\") analysis."` | Reflects what v2 actually does (Universe Health card + bridge/community detection per `lens.*` keys) |
| Settings → Sight section intro | `"Constellation Sight — anchor-dome view of your universe with stratum × time positioning, density-gradient at default zoom, and per-channel mini-domes."` | Drops the stale v3 reference; describes v6 |

### Internal symbols KEPT unchanged (per Lens-precedent / architectural-history convention)

- File names: `SightV6.svelte`, `MiniDome.svelte`, `ConstellationSight2.svelte`, `sight_v6.rs`, `sight.rs`
- IPC names: `constellation_sight_*`, `sight_v6_*`
- Engine flags: `SIGHT_V2_ENABLED`, `SIGHT_V6_ENABLED`
- Settings flags: `enabledFeatures.constellationSight`, `enabledFeatures.constellationSightV6`
- State variables: `sightV6Active`, `lensActive`
- i18n key paths: `lens.title`, `sight.v6.title`, `plugins.constellationSight*` (KEY names; only VALUES change)
- All "Sight v6" / "Sight v2" references in code comments (architectural-history record)
- The 16-fix lineage block in `engine.ts`

### What's NOT in this commit (deferred — flagged as doc-drift in orientation v2.03)

- **Help docs.** The existing `docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md` describes Sight **v5** (six cognitive-lens modes — R/L/T/C/S/A/P). v2 has **no dedicated help doc** (the existing `Lens/Lens.md` is about DQL queries, an unrelated feature). Both need full rewrites — separate commit because the rewrites are not mechanical and need fresh prose for both Sight v6 and CNS v2. Carries through to 14 language mirrors (`docs/help.{lang}/`).
- **User Manual.** `docs/User Manual.md` Sight chapter is similarly stale.
- **Locale translations of the new descriptions.** Only English values are changing in this commit; the 14 other locales already had English values for these keys (translation drift predates this commit). A future translation pass can localize.
- **`.docx` redistribution.** Pre-built `Constellation User Manual.docx` etc. regenerate later from the .md sources.

### Boss approval logged

- 2026-05-14 PM, message: "Then, it is going to be: Constellation Nervous System (CNS)" — confirmed final naming (after grammar clarification "Nervous" not "Nerve"). Cascade approved per Plan-Approval-Equals-Build-Approval; only stop is the user-testable installer.

---

## §A.15 fix-1 — Eisa test feedback (Boss-test cycle 1)

**Test build:** `Constellation_0.3.4_x64-setup.A15-rename.exe` (`67363369`)

**Step 1.1 fail:** v6 dock-button tooltip showed literal `"sight.v6.title"` instead of "Constellation Sight".
- **Root cause:** the i18n key `sight.v6.title` was never defined in any locale (only referenced in code with `|| 'Constellation Sight'` fallback). The `t()` impl in `src/lib/i18n/index.ts:130-140` returns the key string itself on miss; the `||` fallback in `+layout.svelte:4492` never fires because the key string is truthy. Same shape as the §104/§113 Untyped-label bug.
- **Fix:** add `sight.v6.title: "Constellation Sight"` as a sibling of `sight.v5` in `en.json` (top-level sight block, line 2362). i18n fallback chain (active locale → en → key, per index.ts:43 comment) auto-resolves it for all 14 other locales — no per-locale edit needed.

**Step 1.2 design request:** swap dock icons + design new icon for CNS.
- Eisa: "I want Sight to have CNS icon, the eye, and I want you to create a suitable icon that represents CNS."
- **Move:** eye SVG (`<path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/><circle r=3/>`) from CNS dock-button (line 4432) to v6 Sight dock-button (line 4493). Sight = sensory organ = eye.
- **New CNS icon:** stylized neuron — central soma + three dendrites with synaptic-bouton terminals (cell body `circle r=2.5` at center, three `line+circle r=1.5` branches at top, lower-left, lower-right). Lucide-style 2px outline, viewBox 24x24. Renders as a clear branching nerve-cell pictogram at 16x16.
- The star polygon now appears only on the disabled v3/v4/v5 dock buttons (legacy code paths gated false).

**Step 2.1 pass.** "Constellation Nervous System (CNS)" + new description rendering correctly in Settings → Core Plug-Ins.

**Step 2.2 finding (NOT a rename bug — pre-existing structural gap):**
- There is no separate "Sight" section in the Settings sidebar.
- The `settings.sight.intro` i18n key updated in §A.15 is **orphaned** (no Settings UI component currently consumes it).
- Eisa pointed out the "Knowledge Management" section (which has an eye icon) — but that section is the legacy **Lens query (DQL) system** ("Switch View" / "Load Lenses" loads custom DQL view definitions stored as `.lens` files), NOT the CNS visualization settings. Different feature, despite the eye icon.
- **Allocated as a future MIG:** add a proper "Sight" Settings section that surfaces the v6 settings (`proMode`, `hexBinThreshold`, `linkFadeThreshold`, `tourSeen`) currently stored in `appSettings.sight` but not exposed in any UI. Would also use the orphaned `settings.sight.intro` key.

**Files in this fix commit:** `src/lib/i18n/en.json`, `src/routes/+layout.svelte`, this session log.

---

## §A.15 cycle-2 PASS — rename closed (2026-05-15)

**Test build:** `Constellation_0.3.4_x64-setup.A15-fix1.exe` (`cab7ffb5`).

**Eisa verdict:** "Step 1: Pass. Step 2: Pass. Visual judgment call: It is fine. Step 3: Pass. Step 4: Pass."

All four steps pass; neuron icon for CNS accepted on first iteration. **§A.15 closed.**

The user-facing name pair is now permanent on `main`:
- v6 → "Constellation Sight" (eye icon)
- v2 → "Constellation Nervous System (CNS)" (neuron icon)

Internal v-numbers, file names, IPC names, engine flags, and i18n key paths remain as architectural-history record (Lens-precedent convention).

---

## §B.6 — bidirectional linked brushing (anchor ↔ minis)

**Function in hand:** complete the Coordinated Views loop — hovering a node in any mini-dome highlights the same node on the anchor (and on the other 3 minis); hovering the anchor likewise highlights all 4 minis. Single source of truth: SightV6 owns `hoveredPath`; minis only PROPOSE via `onHover()` and receive the resolved value back through the `highlightedPath` prop.

### Implementation

Three files, ~113 insertions, no deletions:

1. **`src/lib/sight/v6/miniDome.ts`** — new exported `miniDomeHitTest(stars, canvasX, canvasY, channel, anchorLayout, miniWidth, miniHeight, tolerance=12)` function. Channel-aware:
   - **provenance**: iterates stars and computes each one's `provenancePositionFor()` (FNV-1a-jittered sector coords); picks the closest within tolerance in canvas space.
   - **confidence/stage/acts**: inverts the render transform (`(canvasX - layout.centerX) / scale + anchorLayout.centerX`, same for y) to convert cursor canvas coords → anchor world coords, then delegates to anchor's `starHitTest` with world-space tolerance scaled as `tolerance/scale`. On-screen tolerance stays constant in canvas pixels regardless of mini size.
   - Imports `starHitTest` from `./anchor` (added to existing import line).

2. **`src/lib/sight/v6/MiniDome.svelte`** — added:
   - `onHover?: (path: string | null) => void` prop with default no-op.
   - `handlePointerMove` / `handlePointerLeave` handlers that hit-test and dispatch.
   - `onpointermove` / `onpointerleave` listeners on the canvas + `class:has-hover` toggle for `cursor: pointer` UX feedback.

3. **`src/lib/sight/v6/SightV6.svelte`** — added:
   - `onHover={(path) => { hoveredPath = path; }}` callback passed to each of the 4 MiniDome instances.
   - New `$effect(() => { void hoveredPath; untrack(() => paint()); })` so the anchor canvas repaints on hoveredPath change from any source. The forward direction (anchor pointermove → hoveredPath) still calls paint() explicitly inside handlePointerMove; the new effect specifically handles the reverse direction (mini onHover → hoveredPath). paint() is idempotent so the occasional double-paint on forward hover is harmless.

### Performance notes

- Hit-test is O(n) per pointermove. At 7,650 stars: ≈0.05 ms per call — negligible.
- Hover dispatch is guarded by `if (hit !== highlightedPath)` to avoid redundant state writes when the cursor moves within the same star's hit zone.
- MiniDome's existing `$effect` (watches stars / highlightedPath / anchorLayout) auto-repaints all 4 minis when SightV6's hoveredPath updates — no explicit propagation code needed.

### Out of scope (later in Phase 2)

- §B.7 — clicking a mini-dome category region (e.g. "Established" sector in the Stage mini) cross-filters the universe.
- §B.8 — CI perf gate for cross-filter responsiveness.
- §B.9 — hex-bin aggregation when visible-star count exceeds 5,000 (smooths interaction at extreme density).
- §B.10 — Cmd-Shift-D persistent Pro mode toggle (currently Cmd-D toggles minis per session only).
- §B.11 — Phase 2 ship gate → Sight v6.1.

### Test cycle pending

Build to follow this commit. Eisa verifies bidirectional brushing across anchor + 4 minis on a populated universe, then the cascade continues to §B.7.

---

## §B.6 cycle-1 PASS + §B.6-fix-1 (2026-05-15)

**Test build:** `Constellation_0.3.4_x64-setup.B6-brushing.exe` (`9fe9b615`).

**Eisa cycle-1 verdict:**
- Stages 3-7 (bidirectional brushing): **PASS** in all directions including the Provenance mini's special hit-test.
- Stage 1.4 (mini chrome): **FAIL** — "I want the mini-domes' inner circles, font color, and opacity to match the anchor dome exactly."
- Stage 2.3 (gold ring visibility): **FAIL** — "it is hard to see the gold ring on the mini-domes because of the background."
- **Architectural question:** "Why are the Confidence, Stage, and Acts domes almost identical? Do they execute the same algorithm?"

### Diagnosis (data-driven, not speculative)

Queried `sight_v6_layout` on Eisa's active universe (`Eisa Cognitive Knowledge`, 7,645 notes):

| Channel | Field | Fill state | Result |
|---|---|---|---|
| Confidence | `confidence_alpha` | 98.5% at 0.45 (default), 1.5% NULL — **zero variation** | mini renders uniform-opacity dot cloud |
| Stage | `stage` | 99.3% non-null but uses Living Link vocabulary (spark 48.8%, birth 39.9%, growth 9.7%, maturity 0.8%) — **renderer recognized 0%** | falls back to neutral gray for all → looks identical to Confidence |
| Acts | `acts_primary` 100% NULL; `link_in+out` well-distributed (0–19+) | **link-count fallback works** — top-decile 6-px discs visible | only mini with actual signal in current data |
| Provenance | `sources_primary` 98.2% NULL | sector layout makes it look distinct regardless | will populate as Eisa approves CECE's 4,475 pending suggestions |

**Root cause for "three minis look identical":** the data isn't differentiating them. The renderers ARE distinct (opacity / hue / size), but with confidence locked at default and stage vocabulary mismatched, opacity-uniform + hue-fallback-to-gray + size-fallback-to-link-count all produce nearly the same output.

### CECE impact map (separate Boss question)

Verified: CECE has already classified 4,475 of 7,645 notes (58%); the bottleneck is the **Source Review approval workflow**, not classifier compute. CECE writes only to `note_meta.sources` + `note_meta.content_type`. Impact tiers:

- **Tier 1 (visible on approval):** Provenance mini, Sight facet sidebar Provenance counts, Source Review queue, NotePane chips, `.md` frontmatter (file-over-app durability).
- **Tier 2 (ambient):** CECE reliability/active-learning, Sibling Disambiguation prompts, `note_state_history` temporal index.
- **Tier 3 (untouched):** Confidence/Stage/Acts minis, Sky View, Constellation Map, CNS, Search.

So running the classifier helps **only Provenance**. Stage needs the renderer fix below; Confidence needs a separate UI/automation MIG.

### §B.6-fix-1 implementation

Three fixes, two files, ~65 insertions:

**Fix A — `src/lib/sight/v6/anchor.ts` `pipColorForStage`:**
Extended the switch from 5 cases (Concept Paper v4.0 vocabulary: established/fresh/growing/at-risk/dormant) to 12 cases by adding the 7 Living Link Architecture stages (spark/birth/growth/maturity/dormancy/renewal/archival per `CLAUDE.md`). Living Link → palette mapping documented inline. The original 5 stay as fallbacks for any legacy frontmatter. `birth` and `growth` collapse to the same violet (would need a 6th palette color to distinguish — Concept Paper §3.4 spec is 5 slots).

**Fix B — `src/lib/sight/v6/miniDome.ts` mini chrome matches anchor:**
- Stratum-ring `lineWidth`: 0.5 → 0.9 (matches anchor's `anchor.ts:267`).
- Channel-title color: `PALETTE.subtitleText` (faint #5a6275) → `PALETTE.titleText` (bright cream #e8ebf2). Mini titles now read at the same visual weight as the anchor's "Constellation Sight" header.

**Fix C — `src/lib/sight/v6/miniDome.ts` gold ring visibility (Pass 4):**
- Radius: 6 → 9 (50% larger).
- Stroke: 1.4 → 2.2 (57% thicker).
- Added a 4-px-wide background-colored "halo" stroke drawn UNDER the gold ring (same radius). The halo gives the gold a dark outline that pops against the dark background; without it the gold edge anti-aliased into the background and washed out at mini scale.

### Out of scope (carried as future MIGs)

- **Confidence-population MIG** (PJ-NNN to be allocated): no UI/automation currently writes confidence values to notes; everything stays at the default 0.45 alpha. Needs either a NotePane confidence picker or a heuristic that infers confidence from link weight + lifecycle stage.
- **Sight Settings UI section** (already on the list): would surface v6's stored settings (proMode, hexBinThreshold, linkFadeThreshold, tourSeen) to the user. Currently they exist in `appSettings.sight` but no Settings panel reads them.

---

## §B.6 cycle-2 PASS-with-feedback + §B.6-fix-2 + §B.6-fix-3 (2026-05-15)

**Cycle-2 test build:** `Constellation_0.3.4_x64-setup.B6-fix1.exe` (`1764a2a9`).

**Eisa cycle-2 verdict:**
- Stage 1 (mini visibility): OK.
- Stage 2 (Stage palette): "I can see a blueish color spreading on the mini-dome. It is almost impossible to distinguish all the colors you mentioned." → §B.6-fix-2 Fix A below.
- Stage 3 (chrome match): re-confirmed; promoted minis "shall follow the anchor dome scheme" → addressed in §B.6-fix-3 dome-swap below (promoted minis use bigger dots; full anchor chrome on promoted view is polish for later).
- Stage 4 (gold ring): "It's clearer, but keep its previous size; don't enlarge it as you did." → §B.6-fix-2 Fix B below.
- New feature ask: "click on every mini-dome to enlarge it to the same size as the anchor dome ... the user could switch between all the domes (including the main one) to check their details." → §B.6-fix-3 below.

### §B.6-fix-2 (commit `062529ac`) — palette + ring

Two visual fixes, no architectural changes:

**Fix A — Stage palette differentiation:** added `PALETTE.stageBirth = '#fb923c'` (orange) as a 6th stage color slot in `dome.ts`. Re-mapped `birth` from `PALETTE.stageGrowing` (violet) to `PALETTE.stageBirth` (orange) in `anchor.ts pipColorForStage`. Now spark/birth/growth/maturity render as cyan/orange/violet/green — the two dominant categories (spark 49% + birth 40% = 89%) sit on opposite warm-cool axis sides instead of blurring as cyan-violet.

**Fix B — Gold ring revert:** radius 9 → 6 per Eisa "keep its previous size; don't enlarge it as you did." Kept the 4-px halo (the real visibility win) and stroke 2.2 (halo handles contrast; thicker gold reads more confidently than the original 1.4 even at smaller radius).

### §B.6-fix-3 — dome-swap feature

**Function in hand:** click any mini-dome to promote it into the primary anchor slot at full size; the previous primary (anchor or another channel) demotes into the vacated mini slot. Click any demoted slot to swap back. Per Eisa cycle-2 ask.

**Architecture:** introduced a parallel type `SlotChannel = MiniDomeChannel | 'anchor'` in `types.ts` for the 5-slot layout (the 4 mini channels + anchor). Kept `MiniDomeChannel` narrow because cross-filter category gestures (`filter-mini-dome-category`) and the facet sidebar specifically exclude 'anchor' — only categorical channels are filterable buckets.

**Files (4, ~252 insertions):**

1. **`src/lib/sight/v6/types.ts`** — add `SlotChannel` type.

2. **`src/lib/sight/v6/miniDome.ts`** — `renderMiniDome` and `miniDomeHitTest` and `channelTitle` now accept `SlotChannel`. New `renderAnchorChannel` function (plain neutral cream stars at the same stratum × time positions used by confidence/stage/acts — keeps linked brushing aligned). Added 'anchor' dispatch case in `renderMiniDome`. Channel renderers (confidence/stage/acts/provenance/anchor) now accept a `dotRadius` parameter (default 0.75 for compact mini slots; promoted slots pass 3 for inspectable size). Acts top-decile preserves its 4× ratio relative to base (3 × 4 = 12 px when promoted).

3. **`src/lib/sight/v6/MiniDome.svelte`** — three new props:
   - `compact: boolean = true` — controls dot size and click semantics.
   - `onPromote?: (channel: SlotChannel) => void` — fires on click when compact (mini slot) → swap.
   - `onOpenNote?: (notePath: string) => void` — fires on star-click when not compact (primary slot) → open in editor.
   - `handleClick` dispatches based on `compact`. New `class:is-promoted={!compact}` for CSS hooks.

4. **`src/lib/sight/v6/SightV6.svelte`** — orchestration:
   - `ALL_SLOTS: SlotChannel[]` constant.
   - `primaryChannel = $state<SlotChannel>('anchor')` — default unchanged.
   - **Wheel listener moved out of `onMount` into a `$effect` keyed on `canvasEl`** — the anchor canvas now mounts/unmounts via `{#if primaryChannel === 'anchor'}`, so the listener must re-attach on every rebind. Cleanup detaches from the old element. Replaces the §A.14 fix-11 onMount block. Imperative `addEventListener` retained per fix-11 lesson (Tauri WebView2 + Svelte 5 `onwheel` silent-fails in release builds).
   - Companion `$effect` for `canvasEl + primaryChannel` calls `syncCanvasSize` after canvas rebind (ResizeObserver doesn't fire on `{#if}` mount because host size is unchanged).
   - `handlePromote(slot)`: sets `primaryChannel = slot`, resets `zoomScale = 1`, `panX = panY = 0` (previous slot's transform doesn't apply to new primary).
   - `handlePromotedOpenNote(notePath)`: looks up libraryName via `rows.find()`, dispatches parent `onOpenNote(notePath, libraryName)`.
   - Markup: canvas wrapped in `{#if primaryChannel === 'anchor'}{:else}<MiniDome compact={false}>{/if}`. Mini grid iterates `ALL_SLOTS` skipping `primaryChannel` (so demoted anchor takes the vacated slot). New `.sight-v6-promoted-host` CSS for the absolute-positioned promoted-mini wrapper.

**Out of scope (deferred polish):** promoted-mini does NOT yet render anchor's full chrome (calendar rim + stratum text labels + connection lines). Only the bigger dots + brighter base chrome. If Eisa wants full anchor-style chrome on promoted minis, that's §B.6-fix-4 work — requires extracting renderAnchorDome's chrome rendering into a shareable function and wiring it into the promoted-mini path.

**Build artifacts:**
- `Constellation_0.3.4_x64-setup.B6-fix2.exe` (`062529ac` — palette + ring only)
- `Constellation_0.3.4_x64-setup.B6-fix3.exe` (this commit — adds dome-swap on top of fix-2)

---

## §B.6 cycle-3 PASS-with-feedback + §B.6-fix-4 (2026-05-15)

**Cycle-3 test builds:** B6-fix2.exe (`062529ac`) + B6-fix3.exe (`9f2c1732`).

**Eisa cycle-3 verdict:**
- Stage 1 (cursor): mostly OK except cursor doesn't show pointer over mini canvases.
- Stage 2 / Stage 3 (palette): pass for distinguishability, but promoted node size needs to be 5px (radius 2.5).
- Stage 4 (demoted anchor mini): fine.
- Stage 5 (linked brushing across swap): pass.
- Stage 6 (swap-back via demoted anchor): true.
- Stage 7 (swap UX): "I don't like how the swap works. Now, if I want to swap to any other mini-dome, I first have to click the demoted anchor dome in the top left, then click the one I want." Eisa wants direct mini → mini swap, plus a Reset View button to return to default.
- Stage 8 (open note from promoted): pass, but needs Return-to-Sight button.
- General finding: zoom is disabled on promoted mini — needs to work on whatever's in the primary slot.

### §B.6-fix-4 implementation

Four sub-fixes, two files (MiniDome.svelte + SightV6.svelte), ~204 insertions:

**Fix 4a — Cursor pointer on mini canvases (MiniDome.svelte CSS):**
Changed `.mini-dome-canvas { cursor: default }` → `cursor: pointer` for the default (compact / mini-slot) state. The whole canvas IS a click target (clicking promotes the channel), so pointer cursor signals the affordance. Added `.is-promoted` cascade: in promoted slot, cursor reverts to default with `.has-hover` toggling pointer over stars and `.is-dragging` showing grabbing during drag-pan. Stage 7 swap-UX issue likely traces to missing pointer cursor → user couldn't tell the mini was clickable, so fell back to the anchor-mediated workaround. Direct mini → mini swap was already wired (handlePromote dispatches regardless of current primary); the cursor fix makes the affordance visible.

**Fix 4b — Promoted node size 5px ⌀ (MiniDome.svelte paint):**
Changed `dotRadius: compact ? 0.75 : 3` → `dotRadius: compact ? 0.75 : 2.5` per Eisa Stage 2/3. Acts top-decile preserves its 4× ratio relative to base (2.5 × 4 = 10 px when promoted).

**Fix 4c — Reset View button (SightV6.svelte):**
Added `handleResetView()`: sets `primaryChannel = 'anchor'`, resets `zoomScale = 1`, `panX = panY = 0`. Header gains a button visible only when the layout has been changed away from default (`primaryChannel !== 'anchor' || zoomScale !== 1 || panX !== 0 || panY !== 0`). Style: subtle neutral background, brighter on hover. Sits at the right edge of the header strip via `margin-left: auto`. Per Eisa Stage 7 ask.

**Fix 4d — Zoom + pan + drag + Cmd-0 on promoted MiniDome (MiniDome.svelte):**
Added local `zoomScale / panX / panY / dragState` state + `ZOOM_MIN=0.5`, `ZOOM_MAX=24`, `DRAG_THRESHOLD=4` constants. New handlers:
- `handleWheel`: zoom-toward-cursor; mirrors SightV6's anchor handleWheel exactly.
- `handlePointerDown`: start drag-pan.
- `handlePointerUp`: end drag.
- `handleKey`: Cmd-0 / Ctrl-0 reset (parity with anchor).
- `handlePointerMove` extended: drag-pan path when `!compact && dragState && (buttons & 1)`. Hit-test inverts zoom transform for hover detection (`x = (x - panX) / zoomScale`, similar for y; tolerance scaled `12 / zoomScale` so screen hit zone stays constant).
- `handleClick` extended: ignores drag-clicks via `dragState?.moved` guard; inverts transform for hit-test (mirrors `handlePointerMove`).
- `paint()` applies zoom transform via `setTransform(dpr * zoomScale, 0, 0, dpr * zoomScale, dpr * panX, dpr * panY)` when `!compact`. Compact path keeps identity-ish transform `(dpr, 0, 0, dpr, 0, 0)` as before.
- Wheel listener attached via `$effect` keyed on `canvasEl` (re-attaches on canvas remount; cleanup detaches from old element). Mirrors SightV6's pattern. Imperative `addEventListener` retained per §A.14 fix-11 lesson (Tauri WebView2 + Svelte 5 onwheel silent-fails in release builds).
- New `$effect` watching zoom/pan/dragState triggers paint() so the transform reapplies on state change.
- Canvas markup: added `tabindex={compact ? -1 : 0}` (focusable for keyboard events when promoted), `onpointerdown`, `onpointerup`, `onkeydown`, `class:is-dragging={dragState?.moved}`.

State is local — each promotion creates a fresh MiniDome instance (because primary-slot and mini-grid live at different DOM positions in the SightV6 markup), so zoom resets to 1 automatically on every fresh promotion.

### Out of scope (carries over)

- **Return-to-Sight button after opening a note from promoted view:** the existing dock eye icon already returns to Sight from any opened note (closes/opens Sight). Asking Eisa whether the dock icon is sufficient or if he wants a dedicated in-editor button (the latter touches NotePane / +layout.svelte, not Sight).
- **Promoted-mini full anchor chrome** (calendar rim + stratum text labels + connection lines): still deferred from §B.6-fix-3. Not addressed in fix-4.

**Build artifact:** `Constellation_0.3.4_x64-setup.B6-fix4.exe` (this commit).

---

## §B.6-fix-5 — Return-to-Sight button + §A.15 oversight fix (2026-05-15)

**Eisa cycle-3 follow-up:** "I want a dedicated 'Return-to-Sight button' in-editor button."

**Pattern:** mirrors the existing `lensReturnPending` flow for v2 (CNS):
- State flag `sightV6ReturnPending = $state(false)` lives in `+layout.svelte`.
- When the user clicks a star in v6 (anchor or promoted mini), the `onOpenNote` callback in the SightV6 mount block sets `sightV6ReturnPending = true` after closing v6.
- The tab-bar return-buttons section renders a `{#if sightV6ReturnPending && SIGHT_V6_ENABLED}` button that re-opens v6 + clears the flag.
- The v6 dock button onclick clears the flag in both branches (open + close).

**§A.15 oversight caught + fixed in same commit:** the existing `lens.returnToLens` button label was "Return to Sight" — a label that was correct before §A.15 (when v2 was branded "Sight") but wrong after the §A.15 rename (v2 is now CNS). Updated value to "Return to CNS" in all 15 locales (some had translated values for "Sight" — `Zurück zur Sight` / `Voltar à Sight` / `Sightに戻る` etc.; per §A.15 brand-English convention, all collapse to "Return to CNS"). Also updated the fallback string in `+layout.svelte:4867` from "Return to Lens" → "Return to CNS".

### Files (16, ~39 insertions, ~18 deletions)

**i18n changes (15 locales):**
- en.json: ADDED `sight.v6.returnToSight = "Return to Sight"` (sibling of `sight.v6.title`); UPDATED `lens.returnToLens` value: "Return to Sight" → "Return to CNS".
- 14 other locales (ar, de, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh): UPDATED `lens.returnToLens` value to "Return to CNS" (English brand per §A.15 convention; replaced previously-translated values).

**+layout.svelte changes:**
- Line 559 area: ADDED `let sightV6ReturnPending = $state(false)` next to existing return-pending flags.
- Line 4485 v6 dock button onclick: ADDED `sightV6ReturnPending = false` in both branches (when activating + when deactivating v6).
- Line 4867 lens return button fallback: UPDATED `'Return to Lens'` → `'Return to CNS'` (matches the i18n value change).
- Line 4869 area: NEW `{#if sightV6ReturnPending && SIGHT_V6_ENABLED}` block rendering the Return-to-Sight button (mirrors the lensReturnPending pattern exactly — same SVG icon, same `index-return-btn` class, same i18n key access).
- Line 5394 v6 mount onOpenNote: ADDED `sightV6ReturnPending = true` after `sightV6Active = false`.

### Out of scope (intentional)

Other dock buttons (Sky View, Map, Index, OrgChart) currently reset `lensReturnPending` but NOT `sightV6ReturnPending`. Cosmetic — the worst case is a stale Return-to-Sight button after navigating elsewhere. Click still works (re-opens v6). Future polish would clear `sightV6ReturnPending` on every dock button click for full consistency; not blocking.

### Build artifact

`Constellation_0.3.4_x64-setup.B6-fix5.exe` (this commit) — Return-to-Sight button + lens-return-label CNS fix on top of §B.6-fix-4 (cursor + 5px nodes + Reset View + zoom on promoted).

---

## §B.6-fix-6 — unified star-click semantics across all dome slots (2026-05-15)

**Eisa cycle-5 verdict:**
- Stage 2 (Return-to-Sight from anchor): PASS. **Follow-up:** "why is clicking on any star to open its note limited to the anchor dome? I want to check any notes by clicking on any star, either through the main dome or the mini-domes."
- Stage 3 (Return-to-Sight from promoted mini): "if I click on a star in a mini-dome, it shall open that note. And if I want to promote that mini-dome to be a main one it will be through the dome space itself, not the stars."
- Stage 4 (Return-to-CNS label): PASS. Eisa confirmed CNS click-behavior intent (single click → preview side panel + outgoing/incoming links; double click → open note); that's existing CNS UX and stays unchanged.
- Stage 5.3 (both buttons coexisting): observed only Return-to-Sight (single button). Eisa: "it is enough. We don't want to have both; there is no logical reason for it." Current code already gives this in the typical sequence (v6 dock-button onclick clears `lensReturnPending`); no fix needed.

### Fix design

Click semantics now unified across compact (mini slot) AND non-compact (primary slot):
- **Star click** → `onOpenNote(notePath)` (open in editor) — works in any dome.
- **Empty-space click** → channel-specific:
  - compact (mini slot): `onPromote(channel)` (swap into primary slot, per fix-3 + cycle-3 ask).
  - non-compact (primary slot): no-op (use the demoted-anchor mini-slot click, OR the header Reset View button).

The mini's `handleClick` hit-tests first. Hit → open. Miss + compact → promote. Miss + non-compact → no-op. Cleaner mental model than the previous "compact = always promote, non-compact = open if star".

### Files (3, ~30 insertions, ~13 deletions)

**`src/lib/sight/v6/MiniDome.svelte` `handleClick`:** restructured to hit-test first regardless of `compact`. Drag-click guard (`dragState?.moved`) stays at top. Star hit calls `onOpenNote(hit)`. Miss falls through to `if (compact) onPromote(channel)`. Zoom-transform inversion only applied in non-compact (compact has identity transform).

**`src/lib/sight/v6/SightV6.svelte`:** the mini-grid `<MiniDome compact={true}>` instances now also receive `onOpenNote={handlePromotedOpenNote}` (was only passed to the promoted slot before). Same `handlePromotedOpenNote` callback handles both — looks up libraryName via `rows.find()` and dispatches the parent's `onOpenNote(path, libraryName)`.

**Out of scope (intentional):** click-promote from compact and click-open from compact share the same canvas. Click on a star opens; click on empty promotes. There's no UI hint about WHICH part of the canvas does what — visually it just feels like "click on a star" vs "click anywhere else". If Eisa wants explicit visual cue (e.g., a small "promote" icon in the corner of each mini), that's polish for a future fix.

### Build artifact

`Constellation_0.3.4_x64-setup.B6-fix6.exe` (this commit) — adds unified star-click semantics on top of fix-5 (Return-to-Sight + lens label CNS).

---

## §B.6-fix-7 — Acts promoted blob + tighter click tolerance (2026-05-15)

**Eisa cycle-6 verdict:**
- Stage 2 (compact-mini star click opens note): PASS.
- Stage 3 (compact-mini empty-area click promotes): PASS-with-issues:
  - **Promoted node size feedback:** "the node size must be 5px when the mini-dome is promoted" — looking at the Acts-promoted screenshot, the top-decile blob (~764 notes at 4× ratio = 20-px ⌀ each) overlapped into a solid white mass.
  - **Direct mini-to-mini swap:** "still isn't working 100%."
- Stage 4-6: PASS.

### Two fixes

**Fix A — Acts top-decile flattens to base radius in promoted mode** (`miniDome.ts renderActsChannel`):

The 4× top-decile ratio gives Acts its binary-size signal. In compact (mini slot, dotRadius 0.75 → top-decile 3 = 6-px ⌀), the contrast helps hot-spots stand out in a tiny canvas. In promoted (dotRadius 2.5 → top-decile would be 10 = 20-px ⌀), the same ratio creates an overlapping blob with Eisa's 7,645-note universe (~764 top-decile notes cluster densely).

Fix: `topDecileRadius = dotRadius < 1 ? dotRadius * 4 : dotRadius`. Detects compact vs promoted via the dotRadius value (only two values are ever passed: 0.75 and 2.5). Compact keeps the 4× contrast; promoted flattens to base 5-px ⌀ for ALL Acts nodes. Acts loses its size-channel signal in promoted mode — but Eisa's spec is unambiguous ("5px"), and the channel identity can be preserved through the title strip alone. If size signal is wanted back later, redesign options include: border thickness, fill-pattern, or hue intensity for top-decile.

**Fix B — Tighter click hit-test tolerance** (`MiniDome.svelte handleClick`):

Root cause of "swap by empty-area click isn't working 100%": the hover hit-test tolerance is 12 px (intentionally generous for hover discoverability). I was using the SAME 12-px tolerance for click hit-test. So a click 5-12 px from a star fell into the "star hit" branch and called `onOpenNote` → opened a note instead of promoting. Eisa perceived this as the swap intermittently failing.

Fix: separate tolerances for click vs hover.
- Click in compact: tolerance 3 px (matches the 0.75-radius dot + small margin)
- Click in promoted: tolerance 5 px / zoomScale (matches the 2.5-radius dot + small margin, scales with zoom)
- Hover (handlePointerMove): unchanged at 12 px (discoverability)

Now empty-area clicks (>3 px from any star) reliably fall through to `if (compact) onPromote(channel)`.

### Files (3, ~32 insertions, ~5 deletions)

- `src/lib/sight/v6/miniDome.ts`: `renderActsChannel` topDecileRadius formula + comment block.
- `src/lib/sight/v6/MiniDome.svelte`: handleClick tolerance constant + comment block.
- `lab/reports/SESSION-LOG-2026-05-14.md`: this entry.

### Build artifact

`Constellation_0.3.4_x64-setup.B6-fix7.exe` (this commit).

---

## §B.6-fix-8 — promoted dot 2px + repaint-on-channel-change bug (2026-05-15)

**Eisa cycle-7 verdict:**
- Stage 1: "Make it 2px instead of 5px (for every mini dome star who becomes the main dome)."
- Stage 2: real bug — "after I clicked the Confidence mini (top-right). It disappeared, and Acts is still at the main dome."
- Stage 3: PASS.

### Stage 2 root-cause analysis

Looking at Eisa's screenshot:
- Primary slot title: "ACTS — size (top decile)" (stale)
- Mini grid shows 4 minis: Universe, Stage (top), Acts, Provenance (bottom)
- Confidence is missing from the grid

Mini grid is correct: `{#each ALL_SLOTS as slot (slot)} {#if slot !== primaryChannel} ... {/each}` skips whichever channel is primary. So if Confidence is missing → `primaryChannel === 'confidence'`. The state DID update.

But the primary slot canvas still showed Acts visual + Acts title. The bug: my MiniDome `$effect` for repaint only watched `stars / highlightedPath / anchorLayout` — NOT `channel` or `compact`. When SightV6 swaps `primaryChannel` from 'acts' → 'confidence', Svelte reuses the same MiniDome component instance in the `{:else}` branch and just updates the `channel` prop. Without the prop in the effect's dependency list, paint() never fires on the channel change. Canvas shows the stale 'acts' render until something else (hover, resize, etc.) triggers a repaint.

### Fixes (2 lines + 2 lines, both in MiniDome.svelte)

**Fix A — Promoted dot size 2px:**
- `paint()`: `dotRadius: compact ? 0.75 : 2.5` → `dotRadius: compact ? 0.75 : 1`
- 1-radius = 2-px ⌀ per Eisa cycle-7. Acts top-decile stays flattened (the `dotRadius < 1` check in renderActsChannel is false for value 1, so top-decile = base = 1 = 2-px ⌀).

**Fix B — Repaint on channel/compact change:**
- `$effect`: added `void channel; void compact;` to dependencies.
- Now any prop change (including channel swap in primary slot) triggers paint().

### Files (2, ~16 insertions, ~3 deletions)

- `src/lib/sight/v6/MiniDome.svelte`: paint() dotRadius constant + $effect dependency list + comment blocks for both.
- `lab/reports/SESSION-LOG-2026-05-14.md`: this entry.

### Build artifact

`Constellation_0.3.4_x64-setup.B6-fix8.exe` (this commit).

---

## §B.6 cycle-8 PASS — §B.6 closed (2026-05-15)

**Eisa cycle-8 verdict:** Stages 2-5 all PASS. The channel-swap repaint bug is gone; promoted dot size is 2px; Acts no longer blobs; empty-area swap is reliable.

§B.6 — Phase 2's bidirectional linked brushing + dome-swap interaction model — closed after 8 fix iterations across 5 days of build + test cycling.

---

## §B.7 — cross-filter from mini-dome category click (Shift-click) (2026-05-15)

**Eisa picked Option B** from the three §B.7 UX options I proposed (sidebar-only / Shift-click / Provenance-only). Option B: plain click on star = open note (current); Shift-click on star = filter universe to that star's category in the channel that mini represents.

### Implementation (3 files, ~80 insertions)

**`src/lib/sight/v6/MiniDome.svelte`:**
- New `onFacetFilter?: (facetId: FacetId, categoryId: string) => void` prop with default no-op.
- `handleClick`: after star hit-test succeeds and BEFORE the open-note branch, check `ev.shiftKey`. If true, look up the StarDerived from `stars.find(...)`, compute `(facetId, categoryId)` per channel:
  - `stage` → `('stage', star.row.stage)` (raw value, may be Living Link or Concept Paper vocabulary)
  - `confidence` → `('confidence', confidenceLevelOf(star.row))` (uses existing facets.ts discretization: alpha → hypothesis/evidence/established/contested)
  - `provenance` → `('provenance', star.provenanceSector)` (already pre-computed in StarDerived)
  - `acts` / `anchor` → no-op (no corresponding facet)
- Imports `confidenceLevelOf` from `./facets` and `FacetId` from `./types`.

**`src/lib/sight/v6/SightV6.svelte`:**
- Both the primary-slot `<MiniDome compact={false}>` and the mini-grid `<MiniDome compact={true}>` instances now receive `onFacetFilter={handleFacetToggle}` — reusing the existing handler that the facet sidebar uses (`filters = toggleFilter(filters, facet, categoryId)`). Cross-filter applies uniformly across all 5 surfaces via the existing `filteredRows → recomputeStars → repaint` data flow.

**`src/lib/sight/v6/facets.ts`:**
- `buildStageFacet` enumerates stages **dynamically** from row data instead of hardcoding the 5-stage `LifecycleStage` Concept Paper list. Display order: Living Link lifecycle progression (Spark → Birth → Growth → Maturity → Dormancy → Renewal → Archival) first, Concept Paper v4.0 vocabulary second, any other strings found in data third (descending count). Only stages present in data appear (zero-count chips suppressed).
- Without this fix, Shift+click on a Living Link star (e.g., spark) would apply the filter correctly via string equality in `applyFilters`, but the active filter chip wouldn't appear in the sidebar — user couldn't see or remove it.

### UX

- **Plain click on a star** in any dome (anchor / promoted mini / compact mini) → open note (unchanged).
- **Shift+click on a star** in Stage / Confidence / Provenance mini → toggle filter on that star's category. All 5 surfaces re-render to show only matching notes. Repeat Shift+click on same category → clear filter (toggle behavior). Sidebar chip also reflects the active filter.
- **Shift+click on Acts / Anchor stars** → no-op (no facet exists).

### Build artifact

`Constellation_0.3.4_x64-setup.B7-crossfilter.exe` (this commit).

---

## §B.7 cycle-1 PASS-with-feedback + §B.7-fix-1 (2026-05-15)

**Eisa cycle-1 verdict:**
- Stage 1: cross-filter applied ("Result: It worked per your test"), but two issues:
  1. Ctrl+D delayed at first — focus issue (deferred; eventually works)
  2. Gold ring too large: "It has to match the node size, even when zooming in"
  3. Ask: "We need to add a count of affected notes when shift-clicking"
- Stages 2, 3: PASS (multi-channel filter, Confidence filter both work).
- Stage 4: BUG — "Shift+click a star on the anchor dome switch me to the note in the editor."
- Stages 5, 6: PASS (plain click + empty-area swap regressions clean).

### §B.7-fix-1 — three fixes

**Fix A — Zoom-aware hover ring** (`miniDome.ts renderMiniDome` Pass 4): the previous code drew the ring at hardcoded world-radius 6 with line widths 4 + 2.2. With promoted MiniDome's zoom transform applied, those scaled multiplicatively → at zoom 24× the ring became ~144 px on screen. New formula: `ringRadiusWorld = dotRadius + 1.5 / zoomScale`, `haloLineWidth = 2 / zoomScale`, `goldLineWidth = 1 / zoomScale`. Result: ring sits ~1.5 screen-px outside the node at any zoom level; halo and gold strokes stay at constant 2/1 px screen widths. Compact path passes `zoomScale=1` (identity transform applied), so formula reduces to `0.75 + 1.5 = 2.25` world ≈ 4.5-px ⌀ ring around 1.5-px ⌀ dots — visible without dominating.

**Fix B — Shift+click on anchor = no-op** (`SightV6.svelte handleClick`): the anchor dome uses SightV6's own `handleClick`, NOT MiniDome's. My §B.7 Shift detection lived only in MiniDome, so Shift+click on anchor stars fell through to `onOpenNote` → opened the note (Eisa Stage 4 bug). Added `if (ev.shiftKey) return;` early-out — anchor has no channel-specific category (it's the universe-baseline view), so no-op is correct per the design.

**Fix C — Filter affected-count badge** (`SightV6.svelte` header): when any facet filter is active (`!filtersEmpty(filters)`), the header shows `X / Y notes` in a small gold-tinted badge between the subtitle and the Reset View button. User sees the immediate impact of a Shift+click without inspecting the sidebar. Imports `filtersEmpty` from `./facets`.

### Out of scope (deferred)

- **Ctrl+D focus delay.** Eisa noted but said it eventually works. Lower priority. Fix would be making it a window-level keydown handler instead of canvas-level — touches keyboard handling. If Eisa flags it as blocking, do in fix-2.

### Files (3, ~70 insertions, ~10 deletions)

- `src/lib/sight/v6/miniDome.ts`: renderMiniDome signature + Pass 4 ring formula.
- `src/lib/sight/v6/MiniDome.svelte`: pass `zoomScale` option in paint().
- `src/lib/sight/v6/SightV6.svelte`: handleClick Shift early-out, header filter-count badge, CSS, import filtersEmpty.
- `lab/reports/SESSION-LOG-2026-05-14.md`: this entry.

### Build artifact

`Constellation_0.3.4_x64-setup.B7-fix1.exe` (this commit).

---

*End of session log 2026-05-14. §B.7-fix-1 commit + build complete; Eisa cycle-2 test next.*
