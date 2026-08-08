/**
 * PJ-207 §10 — the ONE state machine behind every background-job progress strip.
 *
 * Before this module, `ClassifierScanProgressStrip.svelte` and
 * `NscBackfillProgressStrip.svelte` were 159 lines each and byte-equivalent modulo six
 * identifiers — the copy-paste-and-adapt shape the standing rules forbid, one step before
 * §11's repair strip would have made it three copies. The `.svelte` shell
 * (`JobProgressStrip.svelte`) now owns only the wiring (listen / invoke / markup); every
 * DECISION lives here, where vitest can reach it — this repo has no component-mount
 * harness (vitest only, no jsdom/testing-library), so a testable strip means a plain-TS
 * core, not a new test stack.
 *
 * The contract both Rust jobs already share, verified field-for-field
 * (`classifier/scan_job.rs::ScanStatus`, `nsc/backfill.rs::NscBackfillStatus`, and the
 * matching event payloads):
 *
 *   event  { phase: start|progress|done|cancelled|error, total, completed, error }
 *   status { running, cancelling, completed, total, last_error }
 *
 * §11's repair strip is the intended third consumer. Its status command
 * (`index_repair_status`) already returns a superset of `JobStatus` and plugs in as-is;
 * its EVENT side does not exist yet — the repair currently emits only `index-repair:done`
 * in a different shape, so §11 must add a progress event carrying `JobProgressEvent`
 * (the natural emit site is `index_repair::note_progress`). That is §11's obligation,
 * not a licence to widen this contract.
 */

/** The five phases every job strip understands. */
export type JobPhase = 'start' | 'progress' | 'done' | 'cancelled' | 'error';

/** The Tauri event payload both Rust jobs emit. */
export interface JobProgressEvent {
	phase: JobPhase;
	total: number;
	completed: number;
	error: string | null;
}

/** The status-command snapshot both Rust jobs return (recover-on-mount). */
export interface JobStatus {
	running: boolean;
	cancelling: boolean;
	completed: number;
	total: number;
	last_error: string | null;
}

/** What the strip renders. One immutable snapshot per change. */
export interface StripState {
	visible: boolean;
	phase: JobPhase | null;
	total: number;
	completed: number;
	cancelling: boolean;
}

/** How long a finished strip stays on screen so the user sees the final count. */
export const LINGER_MS = 4000;

export const HIDDEN: StripState = Object.freeze({
	visible: false,
	phase: null,
	total: 0,
	completed: 0,
	cancelling: false,
});

/**
 * The controller. `onChange` receives a fresh snapshot after every decision; the Svelte
 * shell assigns it into one `$state` and renders. Timers run on the global clock so
 * vitest's fake timers govern them in tests.
 *
 * Behaviour is the two clones' behaviour, preserved exactly — including the pre-existing
 * quirk that a `progress` event clears `cancelling` (so the button re-enables until the
 * job actually stops). An extraction is not the place to change behaviour; the quirk is
 * pinned by test so a future change to it is a decision, not a drift.
 */
export function createJobStripController(onChange: (s: StripState) => void) {
	let state: StripState = { ...HIDDEN };
	let hideTimer: ReturnType<typeof setTimeout> | null = null;
	let destroyed = false;

	const emit = () => {
		if (!destroyed) onChange({ ...state });
	};
	const clearHide = () => {
		if (hideTimer) {
			clearTimeout(hideTimer);
			hideTimer = null;
		}
	};

	return {
		/** A `{eventName}` Tauri event arrived. */
		handleEvent(p: JobProgressEvent) {
			state.phase = p.phase;
			state.total = p.total;
			state.completed = p.completed;
			if (p.phase === 'start' || p.phase === 'progress') {
				state.visible = true;
				state.cancelling = false;
				clearHide();
			} else {
				// done | cancelled | error: keep the final count on screen for a beat,
				// then hide. A new start before the beat ends must cancel the hide —
				// otherwise back-to-back jobs blink the strip away mid-run.
				clearHide();
				hideTimer = setTimeout(() => {
					hideTimer = null;
					state = { ...HIDDEN };
					emit();
				}, LINGER_MS);
			}
			emit();
		},

		/**
		 * The status command answered on mount — covers the strip mounting late or the
		 * user navigating away and back mid-job. A job that is NOT running adopts
		 * nothing: mounting must never resurrect a finished job's counts.
		 */
		adoptStatus(s: JobStatus) {
			if (!s.running) return;
			state = {
				visible: true,
				phase: 'progress',
				total: s.total,
				completed: s.completed,
				cancelling: s.cancelling,
			};
			emit();
		},

		/** The user pressed Cancel; reflect it immediately, before the invoke resolves. */
		markCancelling() {
			state.cancelling = true;
			emit();
		},

		/** onDestroy: no timer may outlive the component (Rule 4). */
		destroy() {
			destroyed = true;
			clearHide();
		},
	};
}
