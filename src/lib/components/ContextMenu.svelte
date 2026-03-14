<script lang="ts">
	import { onMount } from 'svelte';

	interface MenuItem {
		label: string;
		icon?: string;
		action: () => void;
		danger?: boolean;
	}

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

	// Adjust position so menu doesn't overflow viewport
	const adjustedX = $derived(Math.min(x, window.innerWidth - 180));
	const adjustedY = $derived(Math.min(y, window.innerHeight - items.length * 32 - 16));
</script>

<div class="ctx-menu" bind:this={menuEl} style="left: {adjustedX}px; top: {adjustedY}px;">
	{#each items as item}
		<button
			class="ctx-item"
			class:danger={item.danger}
			onclick={() => { item.action(); onClose(); }}
		>
			{#if item.icon}<span class="ctx-icon">{item.icon}</span>{/if}
			<span>{item.label}</span>
		</button>
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
</style>
