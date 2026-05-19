# Constellation Sight — Subsystem Concept Paper

**Version 1.0 | 2026-05-19**

> **Purpose**: Define Constellation Sight as a subsystem of Constellation, explain all its functions, and position it among the other subsystems. Complement to `docs/Constellation-Sight-Concept-Paper-v4.1.md` (the internal-design contract).
>
> **Filed as PJ-058**, 2026-05-19. Tracks the Sight delivery cascade (MIGs 025/026/027/028/030/031/032 closed; MIGs 029/033/034/035 reserved/deferred).

---

## §1 — What Sight is

Constellation Sight is the **whole-universe diagnostic visualization subsystem** of Constellation. It renders the user's entire epistemic content — every note across every library inside the active Universe — as a single radial dome whose geometry, color, and density encode the shape and organization of that content.

A one-paragraph elaboration. Sight reads from Constellation's write-time-derived layout cache (`sight_v6_layout`) and from the typed-link table (`note_links`), and paints each note as a star whose position along the radius encodes its place in the eight-level knowledge stratum, whose angular position encodes the month of creation, whose shape encodes its library of residence, whose opacity encodes its confidence level, whose size encodes a top-decile activity flag, and whose inner pip color encodes its lifecycle stage. Around that anchor dome sit four mini-domes that isolate single channels (Confidence, Stage, Acts, Provenance) for pre-attentive read-off, a facet sidebar that cross-filters by six dimensions (Folder, Library, Stratum, Confidence, Stage, Provenance), and a tradition chip that re-frames the dome's spatial grammar through twenty-four scholarly epistemic vocabularies plus any user-authored ones. The user does not configure Sight, query it, or train it; the user *reads* it. The diagnosis it offers is at-a-glance and pre-attentive.

Sight is currently shipping at **v6.3** as of 2026-05-19, ratified by the milestone tag `milestone/sight-v6.3-traditions-ship` cut on 2026-05-18 at the close of MIG-026 Phase μ. The v6.3 surface comprises twenty-four curated tradition modules across ten epistemic families, nine `TraditionShape` renderers (sectoral, concentric rings, two-dimensional grid, spiral ladder, hub-and-spoke relational, cyclic flow, three-layout binary flow, continuous gradient, horizontal bands), a two-tier user-definable plugin layer for traditions outside the curated set, full localization across all fifteen of Constellation's launch locales (ar / de / en / es / fa / fr / he / hi / ja / ko / pt / ru / tr / ur / zh), and full theme awareness so the chrome follows the active app theme while semantic encodings stay theme-invariant. The internal-design contract is `docs/Constellation-Sight-Concept-Paper-v4.1.md`; this subsystem paper sits one level above it and locates Sight in Constellation's architecture rather than re-deriving Sight's internals.

The canonical question Sight answers is — at the user-facing function level — **"How is my epistemic content shaped and/or organized?"** That question's scope is *the whole universe*. Single-note shape and organization is the domain of a sister subsystem (360.3D / Inspector 360) and is explicitly out of Sight's frame. Concept Paper v4.1 §1.1 phrases the user-facing question in a more clinical register ("Is my universe healthy? If not, where does it need to be handled?"); the version in this paper is the more general design-question both phrasings rest on. The healthy/handled framing is the diagnostic *use* of the more general shape/organization read.

---

## §2 — What Sight is not

A subsystem definition is sharpened by what the definition excludes. Sight is not several adjacent things it could be mistaken for; the negations matter because they show where the subsystem boundary actually runs.

**Not a file manager.** Files in Constellation are managed by the sidebar tree, the Library hierarchy, the tab manager, and the file-system watcher. Sight never creates, deletes, renames, moves, or modifies a note. Clicking a star opens that note in NotePane via an event the parent layout consumes; Sight itself does not own the open-tab operation.

**Not a single-note view.** The single-note diagnostic surface is 360.3D / Inspector 360 (`inspector360.rs` + the Stratification Matrix in `+layout.svelte`). 360.3D shows one note's place in the eight-stratum × eight-link-type matrix; Sight shows the whole-universe dome. The two are *orthogonal in scope*: same data sources, mutually-exclusive cardinality. A request for the shape of a single note belongs in 360.3D; a request for the shape of the universe belongs in Sight.

