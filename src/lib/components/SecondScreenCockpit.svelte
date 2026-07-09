<script lang="ts">
	/**
	 * PJ-068 v2 — The Knowledge Cockpit (P1).
	 *
	 * The read-only three-zone complement the second screen shows for a focused note:
	 *   ① Estimation Map   — where the note sits + the universe's shape (a first cut of
	 *                        the holistic past/present/future map — full design at P4).
	 *   ② Control Dashboard — the health of the note's living links + review status.
	 *   ③ Operation Map     — the decision-space: backlinks, outgoing typed links,
	 *                        unlinked mentions. The one action is click-to-navigate.
	 * Plus the Normal / Live / Locked coupling dial. Read-only always: it never writes;
	 * clicking an item asks the MAIN window to navigate (onNavigate → sendNoteToMain).
	 *
	 * All reads are cheap per-note lookups (get_backlink_rows / get_outgoing_rows —
	 * MIG-079 index seeks, not walks; get_note_review_status; scan_unlinked_mentions),
	 * debounced + generation-guarded so a rapid focus change never renders stale data.
	 */
	import { invoke } from '@tauri-apps/api/core';
	import {
		getBacklinks, getOutgoingLinks, scanUnlinkedMentions,
		type NoteLink,
	} from '$lib/libraries/store';
	import { detectDir } from '$lib/utils';
	import { t } from '$lib/i18n';
	import type { DialMode } from '$lib/cockpitFlag';

	interface Focus { path: string; name: string; libraryName: string; libraryPath: string; content?: string; }

	let {
		focus = null,
		dialMode = 'normal' as DialMode,
		onDialChange,
		onNavigate,
		libraryColorMap = {} as Record<string, string>,
		universeNotes = 0,
		allNotes = [] as { name: string; path: string; libraryName: string }[],
		reloadNonce = 0,
	}: {
		focus?: Focus | null;
		dialMode?: DialMode;
		onDialChange?: (m: DialMode) => void;
		onNavigate?: (path: string, name: string, libraryName: string) => void;
		libraryColorMap?: Record<string, string>;
		universeNotes?: number;
		allNotes?: { name: string; path: string; libraryName: string }[];
		/** Bumped by the host when the shown note is saved/cascade-rewritten → forces a re-read. */
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
	let prevDial: DialMode = dialMode;
	$effect(() => {
		if (dialMode === 'locked' && prevDial !== 'locked') pinned = focus;
		prevDial = dialMode;
	});
	let shown = $derived(dialMode === 'locked' ? pinned : focus);

	let backlinks = $state<any[]>([]);
	let outgoing = $state<any[]>([]);
	let unlinked = $state<any[]>([]);
	let review = $state<any | null>(null);
	let loading = $state(false);
	let gen = 0;
	let lastKey = '';

	// Fetch the shown note's zones. Guarded on the note PATH so a parent re-render with a
	// fresh focus object (same note) never refetches — zero IPC churn on the main window's edits.
	$effect(() => {
		const f = shown;
		const key = (f?.path ?? '') + '#' + reloadNonce;
		if (key === lastKey) return;
		lastKey = key;
		if (!f?.path) { backlinks = []; outgoing = []; unlinked = []; review = null; return; }
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
				invoke('get_note_review_status', { notePath: f.path, staleGraceDays: 1 })
					.then((r) => { if (myGen === gen) review = r; })
					.catch(() => { if (myGen === gen) review = null; });
				scanUnlinkedMentions(name, f.path)
					.then((u) => { if (myGen === gen) unlinked = u; })
					.catch(() => { if (myGen === gen) unlinked = []; });
			} finally {
				if (myGen === gen) loading = false;
			}
		})();
	});

	// ── Control Dashboard aggregates ──
	const CONF_ORDER = ['hypothesis', 'evidence', 'established', 'contested'];
	let allLinks = $derived([...backlinks, ...outgoing]);
	let contestedCount = $derived(allLinks.filter((l) => l.confidence === 'contested').length);
	let staleCount = $derived(allLinks.filter((l) => l.tier === 'stale').length);
	let loadBearingCount = $derived(allLinks.filter((l) => l.tier === 'load-bearing').length);
	let dominantConfidence = $derived.by(() => {
		if (!allLinks.length) return '—';
		const avg = allLinks.reduce((s, l) => s + Math.max(0, CONF_ORDER.indexOf(l.confidence)), 0) / allLinks.length;
		return CONF_ORDER[Math.round(avg)] ?? 'hypothesis';
	});

	// review status → a single human line + a severity
	let reviewLine = $derived.by(() => {
		if (!review) return { text: '—', sev: 'muted' };
		if (review.is_stale) return { text: $t('cockpit.reviewStale') || 'stale — a linked note changed', sev: 'warning' };
		if (review.never_reviewed) return { text: $t('cockpit.reviewNever') || 'never reviewed', sev: 'muted' };
		if ((review.days_overdue ?? 0) > 0) return { text: `${$t('cockpit.reviewOverdue') || 'review due'} · ${review.days_overdue}d`, sev: 'warning' };
		return { text: $t('cockpit.reviewOk') || 'up to date', sev: 'success' };
	});

	function nav(path: string, name: string, libraryName: string) {
		if (path) onNavigate?.(path, name, libraryName);
	}
	const clean = (n: string) => (n || '').replace(/\.md$/, '');
	const dot = (lib: string) => libraryColorMap[lib] || 'var(--interactive-accent, #7c3aed)';

	const DIALS: { id: DialMode; label: string; icon: string }[] = [
		{ id: 'normal', label: $t('cockpit.dialNormal') || 'Follow', icon: 'M8 3a5 5 0 100 10A5 5 0 008 3zM8 5a3 3 0 110 6 3 3 0 010-6z' },
		{ id: 'live', label: $t('cockpit.dialLive') || 'Peek', icon: 'M8 3C4 3 1.5 8 1.5 8S4 13 8 13s6.5-5 6.5-5S12 3 8 3zm0 8a3 3 0 110-6 3 3 0 010 6z' },
		{ id: 'locked', label: $t('cockpit.dialLocked') || 'Pin', icon: 'M4.5 7V5a3.5 3.5 0 117 0v2H12a1 1 0 011 1v5a1 1 0 01-1 1H4a1 1 0 01-1-1V8a1 1 0 011-1h.5zm2 0h3V5a1.5 1.5 0 10-3 0v2z' },
	];
