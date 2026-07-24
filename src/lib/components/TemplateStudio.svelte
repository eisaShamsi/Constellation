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
	/** Optional fields the user ticked, per kind. Parent-owned so a virtualized row
	 *  scrolling out of view cannot lose a tick. */
	let picks = new SvelteMap<string, Set<string>>();
	/** Outcome of the last Keep, per kind. */
	let statuses = new SvelteMap<string, { state: 'kept' | 'clash' | 'error' | 'merged'; path?: string; message?: string; added?: string[] }>();
	/** What we wrote, per kind — undo refuses unless the file still matches it byte
	 *  for byte, so an edited mold is never silently discarded. */
	let written = new SvelteMap<string, string>();
	/** Kinds that ALREADY have a mold, read from the templates' own `from_kind:` line.
	 *  Survives restarts because it is not app state — it is in the user's files. */
	let keptKinds = new SvelteMap<string, { name: string; path: string }>();

	const TEMPLATE_EXISTS = 'TEMPLATE_EXISTS:';

	/** Shared empty set, so the render path can READ a missing entry without creating it. */
	const NO_PICKS: ReadonlySet<string> = new Set();

	/**
	 * Read-only. **Never writes.**
	 *
	 * The first version did `if (!picks.has(sig)) picks.set(sig, new Set())` — and was
	 * called from the markup. Reading a reactive map and writing it in the same breath
	 * DURING RENDER invalidates what the render just read, so Svelte re-runs the render,
	 * which calls it again: an infinite loop that pinned the UI thread. The header
	 * painted its loaded state while the body stayed on "Looking through your notes…",
	 * and the app stopped responding.
	 *
	 * The project's Rule 2 already forbids exactly this ("never read and write the same
	 * reactive variable") — the rule names `$effect`, and a render expression is the
	 * same fault in different clothes. Writes belong in event handlers only.
	 */
	function picksFor(sig: string): ReadonlySet<string> {
		return picks.get(sig) ?? NO_PICKS;
	}

	/** Core fields plus the ticked optional ones, in the members' own spelling. */
	function fieldsFor(k: DiscoveredShape): string[] {
		const sig = kindSignature(k);
		const chosen = picks.get(sig) ?? new Set<string>();
		return [
			...k.core.map((key) => k.fields.find((f) => f.key === key)?.display ?? key),
			...k.fields.filter((f) => chosen.has(f.key)).map((f) => f.display),
		];
	}

	async function keep(k: DiscoveredShape) {
		const sig = kindSignature(k);
		const name = (drafts.get(sig) ?? k.proposed_name?.name ?? '').trim();
		if (!name) return;
		try {
			const path = await invoke<string>('adopt_discovered_kind', {
				name,
				fields: fieldsFor(k),
				headings: k.headings.map((h) => h.display),
				core: k.core,
				folder: $appSettings.templateFolder || null,
			});
			keptKinds.set(sig, { name, path });
			// Remember the exact bytes so undo can prove the file is untouched.
			written.set(sig, await invoke<string>('read_note', { filePath: path }).catch(() => ''));
			statuses.set(sig, { state: 'kept', path });
		} catch (e) {
			const msg = String(e);
			statuses.set(sig, msg.startsWith(TEMPLATE_EXISTS)
				? { state: 'clash', path: msg.slice(TEMPLATE_EXISTS.length) }
				: { state: 'error', message: msg });
		}
	}

	async function resolve(k: DiscoveredShape, choice: 'merge' | 'cancel') {
		const sig = kindSignature(k);
		const clash = statuses.get(sig);
		if (choice === 'cancel') {
			// Back to the name box so the user can rename — the field is where the
			// decision lives, so we return them to it rather than opening a dialog.
			statuses.delete(sig);
			return;
		}
		try {
			const added = await invoke<string[]>('merge_fields_into_template', {
				templatePath: clash?.path ?? '',
				fields: fieldsFor(k),
			});
			statuses.set(sig, { state: 'merged', added });
		} catch (e) {
			statuses.set(sig, { state: 'error', message: String(e) });
		}
	}

	async function undo(k: DiscoveredShape) {
		const sig = kindSignature(k);
		const st = statuses.get(sig);
		if (!st?.path) return;
		try {
			const ok = await invoke<boolean>('undo_adopt_kind', {
				templatePath: st.path,
				expectedContent: written.get(sig) ?? '',
			});
			if (ok) { statuses.delete(sig); written.delete(sig); keptKinds.delete(sig); }
			else statuses.set(sig, { state: 'error', message: $t('templateStudio.undoRefused') });
		} catch (e) {
			statuses.set(sig, { state: 'error', message: String(e) });
		}
	}

	let selected = $derived(kinds.find((k) => kindSignature(k) === selectedSig) ?? null);
	let numerals = $derived($appSettings.numeralStyle ?? 'arabic');

	onMount(async () => {
		try {
			// One on-demand read. Measured at ~0.33s warm on a 7,802-note Universe;
			// a cold first touch of a large index file can take several seconds, which
			// is what the loading state below is for.
			const res = await invoke<DiscoveredShape[]>('discover_template_shapes', { maxShapes: 40 });
			kinds = res;
			// Which of these already have a mold? The templates themselves answer.
			try {
				const kept = await invoke<{ signature: string; name: string; path: string }[]>(
					'list_kept_kinds', { folder: $appSettings.templateFolder || null });
				for (const k of kept) keptKinds.set(k.signature, { name: k.name, path: k.path });
			} catch { /* no templates folder yet is not an error */ }
			if (res.length > 0) selectedSig = kindSignature(res[0]);
		} catch (e) {
			error = String(e);
		}
		loading = false;
	});

	function draftFor(k: DiscoveredShape): string {
		const sig = kindSignature(k);
		if (drafts.has(sig)) return drafts.get(sig)!;
		// A kind that already has a mold shows the name the USER gave it, not a fresh
		// proposal — re-proposing a name for something already named reads as amnesia.
		return keptKinds.get(sig)?.name ?? k.proposed_name?.name ?? '';
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
						adoptedName={keptKinds.get(kindSignature(k))?.name ?? ''}
						onSelect={() => (selectedSig = kindSignature(k))} />
				{/each}
			</nav>
			<div class="ks-detail">
				{#if selected}
					<!-- Keyed on the selected kind (2026-07-24 inspection): the detail column
					     holds local state about THIS kind — `shownCandidate`, the alternate
					     name the user clicked. Rendered unkeyed, that state survived a `kind`
					     prop change, so selecting a different kind left the previous kind's
					     token heading the evidence block: the wrong kind's naming reasoning
					     attached to the kind on screen. A different kind is a different
					     subject; remount it, so this and any future local state reset with it.
					     (The visible-reasoning surface is exactly what The Constellation Way
					     relies on to keep "smart" from becoming presumptuous.) -->
					{#key kindSignature(selected)}
					<TemplateStudioDetail kind={selected} draftName={draftFor(selected)} {onOpenExample}
						picked={picksFor(kindSignature(selected))}
						status={statuses.get(kindSignature(selected)) ?? null}
						onNameChange={(v) => { drafts.set(kindSignature(selected), v); statuses.delete(kindSignature(selected)); }}
						onTogglePick={(key) => {
							const sig = kindSignature(selected);
							const next = new Set(picks.get(sig) ?? []);
							if (next.has(key)) next.delete(key); else next.add(key);
							picks.set(sig, next);
						}}
						onKeep={() => keep(selected)}
						onUndo={() => undo(selected)}
						onResolve={(c) => resolve(selected, c)} />
					{/key}
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
