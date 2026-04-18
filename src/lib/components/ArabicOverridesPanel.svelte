<script lang="ts">
	/**
	 * Settings sub-page: manage the active Universe's Arabic Layer-0 overrides.
	 *
	 * Per the Constellation Arabic Engine five-layer pipeline, Layer 0 is the
	 * sovereign layer — when the user pins a surface (e.g. treat "خليفة" as
	 * a Proper Noun, not a verbal noun), that choice wins over the
	 * generative FST, the cascade, the heuristic, everything. This panel is
	 * the authoring surface for that layer.
	 *
	 * Backed by three Tauri commands from `arabic::overrides` (M8b):
	 *   - read_arabic_overrides → Vec<UserOverride>
	 *   - add_arabic_override(entry)
	 *   - remove_arabic_override(surface) → bool
	 * Plus one reindex command from the same module (M8c):
	 *   - reindex_arabic_overrides(surface) → u32  (count of re-tokenized notes)
	 *
	 * Every CRUD action is followed by a targeted FTS5 retokenization so the
	 * on-disk stem index reflects the new verdict immediately — users see
	 * their override land in search hits on the next keystroke, no full
	 * rebuild required.
	 */
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { t } from '$lib/i18n';
	import { detectDir } from '$lib/utils';

	type PartOfSpeech =
		| 'ProperNoun'
		| 'Noun'
		| 'Adjective'
		| 'Adverb'
		| 'Verb'
		| 'Particle'
		| 'Foreign'
		| 'Unknown';

	interface UserOverride {
		surface: string;
		lemma: string;
		root: string;
		pattern_label: string;
		pos: PartOfSpeech;
		note: string;
		created_at: string;
	}

	const POS_OPTIONS: PartOfSpeech[] = [
		'ProperNoun',
		'Noun',
		'Adjective',
		'Adverb',
		'Verb',
		'Particle',
		'Foreign',
		'Unknown'
	];

	let overrides = $state<UserOverride[]>([]);
	let loading = $state(true);
	let loadError = $state('');

	// New-override form state
	let showForm = $state(false);
	let formSurface = $state('');
	let formLemma = $state('');
	let formRoot = $state('');
	let formPattern = $state('user:override');
	let formPos = $state<PartOfSpeech>('ProperNoun');
	let formNote = $state('');
	let formError = $state('');
	let saving = $state(false);

	// Reindex status strip (transient after each mutation)
	let statusMessage = $state('');
	let statusKind = $state<'info' | 'success' | 'error'>('info');
	let statusTimer: ReturnType<typeof setTimeout> | null = null;

	function posLabel(p: PartOfSpeech): string {
		const key = 'settings.arabicOverrides.pos' + p;
		const v = $t(key);
		return v && v !== key ? v : p;
	}

	function setStatus(msg: string, kind: 'info' | 'success' | 'error' = 'info', ttlMs = 3500) {
		statusMessage = msg;
		statusKind = kind;
		if (statusTimer) clearTimeout(statusTimer);
		if (ttlMs > 0) {
			statusTimer = setTimeout(() => {
				statusMessage = '';
			}, ttlMs);
		}
	}

	async function loadOverrides() {
		loading = true;
		loadError = '';
		try {
			const list = await invoke<UserOverride[]>('read_arabic_overrides');
			overrides = list ?? [];
		} catch (e: any) {
			loadError = String(e?.message ?? e ?? 'unknown');
			overrides = [];
		} finally {
			loading = false;
		}
	}

	function openForm() {
		formSurface = '';
		formLemma = '';
		formRoot = '';
		formPattern = 'user:override';
		formPos = 'ProperNoun';
		formNote = '';
		formError = '';
		showForm = true;
	}

	function closeForm() {
		showForm = false;
		formError = '';
	}

	async function saveOverride() {
		formError = '';
		const surface = formSurface.trim();
		const lemma = formLemma.trim() || surface;
		if (!surface) {
			formError = $t('settings.arabicOverrides.surfaceRequired') || 'Surface is required.';
			return;
		}
		saving = true;
		const entry: UserOverride = {
			surface,
			lemma,
			root: formRoot.trim(),
			pattern_label: formPattern.trim() || 'user:override',
			pos: formPos,
			note: formNote.trim(),
			created_at: new Date().toISOString()
		};
		try {
			await invoke('add_arabic_override', { entry });
			closeForm();
			await loadOverrides();
			await reindexFor(surface);
		} catch (e: any) {
			formError = String(e?.message ?? e ?? 'unknown');
		} finally {
			saving = false;
		}
	}

	async function removeOverride(surface: string) {
		try {
			const removed = await invoke<boolean>('remove_arabic_override', { surface });
			if (removed) {
				await loadOverrides();
				await reindexFor(surface);
			}
		} catch (e: any) {
			setStatus(String(e?.message ?? e ?? 'unknown'), 'error');
		}
	}

	async function reindexFor(surface: string) {
		setStatus($t('settings.arabicOverrides.reindexing') || 'Reindexing…', 'info', 0);
		try {
			const count = await invoke<number>('reindex_arabic_overrides', { surface });
			const tmpl =
				$t('settings.arabicOverrides.reindexed') || 'Reindexed {count} note(s)';
			setStatus(tmpl.replace('{count}', String(count ?? 0)), 'success');
		} catch (e: any) {
			setStatus(String(e?.message ?? e ?? 'unknown'), 'error');
		}
	}

	onMount(() => {
		loadOverrides();
		return () => {
			if (statusTimer) clearTimeout(statusTimer);
		};
	});
