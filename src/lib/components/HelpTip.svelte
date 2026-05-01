<script lang="ts">
	// §119: small "?" affordance that surfaces an explanation on hover or
	// click-to-pin. Used across the 360.3D matrix to teach a first-time
	// reader what each stratum / link type / dimension means without
	// pushing the explanations into the visual itself.
	//
	// Hover model:
	//   mouseenter on the trigger → tooltip floats above (or below) the ? icon
	//   mouseleave → tooltip hides UNLESS pinned
	//
	// Click model:
	//   click on trigger → toggle pinned state; tooltip stays visible
	//   click outside trigger AND outside tooltip → unpin + hide
	//
	// Tooltip uses position: fixed driven by getBoundingClientRect() so it
	// escapes overflow: hidden on the matrix and matrix-cell boundaries.

	let {
		tooltip = '',
		position = 'top' as 'top' | 'bottom',
	}: {
		tooltip: string;
		position?: 'top' | 'bottom';
	} = $props();

	let triggerEl: HTMLButtonElement | null = $state(null);
	let tooltipEl: HTMLDivElement | null = $state(null);
	let visible = $state(false);
	let pinned = $state(false);
	let coords = $state<{ x: number; y: number } | null>(null);

	function computeCoords() {
		if (!triggerEl) return;
		const rect = triggerEl.getBoundingClientRect();
		// Clamp x to the viewport so the tooltip's transform: translate(-50%)
		// doesn't push the right or left edge off-screen. Use the tooltip's
		// max-width (380 px from CSS) as the conservative half-width.
		const halfWidth = 200; // 380 / 2 + a touch of breathing room
		const margin = 12;
		const vw = typeof window !== 'undefined' ? window.innerWidth : 1920;
		let x = rect.left + rect.width / 2;
		const minX = halfWidth + margin;
		const maxX = vw - halfWidth - margin;
		if (x < minX) x = minX;
		if (x > maxX) x = maxX;
		coords = {
			x,
			y: position === 'top' ? rect.top : rect.bottom,
		};
	}

	function showOnHover() {
		computeCoords();
		visible = true;
	}

	function hideOnLeave() {
		if (!pinned) visible = false;
	}

	function togglePin(e: MouseEvent) {
		e.stopPropagation();
		if (pinned) {
			pinned = false;
			visible = false;
		} else {
			computeCoords();
			pinned = true;
			visible = true;
		}
	}

	$effect(() => {
		if (!pinned) return;
		const handler = (e: MouseEvent) => {
			const target = e.target as Node;
			if (triggerEl?.contains(target)) return;
			if (tooltipEl?.contains(target)) return;
			pinned = false;
			visible = false;
		};
		document.addEventListener('click', handler);
		return () => document.removeEventListener('click', handler);
	});
</script>

<button
	bind:this={triggerEl}
	class="help-tip"
	class:pinned
	type="button"
	aria-label="Help"
	onmouseenter={showOnHover}
	onmouseleave={hideOnLeave}
	onclick={togglePin}
>?</button>

{#if visible && coords}
	<div
		bind:this={tooltipEl}
		class="help-tooltip"
		class:pinned
		data-position={position}
		style="left: {coords.x}px; top: {coords.y}px;"
		role="tooltip"
	>
		{tooltip}
	</div>
{/if}

<style>
	.help-tip {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 14px;
		height: 14px;
		border-radius: 50%;
		background: transparent;
		border: 1px solid var(--text-muted);
		color: var(--text-muted);
		font-size: 9px;
		font-weight: 700;
		line-height: 1;
		cursor: help;
		padding: 0;
		opacity: 0.55;
		transition: opacity 0.15s ease, color 0.15s ease, border-color 0.15s ease;
		flex-shrink: 0;
		vertical-align: middle;
	}
	.help-tip:hover,
	.help-tip.pinned {
		opacity: 1;
		color: var(--text-accent);
		border-color: var(--text-accent);
	}

	.help-tooltip {
		position: fixed;
		padding: 10px 14px;
		background: var(--background-secondary);
		border: 1px solid var(--background-modifier-border-focus);
		border-radius: 8px;
		color: var(--text-normal);
		font-size: 14px;
		font-weight: 400;
		line-height: 1.55;
		letter-spacing: normal;
		text-transform: none; /* override any uppercase from ancestor labels */
		max-width: 380px;
		min-width: 240px;
		z-index: 9999;
		pointer-events: none;
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
		white-space: normal;
	}
	.help-tooltip[data-position='top'] {
		transform: translate(-50%, calc(-100% - 10px));
	}
	.help-tooltip[data-position='bottom'] {
		transform: translate(-50%, 10px);
	}
	.help-tooltip.pinned {
		pointer-events: auto;
		border-color: var(--text-accent);
	}
</style>