**Not Sky View.** Sky View is the graph-density visualization (`+layout.svelte` mounts the PIXI canvas; data flows through `sky_nodes` / `sky_links` and `cache.rs` snapshots). It renders notes as PIXI-bubble nodes positioned by force-directed layout, with link-density and clustering as the read-out. Sight renders notes as Canvas-2D stars positioned by stratum × time (or a tradition's remap), with epistemic structure as the read-out. Different mechanics, different questions, different visual languages, different rendering technologies.

**Not Constellation Map.** Constellation Map is the organizational-hierarchy visualization (`map.rs` returns a sunburst-arc tree; `ConstellationMap.svelte` renders via D3). It shows the folder-and-library nesting as concentric arcs and lets the user navigate by drill-down. Sight does not encode folder nesting in geometry at all; folder is one facet among six in the sidebar. Different mechanics (D3 sunburst vs Canvas-2D stars), different questions (organizational nesting vs epistemic structure), different visual languages.

**Not Org Chart.** Org Chart is the pure-hierarchy tree (`OrgChartPanel.svelte` + tree-layout primitives). It renders explicit parent/child relationships from typed `part-of` links as a top-down node-link diagram. Sight does not draw hierarchical trees; the closest Sight gets to tree-thinking is the stratum-band radial encoding, which is a *taxonomic depth* read, not a parent/child read.

**Not the Living Link editor.** The Living Link Architecture's editing surfaces — the link-type picker, the typed-wikilink syntax, the link annotation editor, the confidence dropdown, the Backlinks / Outgoing-Links sidebars, the link-archive workflow — all live in NotePane and adjacent sidebar panels. Sight *reads* the link data (color-coded line strokes for typed-link kinds, line auto-fade above eight hundred visible) but never writes to it.

**Not a recommendation, scoring, or coaching engine.** The original Sight v3.1 / v5 design ambition envisioned a four-layer instrument: Layer 1 visual, Layer 2 diagnostic findings, Layer 3 recommendation, Layer 4 coaching. The v6 architecture re-scoped to Layer 1 and the diagnostic *reading* of Layer 1; the recommendation and coaching layers were retired by Eisa on 2026-05-19 as no longer aligned with Sight's identity. Sight surfaces structure for the user to interpret; the interpretation is the user's act, not the subsystem's. (See §11 for the v6 trajectory and the Tier 4 retirement.)

---

## §3 — Constellation's subsystem map and Sight's place in it

Constellation is composed of subsystems at several different architectural strata. Locating Sight requires first naming the strata and the subsystems that populate them; otherwise the placement claim is empty.

### 3.1 Structural subsystems (the knowledge hierarchy)

These are the load-bearing data containers. They are not visualizations; they are the shape data the visualizations read. The structural hierarchy is **Universe → Library → Folder → Note**, with an optional **cUniverse** federation layer above the Universe (per CLAUDE.md "Constellation Knowledge Hierarchy"). A Universe is a portable directory containing `universe.json`, `.constellation/libraries.json`, settings, bookmarks, workspaces, and bases manifests; the Universe root auto-registers a default `universe_notes` library (Obsidian-style flat layout) and may register additional libraries with their own paths. A Library is a self-contained knowledge base with its own color, appearance, tags, links, and index. A Folder is plain file-system organization inside a library. A Note is a single `.md` file with optional YAML frontmatter and is the atomic unit of knowledge. A cUniverse is a *linked* Universe whose libraries get flattened into the parent at runtime via `resolve_libraries_recursive`. The structural subsystem provides the *what* — the files, folders, libraries, universes — and every other subsystem reads through it.

### 3.2 Authoring and editing subsystems

These are the surfaces where the user *makes* notes and links. They include NotePane (the full WYSIWYG CM6 editor, ~388 lines, supporting live-preview Markdown rendering, callouts, code blocks, tables, wikilinks, embeds, and the typed-link `[[Target|type]]` syntax), FocusPane (the plain-text capture editor, ~213 lines, intentionally stripped of every Markdown decoration so typing is instantaneous), the sidebar tree (the file/folder navigator), the tab manager (the open-note registry with split-view and write-ahead buffer per `src/lib/libraries/store.ts`), the command palette (the search-and-action surface), the Settings panel (the user-facing configuration UI), the Living Link editing affordances (typed-wikilink syntax, link annotation, confidence editing, Link Dashboard archive/unarchive), and the property editor in NotePane. These subsystems own the *write* side of Constellation; they are where source-of-truth data is created, modified, and saved to disk.

### 3.3 Diagnostic and visualization subsystems

These are the surfaces where the user *reads* the universe's shape. They include:

- **Sight** — the whole-universe radial-dome diagnostic. The subject of this paper.
- **Sky View** — graph-density visualization in PIXI, showing typed-link connectivity as a force-directed bubble graph.
- **Constellation Map** — organizational-hierarchy visualization in D3, showing folder-and-library nesting as a sunburst.
- **360.3D / Inspector 360** — single-note Stratification Matrix; renders one note's eight-stratum × eight-link-type cell distribution as an HTML/CSS grid.
- **Org Chart** — pure hierarchy tree from `part-of` typed links.
- **Knowledge Health** — summary card surfaces (universe-wide health metrics: tension count, fragile-note count, blind-spot count, etc.).
- **Index panel** — term-browser sidebar showing the FTS5 vocabulary with `via {lemma}` and `≈ similar` badges (the MIG-010/011/012 lineage); answers "where is the topic of X mentioned across my universe?"

These subsystems all *read* from the same source-of-truth storage and present different cuts of it. None of them modifies that storage. Sight is one of them.

### 3.4 Infrastructure subsystems

These are the cross-cutting subsystems every other subsystem depends on. They include:

- **CECE** (Constellation Epistemic Content Engine) — the classifier that assigns each note to axes of source (Self / Read / Heard / Reasoned / Tradition) and content type (Theory / Concept / Proposition / etc.). Outputs to `axis_assignment` rows.
- **CTSE** (Constellation Term Scanning Engine) — the morphological / lemma / semantic-bridge pipeline that maps user-typed terms to canonical concept ids via the Bridge Vector Store (M11 / `bridge_vectors/`).
- **The Arabic Engine + Lexical Bridge** — the five-layer Arabic morphological engine plus the polylingual lemma graph.
- **Search and Index** — the FTS5 + `note_meta` write-time-derived search system plus the Index panel's term-browser surface.
- **The Watcher** — `notify`-based external file-change detection.
- **The Boot Bundle** — the IPC-collapsed early-boot data hydration (`boot_bundle.rs`).
- **i18n** — the fifteen-locale translation system (`$t()` and `labelize`).
- **The Theme system** — six built-in themes (Constellation / Nord / Solarized × Light / Dark) plus per-library appearance overrides plus `deriveThemeVariables` plus `theme-light` / `theme-dark` body classes.
- **Storage and triggers** — SQLite (`note_meta`, `note_links`, `note_aliases`, `note_embeddings`, `sky_nodes`, `sky_links`, the `sight_v6_layout` cache, and several derived-view caches) plus the write-time triggers that maintain those caches from `note_meta` and `note_links` updates.

Sight sits cleanly downstream of each of these. It reads, never writes. The infrastructure subsystems are oblivious to Sight's existence; they exist independently and would still exist if Sight were removed tomorrow.

### 3.5 Where Sight sits

Sight is one of the **diagnostic and visualization subsystems** (§3.3). Laterally, Sight is a sister of Sky View, Constellation Map, 360.3D, Org Chart, Knowledge Health, and the Index panel; each of these is a different *cut* of the same underlying universe. Vertically, Sight is *downstream* of the storage triggers (which maintain `sight_v6_layout`), CECE (whose axis assignments inform the sources / content-type encoding), CTSE (whose conceptual bridges feed search affordances that interact with Sight via cross-surface event flow), the Arabic Engine and Lexical Bridge (whose lemma graph underwrites multilingual search and label translation), the i18n system (which translates every label), and the theme system (which paints the chrome). The only upstream consumer of Sight's output is the user; no other subsystem reads Sight's render or its computed visual encoding.

---

## §4 — Sight's functions

Sight does eight things. They are listed here at the subsystem-level grain; the per-tradition geometric details and the channel-encoding internals live in Concept Paper v4.1 §§3–4.

### 4.1 F1 — Render the universe as a unified radial dome

Sight paints the active Universe (its own libraries plus any cUniverse-federated libraries, flattened) as a single Canvas-2D dome covering the central content area. The dome is *stable*: notes do not drift between sessions, and the spatial grammar (radial = stratum; angular = month) is learnable. The Suwaidi criterion — that a user opening Sight for the first time can read where the cognitive density sits, how confident the universe is overall, where the orphans cluster, when the last surge of thinking happened, which library dominates, and whether the universe is in growth / dormancy / imbalance, all within roughly thirty seconds — is the acceptance test for the dome render (v4.1 §1.2).

### 4.2 F2 — Encode per-note characteristics via visual variables

Each note appears as one star. The star's *radial position* encodes its stratum (Foundation innermost, Edge of Knowing outermost); its *angular position* encodes the month of creation (January at the top, clockwise); its *shape* encodes its library of residence (circle / square / diamond / triangle / hexagon for one through five libraries, with outline-style rotation up to twenty-five); its *opacity* encodes its confidence level (0.4 hypothesis → 1.0 established, with desaturation for contested); its *size* encodes a top-decile-acts binary flag; its *inner pip color* encodes lifecycle stage (cyan fresh / violet growing / yellow at-risk / green established / gray dormant); and the *connector lines* between linked stars encode the typed-link kind (nine colors, dashed for contradicts, auto-fade above eight hundred visible). The channel-orthogonality invariant (v4.1 §3.2) ensures no two channels share a Bertin visual variable, so the encoding is pre-attentively legible at first glance.

### 4.3 F3 — Switch the dome's geometric grammar through 24 scholarly traditions

The Aristotelian radial-stratum / angular-time grammar is one frame among many. A *tradition chip* in the title bar exposes twenty-four curated scholarly vocabularies across ten epistemic families, each re-arranging the dome under a different culturally-grounded epistemic logic. The traditions, organized by family:

- **Western classical** — Aristotelian (default, identity remap; the five stratum bands are the structure).
- **Indian Nyāya** — pramāṇa (four quadrants of valid knowing: pratyakṣa / anumāna / upamāna / śabda).
- **Sunni Islamic uṣūl** — masādir (four sources of authoritative proof: Qurʾan / sunnah / ijmāʿ / qiyās plus four extension chips).
- **Arabic / Islamic beyond uṣūl** — Ibn Rushd burhān (four-ring demonstration ladder), Shāṭibī maqāṣid (3-tier × 5-essential grid = 15 cells), Ibn Khaldūn ʿumrān (badawī / ḥaḍarī binary with cyclic flow).
- **Modern Western** — Polanyi (tacit-explicit gradient fog), Peirce (Firstness / Secondness / Thirdness three sectors), Habermas (technical / practical / emancipatory three knowledge-interests), Dewey (five-stage cyclic inquiry with chevron arrows), Husserl (four regional ontologies as concentric rings), Longino (four CCE norms as sectors).
- **Jewish (Abrahamic)** — PaRDeS (four exegetical levels as concentric depth rings), Maimonidean prophecy (eleven-step spiral ladder), Talmudic 13 middot (thirteen-step spiral ladder).
- **East Asian Confucian** — Mencian sprouts (four sprouts as quadrants plus central xìn ring), Wang Yangming (zhī / xíng vertical binary flow with central liángzhī), Korean Sŏngnihak (4-wedge 2×2 grid encoding Four-Seven debate).
- **Chinese pragmatist** — Mohist sān biǎo (three horizontal bands: 本 běn / 原 yuán / 用 yòng).
- **Latin American decolonial** — Mignolo pluriversal (central modernity hub + five satellite clusters relational), Dussel transmodernity (totality / exteriority concentric binary with inward flow), Maldonado-Torres (three coloniality tiers as concentric rings).
- **African philosophical** — Akan Wiredu (nokware / ahonyam / adwene three sectors), Ibuanyidanda (central missing-link hub + complementary clusters relational).

Each tradition's hero metaphor, cultural framing, geometry, scope, critique, and primary scholarly citation are specified in v4.1 §4.1.1 → §4.1.24. The religious-lineage rule (orientation v2.09) determined which families and which schools within families enter the curated set — non-Abrahamic religious-source traditions are excluded, and for Islamic traditions, Sunni-only.

### 4.4 F4 — Surface per-channel encoding via four mini-domes

A 2×2 grid of mini-domes sits to the right of the anchor dome, hidden by default and revealed via Cmd-D or the "Show diagnostics" button. Each mini-dome shows the same notes in the same radial position as the anchor but isolates one channel with its visually-optimal encoding: **Confidence** as opacity gradient, **Stage** as full-disk categorical hue, **Acts** as binary size, **Provenance** as five angular sectors (Self / Read / Heard / Reasoned / Tradition). The mini-domes are **tradition-agnostic by design** (v4.1 §11 invariant 6, the channel-isolation invariant) — switching the anchor's tradition never changes what the mini-domes render. This is the architectural commitment that prevents rhetorical pluralism: the cultural framing applies to the anchor's spatial grammar; the mini-dome channels stay on their Western-analytic stipulation across every tradition.

### 4.5 F5 — Filter via six facets with Hearst-Flamenco preview

A facet sidebar on the left edge (collapsed by default, expanded via the `Filters ▶` tab) exposes six facets: **Folder, Library, Stratum, Confidence, Stage, Provenance**. Each facet shows live counts for each of its categories; clicking a category cross-filters all five surfaces (anchor + four minis) to matching stars. The pattern is Hearst-Flamenco: counts in each facet show what's available *given the current filter set in the other facets* (AND across facets, OR within). Folder is the top facet (v4.1 §11 invariant 8) because it's the user's primary navigational metaphor inside a library and was the most-asked-for facet in the v0.2 LIS critique.

### 4.6 F6 — Cross-link to NotePane

Clicking a star opens that note in a new NotePane tab. The interaction emits an `onOpenNote(path, libraryName)` event the parent layout consumes; the actual tab-opening logic is the same path the sidebar tree, the search results, and the wikilink-resolver all use. Sight is the *producer* of the open-note event; NotePane (via the tab manager) is the consumer. This is the single point of cross-surface coupling between Sight and the editing subsystems — and the coupling is one-directional (Sight → tab manager) and one-event-wide (open-note only; no save, no rename, no close).

### 4.7 F7 — Provide a user-definable plugin layer

The curated twenty-four traditions are a starting set, not an enclosure. A two-tier loader in `traditions/userDefinedLoader.ts` and `traditions/pluginLoader.ts` lets users add their own traditions without a code commit. **Tier 1** is declarative JSON: users drop a `.json` file conforming to `docs/traditions/schema/tradition.v1.schema.json` into `<Universe>/.constellation/traditions/`, and on next Sight mount the file is validated, the tradition is registered, and it appears in the chip dropdown under a "User-defined" section. Tier 1 covers four shape categories — `sectoral`, `rings`, `horizontal-bands`, `gradient`. **Tier 2** is a full-trust JavaScript plugin: users drop a `.js` file with an `export default` `PluginModule` (containing a deterministic `remapStarPosition` function) into the same folder; an Obsidian-style consent banner asks the user to opt in before the plugin loads. Tier 2 covers shapes outside the declarative four (`grid`, `ladder`, `relational`, `cyclic-flow`, `binary-flow`) and arbitrary remap logic. The Obsidian-trust model and the consent gate (which writes the plugin filename to `appSettings.sight.enabledTraditionPlugins`) is the deliberate widening of the CSP `script-src` to accept `asset:` URLs, paid for by the consent ask. Plugin labels pass through `$t()` via the i18n fallback chain unchanged (v4.1 §11 invariant 13).

### 4.8 F8 — Render with full theme- and locale-awareness

Sight follows the active app theme — chrome reads from a `ChromePalette` (8 fields, including axis labels, sector dividers, ring boundaries, ladder steps, stratum labels, hover ring) populated from CSS variables on `document.body` via `readChromePalette()`; the chrome repaints synchronously when `+layout.svelte`'s theme `$effect` rewrites the variables (MIG-027 cascade). Semantic encodings (Stage palette, typed-link palette) are theme-invariant by design — making them theme-aware would erase the information their categorical hues carry. A theme-conditional `--sight-highlight` CSS variable family resolves to bright amber `#fbbf24` on dark themes and deep amber `#b45309` on light themes (WCAG AA on cream). Sight follows the active interface locale through `$t()` and the `labelize` injection (v4.1 §11 invariant 12): every on-canvas label (stratum names, mini-dome titles, provenance sector labels, per-tradition canvas labels) and every chrome label (facet group names, chip dropdown family headers, ⓘ disclosure modal copy) translates across all fifteen launch locales without per-renderer special cases.

---

## §5 — What Sight reads (input contracts)

Sight is a *read-only consumer* of Constellation's source-of-truth data. The inputs are enumerated here so the data dependencies are explicit.

**The `note_meta` table.** SQLite, populated and maintained by `libraries.rs` write paths and the `note_meta_ai` / `note_meta_au` / `note_meta_ad` triggers. Columns Sight cares about include `path`, `name`, `library_name`, `created_at`, `modified`, `properties_json` (the parsed frontmatter blob), `tags_json`, `sources`, and `cid_cn`. This is the canonical view of each note's metadata; everything else Sight reads is derived from it.

**The `note_links` table.** SQLite, populated by `libraries.rs` link-extraction during reindex and maintained by the rename-cascade write paths. Columns relevant to Sight: `source_path`, `target_path`, `link_type` (one of the nine typed-link kinds), `confidence` (one of four confidence levels). Sight reads link edges between the visible note set to draw the connector lines; the `sight_v6_get_link_set_for_notes` IPC returns the matching edges in one round-trip.

**The `sight_v6_layout` cache.** SQLite, created and maintained by `sight_v6.rs::ensure_sight_v6_layout_table` + `ensure_sight_v6_invalidation_trigger` (registered into `init_db` per `search.rs` around line 1546). Each row is a per-note layout-cache snapshot: `stratum`, `maturity`, `confidence_alpha`, `contested`, `library_name`, `folder_path`, `created_month`, `sources_primary`, `stage`, `acts_primary`, `dominant_link_type`, `computed_at`, plus the v6 additions (`link_in_count`, `link_out_count`, `frontmatter_key_count`, `body_chars`). The cache is **maintained by triggers** — `sight_v6_layout_invalidate_au` deletes the row on `note_meta` UPDATE and `sight_v6_layout_invalidate_ad` on DELETE — and **populated lazily** by the `sight_v6_warm_cache` IPC on first Sight open (with a sentinel `mig025_sight_v6_layout_backfill_v1` gating the first-boot backfill). This is the canonical write-time-derivation pattern (CLAUDE.md Performance Rule 8): Sight never computes the layout on render; it always reads what's already stored.

**CECE classifications.** When the user has run CECE on a note (or accepted CECE's suggestions via the Source Review queue), the note carries an `axis_assignment` row recording its horizontal axis (sources) and vertical axis (content type). Sight reads the axis values for sources-based encoding (the Provenance mini-dome's five sectors) and, in the planned MIG-029, will read content-type axis values for tradition-aware per-note placement (`pramana_kind`, `masadir_source`, `peirce_category`, etc.).

**The stratum cache** (per-note stratum band). Maintained by the existing `note_meta` triggers (the `note_meta_sky_stratum_au` family in `search.rs`); Sight reads the resolved stratum band for each note as part of the layout cache row.

**Frontmatter** (planned in MIG-029). Per-note tradition-kind fields like `pramana_kind: pratyaksa` or `masadir_source: quran` will, once the Rust-side frontmatter extraction lands, override the default tradition-driven placement. Until MIG-029 ships, the default placement applies; the surface is therefore correct but unable to reflect user intent at the per-note grain.

**Theme CSS variables.** Read via `readChromePalette(canvasHostEl)` → `getComputedStyle().getPropertyValue('--name')` for the chrome palette; recomputed on every paint when the parent's theme `$effect` fires.

**i18n keys.** Every on-canvas and chrome label resolves through `$t(key)` on the active interface locale, with the `labelize` option on `renderAnchorDome` and `renderMiniDome` defaulting to identity for tests but always set to `$t` in production paint.

---

## §6 — What Sight writes

**Nothing on disk.** Sight is purely a visualization subsystem; it does not produce files, modify `.md` content, or alter SQLite rows that any other subsystem reads.

Sight *reads* write-time-derived caches (`sight_v6_layout`) but does not *populate or maintain* those caches itself — the triggers in `init_db` do that work in response to writes from elsewhere (the libraries write path, the rename cascade, the reindex pipeline). Sight's `sight_v6_warm_cache` IPC triggers a one-time first-boot backfill for the cache, but that is bootstrap work; steady-state cache maintenance is trigger-driven and Sight has no part in it.

User actions inside Sight (hover, filter selection, zoom, pan, tradition switch) are ephemeral UI state held in the SightV6.svelte component's `$state` variables. They do not persist across sessions or across windows. Two exceptions: `appSettings.sight.activeTradition` (the user's last-selected tradition) and `appSettings.sight.extended` (the persistent Pro mode flag, toggled with Cmd-Shift-D) both persist via the standard settings subsystem write path. Both writes go through `saveSettings()` in `src/lib/libraries/store.ts`; Sight emits the request and the settings subsystem owns the persistence.

