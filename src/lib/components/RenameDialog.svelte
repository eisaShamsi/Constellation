<script lang="ts">
	// MIG-077 A3-R — a small, reusable rename dialog. The file tree renames
	// inline (FileTree input -> renamingPath); but full-page surfaces (OrgChart,
	// Search, List-mode) have no inline row, so they drive a rename through this
	// dialog, which simply hands the new name back to the host's existing
	// handleRenameComplete (rename + wikilink cascade + collision dialog all
	// reused — nothing reinvented). RTL/multilingual-safe via detectDir.
	import { onMount, untrack } from 'svelte';
	import { detectDir } from '$lib/utils';

	let {
		initialValue = '',
		title = '',
		confirmLabel = 'OK',
		cancelLabel = 'Cancel',
		onConfirm,
		onCancel,
	}: {
		initialValue?: string;
		title?: string;
		confirmLabel?: string;
		cancelLabel?: string;
		onConfirm: (name: string) => void;
		onCancel: () => void;
	} = $props();

	// Snapshot the prop once at mount — the dialog is freshly mounted per open
	// (keyed by `{#if renameDialog}`), so initialValue never changes mid-life.
	let value = $state(untrack(() => initialValue));
	let inputEl: HTMLInputElement | undefined;

	function submit() {
		const v = value.trim();
		if (v) onConfirm(v);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') { e.preventDefault(); onCancel(); }
		else if (e.key === 'Enter') { e.preventDefault(); submit(); }
	}

	onMount(() => {
		inputEl?.focus();
		inputEl?.select();
	});
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="dialog-overlay" onclick={onCancel} onkeydown={handleKeydown}>
	<div class="dialog" onclick={(e) => e.stopPropagation()}>
		{#if title}<p class="dialog-title">{title}</p>{/if}
		<input
			bind:this={inputEl}
			bind:value
			class="dialog-input"
			dir={detectDir(value)}
			onkeydown={handleKeydown}
		/>
		<div class="dialog-actions">
			<button class="dialog-btn cancel" onclick={onCancel}>{cancelLabel}</button>
			<button class="dialog-btn" onclick={submit} disabled={!value.trim()}>{confirmLabel}</button>
		</div>
	</div>
</div>

<style>
	.dialog-overlay {
		position: fixed;
		inset: 0;
		z-index: 2000;
		background: var(--background-modifier-cover);
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.dialog {
		background: var(--background-primary);
		border-radius: 8px;
		box-shadow: var(--shadow-l);
		padding: 20px 24px;
		min-width: 340px;
		max-width: 460px;
	}
	.dialog-title {
		font-size: 0.95rem;
		font-weight: 600;
		color: var(--text-normal);
		margin: 0 0 12px;
	}
	.dialog-input {
		width: 100%;
		box-sizing: border-box;
		padding: 8px 10px;
		margin-bottom: 16px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 5px;
		background: var(--background-primary);
		color: var(--text-normal);
		font-size: 0.9rem;
		font-family: inherit;
	}
	.dialog-input:focus {
		outline: none;
		border-color: var(--interactive-accent);
	}
	.dialog-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
	}
	.dialog-btn {
		padding: 6px 16px;
		border: none;
		border-radius: 5px;
		font-size: 0.82rem;
		font-family: inherit;
		cursor: pointer;
		font-weight: 500;
		background: var(--interactive-accent);
		color: var(--text-on-accent);
	}
	.dialog-btn:hover { background: var(--interactive-accent-hover); }
	.dialog-btn:disabled { opacity: 0.5; cursor: default; }
	.dialog-btn.cancel {
		background: var(--background-secondary-alt);
		color: var(--text-muted);
	}
	.dialog-btn.cancel:hover { background: var(--background-modifier-border); }
</style>
