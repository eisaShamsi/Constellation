/**
 * MIG-094 (PJ-069 Step 1) — the frontend home for Constellation's named
 * structural-connectivity predicates, mirroring `src-tauri/src/connectivity.rs`.
 * Each predicate is computed from write-time `note_meta` facts (incoming_count /
 * outgoing_count / outgoing_link_types_json) — never from a graph re-walk.
 *
 * KEEP IN PARITY with the Rust module (the two are the same three concepts):
 *   UNREFERENCED — graph "source node", in-degree 0: nothing points here (may link
 *     out). Surfaces layer their OWN substance filter (e.g. word_count > 20) on top.
 *   ISOLATED     — graph "isolated vertex", degree 0: no links either direction.
 *   FRAGILE      — single-point-of-failure: many depend on it, ≤1 derives-from support
 *     (support read from outgoing_link_types_json, the write-time {type:count} map).
 *
 * Pure module — no imports that could carry an IPC (Collections' pinned invariant).
 */

/** UNREFERENCED — nothing points here (in-degree 0). */
export function isUnreferenced(incomingCount: number): boolean {
	return incomingCount === 0;
}

/** ISOLATED — no links in either direction (degree 0). Strictly stronger than UNREFERENCED. */
export function isIsolated(incomingCount: number, outgoingCount: number): boolean {
	return incomingCount === 0 && outgoingCount === 0;
}

/** FRAGILE — many dependents (≥5 incoming) resting on ≤1 derives-from support. */
export function isFragile(incomingCount: number, derivesFromSupport: number): boolean {
	return incomingCount >= 5 && derivesFromSupport <= 1;
}

/**
 * The active `derives-from` support count from a note's outgoing_link_types_json
 * (`{"type":count}`). Returns 0 when the key is absent or the JSON is malformed.
 * Occurrence-count-equivalent to the Rust `derives_from_support`.
 */
export function derivesFromSupport(outgoingLinkTypesJson: string): number {
	try {
		const o = JSON.parse(outgoingLinkTypesJson || '{}');
		const n = o['derives-from'];
		return typeof n === 'number' ? n : 0;
	} catch {
		return 0;
	}
}
