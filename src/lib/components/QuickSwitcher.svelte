<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { constellationSearch, parseSearchQuery } from '$lib/libraries/store';

	let {
		notes = [] as { name: string; path: string; libraryName: string }[],
		onSelect,
		onClose,
	}: {
		notes: { name: string; path: string; libraryName: string }[];
		onSelect: (path: string, libraryName: string) => void;
		onClose: () => void;
	} = $props();

	let query = $state('');
	let selectedIndex = $state(0);
	let inputEl: HTMLInputElement;
	let extendedResults = $state<{ name: string; path: string; libraryName: string }[]>([]);
	let searchTimer: ReturnType<typeof setTimeout>;

	const filtered = $derived.by(() => {
		if (!query.trim()) return notes.slice(0, 30);
		const q = query.toLowerCase();
		const local = notes
			.filter(n => n.name.toLowerCase().includes(q) || n.path.toLowerCase().includes(q))
			.slice(0, 20);
		// Merge extended results (deduplicated)
		if (extendedResults.length > 0) {
			const seen = new Set(local.map(n => n.path));
			for (const r of extendedResults) {
				if (!seen.has(r.path)) { local.push(r); seen.add(r.path); }
			}
		}
		return local.slice(0, 30);
	});

	// Async extended search for queries >= 3 chars
	$effect(() => {
		const q = query;
		clearTimeout(searchTimer);
		if (q.trim().length >= 3) {
			searchTimer = setTimeout(async () => {
				try {
					const req = parseSearchQuery(q);
					req.limit = 15;
					const results = await constellationSearch(req);
					extendedResults = results.map(r => ({ name: r.name, path: r.path, libraryName: r.library_name }));
				} catch { extendedResults = []; }
			}, 300);
		} else {
			extendedResults = [];
		}
	});

	$effect(() => {
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
				onSelect(filtered[selectedIndex].path, filtered[selectedIndex].libraryName);
				onClose();
			}
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="qs-overlay" onclick={onClose} onkeydown={handleKeydown}>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="qs-panel" onclick={(e) => e.stopPropagation()}>
		<input
			bind:this={inputEl}
			type="text"
			class="qs-input"
			placeholder={$t('quickSwitcher.placeholder')}
			bind:value={query}
			onkeydown={handleKeydown}
		/>
		<div class="qs-list">
			{#each filtered as note, i (note.path)}
				<button
					class="qs-item"
					class:selected={i === selectedIndex}
					onclick={() => { onSelect(note.path, note.libraryName); onClose(); }}
					onmouseenter={() => selectedIndex = i}
				>
					<span class="qs-name">{note.name}</span>
					<span class="qs-path">{note.libraryName}</span>
				</button>
			{/each}
			{#if filtered.length === 0 && query}
				<div class="qs-empty">{$t('quickSwitcher.noResults')}</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.qs-overlay {
		position: fixed; inset: 0; z-index: 1000;
		background: var(--background-modifier-cover);
		display: flex; justify-content: center; padding-top: 15vh;
	}
	.qs-panel {
		background: var(--background-primary); border-radius: 8px;
		box-shadow: var(--shadow-l);
		width: 500px; max-height: 400px;
		display: flex; flex-direction: column;
		overflow: hidden;
	}
	.qs-input {
		border: none; padding: 12px 16px;
		font-size: 0.95rem; font-family: inherit;
		color: var(--text-normal); outline: none;
		background: var(--background-primary);
		border-bottom: 1px solid var(--background-modifier-border-focus);
	}
	.qs-input::placeholder { color: var(--color-base-40); }
	.qs-list { flex: 1; overflow-y: auto; padding: 4px; }
	.qs-item {
		display: flex; align-items: center; justify-content: space-between;
		width: 100%; padding: 6px 12px;
		background: none; border: none; border-radius: 4px;
		cursor: pointer; font-family: inherit; text-align: start;
		color: var(--text-normal); font-size: 0.85rem;
	}
	.qs-item.selected { background: var(--interactive-accent); color: var(--text-on-accent); }
	.qs-name { font-weight: 500; }
	.qs-path { font-size: 0.72rem; color: var(--text-faint); }
	.qs-item.selected .qs-path { color: rgba(255,255,255,0.7); }
	.qs-empty { padding: 16px; text-align: center; color: var(--text-faint); font-size: 0.85rem; }
</style>
