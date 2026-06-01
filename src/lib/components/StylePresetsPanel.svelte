<script lang="ts">
	/**
	 * MIG-069 §C — the Settings → Styles panel.
	 *
	 * Named, app-GLOBAL style presets: save the current look (ticking which sections to
	 * include), apply one with a click, rename / duplicate / delete. Reusable across every
	 * universe (presets live in {app_data_dir}/style-presets.json). Export / import = §D.
	 */
	import { onMount, onDestroy } from 'svelte';
	import { t } from '$lib/i18n';
	import { detectDir } from '$lib/utils';
	import {
		loadStylePresets, saveStylePresets, newPresetFromCurrent, clonePreset, applyPreset,
		presetSectionKeys, exportPreset, importPreset, stylePreview, SECTION_CATALOGUE, type StylePreset, type SectionKey,
	} from '$lib/libraries/stylePresets';

	let presets = $state<StylePreset[]>([]);
	let loaded = $state(false);
	let busy = $state(false);
	let appliedId = $state<string | null>(null);
	let importError = $state(false);
	let applyTimer: ReturnType<typeof setTimeout> | null = null;

	// Save-new form
	let showSave = $state(false);
	let newName = $state('');
	let ticks = $state<Record<string, boolean>>({});

	// Rename
	let renamingId = $state<string | null>(null);
	let renameValue = $state('');

	const freshTicks = () => Object.fromEntries(SECTION_CATALOGUE.map((s) => [s.key, s.defaultOn]));
	const sectionLabel = (k: SectionKey) => $t(`styles.section.${k}`);

	onMount(async () => { presets = await loadStylePresets(); loaded = true; });
	onDestroy(() => { if (applyTimer) clearTimeout(applyTimer); });

	async function persist() {
		busy = true;
		try { await saveStylePresets($state.snapshot(presets) as StylePreset[]); } finally { busy = false; }
	}

	function openSave() { newName = ''; ticks = freshTicks(); showSave = true; }
	async function confirmSave() {
		const keys = SECTION_CATALOGUE.filter((s) => ticks[s.key]).map((s) => s.key);
		if (!newName.trim() || keys.length === 0) return;
		presets = [...presets, newPresetFromCurrent(newName, keys)];
		showSave = false;
		await persist();
	}

	async function apply(p: StylePreset) {
		busy = true;
		try {
			await applyPreset($state.snapshot(p) as StylePreset);
			appliedId = p.id;
			if (applyTimer) clearTimeout(applyTimer);
			applyTimer = setTimeout(() => { if (appliedId === p.id) appliedId = null; }, 1600);
		} finally { busy = false; }
	}

	async function duplicate(p: StylePreset) {
		presets = [...presets, clonePreset($state.snapshot(p) as StylePreset)];
		await persist();
	}

	async function remove(p: StylePreset) {
		presets = presets.filter((x) => x.id !== p.id);
		await persist();
	}

	async function doImport() {
		importError = false;
		try {
			const p = await importPreset();
			if (p) { presets = [...presets, p]; await persist(); }
		} catch {
			importError = true;
			setTimeout(() => (importError = false), 3500);
		}
	}

	function startRename(p: StylePreset) { renamingId = p.id; renameValue = p.name; }
	async function confirmRename() {
		const id = renamingId;
		presets = presets.map((p) => (p.id === id ? { ...p, name: renameValue.trim() || p.name } : p));
		renamingId = null;
		await persist();
	}
</script>

