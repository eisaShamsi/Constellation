<script lang="ts">
	// MIG-080 §F — the universe-wide Review reviewer: a LEFT-DOCK full-page surface
	// (Boss ruling 2026-06-22) over the now-cheap get_due_notes, presenting the two
	// lenses (Stale + Due-for-Review/Checkpoints/Never) via the reused ReviewPulsePanel.
	// The per-note status lives in the right-sidebar Review tab (ReviewStatusPanel).
	import { t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import ReviewPulsePanel from './ReviewPulsePanel.svelte';

	let {
		libraryPath = null,
		staleGraceDays = 1,
		onNoteClick,
		onClose,
	}: {
		libraryPath?: string | null;
		staleGraceDays?: number;
		onNoteClick?: (path: string, name: string) => void;
		onClose?: () => void;
	} = $props();

	let dueNotes = $state<any[]>([]);
	let loading = $state(true);

	async function load() {
		if (!libraryPath) { dueNotes = []; loading = false; return; }
		loading = true;
		try { dueNotes = await invoke<any[]>('get_due_notes', { libraryPath, staleGraceDays }); }
		catch { dueNotes = []; }
		loading = false;
	}
	onMount(load);
</script>

<div class="reviewer">
	<div class="reviewer-header">
		<h1>🕐 {$t('panels.review') || 'Review Pulse'}</h1>
		<span class="reviewer-count">{dueNotes.length}</span>
		<button class="reviewer-close" onclick={() => onClose?.()} aria-label="Close" title={$t('common.close') || 'Close'}>✕</button>
	</div>
	<div class="reviewer-body">
		{#if loading}
			<div class="reviewer-msg">{$t('common.loading') || 'Loading…'}</div>
		{:else}
			<ReviewPulsePanel {dueNotes} {onNoteClick} onRefresh={load} />
		{/if}
	</div>
</div>

<style>
	.reviewer { display: flex; flex-direction: column; height: 100%; min-height: 0; }
	.reviewer-header {
		display: flex; align-items: center; gap: 10px; padding: 14px 20px;
		border-bottom: 1px solid var(--background-modifier-border); flex-shrink: 0;
	}
	.reviewer-header h1 { margin: 0; font-size: calc(1.1rem * var(--rs-scale, 1)); font-weight: 600; color: var(--text-normal); }
	.reviewer-count { font-size: calc(0.85rem * var(--rs-scale, 1)); color: var(--text-faint); }
	.reviewer-close {
		margin-inline-start: auto; border: none; background: none; cursor: pointer;
		color: var(--text-muted); font-size: calc(1rem * var(--rs-scale, 1)); padding: 4px 8px; border-radius: 4px;
	}
	.reviewer-close:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.reviewer-body { flex: 1; min-height: 0; overflow-y: auto; }
	.reviewer-msg { text-align: center; color: var(--text-muted); padding: 32px; font-size: calc(0.85rem * var(--rs-scale, 1)); }
</style>
