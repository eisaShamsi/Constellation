<!--
  MIG-025 §A.11 — Sight v6 first-boot orientation tour.

  Per Concept Paper v4.0 §5 + §11 invariant 10:
    - 4 skippable steps
    - Auto-skipped on subsequent opens (gated on `appSettings.sight.tourSeen`)
    - Always re-available via Help → Sight tour (clears tourSeen flag —
      wired in §C.10 when the tradition chip ships)

  v6.0 ships a centered card layout (no spotlight effect on the
  underlying canvas — that's a v4.1 polish target). Esc dismisses;
  click-outside does NOT dismiss (so progress isn't lost accidentally).
-->
<script lang="ts">
	import { onMount, onDestroy } from 'svelte';

	let {
		onComplete,
	}: {
		onComplete: () => void;
	} = $props();

	const STEPS = [
		{
			title: 'Welcome to Sight',
			body:
				'This is your knowledge universe. Each star is a note; ' +
				'position shows where it lives in your thinking — radial ' +
				'distance from the center marks the stratum (Foundation ' +
				'innermost, Edge of Knowing at the rim), and the angle ' +
				'around the dome marks the month you wrote it.',
		},
		{
			title: 'Inspect a star',
			body:
				'Hover any star for detail — its path appears at the bottom ' +
				'of the dome. Click to open the note in the editor.',
		},
		{
			title: 'Filter your view',
			body:
				'The tab on the left edge opens the facet sidebar — six ' +
				'lenses (Folder, Library, Stratum, Confidence, Stage, ' +
				'Provenance) that filter the dome to just the notes you ' +
				'want to see. Counts rebalance as you click.',
		},
		{
			title: 'Reset and search',
			body:
				'Esc clears any active filter or hover. Cmd-F (or Ctrl-F) ' +
				'will open a search overlay in a future Phase. Try ' +
				'right-clicking a stratum band to isolate just that band ' +
				'when you want to focus deep.',
		},
	];

	let stepIndex = $state(0);

	function next(): void {
		if (stepIndex < STEPS.length - 1) {
			stepIndex++;
		} else {
			onComplete();
		}
	}

	function skip(): void {
		onComplete();
	}

	function handleKey(e: KeyboardEvent): void {
		if (e.key === 'Escape') {
			e.preventDefault();
			e.stopPropagation();
			skip();
		} else if (e.key === 'ArrowRight' || e.key === 'Enter') {
			e.preventDefault();
			next();
		}
	}

	onMount(() => {
		window.addEventListener('keydown', handleKey, true);
	});

	onDestroy(() => {
		window.removeEventListener('keydown', handleKey, true);
	});
</script>

<div class="sight-v6-tour-overlay" role="dialog" aria-modal="true" aria-labelledby="sight-v6-tour-title">
	<div class="sight-v6-tour-card">
		<div class="sight-v6-tour-step">Step {stepIndex + 1} of {STEPS.length}</div>
		<h2 id="sight-v6-tour-title" class="sight-v6-tour-title">{STEPS[stepIndex].title}</h2>
		<p class="sight-v6-tour-body">{STEPS[stepIndex].body}</p>

		<div class="sight-v6-tour-pagination" aria-hidden="true">
			{#each STEPS as _, i (i)}
				<span class="sight-v6-tour-dot" class:active={i === stepIndex}></span>
			{/each}
		</div>

		<div class="sight-v6-tour-actions">
			<button type="button" class="sight-v6-tour-skip" onclick={skip}>
				Skip tour
			</button>
			<button type="button" class="sight-v6-tour-next" onclick={next}>
				{stepIndex < STEPS.length - 1 ? 'Next' : 'Done'}
			</button>
		</div>
	</div>
</div>

<style>
	.sight-v6-tour-overlay {
		position: absolute;
		inset: 0;
		background: rgba(8, 12, 22, 0.78);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 10;
		pointer-events: auto;
	}

	.sight-v6-tour-card {
		max-width: 440px;
		background: #0c1322;
		border: 1px solid #2a3245;
		border-radius: 8px;
		padding: 24px 28px 20px;
		color: #cdd5e0;
		box-shadow: 0 4px 24px rgba(0, 0, 0, 0.5);
		font-family: var(--interface-font, 'Inter', system-ui, sans-serif);
	}

	.sight-v6-tour-step {
		font-size: 10px;
		color: #5a6275;
		letter-spacing: 1px;
		text-transform: uppercase;
		margin-bottom: 4px;
	}

	.sight-v6-tour-title {
		margin: 0 0 12px 0;
		font-size: 18px;
		font-weight: 500;
		color: #e8ebf2;
		letter-spacing: 0.3px;
	}

	.sight-v6-tour-body {
		margin: 0 0 18px 0;
		font-size: 13px;
		line-height: 1.5;
		color: #a0aabe;
	}

	.sight-v6-tour-pagination {
		display: flex;
		justify-content: center;
		gap: 6px;
		margin-bottom: 18px;
	}
	.sight-v6-tour-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: #2a3245;
	}
	.sight-v6-tour-dot.active {
		background: #7dd3fc;
	}

	.sight-v6-tour-actions {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.sight-v6-tour-skip {
		background: transparent;
		border: 0;
		color: #7a8295;
		cursor: pointer;
		font-size: 11px;
		padding: 6px 8px;
		font-family: inherit;
	}
	.sight-v6-tour-skip:hover { color: #cdd5e0; }

	.sight-v6-tour-next {
		background: #1e2a45;
		border: 1px solid #3b5998;
		color: #7dd3fc;
		cursor: pointer;
		font-size: 12px;
		font-weight: 500;
		padding: 8px 18px;
		border-radius: 4px;
		font-family: inherit;
		transition: background 0.12s;
	}
	.sight-v6-tour-next:hover {
		background: #25325a;
	}
</style>