This is the canonical **File-Over-App** read-only-consumer pattern (CLAUDE.md "Architecture Principles" §File Over App): the source of truth is on disk in `.md` files and in the SQLite database; visualizations are windows onto that truth, never authors of it.

---

## §7 — Composition with sibling subsystems

Where Sight touches other subsystems, the contracts are narrow, one-directional, and documented. The compositions below are listed at the subsystem-boundary grain.

**360.3D / Inspector 360 — orthogonal scope.** Same underlying data sources (`note_meta`, `note_links`, the stratum cache, the Living Link properties), different cardinality. 360.3D renders one note as a Stratification Matrix; Sight renders the whole universe as a dome. A user investigating "the shape of this specific note" goes to 360.3D; a user investigating "the shape of my universe" goes to Sight. The two surfaces share no code; they share data substrate. Future cross-linking is conceivable (right-click a star in Sight → "Open 360.3D for this note") but not currently wired.

**Sky View — different mechanic, similar data substrate.** Sky View uses PIXI for force-directed graph-density rendering; Sight uses Canvas-2D for radial-stratum-by-time rendering. Sky View answers "which notes are densely linked to which clusters?"; Sight answers "what's the epistemic structure of my universe?". Both read from `note_meta` and `note_links` and the stratum cache; both render multilingual stars; both follow the theme system (Sight's MIG-027 inheritance applies here directly, though Sky View's theme integration was deferred from MIG-027 scope). The mechanics are deliberately different — graph density is a force-directed problem, epistemic structure is a radial-projection problem; using one renderer for both would compromise both reads.

