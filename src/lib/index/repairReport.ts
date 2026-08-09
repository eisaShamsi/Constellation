/**
 * PJ-207 §11 — the repair's account of itself, on the frontend side of the wire.
 *
 * Mirrors `index_repair::RepairReport` (serde `rename_all = "camelCase"` on the
 * envelope; `ConvergeOutcome` is tagged `{ kind, value }`; `WalkReport` keeps its Rust
 * field names — it has no serde rename, verified). The JSON shape is pinned Rust-side by
 * `the_repair_report_serialises_with_the_shape_the_frontend_reads`.
 *
 * The report is rendered VERBATIM, per family — `converged(n)` / `skipped(reason)` /
 * `failed(msg)` — so a stamp-gated skip can never be presented as part of a whole
 * repair. That sentence is the reason this type exists.
 */

import { invoke } from '@tauri-apps/api/core';

/** One derived family's outcome. `skipped` reasons are diagnostic strings from Rust. */
export type ConvergeOutcome =
	| { kind: 'converged'; value: number }
	| { kind: 'skipped'; value: string }
	| { kind: 'failed'; value: string };

/** The five derived families, exactly as `converge::ConvergeReport` names them. */
export interface ConvergeReport {
	outgoing: ConvergeOutcome;
	incoming: ConvergeOutcome;
	sky: ConvergeOutcome;
	tag_counts: ConvergeOutcome;
	review: ConvergeOutcome;
}

/** The walk's tally (Rust `WalkReport` — snake_case, no serde rename). */
export interface WalkReport {
	seen: number;
	indexed: number;
	unchanged: number;
	raced: number;
	skipped: number;
	failed: number;
	dirs_unreadable: number;
	unreadable_sample: string[];
	stopped_early: boolean;
}

export interface RepairReport {
	runId: number;
	ok: boolean;
	stoppedEarly: boolean;
	walk?: WalkReport;
	converge?: ConvergeReport;
	error?: string;
}

/** The families in render order, with their report keys. */
export const CONVERGE_FAMILIES = [
	'outgoing',
	'incoming',
	'sky',
	'tag_counts',
	'review',
] as const;

/**
 * Did anything in the run fail? Judged from the REPORT, not from `ok` — the walk can
 * complete `ok: true` while a best-effort family failed (only the outgoing family
 * promotes to the run's own error), and D2's contract is "a repair reporting ≥1 failure
 * leaves the red bar".
 */
export function repairHasFailures(r: RepairReport | null | undefined): boolean {
	if (!r) return false;
	if (!r.ok || r.error) return true;
	if (r.walk && r.walk.failed > 0) return true;
	if (r.converge) {
		for (const fam of CONVERGE_FAMILIES) {
			if (r.converge[fam]?.kind === 'failed') return true;
		}
	}
	return false;
}

/**
 * The last Full run's report — recover-on-mount for the Settings section (the
 * `classifier_scan_status` discipline). `null` when no repair has run this session.
 */
export async function loadLastRepairReport(): Promise<RepairReport | null> {
	try {
		return (await invoke<RepairReport | null>('index_repair_last_report')) ?? null;
	} catch {
		return null;
	}
}

/**
 * Submit a whole-universe repair through the ONE door (`constellation_search_init` →
 * the single-flight runner). Shared by every repair button — the drift notice, the
 * Settings control, and §13's removal offer when it comes — so the "blocked is the one
 * refusal with no run behind it" decision lives once.
 *
 * Returns `true` when a run is going somewhere (started / queued / already running —
 * the caller keeps its busy state; the `index-repair:done` event releases it), `false`
 * when nothing is running (blocked, or the invoke itself failed) — the caller re-enables
 * its button and surfaces `onBlocked`'s reason on its own error surface.
 */
export async function submitRepair(
	onBlocked: (reason: string) => void,
	opts?: { fullReread?: boolean }
): Promise<boolean> {
	try {
		// PJ-207 §14 — one door, two scopes. The parameter is optional so every existing
		// caller keeps its exact call shape AND its exact meaning; only the Settings
		// "Full re-read" control passes it. Rust refuses that scope outright unless its
		// own gate is open, so this argument cannot smuggle the feature in.
		const outcome = await invoke<{ kind: string; reason?: string }>('constellation_search_init', {
			fullReread: opts?.fullReread ?? false,
		});
		if (outcome?.kind === 'blocked') {
			onBlocked(outcome.reason ?? '');
			return false;
		}
		return true;
	} catch {
		return false;
	}
}
