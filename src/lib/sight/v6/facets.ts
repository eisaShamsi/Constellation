/**
 * MIG-025 §A.10 — Sight v6 facet logic (pure functions).
 *
 * Hearst Flamenco pattern (cf. Hearst, "UIs for Faceted Navigation",
 * 2009): counts in EACH facet show what's available **given the
 * current filter set in the OTHER facets**. Multi-facet AND across,
 * OR within. So if the user has Library=Research selected, the
 * Stratum facet shows count of Research-AND-{stratum-X} per stratum.
 *
 * Per Concept Paper v4.0 §2.3 + §11 invariant 8:
 *   six facets total; Folder is the TOP facet (round-2/3 LIS critique
 *   demanded explicit Folder visibility — flagged as missing in v0.2).
 *
 * Pure functions; no Svelte runes, no state. Sidebar component
 * consumes these at render time.
 */

import type {
	LayoutCacheRow,
	Facet,
	FacetCategory,
	FacetId,
	ConfidenceLevel,
	LifecycleStage,
	ProvenanceSector,
} from './types';
import { bandForRawStratum, STRATUM_BANDS, STRATUM_LABELS, type StratumBand } from './dome';

// ════════════════════════════════════════════════════════════════════
// Filter state
// ════════════════════════════════════════════════════════════════════

/**
 * Active filter set. Sets are EMPTY = no filter on that facet (all
 * categories visible); a non-empty set is OR-within (any matching
 * category passes). AND across facets.
 *
 * Stored as Sets for O(1) membership tests in `applyFilters`. Stage
 * + Confidence + Provenance use string union types; Stratum uses
 * the StratumBand union from dome.ts.
 */
export interface FacetFilters {
	folder: Set<string>;
	library: Set<string>;
	stratum: Set<StratumBand>;
	confidence: Set<ConfidenceLevel>;
	stage: Set<LifecycleStage>;
	provenance: Set<ProvenanceSector>;
}

/** Factory: a fresh, all-empty filter set (no constraints). */
export function emptyFilters(): FacetFilters {
	return {
		folder: new Set<string>(),
		library: new Set<string>(),
		stratum: new Set<StratumBand>(),
		confidence: new Set<ConfidenceLevel>(),
		stage: new Set<LifecycleStage>(),
		provenance: new Set<ProvenanceSector>(),
	};
}

/** True when no facet has any active category (universe-wide view). */
export function filtersEmpty(f: FacetFilters): boolean {
	return (
		f.folder.size === 0 &&
		f.library.size === 0 &&
		f.stratum.size === 0 &&
		f.confidence.size === 0 &&
		f.stage.size === 0 &&
		f.provenance.size === 0
	);
}

/** Toggle a category in a facet. Returns a NEW FacetFilters object
 *  (immutable update so Svelte $state reactivity fires). */
export function toggleFilter<K extends FacetId>(
	f: FacetFilters,
	facet: K,
	categoryId: string,
): FacetFilters {
	const next: FacetFilters = {
		folder: new Set(f.folder),
		library: new Set(f.library),
		stratum: new Set(f.stratum),
		confidence: new Set(f.confidence),
		stage: new Set(f.stage),
		provenance: new Set(f.provenance),
	};
	const set = next[facet] as Set<string>;
	if (set.has(categoryId)) set.delete(categoryId);
	else set.add(categoryId);
	return next;
}

// ════════════════════════════════════════════════════════════════════
// Row → facet category mapping
// ════════════════════════════════════════════════════════════════════

/** Confidence level inferred from confidenceAlpha (per v6 schema +
 *  §3.4 palette). Must match the §A.3 backfill aggregation:
 *    1.0  → established
 *    0.85 → contested
 *    0.7  → evidence
 *    other / null → hypothesis (default) */
