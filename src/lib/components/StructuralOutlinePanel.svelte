<script lang="ts">
	// PJ-065 §7 — the Structure (parent / table-of-contents) panel. Mirrors
	// BacklinksPanel's grammar (VirtualList ≥50 rows, teal structural accent), but
	// renders the note's STRUCTURAL spine: an ancestor breadcrumb + the descendant
	// outline. Self-fetching via the §6 read APIs; reads ONLY on note change while the
	// tab is mounted (gesture-gated — the panel mounts only when the Structure tab is
	// active), never on note open, never writes content (Editor-Surface Gate).
	import { invoke } from '@tauri-apps/api/core';
	import { openNoteTab, libraries } from '$lib/libraries/store';
	import { t, dir as uiDir } from '$lib/i18n';
	import VirtualList from './VirtualList.svelte';

	type SNode = { path: string; name: string; seq: number | null };
	type OutlineNode = { path: string; name: string; seq: number | null; children: OutlineNode[]; truncated: boolean };
	type Row = { path: string; name: string; depth: number; truncated: boolean };

	let {
		activeNoteName = '',
		activeNotePath = '',
		libraryColorMap = {} as Record<string, string>,
	}: {
		activeNoteName?: string;
		activeNotePath?: string;
		libraryColorMap?: Record<string, string>;
	} = $props();

	let ancestors = $state<SNode[]>([]);
	let outline = $state<OutlineNode[]>([]);
	let loading = $state(false);
	let lastFetched = ''; // fetch once per note (Rule 1/3 — no per-keystroke IPC)

	$effect(() => {
		const path = activeNotePath;
		const name = activeNoteName;
		if (!path || !name) { ancestors = []; outline = []; lastFetched = ''; return; }
		if (path === lastFetched) return;
		lastFetched = path;
		loading = true;
		(async () => {
			try {
				const [anc, desc] = await Promise.all([
					invoke<SNode[]>('get_structural_ancestors', { notePath: path, noteName: name }),
					invoke<OutlineNode[]>('get_structural_descendants', { notePath: path, noteName: name }),
				]);
				// Guard against an out-of-order resolve after a fast note switch.
				if (lastFetched === path) { ancestors = anc ?? []; outline = desc ?? []; }
			} catch {
				if (lastFetched === path) { ancestors = []; outline = []; }
			} finally {
				if (lastFetched === path) loading = false;
			}
		})();
	});

	// Flatten the descendant tree to indented rows (virtualizable + Rule 3 safe).
	const rows = $derived.by<Row[]>(() => {
		const out: Row[] = [];
		const walk = (nodes: OutlineNode[], depth: number) => {
			for (const n of nodes) {
				out.push({ path: n.path, name: n.name, depth, truncated: n.truncated });
				if (n.children.length) walk(n.children, depth + 1);
			}
		};
		walk(outline, 0);
		return out;
	});

	function libNameFor(path: string): string {
		const libs = $libraries ?? [];
		const hit = libs.find((l) => {
			const lp = l.path.replace(/[/\\]+$/, '');
			return path === lp || path.startsWith(lp + '/') || path.startsWith(lp + '\\');
		});
		return hit?.name ?? '';
	}
	async function open(path: string, e?: MouseEvent) {
		const libName = libNameFor(path);
		const color = libraryColorMap[libName] ?? '#14B8A6';
		const newTab = e ? (e.ctrlKey || e.metaKey || e.button === 1) : false;
		await openNoteTab(path, libName, color, undefined, newTab, activeNotePath || undefined);
	}

	const VTHRESH = 50;
	const ROW_H = 24;
</script>

