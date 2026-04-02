<script lang="ts">
	import { t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';

	interface DueNote {
		note_path: string;
		note_name: string;
		reason: string;
		days_overdue: number;
		stratum: number;
		last_reviewed: string | null;
	}

	let {
		dueNotes = [] as DueNote[],
		onNoteClick,
		onRefresh,
	}: {
		dueNotes?: DueNote[];
		onNoteClick?: (path: string, name: string) => void;
		onRefresh?: () => void;
	} = $props();

	let showResurfacing = $state(true);
	let showCheckpoints = $state(true);
	let showNeverReviewed = $state(true);

	const resurfacing = $derived(dueNotes.filter(n => n.reason === 'interval_due' || n.reason === 'stale'));
	const checkpoints = $derived(dueNotes.filter(n => n.reason === 'checkpoint'));
	const neverReviewed = $derived(dueNotes.filter(n => n.reason === 'never_reviewed'));

	async function markReviewed(path: string) {
		try {
			await invoke('mark_reviewed', { notePath: path });
			onRefresh?.();
		} catch {}
	}

	async function snooze(path: string) {
		try {
			await invoke('snooze_note', { notePath: path, days: 7 });
			onRefresh?.();
		} catch {}
	}

	async function dismiss(path: string) {
		try {
			await invoke('dismiss_note', { notePath: path });
			onRefresh?.();
		} catch {}
	}

	function reasonIcon(reason: string): string {
		return reason === 'interval_due' ? '🔄' : reason === 'stale' ? '🥀' : reason === 'checkpoint' ? '🧠' : '📝';
	}
</script>

<div class="rp-panel">
	{#if dueNotes.length === 0}
		<div class="rp-empty">
			<div class="rp-empty-icon">✅</div>
			<div class="rp-empty-text">{$t('reviewPanel.allCaughtUp') || 'All caught up! No notes due for review.'}</div>
		</div>
	{:else}
		<!-- Spaced Resurfacing -->
		{#if resurfacing.length > 0}
		<div class="rp-section">
			<button class="rp-header" onclick={() => showResurfacing = !showResurfacing}>
				<span class="rp-chevron" class:collapsed={!showResurfacing}>▾</span>
				<span>🔄 {$t('reviewPanel.resurfacing') || 'Due for Review'}</span>
				<span class="rp-count">{resurfacing.length}</span>
			</button>
			{#if showResurfacing}
				{#each resurfacing.slice(0, 20) as note}
					<div class="rp-item">
						<button class="rp-name" onclick={() => onNoteClick?.(note.note_path, note.note_name)}>
							<span class="rp-reason">{reasonIcon(note.reason)}</span>
							{note.note_name}
						</button>
						<span class="rp-detail">{note.days_overdue}d overdue</span>
						<div class="rp-actions">
							<button class="rp-action" title={$t('reviewPanel.reviewed') || 'Reviewed'} onclick={() => markReviewed(note.note_path)}>✓</button>
							<button class="rp-action" title={$t('reviewPanel.snooze') || 'Snooze 7d'} onclick={() => snooze(note.note_path)}>👁</button>
							<button class="rp-action" title={$t('reviewPanel.dismiss') || 'Dismiss'} onclick={() => dismiss(note.note_path)}>🗄️</button>
						</div>
					</div>
				{/each}
			{/if}
		</div>
		{/if}

		<!-- Mental Model Checkpoints -->
		{#if checkpoints.length > 0}
		<div class="rp-section">
			<button class="rp-header" onclick={() => showCheckpoints = !showCheckpoints}>
				<span class="rp-chevron" class:collapsed={!showCheckpoints}>▾</span>
				<span>🧠 {$t('reviewPanel.checkpoints') || 'Mental Model Checkpoints'}</span>
				<span class="rp-count">{checkpoints.length}</span>
			</button>
			{#if showCheckpoints}
				{#each checkpoints as note}
					<div class="rp-item">
						<button class="rp-name" onclick={() => onNoteClick?.(note.note_path, note.note_name)}>
							<span class="rp-reason">🧠</span>
							{note.note_name}
						</button>
						<span class="rp-detail">{$t('reviewPanel.checkpointHint') || 'Do you still hold this view?'}</span>
						<div class="rp-actions">
							<button class="rp-action" title={$t('reviewPanel.reviewed') || 'Reviewed'} onclick={() => markReviewed(note.note_path)}>✓</button>
							<button class="rp-action" title={$t('reviewPanel.snooze') || 'Snooze 7d'} onclick={() => snooze(note.note_path)}>👁</button>
						</div>
					</div>
				{/each}
			{/if}
		</div>
		{/if}

		<!-- Never Reviewed -->
		{#if neverReviewed.length > 0}
		<div class="rp-section">
			<button class="rp-header" onclick={() => showNeverReviewed = !showNeverReviewed}>
				<span class="rp-chevron" class:collapsed={!showNeverReviewed}>▾</span>
				<span>📝 {$t('reviewPanel.neverReviewed') || 'Never Reviewed'}</span>
				<span class="rp-count">{neverReviewed.length}</span>
			</button>
			{#if showNeverReviewed}
				{#each neverReviewed.slice(0, 30) as note}
					<div class="rp-item">
						<button class="rp-name" onclick={() => onNoteClick?.(note.note_path, note.note_name)}>
							<span class="rp-reason">📝</span>
							{note.note_name}
						</button>
						<span class="rp-detail">{note.days_overdue}d old</span>
						<div class="rp-actions">
							<button class="rp-action" title={$t('reviewPanel.reviewed') || 'Reviewed'} onclick={() => markReviewed(note.note_path)}>✓</button>
							<button class="rp-action" title={$t('reviewPanel.snooze') || 'Snooze 7d'} onclick={() => snooze(note.note_path)}>👁</button>
							<button class="rp-action" title={$t('reviewPanel.dismiss') || 'Dismiss'} onclick={() => dismiss(note.note_path)}>🗄️</button>
						</div>
					</div>
				{/each}
				{#if neverReviewed.length > 30}
					<div class="rp-more">+{neverReviewed.length - 30} more</div>
				{/if}
			{/if}
		</div>
		{/if}
	{/if}
</div>

<style>
	.rp-panel { padding: 8px 0; }
	.rp-empty { text-align: center; padding: 24px 16px; }
	.rp-empty-icon { font-size: 2rem; margin-bottom: 8px; }
	.rp-empty-text { font-size: 0.82rem; color: var(--text-muted); line-height: 1.4; }
	.rp-section { margin-bottom: 4px; }
	.rp-header {
		display: flex; align-items: center; gap: 6px; width: 100%;
		padding: 6px 12px; border: none; background: none; cursor: pointer;
		font-size: 0.78rem; font-weight: 600; color: var(--text-normal); font-family: inherit;
		text-align: start;
	}
	.rp-header:hover { background: var(--background-modifier-hover); }
	.rp-chevron { font-size: 0.65rem; transition: transform 0.15s; flex-shrink: 0; }
	.rp-chevron.collapsed { transform: rotate(-90deg); }
	:global([dir="rtl"]) .rp-chevron.collapsed { transform: rotate(90deg); }
	.rp-count { margin-inline-start: auto; font-size: 0.7rem; color: var(--text-faint); font-weight: 400; }
	.rp-item {
		display: flex; align-items: center; gap: 4px;
		padding: 4px 12px 4px 20px; font-size: 0.78rem;
	}
	.rp-name {
		flex: 1; display: flex; align-items: center; gap: 4px;
		border: none; background: none; cursor: pointer; padding: 2px 4px;
		border-radius: 3px; font-family: inherit; font-size: 0.78rem;
		color: var(--text-normal); text-align: start; min-width: 0;
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.rp-name:hover { background: var(--background-modifier-hover); }
	.rp-reason { font-size: 0.7rem; flex-shrink: 0; }
	.rp-detail { font-size: 0.68rem; color: var(--text-faint); white-space: nowrap; flex-shrink: 0; }
	.rp-actions { display: flex; gap: 2px; flex-shrink: 0; }
	.rp-action {
		width: 22px; height: 22px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; cursor: pointer;
		font-size: 0.7rem; color: var(--text-muted);
	}
	.rp-action:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.rp-more { font-size: 0.72rem; color: var(--text-faint); padding: 4px 20px; }
</style>
