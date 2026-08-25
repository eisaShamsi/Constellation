<script lang="ts">
	let {
		message,
		confirmLabel = 'Delete',
		cancelLabel = 'Cancel',
		danger = true,
		enterConfirms = true,
		onConfirm,
		onCancel
	}: {
		message: string;
		confirmLabel?: string;
		cancelLabel?: string;
		danger?: boolean;
		/** PJ-369 Step 4 — set false when the action CANNOT BE UNDONE, so a stray Enter cannot
		 *  perform it. Separate from `danger` on purpose: `danger` marks "destructive", and most
		 *  destructive things here are recoverable — a deleted note goes to the trash. This is
		 *  for the ones with no way back. */
		enterConfirms?: boolean;
		onConfirm: () => void;
		onCancel: () => void;
	} = $props();

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onCancel();
		// PJ-369 Step 4 (2026-08-24, found by the ui-inspector gating a destructive test) —
		// Enter confirms unless the caller says the action cannot be undone.
		//
		// This dialog is shared by every confirmation in the app, and Enter was bound to the
		// confirm button unconditionally: a single stray keypress — a held key, a return pressed
		// at a dialog not yet read — was enough to perform the action. For a note delete that is
		// survivable, because the file goes to the trash. For removing hundreds of index entries
		// whose archive no code path in this app can read back, it is not.
		//
		// Gated on `enterConfirms` rather than on `danger` deliberately. `danger` marks
		// DESTRUCTIVE, and most destructive things here are RECOVERABLE; switching on it would
		// have silently taken Enter away from the everyday note-delete confirmation, which is a
		// change nobody asked for. Escape still cancels either way — the keyboard keeps its safe
		// direction, and only the unsafe one costs a deliberate click.
		if (e.key === 'Enter' && enterConfirms) onConfirm();
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