export function confidenceLevelOf(row: LayoutCacheRow): ConfidenceLevel {
	const a = row.confidenceAlpha;
	if (a === null) return 'hypothesis';
	if (a >= 0.99) return 'established';
	if (a >= 0.84 && a <= 0.86) return 'contested';
	if (a >= 0.69 && a <= 0.71) return 'evidence';
	return 'hypothesis';
}

/** Provenance sector — same heuristic as anchor.ts. v4.1 polish target
 *  swaps to the masādir-aware classifier per Concept Paper §10. */
export function provenanceSectorOf(sourcesPrimary: string | null): ProvenanceSector {
	if (!sourcesPrimary) return 'Self';
	const s = sourcesPrimary.toLowerCase();
	if (s.includes('http://') || s.includes('https://') || s.includes('book:')) return 'Read';
	if (s.includes('podcast:') || s.includes('heard:') || s.includes('audio:')) return 'Heard';
	if (s.includes('reasoned:') || s.includes('inference:')) return 'Reasoned';
	if (s.includes('tradition:') || s.includes('canon:') || s.includes('scripture:')) return 'Tradition';
	return 'Self';
}

// ════════════════════════════════════════════════════════════════════
// Filter application
// ════════════════════════════════════════════════════════════════════

/**
 * Apply a filter set to row list. AND across facets, OR within.
 * Empty facets are pass-through (no constraint).
 */
export function applyFilters(rows: LayoutCacheRow[], filters: FacetFilters): LayoutCacheRow[] {
	if (filtersEmpty(filters)) return rows;
	return rows.filter((r) => rowMatches(r, filters));
}

function rowMatches(r: LayoutCacheRow, f: FacetFilters): boolean {
	if (f.folder.size > 0 && !folderMatch(r, f.folder)) return false;
	if (f.library.size > 0 && (!r.libraryName || !f.library.has(r.libraryName))) return false;
	if (f.stratum.size > 0 && !f.stratum.has(bandForRawStratum(r.stratum))) return false;
	if (f.confidence.size > 0 && !f.confidence.has(confidenceLevelOf(r))) return false;
	if (f.stage.size > 0 && !f.stage.has(r.stage as LifecycleStage)) return false;
	if (f.provenance.size > 0 && !f.provenance.has(provenanceSectorOf(r.sourcesPrimary))) return false;
	return true;
}

/** Folder match: row's folderPath is in the active set OR any active
 *  folder is a parent of the row's folder (so selecting "Research"
 *  matches "Research/Subfolder/note.md"). */
function folderMatch(r: LayoutCacheRow, activeFolders: Set<string>): boolean {
	if (!r.folderPath) return false;
	if (activeFolders.has(r.folderPath)) return true;
	for (const f of activeFolders) {
		if (r.folderPath.startsWith(f + '/')) return true;
	}
	return false;
}

// ════════════════════════════════════════════════════════════════════
// Facet count computation (Hearst preview pattern)
// ════════════════════════════════════════════════════════════════════

/**
 * For each facet F, compute counts on the row set FILTERED BY ALL
 * FACETS EXCEPT F. This is the Hearst preview pattern: clicking a
 * category in F shows what would happen WITHOUT changing F's own
 * filter — so the user previews the impact of toggling that
 * category without committing.
 *
 * Returns the 6 facets in display order (Folder TOP per §11 inv. 8).
 */
export function computeFacetCounts(rows: LayoutCacheRow[], filters: FacetFilters): Facet[] {
	return [
		buildFolderFacet(rows, filters),
		buildLibraryFacet(rows, filters),
		buildStratumFacet(rows, filters),
		buildConfidenceFacet(rows, filters),
		buildStageFacet(rows, filters),
		buildProvenanceFacet(rows, filters),
	];
}

/** Helper: filter rows by all facets EXCEPT the one we're computing
 *  counts for (Hearst preview). */
