/**
 * Sight v5 — per-mode azimuth dispatch (§4).
 *
 * Each mode declares how it slices the rim into wedges. Stars are
 * positioned at (radiusForStratum, azimuthForMode) — strata stays the
 * radius (the four constants from §7), only the angle changes per
 * mode. This is what makes spatial memory survive mode toggles
 * (Concept Paper §6.2 + invariant I-11).
 *
 * MIG-024 §4 ships the dispatch tables + jitter logic. §5 puts the
 * results on the canvas (renders the actual stars at the computed
 * angles).
 */

import type { LayoutCacheRow } from './types';

/** A wedge bucket — the unit of rim slicing for a given mode.
 *  azimuthStart/azimuthEnd in radians; 0 = top (north), clockwise.
 *
 *  fix-9 (2026-05-13): adds optional `bgFill` for wedges that carry
 *  data-encoding background tint. Currently used by Mode S to mute
 *  Dormancy + Archival ("less alive" lifecycle phases per Eisa-
 *  confirmed §8.6 of mode-concepts). All other wedges leave bgFill
 *  null → standard parchment background. Refused for Mode P per
 *  civilizational pluralism principle (§10.6) — no source family
 *  gets visual differentiation. */
export interface WedgeBucket {
	key: string;             // bucket identifier (library name, month index, source family, ...)
	label: string;           // user-facing label
	count: number;           // notes in this bucket after scope filter
	azimuthStart: number;
	azimuthEnd: number;
	bgFill?: string | null;  // fix-9: optional muted background fill
}

/** Context the dispatch needs. Computed once per render frame on
 *  mode/scope change; not per-star. */
export interface ModeContext {
	wedges: WedgeBucket[];
	bucketKeyFor: (note: LayoutCacheRow) => string;
	/** fix-8 (2026-05-13): per-mode "data is present for this mode"
	 *  predicate. Returns true if the note has the data the active
	 *  mode classifies on (so the star renders SOLID). False if the
	 *  note is in the missing-data wedge for this mode (so the star
	 *  renders HOLLOW with the gold frame per §2 cascade priority 1). */
	hasModeData: (note: LayoutCacheRow) => boolean;
}

/** The 9 typed-link kinds (in canonical render order) + Untyped.
 *  Matches livePreview.ts TYPED_LINK_TYPES + an Untyped catch-all. */
const LINK_TYPE_ORDER = [
	'supports', 'contradicts', 'causes', 'exemplifies',
	'generalizes', 'derives-from', 'part-of', 'associative', 'supersedes',
	'untyped',
] as const;

/** 4 confidence buckets — fixed order (Concept Paper §6 mode C). */
const CONFIDENCE_ORDER = ['hypothesis', 'evidence', 'established', 'contested'] as const;

/** 6 lifecycle stage buckets (MIG-014). */
const STAGE_ORDER = ['Spark', 'Birth', 'Growth', 'Maturity', 'Dormancy', 'Archival'] as const;

/** 5 Acts + Unacted (CE Layer 2 / Concept Paper §6 mode A). */
const ACTS_ORDER = ['Observation', 'Connection', 'Tension', 'Synthesis', 'Conviction', 'Unacted'] as const;

/** 11 source families + Unsourced (Concept Paper §8.1 mode P). */
const SOURCE_FAMILIES = [
	'perception', 'inference', 'testimony', 'mass-transmission',
	'comparison', 'postulation', 'non-apprehension', 'memory',
	'innate-disposition', 'inspiration', 'revelation', 'unsourced',
] as const;

/** Build the mode context for a given mode + visible note set + locale.
 *  Returns wedge layout (positions + counts) AND a per-note bucket-key
 *  resolver that azimuthForMode uses to place individual stars. */
export function buildModeContext(
	mode: 'R' | 'L' | 'T' | 'C' | 'S' | 'A' | 'P',
	notes: LayoutCacheRow[],
	locale: string,
): ModeContext {
	switch (mode) {
		case 'R': return buildRegionsContext(notes);
		case 'L': return buildLinkTypesContext(notes);
		case 'T': return buildTimeContext(notes, locale);
		case 'C': return buildConfidenceContext(notes);
		case 'S': return buildStagesContext(notes);
		case 'A': return buildActsContext(notes);
		case 'P': return buildProvenanceContext(notes);
	}
}

