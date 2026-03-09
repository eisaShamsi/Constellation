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
		// Delay slightly to avoid the same click that opened the menu
		setTimeout(() => {
			document.addEventListener('click', handleClickOutside);
			document.addEventListener('keydown', handleEscape);
		}, 10);

		return () => {
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
		background: #fff;
		border: 1px solid #e0e0e4;
		border-radius: 6px;
		box-shadow: 0 4px 16px rgba(0,0,0,0.12);
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
		color: #1f2328;
		cursor: pointer;
		border-radius: 4px;
		text-align: start;
	}
	.ctx-item:hover {
		background: #f0f0f4;
	}
	.ctx-item.danger {
		color: #cf222e;
	}
	.ctx-item.danger:hover {
		background: #fef1f2;
	}
	.ctx-icon {
		font-size: 0.9rem;
		width: 18px;
		text-align: center;
		flex-shrink: 0;
	}
</style>
