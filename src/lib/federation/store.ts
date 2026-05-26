/**
 * MIG-056 §H — TS bridge for the cross-universe federation warning surface.
 *
 * The status-bar badge + popup in `+layout.svelte` poll this to surface
 * cUniverses that failed to attach for federation (skip_unavailable
 * model — Architect §5.2). User can hover/click the badge to see WHICH
 * cUniverses are unavailable + WHY.
 *
 * Mirrors the Rust `federation::FederationWarning` struct verbatim.
 */

import { invoke } from '@tauri-apps/api/core';

/** A single cUniverse-failed-to-attach warning. */
export interface FederationWarning {
	/** Filesystem path of the cUniverse root (the universe directory). */
	cuniverse_path: string;
	/** Human-readable reason (e.g., "search.db missing", "locked by another process"). */
	reason: string;
	/** Unix-second timestamp when the warning was emitted. */
	when_unix: number;
}

/**
 * Fetch the current federation warnings. Returns an empty array when:
 * - Federation isn't ready yet (boot still in progress)
 * - No cUniverses are linked
 * - All cUniverses attached cleanly
 *
 * Cheap to call (just a Mutex lock + clone on the Rust side); safe to
 * poll on a short interval if needed.
 */
export async function getFederationWarnings(): Promise<FederationWarning[]> {
	return await invoke<FederationWarning[]>('federation_get_warnings');
}
