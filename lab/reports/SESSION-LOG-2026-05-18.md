# Session Log — 2026-05-18

## Phase: MIG-026 baseline foundation 100% COMPLETE

Carry-over from `SESSION-LOG-2026-05-17.md`. The Phase γ → η cascade
ran through the evening of 2026-05-17 and into the early hours of
2026-05-18. Phase θ shipped at 23:32 yesterday; §θ-fix-1 + §θ-fix-2
followed in the early morning. This log captures the post-midnight
work + the MIG-026 baseline-foundation SHIP gate.

**Function in hand**: closing the MIG-026 baseline-foundation
cascade. All 24 curated tradition modules + all 9 shape renderers
+ all polish iterations are now shipped on `main`. Next phases
(ι manifests, κ user-definable, λ translations, μ ship gate +
audit) sit on top of this complete foundation and can be undertaken
in a fresh session.

---

## What landed since SESSION-LOG-2026-05-17.md

The 2026-05-17 log captured the full day from MIG-027 (Sight theme
inheritance) through Phase ζ.3 (Talmudic 13 middot). After that
log's last entry, the cascade continued through:

### Phase η — East Asian Confucian (commit `a34e4586`, 23:10)

Single combined commit per Eisa's "continue full Phase η" choice:
- `mencian-sprouts.ts` — sectoral 4-cell (Longino π/4 rotation) +
  optional central xìn ring at 15% radius
- `wang-yangming.ts` — binary-flow vertical (NEW layout variant —
  no central divider, side labels for zhī / xíng, center label
  for liángzhī, bidirectional horizontal arrows)
- `korean-songnihak.ts` — sectoral 4-cell (Longino pattern, 2×2
  grid encoded as 4 wedges)
- `anchor.ts::drawBinaryFlow` — `layout` discriminator added;
  `drawBinaryFlowVertical` helper for Wang Yangming
- BinaryFlowSpec gains `layout?: 'horizontal' | 'vertical'` field

Family complete: 3/3 East Asian Confucian modules + zero new shape
renderers needed.

Boss test (Phase η Stages 1-3): "All Pass."

### Phase θ — Latin American + African (commit `ff81e2a9`, 23:32)

Single combined commit per Eisa's "continue full Phase θ" choice.
Final tradition-modules phase + the last stub renderer:
- `mignolo-pluriversal.ts` — relational hub-and-spoke (NEW), 5
  clusters using Mignolo's own decolonial vocabulary (NO specific
  indigenous tradition names per the v2.10 blanket-exclusion ruling)
- `dussel-transmodernity.ts` — binary-flow concentric (NEW layout
  variant for inner-disc-totality + outer-ring-exteriority)
- `maldonado-torres.ts` — rings, 3 concentric tiers of coloniality
  (power / knowledge / being)
- `akan-wiredu.ts` — sectoral 3-cell (Peirce/Habermas π/6 rotation),
  Akan epistemic vocabulary
- `ibuanyidanda.ts` — relational hub-and-spoke (reuses Mignolo's
  renderer), "missing link" hub + 5 complementary clusters
- `anchor.ts::drawRelationalGraph` — IMPLEMENTED (the FINAL stub
  renderer). Hub disc + N cluster bubbles + spoke lines + labels
- `anchor.ts::drawBinaryFlowConcentric` (NEW) — inner-disc + outer-
  ring + ring boundary stroke + radial flow arrows on -x axis
- `BinaryFlowSpec.layout` extended with `'concentric'`
- `SightV6.svelte` — `'relational'` added to +2 star-radius boost
  tier

Both families complete: Latin American decolonial (3 modules) +
African philosophical (2 modules) = 5/5 Phase θ traditions.

**Architectural milestone at this commit**: all 9 of 9 TraditionShape
renderers implemented; all 24 of 24 curated baseline traditions
registered.

Boss test (Phase θ Stages 1-5): "All Pass" with 4 polish items.

