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
		background: rgba(0,0,0,0.3);
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.dialog {
		background: #fff;
		border-radius: 8px;
		box-shadow: 0 8px 32px rgba(0,0,0,0.18);
		padding: 20px 24px;
		min-width: 320px;
		max-width: 420px;
	}
	.dialog-msg {
		font-size: 0.9rem;
		color: #1f2328;
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
		background: #7c3aed;
		color: #fff;
	}
	.dialog-btn:hover { background: #6d28d9; }
	.dialog-btn.danger {
		background: #cf222e;
	}
	.dialog-btn.danger:hover { background: #a40e26; }
	.dialog-btn.cancel {
		background: #f0f0f4;
		color: #5c5c66;
	}
	.dialog-btn.cancel:hover { background: #e0e0e4; }
</style>
