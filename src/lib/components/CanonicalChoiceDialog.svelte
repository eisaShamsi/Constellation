<script lang="ts">
	import { t } from '$lib/i18n';

	let {
		onAdopt,
		onKeepIntact,
		onCancel,
	}: {
		onAdopt: () => void;
		onKeepIntact: () => void;
		onCancel: () => void;
	} = $props();

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onCancel();
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="ccd-overlay" onclick={onCancel} onkeydown={handleKeydown} tabindex="-1" role="dialog" aria-modal="true">
	<div class="ccd-modal" onclick={(e) => e.stopPropagation()}>
		<div class="ccd-header">
			<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="var(--text-accent)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
				<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
				<polyline points="14 2 14 8 20 8"/>
				<path d="M12 18v-6"/><path d="M9 15l3 3 3-3"/>
			</svg>
			<h2>{$t('canonical.choiceTitle')}</h2>
		</div>

		<p class="ccd-desc">{$t('canonical.choiceDesc')}</p>

		<div class="ccd-options">
			<button class="ccd-option adopt" onclick={onAdopt}>
				<div class="ccd-option-header">
					<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 2L3 7v10l9 5 9-5V7l-9-5z"/><path d="M12 22V12"/><path d="M3 7l9 5 9-5"/></svg>
					<span class="ccd-option-title">{$t('canonical.adoptTitle')}</span>
				</div>
				<p class="ccd-option-desc">{$t('canonical.adoptDesc')}</p>
				<p class="ccd-option-note">{$t('canonical.adoptNote')}</p>
			</button>

			<button class="ccd-option keep" onclick={onKeepIntact}>
				<div class="ccd-option-header">
					<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
					<span class="ccd-option-title">{$t('canonical.keepTitle')}</span>
				</div>
				<p class="ccd-option-desc">{$t('canonical.keepDesc')}</p>
				<p class="ccd-option-note">{$t('canonical.keepNote')}</p>
			</button>
		</div>

		<div class="ccd-footer">
			<button class="ccd-cancel" onclick={onCancel}>{$t('common.cancel')}</button>
		</div>
	</div>
</div>

<style>
	.ccd-overlay {
		position: fixed;
		inset: 0;
		z-index: 99998;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		animation: ccdFadeIn 0.2s ease;
	}
	.ccd-modal {
		background: var(--background-primary);
		border-radius: 14px;
		padding: 32px 36px;
		max-width: 540px;
		width: 92vw;
		box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
	}
	.ccd-header {
		display: flex;
		align-items: center;
		gap: 12px;
		margin-bottom: 12px;
	}
	.ccd-header h2 {
		margin: 0;
		font-size: 1.15rem;
		font-weight: 700;
	}
	.ccd-desc {
		margin: 0 0 24px;
		font-size: 0.88rem;
		color: var(--text-muted);
		line-height: 1.6;
	}
	.ccd-options {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin-bottom: 20px;
	}
	.ccd-option {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 16px 18px;
		border: 2px solid var(--background-modifier-border);
		border-radius: 10px;
		background: var(--background-primary);
		cursor: pointer;
		text-align: start;
		transition: all 0.15s;
	}
	.ccd-option:hover {
		border-color: var(--text-accent);
		background: var(--background-secondary);
	}
	.ccd-option-header {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.ccd-option-title {
		font-size: 0.95rem;
		font-weight: 600;
	}
	.ccd-option-desc {
		margin: 4px 0 0;
		font-size: 0.82rem;
		color: var(--text-muted);
		line-height: 1.5;
		padding-inline-start: 28px;
	}
	.ccd-option-note {
		margin: 4px 0 0;
		font-size: 0.75rem;
		color: var(--text-faint);
		font-style: italic;
		padding-inline-start: 28px;
	}
	.ccd-footer {
		display: flex;
		justify-content: flex-end;
	}
	.ccd-cancel {
		padding: 6px 18px;
		border-radius: 6px;
		border: 1px solid var(--background-modifier-border);
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 0.82rem;
	}
	.ccd-cancel:hover {
		background: var(--background-modifier-hover);
		color: var(--text-normal);
	}
	@keyframes ccdFadeIn {
		from { opacity: 0; }
		to { opacity: 1; }
	}
</style>