function rowsExcluding(rows: LayoutCacheRow[], filters: FacetFilters, exclude: FacetId): LayoutCacheRow[] {
	const f: FacetFilters = {
		folder:     exclude === 'folder'     ? new Set() : filters.folder,
		library:    exclude === 'library'    ? new Set() : filters.library,
		stratum:    exclude === 'stratum'    ? new Set() : filters.stratum,
		confidence: exclude === 'confidence' ? new Set() : filters.confidence,
		stage:      exclude === 'stage'      ? new Set() : filters.stage,
		provenance: exclude === 'provenance' ? new Set() : filters.provenance,
	};
	return applyFilters(rows, f);
}

function buildFolderFacet(rows: LayoutCacheRow[], filters: FacetFilters): Facet {
	const subset = rowsExcluding(rows, filters, 'folder');
	const counts = new Map<string, number>();
	for (const r of subset) {
		if (r.folderPath) counts.set(r.folderPath, (counts.get(r.folderPath) ?? 0) + 1);
	}
	// MIG-026 §λ-fix-4 — folder names are user data, NOT translated.
	// `label: id` keeps the user's folder path; the sidebar wraps every
	// label in $t which acts as identity for unknown keys.
	const cats: FacetCategory[] = [...counts.entries()]
		.map(([id, count]) => ({ id, label: id, count }))
		.sort((a, b) => b.count - a.count); // most-populated first
	return { id: 'folder', label: 'sight.v6.facet.folder', categories: cats };
}

function buildLibraryFacet(rows: LayoutCacheRow[], filters: FacetFilters): Facet {
	const subset = rowsExcluding(rows, filters, 'library');
	const counts = new Map<string, number>();
	for (const r of subset) {
		if (r.libraryName) counts.set(r.libraryName, (counts.get(r.libraryName) ?? 0) + 1);
	}
	// MIG-026 §λ-fix-4 — library names are user data, NOT translated.
	const cats: FacetCategory[] = [...counts.entries()]
		.map(([id, count]) => ({ id, label: id, count }))
		.sort((a, b) => a.id.localeCompare(b.id)); // stable alpha order — matches libraryShapeIndex assignment
	return { id: 'library', label: 'sight.v6.facet.library', categories: cats };
}

function buildStratumFacet(rows: LayoutCacheRow[], filters: FacetFilters): Facet {
	const subset = rowsExcluding(rows, filters, 'stratum');
	const counts = new Map<StratumBand, number>();
	for (const r of subset) {
		const band = bandForRawStratum(r.stratum);
		counts.set(band, (counts.get(band) ?? 0) + 1);
	}
	// Stable order: inner→outer per STRATUM_BANDS.
	// MIG-026 §λ-fix-4 — labels become i18n keys; sidebar wraps in $t.
	// Keys live at sight.v6.stratum.<band> in every locale file.
	const cats: FacetCategory[] = STRATUM_BANDS.map((b) => ({
		id: b,
		label: `sight.v6.stratum.${b}`,
		count: counts.get(b) ?? 0,
	}));
	return { id: 'stratum', label: 'sight.v6.facet.stratum', categories: cats };
}

function buildConfidenceFacet(rows: LayoutCacheRow[], filters: FacetFilters): Facet {
	const subset = rowsExcluding(rows, filters, 'confidence');
	const counts = new Map<ConfidenceLevel, number>();
	for (const r of subset) {
		const c = confidenceLevelOf(r);
		counts.set(c, (counts.get(c) ?? 0) + 1);
	}
	const order: ConfidenceLevel[] = ['hypothesis', 'evidence', 'established', 'contested'];
	// MIG-026 §λ-fix-4 — i18n keys at sight.v6.confidence.<level>.
	const cats: FacetCategory[] = order.map((c) => ({
		id: c,
		label: `sight.v6.confidence.${c}`,
		count: counts.get(c) ?? 0,
	}));
	return { id: 'confidence', label: 'sight.v6.facet.confidence', categories: cats };
}

