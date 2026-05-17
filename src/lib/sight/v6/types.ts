/**
 * MIG-025 §A.6 — Sight v6 frontend type contracts.
 *
 * Mirrors `src-tauri/src/sight_v6.rs` Serde structs (camelCase
 * serialization). The Rust side is the source of truth for cache
 * row + link edge shapes; the contracts in this file just rename
 * for TypeScript ergonomics.
 *
 * No mode/scope union types from v5 (Sight v6 has no modes; scope
 * filtering is sidebar-driven, not a typed enum).
 *
 * References:
 *   docs/Constellation-Sight-Concept-Paper-v4.0.md  (§2, §3, §4, §5)
 *   src-tauri/src/sight_v6.rs                       (Rust types)
 *   docs/sight-redesign-v0.3-full-layout.svg        (visual contract)
 */

// ════════════════════════════════════════════════════════════════════
// Cache row + link edge — Rust IPC outputs
// ════════════════════════════════════════════════════════════════════

/**
 * Per-note layout cache row returned by `sight_v6_get_layout`.
 * Fields match `sight_v6::LayoutCacheRow` Serde camelCase output.
 *
 * Optional fields are nullable in SQLite; the renderer treats
 * null-stratum notes as Edge-of-Knowing rim residents per §A.4
 * tier-5 sweep.
 */
export interface LayoutCacheRow {
	notePath: string;
	stratum: number | null;
	maturity: string | null;
	confidenceAlpha: number | null;
	contested: boolean;
	libraryName: string | null;
	folderPath: string | null;
	createdMonth: number | null;
	sourcesPrimary: string | null;
	stage: string | null;
	actsPrimary: string | null;
	dominantLinkType: string | null;
	computedAt: number;
	// v6 additions (Architect §1.2):
	linkInCount: number;
	linkOutCount: number;
	frontmatterKeyCount: number;
	bodyChars: number;
}

/**
 * Typed-link edge between two visible notes — used by the anchor
 * dome's connector-line rendering (§A.9). `confidence` is one of
 * 'hypothesis' | 'evidence' | 'established' | 'contested';
 * `linkType` is one of the 9 typed-link kinds.
 */
export interface LinkEdge {
	sourcePath: string;
	targetPath: string;
	linkType: TypedLinkKind;
	confidence: ConfidenceLevel;
}

// ════════════════════════════════════════════════════════════════════
// Vocabulary — locked in Concept Paper v4.0 §3
// ════════════════════════════════════════════════════════════════════

/** Confidence levels (Concept Paper §3.4 + the Living Link Architecture). */
export type ConfidenceLevel = 'hypothesis' | 'evidence' | 'established' | 'contested';

/** The 9 typed-link kinds (Concept Paper §3.4 line palette). */
export type TypedLinkKind =
	| 'supports'
	| 'contradicts'
	| 'causes'
	| 'exemplifies'
	| 'generalizes'
	| 'derives-from'
	| 'part-of'
	| 'associative'
	| 'supersedes';

/**
 * Lifecycle stages (Concept Paper §3.4 stage palette). Carried from
 * the Living Link Architecture; encoded as full-disk hue in the
 * Stage mini-dome and as the inner pip on the anchor dome.
 */
export type LifecycleStage =
	| 'established'
	| 'fresh'
	| 'growing'
	| 'at-risk'
	| 'dormant';

/**
 * Provenance source buckets (Concept Paper §2.4). Five visible
 * sectors in the Provenance mini-dome; also a facet sidebar
 * category. Mapping from sourcesPrimary string → bucket happens
 * in JS via this lookup.
 */
export type ProvenanceSector =
	| 'Self'
	| 'Read'
	| 'Heard'
	| 'Reasoned'
	| 'Tradition';

// ════════════════════════════════════════════════════════════════════
// Facet sidebar (Concept Paper §2.3, §3.2)
// ════════════════════════════════════════════════════════════════════

/** Facet identifiers — the 6 sidebar groups. Folder is TOP per §11 invariant 8. */
export type FacetId =
	| 'folder'
	| 'library'
	| 'stratum'
	| 'confidence'
	| 'stage'
	| 'provenance';

