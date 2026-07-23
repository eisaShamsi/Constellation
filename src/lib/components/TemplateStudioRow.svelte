<script lang="ts">
	/**
	 * MIG-103 §4 Slice 1 — one rail row in the Kind Studio.
	 *
	 * A pure function of its props: no local state, no IPC, no store reads beyond
	 * settings. That is what lets the rail adopt `VirtualList` later without a rewrite
	 * (Rule 7 — the cap is 40 today, below the 50 threshold).
	 *
	 * **The proposed name is deliberately NOT shown here.** The rail carries only what
	 * Constellation MEASURED — a count and the fields these notes share. A proposal is
	 * not a measurement, it is an offer, and it belongs beside the decision. Three
	 * consequences, all intended: the user cannot triage by "which ones did the app
	 * solve" (the wrong triage — the 679-note kind is the one whose naming pays most);
	 * the rail never mixes guesses with facts; and the moment a name appears here is the
	 * moment the USER put it there. The Constellation Way as a layout rule, not as copy.
	 */
	import { detectDir, formatNumerals } from '$lib/utils';
	import { appSettings } from '$lib/libraries/store';
	import { t } from '$lib/i18n';
	import type { DiscoveredShape } from '$lib/templates/discoveredKinds';

	let {
		kind,
		selected = false,
		adoptedName = '',
		onSelect,
	}: {
		kind: DiscoveredShape;
		selected?: boolean;
		adoptedName?: string;
		onSelect: () => void;
	} = $props();

	const MAX_CHIPS = 3;
	let shown = $derived(kind.core.slice(0, MAX_CHIPS));
	let overflow = $derived(kind.core.length - shown.length);

	/** The field's own casing as its notes write it — `Country`, not `country`. */
	function display(key: string): string {
		return kind.fields.find((f) => f.key === key)?.display ?? key;
	}
</script>

<button class="ks-row" class:ks-row-sel={selected} onclick={onSelect} type="button">
	<!-- Column 1 — the count. Tabular numerals give the rail a straight numeric edge,
	     so the shape of the corpus reads in one pass with no chart drawn. Honours the
	     user's numeral setting rather than Intl's locale default. -->
	<span class="ks-count">{formatNumerals(kind.support, $appSettings.numeralStyle ?? 'arabic')}</span>

	<span class="ks-ident">
		{#if adoptedName}
			<span class="ks-name" dir={detectDir(adoptedName)}>{adoptedName}</span>
		{:else}
			<!-- Bordered chips with NO separator glyph: a literal `·` between an English
			     key and an Arabic key is a bidi jump hazard, and the border removes any
			     need for one. Each chip isolates its own direction. -->
			{#each shown as key (key)}
				<span class="ks-chip" dir={detectDir(display(key))}>{display(key)}</span>
			{/each}
			{#if overflow > 0}
				<span class="ks-chip ks-chip-more"
					>+{formatNumerals(overflow, $appSettings.numeralStyle ?? 'arabic')}</span>
			{/if}
		{/if}
	</span>

	<span class="ks-state">
		{#if adoptedName}<span class="ks-kept" aria-label={$t('templateStudio.kept')}>✓</span>{/if}
	</span>
</button>

<style>
	.ks-row {
		display: grid;
		grid-template-columns: 4.5rem 1fr auto; /* inline axis — flips under RTL for free */
		align-items: center;
		gap: 8px;
		inline-size: 100%;
		block-size: 36px;
		padding-inline: 10px;
		border: none;
		background: none;
		color: var(--text-normal);
		cursor: pointer;
		text-align: start;
		font-size: calc(0.82rem * var(--rs-scale, 1));
	}
	.ks-row:hover { background: var(--background-modifier-hover); }
	.ks-row-sel { background: var(--background-modifier-active-hover, var(--background-modifier-hover)); }
	.ks-count {
		font-variant-numeric: tabular-nums;
		text-align: end;
		color: var(--text-muted);
	}
	.ks-ident {
		display: flex;
		align-items: center;
		gap: 4px;
		min-inline-size: 0;
		overflow: hidden;
	}
	.ks-chip {
		unicode-bidi: isolate;
		flex: none;
		padding: 1px 6px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 3px;
		color: var(--text-muted);
		font-size: calc(0.74rem * var(--rs-scale, 1));
		white-space: nowrap;
	}
	.ks-chip-more { border-style: dashed; }
	.ks-name { font-weight: 500; unicode-bidi: isolate; }
	.ks-kept { color: var(--interactive-accent); }
</style>
