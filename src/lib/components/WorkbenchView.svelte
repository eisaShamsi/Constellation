<script lang="ts">
	/**
	 * MIG-090 — the Workbench (Form C, v1). The ratified horse:
	 * "The Navigator translates what is in the user's mind into the notes it
	 *  refers to — and holds that working set while the user works it."
	 *
	 * Two moments, one surface: ASK (the Intent Bar — a verbatim phrase down
	 * the existing hybrid engine, no grammar, no configuration) and HOLD/ACT
	 * (the desk — a persisted membership whose every displayed fact is re-read
	 * live via workbench_hydrate; read-only verbs: open · done · set down).
	 * v1 writes NO note content. Composes Search/Index/Bases/Reviewer — never
	 * re-implements them (the whole-entity complementarity contract).
	 */
	import { onDestroy, onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { invoke } from '@tauri-apps/api/core';
	import { t } from '$lib/i18n';
	import {
		workbenchSets, workbenchKey, addToWorkbench, removeFromWorkbench,
		toggleWorkbenchDone, sweepWorkbenchDone, adoptWorkbenchIdentities,
		constellationSearch, embedText, appSettings, splitStage,
		type ConstellationSearchResult, type WorkbenchItem,
	} from '$lib/libraries/store';
	import NoteRow, { NOTE_ROW_HEIGHT } from './NoteRow.svelte';
	import NoteList from './NoteList.svelte';

	let {
		onNoteClick,
		onClose,
	}: {
		onNoteClick: (path: string, libraryName: string, newTab?: boolean) => void;
		onClose: () => void;
	} = $props();

	// ─── Hydration (membership → live facts; the stale-snapshot cure) ───
	interface HydratedRow {
		key: string; path: string; cid_cn: string; name: string;
		library_name: string; modified: number; word_count: number;
		stage: string | null; incoming_count: number; outgoing_count: number;
		incoming_link_types_json: string; outgoing_link_types_json: string;
		review_reason: string | null; review_due: boolean; snoozed: boolean;
	}
	let rows = $state<HydratedRow[]>([]);
	let hydrating = $state(false);
	let hydrateSeq = 0;

	async function hydrate() {
		const seq = ++hydrateSeq;
		const items = get(workbenchSets)[0]?.items ?? [];
		if (items.length === 0) { rows = []; return; }
		hydrating = true;
		const cids = items.filter(i => i.cid).map(i => i.cid as string);
		const paths = items.filter(i => !i.cid).map(i => i.path);
		try {
			const res = await invoke<HydratedRow[]>('workbench_hydrate', { cids, paths });
			if (seq !== hydrateSeq) return; // a newer hydration owns the write
			rows = res;
			// Self-upgrade path-keyed members to rename-proof cid keys +
			// refresh display paths (saves only when something changed).
			adoptWorkbenchIdentities(res.map(r => ({ key: r.key, path: r.path, cid_cn: r.cid_cn })));
		} catch (e) {
			if (seq === hydrateSeq) console.error('[workbench] hydrate failed:', e);
		} finally {
			if (seq === hydrateSeq) hydrating = false;
		}
	}
	onMount(hydrate);

	// ─── The held desk (membership order: newest first; missing = no row) ───
	const held = $derived.by(() => {
		const set = $workbenchSets[0];
		if (!set) return [] as { item: WorkbenchItem; row: HydratedRow | null }[];
		const byKey = new Map(rows.map(r => [r.key, r]));
		return [...set.items]
			.sort((a, b) => b.addedAt - a.addedAt)
			.map(item => ({ item, row: byKey.get(workbenchKey(item)) ?? null }));
	});

	// ─── The four v1 chips — client-side intersection ONLY (they narrow the
	//     held set; they never query — the pinned-test invariant). ───
	let chipDue = $state(false);
	let chipUnlinked = $state(false);
	let chipContested = $state(false);
	let chipForming = $state(false);

	function isContested(r: HydratedRow): boolean {
		for (const j of [r.incoming_link_types_json, r.outgoing_link_types_json]) {
			try {
				const o = JSON.parse(j || '{}');
				if ((o['contradicts'] ?? 0) > 0) return true;
			} catch { /* malformed json → not contested */ }
		}
		return false;
	}
	function isForming(r: HydratedRow): boolean {
		if (!r.stage) return false; // honest: unstaged notes are not "forming"
		const { lifecycle } = splitStage(r.stage);
		return lifecycle === 'spark' || lifecycle === 'birth' || lifecycle === 'growth';
	}
	function isUnlinked(r: HydratedRow): boolean {
		return r.incoming_count === 0 && r.outgoing_count === 0;
	}

	const anyChip = $derived(chipDue || chipUnlinked || chipContested || chipForming);
	const visible = $derived(held.filter(h => {
		if (!anyChip) return true;
		if (!h.row) return false; // missing rows carry no state to match
		if (chipDue && !h.row.review_due) return false;
		if (chipUnlinked && !isUnlinked(h.row)) return false;
		if (chipContested && !isContested(h.row)) return false;
		if (chipForming && !isForming(h.row)) return false;
		return true;
	}));
	const doneCount = $derived(($workbenchSets[0]?.items ?? []).filter(i => i.done).length);

	// ─── The Intent Bar (§5) — QuickSwitcher conventions: IME guard, 300 ms
	//     debounce, stale-result seq guard; the phrase goes VERBATIM to the
	//     existing hybrid engine (+ the embeddings vector when enabled). ───
	let query = $state('');
	let composing = $state(false);
	let results = $state<ConstellationSearchResult[]>([]);
	let searching = $state(false);
	let searchSeq = 0;
	let debounceTimer: ReturnType<typeof setTimeout> | undefined;
	const semanticOn = $derived($appSettings.enabledFeatures?.semanticSearch === true);

	function onQueryInput() {
		if (composing) return;
		clearTimeout(debounceTimer);
		debounceTimer = setTimeout(runSearch, 300);
	}
	async function runSearch() {
		const q = query.trim();
		const seq = ++searchSeq;
		if (!q) { results = []; searching = false; return; }
		searching = true;
		let vec: number[] | undefined;
		if (semanticOn) {
			try { vec = await embedText(q); } catch { /* lexical-only, hinted below */ }
		}
		if (seq !== searchSeq) return;
		try {
			const res = await constellationSearch({
				query: q,
				query_embedding: vec,
				mode: 'hybrid',
				limit: 50,
				include_snippet: false,
			});
			if (seq !== searchSeq) return; // a newer query owns the write
			results = res;
		} catch (e) {
			if (seq === searchSeq) { results = []; console.error('[workbench] search failed:', e); }
		} finally {
			if (seq === searchSeq) searching = false;
		}
	}
	function clearSearch() {
		searchSeq++;
		query = '';
		results = [];
		searching = false;
	}

	const heldPaths = $derived(new Set(($workbenchSets[0]?.items ?? []).map(i => i.path)));

	// ─── Verbs (v1: read-only — open / hold / done / set down) ───
	function openRow(path: string, libraryName: string, e: MouseEvent) {
		const newTab = e.ctrlKey || e.metaKey || e.button === 1;
		onNoteClick(path, libraryName, newTab);
		if (!newTab) onClose();
	}
	function holdResult(r: ConstellationSearchResult, e: MouseEvent) {
		if (e.ctrlKey || e.metaKey || e.button === 1) {
			onNoteClick(r.path, r.library_name, true);
			return;
		}
		if (addToWorkbench(r.path)) hydrate();
	}

	function fmtDate(epochSec: number): string {
		try { return new Date(epochSec * 1000).toLocaleDateString(); } catch { return ''; }
	}
	function chipsFor(r: HydratedRow): string[] {
		const out: string[] = [];
		if (r.review_due) out.push($t('workbench.chipDue') || 'due');
		if (isUnlinked(r)) out.push($t('workbench.chipUnlinked') || 'unlinked');
		if (isContested(r)) out.push($t('workbench.chipContested') || 'contested');
		if (isForming(r)) out.push($t('workbench.chipForming') || 'forming');
		return out;
	}

	onDestroy(() => clearTimeout(debounceTimer));
</script>

<div class="wb" role="region" aria-label={$t('workbench.title') || 'Workbench'}>
	<div class="wb-header">
		<h2 class="wb-title">{$t('workbench.title') || 'Workbench'}</h2>
		<span class="wb-count">{($workbenchSets[0]?.items ?? []).length}</span>
		{#if doneCount > 0}
			<button class="wb-sweep" onclick={() => { sweepWorkbenchDone(); hydrate(); }}>
				{$t('workbench.sweepDone') || 'Sweep done'} ({doneCount})
			</button>
		{/if}
		<button class="wb-close" onclick={onClose} title={$t('workbench.close') || 'Close'}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
		</button>
	</div>

	<!-- The Intent Bar -->
	<div class="wb-bar">
		<input
			class="wb-input"
			type="search"
			dir="auto"
			placeholder={$t('workbench.askPlaceholder') || 'Ask for notes in your own words…'}
			bind:value={query}
			oninput={onQueryInput}
			oncompositionstart={() => composing = true}
			oncompositionend={() => { composing = false; onQueryInput(); }}
		/>
		{#if query && !semanticOn}
			<span class="wb-hint">{$t('workbench.semanticOff') || 'semantic search off — matching words only'}</span>
		{/if}
	</div>

	{#if query.trim()}
		<!-- ASK moment: results — click holds, Ctrl/middle-click opens -->
		<div class="wb-section-label">
			{searching ? ($t('workbench.searching') || 'Searching…') : `${results.length} · ${$t('workbench.holdHint') || 'click to hold · Ctrl+click to open'}`}
			<button class="wb-clear" onclick={clearSearch}>{$t('workbench.backToDesk') || 'Back to desk'}</button>
		</div>
		<NoteList items={results} rowHeight={NOTE_ROW_HEIGHT} scrollResetKey={query}>
			{#snippet row(r: ConstellationSearchResult)}
				<NoteRow
					name={r.name.replace(/\.md$/, '')}
					meta={`${r.library_name} · ${fmtDate(r.modified)}`}
					selected={heldPaths.has(r.path)}
					onActivate={(e) => holdResult(r, e)}
				>
					{#snippet trailing()}
						{#if heldPaths.has(r.path)}
							<span class="wb-heldmark" title={$t('workbench.alreadyHeld') || 'On the desk'}>✓</span>
						{:else}
							<span class="wb-holdmark">+</span>
						{/if}
					{/snippet}
				</NoteRow>
			{/snippet}
		</NoteList>
	{:else}
		<!-- HOLD moment: the desk -->
		<div class="wb-chips">
			<button class="wb-chip" class:on={chipDue} onclick={() => chipDue = !chipDue}>{$t('workbench.chipDue') || 'due'}</button>
			<button class="wb-chip" class:on={chipUnlinked} onclick={() => chipUnlinked = !chipUnlinked}>{$t('workbench.chipUnlinked') || 'unlinked'}</button>
			<button class="wb-chip" class:on={chipContested} onclick={() => chipContested = !chipContested}>{$t('workbench.chipContested') || 'contested'}</button>
			<button class="wb-chip" class:on={chipForming} onclick={() => chipForming = !chipForming}>{$t('workbench.chipForming') || 'forming'}</button>
			{#if anyChip}
				<span class="wb-chip-note">{visible.length}/{held.length}</span>
			{/if}
		</div>
		{#if held.length === 0}
			<div class="wb-empty">
				<p class="wb-empty-title">{$t('workbench.empty') || 'The desk is clear.'}</p>
				<p class="wb-empty-hint">{$t('workbench.emptyHint') || 'Ask above in your own words, or right-click any note → Add to Workbench.'}</p>
			</div>
		{:else}
			<NoteList items={visible} rowHeight={NOTE_ROW_HEIGHT} scrollResetKey={anyChip}>
				{#snippet row(h: { item: WorkbenchItem; row: HydratedRow | null })}
					<NoteRow
						name={(h.row?.name ?? h.item.path.split(/[\\/]/).pop()?.replace(/\.md$/, '')) || h.item.path}
						meta={h.row ? `${h.row.library_name} · ${fmtDate(h.row.modified)}` : ($t('workbench.missing') || 'missing — the file is gone; set it down when ready')}
						chips={h.row ? chipsFor(h.row) : []}
						done={h.item.done === true}
						missing={!h.row && !hydrating}
						onActivate={(e) => { if (h.row) openRow(h.row.path, h.row.library_name, e); }}
					>
						{#snippet trailing()}
							<button class="wb-act" title={$t('workbench.done') || 'Done'} onclick={() => toggleWorkbenchDone(workbenchKey(h.item))}>
								{h.item.done ? '↩' : '✓'}
							</button>
							<button class="wb-act" title={$t('workbench.setDown') || 'Set down'} onclick={() => removeFromWorkbench(workbenchKey(h.item))}>✕</button>
						{/snippet}
					</NoteRow>
				{/snippet}
			</NoteList>
		{/if}
	{/if}
</div>

<style>
	.wb {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: var(--background-primary);
		font-family: var(--font-interface-theme);
	}
	.wb-header {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 14px 18px 10px;
	}
	.wb-title { font-size: 18px; font-weight: 700; color: var(--text-normal); margin: 0; }
	.wb-count {
		font-size: 11px; color: var(--text-faint);
		background: var(--background-modifier-border);
		padding: 1px 8px; border-radius: 9px;
	}
	.wb-sweep {
		margin-inline-start: auto;
		background: none; border: 1px solid var(--background-modifier-border);
		border-radius: 6px; padding: 3px 10px; font-size: 12px;
		color: var(--text-muted); cursor: pointer;
	}
	.wb-sweep:hover { color: var(--text-normal); background: var(--background-modifier-hover); }
	.wb-close {
		background: none; border: none; cursor: pointer; color: var(--text-muted);
		padding: 4px; border-radius: 4px; display: flex;
	}
	.wb-sweep + .wb-close { margin-inline-start: 0; }
	.wb-header > .wb-close:last-child { margin-inline-start: auto; }
	.wb-sweep ~ .wb-close { margin-inline-start: 8px; }
	.wb-close:hover { color: var(--text-normal); background: var(--background-modifier-hover); }
	.wb-bar { padding: 0 18px 10px; display: flex; flex-direction: column; gap: 4px; }
	.wb-input {
		width: 100%;
		padding: 9px 12px;
		font-size: 14px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		background: var(--background-secondary);
		color: var(--text-normal);
	}
	.wb-input:focus { outline: none; border-color: var(--interactive-accent); }
	.wb-hint { font-size: 11px; color: var(--text-faint); }
	.wb-section-label {
		display: flex; align-items: center; gap: 10px;
		padding: 0 18px 6px; font-size: 11px; color: var(--text-muted);
	}
	.wb-clear {
		margin-inline-start: auto; background: none; border: none;
		color: var(--interactive-accent); cursor: pointer; font-size: 11px;
	}
	.wb-chips { display: flex; align-items: center; gap: 6px; padding: 0 18px 8px; }
	.wb-chip {
		padding: 2px 10px; border-radius: 12px; font-size: 11px; cursor: pointer;
		border: 1px solid var(--background-modifier-border);
		background: var(--background-secondary); color: var(--text-muted);
	}
	.wb-chip.on {
		background: var(--interactive-accent); color: white;
		border-color: var(--interactive-accent);
	}
	.wb-chip-note { font-size: 11px; color: var(--text-faint); }
	.wb-empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 6px; }
	.wb-empty-title { font-size: 15px; font-weight: 600; color: var(--text-muted); margin: 0; }
	.wb-empty-hint { font-size: 12px; color: var(--text-faint); margin: 0; max-width: 420px; text-align: center; }
	.wb-heldmark { color: var(--interactive-accent); font-size: 13px; }
	.wb-holdmark { color: var(--text-faint); font-size: 15px; }
	.wb-act {
		background: none; border: none; cursor: pointer; color: var(--text-muted);
		font-size: 13px; padding: 4px 6px; border-radius: 4px;
	}
	.wb-act:hover { color: var(--text-normal); background: var(--background-modifier-hover); }
</style>
