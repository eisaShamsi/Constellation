/**
 * Sight v5 — TypeScript type contracts.
 *
 * Per Concept Paper v3.1 §5–§7, Sight v5 is a four-layer analytical
 * instrument anchored on a stable star chart. This module declares the
 * type vocabulary every v5 module shares: the seven modes, the three
 * scopes, the per-star encoding, the per-mode wedge bucket, and the
 * SQLite layout cache row shape. Pure types — no runtime behavior.
 *
 * MIG-024 §1 (skeleton). Real implementations land in §2 (cache),
 * §3 (dome render), §4 (mode + scope dispatch), §5 (interactivity).
 */

/** The seven Sight v5 modes per Concept Paper v3.1 §6.
 *  - R Regions   — wedge by Library
 *  - L Link Types — wedge by dominant outgoing typed-link kind
 *  - T Time      — wedge by creation month
 *  - C Confidence — wedge by per-note primary link confidence
 *  - S Stages    — wedge by lifecycle stage
 *  - A Acts      — wedge by which Act produced the note
 *  - P Provenance — wedge by primary horizontal-axis ID from CECE's live taxonomy
 */
export type SightV5Mode = 'R' | 'L' | 'T' | 'C' | 'S' | 'A' | 'P';

/** The three Sight v5 scopes per D-V3 (Eisa, 2026-05-12).
 *  Filters the visible note set BEFORE wedge computation.
 *  - universe — all notes in the active Universe (default)
 *  - library  — notes within the current sidebar Library context
 *  - folder   — notes within the current sidebar Folder context
 */
export type SightV5Scope = 'universe' | 'library' | 'folder';

/** Per-star encoding read from the layout cache. The four constants
 *  (radial position, size, brightness, color) are derived from this
 *  shape; they NEVER change with mode toggle (Concept Paper §7). */
export interface Star {
	notePath: string;
	stratum: number;            // 1..8 — radial position (constant)
	maturity: 'seed' | 'sapling' | 'evergreen' | 'canonical' | 'wilting';
	confidenceAlpha: number;    // 0.45 (hypothesis) | 0.7 (evidence) | 1.0 (established)
	contested: boolean;         // true → red dot
	libraryName: string;        // matches Rust LayoutCacheRow.library_name via serde camelCase rename
	folderPath: string;
	createdMonth: number;       // 0..11 (used by mode T)
	sourcesPrimary: string | null;  // top-level horizontal-axis family (mode P), or null = Unsourced
	stage: string | null;       // lifecycle stage (mode S)
	actsPrimary: string | null; // per-note Act tag (mode A)
	dominantLinkType: string | null; // mode L
}

/** A wedge bucket — the unit of rim slicing. Each mode produces its
 *  own bucket set; stars are assigned to a bucket via `azimuthForMode`. */
export interface Wedge {
	key: string;                // bucket identifier (library path, month index, source id, ...)
	label: string;              // user-facing label (locale-aware where applicable)
	count: number;              // notes in this wedge after scope filter
	azimuthStart: number;       // radians; 0 = top (north), clockwise
	azimuthEnd: number;
}

/** Context the per-mode dispatcher needs. Read once per render frame;
 *  mutated only on scope/mode change, not per star. */
export interface ModeContext {
	mode: SightV5Mode;
	scope: SightV5Scope;
	scopeId: string | null;     // library_path | folder_path | null (for universe)
	wedges: Wedge[];            // current mode's bucket set (computed by wedgeBucketsForMode)
	currentMonth: number;       // 0..11 — used to highlight the current month wedge in mode T
}

/** SQLite layout cache row shape (§2). One row per note per
 *  (library_set_hash) — D-V4 locked the per-note × 1 strategy; per-mode
 *  reprojection happens in JS at render time.
 *
 *  Field names match the Rust LayoutCacheRow serde output (camelCase
 *  via #[serde(rename_all = "camelCase")]). Optional fields use
 *  TypeScript `| null` — Rust Option<T> serializes to null when None. */
export interface LayoutCacheRow {
	notePath: string;
	stratum: number | null;
	maturity: string | null;
	confidenceAlpha: number | null;
	contested: boolean;         // Rust converts SQLite 0/1 → bool at the IPC boundary
	libraryName: string | null;
	folderPath: string | null;
	createdMonth: number | null;
	sourcesPrimary: string | null;
	stage: string | null;
	actsPrimary: string | null;
	dominantLinkType: string | null;
	computedAt: number;         // Unix epoch ms
}

/** A typed-link edge between two notes — read by §5 connector-line
 *  rendering. Color is mapped per Concept Paper §5.4 (9 typed-link
 *  kinds + supersedes slate-blue + associative cool-grey).
 *
 *  Field names match Rust LinkEdge serde output (camelCase). */
export interface LinkEdge {
	sourcePath: string;
	targetPath: string;
	linkType: string;           // 9 typed kinds + 'untyped' (validation at render time)
	confidence: string;         // 4 levels (hypothesis | evidence | established | contested)
}