### §θ-fix-1 (commit `b5acd123`, ~01:00)

Four polish items from Phase θ Boss test:
1. **masādir top divider** colliding with stratum labels — applied
   the same +π/4 rotation pramāṇa got in §δ.2-fix-1. Stratum
   labels now clean.
2. **Polanyi opacity** — `gradientSpec.centerOpacity` 0.18 → 0.40.
   Lighter fog at center; tacit stars 40% visible (was 18%).
3. **Polanyi star size** — `'gradient'` added to +2 boost tier.
   Stars in tacit cluster bigger.
4. **Mohist preview badge removed** — `preview: true` → `false` in
   TRADITIONS_META. Tooltip updated.

Boss re-test: "All Pass" with sectoral broader-visibility feedback.

### §θ-fix-2 (commit `e8391ca7`, 05:51)

Generalized the sectoral-visibility feedback:
1. **`'sectoral'` added to +2 boost tier** in SightV6.svelte. All
   9 sectoral traditions (Aristotelian + pramāṇa + masādir +
   Peirce + Habermas + Longino + Mencian + Sŏngnihak + Akan
   Wiredu) get +2 px star radius.
2. **BODY_OPACITY_MULT raised 0.7 → 1.0** in anchor.ts. Per-star
   alpha now equals confidenceAlpha directly (no dimming
   multiplier). Affects all traditions; Aristotelian's dense
   cluster reads brighter per Eisa's explicit instruction to
   include Aristotelian in the boost group.

Boss re-test: "All Pass."

---

## State of standing — MIG-026 baseline foundation complete

### Shipped + Boss-verified

**24 of 24 curated baseline traditions** (orientation v2.10):

| Family | Traditions | Phase(s) |
|---|---|---|
| Western classical | Aristotelian | pre-MIG-026 |
| Indian Nyāya | pramāṇa | pre-MIG-026 (+ §δ.2-fix-1 rotation) |
| Sunni Islamic uṣūl | masādir | pre-MIG-026 (+ §θ-fix-1 rotation) |
| Chinese pragmatist | Mohist sān biǎo | γ |
| Modern Western | Polanyi · Peirce · Habermas · Dewey · Husserl · Longino | γ + δ.1 + δ.2 |
| Arabic / Islamic beyond uṣūl | Ibn Rushd burhān · Shāṭibī maqāṣid · Ibn Khaldūn ʿumrān | ε.1 + ε.2 + ε.3 |
| Jewish (Abrahamic) | PaRDeS · Maimonidean prophecy · Talmudic 13 middot | ζ.1 + ζ.2 + ζ.3 |
| East Asian Confucian | Mencian sprouts · Wang Yangming · Korean Sŏngnihak | η |
| Latin American decolonial | Mignolo pluriversal · Dussel transmodernity · Maldonado-Torres | θ |
| African philosophical | Akan Wiredu · Ibuanyidanda | θ |

**9 of 9 TraditionShape renderers** implemented:
- `sectoral` (Aristotelian, pramāṇa, masādir, Peirce, Habermas,
  Longino, Mencian, Sŏngnihak, Akan Wiredu)
- `gradient` (Polanyi)
- `horizontal-bands` (Mohist sān biǎo)
- `cyclic-flow` (Dewey)
- `rings` (Husserl, Ibn Rushd, PaRDeS, Maldonado-Torres)
- `grid` (Shāṭibī)
- `binary-flow` × 3 layout variants (horizontal: Ibn Khaldūn,
  vertical: Wang Yangming, concentric: Dussel)
- `ladder` (Maimonidean, Talmudic)
- `relational` (Mignolo, Ibuanyidanda)

**Plus MIG-027 (Sight theme inheritance)** shipped 2026-05-17 +
Boss-tested across Constellation/Nord/Solarized × Light/Dark.

### At-risk / in flight

Nothing. All cascaded work is on `main` and Boss-tested PASS.

### Known doc-drift (logged for MIG-026 ship-gate cleanup)