/** A single category within a facet (e.g., "Library: Research" with count 3,124). */
export interface FacetCategory {
	id: string;          // stable category key (e.g., 'Research', 'established', '3')
	label: string;       // user-visible label (may differ from id for i18n)
	count: number;       // live count after current cross-filter set
}

/** A facet group as rendered in the sidebar. */
export interface Facet {
	id: FacetId;
	label: string;
	categories: FacetCategory[];
}

// ════════════════════════════════════════════════════════════════════
// Mini-dome channels (Concept Paper §2.3)
// ════════════════════════════════════════════════════════════════════

/**
 * Each mini-dome isolates one channel. Per §7 (Western-analytic
 * stipulation), these channel labels are constant across all
 * scholarly traditions in v1 — the tradition chip remaps the
 * anchor dome's spatial semantics only.
 */
export type MiniDomeChannel =
	| 'confidence'
	| 'stage'
	| 'acts'
	| 'provenance';

/**
 * §B.6-fix-3 (2026-05-15) — broader slot identifier covering the
 * 4 mini-dome channels PLUS 'anchor' (the universe-as-cream-stars
 * baseline view). Used by the dome-swap layout in SightV6.svelte
 * where any of the 5 slots can occupy the primary canvas at any
 * time. Distinct from MiniDomeChannel because cross-filter category
 * gestures (`filter-mini-dome-category`) and the facet sidebar
 * specifically exclude 'anchor' — only the channel mini-domes have
 * categorical filterable buckets.
 */
export type SlotChannel = MiniDomeChannel | 'anchor';

// ════════════════════════════════════════════════════════════════════
// Tradition chip (Concept Paper §4 — renamed in MIG-026 Phase 0 / K1)
// ════════════════════════════════════════════════════════════════════
//
// MIG-026 Phase 0 — K1 full rename: "register" → "tradition" throughout
// (per Eisa's locked Direction A+D and the architect §3.K choice).
// The historical commit chain still uses "register" in MIG-025-era
// commits (§C.1-fix-1, §C.4-religious-rule); migration block in
// store.ts rewrites persisted `activeRegister` → `activeTradition`.

/**
 * The 5 baseline scholarly traditions shipping in v6.2 (and through
 * MIG-026 Phase 0 rename). Aristotelian is default; pramāṇa/masādir/
 * Polanyi are production polish (4.1.x); Mohist is v1-preview, polish
 * target v4.1 per Concept Paper §4.2. MIG-026 Phases γ–θ add 19 more
 * to bring the curated baseline to 24 traditions.
 *
 * §C.1-fix-1 (Eisa 2026-05-16): Dignāga tradition EXCLUDED entirely
 * per Eisa's direction "don't include the 'Dignāga' at all in any
 * of Constellation functions". The 'dignaga' literal is not in this
 * union.
 *
 * §C.4-religious-rule (Eisa 2026-05-16): Suhrawardi Ishrāqī tradition
 * also EXCLUDED per the top-principal religious-lineage rule
 * (orientation v2.09): non-Abrahamic religious-source frames are
 * out; for Islamic, Sunni-only. The 'ishraqi' literal is not in
 * this union.
 *
 * Settings migrations in store.ts applyParsedSettings rewrite
 * persisted 'dignaga' and 'ishraqi' values back to 'aristotelian'
 * (plus the MIG-026 Phase 0 rename block rewrites 'activeRegister'
 * → 'activeTradition'). Concept Paper §4.2.1 (Dignāga) and §4.2.2
 * (Ishrāqī) carry EXCLUDED notes.
 */
export type TraditionId =
	| 'aristotelian'
	| 'pramana'
	| 'masadir'
	| 'polanyi'
	| 'mohist-san-biao'
	| 'peirce'
	| 'habermas'
	| 'dewey'
	| 'husserl'
	| 'longino'
	| 'ibn-rushd-burhan'
	| 'shatibi-maqasid'
	| 'ibn-khaldun-umran';

