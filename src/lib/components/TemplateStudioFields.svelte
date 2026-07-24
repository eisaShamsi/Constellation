<script lang="ts">
	/**
	 * MIG-103 §4 Slice 1 — what these notes carry.
	 *
	 * Two groups, one field per line, counts end-aligned in a tabular column.
	 * **No bars, no sparklines, no percentages, no tint.** The list is already sorted
	 * by count descending, so the eye reads the gradient down the numeral column and
	 * finds the knee where the tail stops being worth reading — the entire cognitive
	 * payload of a fill-rate histogram, delivered by ordering, at zero ink. A bar beside
	 * `163` would only make 163-vs-156 comparable, and that distinction carries no
	 * decision. (Form-Aligns-To-Purpose.)
	 *
	 * Counts are the engine's own integers. The surface never computes
	 * `round(fill × support)` — a derived integer can be off by one, and a wrong count
	 * is fabrication.
	 */
	import { formatNumerals, detectDir } from '$lib/utils';
	import { appSettings } from '$lib/libraries/store';
	import { t } from '$lib/i18n';
	import { coreFields, tailFields, kindSignature, type DiscoveredShape } from '$lib/templates/discoveredKinds';

	let {
		kind,
		picked,
		onToggle,
	}: {
		kind: DiscoveredShape;
		/** Optional field keys the user ticked. Owned by the PARENT, so a row that
		 *  scrolls out of a virtualized list cannot lose its tick. */
		picked?: ReadonlySet<string>;
		onToggle?: (key: string) => void;
	} = $props();

	let core = $derived(coreFields(kind));
	let tail = $derived(tailFields(kind));
	let numerals = $derived($appSettings.numeralStyle ?? 'arabic');

	function n(v: number) { return formatNumerals(v, numerals); }
</script>

<section class="kf">
	<h3 class="kf-h">{$t('templateStudio.fieldsHeading')}</h3>

	<p class="kf-group">{$t('templateStudio.coreGroup', { count: n(kind.support) })}</p>
	<ul class="kf-list">
		{#each core as f (f.key)}
			<li class="kf-row">
				<span class="kf-key" dir={detectDir(f.display)}>{f.display}</span>
				<span class="kf-count">{$t('templateStudio.allOf', { count: n(kind.support) })}</span>
			</li>
		{/each}
	</ul>

	{#if tail.length > 0}
		<p class="kf-group">{$t('templateStudio.tailGroup')}</p>
		<!-- Boss 2026-07-23: chips, not a checkbox column. A vertical list of 21 optional
		     fields was taller than the screen and pushed everything below it out of view;
		     chips wrap, so the whole tail is visible at once and its shape is readable at a
		     glance. Each chip still carries its count — the evidence never leaves.

		     No virtualization here: chips wrap rather than stack, so even 60 fields is a
		     few rows, not 60. Rule 7's threshold exists for LISTS that grow without bound
		     in one direction; this no longer is one. -->
		<div class="kf-chips">
			{#each tail as f (f.key)}
				{@const on = picked?.has(f.key) ?? false}
				{#if onToggle}
					<button type="button" class="kf-chip" class:kf-chip-on={on}
						aria-pressed={on} dir={detectDir(f.display)}
						onclick={() => onToggle?.(f.key)}>
						<span class="kf-chip-key">{f.display}</span>
						<span class="kf-chip-n">{n(f.count)}</span>
					</button>
				{:else}
					<span class="kf-chip" dir={detectDir(f.display)}>
						<span class="kf-chip-key">{f.display}</span>
						<span class="kf-chip-n">{n(f.count)}</span>
					</span>
				{/if}
			{/each}
		</div>
	{/if}

	{#if kind.headings.length > 0}
		<p class="kf-group">{$t('templateStudio.headingsGroup')}</p>
		<ul class="kf-list kf-heads">
			{#each kind.headings as h (h.text)}
				<li class="kf-row"><span class="kf-key" dir={detectDir(h.display)}>{h.display}</span></li>
			{/each}
		</ul>
	{/if}
</section>

<style>
	.kf { margin-block-start: 22px; }
	.kf-h {
		margin: 0 0 10px;
		font-size: calc(0.86rem * var(--rs-scale, 1));
		font-weight: 600;
		color: var(--text-normal);
	}
	.kf-group {
		margin: 14px 0 4px;
		font-size: calc(0.76rem * var(--rs-scale, 1));
		color: var(--text-muted);
	}
	.kf-list { list-style: none; margin: 0; padding: 0; }
	.kf-virtual { block-size: 320px; }
	.kf-row {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
		block-size: 30px;
		padding-inline: 4px;
		font-size: calc(0.82rem * var(--rs-scale, 1));
	}
	.kf-key { unicode-bidi: isolate; min-inline-size: 0; overflow-wrap: anywhere; }
	.kf-count {
		flex: none;
		font-variant-numeric: tabular-nums;
		color: var(--text-faint);
		font-size: calc(0.76rem * var(--rs-scale, 1));
	}
	.kf-heads .kf-key { color: var(--text-muted); }
	.kf-chips { display: flex; flex-wrap: wrap; gap: 6px; margin-block-start: 2px; }
	.kf-chip {
		display: inline-flex; align-items: baseline; gap: 6px;
		padding: 3px 9px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 999px;
		background: var(--background-primary);
		color: var(--text-muted);
		font-size: calc(0.8rem * var(--rs-scale, 1));
		cursor: pointer;
		unicode-bidi: isolate;
	}
	.kf-chip:hover { background: var(--background-modifier-hover); }
	/* Chosen chips read as SELECTED, not merely hovered — the accent border is the
	   signal, so it survives a colour-blind reading and a dark theme alike. */
	.kf-chip-on {
		border-color: var(--interactive-accent);
		color: var(--text-normal);
		font-weight: 500;
	}
	.kf-chip-n { font-variant-numeric: tabular-nums; color: var(--text-faint); font-size: calc(0.72rem * var(--rs-scale, 1)); }
</style>
