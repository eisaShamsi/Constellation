/**
 * MIG-025 §C.2 — Tradition registry.
 * MIG-026 Phase 0 — K1 full rename: "register" → "tradition" throughout.
 * MIG-026 Phase β — adds FAMILIES const for the A3+A6 chip UI
 *   (family-categorized dropdown + 4 favorites inline).
 *
 * Central lookup table for the curated baseline traditions. Each
 * tradition's module lives in this directory as `<id>.ts` and exports
 * a single `TraditionModule` const. This index re-exports them and
 * provides:
 *   - `getTraditionById(id)`  — O(1) lookup
 *   - `allTraditions()`        — enumeration for §C.9 channel-iso test
 *   - `FAMILIES` const         — family groupings used by the chip
 *                                dropdown UI in Phase β
 *
 * Phase 0 (rename) ships the existing 3 modules (Aristotelian,
 * pramāṇa, masādir) under the renamed namespace. Phase α adds the
 * multi-shape architecture. Phase β rebuilds the chip UI per A3+A6.
 * Phase γ adds Polanyi + Mohist modules. Phases δ–θ add the 19 new
 * traditions to bring the curated baseline to 24.
 *
 * Dignāga + Suhrawardi Ishrāqī excluded entirely per §C.1-fix-1 +
 * §C.4-religious-rule — not in this registry, not in TraditionId,
 * not in store.ts.
 */
import type { TraditionId, TraditionModule } from '../types';
import { aristotelian } from './aristotelian';
import { pramana } from './pramana';
import { masadir } from './masadir';
import { polanyi } from './polanyi';
import { mohistSanBiao } from './mohist-san-biao';
import { peirce } from './peirce';
import { habermas } from './habermas';
import { dewey } from './dewey';
import { husserl } from './husserl';
import { longino } from './longino';
import { ibnRushdBurhan } from './ibn-rushd-burhan';
import { shatibiMaqasid } from './shatibi-maqasid';
import { ibnKhaldunUmran } from './ibn-khaldun-umran';
import { pardes } from './pardes';
import { maimonideanProphecy } from './maimonidean-prophecy';

/**
 * The registered tradition modules. Keyed by TraditionId for O(1)
 * lookup. Entries are added incrementally as MIG-026 phases ship the
 * remaining tradition modules.
 *
 * Phase 0:      aristotelian, pramana, masadir
 * Phase γ:      polanyi (gradient), mohist-san-biao (horizontal-bands)
 * Phase δ.1:    peirce (sectoral), habermas (sectoral)
 * Phase δ.2:    dewey (cyclic-flow), husserl (rings), longino (sectoral)
 * Phase ε.1:    ibn-rushd-burhan (rings, 4 zones)
 * Phase ε.2:    shatibi-maqasid (grid, 3×5 = 15 cells)
 * Phase ε.3:    ibn-khaldun-umran (binary-flow, 2 horizontal bands)
 * Phase ζ.1:    pardes (rings, 4 zones)
 * Phase ζ.2:    maimonidean-prophecy (ladder spiral, 11 steps)
 * Phases ζ.3–θ: the 9 remaining newcomers across the remaining shapes
 */
const REGISTRY: Partial<Record<TraditionId, TraditionModule>> = {
	aristotelian,
	pramana,
	masadir,
	polanyi,
	'mohist-san-biao': mohistSanBiao,
	peirce,
	habermas,
	dewey,
	husserl,
	longino,
	'ibn-rushd-burhan': ibnRushdBurhan,
	'shatibi-maqasid': shatibiMaqasid,
	'ibn-khaldun-umran': ibnKhaldunUmran,
	pardes,
	'maimonidean-prophecy': maimonideanProphecy,
	// (Phases ζ.3–θ add the 9 remaining traditions per the MIG-026 Plan)
};

/**
 * Look up a tradition module by id. Returns null when the requested
 * tradition has no module yet (incremental build state) so callers
 * can fall back to default Aristotelian positions without crashing.
 *
 * Accepts undefined/null for ergonomic call sites that read directly
 * from `$appSettings.sight?.activeTradition` (which is optional).
 */
export function getTraditionById(id: TraditionId | undefined | null): TraditionModule | null {
	if (!id) return null;
	return REGISTRY[id] ?? null;
}