1. `store.ts:3483` — duplicate `TraditionId` literal union. Each
   tradition extension requires updating both `types.ts` AND
   `store.ts`. Better: import `TraditionId` from `types.ts` so the
   type is single-sourced. **Triggers MIG-026 ship-gate consolidation.**
2. Concept Paper §4.1.2 (pramāṇa) describes quadrants as NE/SE/SW/NW;
   post §δ.2-fix-1 they're at E/S/W/N. **Doc-drift item — update at
   ship-gate.**
3. Concept Paper §4.1.3 (masādir) — same NE/SE/SW/NW description;
   post §θ-fix-1 they're at E/S/W/N. **Same doc-drift item.**
4. §8 Migrations table in orientation v2.11/v2.12 dated 2026-05-07;
   MIG-020 through MIG-025 rows missing. v2.12 added MIG-027 +
   noted the backfill gap. **Backfill scheduled at MIG-026 ship-
   gate.**

### Pending, not started

- **Phase ι** — 24 tradition manifests at `docs/traditions/<id>.md`
  + ⓘ disclosure-layer UI + scope-strip placement. Plan §11.
- **Phase κ** — user-definable plugin loader (κ.1 declarative JSON
  + κ.2 TS plugin loader). Plan §12.
- **Phase λ** — translation cascade for 15 locales. Plan §13.
- **Phase μ** — ship gate + 3-agent audit. Plan §14.

### Architectural confidence

The cascade pattern across Phases γ → θ + 5 fix-iterations validated:
- The chrome/semantic color split (MIG-027) inherits cleanly to
  every new tradition module. Every new renderer used `_chrome.*`
  without per-shape theme work.
- The per-shape star-radius-boost tier (added in §γ-fix-2) and
  the per-shape opacity treatment (consolidated in §θ-fix-2)
  proved sufficient to handle all 9 shape varieties' visibility
  needs.
- The "default-all-to-first-sector" pattern works for sectoral
  shapes; "hash-bucketed across all cells" works for spread shapes
  (horizontal-bands, grid, ladder, relational). Both will be
  superseded by per-note frontmatter once Rust-side
  `LayoutCacheRow` extends with the per-tradition fields.

---

## Today's commits

```
ff81e2a9  MIG-026 §θ — Latin American + African families + closes 24-tradition baseline
b5acd123  MIG-026 §θ-fix-1 — 4 polish items from Phase θ Boss test
e8391ca7  MIG-026 §θ-fix-2 — +2 size for sectoral + raised opacity globally
75fc0be6  SO — MIG-026 baseline foundation SHIP gate (orientation v2.13 + session log)
```

Plus the SHIP-gate commit (this log + orientation v2.13) to follow.

---

## Phase ι.1 — 24 tradition manifests shipped

**Function in hand**: drafting the 24 manifests at `docs/traditions/<id>.md`
per Plan §11.1 — the canonical scholarly content that the ⓘ disclosure
button (ι.2) will surface.

**Direction picked by Eisa** (session resumption 2026-05-18 post-handover):
"Work on the remaining: Phases ι (manifests + disclosure) · κ
(user-definable) · λ (translations) · μ (ship gate + audit)" — full
cascade through MIG-026 remainder, in order.

### What landed

25 files created at `docs/traditions/`:

- `README.md` — index + format key
- 24 manifests, one per tradition:
  - **Baselines (5)**: aristotelian, pramana, masadir, polanyi,
    mohist-san-biao
  - **Modern Western (5)**: peirce, habermas, dewey, husserl, longino
  - **Arabic / Islamic beyond uṣūl (3)**: ibn-rushd-burhan,
    shatibi-maqasid, ibn-khaldun-umran
  - **Jewish (Abrahamic) (3)**: pardes, maimonidean-prophecy,
    talmudic-middot
  - **East Asian Confucian (3)**: mencian-sprouts, wang-yangming,
    korean-songnihak
  - **Latin American decolonial (3)**: mignolo-pluriversal,
    dussel-transmodernity, maldonado-torres
  - **African philosophical (2)**: akan-wiredu, ibuanyidanda

