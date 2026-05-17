/**
 * MIG-025 §C.2 — Tradition registry (renamed from Register registry).
 * MIG-026 Phase 0 — K1 full rename: "register" → "tradition" throughout.
 *
 * Central lookup table for the curated baseline traditions. Each tradition's
 * module lives in this directory as `<id>.ts` and exports a single
 * `TraditionModule` const. This index re-exports them and provides a
 * `getTraditionById` lookup used by anchor.ts and SightV6.svelte.
 *
 * Phase 0 of MIG-026 ships the rename only; the existing 3 modules
 * (Aristotelian, pramāṇa, masādir) continue to work under the renamed
 * namespace. Phase γ adds Polanyi + Mohist; Phases δ–θ add the 19 new
 * traditions per the MIG-026 Plan.
 *
 * Per Concept Paper §11 invariant 7: each tradition's geometry is
 * documented + citation-tracked in a manifest at `docs/traditions/<id>.md`
 * (manifests ship in Phase ι.1).
 *
 * Dignāga + Suhrawardi Ishrāqī excluded entirely per §C.1-fix-1 + §C.4-
 * religious-rule (Eisa 2026-05-16) — not in this registry, not in
 * TraditionId, not in store.ts.
 */
import type { TraditionId, TraditionModule } from '../types';
import { aristotelian } from './aristotelian';
import { pramana } from './pramana';
import { masadir } from './masadir';

/**
 * The registered tradition modules. Keyed by TraditionId for O(1)
 * lookup. Entries are added incrementally as MIG-026 phases ship the
 * remaining tradition modules.
 *
 * Phase 0 (MIG-026, this rename): aristotelian, pramana, masadir
 * Phase γ (next):                  polanyi, mohist-san-biao
 * Phase δ:                         peirce, dewey, husserl, habermas, longino
 * Phase ε:                         ibn-rushd-burhan, shatibi-maqasid, ibn-khaldun-umran
 * Phase ζ:                         pardes, maimonidean-prophecy, talmudic-middot
 * Phase η:                         mencian-sprouts, wang-yangming, korean-songnihak
 * Phase θ:                         mignolo-pluriversal, dussel-transmodernity,
 *                                  maldonado-torres, akan-wiredu, ibuanyidanda
 */
const REGISTRY: Partial<Record<TraditionId, TraditionModule>> = {
	aristotelian,
	pramana,
	masadir,
	// polanyi — Phase γ
	// 'mohist-san-biao' — Phase γ
	// (Phases δ–θ add the 19 new traditions per the MIG-026 Plan)
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

export { aristotelian, pramana, masadir };
