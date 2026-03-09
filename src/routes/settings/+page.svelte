<script lang="ts">
	import { t, locale, setLocale, type Locale } from '$lib/i18n';
	import { aiSettings, updateAISettings, setProvider } from '$lib/ai/store';
	import { validateConnection } from '$lib/ai/engine';
	import { PROVIDER_INFO, DEFAULT_MODELS, type ProviderId } from '$lib/ai/provider';
	import { appSettings, updateSettings } from '$lib/vaults/store';

	const ar = $derived($locale === 'ar');
	let testStatus = $state('');
	let testing = $state(false);
	let activeSection = $state('general');

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

	const sections = $derived([
		{ id: 'general', label: ar ? 'عام' : 'General', icon: '⚙️' },
		{ id: 'editor', label: ar ? 'المحرر' : 'Editor', icon: '✏️' },
		{ id: 'files', label: ar ? 'الملفات والروابط' : 'Files & Links', icon: '📁' },
		{ id: 'appearance', label: ar ? 'المظهر' : 'Appearance', icon: '🎨' },
		{ id: 'daily', label: ar ? 'الملاحظات اليومية' : 'Daily Notes', icon: '📅' },
		{ id: 'ai', label: ar ? 'الذكاء الاصطناعي' : 'AI Provider', icon: '🤖' },
	]);
</script>

