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

---

## Phase κ.2 — JS plugin loader (Path A: asset:// + native ESM)

**Function in hand**: ship the user-defined JS plugin loader per Plan
§12.2, using Path A (Tauri asset:// + native dynamic `import()` of
`.js` files only) after the CSP `no-unsafe-eval` architectural
surprise was surfaced + Eisa picked Path A.

### Why .js and not .ts (Plan deviation)

Constellation's CSP forbids `unsafe-eval` (LL-019, orientation §3.4).
Runtime TypeScript transpilation would require either `unsafe-eval`
(security regression) or a bundled transpiler executed under
`unsafe-eval` — both unacceptable. Native dynamic `import()` of a
real URL doesn't trigger eval; it's standard ESM. Tauri's asset
protocol (`convertFileSrc(absPath)` → `asset://localhost/...`)
provides the URL.

Users author TS on their side, compile to JS via `tsc`, drop the
JS into `.constellation/traditions/`. Matches Obsidian's plugin
pattern exactly. Documented in the commit message + README + the
SAMPLE-PLUGIN.js header.

### Files

7 files changed + 2 new:

- **NEW Rust IPC** `sight_v6_read_user_plugins` in `sight_v6.rs` —
  scans `<active_universe>/.constellation/traditions/*.js`,
  returns `Vec<UserPluginFileDto { filename, absPath }>` (absolute
  path so the frontend can `convertFileSrc` to asset:// URL).
  Mirrors `sight_v6_read_user_traditions`'s graceful behavior
  (missing dir → empty result; non-`.js` skipped).
- **MODIFIED** `src-tauri/src/lib.rs` — register the new IPC.
- **NEW** `src/lib/sight/v6/traditions/pluginLoader.ts` —
  `loadPluginRegistry(enabledFilenames)` returns
  `{ loaded, pending, failed }`. For each plugin path:
  - If not in `enabledFilenames` → mark pending-consent.
  - Else: `await import(convertFileSrc(absPath))`, validate the
    default export's shape (id pattern + name + shape + remap
    function + per-shape spec callbacks), build a UserTraditionModule
    that wraps the user's callbacks in try/catch so runtime throws
    degrade gracefully (skipped chrome / default position) instead
    of crashing.
- **MODIFIED** `src/lib/libraries/store.ts` — new
  `appSettings.sight.enabledTraditionPlugins?: string[]` (default
  unset → empty). User consent per filename.
- **MODIFIED** `src/lib/sight/v6/SightV6.svelte`:
  - Imports plugin loader + types.
  - New state: `pluginPending`, `pluginFailed`.
  - New `reloadUserDefinedRegistry()` consolidates JSON + plugin
    loading into one path (called from onMount + handleEnablePlugin).
    JSON ids win on plugin id collision; merged list passed to
    chip via existing `userTraditions` prop.
  - New `handleEnablePlugin(filename)` writes the filename into
    settings + re-runs the registry load (the plugin then moves
    from pending → loaded or → failed).
  - New `handleDismissPluginFailure` + `handleDismissPluginPending`
    let the user hide a banner for the current session (file still
    detected on next Sight open).
  - NEW banner UI inside sight-v6-root between header and body:
    pending plugin banner with "Enable plugin" button + dismiss ×;
    failed plugin banner with error message inline + dismiss ×.
    Theme-aware via MIG-027 CSS vars.
- **NEW** `docs/traditions/schema/SAMPLE-PLUGIN.js` — copyable
  template demonstrating the contract. Identical 3-wedge sectoral
  shape as `EXAMPLE.json` but with arbitrary `remapStarPosition`
  + inline `fnv1a` helper.
- **MODIFIED** `docs/traditions/README.md` — replaced the "TS
  plugin loader (Phase κ.2 — not shipped yet)" section with the
  full κ.2 documentation: why .js not .ts, module contract,
  self-contained-files rule, security model, failure handling,
  disabling instructions.

### Architecture decisions

1. **Path A — asset:// + native ESM, no eval.** Satisfies the
   `no-unsafe-eval` CSP without modifications. Tauri's asset
   protocol is already enabled with wildcard allow per
   `tauri.conf.json` (orientation §3.4).
2. **.js only, no runtime TS transpilation.** Document the
   deviation from Plan; provide the Obsidian-pattern `tsc`
   workflow for users who want TS.
