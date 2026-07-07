/**
 * MIG-092 §7 (was MIG-090) — a Collection's four state chips as PURE predicates + one
 * pure intersection filter. Extracted from the component so the invariant is
 * pinned by test forever: **chips NARROW the held set client-side and never
 * query** — guarding the verified engine landmine where hybrid-mode
 * structured filters APPEND (expand) results instead of narrowing.
 * No IPC exists in this module by construction (no imports that could carry
 * one; the pinned test asserts the filter is a pure array intersection).
 */
import { splitStage } from '$lib/libraries/store';
import { isIsolated } from '$lib/connectivity';

export interface ChipFacts {
	stage: string | null;
	incoming_count: number;
	outgoing_count: number;
	incoming_link_types_json: string;
	outgoing_link_types_json: string;
	review_due: boolean;
}

export interface ChipToggles {
	due: boolean;
	unlinked: boolean;
	contested: boolean;
	forming: boolean;
}

export function isContested(r: ChipFacts): boolean {
	for (const j of [r.incoming_link_types_json, r.outgoing_link_types_json]) {
		try {
			const o = JSON.parse(j || '{}');
			if ((o['contradicts'] ?? 0) > 0) return true;
		} catch { /* malformed json → not contested */ }
	}
	return false;
}

/** The user's OWN declared stage decides "forming" (honest absence when
 *  unstaged — the ratified default). Lifecycle prefix per splitStage. */
export function isForming(r: ChipFacts): boolean {
	if (!r.stage) return false;
	const { lifecycle } = splitStage(r.stage);
	return lifecycle === 'spark' || lifecycle === 'birth' || lifecycle === 'growth';
}

/** The Collections "Unlinked" chip = the shared ISOLATED predicate (MIG-094). */
export function isUnlinked(r: ChipFacts): boolean {
	return isIsolated(r.incoming_count, r.outgoing_count);
}

export function anyChipOn(c: ChipToggles): boolean {
	return c.due || c.unlinked || c.contested || c.forming;
}

/**
 * The intersection filter: an entry survives only if it matches EVERY active
 * chip (AND semantics — chips narrow, never expand). Entries without facts
 * (missing members) survive only when no chip is active.
 */
export function filterByChips<T>(
	entries: T[],
	factsOf: (e: T) => ChipFacts | null,
	chips: ChipToggles
): T[] {
	if (!anyChipOn(chips)) return entries;
	return entries.filter(e => {
		const r = factsOf(e);
		if (!r) return false;
		if (chips.due && !r.review_due) return false;
		if (chips.unlinked && !isUnlinked(r)) return false;
		if (chips.contested && !isContested(r)) return false;
		if (chips.forming && !isForming(r)) return false;
		return true;
	});
}
