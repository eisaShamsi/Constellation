<script lang="ts">
	import { t, tn } from '$lib/i18n';
	import { importPickSource, importPreview, importWithCanonical } from '$lib/importers/store';
	import type { ImportFormat, ImportPreview, ImportResult } from '$lib/importers/types';

	let {
		onClose,
		libraries = [] as { name: string; path: string }[],
		onImportComplete,
	}: {
		onClose: () => void;
		libraries: { name: string; path: string }[];
		onImportComplete?: () => void;
	} = $props();

	type Step = 'format' | 'source' | 'preview' | 'importing' | 'done';

	let step = $state<Step>('format');
	let selectedFormat = $state<ImportFormat>('markdown');
	let sourcePath = $state('');
	let targetLibrary = $state(libraries[0]?.path ?? '');
	let subfolder = $state('Imported');
	let preview = $state<ImportPreview | null>(null);
	let result = $state<ImportResult | null>(null);
	let error = $state('');
	let loading = $state(false);

	const formats: { id: ImportFormat; icon: string; labelKey: string; descKey: string }[] = [
		{ id: 'obsidian', icon: 'M12 2L3 7v10l9 5 9-5V7l-9-5zM12 22V12M3 7l9 5 9-5', labelKey: 'importer.formats.obsidian', descKey: 'importer.formats.obsidianDesc' },
		{ id: 'markdown', icon: 'M3 3h18v18H3zM7 15V9l3 4 3-4v6M17 9v6', labelKey: 'importer.formats.markdown', descKey: 'importer.formats.markdownDesc' },
		{ id: 'notion', icon: 'M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4zM14 14h6v6h-6z', labelKey: 'importer.formats.notion', descKey: 'importer.formats.notionDesc' },
		{ id: 'bear', icon: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z', labelKey: 'importer.formats.bear', descKey: 'importer.formats.bearDesc' },
		{ id: 'enex', icon: 'M9 3H5a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2V5a2 2 0 00-2-2h-4M9 3v4a1 1 0 001 1h4a1 1 0 001-1V3M9 3h6', labelKey: 'importer.formats.evernote', descKey: 'importer.formats.evernoteDesc' },
		{ id: 'html', icon: 'M4 7l4-4 4 4M4 17l4 4 4-4M14 3l4 9-4 9', labelKey: 'importer.formats.html', descKey: 'importer.formats.htmlDesc' },
		{ id: 'csv', icon: 'M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8zM14 2v6h6M8 13h2M8 17h2M14 13h2M14 17h2', labelKey: 'importer.formats.csv', descKey: 'importer.formats.csvDesc' },
		{ id: 'txt', icon: 'M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8zM14 2v6h6M16 13H8M16 17H8M10 9H8', labelKey: 'importer.formats.txt', descKey: 'importer.formats.txtDesc' },
	];

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onClose();
	}

	async function pickSource() {
		error = '';
		try {
			const pickType = ['markdown', 'notion', 'bear', 'obsidian'].includes(selectedFormat) ? 'folder' : selectedFormat;
			sourcePath = await importPickSource(pickType);
			if (sourcePath) {
				loading = true;
				step = 'preview';
				preview = await importPreview(sourcePath, selectedFormat);
				loading = false;
			}
		} catch (e: any) {
			error = e?.toString() ?? 'Failed to pick source';
			loading = false;
		}
	}

	async function executeImport() {
		error = '';
		loading = true;
		step = 'importing';
		try {
			result = await importWithCanonical(sourcePath, selectedFormat, targetLibrary, subfolder);
			step = 'done';
		} catch (e: any) {
			error = e?.toString() ?? 'Import failed';
			step = 'preview';
		}
		loading = false;
	}

	function formatBytes(bytes: number): string {
		if (bytes < 1024) return bytes + ' B';
		if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
		return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
	}

	function handleDone() {
		onImportComplete?.();
		onClose();
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="importer-overlay" onclick={onClose} onkeydown={handleKeydown} tabindex="-1" role="dialog" aria-modal="true">
	<div class="importer-modal" onclick={(e) => e.stopPropagation()}>
		<div class="importer-header">
			<h2>{$t('importer.title')}</h2>
			<button class="importer-close" onclick={onClose}>
				<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
			</button>
		</div>

		<div class="importer-body">
			{#if step === 'format'}
				<p class="importer-desc">{$t('importer.selectFormat')}</p>
				<div class="format-grid">
					{#each formats as fmt}
						<button
							class="format-card"
							class:active={selectedFormat === fmt.id}
							onclick={() => selectedFormat = fmt.id}
						>
							<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d={fmt.icon}/></svg>
							<span class="format-name">{$t(fmt.labelKey)}</span>
							<span class="format-desc">{$t(fmt.descKey)}</span>
						</button>
					{/each}
				</div>

				<div class="importer-target">
					<label class="import-label">
						{$t('importer.targetLibrary')}
						<select bind:value={targetLibrary}>
							{#each libraries as lib}
								<option value={lib.path}>{lib.name}</option>
							{/each}
						</select>
					</label>
					<label class="import-label">
						{$t('importer.subfolder')}
						<input type="text" dir="auto" bind:value={subfolder} placeholder="Imported" />
					</label>
				</div>

	
				<div class="importer-actions">
					<button class="btn-secondary" onclick={onClose}>{$t('common.cancel')}</button>
					<button class="btn-primary" onclick={pickSource}>{$t('importer.selectSource')}</button>
				</div>

			{:else if step === 'preview'}
				{#if loading}
					<div class="importer-loading">
						<div class="spinner"></div>
						<p>{$t('importer.scanning')}</p>
					</div>
				{:else if preview}
					<div class="preview-header">
						<h3>{$t('importer.previewTitle')}</h3>
						<span class="preview-count">{$tn('plurals.files', preview.file_count)}</span>
					</div>
					<div class="preview-list">
						{#each preview.files.slice(0, 50) as file}
							<div class="preview-item">
								<span class="preview-source">{file.source_name}</span>
								<span class="preview-arrow">→</span>
								<span class="preview-target">{file.target_name}</span>
								<span class="preview-size">{formatBytes(file.size_bytes)}</span>
							</div>
						{/each}
						{#if preview.files.length > 50}
							<div class="preview-more">...{$t('importer.andMore', { noun: $tn('plurals.more', preview.files.length - 50) })}</div>
						{/if}
					</div>

					{#if error}
						<div class="importer-error">{error}</div>
					{/if}

					<div class="importer-actions">
						<button class="btn-secondary" onclick={() => { step = 'format'; preview = null; }}>{$t('importer.back')}</button>
						<button class="btn-primary" onclick={executeImport}>{$t('importer.importNow')}</button>
					</div>
				{/if}

			{:else if step === 'importing'}
				<div class="importer-loading">
					<div class="spinner"></div>
					<p>{$t('importer.importing')}</p>
				</div>

			{:else if step === 'done' && result}
				<div class="import-result">
					<div class="result-icon success">
						<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
					</div>
					<h3>{$t('importer.complete')}</h3>
					<div class="result-stats">
						<div class="stat">
							<span class="stat-num">{result.imported}</span>
							<span class="stat-label">{$t('importer.imported')}</span>
						</div>
						{#if result.skipped > 0}
							<div class="stat">
								<span class="stat-num">{result.skipped}</span>
								<span class="stat-label">{$t('importer.skipped')}</span>
							</div>
						{/if}
						{#if result.errors.length > 0}
							<div class="stat error">
								<span class="stat-num">{result.errors.length}</span>
								<span class="stat-label">{$t('importer.errors')}</span>
							</div>
						{/if}
					</div>
					{#if result.errors.length > 0}
						<div class="error-list">
							{#each result.errors.slice(0, 10) as err}
								<div class="error-item">{err}</div>
							{/each}
						</div>
					{/if}
					<div class="importer-actions">
						<button class="btn-primary" onclick={handleDone}>{$t('importer.done')}</button>
					</div>
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.importer-overlay {
		position: fixed;
		inset: 0;
		z-index: 9999;
		background: rgba(0,0,0,0.55);
		display: flex;
		align-items: center;
		justify-content: center;
		animation: fadeIn 0.15s ease;
	}

	.importer-modal {
		background: var(--background-primary);
		border-radius: 12px;
		width: 600px;
		max-width: 92vw;
		max-height: 80vh;
		display: flex;
		flex-direction: column;
		box-shadow: var(--modal-shadow, 0 20px 60px rgba(0,0,0,0.3));
		overflow: hidden;
	}

	.importer-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid var(--background-modifier-border);
	}

	.importer-header h2 {
		margin: 0;
		font-size: 1.1rem;
		font-weight: 600;
	}

	.importer-close {
		background: none;
		border: none;
		cursor: pointer;
		color: var(--text-muted);
		padding: 4px;
		border-radius: 4px;
	}
	.importer-close:hover { color: var(--text-normal); }

	.importer-body {
		padding: 20px;
		overflow-y: auto;
		flex: 1;
	}

	.importer-desc {
		margin: 0 0 16px;
		color: var(--text-muted);
		font-size: 0.9rem;
	}

	.format-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
		gap: 10px;
		margin-bottom: 20px;
	}

	.format-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
		padding: 14px 10px;
		border: 2px solid var(--background-modifier-border);
		border-radius: 10px;
		background: var(--background-primary);
		cursor: pointer;
		transition: all 0.15s;
		text-align: center;
	}

	.format-card:hover {
		border-color: var(--text-accent);
		background: var(--background-secondary);
	}

	.format-card.active {
		border-color: var(--text-accent);
		background: color-mix(in srgb, var(--text-accent) 8%, transparent);
	}

	.format-name {
		font-weight: 600;
		font-size: 0.85rem;
	}

	.format-desc {
		font-size: 0.7rem;
		color: var(--text-muted);
		line-height: 1.3;
	}

	.importer-target {
		display: flex;
		gap: 12px;
		margin-bottom: 16px;
	}

	.import-label {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 4px;
		font-size: 0.82rem;
		font-weight: 500;
		color: var(--text-muted);
	}

	.import-label select,
	.import-label input {
		padding: 6px 10px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		background: var(--background-primary);
		color: var(--text-normal);
		font-size: 0.85rem;
	}

	/* RTL fixes */
	:global([dir="rtl"]) .preview-arrow { transform: scaleX(-1); }
	.preview-size { text-align: end; }

	.importer-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 16px;
	}

	.btn-primary, .btn-secondary {
		padding: 8px 20px;
		border-radius: 8px;
		font-size: 0.85rem;
		font-weight: 500;
		cursor: pointer;
		border: none;
	}

	.btn-primary {
		background: var(--text-accent);
		color: white;
	}
	.btn-primary:hover { filter: brightness(1.1); }

	.btn-secondary {
		background: var(--background-modifier-hover);
		color: var(--text-normal);
	}
	.btn-secondary:hover { background: var(--background-modifier-border); }

	.importer-loading {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		padding: 40px;
		color: var(--text-muted);
	}

	.spinner {
		width: 32px;
		height: 32px;
		border: 3px solid var(--background-modifier-border);
		border-top-color: var(--text-accent);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	.preview-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 12px;
	}

	.preview-header h3 { margin: 0; font-size: 1rem; }

	.preview-count {
		font-size: 0.82rem;
		color: var(--text-accent);
		font-weight: 600;
	}

	.preview-list {
		max-height: 300px;
		overflow-y: auto;
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
	}

	.preview-item {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 12px;
		font-size: 0.8rem;
		border-bottom: 1px solid var(--background-modifier-border);
	}

	.preview-item:last-child { border-bottom: none; }

	.preview-source {
		flex: 1;
		color: var(--text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.preview-arrow { color: var(--text-faint); }

	.preview-target {
		flex: 1;
		font-weight: 500;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.preview-size {
		font-size: 0.72rem;
		color: var(--text-faint);
		min-width: 50px;
		text-align: end;
	}

	.preview-more {
		padding: 8px 12px;
		font-size: 0.78rem;
		color: var(--text-muted);
		text-align: center;
	}

	.importer-error {
		padding: 8px 12px;
		background: color-mix(in srgb, var(--color-red) 10%, transparent);
		color: var(--color-red);
		border-radius: 6px;
		font-size: 0.82rem;
		margin-top: 8px;
	}

	.import-result {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		padding: 20px 0;
	}

	.result-icon.success { color: var(--color-green, #4caf50); }

	.import-result h3 { margin: 0; font-size: 1.15rem; }

	.result-stats {
		display: flex;
		gap: 24px;
	}

	.stat {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 2px;
	}

	.stat-num {
		font-size: 1.6rem;
		font-weight: 700;
		color: var(--text-accent);
	}

	.stat.error .stat-num { color: var(--color-red); }

	.stat-label {
		font-size: 0.75rem;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.error-list {
		width: 100%;
		max-height: 120px;
		overflow-y: auto;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		margin-top: 8px;
	}

	.error-item {
		padding: 4px 10px;
		font-size: 0.75rem;
		color: var(--color-red);
		border-bottom: 1px solid var(--background-modifier-border);
	}

	@keyframes fadeIn {
		from { opacity: 0; }
		to { opacity: 1; }
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}
</style>
