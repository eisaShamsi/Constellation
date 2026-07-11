<script lang="ts">
	/**
	 * PJ-068 v2 — the note-focus cockpit.
	 *
	 * The read-only complement the second screen shows for the open note: the
	 * Normal / Live / Locked coupling dial + the **Note Radial Graph** (the open note
	 * centred, its backlinks radiating left and outgoing links right). Read-only:
	 * clicking a node asks the MAIN window to navigate (onNavigate → sendNoteToMain).
	 *
	 * Link data via the cheap per-note IPCs (get_backlink_rows / get_outgoing_rows —
	 * MIG-079 index seeks, not walks): Rule-8 clean, gen-guarded, path+nonce-keyed
	 * (zero IPC churn on the main window's keystrokes; a same-note save re-reads after
	 * the reindex settles).
	 */
	import { invoke } from '@tauri-apps/api/core';
	import { getBacklinks, getOutgoingLinks, appSettings, type NoteLink } from '$lib/libraries/store';
	import { detectDir } from '$lib/utils';
	import { t } from '$lib/i18n';
	import { onDestroy } from 'svelte';
	import { type DialMode, normalizeGraphStyle, NOTE_GRAPH_STYLES } from '$lib/cockpitFlag';
	import { requestLensChange } from '$lib/secondScreen';
	import NoteRadialGraph from './NoteRadialGraph.svelte';
	import NoteButterflyGraph from './NoteButterflyGraph.svelte';
	import NoteLedgerGraph from './NoteLedgerGraph.svelte';
	import NoteOrreryGraph from './NoteOrreryGraph.svelte';

	interface Focus { path: string; name: string; libraryName: string; libraryPath: string; content?: string; }

	let {
		focus = null,
		dialMode = 'normal' as DialMode,
		onDialChange,
		onNavigate,
		libraryColorMap = {} as Record<string, string>,
		allNotes = [] as { name: string; path: string; libraryName: string }[],
		reloadNonce = 0,
	}: {
		focus?: Focus | null;
		dialMode?: DialMode;
		onDialChange?: (m: DialMode) => void;
		onNavigate?: (path: string, name: string, libraryName: string) => void;
		libraryColorMap?: Record<string, string>;
		allNotes?: { name: string; path: string; libraryName: string }[];
		reloadNonce?: number;
	} = $props();

	/** Resolve an outgoing wikilink target name → its note path via the host's note list. */
	function resolveTarget(targetName: string): { path: string; libraryName: string } {
		const key = (targetName || '').replace(/\.md$/, '').toLowerCase();
		const m = allNotes.find((n) => n.name.replace(/\.md$/, '').toLowerCase() === key);
		return m ? { path: m.path, libraryName: m.libraryName } : { path: '', libraryName: '' };
	}

	// Locked captures the focus at the instant the dial flips to Locked, then freezes it;
	// Normal/Live follow the live focus. (Live's hover source comes online with P3.)
	let pinned = $state<Focus | null>(null);
	let prevDial: DialMode = 'normal';
	$effect(() => {
		if (dialMode === 'locked' && prevDial !== 'locked') pinned = focus;
		prevDial = dialMode;
	});
	let shown = $derived(dialMode === 'locked' ? pinned : focus);

	let backlinks = $state<any[]>([]);
	let outgoing = $state<any[]>([]);
	let review = $state<any | null>(null);
	let loading = $state(false);
	let gen = 0;
	let lastKey = '';
	let lastPath = '';
	let refetchTimer: ReturnType<typeof setTimeout> | undefined;

	function fetchLinks(f: Focus) {
		const myGen = ++gen;
		const name = f.name.replace(/\.md$/, '');
		loading = true;
		(async () => {
			try {
				const [blRows, ogRows] = await Promise.all([
					invoke<NoteLink[]>('get_backlink_rows', { noteName: name, aliases: [] }).catch(() => [] as NoteLink[]),
					invoke<NoteLink[]>('get_outgoing_rows', { notePath: f.path }).catch(() => [] as NoteLink[]),
				]);
				if (myGen !== gen) return;
				backlinks = getBacklinks(blRows, name, undefined, []);
				outgoing = getOutgoingLinks(ogRows, f.path, undefined);
				// Per-note review/health stats (maturity, word count, review status) — cheap IPC.
				invoke('get_note_review_status', { notePath: f.path, staleGraceDays: 1 })
					.then((r) => { if (myGen === gen) review = r; })
					.catch(() => { if (myGen === gen) review = null; });
			} finally {
				if (myGen === gen) loading = false;
			}
		})();
	}

	// Path-guarded so a parent re-render (same note) never refetches — zero IPC churn on
	// keystrokes. Immediate on note-open; ~450ms-delayed on a same-note save/cascade
	// (reloadNonce bump) so the async note_links reindex has settled before re-reading.
	$effect(() => {
		const f = shown;
		const path = f?.path ?? '';
		const key = path + '#' + reloadNonce;
		if (key === lastKey) return;
		const pathChanged = path !== lastPath;
		lastKey = key; lastPath = path;
		clearTimeout(refetchTimer);
		if (!f?.path) { backlinks = []; outgoing = []; return; }
		if (pathChanged) fetchLinks(f);
		else refetchTimer = setTimeout(() => fetchLinks(f), 450);
	});
	onDestroy(() => clearTimeout(refetchTimer));

	const clean = (n: string) => (n || '').replace(/\.md$/, '');
	const dot = (lib: string) => libraryColorMap[lib] || 'var(--interactive-accent, #7c3aed)';

	let DIALS = $derived<{ id: DialMode; label: string; icon: string }[]>([
		{ id: 'normal', label: $t('cockpit.dialNormal') || 'Follow', icon: 'M8 3a5 5 0 100 10A5 5 0 008 3zM8 5a3 3 0 110 6 3 3 0 010-6z' },
		{ id: 'locked', label: $t('cockpit.dialLocked') || 'Pin', icon: 'M4.5 7V5a3.5 3.5 0 117 0v2H12a1 1 0 011 1v5a1 1 0 01-1 1H4a1 1 0 01-1-1V8a1 1 0 011-1h.5zm2 0h3V5a1.5 1.5 0 10-3 0v2z' },
	]);

	// The note's facets (the top tabs). The radial link-graph is the default 'Links' view;
	// the rest are wired in the next increments (contextual — shown per the note's content).
	let activeTab = $state('links');
	// Each facet's label is pulled from the app's EXISTING translation for that panel (all 15
	// languages already have these), so the tab bar localizes with zero new i18n keys and never
	// disagrees with the panel it opens. `facetLabel` falls back to the English name on a miss.
	const FACETS = [
		{ id: 'links', label: 'Links', labelKey: 'settings.sections.links' },
		{ id: 'properties', label: 'Properties', labelKey: 'settings.panels.panelProperties' },
		{ id: 'structure', label: 'Structure', labelKey: 'panels.structure' },
		{ id: 'tags', label: 'Tags', labelKey: 'settings.plugins.tags' },
		{ id: 'skyview', label: 'Sky View', labelKey: 'settings.plugins.graphView' },
		{ id: 'tasks', label: 'Tasks', labelKey: 'settings.panels.panelTasks' },
		{ id: 'health', label: 'Knowledge Health', labelKey: 'ribbon.knowledgeHealth' },
		{ id: 'provenance', label: 'Provenance', labelKey: 'settings.panels.panelProvenance' },
		{ id: 'review', label: 'Review Pulse', labelKey: 'panels.review' },
		{ id: 'sources', label: 'Source Review', labelKey: 'panels.sourceReview' },
	];
	const facetLabel = (f: { label: string; labelKey: string }) => { const v = $t(f.labelKey); return v === f.labelKey ? f.label : v; };

	// The lens toggle lives on this page (Boss ruling 2026-07-10). The SS only *requests* the
	// switch — main owns the settings write and broadcasts it back (Display-not-Domain).
	let lens = $derived(normalizeGraphStyle($appSettings.noteGraphStyle));
	const lensLabel = (s: { labelKey: string; label: string }) => {
		const v = $t(s.labelKey);
		return (v === s.labelKey ? s.label : v).replace(/\s*\(.*\)\s*$/, '');
	};