**Constellation Map — different mechanic, complementary read.** Constellation Map uses D3 for sunburst-arc rendering of the folder-and-library hierarchy; Sight uses Canvas-2D for stratum-radial rendering of the epistemic structure. Map answers "where is this file in the organizational tree?"; Sight answers "what's its epistemic standing?". A user navigating the universe by folder structure uses Map; a user diagnosing the universe by epistemic shape uses Sight. The two surfaces are explicitly *not* cross-linked at user request — Map ↔ Sight integration was filed as PJ-037 and rejected by Eisa on 2026-05-07: *"There won't be Map-Sight integration."*

**Org Chart — different mechanic, pure hierarchy.** Org Chart renders explicit `part-of` typed-link relationships as a top-down node-link tree. It has no overlap with Sight's epistemic dimensions; the two answer entirely different questions. Sight's stratum encoding is *taxonomic depth* (Foundation → Edge of Knowing, a maturity gradient), not *parent / child relation*; Org Chart's tree is the latter.

**NotePane — producer / consumer relationship.** Sight is a *producer* of "open note" events; NotePane (via the tab manager in `+layout.svelte`) is the *consumer*. Click a star in Sight → an `onOpenNote(path, libraryName)` callback fires → the parent layout dispatches to the same tab-opening path the sidebar tree, search results, and wikilink resolver use. Sight does not own the open operation; it requests it via callback. NotePane never reads from Sight.

