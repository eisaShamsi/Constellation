<script lang="ts">
	import { onMount } from 'svelte';
	import { t, dir } from '$lib/i18n';
	import { appSettings, updateSettings, type ConstellationTheme } from '$lib/libraries/store';
	import {
		fetchObsidianThemeList, downloadThemeCSS, parseObsidianCSS,
		extractPreviewColors, parseStyleSettings,
		getScreenshotUrl, type ObsidianThemeEntry, type ThemePreviewColors, type StyleSettingsOption,
	} from '$lib/theme/obsidianImporter';

	let { onClose, onImported }: {
		onClose?: () => void;
		onImported?: (theme: ConstellationTheme) => void;
	} = $props();

	const isRTL = $derived($dir === 'rtl');

	let themes = $state<ObsidianThemeEntry[]>([]);
	let filteredThemes = $state<ObsidianThemeEntry[]>([]);
	let searchQuery = $state('');
	let loading = $state(true);
	let error = $state('');
	let importing = $state<string | null>(null); // repo being imported
	let previewEntry = $state<ObsidianThemeEntry | null>(null);
	let previewColors = $state<ThemePreviewColors | null>(null);
	let previewLoading = $state(false);
	let styleSettings = $state<StyleSettingsOption[]>([]);

	onMount(async () => {
		try {
			themes = await fetchObsidianThemeList();
			filteredThemes = themes;
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	});

	function filterThemes() {
		const q = searchQuery.toLowerCase();
		filteredThemes = q
			? themes.filter(t => t.name.toLowerCase().includes(q) || t.author.toLowerCase().includes(q))
			: themes;
	}

	async function previewTheme(entry: ObsidianThemeEntry) {
		if (previewEntry?.repo === entry.repo) { previewEntry = null; previewColors = null; styleSettings = []; return; }
		previewEntry = entry;
		previewColors = null;
		previewLoading = true;
		styleSettings = [];
		try {
			const css = await downloadThemeCSS(entry.repo);
			const type = entry.modes?.includes('dark') ? 'dark' : 'light';
			previewColors = extractPreviewColors(css, type);
			styleSettings = parseStyleSettings(css);
		} catch {
			previewColors = null;
		} finally {
			previewLoading = false;
		}
	}

	async function importTheme(entry: ObsidianThemeEntry) {
		importing = entry.repo;
		try {
			const css = await downloadThemeCSS(entry.repo);
			const parsed = parseObsidianCSS(css, entry.name, entry.author, entry.modes || ['dark', 'light']);
			if (parsed.length === 0) throw new Error('Could not parse theme');

			// Add all variants to custom themes
			const customs = [...($appSettings.customThemes ?? [])];
			for (const theme of parsed) {
				// Remove existing with same ID
				const idx = customs.findIndex(t => t.id === theme.id);
				if (idx >= 0) customs[idx] = theme;
				else customs.push(theme);
			}

			// Activate the first variant (dark preferred if available)
			const preferred = parsed.find(t => t.type === 'dark') ?? parsed[0];
			updateSettings({ customThemes: customs, activeThemeId: preferred.id });
			onImported?.(preferred);
		} catch (e) {
			error = `Failed to import "${entry.name}": ${e}`;
			setTimeout(() => error = '', 5000);
		} finally {
			importing = null;
		}
	}
</script>

<div class="otb-overlay" onclick={() => onClose?.()}>
	<div class="otb-modal" dir={isRTL ? 'rtl' : 'ltr'} onclick={(e) => e.stopPropagation()}>
		<!-- Header -->
		<div class="otb-header">
			<div class="otb-header-left">
				<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M8 12l2 2 4-4"/></svg>
				<span class="otb-title">{$t('settings.appearance.obsidianThemes') || 'Obsidian Community Themes'}</span>
				<span class="otb-count">{filteredThemes.length}</span>
			</div>
			<button class="otb-close" onclick={() => onClose?.()}>×</button>
		</div>

		<!-- Search -->
		<div class="otb-search">
			<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
			<input type="text" dir="auto"
				placeholder={$t('settings.appearance.searchThemes') || 'Search themes by name or author...'}
				bind:value={searchQuery}
				oninput={filterThemes} />
		</div>

		<!-- Error -->
		{#if error}
			<div class="otb-error">{error}</div>
		{/if}

		<!-- Content -->
		<div class="otb-content">
			{#if loading}
				<div class="otb-loading">
					<div class="otb-spinner"></div>
					<span>{$t('settings.appearance.loadingThemes') || 'Loading themes...'}</span>
				</div>
			{:else}
				<div class="otb-grid">
					{#each filteredThemes as entry (entry.repo)}
						<div class="otb-card">
							<div class="otb-screenshot">
								<img
									src={getScreenshotUrl(entry)}
									alt={entry.name}
									loading="lazy"
									onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
								/>
							</div>
							<div class="otb-card-info">
								<div class="otb-card-name">{entry.name}</div>
								<div class="otb-card-author">@{entry.author}</div>
								<div class="otb-card-modes">
									{#each entry.modes || [] as mode}
										<span class="otb-mode" class:dark={mode === 'dark'}>{mode}</span>
									{/each}
								</div>
							</div>
							<div class="otb-card-actions">
								<button class="otb-preview-btn"
									class:active={previewEntry?.repo === entry.repo}
									onclick={() => previewTheme(entry)}>
									{$t('settings.appearance.preview') || 'Preview'}
								</button>
								<button class="otb-import-btn"
									disabled={importing === entry.repo}
									onclick={() => importTheme(entry)}>
									{#if importing === entry.repo}
										<span class="otb-btn-spinner"></span>
									{:else}
										{$t('settings.appearance.importTheme') || 'Import'}
									{/if}
								</button>
							</div>

							<!-- Inline preview panel -->
							{#if previewEntry?.repo === entry.repo}
								<div class="otb-preview-panel">
									{#if previewLoading}
										<span class="otb-btn-spinner" style="margin:8px auto;display:block"></span>
									{:else if previewColors}
										<div class="otb-preview-sample" style="background:{previewColors.background};color:{previewColors.text};border-color:{previewColors.border}">
											<div class="otb-preview-sidebar" style="background:{previewColors.surface}">
												<div class="otb-preview-bar" style="background:{previewColors.accent};width:60%"></div>
												<div class="otb-preview-bar" style="background:{previewColors.border};width:80%"></div>
												<div class="otb-preview-bar" style="background:{previewColors.border};width:45%"></div>
											</div>
											<div class="otb-preview-content">
												<div class="otb-preview-title" style="color:{previewColors.text}">Note Title</div>
												<div class="otb-preview-text" style="color:{previewColors.text};opacity:0.7">Lorem ipsum dolor sit amet, consectetur adipiscing elit.</div>
												<div class="otb-preview-link" style="color:{previewColors.accent}">[[Related Note]]</div>
											</div>
										</div>
										<div class="otb-preview-swatches">
											<span title="Background" style="background:{previewColors.background}"></span>
											<span title="Surface" style="background:{previewColors.surface}"></span>
											<span title="Text" style="background:{previewColors.text}"></span>
											<span title="Accent" style="background:{previewColors.accent}"></span>
											<span title="Border" style="background:{previewColors.border}"></span>
										</div>
										{#if styleSettings.length > 0}
											<div class="otb-style-settings">
												<span class="otb-ss-label">Style Settings: {styleSettings.length} options</span>
											</div>
										{/if}
									{/if}
								</div>
							{/if}
						</div>
					{/each}
				</div>
				{#if filteredThemes.length === 0 && !loading}
					<div class="otb-empty">{$t('sightPanel.noResults') || 'No themes found'}</div>
				{/if}
			{/if}
		</div>
	</div>
</div>

<style>
	.otb-overlay {
		position: fixed; inset: 0; z-index: 9999;
		background: rgba(0,0,0,0.5); display: flex;
		align-items: center; justify-content: center;
	}
	.otb-modal {
		width: 90%; max-width: 900px; max-height: 85vh;
		background: var(--background-primary, #fff);
		border-radius: 12px; box-shadow: 0 8px 32px rgba(0,0,0,0.2);
		display: flex; flex-direction: column; overflow: hidden;
	}
	.otb-header {
		display: flex; align-items: center; justify-content: space-between;
		padding: 14px 20px; border-bottom: 1px solid var(--background-modifier-border, #e5e7eb);
		flex-shrink: 0;
	}
	.otb-header-left { display: flex; align-items: center; gap: 10px; }
	.otb-header-left svg { color: var(--interactive-accent, #7c3aed); }
	.otb-title { font-size: 15px; font-weight: 700; color: var(--text-normal, #1a1a1a); }
	.otb-count {
		font-size: 11px; color: var(--text-faint, #94a3b8);
		background: var(--background-modifier-border, #e5e7eb);
		padding: 1px 8px; border-radius: 10px;
	}
	.otb-close {
		border: none; background: none; font-size: 20px;
		color: var(--text-muted, #64748b); cursor: pointer; padding: 4px 8px;
	}
	.otb-close:hover { color: var(--text-normal, #1a1a1a); }
	.otb-search {
		display: flex; align-items: center; gap: 8px;
		padding: 10px 20px; border-bottom: 1px solid var(--background-modifier-border, #e5e7eb);
		flex-shrink: 0;
	}
	.otb-search svg { color: var(--text-muted, #64748b); flex-shrink: 0; }
	.otb-search input {
		border: none; outline: none; background: none; flex: 1;
		font-size: 13px; font-family: inherit; color: var(--text-normal, #1a1a1a);
	}
	.otb-error {
		padding: 8px 20px; background: #ef444415; color: #ef4444;
		font-size: 12px; flex-shrink: 0;
	}
	.otb-content { flex: 1; overflow-y: auto; padding: 16px 20px; }
	.otb-loading {
		display: flex; flex-direction: column; align-items: center;
		justify-content: center; gap: 12px; padding: 40px;
		color: var(--text-muted, #64748b); font-size: 13px;
	}
	.otb-spinner {
		width: 24px; height: 24px; border: 3px solid var(--background-modifier-border, #e5e7eb);
		border-top-color: var(--interactive-accent, #7c3aed);
		border-radius: 50%; animation: otb-spin 0.7s linear infinite;
	}
	@keyframes otb-spin { to { transform: rotate(360deg); } }
	.otb-grid {
		display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
		gap: 14px;
	}
	.otb-card {
		border: 1px solid var(--background-modifier-border, #e5e7eb);
		border-radius: 10px; overflow: hidden; background: var(--background-secondary, #f8fafc);
		display: flex; flex-direction: column;
	}
	.otb-card:hover { border-color: var(--interactive-accent, #7c3aed); }
	.otb-screenshot {
		width: 100%; height: 120px; overflow: hidden;
		background: var(--background-modifier-border, #e5e7eb);
	}
	.otb-screenshot img { width: 100%; height: 100%; object-fit: cover; }
	.otb-card-info { padding: 10px 12px; flex: 1; }
	.otb-card-name { font-size: 13px; font-weight: 600; color: var(--text-normal, #1a1a1a); }
	.otb-card-author { font-size: 11px; color: var(--text-muted, #64748b); margin-top: 2px; }
	.otb-card-modes { display: flex; gap: 4px; margin-top: 6px; }
	.otb-mode {
		font-size: 9px; padding: 1px 6px; border-radius: 4px;
		background: #f59e0b22; color: #f59e0b; text-transform: capitalize;
	}
	.otb-mode.dark { background: #6366f122; color: #6366f1; }
	.otb-card-actions { display: flex; gap: 6px; margin: 8px 12px 12px; }
	.otb-preview-btn {
		flex: 1; padding: 6px 0; border-radius: 6px;
		border: 1px solid var(--background-modifier-border, #e5e7eb);
		background: none; color: var(--text-muted, #64748b);
		font-size: 11px; cursor: pointer; font-family: inherit;
	}
	.otb-preview-btn:hover { background: var(--background-modifier-hover, #f1f5f9); }
	.otb-preview-btn.active { background: var(--interactive-accent, #7c3aed); color: white; border-color: var(--interactive-accent); }
	.otb-import-btn {
		flex: 1; padding: 6px 0; border-radius: 6px;
		border: 1px solid var(--interactive-accent, #7c3aed);
		background: none; color: var(--interactive-accent, #7c3aed);
		font-size: 11px; font-weight: 600; cursor: pointer; font-family: inherit;
		display: flex; align-items: center; justify-content: center; gap: 6px;
	}
	.otb-import-btn:hover { background: var(--interactive-accent, #7c3aed); color: white; }
	.otb-import-btn:disabled { opacity: 0.6; cursor: wait; }
	.otb-btn-spinner {
		width: 14px; height: 14px; border: 2px solid currentColor;
		border-top-color: transparent; border-radius: 50%;
		animation: otb-spin 0.6s linear infinite;
	}
	/* Preview panel */
	.otb-preview-panel { padding: 8px 12px 12px; border-top: 1px solid var(--background-modifier-border, #e5e7eb); }
	.otb-preview-sample {
		display: flex; border-radius: 6px; overflow: hidden;
		border: 1px solid; height: 80px; font-size: 10px;
	}
	.otb-preview-sidebar { width: 30%; padding: 8px 6px; display: flex; flex-direction: column; gap: 4px; }
	.otb-preview-bar { height: 4px; border-radius: 2px; }
	.otb-preview-content { flex: 1; padding: 8px; }
	.otb-preview-title { font-weight: 700; font-size: 11px; margin-bottom: 4px; }
	.otb-preview-text { font-size: 9px; line-height: 1.4; }
	.otb-preview-link { font-size: 9px; margin-top: 4px; text-decoration: underline; }
	.otb-preview-swatches {
		display: flex; gap: 4px; margin-top: 8px; justify-content: center;
	}
	.otb-preview-swatches span {
		width: 20px; height: 20px; border-radius: 50%;
		border: 1px solid rgba(0,0,0,0.15);
	}
	.otb-style-settings {
		margin-top: 6px; text-align: center;
	}
	.otb-ss-label {
		font-size: 10px; color: var(--interactive-accent, #7c3aed);
		background: rgba(124,58,237,0.08); padding: 2px 8px; border-radius: 4px;
	}
	.otb-empty {
		text-align: center; padding: 40px; color: var(--text-faint, #94a3b8);
		font-size: 13px;
	}
</style>
