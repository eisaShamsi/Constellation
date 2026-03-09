<script lang="ts">
	import { onMount } from 'svelte';

	let {
		notes = [] as { name: string; path: string; vaultName: string }[],
		onSelect,
		onClose,
		ar = false,
	}: {
		notes: { name: string; path: string; vaultName: string }[];
		onSelect: (path: string, vaultName: string) => void;
		onClose: () => void;
		ar?: boolean;
	} = $props();

	let query = $state('');
	let selectedIndex = $state(0);
	let inputEl: HTMLInputElement;

	const filtered = $derived.by(() => {
		if (!query.trim()) return notes.slice(0, 30);
		const q = query.toLowerCase();
		return notes
			.filter(n => n.name.toLowerCase().includes(q) || n.path.toLowerCase().includes(q))
			.slice(0, 30);
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
				onSelect(filtered[selectedIndex].path, filtered[selectedIndex].vaultName);
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
			placeholder={ar ? 'ابحث عن ملاحظة...' : 'Find a note...'}
			bind:value={query}
			onkeydown={handleKeydown}
		/>
		<div class="qs-list">
			{#each filtered as note, i (note.path)}
				<button
					class="qs-item"
					class:selected={i === selectedIndex}
					onclick={() => { onSelect(note.path, note.vaultName); onClose(); }}
					onmouseenter={() => selectedIndex = i}
				>
					<span class="qs-name">{note.name}</span>
					<span class="qs-path">{note.vaultName}</span>
				</button>
			{/each}
			{#if filtered.length === 0 && query}
				<div class="qs-empty">{ar ? 'لا توجد نتائج' : 'No matching notes'}</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.qs-overlay {
		position: fixed; inset: 0; z-index: 1000;
		background: rgba(0, 0, 0, 0.3);
		display: flex; justify-content: center; padding-top: 15vh;
	}
	.qs-panel {
		background: #fff; border-radius: 8px;
		box-shadow: 0 16px 48px rgba(0,0,0,0.2);
		width: 500px; max-height: 400px;
		display: flex; flex-direction: column;
		overflow: hidden;
	}
	.qs-input {
		border: none; padding: 12px 16px;
		font-size: 0.95rem; font-family: inherit;
		color: #1f2328; outline: none;
		border-bottom: 1px solid #e8e8ec;
	}
	.qs-input::placeholder { color: #b0b0b8; }
	.qs-list { flex: 1; overflow-y: auto; padding: 4px; }
	.qs-item {
		display: flex; align-items: center; justify-content: space-between;
		width: 100%; padding: 6px 12px;
		background: none; border: none; border-radius: 4px;
		cursor: pointer; font-family: inherit; text-align: start;
		color: #1f2328; font-size: 0.85rem;
	}
	.qs-item.selected { background: #7c3aed; color: #fff; }
	.qs-name { font-weight: 500; }
	.qs-path { font-size: 0.72rem; color: #8b8b96; }
	.qs-item.selected .qs-path { color: rgba(255,255,255,0.7); }
	.qs-empty { padding: 16px; text-align: center; color: #8b8b96; font-size: 0.85rem; }
</style>
