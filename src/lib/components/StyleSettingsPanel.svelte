<script lang="ts">
	/**
	 * StyleSettingsPanel — Full Obsidian-compatible Style Settings UI.
	 * Renders all 10 setting types with appropriate controls.
	 */
	import { t, locale } from '$lib/i18n';
	import type { StyleSettingsBlock, StyleSetting } from '$lib/theme/styleSettings';
	import { getLocalizedTitle, getLocalizedDescription } from '$lib/theme/styleSettings';

	let {
		blocks = [] as StyleSettingsBlock[],
		values = {} as Record<string, string>,
		onChange,
	}: {
		blocks?: StyleSettingsBlock[];
		values?: Record<string, string>;
		onChange?: (id: string, value: string) => void;
	} = $props();

	const lang = $derived($locale?.slice(0, 2) ?? 'en');

	let collapsedSections = $state<Set<string>>(new Set());

	function toggleSection(id: string) {
		const next = new Set(collapsedSections);
		if (next.has(id)) next.delete(id); else next.add(id);
		collapsedSections = next;
	}

	function setValue(id: string, value: string) {
		values[id] = value;
		onChange?.(id, value);
	}

	function resetValue(setting: StyleSetting) {
		const def = setting.default ?? '';
		setValue(setting.id, def);
	}

	function getTitle(s: StyleSetting): string { return getLocalizedTitle(s, lang); }
	function getDesc(s: StyleSetting): string | undefined { return getLocalizedDescription(s, lang); }
</script>

