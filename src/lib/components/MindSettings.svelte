<script lang="ts">
	// MIG-047 Phase 0b §F — Settings → Mind UI.
	// Shows the bundled catalog of installable models, lets the user
	// install one (chunked download from GitHub Releases with progress
	// bar), and pick which installed model is currently active. The
	// active model is what `mind_start_turn` will load once §G's
	// refactor lands.
	//
	// Until the model-pipeline workflow (§A) has run and the catalog's
	// final_sha256 is populated (not "TBD-..."), the UI shows a clear
	// "Not yet ready" badge instead of an install button.

	import { onMount } from 'svelte';
	import { invoke, Channel } from '@tauri-apps/api/core';
	import { t, locale } from '$lib/i18n';
	import { detectDir } from '$lib/utils';

	// ─── Types mirror Rust shapes in src-tauri/src/mind/model_install/ ──

	type ModelEntry = {
		id: string;
		version: string;
		display_name: string;
		display_name_ar: string;
		description: string;
		description_ar: string;
		model_family: string;
		quantization: string;
		language_focus: string[];
		context_window: number;
		license: string;
		license_notes_url: string;
		manifest_url: string;
		release_url: string;
		final_sha256: string;
		final_size_bytes: number;
	};

	type ModelsCatalog = { models: ModelEntry[] };

	type InstalledModel = {
		id: string;
		version: string;
		display_name: string;
		file_path: string;
		size_bytes: number;
		sha256: string;
		installed_at_unix: number;
	};

	type Registry = {
		models: InstalledModel[];
		active_model_id: string | null;
	};

	type DownloadProgress =
		| { stage: 'fetching_manifest'; model_id: string }
		| {
				stage: 'downloading_part';
				model_id: string;
				part_index: number;
				total_parts: number;
				part_name: string;
				bytes_done: number;
				bytes_total: number;
		  }
		| { stage: 'part_verified'; model_id: string; part_index: number; total_parts: number }
		| { stage: 'assembling'; model_id: string }
		| { stage: 'verifying_final'; model_id: string }
		| { stage: 'done'; model_id: string; final_path: string; final_size_bytes: number }
		| { stage: 'failed'; model_id: string; error: string };

	// ─── Reactive state ────────────────────────────────────────────────

	let catalog = $state<ModelsCatalog | null>(null);
	let registry = $state<Registry | null>(null);
	let loadError = $state<string | null>(null);
	let busy = $state(false);

	// Per-model installation status — keyed by model_id.
	let installing = $state<Record<string, DownloadProgress | null>>({});

	// ─── Helpers ───────────────────────────────────────────────────────

	function formatSize(bytes: number): string {
		if (bytes <= 0) return '—';
		if (bytes >= 1024 ** 3) return `${(bytes / 1024 ** 3).toFixed(2)} GiB`;
		if (bytes >= 1024 ** 2) return `${(bytes / 1024 ** 2).toFixed(1)} MiB`;
		if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KiB`;
		return `${bytes} B`;
	}

	function isInstalled(modelId: string): boolean {
		return !!registry?.models.some(m => m.id === modelId);
	}

	function isReady(entry: ModelEntry): boolean {
		return !entry.final_sha256.startsWith('TBD') && entry.final_size_bytes > 0;
	}

	function localizedName(entry: ModelEntry): string {
		// Show Arabic display name when the UI is Arabic, otherwise English.
		return $locale === 'ar' && entry.display_name_ar
			? entry.display_name_ar
			: entry.display_name;
	}

	function localizedDescription(entry: ModelEntry): string {
		return $locale === 'ar' && entry.description_ar
			? entry.description_ar
			: entry.description;
	}

	function progressPercent(p: DownloadProgress): number {
		if (p.stage !== 'downloading_part') return 0;
		const total = p.bytes_total;
		if (total === 0) return 0;
		return Math.min(100, Math.round((p.bytes_done / total) * 100));
	}

	function progressLabel(p: DownloadProgress): string {
		switch (p.stage) {
			case 'fetching_manifest':
				return $t('settings.mind.progress.fetchingManifest') || 'Fetching manifest…';
			case 'downloading_part':
				return (
					($t('settings.mind.progress.downloadingPart') ||
						`Downloading part ${p.part_index}/${p.total_parts}`) +
					` — ${formatSize(p.bytes_done)} / ${formatSize(p.bytes_total)}`
				);
			case 'part_verified':
				return $t('settings.mind.progress.partVerified') || `Part ${p.part_index}/${p.total_parts} verified`;
			case 'assembling':
				return $t('settings.mind.progress.assembling') || 'Assembling parts…';
			case 'verifying_final':
				return $t('settings.mind.progress.verifyingFinal') || 'Verifying final hash…';
			case 'done':
				return $t('settings.mind.progress.done') || 'Installed.';
			case 'failed':
				return ($t('settings.mind.progress.failed') || 'Failed') + `: ${p.error}`;
		}
	}

	// ─── IPC actions ───────────────────────────────────────────────────

	async function reloadAll() {
		loadError = null;
		try {
			const [c, r] = await Promise.all([
				invoke<ModelsCatalog>('mind_list_catalog'),
				invoke<Registry>('mind_list_installed_models'),
			]);
			catalog = c;
			registry = r;
		} catch (e) {
			loadError = String(e);
		}
	}

	async function install(modelId: string) {
		busy = true;
		installing = { ...installing, [modelId]: { stage: 'fetching_manifest', model_id: modelId } };
		const ch = new Channel<DownloadProgress>();
		ch.onmessage = (msg) => {
			installing = { ...installing, [modelId]: msg };
		};
		try {
			await invoke('mind_install_model', { modelId, onProgress: ch });
			await reloadAll();
		} catch (e) {
			installing = {
				...installing,
				[modelId]: { stage: 'failed', model_id: modelId, error: String(e) },
			};
		} finally {
			busy = false;
		}
	}

	async function setActive(modelId: string) {
		busy = true;
		try {
			await invoke('mind_set_active_model', { modelId });
			await reloadAll();
			// MIG-048 §J — re-warm the newly-active model so the user's
			// next chat turn is warm. Fire-and-forget; pre-warm errors
			// don't block the active-model change.
			try {
				await invoke('mind_prewarm_active_model');
			} catch (_) {
				/* silent — next real turn re-loads */
			}
		} catch (e) {
			loadError = String(e);
		} finally {
			busy = false;
		}
	}

	onMount(() => {
		void reloadAll();
	});
</script>

<div class="mind-settings">
	<p class="section-intro">
		{$t('settings.mind.intro') ||
			'Constellation Mind is the local Large Language Model layer. It runs entirely on your device — no cloud — and only speaks about your notes when you ask. Install a model below to enable chat with your Universe.'}
	</p>

	{#if loadError}
		<div class="error-banner" dir="auto">
			{$t('settings.mind.loadError') || 'Could not load model catalog'}: {loadError}
		</div>
	{/if}

	{#if catalog === null && !loadError}
		<div class="loading">{$t('common.loading') || 'Loading…'}</div>
	{:else if catalog}
		{#each catalog.models as entry (entry.id)}
			{@const ready = isReady(entry)}
			{@const installed = isInstalled(entry.id)}
			{@const active = registry?.active_model_id === entry.id}
			{@const progress = installing[entry.id]}
			<article class="model-card" class:active>
				<header class="model-header">
					<div class="model-title">
						<h3 dir={detectDir(localizedName(entry))}>{localizedName(entry)}</h3>
						<div class="badges">
							{#if !ready}
								<span class="badge badge-pending">
									{$t('settings.mind.badge.notReady') || 'Not yet ready'}
								</span>
							{:else if installed && active}
								<span class="badge badge-active">
									{$t('settings.mind.badge.active') || 'Active'}
								</span>
							{:else if installed}
								<span class="badge badge-installed">
									{$t('settings.mind.badge.installed') || 'Installed'}
								</span>
							{:else}
								<span class="badge badge-available">
									{$t('settings.mind.badge.available') || 'Available'}
								</span>
							{/if}
						</div>
					</div>
					<div class="model-meta">
						<span class="meta-item">{entry.quantization}</span>
						<span class="meta-item">{entry.license}</span>
						{#if entry.final_size_bytes > 0}
							<span class="meta-item">{formatSize(entry.final_size_bytes)}</span>
						{/if}
						<span class="meta-item">
							{$t('settings.mind.contextWindow') || 'Context'}: {entry.context_window} tok
						</span>
					</div>
				</header>

				<p class="model-description" dir={detectDir(localizedDescription(entry))}>
					{localizedDescription(entry)}
				</p>

				<div class="model-actions">
					{#if !ready}
						<p class="hint">
							{$t('settings.mind.hint.notReady') ||
								'This model is queued for the next Constellation release. The hash placeholder ("TBD-…") will be replaced once the model-pipeline workflow has run.'}
						</p>
					{:else if !installed}
						<button
							class="install-btn"
							disabled={busy || (progress && progress.stage !== 'failed' && progress.stage !== 'done')}
							onclick={() => install(entry.id)}>
							{$t('settings.mind.install') || 'Install'}
						</button>
					{:else if !active}
						<button class="set-active-btn" disabled={busy} onclick={() => setActive(entry.id)}>
							{$t('settings.mind.setActive') || 'Set active'}
						</button>
					{/if}

					<a class="license-link" href={entry.license_notes_url} target="_blank" rel="noopener noreferrer">
						{$t('settings.mind.licenseNotes') || 'License notes'}
					</a>

					<a class="release-link" href={entry.release_url} target="_blank" rel="noopener noreferrer">
						{$t('settings.mind.releaseNotes') || 'Release'}
					</a>
				</div>

				{#if progress}
					<div class="progress-strip" class:progress-failed={progress.stage === 'failed'}>
						<div class="progress-label">{progressLabel(progress)}</div>
						{#if progress.stage === 'downloading_part'}
							<progress max="100" value={progressPercent(progress)}></progress>
						{/if}
					</div>
				{/if}
			</article>
		{/each}
	{/if}
</div>

<style>
	.mind-settings {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		padding-bottom: 1rem;
	}

	.section-intro {
		color: var(--muted-fg, #999);
		font-size: 0.875rem;
		line-height: 1.5;
		margin: 0 0 0.5rem 0;
	}

	.error-banner {
		background: var(--danger-bg, #f8d7da);
		color: var(--danger-fg, #842029);
		padding: 0.5rem 0.75rem;
		border-radius: 6px;
		font-size: 0.875rem;
	}

	.loading {
		color: var(--muted-fg, #999);
		font-style: italic;
		font-size: 0.875rem;
	}

	.model-card {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		padding: 1rem;
		border-radius: 8px;
		border: 1px solid var(--border, #e5e7eb);
		background: var(--card-bg, transparent);
	}

	.model-card.active {
		border-color: var(--accent, #3b82f6);
		box-shadow: 0 0 0 1px var(--accent, #3b82f6) inset;
	}

	.model-header {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.model-title {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: 0.5rem;
		flex-wrap: wrap;
	}

	.model-title h3 {
		margin: 0;
		font-size: 1rem;
		font-weight: 600;
	}

	.badges {
		display: flex;
		gap: 0.25rem;
	}

	.badge {
		font-size: 0.75rem;
		padding: 0.125rem 0.5rem;
		border-radius: 12px;
		font-weight: 500;
		white-space: nowrap;
	}
	.badge-pending {
		background: var(--warning-bg, #fff3cd);
		color: var(--warning-fg, #664d03);
	}
	.badge-available {
		background: var(--info-bg, #cff4fc);
		color: var(--info-fg, #055160);
	}
	.badge-installed {
		background: var(--success-bg, #d1e7dd);
		color: var(--success-fg, #0f5132);
	}
	.badge-active {
		background: var(--accent, #3b82f6);
		color: var(--accent-fg, #fff);
	}

	.model-meta {
		display: flex;
		gap: 0.5rem;
		flex-wrap: wrap;
		font-size: 0.75rem;
		color: var(--muted-fg, #999);
	}

	.meta-item {
		padding: 0.125rem 0.375rem;
		background: var(--meta-bg, rgba(0, 0, 0, 0.04));
		border-radius: 4px;
	}

	.model-description {
		margin: 0;
		font-size: 0.875rem;
		color: var(--fg, inherit);
		line-height: 1.5;
	}

	.model-actions {
		display: flex;
		gap: 0.5rem;
		flex-wrap: wrap;
		align-items: center;
	}

	.install-btn,
	.set-active-btn {
		padding: 0.375rem 0.75rem;
		border-radius: 6px;
		font-size: 0.875rem;
		cursor: pointer;
		border: 1px solid var(--accent, #3b82f6);
		background: var(--accent, #3b82f6);
		color: var(--accent-fg, #fff);
		font-weight: 500;
	}
	.install-btn:disabled,
	.set-active-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
	.set-active-btn {
		background: transparent;
		color: var(--accent, #3b82f6);
	}

	.hint {
		margin: 0;
		font-size: 0.75rem;
		color: var(--muted-fg, #999);
		font-style: italic;
	}

	.license-link,
	.release-link {
		font-size: 0.75rem;
		color: var(--link-fg, #3b82f6);
		text-decoration: underline;
	}

	.progress-strip {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		padding: 0.5rem 0.75rem;
		background: var(--meta-bg, rgba(0, 0, 0, 0.04));
		border-radius: 6px;
		font-size: 0.75rem;
	}

	.progress-strip.progress-failed {
		background: var(--danger-bg, #f8d7da);
		color: var(--danger-fg, #842029);
	}

	.progress-label {
		font-family: var(--font-mono, monospace);
	}

	progress {
		width: 100%;
		height: 6px;
	}
</style>