// ════════════════════════════════════════════════════════════════════
// Tradition module contract (Concept Paper §4 + Plan §C.2)
// ════════════════════════════════════════════════════════════════════

/**
 * Dome layout passed to tradition callbacks — centerX/centerY are the
 * pixel center of the anchor dome in world-space (un-zoomed), radius
 * is the outer-rim radius. Same shape as anchor.DomeLayout but
 * referenced here to keep traditions/* free of upstream imports.
 */
export interface TraditionLayout {
	centerX: number;
	centerY: number;
	radius: number;
}

/**
 * Tradition geometric shape discriminator (MIG-026 Phase α).
 *
 * Each tradition declares its `shape` so the anchor renderer can
 * dispatch to the right per-shape renderer. New shapes added in
 * MIG-026 cover the 19 newcomer traditions:
 *
 * - `sectoral`         — 4-or-N pie quadrants (Aristotelian as identity,
 *                        pramāṇa 4 quadrants, masādir 4 sectors,
 *                        Mencian 4 sprouts, Peirce 3, Habermas 3,
 *                        Longino 4, Akan Wiredu 2-3)
 * - `rings`            — concentric ring boundaries (Ibn Rushd burhān
 *                        ladder 4, PaRDeS 4, Maldonado-Torres 3,
 *                        Husserl regional ontologies central+petals)
 * - `grid`             — 2D sectoral × ring (Shāṭibī maqāṣid 3×5,
 *                        Korean Sŏngnihak 2×2)
 * - `ladder`           — N-step ladder (D3 spiral by Eisa pick;
 *                        Maimonidean prophecy 11, Talmudic 13)
 * - `relational`       — node-link / network (E3 hub-and-spoke by
 *                        Eisa pick; Mignolo pluriversal, Ibuanyidanda)
 * - `cyclic-flow`      — N-segment ring + arrow flow (Dewey inquiry 5)
 * - `binary-flow`      — 2-cell + directional flow (Dussel transmodernity,
 *                        Ibn Khaldūn ʿumrān, Wang Yangming)
 * - `gradient`         — continuous opacity gradient (Polanyi tacit-
 *                        explicit fog)
 * - `horizontal-bands` — N horizontal zones (Mohist sān biǎo 3)
 *
 * Phase 0 + α preserve the existing 3 modules as `shape: 'sectoral'`.
 * Phase γ adds Polanyi (`gradient`) + Mohist (`horizontal-bands`).
 * Phases δ–θ add the 19 newcomers across the remaining shapes.
 */
export type TraditionShape =
	| 'sectoral'
	| 'rings'
	| 'grid'
	| 'ladder'
	| 'relational'
	| 'cyclic-flow'
	| 'binary-flow'
	| 'gradient'
	| 'horizontal-bands';

/**
 * Optional sector divider stroke spec returned by a tradition module's
 * sectorDividers() callback. Used by sectoral-shaped traditions
 * (pramāṇa, masādir, Mencian, Peirce, etc.). For Aristotelian (which
 * is `shape: 'sectoral'` with identity remap) the dividers are absent
 * — the 5 concentric stratum bands are the visual structure, drawn
 * by dome.ts not by the tradition.
 *
 * Angles are in canvas math convention: 0 = EAST, increases clockwise
 * (since canvas y-axis is inverted), measured in radians.
 */
export interface SectorSpec {
	angleStart: number;
	angleEnd: number;
	label?: string;
}

/**
 * MIG-026 Phase α — Concentric ring boundary spec for `shape: 'rings'`
 * (and the ring component of `shape: 'grid'`). Used by Ibn Rushd
 * burhān ladder (Phase ε.1), PaRDeS (Phase ζ.1), Maldonado-Torres
 * (Phase θ.3), Husserl regional ontologies (Phase δ.2).
 *
 * `radiusFrac` is the ring boundary expressed as a fraction of the
 * outer dome radius (0.0 = center, 1.0 = outer rim). Boundaries are
 * drawn as concentric arcs at these fractions. Labels are placed at
 * the midpoint of the ring annulus along a chosen radial spoke
 * (renderer decides — typically along the +y axis for legibility).
 */
