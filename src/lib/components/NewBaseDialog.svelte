<script lang="ts">
	import { onMount } from 'svelte';
	import { t, dir } from '$lib/i18n';
	import { libraries, type LibraryInfo } from '$lib/libraries/store';

	let {
		colorMap,
		onCreate,
		onClose,
	}: {
		colorMap: Record<string, string>;
		onCreate: (saveLibrary: LibraryInfo, baseName: string, selectedLibraries: string[]) => void;
		onClose: () => void;
	} = $props();

	let baseName = $state('');
	let selectedLibraryNames: string[] = $state([]);
	let nameInput: HTMLInputElement;
	let overlayEl: HTMLDivElement;

	const allSelected = $derived(selectedLibraryNames.length === 0);

	function toggleLibrary(name: string) {
		if (selectedLibraryNames.includes(name)) {
			selectedLibraryNames = selectedLibraryNames.filter(v => v !== name);
		} else {
			selectedLibraryNames = [...selectedLibraryNames, name];
		}
	}

	function selectAll() {
		selectedLibraryNames = [];
	}

	function handleCreate() {
		// Pass a dummy library — workspace bases don't need a save library
		const dummyLibrary = $libraries[0] ?? { id: '', name: '', path: '' };
		onCreate(dummyLibrary, baseName || $t('bases.untitled'), selectedLibraryNames);
		onClose();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') { e.preventDefault(); onClose(); }
		if (e.key === 'Enter') { e.preventDefault(); handleCreate(); }
	}

	function handleOverlayClick(e: MouseEvent) {
		if (e.target === overlayEl) onClose();
	}

	onMount(() => {
		document.addEventListener('keydown', handleKeydown);
		nameInput?.focus();
		return () => document.removeEventListener('keydown', handleKeydown);
	});
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="nbd-overlay" bind:this={overlayEl} onclick={handleOverlayClick}>
	<div class="nbd-modal" dir={$dir}>
		<div class="nbd-title">{$t('commands.newBase')}</div>

		<div class="nbd-body">
			<!-- Name -->
			<label class="nbd-field">
				<span class="nbd-label">{$t('bases.baseName')}</span>
				<input
					type="text"
					bind:this={nameInput}
					bind:value={baseName}
					placeholder={$t('bases.untitled')}
				/>
			</label>

			<!-- Query libraries -->
			<div class="nbd-section">
				<span class="nbd-label">{$t('bases.source.vaultsLabel')}</span>
				<div class="nbd-vault-list">
					<label class="nbd-vault" class:active={allSelected}>
						<input type="checkbox" checked={allSelected} onchange={selectAll} />
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M2 12h20"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
						<span>{$t('bases.source.allVaults')}</span>
					</label>
					{#each $libraries as v}
						<label class="nbd-vault" class:active={selectedLibraryNames.includes(v.name)}>
							<input
								type="checkbox"
								checked={allSelected || selectedLibraryNames.includes(v.name)}
								onchange={() => toggleLibrary(v.name)}
							/>
							<span class="nbd-dot" style="background: {colorMap[v.name] || '#7c3aed'}"></span>
							<span>{v.name}</span>
						</label>
					{/each}
				</div>
			</div>
		</div>

		<div class="nbd-actions">
			<button class="nbd-create" onclick={handleCreate}>{$t('bases.create')}</button>
			<button class="nbd-cancel" onclick={onClose}>{$t('bases.source.cancel')}</button>
		</div>
	</div>
</div>

<style>
	.nbd-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.4);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}
	.nbd-modal {
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 10px;
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
		width: 340px;
		max-height: 70vh;
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}
	.nbd-title {
		padding: 14px 16px 10px;
		font-size: 0.95rem;
		font-weight: 600;
		color: var(--text-normal);
		border-bottom: 1px solid var(--background-modifier-border);
	}
	.nbd-body {
		padding: 12px 16px;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.nbd-field {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.nbd-label {
		font-size: 0.75rem;
		font-weight: 600;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}
	.nbd-field input, .nbd-field select {
		padding: 6px 10px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		background: var(--background-secondary);
		color: var(--text-normal);
		font-size: 0.85rem;
		font-family: inherit;
	}
	.nbd-field input:focus, .nbd-field select:focus {
		outline: none;
		border-color: var(--interactive-accent);
	}

	.nbd-section {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.nbd-vault-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
		max-height: 180px;
		overflow-y: auto;
	}
	.nbd-vault {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 5px 8px;
		border-radius: 6px;
		font-size: 0.82rem;
		color: var(--text-muted);
		cursor: pointer;
		transition: all 0.1s;
	}
	.nbd-vault:hover {
		background: var(--background-modifier-hover);
		color: var(--text-normal);
	}
	.nbd-vault.active {
		color: var(--text-normal);
	}
	.nbd-vault input[type="checkbox"] {
		cursor: pointer;
		accent-color: var(--interactive-accent);
	}
	.nbd-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.nbd-actions {
		display: flex;
		gap: 8px;
		padding: 10px 16px;
		border-top: 1px solid var(--background-modifier-border);
		justify-content: flex-end;
	}
	.nbd-create {
		padding: 6px 18px;
		border: none;
		border-radius: 6px;
		background: var(--interactive-accent);
		color: white;
		cursor: pointer;
		font-size: 0.82rem;
		font-family: inherit;
	}
	.nbd-create:hover { opacity: 0.9; }
	.nbd-cancel {
		padding: 6px 18px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		background: none;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 0.82rem;
		font-family: inherit;
	}
	.nbd-cancel:hover { color: var(--text-normal); }
</style>
