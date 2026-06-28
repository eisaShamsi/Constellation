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
	type OutlineNode = { path: string; name: string; seq: number | null; children: OutlineNode[]; truncated: boolean; contested: boolean; contested_owner: string | null };
	type Row = { path: string; name: string; depth: number; truncated: boolean; isCurrent: boolean; contested: boolean; contestedOwner: string | null };

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
	// Boss 2026-06-28: the panel shows the WHOLE work by default — rooted at the topmost
	// structural ancestor, with the open note highlighted ("you are here") — so the note is
	// a position WITHIN the work, not the root of a fragment. A 'focus' toggle collapses to
	// just this note's subtree. 'whole' is the default per the ruling.
	let scope = $state<'whole' | 'subtree'>('whole');
	let lastFetched = ''; // key = path|scope — fetch once per (note, scope) (Rule 1/3 — no per-keystroke IPC)

	$effect(() => {
		const path = activeNotePath;
		const name = activeNoteName;
		const sc = scope;
		if (!path || !name) { ancestors = []; outline = []; lastFetched = ''; return; }
		const key = path + '|' + sc;
		if (key === lastFetched) return;
		lastFetched = key;
		loading = true;
		(async () => {
			try {
				// Ancestors give the breadcrumb AND the work's root (the topmost ancestor).
				const anc = (await invoke<SNode[]>('get_structural_ancestors', { notePath: path, noteName: name })) ?? [];
				// Whole-work renders from the root; focus renders from the open note itself.
				const root = (sc === 'whole' && anc.length > 0) ? anc[0] : { path, name };
				const desc = await invoke<OutlineNode[]>('get_structural_descendants', { notePath: root.path, noteName: root.name });
				// Guard against an out-of-order resolve after a fast note/scope switch.
				if (lastFetched === key) { ancestors = anc; outline = desc ?? []; }
			} catch {
				if (lastFetched === key) { ancestors = []; outline = []; }
			} finally {
				if (lastFetched === key) loading = false;
			}
		})();
	});

	// Flatten the descendant tree to indented rows (virtualizable + Rule 3 safe).
	// isCurrent marks the open note ("you are here") within the whole-work tree.
	const rows = $derived.by<Row[]>(() => {
		const cur = activeNotePath;
		const out: Row[] = [];
		const walk = (nodes: OutlineNode[], depth: number) => {
			for (const n of nodes) {
				out.push({ path: n.path, name: n.name, depth, truncated: n.truncated, isCurrent: n.path === cur, contested: n.contested, contestedOwner: n.contested_owner });
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

	// Bleeding-tip fix (Boss 2026-06-28): replace the native `title` (WebView2 renders it
	// as a wide box that bleeds off the panel edge) with a position:fixed tooltip whose x is
	// clamped to the viewport — the HelpTip pattern. Escapes the sidebar's overflow; never
	// bleeds off-screen. Shared by the loop (↻) marker and the contested (⚠) badge.
	let tip = $state<{ x: number; y: number; text: string } | null>(null);
	function showTip(e: MouseEvent, text: string) {
		const el = e.currentTarget as HTMLElement | null;
		if (!el) return;
		const r = el.getBoundingClientRect();
		const halfW = 150; // ~max-width/2 + breathing room
		const margin = 12;
		const vw = typeof window !== 'undefined' ? window.innerWidth : 1920;
		let x = r.left + r.width / 2;
		if (x < halfW + margin) x = halfW + margin;
		if (x > vw - halfW - margin) x = vw - halfW - margin;
		tip = { x, y: r.bottom + 6, text };
	}
	function hideTip() { tip = null; }

	const VTHRESH = 50;
	const ROW_H = 24;
</script>

{#snippet outlineRow(r: Row)}
	<button class="toc-row" class:toc-row-current={r.isCurrent} class:toc-row-contested={r.contested} dir="auto" style="padding-inline-start: {r.depth * 14 + 8}px"
		onclick={(e) => open(r.path, e)}>
		<span class="toc-bullet"></span>
		<!-- No native `title` (it doubled with — and bled past — the marker tooltips). Show the
		     full name in the clamped tooltip ONLY when the name is actually truncated. -->
		<span class="toc-name"
			onmouseenter={(e) => { const el = e.currentTarget as HTMLElement; if (el && el.scrollWidth > el.clientWidth) showTip(e, r.name); }}
			onmouseleave={hideTip}>{r.name}</span>
		{#if r.contested}
			<span class="toc-contested" role="img"
				aria-label={`${$t('panels.structureContested') || 'Contested'}${r.contestedOwner ? ' — ' + r.contestedOwner : ''}`}
				onmouseenter={(e) => showTip(e, `${$t('panels.structureContested') || 'Contested'}${r.contestedOwner ? ' — ' + r.contestedOwner : ''}`)}
				onmouseleave={hideTip}>⚠ {$t('panels.structureContested') || 'Contested'}</span>
		{:else if r.truncated}
			<span class="toc-loop" role="img"
				aria-label={$t('panels.structureLoop') || 'A loop was detected; the outline stops here.'}
				onmouseenter={(e) => showTip(e, $t('panels.structureLoop') || 'A loop was detected; the outline stops here.')}
				onmouseleave={hideTip}>↻</span>
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
			{#if ancestors.length > 0}
				<!-- Boss 2026-06-28: whole-work ↔ this-note's-subtree. Shown only when the note has
				     a parent (so the two scopes differ); a root note already shows its whole tree. -->
				<div class="toc-scope-toggle" role="group" aria-label={$t('panels.structure') || 'Structure'}>
					<button class="toc-scope-btn" class:active={scope === 'whole'} onclick={() => (scope = 'whole')}>{$t('panels.structureWholeWork') || 'Whole work'}</button>
					<button class="toc-scope-btn" class:active={scope === 'subtree'} onclick={() => (scope = 'subtree')}>{$t('panels.structureFocusNote') || 'This note'}</button>
				</div>
			{/if}
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

<!-- Clamped, overflow-escaping tooltip (replaces the bleeding native title). -->
{#if tip}
	<div class="toc-tip" style="left: {tip.x}px; top: {tip.y}px;">{tip.text}</div>
{/if}

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
	/* Whole-work ↔ focus segmented toggle (pushed to the row's end). */
	.toc-scope-toggle {
		display: inline-flex; margin-inline-start: auto;
		border: 1px solid var(--border); border-radius: 4px; overflow: hidden; flex-shrink: 0;
	}
	.toc-scope-btn {
		background: none; border: none; cursor: pointer; font-family: inherit;
		font-size: calc(0.64rem * var(--rs-scale, 1)); padding: 1px 6px;
		color: var(--text-muted); text-transform: none; letter-spacing: 0; white-space: nowrap;
	}
	.toc-scope-btn:hover { background: var(--background-modifier-hover); }
	.toc-scope-btn.active { background: #14B8A6; color: #fff; }
	/* Bounded scroller so VirtualList sees a real clientHeight (the BacklinksPanel
	   .bl-vlist-wrap lesson — without a max-height it silently de-virtualizes). */
	.toc-vlist { display: flex; flex-direction: column; max-height: 60vh; min-height: 0; }
	.toc-row {
		display: flex; align-items: center; gap: 6px; width: 100%;
		padding: 3px 8px; background: none; border: none; cursor: pointer;
		text-align: start; border-radius: 3px; font-family: inherit;
	}
	.toc-row:hover { background: var(--background-modifier-hover); }
	/* "You are here" — the open note, highlighted within the whole-work tree. */
	.toc-row-current { background: color-mix(in srgb, #14B8A6 14%, transparent); }
	.toc-row-current .toc-name { font-weight: 700; color: var(--text-normal); }
	.toc-row-current .toc-bullet { box-shadow: 0 0 0 2px color-mix(in srgb, #14B8A6 40%, transparent); }
	.toc-bullet { width: 5px; height: 5px; border-radius: 50%; background: #14B8A6; flex-shrink: 0; }
	.toc-name {
		color: var(--text-normal); font-size: calc(0.8rem * var(--rs-scale, 1));
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.toc-loop { color: #d97706; flex-shrink: 0; font-size: 0.8rem; cursor: help; }
	.toc-empty { color: var(--color-base-40); font-size: calc(0.78rem * var(--rs-scale, 1)); padding: 4px 8px; }
	/* Contested (overruled `contains:` claim): shown muted + an amber ⚠ badge so the
	   conflict is visible and the user can fix the frontmatter — never silently dropped. */
	.toc-row-contested .toc-name { color: var(--text-muted); font-style: italic; }
	.toc-row-contested .toc-bullet { background: transparent; box-shadow: inset 0 0 0 1px var(--text-faint); }
	.toc-contested {
		flex-shrink: 0; cursor: help; white-space: nowrap;
		color: #d97706; font-size: calc(0.64rem * var(--rs-scale, 1));
		font-weight: 600; text-transform: none; letter-spacing: 0;
	}
	/* The clamped tooltip — position:fixed escapes the sidebar's overflow; x is clamped
	   in showTip() so it never bleeds off-screen (the native-title bug it replaces). */
	.toc-tip {
		position: fixed; transform: translateX(-50%);
		max-width: 280px; padding: 8px 12px;
		background: var(--background-secondary); color: var(--text-normal);
		border: 1px solid var(--background-modifier-border-focus, var(--border));
		border-radius: 8px; line-height: 1.5;
		font-size: calc(0.78rem * var(--rs-scale, 1));
		text-transform: none; letter-spacing: normal; white-space: normal;
		z-index: 9999; pointer-events: none; box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
	}
</style>
