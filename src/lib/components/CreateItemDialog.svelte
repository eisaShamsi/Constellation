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
		hideLocation = false,
		extras,
		onClose,
		onCreate,
	}: {
		open: boolean;
		kind: CreateKind;
		/** Parent location. Empty string → user must pick (Library / no-context
		 *  invocations). Non-empty → location shown read-only. Ignored when
		 *  `hideLocation` is true. */
		parentPath?: string;
		/** Hide the location field entirely (workspace bases). */
		hideLocation?: boolean;
		/** Optional kind-specific extras snippet (e.g. Base's library multi-select). */
		extras?: Snippet;
		onClose: () => void;
		/** Caller commits the create on disk + runs post-create UX. Returns
		 *  `true`/void on success (dialog closes), `false` to keep dialog open.
		 *  May throw — message displayed as inline error. */
		onCreate: (args: { name: string; location: string }) => Promise<boolean | void> | boolean | void;
	} = $props();

	// §152 — single per-kind label table. Replaces the two switch statements
	// (titleForKind, defaultNameForKind) with one lookup, keeps the i18n key
	// pairing co-located so adding a new kind is a single-row edit.
	const KIND_LABELS: Record<CreateKind, { titleKey: string; defaultNameKey: string; titleFallback: string; defaultFallback: string }> = {
		folder: { titleKey: 'createDialog.titleFolder', defaultNameKey: 'actions.newFolder', titleFallback: 'New Folder', defaultFallback: 'New Folder' },
		note: { titleKey: 'createDialog.titleNote', defaultNameKey: 'actions.untitled', titleFallback: 'New Note', defaultFallback: 'Untitled' },
		base: { titleKey: 'createDialog.titleBase', defaultNameKey: 'bases.untitled', titleFallback: 'New Base', defaultFallback: 'Untitled Base' },
		library: { titleKey: 'createDialog.titleLibrary', defaultNameKey: 'createDialog.defaultLibrary', titleFallback: 'New Library', defaultFallback: 'My Library' },
	};

	// §152 — `{#if createDialog}` re-mounts this component on each open, so
	// `$state` initializers run once per invocation. Replaces the §Build.1
	// `lastOpenState + $effect` dance with the simpler init-on-mount pattern.
	const labels = KIND_LABELS[kind];
	let name = $state($t(labels.defaultNameKey) || labels.defaultFallback);
	let location = $state(parentPath);
	let inlineError = $state('');
	let submitting = $state(false);
	let inputEl: HTMLInputElement | undefined = $state();

	// Pre-select the name on mount so overtype-from-default is one keystroke (I2).
	$effect(() => {
		queueMicrotask(() => {
			inputEl?.focus();
			inputEl?.select();
		});
	});

	// I5 — validation. Empty → disabled. Illegal chars → inline error + disabled.
	const ILLEGAL_CHARS_RE = /[\\/:*?"<>|]/;
	let validationError = $derived.by(() => {
		const trimmed = name.trim();
		if (!trimmed) return $t('createDialog.errorEmpty') || 'Name cannot be empty';
		if (ILLEGAL_CHARS_RE.test(trimmed)) {
			return $t('createDialog.errorIllegalChars') || 'Name contains illegal characters';
		}
		return '';
	});

	// Location is required only when user-pickable (not hidden, not pre-filled read-only).
	let canCreate = $derived(
		!submitting && !validationError && (hideLocation || parentPath !== '' || location.trim() !== '')
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
				return;
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
		// §152 — IME composition guard. During Arabic / CJK / any IME composition,
		// Enter commits the candidate to the input — it MUST NOT also submit the
		// dialog. Both browser flags are checked because some browsers only set
		// one (`isComposing` is the spec; `keyCode === 229` is the legacy fallback).
		if (e.isComposing || e.keyCode === 229) return;
		if (e.key === 'Escape') {
			e.preventDefault();
			handleCancel();
		} else if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleCreate();
		}
	}

	async function pickLocation() {
		try {
			const picked: string | null = await invoke('pick_folder');
			if (picked) location = picked;
		} catch { /* user cancelled */ }
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div class="dialog-overlay" onclick={handleCancel} onkeydown={handleKeydown}>
		<div class="dialog" role="dialog" aria-modal="true" aria-labelledby="cd-title" onclick={(e) => e.stopPropagation()}>
			<h2 class="dialog-title" id="cd-title">{$t(labels.titleKey) || labels.titleFallback}</h2>

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
					dir="auto"
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