{#snippet outlineRow(r: Row)}
	<button class="toc-row" dir="auto" style="padding-inline-start: {r.depth * 14 + 8}px"
		onclick={(e) => open(r.path, e)} title={r.name}>
		<span class="toc-bullet"></span>
		<span class="toc-name">{r.name}</span>
		{#if r.truncated}
			<span class="toc-loop" title={$t('panels.structureLoop') || 'A loop was detected here; the outline stops cleanly.'}>↻</span>
		{/if}
	</button>
{/snippet}

<div class="toc-panel" dir={$uiDir}>
	{#if ancestors.length > 0}
		<div class="toc-breadcrumb" dir="auto">
			{#each ancestors as a (a.path)}
				<button class="toc-crumb" onclick={(e) => open(a.path, e)}>{a.name}</button>
				<span class="toc-sep">›</span>
			{/each}
			<span class="toc-crumb toc-crumb-current">{activeNoteName}</span>
		</div>
	{/if}

	<div class="toc-section">
		<div class="toc-header">
			{$t('panels.structureChildren') || 'Outline'}
			<span class="toc-count">{rows.length}</span>
		</div>
		{#if loading}
			<div class="toc-empty">{$t('common.loading') || 'Loading…'}</div>
		{:else if rows.length === 0}
			<div class="toc-empty">{$t('panels.structureEmpty') || 'No structural children. Add a parent: or contains: link in the note frontmatter.'}</div>
		{:else if rows.length > VTHRESH}
			<div class="toc-vlist">
				<VirtualList items={rows} getItemHeight={() => ROW_H} overscan={10}>
					{#snippet row(r, _i)}{@render outlineRow(r as Row)}{/snippet}
				</VirtualList>
			</div>
		{:else}
			{#each rows as r (r.path + ':' + r.depth)}{@render outlineRow(r)}{/each}
		{/if}
	</div>
</div>

<style>
	.toc-panel { font-size: calc(0.8rem * var(--rs-scale, 1)); }
	.toc-breadcrumb {
		display: flex; flex-wrap: wrap; align-items: center; gap: 1px;
		padding: 4px 8px 6px; border-bottom: 1px solid var(--border); margin-bottom: 4px;
	}
	.toc-crumb {
		background: none; border: none; cursor: pointer; color: #14B8A6;
		font-family: inherit; font-size: calc(0.72rem * var(--rs-scale, 1)); padding: 0 2px;
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 140px;
	}
	.toc-crumb:hover { text-decoration: underline; }
	.toc-crumb-current { color: var(--text-muted); cursor: default; font-weight: 600; }
	.toc-crumb-current:hover { text-decoration: none; }
	.toc-sep { color: var(--text-faint); font-size: 0.72rem; }
	.toc-header {
		display: flex; align-items: center; gap: 6px; padding: 4px 8px;
		font-weight: 600; color: var(--text-muted); font-size: calc(0.75rem * var(--rs-scale, 1));
		text-transform: uppercase; letter-spacing: 0.03em;
	}
	.toc-count {
		color: #14B8A6; font-variant-numeric: tabular-nums;
		font-size: calc(0.72rem * var(--rs-scale, 1));
	}
	/* Bounded scroller so VirtualList sees a real clientHeight (the BacklinksPanel
	   .bl-vlist-wrap lesson — without a max-height it silently de-virtualizes). */
	.toc-vlist { display: flex; flex-direction: column; max-height: 60vh; min-height: 0; }
	.toc-row {
		display: flex; align-items: center; gap: 6px; width: 100%;
		padding: 3px 8px; background: none; border: none; cursor: pointer;
		text-align: start; border-radius: 3px; font-family: inherit;
	}
	.toc-row:hover { background: var(--background-modifier-hover); }
	.toc-bullet { width: 5px; height: 5px; border-radius: 50%; background: #14B8A6; flex-shrink: 0; }
	.toc-name {
		color: var(--text-normal); font-size: calc(0.8rem * var(--rs-scale, 1));
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.toc-loop { color: #d97706; flex-shrink: 0; font-size: 0.8rem; }
	.toc-empty { color: var(--color-base-40); font-size: calc(0.78rem * var(--rs-scale, 1)); padding: 4px 8px; }
</style>