**CECE — Sight depends on CECE, not the reverse.** Sight reads CECE's `axis_assignment` outputs as inputs to the Provenance mini-dome's sector encoding and (in the planned MIG-029) to tradition-aware per-note placement. CECE has no awareness of Sight; the classifier runs whether Sight is mounted or not, and the classification outputs would be valuable to other subsystems (Source Review queue, the Provenance facet, search filtering) regardless of Sight's existence. This is a clean one-way dependency.

**CTSE / search / Index — orthogonal questions, no direct coupling.** CTSE / search / Index answers "where is the topic of X mentioned across my notes?" — a content-search question. Sight answers "what's the shape of my universe?" — a structural-diagnosis question. The two are independent in everyday use; a user might bounce between them, but neither calls into the other. Future integration was envisioned in earlier Sight versions (search overlay highlighting matching stars in Sight) and was carried as a verification clause in v6.1 but not as a Sight-internal feature — the highlighting happens via the parent layout's gesture dispatch when a search is active.

**Knowledge Health — different lens on overlapping data.** Knowledge Health summarizes universe-wide quality metrics (tension count, fragile-note count, blind-spot count, orphan count) as a card-style readout. Sight visualizes the underlying shape from which Knowledge Health's metrics are derived. The two are complementary surfaces of the same diagnostic concern: Knowledge Health gives you a number; Sight gives you the picture. Neither calls the other; both read from the same source data.