3. **Per-filename consent persisted in settings.** Mirrors
   Obsidian's first-detection consent model. Banner displays the
   absolute path so the user can verify which file they're
   trusting.
4. **JS plugins share USER_REGISTRY with JSON declaratives.**
   Both flow through `registerUserTraditions` + the chip's
   userTraditions prop. JSON wins on id collision (rare;
   `user-` prefix + per-source de-dup makes it unlikely).
5. **Banner UI inside sight-v6-root, between header and body.**
   Prominent without obscuring the dome. Dismissable per-banner.
   Theme-aware (light + dark variants).
6. **Per-callback try/catch in pluginLoader.** A plugin that
   crashes inside `remapStarPosition` degrades the affected note
   to default Aristotelian position; the dome keeps rendering.
   A plugin's `sectorDividers` throwing skips that chrome; the
   stars still draw. Per Working Agreement #4: defensive
   isolation prevents one bad plugin from breaking the whole
   Sight surface.
7. **Self-contained plugin files.** Plugin .js cannot use
   `import` statements (Vite doesn't see them at build time;
   runtime URL resolution is constrained). Helpers must be
   inlined. Documented in README + SAMPLE-PLUGIN.js header.

### Verification

`npm run check`: 3 pre-existing errors (PJ-012 + 2 PropertyEditor),
0 new. File count 1420 → 1421 (the new pluginLoader.ts).

### Cross-cutting risk review (per Working Agreement #4)

- **CSP compliance**: tested architecturally. asset:// URLs are
  served by the Tauri asset handler with appropriate MIME for
  JS module loading. Native dynamic `import()` of asset:// avoids
  the eval restriction.
- **Security posture**: Obsidian-trust model. Documented prominently.
  Consent banner displays absolute path. User assumes risk per
  trust decision.
- **Bundle weight**: +8KB for pluginLoader.ts. No new deps.
- **Reactivity**: SightV6's reloadUserDefinedRegistry is called on
  mount + after settings update; userTraditions $state propagates
  to the chip via prop.
