<script lang="ts">
	let {
		message,
		confirmLabel = 'Delete',
		cancelLabel = 'Cancel',
		danger = true,
		onConfirm,
		onCancel
	}: {
		message: string;
		confirmLabel?: string;
		cancelLabel?: string;
		danger?: boolean;
		onConfirm: () => void;
		onCancel: () => void;
	} = $props();

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onCancel();
		if (e.key === 'Enter') onConfirm();
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="dialog-overlay" onclick={onCancel} onkeydown={handleKeydown}>
	<div class="dialog" onclick={(e) => e.stopPropagation()}>
		<p class="dialog-msg">{message}</p>
		<div class="dialog-actions">
			<button class="dialog-btn cancel" onclick={onCancel}>{cancelLabel}</button>
			<button class="dialog-btn" class:danger onclick={onConfirm}>{confirmLabel}</button>
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
		min-width: 320px;
		max-width: 420px;
	}
	.dialog-msg {
		font-size: 0.9rem;
		color: var(--text-normal);
		margin: 0 0 16px;
		line-height: 1.5;
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
	.dialog-btn.danger {
		background: var(--text-error);
	}
	.dialog-btn.danger:hover { background: color-mix(in srgb, var(--text-error) 80%, black); }
	.dialog-btn.cancel {
		background: var(--background-secondary-alt);
		color: var(--text-muted);
	}
	.dialog-btn.cancel:hover { background: var(--background-modifier-border); }
</style>