**Living Link Architecture — Sight uses Living Link data, does not modify it.** Sight reads typed-link edges (with their `link_type`, `confidence`, weight, and traversal-count fields) and renders them as connector lines colored by link kind. Sight respects the confidence level (the line's opacity / saturation tracks confidence) and the typed-link palette (the nine kinds get distinct hues per v4.1 §3.4). Sight does not modify links, increment traversal counts, change confidence, or archive/unarchive — those operations live in NotePane, the typed-wikilink editor, and the Link Dashboard. The Living Link Architecture's commands (`_link_traverse`, `_link_set_confidence`, `_link_archive`, etc.) are not part of Sight's IPC surface.

---

## §8 — What depends on Sight

The user. That is the canonical and primary consumer.

No other Constellation subsystem reads Sight's output. Sight is a *terminal consumer* in Constellation's data flow: data flows from disk (`.md` files) → into storage (`note_meta`, `note_links`, the various derived caches) → into Sight → onto the screen → into the user's understanding. The screen is the end of the pipeline; nothing inside Constellation reads the rendered pixels back out. Sight emits the open-note event when the user clicks a star, but that emission is a *user-initiated dispatch*, not a Sight-output consumption — the tab manager would receive the same event from the sidebar tree, the search results, or a wikilink click, and treats it identically.

A potential future direction: federation queries across cUniverses (the **MIG-035** reservation in §11 below) may surface Sight diagnostic queries against linked Universes — e.g., "show me how the linked Universe's epistemic structure compares to mine." That would make Sight a *queryable* surface in addition to a viewable one. As of v6.3 Sight is purely viewable, not queryable.

---

## §9 — Architectural invariants Sight maintains

Sight's contract with the rest of Constellation has eight architectural invariants. These are commitments the subsystem makes to the surrounding architecture; a future Sight that violates any one of them is, by definition, no longer Sight v6 — it is v7 or a regression.

**I1 — Read-only with respect to source-of-truth data.** Sight does not write to `note_meta`, `note_links`, the SQLite caches, or the `.md` files on disk. The only persistent writes Sight initiates are to `appSettings.sight.activeTradition` and `appSettings.sight.extended`, which are routed through the standard settings subsystem write path. This is the architectural commitment that lets Sight be added, removed, or rebuilt without any risk to user data. It also enables the read-only-consumer pattern (File-Over-App; CLAUDE.md "Architecture Principles").

**I2 — Locale-reactive.** Every label flows through `$t()`; switching locale repaints chrome and canvas synchronously without per-renderer special cases. The `labelize` option on `renderAnchorDome` and `renderMiniDome` defaults to identity for tests and is always `$t` in production paint; the `_labelize` module-level state mirrors the chrome-palette pattern so every `fillText` callsite translates uniformly (v4.1 §11 invariant 12).

**I3 — Theme-reactive.** Chrome paints from a `ChromePalette` populated from CSS variables on `document.body`; semantic encodings stay theme-invariant. The chrome / semantic split is the architectural commitment that lets Sight follow the app theme without erasing the categorical information its semantic hues carry (MIG-027; v4.1 §3.4).

**I4 — Write-time-derived cache.** The `sight_v6_layout` cache is maintained by triggers fired from `note_meta` UPDATE / DELETE; it is never recomputed on Sight open. The first-boot backfill (`sight_v6_warm_cache`) runs once per Universe lifetime, gated by a `schema_versions` sentinel. This is the canonical write-time-derivation pattern (CLAUDE.md Performance Rule 8); it is the architectural commitment that lets Sight open instantly on a 7,636-note universe without any compute cost on the open-path.

**I5 — Mini-dome tradition-agnosticism.** The mini-domes (Confidence / Stage / Acts / Provenance) never see the active tradition; they always render the stratum × time arrangement under their Western-analytic stipulation regardless of which tradition is active on the anchor (v4.1 §7; v4.1 §11 invariant 6). This is the architectural commitment that prevents rhetorical pluralism — the cultural framing applies to the anchor's spatial grammar, not to the channel-encoding labels.

**I6 — Per-keystroke instant response.** The renderer must paint in ≤16 ms on a 7,636-note universe with five coordinated views; cross-filter response must hold the same budget (v4.1 §8.3). The CLAUDE.md Rule 1 ("Every Keystroke Must Be Instant") generalizes from typing into rendering; Sight's paint loop must hold that budget.

**I7 — User-definable plugin label passthrough.** Plugin authors' literal labels flow through `$t()` via the i18n fallback chain unchanged (active-locale → en → raw-key returned literal) so user-authored traditions integrate with Constellation's i18n without requiring plugin authors to ship localized strings (v4.1 §11 invariant 13). The architectural commitment is that user-defined traditions are *first-class* in the chip dropdown, not visually segregated as second-tier; they appear alongside the curated 24 under a "User-defined" section, with the same disclosure modal affordance.

**I8 — Zero `invoke()` on the keystroke hot path.** Sight's hover, filter, zoom, pan, and tradition-switch interactions never call Tauri IPCs in the synchronous path (CLAUDE.md "IPC boundary rules"). The three IPCs Sight uses — `sight_v6_get_layout`, `sight_v6_get_link_set_for_notes`, `sight_v6_warm_cache` — fire only at mount or when the user explicitly requests new data. Interaction is local to the rendered cache.

---

## §10 — Subsystem history (Sight's evolution within Constellation)

Sight's current architecture is the product of several earlier attempts. Each attempt taught what the next had to do differently; the history is preserved here as a record, not a regret.

**Sight v2** — graph-based; the original "Constellation Lens" surface. Showed typed-link relationships as a force-directed graph overlay. Retired via **MIG-017** (2026-05-07), which set `SIGHT_V2_ENABLED = false` in `src/lib/sight/engine.ts` and gated four UI entry points (dock button, modal mount, Return-to-Lens button, Settings plugin entry). The v2 Svelte component and Rust analytics IPCs were preserved on disk as a known-good fallback while v3 was built fresh.

**Sight v3** — star-chart projection foundation. Shipped via **MIG-018** (2026-05-07) with graph-distance Landmark-MDS embedding in Rust + Lambert / stereographic projections (user-toggle) + constellation territories (Suwaidi warm-cream + gold palette) + faint-at-rest connector lines + side panel. Production-reachable with `SIGHT_V3_ENABLED = true`. Superseded by the v6 pivot when the design direction shifted away from the InfraNodus-density mechanic v3 was building toward.

**Sight v4** — clean-slate intermediate. Shipped briefly as a design exploration; superseded by v6.

**Sight v5** — Layer 1 visual foundation for the four-layer instrument vision (Layer 1 visual / Layer 2 diagnostic / Layer 3 recommendation / Layer 4 coaching). Shipped via **MIG-024** (2026-05-12) with the coordinated-views architecture intended to grow into Layers 2–4. **Retired** via **MIG-028** (2026-05-19) when v6 took a different architectural direction; v5 dual-mount removed.

**Sight v6** — current architecture. The coordinated-views design with anchor dome + four mini-domes + facet sidebar + tradition chip; ships at v6.3 as of 2026-05-19. The build cascade across the v6 line:
- **MIG-025** (2026-05-15 → 2026-05-17) — foundation: §A (Sight v6.0, anchor dome + facets + default-simple), §B (Sight v6.1, mini-domes + cross-filter + Pro mode), §C (Sight v6.2, initial tradition chip + Aristotelian + pramāṇa + masādir + Polanyi placeholder).
- **MIG-026** (2026-05-17 → 2026-05-19) — expansion: Phase 0 (`register` → `tradition` rename) + Phase α (multi-shape `TraditionModule` foundation) + Phase β (A3+A6 chip UI) + Phases γ → θ (19 new tradition modules + 9 shape renderers) + Phase ι (24 manifests + ⓘ disclosure) + Phase κ (Tier 1 JSON + Tier 2 JS plugin loader) + Phase λ (full 15-locale chrome + canvas localization) + Phase μ (ship gate + 3-agent audit). Closed at milestone tag `milestone/sight-v6.3-traditions-ship`.
- **MIG-027** (2026-05-17) — theme inheritance: ChromePalette / SEMANTIC_COLORS split + theme-conditional `--sight-highlight` variable family; Sight follows the active app theme without erasing semantic information.
- **MIG-028** (2026-05-19) — v5 retirement: cleanup migration that removes the v5 dual-mount surface and reclaims the surface area for v6.
- **MIG-030 / 031 / 032** (2026-05-18 → 2026-05-19) — polish and tests: `tradition-perf.test.ts` verifying the ≤16ms render budget; further chrome polish and i18n cleanup; v6.3 hardening for ship.

The Concept Paper for Sight has tracked the implementation in two version axes (per v4.1's header): the **Concept Paper version** moved v1.x → v2.0 → v3.0 → v3.1 → v4.0 → v4.1; the **implementation version** moved v2 → v3 → v4 → v5 → v6.0 → v6.1 → v6.2 → v6.3. The two axes are independent because Concept Paper edits often follow shipped code (v4.1 documents v6.3 after the fact) and shipped code sometimes outruns the Concept Paper (v6.0/v6.1/v6.2/v6.3 evolved inside what v4.0 had specified, with v4.1 the reconciliation).

---

## §11 — Future workstreams (post-v6.3)

These are reserved or deferred workstreams; none of them block the current ship. They are enumerated here so the architectural intent is on the record.

**MIG-029 — per-note frontmatter wiring for tradition-kind fields.** Architect-deferred. The per-tradition placement currently uses default heuristics (the v4.1 §10 #1 polish target). Once the Rust-side `LayoutCacheRow` is extended to carry per-note frontmatter fields like `pramana_kind`, `masadir_source`, `peirce_category`, etc., the renderer will override the default placement with the user's explicit declaration. Until shipped, the default placement applies; the surface is therefore correct but unable to reflect user intent at the per-note grain.

**MIG-033 — Wasm/QuickJS sandbox for Tier 2 TS plugin layer.** Architect-deferred (referenced in `lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md` §632). The current v6.3 Tier 2 plugin layer is Obsidian-trust: full-trust JavaScript, opt-in via consent banner. A Wasm or QuickJS sandbox would tighten the trust envelope so users could load Tier 2 plugins without granting full JS privileges. Deferred until there is concrete user demand and a clear cost/benefit; the consent gate handles the immediate trust ask.

**MIG-034 — v4.1 per-tradition internal-structure polish.** Reserved. Each tradition's current geometry is the locked v4.1 specification; future scholarly review may surface refinement opportunities (sector rotation, ring count, label rephrasing) that don't break the `TraditionShape` API but improve fidelity. These would land as a focused polish MIG.

**MIG-035 — federation cUniverse tradition behavior.** Reserved; design call needed. When a Universe has cUniverse-federated libraries, how should Sight render notes from those libraries? Options: (a) flatten into the parent dome and treat them indistinguishably (current behavior); (b) badge them so the user can see the federation distinction; (c) offer a per-cUniverse switch in the chip dropdown ("show only this Universe's notes" vs "show all federated notes"). The design call has not been opened.

**Tier 4 — abandoned.** Per Eisa's direction on 2026-05-19: the original Sight v5 vision's instrument-layers ambition — Layer 2 (diagnostic findings as surfaced text), Layer 3 (recommendation engine that suggests user actions), Layer 4 (coaching surface that walks the user through interpretation) — is retired. The v6 architecture re-scoped to Layer 1 and the diagnostic *reading* of Layer 1 (the user interprets the surface; the subsystem does not interpret on the user's behalf). The Layer 2–4 surface area is no longer in Sight's roadmap.

---

## §12 — Internal-design contract (cross-reference)

This subsystem paper defines *what* Sight is in Constellation and *how* it composes with the surrounding architecture. The *internal* design — the renderer pipeline's per-shape draw helpers, the channel-orthogonality invariant's per-Bertin-variable map, the per-tradition scholarly contracts and citations, the cache schema's per-column semantics, the gesture grammar's per-event dispatch table — lives in `docs/Constellation-Sight-Concept-Paper-v4.1.md`. That document is Sight's **internal-design contract**; it specifies the implementation that this paper points at.

The relationship between the two papers, plainly stated. The **subsystem paper** (this document) answers: "What is Sight inside Constellation? Where does it sit? What does it depend on? What depends on it? What invariants does it hold?" The **internal-design paper** (v4.1) answers: "How is Sight built? What are its rendering primitives? What are its 24 traditions in scholarly detail? What are its 9 shape renderers? What are the visual encodings?" A new contributor reading this paper learns where Sight fits; a contributor preparing to modify Sight's renderer should then read v4.1 for the implementation contract. A user trying to understand what Sight does for them in a Constellation workflow reads this paper alone. A reviewer auditing Sight's architectural invariants reads §9 of this paper and v4.1 §11 together — they are the same invariant set, expressed at two grain levels (architectural in §9 here, implementational in v4.1 §11).

The two papers are kept in sync structurally but not in word-for-word duplication. Where this paper cites a specific encoding, geometry, or invariant, the citation form is `v4.1 §X.Y`; the canonical statement is in v4.1, the architectural placement is here. When v4.1 is bumped to v4.2 or v5.0, this paper's version axis advances independently as the subsystem-level architecture changes (which may or may not coincide with internal-design changes).

---

*End of Subsystem Concept Paper v1.0.*

*Cut 2026-05-19 alongside Concept Paper v4.1 (the internal-design contract) and the milestone tag `milestone/sight-v6.3-traditions-ship`. Future revisions either ship as v1.1 alongside this file (subsystem-level architectural changes that don't change the implementation contract) or as v2.0 (subsystem-level architectural changes that do).*
