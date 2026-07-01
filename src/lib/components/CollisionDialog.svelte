<script lang="ts">
	/**
	 * MIG-076 §E1b — the name-collision dialog (PJ-003, Eisa ruling 2026-06-11:
	 * "the conventional way"). Shown when a CREATE with a typed name, or a
	 * RENAME, would land on an existing note's filename. Replaces create's
	 * silent auto-suffix and rename's swallowed "already exists" failure in BOTH
	 * flows with one conventional modal: Change name / Overwrite / Cancel.
	 *
	 * Overwrite is recoverable — the host moves the existing note to the
	 * library's `.trash` first (via move_to_trash), so reversibility holds.
	 * The host owns the actual create/rename re-attempt; this component only
	 * surfaces the choice + the edited name.
	 */
	import { t } from '$lib/i18n';
	import { detectDir } from '$lib/utils';

	let {
		existingName,
		existingLibrary,
		suggestedName,
		onChangeName,
		onOverwrite,
		onCancel,
	}: {
		/** The colliding base name (no extension), e.g. "Foo". */
		existingName: string;
		/** The library the existing note lives in (universe-wide collision). */
		existingLibrary: string;
		/** Pre-fill for the rename input — the auto-suffix suggestion, e.g. "Foo 1". */
		suggestedName: string;
		/** Use a different name (the host re-checks it — it may collide again). */
		onChangeName: (newName: string) => void;
		/** Replace the existing note (host trashes it first, then proceeds). */
		onOverwrite: () => void;
		onCancel: () => void;
	} = $props();

	let nameValue = $state(suggestedName);
	let trimmed = $derived(nameValue.trim());
	let canRename = $derived(trimmed.length > 0 && trimmed !== existingName);

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') { onCancel(); return; }
		if (e.key === 'Enter' && canRename) onChangeName(trimmed);
	}
	function focusInput(node: HTMLInputElement) {
		node.focus();
		node.select();
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="col-overlay" onclick={onCancel} onkeydown={handleKeydown} tabindex="-1" role="dialog" aria-modal="true">
	<div class="col-modal" onclick={(e) => e.stopPropagation()}>
		<div class="col-header">
			<svg width="30" height="30" viewBox="0 0 24 24" fill="none" stroke="var(--text-accent)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
				<path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
				<line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/>
			</svg>
			<h2 dir="auto">{$t('collision.title', { name: existingName })}</h2>
		</div>

		<p class="col-desc">{$t('collision.desc')}</p>
		{#if existingLibrary}
			<p class="col-location">{$t('collision.inLibrary', { library: existingLibrary })}</p>
		{/if}

		<div class="col-field">
			<label for="col-name">{$t('collision.nameLabel')}</label>
			<input
				id="col-name"
				class="col-input"
				bind:value={nameValue}
				dir={detectDir(nameValue)}
				onkeydown={handleKeydown}
				use:focusInput
				autocomplete="off"
				spellcheck="false"
			/>
		</div>

		<div class="col-options">
			<button class="col-option" disabled={!canRename} onclick={() => canRename && onChangeName(trimmed)}>
				<span class="col-option-title">{$t('collision.changeNameTitle')}</span>
				<p class="col-option-desc">{$t('collision.changeNameDesc')}</p>
			</button>

			<button class="col-option danger" onclick={onOverwrite}>
				<span class="col-option-title">{$t('collision.overwriteTitle')}</span>
				<p class="col-option-desc">{$t('collision.overwriteDesc')}</p>
			</button>
		</div>

		<div class="col-footer">
			<button class="col-cancel" onclick={onCancel}>{$t('common.cancel')}</button>
		</div>
	</div>
</div>

<style>
	.col-overlay {
		position: fixed;
		inset: 0;
		z-index: 100000; /* above the create dialog / canonical dialog (99998) so it stacks on top */
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		animation: colFadeIn 0.2s ease;
	}
	.col-modal {
		background: var(--background-primary);
		border-radius: 14px;
		padding: 30px 34px;
		max-width: 520px;
		width: 92vw;
		box-shadow: var(--modal-shadow, 0 20px 60px rgba(0, 0, 0, 0.4));
	}
	.col-header {
		display: flex;
		align-items: center;
		gap: 12px;
		margin-bottom: 10px;
	}
	.col-header h2 {
		margin: 0;
		font-size: 1.1rem;
		font-weight: 700;
		line-height: 1.3;
	}
	.col-desc {
		margin: 0 0 8px;
		font-size: 0.86rem;
		color: var(--text-muted);
		line-height: 1.55;
	}
	.col-location {
		margin: 0 0 18px;
		font-size: 0.82rem;
		color: var(--text-faint);
	}
	.col-field {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin-bottom: 18px;
	}
	.col-field label {
		font-size: 0.78rem;
		color: var(--text-muted);
		font-weight: 600;
	}
	.col-input {
		padding: 9px 12px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		background: var(--background-secondary);
		color: var(--text-normal);
		font-size: 0.95rem;
		width: 100%;
		box-sizing: border-box;
	}
	.col-input:focus {
		outline: none;
		border-color: var(--text-accent);
	}
	.col-options {
		display: flex;
		flex-direction: column;
		gap: 10px;
		margin-bottom: 18px;
	}
	.col-option {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 13px 16px;
		border: 2px solid var(--background-modifier-border);
		border-radius: 10px;
		background: var(--background-primary);
		cursor: pointer;
		text-align: start;
		transition: border-color 0.15s, background 0.15s;
	}
	.col-option:hover:not(:disabled) {
		border-color: var(--text-accent);
		background: var(--background-secondary);
	}
	.col-option:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}
	.col-option.danger:hover {
		border-color: var(--text-error, #e5484d);
	}
	.col-option-title {
		font-size: 0.92rem;
		font-weight: 600;
	}
	.col-option-desc {
		margin: 2px 0 0;
		font-size: 0.8rem;
		color: var(--text-muted);
		line-height: 1.45;
	}
	.col-footer {
		display: flex;
		justify-content: flex-end;
	}
	.col-cancel {
		padding: 6px 18px;
		border-radius: 6px;
		border: 1px solid var(--background-modifier-border);
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 0.82rem;
	}
	.col-cancel:hover {
		background: var(--background-modifier-hover);
		color: var(--text-normal);
	}
	@keyframes colFadeIn {
		from { opacity: 0; }
		to { opacity: 1; }
	}
</style>
