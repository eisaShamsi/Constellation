/**
 * Typing Latency Benchmark
 *
 * Measures the time between a keystroke and the editor processing it.
 * Inject this into the CM6 updateListener to measure real-world latency.
 *
 * Usage: import { createLatencyTracker } from './typing-latency';
 *        const tracker = createLatencyTracker();
 *        // In CM6 updateListener:
 *        if (update.docChanged) tracker.record();
 *        // After test:
 *        tracker.report();
 *
 * Target: < 5ms average processing latency (from eNotePane spec)
 */

export interface LatencyReport {
	samples: number;
	average: number;
	median: number;
	p95: number;
	p99: number;
	max: number;
	min: number;
	pass: boolean; // true if average < 5ms
}

export function createLatencyTracker() {
	const times: number[] = [];
	let lastKeystroke = 0;

	return {
		/** Call on keydown (before CM6 processes) */
		keystroke() {
			lastKeystroke = performance.now();
		},

		/** Call in CM6 updateListener when docChanged */
		record() {
			if (lastKeystroke > 0) {
				const latency = performance.now() - lastKeystroke;
				times.push(latency);
				lastKeystroke = 0;
			}
		},

		/** Generate report */
		report(): LatencyReport {
			if (times.length === 0) {
				return { samples: 0, average: 0, median: 0, p95: 0, p99: 0, max: 0, min: 0, pass: true };
			}

			const sorted = [...times].sort((a, b) => a - b);
			const sum = sorted.reduce((a, b) => a + b, 0);
			const avg = sum / sorted.length;

			return {
				samples: sorted.length,
				average: Math.round(avg * 100) / 100,
				median: Math.round(sorted[Math.floor(sorted.length / 2)] * 100) / 100,
				p95: Math.round(sorted[Math.floor(sorted.length * 0.95)] * 100) / 100,
				p99: Math.round(sorted[Math.floor(sorted.length * 0.99)] * 100) / 100,
				max: Math.round(sorted[sorted.length - 1] * 100) / 100,
				min: Math.round(sorted[0] * 100) / 100,
				pass: avg < 5, // eNotePane spec: < 5ms
			};
		},

		/** Reset for new test */
		reset() {
			times.length = 0;
			lastKeystroke = 0;
		},

		/** Get live count */
		count() { return times.length; },
	};
}