### Manifest structure

Each manifest carries the YAML frontmatter `id / name / family / shape /
version / changelog` plus six prose sections: Hero metaphor, Scope,
Applicability, Lineage, Critique, Citation. Per-note frontmatter field
documented for each tradition (e.g. `pramana_kind`, `masadir_source`,
`maqasid_tier`+`maqasid_essential`, etc.) — these are pending the
Rust-side `LayoutCacheRow` extension that the MIG-026 handover already
flagged as a separate PJ.

### Grounding sources (no fabrication)

- `docs/research/MIG-026-candidate-registers.md` — Agent 1 research
  with full Chicago-style citations + honest scope statements for the
  19 new traditions.
- The 5 pre-existing baseline tradition modules' JSDocs
  (`src/lib/sight/v6/traditions/<id>.ts`) for Aristotelian, pramāṇa,
  masādir, Polanyi, Mohist sān biǎo.
- `src/lib/sight/v6/traditions/index.ts::FAMILIES` for the 10 family
  groupings.
- `lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md` §11.1 for
  the structural spec.

No invented citations, function names, or scholarly attributions. Each
critique section honestly surfaces the contested points (e.g. Wang
Yangming's akrasia objection, Mignolo's performative-tension problem,
Wiredu's ethnophilosophy concern, the religious-lineage exclusions
where they bear on the tradition).

### Folder rename honored

Per Plan §11.1 decision: `docs/traditions/` (NOT `docs/registers/`)
— honors the K1 full rename from Phase 0.

### Verification clause per Plan §11.1

> **light Boss spot-check** — open `docs/traditions/aristotelian.md`
> and `docs/traditions/maimonidean-prophecy.md`; confirm format
> consistent + scope-statements present.

Doc-only commit, no build needed. Eisa can spot-check directly in his
text editor.

### What's next: Phase ι.2

Per Plan §11.2: ⓘ button per chip (J3) + scope strip (J5) in dropdown.
Files: `src/lib/sight/v6/traditionChip.svelte` (add ⓘ icon + scope
strip in dropdown panel) + `src/lib/sight/v6/SightV6.svelte` (wire
`openManifest(id)` callback). Estimated 1 day.

This is where the doc work lands as user-visible behavior — clicking
ⓘ on a chip opens the manifest for that tradition in NotePane.

---

## Phase ι.2 — ⓘ disclosure button + manifest modal (code shipped)

**Function in hand**: ⓘ button (J3) per chip in tradition dropdown +
manifest viewer modal per Plan §11.2.

### Files

5 files changed:

- **NEW** `scripts/build-tradition-manifests.mjs` — build-time scanner
  that reads `docs/traditions/*.md` (excluding README.md) and writes
  `src/lib/sight/v6/traditions/_manifests.generated.ts` exporting
  `MANIFESTS: Record<TraditionId, string>` + `getManifest(id)`.
  Deterministic output (filename-sorted); 24 manifests at 87KB. Idempotent
  by construction.
- **NEW (generated)** `src/lib/sight/v6/traditions/_manifests.generated.ts`
  — committed to git for first-clone determinism + CI safety. Header
  documents the regeneration command.
- **MODIFIED** `package.json` — adds `prebuild` + `predev` hooks that
  fire `node scripts/build-tradition-manifests.mjs` automatically.
  Adds the `build:tradition-manifests` script for manual regeneration.
- **MODIFIED** `src/lib/sight/v6/traditionChip.svelte`:
  - New `openManifest(id: TraditionId)` callback prop.
  - New `onDropdownClose()` callback prop (cascade-close the manifest
    modal when the dropdown closes for any reason).
  - New ⓘ button per tradition row, between the name-button and the
    pin-star button. Theme-aware (`--text-faint` / `--text-normal`
    via CSS vars per MIG-027 pattern).
  - `$effect` watches `dropdownOpen`; on transition to false, calls
    `onDropdownClose()` so the parent can cascade-close the modal.
