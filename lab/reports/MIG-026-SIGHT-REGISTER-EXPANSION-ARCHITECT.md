# MIG-026 — Sight Register Expansion + User-Definable Architecture — Architect Doc

**Status**: Architect phase (Migration Rule step 1 of 4). Awaiting Eisa review → Plan phase.

**Authored**: 2026-05-17. Eisa locked 5 antecedent decisions through Phase 3 §C cascade + Agent 1/Agent 2 research + 6 AskUserQuestion rounds. This doc maps the territory, enumerates design options, lists invariants. Plan phase ships next with step-by-step build sequence after Eisa approves.

**Antecedent commits**:
- Phase 3 §C cascade (§C.1 through §C.4 shipped + §C.3-fix-1 + §C.1-fix-1): `f295b296` → `b6b5743a`
- Religious-lineage rule + Ishrāqī exclusion: `5ef7d863`
- MIG-026 research persisted: `00938942`
- Curated-baseline picks locked: `851844d5`

**Antecedent docs**:
- `docs/Constellation Orientation & Onboarding v2.10.md` — the canonical lock for MIG-026 scope
- `docs/research/MIG-026-candidate-registers.md` — Agent 1 candidate survey
- `docs/research/MIG-026-userdefinable-architecture.md` — Agent 2 architecture survey
- `docs/Constellation-Sight-Concept-Paper-v4.0.md` — Sight v6 specification
- `lab/reports/MIG-025-SIGHT-V6-PLAN.md` — MIG-025 plan (closes at §B.11 ship; §C paused at §C.4; §D.1, §D.2 SUPERSEDED)

---

## §1 — Territory

### §1.1 What exists today (the v6.2-pre-MIG-026 register architecture)

After MIG-025 Phase 3 §C.1–§C.4 + the religious-lineage-rule landing, the live register architecture in `main` carries:

**Code surfaces** (`src/lib/sight/v6/`):
- `types.ts`:
  - `RegisterId` union — 5 members: `aristotelian | pramana | masadir | polanyi | mohist-san-biao`
  - `RegisterLayout` interface — `{ centerX, centerY, radius }`
  - `SectorSpec` interface — `{ angleStart, angleEnd, label? }` for register-supplied sector boundaries in canvas-math angle convention
  - `RegisterModule` interface — the per-register contract: `{ id, name, remapStarPosition(row, defaultPos, layout): {x,y}, sectorDividers?(layout): SectorSpec[], extensionChips?(): string[] }`
