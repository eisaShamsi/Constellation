<script lang="ts">
	/**
	 * MIG-103 §4 — THE KIND STUDIO.
	 *
	 * **Concept (the horse):** Constellation shows you the kinds of note you have
	 * already been writing, so you can name one and keep it. An impression taken from
	 * casts already made — not invention, and not a catalogue to browse.
	 *
	 * The layout is that sentence in two columns. The RAIL is what Constellation
	 * measured: a count and the fields these notes share, never a guess. The DETAIL
	 * column is the offer and the decision. Nothing re-sorts, ever — the order is the
	 * engine's own, so the user learns the list once.
	 *
	 * SLICE 1 IS READ-ONLY. It calls one command and writes nothing. The name field is
	 * live but its value is held in memory only; Keep and Not-a-kind arrive in Slices
	 * 2 and 3.
	 *
	 * This component takes props and emits callbacks — it does no routing and assumes
	 * no page chrome — which is what makes it liftable into the later "core plugin,
	 * an app within the app" without a rewrite.
	 */
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { SvelteMap } from 'svelte/reactivity';
	import { t } from '$lib/i18n';
	import { formatNumerals } from '$lib/utils';
	import { appSettings } from '$lib/libraries/store';
	import TemplateStudioRow from '$lib/components/TemplateStudioRow.svelte';
	import TemplateStudioDetail from '$lib/components/TemplateStudioDetail.svelte';
	import { kindSignature, type DiscoveredShape } from '$lib/templates/discoveredKinds';

	let { onOpenExample, onClose }: {
		onOpenExample: (path: string) => void;
		onClose: () => void;
	} = $props();

	let kinds = $state<DiscoveredShape[]>([]);
	let loading = $state(true);
	let error = $state('');
	let selectedSig = $state<string>('');
	/** Names typed but not yet kept. Keyed by SIGNATURE, never by index — the order can
	 *  change between scans as notes are written, and an index would silently reattach
	 *  a name to a different kind. */
	let drafts = new SvelteMap<string, string>();

	let selected = $derived(kinds.find((k) => kindSignature(k) === selectedSig) ?? null);
	let numerals = $derived($appSettings.numeralStyle ?? 'arabic');

	onMount(async () => {
		try {
			// One on-demand read. Measured at ~0.33s warm on a 7,802-note Universe;
			// a cold first touch of a large index file can take several seconds, which
			// is what the loading state below is for.
			const res = await invoke<DiscoveredShape[]>('discover_template_shapes', { maxShapes: 40 });
			kinds = res;
			if (res.length > 0) selectedSig = kindSignature(res[0]);
		} catch (e) {
			error = String(e);
		}
		loading = false;
	});

	function draftFor(k: DiscoveredShape): string {
		const sig = kindSignature(k);
		if (drafts.has(sig)) return drafts.get(sig)!;
		return k.proposed_name?.name ?? '';
	}
</script>

<div class="ks">
	<header class="ks-head">
		<div class="ks-head-text">
			<h2 class="ks-title">{$t('templateStudio.title')}</h2>
			{#if !loading && !error}
				<p class="ks-sub">{$t('templateStudio.subtitle', { count: formatNumerals(kinds.length, numerals) })}</p>
			{/if}
		</div>
		<button class="ks-close" type="button" onclick={onClose} aria-label={$t('common.close')}>✕</button>
	</header>

	{#if loading}
		<p class="ks-msg">{$t('templateStudio.loading')}</p>
	{:else if error}
		<p class="ks-msg ks-err">{error}</p>
	{:else if kinds.length === 0}
		<!-- Not an error and not a failure: a Universe can genuinely have no shape yet. -->
		<p class="ks-msg">{$t('templateStudio.emptyState')}</p>
	{:else}
		<div class="ks-body">
			<nav class="ks-rail" aria-label={$t('templateStudio.title')}>
				{#each kinds as k (kindSignature(k))}
					<TemplateStudioRow kind={k} selected={kindSignature(k) === selectedSig}
						onSelect={() => (selectedSig = kindSignature(k))} />
				{/each}
			</nav>
			<div class="ks-detail">
				{#if selected}
					<TemplateStudioDetail kind={selected} draftName={draftFor(selected)} {onOpenExample}
						onNameChange={(v) => drafts.set(kindSignature(selected), v)} />
				{/if}
			</div>
		</div>
	{/if}
</div>

<style>
	/* Matches `.rv` in ReviewerView — the same overlay, so the same sizing contract.
	   (The half-page symptom was never here: the welcome pane was rendering as a
	   sibling and splitting the height. See the note at the `{:else if isHome}`
	   branch in +layout.svelte.) */
	.ks {
		display: flex;
		flex-direction: column;
		block-size: 100%;
		min-block-size: 0;
		background: var(--background-primary);
	}
	.ks-head {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 16px;
		padding: 18px 24px 14px;
		border-block-end: 1px solid var(--background-modifier-border);
	}
	.ks-title { margin: 0; font-size: calc(1.05rem * var(--rs-scale, 1)); font-weight: 600; }
	.ks-sub {
		margin: 4px 0 0;
		color: var(--text-muted);
		font-size: calc(0.82rem * var(--rs-scale, 1));
		max-inline-size: 62ch;
		line-height: 1.5;
	}
	.ks-close {
		flex: none;
		border: none;
		background: none;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 15px;
		padding: 2px 6px;
	}
	.ks-msg { padding: 28px 24px; color: var(--text-muted); font-size: calc(0.86rem * var(--rs-scale, 1)); }
	.ks-err { color: var(--text-error, var(--color-red)); }
	.ks-body {
		display: grid;
		grid-template-columns: minmax(300px, 360px) 1fr; /* inline axis → flips under RTL */
		min-block-size: 0;
		flex: 1;
	}
	.ks-rail {
		overflow-y: auto;
		border-inline-end: 1px solid var(--background-modifier-border);
		padding-block: 6px;
	}
	.ks-detail { overflow-y: auto; }
</style>