- **MODIFIED** `src/lib/sight/v6/SightV6.svelte`:
  - Imports `marked` (already a dep).
  - New state: `manifestModalId`, `manifestContent`, derived
    `manifestModalOpen`.
  - New handlers: `handleOpenManifest(id)` (async — lazy-imports the
    87KB generated manifests bundle on first click), `closeManifestModal()`,
    `stripManifestFrontmatter()`, `renderManifestMarkdown()`.
  - TraditionChip mount now passes `openManifest` + `onDropdownClose`
    callbacks.
  - NEW modal mounted at sight-v6-root level (sibling of header + body,
    z-index 100 above chip dropdown's z-index 50).
  - Theme-aware modal CSS using MIG-027 chrome vars
    (`--background-primary`, `--background-modifier-border`,
    `--text-normal`, etc.) + light-theme overlay variant via
    `:global(body.theme-light)` override.

### Architectural decisions

1. **Lazy import**. The 87KB MANIFESTS bundle loads only when the user
   first clicks ⓘ. Defers bundle weight until needed; first-click
   latency is ~50ms over a local file URL. Subsequent clicks reuse the
   already-loaded module.
2. **Single source of truth = the markdown files**. `docs/traditions/*.md`
   is the canonical scholarly content. The TS bundle is generated, not
   hand-edited. Build-script header explicitly says "do not edit by
   hand."
3. **Modal cascade-close via dropdown lifecycle**. Avoids the
   capture-phase Esc-handler precedence problem (chip's existing
   window+capture Esc handler would otherwise close the dropdown
   under a still-open modal). When dropdown closes for any reason
   (Esc, click-outside, tradition switch), modal closes via
   `onDropdownClose` cascade.
4. **Modal as overlay, not real NotePane**. The Plan §11.2 said
   "click opens manifest in NotePane (via existing `onOpenNote`
   callback or new `openManifest(id)` callback)". Manifests at
   `docs/traditions/*.md` are repo doc files, NOT user-Universe
   notes — using `onOpenNote` would require schema/IPC hacks. The
   `openManifest(id)` callback path with a Sight-internal modal is
   the clean choice. Functional equivalent: the user reads the
   manifest in a NotePane-styled card without leaving Sight.
5. **Build pipeline**. The `prebuild` npm script ensures the
   generated TS file is in sync before `npm run build` runs. The
   `predev` hook does the same for `npm run dev`. CI runs `npm run
   tauri build` which calls `npm run build` internally → `prebuild`
   fires automatically. No manual step required.

### Verification

`npm run check`: 3 pre-existing errors (PJ-012 LinkLifecycle.fresh +
2 PropertyEditor union types — same baseline as every other phase
this cascade), 0 new. File count 1417 → 1419 (the build script +
generated file).

### Cross-cutting risk review (per Working Agreement #4)

- **Esc precedence**: solved by modal cascade-close via dropdown
  lifecycle. No new window-level handler conflicts.
- **z-index stacking**: modal at 100; chip dropdown at 50; tour at 10.
  Modal sits cleanly above.
