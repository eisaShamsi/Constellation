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

*End of session log 2026-05-14. §B.6 commit + build follow; §B.7 opens after Eisa accepts §B.6.*
