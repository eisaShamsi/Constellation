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
 * stipulation), these channel labels are constant across all 7
 * registers in v1 — the register chip remaps the anchor dome's
 * spatial semantics only.
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
// Register chip (Concept Paper §4)
// ════════════════════════════════════════════════════════════════════

/**
 * The 6 epistemic registers in v1. Aristotelian is default;
 * pramāṇa/masādir/Polanyi are production polish (4.1.x) per Eisa's
 * polish-tiered decision; Ishrāqī/Mohist are v1-preview labeled,
 * polish target v4.1 per Concept Paper §4.2.
 *
 * §C.1-fix-1 (Eisa 2026-05-16): Dignāga register EXCLUDED entirely
 * per Eisa's direction "don't include the 'Dignāga' at all in any
 * of Constellation functions". The 'dignaga' literal is removed
 * from this union. A settings migration in store.ts applyParsedSettings
 * rewrites any persisted activeRegister: 'dignaga' value back to
 * 'aristotelian' for users who selected it during §C.1 testing.
 * Concept Paper §4.2.1 and Plan §D.1 both carry EXCLUDED notes.
 */
export type RegisterId =
	| 'aristotelian'
	| 'pramana'
	| 'masadir'
	| 'polanyi'
	| 'ishraqi'
	| 'mohist-san-biao';

// ════════════════════════════════════════════════════════════════════
// Register module contract (Concept Paper §4 + Plan §C.2)
// ════════════════════════════════════════════════════════════════════

/**
 * Dome layout passed to register callbacks — centerX/centerY are the
 * pixel center of the anchor dome in world-space (un-zoomed), radius
 * is the outer-rim radius. Same shape as anchor.DomeLayout but
 * referenced here to keep registers/* free of upstream imports.
 */
export interface RegisterLayout {
	centerX: number;
	centerY: number;
	radius: number;
}

/**
 * Optional sector divider stroke spec returned by a register module's
 * sectorDividers() callback. Used by pramāṇa (4 quadrants, §C.3),
 * masādir (4 sectors, §C.4), etc. For Aristotelian (§C.2) the
 * dividers are absent — the 5 concentric stratum bands are the visual
 * structure, drawn by dome.ts not by the register.
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
 * Register module contract — each of the 6 epistemic registers
 * (aristotelian, pramana, masadir, polanyi, ishraqi, mohist-san-biao)
 * exports one of these via `src/lib/sight/v6/registers/<id>.ts`.
 *
 * Per Concept Paper §4.3 + §7 + §11 invariant 6: registers remap the
 * anchor dome's spatial semantics ONLY; mini-domes stay culturally
 * neutral. This is the architectural commitment that prevents
 * rhetorical pluralism — see the mini-dome stipulation in §7.
 *
 * MIG-025 §C.2 ships the module pattern + aristotelian (identity
 * remap). §C.3 ships pramāṇa; §C.4 masādir; §C.5 Polanyi; §D.2
 * Suhrawardi Ishrāqī; §D.3 Mohist sān biǎo.
 */
export interface RegisterModule {
	id: RegisterId;
	/** English brand label (matches the chip label in registerChip.svelte). */
	name: string;
	/**
	 * Given a star's row data and its default Aristotelian position,
	 * return the position under this register. For Aristotelian, this
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
		layout: RegisterLayout,
	): { x: number; y: number };
	/**
	 * Optional sector divider strokes to draw on the anchor under this
	 * register (e.g., pramāṇa's 4 quadrant dividers in §C.3). Called
	 * once per paint with the current layout. Return [] or omit for
	 * registers with no sector structure (Aristotelian, Polanyi).
	 */
	sectorDividers?: (layout: RegisterLayout) => SectorSpec[];
	/**
	 * Optional extension chips rendered below the anchor dome (e.g.,
	 * masādir's 4 supplementary sources: istiḥsān, istiṣḥāb, maṣlaḥa
	 * mursalah, ʿurf, per Concept Paper §4.1.3). These are visual
	 * reminders that the register's full epistemic vocabulary includes
	 * supplementary categories beyond the 4 main sectors — not
	 * separate dome regions, but additional categories the user can
	 * tag notes with (post-§C.4-fix-N when Rust-side extraction lands).
	 *
	 * Return an array of plain string labels (kept English/transliterated
	 * per the brand convention). The SightV6 host renders them as a row
	 * of small HTML badges below the anchor canvas. Omit (or return [])
	 * for registers with no extension category — Aristotelian, pramāṇa,
	 * Polanyi, Ishrāqī, Mohist sān biǎo all omit this.
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
	| { kind: 'expand-register-chip' }
	| { kind: 'show-diagnostics' }              // Cmd-D / button
	| { kind: 'pro-mode-toggle' }                // Cmd-Shift-D persistent
	| { kind: 'isolate-stratum'; stratum: number }
	| { kind: 'filter-facet'; facet: FacetId; categoryId: string }
	| { kind: 'filter-mini-dome-category'; channel: MiniDomeChannel; categoryId: string }
	| { kind: 'select-register'; register: RegisterId }
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
