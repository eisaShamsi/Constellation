<script lang="ts">
	import { t, locale, setLocale, type Locale } from '$lib/i18n';
	import { aiSettings, updateAISettings, setProvider } from '$lib/ai/store';
	import { validateConnection } from '$lib/ai/engine';
	import { PROVIDER_INFO, DEFAULT_MODELS, type ProviderId } from '$lib/ai/provider';

	let testStatus = $state('');
	let testing = $state(false);

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
</script>

<div class="settings-page">
	<h1>{$t('settings.title')}</h1>

	<!-- Language Section -->
	<section class="settings-section">
		<h2>{$t('settings.language')}</h2>
		<p class="section-desc">{$t('settings.languageDescription')}</p>
		<select value={$locale} onchange={handleLangChange}>
			<option value="en">English</option>
			<option value="ar">العربية</option>
		</select>
	</section>

	<!-- AI Provider Section -->
	<section class="settings-section">
		<h2>{$t('settings.ai')}</h2>
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
</div>

<style>
	.settings-page { max-width: 600px; }
	h1 { font-size: 1.8rem; margin-bottom: 2rem; }
	h2 { font-size: 1.2rem; margin-bottom: 0.25rem; }
	.section-desc { color: #57606a; font-size: 0.9rem; margin-bottom: 1rem; }

	.settings-section {
		background: #f6f8fa;
		border: 1px solid #d0d7de;
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
		color: #57606a;
		margin-bottom: 0.3rem;
	}

	select, input {
		width: 100%;
		padding: 0.6em 0.8em;
		background: #ffffff;
		border: 1px solid #d0d7de;
		border-radius: 6px;
		color: #1f2328;
		font-size: 0.95rem;
		box-sizing: border-box;
	}
	select:focus, input:focus { border-color: #7c3aed; outline: none; }

	.test-btn {
		background: #eaeef2;
		border: 1px solid #d0d7de;
		color: #24292f;
		padding: 0.6em 1.2em;
		border-radius: 6px;
		cursor: pointer;
		font-size: 0.9rem;
		transition: all 0.2s;
	}
	.test-btn:hover { border-color: #7c3aed; background: #d0d7de; }
	.test-btn:disabled { opacity: 0.5; cursor: not-allowed; }

	.status { font-size: 0.9rem; margin-top: 0.5rem; }
	.success { color: #1a7f37; }
	.error { color: #cf222e; }
</style>
