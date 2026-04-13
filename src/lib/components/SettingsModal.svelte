<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { getVersion } from '@tauri-apps/api/app';
	import { check } from '@tauri-apps/plugin-updater';
	import { relaunch } from '@tauri-apps/plugin-process';
	import { t, locale, setLocale, SUPPORTED_LOCALES, type Locale } from '$lib/i18n';
	import { appSettings, updateSettings, updateSecuritySettings, libraries, libraryStats, SCRIPT_UNICODE_RANGES, SCRIPT_LABELS, SCRIPT_SAMPLES, getAllFontSets, getFontSetById, type FontSet, TYPEWRITER_FONTS } from '$lib/libraries/store';
	import { notifySettingsChanged } from '$lib/secondScreen';
	import { aiSettings, updateAISettings, setProvider } from '$lib/ai/store';
	import { validateConnection } from '$lib/ai/engine';
	import { PROVIDER_INFO, DEFAULT_MODELS, type ProviderId } from '$lib/ai/provider';

	let {
		onClose,
		commands = [] as { id: string; name: string; shortcut?: string; icon?: string; category?: string }[],
	}: {
		onClose: () => void;
		commands?: { id: string; name: string; shortcut?: string; icon?: string; category?: string }[];
	} = $props();

	let activeSection = $state('dashboard');
	let hotkeyFilter = $state('');
	let testStatus = $state('');
	let testing = $state(false);
	let appVersion = $state('');
	getVersion().then(v => appVersion = v).catch(() => {});
	let updateChecking = $state(false);
	let updateStatus = $state('');
	let showPinSetup = $state(false);
	let pinInput = $state('');
	let pinConfirm = $state('');
	let pinError = $state('');
	let pinChanging = $state(false);
	let editingHotkey = $state<string | null>(null);
	let hotkeyListening = $state(false);

	const sections = $derived([
		{ id: 'dashboard', label: $t('settings.sections.dashboard'), icon: 'dashboard' },
		{ id: 'universe', label: $t('settings.sections.universe'), icon: 'universe' },
		{ id: 'editor', label: $t('settings.sections.editor'), icon: 'edit' },
		{ id: 'language', label: $t('settings.language.title') || 'Language', icon: 'translate' },
		{ id: 'skyview', label: $t('settings.sections.skyview'), icon: 'graph' },
		{ id: 'intelligence', label: $t('settings.sections.intelligence'), icon: 'bot' },
		{ id: 'security', label: $t('settings.sections.security'), icon: 'shield' },
		{ id: 'knowledge', label: $t('settings.sections.knowledge') || 'Knowledge Management', icon: 'brain' },
		{ id: 'appearance', label: $t('settings.sections.appearance'), icon: 'palette' },
		{ id: 'hotkeys', label: $t('settings.sections.hotkeys') || 'Hotkeys', icon: 'keyboard' },
		{ id: 'templates', label: $t('settings.sections.templates') || 'Templates', icon: 'template' },
		{ id: 'plugins', label: $t('settings.sections.plugins') || 'Plug-Ins', icon: 'grid' },
	]);

	const filteredCommands = $derived(
		hotkeyFilter.trim()
			? commands.filter(c => c.name.toLowerCase().includes(hotkeyFilter.toLowerCase()) || (c.shortcut?.toLowerCase().includes(hotkeyFilter.toLowerCase())))
			: commands
	);

	// Plug-in cards grouped by category
	const featureGroups = $derived([
		{
			category: $t('settings.plugins.navigation') || 'Navigation',
			icon: 'compass',
			features: [
				{ id: 'quickSwitcher', name: $t('settings.plugins.quickSwitcher') || 'Quick Switcher', desc: $t('settings.plugins.quickSwitcherDesc') || 'Quickly navigate between notes', icon: '⚡' },
				{ id: 'commandPalette', name: $t('settings.plugins.commandPalette') || 'Command Palette', desc: $t('settings.plugins.commandPaletteDesc') || 'Quick access to all commands', icon: '🎯' },
			]
		},
		{
			category: $t('settings.plugins.discovery') || 'Discovery',
			icon: 'eye',
			features: [
				{ id: 'graphView', name: $t('settings.plugins.graphView') || 'Sky View', desc: $t('settings.plugins.graphViewDesc') || 'Visualize links between notes', icon: '🌐' },
				{ id: 'constellationSight', name: $t('settings.plugins.constellationSight') || 'Constellation Sight', desc: $t('settings.plugins.constellationSightDesc') || 'Gravity-well knowledge visualization with analytics', icon: '👁️' },
				{ id: 'constellationMap', name: $t('settings.plugins.constellationMap') || 'Constellation Map', desc: $t('settings.plugins.constellationMapDesc') || 'Sunburst visualization of knowledge structure', icon: '🗺️' },
				{ id: 'backlinks', name: $t('settings.plugins.backlinks') || 'Backlinks', desc: $t('settings.plugins.backlinksDesc') || 'Show notes that link to the current note', icon: '🔗' },
				{ id: 'outgoingLinks', name: $t('settings.plugins.outgoingLinks') || 'Outgoing Links', desc: $t('settings.plugins.outgoingLinksDesc') || 'Show links in the current note', icon: '↗️' },
				{ id: 'pagePreview', name: $t('settings.plugins.pagePreview') || 'Page Preview', desc: $t('settings.plugins.pagePreviewDesc') || 'Preview notes on link hover', icon: '👁️' },
				{ id: 'tags', name: $t('settings.plugins.tags') || 'Tags', desc: $t('settings.plugins.tagsDesc') || 'View and browse all tags', icon: '🏷️' },
				{ id: 'index', name: $t('settings.plugins.index') || 'Index', desc: $t('settings.plugins.indexDesc') || 'Collect and browse terms from all notes', icon: '📑' },
				{ id: 'semanticSearch', name: $t('settings.plugins.semanticSearch') || 'Semantic Search', desc: $t('settings.plugins.semanticSearchDesc') || 'Find conceptually related notes using AI', icon: '🧠' },
			]
		},
		{
			category: $t('settings.plugins.organization') || 'Organization',
			icon: 'layers',
			features: [
				{ id: 'dailyNotes', name: $t('settings.plugins.dailyNotes') || 'Daily Notes', desc: $t('settings.plugins.dailyNotesDesc') || 'Create and open daily notes', icon: '📅' },
				{ id: 'workspaces', name: $t('settings.plugins.workspaces') || 'Workspaces', desc: $t('settings.plugins.workspacesDesc') || 'Save and restore workspace layouts', icon: '📐' },
				{ id: 'wordCount', name: $t('settings.plugins.wordCount') || 'Word Count', desc: $t('settings.plugins.wordCountDesc') || 'Show word count in status bar', icon: '📊' },
			]
		},
	]);

	// Dashboard computed stats
	const totalNotes = $derived($libraryStats.reduce((sum, v) => sum + (v.star_count || 0), 0));
	const totalLibraries = $derived($libraries.length);

	function handleKeydown(e: KeyboardEvent) {
		if (hotkeyListening) return;
		if (e.key === 'Escape') {
			e.preventDefault();
			e.stopPropagation();
			onClose();
		}
	}

	function handleProviderChange(e: Event) {
		const value = (e.target as HTMLSelectElement).value as ProviderId;
		if (value) setProvider(value);
	}

	// Map locale → primary script for font auto-sync
	const localeToScript: Record<string, string> = {
		en: 'latin', de: 'latin', es: 'latin', fr: 'latin', pt: 'latin', tr: 'latin',
		ar: 'arabic', fa: 'arabic', ur: 'arabic',
		he: 'hebrew',
		hi: 'devanagari',
		ja: 'cjk', ko: 'cjk', zh: 'cjk',
		ru: 'cyrillic',
	};

	function handleLangChange(e: Event) {
		const newLocale = (e.target as HTMLSelectElement).value as Locale;
		setLocale(newLocale);
		notifySettingsChanged({ locale: newLocale });
		// Auto-sync primary script to match the interface language
		const script = localeToScript[newLocale] || 'latin';
		if ($appSettings.primaryScript !== script) {
			updateSettings({ primaryScript: script });
			syncScriptToolbars(script);
		}
	}

	// Propagate visual settings changes to second screen
	let _lastSettingsHash = '';
	$effect(() => {
		const s = $appSettings;
		const hash = `${s.colorScheme}|${s.fontSize}|${s.interfaceFontSize}|${s.readableLineLength}|${s.showLineNumbers}|${s.showFloatingToolbar}|${s.fontTheme}|${s.primaryScript}|${s.accentColor}`;
		if (_lastSettingsHash && hash !== _lastSettingsHash) {
			notifySettingsChanged({
				colorScheme: s.colorScheme, fontSize: s.fontSize, interfaceFontSize: s.interfaceFontSize,
				readableLineLength: s.readableLineLength, showLineNumbers: s.showLineNumbers,
				showFloatingToolbar: s.showFloatingToolbar, fontTheme: s.fontTheme,
				primaryScript: s.primaryScript, accentColor: s.accentColor,
			});
		}
		_lastSettingsHash = hash;
	});

	// System fonts detection — start with curated list, then enhance with system fonts
	const CURATED_FONTS = [
		'Arial', 'Calibri', 'Cambria', 'Cascadia Code', 'Comic Sans MS', 'Consolas',
		'Constantia', 'Corbel', 'Courier New', 'Dubai', 'Fira Code',
		'Georgia', 'Impact', 'Inter', 'JetBrains Mono', 'Lucida Console',
		'Noto Sans', 'Noto Sans Arabic', 'Noto Naskh Arabic', 'Noto Serif',
		'Open Sans', 'Palatino Linotype', 'Roboto', 'Segoe UI',
		'Tahoma', 'Times New Roman', 'Trebuchet MS', 'Verdana',
		'Amiri', 'Cairo', 'Tajawal', 'Lora', 'Merriweather',
		'Sakkal Majalla', 'Traditional Arabic', 'Simplified Arabic',
	].sort((a, b) => a.localeCompare(b));

	let systemFonts = $state<string[]>(CURATED_FONTS);

	async function loadSystemFonts() {
		try {
			if ('queryLocalFonts' in window) {
				const fonts = await (window as any).queryLocalFonts();
				const families = new Set<string>();
				for (const font of fonts) {
					families.add(font.family);
				}
				if (families.size > 0) {
					systemFonts = [...families].sort((a, b) => a.localeCompare(b));
				}
			}
		} catch {}
	}

	// Font Sets state
	let showCustomFontSetEditor = $state(false);
	// CE Phase 9: Lens management
	let lensesLoaded = $state(false);
	let lensesList = $state<any[]>([]);
	let showLensEditor = $state(false);
	let editLensName = $state('');
	let editLensType = $state<'property-query' | 'tag-hierarchy'>('property-query');
	let editLensProperty = $state('');
	let editLensValues = $state('');
	let editLensId = $state('');
	async function loadLenses() {
		try { lensesList = await invoke('list_lenses'); lensesLoaded = true; } catch { lensesList = []; }
	}
	async function saveLensItem() {
		const lens: any = {
			id: editLensId || `lens_${Date.now()}`,
			name: editLensName,
			lens_type: editLensType,
			root_tags: editLensType === 'tag-hierarchy' ? (editLensValues ? editLensValues.split(',').map((s: string) => s.trim()) : null) : null,
			property: editLensType === 'property-query' ? editLensProperty : null,
			values: editLensType === 'property-query' && editLensValues ? editLensValues.split(',').map((s: string) => s.trim()) : null,
			built_in: false,
		};
		if (editLensId) {
			lensesList = lensesList.map(l => l.id === editLensId ? lens : l);
		} else {
			lensesList = [...lensesList, lens];
		}
		try { await invoke('save_lenses', { lenses: lensesList }); } catch {}
		showLensEditor = false;
		editLensId = ''; editLensName = ''; editLensProperty = ''; editLensValues = '';
	}
	async function deleteLens(id: string) {
		lensesList = lensesList.filter(l => l.id !== id);
		try { await invoke('save_lenses', { lenses: lensesList }); } catch {}
	}
	let editingFontSet = $state<FontSet | null>(null);
	let customSetName = $state('');
	let customSetInterface = $state('');
	let customSetText = $state('');
	let customSetMono = $state('');

	function startCreateFontSet() {
		editingFontSet = null;
		customSetName = '';
		customSetInterface = '';
		customSetText = '';
		customSetMono = '';
		showCustomFontSetEditor = true;
		loadSystemFonts();
	}

	function startEditFontSet(set: FontSet) {
		editingFontSet = set;
		customSetName = set.name;
		customSetInterface = set.interfaceFont;
		customSetText = set.textFont;
		customSetMono = set.monoFont;
		showCustomFontSetEditor = true;
		loadSystemFonts();
	}

	function saveCustomFontSet() {
		const existing = $appSettings.customFontSets || [];
		// Auto-fill empty font fields from the set name
		const iFont = customSetInterface || customSetName;
		const tFont = customSetText || customSetName;
		if (editingFontSet) {
			// Update existing
			const updated = existing.map(s => s.id === editingFontSet!.id
				? { ...s, name: customSetName, interfaceFont: iFont, textFont: tFont, monoFont: customSetMono }
				: s
			);
			updateSettings({ customFontSets: updated });
		} else {
			// Create new
			const id = 'custom-' + Date.now();
			const newSet: FontSet = { id, name: customSetName, interfaceFont: iFont, textFont: tFont, monoFont: customSetMono, isBuiltIn: false };
			updateSettings({ customFontSets: [...existing, newSet] });
		}
		showCustomFontSetEditor = false;
	}

	function deleteCustomFontSet(id: string) {
		const existing = $appSettings.customFontSets || [];
		const updates: Record<string, any> = { customFontSets: existing.filter(s => s.id !== id) };

		// Reset activeFontSetId if it points to the deleted set
		if ($appSettings.activeFontSetId === id) {
			updates.activeFontSetId = 'system';
		}

		// Reset any languageFontSets entries pointing to the deleted set
		const langSets = $appSettings.languageFontSets;
		if (langSets) {
			const cleaned = { ...langSets };
			let changed = false;
			for (const [script, setId] of Object.entries(cleaned)) {
				if (setId === id) {
					cleaned[script] = 'system';
					changed = true;
				}
			}
			if (changed) {
				updates.languageFontSets = cleaned;
			}
		}

		updateSettings(updates);
	}

	function syncScriptToolbars(primary?: string, secondary?: string) {
		if (!$appSettings.enableScriptToolbar) return;
		const scripts: string[] = [];
		const p = primary ?? $appSettings.primaryScript;
		const s = secondary ?? ($appSettings.enableSecondaryScript ? $appSettings.secondaryScript : '');
		if (p) scripts.push(p);
		if (s) scripts.push(s);
		updateSettings({ scriptToolbarScripts: scripts });
	}

	function setLanguageFontSet(script: string, fontSetId: string) {
		const current = { ...($appSettings.languageFontSets || {}) };
		current[script] = fontSetId;
		updateSettings({ languageFontSets: current });
	}

	let allFontSets = $derived(getAllFontSets($appSettings.customFontSets || []));

	let updateAvailable = $state<any>(null);
	let updateDownloading = $state(false);
	let updateProgress = $state(0);

	function getUpdateHeaders(): HeadersInit | undefined {
		const token = $appSettings.githubToken;
		if (token) {
			return { 'Authorization': `token ${token}`, 'Accept': 'application/octet-stream' };
		}
		return undefined;
	}

	async function handleCheckUpdate() {
		updateChecking = true;
		updateStatus = '';
		updateAvailable = null;
		try {
			const update = await check({ headers: getUpdateHeaders() });
			if (update) {
				updateAvailable = update;
				updateStatus = $t('settings.general.updateAvailable').replace('{version}', update.version);
			} else {
				updateStatus = $t('settings.general.upToDate');
			}
		} catch (e) {
			updateStatus = $t('settings.general.updateError');
			console.error('Update check failed:', e);
		}
		updateChecking = false;
	}

	async function handleDownloadAndInstall() {
		if (!updateAvailable) return;
		updateDownloading = true;
		updateProgress = 0;
		try {
			let totalBytes = 0;
			let downloadedBytes = 0;
			await updateAvailable.downloadAndInstall((event: any) => {
				if (event.event === 'Started' && event.data?.contentLength) {
					totalBytes = event.data.contentLength;
				} else if (event.event === 'Progress' && event.data?.chunkLength) {
					downloadedBytes += event.data.chunkLength;
					if (totalBytes > 0) updateProgress = Math.round((downloadedBytes / totalBytes) * 100);
				} else if (event.event === 'Finished') {
					updateProgress = 100;
				}
			}, { headers: getUpdateHeaders() });
			await relaunch();
		} catch (e) {
			updateStatus = $t('settings.general.updateError');
			updateDownloading = false;
			console.error('Update install failed:', e);
		}
	}

	async function testConnection() {
		if (!$aiSettings.provider) return;
		testing = true;
		testStatus = '';
		try {
			const ok = await validateConnection({
				provider: $aiSettings.provider,
				apiKey: $aiSettings.apiKey,
				model: $aiSettings.model,
				baseUrl: $aiSettings.baseUrl
			});
			testStatus = ok ? 'success' : 'failed';
		} catch {
			testStatus = 'failed';
		}
		testing = false;
	}

	function getFeatureEnabled(id: string): boolean {
		const features = ($appSettings as any).enabledFeatures;
		if (!features) return true;
		return features[id] !== false;
	}

	function toggleFeature(id: string) {
		const current = ($appSettings as any).enabledFeatures || {};
		updateSettings({
			enabledFeatures: { ...current, [id]: !getFeatureEnabled(id) }
		} as any);
	}

	function updateAIPrefs(partial: Partial<typeof $appSettings.ai>) {
		updateSettings({
			ai: { ...$appSettings.ai, ...partial }
		} as any);
	}

	function sectionIcon(icon: string): string {
		const icons: Record<string, string> = {
			dashboard: 'M3 13h8V3H3v10zm0 8h8v-6H3v6zm10 0h8V11h-8v10zm0-18v6h8V3h-8z',
			universe: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z',
			edit: 'M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z',
			graph: 'M14 6l-3.75 5 2.85 3.8-1.6 1.2C9.81 13.75 7 10 7 10l-6 8h22L14 6z',
			palette: 'M12 3c-4.97 0-9 4.03-9 9s4.03 9 9 9c.83 0 1.5-.67 1.5-1.5 0-.39-.15-.74-.39-1.01-.23-.26-.38-.61-.38-1 0-.83.67-1.5 1.5-1.5H16c2.76 0 5-2.24 5-5 0-4.42-4.03-8-9-8z',
			keyboard: 'M20 5H4c-1.1 0-1.99.9-1.99 2L2 17c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm-9 3h2v2h-2V8zm0 3h2v2h-2v-2zM8 8h2v2H8V8zm0 3h2v2H8v-2zm-1 2H5v-2h2v2zm0-3H5V8h2v2zm9 7H8v-2h8v2zm0-4h-2v-2h2v2zm0-3h-2V8h2v2zm3 3h-2v-2h2v2zm0-3h-2V8h2v2z',
			grid: 'M3 3v8h8V3H3zm6 6H5V5h4v4zm-6 4v8h8v-8H3zm6 6H5v-4h4v4zm4-16v8h8V3h-8zm6 6h-4V5h4v4zm-6 4v8h8v-8h-8zm6 6h-4v-4h4v4z',
			shield: 'M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z',
			bot: 'M12 2a2 2 0 0 1 2 2c0 .74-.4 1.39-1 1.73V7h1a7 7 0 0 1 7 7h1a1 1 0 0 1 1 1v3a1 1 0 0 1-1 1h-1v1a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-1H3a1 1 0 0 1-1-1v-3a1 1 0 0 1 1-1h1a7 7 0 0 1 7-7h1V5.73c-.6-.34-1-.99-1-1.73a2 2 0 0 1 2-2zM9 14a1 1 0 1 0 0 2 1 1 0 0 0 0-2zm6 0a1 1 0 1 0 0 2 1 1 0 0 0 0-2z',
			brain: 'M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8zm11 5a5 5 0 1 0 0-10 5 5 0 0 0 0 10zm0-2a3 3 0 1 1 0-6 3 3 0 0 1 0 6z',
			translate: 'M12.87 15.07l-2.54-2.51.03-.03A17.52 17.52 0 0014.07 6H17V4h-7V2H8v2H1v2h11.17C11.5 7.92 10.44 9.75 9 11.35 8.07 10.32 7.3 9.19 6.69 8h-2c.73 1.63 1.73 3.17 2.98 4.56l-5.09 5.02L4 19l5-5 3.11 3.11.76-2.04zM18.5 10h-2L12 22h2l1.12-3h4.75L21 22h2l-4.5-12zm-2.62 7l1.62-4.33L19.12 17h-3.24z',
		};
		return icons[icon] || icons.dashboard;
	}

	async function hashPin(pin: string): Promise<string> {
		const encoder = new TextEncoder();
		const data = encoder.encode(pin);
		const hash = await crypto.subtle.digest('SHA-256', data);
		return Array.from(new Uint8Array(hash)).map(b => b.toString(16).padStart(2, '0')).join('');
	}

	async function handleSetPin() {
		pinError = '';
		if (pinInput.length < 4) {
			pinError = $t('settings.security.pinTooShort');
			return;
		}
		if (pinInput !== pinConfirm) {
			pinError = $t('settings.security.pinMismatch');
			return;
		}
		const hash = await hashPin(pinInput);
		updateSecuritySettings({ lockPinHash: hash, lockOnIdle: true });
		pinInput = '';
		pinConfirm = '';
		showPinSetup = false;
		pinChanging = false;
	}

	function handleCancelPin() {
		pinInput = '';
		pinConfirm = '';
		pinError = '';
		showPinSetup = false;
		pinChanging = false;
	}

	function handleToggleLockOnIdle() {
		const current = $appSettings.security.lockOnIdle;
		if (!current && !$appSettings.security.lockPinHash) {
			showPinSetup = true;
			pinChanging = false;
		} else {
			updateSecuritySettings({ lockOnIdle: !current });
		}
	}

	function handleHotkeyCapture(cmdId: string, e: KeyboardEvent) {
		if (!['Shift', 'Control', 'Alt', 'Meta'].includes(e.key)) {
			e.preventDefault();
			const parts: string[] = [];
			if (e.ctrlKey) parts.push('Ctrl');
			if (e.altKey) parts.push('Alt');
			if (e.shiftKey) parts.push('Shift');
			parts.push(e.key.length === 1 ? e.key.toUpperCase() : e.key);
			// For now just display the captured shortcut (hotkey persistence is a future feature)
			editingHotkey = null;
			hotkeyListening = false;
		}
	}

	let containerEl: HTMLDivElement;
	onMount(() => { containerEl?.focus(); });
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div class="settings-overlay" onclick={onClose} onkeydown={handleKeydown} tabindex="0" bind:this={containerEl} role="dialog" aria-modal="true" aria-label={$t('settings.title')}>
	<div class="settings-modal" onclick={(e) => e.stopPropagation()}>
		<!-- Sidebar -->
		<div class="settings-sidebar">
			<div class="settings-sidebar-header">{$t('settings.title')}</div>
			{#each sections as section}
				<button
					class="settings-nav-item"
					class:active={activeSection === section.id}
					onclick={() => activeSection = section.id}
				>
					<svg class="nav-svg" width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d={sectionIcon(section.icon)}/></svg>
					<span>{section.label}</span>
				</button>
			{/each}
		</div>

		<!-- Content -->
		<div class="settings-content">
			<div class="settings-content-header">
				<h2>{sections.find(s => s.id === activeSection)?.label ?? ''}</h2>
				<button class="settings-close" onclick={onClose} aria-label="Close">
					<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
				</button>
			</div>

			<div class="settings-content-body">
				<!-- ═══ DASHBOARD ═══ -->
				{#if activeSection === 'dashboard'}
					<div class="dashboard">
						<div class="dashboard-header">
							<svg class="dash-logo" width="64" height="64" viewBox="0 0 160 160" fill="none" xmlns="http://www.w3.org/2000/svg">
								<defs>
									<linearGradient id="starGrad" x1="0%" y1="0%" x2="100%" y2="100%">
										<stop offset="0%" stop-color="var(--interactive-accent)" />
										<stop offset="100%" stop-color="var(--color-purple)" />
									</linearGradient>
									<filter id="starGlow"><feGaussianBlur stdDeviation="2" result="blur" /><feMerge><feMergeNode in="blur" /><feMergeNode in="SourceGraphic" /></feMerge></filter>
								</defs>
								<path d="M80,64 L85,75 L96,80 L85,85 L80,96 L75,85 L64,80 L75,75 Z" fill="url(#starGrad)" filter="url(#starGlow)" />
								<path d="M80,42 L82.5,47 L88,50 L82.5,53 L80,58 L77.5,53 L72,50 L77.5,47 Z" fill="url(#starGrad)" filter="url(#starGlow)" />
								<path d="M108,58 L110.5,63 L116,66 L110.5,69 L108,74 L105.5,69 L100,66 L105.5,63 Z" fill="url(#starGrad)" filter="url(#starGlow)" />
								<path d="M104,96 L106.5,101 L112,104 L106.5,107 L104,112 L101.5,107 L96,104 L101.5,101 Z" fill="url(#starGrad)" filter="url(#starGlow)" />
								<path d="M60,102 L62,106 L67,109 L62,112 L60,116 L58,112 L53,109 L58,106 Z" fill="url(#starGrad)" filter="url(#starGlow)" />
								<path d="M50,66 L52.5,71 L58,74 L52.5,77 L50,82 L47.5,77 L42,74 L47.5,71 Z" fill="url(#starGrad)" filter="url(#starGlow)" />
							</svg>
							<div class="dash-title">
								<span class="dash-name">Constellation</span>
								<span class="dash-version">v{appVersion}</span>
							</div>
						</div>

						<div class="dash-stats">
							<div class="stat-card">
								<div class="stat-value">{totalLibraries}</div>
								<div class="stat-label">{$t('settings.dashboard.libraries')}</div>
							</div>
							<div class="stat-card">
								<div class="stat-value">{totalNotes}</div>
								<div class="stat-label">{$t('settings.dashboard.notes')}</div>
							</div>
							<div class="stat-card">
								<div class="stat-value">{$aiSettings.provider ? PROVIDER_INFO[$aiSettings.provider]?.name ?? '—' : '—'}</div>
								<div class="stat-label">{$t('settings.dashboard.aiProvider')}</div>
							</div>
							<div class="stat-card">
								<div class="stat-value">{$appSettings.security.lockOnIdle ? '🔒' : '🔓'}</div>
								<div class="stat-label">{$t('settings.dashboard.lockStatus')}</div>
							</div>
						</div>

						<div class="dash-section">
							<div class="setting-item">
								<div class="setting-info">
									<div class="setting-name">{$t('settings.general.checkForUpdates')}</div>
									<div class="setting-desc">{$t('settings.general.checkForUpdatesDesc')}</div>
								</div>
								<button class="setting-btn" onclick={handleCheckUpdate} disabled={updateChecking}>
									{updateChecking ? $t('settings.general.checking') : $t('settings.general.checkNow')}
								</button>
							</div>
							{#if updateStatus}
								<div class="setting-item">
									<div class="setting-info">
										<div class="setting-desc" style="color: var(--interactive-accent)">{updateStatus}</div>
									</div>
									{#if updateAvailable && !updateDownloading}
										<button class="setting-btn" style="background: var(--interactive-accent); color: white;" onclick={handleDownloadAndInstall}>
											{$t('settings.general.downloadAndInstall')}
										</button>
									{/if}
									{#if updateDownloading}
										<div class="update-progress">
											<div class="update-progress-bar" style="width: {updateProgress}%"></div>
											<span class="update-progress-text">{updateProgress}%</span>
										</div>
									{/if}
								</div>
							{/if}
							<div class="setting-item">
								<div class="setting-info">
									<div class="setting-name">{$t('settings.general.autoUpdate')}</div>
									<div class="setting-desc">{$t('settings.general.autoUpdateDesc')}</div>
								</div>
								<label class="toggle">
									<input type="checkbox" checked={$appSettings.autoUpdate} onchange={() => updateSettings({ autoUpdate: !$appSettings.autoUpdate })} />
									<span class="toggle-slider"></span>
								</label>
							</div>
							<div class="setting-item">
								<div class="setting-info">
									<div class="setting-name">{$t('settings.general.githubToken')}</div>
									<div class="setting-desc">{$t('settings.general.githubTokenDesc')}</div>
								</div>
								<input class="setting-input" type="password" value={$appSettings.githubToken}
									placeholder="ghp_..."
									oninput={(e) => updateSettings({ githubToken: (e.target as HTMLInputElement).value })} />
							</div>
						</div>

						<div class="dash-footer">
							<span class="dash-dev">{$t('settings.dashboard.developedBy')} Eisa ALSHAMSI</span>
							<span class="dash-link">
								<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.4 5.4 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4"/><path d="M9 18c-4.51 2-5-2-7-2"/></svg>
								GitHub
							</span>
						</div>
					</div>

				<!-- ═══ UNIVERSE & LIBRARIES ═══ -->
				{:else if activeSection === 'universe'}
					<p class="section-intro">{$t('settings.universe.intro')}</p>

					<div class="setting-section-heading">{$t('settings.universe.noteDefaults')}</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.files.defaultLocation')}</div>
							<div class="setting-desc">{$t('settings.files.defaultLocationDesc')}</div>
						</div>
						<select class="setting-control" value={$appSettings.defaultNoteLocation} onchange={(e) => updateSettings({ defaultNoteLocation: (e.target as HTMLSelectElement).value as any })}>
							<option value="root">{$t('settings.files.libraryRoot')}</option>
							<option value="current">{$t('settings.files.currentFolder')}</option>
							<option value="folder">{$t('settings.files.specifiedFolder')}</option>
						</select>
					</div>

					{#if $appSettings.defaultNoteLocation === 'folder'}
						<div class="setting-item sub-setting">
							<div class="setting-info">
								<div class="setting-name">{$t('settings.files.folderPath')}</div>
							</div>
							<input class="setting-input" type="text" value={$appSettings.defaultNoteFolder}
								placeholder={$t('settings.files.folderPathPlaceholder')}
								oninput={(e) => updateSettings({ defaultNoteFolder: (e.target as HTMLInputElement).value })} />
						</div>
					{/if}

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.files.defaultAttachmentFolder')}</div>
							<div class="setting-desc">{$t('settings.files.defaultAttachmentFolderDesc')}</div>
						</div>
						<div class="setting-input-browse">
							<input class="setting-input" type="text" value={$appSettings.defaultAttachmentFolder}
								placeholder={$t('settings.files.sameAsNotePlaceholder')}
								oninput={(e) => updateSettings({ defaultAttachmentFolder: (e.target as HTMLInputElement).value })} />
							<button class="browse-btn" onclick={async () => {
								try {
									const result = await invoke<string | null>('pick_folder');
									if (result) updateSettings({ defaultAttachmentFolder: result });
								} catch { /* cancelled */ }
							}}>
								<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/></svg>
							</button>
						</div>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.files.confirmDelete')}</div>
							<div class="setting-desc">{$t('settings.files.confirmDeleteDesc')}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.confirmDelete}
								onchange={(e) => updateSettings({ confirmDelete: (e.target as HTMLInputElement).checked })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.files.deletedFiles')}</div>
							<div class="setting-desc">{$t('settings.files.deletedFilesDesc')}</div>
						</div>
						<select class="setting-control" value={$appSettings.trashDestination} onchange={(e) => updateSettings({ trashDestination: (e.target as HTMLSelectElement).value as any })}>
							<option value="system">{$t('settings.files.systemTrash')}</option>
							<option value="local">{$t('settings.files.trashFolder')}</option>
							<option value="permanent">{$t('settings.files.permanentDelete')}</option>
						</select>
					</div>

					<div class="setting-section-heading">{$t('settings.universe.templates')}</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.templates.templateFolder')}</div>
							<div class="setting-desc">{$t('settings.templates.templateFolderDesc')}</div>
						</div>
						<div class="setting-input-browse">
							<input class="setting-input" type="text" value={$appSettings.templateFolder}
								placeholder="Templates"
								oninput={(e) => updateSettings({ templateFolder: (e.target as HTMLInputElement).value })} />
							<button class="browse-btn" onclick={async () => {
								try {
									const result = await invoke<string | null>('pick_folder');
									if (result) updateSettings({ templateFolder: result });
								} catch { /* cancelled */ }
							}}>
								<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/></svg>
							</button>
						</div>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.templates.dailyNoteTemplate')}</div>
							<div class="setting-desc">{$t('settings.templates.dailyNoteTemplateDesc')}</div>
						</div>
						<input class="setting-input" type="text" value={$appSettings.dailyNoteTemplate}
							placeholder=""
							oninput={(e) => updateSettings({ dailyNoteTemplate: (e.target as HTMLInputElement).value })} />
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.templates.variables')}</div>
							<div class="setting-desc">{$t('settings.templates.variablesDesc')}</div>
						</div>
						<div class="setting-info-box">
							<code>{'{{date}}'}</code> — {$t('settings.templates.varDate')}<br/>
							<code>{'{{date:FORMAT}}'}</code> — {$t('settings.templates.varDateFormat')}<br/>
							<code>{'{{yesterday}}'}</code> — {$t('settings.templates.varYesterday')}<br/>
							<code>{'{{tomorrow}}'}</code> — {$t('settings.templates.varTomorrow')}<br/>
							<code>{'{{date+N}}'}</code> / <code>{'{{date-N}}'}</code> — {$t('settings.templates.varDateOffset')}<br/>
							<code>{'{{time}}'}</code> — {$t('settings.templates.varTime')}<br/>
							<code>{'{{title}}'}</code> — {$t('settings.templates.varTitle')}<br/>
							<code>{'{{folder}}'}</code> — {$t('settings.templates.varFolder')}<br/>
							<code>{'{{library}}'}</code> — {$t('settings.templates.varLibrary')}<br/>
							<code>{'{{clipboard}}'}</code> — {$t('settings.templates.varClipboard')}<br/>
							<code>{'{{frontmatter.KEY}}'}</code> — {$t('settings.templates.varFrontmatter')}<br/>
							<code>{'{{file.createdAt}}'}</code> — {$t('settings.templates.varFileCreated')}<br/>
							<code>{'{{file.modifiedAt}}'}</code> — {$t('settings.templates.varFileModified')}<br/>
							<code>{'{{prompt:Question}}'}</code> — {$t('settings.templates.varPrompt')}<br/>
							<code>{'{{suggester:a,b,c}}'}</code> — {$t('settings.templates.varSuggester')}<br/>
							<code>{'{{cursor}}'}</code> — {$t('settings.templates.varCursor')}
						</div>
					</div>

				<!-- ═══ EDITOR ═══ -->
				{:else if activeSection === 'editor'}
					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.editor.alwaysFocusNewTabs')}</div>
							<div class="setting-desc">{$t('settings.editor.alwaysFocusNewTabsDesc')}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.alwaysFocusNewTabs}
								onchange={(e) => updateSettings({ alwaysFocusNewTabs: (e.target as HTMLInputElement).checked })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<div class="setting-section-heading">{$t('settings.editor.display')}</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.editor.readableLineLength')}</div>
							<div class="setting-desc">{$t('settings.editor.readableLineLengthDesc')}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.readableLineLength}
								onchange={(e) => updateSettings({ readableLineLength: (e.target as HTMLInputElement).checked })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.editor.propertiesInDocument')}</div>
							<div class="setting-desc">{$t('settings.editor.propertiesInDocumentDesc')}</div>
						</div>
						<select class="setting-control" value={$appSettings.propertiesInDocument} onchange={(e) => updateSettings({ propertiesInDocument: (e.target as HTMLSelectElement).value as any })}>
							<option value="visible">{$t('settings.editor.propsVisible')}</option>
							<option value="hidden">{$t('settings.editor.propsHidden')}</option>
							<option value="source">{$t('settings.editor.propsSource')}</option>
						</select>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.editor.showLineNumbers')}</div>
							<div class="setting-desc">{$t('settings.editor.showLineNumbersDesc')}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.showLineNumbers}
								onchange={(e) => updateSettings({ showLineNumbers: (e.target as HTMLInputElement).checked })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.editor.indentationGuides')}</div>
							<div class="setting-desc">{$t('settings.editor.indentationGuidesDesc')}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.indentationGuides}
								onchange={(e) => updateSettings({ indentationGuides: (e.target as HTMLInputElement).checked })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<div class="setting-section-heading">{$t('settings.editor.behavior')}</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.editor.spellcheck')}</div>
							<div class="setting-desc">{$t('settings.editor.spellcheckDesc')}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.spellcheck}
								onchange={(e) => updateSettings({ spellcheck: (e.target as HTMLInputElement).checked })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.editor.floatingToolbar') || 'Floating toolbar'}</div>
							<div class="setting-desc">{$t('settings.editor.floatingToolbarDesc') || 'Show formatting toolbar when text is selected'}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.showFloatingToolbar}
								onchange={(e) => updateSettings({ showFloatingToolbar: (e.target as HTMLInputElement).checked })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<!-- Script Tools moved to Appearance section -->

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.editor.autoPairBrackets')}</div>
							<div class="setting-desc">{$t('settings.editor.autoPairBracketsDesc')}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.autoPairBrackets}
								onchange={(e) => updateSettings({ autoPairBrackets: (e.target as HTMLInputElement).checked })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.editor.autoPairMarkdown')}</div>
							<div class="setting-desc">{$t('settings.editor.autoPairMarkdownDesc')}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.autoPairMarkdown}
								onchange={(e) => updateSettings({ autoPairMarkdown: (e.target as HTMLInputElement).checked })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.editor.smartLists')}</div>
							<div class="setting-desc">{$t('settings.editor.smartListsDesc')}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.smartLists}
								onchange={(e) => updateSettings({ smartLists: (e.target as HTMLInputElement).checked })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.editor.tabSize')}</div>
							<div class="setting-desc">{$t('settings.editor.tabSizeDesc')}</div>
						</div>
						<select class="setting-control" value={$appSettings.tabSize} onchange={(e) => updateSettings({ tabSize: parseInt((e.target as HTMLSelectElement).value) })}>
							<option value="2">2</option>
							<option value="4">4</option>
						</select>
					</div>

				<!-- ═══ LANGUAGE ═══ -->
				{:else if activeSection === 'language'}

					<!-- ── Language & Font ── -->
					<div class="setting-heading">{$t('settings.language.interfaceLanguage')}</div>

					{#each [($appSettings.primaryScript || 'latin')] as ps}
						{@const psSetId = ($appSettings.languageFontSets || {})[ps] || 'system'}
						{@const psSet = getFontSetById(psSetId, $appSettings.customFontSets || [])}
						<div class="lang-card">
							<div class="lang-card-row">
								<select class="lang-card-select" value={$locale} onchange={handleLangChange}>
									{#each SUPPORTED_LOCALES as loc}
										<option value={loc.code}>{loc.label}</option>
									{/each}
								</select>
								<select class="lang-card-select" value={psSetId}
									onchange={(e) => setLanguageFontSet(ps, (e.target as HTMLSelectElement).value)}>
									{#each allFontSets as fs}
										<option value={fs.id}>{fs.name}</option>
									{/each}
								</select>
							</div>
							<div class="lang-card-preview" style="font-family: {psSet?.textFont || psSet?.name || 'inherit'}">
								{SCRIPT_SAMPLES[ps] || ''}
							</div>
						</div>
					{/each}

					<!-- Enable second language -->
					<div class="setting-item" style="margin-top: 4px;">
						<label class="setting-checkbox">
							<input type="checkbox" checked={$appSettings.enableSecondaryScript}
								onchange={(e) => { const v = (e.target as HTMLInputElement).checked; updateSettings({ enableSecondaryScript: v }); syncScriptToolbars(undefined, v ? $appSettings.secondaryScript : ''); }} />
							<span>{$t('fontSets.enableSecondLanguage') || 'Enable second language'}</span>
						</label>
					</div>

						{#if $appSettings.enableSecondaryScript}
							{#each [($appSettings.secondaryScript || 'arabic')] as ss}
								{@const ssSetId = ($appSettings.languageFontSets || {})[ss] || 'system'}
								{@const ssSet = getFontSetById(ssSetId, $appSettings.customFontSets || [])}
								<div class="lang-card">
									<div class="lang-card-title">{$t('fontSets.secondaryLanguage') || 'Secondary Language'}</div>
									<div class="lang-card-row">
										<select class="lang-card-select" value={ss}
											onchange={(e) => { const v = (e.target as HTMLSelectElement).value; updateSettings({ secondaryScript: v }); syncScriptToolbars(undefined, v); }}>
											{#each Object.keys(SCRIPT_UNICODE_RANGES).filter(s => s !== ($appSettings.primaryScript || 'latin')) as script}
												<option value={script}>{SCRIPT_LABELS[script] || script}</option>
											{/each}
										</select>
										<select class="lang-card-select" value={ssSetId}
											onchange={(e) => setLanguageFontSet(ss, (e.target as HTMLSelectElement).value)}>
											{#each allFontSets as fs}
												<option value={fs.id}>{fs.name}</option>
											{/each}
										</select>
									</div>
									<div class="lang-card-preview" style="font-family: {ssSet?.textFont || ssSet?.name || 'inherit'}">
										{SCRIPT_SAMPLES[ss] || ''}
									</div>
								</div>
							{/each}
						{/if}

					<!-- ── Date & Numbers ── -->
					<div class="setting-heading">{$t('settings.language.dateAndNumbers')}</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.appearance.numeralStyle') || 'Numeral style'}</div>
							<div class="setting-desc">{$t('settings.appearance.numeralStyleDesc')}</div>
						</div>
						<select class="setting-control" value={$appSettings.numeralStyle || 'arabic'}
							onchange={(e) => updateSettings({ numeralStyle: (e.target as HTMLSelectElement).value as any })}>
							<option value="arabic">{$t('settings.appearance.arabicNumerals') || 'Arabic (0-9)'}</option>
							<option value="hindi">{$t('settings.appearance.hindiNumerals')}</option>
						</select>
					</div>

					{#each [$appSettings.primaryScript || 'latin', ...(($appSettings.enableSecondaryScript && $appSettings.secondaryScript) ? [$appSettings.secondaryScript] : [])].filter(Boolean) as script}
						{@const fmt = ($appSettings.scriptDateFormats || {})[script] || $appSettings.dateFormat || 'DD/MM/YYYY'}
						{@const isContextual = ($appSettings.contextualDates || {})[script] ?? false}
						<div class="lang-card-row" style="padding: 4px 16px;">
							<span class="lang-card-label">{SCRIPT_LABELS[script] || script}</span>
							<select class="lang-card-select" value={fmt}
								onchange={(e) => {
									const current = { ...($appSettings.scriptDateFormats || {}) };
									current[script] = (e.target as HTMLSelectElement).value;
									updateSettings({ scriptDateFormats: current });
								}}>
								<option value="DD/MM/YYYY">DD/MM/YYYY</option>
								<option value="MM/DD/YYYY">MM/DD/YYYY</option>
								<option value="YYYY-MM-DD">YYYY-MM-DD</option>
								<option value="YYYY/MM/DD">YYYY/MM/DD</option>
								<option value="DD.MM.YYYY">DD.MM.YYYY</option>
								<option value="D MMMM YYYY">D MMMM YYYY</option>
								<option value="MMMM D, YYYY">MMMM D, YYYY</option>
							</select>
							<label style="display: flex; align-items: center; gap: 6px; margin-inline-start: 8px;">
								<input type="checkbox" checked={isContextual}
									onchange={(e) => {
										const current = { ...($appSettings.contextualDates || {}) };
										current[script] = (e.target as HTMLInputElement).checked;
										updateSettings({ contextualDates: current });
									}} />
								<span style="font-size: 12px; color: var(--text-muted);">{$t('settings.appearance.contextual') || 'Contextual'}</span>
							</label>
						</div>
					{/each}

					<!-- ── Script Tools ── -->
					<div class="setting-heading">{$t('settings.language.scriptTools')}</div>
					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.language.scriptToolsDesc')}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.enableScriptToolbar}
								onchange={(e) => {
									const enabled = (e.target as HTMLInputElement).checked;
									const scripts: string[] = [];
									if ($appSettings.primaryScript) scripts.push($appSettings.primaryScript);
									if ($appSettings.enableSecondaryScript && $appSettings.secondaryScript) scripts.push($appSettings.secondaryScript);
									updateSettings({ enableScriptToolbar: enabled, scriptToolbarScripts: scripts });
								}} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<!-- ── Font Theme ── -->
					<div class="setting-heading">{$t('settings.appearance.fontTheme') || 'Font Theme'}</div>
					<div class="font-theme-cards">
						<button class="font-theme-card" class:active={($appSettings.fontTheme || 'default') === 'default'}
							onclick={() => updateSettings({ fontTheme: 'default' })}>
							<div class="font-theme-preview" style="font-family: inherit;">Aa</div>
							<div class="font-theme-label">{$t('settings.appearance.fontThemeDefault') || 'Default'}</div>
						</button>
						<button class="font-theme-card" class:active={$appSettings.fontTheme === 'typewriter'}
							onclick={() => updateSettings({ fontTheme: $appSettings.fontTheme === 'typewriter' ? 'default' : 'typewriter' })}>
							<div class="font-theme-preview" style="font-family: 'Courier Prime', monospace;">Aa</div>
							<div class="font-theme-label">{$t('settings.appearance.fontThemeTypewriter') || 'Typewriter'}</div>
							<div class="font-theme-scripts">en · ar · עב · हि · Ру · 中 · 日 · 한</div>
						</button>
					</div>
					{#if $appSettings.fontTheme === 'typewriter'}
					<div class="font-theme-info">
						<span>Courier Prime · Noto Naskh · Miriam Libre · PT Mono · Tiro Devanagari · system CJK</span>
					</div>
					{/if}

					<!-- ── Custom Font Sets ── -->
					<div class="setting-heading">{$t('fontSets.customFontSets') || 'Custom Font Sets'}</div>
					{#each ($appSettings.customFontSets || []) as customSet}
						<div class="custom-fontset-row">
							<span class="custom-fontset-name">{customSet.name}</span>
							<div class="custom-fontset-actions">
								<button class="custom-fontset-btn" onclick={() => startEditFontSet(customSet)}>
									<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 3a2.85 2.85 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/></svg>
								</button>
								<button class="custom-fontset-btn custom-fontset-delete" onclick={() => deleteCustomFontSet(customSet.id)}>
									<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
								</button>
							</div>
						</div>
					{/each}

					<button class="create-fontset-btn" onclick={startCreateFontSet}>
						+ {$t('fontSets.createFontSet') || 'Create Font Set'}
					</button>

					{#if showCustomFontSetEditor}
						<div class="fontset-editor">
							<div class="fontset-editor-field">
								<label>{$t('fontSets.fontSetName') || 'Name'}</label>
								<input type="text" bind:value={customSetName} placeholder="My Font Set" />
							</div>
							<div class="fontset-editor-field">
								<label>{$t('fontSets.interfaceFont') || 'Interface Font'}</label>
								<select value={customSetInterface} onchange={(e) => { customSetInterface = (e.target as HTMLSelectElement).value; if (!customSetName) customSetName = customSetInterface; }}>
									<option value="">{$t('fontSets.systemDefault') || 'System Default'}</option>
									{#each systemFonts as font}
										<option value={font}>{font}</option>
									{/each}
								</select>
							</div>
							<div class="fontset-editor-field">
								<label>{$t('fontSets.textFont') || 'Text Font'}</label>
								<select value={customSetText} onchange={(e) => customSetText = (e.target as HTMLSelectElement).value}>
									<option value="">{$t('fontSets.systemDefault') || 'System Default'}</option>
									{#each systemFonts as font}
										<option value={font}>{font}</option>
									{/each}
								</select>
							</div>
							<div class="fontset-editor-field">
								<label>{$t('fontSets.monoFont') || 'Monospace Font'}</label>
								<select value={customSetMono} onchange={(e) => customSetMono = (e.target as HTMLSelectElement).value}>
									<option value="">{$t('fontSets.systemDefault') || 'System Default'}</option>
									{#each systemFonts as font}
										<option value={font}>{font}</option>
									{/each}
								</select>
							</div>
							<div class="fontset-editor-actions">
								<button class="fontset-save-btn" onclick={saveCustomFontSet} disabled={!customSetName}>
									{$t('fontSets.save') || 'Save'}
								</button>
								<button class="fontset-cancel-btn" onclick={() => showCustomFontSetEditor = false}>
									{$t('fontSets.cancel') || 'Cancel'}
								</button>
							</div>
						</div>
					{/if}

				<!-- ═══ SKY VIEW & LINKS ═══ -->
				{:else if activeSection === 'skyview'}
					<p class="section-intro">{$t('settings.skyview.intro')}</p>

					<div class="setting-section-heading">{$t('settings.skyview.linking')}</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.files.linkFormat')}</div>
							<div class="setting-desc">{$t('settings.files.linkFormatDesc')}</div>
						</div>
						<select class="setting-control" value={$appSettings.linkFormat} onchange={(e) => updateSettings({ linkFormat: (e.target as HTMLSelectElement).value as any })}>
							<option value="shortest">{$t('settings.files.shortestPath')}</option>
							<option value="relative">{$t('settings.files.relativePath')}</option>
							<option value="absolute">{$t('settings.files.absolutePath')}</option>
						</select>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.files.autoUpdateLinks')}</div>
							<div class="setting-desc">{$t('settings.files.autoUpdateLinksDesc')}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.autoUpdateLinks}
								onchange={(e) => updateSettings({ autoUpdateLinks: (e.target as HTMLInputElement).checked })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.files.useWikilinks')}</div>
							<div class="setting-desc">{$t('settings.files.useWikilinksDesc')}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.useWikilinks}
								onchange={(e) => updateSettings({ useWikilinks: (e.target as HTMLInputElement).checked })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<div class="setting-section-heading">{$t('settings.skyview.graphAppearance')}</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.skyview.nodeSize')}</div>
							<div class="setting-desc">{$t('settings.skyview.nodeSizeDesc')}</div>
						</div>
						<div class="setting-range">
							<input type="range" min="0.5" max="10" step="0.5" value={$appSettings.skyView?.nodeSize ?? 1.5}
								oninput={(e) => updateSettings({ skyView: { ...$appSettings.skyView, nodeSize: Number((e.target as HTMLInputElement).value) } })} />
							<span class="range-value">{$appSettings.skyView?.nodeSize ?? 1.5}</span>
						</div>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.skyview.labelVisibility')}</div>
							<div class="setting-desc">{$t('settings.skyview.labelVisibilityDesc')}</div>
						</div>
						<select class="setting-control" value={$appSettings.skyView?.labelVisibility ?? 'hover'}
							onchange={(e) => updateSettings({ skyView: { ...$appSettings.skyView, labelVisibility: (e.target as HTMLSelectElement).value as any } })}>
							<option value="hover">{$t('settings.skyview.labelHover')}</option>
							<option value="always">{$t('settings.skyview.labelAlways')}</option>
							<option value="none">{$t('settings.skyview.labelNone')}</option>
						</select>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.skyview.labelFontSize')}</div>
						</div>
						<div class="setting-range">
							<input type="range" min="8" max="20" step="1" value={$appSettings.skyView?.labelFontSize ?? 12}
								oninput={(e) => updateSettings({ skyView: { ...$appSettings.skyView, labelFontSize: Number((e.target as HTMLInputElement).value) } })} />
							<span class="range-value">{$appSettings.skyView?.labelFontSize ?? 12}px</span>
						</div>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.skyview.linkThickness')}</div>
							<div class="setting-desc">{$t('settings.skyview.linkThicknessDesc')}</div>
						</div>
						<div class="setting-range">
							<input type="range" min="0.5" max="3" step="0.5" value={$appSettings.skyView?.linkThickness ?? 1}
								oninput={(e) => updateSettings({ skyView: { ...$appSettings.skyView, linkThickness: Number((e.target as HTMLInputElement).value) } })} />
							<span class="range-value">{$appSettings.skyView?.linkThickness ?? 1}</span>
						</div>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.skyview.showOrphans')}</div>
							<div class="setting-desc">{$t('settings.skyview.showOrphansDesc')}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.skyView?.showOrphans ?? true}
								onchange={(e) => updateSettings({ skyView: { ...$appSettings.skyView, showOrphans: (e.target as HTMLInputElement).checked } })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<div class="setting-section-heading">{$t('settings.skyview.physics')}</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.skyview.repelForce')}</div>
							<div class="setting-desc">{$t('settings.skyview.repelForceDesc')}</div>
						</div>
						<div class="setting-range">
							<input type="range" min="10" max="200" step="5" value={$appSettings.skyView?.repelForce ?? 80}
								oninput={(e) => updateSettings({ skyView: { ...$appSettings.skyView, repelForce: Number((e.target as HTMLInputElement).value) } })} />
							<span class="range-value">{$appSettings.skyView?.repelForce ?? 80}</span>
						</div>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.skyview.linkForce')}</div>
							<div class="setting-desc">{$t('settings.skyview.linkForceDesc')}</div>
						</div>
						<div class="setting-range">
							<input type="range" min="0.01" max="0.2" step="0.01" value={$appSettings.skyView?.linkForce ?? 0.05}
								oninput={(e) => updateSettings({ skyView: { ...$appSettings.skyView, linkForce: Number((e.target as HTMLInputElement).value) } })} />
							<span class="range-value">{($appSettings.skyView?.linkForce ?? 0.05).toFixed(2)}</span>
						</div>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.skyview.linkDistance')}</div>
							<div class="setting-desc">{$t('settings.skyview.linkDistanceDesc')}</div>
						</div>
						<div class="setting-range">
							<input type="range" min="20" max="150" step="5" value={$appSettings.skyView?.linkDistance ?? 30}
								oninput={(e) => updateSettings({ skyView: { ...$appSettings.skyView, linkDistance: Number((e.target as HTMLInputElement).value) } })} />
							<span class="range-value">{$appSettings.skyView?.linkDistance ?? 30}</span>
						</div>
					</div>

				<!-- ═══ INTELLIGENCE (AI) ═══ -->
				{:else if activeSection === 'intelligence'}
					<p class="section-intro">{$t('settings.intelligence.intro')}</p>

					<div class="setting-section-heading">{$t('settings.intelligence.connection')}</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.ai.provider')}</div>
							<div class="setting-desc">{$t('settings.ai.providerDesc')}</div>
						</div>
						<select class="setting-control" value={$aiSettings.provider ?? ''} onchange={handleProviderChange}>
							<option value="">— {$t('settings.ai.none')} —</option>
							{#each Object.entries(PROVIDER_INFO) as [id, info]}
								<option value={id}>{info.name}</option>
							{/each}
						</select>
					</div>

					{#if $aiSettings.provider}
						{@const info = PROVIDER_INFO[$aiSettings.provider]}

						{#if info.requiresKey}
							<div class="setting-item">
								<div class="setting-info">
									<div class="setting-name">{$t('settings.ai.apiKey')}</div>
									<div class="setting-desc">{$t('settings.ai.apiKeyDesc')}</div>
								</div>
								<input class="setting-input" type="password"
									placeholder={$t('settings.ai.apiKeyPlaceholder')}
									value={$aiSettings.apiKey}
									oninput={(e) => updateAISettings({ apiKey: (e.target as HTMLInputElement).value })} />
							</div>
						{/if}

						{#if info.hasBaseUrl}
							<div class="setting-item">
								<div class="setting-info">
									<div class="setting-name">{$t('settings.ai.serverUrl')}</div>
									<div class="setting-desc">{$t('settings.ai.serverUrlDesc')}</div>
								</div>
								<input class="setting-input" type="text"
									placeholder="http://localhost:11434"
									value={$aiSettings.baseUrl}
									oninput={(e) => updateAISettings({ baseUrl: (e.target as HTMLInputElement).value })} />
							</div>
						{/if}

						<div class="setting-item">
							<div class="setting-info">
								<div class="setting-name">{$t('settings.ai.model')}</div>
								<div class="setting-desc">{$t('settings.ai.modelDesc')}</div>
							</div>
							<input class="setting-input" type="text"
								value={$aiSettings.model}
								placeholder={DEFAULT_MODELS[$aiSettings.provider]}
								oninput={(e) => updateAISettings({ model: (e.target as HTMLInputElement).value })} />
						</div>

						<div class="setting-item">
							<div class="setting-info">
								<div class="setting-name">{$t('settings.ai.connectionTest')}</div>
								<div class="setting-desc">
									{#if testStatus === 'success'}
										<span class="test-success">{$t('settings.ai.connectedSuccess')}</span>
									{:else if testStatus === 'failed'}
										<span class="test-failed">{$t('settings.ai.connectionFailed')}</span>
									{:else}
										{$t('settings.ai.verifyConnection')}
									{/if}
								</div>
							</div>
							<button class="test-btn" onclick={testConnection} disabled={testing}>
								{testing ? $t('settings.ai.testing') : $t('settings.ai.test')}
							</button>
						</div>
					{/if}

					<div class="setting-section-heading">{$t('settings.intelligence.preferences')}</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.intelligence.contextLines')}</div>
							<div class="setting-desc">{$t('settings.intelligence.contextLinesDesc')}</div>
						</div>
						<div class="slider-row">
							<input type="range" class="setting-slider" min="10" max="200" step="10" value={$appSettings.ai.contextLines}
								oninput={(e) => updateAIPrefs({ contextLines: parseInt((e.target as HTMLInputElement).value) })} />
							<span class="slider-val">{$appSettings.ai.contextLines}</span>
						</div>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.intelligence.libraryAccess')}</div>
							<div class="setting-desc">{$t('settings.intelligence.libraryAccessDesc')}</div>
						</div>
						<select class="setting-control" value={$appSettings.ai.libraryAccess} onchange={(e) => updateAIPrefs({ libraryAccess: (e.target as HTMLSelectElement).value as any })}>
							<option value="all">{$t('settings.intelligence.allLibraries')}</option>
							<option value="active">{$t('settings.intelligence.activeOnly')}</option>
							<option value="none">{$t('settings.intelligence.noAccess')}</option>
						</select>
					</div>

				<!-- ═══ SECURITY ═══ -->
				{:else if activeSection === 'security'}
					<p class="section-intro">{$t('settings.security.intro')}</p>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.security.libraryEncryption')}</div>
							<div class="setting-desc">{$t('settings.security.libraryEncryptionDesc')}</div>
						</div>
						<div class="security-control-row">
							{#if $appSettings.security.libraryEncryption}
								<span class="security-badge active">{$t('settings.security.enabled')}</span>
							{:else}
								<span class="security-badge">{$t('settings.security.disabled')}</span>
							{/if}
							<label class="toggle">
								<input type="checkbox"
									checked={$appSettings.security.libraryEncryption}
									onchange={() => updateSecuritySettings({ libraryEncryption: !$appSettings.security.libraryEncryption })} />
								<span class="toggle-slider"></span>
							</label>
						</div>
					</div>

					<div class="setting-heading">{$t('settings.security.lockHeading')}</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.security.lockOnIdle')}</div>
							<div class="setting-desc">{$t('settings.security.lockOnIdleDesc')}</div>
						</div>
						<label class="toggle">
							<input type="checkbox"
								checked={$appSettings.security.lockOnIdle}
								onchange={handleToggleLockOnIdle} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					{#if $appSettings.security.lockOnIdle}
						<div class="setting-item sub-setting">
							<div class="setting-info">
								<div class="setting-name">{$t('settings.security.idleTimeout')}</div>
								<div class="setting-desc">{$t('settings.security.idleTimeoutDesc')}</div>
							</div>
							<select class="setting-control"
								value={$appSettings.security.lockIdleTimeout}
								onchange={(e) => updateSecuritySettings({ lockIdleTimeout: parseInt((e.target as HTMLSelectElement).value) })}>
								<option value={1}>1 {$t('settings.security.minutes')}</option>
								<option value={5}>5 {$t('settings.security.minutes')}</option>
								<option value={10}>10 {$t('settings.security.minutes')}</option>
								<option value={15}>15 {$t('settings.security.minutes')}</option>
								<option value={30}>30 {$t('settings.security.minutes')}</option>
								<option value={60}>60 {$t('settings.security.minutes')}</option>
							</select>
						</div>

						<div class="setting-item sub-setting">
							<div class="setting-info">
								<div class="setting-name">{$t('settings.security.pin')}</div>
								<div class="setting-desc">{$t('settings.security.pinDesc')}</div>
							</div>
							<button class="setting-btn" onclick={() => { showPinSetup = true; pinChanging = true; }}>
								{$t('settings.security.changePin')}
							</button>
						</div>
					{/if}

					{#if showPinSetup}
						<div class="pin-setup">
							<div class="pin-setup-title">
								{pinChanging ? $t('settings.security.changePin') : $t('settings.security.setPin')}
							</div>
							<div class="pin-fields">
								<input class="setting-input" type="password"
									placeholder={$t('settings.security.enterPin')}
									maxlength="8"
									value={pinInput}
									oninput={(e) => pinInput = (e.target as HTMLInputElement).value} />
								<input class="setting-input" type="password"
									placeholder={$t('settings.security.confirmPin')}
									maxlength="8"
									value={pinConfirm}
									oninput={(e) => pinConfirm = (e.target as HTMLInputElement).value} />
							</div>
							{#if pinError}
								<div class="pin-error">{pinError}</div>
							{/if}
							<div class="pin-actions">
								<button class="dialog-btn cancel" onclick={handleCancelPin}>{$t('common.cancel')}</button>
								<button class="setting-btn" onclick={handleSetPin}>{$t('common.confirm')}</button>
							</div>
						</div>
					{/if}

					<div class="setting-heading">{$t('settings.security.apiHeading')}</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.security.apiKeyProtection')}</div>
							<div class="setting-desc">{$t('settings.security.apiKeyProtectionDesc')}</div>
						</div>
						<div class="security-control-row">
							{#if $appSettings.security.apiKeyProtection}
								<span class="security-badge active">{$t('settings.security.enabled')}</span>
							{:else}
								<span class="security-badge">{$t('settings.security.disabled')}</span>
							{/if}
							<label class="toggle">
								<input type="checkbox"
									checked={$appSettings.security.apiKeyProtection}
									onchange={() => updateSecuritySettings({ apiKeyProtection: !$appSettings.security.apiKeyProtection })} />
								<span class="toggle-slider"></span>
							</label>
						</div>
					</div>

				<!-- ═══ KNOWLEDGE MANAGEMENT ═══ -->
				{:else if activeSection === 'knowledge'}
					<p class="section-intro">{$t('settings.knowledge.intro') || 'Configure how your knowledge is organized, viewed, and analyzed.'}</p>

					<!-- ── Lenses ── -->
					<div class="setting-heading">{$t('lensPanel.switchLens') || 'Lenses'}</div>
					<p class="setting-desc" style="padding:0 0 8px; font-size:0.8rem;">{$t('settings.knowledge.lensDesc') || 'View your library through different classification schemes — by topic, stage, or any custom property.'}</p>

					{#if !lensesLoaded}
						<button class="w-btn" style="font-size:0.8rem; padding:6px 16px;" onclick={loadLenses}>{$t('settings.knowledge.loadLenses') || 'Load Lenses'}</button>
					{:else}
						{#each lensesList as lens}
							<div class="custom-fontset-row">
								<span class="custom-fontset-name">{lens.built_in ? '🔒' : '🔍'} {lens.name}</span>
								<span class="custom-fontset-type" style="font-size:0.7rem; color:var(--text-faint); margin-inline-start:auto; margin-inline-end:8px;">
									{lens.lens_type === 'property-query' ? (lens.property ?? '') : 'tags'}
								</span>
								<div class="custom-fontset-actions">
									{#if !lens.built_in}
										<button class="custom-fontset-btn" onclick={() => {
											editLensId = lens.id; editLensName = lens.name;
											editLensType = lens.lens_type;
											editLensProperty = lens.property ?? '';
											editLensValues = (lens.values ?? lens.root_tags ?? []).join(', ');
											showLensEditor = true;
										}}>
											<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 3a2.85 2.85 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/></svg>
										</button>
										<button class="custom-fontset-btn custom-fontset-delete" onclick={() => deleteLens(lens.id)}>
											<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
										</button>
									{/if}
								</div>
							</div>
						{/each}
						<button class="w-btn" style="font-size:0.8rem; padding:6px 16px; margin-top:8px;" onclick={() => { editLensId = ''; editLensName = ''; editLensType = 'property-query'; editLensProperty = ''; editLensValues = ''; showLensEditor = true; }}>+ {$t('commands.createLens') || 'Create Lens'}</button>
						{#if showLensEditor}
							<div class="custom-fontset-editor" style="margin-top:12px; padding:12px; background:var(--background-secondary); border-radius:8px;">
								<div style="font-weight:600; font-size:0.85rem; margin-bottom:8px;">{editLensId ? 'Edit Lens' : 'New Lens'}</div>
								<input class="setting-control" type="text" placeholder={$t('settings.knowledge.lensName') || 'Lens name'} bind:value={editLensName} style="margin-bottom:8px;" />
								<select class="setting-control" bind:value={editLensType} style="margin-bottom:8px;">
									<option value="property-query">{$t('settings.knowledge.groupByProperty') || 'Group by property'}</option>
									<option value="tag-hierarchy">{$t('settings.knowledge.groupByTags') || 'Group by tags'}</option>
								</select>
								{#if editLensType === 'property-query'}
									<input class="setting-control" type="text" placeholder={$t('settings.knowledge.propertyKey') || 'Property key (e.g., stage, certainty, priority)'} bind:value={editLensProperty} style="margin-bottom:8px;" />
									<input class="setting-control" type="text" placeholder={$t('settings.knowledge.propertyValues') || 'Values (comma-separated, e.g., high, medium, low)'} bind:value={editLensValues} style="margin-bottom:8px;" />
								{:else}
									<input class="setting-control" type="text" placeholder={$t('settings.knowledge.rootTags') || 'Root tags (comma-separated, leave empty for all tags)'} bind:value={editLensValues} style="margin-bottom:8px;" />
								{/if}
								<div style="display:flex; gap:8px;">
									<button class="w-btn" style="font-size:0.8rem; padding:6px 16px;" onclick={saveLensItem}>{editLensId ? ($t('settings.knowledge.save') || 'Save') : ($t('settings.knowledge.create') || 'Create')}</button>
									<button class="w-btn" style="font-size:0.8rem; padding:6px 16px; background:none; border:1px solid var(--background-modifier-border); color:var(--text-muted);" onclick={() => showLensEditor = false}>{$t('settings.knowledge.cancel') || 'Cancel'}</button>
								</div>
							</div>
						{/if}
					{/if}

				<!-- ═══ APPEARANCE ═══ -->
				{:else if activeSection === 'appearance'}
					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.appearance.colorScheme')}</div>
							<div class="setting-desc">{$t('settings.appearance.colorSchemeDesc')}</div>
						</div>
						<select class="setting-control" value={$appSettings.colorScheme} onchange={(e) => updateSettings({ colorScheme: (e.target as HTMLSelectElement).value as any })}>
							<option value="light">{$t('settings.appearance.light')}</option>
							<option value="dark">{$t('settings.appearance.dark')}</option>
							<option value="system">{$t('settings.appearance.system')}</option>
						</select>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.appearance.titleAlignment')}</div>
							<div class="setting-desc">{$t('settings.appearance.titleAlignmentDesc')}</div>
						</div>
						<select class="setting-control" value={$appSettings.titleAlignment} onchange={(e) => updateSettings({ titleAlignment: (e.target as HTMLSelectElement).value as any })}>
							<option value="start">{$t('settings.appearance.titleAlignStart')}</option>
							<option value="center">{$t('settings.appearance.titleAlignCenter')}</option>
						</select>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.appearance.accentColor')}</div>
							<div class="setting-desc">{$t('settings.appearance.accentColorDesc')}</div>
						</div>
						<div class="color-row">
							<input type="color" class="color-input" value={$appSettings.accentColor}
								onchange={(e) => updateSettings({ accentColor: (e.target as HTMLInputElement).value })} />
							<span class="color-hex">{$appSettings.accentColor}</span>
						</div>
					</div>

					<!-- Interface Font Size -->
					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.appearance.interfaceFontSize') || 'Interface font size'}</div>
							<div class="setting-desc">{$t('settings.appearance.interfaceFontSizeDesc') || 'Size of sidebar, toolbar, menus, and UI elements'}</div>
						</div>
						<div class="slider-row">
							<input type="range" class="setting-slider" min="11" max="18" step="1" value={$appSettings.interfaceFontSize || 14}
								oninput={(e) => updateSettings({ interfaceFontSize: parseInt((e.target as HTMLInputElement).value) })} />
							<span class="slider-val">{$appSettings.interfaceFontSize || 14}px</span>
						</div>
					</div>

					<!-- Note Content Font Size -->
					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.appearance.fontSize')}</div>
							<div class="setting-desc">{$t('settings.appearance.fontSizeDesc')}</div>
						</div>
						<div class="slider-row">
							<input type="range" class="setting-slider" min="12" max="24" step="1" value={$appSettings.fontSize}
								oninput={(e) => updateSettings({ fontSize: parseInt((e.target as HTMLInputElement).value) })} />
							<span class="slider-val">{$appSettings.fontSize}px</span>
						</div>
					</div>

				<!-- ═══ KEYBOARD ═══ -->
				{:else if activeSection === 'hotkeys'}
					<div class="hotkey-filter">
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
						<input type="text" placeholder={$t('settings.keyboard.filter')}
							value={hotkeyFilter} oninput={(e) => hotkeyFilter = (e.target as HTMLInputElement).value} />
					</div>

					<div class="hotkey-list">
						{#each filteredCommands as cmd}
							<div class="hotkey-item">
								<div class="hotkey-info">
									{#if cmd.icon}<span class="hotkey-icon">{cmd.icon}</span>{/if}
									<span class="hotkey-name">{cmd.name}</span>
									{#if cmd.category}<span class="hotkey-cat">{cmd.category}</span>{/if}
								</div>
								<div class="hotkey-binding">
									{#if editingHotkey === cmd.id}
										<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
										<kbd class="hotkey-recording" tabindex="0"
											onkeydown={(e) => handleHotkeyCapture(cmd.id, e)}
											onblur={() => { editingHotkey = null; hotkeyListening = false; }}>
											{$t('settings.keyboard.pressKeys')}
										</kbd>
									{:else}
										<button class="hotkey-edit-btn" onclick={() => { editingHotkey = cmd.id; hotkeyListening = true; }}>
											{#if cmd.shortcut}
												<kbd>{cmd.shortcut}</kbd>
											{:else}
												<span class="hotkey-unset">{$t('settings.keyboard.notSet')}</span>
											{/if}
										</button>
									{/if}
								</div>
							</div>
						{/each}
						{#if filteredCommands.length === 0}
							<div class="hotkey-empty">{$t('settings.keyboard.noCommands')}</div>
						{/if}
					</div>

				<!-- ═══ TEMPLATES ═══ -->
				{:else if activeSection === 'templates'}
					<p class="section-intro">{$t('settings.templates.intro') || 'Manage note templates. Templates let you insert predefined content into new notes.'}</p>
					<div class="setting-row">
						<div class="setting-info">
							<div class="setting-name">{$t('settings.plugins.templates') || 'Templates'}</div>
							<div class="setting-desc">{$t('settings.plugins.templatesDesc') || 'Insert content from template files'}</div>
						</div>
						<button class="toggle-btn" class:on={getFeatureEnabled('templates')} onclick={() => toggleFeature('templates')}>
							{getFeatureEnabled('templates') ? $t('settings.plugins.on') || 'On' : $t('settings.plugins.off') || 'Off'}
						</button>
					</div>

				<!-- ═══ PLUG-INS ═══ -->
				{:else if activeSection === 'plugins'}
					<p class="section-intro">{$t('settings.plugins.intro') || 'Toggle plug-ins on or off. Disabled plug-ins are hidden from the interface.'}</p>

					{#each featureGroups as group}
						<div class="plugin-group">
							<div class="plugin-group-header">{group.category}</div>
							{#each group.features as feature}
								<div class="plugin-row">
									<span class="plugin-icon">{feature.icon}</span>
									<div class="plugin-info">
										<div class="plugin-name">{feature.name}</div>
										<div class="plugin-desc">{feature.desc}</div>
									</div>
									<button class="plugin-switch" class:on={getFeatureEnabled(feature.id)}
										onclick={() => toggleFeature(feature.id)}
										title={getFeatureEnabled(feature.id) ? 'On' : 'Off'}>
										<span class="plugin-switch-knob"></span>
									</button>
								</div>
							{/each}
						</div>
					{/each}
				{/if}
			</div>
		</div>
	</div>
</div>

<style>
	/* ═══ OVERLAY ═══ */
	.settings-overlay {
		position: fixed; inset: 0; z-index: 1000;
		background: var(--background-modifier-cover);
		display: flex; align-items: center; justify-content: center;
		outline: none;
	}
	.settings-modal {
		width: 90vw; max-width: 920px; height: 82vh;
		display: flex;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 12px;
		box-shadow: var(--shadow-l);
		overflow: hidden;
	}

	/* ═══ SIDEBAR ═══ */
	.settings-sidebar {
		width: 210px; flex-shrink: 0;
		background: var(--background-secondary);
		border-inline-end: 1px solid var(--background-modifier-border);
		display: flex; flex-direction: column;
		padding: 12px 8px;
		overflow-y: auto;
	}
	.settings-sidebar-header {
		font-size: 0.75rem; font-weight: 600;
		text-transform: uppercase; letter-spacing: 0.05em;
		color: var(--text-faint);
		padding: 8px 10px 12px;
	}
	.settings-nav-item {
		display: flex; align-items: center; gap: 8px;
		padding: 7px 10px; border-radius: 6px;
		border: none; background: none; cursor: pointer;
		font-family: inherit; font-size: 0.85rem;
		color: var(--text-muted); text-align: start;
		transition: all 0.15s ease;
		width: 100%;
	}
	.settings-nav-item:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.settings-nav-item.active {
		background: var(--interactive-accent);
		color: var(--text-on-accent);
	}
	.nav-svg { flex-shrink: 0; opacity: 0.7; }
	.settings-nav-item.active .nav-svg { opacity: 1; }

	/* ═══ CONTENT ═══ */
	.settings-content {
		flex: 1; min-width: 0;
		display: flex; flex-direction: column;
	}
	.settings-content-header {
		display: flex; align-items: center; justify-content: space-between;
		padding: 16px 24px;
		border-bottom: 1px solid var(--background-modifier-border);
		flex-shrink: 0;
	}
	.settings-content-header h2 {
		margin: 0; font-size: 1.15rem; font-weight: 600; color: var(--text-normal);
	}
	.settings-close {
		width: 28px; height: 28px;
		display: flex; align-items: center; justify-content: center;
		border: none; background: none; cursor: pointer;
		border-radius: 6px; color: var(--text-muted);
		transition: all 0.15s ease;
	}
	.settings-close:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.settings-content-body {
		flex: 1; overflow-y: auto; padding: 20px 24px;
	}

	/* ═══ SETTING ITEM ═══ */
	.setting-item {
		display: flex; align-items: center; justify-content: space-between;
		gap: 16px; padding: 14px 0;
		border-bottom: 1px solid var(--background-modifier-border);
	}
	.setting-item:last-child { border-bottom: none; }
	.setting-info { flex: 1; min-width: 0; }
	.setting-name { font-size: 0.88rem; font-weight: 500; color: var(--text-normal); }
	.setting-desc { font-size: 0.78rem; color: var(--text-muted); margin-top: 2px; }
	.setting-section-heading {
		font-size: 0.85rem; font-weight: 600; color: var(--text-accent);
		padding: 16px 0 4px; margin-top: 8px;
		border-bottom: none;
	}
	.setting-heading {
		font-size: 0.8rem; font-weight: 600; color: var(--text-faint);
		text-transform: uppercase; letter-spacing: 0.04em;
		padding: 16px 0 4px; border-bottom: 1px solid var(--background-modifier-border);
		margin-top: 4px;
	}
	.section-intro {
		font-size: 0.85rem; color: var(--text-muted); margin: 0 0 12px;
	}

	/* ═══ CONTROLS ═══ */
	.setting-control {
		min-width: 180px; padding: 6px 10px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px; color: var(--text-normal);
		font-size: 0.85rem; font-family: inherit;
		cursor: pointer;
	}
	.setting-control:focus { border-color: var(--interactive-accent); outline: none; }
	.setting-range {
		display: flex; align-items: center; gap: 8px; min-width: 180px;
	}
	.setting-range input[type="range"] {
		flex: 1; accent-color: var(--interactive-accent);
	}
	.range-value {
		font-size: 12px; color: var(--text-muted); min-width: 32px; text-align: end;
	}
	.setting-input {
		min-width: 180px; max-width: 240px; padding: 6px 10px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px; color: var(--text-normal);
		font-size: 0.85rem; font-family: inherit;
	}
	.setting-input:focus { border-color: var(--interactive-accent); outline: none; }

	.setting-input-browse {
		display: flex; align-items: center; gap: 6px;
	}
	.setting-input-browse .setting-input { flex: 1; min-width: 140px; }
	.browse-btn {
		display: flex; align-items: center; justify-content: center;
		width: 32px; height: 32px; padding: 0; border: 1px solid var(--background-modifier-border);
		border-radius: 6px; background: var(--background-primary); color: var(--text-muted);
		cursor: pointer; flex-shrink: 0;
	}
	.browse-btn:hover { background: var(--background-modifier-hover); color: var(--text-normal); }

	.setting-info-box {
		font-size: 0.78rem; color: var(--text-faint);
		background: var(--background-secondary); border-radius: 6px;
		padding: 10px 14px; line-height: 1.8; width: 100%;
	}
	.setting-info-box code {
		background: var(--background-modifier-border); padding: 2px 6px;
		border-radius: 3px; font-size: 0.75rem; color: var(--text-normal);
	}

	/* Toggle Switch */
	.toggle { position: relative; display: inline-block; width: 40px; height: 22px; flex-shrink: 0; }
	.toggle input { opacity: 0; width: 0; height: 0; position: absolute; }
	.toggle-slider {
		position: absolute; inset: 0; cursor: pointer;
		background: var(--background-modifier-border-focus);
		border-radius: 22px; transition: background 0.2s ease;
	}
	.toggle-slider::after {
		content: ''; position: absolute; top: 3px; left: 3px;
		width: 16px; height: 16px; border-radius: 50%;
		background: var(--background-primary);
		transition: transform 0.2s ease;
		box-shadow: 0 1px 2px rgba(0,0,0,0.15);
	}
	.toggle input:checked + .toggle-slider { background: var(--interactive-accent); }
	.toggle input:checked + .toggle-slider::after { transform: translateX(18px); }

	/* Script toolbar checkboxes */
	.script-toolbar-checkboxes {
		display: flex; flex-wrap: wrap; gap: 8px;
	}
	.script-check {
		display: flex; align-items: center; gap: 4px;
		font-size: 13px; cursor: pointer;
	}
	.script-check input { cursor: pointer; }

	/* Color Picker */
	.color-row { display: flex; align-items: center; gap: 8px; }
	.color-input {
		width: 36px; height: 36px; border: 1px solid var(--background-modifier-border);
		border-radius: 6px; padding: 2px; cursor: pointer; background: none;
	}
	.color-hex { font-size: 0.82rem; color: var(--text-muted); font-family: var(--font-monospace-theme); }

	/* Slider */
	.slider-row { display: flex; align-items: center; gap: 10px; min-width: 180px; }
	.setting-slider { flex: 1; accent-color: var(--interactive-accent); }
	.slider-val { font-size: 0.82rem; color: var(--text-muted); min-width: 38px; text-align: end; }

	/* ═══ FONT SETS ═══ */
	.font-mode-toggle { display: flex; gap: 2px; background: var(--background-secondary); border-radius: 6px; padding: 2px; }
	.font-mode-btn {
		padding: 5px 14px; border: none; border-radius: 5px; cursor: pointer;
		background: transparent; color: var(--text-muted); font-size: 0.82rem;
		font-family: var(--font-interface-theme); transition: all 0.15s;
	}
	.font-mode-btn.active { background: var(--interactive-accent); color: white; }
	.font-preview-box {
		background: var(--background-secondary); border-radius: 8px; padding: 12px 16px;
		display: flex; flex-direction: column; gap: 6px; margin: 4px 0;
	}
	.font-preview-row { display: flex; align-items: baseline; gap: 10px; font-size: 0.85rem; }
	.font-preview-label { color: var(--text-muted); font-size: 0.75rem; min-width: 60px; text-transform: uppercase; }
	.font-script-table { display: flex; flex-direction: column; gap: 8px; margin: 4px 0; }
	.font-script-row {
		display: grid; grid-template-columns: 160px 1fr; gap: 10px; align-items: center;
		padding: 8px 12px; background: var(--background-secondary); border-radius: 8px;
	}
	.font-script-label { font-size: 0.82rem; font-weight: 500; color: var(--text-normal); }
	.font-script-select {
		padding: 4px 8px; border: 1px solid var(--background-modifier-border); border-radius: 6px;
		background: var(--background-primary); color: var(--text-normal); font-size: 0.82rem;
		font-family: var(--font-interface-theme);
	}
	.font-script-preview {
		grid-column: 1 / -1; font-size: 0.85rem; color: var(--text-muted);
		padding-top: 4px; border-top: 1px solid var(--background-modifier-border);
		white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
	}

	/* Contextual font language selector */
	.font-lang-section {
		padding: 12px; background: var(--background-secondary); border-radius: 10px;
		margin-bottom: 8px; display: flex; flex-direction: column; gap: 8px;
	}
	.lang-card {
		padding: 12px 16px; background: var(--background-secondary); border-radius: 10px;
		margin: 4px 16px; border-left: 3px solid var(--interactive-accent);
	}
	.lang-card-title {
		font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px;
		color: var(--text-muted); margin-bottom: 8px;
	}
	.lang-card-row {
		display: flex; gap: 8px; align-items: center; flex-wrap: wrap;
	}
	.lang-card-select {
		flex: 1; min-width: 120px; padding: 6px 10px; border: 1px solid var(--background-modifier-border);
		border-radius: 6px; background: var(--background-primary); color: var(--text-normal);
		font-size: 13px; font-family: var(--font-interface-theme);
	}
	.lang-card-preview {
		font-size: 13px; color: var(--text-muted); margin-top: 6px; padding: 4px 0;
	}
	.lang-card-label {
		font-size: 13px; font-weight: 500; min-width: 100px; color: var(--text-normal);
	}
	.date-format-grid { display: flex; flex-direction: column; gap: 8px; padding: 0 16px 8px; }
	.date-format-row { display: flex; align-items: center; gap: 12px; }
	.date-format-label { font-size: 13px; font-weight: 500; min-width: 120px; color: var(--text-normal); }
	.date-format-select {
		flex: 1; padding: 6px 10px; border: 1px solid var(--background-modifier-border);
		border-radius: 6px; background: var(--background-primary); color: var(--text-normal);
		font-size: 13px;
	}
	.date-contextual-check {
		display: flex; align-items: center; gap: 4px; font-size: 12px;
		color: var(--text-muted); cursor: pointer; white-space: nowrap;
	}
	.date-contextual-check input { margin: 0; }
	.font-lang-header { display: flex; align-items: center; gap: 8px; }
	.font-lang-label { font-size: 0.82rem; font-weight: 600; color: var(--text-normal); text-transform: uppercase; letter-spacing: 0.5px; }
	.font-lang-select, .font-lang-font-select {
		width: 100%; padding: 6px 10px; border: 1px solid var(--background-modifier-border);
		border-radius: 8px; background: var(--background-primary); color: var(--text-normal);
		font-size: 0.85rem; font-family: var(--font-interface-theme);
	}
	.font-lang-preview {
		font-size: 0.9rem; color: var(--text-muted); padding: 6px 0;
		border-top: 1px solid var(--background-modifier-border);
		white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
	}
	.font-lang-toggle { padding: 4px 0; }
	.font-lang-check {
		display: flex; align-items: center; gap: 8px; cursor: pointer;
		font-size: 0.85rem; color: var(--text-normal);
	}
	/* ── Font Theme Cards ── */
	.font-theme-cards {
		display: flex; gap: 10px; padding: 0 16px; margin-bottom: 4px;
	}
	.font-theme-card {
		flex: 1; padding: 12px 10px; border-radius: 10px; border: 2px solid var(--background-modifier-border);
		background: var(--background-secondary); cursor: pointer; text-align: center;
		display: flex; flex-direction: column; gap: 4px; align-items: center;
		transition: border-color 0.15s, background 0.15s;
	}
	.font-theme-card:hover { border-color: var(--interactive-accent); }
	.font-theme-card.active {
		border-color: var(--interactive-accent);
		background: color-mix(in srgb, var(--interactive-accent) 8%, var(--background-secondary));
	}
	.font-theme-preview {
		font-size: 28px; line-height: 1; color: var(--text-normal); font-weight: 400;
	}
	.font-theme-label {
		font-size: 12px; font-weight: 600; color: var(--text-normal);
	}
	.font-theme-scripts {
		font-size: 10px; color: var(--text-muted); letter-spacing: 0.5px;
	}
	.font-theme-info {
		margin: 0 16px 8px; padding: 8px 12px; background: var(--background-secondary);
		border-radius: 8px; font-size: 11px; color: var(--text-muted); font-family: 'Courier Prime', monospace;
	}

	.custom-fontset-row {
		display: flex; justify-content: space-between; align-items: center;
		padding: 8px 12px; background: var(--background-secondary); border-radius: 8px;
	}
	.custom-fontset-name { font-size: 0.85rem; font-weight: 500; }
	.custom-fontset-actions { display: flex; gap: 4px; }
	.custom-fontset-btn {
		padding: 4px 6px; border: none; border-radius: 4px; cursor: pointer;
		background: transparent; color: var(--text-muted);
	}
	.custom-fontset-btn:hover { background: var(--background-modifier-hover); }
	.custom-fontset-delete:hover { color: var(--text-error, #e53e3e); }
	.create-fontset-btn {
		padding: 6px 12px; border: 1px dashed var(--background-modifier-border); border-radius: 8px;
		background: transparent; color: var(--text-muted); cursor: pointer; width: 100%;
		font-family: var(--font-interface-theme); font-size: 0.82rem;
	}
	.create-fontset-btn:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.fontset-editor {
		background: var(--background-secondary); border-radius: 8px; padding: 16px;
		display: flex; flex-direction: column; gap: 10px; margin: 4px 0;
		border: 1px solid var(--interactive-accent);
	}
	.fontset-editor-field { display: flex; flex-direction: column; gap: 4px; }
	.fontset-editor-field label { font-size: 0.75rem; color: var(--text-muted); text-transform: uppercase; }
	.fontset-editor-field input, .fontset-select {
		padding: 6px 10px; border: 1px solid var(--background-modifier-border); border-radius: 6px;
		background: var(--background-primary); color: var(--text-normal); font-size: 0.85rem;
		font-family: var(--font-interface-theme); width: 100%;
	}
	.fontset-select { cursor: pointer; }
	.fontset-editor-preview {
		padding: 8px 12px; background: var(--background-primary); border-radius: 6px;
		font-size: 0.9rem; color: var(--text-normal);
	}
	.fontset-editor-actions { display: flex; gap: 8px; justify-content: flex-end; }
	.fontset-save-btn {
		padding: 6px 16px; border: none; border-radius: 6px; cursor: pointer;
		background: var(--interactive-accent); color: white; font-family: var(--font-interface-theme);
	}
	.fontset-save-btn:disabled { opacity: 0.5; cursor: not-allowed; }
	.fontset-cancel-btn {
		padding: 6px 16px; border: 1px solid var(--background-modifier-border); border-radius: 6px;
		cursor: pointer; background: transparent; color: var(--text-muted);
		font-family: var(--font-interface-theme);
	}

	/* ═══ DASHBOARD ═══ */
	.dashboard { }
	.dashboard-header {
		display: flex; align-items: center; gap: 16px;
		padding-bottom: 20px; border-bottom: 1px solid var(--background-modifier-border);
	}
	.dash-title { display: flex; flex-direction: column; gap: 2px; }
	.dash-name { font-size: 1.3rem; font-weight: 700; color: var(--text-normal); }
	.dash-version {
		font-size: 0.75rem; color: var(--text-faint);
		background: var(--background-secondary-alt);
		padding: 1px 8px; border-radius: 10px; width: fit-content;
	}
	.dash-stats {
		display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px;
		padding: 20px 0;
	}
	.stat-card {
		text-align: center; padding: 16px 8px;
		background: var(--background-secondary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 10px;
	}
	.stat-value { font-size: 1.2rem; font-weight: 700; color: var(--text-normal); }
	.stat-label { font-size: 0.72rem; color: var(--text-faint); margin-top: 4px; text-transform: uppercase; letter-spacing: 0.03em; }
	.dash-section { border-top: 1px solid var(--background-modifier-border); }
	.dash-footer {
		display: flex; align-items: center; justify-content: space-between;
		padding-top: 16px; margin-top: 8px;
		border-top: 1px solid var(--background-modifier-border);
	}
	.dash-dev { font-size: 0.82rem; color: var(--text-muted); }
	.dash-link {
		display: flex; align-items: center; gap: 4px;
		font-size: 0.82rem; color: var(--interactive-accent); cursor: pointer;
	}

	/* ═══ KEYBOARD ═══ */
	.hotkey-filter {
		display: flex; align-items: center; gap: 8px;
		padding: 8px 12px; margin-bottom: 12px;
		background: var(--background-secondary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
	}
	.hotkey-filter svg { color: var(--text-faint); flex-shrink: 0; }
	.hotkey-filter input {
		flex: 1; border: none; background: none; outline: none;
		color: var(--text-normal); font-size: 0.88rem; font-family: inherit;
	}
	.hotkey-filter input::placeholder { color: var(--text-faint); }
	.hotkey-list { display: flex; flex-direction: column; }
	.hotkey-item {
		display: flex; align-items: center; justify-content: space-between;
		padding: 10px 0; border-bottom: 1px solid var(--background-modifier-border);
		gap: 12px;
	}
	.hotkey-item:last-child { border-bottom: none; }
	.hotkey-info { display: flex; align-items: center; gap: 8px; min-width: 0; }
	.hotkey-icon { font-size: 0.9rem; flex-shrink: 0; }
	.hotkey-name { font-size: 0.88rem; color: var(--text-normal); }
	.hotkey-cat {
		font-size: 0.7rem; color: var(--text-faint);
		background: var(--background-secondary-alt); padding: 1px 6px; border-radius: 4px;
	}
	.hotkey-binding { flex-shrink: 0; }
	.hotkey-edit-btn {
		background: none; border: none; cursor: pointer; padding: 0;
	}
	.hotkey-edit-btn:hover kbd { border-color: var(--interactive-accent); }
	kbd {
		font-family: var(--font-monospace-theme);
		font-size: 0.78rem; color: var(--text-muted);
		background: var(--background-secondary);
		border: 1px solid var(--background-modifier-border);
		padding: 2px 8px; border-radius: 4px;
	}
	.hotkey-recording {
		background: var(--interactive-accent); color: var(--text-on-accent);
		border-color: var(--interactive-accent); animation: pulse 1s infinite;
		outline: none;
	}
	@keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.7; } }
	.hotkey-unset { font-size: 0.78rem; color: var(--text-faint); font-style: italic; }
	.hotkey-empty { text-align: center; padding: 24px; color: var(--text-faint); font-size: 0.85rem; }

	/* ═══ FEATURES GRID ═══ */
	.feature-group { margin-bottom: 24px; }
	.feature-group-header {
		font-size: 0.8rem; font-weight: 600; color: var(--text-faint);
		text-transform: uppercase; letter-spacing: 0.04em;
		margin-bottom: 10px;
	}
	.feature-grid {
		display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px;
	}
	.feature-card {
		display: flex; flex-direction: column; gap: 4px;
		padding: 14px; border-radius: 10px;
		border: 1px solid var(--background-modifier-border);
		background: var(--background-secondary);
		cursor: pointer; text-align: start;
		font-family: inherit;
		transition: all 0.15s ease;
	}
	.feature-card:hover { border-color: var(--interactive-accent); }
	.feature-card.enabled { border-color: color-mix(in srgb, var(--interactive-accent) 40%, transparent); }
	.feature-card-icon { font-size: 1.2rem; }
	.feature-card-name { font-size: 0.82rem; font-weight: 600; color: var(--text-normal); }
	.feature-card-desc { font-size: 0.72rem; color: var(--text-muted); line-height: 1.4; }
	.feature-card-toggle {
		display: flex; align-items: center; gap: 5px;
		font-size: 0.7rem; color: var(--text-faint); margin-top: 4px;
		text-transform: uppercase; letter-spacing: 0.03em;
	}
	.feature-dot {
		width: 8px; height: 8px; border-radius: 50%;
		background: var(--background-modifier-border);
		transition: background 0.2s ease;
	}
	.feature-dot.on { background: var(--color-green, #4ade80); }

	/* Plug-in toggle switches */
	.plugin-group { margin-bottom: 20px; }
	.plugin-group-header {
		font-size: 0.8rem; font-weight: 600; color: var(--text-faint);
		text-transform: uppercase; letter-spacing: 0.04em;
		margin-bottom: 8px;
	}
	.plugin-row {
		display: flex; align-items: center; gap: 12px;
		padding: 10px 12px; border-radius: 8px;
		border-bottom: 1px solid var(--background-modifier-border);
	}
	.plugin-row:hover { background: var(--background-modifier-hover); }
	.plugin-icon { font-size: 1.1rem; flex-shrink: 0; }
	.plugin-info { flex: 1; min-width: 0; }
	.plugin-name { font-size: 0.85rem; font-weight: 600; color: var(--text-normal); }
	.plugin-desc { font-size: 0.72rem; color: var(--text-muted); line-height: 1.4; margin-top: 1px; }
	.plugin-switch {
		width: 40px; height: 22px; border-radius: 11px; border: none;
		background: var(--background-modifier-border); cursor: pointer;
		position: relative; flex-shrink: 0; transition: background 0.2s;
		padding: 0;
	}
	.plugin-switch.on { background: var(--interactive-accent, #7c3aed); }
	.plugin-switch-knob {
		position: absolute; top: 2px; inset-inline-start: 2px;
		width: 18px; height: 18px; border-radius: 50%;
		background: white; transition: inset-inline-start 0.2s;
		box-shadow: 0 1px 3px rgba(0,0,0,0.2);
	}
	.plugin-switch.on .plugin-switch-knob { inset-inline-start: 20px; }

	/* ═══ AI / INTELLIGENCE ═══ */
	.test-btn {
		padding: 6px 16px;
		background: var(--interactive-normal);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px; cursor: pointer;
		color: var(--text-normal); font-size: 0.85rem; font-family: inherit;
		transition: all 0.15s ease;
		white-space: nowrap;
	}
	.test-btn:hover { border-color: var(--interactive-accent); }
	.test-btn:disabled { opacity: 0.5; cursor: not-allowed; }
	.test-success { color: var(--color-green); font-weight: 500; }
	.test-failed { color: var(--text-error); font-weight: 500; }

	/* ═══ SETTING BUTTON ═══ */
	.setting-btn {
		background: var(--interactive-accent); color: var(--text-on-accent);
		border: none; padding: 5px 14px; border-radius: 4px; font-size: 0.82rem;
		cursor: pointer; white-space: nowrap;
	}
	.setting-btn:hover { opacity: 0.9; }
	.setting-btn:disabled { opacity: 0.5; cursor: not-allowed; }

	.update-progress {
		position: relative; width: 120px; height: 28px;
		background: var(--background-modifier-border); border-radius: 6px; overflow: hidden;
	}
	.update-progress-bar {
		height: 100%; background: var(--interactive-accent);
		border-radius: 6px; transition: width 0.3s;
	}
	.update-progress-text {
		position: absolute; inset: 0; display: flex; align-items: center; justify-content: center;
		font-size: 0.75rem; font-weight: 600; color: var(--text-normal);
	}

	/* ═══ SECURITY ═══ */
	.security-control-row {
		display: flex; align-items: center; gap: 10px;
	}
	.security-badge {
		font-size: 0.75rem; padding: 2px 8px;
		border-radius: 10px;
		background: var(--background-secondary-alt);
		color: var(--text-faint);
	}
	.security-badge.active {
		background: color-mix(in srgb, var(--color-green, #4ade80) 15%, transparent);
		color: var(--color-green, #4ade80);
	}
	.sub-setting {
		padding-inline-start: 16px;
		border-inline-start: 2px solid var(--background-modifier-border);
	}
	.pin-setup {
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px; padding: 16px; margin: 8px 0;
		background: var(--background-secondary);
	}
	.pin-setup-title {
		font-size: 0.88rem; font-weight: 600; color: var(--text-normal);
		margin-bottom: 10px;
	}
	.pin-fields {
		display: flex; gap: 8px; margin: 8px 0;
	}
	.pin-fields .setting-input {
		flex: 1; min-width: 0;
	}
	.pin-error {
		font-size: 0.78rem; color: var(--text-error); margin: 4px 0;
	}
	.pin-actions {
		display: flex; justify-content: flex-end; gap: 8px; margin-top: 10px;
	}
	.dialog-btn.cancel {
		background: var(--background-modifier-border);
		color: var(--text-muted);
		border: none; padding: 5px 14px; border-radius: 4px;
		font-size: 0.82rem; cursor: pointer;
	}
	.dialog-btn.cancel:hover { opacity: 0.85; }
</style>