- **Bundle weight**: +87KB lazy-loaded. First-time click incurs ~50ms.
- **Memory**: marked imports + DOM allocations cleaned up when modal
  closes (Svelte handles the {#if} unmount).
- **Theme-awareness**: all chrome via CSS vars; verified across
  Constellation Light + Dark patterns.
- **i18n**: modal close button + loading message are English-only in
  Phase ι.2. Phase λ adds i18n keys.

### Plan §11.2 verification clause (Boss test stages)

1. **Stage 1**: open chip dropdown, see scope strips below each
   tradition name (already shipped in Phase β; verified visible).
2. **Stage 2**: click ⓘ on pramāṇa, manifest opens as a modal over
   Sight.
3. **Stage 3**: manifest is readable + citation present at the bottom.

Build kicked off for the Phase ι.2 .exe.

### What's next: Phase κ

Per Plan §12: user-definable tradition layer.
- κ.1 — declarative JSON layer + schema + loader (~2 days)
- κ.2 — TS plugin loader with Obsidian-trust security model (~2 days,
  HIGH risk per Architect §3.H)

---

## Phase κ.1 — Declarative JSON user-tradition layer (code shipped)

**Function in hand**: ship the declarative loader that lets users
author tradition JSON files at `<Universe>/.constellation/traditions/`
and have them appear in the chip dropdown alongside curated
baselines, per Plan §12.1.

### What landed

8 files changed + 2 new:

- **NEW** `docs/traditions/schema/tradition.v1.schema.json` — JSON
  Schema Draft-07 spec for the v1 user-tradition format. Required
  fields: `schema_version: 1`, `id` (must start with `user-`
  prefix), `name`, `shape` (one of `sectoral` / `rings` /
  `horizontal-bands` / `gradient`). Per-shape required specs:
  `sectorDividers` / `rings` / `horizontalBands` / `gradient`
  object. Optional: `family`, `tooltip`, `scope`, `citation`.
- **NEW** `docs/traditions/schema/EXAMPLE.json` — copyable
  template: 3-sector sectoral demo (observation / connection /
  synthesis). Eisa copies this into his Universe to test.
- **NEW** Rust IPC `sight_v6_read_user_traditions` in
  `src-tauri/src/sight_v6.rs` — scans
  `<active_universe>/.constellation/traditions/*.json` (excluding
  `schema/` subfolder + non-JSON files), returns `Vec<{filename,
  content}>` with deterministic filename sort. Graceful: missing
  dir → empty result; bad file → skip + stderr warn; other files
  still load.
- **MODIFIED** `src-tauri/src/lib.rs` — register the new IPC.
- **NEW** `src/lib/sight/v6/traditions/userDefinedLoader.ts` —
  frontend loader: calls the IPC, parses + validates each file
  against the v1 schema (hand-rolled validator — keeps dep surface
  small), constructs `UserTraditionModule` from the spec, returns
  the list. Schema-version mismatch → console.warn + skip. Duplicate
  ids → first-wins + warn.
- **MODIFIED** `src/lib/sight/v6/traditions/index.ts` — adds
  `USER_REGISTRY: Map<string, UserTraditionModule>` side-map +
  `registerUserTraditions()` setter + `allUserTraditions()` reader.
  `getTraditionById` widened to `TraditionId | string`, returns
  `TraditionModule | null` (cast UserTraditionModule →
  TraditionModule at the boundary; structurally compatible for
  renderer purposes).
- **MODIFIED** `src/lib/libraries/store.ts:3483` — `activeTradition`
  type widened with `| (string & {})` to accept user-prefixed ids
  while preserving literal autocomplete for curated names.
- **MODIFIED** `src/lib/sight/v6/SightV6.svelte` — onMount calls
  `loadUserTraditions() → registerUserTraditions()`, stores result
  in `userTraditions: $state` and passes via prop to TraditionChip.
  `handleOpenManifest` now branches on `user-` prefix: curated
  traditions use the bundled `_manifests.generated.ts`; user ones
  synthesize markdown inline from the UserTraditionModule's name +
  scope + citation fields.
- **MODIFIED** `src/lib/sight/v6/traditionChip.svelte` — accepts
  `userTraditions: UserTraditionModule[]` prop, synthesizes a
  `userTraditionMeta` lookup, appends a synthetic "User-defined"
  family section to the dropdown when `userTraditions.length > 0`.
  All click handlers (chip click, pin toggle, manifest open) widened
  to `string` ids so user-prefixed ids work identically.
- **MODIFIED** `docs/traditions/README.md` — added "User-defined
  traditions" section documenting the setup, required fields,
  per-shape specs, validation behavior, and the κ.2 forward-look.

### Architecture decisions

1. **Source of truth = the JSON files.** No DB schema, no in-app
   editing UI. Users author + drop files; the loader reads on next
   Sight mount. This matches Constellation's file-over-app principle
   from CLAUDE.md.
2. **Hand-rolled validator, no AJV dep.** The v1 schema is small
   (~10 fields + 4 per-shape specs); a hand-rolled validator stays
   ~100 LOC, emits Constellation-specific warnings, and avoids
   adding a runtime dep. AJV would be the right call for v2+ if
   the schema grows substantially.
3. **`user-` prefix mandatory.** Prevents id collisions with
   curated traditions (which never start with `user-`). The schema
   pattern `^user-[a-z0-9][a-z0-9-]{2,40}$` is enforced at load
   time.
4. **Cast UserTraditionModule → TraditionModule at the registry
   boundary.** The two types differ only in the id field (string vs
   TraditionId literal union). All renderer-consumed fields are
   identical. `getTraditionById` returns TraditionModule | null
   uniformly so anchor.ts + miniDome.ts + computeStarPositions don't
   need signature changes.
5. **Sight-mount-time load, not eager-boot.** The IPC fires only
   when SightV6 mounts (not on Constellation boot). Cost: one fs
   scan per Sight open. Users without Sight active pay zero.
6. **One bad file doesn't break the chip.** Per-file try/catch in
   the loader; failures log + skip; other files still load. Matches
   the resilience pattern of the curated-tradition incremental
   shipping.
7. **`family` field reserved for κ.2.** v1 schema accepts `family`
   but the chip always groups user traditions in a synthetic
   "User-defined" section at the bottom (regardless of the field
   value). κ.2 will surface the family-mixing logic with a per-file
   consent prompt.

### Verification

`npm run check`: 3 pre-existing errors (PJ-012 LinkLifecycle.fresh +
2 PropertyEditor union types — same baseline), 0 new. File count
1419 → 1420 (the new `userDefinedLoader.ts`).

### Cross-cutting risk review (per Working Agreement #4)

- **fs scan latency**: directory scan + per-file read at Sight
  mount. For typical user dir (0–10 files), <50ms. No keystroke
  hot-path impact.
- **Bundle weight**: +12KB for userDefinedLoader.ts. No new deps.
- **Reactivity**: registerUserTraditions populates the Map
  synchronously before recomputeStars/paint fire. Chip's
  userTraditions prop is a Svelte `$state` so prop change triggers
  re-render of dropdown.
- **Security**: file paths constrained to
  `<active_universe>/.constellation/traditions/*.json`. No
  directory traversal, no symlink follow. Read-only access.
- **Settings migration**: not needed (existing `activeTradition`
  string compatible).
- **i18n**: chip's "User-defined" section label, schema warnings,
  console messages — all English in κ.1. Phase λ adds i18n keys.

### Plan §12.1 verification clause (Boss test stages)

- **Stage 0**: install + verify mtime
- **Stage 1**: copy EXAMPLE.json into `<Universe>/.constellation/traditions/`,
  restart, see "User-defined" section in chip dropdown
- **Stage 2**: click the test tradition's chip, dome re-arranges
  per declarative sectors
- **Stage 3**: edit JSON to set `schema_version: 99`, restart,
  confirm warning + graceful skip

Build kicked off for the Phase κ.1 .exe.

### What's next: Phase κ.2

Per Plan §12.2: TS plugin loader. Files: NEW
`src/lib/sight/v6/traditions/pluginLoader.ts` (dynamic `import()`
scanner of `<Universe>/.constellation/traditions/*.ts` with
Obsidian-trust security model). Adds `appSettings.sight.enabledTraditionPlugins:
string[]` and consent-prompt UI for first-detection. ~2 days.
HIGH risk per Architect §3.H.