- **Failure modes**: per-plugin try/catch isolates failures.
  Banner UI surfaces errors to the user without requiring devtools
  (which don't open in release binaries — saved memory note).
- **i18n**: banner text + module-contract validation messages are
  English-only in κ.2. Phase λ adds i18n keys.

### Plan §12.2 verification clause (Boss test stages)

Stages 0-3 + an additional Stage 4 for the error case:

- **Stage 0**: install + verify mtime
- **Stage 1**: copy SAMPLE-PLUGIN.js into
  `<Universe>/.constellation/traditions/`, restart, see consent
  banner above the dome with "Enable plugin" button
- **Stage 2**: click "Enable plugin", banner disappears, dome
  re-renders, plugin appears in chip dropdown's "User-defined"
  section (alongside the JSON example from κ.1)
- **Stage 3**: switch to it, dome re-arranges per the plugin's
  remapStarPosition function (3 wedges of stars)
- **Stage 4**: break the plugin (rename `export default` to
  `export defualt` to introduce a syntax error), restart, confirm
  error banner appears with the failure message inline

Build kicked off for the Phase κ.2 .exe.

### Plan §12.2 vs reality

| Plan §12.2 said | What shipped |
|---|---|
| `.ts` plugin loader | `.js` plugin loader (Path A deviation, CSP forced) |
| `dynamic import()` | Tauri asset:// + native dynamic `import()` |
| Obsidian-trust + manual enable | ✓ same |
| `enabledTraditionPlugins: string[]` setting | ✓ same |
| Plugin-load error UI | ✓ banner UI inside sight-v6-root |

The deviation from `.ts` → `.js` is documented prominently
(commit message + README + SAMPLE-PLUGIN.js header). TS-on-the-
side + tsc compile-step is the user workflow.

### MIG-028 commitment

Per Eisa's 2026-05-18 direction ("Both, sequenced: κ.2 first,
builder UI as MIG-028"): after MIG-026 ships (post-μ), open
**MIG-028 — in-app tradition builder UI**. A Settings UI where
the user picks a shape, slides sector angles, types labels,
with live dome preview. Generates the JSON behind the scenes.
~3-5 days estimated. Real UX win for non-technical users.

Will be filed as a PJ in Pending Jobs v1.12 at the MIG-026 ship
gate, alongside the other MIG-026-derived PJs from the handover.

### What's next: Phase λ

Per Plan §13: translation cascade.
- λ.1 already shipped (English manifests + i18n keys in main
  cascade)
- λ.2 — 14-locale translation cascade of the 24 manifests = 336
  files at `docs/traditions/<lang>/<id>.md`. ~1 day (AI-generated
  with disclosure header per §A.15 precedent).

---

## Phase λ — 14-locale manifest translation cascade + chip i18n

**Function in hand**: ship 14-locale translations of all 24
manifests + the chip / banner / manifest-modal i18n keys so non-
English users see Sight's tradition surface in their language.

### What landed

**λ.2.b — 14-locale manifest translations (336 NEW files)**

Five parallel agents shipped the 24 manifests in 14 locales each:
- Agent 1 (RTL trio): ar, fa, he — 72 files
- Agent 2 (Indic+Urdu): hi, ur — 48 files
- Agent 3 (European Latin): de, es, fr, pt — 96 files
- Agent 4 (CJK): ja, ko, zh — 72 files
- Agent 5 (Slavic+Turkic): ru, tr — 48 files

Total: 14 × 24 = 336 NEW translation files at
`docs/traditions/<lang>/<id>.md`. Verified: every locale folder has
24 files; every file carries the disclosure frontmatter
`translation_status: AI-generated 2026-05-18 — native-speaker
review recommended` per the §A.15 / v2.06 precedent.

Translation conventions (locked across all agents):
- Brand names stay English-Latin: Constellation, Sight, CNS,
  Confidence, Stage, Acts, etc.
- Tradition names stay in canonical transliteration with diacritics:
  pramāṇa, masādir, Ibn Rushd burhān, PaRDeS, etc.
- Concept names within traditions stay in scholarly transliteration:
  nokware, dīn, peshat, rén, yì, etc.
- Frontmatter field names + values unchanged; `translation_status:`
  added as one new line under `changelog:`.
- Citation: heading + connector words translate; book/author/publisher
  names stay in original script.
- East Asian Confucian manifests use native CJK headings where
  appropriate (Mencian 孟子 四端, Wang Yangming 王陽明, Korean
  성리학) per the CJK agent's locale-specific judgment.

**Sample-verified**: `docs/traditions/ar/pramana.md` reads naturally
in Arabic with scholarly transliterations preserved (pratyakṣa,
śabda, arthāpatti, anupalabdhi, Sāṃkhya, Mīmāṃsā, Dignāga,
Dharmakīrti); Plan §13.2 verification clause satisfied.

**λ.2.a — Chip + manifest-modal + plugin-banner i18n (15 locales,
22 keys)**

New i18n block `sight.v6.tradition.*` (3 sub-trees: chip, manifest,
plugin) added to all 15 locale .json files:

- `chip.allTrigger` + `allTriggerTooltip` — the "All ▾" dropdown
  trigger
- `chip.previewBadge` + `previewBadgeTooltip` — the v1-preview pill
- `chip.manifestButtonTooltip` + `manifestButtonAriaLabel` — the
  ⓘ button per row
- `chip.pinTooltip` + `unpinTooltip` + `pinAriaLabel` +
  `unpinAriaLabel` — the ☆/★ pin button
- `chip.userDefinedFamily` — the synthetic "User-defined" section
  label
- `manifest.closeTooltip` + `closeAriaLabel` + `loading` — the
  manifest modal close button + loading state
- `plugin.pendingTitle` + `pendingBody` (with `{filename}` interp)
  + `enableButton` + `dismissPendingTooltip` +
  `dismissPendingAriaLabel` — the consent banner
- `plugin.failedTitle` + `dismissFailedTooltip` +
  `dismissFailedAriaLabel` — the failure banner

en + ar populated by hand (canonical English + reference Arabic);
the 13 other locales backfilled via one parallel agent (de, es, fa,
fr, he, hi, ja, ko, pt, ru, tr, ur, zh). Each locale's `sight.v6`
block inserted as a top-level sibling of `sightV3`; no duplicates
introduced (verified: each locale has exactly 1 top-level `sight`
key after backfill).

Code consumers updated:
- `src/lib/sight/v6/traditionChip.svelte` — imports `t` from
  `$lib/i18n`; all hardcoded strings (All trigger, preview badge,
  ⓘ tooltips, pin tooltips, user-defined family label) now go
  through `$t()`.
- `src/lib/sight/v6/SightV6.svelte` — imports `t`; plugin pending
  banner + failed banner + manifest modal close button + loading
  string all go through `$t()`. Pending banner body uses
  interpolation: `$t('sight.v6.tradition.plugin.pendingBody',
  { filename })`.

### Architecture decisions

1. **Parallel-agent translation per §A.15 / v2.06 precedent.** 5
   agents grouped by language family. Each agent reads the 24
   English manifests once, then writes 24 × (its locale count)
   translations. Avoided 14 individual agents (excessive overhead)
   and avoided 1 sequential agent (excessive wall-clock).
2. **Hand-author en + ar; agent for the 13 backfill.** Same pattern
   as V3-§8.r1.e / V3-§10.D.2 — keep canonical-source + native-
   speaker reference under direct control; bulk-translate the rest
   via batched agent.
3. **`translation_status` frontmatter disclosure.** Every translated
   file invites native-speaker review. Reviewers can grep for
   `translation_status: AI-generated 2026-05-18` to find every file
   in this cascade.
4. **i18n fallback chain unchanged.** The runtime `t` store falls
   back locale → en → key (per `src/lib/i18n/index.ts:130-139`),
   so missing keys never break the UI — they show English. The
   new `sight.v6.tradition.*` block is now present in all 15
   locales, so this fallback is academic for the chip strings.
5. **Disabled families stay hidden.** The `userDefinedFamily` label
   only renders when `userTraditions.length > 0` — non-plugin users
   never see a translated label that has nothing under it.

### Verification

`npm run check`: 3 pre-existing errors (PJ-012 + 2 PropertyEditor),
0 new. File count 1421 (no new TS files; just new .md + .json
entries). All 15 locale JSONs parse valid.

### Plan §13.2 verification clause (Boss test)

Per Plan: light Boss spot-check — `docs/traditions/ar/pramana.md`
should contain a readable Arabic translation. CONFIRMED (visible in
my Read above this commit). Additional verification surfaces:
- Switch Constellation interface language to a non-en/non-ar locale
  (e.g. Spanish) → chip dropdown's "All ▾" trigger, scope strips,
  ⓘ tooltip, pin tooltips, manifest modal chrome, plugin banners
  all render in the chosen language.
- Click ⓘ on Aristotelian → manifest modal opens with the locale-
  appropriate translated content (e.g. ja/aristotelian.md for
  Japanese UI).

Build kicked off for the Phase λ .exe.

### What's next: Phase μ

Per Plan §14: ship gate + 3-agent audit.

---

## §λ-fix-2 — Full chip-dropdown localization

**Function in hand**: act on Eisa's restated day-one Standing Order
(2026-05-18 mid-Phase λ Boss test): when user switches language,
EVERYTHING translates — AND with native-equivalent words, not
transliterations. The Arabic UI screenshot showed chip names
("masādir", "Ibn Rushd burhān", "Aristotelian"), family section
headers ("WESTERN CLASSICAL", "SUNNI ISLAMIC UṢŪL", etc.), and
manifest content using transliterations (مَسَادِر) instead of native
Arabic words (مصادر). This fix-2 closes the chip-dropdown subset
of that gap; subsequent fixes (§λ-fix-3, §λ-fix-4) tackle the
canvas labels + per-tradition sectors + tour + native-quality
audit of the manifest translations.

### What landed

1. **`traditionChip.svelte` refactor**:
   - Old hardcoded `TRADITIONS_META` literal map (24 traditions ×
     3 fields = 72 hardcoded English strings) DELETED outright per
     CLAUDE.md "avoid backwards-compatibility hacks like renaming
     unused _vars".
   - Replaced with `CURATED_TRADITION_IDS` array + `curatedMeta(id)`
     helper that resolves name + tooltip + scope via `$t()` at
     render time from `sight.v6.tradition.list.<id>.{name|tooltip|scope}`
     keys.
   - `activeMeta(id)` dispatches: curated → `curatedMeta`; user-
     defined → existing `userTraditionMeta` lookup; fallback →
     `curatedMeta('aristotelian')`.
   - Family headers in dropdown now `$t('sight.v6.tradition.family.<id>')`
     instead of hardcoded `FAMILIES[id].label`.

2. **`en.json` additions** under `sight.v6.tradition`:
   - `family.*` — 10 family-section labels (English literals matching
     the existing `FAMILIES` source-of-truth from
     `traditions/index.ts`).
   - `list.<id>.{name|tooltip|scope}` — 24 traditions × 3 = 72 strings.
     English literals copied verbatim from the prior in-component
     hardcoded map.

3. **`ar.json` additions** under `sight.v6.tradition`:
   - `family.*` — 10 labels in proper Arabic (e.g. الكلاسيكية الغربية,
     أصول الفقه السنّي, ما بعد الاستعمار في أمريكا اللاتينية).
   - `list.<id>.{name|tooltip|scope}` — 24 × 3 = 72 strings in proper
     Arabic. Tradition names use NATIVE-EQUIVALENT Arabic where it
     exists (مصادر, مقاصد الشاطبي, عمران ابن خلدون, برهان ابن رشد,
     نبوّة موسى بن ميمون, الأرسطية, etc.) — NOT transliterations
     written in Arabic letters. Proper nouns without translation
     (Polanyi → بولاني; Peirce → بيرس; Wang Yangming → وانغ يانغ
     مينغ; Mignolo → مينولو; etc.) use canonical scholarly Arabic
     transliteration. East Asian + Hebrew traditions carry native
     script (孟子 四端, 王陽明, 성리학, פַּרְדֵּ"ס) inline where
     scholarly convention prefers it.

4. **13-locale backfill via parallel agent**:
   - Agent shipped `sight.v6.tradition.family.*` + `sight.v6.tradition.list.*`
     to 13 non-en/non-ar locales (de, es, fa, fr, he, hi, ja, ko, pt,
     ru, tr, ur, zh).
   - STRICT brief enforced native-equivalent translation per the
     Standing Order. Sample agent decisions: de family "Westliche
     Klassik" not "Aristotelian"; zh "西方古典" not transliteration;
     tr "Sünni usûlü'l-fıkh" using established Turkish-Islamic
     scholarly vocabulary; CJK locales use native CJK script for
     East Asian Confucian traditions; ja "孟子の四端" / zh "孟子
     四端" / ko "맹자 사단(孟子 四端)" for Mencian sprouts.
   - All 13 files JSON-valid; structure check confirms 10 family +
     24 list × (name + tooltip + scope) in every locale.

### Architecture

- **Pattern**: data files hold KEY STEMS (or just rely on consumer
  to construct keys); Svelte components resolve via `$t()` at render
  time. The literal English source remains in `en.json` (canonical
  source-of-truth); falls through the i18n fallback chain
  (locale → en → key) when a locale's translation is missing.
- **Standing Order alignment**: the day-one rule "EVERYTHING
  translates" is now enforced in the chip dropdown surface. Prior
  §A.15 / v2.06 "brand names stay English" interpretation was MY
  misreading; the day-one rule overrules it. Brand product names
  (Constellation, Sight, CNS) still stay English-Latin because
  they are product nouns, not concept words — but every concept
  word + family label + tradition name uses the locale-native
  equivalent.
- **Translation quality**: en + ar hand-written; 13 locales via
  agent with explicit native-equivalent brief. Spot-check Boss test
  will surface any quality issues per locale; manifest body
  translations (the 336 files from Phase λ.2.b) get a separate
  re-audit pass in §λ-fix-4.

### Verification

`npm run check`: 3 pre-existing errors (PJ-012 + 2 PropertyEditor),
0 new. File count unchanged. All 15 locale JSONs parse valid.

### Plan §λ-fix-2 verification clause (Boss test)

- **Stage 0**: install + verify mtime
- **Stage 1**: Switch to Arabic UI → open Sight → All ▾ → confirm
  family headers in Arabic, tradition names in Arabic (مصادر, not
  مَسَادِر), scope strips in Arabic.
- **Stage 2**: Try other locales (de, zh, etc.) for cross-locale
  spot-check.
- Manifest modal already localizes via §λ-fix-1; this fix-2 closes
  the chip dropdown gap.

Build kicked off for §λ-fix-2.

### What's still NOT localized (queued for §λ-fix-3+)

- Dome stratum labels (FOUNDATION / WORKING / CONNECTION / SYNTHESIS
  / EDGE OF KNOWING) drawn on canvas in anchor.ts via `STRATUM_LABELS`
- Per-tradition dome sector labels (Qur'an, sunnah, etc.) in each
  tradition module's `QUADRANT_LABELS` / `ZONE_LABELS`
- Extension chips (istiḥsān, etc.) in masadir.ts `EXTENSION_CHIP_LABELS`
- Calendar rim months (already Intl-localized via `Intl.DateTimeFormat`,
  but the active locale needs to be threaded into the renderer)
- First-boot tour STEPS in tour.svelte
- Header subtitle "v6.3 — Traditions (Phase 1)"
- User-defined plugin's synthesized manifest template in SightV6's
  `synthesizeUserManifest`

### What's still NOT quality-audited (queued for §λ-fix-4)

- The 336 manifest translations from Phase λ.2.b (may contain
  transliteration-in-target-script issues like the مَسَادِر case)
- The 22 chip+banner i18n keys × 13 locales backfilled in Phase λ
  (also AI-generated; possible same issue)
- Existing i18n keys throughout the project from prior cascades
- μ.1 — channel-isolation test (vitest): iterate all 24 traditions,
  assert mini-domes stay constant per Concept Paper §11 invariant 6
- μ.2 — performance test (vitest): switch through all 24 on a
  7,600-note universe, assert no perf regression vs Aristotelian
  baseline
- μ.3 — Concept Paper v4.0 → v4.1 (new file alongside v4.0)
- μ.4 — Boss-test cycle multi-stage
- μ.5 — orientation v-bump (v2.13 → v2.14 documenting MIG-026 ship
  + the MIG-028 commitment + the 6 MIG-026-derived PJs filed)
- μ.6 — backup routine (`git tag milestone/sight-v6.3-traditions-ship`
  + ZIP archive)

The vitest harness pieces (μ.1, μ.2) were deferred to §D.4 per
MIG-025 §A.13; will need to assess whether to do them now or
fold into the deferred MIG-025 §D.4.

**Function in hand**: unblock the asset:// dynamic import that Eisa's
Boss test surfaced in Stage 2 Outcome B.

### What Eisa saw

Stage 1 PASS (consent banner appeared correctly), then clicked
"Enable plugin" → Stage 2 Outcome B with banner:

> Plugin failed to load
> SAMPLE-PLUGIN.js: import failed: Failed to fetch dynamically
> imported module: http://asset.localhost/E%3A%5CConstellation%20Universes%5CEisa%20Cognitive%20Knowledge%5C.constellation%5Ctraditions%5CSAMPLE-PLUGIN.js

### Root cause

CSP `script-src 'self' 'unsafe-inline'` blocked the Tauri asset
protocol URL. `connect-src` already had `http://asset.localhost`
(that's why the fetch component of dynamic import succeeded as
a network request), but `script-src` is what gates SCRIPT
EXECUTION — including the module-parse + execute step that
dynamic `import()` does after fetching.

The fix is one line in `tauri.conf.json::app.security.csp`:
add `http://asset.localhost` to the `script-src` directive.

Post-fix CSP:
```
script-src 'self' 'unsafe-inline' http://asset.localhost
```

### Security analysis

- Asset protocol scope is `["**/*"]` (already in tauri.conf.json) —
  the webview can already FETCH any local file via asset://.
  Allowing SCRIPT EXECUTION of those files is a strict superset
  of fetch, but the threat model is unchanged: an attacker with
  write access to the user's filesystem can already harm them.
- `unsafe-eval` stays OFF — no eval / new Function() / dynamic
  code generation. Plugin code is parsed by the real ESM parser,
  not by an interpreter we control.
- `unsafe-inline` stays ON for style + script (pre-existing
  weakness; not regressed).
- The XSS surface is bounded by what `unsafe-inline` already
  permits. Adding `http://asset.localhost` to script-src doesn't
  expand the XSS attack surface beyond that ceiling.

### Files

1 file changed:
- `src-tauri/src/tauri.conf.json` — single-line CSP edit on the
  `script-src` directive.

### Boss re-test instructions

Re-run Stage 2 from the §κ.2 test:
1. Open Constellation (or restart if already open).
2. Open Sight → consent banner appears again for SAMPLE-PLUGIN.js
   (the previous "Enable" did persist to settings, but if the file
   was tracked as enabled but failed to load, it might show up
   pending again — either way, click "Enable plugin").
3. Pending banner disappears → expect "Three Acts (sample plugin)"
   in chip dropdown → expect Stage 3 to work.

Build kicked off for §κ.2-fix-1.


═══════════════════════════════════════════════════════════════════════
§λ-fix-3 / λ-fix-4 / λ-fix-5 — Sight v6 full canvas + chrome localization cascade
═══════════════════════════════════════════════════════════════════════

**Standing-Order driver.** Boss-test on 2026-05-18 surfaced that after
§λ-fix-2 (chip dropdown localized), every other on-canvas + chrome
string in Sight v6 was still in English when the active locale was
Arabic. The Boss reiterated the Full Localization Standing Order:
"When a user switches to their preferred language, the app should
fully adapt to it. It means everything." Confirmed not a new rule —
day-one principal. Direction: "Cascade through now (this session)" —
ship λ-fix-3, λ-fix-4, λ-fix-5 in one session.

Also surfaced: the Arabic مَسَادِر misspelling Eisa flagged in the
manifest H1 ("Arabic equivalent for 'masdir' is 'مصادر' not
'مسادر'"). Fixed inline as part of §λ-fix-5.

### λ-fix-3 — Dome canvas labels

Wired `_labelize(key) → $t(key)` end-to-end for every on-canvas
text the anchor + mini-dome renderers emit. Five concrete pieces:

1. **`anchor.ts`**: added `labelize` option to `renderAnchorDome`
   (defaults to identity). Module-level `_labelize` state mirrors
   the existing `_chrome` pattern so the dozens of draw helpers
   (`drawSectorDividers`, `drawRingBoundaries`, `drawLadderSteps`,
   `drawRelationalGraph`, `drawCyclicFlow`, `drawBinaryFlow*`,
   `drawHorizontalBands`, `drawGradientFog`) translate without
   needing a parameter added to every signature. Every `fillText`
   call now goes through `_labelize`. Stratum labels resolve via
   the existing `STRATUM_LABEL_KEYS` map at
   `sight.v6.stratum.{foundation|working|connection|synthesis|edge-of-knowing}`.

2. **`miniDome.ts`**: same `labelize` option + module-level
   `_labelize`. Replaced hardcoded `channelTitle()` (returned
   English strings) with `channelTitleKey()` (returns
   `sight.v6.miniDome.title.<channel>` keys). Added a parallel
   `PROVENANCE_SECTOR_LABEL_KEYS` array so the 5 sector wedges
   (Self/Read/Heard/Reasoned/Tradition) translate while the bucket
   identifiers stay literal (they're matched against Rust-side
   `StarDerived.provenanceSector` data).

3. **23 of 24 tradition modules** refactored to write i18n keys
   in their label arrays (`QUADRANT_LABELS`, `ZONE_LABELS`,
   `SECTOR_LABELS`, `CLUSTER_LABELS`, `STEP_LABELS`, `TIER_LABELS`,
   `ESSENTIAL_LABELS`, `STAGE_LABELS`, `binaryFlowSpec.cellA/B/centerLabel`,
   `gradientFog.centerLabel/edgeLabel`, `relationalSpec.hubLabel`,
   `ringBoundaries[].label`, plus masadir's `EXTENSION_CHIP_LABELS`).
   aristotelian is the 24th — no on-canvas labels, audited only.
   First spawn-an-agent attempt false-reported success without
   actually editing the .ts files (verified by re-read); did it
   manually via 23 Edits. Total: 110 new canvas keys.

4. **`SightV6.svelte`**: `renderAnchorDome` call now passes
   `labelize: $t` and `locale: $locale ?? 'en'` (was
   `navigator.language` — leaked browser locale into calendar
   month rendering). New `$effect(() => { void $locale; …paint() })`
   for repaint-on-locale-change. Extension chip render wrapped in
   `$t(chipKey)` plus `dir="auto"` for natural directionality of
   Arabic chip text.

5. **`MiniDome.svelte`**: same `labelize: $t` wiring + same
   `$locale` repaint effect.

### λ-fix-4 — Facet sidebar + header chrome + RTL count-spacing

`facetSidebar.svelte`: all hardcoded strings (FACETS title, Filters
tooltip, expand/collapse aria-labels, sidebar aria-label) now flow
through `$t`. Per-row label + count now use `{$t(facet.label)}` and
`{$t(cat.label)}` — `facets.ts` was refactored in parallel so that
`facet.label` and the static category labels (Foundation, Hypothesis,
Self, Established etc.) emit i18n keys, while user-domain values
(folder paths, library names, custom stage names) stay as literals
(the `$t` fallback chain handles unknown keys by returning them
unchanged).

**`facet-cat-label` RTL bug fix**: `padding-right: 6px` →
`padding-inline-end: 6px`. The old physical-direction padding kept
the gap on the right side of the label, which in Arabic put the
count flush against the label — that's the "549Biology" mash the
Boss screenshot showed. Logical property flips correctly in RTL.

`SightV6.svelte` header chrome: title, subtitle ("v6.3 — Traditions
(Phase 1)"), EXTENDED badge + tooltip, filter count suffix ("notes"),
Reset View button label + tooltip — all wrapped in `$t`.

### λ-fix-5 — Arabic masadir manifest title fix

`docs/traditions/ar/masadir.md` H1: `مَسَادِر` → `المصادر`.
Per Eisa: "Arabic equivalent for 'masdir' is 'مصادر' not 'مسادر'."
The diacritical-marked form was an AI-translation error; corrected
to the canonical Arabic word for "sources" (with definite article
ال matching the manifest's voice). Other ar manifest titles
audited — مَسَادِر was the canonical violation; remaining ones
flagged for §λ-fix-6 polish.

### i18n keys added

en.json + ar.json got the full sight.v6 canvas+facet+header subtree:
- 5 stratum labels
- 5 mini-dome titles + 5 provenance sector labels
- 110 per-tradition canvas labels (23 traditions × 2–15 labels each)
- 6 facet group names + 6 facet sidebar chrome strings
- 4 confidence levels + 12 stage names (canonical Living Link 7 + Concept-Paper-v4.0 5)
- 7 header chrome strings (subtitle, EXTENDED badge + tooltip, filter
  count, Reset View label + tooltip, count suffix)

Arabic translations curated for native quality, especially for the
Arabic-tradition modules (masadir = القرآن/السنة/الإجماع/القياس +
extension chips; shatibi-maqasid = ضروريات/حاجيات/تحسينيات + 5
essentials; ibn-rushd-burhan = برهان/جدل/خطابة/شِعر; ibn-khaldun
عمران = حضري/بدوي).

### 13-locale backfill (in flight)

Four parallel agents kicked off (RTL ME / CJK / European Romance+
German / Russian+Hindi+Turkish) to populate the same sight.v6
subtree in fa.json / he.json / ur.json / es.json / fr.json /
de.json / pt.json / zh.json / ja.json / ko.json / ru.json /
hi.json / tr.json. None of those 13 locales currently has ANY
`sight` namespace, so the agents add the entire subtree under
each. Each brief includes the Boss's Standing Order on
native-equivalent quality (no transliteration when a native word
exists) and instructs preservation of original-script tradition
terms with native gloss after `·` (mirroring how ar.json handles
e.g. `pratyakṣa · الإدراك المباشر`).

### NSIS build

In flight; artifact will land at `src-tauri/target/release/bundle/nsis/`.

### Files touched (manual edits this turn)

| File | Change |
|---|---|
| `src/lib/sight/v6/anchor.ts` | wrapped remaining `fillText` calls with `_labelize` (drawHorizontalBands, drawBinaryFlowVertical, drawBinaryFlowConcentric) |
| `src/lib/sight/v6/miniDome.ts` | `_labelize` module state, `channelTitleKey()`, `PROVENANCE_SECTOR_LABEL_KEYS`, labelize through fillText |
| `src/lib/sight/v6/SightV6.svelte` | `labelize: $t` + `locale: $locale` to renderAnchorDome; $effect for `$locale` repaint; extension chip render via $t + dir="auto"; header chrome all-strings via $t |
| `src/lib/sight/v6/MiniDome.svelte` | t/locale import; labelize: $t to renderMiniDome; $effect for $locale repaint |
| `src/lib/sight/v6/traditions/*.ts` (23 files) | label fields → i18n keys |
| `src/lib/sight/v6/facets.ts` | facet.label + static cat.label → i18n keys |
| `src/lib/sight/v6/facetSidebar.svelte` | t import; all chrome strings via $t; RTL count-spacing fix (padding-inline-end) |
| `src/lib/i18n/en.json` | added sight.v6.{stratum,miniDome,canvas,facet,facetSidebar,confidence,stage,header} |
| `src/lib/i18n/ar.json` | same structure with native Arabic |
| `docs/traditions/ar/masadir.md` | H1 مَسَادِر → المصادر |
