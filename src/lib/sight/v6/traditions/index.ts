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
import type { UserTraditionModule } from './userDefinedLoader';
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
import { talmudicMiddot } from './talmudic-middot';
import { mencianSprouts } from './mencian-sprouts';
import { wangYangming } from './wang-yangming';
import { koreanSongnihak } from './korean-songnihak';
import { mignoloPluriversal } from './mignolo-pluriversal';
import { dusselTransmodernity } from './dussel-transmodernity';
import { maldonadoTorres } from './maldonado-torres';
import { akanWiredu } from './akan-wiredu';
import { ibuanyidanda } from './ibuanyidanda';

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
 * Phase ζ.3:    talmudic-middot (ladder spiral, 13 steps)
 * Phase η.1:    mencian-sprouts (sectoral 4-cell + center ring xìn)
 * Phase η.2:    wang-yangming (binary-flow vertical)
 * Phase η.3:    korean-songnihak (sectoral 4-cell, 2×2 grid concept)
 * Phase θ.1:    mignolo-pluriversal (relational hub-and-spoke)
 * Phase θ.2:    dussel-transmodernity (binary-flow concentric)
 * Phase θ.3:    maldonado-torres (rings, 3 zones)
 * Phase θ.4:    akan-wiredu (sectoral 3-cell)
 * Phase θ.5:    ibuanyidanda (relational hub-and-spoke)
 * REGISTRY COMPLETE — all 24 curated baseline traditions shipped.
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
	'talmudic-middot': talmudicMiddot,
	'mencian-sprouts': mencianSprouts,
	'wang-yangming': wangYangming,
	'korean-songnihak': koreanSongnihak,
	'mignolo-pluriversal': mignoloPluriversal,
	'dussel-transmodernity': dusselTransmodernity,
	'maldonado-torres': maldonadoTorres,
	'akan-wiredu': akanWiredu,
	ibuanyidanda,
};

// ════════════════════════════════════════════════════════════════════
// MIG-026 Phase κ.1 — user-defined traditions side-map
// ════════════════════════════════════════════════════════════════════
//
// User-authored declarative tradition JSON files load at Sight mount
// (via userDefinedLoader.loadUserTraditions()) and are registered
// here in a Map keyed by their `user-` prefixed string id. The
// curated REGISTRY above stays type-safe (closed TraditionId union);
// the user side-map accepts any string id matching the v1 schema's
// `^user-[a-z0-9][a-z0-9-]{2,40}$` pattern.
//
// getTraditionById + allTraditions check both maps so the chip,
// anchor renderer, and channel-isolation test all see user traditions
// alongside the curated set.
//
// Lifecycle: registerUserTraditions REPLACES the side-map contents
// (called once per Universe switch). The chip + renderer re-read
// via getTraditionById on every paint, so no manual cache
// invalidation needed.

const USER_REGISTRY = new Map<string, UserTraditionModule>();

/** Replace the user-tradition side-map with the given list. Called
 *  once per Sight mount + once per Universe switch. Passing an empty
 *  list clears the map (useful for switching to a Universe with no
 *  user traditions). */
export function registerUserTraditions(modules: UserTraditionModule[]): void {
	USER_REGISTRY.clear();
	for (const m of modules) {
		USER_REGISTRY.set(m.id, m);
	}
}

/** Read-only view of user-defined traditions, for the chip dropdown.
 *  Returns modules in registration order (which is filename order
 *  per the Rust IPC's sort). */
export function allUserTraditions(): UserTraditionModule[] {
	return Array.from(USER_REGISTRY.values());
}

/**
 * Look up a tradition module by id. Returns null when the requested
 * tradition has no module yet (incremental build state) so callers
 * can fall back to default Aristotelian positions without crashing.
 *
 * Accepts undefined/null for ergonomic call sites that read directly
 * from `$appSettings.sight?.activeTradition` (which is optional).
 *
 * MIG-026 Phase κ.1: the id type widened from `TraditionId` to
 * `TraditionId | string` so user-defined traditions (with `user-`
 * prefix string ids) resolve cleanly. Lookup order: curated REGISTRY
 * first, then USER_REGISTRY. Curated ids cannot collide with user
 * ids because the schema's `^user-` prefix rule prevents it.
 *
 * UserTraditionModule is structurally compatible with TraditionModule
 * for every field the renderers read (shape, remapStarPosition,
 * sectorDividers, ringBoundaries, horizontalBandsSpec, gradientSpec).
 * The only difference is `id: string` vs `id: TraditionId`. Renderers
 * never read id; the cast at the registry boundary keeps the consumer
 * API stable as `TraditionModule | null`.
 */
export function getTraditionById(
	id: TraditionId | string | undefined | null,
): TraditionModule | null {
	if (!id) return null;
	const curated = REGISTRY[id as TraditionId];
	if (curated) return curated;
	const user = USER_REGISTRY.get(id);
	// Renderer-safe cast: UserTraditionModule is structurally a
	// TraditionModule modulo the id type. All renderer-consumed
	// fields are identical.
	return user ? (user as unknown as TraditionModule) : null;
}

/**
 * All currently-registered tradition modules. Used by the channel-
 * isolation test (Phase μ.1) to iterate through every tradition and
 * assert the mini-domes stay constant.
 *
 * MIG-026 Phase κ.1: now includes user-defined traditions registered
 * via registerUserTraditions(), cast to TraditionModule at the
 * boundary (same justification as getTraditionById above).
 */
export function allTraditions(): TraditionModule[] {
	const curated = Object.values(REGISTRY).filter(
		(m): m is TraditionModule => m !== undefined,
	);
	const user = Array.from(USER_REGISTRY.values()).map(
		(m) => m as unknown as TraditionModule,
	);
	return [...curated, ...user];
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
		// Phase ζ.1 added pardes; ζ.2 added maimonidean-prophecy;
		// ζ.3 adds talmudic-middot. Family complete.
		traditions: ['pardes', 'maimonidean-prophecy', 'talmudic-middot'],
	},
	'east-asian-confucian': {
		label: 'East Asian Confucian',
		// Phase η adds all 3 in one cascade. Family complete.
		traditions: ['mencian-sprouts', 'wang-yangming', 'korean-songnihak'],
	},
	'chinese-pragmatist': {
		label: 'Chinese pragmatist',
		traditions: ['mohist-san-biao'],
	},
	'african-philosophical': {
		label: 'African philosophical',
		// Phase θ adds akan-wiredu (θ.4) + ibuanyidanda (θ.5). Family complete.
		traditions: ['akan-wiredu', 'ibuanyidanda'],
	},
	'latin-decolonial': {
		label: 'Latin American decolonial',
		// Phase θ adds mignolo (θ.1) + dussel (θ.2) + maldonado-torres (θ.3).
		// Family complete.
		traditions: ['mignolo-pluriversal', 'dussel-transmodernity', 'maldonado-torres'],
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
	talmudicMiddot,
	mencianSprouts,
	wangYangming,
	koreanSongnihak,
	mignoloPluriversal,
	dusselTransmodernity,
	maldonadoTorres,
	akanWiredu,
	ibuanyidanda,
};
