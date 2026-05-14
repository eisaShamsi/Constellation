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

// ════════════════════════════════════════════════════════════════════
// Register chip (Concept Paper §4)
// ════════════════════════════════════════════════════════════════════

/**
 * The 7 epistemic registers in v1. Aristotelian is default;
 * pramāṇa/masādir/Polanyi are production polish (4.1.x) per Eisa's
 * 7-in-v1 + polish-tiered decision; Dignāga/Ishrāqī/Mohist are
 * v1-preview labeled, polish target v4.1 per Concept Paper §4.2.
 */
export type RegisterId =
	| 'aristotelian'
	| 'pramana'
	| 'masadir'
	| 'polanyi'
	| 'dignaga'
	| 'ishraqi'
	| 'mohist-san-biao';

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
