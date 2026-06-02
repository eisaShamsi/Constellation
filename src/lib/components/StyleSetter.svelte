<script lang="ts">
	/**
	 * Constellation Style Setter (CSS) — MIG-070, built from scratch.
	 *
	 * The SINGLE SOURCE for all styles + themes. Replaces the (disabled, not deleted) MIG-069
	 * StylePresetsPanel. The old panel froze the main thread by rendering N heavy mini-interface
	 * preview cards at once — so the Setter renders the unified list as LIGHT rows (name + a
	 * single accent swatch + Apply), computed IMPERATIVELY. The rich full-interface preview will
	 * be a SINGLE pane for the draft being edited — never a gallery of heavy cards.
	 *
	 * This first iteration delivers: the unified list (built-in themes + saved Styles) with
	 * Apply, with zero heavy rendering. Live preview + edit controls + full-page land next.
	 */
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { detectDir } from '$lib/utils';
	import {
		loadStylePresets, unifiedStyleList, applyPreset, type StylePreset,
	} from '$lib/libraries/stylePresets';

	let styles = $state<StylePreset[]>([]);
	let loaded = $state(false);
	let busy = $state(false);
	let appliedId = $state<string | null>(null);
	let appliedTimer: ReturnType<typeof setTimeout> | null = null;

	/** The Style's accent colour, read cheaply from the captured section (no preview compute). */
	function accentOf(p: StylePreset): string {
		const ct = p.sections?.colorsTheme as { accentColor?: string } | undefined;
		return ct?.accentColor || '#7c3aed';
	}

	/** Rebuild the unified list IMPERATIVELY (built-in/custom Themes + saved Styles). */
	async function refresh() { styles = unifiedStyleList(await loadStylePresets()); }

	onMount(async () => { try { await refresh(); } finally { loaded = true; } });

	async function apply(p: StylePreset) {
		busy = true;
		try {
			await applyPreset($state.snapshot(p) as StylePreset);
			appliedId = p.id;
			if (appliedTimer) clearTimeout(appliedTimer);
			appliedTimer = setTimeout(() => { if (appliedId === p.id) appliedId = null; }, 1600);
		} finally { busy = false; }
	}
</script>

<div class="css">
	<div class="setting-section-heading">{$t('styles.title') || 'Styles'}</div>
	<div class="setting-desc" style="margin-bottom: 10px;">
		{$t('styles.desc') || 'Switch between styles with a click, and reuse them across every universe.'}
	</div>

	{#if !loaded}
		<div class="css-state">{$t('lensBlock.loading') || 'Loading…'}</div>
	{:else}
		<div class="css-list">
			{#each styles as p (p.id)}
				<div class="css-row" class:applied={appliedId === p.id}>
					<span class="css-swatch" style="background:{accentOf(p)}" aria-hidden="true"></span>
					<span class="css-name" dir={detectDir(p.name)} title={p.name}>{p.name}</span>
					<button class="css-apply" disabled={busy} onclick={() => apply(p)}>
						{appliedId === p.id ? ($t('styles.applied') || 'Applied ✓') : ($t('styles.apply') || 'Apply')}
					</button>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.css { margin-block: 6px; }
	.css-state { color: var(--text-muted); font-size: 0.82rem; padding: 8px 2px; }
	.css-list { display: flex; flex-direction: column; gap: 3px; }
	.css-row {
		display: flex; align-items: center; gap: 9px; padding: 6px 9px;
		border-radius: 7px; border: 1px solid var(--background-modifier-border);
		background: var(--background-primary);
	}
	.css-row.applied { border-color: var(--interactive-accent); box-shadow: inset 0 0 0 1px var(--interactive-accent); }
	.css-row:hover { background: var(--background-modifier-hover); }
	.css-swatch { width: 15px; height: 15px; border-radius: 50%; flex: none; box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.18); }
	.css-name {
		flex: 1; min-width: 0; font-size: 0.86rem; color: var(--text-normal);
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.css-apply {
		flex: none; padding: 4px 12px; font: inherit; font-size: 0.78rem; font-weight: 600;
		border: none; border-radius: 6px; cursor: pointer;
		background: var(--interactive-accent); color: var(--text-on-accent, #fff);
	}
	.css-apply:disabled { opacity: 0.5; cursor: default; }
</style>