/** Compute a star's azimuth (radians, 0 = north, clockwise) for the
 *  active mode context. The angle is the bucket's wedge center plus a
 *  small deterministic jitter (hash of note_path → fraction of wedge
 *  span) so stars within the same wedge don't all stack on the spoke. */
export function azimuthForMode(note: LayoutCacheRow, ctx: ModeContext): number {
	const key = ctx.bucketKeyFor(note);
	const bucket = ctx.wedges.find(w => w.key === key);
	if (!bucket) {
		// Note's bucket isn't in the wedge set (e.g., unrecognized
		// source family). Place at 0 = top of dome; the visible
		// jamming is the diagnostic.
		return -Math.PI / 2;
	}
	const span = bucket.azimuthEnd - bucket.azimuthStart;
	const jitter = pathHash01(note.notePath);
	return bucket.azimuthStart + span * jitter;
}

/** Deterministic [0, 1) hash of a note path. FNV-1a 32-bit / 2^32. */
function pathHash01(path: string): number {
	let h = 0x811c9dc5;
	for (let i = 0; i < path.length; i++) {
		h ^= path.charCodeAt(i);
		h = (h * 0x01000193) >>> 0;
	}
	return h / 0x100000000;
}

// ─── R Regions ─────────────────────────────────────────────────────
function buildRegionsContext(notes: LayoutCacheRow[]): ModeContext {
	const counts = new Map<string, number>();
	for (const n of notes) {
		const k = n.libraryName ?? '(unknown)';
		counts.set(k, (counts.get(k) ?? 0) + 1);
	}
	// Sort largest library first; wedge spans proportional to count.
	const sorted = Array.from(counts.entries()).sort((a, b) => b[1] - a[1]);
	const total = sorted.reduce((s, [, c]) => s + c, 0) || 1;
	const wedges: WedgeBucket[] = [];
	let cursor = -Math.PI / 2; // start at top
	for (const [key, count] of sorted) {
		const span = (count / total) * 2 * Math.PI;
		wedges.push({
			key,
			label: key,
			count,
			azimuthStart: cursor,
			azimuthEnd: cursor + span,
		});
		cursor += span;
	}
	return {
		wedges,
		bucketKeyFor: (n) => n.libraryName ?? '(unknown)',
		// Mode R: every note has a libraryName in practice → always solid.
		hasModeData: (n) => n.libraryName != null,
	};
}

// ─── L Link Types ──────────────────────────────────────────────────
function buildLinkTypesContext(notes: LayoutCacheRow[]): ModeContext {
	const counts = new Map<string, number>();
	for (const n of notes) {
		const k = n.dominantLinkType ?? 'untyped';
		counts.set(k, (counts.get(k) ?? 0) + 1);
	}
	// Use the canonical order (LINK_TYPE_ORDER) for stable layout.
	return makeUniformWedges(
		LINK_TYPE_ORDER.map(k => k as string),
		counts,
		(n) => n.dominantLinkType ?? 'untyped',
		(k) => k,
		(n) => n.dominantLinkType != null,    // hollow if untyped
	);
}

// ─── T Time ────────────────────────────────────────────────────────
function buildTimeContext(notes: LayoutCacheRow[], locale: string): ModeContext {
	const counts = new Map<string, number>();
	for (const n of notes) {
		const m = n.createdMonth ?? -1;
		const k = String(m);
		counts.set(k, (counts.get(k) ?? 0) + 1);
	}
	const fmt = new Intl.DateTimeFormat(locale, { month: 'short' });
	const monthKeys = Array.from({ length: 12 }, (_, i) => String(i));
	return makeUniformWedges(
		monthKeys,
		counts,
		(n) => String(n.createdMonth ?? -1),
		(k) => {
			const m = parseInt(k, 10);
			if (m < 0 || m > 11) return '?';
			return fmt.format(new Date(Date.UTC(2024, m, 15)));
		},
		(n) => n.createdMonth != null,        // hollow if no createdMonth
	);
}

