<script lang="ts">
	/**
	 * MIG-108 Slice 5 — the D2 "ask each time" dialog.
	 *
	 * Shown when the user picks a folder OUTSIDE the universe to add as a library. One
	 * Universe, One Location means it cannot be referenced in place; the user chooses what
	 * happens to the original: Copy in (default — the original stays untouched, unmanaged)
	 * or Move in (the folder physically relocates under the universe root).
	 */
	import { t } from '$lib/i18n';

	let {
		sourcePath,
		onChoose,
		onCancel,
	}: {
		sourcePath: string;
		onChoose: (mode: 'copy' | 'move') => void;
		onCancel: () => void;
	} = $props();
</script>

<div class="bi-backdrop" role="dialog" aria-modal="true">
	<div class="bi-card" dir="auto">
		<h2>{$t('bringIn.title') || 'Bring this folder into your universe'}</h2>
		<p class="bi-path" dir="ltr">{sourcePath}</p>
		<p class="bi-body">{$t('bringIn.body') || 'A universe keeps all of its knowledge in one folder, so this library will live inside it. Choose what happens to the original:'}</p>
		<div class="bi-options">
			<button class="bi-option bi-primary" onclick={() => onChoose('copy')}>
				<span class="bi-label">{$t('bringIn.copy') || 'Copy in'}</span>
				<span class="bi-desc">{$t('bringIn.copyDesc') || 'The contents are copied into the universe. The original folder stays exactly where it is, untouched.'}</span>
			</button>
			<button class="bi-option" onclick={() => onChoose('move')}>
				<span class="bi-label">{$t('bringIn.move') || 'Move in'}</span>
				<span class="bi-desc">{$t('bringIn.moveDesc') || 'The folder itself moves into the universe. Its old location will be empty afterwards.'}</span>
			</button>
		</div>
		<div class="bi-actions">
			<button class="bi-cancel" onclick={onCancel}>{$t('bringIn.cancel') || 'Cancel'}</button>
		</div>
	</div>
</div>

<style>
	.bi-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.45);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 2000;
	}
	.bi-card {
		background: var(--background-primary);
		color: var(--text-normal);
		border-radius: 10px;
		padding: 22px 26px;
		width: min(560px, 92vw);
		box-shadow: 0 12px 40px rgba(0, 0, 0, 0.35);
	}
	h2 { margin: 0 0 8px; font-size: 1.05rem; }
	.bi-path { font-size: 0.78rem; color: var(--text-faint); overflow-wrap: anywhere; margin: 0 0 8px; }
	.bi-body { font-size: 0.88rem; color: var(--text-muted); line-height: 1.5; }
	.bi-options { display: flex; flex-direction: column; gap: 8px; margin: 14px 0; }
	.bi-option {
		text-align: start;
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		background: transparent;
		color: var(--text-normal);
		padding: 10px 14px;
		cursor: pointer;
		display: flex;
		flex-direction: column;
		gap: 3px;
	}
	.bi-option:hover { border-color: var(--interactive-accent); }
	.bi-primary { border-color: var(--interactive-accent); }
	.bi-label { font-weight: 600; font-size: 0.9rem; }
	.bi-desc { font-size: 0.78rem; color: var(--text-muted); line-height: 1.4; }
	.bi-actions { display: flex; justify-content: flex-end; }
	.bi-cancel {
		background: transparent;
		color: var(--text-muted);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		padding: 7px 14px;
		cursor: pointer;
		font-size: 0.85rem;
	}
</style>