export interface RingSpec {
	radiusFrac: number;
	label?: string;
}

/**
 * MIG-026 Phase α — Ladder step spec for `shape: 'ladder'`. Used by
 * Maimonidean prophecy (Phase ζ.2 — D3 spiral, 11 steps) and Talmudic
 * 13 middot (Phase ζ.3 — spiral, 13 steps).
 *
 * Per Eisa's locked D3 choice in Architect §3.D: ladder is rendered
 * as a logarithmic spiral from center to outer rim, with N step-marks
 * along the spiral. The `variant` discriminator allows future ladder
 * shapes (stack, fan) without breaking the interface.
 *
 * Phase ζ.2 spike implements the spiral path math (likely
 * equiangular: r(θ) = a·exp(b·θ) where a and b are derived from N).
 */
export interface LadderSpec {
	variant: 'spiral' | 'stack';
	steps: { label: string }[];
}

/**
 * MIG-026 Phase α — Relational/network spec for `shape: 'relational'`.
 * Used by Mignolo pluriversal (Phase θ.1 — hub-and-spoke) and
 * Ibuanyidanda (Phase θ.5 — complementary hub-and-spoke).
 *
 * Per Eisa's locked E3 choice in Architect §3.E: relational shape
 * is rendered as a central disc (hub) + N outer clusters (spokes),
 * with lines connecting each cluster to the hub. The `variant`
 * discriminator allows future relational shapes (force-directed,
 * radial-tree) without breaking the interface.
 *
 * For Mignolo: hub = "modernity/totality"; clusters = subaltern
 * positions (e.g., Andean / Yoruba / Māori).
 * For Ibuanyidanda: hub = "missing link"; every entity-cluster
 * connects to it (complementary ontology).
 */
export interface RelationalSpec {
	variant: 'hub-and-spoke';
	hubLabel: string;
	clusters: { label: string }[];
}

/**
 * MIG-026 Phase α — Cyclic-flow spec for `shape: 'cyclic-flow'`. Used
 * by Dewey's pattern of inquiry (Phase δ.2 — 5-segment ring with
 * chronological arrow flow).
 *
 * Renderer draws N segments arranged in a ring + curved arrow flow
 * indicating sequence direction. Sequence wraps (segment N → segment 1).
 */
export interface CyclicFlowSpec {
	segments: { label: string }[];
}

/**
 * MIG-026 Phase α — Binary-flow spec for `shape: 'binary-flow'`. Used
 * by Dussel transmodernity (Phase θ.2), Ibn Khaldūn ʿumrān (Phase
 * ε.3), Wang Yangming (Phase η.2).
 *
 * Two cells (typically inner/outer disc, or left/right hemispheres)
 * with a directional flow arrow between them. The arrow direction is
 * tradition-specific (Dussel: exteriority → totality / analectic;
 * Ibn Khaldūn: badawī → ḥaḍarī → badawī cyclic; Wang Yangming:
 * zhī ↔ xíng bidirectional via central liángzhī).
 */
export interface BinaryFlowSpec {
	cellA: { label: string };
	cellB: { label: string };
	flowDirection: 'a-to-b' | 'b-to-a' | 'bidirectional' | 'cyclic';
	centerLabel?: string;
}

/**
 * MIG-026 Phase α — Gradient spec for `shape: 'gradient'`. Used by
 * Polanyi (Phase γ — tacit/explicit fog gradient).
 *
 * Renderer applies an opacity gradient across the dome based on
 * radial distance from center. `centerOpacity` typically 0.14–0.18
 * (tacit core, intentionally low so stars there read as
 * "acknowledged but inarticulable"); `edgeOpacity` typically
 * 0.85–0.95 (explicit periphery, clearly readable).
 */
export interface GradientSpec {
	centerOpacity: number;
	edgeOpacity: number;
	centerLabel?: string;
	edgeLabel?: string;
}