</script>

<div class="ck">
	<div class="ck-bar">
		<div class="ck-dial" role="tablist" aria-label={$t('cockpit.dial') || 'Coupling'}>
			{#each DIALS as d}
				<button class="ck-seg" class:on={dialMode === d.id} role="tab" aria-selected={dialMode === d.id}
					onclick={() => onDialChange?.(d.id)} title={d.label}>
					<svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor" aria-hidden="true"><path d={d.icon} /></svg>
					<span>{d.label}</span>
				</button>
			{/each}
		</div>
		<div class="ck-anchor" dir={detectDir(shown?.name || '')}>
			{#if shown?.path}
				<span class="ck-dotlib" style="background:{dot(shown.libraryName)}"></span>
				<span class="ck-anchor-name">{clean(shown.name)}</span>
				{#if dialMode === 'locked'}<span class="ck-pinbadge">{$t('cockpit.pinned') || 'pinned'}</span>{/if}
			{:else}
				<span class="ck-anchor-idle">{$t('cockpit.idle') || 'open a note in the main window'}</span>
			{/if}
		</div>
		<span class="ck-ro">{$t('cockpit.readOnly') || 'read-only'}</span>
	</div>

	{#if shown?.path}
		<div class="ck-tabs" role="tablist">
			{#each FACETS as f}
				<button class="ck-tab" class:on={activeTab === f.id} role="tab" aria-selected={activeTab === f.id}
					onclick={() => activeTab = f.id}>{facetLabel(f)}</button>
			{/each}
			{#if activeTab === 'links'}
				<div class="ck-lens" role="group" aria-label="note graph lens">
					{#each NOTE_GRAPH_STYLES.filter((s) => s.built) as s}
						<button class="ck-lensbtn" class:on={lens === s.id}
							onclick={() => requestLensChange(s.id)}
							aria-pressed={lens === s.id}>{lensLabel(s)}</button>
					{/each}
				</div>
			{/if}
		</div>
		<div class="ck-facet">
			{#if activeTab === 'links'}
				{#if lens === 'butterfly'}
					<NoteButterflyGraph noteName={shown.name} content={shown.content ?? ''} {review} {backlinks} {outgoing} {resolveTarget} {onNavigate} />
				{:else if lens === 'ledger'}
					<NoteLedgerGraph noteName={shown.name} content={shown.content ?? ''} {review} {backlinks} {outgoing} {resolveTarget} {onNavigate} />
				{:else if lens === 'orrery'}
					<NoteOrreryGraph noteName={shown.name} content={shown.content ?? ''} {review} {backlinks} {outgoing} {resolveTarget} {onNavigate} />
				{:else}
					<NoteRadialGraph noteName={shown.name} {backlinks} {outgoing} {resolveTarget} {onNavigate} />
				{/if}
			{:else}
				<div class="ck-facet-soon">
					<span class="ck-facet-name">{(() => { const f = FACETS.find((x) => x.id === activeTab); return f ? facetLabel(f) : ''; })()}</span>
					<span>this facet is wired in the next pass</span>
				</div>
			{/if}
		</div>
	{:else}
		<div class="ck-idle">
			<svg viewBox="0 0 24 24" width="40" height="40" fill="none" stroke="currentColor" stroke-width="1.4" opacity="0.35" aria-hidden="true"><circle cx="12" cy="12" r="9" /><path d="M12 7v5l3 2" /></svg>
			<p>{$t('cockpit.idleFull') || 'Open a note in the main window — its links appear here.'}</p>
		</div>
	{/if}
</div>

<style>
	.ck { display: flex; flex-direction: column; height: 100%; min-height: 0; padding: 14px 16px; box-sizing: border-box; color: var(--text-normal, #1a1a1a); font-size: 14px; }
	.ck-bar { display: flex; align-items: center; gap: 14px; flex-wrap: wrap; margin-bottom: 6px; }
	.ck-dial { display: inline-flex; border: 1px solid var(--background-modifier-border, #d4d4d8); border-radius: 8px; overflow: hidden; }
	.ck-seg { display: inline-flex; align-items: center; gap: 6px; padding: 8px 16px; border: none; border-inline-end: 1px solid var(--background-modifier-border, #d4d4d8); background: transparent; color: var(--text-muted, #6b7280); font-size: 13px; cursor: pointer; }
	.ck-seg:last-child { border-inline-end: none; }
	.ck-seg:hover { background: var(--background-modifier-hover, rgba(0,0,0,0.04)); }
	.ck-seg.on { background: color-mix(in srgb, var(--interactive-accent, #7c3aed) 16%, transparent); color: var(--interactive-accent, #7c3aed); }
	.ck-anchor { display: flex; align-items: center; gap: 8px; min-width: 0; flex: 1; }
	.ck-anchor-name { font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
	.ck-anchor-idle { color: var(--text-faint, #9ca3af); font-size: 13px; }
	.ck-pinbadge { font-size: 11px; color: var(--interactive-accent, #7c3aed); background: color-mix(in srgb, var(--interactive-accent, #7c3aed) 14%, transparent); border-radius: 5px; padding: 1px 6px; }
	.ck-ro { font-size: 11px; color: var(--text-faint, #9ca3af); border: 1px solid var(--background-modifier-border, #d4d4d8); border-radius: 5px; padding: 2px 8px; }
	.ck-dotlib { width: 8px; height: 8px; border-radius: 50%; display: inline-block; flex: none; }
	.ck-tabs { display: flex; align-items: center; gap: 2px; flex-wrap: wrap; border-bottom: 1px solid var(--background-modifier-border, #d4d4d8); padding-bottom: 6px; margin-bottom: 6px; }
	.ck-lens { margin-inline-start: auto; display: flex; gap: 2px; padding: 2px; border-radius: 8px;
		background: var(--background-secondary, #f4f4f5); border: 1px solid var(--background-modifier-border, #e2e2e2); }
	.ck-lensbtn { border: none; background: transparent; padding: 4px 11px; border-radius: 6px; font-size: 12px;
		color: var(--text-muted, #6b7280); cursor: pointer; white-space: nowrap; }
	.ck-lensbtn:hover { color: var(--text-normal, #1a1a1a); }
	.ck-lensbtn.on { background: var(--background-primary, #fff); color: var(--text-normal, #1a1a1a); font-weight: 600;
		box-shadow: 0 1px 2px rgba(0,0,0,0.08); }
	.ck-tab { border: none; background: transparent; padding: 6px 12px; border-radius: 7px; font-size: 12.5px; color: var(--text-muted, #6b7280); cursor: pointer; }
	.ck-tab:hover { background: var(--background-modifier-hover, rgba(0,0,0,0.04)); }
	.ck-tab.on { color: var(--interactive-accent, #7c3aed); background: color-mix(in srgb, var(--interactive-accent, #7c3aed) 12%, transparent); font-weight: 500; }
	.ck-facet { flex: 1; min-height: 0; }
	.ck-facet-soon { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 6px; color: var(--text-faint, #9ca3af); font-size: 13px; }
	.ck-facet-name { font-size: 15px; font-weight: 500; color: var(--text-muted, #6b7280); }
	.ck-idle { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; color: var(--text-faint, #9ca3af); }
	.ck-idle p { font-size: 13px; }
</style>