<div class="settings-page">
	<div class="settings-nav">
		{#each sections as section}
			<button class="nav-item" class:active={activeSection === section.id} onclick={() => activeSection = section.id}>
				<span class="nav-icon">{section.icon}</span>
				<span class="nav-label">{section.label}</span>
			</button>
		{/each}
	</div>

	<div class="settings-content">
		{#if activeSection === 'general'}
			<h2>{ar ? 'عام' : 'General'}</h2>
			<section class="settings-section">
				<label class="field">
					<span>{$t('settings.language')}</span>
					<p class="field-desc">{$t('settings.languageDescription')}</p>
					<select value={$locale} onchange={handleLangChange}>
						<option value="en">English</option>
						<option value="ar">العربية</option>
					</select>
				</label>
			</section>

		{:else if activeSection === 'editor'}
			<h2>{ar ? 'المحرر' : 'Editor'}</h2>
			<section class="settings-section">
				<label class="field">
					<span>{ar ? 'العرض الافتراضي' : 'Default view for new tabs'}</span>
					<select value={$appSettings.defaultView} onchange={(e) => updateSettings({ defaultView: (e.target as HTMLSelectElement).value as any })}>
						<option value="reading">{ar ? 'وضع القراءة' : 'Reading view'}</option>
						<option value="editing">{ar ? 'وضع التحرير' : 'Editing view'}</option>
					</select>
				</label>

				<label class="field toggle-field">
					<div>
						<span>{ar ? 'إظهار أرقام الأسطر' : 'Show line numbers'}</span>
						<p class="field-desc">{ar ? 'عرض أرقام الأسطر في المحرر' : 'Display line numbers in the editor gutter'}</p>
					</div>
					<input type="checkbox" checked={$appSettings.showLineNumbers}
						onchange={(e) => updateSettings({ showLineNumbers: (e.target as HTMLInputElement).checked })} />
				</label>

				<label class="field toggle-field">
					<div>
						<span>{ar ? 'عرض سطر مقروء' : 'Readable line length'}</span>
						<p class="field-desc">{ar ? 'تحديد عرض المحتوى لتسهيل القراءة' : 'Limit content width for easier reading'}</p>
					</div>
					<input type="checkbox" checked={$appSettings.readableLineLength}
						onchange={(e) => updateSettings({ readableLineLength: (e.target as HTMLInputElement).checked })} />
				</label>

				<label class="field">
					<span>{ar ? 'حجم المسافة' : 'Tab size'}</span>
					<select value={$appSettings.tabSize} onchange={(e) => updateSettings({ tabSize: parseInt((e.target as HTMLSelectElement).value) })}>
						<option value="2">2 {ar ? 'مسافات' : 'spaces'}</option>
						<option value="4">4 {ar ? 'مسافات' : 'spaces'}</option>
					</select>
				</label>

				<label class="field toggle-field">
					<div>
						<span>{ar ? 'قوائم ذكية' : 'Smart lists'}</span>
						<p class="field-desc">{ar ? 'متابعة القوائم تلقائيا عند الضغط على Enter' : 'Auto-continue lists on Enter'}</p>
					</div>
					<input type="checkbox" checked={$appSettings.smartLists}
						onchange={(e) => updateSettings({ smartLists: (e.target as HTMLInputElement).checked })} />
				</label>

				<label class="field toggle-field">
					<div>
						<span>{ar ? 'إغلاق الأقواس تلقائيا' : 'Auto-pair brackets'}</span>
						<p class="field-desc">{ar ? 'إضافة قوس الإغلاق تلقائيا' : 'Automatically insert closing brackets'}</p>
					</div>
					<input type="checkbox" checked={$appSettings.autoPairBrackets}
						onchange={(e) => updateSettings({ autoPairBrackets: (e.target as HTMLInputElement).checked })} />
				</label>

				<label class="field toggle-field">
					<div>
						<span>{ar ? 'التدقيق الإملائي' : 'Spellcheck'}</span>
					</div>
					<input type="checkbox" checked={$appSettings.spellcheck}
						onchange={(e) => updateSettings({ spellcheck: (e.target as HTMLInputElement).checked })} />
				</label>
			</section>

		{:else if activeSection === 'files'}
			<h2>{ar ? 'الملفات والروابط' : 'Files & Links'}</h2>
			<section class="settings-section">
				<label class="field">
					<span>{ar ? 'موقع الملاحظات الجديدة' : 'Default location for new notes'}</span>
					<select value={$appSettings.defaultNoteLocation} onchange={(e) => updateSettings({ defaultNoteLocation: (e.target as HTMLSelectElement).value as any })}>
						<option value="root">{ar ? 'جذر الخزينة' : 'Vault root folder'}</option>
						<option value="current">{ar ? 'المجلد الحالي' : 'Same folder as current file'}</option>
						<option value="folder">{ar ? 'مجلد محدد' : 'Specified folder'}</option>
					</select>
				</label>

				{#if $appSettings.defaultNoteLocation === 'folder'}
					<label class="field">
						<span>{ar ? 'المجلد' : 'Folder path'}</span>
						<input type="text" value={$appSettings.defaultNoteFolder}
							oninput={(e) => updateSettings({ defaultNoteFolder: (e.target as HTMLInputElement).value })} />
					</label>
				{/if}

				<label class="field">
					<span>{ar ? 'مجلد المرفقات' : 'Default attachment folder'}</span>
					<input type="text" value={$appSettings.defaultAttachmentFolder}
						placeholder={ar ? 'المجلد الحالي' : 'Same folder as note'}
						oninput={(e) => updateSettings({ defaultAttachmentFolder: (e.target as HTMLInputElement).value })} />
				</label>

				<label class="field toggle-field">
					<div>
						<span>{ar ? 'تحديث الروابط تلقائيا' : 'Auto-update links on rename'}</span>
						<p class="field-desc">{ar ? 'تحديث جميع الروابط عند إعادة تسمية ملاحظة' : 'Update all links when renaming a note'}</p>
					</div>
					<input type="checkbox" checked={$appSettings.autoUpdateLinks}
						onchange={(e) => updateSettings({ autoUpdateLinks: (e.target as HTMLInputElement).checked })} />
				</label>

				<label class="field toggle-field">
					<div>
						<span>{ar ? 'استخدام ويكي لينك' : 'Use WikiLinks'}</span>
						<p class="field-desc">{ar ? 'استخدام [[ويكي لينك]] بدلا من [روابط ماركداون](...)' : 'Use [[WikiLinks]] instead of [Markdown links](...)'}</p>
					</div>
					<input type="checkbox" checked={$appSettings.useWikilinks}
						onchange={(e) => updateSettings({ useWikilinks: (e.target as HTMLInputElement).checked })} />
				</label>

				<label class="field toggle-field">
					<div>
						<span>{ar ? 'تأكيد الحذف' : 'Confirm file deletion'}</span>
					</div>
					<input type="checkbox" checked={$appSettings.confirmDelete}
						onchange={(e) => updateSettings({ confirmDelete: (e.target as HTMLInputElement).checked })} />
				</label>

				<label class="field">
					<span>{ar ? 'وجهة الحذف' : 'Deleted files destination'}</span>
					<select value={$appSettings.trashDestination} onchange={(e) => updateSettings({ trashDestination: (e.target as HTMLSelectElement).value as any })}>
						<option value="system">{ar ? 'سلة المهملات' : 'System trash'}</option>
						<option value="obsidian">{ar ? 'مجلد .trash' : 'Obsidian .trash folder'}</option>
						<option value="permanent">{ar ? 'حذف دائم' : 'Permanently delete'}</option>
					</select>
				</label>
			</section>

		{:else if activeSection === 'appearance'}
			<h2>{ar ? 'المظهر' : 'Appearance'}</h2>
			<section class="settings-section">
				<label class="field">
					<span>{ar ? 'نظام الألوان' : 'Color scheme'}</span>
					<select value={$appSettings.colorScheme} onchange={(e) => updateSettings({ colorScheme: (e.target as HTMLSelectElement).value as any })}>
						<option value="light">{ar ? 'فاتح' : 'Light'}</option>
						<option value="dark">{ar ? 'داكن' : 'Dark'}</option>
						<option value="system">{ar ? 'تلقائي (النظام)' : 'Adapt to system'}</option>
					</select>
				</label>

				<label class="field">
					<span>{ar ? 'اللون التمييزي' : 'Accent color'}</span>
					<div class="color-picker">
						<input type="color" value={$appSettings.accentColor}
							onchange={(e) => updateSettings({ accentColor: (e.target as HTMLInputElement).value })} />
						<span>{$appSettings.accentColor}</span>
					</div>
				</label>

				<label class="field">
					<span>{ar ? 'خط الواجهة' : 'Interface font'}</span>
					<input type="text" value={$appSettings.interfaceFont}
						placeholder={ar ? 'افتراضي النظام' : 'System default'}
						oninput={(e) => updateSettings({ interfaceFont: (e.target as HTMLInputElement).value })} />
				</label>

				<label class="field">
					<span>{ar ? 'خط النص' : 'Text font'}</span>
					<input type="text" value={$appSettings.textFont}
						placeholder={ar ? 'افتراضي النظام' : 'System default'}
						oninput={(e) => updateSettings({ textFont: (e.target as HTMLInputElement).value })} />
				</label>

				<label class="field">
					<span>{ar ? 'الخط الثابت العرض' : 'Monospace font'}</span>
					<input type="text" value={$appSettings.monoFont}
						placeholder="Cascadia Code, Fira Code, Consolas"
						oninput={(e) => updateSettings({ monoFont: (e.target as HTMLInputElement).value })} />
				</label>

				<label class="field">
					<span>{ar ? 'حجم الخط' : 'Font size'}</span>
					<div class="slider-field">
						<input type="range" min="12" max="24" step="1" value={$appSettings.fontSize}
							oninput={(e) => updateSettings({ fontSize: parseInt((e.target as HTMLInputElement).value) })} />
						<span class="slider-value">{$appSettings.fontSize}px</span>
					</div>
				</label>
			</section>

		{:else if activeSection === 'daily'}
			<h2>{ar ? 'الملاحظات اليومية' : 'Daily Notes'}</h2>
			<section class="settings-section">
				<label class="field">
					<span>{ar ? 'تنسيق اسم الملف' : 'Date format'}</span>
					<p class="field-desc">{ar ? 'تنسيق اسم ملف الملاحظة اليومية' : 'Format for daily note filenames'}</p>
					<input type="text" value={$appSettings.dailyNoteFormat}
						placeholder="%Y-%m-%d"
						oninput={(e) => updateSettings({ dailyNoteFormat: (e.target as HTMLInputElement).value })} />
				</label>

				<label class="field">
					<span>{ar ? 'مجلد الملاحظات اليومية' : 'Daily notes folder'}</span>
					<input type="text" value={$appSettings.dailyNoteFolder}
						placeholder={ar ? 'جذر الخزينة' : 'Vault root'}
						oninput={(e) => updateSettings({ dailyNoteFolder: (e.target as HTMLInputElement).value })} />
				</label>

				<label class="field">
					<span>{ar ? 'مجلد القوالب' : 'Templates folder'}</span>
					<input type="text" value={$appSettings.templateFolder}
						placeholder="Templates"
						oninput={(e) => updateSettings({ templateFolder: (e.target as HTMLInputElement).value })} />
				</label>
			</section>

		{:else if activeSection === 'ai'}
			<h2>{$t('settings.ai')}</h2>
			<section class="settings-section">
				<p class="section-desc">{$t('settings.aiDescription')}</p>

				<label class="field">
					<span>{$t('settings.provider')}</span>
					<select value={$aiSettings.provider ?? ''} onchange={handleProviderChange}>
						<option value="">—</option>
						{#each Object.entries(PROVIDER_INFO) as [id, info]}
							<option value={id}>{info.name}</option>
						{/each}
					</select>
				</label>

				{#if $aiSettings.provider}
					{@const info = PROVIDER_INFO[$aiSettings.provider]}

					{#if info.requiresKey}
						<label class="field">
							<span>{$t('settings.apiKey')}</span>
							<input
								type="password"
								placeholder={$t('settings.apiKeyPlaceholder')}
								value={$aiSettings.apiKey}
								oninput={(e) => updateAISettings({ apiKey: (e.target as HTMLInputElement).value })}
							/>
						</label>
					{/if}

					{#if info.hasBaseUrl}
						<label class="field">
							<span>{$t('settings.ollamaUrl')}</span>
							<input
								type="text"
								placeholder={$t('settings.ollamaUrlPlaceholder')}
								value={$aiSettings.baseUrl}
								oninput={(e) => updateAISettings({ baseUrl: (e.target as HTMLInputElement).value })}
							/>
						</label>
					{/if}

					<label class="field">
						<span>{$t('settings.model')}</span>
						<input
							type="text"
							value={$aiSettings.model}
							placeholder={DEFAULT_MODELS[$aiSettings.provider]}
							oninput={(e) => updateAISettings({ model: (e.target as HTMLInputElement).value })}
						/>
					</label>

					<button class="test-btn" onclick={testConnection} disabled={testing}>
						{testing ? $t('common.loading') : $t('settings.testConnection')}
					</button>

					{#if testStatus === 'success'}
						<p class="status success">{$t('settings.connectionSuccess')}</p>
					{:else if testStatus === 'failed'}
						<p class="status error">{$t('settings.connectionFailed')}</p>
					{/if}
				{/if}
			</section>
		{/if}
	</div>
</div>

<style>
	.settings-page {
		display: flex; gap: 24px; max-width: 900px;
	}
	.settings-nav {
		width: 180px; flex-shrink: 0;
		display: flex; flex-direction: column; gap: 2px;
	}
	.nav-item {
		display: flex; align-items: center; gap: 8px;
		padding: 6px 10px; border-radius: 4px;
		border: none; background: none; cursor: pointer;
		font-family: inherit; font-size: 0.85rem;
		color: var(--text-secondary); text-align: start;
	}
	.nav-item:hover { background: var(--bg-hover); color: var(--text); }
	.nav-item.active { background: var(--accent-bg); color: var(--accent); font-weight: 500; }
	.nav-icon { font-size: 0.9rem; }

	.settings-content { flex: 1; min-width: 0; }

	h2 { font-size: 1.3rem; margin-bottom: 1rem; color: var(--text); }
	.section-desc { color: var(--text-muted); font-size: 0.85rem; margin-bottom: 1rem; }

	.settings-section {
		background: var(--bg-secondary);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 1.5rem;
		margin-bottom: 1.5rem;
	}

	.field {
		display: block;
		margin-bottom: 1rem;
	}
	.field span {
		display: block;
		font-size: 0.85rem;
		color: var(--text);
		margin-bottom: 0.3rem;
		font-weight: 500;
	}
	.field-desc { color: var(--text-muted); font-size: 0.78rem; margin: 0 0 4px; }

	.toggle-field {
		display: flex; align-items: center; justify-content: space-between; gap: 12px;
	}
	.toggle-field input[type="checkbox"] {
		width: 36px; height: 20px; appearance: none;
		background: var(--border); border-radius: 10px;
		position: relative; cursor: pointer; flex-shrink: 0;
		transition: background 0.2s;
	}
	.toggle-field input[type="checkbox"]::after {
		content: ''; position: absolute; top: 2px; left: 2px;
		width: 16px; height: 16px; border-radius: 50%;
		background: #fff; transition: transform 0.2s;
	}
	.toggle-field input[type="checkbox"]:checked {
		background: var(--accent);
	}
	.toggle-field input[type="checkbox"]:checked::after {
		transform: translateX(16px);
	}

	select, input[type="text"], input[type="password"] {
		width: 100%;
		padding: 0.5em 0.8em;
		background: var(--bg);
		border: 1px solid var(--border);
		border-radius: 6px;
		color: var(--text);
		font-size: 0.9rem;
		box-sizing: border-box;
		font-family: inherit;
	}
	select:focus, input:focus { border-color: var(--accent); outline: none; }

	.color-picker {
		display: flex; align-items: center; gap: 8px;
	}
	.color-picker input[type="color"] {
		width: 36px; height: 36px; border: 1px solid var(--border); border-radius: 6px;
		padding: 2px; cursor: pointer;
	}
	.color-picker span { font-size: 0.82rem; color: var(--text-muted); font-family: monospace; }

	.slider-field {
		display: flex; align-items: center; gap: 8px;
	}
	.slider-field input[type="range"] { flex: 1; }
	.slider-value { font-size: 0.82rem; color: var(--text-muted); min-width: 40px; }

	.test-btn {
		background: var(--bg-hover);
		border: 1px solid var(--border);
		color: var(--text);
		padding: 0.5em 1.2em;
		border-radius: 6px;
		cursor: pointer;
		font-size: 0.85rem;
		font-family: inherit;
		transition: all 0.2s;
	}
	.test-btn:hover { border-color: var(--accent); }
	.test-btn:disabled { opacity: 0.5; cursor: not-allowed; }

	.status { font-size: 0.85rem; margin-top: 0.5rem; }
	.success { color: #1a7f37; }
	.error { color: var(--danger); }
</style>
