<script lang="ts">
	/**
	 * MIG-076 §D1 — the quiesced-rename freeze overlay.
	 *
	 * A self-contained read-only overlay shown over an editor pane while that
	 * pane's note is inside a rename + wikilink-cascade window (the ~7s scan).
	 * It subscribes to the reactive `cascadeFreeze` store and renders when THIS
	 * pane's `path` falls inside it.
	 *
	 * PJ-174 #1 — that store now holds cascading LIBRARY ROOTS, not a snapshot of
	 * the tab paths open when the rename started. The snapshot could not cover a
	 * note the user opened DURING the multi-second walk (the sidebar tree is never
	 * blocked), so exactly the pane most at risk — one being rewritten under the
	 * user's cursor — was the one pane that got no overlay. Membership goes through
	 * the shared `isPathFrozen`, which normalises both sides and enforces the
	 * separator boundary.
	 *
	 * Self-contained by design (CLAUDE.md self-contained-components rule): owns
	 * its own styling, blocks all pointer/keyboard input to the pane beneath it
	 * (pointer-events:auto), and is keyed to the store — NOT to the pane's
	 * {#key reloadVersion} remount — so it survives the deliberate remount the
	 * cascade performs mid-window. Mount it as the last child of a
	 * position:relative pane container; it absolutely fills that container.
	 */
	import { cascadeFreeze, isPathFrozen } from '$lib/libraries/store';
	import { t } from '$lib/i18n';

	// Two modes: pass `path` and the overlay self-derives from the cascadeFreeze store
	// (NotePane/split/index panes, keyed by tab.path); OR pass a controlled `frozen`
	// boolean (FocusPane, which is a fixed-position self-contained surface and already
	// computes its freeze state). `frozen` wins when provided.
	let { path, frozen: frozenProp }: { path?: string | null; frozen?: boolean } = $props();
	let frozen = $derived(frozenProp ?? (!!path && isPathFrozen(path, $cascadeFreeze)));
</script>

{#if frozen}
	<div class="cascade-freeze-overlay" role="status" aria-live="polite">
		<div class="cascade-freeze-card">
			<span class="cascade-freeze-spinner" aria-hidden="true"></span>
			<span class="cascade-freeze-label" dir="auto">{$t('cascade.updating')}</span>
		</div>
	</div>
{/if}

<style>
	.cascade-freeze-overlay {
		position: absolute;
		inset: 0;
		z-index: 500; /* above the editor content, below app-modal overlays */
		display: flex;
		align-items: center;
		justify-content: center;
		background: color-mix(in srgb, var(--background-primary, #1e1e2e) 55%, transparent);
		backdrop-filter: blur(1.5px);
		pointer-events: auto; /* block every input to the frozen pane */
		cursor: progress;
		animation: cfo-fade 0.15s ease;
	}
	.cascade-freeze-card {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 14px 22px;
		border-radius: 12px;
		background: var(--background-secondary, #2a2a3a);
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.35);
		font-size: 0.95rem;
		color: var(--text-normal, #e0e0e0);
		max-width: 80%;
	}
	.cascade-freeze-spinner {
		flex: none;
		width: 18px;
		height: 18px;
		border: 2.5px solid color-mix(in srgb, var(--text-normal, #888888) 30%, transparent);
		border-top-color: var(--interactive-accent, #7c3aed);
		border-radius: 50%;
		animation: cfo-spin 0.7s linear infinite;
	}
	.cascade-freeze-label {
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	@keyframes cfo-spin {
		to { transform: rotate(360deg); }
	}
	@keyframes cfo-fade {
		from { opacity: 0; }
		to { opacity: 1; }
	}
	@media (prefers-reduced-motion: reduce) {
		.cascade-freeze-spinner { animation: none; }
		.cascade-freeze-overlay { animation: none; }
	}
</style>
