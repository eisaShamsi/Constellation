/**
 * MIG-025 §C.2 — Register registry.
 *
 * Central lookup table for the 6 epistemic registers. Each register's
 * module lives in this directory as `<id>.ts` and exports a single
 * `RegisterModule` const. This index re-exports them and provides a
 * `getRegisterById` lookup used by anchor.ts and SightV6.svelte.
 *
 * §C.2 ships with Aristotelian only. As §C.3 / §C.4 / §C.5 / §D.2 /
 * §D.3 land, REGISTRY gains entries. Until a register's module ships,
 * `getRegisterById('<id>')` returns null and the anchor falls back to
 * Aristotelian default positions (the chip will still show the user's
 * selection — chip state is separate from module availability).
 *
 * Per Concept Paper §11 invariant 7: each register's geometry is
 * documented + citation-tracked in a manifest at
 * `docs/registers/<id>.md` (ships in §C.7).
 *
 * Dignāga is excluded entirely per §C.1-fix-1 (Eisa 2026-05-16) —
 * not in this registry, not in RegisterId, not in store.ts.
 */
import type { RegisterId, RegisterModule } from '../types';
import { aristotelian } from './aristotelian';
import { pramana } from './pramana';
import { masadir } from './masadir';

/**
 * The registered modules. Keyed by RegisterId for O(1) lookup.
 * Entries are added incrementally as Phase 3 / Phase 4 ship the
 * remaining register modules.
 *
 * §C.2 (this commit): aristotelian
 * §C.3 (next):        pramana
 * §C.4:               masadir
 * §C.5:               polanyi
 * §D.2:               ishraqi
 * §D.3:               mohist-san-biao
 */
const REGISTRY: Partial<Record<RegisterId, RegisterModule>> = {
	aristotelian,
	pramana,
	masadir,
	// polanyi — §C.5
	// ishraqi — §D.2 (v1-preview)
	// 'mohist-san-biao' — §D.3 (v1-preview)
};

/**
 * Look up a register module by id. Returns null when the requested
 * register has no module yet (incremental build state) so callers
 * can fall back to default Aristotelian positions without crashing.
 *
 * Accepts undefined/null for ergonomic call sites that read directly
 * from `$appSettings.sight?.activeRegister` (which is optional).
 */
export function getRegisterById(id: RegisterId | undefined | null): RegisterModule | null {
	if (!id) return null;
	return REGISTRY[id] ?? null;
}

/**
 * All currently-registered modules. Used by the channel-isolation
 * test (§C.9) to iterate through every register and assert the
 * mini-domes stay constant.
 */
export function allRegisters(): RegisterModule[] {
	return Object.values(REGISTRY).filter((m): m is RegisterModule => m !== undefined);
}

export { aristotelian, pramana, masadir };