/**
 * MIG-026 Phase α — Horizontal bands spec for `shape: 'horizontal-bands'`.
 * Used by Mohist sān biǎo (Phase γ — 3 horizontal zones: 本/root,
 * 原/origin, 用/use).
 *
 * Renderer divides the dome into N horizontal bands (top to bottom)
 * with band labels. Stars are placed in their band by frontmatter
 * field (default to the topmost band if absent).
 */
export interface HorizontalBandsSpec {
	bands: { label: string }[];
}

/**
 * Tradition module contract — each of the curated baseline traditions
 * exports one of these via `src/lib/sight/v6/traditions/<id>.ts`.
 *
 * Per Concept Paper §4.3 + §7 + §11 invariant 6: traditions remap the
 * anchor dome's spatial semantics ONLY; mini-domes stay culturally
 * neutral. This is the architectural commitment that prevents
 * rhetorical pluralism — see the mini-dome stipulation in §7.
 *
 * MIG-025 §C.2 shipped the module pattern + aristotelian (identity
 * remap, sectoral). §C.3 shipped pramāṇa (sectoral); §C.4 masādir
 * (sectoral). MIG-026 Phase α (this commit) extends the interface
 * with the `shape` discriminator + shape-specific optional spec
 * callbacks. Phase γ adds Polanyi (`gradient`) + Mohist sān biǎo
 * (`horizontal-bands`); Phases δ–θ add the 19 newcomers across
 * the remaining shapes.
 */
export interface TraditionModule {
	id: TraditionId;
	/** English brand label (matches the chip label in traditionChip.svelte). */
	name: string;
	/**
	 * MIG-026 Phase α — geometric shape discriminator. The anchor
	 * renderer dispatches to the right per-shape renderer based on
	 * this field. Each value gates which of the optional spec
	 * callbacks below is meaningful for this tradition.
	 */
	shape: TraditionShape;
	/**
	 * Given a star's row data and its default Aristotelian position,
	 * return the position under this tradition. For Aristotelian, this
	 * is identity. For pramāṇa, this redistributes to 4 quadrants by
	 * row.pramana_kind frontmatter field (§C.3). Etc.
	 *
	 * Implementations MUST be deterministic per (row, defaultPos) — the
	 * anchor renderer calls this once per star per recompute, and the
	 * hit-test relies on the same x/y being returned for the same row
	 * on a subsequent call.
	 */
	remapStarPosition(
		row: LayoutCacheRow,
		defaultPos: { x: number; y: number },
		layout: TraditionLayout,
	): { x: number; y: number };
	/**
	 * Optional sector divider strokes for `shape: 'sectoral'` or
	 * `shape: 'grid'` traditions. Drawn by anchor.ts's
	 * drawSectorDividers helper.
	 */
	sectorDividers?: (layout: TraditionLayout) => SectorSpec[];
	/**
	 * MIG-026 Phase α — Optional ring boundary strokes for
	 * `shape: 'rings'` or `shape: 'grid'` traditions. Drawn by
	 * anchor.ts's drawRingBoundaries helper (stub in Phase α; filled
	 * in Phase ε.1 for the first ring-shape tradition).
	 */
	ringBoundaries?: (layout: TraditionLayout) => RingSpec[];
	/**
	 * MIG-026 Phase α — Optional ladder spec for `shape: 'ladder'`
	 * traditions. Drawn by anchor.ts's drawLadderSteps helper (stub
	 * in Phase α; filled in Phase ζ.2 — Maimonidean spiral spike).
	 */
	ladderSteps?: (layout: TraditionLayout) => LadderSpec;
	/**
	 * MIG-026 Phase α — Optional relational spec for `shape:
	 * 'relational'` traditions. Drawn by anchor.ts's drawRelationalGraph
	 * helper (stub in Phase α; filled in Phase θ.1 — Mignolo hub-and-
	 * spoke spike).
	 */
	relationalSpec?: (layout: TraditionLayout) => RelationalSpec;
	/**
	 * MIG-026 Phase α — Optional cyclic-flow spec for `shape:
	 * 'cyclic-flow'` traditions. Drawn by anchor.ts's drawCyclicFlow
	 * helper (stub in Phase α; filled in Phase δ.2 — Dewey inquiry).
	 */
	cyclicFlowSpec?: (layout: TraditionLayout) => CyclicFlowSpec;
	/**
	 * MIG-026 Phase α — Optional binary-flow spec for `shape:
	 * 'binary-flow'` traditions. Drawn by anchor.ts's drawBinaryFlow
	 * helper (stub in Phase α; filled in Phase ε.3 — Ibn Khaldūn
	 * ʿumrān).
	 */
	binaryFlowSpec?: (layout: TraditionLayout) => BinaryFlowSpec;
	/**
	 * MIG-026 Phase α — Optional gradient spec for `shape: 'gradient'`
	 * traditions. Drawn by anchor.ts's drawGradientFog helper (stub
	 * in Phase α; filled in Phase γ — Polanyi).
	 */
	gradientSpec?: (layout: TraditionLayout) => GradientSpec;
	/**
	 * MIG-026 Phase α — Optional horizontal-bands spec for `shape:
	 * 'horizontal-bands'` traditions. Drawn by anchor.ts's
	 * drawHorizontalBands helper (stub in Phase α; filled in Phase γ
	 * — Mohist sān biǎo).
	 */
	horizontalBandsSpec?: (layout: TraditionLayout) => HorizontalBandsSpec;
	/**
	 * Optional extension chips rendered below the anchor dome (e.g.,
	 * masādir's 4 supplementary sources: istiḥsān, istiṣḥāb, maṣlaḥa
	 * mursalah, ʿurf, per Concept Paper §4.1.3). Independent of
	 * `shape` — any tradition can carry extension chips.
	 *
	 * Return an array of plain string labels (kept English/transliterated
	 * per the brand convention). The SightV6 host renders them as a row
	 * of small HTML badges below the anchor canvas. Omit (or return [])
	 * for traditions with no extension category — Aristotelian, pramāṇa,
	 * Polanyi, Mohist sān biǎo all omit this.
	 */
	extensionChips?: () => string[];
}