function buildStageFacet(rows: LayoutCacheRow[], filters: FacetFilters): Facet {
	const subset = rowsExcluding(rows, filters, 'stage');
	const counts = new Map<string, number>();
	for (const r of subset) {
		if (r.stage) counts.set(r.stage, (counts.get(r.stage) ?? 0) + 1);
	}
	// §B.7 (2026-05-15): enumerate stages dynamically. The previous
	// hardcoded `LifecycleStage` list (established / fresh / growing /
	// at-risk / dormant) is the Concept Paper v4.0 vocabulary, but
	// CLAUDE.md's Living Link Architecture defines the canonical
	// lifecycle as Spark → Birth → Growth → Maturity → Dormancy →
	// Renewal → Archival — and that's what user data actually contains
	// (Eisa's universe: 99.3% of stage values are Living Link tokens).
	// Without dynamic enumeration, Shift+click filtering on a Living
	// Link star would apply the filter correctly (string equality in
	// applyFilters) but the resulting chip wouldn't appear in the
	// sidebar — the user couldn't see or remove the active filter.
	//
	// Display order: Living Link lifecycle progression (most common
	// in user data), then Concept Paper v4.0 vocabulary, then any
	// other strings found in data (descending count). Only stages
	// present in the data appear; zero-count chips are suppressed
	// to keep the sidebar focused on what's actionable.
	const orderedKnown: string[] = [
		'spark', 'birth', 'growth', 'maturity', 'dormancy', 'renewal', 'archival',
		'established', 'fresh', 'growing', 'at-risk', 'dormant',
	];
	// MIG-026 §λ-fix-4 — labels become i18n keys for the known stages
	// (sight.v6.stage.<id>). Custom user-defined stages keep their raw
	// string as the label; $t in the sidebar acts as identity for any
	// key that doesn't resolve in the active locale + en fallback.
	const cats: FacetCategory[] = [];
	const seen = new Set<string>();
	for (const s of orderedKnown) {
		if (counts.has(s)) {
			cats.push({ id: s, label: `sight.v6.stage.${s}`, count: counts.get(s) ?? 0 });
			seen.add(s);
		}
	}
	for (const [id, count] of [...counts.entries()].sort((a, b) => b[1] - a[1])) {
		if (!seen.has(id)) {
			cats.push({ id, label: id, count });
		}
	}
	return { id: 'stage', label: 'sight.v6.facet.stage', categories: cats };
}

function buildProvenanceFacet(rows: LayoutCacheRow[], filters: FacetFilters): Facet {
	const subset = rowsExcluding(rows, filters, 'provenance');
	const counts = new Map<ProvenanceSector, number>();
	for (const r of subset) {
		const p = provenanceSectorOf(r.sourcesPrimary);
		counts.set(p, (counts.get(p) ?? 0) + 1);
	}
	const order: ProvenanceSector[] = ['Self', 'Read', 'Heard', 'Reasoned', 'Tradition'];
	// MIG-026 §λ-fix-4 — i18n keys at sight.v6.miniDome.provenance.<lower>.
	// Reuses the same key namespace already defined for mini-dome canvas
	// labels so a single translation entry serves both the sidebar chip
	// and the canvas sector label.
	const cats: FacetCategory[] = order.map((p) => ({
		id: p,
		label: `sight.v6.miniDome.provenance.${p.toLowerCase()}`,
		count: counts.get(p) ?? 0,
	}));
	return { id: 'provenance', label: 'sight.v6.facet.provenance', categories: cats };
}

// ════════════════════════════════════════════════════════════════════
// Helpers
// ════════════════════════════════════════════════════════════════════

function titleCase(label: string): string {
	// "FOUNDATION" → "Foundation", "EDGE OF KNOWING" → "Edge of Knowing"
	return label
		.toLowerCase()
		.split(' ')
		.map((w, i) => (i === 0 || (w !== 'of' && w !== 'and') ? w[0].toUpperCase() + w.slice(1) : w))
		.join(' ');
}
