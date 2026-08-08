/**
 * PJ-207 §9 — what Constellation noticed had changed on disk while it was closed.
 *
 * The numbers are produced by the boot reconcile (`src-tauri/src/reconcile.rs`), which
 * has walked exactly these roots on every launch since MIG-078 and reported what it found
 * to `diagnostics.log` and nowhere else. On the Boss's live universe that silence was 825
 * notes absent from the search index — detected four separate times on 2026-08-07, told to
 * nobody. This module is the wire between that pass and a surface a person can read.
 *
 * Shared rather than inlined in `+layout.svelte` because §11 gives the second screen the
 * same listener, and a second copy of a payload shape is how the two drift apart.
 */

import { invoke } from '@tauri-apps/api/core';

/** The event the boot pass pushes when — and only when — it has something to report. */
export const DRIFT_REPORT_EVENT = 'index-drift:report';

/**
 * Mirrors `reconcile::DriftReport` (serde `rename_all = "camelCase"`).
 *
 * Every count is the RESIDUAL: what is still true after the boot pass healed whatever it
 * could. A file it re-adopted is not reported as missing, because by the time anyone reads
 * the notice it is not.
 */
export interface DriftReport {
	/** File and row both exist; their timestamps disagree. Edited outside Constellation. */
	drifted: number;
	/** A `.md` in one of your libraries with no row in the index — invisible to search. */
	missingFromIndex: number;
	/** A row whose file is gone, under a library that is currently reachable. */
	missingOnDisk: number;
	/** Rows whose path belongs to a linked universe (§13 may offer to remove these). */
	foreignRows: number;
	unchanged: number;
	filesSeen: number;
	rowsSeen: number;
	/** False when a folder could not be listed — so "nothing found" may be incomplete. */
	walkComplete: boolean;
	dirsUnreadable: number;
	filesUnreadable: number;
}

/**
 * Is there anything worth telling the user? Mirrors `DriftReport::has_findings`
 * (`src-tauri/src/reconcile.rs`) — Rust decides whether to EMIT, this decides whether to
 * RENDER, and they must agree. If either gains a condition, so must the other.
 *
 * A type predicate so a caller that has narrowed with it does not then need a second
 * null check the compiler already knows is dead.
 */
export function hasFindings(r: DriftReport | null | undefined): r is DriftReport {
	return (
		!!r &&
		(r.drifted > 0 ||
			r.missingFromIndex > 0 ||
			r.missingOnDisk > 0 ||
			// "I could not look" is a finding — otherwise a library with one unreadable
			// folder renders as a clean launch while its notes are absent from search,
			// and the "Some folders could not be read" sentence below is unreachable.
			r.dirsUnreadable > 0 ||
			r.filesUnreadable > 0)
	);
}

/**
 * Read the report the boot pass already produced. Starts nothing and walks nothing.
 *
 * `null` means "no answer yet" — the pass runs on a background thread and may still be
 * going. It does NOT mean "all clear", which is why the caller listens for the event too
 * rather than treating one null as a verdict.
 */
export async function loadDriftReport(): Promise<DriftReport | null> {
	try {
		return (await invoke<DriftReport | null>('index_drift_report')) ?? null;
	} catch {
		return null;
	}
}
