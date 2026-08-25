/**
 * PJ-207 §9 — what Constellation noticed had changed on disk while it was closed.
 *
 * The numbers are produced by the boot reconcile (`src-tauri/src/reconcile.rs`), which
 * has walked exactly these roots on every launch since MIG-078 and reported what it found
 * to `diagnostics.log` and nowhere else. On the Boss's live universe that silence was 825
 * notes absent from the search index — detected four separate times on 2026-08-07, told to
 * nobody. This module is the wire between that pass and a surface a person can read.
 *
 * Shared rather than inlined in `+layout.svelte` so any future second consumer (the
 * second screen refreshes on `index-repair:done` instead — it renders no drift band)
 * starts from one payload shape rather than a copy.
 */

import { invoke } from '@tauri-apps/api/core';
import { writable } from 'svelte/store';

/**
 * The event the boot pass pushes after every scan — clean or not (§11: a post-repair
 * rescan must be able to REPLACE stale counts with a clean report so the band clears
 * from facts). Whether anything RENDERS is `hasFindings`' decision, below.
 */
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
	/**
	 * PJ-369 — rows whose note is gone, that belong to no library of this universe and no
	 * linked universe, and that carry no earned link or review data. They surface as search
	 * results that open nothing, plus phantom edges in the link graph, Sky View and the
	 * Reviewer. On the Boss's `Eisa Universe`: 603 rows dragging 19,472 link edges. On his
	 * DAILY universe (`Eisa Cognitive Knowledge`) the count is zero — measured, not assumed —
	 * so a clean daily boot showing no sentence is the feature working, not a failure.
	 *
	 * Counted at boot, never acted on there. Removal is offered from Settings → Index, and
	 * this is deliberately NOT part of `hasFindings` — see the note there.
	 */
	stalePhantoms: number;
}

/**
 * Is there anything worth telling the user? Mirrors `DriftReport::has_findings`
 * (`src-tauri/src/reconcile.rs`, the canonical definition of a finding). Since §11's
 * always-emit, THIS is the only live render gate — if either side gains a condition,
 * so must the other.
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
 * PJ-369 — are there stale phantom entries to offer removal for? Deliberately separate from
 * `hasFindings`, mirroring `DriftReport::has_phantoms`.
 *
 * The reason is the button. The notice band that `hasFindings` gates offers **Repair now**,
 * and a repair provably cannot fix a phantom: the repair walks libraries and re-reads files,
 * while a phantom's whole nature is that it lives under no library and has no file. Folding
 * this into `hasFindings` would put a control in front of the user that cannot act on the
 * sentence above it — the "false door" the PJ-369 design attack named. Phantoms get their own
 * sentence and their own control in Settings → Index.
 */
export function hasPhantoms(r: DriftReport | null | undefined): r is DriftReport {
	return !!r && r.stalePhantoms > 0;
}

/**
 * PJ-369 Step 3/4 — what a prune run did. Mirrors `phantom_prune::PruneReceipt`
 * (serde `rename_all = "camelCase"`).
 *
 * Every field is a count the user is entitled to see. A removal that cannot be described is a
 * silent removal, and `removed` alone would be a summary that flatters: a run that stopped on a
 * universe switch, skipped rows whose file reappeared, or failed some deletes has a real
 * residue, and the receipt has to say so.
 */
export interface PruneReceipt {
	/** Rows removed through the delete funnel, archive-first. */
	removed: number;
	/** Not removed because the second look disagreed with the first — the file reappeared, or
	 *  the row had no content id so its history could not be archived. Never an error. */
	skipped: number;
	/** Deletes that returned an error. Those rows are still there. */
	failed: number;
	/** Rows the classifier declined to judge. Never acted on. */
	unknown: number;
	/** Set when the run stopped before finishing (a universe switch). The counts are what
	 *  actually happened up to that point, not a projection. */
	stoppedEarly: string | null;
	/** Set when the whole run refused before touching anything. `removed` is then always 0. */
	refused: string | null;
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

/**
 * PJ-369 Step 4 (2026-08-24 panel) — the last prune's receipt, for the length of the session.
 *
 * It lived as component-local state, so closing Settings destroyed it. The modal can be closed
 * mid-run (its overlay closes on click) and reopening showed nothing at all: the button that
 * changes NOTHING — Repair — keeps a durable report, while the one that permanently removes
 * hundreds of rows kept none. A removal the user cannot see the outcome of is a silent removal.
 *
 * A module-level store rather than component state, so it survives close/reopen. Not persisted to
 * disk on purpose: it describes one run in one session, and a receipt that outlived the app would
 * start asserting a past it can no longer vouch for.
 */
export const lastPruneReceipt = writable<PruneReceipt | null>(null);