// ════════════════════════════════════════════════════════════════════
// Gesture grammar (Concept Paper §5)
// ════════════════════════════════════════════════════════════════════

/**
 * The progressive-disclosure gesture vocabulary. No persistent
 * toggle bars per §11 invariant 9. Each gesture is dispatched as
 * an event with this discriminated union shape.
 */
export type GestureEvent =
	| { kind: 'expand-sidebar' }
	| { kind: 'expand-tradition-chip' }
	| { kind: 'show-diagnostics' }              // Cmd-D / button
	| { kind: 'pro-mode-toggle' }                // Cmd-Shift-D persistent
	| { kind: 'isolate-stratum'; stratum: number }
	| { kind: 'filter-facet'; facet: FacetId; categoryId: string }
	| { kind: 'filter-mini-dome-category'; channel: MiniDomeChannel; categoryId: string }
	| { kind: 'select-tradition'; tradition: TraditionId }
	| { kind: 'hover-star'; notePath: string | null }
	| { kind: 'click-star'; notePath: string }
	| { kind: 'reset' }                          // Esc
	| { kind: 'open-search' }                    // Cmd-F
	| { kind: 'open-tour' };                     // Help → Sight tour

// ════════════════════════════════════════════════════════════════════
// Star derived view (computed from LayoutCacheRow + Universe context)
// ════════════════════════════════════════════════════════════════════

/**
 * Per-star derived rendering data. Computed in JS at render time
 * from `LayoutCacheRow` + Universe-wide context (acts distribution
 * for top-decile flag, library ordering for shape index, sources
 * mapping for provenance sector). Not serialized; not stored.
 */
export interface StarDerived {
	row: LayoutCacheRow;
	libraryShapeIndex: number;        // 0..4 → circle, square, diamond, triangle, hexagon
	topDecileActs: boolean;           // size +40% if true
	provenanceSector: ProvenanceSector | null;
	x: number;                        // canvas x
	y: number;                        // canvas y
}