- `registers/index.ts` — registry: `REGISTRY: Partial<Record<RegisterId, RegisterModule>>` + `getRegisterById(id)` + `allRegisters()`
- `registers/aristotelian.ts` — identity remap (passthrough)
- `registers/pramana.ts` — 4-quadrant remap (Nyāya pratyakṣa/anumāna/upamāna/śabda); default `pratyaksha` if no `pramana_kind` frontmatter
- `registers/masadir.ts` — 4-sector remap (Qur'an/sunnah/ijmāʿ/qiyās) + 4 extension chips (istiḥsān/istiṣḥāb/maṣlaḥa mursalah/ʿurf); default `quran`
- `registerChip.svelte` — title-bar UI; collapsed default; expand-to-row on click; hover tooltips; click-outside + Esc collapse (Esc fix via capture-phase window listener per §C.1-fix-1)
- `anchor.ts` — `computeStarPositions` accepts optional `register?`; `renderAnchorDome` accepts optional `register?` and dispatches to `drawSectorDividers(ctx, layout, sectors)` helper between strata circles (step 2) and calendar rim (step 3)
- `SightV6.svelte` — paint() reads active register from `$appSettings.sight.activeRegister`; `recomputeStars()` computes `stars` (register-remapped, for anchor) AND `starsDefault` (Aristotelian, for mini-domes per §11.6 isolation); `$effect` re-fires recompute + paint when activeRegister changes
- `store.ts` — `appSettings.sight.activeRegister: RegisterId` field; default `'aristotelian'`; migration blocks for legacy `'dignaga'` and `'ishraqi'` → `'aristotelian'`
- Plus the inherited Aristotelian default geometry: radial = stratum band, angular = creation month (12 wedges), with per-note FNV-1a hash jitter

**Spec surfaces**:
- Concept Paper v4.0 — §4.1.1 Aristotelian, §4.1.2 pramāṇa, §4.1.3 masādir, §4.1.4 Polanyi (currently UNBUILT — chip only); §4.2.1 Dignāga (EXCLUDED), §4.2.2 Ishrāqī (EXCLUDED), §4.2.3 Mohist sān biǎo (chip only); §7 mini-dome stipulation; §11 invariants
- MIG-025 Plan — §C paused at §C.4; §D.1 + §D.2 SUPERSEDED; §C.5 (Polanyi build), §C.6 (transition), §C.7 (manifests + translations), §C.8 (persistence), §C.9 (isolation test), §C.10 (Help → Sight tour), §C.11 (ship gate) all defer to MIG-026
- Orientation v2.04 → v2.10: ship history + locked rules

**Chip UI** (current state, 5 chips in expanded row):
```
[ Aristotelian ●  pramāṇa  masādir  Polanyi  Mohist sān biǎo ]
```
Layout: horizontal inline-flex, padding 3×9px per chip, ~80–140px each. Total width at expansion ≈ 600–700px. **Will not fit 24 chips** in the same row at a typical title bar width.

**Mini-domes (per §11.6 invariant, MUST NOT change)**:
- 4 mini-dome channels: Confidence (opacity), Stage (hue), Acts (size, top decile), Provenance (5 sectors: Self/Read/Heard/Reasoned/Tradition)
- Render from `starsDefault` (Aristotelian positions), NOT from `stars` (register-remapped)
- This isolation is the **canonical commitment** that prevents rhetorical pluralism (Concept Paper §7)

**i18n surfaces**:
- 15 locales: ar, de, en, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh
- `src/lib/i18n/*.json` — register chip currently English-only (per §A.15 brand convention); register-specific keys not yet added (planned for §C.7 → now MIG-026 scope)
- `docs/help.{lang}/` — Sight + CNS help docs translated as of v2.05/v2.06; register-specific help waits for MIG-026
- `docs/registers/` — manifests folder does NOT yet exist; planned for MIG-026

**Settings persistence**:
- `appSettings.sight.activeRegister` — currently `RegisterId` union (5 values)
- Will need to extend to accept user-defined IDs (open string)

**Build path**:
- Tauri v2 + Rust + SvelteKit/Svelte 5 + SQLite FTS5
- Frontend bundle is loaded from disk at app launch
- Register modules are TypeScript files compiled into the frontend bundle (no dynamic loading today)

### §1.2 What MIG-026 adds

**24-register curated baseline** (5 existing + 19 new):

| Family | Existing | New (per v2.10 lock) |
|---|---|---|
| Aristotelian / Western classical | Aristotelian | — |
| Indian Nyāya philosophical | pramāṇa | — |
| Sunni Islamic uṣūl | masādir | Ibn Rushd burhān ladder · Shāṭibī maqāṣid · Ibn Khaldūn ʿumrān |
| Modern Western philosophical | Polanyi (chip only) | Peirce categories · Dewey inquiry · Husserl regional ontologies · Habermas 3 interests · Longino CCE |
| Chinese pragmatist | Mohist sān biǎo (chip only) | — |
| East Asian Confucian / Neo-Confucian | — | Mencian 4 sprouts · Wang Yangming liángzhī · Korean Sŏngnihak Four-Seven |
| African philosophical | — | Akan Wiredu · Ibuanyidanda (Asouzu) |
| Latin American decolonial | — | Mignolo pluriversal · Dussel transmodernity · Maldonado-Torres |
| Jewish (Abrahamic) | — | PaRDeS · Maimonidean prophecy · Talmudic 13 middot |

**Multi-shape architecture** — currently sectoral-only (`sectorDividers` returns angle-sectors). Need to add:
- **Ring-band shape** (concentric rings): Ibn Rushd burhān ladder (4 rings), PaRDeS (4 rings), Maldonado-Torres (3 rings), Husserl regional ontologies (central + petals)
- **Multi-tier grid shape** (2D sectoral × ring): Shāṭibī maqāṣid (3 × 5), Korean Sŏngnihak (2 × 2)
- **Cyclic-flow shape** (sectoral + arrow): Dewey inquiry (5-segment with chronology arrow)
- **Two-cell flow shape** (binary + arrow): Dussel transmodernity, Ibn Khaldūn ʿumrān, Wang Yangming
- **Ladder shape** (N-step linear or N-ring): Maimonidean prophecy (11 levels), Talmudic 13 middot
- **Relational/network shape** (nodes + edges): Ibuanyidanda (complementary network), Mignolo pluriversal (center-vs-border + clusters)
- **Continuous gradient** (already shipped as concept): Polanyi tacit/explicit fog

**User-definable register layer**:
- Declarative JSON files (per Eisa-locked hybrid choice) at a path TBD (§3.F)
- Optional TypeScript plugin modules for arbitrary geometry (per Eisa-locked hybrid choice)
- Chip UI must list curated + user-defined together with no UI distinction
- Persistence: `activeRegister` field extends from `RegisterId` union to open string

**Manifests + disclosure layer**:
- 24 manifests in `docs/registers/<id>.md` (curated only; user-defined registers carry their own scope/citation/critique inline in their JSON)
- Each manifest carries: id, name, citation, geometry_spec, scope, applicability, lineage, critique, version, changelog
- ⓘ affordance on each chip → opens manifest in editor (per §11.7)
- CARE-aligned disclosure layer (Indigenous-data-sovereignty patterns from Agent 1 §4)
- "Scholarly tradition" reframing of "epistemic register" terminology (Direction D from earlier questionnaire) — per Eisa earlier "(Recommended)" choice

**Translation cascade**:
- 24 manifests × 14 non-English locales = 336 translation files
- Plus i18n keys for new chip labels, scope hints, ⓘ panel content

**Architecture implications**:
- Chip UI must accommodate 24+ chips (current row-of-5 layout breaks)
- Renderer dispatches by register-shape type
- `RegisterModule` contract extends with shape declaration + optional shape-specific fields
- Hot-reload of user-defined registers without app restart (or at least graceful schema-version-mismatch handling)
- Performance gate: 24 registers × 7,600 stars × per-star remap stays under §11.3 (≤16ms cross-filter response)

### §1.3 What MIG-026 deliberately does NOT add

- No mini-dome channel changes (§11.6 isolation enforced)
- No new chip mounting locations (chip stays in title bar)
- No new note-side metadata fields (registers operate on existing row data + an upcoming `register_kind: <id>` opt-in frontmatter pattern, but the latter is a §C.X-fix-N follow-up after the user starts opting notes in — not MIG-026 scope)
- No CNS (Constellation Nervous System) changes — registers are Sight-only
- No federation-mode (cUniverse) implications — registers are universe-local settings

---

## §2 — Invariants (what must not break)

These 12 invariants are NON-NEGOTIABLE through MIG-026. Each ship gate verifies them. Any architectural option in §3 that violates an invariant is removed from the option set.

### §2.1 Concept Paper §11 invariants (carry forward)
- **§11.3 Cross-filter response ≤16ms** — register switch must not regress to >16ms on a 7,600-note Universe
- **§11.5 Pip foveation threshold** — pip suppressed when <1.5px (existing renderer rule)
- **§11.6 Anchor-only register remap** — mini-domes never see the active register; `starsDefault` array stays Aristotelian; this isolation is the canonical commitment against rhetorical pluralism
- **§11.7 Manifest + citation per register** — every curated register has a `docs/registers/<id>.md` manifest with citation; ⓘ chip opens it; user-defined registers ship their own inline citation field
- **§11.9 No persistent toggle bars** — chip is in title bar; clicking chip is the toggle, not a persistent strip
- **§11.10 Tour re-availability** — Help → Sight tour re-fires the orientation overlay (MIG-026 ships this if §C.10 hasn't already; per current state §C.10 is unbuilt)

### §2.2 Constellation foundational invariants (CLAUDE.md)
- **File over app** — register definitions live on disk in standard formats (Markdown manifests for curated; JSON for user-defined declarative; TS files for user-defined plugin). Users can open them in any text editor. Survives Constellation uninstall.
- **Local first** — no cloud round-trip for any register operation. No telemetry on register choice or switch.
- **Every keystroke instant** — register switch repaint must not block typing in NotePane/FocusPane. The repaint is anchor-only, so this should hold; but the architecture must not introduce IPC calls on hot paths.
- **Multilingual by design** — chip labels stay English (per §A.15 brand convention); scope/applicability text supports 15 locales; per-line bidi continues to work in chip UI for RTL languages.
- **Reversibility** — adding a user-defined register, switching to it, then deleting the file → graceful fallback to Aristotelian default + warning; never data loss.
- **No silent file modification** — Constellation does not write `register_kind: <id>` frontmatter to a user's notes without explicit user action.

### §2.3 New invariants introduced by MIG-026

- **NEW: Religious-lineage rule** (orientation v2.09) — applies to all future curated-register additions. User-defined registers are NOT scoped by this rule (user is responsible for their own scope).
- **NEW: Geometric-shape isolation** — each register declares its shape; renderer dispatches by shape; no register can mix shapes (no register both has sectoral AND relational geometry). This keeps the shape-renderer interfaces clean.
- **NEW: Chip-list parity** — curated and user-defined registers appear identically in the chip UI. User-defined registers carry no visual stigma. (Per Agent 2 §6: "the chip can list both kinds of registers with no UI distinction.")
- **NEW: CARE-aligned disclosure** — each register manifest carries explicit scope + applicability statements visible on the ⓘ chip (per Agent 1 §4 — CARE Principles for Indigenous Data Governance, Drabinski's critical catalog theory). This is the "Direction A" scope-clarity commitment Eisa locked earlier.
- **NEW: User-defined register sandbox boundary** — TS plugin registers run with no explicit sandbox in v6.3 (Obsidian-style trust model). Wasm/QuickJS sandbox is a future MIG. Until then, plugin loader requires explicit user consent (default-off + manual enable, per Obsidian community-plugin pattern).
- **NEW: Schema-version compatibility** — user-defined register JSON declares `schema_version`; loader gracefully handles version mismatch (warns + falls back); breaking schema changes get a migration block in `applyParsedSettings`-style.

---

## §3 — Design Options (with speed/effort/risk)

For each architectural choice, candidate options are enumerated with engineering cost / UX cost / risk. **Eisa picks one per choice in the Plan phase.** Where I have a Recommended (per Constellation patterns + research), it's marked.

### §3.A — Chip UI redesign for 24+ registers

**Problem**: Current row-of-5 inline expansion does not scale to 24+ chips. Title bar width at typical Constellation window size (1440px) accommodates ~10 chips horizontally before wrapping or overflow.

| Option | Description | Eng cost | UX cost | Risk |
|---|---|---|---|---|
| **A1. Multi-row inline expansion** | Chips wrap to 2–3 rows when expanded. Title bar grows vertically. | Low | Low (still scannable) | Title bar gets tall (~80–120px when expanded); pushes dome down |
| **A2. Pagination** | Chips remain row-of-5–10; left/right arrows page through 24 chips. | Medium | Medium (less scannable; users must remember chip position) | Discoverability loss; some chips hide behind page-N |
| **A3. Family categorization** | Chips grouped by family with collapsible sub-headers (Arabic / Western / etc.) | Medium | Medium (more navigation; better orientation) | Family taxonomy is opinionated; some users may disagree |
| **A4. Dropdown menu** | Chip click opens a vertical stacked menu (Anytype-style). | Medium-Low | High change from current UX; one extra click to see options | Loses inline visibility |
| **A5. Searchable dropdown** | Chip click opens menu with type-to-filter input at top. | Medium | Best UX for many registers; worst UX for few | Search feels heavy when there are only 24 |
| **A6. Hybrid: 4 anchored "favorites" + dropdown for rest** | First 4 chips always visible inline (user-pinned favorites); dropdown for the rest. | High | Best balance | Requires "favorites" persistence in settings; non-trivial state machine |

**Recommended**: **A3 (family categorization) + A6 (4 favorites + dropdown for rest hybrid)**. The family categorization preserves scannability + gives users an orienting taxonomy; the favorites pattern gives power-users low-friction access to their working set. But **A1 is the simplest and likely good enough for v6.3** — defer A3/A6 to a follow-up MIG if 24 chips on 3 rows reads cleanly in Boss-test.

### §3.B — Multi-shape renderer architecture

**Problem**: Current `sectorDividers` contract returns angle-sectors. Need to add ring-bands, ladder, relational, etc.

| Option | Description | Eng cost | UX cost | Risk |
|---|---|---|---|---|
| **B1. Discriminated union in RegisterModule** | `RegisterModule` gains `shape: 'sectoral' \| 'rings' \| 'grid' \| 'ladder' \| 'relational' \| 'cyclic-flow' \| 'binary-flow' \| 'gradient'` discriminator. Each shape has its own optional fields. Renderer dispatches by `register.shape`. | Medium | Low (devs see shape on chip module; users don't see this) | Adding a new shape later requires both a new shape value and a new renderer branch |
| **B2. Renderer-as-method** | `RegisterModule` gains `render(layout, ctx, register): void` method. Each register provides its own renderer code. | Low | Low | Maximum flexibility; harder to enforce style consistency; harder to extend chrome (calendar rim etc.) uniformly across shapes |
| **B3. Strategy pattern with shape-classes** | Each shape is a class (`SectoralStrategy`, `LadderStrategy`, etc.); `RegisterModule` references a strategy. | Medium-High | Low | Classier; might be over-engineered for ~6 shapes |
| **B4. Plugin renderer + base contract** | Core renderer handles sectoral. Each non-sectoral shape registers a renderer plugin that runs after the base chrome. | High | Low | Architectural elegance; meaningful refactor of anchor.ts |

**Recommended**: **B1 (discriminated union) — proven, ergonomic, fits the Svelte+TS stack**. Renderer code in `anchor.ts` becomes a switch on `register.shape`. Each shape gets its own private `drawXxxShape(ctx, layout, register)` helper in anchor.ts. New shapes require touching one switch + adding one helper — bounded change.

### §3.C — Ring-band renderer (4 registers want this)

**Problem**: Ibn Rushd burhān (4 rings), PaRDeS (4 rings), Maldonado-Torres (3 rings), Husserl (central + petals) need concentric-ring boundaries rendered.

| Option | Description |
|---|---|
| **C1. RingSpec interface** | `interface RingSpec { radiusFrac: number; label?: string }`. Register returns array. Renderer draws concentric circles at those radii + labels at midpoints. |
| **C2. Reuse SectorSpec with sweep=full** | A "ring" is a sector spanning 0..2π. Adds noise to SectorSpec semantics. |
| **C3. Composite ShapeSpec** | A polymorphic `ShapeSpec` union covering sectors + rings + ladder steps. Register returns `ShapeSpec[]`. Generic renderer dispatches per spec kind. |

**Recommended**: **C1 (RingSpec)**. Clean, additive, doesn't disturb SectorSpec semantics. `RegisterModule` extends with optional `ringBoundaries?(layout): RingSpec[]`. Husserl's "central + petals" is then `ringBoundaries` (the central disc) + `sectorDividers` (the petals) — composes naturally.

### §3.D — Ladder renderer (Maimonidean 11, Talmudic 13)

**Problem**: 11-step or 13-step structures don't fit cleanly as sectors (too many; angles cramped) or rings (too many; concentric-circles get visually busy).

| Option | Description | Eng cost | UX cost | Risk |
|---|---|---|---|---|
| **D1. Concentric N-ring** | 11 concentric rings (or 13). Innermost = Moses-tier; outermost = least. | Low (reuses RingSpec from C1) | Medium (11 concentric rings in a small dome read as visual noise) | Visual clutter |
| **D2. Vertical step-list** | 11 horizontal bands stacked from top to bottom (or bottom to top). Like Mohist sān biǎo's 3 zones but with 11 zones. | Medium | Low (intuitive ladder metaphor) | Loses radial-symmetry of dome; visually disconnected from sectoral siblings |
| **D3. Spiral N-step** | Spiral arc from center outward; 11 marks along the spiral. | High | Medium-Low (novel; might delight or confuse) | Most original; honors "ladder" metaphor without being literal |
| **D4. Radial N-spoke** | 11 spokes from center to rim; spokes labeled at midpoint. Different from sectors (sectors are wedges; spokes are lines). | Low | Medium (spokes can read as dividers, not levels) | Visual ambiguity with sectors |

**Recommended**: **D2 (vertical step-list)** for Maimonidean and Talmudic. Honors the "ladder" metaphor literally. Reuses Mohist's horizontal-zone rendering (currently unbuilt but planned per §C.5 → now MIG-026). For users this reads as "the dome is a stack of bands"; the natural mental model is depth/level. Spec it via a new `LadderSpec` interface that the multi-shape switch dispatches to.

### §3.E — Relational/network renderer (Ibuanyidanda, Mignolo)

**Problem**: Two registers (Ibuanyidanda complementary, Mignolo pluriversal/border) want a node-link / network rendering. Sectoral is a fidelity loss.

| Option | Description | Eng cost | UX cost | Risk |
|---|---|---|---|---|
| **E1. Reuse Sky View renderer** | Sky View already renders a force-directed bubble graph in PIXI. Register-as-Sky-View-overlay: when relational register active, anchor temporarily hands off to a Sky-View-style renderer. | Medium | Medium (visual shift from dome to graph is jarring; users may not expect dome-to-graph transformation) | Significant architectural mixing; need to manage two renderer paths |
| **E2. Lightweight in-anchor network** | Draw nodes + edges within the anchor canvas, ignoring stratum/time. Custom force-directed layout in 50 lines of JS. | Medium-High | Low (stays in dome) | Performance: force-directed on 7,600 nodes is expensive (~50–200ms steady-state); only OK if relational registers are EXPECTED to be slower |
| **E3. Hub-and-spoke fixed layout** | For Mignolo: center disc (modernity/totality) + N outer clusters (subaltern positions). For Ibuanyidanda: every node connected to center "missing link" hub. Fixed geometry, no force simulation. | Low-Medium | Low | Loses fidelity for Ibuanyidanda's true network character |
| **E4. Defer relational entirely** | Cut Ibuanyidanda + Mignolo from the v6.3 baseline; they ship in MIG-027 when relational renderer is mature. | None | High (user picks them in chip, sees... what?) | Honest about not-shipping; chip would need to show "preview" badge or similar |
| **E5. Sectoral force-fit** | Render Ibuanyidanda as a sectored circle with sectors labeled by complementarity-pairs; render Mignolo as a center-disc + outer ring. | Low | Medium (significant fidelity loss vs the tradition's intent) | Honest only if Concept Paper §4 explicitly admits the fit-loss |

**Recommended**: **E3 (hub-and-spoke fixed layout) for v6.3 + planned E2 (lightweight in-anchor network) for v6.4**. E3 honors the central-vs-periphery shape of both traditions with a tractable rendering; E2 gives the full network experience but its performance unknowns (7,600 force-directed nodes) want a benchmark spike before committing. Eisa can pick differently if he wants the network render now.

### §3.F — User-definable register storage location

**Problem**: Where do user-defined register JSON/TS files live on disk?

| Option | Path | Visibility | Sync-friendliness |
|---|---|---|---|
| **F1. Per-user system-wide** | `~/.constellation/registers/<id>.json` | User has to navigate to AppData. Hidden. | Doesn't sync with Universe — same registers across all Universes. |
| **F2. Per-universe, in dot-folder** | `<Universe>/.constellation/registers/<id>.json` | User can find via Universe folder. Dot-prefixed so hidden by default. | Syncs with Universe (Git, iCloud, etc.). Different Universes can have different register sets. |
| **F3. Per-universe, visible** | `<Universe>/registers/<id>.json` | Top-level visible folder in Universe. | Syncs with Universe. Discoverable. |
| **F4. Per-library** | `<Universe>/<Library>/.constellation/registers/<id>.json` | Per-library scoping. | Library-level customization (e.g., Library "Hadith Studies" has different registers than Library "Linguistics"). |
| **F5. Hybrid: system + per-universe override** | `~/.constellation/registers/` + per-universe override directory. | More flexible. | Resolution rules: per-universe wins; falls back to system. |

**Recommended**: **F2 (per-universe in `.constellation/registers/`)**. Matches existing Constellation pattern (`.constellation/` is already the convention for per-universe config). Syncs with Universe; doesn't pollute user home; doesn't force a top-level visible folder on users who don't define registers. Hybrid (F5) is a v6.4+ enhancement if users ask.

### §3.G — User-definable register declarative schema

**Problem**: What does a user-defined JSON register file look like? What can it specify?

| Option | Description | Cost | Expressiveness |
|---|---|---|---|
| **G1. JSON Schema validated, fixed shape vocabulary** | `{"id":"my-register","name":"My Register","shape":"sectoral","sectors":[{"angleStart":-1.5708,"angleEnd":0,"label":"NE"},...],"defaultSector":"NE","frontmatterField":"my_kind","scope":"...","citation":"..."}`. Validator checks. | Low | Covers sectoral, rings, grids, ladders, binary-flow, cyclic-flow. Does NOT cover arbitrary geometry. |
| **G2. TypeBox / Zod schema** | Same shape but Zod-validated; richer TS-side types; harder for users to author without docs. | Low-Medium | Same coverage as G1; better type safety. |
| **G3. Free-form JSON + runtime tolerance** | No schema enforcement; loader interprets best-effort; warnings instead of errors for unknown fields. | Lowest | Highest tolerance; lowest contract clarity. |
| **G4. Hybrid: required core fields + open-ended `extra` field** | Mandatory: `id, name, shape, citation, scope`. Optional: any shape-specific fields. Schema validates the mandatory; `extra` is opaque. | Low | Best of both. |

**Recommended**: **G1 (JSON Schema validated, fixed shape vocabulary)**. Per Agent 2 §3 — DCMI Application Profile pattern + JSON Schema is the mature path. Constellation ships a JSON Schema at `docs/registers/schema/register.v1.schema.json`; user editors can hand-author + validate against it; future schema versions get a `register.v2.schema.json` + migration block.

### §3.H — TypeScript plugin loader mechanism

**Problem**: How are user-defined TS plugin registers loaded?

| Option | Description | Cost | Security |
|---|---|---|---|
| **H1. Static + dynamic import** | Plugin TS files live in `<Universe>/.constellation/registers/*.ts`. App on boot scans the directory, dynamically `import()`s each. Same trust level as Obsidian community plugins. | Medium | Same as Obsidian — full filesystem/network access via the plugin |
| **H2. Bundled at app-build time** | User submits TS to a Constellation plugin marketplace; reviewed; bundled into next Constellation release. | Low (no runtime loader) | Highest (curated review) |
| **H3. Wasm sandbox via Extism/Wasmtime** | Plugin compiled to Wasm; runs in sandboxed runtime; can't reach filesystem unless explicitly granted. | High (substantial new runtime; binary footprint adds ~5–15MB to installer per Agent 2 §7) | Highest in-process |
| **H4. QuickJS via embedded JS runtime** | Plugin written in JS; runs in QuickJS Rust embed; can't reach browser/filesystem APIs. | High | Medium-High (QuickJS has no built-in syscall capability; needs memory/fuel limits set by embedder) |
| **H5. Defer plugin layer entirely; ship declarative-only in v6.3** | Plugin layer waits for MIG-027. User registers stay JSON in v6.3. | None | Sandboxing problem deferred |

**Recommended**: **H5 (declarative-only in v6.3) → H1 (dynamic import in v6.4) → H3 (Wasm sandbox in v6.5)**. Phased delivery: declarative-only is the safe ship; dynamic-import adds power for users who want it (with explicit consent + default-off, Obsidian-pattern); Wasm sandbox is the eventual right answer but blocks on a benchmark spike per Agent 2 §7. Eisa can pick differently — H1 in v6.3 if he's comfortable with the Obsidian trust model.

### §3.I — Translation cascade strategy

**Problem**: 24 curated manifests × 14 non-English locales = 336 translation files. How to ship?

| Option | Description | Cost |
|---|---|---|
| **I1. English first commit, translation cascade in follow-up** | Per §A.15 precedent (v2.05 English + v2.06 14-locale cascade). Saves 336 files for a follow-up commit + v-bump. | Medium |
| **I2. Single commit (all 336 + English in one)** | One mega-commit with everything. | Highest immediate doc-drift-free state |
| **I3. AI-generated only for curated; user-defined ships English-only** | User-defined registers don't get translated automatically; user authors translations themselves if they want them. | Medium (only handles curated) |
| **I4. Per-batch (4 families × 4 commits, each batch shipping with its translations)** | Ship Arabic family + 14 translations; then Jewish family + 14 translations; etc. | Highest steady-state safety; longest cascade |

**Recommended**: **I1 (English first, translation cascade in follow-up) + I3 (user-defined ships English-only with user-translation hooks)**. Matches the §A.15 precedent Eisa is comfortable with. Follow-up commit gets its own v-bump (likely v2.13 or similar after MIG-026's main ship).

### §3.J — Disclosure / CARE layer

**Problem**: Per Agent 1 §4 (CARE Principles, Drabinski's critical catalog theory) + Eisa's locked "Direction A + D" choice (scope clarity + reframing). Each register needs visible scope, lineage, applicability, critique surfacing.

| Option | Description | Cost |
|---|---|---|
| **J1. Chip tooltip extended (single sentence scope)** | Hover chip → tooltip reads "pramāṇa — Nyāya fourfold; for epistemological analysis of cognitive acts; not designed for ritual or Vedic content." | Low |
| **J2. ⓘ button on each chip → panel** | Hover/click ⓘ → opens a side panel showing: name, citation, scope, applicability, lineage, critique, version. | Medium |
| **J3. ⓘ button → opens manifest in editor** | Click ⓘ → opens `docs/registers/<id>.md` in NotePane. User can read full manifest. | Low |
| **J4. Just-in-time popup on first switch** | First time user clicks pramāṇa, popup explains scope. Dismissible. | Medium |
| **J5. Always-visible scope strip under chip when expanded** | Chip row when expanded shows scope text under each chip. | Low |

**Recommended**: **J3 (ⓘ opens manifest in editor) + J5 (scope strip under chip when expanded)**. J3 is the proper CARE-aligned disclosure (full lineage + critique + citation); J5 gives at-a-glance scope without modal interruption. J2 is also a clean option; J4 risks alert-fatigue.

### §3.K — Reframe "epistemic register" → "scholarly tradition" terminology

**Problem**: Per Eisa's earlier "Direction A + D combined (Recommended)" choice, register terminology should be reframed to avoid implicit-universalism. The term "epistemic register" suggests a universal frame; "scholarly tradition" or "cultural-philosophical lens" admits each is a particular standpoint.

| Option | Description | Cost |
|---|---|---|
| **K1. Full rename throughout** | Chip label "register" → "tradition"; comments + Concept Paper + manifests + i18n all renamed. | Medium |
| **K2. UI-only rename, internal "register" preserved** | Chip label, tooltip, ⓘ panel all say "tradition". Internal code keeps `RegisterModule`, `RegisterId`, `registerChip.svelte` etc. (same precedent as Lens → Sight: keep internal names, rename labels). | Low |
| **K3. Skip the rename; just add scope to disclosure** | Keep "register" everywhere; rely on J3/J5 disclosure to clarify each one's scope. | None |

**Recommended**: **K2 (UI-only rename, internal "register" preserved)**. Same precedent as MIG-005 "Lens" → "Sight" (Eisa's pattern: rename labels, keep code-history). UI-facing strings change; the codebase stays stable. Concept Paper §4 gets a "what we call 'register' in this paper is rendered as 'scholarly tradition' in the v6.3 UI" footnote.

### §3.L — Active-register persistence schema extension

**Problem**: Currently `appSettings.sight.activeRegister: RegisterId` is a TypeScript union of literal strings. With user-defined registers, the ID is an open string.

| Option | Description |
|---|---|
| **L1. Open string** | `activeRegister?: string`. Validation at runtime. |
| **L2. Branded type** | `type RegisterId = (CuratedRegisterId | UserRegisterId)` where `UserRegisterId = string & { __brand: 'UserRegisterId' }`. |
| **L3. Curated + user-defined as separate fields** | `activeCuratedRegister?: CuratedRegisterId` + `activeUserRegister?: string`; either-or; chip determines which is active. |

**Recommended**: **L1 (open string)**. Simplest, matches the "chip-list parity" invariant. Validator at load time checks that the ID exists in the curated set OR resolves to a user-defined register file; if neither, falls back to `'aristotelian'` with a warning logged to console.

---

## §4 — Migration concerns

### §4.1 First-boot users with existing v6.2 settings

- Existing users have `activeRegister: 'aristotelian'` (or one of the 5 currently shipped). Continues to work.
- Existing users who set `activeRegister: 'dignaga'` or `'ishraqi'`: already covered by the §C.1-fix-1 + §C.4-religious-rule migration blocks in `store.ts`. Continues to work.
- No new migration needed for first-boot of v6.3 if user has no user-defined registers.

### §4.2 User-defined register schema-version mismatch

- A user-defined register file written under schema v1 should still load on v6.4 (where schema v2 ships, hypothetically) — with a `schema_version: 1` field in the JSON and a loader that handles older versions.
- Schema-version-mismatch handling: warn in console, attempt graceful interpretation, fall back to disabling the register if interpretation fails (chip greys out + tooltip explains).

### §4.3 Mid-build / interrupted-cascade users

- If user installs an interim v6.3-pre-final build, their persisted state should survive the final v6.3. The schema doesn't change between pre-final and final, only the register set expands. Safe.

### §4.4 Plugin (TS) registers on user disk during update

- If H5 (defer plugin layer) is picked: no plugin loader in v6.3. User who hand-creates a `.ts` file in `<Universe>/.constellation/registers/` sees no effect. No regression.
- If H1 (dynamic import) is picked: plugin file loaded on boot. If file fails to compile or throws, loader logs warning + skips file. No crash.

### §4.5 Concept Paper version handling

- Concept Paper v4.0 carries §4.1.1–§4.1.4 + §4.2.1–§4.2.3 (some EXCLUDED). MIG-026 expands to §4.1.5–§4.1.10 (modern Western), §4.1.11–§4.1.13 (Jewish), §4.1.14–§4.1.16 (Arabic non-masādir), §4.1.17–§4.1.19 (East Asian), §4.1.20–§4.1.21 (African), §4.1.22–§4.1.24 (Latin American). 19 new subsections.
- Each new subsection follows the existing format: geometry / cultural framing / citation / scope / v4.1 polish.
- Concept Paper version bump: v4.0 → v4.1 reflects the register-set expansion.

### §4.6 Rollback path

- If MIG-026 ships broken: revert to commit `851844d5` (the lock-but-no-code-touch state). Users lose user-defined registers but keep the 5 currently shipping. Settings migration handles graceful downgrade.

---

## §5 — Risks + Open Questions

### §5.1 Performance risks (benchmark before committing)

- **24 registers × 7,600 stars × per-star remap on switch**: At ~50ns per `remapStarPosition` call (TS function, no Wasm), 7,600 × 24 = 182,400 calls = ~10ms. Should clear the §11.3 16ms bar. Verify with a Vitest harness.
- **Multi-shape renderer dispatch**: switch-case overhead is negligible.
- **24-chip UI render**: 24 chips × ~80px = ~2KB DOM nodes. No issue.
- **Plugin loader (if H1 is picked)**: dynamic `import()` cost depends on browser/Tauri runtime. Benchmark needed if H1.
- **Wasm/QuickJS (if H3/H4 is picked)**: per Agent 2 §7, performance unknown without spike. Defer H3/H4 to MIG-027.

### §5.2 Scope risks

- **Concept Paper §4 expansion is the long pole**: 19 new subsections, each with proper scholarly grounding + citations + critique-awareness. Per Agent 1 §4 (CARE + Drabinski), each new subsection benefits from explicit "what this register is for and is not for" prose. Probably 200–500 words per subsection × 19 = ~7,000 words of Concept Paper expansion alone.
- **Manifests are similar scope**: 24 manifests × ~300 words each = ~7,200 words.
- **Translation cascade**: 24 manifests × 14 locales = 336 translation files in the follow-up commit.
- **Total new prose**: ~15,000 words of curation + ~14,000 words of translation (assuming similar length per locale). Substantial but bounded.

### §5.3 Architectural risks

- **Chip UI redesign**: A3 (categorization) introduces an opinionated taxonomy. If Eisa disagrees with "Modern Western" vs "Arabic / Islamic" family labels in chip UI, redesign needed.
- **Multi-shape architecture**: B1 (discriminated union) means adding a new shape later requires touching multiple files. Acceptable but not future-proof for arbitrary shapes.
- **User-defined trust model**: declarative-only in v6.3 dodges the security question; pushing it to v6.4 or v6.5 leaves users without arbitrary geometry until then.
- **Reframe terminology**: K2 (UI-only) preserves internal stability but introduces a permanent rename mismatch between code and UI. Same pattern as Lens → Sight; manageable.

### §5.4 Cultural-pluralism risks

- **Religious-lineage rule edge cases**: future candidates that are borderline (e.g., a register grounded in a philosopher who was also a theologian — like Augustine or al-Ghazālī himself) need a clear interpretation. The rule sidesteps Augustinian / Ghazālīan-Sufi candidates today; we should write down the interpretation for future candidate evaluations.
- **CARE-aligned disclosure**: J3 (ⓘ opens manifest) means each manifest needs a complete + careful scope/applicability/critique section. If we ship superficial manifests, the disclosure is a fig leaf. Quality-gate the prose.
- **Strict-lineage application has known costs**: 4 Indigenous frameworks + all Hindu + all Buddhist + Yoruba Ifá excluded. Some users may want them. The rule is Eisa's choice; the architecture supports user-defined registers as the escape hatch for "I want a register the curators rejected."

### §5.5 Open questions (need Eisa input before Plan phase)

These are decisions that fit naturally in the Plan phase but could shift the Architect if Eisa wants different defaults:

- **Chip UI: A1 (multi-row) vs A3+A6 (category + favorites hybrid)** — A1 is simpler; A3/A6 is more polished. Defer call to Plan?
- **Ladder shape: D2 (vertical bands) vs D3 (spiral)** — D2 is conventional; D3 is original.
- **Relational shape: E3 (hub-and-spoke) for v6.3** confirmed, OR E4 (defer Ibuanyidanda + Mignolo to MIG-027)?
- **Plugin loader: H5 (defer entirely)** confirmed, OR H1 (dynamic import in v6.3 with Obsidian-trust)?
- **Disclosure: J3 + J5** confirmed, OR J2 (panel-style ⓘ instead of opening manifest in editor)?
- **Concept Paper expansion**: do we land all 19 new subsections in MIG-026's main ship commit, or stagger across phases?

### §5.6 Cross-cutting risk: pace

- MIG-026 is the largest MIG yet attempted (24 registers + multi-shape + user-definable + translations). Eisa-locked "Get it right — take the time" priority means we don't rush; safe ship is the goal.
- Recommended pace: 4–6 phases over ~2 weeks of focused work, with Boss-test gate after each phase. Don't fuse phases.

---

## §6 — Phase decomposition (preliminary — refined in Plan phase)

Sketch of phase-by-phase build. Each phase is one commit (or small commit group) + Boss test. Plan phase formalizes verification clauses.

### Phase α (architecture foundation, no user-visible change)
- Extend `RegisterModule` interface with `shape`, `ringBoundaries?`, `ladderSteps?`, `relationalSpec?`, `cyclicFlow?` fields per §3.B (B1 discriminated union)
- Add new shape renderers as private helpers in `anchor.ts`: `drawRingBoundaries`, `drawLadderSteps`, `drawRelationalGraph`, `drawCyclicFlow`
- Existing 5 registers explicitly declare `shape: 'sectoral'` (Aristotelian, pramāṇa, masādir) / `'gradient'` (Polanyi, when its module ships) / `'horizontal-bands'` (Mohist, when its module ships)
- Add `RingSpec`, `LadderSpec`, `RelationalSpec`, `CyclicFlowSpec` interfaces in `types.ts`
- Add `register-shape` to chip UI architecture (chip is shape-agnostic; only the dome renderer cares about shape)

### Phase β (chip UI redesign)
- Decide A1 vs A3/A6 (in Plan phase or now)
- Implement chosen chip layout
- Verify the existing 5 registers still chip-display correctly

### Phase γ (add Polanyi + Mohist register modules — already planned in MIG-025 §C.5, §D.3)
- Build `registers/polanyi.ts` (gradient shape; fog opacity per-star)
- Build `registers/mohist-san-biao.ts` (horizontal-bands shape; 3 zones)
- These have been pending as chip-only placeholders since v6.1; MIG-026 ships their modules

### Phase δ (Modern Western family — 5 registers)
- Peirce, Dewey, Husserl, Habermas, Longino
- 4 sectoral / 1 cyclic-flow / 1 mixed (Husserl regional ontologies = central disc + petals)
- Easiest family (no diacritics; established Western scholarly format)

### Phase ε (Arabic Islamic family — 3 new registers)
- Ibn Rushd burhān ladder (rings)
- Shāṭibī maqāṣid (multi-tier grid 3×5)
- Ibn Khaldūn ʿumrān (binary-flow)

### Phase ζ (Jewish family — 3 registers)
- PaRDeS (4 rings)
- Maimonidean prophecy (ladder, 11 steps)
- Talmudic 13 middot (ladder OR toolkit-chip-overlay — Plan decides)

### Phase η (East Asian family — 3 registers)
- Mencian 4 sprouts (sectoral with center)
- Wang Yangming (binary-flow with center)
- Korean Sŏngnihak (2×2 grid)

### Phase θ (Latin American + African families — 5 registers)
- Mignolo pluriversal (relational — hub-and-spoke per E3)
- Dussel transmodernity (binary-flow)
- Maldonado-Torres (3 rings)
- Akan Wiredu (sectoral, 2–3 thin cells)
- Ibuanyidanda (relational — hub-and-spoke per E3)

### Phase ι (terminology reframe + disclosure layer)
- K2 (UI-only "register" → "tradition" rename)
- J3 + J5 (ⓘ button opens manifest + scope strip under expanded chip row)
- 24 manifests in `docs/registers/<id>.md` shipped with the registers

### Phase κ (user-definable declarative JSON layer)
- JSON Schema published at `docs/registers/schema/register.v1.schema.json`
- Loader scans `<Universe>/.constellation/registers/*.json` on boot
- Loaded user-defined registers appear in chip alongside curated
- Validator + graceful schema-version-mismatch handling
- Settings persistence extends to open string per L1

### Phase λ (translation cascade)
- English manifests (24) + i18n keys ship in main MIG-026 commit
- 14-locale translation cascade in follow-up commit (per §A.15 → v2.05/v2.06 precedent)

### Phase μ (ship gate)
- Channel-isolation test (per §C.9, deferred from MIG-025): iterate all 24 registers, assert mini-dome encodings stay constant
- Performance test (per §11.3): switch through all 24 registers; assert each switch <16ms
- Boss-test cycle: full 24-register Stage 1–N verification
- Concept Paper v4.0 → v4.1 ship
- Orientation v2.X bump documenting MIG-026 ship

---

## §7 — Decisions log (locked antecedents)

| Decision | Locked | Where |
|---|---|---|
| Religious-lineage rule (Abrahamic + Sunni-only for Islamic) | 2026-05-16 | Orientation v2.09 |
| Hybrid baseline + user-definable architecture | 2026-05-16 | AskUserQuestion + Orientation v2.09 |
| 24-register curated baseline (19 new + 5 existing) | 2026-05-17 | Orientation v2.10 |
| Multi-shape architecture (sectoral + ladder + relational + rings + grids + cyclic-flow + binary-flow + gradient) | 2026-05-17 | AskUserQuestion |
| Hybrid user-definable (declarative JSON + TS plugin layer) | 2026-05-17 | AskUserQuestion |
| Dignāga register EXCLUDED | 2026-05-16 | §C.1-fix-1 |
| Suhrawardi Ishrāqī register EXCLUDED | 2026-05-16 | §C.4-religious-rule |
| Phase 3 §C cascade PAUSED at §C.4 | 2026-05-16 | Orientation v2.09 |
| Translation cascade: English first, 14-locale follow-up | (per §A.15 precedent) | This Architect doc §3.I |
| Terminology reframe ("register" → "tradition" UI-only) | (per Eisa's earlier Direction A+D answer) | This Architect doc §3.K |
| Ship priority: "Get it right — take the time" | 2026-05-16 | AskUserQuestion |

---

## §8 — Open architectural choices needing Eisa's call before Plan phase

The following choices need Eisa's explicit pick to lock the Plan. Most have a Recommended; Eisa overrides if he wants different.

| Choice | My Recommended | Alternative(s) |
|---|---|---|
| §3.A chip UI redesign | A1 (multi-row inline expansion) for v6.3 simplicity; A3+A6 (category + favorites hybrid) as v6.4 polish if Boss-test reveals friction | A2 / A4 / A5 |
| §3.D ladder renderer | D2 (vertical step-list) | D1 / D3 / D4 |
| §3.E relational renderer | E3 (hub-and-spoke fixed layout) for v6.3 | E1 / E2 / E4 / E5 |
| §3.H TS plugin loader | H5 (declarative-only in v6.3); H1 in v6.4; H3 in v6.5 | H1 in v6.3 if Eisa is OK with Obsidian trust model |
| §3.J disclosure layer | J3 (ⓘ opens manifest in editor) + J5 (scope strip under expanded chip row) | J1 / J2 / J4 |
| §3.K terminology reframe | K2 (UI-only) | K1 (full rename) / K3 (no rename) |

If Eisa accepts the Recommendeds (or comments on the few he wants to change), I move to Plan phase: a step-by-step build sequence with verification clauses for each phase α–μ above.

---

**End of MIG-026 Architect doc.**

Awaiting Eisa review. After review + acceptance of §8 choices, the Plan phase doc lands at `lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md`.
