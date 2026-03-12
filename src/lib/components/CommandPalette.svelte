<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';

	export interface Command {
		id: string;
		name: string;
		shortcut?: string;
		icon?: string;
		action: () => void;
		category?: string;
	}

	let {
		commands = [] as Command[],
		onClose,
	}: {
		commands: Command[];
		onClose: () => void;
	} = $props();

	let query = $state('');
	let selectedIndex = $state(0);
	let inputEl: HTMLInputElement;

	const filtered = $derived(
		query.trim()
			? commands.filter(c =>
				c.name.toLowerCase().includes(query.toLowerCase()) ||
				(c.category?.toLowerCase().includes(query.toLowerCase()))
			)
			: commands
	);

	$effect(() => {
		// Reset selection when filter changes
		if (filtered) selectedIndex = 0;
	});

	onMount(() => {
		inputEl?.focus();
	});

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			onClose();
		} else if (e.key === 'ArrowDown') {
			e.preventDefault();
			selectedIndex = Math.min(selectedIndex + 1, filtered.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			selectedIndex = Math.max(selectedIndex - 1, 0);
		} else if (e.key === 'Enter') {
			e.preventDefault();
			if (filtered[selectedIndex]) {
				filtered[selectedIndex].action();
				onClose();
			}
		}
	}

	function handleItemClick(cmd: Command) {
		cmd.action();
		onClose();
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="palette-overlay" onclick={onClose} onkeydown={handleKeydown}>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="palette" onclick={(e) => e.stopPropagation()}>
		<div class="palette-input-wrap">
			<svg class="palette-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
			<input
				bind:this={inputEl}
				type="text"
				placeholder={$t('commandPalette.placeholder')}
				bind:value={query}
				onkeydown={handleKeydown}
			/>
		</div>
		<div class="palette-list">
			{#each filtered as cmd, i (cmd.id)}
				<button
					class="palette-item"
					class:selected={i === selectedIndex}
					onclick={() => handleItemClick(cmd)}
					onmouseenter={() => selectedIndex = i}
				>
					<span class="pi-icon">{cmd.icon ?? '⌘'}</span>
					<span class="pi-name">{cmd.name}</span>
					{#if cmd.shortcut}
						<span class="pi-shortcut">{cmd.shortcut}</span>
					{/if}
				</button>
			{/each}
			{#if filtered.length === 0}
				<div class="palette-empty">{$t('commandPalette.noResults')}</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.palette-overlay {
		position: fixed; inset: 0; z-index: 1000;
		background: var(--background-modifier-cover);
		display: flex; justify-content: center; padding-top: 15vh;
	}
	.palette {
		background: var(--background-primary); border-radius: 8px;
		box-shadow: var(--shadow-l);
		width: 500px; max-height: 400px;
		display: flex; flex-direction: column;
		overflow: hidden;
	}
	.palette-input-wrap {
		display: flex; align-items: center; gap: 8px;
		padding: 8px 12px; border-bottom: 1px solid var(--background-modifier-border-focus);
	}
	.palette-icon { color: var(--text-faint); flex-shrink: 0; }
	.palette-input-wrap input {
		flex: 1; border: none; background: none;
		font-size: 0.92rem; font-family: inherit;
		color: var(--text-normal); outline: none;
	}
	.palette-input-wrap input::placeholder { color: var(--color-base-40); }
	.palette-list { flex: 1; overflow-y: auto; padding: 4px; }
	.palette-item {
		display: flex; align-items: center; gap: 8px;
		width: 100%; padding: 6px 8px;
		background: none; border: none; border-radius: 4px;
		cursor: pointer; font-family: inherit; text-align: start;
		color: var(--text-normal); font-size: 0.85rem;
	}
	.palette-item.selected { background: var(--interactive-accent); color: var(--text-on-accent); }
	.pi-icon { width: 20px; text-align: center; font-size: 0.9rem; }
	.pi-name { flex: 1; }
	.pi-shortcut {
		font-size: 0.72rem; color: var(--text-faint);
		background: var(--background-secondary-alt); padding: 1px 5px;
		border-radius: 3px; font-family: monospace;
	}
	.palette-item.selected .pi-shortcut { background: rgba(255,255,255,0.2); color: var(--text-on-accent); }
	.palette-empty { padding: 16px; text-align: center; color: var(--text-faint); font-size: 0.85rem; }
</style>
