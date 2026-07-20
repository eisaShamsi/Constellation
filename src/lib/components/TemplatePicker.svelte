<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';

	let {
		templates = [] as { name: string; path: string; libraryName: string }[],
		onSelect,
		onClose,
		// MIG-TPL §1 — the empty state names the REAL folder and offers a way out of it.
		// All three are optional so existing mount sites keep working unchanged.
		folderPath = '',
		onOpenFolder = undefined,
	}: {
		templates: { name: string; path: string; libraryName: string }[];
		onSelect: (path: string, libraryName: string) => void;
		onClose: () => void;
		folderPath?: string;
		onOpenFolder?: () => void;
	} = $props();

	let query = $state('');
	let selectedIndex = $state(0);
	let inputEl: HTMLInputElement;

	const filtered = $derived.by(() => {
		if (!query.trim()) return templates.slice(0, 30);
		const q = query.toLowerCase();
		return templates
			.filter(n => n.name.toLowerCase().includes(q))
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
				onSelect(filtered[selectedIndex].path, filtered[selectedIndex].libraryName);
				onClose();
			}
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="tp-overlay" onclick={onClose} onkeydown={handleKeydown}>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="tp-panel" onclick={(e) => e.stopPropagation()}>
		<div class="tp-header">{$t('templates.pickerTitle')}</div>
		<input
			bind:this={inputEl}
			type="text"
			class="tp-input"
			placeholder={$t('templates.searchTemplates')}
			bind:value={query}
			onkeydown={handleKeydown}
		/>
		<div class="tp-list">
			{#each filtered as tpl, i (tpl.path)}
				<button
					class="tp-item"
					class:selected={i === selectedIndex}
					onclick={() => { onSelect(tpl.path, tpl.libraryName); onClose(); }}
					onmouseenter={() => selectedIndex = i}
				>
					<span class="tp-icon">📄</span>
					<span class="tp-name">{tpl.name}</span>
					{#if tpl.libraryName}
						<span class="tp-lib-name">{tpl.libraryName}</span>
					{/if}
				</button>
			{/each}
			{#if filtered.length === 0 && query}
				<div class="tp-empty">{$t('templates.noTemplates')}</div>
			{:else if templates.length === 0}
				<!-- MIG-TPL §1 — an ACTIONABLE empty state. It used to say "create .md files in your
				     template folder" without naming one, while the folder it actually read was hidden
				     inside .constellation and never shown. Now it names the REAL folder and offers the
				     two things a user with no templates needs. -->
				<div class="tp-empty">
					<div>{$t('templates.noTemplatesConfigured')}</div>
					{#if folderPath}
						<div class="tp-empty-path" dir="auto" title={folderPath}>{folderPath}</div>
					{/if}
					<div class="tp-empty-actions">
						{#if onOpenFolder}
							<button class="tp-empty-btn" onclick={() => { onOpenFolder?.(); onClose(); }}>
								{$t('templates.openFolder')}
							</button>
						{/if}
					</div>
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.tp-overlay {
		position: fixed; inset: 0; z-index: 1000;
		background: var(--background-modifier-cover);
		display: flex; justify-content: center; padding-top: 15vh;
	}
	.tp-panel {
		background: var(--background-primary); border-radius: 8px;
		box-shadow: var(--shadow-l);
		width: 480px; max-height: 400px;
		display: flex; flex-direction: column;
		overflow: hidden;
	}
	.tp-header {
		padding: 10px 16px 6px;
		font-size: 0.75rem;
		font-weight: 600;
		color: var(--text-faint);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}
	.tp-input {
		border: none; padding: 10px 16px;
		font-size: 0.95rem; font-family: inherit;
		color: var(--text-normal); outline: none;
		background: var(--background-primary);
		border-bottom: 1px solid var(--background-modifier-border-focus);
	}
	.tp-input::placeholder { color: var(--color-base-40); }
	.tp-list { flex: 1; overflow-y: auto; padding: 4px; }
	.tp-item {
		display: flex; align-items: center; gap: 8px;
		width: 100%; padding: 8px 12px;
		background: none; border: none; border-radius: 4px;
		cursor: pointer; font-family: inherit; text-align: start;
		color: var(--text-normal); font-size: 0.85rem;
	}
	.tp-item.selected { background: var(--interactive-accent); color: var(--text-on-accent); }
	.tp-icon { font-size: 0.9rem; }
	.tp-name { flex: 1; font-weight: 500; }
	.tp-lib-name { font-size: 0.7rem; color: var(--text-faint); }
	.tp-item.selected .tp-lib-name { color: rgba(255,255,255,0.7); }
	.tp-empty { padding: 20px; text-align: center; color: var(--text-faint); font-size: 0.85rem; }
	.tp-empty-path {
		margin-top: 6px; font-size: 0.78rem; color: var(--text-muted, #888);
		word-break: break-all; font-family: var(--font-monospace-theme, monospace);
	}
	.tp-empty-actions { margin-top: 12px; display: flex; gap: 8px; justify-content: center; flex-wrap: wrap; }
	.tp-empty-btn {
		padding: 5px 12px; border-radius: 6px; cursor: pointer;
		border: 1px solid var(--background-modifier-border, #ccc);
		background: var(--background-primary, #fff); color: var(--text-normal);
		font-size: 0.8rem; font-family: inherit;
	}
	.tp-empty-btn:hover { border-color: var(--interactive-accent); color: var(--interactive-accent); }
</style>
