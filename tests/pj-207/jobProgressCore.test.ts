/**
 * PJ-207 §10 — the shared job-progress strip's state machine.
 *
 * Why these tests exist: the extraction collapsed two byte-equivalent 159-line strips
 * (classifier scan, NSC backfill) into ONE implementation that §11's repair strip will
 * also consume — so after §10, one regression here breaks all three consumers at once,
 * and before §10 nothing under tests/ mentioned ProgressStrip at all (verified).
 *
 * The two behaviours the plan names — recover-on-mount and the 4 s linger — are pinned
 * against `jobProgressCore.ts`, the plain-TS module that owns every decision. The .svelte
 * shell is deliberately too thin to hide logic: this repo has no component-mount harness
 * (vitest only — no jsdom, no testing-library), and adding one is its own decision, not
 * §10's. The wiring the shell keeps (listen → handleEvent, invoke(status) → adoptStatus)
 * is five readable lines.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
	createJobStripController,
	HIDDEN,
	LINGER_MS,
	type JobStatus,
	type StripState,
} from '../../src/lib/components/jobProgressCore';

const runningStatus = (over: Partial<JobStatus> = {}): JobStatus => ({
	running: true,
	cancelling: false,
	completed: 120,
	total: 7824,
	last_error: null,
	...over,
});

function harness() {
	const states: StripState[] = [];
	const ctl = createJobStripController((s) => states.push(s));
	return { ctl, states, last: () => states[states.length - 1] };
}

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

describe('recover-on-mount (the invoke(statusCommand) path)', () => {
	it('a RUNNING job is adopted: visible, in progress, counts and cancelling taken as-is', () => {
		const { ctl, last } = harness();
		ctl.adoptStatus(runningStatus({ completed: 4200, total: 7824, cancelling: true }));
		expect(last()).toEqual({
			visible: true,
			phase: 'progress',
			total: 7824,
			completed: 4200,
			cancelling: true,
		});
	});

	it('a job that is NOT running adopts nothing — a late mount must never resurrect a finished job', () => {
		const { ctl, states } = harness();
		ctl.adoptStatus(runningStatus({ running: false, completed: 7824, total: 7824 }));
		expect(states).toHaveLength(0); // not even a hidden emit — nothing changed
	});
});

describe('the 4 s linger', () => {
	it('done keeps the final count on screen, then hides exactly at LINGER_MS', () => {
		const { ctl, last } = harness();
		ctl.handleEvent({ phase: 'start', total: 100, completed: 0, error: null });
		ctl.handleEvent({ phase: 'done', total: 100, completed: 100, error: null });
		expect(last().visible).toBe(true); // the user sees the final count…
		expect(last().completed).toBe(100);

		vi.advanceTimersByTime(LINGER_MS - 1);
		expect(last().visible).toBe(true); // …for the whole beat…
		vi.advanceTimersByTime(1);
		expect(last()).toEqual(HIDDEN); // …and not a millisecond longer.
	});

	it('a new start DURING the linger cancels the pending hide — back-to-back jobs must not blink away mid-run', () => {
		const { ctl, last } = harness();
		ctl.handleEvent({ phase: 'done', total: 50, completed: 50, error: null });
		vi.advanceTimersByTime(LINGER_MS / 2);
		ctl.handleEvent({ phase: 'start', total: 200, completed: 0, error: null });

		vi.advanceTimersByTime(LINGER_MS * 2); // well past the abandoned deadline
		expect(last().visible).toBe(true);
		expect(last().total).toBe(200); // the SECOND job's run, uninterrupted
	});

	it('cancelled and error linger identically (one timer, all three terminal phases)', () => {
		for (const phase of ['cancelled', 'error'] as const) {
			const { ctl, last } = harness();
			ctl.handleEvent({ phase: 'start', total: 10, completed: 0, error: null });
			ctl.handleEvent({ phase, total: 10, completed: 3, error: phase === 'error' ? 'boom' : null });
			expect(last().visible).toBe(true);
			vi.advanceTimersByTime(LINGER_MS);
			expect(last()).toEqual(HIDDEN);
		}
	});

	it('PRESERVED BEHAVIOUR: a terminal event with NO preceding start never shows the strip — a job with nothing to do must not flash a "done" at every launch', () => {
		const { ctl, last, states } = harness();
		ctl.handleEvent({ phase: 'done', total: 0, completed: 0, error: null });
		expect(last().visible).toBe(false); // phase/counts recorded, visibility never granted
		vi.advanceTimersByTime(LINGER_MS);
		expect(states.every((s) => !s.visible)).toBe(true);
	});

	it('destroy() clears the pending hide — no timer outlives the component (Rule 4)', () => {
		const { ctl, states } = harness();
		ctl.handleEvent({ phase: 'done', total: 10, completed: 10, error: null });
		const emitted = states.length;
		ctl.destroy();
		vi.advanceTimersByTime(LINGER_MS * 2);
		expect(states).toHaveLength(emitted); // nothing fired after teardown
		expect(vi.getTimerCount()).toBe(0);
	});
});

describe('cancel', () => {
	it('markCancelling reflects the click immediately, before the invoke resolves', () => {
		const { ctl, last } = harness();
		ctl.handleEvent({ phase: 'progress', total: 100, completed: 10, error: null });
		ctl.markCancelling();
		expect(last().cancelling).toBe(true);
	});

	it('PRESERVED QUIRK: a progress event clears cancelling (the button re-enables until the job actually stops) — both original strips behaved this way; changing it is a decision, not a drift', () => {
		const { ctl, last } = harness();
		ctl.markCancelling();
		ctl.handleEvent({ phase: 'progress', total: 100, completed: 11, error: null });
		expect(last().cancelling).toBe(false);
	});
});