</script>

<div class="aop-root">
	<header class="aop-header">
		<div>
			<h3>{$t('settings.arabicOverrides.title') || 'Arabic Engine Overrides'}</h3>
			<p class="aop-sub">
				{$t('settings.arabicOverrides.intro') ||
					'Pin how the engine analyses specific Arabic surfaces in this Universe. Each override is the sovereign answer — it wins over the generative FST, the cascade, and the heuristic fallback.'}
			</p>
		</div>
		<div class="aop-actions">
			<span class="aop-count"
				>{overrides.length}
				{overrides.length === 1
					? $t('settings.arabicOverrides.countOne') || 'override'
					: $t('settings.arabicOverrides.countMany') || 'overrides'}</span
			>
			<button class="aop-add-btn" onclick={openForm}>
				+ {$t('settings.arabicOverrides.add') || 'Add override'}
			</button>
		</div>
	</header>

	{#if statusMessage}
		<div class="aop-status aop-status--{statusKind}">{statusMessage}</div>
	{/if}

	{#if loading}
		<div class="aop-empty">{$t('settings.arabicOverrides.loading') || 'Loading…'}</div>
	{:else if loadError}
		<div class="aop-empty aop-empty--error">{loadError}</div>
	{:else if overrides.length === 0}
		<div class="aop-empty">{$t('settings.arabicOverrides.empty') || 'No overrides yet.'}</div>
	{:else}
		<div class="aop-table" role="table">
			<div class="aop-row aop-row--head" role="row">
				<div role="columnheader">{$t('settings.arabicOverrides.surface') || 'Surface'}</div>
				<div role="columnheader">{$t('settings.arabicOverrides.lemma') || 'Lemma'}</div>
				<div role="columnheader">{$t('settings.arabicOverrides.root') || 'Root'}</div>
				<div role="columnheader">{$t('settings.arabicOverrides.pos') || 'POS'}</div>
				<div role="columnheader">{$t('settings.arabicOverrides.note') || 'Note'}</div>
				<div role="columnheader" aria-label={$t('settings.arabicOverrides.remove') || 'Remove'}
				></div>
			</div>
			{#each overrides as o (o.surface)}
				<div class="aop-row" role="row">
					<div class="aop-cell aop-cell--rtl" dir={detectDir(o.surface)} role="cell"
						>{o.surface}</div
					>
					<div class="aop-cell aop-cell--rtl" dir={detectDir(o.lemma)} role="cell">{o.lemma}</div>
					<div class="aop-cell aop-cell--rtl" dir={detectDir(o.root)} role="cell">{o.root}</div>
					<div class="aop-cell" role="cell">{posLabel(o.pos)}</div>
					<div class="aop-cell aop-cell--note" title={o.note} role="cell">{o.note}</div>
					<div class="aop-cell aop-cell--action" role="cell">
						<button
							class="aop-remove"
							onclick={() => removeOverride(o.surface)}
							title={$t('settings.arabicOverrides.remove') || 'Remove'}
							aria-label={$t('settings.arabicOverrides.remove') || 'Remove'}>×</button
						>
					</div>
				</div>
			{/each}
		</div>
	{/if}

	{#if showForm}
		<div class="aop-form">
			<h4>{$t('settings.arabicOverrides.newTitle') || 'New override'}</h4>
			<div class="aop-field">
				<label for="aop-surface">{$t('settings.arabicOverrides.surface') || 'Surface'} *</label>
				<input
					id="aop-surface"
					type="text"
					bind:value={formSurface}
					dir={detectDir(formSurface) || 'auto'}
					placeholder="خليفة"
				/>
			</div>
			<div class="aop-field">
				<label for="aop-lemma">{$t('settings.arabicOverrides.lemma') || 'Lemma'}</label>
				<input
					id="aop-lemma"
					type="text"
					bind:value={formLemma}
					dir={detectDir(formLemma) || 'auto'}
					placeholder={formSurface || 'خليفة'}
				/>
				<span class="aop-hint"
					>{$t('settings.arabicOverrides.lemmaHint') || 'Defaults to surface if left blank.'}</span
				>
			</div>
			<div class="aop-field-row">
				<div class="aop-field">
					<label for="aop-root">{$t('settings.arabicOverrides.root') || 'Root'}</label>
					<input
						id="aop-root"
						type="text"
						bind:value={formRoot}
						dir={detectDir(formRoot) || 'auto'}
						placeholder="خ-ل-ف"
					/>
				</div>
				<div class="aop-field">
					<label for="aop-pattern">{$t('settings.arabicOverrides.pattern') || 'Pattern'}</label>
					<input
						id="aop-pattern"
						type="text"
						bind:value={formPattern}
						placeholder="user:override"
					/>
				</div>
				<div class="aop-field">
					<label for="aop-pos">{$t('settings.arabicOverrides.pos') || 'POS'}</label>
					<select id="aop-pos" bind:value={formPos}>
						{#each POS_OPTIONS as p}
							<option value={p}>{posLabel(p)}</option>
						{/each}
					</select>
				</div>
			</div>
			<div class="aop-field">
				<label for="aop-note">{$t('settings.arabicOverrides.note') || 'Note'}</label>
				<input
					id="aop-note"
					type="text"
					bind:value={formNote}
					placeholder={$t('settings.arabicOverrides.notePlaceholder') ||
						'Why does this override exist? (optional)'}
				/>
			</div>
			{#if formError}
				<div class="aop-error">{formError}</div>
			{/if}
			<div class="aop-form-actions">
				<button class="aop-save" onclick={saveOverride} disabled={saving || !formSurface.trim()}>
					{saving
						? $t('settings.arabicOverrides.saving') || 'Saving…'
						: $t('settings.arabicOverrides.save') || 'Save'}
				</button>
				<button class="aop-cancel" onclick={closeForm} disabled={saving}>
					{$t('settings.arabicOverrides.cancel') || 'Cancel'}
				</button>
			</div>
		</div>
	{/if}
</div>

<style>
	.aop-root {
		padding: 8px 4px;
	}
	.aop-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: 12px;
		margin-bottom: 10px;
	}
	.aop-header h3 {
		margin: 0;
		font-size: 14px;
		font-weight: 600;
	}
	.aop-sub {
		margin: 4px 0 0;
		font-size: 12px;
		color: var(--text-muted);
		max-width: 48em;
	}
	.aop-actions {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-shrink: 0;
	}
	.aop-count {
		font-size: 11px;
		color: var(--text-muted);
	}
	.aop-add-btn {
		background: var(--interactive-accent);
		color: var(--text-on-accent, white);
		border: none;
		border-radius: 6px;
		padding: 6px 12px;
		font-size: 12px;
		cursor: pointer;
	}
	.aop-add-btn:hover {
		opacity: 0.9;
	}

	.aop-status {
		margin: 8px 0;
		padding: 6px 10px;
		border-radius: 6px;
		font-size: 12px;
	}
	.aop-status--info {
		background: var(--background-modifier-hover);
		color: var(--text-normal);
	}
	.aop-status--success {
		background: var(--background-modifier-success, rgba(60, 180, 90, 0.15));
		color: var(--text-success, var(--text-normal));
	}
	.aop-status--error {
		background: var(--background-modifier-error, rgba(200, 60, 60, 0.15));
		color: var(--text-error, #e06666);
	}

	.aop-empty {
		padding: 24px 10px;
		text-align: center;
		color: var(--text-muted);
		font-size: 12px;
		background: var(--background-secondary);
		border: 1px dashed var(--background-modifier-border);
		border-radius: 6px;
	}
	.aop-empty--error {
		color: var(--text-error, #e06666);
		border-color: var(--text-error, #e06666);
	}

	.aop-table {
		display: flex;
		flex-direction: column;
		gap: 2px;
		margin-top: 8px;
	}
	.aop-row {
		display: grid;
		grid-template-columns: 1.2fr 1.2fr 0.9fr 0.9fr 2fr 32px;
		gap: 8px;
		align-items: center;
		padding: 6px 8px;
		background: var(--background-secondary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		font-size: 12px;
	}
	.aop-row--head {
		background: transparent;
		border: none;
		padding: 4px 8px;
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-muted);
	}
	.aop-cell {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.aop-cell--note {
		color: var(--text-muted);
	}
	.aop-cell--action {
		display: flex;
		justify-content: flex-end;
	}
	.aop-remove {
		width: 22px;
		height: 22px;
		background: none;
		border: none;
		border-radius: 4px;
		cursor: pointer;
		color: var(--text-muted);
		font-size: 16px;
		line-height: 1;
		padding: 0;
	}
	.aop-remove:hover {
		background: var(--background-modifier-hover);
		color: var(--text-error, #e06666);
	}

	.aop-form {
		margin-top: 14px;
		padding: 12px;
		background: var(--background-secondary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
	}
	.aop-form h4 {
		margin: 0 0 10px;
		font-size: 13px;
		font-weight: 600;
	}
	.aop-field {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin-bottom: 8px;
		flex: 1;
		min-width: 0;
	}
	.aop-field label {
		font-size: 11px;
		color: var(--text-muted);
	}
	.aop-field input,
	.aop-field select {
		width: 100%;
		padding: 6px 8px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		background: var(--background-primary);
		color: var(--text-normal);
		font-family: inherit;
		font-size: 12px;
	}
	.aop-hint {
		font-size: 10px;
		color: var(--text-faint);
	}
	.aop-field-row {
		display: flex;
		gap: 8px;
	}

	.aop-error {
		color: var(--text-error, #e06666);
		font-size: 12px;
		margin-bottom: 8px;
	}
	.aop-form-actions {
		display: flex;
		gap: 8px;
		margin-top: 4px;
	}
	.aop-save {
		background: var(--interactive-accent);
		color: var(--text-on-accent, white);
		border: none;
		border-radius: 6px;
		padding: 6px 14px;
		cursor: pointer;
		font-size: 12px;
	}
	.aop-save:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
	.aop-cancel {
		background: transparent;
		color: var(--text-muted);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		padding: 6px 14px;
		cursor: pointer;
		font-size: 12px;
	}
	.aop-cancel:hover {
		color: var(--text-normal);
		background: var(--background-modifier-hover);
	}
</style>
