<script lang="ts">
	/**
	 * MIG-008 §Build.1 — shared modal dialog for creating Folder / Note / Base / Library.
	 *
	 * Replaces the four pre-MIG-008 inconsistent create flows (Folder
	 * auto-named "New Folder" with no dialog; Note auto-named "Untitled"
	 * with no dialog; Base in `NewBaseDialog`; Library inline `<input>`
	 * in Library Manager). Boss directive 2026-05-03: "follow the standard
	 * way of any file system. A popup dialog box should emerge to name the
	 * new folder and to choose the location."
	 *
	 * Architect plan: `lab/reports/MIG-008-CREATE-DIALOG-ARCHITECT.md`.
	 *
	 * The dialog OWNS: name input state, validation, location display /
	 * picker UI, focus management.
	 *
	 * The dialog DOES NOT OWN: the actual create-on-disk IPC nor the
	 * post-create routing (open in tab? auto-enter edit mode?). Those stay
	 * with each affordance's caller — passed in via `onCreate`.
	 */
	import { t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import type { Snippet } from 'svelte';

	export type CreateKind = 'folder' | 'note' | 'base' | 'library';

	let {
		open,
		kind,
		parentPath = '',
		defaultName,
		hideLocation = false,
		extras,
		onClose,
		onCreate,
	}: {
		open: boolean;
		kind: CreateKind;
		/** Parent location. Empty string → user must pick (Library / no-context invocations).
		 *  Non-empty → location shown read-only (right-click context already knows it).
		 *  Ignored when `hideLocation` is true. */
		parentPath?: string;
		/** Override the kind's default name. If absent, the kind's i18n default is used. */
		defaultName?: string;
		/** Hide the location field entirely (workspace bases — always live in the workspace
		 *  directory, no location for the user to pick or confirm). */
		hideLocation?: boolean;
		/** Optional kind-specific extras (e.g. Base's library multi-select). Snippet pattern. */
		extras?: Snippet;
		onClose: () => void;
		/** Caller commits the create on disk + runs post-create UX.
		 *  Receives the validated name + location.
		 *  Returns true on success (dialog closes), false on failure (dialog stays open).
		 *  May throw — message will be displayed as the inline error. */
		onCreate: (args: { name: string; location: string }) => Promise<boolean | void> | boolean | void;
	} = $props();

	// ─── State ───
	let name = $state('');
	let location = $state('');
	let inlineError = $state('');
	let submitting = $state(false);
	let inputEl: HTMLInputElement | undefined = $state();
	let lastOpenState = false;

	// Reset state on each open (capture defaults). Pre-select the name so
	// overtype-from-empty is one keystroke (I2).
	$effect(() => {
		if (open && !lastOpenState) {
			name = defaultName ?? defaultNameForKind(kind);
			location = parentPath;
			inlineError = '';
			submitting = false;
			// Focus + select on next microtask so the input element is mounted
			queueMicrotask(() => {
				inputEl?.focus();
				inputEl?.select();
			});
		}
		lastOpenState = open;
	});

	function defaultNameForKind(k: CreateKind): string {
		switch (k) {
			case 'folder': return $t('actions.newFolder') || 'New Folder';
			case 'note': return $t('actions.untitled') || 'Untitled';
			case 'base': return $t('bases.untitled') || 'Untitled Base';
			case 'library': return $t('createDialog.defaultLibrary') || 'My Library';
		}
	}

	function titleForKind(k: CreateKind): string {
		switch (k) {
			case 'folder': return $t('createDialog.titleFolder') || 'New Folder';
			case 'note': return $t('createDialog.titleNote') || 'New Note';
			case 'base': return $t('createDialog.titleBase') || 'New Base';
			case 'library': return $t('createDialog.titleLibrary') || 'New Library';
		}
	}

	// I5 — validation. Empty → disabled. Illegal chars → inline error + disabled.
	// Collision is checked at IPC time (caller surfaces errors via thrown message
	// or false return) — until the planned filename-collision popup lands.
	const ILLEGAL_CHARS_RE = /[\\/:*?"<>|]/;

	let validationError = $derived.by(() => {
		const trimmed = name.trim();
		if (!trimmed) return $t('createDialog.errorEmpty') || 'Name cannot be empty';
		if (ILLEGAL_CHARS_RE.test(trimmed)) {
			return $t('createDialog.errorIllegalChars') || 'Name contains illegal characters';
		}
		return '';
	});

	// Location is required only when it's user-pickable (not when hidden, not
	// when shown read-only — those are already known/valid by construction).
	let canCreate = $derived(
		!submitting
		&& !validationError
		&& (hideLocation || parentPath !== '' || location.trim() !== '')
	);

	async function handleCreate() {
		if (!canCreate) return;
		const trimmed = name.trim();
		submitting = true;
		inlineError = '';
		try {
			const result = await onCreate({ name: trimmed, location });
			if (result === false) {
				submitting = false;
				return; // caller decided to keep dialog open (e.g. validation failure)
			}
			onClose();
		} catch (e) {
			submitting = false;
			inlineError = String(e instanceof Error ? e.message : e);
		}
	}

	function handleCancel() {
		if (submitting) return;
		onClose();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			handleCancel();
		} else if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleCreate();
		}
	}

	async function pickLocation() {
		// Library invocation flow — pick a parent folder via the Rust folder picker.
		try {
			const picked: string | null = await invoke('pick_folder');
			if (picked) location = picked;
		} catch { /* user cancelled or picker error — leave location unchanged */ }
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div class="dialog-overlay" onclick={handleCancel} onkeydown={handleKeydown}>
		<div class="dialog" role="dialog" aria-modal="true" aria-labelledby="cd-title" onclick={(e) => e.stopPropagation()}>
			<h2 class="dialog-title" id="cd-title">{titleForKind(kind)}</h2>

			<!-- Location (omitted entirely when hideLocation; e.g. workspace bases) -->
			{#if !hideLocation}
				<div class="dialog-field">
					<label class="dialog-label" for="cd-location">{$t('createDialog.locationLabel') || 'Location'}</label>
					{#if !parentPath}
						<div class="dialog-location-pick">
							<input
								id="cd-location"
								class="dialog-input dialog-input-readonly"
								type="text"
								readonly
								value={location || ($t('createDialog.locationPlaceholder') || '— pick a location —')}
								aria-readonly="true"
							/>
							<button class="dialog-pick-btn" type="button" onclick={pickLocation}>
								{$t('createDialog.pickLocationButton') || 'Pick…'}
							</button>
						</div>
					{:else}
						<input
							id="cd-location"
							class="dialog-input dialog-input-readonly"
							type="text"
							readonly
							value={location}
							aria-readonly="true"
						/>
					{/if}
				</div>
			{/if}

			<!-- Name -->
			<div class="dialog-field">
				<label class="dialog-label" for="cd-name">{$t('createDialog.nameLabel') || 'Name'}</label>
				<input
					id="cd-name"
					class="dialog-input"
					class:dialog-input-error={validationError && name.length > 0}
					type="text"
					bind:value={name}
					bind:this={inputEl}
					autocomplete="off"
					spellcheck="false"
				/>
				{#if validationError && name.length > 0}
					<div class="dialog-error">{validationError}</div>
				{/if}
			</div>

			<!-- Kind-specific extras (e.g. Base's library multi-select) -->
			{#if extras}
				<div class="dialog-field">
					{@render extras()}
				</div>
			{/if}

			<!-- Inline error (caller-supplied, e.g. collision from IPC) -->
			{#if inlineError}
				<div class="dialog-error dialog-error-block">{inlineError}</div>
			{/if}

			<!-- Actions -->
			<div class="dialog-actions">
				<button class="dialog-btn cancel" type="button" onclick={handleCancel} disabled={submitting}>
					{$t('createDialog.cancel') || 'Cancel'}
				</button>
				<button class="dialog-btn" type="button" onclick={handleCreate} disabled={!canCreate}>
					{submitting ? ($t('createDialog.creating') || 'Creating…') : ($t('createDialog.create') || 'Create')}
				</button>
			</div>
		</div>
	</div>
{/if}

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
		min-width: 380px;
		max-width: 480px;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.dialog-title {
		margin: 0;
		font-size: 1.05rem;
		font-weight: 600;
		color: var(--text-normal);
	}
	.dialog-field {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.dialog-label {
		font-size: 0.78rem;
		color: var(--text-muted);
		font-weight: 500;
	}
	.dialog-input {
		padding: 6px 10px;
		font-size: 0.9rem;
		font-family: inherit;
		background: var(--background-primary);
		color: var(--text-normal);
		border: 1px solid var(--background-modifier-border);
		border-radius: 5px;
		outline: none;
	}
	.dialog-input:focus { border-color: var(--interactive-accent); }
	.dialog-input-readonly {
		background: var(--background-secondary);
		color: var(--text-muted);
		cursor: default;
	}
	.dialog-input-error { border-color: var(--text-error); }
	.dialog-error {
		font-size: 0.75rem;
		color: var(--text-error);
		line-height: 1.4;
	}
	.dialog-error-block {
		padding: 8px 10px;
		background: color-mix(in srgb, var(--text-error) 10%, transparent);
		border-radius: 4px;
	}
	.dialog-location-pick {
		display: flex;
		gap: 6px;
		align-items: stretch;
	}
	.dialog-location-pick .dialog-input { flex: 1; }
	.dialog-pick-btn {
		padding: 6px 12px;
		font-size: 0.82rem;
		font-family: inherit;
		background: var(--background-secondary-alt);
		color: var(--text-normal);
		border: 1px solid var(--background-modifier-border);
		border-radius: 5px;
		cursor: pointer;
		white-space: nowrap;
	}
	.dialog-pick-btn:hover { background: var(--background-modifier-border); }
	.dialog-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 4px;
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
	.dialog-btn:hover:not(:disabled) { background: var(--interactive-accent-hover); }
	.dialog-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
	.dialog-btn.cancel {
		background: var(--background-secondary-alt);
		color: var(--text-muted);
	}
	.dialog-btn.cancel:hover:not(:disabled) { background: var(--background-modifier-border); }
</style>
