<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';

	let {
		options = [] as string[],
		onSelect,
		onCancel,
	}: {
		options: string[];
		onSelect: (value: string) => void;
		onCancel: () => void;
	} = $props();

	let query = $state('');
	let selectedIndex = $state(0);
	let inputEl: HTMLInputElement;

	const filtered = $derived.by(() => {
		if (!query.trim()) return options;
		const q = query.toLowerCase();
		return options.filter(o => o.toLowerCase().includes(q));
	});

	$effect(() => {
		if (filtered) selectedIndex = 0;
	});

	onMount(() => {
		inputEl?.focus();
	});

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			onCancel();
		} else if (e.key === 'ArrowDown') {
			e.preventDefault();
			selectedIndex = Math.min(selectedIndex + 1, filtered.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			selectedIndex = Math.max(selectedIndex - 1, 0);
		} else if (e.key === 'Enter') {
			e.preventDefault();
			if (filtered[selectedIndex]) {
				onSelect(filtered[selectedIndex]);
			}
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="tpl-suggest-overlay" onclick={onCancel} onkeydown={handleKeydown}>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="tpl-suggest-panel" onclick={(e) => e.stopPropagation()}>
		<div class="tpl-suggest-header">{$t('templates.suggesterTitle')}</div>
		<input
			bind:this={inputEl}
			type="text"
			class="tpl-suggest-input"
			placeholder={$t('templates.suggesterSearch')}
			bind:value={query}
			onkeydown={handleKeydown}
		/>
		<div class="tpl-suggest-list">
			{#each filtered as option, i (option)}
				<button
					class="tpl-suggest-item"
					class:selected={i === selectedIndex}
					onclick={() => onSelect(option)}
					onmouseenter={() => selectedIndex = i}
				>
					{option}
				</button>
			{/each}
			{#if filtered.length === 0}
				<div class="tpl-suggest-empty">{$t('templates.noOptions')}</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.tpl-suggest-overlay {
		position: fixed; inset: 0; z-index: 1100;
		background: var(--background-modifier-cover);
		display: flex; justify-content: center; padding-top: 15vh;
	}
	.tpl-suggest-panel {
		background: var(--background-primary); border-radius: 8px;
		box-shadow: var(--shadow-l);
		width: 420px; max-height: 380px;
		display: flex; flex-direction: column;
		overflow: hidden;
	}
	.tpl-suggest-header {
		padding: 10px 16px 6px;
		font-size: 0.75rem; font-weight: 600;
		color: var(--text-faint); text-transform: uppercase;
		letter-spacing: 0.5px;
	}
	.tpl-suggest-input {
		border: none; padding: 10px 16px;
		font-size: 0.95rem; font-family: inherit;
		color: var(--text-normal); outline: none;
		background: var(--background-primary);
		border-bottom: 1px solid var(--background-modifier-border-focus);
	}
	.tpl-suggest-input::placeholder { color: var(--color-base-40); }
	.tpl-suggest-list { flex: 1; overflow-y: auto; padding: 4px; }
	.tpl-suggest-item {
		display: flex; align-items: center;
		width: 100%; padding: 8px 12px;
		background: none; border: none; border-radius: 4px;
		cursor: pointer; font-family: inherit; text-align: start;
		color: var(--text-normal); font-size: 0.85rem;
	}
	.tpl-suggest-item.selected { background: var(--interactive-accent); color: var(--text-on-accent); }
	.tpl-suggest-item:hover:not(.selected) { background: var(--background-modifier-hover); }
	.tpl-suggest-empty { padding: 20px; text-align: center; color: var(--text-faint); font-size: 0.85rem; }
</style>
