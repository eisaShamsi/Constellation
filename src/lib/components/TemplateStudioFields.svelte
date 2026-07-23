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
	import VirtualList from '$lib/components/VirtualList.svelte';
	import { formatNumerals, detectDir } from '$lib/utils';
	import { appSettings } from '$lib/libraries/store';
	import { t } from '$lib/i18n';
	import { coreFields, tailFields, kindSignature, type DiscoveredShape, type ShapeField } from '$lib/templates/discoveredKinds';

	let { kind }: { kind: DiscoveredShape } = $props();

	let core = $derived(coreFields(kind));
	let tail = $derived(tailFields(kind));
	let numerals = $derived($appSettings.numeralStyle ?? 'arabic');

	/** Rule 7 — virtualize any list that can exceed 50 rows. */
	const VIRTUAL_AT = 50;
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
		{#if tail.length >= VIRTUAL_AT}
			<div class="kf-virtual">
				<VirtualList items={tail} getItemHeight={() => 30} scrollResetKey={kindSignature(kind)}>
					{#snippet row(f: ShapeField)}
						<div class="kf-row">
							<span class="kf-key" dir={detectDir(f.display)}>{f.display}</span>
							<span class="kf-count">{$t('templateStudio.someOf', { n: n(f.count), total: n(kind.support) })}</span>
						</div>
					{/snippet}
				</VirtualList>
			</div>
		{:else}
			<ul class="kf-list">
				{#each tail as f (f.key)}
					<li class="kf-row">
						<span class="kf-key" dir={detectDir(f.display)}>{f.display}</span>
						<span class="kf-count">{$t('templateStudio.someOf', { n: n(f.count), total: n(kind.support) })}</span>
					</li>
				{/each}
			</ul>
		{/if}
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
</style>