<div class="sp">
	<div class="setting-section-heading">{$t('styles.title') || 'Styles'}</div>
	<div class="setting-desc" style="margin-bottom: 10px;">
		{$t('styles.desc') || 'Save your look as a named style, switch with a click, and reuse it across every universe.'}
	</div>

	{#if !loaded}
		<div class="sp-state">{$t('lensBlock.loading') || 'Loading…'}</div>
	{:else}
		{#if presets.length === 0}
			<div class="sp-empty">{$t('styles.empty') || 'No styles yet — save your current look to reuse it anywhere.'}</div>
		{:else}
			<div class="sp-grid">
				{#each presets as p (p.id)}
					{@const pv = stylePreview(p)}
					<div class="sp-card" class:sp-applied={appliedId === p.id}>
						<!-- A generated self-portrait: theme paper + text/font sample, the accent
						     pill at the captured corner-radius, and the 8 link-type colours. -->
						<div class="sp-portrait" style="background:{pv.bg}; font-family:{pv.font};" aria-hidden="true">
							<div class="sp-pv-top">
								<span class="sp-pv-aa" style="color:{pv.text}">Aa</span>
								<span class="sp-pv-pill" style="background:{pv.accent}; border-radius:{Math.min(pv.radius, 8)}px;"></span>
							</div>
							<div class="sp-pv-dots">
								{#each pv.dots as d, i (i)}<span class="sp-pv-dot" style="background:{d}"></span>{/each}
							</div>
						</div>
						<div class="sp-card-body">
							{#if renamingId === p.id}
								<input class="sp-rename" bind:value={renameValue} dir={detectDir(renameValue)}
									onkeydown={(e) => { if (e.key === 'Enter') confirmRename(); if (e.key === 'Escape') renamingId = null; }} />
								<button class="sp-mini sp-primary" onclick={confirmRename}>{$t('styles.save') || 'Save'}</button>
							{:else}
								<div class="sp-name" dir={detectDir(p.name)} title={p.name}>{p.name}</div>
								<div class="sp-sections">{presetSectionKeys(p).map(sectionLabel).join(' · ')}</div>
								<div class="sp-actions">
									<button class="sp-apply" disabled={busy} onclick={() => apply(p)}>
										{appliedId === p.id ? ($t('styles.applied') || 'Applied ✓') : ($t('styles.apply') || 'Apply')}
									</button>
									<button class="sp-icon" title={$t('styles.export') || 'Export'} aria-label={$t('styles.export') || 'Export'} onclick={() => exportPreset(p)}>⤓</button>
									<button class="sp-icon" title={$t('styles.rename') || 'Rename'} aria-label={$t('styles.rename') || 'Rename'} onclick={() => startRename(p)}>✎</button>
									<button class="sp-icon" title={$t('styles.duplicate') || 'Duplicate'} aria-label={$t('styles.duplicate') || 'Duplicate'} onclick={() => duplicate(p)}>⧉</button>
									<button class="sp-icon sp-del" title={$t('styles.delete') || 'Delete'} aria-label={$t('styles.delete') || 'Delete'} onclick={() => remove(p)}>✕</button>
								</div>
							{/if}
						</div>
					</div>
				{/each}
			</div>
		{/if}

		{#if showSave}
			<div class="sp-save">
				<input class="sp-name-input" bind:value={newName} placeholder={$t('styles.namePlaceholder') || 'Style name…'}
					dir={detectDir(newName)} onkeydown={(e) => { if (e.key === 'Enter') confirmSave(); if (e.key === 'Escape') showSave = false; }} />
				<div class="sp-include">{$t('styles.include') || 'Include:'}</div>
				<div class="sp-ticks">
					{#each SECTION_CATALOGUE as s (s.key)}
						<label class="sp-tick">
							<input type="checkbox" bind:checked={ticks[s.key]} />
							<span>{sectionLabel(s.key)}</span>
						</label>
					{/each}
				</div>
				<div class="sp-save-actions">
					<button class="sp-mini" onclick={() => (showSave = false)}>{$t('styles.cancel') || 'Cancel'}</button>
					<button class="sp-mini sp-primary" disabled={!newName.trim()} onclick={confirmSave}>{$t('styles.save') || 'Save'}</button>
				</div>
			</div>
		{:else}
			<div class="sp-bottom">
				<button class="sp-savebtn" onclick={openSave}>{$t('styles.saveNew') || '+ Save current style…'}</button>
				<button class="sp-importbtn" disabled={busy} onclick={doImport}>{$t('styles.import') || 'Import…'}</button>
			</div>
			{#if importError}<div class="sp-state sp-warn">{$t('styles.importError') || 'That file is not a valid Constellation style.'}</div>{/if}
		{/if}
	{/if}
</div>

<style>
	.sp { margin-block: 6px; }
	.sp-state, .sp-empty { color: var(--text-muted); font-size: 0.82rem; padding: 8px 2px; }
	.sp-grid {
		display: grid; grid-template-columns: repeat(auto-fill, minmax(148px, 1fr));
		gap: 10px; margin-bottom: 12px;
	}
	.sp-card {
		display: flex; flex-direction: column; overflow: hidden;
		border: 1px solid var(--background-modifier-border); border-radius: 10px;
		background: var(--background-primary); transition: border-color 0.12s, box-shadow 0.12s;
	}
	.sp-card:hover { border-color: var(--interactive-accent); }
	.sp-card.sp-applied { border-color: var(--interactive-accent); box-shadow: inset 0 0 0 1px var(--interactive-accent); }
	/* The generated self-portrait — theme paper, text/font sample, accent pill, link palette. */
	.sp-portrait {
		height: 60px; padding: 8px 10px; display: flex; flex-direction: column;
		justify-content: space-between; overflow: hidden;
		border-bottom: 1px solid var(--background-modifier-border);
	}
	.sp-pv-top { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
	.sp-pv-aa { font-size: 19px; font-weight: 600; line-height: 1; letter-spacing: 0.3px; }
	.sp-pv-pill { display: inline-block; width: 32px; height: 9px; flex: none; }
	.sp-pv-dots { display: flex; gap: 3px; }
	.sp-pv-dot { width: 9px; height: 9px; border-radius: 50%; box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.14); }
	.sp-card-body { padding: 7px 9px 9px; display: flex; flex-direction: column; gap: 5px; }
	.sp-name {
		font-size: 0.86rem; font-weight: 600; color: var(--text-normal);
		white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
	}
	.sp-sections { font-size: 0.66rem; color: var(--text-faint); line-height: 1.3; max-height: 2.6em; overflow: hidden; }
	.sp-rename {
		font: inherit; font-size: 0.86rem; padding: 3px 6px; width: 60%;
		border: 1px solid var(--interactive-accent); border-radius: 5px;
		background: var(--background-primary); color: var(--text-normal); outline: none;
	}
	.sp-actions { display: flex; align-items: center; gap: 3px; margin-top: 1px; }
	.sp-apply {
		flex: 1; padding: 4px 8px; font: inherit; font-size: 0.76rem; font-weight: 600;
		border: none; border-radius: 6px; cursor: pointer; white-space: nowrap;
		background: var(--interactive-accent); color: var(--text-on-accent, #fff);
	}
	.sp-apply:disabled { opacity: 0.5; cursor: default; }
	.sp-icon {
		background: none; border: none; cursor: pointer; color: var(--text-faint);
		font-size: 0.8rem; padding: 3px 4px; border-radius: 5px;
	}
	.sp-icon:hover { color: var(--text-normal); background: var(--background-modifier-hover); }
	.sp-del:hover { color: var(--text-error, #e53e3e); }
	.sp-savebtn {
		padding: 6px 12px; font: inherit; font-size: 0.84rem; cursor: pointer;
		border: 1px dashed var(--background-modifier-border); border-radius: 8px;
		background: none; color: var(--text-muted); flex: 1;
	}
	.sp-savebtn:hover { color: var(--text-normal); border-color: var(--interactive-accent); }
	.sp-bottom { display: flex; gap: 8px; }
	.sp-importbtn {
		padding: 6px 14px; font: inherit; font-size: 0.84rem; cursor: pointer; white-space: nowrap;
		border: 1px dashed var(--background-modifier-border); border-radius: 8px;
		background: none; color: var(--text-muted);
	}
	.sp-importbtn:hover { color: var(--text-normal); border-color: var(--interactive-accent); }
	.sp-warn { color: var(--text-error, #e53e3e); }
	.sp-save {
		padding: 10px; border: 1px solid var(--interactive-accent); border-radius: 8px;
		display: flex; flex-direction: column; gap: 8px;
	}
	.sp-name-input {
		font: inherit; font-size: 0.88rem; padding: 6px 8px;
		border: 1px solid var(--background-modifier-border); border-radius: 6px;
		background: var(--background-primary); color: var(--text-normal); outline: none;
	}
	.sp-name-input:focus { border-color: var(--interactive-accent); }
	.sp-include { font-size: 0.76rem; color: var(--text-muted); }
	.sp-ticks { display: grid; grid-template-columns: 1fr 1fr; gap: 4px 12px; }
	.sp-tick { display: flex; align-items: center; gap: 6px; font-size: 0.82rem; color: var(--text-normal); cursor: pointer; }
	.sp-save-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 2px; }
	.sp-mini {
		padding: 5px 14px; font: inherit; font-size: 0.82rem; cursor: pointer;
		border: 1px solid var(--background-modifier-border); border-radius: 6px;
		background: var(--background-primary); color: var(--text-normal);
	}
	.sp-primary { border: none; background: var(--interactive-accent); color: var(--text-on-accent, #fff); font-weight: 600; }
	.sp-mini:disabled { opacity: 0.5; cursor: default; }
</style>
