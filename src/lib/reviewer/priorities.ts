// MIG-084 §F.2 — the Reviewer's COMPUTED-PRIORITY engine.
//
// A pure, deterministic, two-axis (Eisenhower) urgency × importance score derived from
// the signals each DueNote already carries — no IPC, no storage, read-time O(1). The
// score renders as an additive "recipe" (the contributions sum to the number, so the
// user can read WHY it ranked where it did), and the user's override wins when set.
//
// Grounded in proven methods (design wf_9382a686-578): an FSRS-style retrievability
// decay for staleness/overdue urgency, typed-link disturbance tiers, a RICE-style reach
// term, a maturity-standing term, and a fragility flag. Weighted SUM (not product) so
// contributions are additive and the bar segments literally add up to the score.
//
// The module is i18n-free on purpose (so it unit-tests without the locale layer): it
// returns structured contributions keyed by factor + the raw human value; the component
// maps each key to a localized label.

export interface PrioritySignals {
	reason: string; // stale | interval_due | checkpoint | orphan | fragile | never_reviewed
	days_overdue: number;
	stale_trigger_type?: string | null;
	stale_changed_on?: string | null; // YYYY-MM-DD
	incoming_count: number;
	outgoing_count: number;
	maturity: string; // seed | sapling | evergreen | canonical | wilting
}

export interface Contribution {
	key: 'decay' | 'disturbance' | 'reach' | 'maturity' | 'fragility';
	axis: 'urgency' | 'importance';
	points: number; // contribution to the 0–100 score (segments sum to score)
	value: number; // the raw human input (days, link count, 0..1 standing) for the label
}

export interface ComputedPriority {
	score: number; // 0–100
	contributions: Contribution[]; // sorted high→low, negligible dropped
}

const EPOCH_2020_DAYS = 18262; // days from 1970-01-01 to 2020-01-01

/** Parse a YYYY-MM-DD date to days-since-2020 (the schedule's day-frame), or null. */
function parseDayToEpoch2020(d: string | null | undefined): number | null {
	if (!d) return null;
	const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(d);
	if (!m) return null;
	return Math.floor(Date.UTC(+m[1], +m[2] - 1, +m[3]) / 86_400_000) - EPOCH_2020_DAYS;
}

// Stability seed (days) per lens — smaller decays faster (urgent sooner). Stale decays
// fastest (a changed dependency is pressing); checkpoints/orphans are patient.
const STABILITY: Record<string, number> = {
	stale: 3, interval_due: 7, never_reviewed: 7, checkpoint: 14, orphan: 14, fragile: 10,
};
// Maturity standing (importance): canonical knowledge carries more long-term value.
const MATURITY_W: Record<string, number> = {
	canonical: 1.0, evergreen: 0.8, wilting: 0.6, sapling: 0.35, seed: 0.15,
};

// Axis + factor weights (the visible "recipe"; tunable later behind a panel).
const AXIS = { urgency: 0.6, importance: 0.4 };
const UW = { decay: 0.55, disturbance: 0.45 };
const IW = { reach: 0.5, maturity: 0.3, fragility: 0.2 };
const REACH_CAP = 12; // backlinks at which reach saturates to full importance

/** "How many days of decay" drives urgency, per lens (stale counts from the change). */
function decayDays(n: PrioritySignals, todayDay: number): number {
	if (n.reason === 'stale') {
		const d = parseDayToEpoch2020(n.stale_changed_on);
		if (d != null) return Math.max(0, todayDay - d);
	}
	return Math.max(0, n.days_overdue);
}

/** FSRS retrievability proxy: decay = 1 − R(t,S), R = (1 + t/(9S))^-1. In [0,1). */
function decay(n: PrioritySignals, todayDay: number): number {
	const S = STABILITY[n.reason] ?? 7;
	const x = decayDays(n, todayDay) / (9 * S);
	return x / (1 + x);
}

/** Dependency disturbance (stale only): a contradiction/supersession outranks support. */
function disturbance(n: PrioritySignals): number {
	if (n.reason !== 'stale') return 0;
	const t = (n.stale_trigger_type ?? '').toLowerCase();
	if (t === 'contradicts' || t === 'supersedes') return 1;
	if (t === 'derives-from' || t === 'part-of' || t === 'supports') return 0.7;
	return 0;
}

/** Reach (importance): how much leans on this; depended-on (incoming) outweighs depends-on. */
function reach(n: PrioritySignals): number {
	return Math.min((n.incoming_count + 0.5 * n.outgoing_count) / REACH_CAP, 1);
}

function maturityStanding(n: PrioritySignals): number {
	return MATURITY_W[n.maturity] ?? 0.15;
}

function fragility(n: PrioritySignals): number {
	return n.reason === 'fragile' ? 1 : 0;
}

/**
 * Compute the priority score (0–100) + its additive factor breakdown. Each contribution =
 * 100 · axisWeight · factorWeight · factorValue, so the segments sum to the score.
 */
export function computedPriority(n: PrioritySignals, todayDay: number): ComputedPriority {
	const f = {
		decay: decay(n, todayDay),
		disturbance: disturbance(n),
		reach: reach(n),
		maturity: maturityStanding(n),
		fragility: fragility(n),
	};
	const urgency = UW.decay * f.decay + UW.disturbance * f.disturbance;
	const importance = IW.reach * f.reach + IW.maturity * f.maturity + IW.fragility * f.fragility;
	const score = Math.round(100 * (AXIS.urgency * urgency + AXIS.importance * importance));

	const pts = (axisW: number, factorW: number, val: number) => 100 * axisW * factorW * val;
	const raw: Contribution[] = [
		{ key: 'decay', axis: 'urgency', points: pts(AXIS.urgency, UW.decay, f.decay), value: decayDays(n, todayDay) },
		{ key: 'disturbance', axis: 'urgency', points: pts(AXIS.urgency, UW.disturbance, f.disturbance), value: f.disturbance },
		{ key: 'reach', axis: 'importance', points: pts(AXIS.importance, IW.reach, f.reach), value: n.incoming_count },
		{ key: 'maturity', axis: 'importance', points: pts(AXIS.importance, IW.maturity, f.maturity), value: f.maturity },
		{ key: 'fragility', axis: 'importance', points: pts(AXIS.importance, IW.fragility, f.fragility), value: n.incoming_count },
	];
	// Keep all five (their points sum exactly to the score — the bar's invariant); the
	// component hides zero/negligible segments at render. Sorted high→low.
	const contributions = raw.sort((a, b) => b.points - a.points);
	return { score, contributions };
}

/** The effective priority: a user override wins; otherwise the computed score. */
export function effectivePriority(override: number | null | undefined, computed: number): number {
	return override ?? computed;
}