// ─── C Confidence ──────────────────────────────────────────────────
function buildConfidenceContext(notes: LayoutCacheRow[]): ModeContext {
	const counts = new Map<string, number>();
	for (const n of notes) {
		const k = confidenceBucket(n);
		counts.set(k, (counts.get(k) ?? 0) + 1);
	}
	return makeUniformWedges(
		CONFIDENCE_ORDER.map(k => k as string),
		counts,
		confidenceBucket,
		(k) => k,
		(n) => n.confidenceAlpha != null || n.contested,    // hollow if no confidence data
	);
}
function confidenceBucket(n: LayoutCacheRow): string {
	if (n.contested) return 'contested';
	if (n.confidenceAlpha == null) return 'hypothesis';
	if (n.confidenceAlpha >= 0.95) return 'established';
	if (n.confidenceAlpha >= 0.6) return 'evidence';
	return 'hypothesis';
}

// ─── S Stages ──────────────────────────────────────────────────────
function buildStagesContext(notes: LayoutCacheRow[]): ModeContext {
	const counts = new Map<string, number>();
	for (const n of notes) {
		const k = n.stage ?? 'Spark';
		counts.set(k, (counts.get(k) ?? 0) + 1);
	}
	const ctx = makeUniformWedges(
		STAGE_ORDER.map(k => k as string),
		counts,
		(n) => n.stage ?? 'Spark',
		(k) => k,
		(n) => n.stage != null,            // hollow if unstaged
	);
	// fix-9 (2026-05-13): mute Dormancy + Archival wedge backgrounds to
	// encode "less alive" lifecycle phases (Eisa-confirmed §8.6 of mode-
	// concepts). Weathered cream `#f0e9cb` (~5-7% darker than parchment).
	for (const w of ctx.wedges) {
		if (w.key === 'Dormancy' || w.key === 'Archival') {
			w.bgFill = '#f0e9cb';
		}
	}
	return ctx;
}

// ─── A Acts ────────────────────────────────────────────────────────
function buildActsContext(notes: LayoutCacheRow[]): ModeContext {
	const counts = new Map<string, number>();
	for (const n of notes) {
		const k = n.actsPrimary ?? 'Unacted';
		counts.set(k, (counts.get(k) ?? 0) + 1);
	}
	return makeUniformWedges(
		ACTS_ORDER.map(k => k as string),
		counts,
		(n) => n.actsPrimary ?? 'Unacted',
		(k) => k,
		(n) => n.actsPrimary != null,      // hollow if Unacted
	);
}

// ─── P Provenance ─────────────────────────────────────────────────
function buildProvenanceContext(notes: LayoutCacheRow[]): ModeContext {
	const counts = new Map<string, number>();
	for (const n of notes) {
		const k = sourceFamily(n.sourcesPrimary);
		counts.set(k, (counts.get(k) ?? 0) + 1);
	}
	return makeUniformWedges(
		SOURCE_FAMILIES.map(k => k as string),
		counts,
		(n) => sourceFamily(n.sourcesPrimary),
		(k) => k,
		(n) => n.sourcesPrimary != null,   // hollow if unsourced
	);
}
function sourceFamily(primary: string | null): string {
	if (!primary) return 'unsourced';
	// CECE's live taxonomy uses `family/leaf` ID shape (e.g.
	// `testimony/authoritative`). Top-level family is the prefix.
	const slash = primary.indexOf('/');
	return slash > 0 ? primary.slice(0, slash) : primary;
}

// ─── shared helper ────────────────────────────────────────────────
/** Distribute a fixed-order key set uniformly around the rim. Each
 *  bucket gets an equal-angle span; the active wedge set is the
 *  full canonical key list so empty buckets remain visible (they
 *  become to-do prompts per Concept Paper §6 + D-V6).
 *
 *  fix-8 (2026-05-13): accepts a `hasModeData` predicate so the
 *  per-mode "is this note classified for the active mode?" check is
 *  available to render.ts via the ModeContext. Used by the §2
 *  hollow-cascade priority 1 (mode-data missing → gold frame). */
function makeUniformWedges(
	canonicalKeys: string[],
	counts: Map<string, number>,
	bucketKeyFor: (n: LayoutCacheRow) => string,
	labelFor: (k: string) => string,
	hasModeData: (n: LayoutCacheRow) => boolean,
): ModeContext {
	const total = canonicalKeys.length;
	const span = (2 * Math.PI) / total;
	const wedges: WedgeBucket[] = canonicalKeys.map((key, i) => ({
		key,
		label: labelFor(key),
		count: counts.get(key) ?? 0,
		azimuthStart: -Math.PI / 2 + i * span,
		azimuthEnd: -Math.PI / 2 + (i + 1) * span,
	}));
	return { wedges, bucketKeyFor, hasModeData };
}