</script>

<div class="ck">
	<!-- ── coupling dial + anchor (uses the full width; never squeezed) ── -->
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
		<span class="ck-ro" title={$t('cockpit.readOnly') || 'read-only'}>{$t('cockpit.readOnly') || 'read-only'}</span>
	</div>

	{#if shown?.path}
	<!-- ── ① Estimation Map ── -->
	<section class="ck-zone ck-estimation">
		<div class="ck-zlabel">{$t('cockpit.estimationMap') || 'Estimation map'} <span class="ck-zhint">{$t('cockpit.estimationHint') || 'where this sits in the whole universe'}</span></div>
		<div class="ck-est-body">
			<div class="ck-est-cell"><div class="ck-est-num">{universeNotes.toLocaleString()}</div><div class="ck-est-cap">{$t('cockpit.notes') || 'notes'}</div></div>
			<div class="ck-est-cell"><div class="ck-est-num">{backlinks.length + outgoing.length}</div><div class="ck-est-cap">{$t('cockpit.connections') || 'connections'}</div></div>
			<div class="ck-est-cell"><div class="ck-est-num" dir={detectDir(shown.libraryName)}><span class="ck-dotlib" style="background:{dot(shown.libraryName)}"></span> {shown.libraryName}</div><div class="ck-est-cap">{$t('cockpit.library') || 'library'}</div></div>
			{#if review?.maturity}<div class="ck-est-cell"><div class="ck-est-num ck-cap">{review.maturity}</div><div class="ck-est-cap">{$t('cockpit.maturity') || 'maturity'}</div></div>{/if}
		</div>
		<div class="ck-est-note">{$t('cockpit.holisticPending') || 'The holistic past · present · future map arrives with its dedicated design pass.'}</div>
	</section>

	<div class="ck-cols">
		<!-- ── ② Control Dashboard ── -->
		<section class="ck-zone ck-control">
			<div class="ck-zlabel">{$t('cockpit.controlDashboard') || 'Control dashboard'} <span class="ck-zhint">{$t('cockpit.controlHint') || 'health & what needs attention'}</span></div>
			<div class="ck-rows">
				<div class="ck-row"><span class="ck-dotc s-ok"></span><span class="ck-rk">{$t('cockpit.confidence') || 'confidence'}</span><span class="ck-rv ck-cap">{dominantConfidence === '—' ? '—' : ($t('confidence.' + dominantConfidence) || dominantConfidence)}</span></div>
				{#if contestedCount > 0}<div class="ck-row"><span class="ck-dotc s-bad"></span><span class="ck-rk">{$t('cockpit.contested') || 'contested links'}</span><span class="ck-rv s-badtext">{contestedCount}</span></div>{/if}
				{#if staleCount > 0}<div class="ck-row"><span class="ck-dotc s-warn"></span><span class="ck-rk">{$t('cockpit.dormant') || 'dormant · decaying'}</span><span class="ck-rv s-warntext">{staleCount}</span></div>{/if}
				{#if loadBearingCount > 0}<div class="ck-row"><span class="ck-dotc s-acc"></span><span class="ck-rk">{$t('cockpit.loadBearing') || 'load-bearing'}</span><span class="ck-rv">{loadBearingCount}</span></div>{/if}
				<div class="ck-row"><span class="ck-dotc s-{reviewLine.sev}"></span><span class="ck-rk">{$t('cockpit.review') || 'review'}</span><span class="ck-rv s-{reviewLine.sev}text">{reviewLine.text}</span></div>
				{#if allLinks.length === 0 && !loading}<div class="ck-empty">{$t('cockpit.noLinks') || 'no living links yet'}</div>{/if}
			</div>
		</section>

		<!-- ── ③ Operation Map ── -->
		<section class="ck-zone ck-operation">
			<div class="ck-zlabel">{$t('cockpit.operationMap') || 'Operation map'} <span class="ck-zhint">{$t('cockpit.operationHint') || 'where you could go next'}</span></div>
			<div class="ck-op">
				{#if outgoing.length}
					<div class="ck-op-group">{$t('cockpit.outgoing') || 'outgoing'}</div>
					{#each outgoing.slice(0, 12) as l}
						{@const target = resolveTarget(l.target)}
						<button class="ck-link" onclick={() => nav(target.path, l.target, l.libraryName)} disabled={!target.path} dir={detectDir(l.target)}>
							{#if l.linkType}<span class="ck-type">{l.linkType}</span>{/if}
							<span class="ck-lname">{clean(l.target)}</span>
							{#if target.path}<svg class="ck-go" viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.6" aria-hidden="true"><path d="M6 3l5 5-5 5" /></svg>{/if}
						</button>
					{/each}
				{/if}
				{#if backlinks.length}
					<div class="ck-op-group">{$t('cockpit.backlinks') || 'backlinks'}</div>
					{#each backlinks.slice(0, 12) as l}
						<button class="ck-link" onclick={() => nav(l.path, l.name, l.libraryName)} dir={detectDir(l.name)}>
							{#if l.linkType}<span class="ck-type">{l.linkType}</span>{/if}
							<span class="ck-lname">{clean(l.name)}</span>
							<svg class="ck-go" viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.6" aria-hidden="true"><path d="M6 3l5 5-5 5" /></svg>
						</button>
					{/each}
				{/if}
				{#if unlinked.length}
					<div class="ck-op-group">{$t('cockpit.unlinkedMentions') || 'unlinked mentions'} <span class="ck-cand">{$t('cockpit.candidate') || 'candidate'}</span></div>
					{#each unlinked.slice(0, 8) as m}
						<button class="ck-link ck-link-cand" onclick={() => nav(m.path, m.name, m.libraryName)} dir={detectDir(m.name)}>
							<span class="ck-lname">{clean(m.name)}</span>
							<svg class="ck-go" viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.6" aria-hidden="true"><path d="M6 3l5 5-5 5" /></svg>
						</button>
					{/each}
				{/if}
				{#if !outgoing.length && !backlinks.length && !unlinked.length && !loading}
					<div class="ck-empty">{$t('cockpit.noConnections') || 'no connections yet — this note stands alone'}</div>
				{/if}
			</div>
		</section>
	</div>
	{:else}
		<div class="ck-idle">
			<svg viewBox="0 0 24 24" width="40" height="40" fill="none" stroke="currentColor" stroke-width="1.4" opacity="0.35" aria-hidden="true"><circle cx="12" cy="12" r="9" /><path d="M12 7v5l3 2" /></svg>
			<p>{$t('cockpit.idleFull') || 'Open a note in the main window — its context appears here.'}</p>
		</div>
	{/if}
</div>

<style>
	.ck {
		display: flex; flex-direction: column; gap: 12px;
		height: 100%; min-height: 0; padding: 14px 16px; box-sizing: border-box;
		color: var(--text-normal, #1a1a1a); font-size: 14px;
	}
	/* dial bar — spans the width, generous target sizes (fill the space) */
	.ck-bar { display: flex; align-items: center; gap: 14px; flex-wrap: wrap; }
	.ck-dial { display: inline-flex; border: 1px solid var(--background-modifier-border, #d4d4d8); border-radius: 8px; overflow: hidden; }
	.ck-seg {
		display: inline-flex; align-items: center; gap: 6px; padding: 8px 16px;
		border: none; border-inline-end: 1px solid var(--background-modifier-border, #d4d4d8);
		background: transparent; color: var(--text-muted, #6b7280); font-size: 13px; cursor: pointer;
	}
	.ck-seg:last-child { border-inline-end: none; }
	.ck-seg:hover { background: var(--background-modifier-hover, rgba(0,0,0,0.04)); }
	.ck-seg.on { background: color-mix(in srgb, var(--interactive-accent, #7c3aed) 16%, transparent); color: var(--interactive-accent, #7c3aed); }
	.ck-anchor { display: flex; align-items: center; gap: 8px; min-width: 0; flex: 1; }
	.ck-anchor-name { font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
	.ck-anchor-idle { color: var(--text-faint, #9ca3af); font-size: 13px; }
	.ck-pinbadge { font-size: 11px; color: var(--interactive-accent, #7c3aed); background: color-mix(in srgb, var(--interactive-accent, #7c3aed) 14%, transparent); border-radius: 5px; padding: 1px 6px; }
	.ck-ro { font-size: 11px; color: var(--text-faint, #9ca3af); border: 1px solid var(--background-modifier-border, #d4d4d8); border-radius: 5px; padding: 2px 8px; }
	.ck-dotlib { width: 8px; height: 8px; border-radius: 50%; display: inline-block; flex: none; }

	.ck-zone { background: var(--background-primary-alt, #f6f6f7); border-radius: 12px; padding: 12px 14px; }
	.ck-zlabel { font-size: 12px; color: var(--text-muted, #6b7280); margin-bottom: 10px; display: flex; align-items: baseline; gap: 8px; flex-wrap: wrap; }
	.ck-zhint { font-size: 11px; color: var(--text-faint, #9ca3af); }

	/* estimation map fills its strip */
	.ck-est-body { display: grid; grid-template-columns: repeat(auto-fit, minmax(120px, 1fr)); gap: 10px; }
	.ck-est-cell { background: var(--background-primary, #fff); border-radius: 8px; padding: 10px 12px; }
	.ck-est-num { font-size: 18px; font-weight: 500; display: flex; align-items: center; gap: 6px; min-width: 0; }
	.ck-est-num.ck-cap { text-transform: capitalize; font-size: 15px; }
	.ck-est-cap { font-size: 11px; color: var(--text-muted, #6b7280); margin-top: 2px; }
	.ck-est-note { font-size: 11px; color: var(--text-faint, #9ca3af); margin-top: 10px; }

	.ck-cols { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1.15fr); gap: 12px; flex: 1; min-height: 0; }
	.ck-control, .ck-operation { display: flex; flex-direction: column; min-height: 0; }

	.ck-rows { display: flex; flex-direction: column; gap: 9px; }
	.ck-row { display: flex; align-items: center; gap: 9px; font-size: 13px; }
	.ck-rk { color: var(--text-muted, #6b7280); }
	.ck-rv { margin-inline-start: auto; font-size: 13px; }
	.ck-rv.ck-cap { text-transform: capitalize; }
	.ck-dotc { width: 8px; height: 8px; border-radius: 50%; flex: none; }
	.s-ok, .s-success { background: #30a46c; }
	.s-bad, .s-danger { background: #e5484d; }
	.s-warn, .s-warning { background: #f5a623; }
	.s-acc { background: var(--interactive-accent, #7c3aed); }
	.s-muted { background: var(--text-faint, #9ca3af); }
	.s-badtext, .s-dangertext { color: #e5484d; }
	.s-warntext, .s-warningtext { color: #d98200; }
	.s-successtext { color: #30a46c; }
	.s-mutedtext { color: var(--text-muted, #6b7280); }

	.ck-op { display: flex; flex-direction: column; gap: 4px; overflow-y: auto; min-height: 0; }
	.ck-op-group { font-size: 11px; color: var(--text-faint, #9ca3af); margin: 8px 0 2px; display: flex; align-items: center; gap: 6px; }
	.ck-cand { font-size: 10px; color: var(--text-faint, #9ca3af); border: 1px solid var(--background-modifier-border, #d4d4d8); border-radius: 4px; padding: 0 4px; }
	.ck-link {
		display: flex; align-items: center; gap: 8px; width: 100%; text-align: start;
		background: transparent; border: none; border-radius: 7px; padding: 6px 8px;
		color: var(--text-normal, #1a1a1a); font-size: 13px; cursor: pointer;
	}
	.ck-link:hover:not(:disabled) { background: var(--background-modifier-hover, rgba(0,0,0,0.05)); }
	.ck-link:disabled { cursor: default; opacity: 0.55; }
	.ck-link-cand { opacity: 0.85; }
	.ck-type {
		font-size: 10px; text-transform: lowercase; flex: none;
		color: var(--interactive-accent, #7c3aed);
		background: color-mix(in srgb, var(--interactive-accent, #7c3aed) 12%, transparent);
		border-radius: 5px; padding: 1px 6px;
	}
	.ck-lname { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; min-width: 0; }
	.ck-go { margin-inline-start: auto; color: var(--text-faint, #9ca3af); flex: none; }
	.ck-empty { font-size: 12px; color: var(--text-faint, #9ca3af); padding: 4px 2px; }

	.ck-idle { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; color: var(--text-faint, #9ca3af); }
	.ck-idle p { font-size: 13px; }
</style>
