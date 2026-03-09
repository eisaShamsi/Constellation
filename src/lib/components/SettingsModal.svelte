<script lang="ts">
	import { onMount } from 'svelte';
	import { locale, setLocale, type Locale } from '$lib/i18n';
	import { appSettings, updateSettings } from '$lib/vaults/store';
	import { aiSettings, updateAISettings, setProvider } from '$lib/ai/store';
	import { validateConnection } from '$lib/ai/engine';
	import { PROVIDER_INFO, DEFAULT_MODELS, type ProviderId } from '$lib/ai/provider';

	let {
		onClose,
		ar = false,
		commands = [] as { id: string; name: string; shortcut?: string; icon?: string; category?: string }[],
	}: {
		onClose: () => void;
		ar?: boolean;
		commands?: { id: string; name: string; shortcut?: string; icon?: string; category?: string }[];
	} = $props();

	let activeSection = $state('about');
	let hotkeyFilter = $state('');
	let testStatus = $state('');
	let testing = $state(false);

	const sections = $derived([
		{ id: 'about', label: ar ? 'حول' : 'About', icon: 'info' },
		{ id: 'general', label: ar ? 'عام' : 'General', icon: 'globe' },
		{ id: 'editor', label: ar ? 'المحرر' : 'Editor', icon: 'edit' },
		{ id: 'files', label: ar ? 'الملفات والروابط' : 'Files & Links', icon: 'folder' },
		{ id: 'appearance', label: ar ? 'المظهر' : 'Appearance', icon: 'palette' },
		{ id: 'hotkeys', label: ar ? 'اختصارات لوحة المفاتيح' : 'Hotkeys', icon: 'keyboard' },
		{ id: 'plugins', label: ar ? 'الإضافات الأساسية' : 'Core Plugins', icon: 'puzzle' },
		{ id: 'ai', label: ar ? 'الذكاء الاصطناعي' : 'AI Provider', icon: 'bot' },
	]);

	const filteredCommands = $derived(
		hotkeyFilter.trim()
			? commands.filter(c => c.name.toLowerCase().includes(hotkeyFilter.toLowerCase()) || (c.shortcut?.toLowerCase().includes(hotkeyFilter.toLowerCase())))
			: commands
	);

	const corePlugins = $derived([
		{ id: 'dailyNotes', name: ar ? 'الملاحظات اليومية' : 'Daily Notes', desc: ar ? 'إنشاء وفتح ملاحظات يومية' : 'Create and open daily notes' },
		{ id: 'templates', name: ar ? 'القوالب' : 'Templates', desc: ar ? 'إدراج محتوى من ملفات القوالب' : 'Insert content from template files' },
		{ id: 'graphView', name: ar ? 'عرض الرسم البياني' : 'Graph View', desc: ar ? 'عرض الروابط بين الملاحظات' : 'Visualize links between notes' },
		{ id: 'backlinks', name: ar ? 'الروابط الواردة' : 'Backlinks', desc: ar ? 'عرض الملاحظات التي تشير لهذه الملاحظة' : 'Show notes that link to the current note' },
		{ id: 'outgoingLinks', name: ar ? 'الروابط الصادرة' : 'Outgoing Links', desc: ar ? 'عرض الروابط في الملاحظة الحالية' : 'Show links in the current note' },
		{ id: 'tags', name: ar ? 'الوسوم' : 'Tags', desc: ar ? 'عرض وتصفح جميع الوسوم' : 'View and browse all tags' },
		{ id: 'pagePreview', name: ar ? 'معاينة الصفحة' : 'Page Preview', desc: ar ? 'معاينة الملاحظات عند التمرير فوق الروابط' : 'Preview notes on link hover' },
		{ id: 'search', name: ar ? 'البحث' : 'Search', desc: ar ? 'البحث في جميع الملاحظات' : 'Search across all notes' },
		{ id: 'quickSwitcher', name: ar ? 'التبديل السريع' : 'Quick Switcher', desc: ar ? 'التنقل السريع بين الملاحظات' : 'Quickly navigate between notes' },
		{ id: 'commandPalette', name: ar ? 'لوحة الأوامر' : 'Command Palette', desc: ar ? 'الوصول السريع لجميع الأوامر' : 'Quick access to all commands' },
		{ id: 'wordCount', name: ar ? 'عدد الكلمات' : 'Word Count', desc: ar ? 'عرض عدد الكلمات في شريط الحالة' : 'Show word count in status bar' },
		{ id: 'workspaces', name: ar ? 'مساحات العمل' : 'Workspaces', desc: ar ? 'حفظ واستعادة تخطيطات مساحة العمل' : 'Save and restore workspace layouts' },
	]);

	function handleKeydown(e: KeyboardEvent) {
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

	function handleLangChange(e: Event) {
		setLocale((e.target as HTMLSelectElement).value as Locale);
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

	function getPluginEnabled(id: string): boolean {
		const plugins = ($appSettings as any).enabledPlugins;
		if (!plugins) return true;
		return plugins[id] !== false;
	}

	function togglePlugin(id: string) {
		const current = ($appSettings as any).enabledPlugins || {};
		updateSettings({
			enabledPlugins: { ...current, [id]: !getPluginEnabled(id) }
		} as any);
	}

	function sectionIcon(icon: string): string {
		const icons: Record<string, string> = {
			info: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z',
			globe: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z',
			edit: 'M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z',
			folder: 'M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z',
			palette: 'M12 3c-4.97 0-9 4.03-9 9s4.03 9 9 9c.83 0 1.5-.67 1.5-1.5 0-.39-.15-.74-.39-1.01-.23-.26-.38-.61-.38-1 0-.83.67-1.5 1.5-1.5H16c2.76 0 5-2.24 5-5 0-4.42-4.03-8-9-8z',
			keyboard: 'M20 5H4c-1.1 0-1.99.9-1.99 2L2 17c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm-9 3h2v2h-2V8zm0 3h2v2h-2v-2zM8 8h2v2H8V8zm0 3h2v2H8v-2zm-1 2H5v-2h2v2zm0-3H5V8h2v2zm9 7H8v-2h8v2zm0-4h-2v-2h2v2zm0-3h-2V8h2v2zm3 3h-2v-2h2v2zm0-3h-2V8h2v2z',
			puzzle: 'M20.5 11H19V7c0-1.1-.9-2-2-2h-4V3.5C13 2.12 11.88 1 10.5 1S8 2.12 8 3.5V5H4c-1.1 0-1.99.9-1.99 2v3.8H3.5c1.49 0 2.7 1.21 2.7 2.7s-1.21 2.7-2.7 2.7H2V20c0 1.1.9 2 2 2h3.8v-1.5c0-1.49 1.21-2.7 2.7-2.7 1.49 0 2.7 1.21 2.7 2.7V22H17c1.1 0 2-.9 2-2v-4h1.5c1.38 0 2.5-1.12 2.5-2.5S21.88 11 20.5 11z',
			bot: 'M12 2a2 2 0 0 1 2 2c0 .74-.4 1.39-1 1.73V7h1a7 7 0 0 1 7 7h1a1 1 0 0 1 1 1v3a1 1 0 0 1-1 1h-1v1a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-1H3a1 1 0 0 1-1-1v-3a1 1 0 0 1 1-1h1a7 7 0 0 1 7-7h1V5.73c-.6-.34-1-.99-1-1.73a2 2 0 0 1 2-2zM9 14a1 1 0 1 0 0 2 1 1 0 0 0 0-2zm6 0a1 1 0 1 0 0 2 1 1 0 0 0 0-2z',
		};
		return icons[icon] || icons.info;
	}

	let containerEl: HTMLDivElement;
	onMount(() => { containerEl?.focus(); });
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div class="settings-overlay" onclick={onClose} onkeydown={handleKeydown} tabindex="0" bind:this={containerEl} role="dialog" aria-modal="true" aria-label={ar ? 'الإعدادات' : 'Settings'}>
	<div class="settings-modal" onclick={(e) => e.stopPropagation()}>
		<!-- Sidebar -->
		<div class="settings-sidebar">
			<div class="settings-sidebar-header">{ar ? 'الإعدادات' : 'Settings'}</div>
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
				<!-- ═══ ABOUT ═══ -->
				{#if activeSection === 'about'}
					<div class="about-section">
						<div class="about-logo">✦</div>
						<div class="about-name">Constellation</div>
						<div class="about-tagline">{ar ? 'خريطة من الخرائط' : 'A Map of Maps'}</div>
						<div class="about-version">v0.1.0</div>
						<div class="about-desc">
							{ar
								? 'مستعرض وقارئ خزائن Obsidian مبني بـ Tauri و Svelte'
								: 'An Obsidian vault viewer and reader built with Tauri + Svelte'}
						</div>
						<div class="about-links">
							<span class="about-link">
								<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.4 5.4 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4"/><path d="M9 18c-4.51 2-5-2-7-2"/></svg>
								GitHub
							</span>
						</div>
					</div>

				<!-- ═══ GENERAL ═══ -->
				{:else if activeSection === 'general'}
					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'اللغة' : 'Language'}</div>
							<div class="setting-desc">{ar ? 'اختر لغة واجهة التطبيق' : 'Choose the interface language'}</div>
						</div>
						<select class="setting-control" value={$locale} onchange={handleLangChange}>
							<option value="en">English</option>
							<option value="ar">العربية</option>
						</select>
					</div>

				<!-- ═══ EDITOR ═══ -->
				{:else if activeSection === 'editor'}
					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'العرض الافتراضي للتبويبات الجديدة' : 'Default view for new tabs'}</div>
							<div class="setting-desc">{ar ? 'وضع القراءة أو التحرير عند فتح ملاحظة جديدة' : 'Reading or editing view when opening a new note'}</div>
						</div>
						<select class="setting-control" value={$appSettings.defaultView} onchange={(e) => updateSettings({ defaultView: (e.target as HTMLSelectElement).value as any })}>
							<option value="reading">{ar ? 'وضع القراءة' : 'Reading view'}</option>
							<option value="editing">{ar ? 'وضع التحرير' : 'Editing view'}</option>
						</select>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'إظهار أرقام الأسطر' : 'Show line numbers'}</div>
							<div class="setting-desc">{ar ? 'عرض أرقام الأسطر في هامش المحرر' : 'Display line numbers in the editor gutter'}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.showLineNumbers}
								onchange={(e) => updateSettings({ showLineNumbers: (e.target as HTMLInputElement).checked })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'عرض سطر مقروء' : 'Readable line length'}</div>
							<div class="setting-desc">{ar ? 'تحديد عرض المحتوى لتسهيل القراءة' : 'Limit content width for comfortable reading'}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.readableLineLength}
								onchange={(e) => updateSettings({ readableLineLength: (e.target as HTMLInputElement).checked })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'حجم المسافة البادئة' : 'Tab size'}</div>
							<div class="setting-desc">{ar ? 'عدد المسافات لكل مسافة بادئة' : 'Number of spaces per indentation level'}</div>
						</div>
						<select class="setting-control" value={$appSettings.tabSize} onchange={(e) => updateSettings({ tabSize: parseInt((e.target as HTMLSelectElement).value) })}>
							<option value="2">2</option>
							<option value="4">4</option>
						</select>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'قوائم ذكية' : 'Smart lists'}</div>
							<div class="setting-desc">{ar ? 'متابعة القوائم تلقائيا عند الضغط على Enter' : 'Auto-continue and indent lists on Enter'}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.smartLists}
								onchange={(e) => updateSettings({ smartLists: (e.target as HTMLInputElement).checked })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'إغلاق الأقواس تلقائيا' : 'Auto-pair brackets'}</div>
							<div class="setting-desc">{ar ? 'إضافة قوس الإغلاق تلقائيا عند كتابة قوس الفتح' : 'Automatically insert closing brackets and quotes'}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.autoPairBrackets}
								onchange={(e) => updateSettings({ autoPairBrackets: (e.target as HTMLInputElement).checked })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'التدقيق الإملائي' : 'Spellcheck'}</div>
							<div class="setting-desc">{ar ? 'تمكين التدقيق الإملائي في المحرر' : 'Enable spellcheck in the editor'}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.spellcheck}
								onchange={(e) => updateSettings({ spellcheck: (e.target as HTMLInputElement).checked })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

				<!-- ═══ FILES & LINKS ═══ -->
				{:else if activeSection === 'files'}
					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'موقع الملاحظات الجديدة' : 'Default location for new notes'}</div>
							<div class="setting-desc">{ar ? 'أين يتم إنشاء الملاحظات الجديدة' : 'Where new notes are placed'}</div>
						</div>
						<select class="setting-control" value={$appSettings.defaultNoteLocation} onchange={(e) => updateSettings({ defaultNoteLocation: (e.target as HTMLSelectElement).value as any })}>
							<option value="root">{ar ? 'جذر الخزينة' : 'Vault root folder'}</option>
							<option value="current">{ar ? 'المجلد الحالي' : 'Same folder as current file'}</option>
							<option value="folder">{ar ? 'مجلد محدد' : 'In the folder specified below'}</option>
						</select>
					</div>

					{#if $appSettings.defaultNoteLocation === 'folder'}
						<div class="setting-item sub-setting">
							<div class="setting-info">
								<div class="setting-name">{ar ? 'مسار المجلد' : 'Folder path'}</div>
							</div>
							<input class="setting-input" type="text" value={$appSettings.defaultNoteFolder}
								placeholder={ar ? 'مثال: Notes' : 'e.g. Notes'}
								oninput={(e) => updateSettings({ defaultNoteFolder: (e.target as HTMLInputElement).value })} />
						</div>
					{/if}

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'مجلد المرفقات الافتراضي' : 'Default attachment folder'}</div>
							<div class="setting-desc">{ar ? 'المجلد الذي يتم حفظ المرفقات فيه' : 'Where attachments are stored'}</div>
						</div>
						<input class="setting-input" type="text" value={$appSettings.defaultAttachmentFolder}
							placeholder={ar ? 'نفس مجلد الملاحظة' : 'Same folder as note'}
							oninput={(e) => updateSettings({ defaultAttachmentFolder: (e.target as HTMLInputElement).value })} />
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'تنسيق الرابط الجديد' : 'New link format'}</div>
							<div class="setting-desc">{ar ? 'كيف يتم إنشاء الروابط الداخلية' : 'How internal links are generated'}</div>
						</div>
						<select class="setting-control" value={$appSettings.linkFormat} onchange={(e) => updateSettings({ linkFormat: (e.target as HTMLSelectElement).value as any })}>
							<option value="shortest">{ar ? 'أقصر مسار' : 'Shortest path when possible'}</option>
							<option value="relative">{ar ? 'مسار نسبي' : 'Relative path to file'}</option>
							<option value="absolute">{ar ? 'مسار مطلق' : 'Absolute path in vault'}</option>
						</select>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'تحديث الروابط تلقائيا' : 'Automatically update internal links'}</div>
							<div class="setting-desc">{ar ? 'تحديث جميع الروابط عند إعادة تسمية ملاحظة' : 'Update all links when a note is renamed'}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.autoUpdateLinks}
								onchange={(e) => updateSettings({ autoUpdateLinks: (e.target as HTMLInputElement).checked })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'استخدام ويكي لينك' : 'Use [[Wikilinks]]'}</div>
							<div class="setting-desc">{ar ? 'استخدام [[ويكي لينك]] بدلا من [روابط ماركداون](...)' : 'Use [[Wikilinks]] instead of [Markdown links](...)'}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.useWikilinks}
								onchange={(e) => updateSettings({ useWikilinks: (e.target as HTMLInputElement).checked })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'تأكيد الحذف' : 'Confirm file deletion'}</div>
							<div class="setting-desc">{ar ? 'عرض مربع تأكيد قبل حذف الملفات' : 'Show confirmation dialog before deleting files'}</div>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={$appSettings.confirmDelete}
								onchange={(e) => updateSettings({ confirmDelete: (e.target as HTMLInputElement).checked })} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'وجهة الملفات المحذوفة' : 'Deleted files'}</div>
							<div class="setting-desc">{ar ? 'أين يتم نقل الملفات المحذوفة' : 'Where deleted files are moved to'}</div>
						</div>
						<select class="setting-control" value={$appSettings.trashDestination} onchange={(e) => updateSettings({ trashDestination: (e.target as HTMLSelectElement).value as any })}>
							<option value="system">{ar ? 'سلة مهملات النظام' : 'System trash'}</option>
							<option value="obsidian">{ar ? 'مجلد .trash' : '.trash folder'}</option>
							<option value="permanent">{ar ? 'حذف نهائي' : 'Permanently delete'}</option>
						</select>
					</div>

				<!-- ═══ APPEARANCE ═══ -->
				{:else if activeSection === 'appearance'}
					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'نظام الألوان الأساسي' : 'Base color scheme'}</div>
							<div class="setting-desc">{ar ? 'اختر بين الوضع الفاتح والداكن أو اتبع نظام التشغيل' : 'Choose between light and dark mode, or follow your OS'}</div>
						</div>
						<select class="setting-control" value={$appSettings.colorScheme} onchange={(e) => updateSettings({ colorScheme: (e.target as HTMLSelectElement).value as any })}>
							<option value="light">{ar ? 'فاتح' : 'Light'}</option>
							<option value="dark">{ar ? 'داكن' : 'Dark'}</option>
							<option value="system">{ar ? 'مطابقة النظام' : 'Adapt to system'}</option>
						</select>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'اللون التمييزي' : 'Accent color'}</div>
							<div class="setting-desc">{ar ? 'اللون المستخدم للأزرار والروابط والعناصر التفاعلية' : 'Used for interactive elements like links and buttons'}</div>
						</div>
						<div class="color-row">
							<input type="color" class="color-input" value={$appSettings.accentColor}
								onchange={(e) => updateSettings({ accentColor: (e.target as HTMLInputElement).value })} />
							<span class="color-hex">{$appSettings.accentColor}</span>
						</div>
					</div>

					<div class="setting-heading">{ar ? 'الخطوط' : 'Fonts'}</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'خط الواجهة' : 'Interface font'}</div>
							<div class="setting-desc">{ar ? 'الخط المستخدم للقوائم والأزرار والشريط الجانبي' : 'Font for menus, buttons, and sidebar'}</div>
						</div>
						<input class="setting-input" type="text" value={$appSettings.interfaceFont}
							placeholder={ar ? 'افتراضي النظام' : 'System default'}
							oninput={(e) => updateSettings({ interfaceFont: (e.target as HTMLInputElement).value })} />
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'خط النص' : 'Text font'}</div>
							<div class="setting-desc">{ar ? 'الخط المستخدم في محتوى الملاحظات' : 'Font used for note content in the editor'}</div>
						</div>
						<input class="setting-input" type="text" value={$appSettings.textFont}
							placeholder={ar ? 'افتراضي النظام' : 'System default'}
							oninput={(e) => updateSettings({ textFont: (e.target as HTMLInputElement).value })} />
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'الخط الثابت العرض' : 'Monospace font'}</div>
							<div class="setting-desc">{ar ? 'الخط المستخدم في كتل الكود' : 'Font for code blocks and inline code'}</div>
						</div>
						<input class="setting-input" type="text" value={$appSettings.monoFont}
							placeholder="Cascadia Code, Fira Code, Consolas"
							oninput={(e) => updateSettings({ monoFont: (e.target as HTMLInputElement).value })} />
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'حجم الخط' : 'Font size'}</div>
							<div class="setting-desc">{ar ? 'حجم الخط الأساسي لمحتوى الملاحظات' : 'Base font size for note content'}</div>
						</div>
						<div class="slider-row">
							<input type="range" class="setting-slider" min="12" max="24" step="1" value={$appSettings.fontSize}
								oninput={(e) => updateSettings({ fontSize: parseInt((e.target as HTMLInputElement).value) })} />
							<span class="slider-val">{$appSettings.fontSize}px</span>
						</div>
					</div>

				<!-- ═══ HOTKEYS ═══ -->
				{:else if activeSection === 'hotkeys'}
					<div class="hotkey-filter">
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
						<input type="text" placeholder={ar ? 'تصفية الأوامر...' : 'Filter commands...'}
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
									{#if cmd.shortcut}
										<kbd>{cmd.shortcut}</kbd>
									{:else}
										<span class="hotkey-unset">{ar ? 'لا يوجد' : 'Not set'}</span>
									{/if}
								</div>
							</div>
						{/each}
						{#if filteredCommands.length === 0}
							<div class="hotkey-empty">{ar ? 'لم يتم العثور على أوامر' : 'No commands found'}</div>
						{/if}
					</div>

				<!-- ═══ CORE PLUGINS ═══ -->
				{:else if activeSection === 'plugins'}
					<p class="section-intro">{ar ? 'الإضافات الأساسية المضمنة في التطبيق. يمكنك تفعيل أو تعطيل أي منها.' : 'Core plugins that come built-in. You can enable or disable each one.'}</p>
					{#each corePlugins as plugin}
						<div class="setting-item plugin-item">
							<div class="setting-info">
								<div class="setting-name">{plugin.name}</div>
								<div class="setting-desc">{plugin.desc}</div>
							</div>
							<label class="toggle">
								<input type="checkbox" checked={getPluginEnabled(plugin.id)}
									onchange={() => togglePlugin(plugin.id)} />
								<span class="toggle-slider"></span>
							</label>
						</div>
					{/each}

				<!-- ═══ AI PROVIDER ═══ -->
				{:else if activeSection === 'ai'}
					<p class="section-intro">{ar ? 'ربط خدمة ذكاء اصطناعي لتلخيص الملاحظات والمحادثة مع المحتوى.' : 'Connect an AI service for note summarization and chat with your content.'}</p>

					<div class="setting-item">
						<div class="setting-info">
							<div class="setting-name">{ar ? 'المزود' : 'Provider'}</div>
							<div class="setting-desc">{ar ? 'اختر خدمة الذكاء الاصطناعي' : 'Choose your AI service provider'}</div>
						</div>
						<select class="setting-control" value={$aiSettings.provider ?? ''} onchange={handleProviderChange}>
							<option value="">— {ar ? 'بدون' : 'None'} —</option>
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
									<div class="setting-name">{ar ? 'مفتاح API' : 'API Key'}</div>
									<div class="setting-desc">{ar ? 'مفتاح الوصول للخدمة' : 'Your API access key'}</div>
								</div>
								<input class="setting-input" type="password"
									placeholder={ar ? 'أدخل مفتاح API' : 'Enter API key'}
									value={$aiSettings.apiKey}
									oninput={(e) => updateAISettings({ apiKey: (e.target as HTMLInputElement).value })} />
							</div>
						{/if}

						{#if info.hasBaseUrl}
							<div class="setting-item">
								<div class="setting-info">
									<div class="setting-name">{ar ? 'رابط الخادم' : 'Server URL'}</div>
									<div class="setting-desc">{ar ? 'عنوان خادم Ollama المحلي' : 'Local Ollama server address'}</div>
								</div>
								<input class="setting-input" type="text"
									placeholder="http://localhost:11434"
									value={$aiSettings.baseUrl}
									oninput={(e) => updateAISettings({ baseUrl: (e.target as HTMLInputElement).value })} />
							</div>
						{/if}

						<div class="setting-item">
							<div class="setting-info">
								<div class="setting-name">{ar ? 'النموذج' : 'Model'}</div>
								<div class="setting-desc">{ar ? 'نموذج الذكاء الاصطناعي المستخدم' : 'AI model to use'}</div>
							</div>
							<input class="setting-input" type="text"
								value={$aiSettings.model}
								placeholder={DEFAULT_MODELS[$aiSettings.provider]}
								oninput={(e) => updateAISettings({ model: (e.target as HTMLInputElement).value })} />
						</div>

						<div class="setting-item">
							<div class="setting-info">
								<div class="setting-name">{ar ? 'اختبار الاتصال' : 'Connection'}</div>
								<div class="setting-desc">
									{#if testStatus === 'success'}
										<span class="test-success">✓ {ar ? 'متصل بنجاح' : 'Connected successfully'}</span>
									{:else if testStatus === 'failed'}
										<span class="test-failed">✕ {ar ? 'فشل الاتصال' : 'Connection failed'}</span>
									{:else}
										{ar ? 'تحقق من إعدادات الاتصال' : 'Verify your connection settings'}
									{/if}
								</div>
							</div>
							<button class="test-btn" onclick={testConnection} disabled={testing}>
								{testing ? (ar ? 'جارٍ الاختبار...' : 'Testing...') : (ar ? 'اختبار' : 'Test')}
							</button>
						</div>
					{/if}
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
		width: 90vw; max-width: 900px; height: 80vh;
		display: flex;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 12px;
		box-shadow: var(--shadow-l);
		overflow: hidden;
	}

	/* ═══ SIDEBAR ═══ */
	.settings-sidebar {
		width: 220px; flex-shrink: 0;
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
	.setting-item.sub-setting { padding-inline-start: 16px; }
	.setting-info { flex: 1; min-width: 0; }
	.setting-name { font-size: 0.88rem; font-weight: 500; color: var(--text-normal); }
	.setting-desc { font-size: 0.78rem; color: var(--text-muted); margin-top: 2px; }
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
	.setting-input {
		min-width: 180px; max-width: 240px; padding: 6px 10px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px; color: var(--text-normal);
		font-size: 0.85rem; font-family: inherit;
	}
	.setting-input:focus { border-color: var(--interactive-accent); outline: none; }

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

	/* ═══ ABOUT ═══ */
	.about-section { text-align: center; padding: 32px 0; }
	.about-logo { font-size: 3rem; margin-bottom: 8px; color: var(--interactive-accent); }
	.about-name { font-size: 1.5rem; font-weight: 700; color: var(--text-normal); }
	.about-tagline { font-size: 0.9rem; color: var(--text-muted); margin-top: 2px; }
	.about-version {
		display: inline-block; margin-top: 8px;
		font-size: 0.78rem; color: var(--text-faint);
		background: var(--background-secondary-alt);
		padding: 2px 10px; border-radius: 12px;
	}
	.about-desc { font-size: 0.85rem; color: var(--text-muted); margin-top: 16px; max-width: 400px; margin-inline: auto; }
	.about-links { margin-top: 16px; display: flex; justify-content: center; gap: 12px; }
	.about-link {
		display: flex; align-items: center; gap: 4px;
		font-size: 0.82rem; color: var(--interactive-accent);
		cursor: pointer;
	}

	/* ═══ HOTKEYS ═══ */
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
	kbd {
		font-family: var(--font-monospace-theme);
		font-size: 0.78rem; color: var(--text-muted);
		background: var(--background-secondary);
		border: 1px solid var(--background-modifier-border);
		padding: 2px 8px; border-radius: 4px;
	}
	.hotkey-unset { font-size: 0.78rem; color: var(--text-faint); font-style: italic; }
	.hotkey-empty { text-align: center; padding: 24px; color: var(--text-faint); font-size: 0.85rem; }

	/* ═══ PLUGINS ═══ */
	.plugin-item .setting-name { font-weight: 500; }

	/* ═══ AI ═══ */
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
</style>
