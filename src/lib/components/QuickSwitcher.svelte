<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { detectDir } from '$lib/utils';
	import { foldForMatch } from '$lib/searchFold';
	import { rankSwitcher, hasExactMatch, type SwitcherCandidate } from '$lib/switcherRank';
	import { readRecentOpened, readRecentEdited } from '$lib/libraries/recentNotes';

	// ── MIG-093 §C — the pure title jumper ──
	// Concept (the horse): Ctrl+O answers "take me to the note I can name" —
	// titles + aliases only, ranked, in-memory, ZERO IPC per keystroke. The
	// embedded content search (MIG-058/059's constellationSearch round-trip —
	// the Boss-reproduced 20-27s per keystroke-pause pathology) is DELETED;
	// content search is the Search Hub's owned question, reachable via the
	// pinned "Search in Search Hub" escape row. Ranking is the researched
	// banded model (exact > prefix > word-boundary > fuzzy) — switcherRank.ts,
	// pinned by tests/mig-093.

	let {
		notes = [] as { name: string; path: string; libraryName: string }[],
		aliases = new Map<string, string[]>(),
		onSelect,
		onClose,
		onCreateNote = (_name: string) => {},
		onOpenSearchHub = (_query: string) => {},
	}: {
		notes: { name: string; path: string; libraryName: string }[];
		/** path → alias list (the graph snapshot's notePathToAliases). */
		aliases?: Map<string, string[]>;
		onSelect: (path: string, libraryName: string) => void;
		onClose: () => void;
		onCreateNote?: (name: string) => void;
		onOpenSearchHub?: (query: string) => void;
	} = $props();

	let query = $state('');
	let selectedIndex = $state(0);
	let inputEl: HTMLInputElement;
	let searchTimer: ReturnType<typeof setTimeout>;

	// MIG-058 lesson kept (LL: dropped Arabic keystrokes under per-keystroke
	// synchronous rebuild): results stay a debounced $state, NOT a $derived —
	// but the window drops 300ms → 100ms since the per-keystroke work is now
	// a pure in-memory rank (~ms), no IPC. The composing guard stays for IME
	// scripts (CJK etc.).
	let composing = $state(false);

	type Row =
		| { kind: 'note'; name: string; path: string; libraryName: string; aliasLabel?: string; recent?: boolean }
		| { kind: 'create'; name: string }
		| { kind: 'searchhub'; query: string };
	let rows = $state<Row[]>([]);
	/** QS-speed instrumentation (TEMPORARY, removed at §E): last rank ms. */
	let diag = $state('');

	// Fold ONCE per cache refresh (never per keystroke): title candidates +
	// one candidate per alias. ~8k folds in a few ms, re-run only when the
	// notes prop (allNotes) or the alias map changes.
	const candidates = $derived.by<SwitcherCandidate[]>(() => {
		const out: SwitcherCandidate[] = [];
		const byPath = new Map<string, { name: string; path: string; libraryName: string }>();
		for (const n of notes) {
			byPath.set(n.path, n);
			out.push({ name: n.name, path: n.path, libraryName: n.libraryName, folded: foldForMatch(n.name) });
		}
		for (const [path, list] of aliases) {
			const base = byPath.get(path);
			if (!base) continue;
			for (const a of list) {
				out.push({
					name: base.name,
					path,
					libraryName: base.libraryName,
					folded: foldForMatch(base.name),
					alias: a,
					aliasFolded: foldForMatch(a),
				});
			}
		}
		return out;
	});

	// Recency: position in the recent-opened/edited lists (0 = most recent).
	let recencyIndex = new Map<string, number>();
	let recentRows: Row[] = [];
	onMount(() => {
		inputEl?.focus();
		const opened = readRecentOpened();
		const edited = readRecentEdited();
		const merged: { name: string; path: string; libraryName: string }[] = [];
		const seen = new Set<string>();
		for (const r of [...opened, ...edited]) {
			if (seen.has(r.path)) continue;
			seen.add(r.path);
			merged.push({ name: r.name, path: r.path, libraryName: r.libraryName });
		}
		recencyIndex = new Map(merged.map((r, i) => [r.path, i]));
		// Empty-query state: the recent notes (renamed notes resolve their
		// current title through the cache; deleted ones drop out).
		const byPath = new Map(notes.map(n => [n.path, n]));
		recentRows = merged
			.slice(0, 15)
			.map(r => byPath.get(r.path) ?? null)
			.filter((n): n is { name: string; path: string; libraryName: string } => n !== null)
			.map(n => ({ kind: 'note' as const, name: n.name, path: n.path, libraryName: n.libraryName, recent: true }));
		rows = recentRows;
	});

	$effect(() => {
		const q = query;
		clearTimeout(searchTimer);
		if (composing) return;
		if (!q.trim()) {
			rows = recentRows;
			selectedIndex = 0;
			diag = '';
			return;
		}
		searchTimer = setTimeout(() => {
			const t0 = performance.now();
			const hits = rankSwitcher(q, candidates, { recencyIndex, limit: 50 });
			const next: Row[] = hits.map(h => ({
				kind: 'note' as const,
				name: h.candidate.name,
				path: h.candidate.path,
				libraryName: h.candidate.libraryName,
				aliasLabel: h.candidate.alias,
			}));
			// Pinned rows: create (unless an exact title/alias match exists) +
			// the Search-Hub escape hatch (always — content is its job).
			if (!hasExactMatch(q, candidates)) next.push({ kind: 'create', name: q.trim() });
			next.push({ kind: 'searchhub', query: q.trim() });
			rows = next;
			selectedIndex = 0;
			diag = `rank ${(performance.now() - t0).toFixed(1)}ms · ${hits.length} hits · ${candidates.length} candidates`;
		}, 100);
	});

	function activate(row: Row) {
		if (row.kind === 'note') {
			onSelect(row.path, row.libraryName);
			onClose();
		} else if (row.kind === 'create') {
			onCreateNote(row.name);
			onClose();
		} else {
			onOpenSearchHub(row.query);
			onClose();
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			onClose();
		} else if (e.key === 'ArrowDown') {
			e.preventDefault();
			selectedIndex = Math.min(selectedIndex + 1, rows.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			selectedIndex = Math.max(selectedIndex - 1, 0);
		} else if (e.key === 'Enter') {
			e.preventDefault();
			if (rows[selectedIndex]) activate(rows[selectedIndex]);
		}
	}

	/** Locale lookup with an English fallback until the §E i18n pass. */
	const L = (key: string, fb: string): string => {
		const v = $t(key);
		return v === key ? fb : v;
	};
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="qs-overlay" onclick={onClose} onkeydown={handleKeydown}>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="qs-panel" onclick={(e) => e.stopPropagation()}>
		<input
			bind:this={inputEl}
			type="text"
			class="qs-input"
			dir="auto"
			placeholder={$t('quickSwitcher.placeholder')}
			bind:value={query}
			onkeydown={handleKeydown}
			oncompositionstart={() => composing = true}
			oncompositionend={() => composing = false}
		/>
		<div class="qs-list">
			{#if !query.trim() && rows.length > 0}
				<div class="qs-section">{L('quickSwitcher.recent', 'Recently opened')}</div>
			{/if}
			{#each rows as row, i (row.kind === 'note' ? row.path + (row.aliasLabel ?? '') : row.kind)}
				{#if row.kind === 'note'}
					<button
						class="qs-item"
						class:selected={i === selectedIndex}
						dir={detectDir(row.aliasLabel ?? row.name)}
						onclick={() => activate(row)}
						onmouseenter={() => selectedIndex = i}
					>
						<span class="qs-name">
							{#if row.aliasLabel}<span class="qs-alias">{row.aliasLabel}</span> <span class="qs-alias-arrow">→</span> {/if}{row.name}
						</span>
						<span class="qs-path">{row.libraryName}</span>
					</button>
				{:else if row.kind === 'create'}
					<button class="qs-item qs-pinned" class:selected={i === selectedIndex} dir="auto"
						onclick={() => activate(row)} onmouseenter={() => selectedIndex = i}>
						<span class="qs-name">＋ {L('quickSwitcher.createNote', 'Create note')} “{row.name}”</span>
					</button>
				{:else}
					<button class="qs-item qs-pinned" class:selected={i === selectedIndex} dir="auto"
						onclick={() => activate(row)} onmouseenter={() => selectedIndex = i}>
						<span class="qs-name">🔍 {L('quickSwitcher.searchInHub', 'Search in Search Hub')} “{row.query}”</span>
					</button>
				{/if}
			{/each}
			{#if rows.length === 0 && query}
				<div class="qs-empty">{$t('quickSwitcher.noResults')}</div>
			{/if}
		</div>
		{#if diag}
			<!-- QS-speed instrumentation (TEMPORARY, §E removes) -->
			<div class="qs-diag" dir="ltr">{diag}</div>
		{/if}
	</div>
</div>

<style>
	.qs-overlay {
		position: fixed; inset: 0; z-index: 1000;
		background: var(--background-modifier-cover);
		display: flex; justify-content: center; padding-top: 15vh;
	}
	.qs-panel {
		background: var(--background-primary); border-radius: 8px;
		box-shadow: var(--shadow-l);
		width: 500px; max-height: 420px;
		display: flex; flex-direction: column;
		overflow: hidden;
	}
	.qs-input {
		border: none; padding: 12px 16px;
		font-size: 0.95rem; font-family: inherit;
		color: var(--text-normal); outline: none;
		background: var(--background-primary);
		border-bottom: 1px solid var(--background-modifier-border-focus);
	}
	.qs-input::placeholder { color: var(--color-base-40); }
	.qs-list { flex: 1; overflow-y: auto; padding: 4px; }
	.qs-section {
		padding: 4px 12px 2px; font-size: 0.68rem; text-transform: uppercase;
		letter-spacing: 0.04em; color: var(--text-faint);
	}
	.qs-item {
		display: flex; align-items: center; justify-content: space-between; gap: 8px;
		width: 100%; padding: 6px 12px;
		background: none; border: none; border-radius: 4px;
		cursor: pointer; font-family: inherit; text-align: start;
		color: var(--text-normal); font-size: 0.85rem;
	}
	.qs-item.selected { background: var(--interactive-accent); color: var(--text-on-accent); }
	.qs-name {
		font-weight: 500; min-width: 0;
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.qs-alias { font-style: italic; }
	.qs-alias-arrow { color: var(--text-faint); }
	.qs-item.selected .qs-alias-arrow { color: rgba(255,255,255,0.7); }
	.qs-path { font-size: 0.72rem; color: var(--text-faint); flex-shrink: 0; }
	.qs-item.selected .qs-path { color: rgba(255,255,255,0.7); }
	.qs-pinned {
		border-top: 1px solid var(--background-modifier-border);
		border-radius: 0; margin-top: 2px; color: var(--text-muted);
	}
	.qs-pinned.selected { color: var(--text-on-accent); }
	.qs-empty { padding: 16px; text-align: center; color: var(--text-faint); font-size: 0.85rem; }
	/* QS-speed instrumentation (TEMPORARY) */
	.qs-diag {
		padding: 4px 12px; font-size: 0.68rem; font-family: var(--font-monospace, monospace);
		color: var(--text-faint); border-top: 1px solid var(--background-modifier-border);
	}
</style>