{#if blocks.length === 0}
	<div class="ss-empty">{$t('settings.appearance.noStyleSettings') || 'No style settings available for this theme.'}</div>
{:else}
	{#each blocks as block}
		<div class="ss-block">
			<div class="ss-block-name">{block.name}</div>

			{#each block.settings as setting (setting.id)}
				<!-- Heading -->
				{#if setting.type === 'heading'}
					<button class="ss-heading ss-heading-{setting.level ?? 3}"
						onclick={() => toggleSection(setting.id)}>
						<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
							class:rotated={collapsedSections.has(setting.id)}>
							<polyline points="6 9 12 15 18 9"/>
						</svg>
						{getTitle(setting)}
					</button>

				<!-- Info Text -->
				{:else if setting.type === 'info-text'}
					<div class="ss-info">
						{#if setting.markdown}
							<div class="ss-info-md">{@html getDesc(setting) ?? getTitle(setting)}</div>
						{:else}
							<div class="ss-info-text">{getDesc(setting) ?? getTitle(setting)}</div>
						{/if}
					</div>

				<!-- Class Toggle -->
				{:else if setting.type === 'class-toggle'}
					<div class="ss-row">
						<div class="ss-label">
							<div class="ss-title">{getTitle(setting)}</div>
							{#if getDesc(setting)}<div class="ss-desc">{getDesc(setting)}</div>{/if}
						</div>
						<button class="ss-switch" class:on={values[setting.id] === 'true' || (!values[setting.id] && setting.default === 'true')}
							onclick={() => setValue(setting.id, values[setting.id] === 'true' ? 'false' : 'true')}>
							<span class="ss-switch-knob"></span>
						</button>
					</div>

				<!-- Class Select -->
				{:else if setting.type === 'class-select'}
					<div class="ss-row">
						<div class="ss-label">
							<div class="ss-title">{getTitle(setting)}</div>
							{#if getDesc(setting)}<div class="ss-desc">{getDesc(setting)}</div>{/if}
						</div>
						<select class="ss-select" value={values[setting.id] ?? setting.default ?? ''}
							onchange={(e) => setValue(setting.id, (e.target as HTMLSelectElement).value)}>
							{#if setting.allowEmpty}
								<option value="none">—</option>
							{/if}
							{#each setting.options ?? [] as opt}
								<option value={opt.value}>{opt.label}</option>
							{/each}
						</select>
					</div>

				<!-- Variable Text -->
				{:else if setting.type === 'variable-text'}
					<div class="ss-row">
						<div class="ss-label">
							<div class="ss-title">{getTitle(setting)}{#if setting.quotes} <span class="ss-tag">quoted</span>{/if}</div>
							{#if getDesc(setting)}<div class="ss-desc">{getDesc(setting)}</div>{/if}
						</div>
						<div class="ss-input-wrap">
							<input type="text" class="ss-input" value={values[setting.id] ?? setting.default ?? ''}
								oninput={(e) => setValue(setting.id, (e.target as HTMLInputElement).value)} />
							<button class="ss-reset" onclick={() => resetValue(setting)} title="Reset">↺</button>
						</div>
					</div>

				<!-- Variable Number -->
				{:else if setting.type === 'variable-number'}
					<div class="ss-row">
						<div class="ss-label">
							<div class="ss-title">{getTitle(setting)}</div>
							{#if getDesc(setting)}<div class="ss-desc">{getDesc(setting)}</div>{/if}
						</div>
						<div class="ss-input-wrap">
							<input type="number" class="ss-input ss-input-num" value={values[setting.id] ?? setting.default ?? ''}
								min={setting.min} max={setting.max} step={setting.step}
								oninput={(e) => setValue(setting.id, (e.target as HTMLInputElement).value)} />
							{#if setting.format}<span class="ss-unit">{setting.format}</span>{/if}
							<button class="ss-reset" onclick={() => resetValue(setting)} title="Reset">↺</button>
						</div>
					</div>

				<!-- Variable Number Slider -->
				{:else if setting.type === 'variable-number-slider'}
					<div class="ss-row ss-row-slider">
						<div class="ss-label">
							<div class="ss-title">{getTitle(setting)}</div>
							{#if getDesc(setting)}<div class="ss-desc">{getDesc(setting)}</div>{/if}
						</div>
						<div class="ss-slider-wrap">
							<input type="range" class="ss-slider"
								min={setting.min ?? 0} max={setting.max ?? 100} step={setting.step ?? 1}
								value={values[setting.id] ?? setting.default ?? String(setting.min ?? 0)}
								oninput={(e) => setValue(setting.id, (e.target as HTMLInputElement).value)} />
							<span class="ss-slider-val">{values[setting.id] ?? setting.default ?? setting.min ?? 0}{setting.format ?? ''}</span>
							<button class="ss-reset" onclick={() => resetValue(setting)} title="Reset">↺</button>
						</div>
					</div>

				<!-- Variable Select -->
				{:else if setting.type === 'variable-select'}
					<div class="ss-row">
						<div class="ss-label">
							<div class="ss-title">{getTitle(setting)}</div>
							{#if getDesc(setting)}<div class="ss-desc">{getDesc(setting)}</div>{/if}
						</div>
						<div class="ss-input-wrap">
							<select class="ss-select" value={values[setting.id] ?? setting.default ?? ''}
								onchange={(e) => setValue(setting.id, (e.target as HTMLSelectElement).value)}>
								{#each setting.options ?? [] as opt}
									<option value={opt.value}>{opt.label}</option>
								{/each}
							</select>
							<button class="ss-reset" onclick={() => resetValue(setting)} title="Reset">↺</button>
						</div>
					</div>

				<!-- Variable Color -->
				{:else if setting.type === 'variable-color'}
					<div class="ss-row">
						<div class="ss-label">
							<div class="ss-title">{getTitle(setting)}</div>
							{#if getDesc(setting)}<div class="ss-desc">{getDesc(setting)}</div>{/if}
						</div>
						<div class="ss-input-wrap">
							<input type="color" class="ss-color"
								value={values[setting.id] ?? setting.default ?? '#000000'}
								oninput={(e) => setValue(setting.id, (e.target as HTMLInputElement).value)} />
							<span class="ss-color-hex">{values[setting.id] ?? setting.default ?? ''}</span>
							<button class="ss-reset" onclick={() => resetValue(setting)} title="Reset">↺</button>
						</div>
					</div>

				<!-- Variable Themed Color -->
				{:else if setting.type === 'variable-themed-color'}
					<div class="ss-row ss-row-themed">
						<div class="ss-label">
							<div class="ss-title">{getTitle(setting)}</div>
							{#if getDesc(setting)}<div class="ss-desc">{getDesc(setting)}</div>{/if}
						</div>
						<div class="ss-themed-colors">
							<div class="ss-themed-pair">
								<span class="ss-themed-label">☀️</span>
								<input type="color" class="ss-color"
									value={values[`${setting.id}@@light`] ?? setting.defaultLight ?? '#000000'}
									oninput={(e) => setValue(`${setting.id}@@light`, (e.target as HTMLInputElement).value)} />
							</div>
							<div class="ss-themed-pair">
								<span class="ss-themed-label">🌙</span>
								<input type="color" class="ss-color"
									value={values[`${setting.id}@@dark`] ?? setting.defaultDark ?? '#000000'}
									oninput={(e) => setValue(`${setting.id}@@dark`, (e.target as HTMLInputElement).value)} />
							</div>
							<button class="ss-reset" onclick={() => { resetValue(setting); setValue(`${setting.id}@@light`, setting.defaultLight ?? ''); setValue(`${setting.id}@@dark`, setting.defaultDark ?? ''); }} title="Reset">↺</button>
						</div>
					</div>
				{/if}
			{/each}
		</div>
	{/each}
{/if}

<style>
	.ss-empty { padding: 12px; color: var(--text-faint); font-size: 12px; text-align: center; }
	.ss-block { margin-bottom: 16px; }
	.ss-block-name {
		font-size: 13px; font-weight: 700; color: var(--interactive-accent);
		margin-bottom: 8px; padding-bottom: 4px;
		border-bottom: 1px solid var(--background-modifier-border);
	}
	/* Heading */
	.ss-heading {
		display: flex; align-items: center; gap: 6px; width: 100%;
		border: none; background: none; cursor: pointer;
		font-weight: 600; color: var(--text-normal); font-family: inherit;
		padding: 6px 0; text-align: start;
	}
	.ss-heading svg { color: var(--text-muted); transition: transform 0.15s; }
	.ss-heading svg.rotated { transform: rotate(-90deg); }
	.ss-heading-1 { font-size: 15px; }
	.ss-heading-2 { font-size: 14px; padding-inline-start: 8px; }
	.ss-heading-3 { font-size: 13px; padding-inline-start: 16px; }
	.ss-heading-4 { font-size: 12px; padding-inline-start: 24px; }
	.ss-heading-5 { font-size: 11px; padding-inline-start: 32px; }
	.ss-heading-6 { font-size: 11px; padding-inline-start: 40px; }
	/* Info */
	.ss-info { padding: 6px 8px; font-size: 12px; color: var(--text-muted); background: var(--background-secondary); border-radius: 6px; margin: 4px 0; }
	/* Row */
	.ss-row {
		display: flex; align-items: center; gap: 12px;
		padding: 8px 0; border-bottom: 1px solid var(--background-modifier-border);
	}
	.ss-row-slider { flex-wrap: wrap; }
	.ss-label { flex: 1; min-width: 0; }
	.ss-title { font-size: 12px; font-weight: 500; color: var(--text-normal); }
	.ss-desc { font-size: 10px; color: var(--text-muted); margin-top: 1px; }
	.ss-tag { font-size: 9px; color: var(--text-faint); background: var(--background-modifier-border); padding: 0 4px; border-radius: 3px; }
	/* Inputs */
	.ss-input-wrap { display: flex; align-items: center; gap: 6px; flex-shrink: 0; }
	.ss-input {
		border: 1px solid var(--background-modifier-border); border-radius: 4px;
		background: var(--background-primary); color: var(--text-normal);
		font-size: 12px; padding: 4px 8px; font-family: inherit; width: 180px;
	}
	.ss-input-num { width: 80px; }
	.ss-unit { font-size: 11px; color: var(--text-faint); }
	.ss-select {
		border: 1px solid var(--background-modifier-border); border-radius: 4px;
		background: var(--background-primary); color: var(--text-normal);
		font-size: 12px; padding: 4px 8px; font-family: inherit; min-width: 120px;
	}
	/* Slider */
	.ss-slider-wrap { display: flex; align-items: center; gap: 8px; flex-shrink: 0; }
	.ss-slider { width: 140px; cursor: pointer; }
	.ss-slider-val { font-size: 11px; color: var(--text-muted); min-width: 50px; }
	/* Color */
	.ss-color { width: 32px; height: 24px; border: 1px solid var(--background-modifier-border); border-radius: 4px; cursor: pointer; padding: 0; }
	.ss-color-hex { font-size: 11px; color: var(--text-muted); font-family: var(--font-monospace-theme); }
	/* Themed color */
	.ss-themed-colors { display: flex; align-items: center; gap: 8px; flex-shrink: 0; }
	.ss-themed-pair { display: flex; align-items: center; gap: 3px; }
	.ss-themed-label { font-size: 12px; }
	/* Toggle */
	.ss-switch {
		width: 36px; height: 20px; border-radius: 10px; border: none;
		background: var(--background-modifier-border); cursor: pointer;
		position: relative; flex-shrink: 0; transition: background 0.2s; padding: 0;
	}
	.ss-switch.on { background: var(--interactive-accent); }
	.ss-switch-knob {
		position: absolute; top: 2px; inset-inline-start: 2px;
		width: 16px; height: 16px; border-radius: 50%; background: white;
		transition: inset-inline-start 0.2s; box-shadow: 0 1px 2px rgba(0,0,0,0.2);
	}
	.ss-switch.on .ss-switch-knob { inset-inline-start: 18px; }
	/* Reset */
	.ss-reset {
		border: none; background: none; cursor: pointer; font-size: 14px;
		color: var(--text-faint); padding: 2px; opacity: 0; transition: opacity 0.15s;
	}
	.ss-row:hover .ss-reset, .ss-slider-wrap:hover .ss-reset,
	.ss-input-wrap:hover .ss-reset, .ss-themed-colors:hover .ss-reset { opacity: 1; }
	.ss-reset:hover { color: var(--text-normal); }
</style>
