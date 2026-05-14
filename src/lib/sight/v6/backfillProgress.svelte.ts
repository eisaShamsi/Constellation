/**
 * MIG-025 §A.4 — Sight v6 progressive-backfill progress store.
 *
 * Wraps the `sight-v6-backfill-progress` Tauri event stream emitted by
 * `backfill_sight_v6_layout_progressive` (Rust side). Exposes:
 *
 *   - `progress`     — latest event, or null if backfill hasn't run yet
 *   - `renderReady`  — true once `firstTierComplete` is signaled (Sight
 *                      v6 should gate its initial render on this, per
 *                      Architect §4.1 + Concept Paper §9.3)
 *   - `done`         — true once the backfill has stamped the sentinel
 *
 * Uses Svelte 5 runes via the `.svelte.ts` filename (so $state works
 * outside .svelte components per the Svelte 5 module convention).
 *
 * Event lifecycle (per backfill_sight_v6_layout_progressive):
 *   1..5 — one event per stratum tier completing.
 *          tier 1 carries first_tier_complete=true → renderReady flips.
 *   6    — final done event after sentinel stamp. done flips.
 *
 * Short-circuit (sentinel already set on previous run):
 *   1 event with done=true and first_tier_complete=true. Both flags
 *   flip together.
 */

import { listen, type UnlistenFn } from '@tauri-apps/api/event';

/**
 * Per-event payload from the Rust side. Fields match
 * `BackfillProgress` in `src-tauri/src/sight_v6.rs` with serde
 * camelCase serialization.
 */
export interface BackfillProgressEvent {
	tier: number;
	doneRows: number;
	totalRows: number;
	firstTierComplete: boolean;
	done: boolean;
}

class BackfillProgressStore {
	progress = $state<BackfillProgressEvent | null>(null);
	renderReady = $state(false);
	done = $state(false);

	#unlisten: UnlistenFn | null = null;

	/**
	 * Subscribe to the Tauri progress event stream. Idempotent: a
	 * second `start()` call is a no-op (the existing listener stays
	 * active).
	 */
	async start(): Promise<void> {
		if (this.#unlisten) return;
		this.#unlisten = await listen<BackfillProgressEvent>(
			'sight-v6-backfill-progress',
			(ev) => {
				this.progress = ev.payload;
				if (ev.payload.firstTierComplete) {
					this.renderReady = true;
				}
				if (ev.payload.done) {
					this.done = true;
				}
			}
		);
	}

	/**
	 * Tear down the Tauri listener. Call from onDestroy in the Sight
	 * v6 component to prevent leaks per CLAUDE.md Performance Rule 4.
	 */
	stop(): void {
		if (this.#unlisten) {
			this.#unlisten();
			this.#unlisten = null;
		}
	}

	/**
	 * Reset the store to its initial state. Used when the user
	 * switches Universe (the new Universe will have its own
	 * backfill state — possibly already-done from a prior session).
	 */
	reset(): void {
		this.progress = null;
		this.renderReady = false;
		this.done = false;
	}
}

/**
 * Singleton store. Sight v6 mounts call `.start()` in onMount and
 * `.stop()` in onDestroy. The renderReady gate is read as
 * `backfillProgress.renderReady` from anchor render code.
 */
export const backfillProgress = new BackfillProgressStore();
