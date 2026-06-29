<script lang="ts">
	import { onMount } from 'svelte';
	import { dir, isRTL } from '$lib/i18n';
	// MenuItem (separator / disabled / submenu) is the canonical shape from the
	// shared builder — import it so the renderer and the builder never drift.
	// (MIG-077 A0 separator/disabled; §F-sub adds the one-level `submenu` fly-out.)
	import { type MenuItem } from './contextMenuBuilder';

	let {
		x,
		y,
		items,
		onClose
	}: {
		x: number;
		y: number;
		items: MenuItem[];
		onClose: () => void;
	} = $props();

	let menuEl: HTMLDivElement;
	// Which item's fly-out is open (index into `items`), or null.
	let openSubIdx = $state<number | null>(null);

	onMount(() => {
		function handleClickOutside(e: MouseEvent) {
			if (menuEl && !menuEl.contains(e.target as Node)) {
				onClose();
			}
		}
		function handleEscape(e: KeyboardEvent) {
			if (e.key === 'Escape') onClose();
		}
		// Delay slightly to avoid the same click that opened the menu.
		// Track the timer so we can cancel it if the component unmounts first.
		let openTimer: ReturnType<typeof setTimeout> | null = setTimeout(() => {
			openTimer = null;
			document.addEventListener('click', handleClickOutside);
			document.addEventListener('keydown', handleEscape);
		}, 10);

		return () => {
			if (openTimer !== null) clearTimeout(openTimer);
			document.removeEventListener('click', handleClickOutside);
			document.removeEventListener('keydown', handleEscape);
		};
	});

	// Adjust position so the menu doesn't overflow the viewport. MIG-077 RTL: in an
	// RTL UI the menu opens toward the start side (right edge anchored at the
	// cursor), mirroring GraphMind / EditorContextMenu; LTR is unchanged.
	const MENU_W = 180;
	const adjustedX = $derived(Math.min(x, window.innerWidth - MENU_W));
	const rtlRight = $derived(Math.min(window.innerWidth - x, window.innerWidth - MENU_W));
	const adjustedY = $derived(Math.min(y, window.innerHeight - items.length * 32 - 16));

	// MIG-077 §F — flip a fly-out submenu to the LEFT when there's no room on the
	// right (else it truncates at the viewport edge). Space-based → works LTR + RTL.
	const SUBMENU_W = 180;
	let subFlip = $state(false);
	function openSubmenu(i: number) {
		openSubIdx = i;
		subFlip = false;
		queueMicrotask(() => {
			if (menuEl) subFlip = (menuEl.getBoundingClientRect().right + SUBMENU_W) > window.innerWidth;
		});
	}
</script>

<div
	class="ctx-menu"
	bind:this={menuEl}
	dir={$dir}
	style="{$isRTL ? `right: ${rtlRight}px` : `left: ${adjustedX}px`}; top: {adjustedY}px;"
>
	{#each items as item, i}
		{#if item.separator}
			<div class="ctx-separator"></div>
		{:else if item.submenu && item.submenu.length}
			<!-- svelte-ignore a11y_no_static_element_interactions a11y_mouse_events_have_key_events -->
			<div class="ctx-sub-wrap" onmouseenter={() => openSubmenu(i)} onmouseleave={() => { if (openSubIdx === i) openSubIdx = null; }}>
				<button
					class="ctx-item ctx-has-sub"
					class:danger={item.danger}
					disabled={item.disabled}
					onclick={() => (openSubIdx === i ? (openSubIdx = null) : openSubmenu(i))}
				>
					{#if item.icon}<span class="ctx-icon">{item.icon}</span>{/if}
					<span class="ctx-label">{item.label}</span>
					<span class="ctx-chevron">{$isRTL ? '‹' : '›'}</span>
				</button>
				{#if openSubIdx === i}
					<div class="ctx-submenu" class:flip={subFlip}>
						{#each item.submenu as sub}
							{#if sub.separator}
								<div class="ctx-separator"></div>
							{:else}
								<button
									class="ctx-item"
									class:danger={sub.danger}
									disabled={sub.disabled}
									onclick={() => { sub.action?.(); onClose(); }}
								>
									{#if sub.icon}<span class="ctx-icon">{sub.icon}</span>{/if}
									<span class="ctx-label">{sub.label}</span>
								</button>
							{/if}
						{/each}
					</div>
				{/if}
			</div>
		{:else}
			<button
				class="ctx-item"
				class:danger={item.danger}
				disabled={item.disabled}
				onclick={() => { item.action?.(); onClose(); }}
			>
				{#if item.icon}<span class="ctx-icon">{item.icon}</span>{/if}
				<span class="ctx-label">{item.label}</span>
			</button>
		{/if}
	{/each}
</div>

<style>
	.ctx-menu {
		position: fixed;
		z-index: 1000;
		min-width: 160px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		box-shadow: var(--shadow-l);
		padding: 4px;
		display: flex;
		flex-direction: column;
	}
	.ctx-item {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 6px 10px;
		border: none;
		background: none;
		font-size: 0.82rem;
		font-family: inherit;
		color: var(--text-normal);
		cursor: pointer;
		border-radius: 4px;
		text-align: start;
	}
	.ctx-item:hover {
		background: var(--background-modifier-hover);
	}
	.ctx-item.danger {
		color: var(--text-error);
	}
	.ctx-item.danger:hover {
		background: var(--background-modifier-error-hover);
	}
	.ctx-icon {
		font-size: 0.9rem;
		width: 18px;
		text-align: center;
		flex-shrink: 0;
	}
	/* MIG-077 §F-sub — submenu fly-out. The wrapper anchors the absolute child;
	   `inset-inline-start: 100%` opens toward the reading direction (auto-flips in
	   RTL because the menu carries dir). The chevron + label fill the row. */
	.ctx-label { flex: 1; min-width: 0; }
	.ctx-chevron { flex-shrink: 0; opacity: 0.6; font-size: 0.9rem; }
	.ctx-has-sub:hover, .ctx-sub-wrap:hover > .ctx-has-sub { background: var(--background-modifier-hover); }
	.ctx-sub-wrap { position: relative; display: flex; }
	.ctx-sub-wrap > .ctx-item { width: 100%; }
	.ctx-submenu {
		position: absolute;
		inset-block-start: -4px;
		left: 100%;
		right: auto;
		min-width: 180px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		box-shadow: var(--shadow-l);
		padding: 4px;
		display: flex;
		flex-direction: column;
		z-index: 1001;
	}
	/* Flip the fly-out to the left when there's no room on the right (set by JS). */
	.ctx-submenu.flip {
		left: auto;
		right: 100%;
	}
	/* MIG-077 A0 — separator + disabled */
	.ctx-separator {
		height: 1px;
		margin: 4px 6px;
		background: var(--background-modifier-border);
	}
	.ctx-item:disabled {
		opacity: 0.4;
		cursor: default;
	}
	.ctx-item:disabled:hover {
		background: none;
	}
</style>