/**
 * All currently-registered tradition modules. Used by the channel-
 * isolation test (Phase μ.1) to iterate through every tradition and
 * assert the mini-domes stay constant.
 */
export function allTraditions(): TraditionModule[] {
	return Object.values(REGISTRY).filter((m): m is TraditionModule => m !== undefined);
}

// ════════════════════════════════════════════════════════════════════
// MIG-026 Phase β — Family taxonomy for the A3+A6 chip dropdown UI
// ════════════════════════════════════════════════════════════════════
//
// 10 family groupings used by the chip dropdown panel (per the
// MIG-026 Plan v2.11 + Architect §3.A). The dropdown shows one
// section per family that has ≥1 tradition with a shipping module;
// families with no shipped modules are hidden until their phase
// lands. The label is the section header text shown in the dropdown.

/** Family identifier — used by chip dropdown + tradition-to-family
 *  mapping. Stable across MIG phases. */
export type FamilyId =
	| 'western-classical'
	| 'indian-nyaya'
	| 'sunni-islamic-usul'
	| 'arabic-islamic-beyond'
	| 'modern-western'
	| 'jewish-abrahamic'
	| 'east-asian-confucian'
	| 'chinese-pragmatist'
	| 'african-philosophical'
	| 'latin-decolonial';

/** Per-family display label + list of constituent traditions. The
 *  list includes traditions that will ship in future phases (so the
 *  taxonomy is stable across MIG-026); the dropdown UI filters to
 *  show only families with ≥1 module currently registered. */
export const FAMILIES: Record<FamilyId, { label: string; traditions: TraditionId[] }> = {
	'western-classical': {
		label: 'Western classical',
		traditions: ['aristotelian'],
	},
	'indian-nyaya': {
		label: 'Indian Nyāya',
		traditions: ['pramana'],
	},
	'sunni-islamic-usul': {
		label: 'Sunni Islamic uṣūl',
		traditions: ['masadir'],
	},
	'arabic-islamic-beyond': {
		label: 'Arabic / Islamic beyond uṣūl',
		// Phase ε.1 added ibn-rushd-burhan; ε.2 added shatibi-maqasid;
		// ε.3 adds ibn-khaldun-umran. Family complete (3 modules).
		traditions: ['ibn-rushd-burhan', 'shatibi-maqasid', 'ibn-khaldun-umran'],
	},
	'modern-western': {
		label: 'Modern Western',
		// Phase γ added polanyi; Phase δ.1 added peirce + habermas;
		// Phase δ.2 adds dewey + husserl + longino. Family complete.
		traditions: ['polanyi', 'peirce', 'habermas', 'dewey', 'husserl', 'longino'],
	},
	'jewish-abrahamic': {
		label: 'Jewish (Abrahamic)',
		// Phase ζ.1 added pardes; ζ.2 adds maimonidean-prophecy;
		// ζ.3 will add talmudic-middot.
		traditions: ['pardes', 'maimonidean-prophecy'],
	},
	'east-asian-confucian': {
		label: 'East Asian Confucian',
		// Phase η will add: 'mencian-sprouts', 'wang-yangming', 'korean-songnihak'
		traditions: [],
	},
	'chinese-pragmatist': {
		label: 'Chinese pragmatist',
		traditions: ['mohist-san-biao'],
	},
	'african-philosophical': {
		label: 'African philosophical',
		// Phase θ will add: 'akan-wiredu', 'ibuanyidanda'
		traditions: [],
	},
	'latin-decolonial': {
		label: 'Latin American decolonial',
		// Phase θ will add: 'mignolo-pluriversal', 'dussel-transmodernity',
		// 'maldonado-torres'
		traditions: [],
	},
};

/** Reverse lookup: tradition → family. Computed once at module load. */
export const TRADITION_TO_FAMILY: Map<TraditionId, FamilyId> = (() => {
	const m = new Map<TraditionId, FamilyId>();
	for (const [familyId, family] of Object.entries(FAMILIES)) {
		for (const tradId of family.traditions) {
			m.set(tradId, familyId as FamilyId);
		}
	}
	return m;
})();

export {
	aristotelian,
	pramana,
	masadir,
	polanyi,
	mohistSanBiao,
	peirce,
	habermas,
	dewey,
	husserl,
	longino,
	ibnRushdBurhan,
	shatibiMaqasid,
	ibnKhaldunUmran,
	pardes,
	maimonideanProphecy,
};
