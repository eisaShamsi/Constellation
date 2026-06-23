<script lang="ts">
	// MIG-084 — the rich Reviewer: the constellation's TEMPORAL TRIAGE surface (the
	// "physician's call-back list"). A LEFT-DOCK full-page master-detail decision
	// surface over the now-cheap get_due_notes. Six lenses (Stale · Due · Checkpoints ·
	// 🔗 Orphan · ⚠ Fragile · Never), each a dated, actionable queue; a persistent
	// detail pane puts enough per-note evidence in front of the user to act WITHOUT
	// leaving the page (self-explanatory law, Eisa 2026-06-23). It scans-and-queues;
	// the siblings (360 / Cataloger / Sky / Knowledge Health) EXPLAIN — so the detail
	// pane hands off to them, never rebuilds them.
	import { t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { getSummariesFor } from '$lib/nsc/summaryStore';
	import VirtualList from './VirtualList.svelte';

	interface DueNote {
		note_path: string;
		note_name: string;
		reason: string;
		days_overdue: number;
		stratum: number;
		last_reviewed: string | null;
		stale_trigger_name?: string | null;
		stale_trigger_type?: string | null;
		stale_changed_on?: string | null;
		incoming_count: number;
		outgoing_count: number;
		maturity: string;
		priority: number;
	}

	let {
		libraryPath = null,
		staleGraceDays = 1,
		onNoteClick,
		onClose,
		onOpenWithTab,   // (path, name, rightSidebarTab) — hand-off to 360 / Source Review
	}: {
		libraryPath?: string | null;
		staleGraceDays?: number;
		onNoteClick?: (path: string, name: string) => void;
		onClose?: () => void;
		onOpenWithTab?: (path: string, name: string, tab: string) => void;
	} = $props();

	let dueNotes = $state<DueNote[]>([]);
	let loading = $state(true);
	let selectedPath = $state<string | null>(null);
	let summaries = $state<Map<string, { headline: string; summary: string }>>(new Map());
	let priorityDraft = $state<number | null>(null); // live slider value for the selected note

	let gen = 0;
	async function load() {
		if (!libraryPath) { dueNotes = []; loading = false; return; }
		const my = ++gen;
		loading = true;
		try {
			const rows = await invoke<DueNote[]>('get_due_notes', { libraryPath, staleGraceDays });
			if (my !== gen) return;
			dueNotes = rows;
			// Batched NSC headlines for the row context lines (reused from CeCe/Index — not
			// regenerated here; Rule 3). Fire-and-forget; the UI updates when it resolves.
			getSummariesFor(rows.map(r => r.note_path)).then(m => { if (my === gen) summaries = m; }).catch(() => {});
		} catch { if (my === gen) dueNotes = []; }
		if (my === gen) loading = false;
	}
	onMount(load);

	// The six lenses, in order of consequence. The glyph + key drive header + row icon.
	const LENSES: { reason: string; icon: string; key: string }[] = [
		{ reason: 'stale',         icon: '🥀', key: 'stale' },
		{ reason: 'interval_due',  icon: '🔄', key: 'due' },
		{ reason: 'checkpoint',    icon: '🧠', key: 'checkpoint' },
		{ reason: 'orphan',        icon: '🔗', key: 'orphan' },
		{ reason: 'fragile',       icon: '⚠️', key: 'fragile' },
		{ reason: 'never_reviewed',icon: '📝', key: 'never' },
	];

	const byReason = $derived.by(() => {
		const m = new Map<string, DueNote[]>();
		for (const l of LENSES) m.set(l.reason, []);
		for (const n of dueNotes) { if (m.has(n.reason)) m.get(n.reason)!.push(n); }
		return m;
	});
	const activeLenses = $derived(LENSES.filter(l => (byReason.get(l.reason)?.length ?? 0) > 0));
	// The displayed selection: the clicked note, else fall back to the first due note so
	// the detail pane is never empty when there's work. A $derived (not a selectedPath-
	// writing $effect) — no echo loop (Rule 2). A stale selectedPath after reload simply
	// falls through to dueNotes[0].
	const selected = $derived(dueNotes.find(n => n.note_path === selectedPath) ?? dueNotes[0] ?? null);

	// Reset the live priority draft whenever the displayed note changes.
	$effect(() => { selected?.note_path; priorityDraft = null; });

	const lensLabel = (key: string) =>
		({
			stale: $t('reviewer.lens.stale') || 'Stale',
			due: $t('reviewer.lens.due') || 'Due for Review',
			checkpoint: $t('reviewer.lens.checkpoint') || 'Mental-Model Checkpoints',
			orphan: $t('reviewer.lens.orphan') || 'Orphan — connect me',
			fragile: $t('reviewer.lens.fragile') || 'Fragile — shore me up',
			never: $t('reviewer.lens.never') || 'Never reviewed',
		} as Record<string, string>)[key] ?? key;

	const MATURITY: Record<string, { glyph: string; key: string; fallback: string }> = {
		seed:      { glyph: '·',  key: 'reviewer.maturity.seed',      fallback: 'seed' },
		sapling:   { glyph: '🌱', key: 'reviewer.maturity.sapling',   fallback: 'sapling' },
		evergreen: { glyph: '🌳', key: 'reviewer.maturity.evergreen', fallback: 'evergreen' },
		canonical: { glyph: '⭐', key: 'reviewer.maturity.canonical', fallback: 'canonical' },
		wilting:   { glyph: '🍂', key: 'reviewer.maturity.wilting',   fallback: 'wilting' },
	};
	const maturityWord = (m: string) => $t(MATURITY[m]?.key ?? '') || MATURITY[m]?.fallback || m;
	const maturityGlyph = (m: string) => MATURITY[m]?.glyph ?? '·';

	function sub(s: string, vars: Record<string, string | number>): string {
		return Object.entries(vars).reduce((acc, [k, v]) => acc.replaceAll(`{${k}}`, String(v)), s);
	}

	// The self-explanatory "why now" sentence for a note — the single most load-bearing
	// line. Every row answers "why am I being shown this?" in plain language.
	function whyNow(n: DueNote): string {
		switch (n.reason) {
			case 'stale':
				return sub($t('reviewer.why.stale') || '{type} “{name}” changed on {date}', {
					type: n.stale_trigger_type ?? '', name: n.stale_trigger_name ?? '?', date: n.stale_changed_on ?? '',
				});
			case 'interval_due':
				return n.days_overdue > 0
					? sub($t('reviewer.why.overdue') || 'Review overdue by {n} day(s)', { n: n.days_overdue })
					: ($t('reviewer.why.dueToday') || 'Due for review today');
			case 'checkpoint':
				return $t('reviewer.why.checkpoint') || 'A mental model — do you still hold this view?';
			case 'orphan':
				return $t('reviewer.why.orphan') || 'Nothing links here yet — connect it into your thinking';
			case 'fragile':
				return sub($t('reviewer.why.fragile') || '{in} notes depend on this, with only {out} support', {
					in: n.incoming_count, out: n.outgoing_count,
				});
			case 'never_reviewed':
				return sub($t('reviewer.why.never') || 'Never reviewed · {n} day(s) old', { n: n.days_overdue });
			default:
				return '';
		}
	}

	const isOrphan = (n: DueNote | null) => n?.reason === 'orphan';

	async function act(cmd: 'mark_reviewed' | 'snooze_note' | 'dismiss_note', n: DueNote) {
		try {
			if (cmd === 'snooze_note') await invoke(cmd, { notePath: n.note_path, days: 7 });
			else await invoke(cmd, { notePath: n.note_path });
			await load();
		} catch {}
	}

	async function commitPriority(n: DueNote, value: number) {
		try { await invoke('set_review_priority', { notePath: n.note_path, priority: value }); n.priority = value; } catch {}
	}

	const fmtCount = (n: DueNote) =>
		sub($t('reviewer.connections') || '{in} in · {out} out', { in: n.incoming_count, out: n.outgoing_count });
</script>

<div class="rv">
	<header class="rv-head">
		<h1>🕐 {$t('panels.review') || 'Review Pulse'}</h1>
		<span class="rv-total">{dueNotes.length}</span>
		<button class="rv-close" onclick={() => onClose?.()} aria-label={$t('common.close') || 'Close'} title={$t('common.close') || 'Close'}>✕</button>
	</header>

	{#if loading}
		<div class="rv-msg">{$t('common.loading') || 'Loading…'}</div>
	{:else if dueNotes.length === 0}
		<div class="rv-empty">
			<div class="rv-empty-icon">✅</div>
			<div class="rv-empty-text">{$t('reviewer.allCaughtUp') || 'All caught up — nothing needs your attention right now.'}</div>
		</div>
	{:else}
		<div class="rv-body">
			<!-- MASTER: the six-lens queue -->
			<div class="rv-master">
				{#each activeLenses as lens (lens.reason)}
					{@const items = byReason.get(lens.reason) ?? []}
					<section class="rv-lens">
						<div class="rv-lens-head">
							<span class="rv-lens-icon">{lens.icon}</span>
							<span class="rv-lens-name">{lensLabel(lens.key)}</span>
							<span class="rv-lens-count">{items.length}</span>
						</div>
						{#if items.length > 80}
							<div class="rv-vlist">
								<VirtualList items={items} getItemHeight={() => 46} overscan={8}>
									{#snippet row(n, _i)}
										<button class="rv-row" class:sel={n.note_path === selectedPath} onclick={() => selectedPath = n.note_path}>
											<span class="rv-row-name" dir="auto">{n.note_name}</span>
											<span class="rv-row-why" dir="auto">{whyNow(n)}</span>
										</button>
									{/snippet}
								</VirtualList>
							</div>
						{:else}
							{#each items as n (n.note_path)}
								<button class="rv-row" class:sel={n.note_path === selectedPath} onclick={() => selectedPath = n.note_path}>
									<span class="rv-row-name" dir="auto">{n.note_name}</span>
									<span class="rv-row-why" dir="auto">{whyNow(n)}</span>
								</button>
							{/each}
						{/if}
					</section>
				{/each}
			</div>

			<!-- DETAIL: the selected note's full review context + the decision verbs -->
			<div class="rv-detail">
				{#if selected}
					{@const n = selected}
					<div class="rv-d-title" dir="auto">{n.note_name}</div>
					{#if summaries.get(n.note_path)?.headline}
						<div class="rv-d-headline" dir="auto">{summaries.get(n.note_path)?.headline}</div>
					{/if}

					<div class="rv-d-why">
						<span class="rv-d-why-icon">{LENSES.find(l => l.reason === n.reason)?.icon}</span>
						<span dir="auto">{whyNow(n)}</span>
					</div>

					<div class="rv-d-facts">
						<div class="rv-fact">
							<span class="rv-fact-k">{$t('reviewer.maturityLabel') || 'Maturity'}</span>
							<span class="rv-fact-v">{maturityGlyph(n.maturity)} {maturityWord(n.maturity)}</span>
						</div>
						<div class="rv-fact">
							<span class="rv-fact-k">{$t('reviewer.connectionsLabel') || 'Connections'}</span>
							<span class="rv-fact-v">{fmtCount(n)}</span>
						</div>
						<div class="rv-fact">
							<span class="rv-fact-k">{$t('reviewer.lastReviewedLabel') || 'Last reviewed'}</span>
							<span class="rv-fact-v">{n.last_reviewed ?? ($t('reviewer.never') || 'never')}</span>
						</div>
					</div>

					<!-- Priority: the user's ranking lever (also on the note's Review tab). -->
					<div class="rv-d-priority">
						<label for="rv-prio">{$t('reviewer.priority') || 'Priority'}</label>
						<input id="rv-prio" type="range" min="0" max="100" step="5"
							value={priorityDraft ?? n.priority}
							oninput={(e) => priorityDraft = Number((e.currentTarget as HTMLInputElement).value)}
							onchange={(e) => commitPriority(n, Number((e.currentTarget as HTMLInputElement).value))} />
						<span class="rv-prio-val">{priorityDraft ?? n.priority}</span>
					</div>

					<!-- Decision verbs, each previewing its consequence where it has one. -->
					<div class="rv-d-actions">
						{#if isOrphan(n)}
							<button class="rv-btn primary" onclick={() => onNoteClick?.(n.note_path, n.note_name)}>🔗 {$t('reviewer.connect') || 'Connect'}</button>
						{:else}
							<button class="rv-btn primary" onclick={() => act('mark_reviewed', n)}>✓ {$t('reviewPanel.reviewed') || 'Reviewed'}</button>
						{/if}
						<button class="rv-btn" onclick={() => act('snooze_note', n)} title={$t('reviewPanel.snooze') || 'Snooze 7 days'}>👁 {$t('reviewer.snooze7') || 'Snooze 7d'}</button>
						<button class="rv-btn" onclick={() => act('dismiss_note', n)} title={$t('reviewPanel.dismiss') || 'Dismiss'}>🗄️ {$t('reviewer.dismiss') || 'Dismiss'}</button>
					</div>

					<!-- Hand-offs: the Reviewer triages, the siblings explain. -->
					<div class="rv-d-handoffs">
						<button class="rv-link" onclick={() => onNoteClick?.(n.note_path, n.note_name)}>↗ {$t('reviewer.openEditor') || 'Open in editor'}</button>
						<button class="rv-link" onclick={() => onOpenWithTab?.(n.note_path, n.note_name, 'inspector360')}>🔬 {$t('reviewer.see360') || 'Full context (360°)'}</button>
						<button class="rv-link" onclick={() => onOpenWithTab?.(n.note_path, n.note_name, 'sourceReview')}>🏷️ {$t('reviewer.classify') || 'Classify'}</button>
					</div>
				{:else}
					<div class="rv-msg">{$t('reviewer.selectNote') || 'Select a note to see its review context.'}</div>
				{/if}
			</div>
		</div>
	{/if}
</div>

<style>
	.rv { display: flex; flex-direction: column; height: 100%; min-height: 0; }
	.rv-head {
		display: flex; align-items: center; gap: 10px; padding: 12px 20px; flex-shrink: 0;
		border-bottom: 1px solid var(--background-modifier-border);
	}
	.rv-head h1 { margin: 0; font-size: calc(1.1rem * var(--rs-scale, 1)); font-weight: 600; color: var(--text-normal); }
	.rv-total { font-size: calc(0.85rem * var(--rs-scale, 1)); color: var(--text-faint); }
	.rv-close {
		margin-inline-start: auto; border: none; background: none; cursor: pointer; color: var(--text-muted);
		font-size: calc(1rem * var(--rs-scale, 1)); padding: 4px 8px; border-radius: 4px;
	}
	.rv-close:hover { background: var(--background-modifier-hover); color: var(--text-normal); }

	.rv-body { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(320px, 420px) 1fr; }
	.rv-master { overflow-y: auto; border-inline-end: 1px solid var(--background-modifier-border); padding: 8px 0; min-height: 0; }
	.rv-detail { overflow-y: auto; padding: 24px 28px; min-height: 0; }

	.rv-lens { margin-bottom: 6px; }
	.rv-lens-head {
		display: flex; align-items: center; gap: 8px; padding: 6px 16px; position: sticky; top: 0;
		background: var(--background-primary); z-index: 1;
		font-size: calc(0.8rem * var(--rs-scale, 1)); font-weight: 600; color: var(--text-normal);
	}
	.rv-lens-icon { flex-shrink: 0; }
	.rv-lens-count { margin-inline-start: auto; font-size: calc(0.72rem * var(--rs-scale, 1)); color: var(--text-faint); font-weight: 400; }

	.rv-vlist { display: flex; flex-direction: column; max-height: 50vh; min-height: 0; }
	.rv-row {
		display: flex; flex-direction: column; gap: 2px; width: 100%; text-align: start;
		padding: 6px 16px 6px 30px; border: none; background: none; cursor: pointer; font-family: inherit;
		border-inline-start: 2px solid transparent;
	}
	.rv-row:hover { background: var(--background-modifier-hover); }
	.rv-row.sel { background: var(--background-modifier-hover); border-inline-start-color: var(--interactive-accent, #7c3aed); }
	.rv-row-name {
		font-size: calc(0.82rem * var(--rs-scale, 1)); color: var(--text-normal);
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.rv-row-why {
		font-size: calc(0.7rem * var(--rs-scale, 1)); color: var(--text-faint);
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}

	.rv-d-title { font-size: calc(1.25rem * var(--rs-scale, 1)); font-weight: 600; color: var(--text-normal); }
	.rv-d-headline { font-size: calc(0.88rem * var(--rs-scale, 1)); color: var(--text-muted); margin-top: 4px; line-height: 1.4; }
	.rv-d-why {
		display: flex; align-items: flex-start; gap: 8px; margin-top: 16px; padding: 12px 14px; border-radius: 8px;
		background: var(--background-secondary); font-size: calc(0.92rem * var(--rs-scale, 1)); color: var(--text-normal); line-height: 1.4;
	}
	.rv-d-why-icon { flex-shrink: 0; }

	.rv-d-facts { display: flex; flex-wrap: wrap; gap: 20px; margin-top: 18px; }
	.rv-fact { display: flex; flex-direction: column; gap: 2px; }
	.rv-fact-k { font-size: calc(0.7rem * var(--rs-scale, 1)); color: var(--text-faint); text-transform: uppercase; letter-spacing: 0.04em; }
	.rv-fact-v { font-size: calc(0.95rem * var(--rs-scale, 1)); color: var(--text-normal); }

	.rv-d-priority { display: flex; align-items: center; gap: 12px; margin-top: 22px; }
	.rv-d-priority label { font-size: calc(0.8rem * var(--rs-scale, 1)); color: var(--text-muted); flex-shrink: 0; }
	.rv-d-priority input[type="range"] { flex: 1; accent-color: var(--interactive-accent, #7c3aed); }
	.rv-prio-val { font-size: calc(0.85rem * var(--rs-scale, 1)); color: var(--text-normal); width: 2.5em; text-align: end; }

	.rv-d-actions { display: flex; gap: 8px; margin-top: 24px; flex-wrap: wrap; }
	.rv-btn {
		border: 1px solid var(--background-modifier-border); background: var(--background-primary); color: var(--text-normal);
		border-radius: 8px; padding: 8px 14px; cursor: pointer; font-family: inherit; font-size: calc(0.85rem * var(--rs-scale, 1));
	}
	.rv-btn:hover { background: var(--background-modifier-hover); }
	.rv-btn.primary { background: var(--interactive-accent, #7c3aed); color: var(--text-on-accent, #fff); border-color: transparent; }

	.rv-d-handoffs { display: flex; gap: 6px; margin-top: 18px; flex-wrap: wrap; border-top: 1px solid var(--background-modifier-border); padding-top: 16px; }
	.rv-link {
		border: none; background: none; color: var(--text-muted); cursor: pointer; font-family: inherit;
		font-size: calc(0.8rem * var(--rs-scale, 1)); padding: 4px 8px; border-radius: 6px;
	}
	.rv-link:hover { background: var(--background-modifier-hover); color: var(--text-normal); }

	.rv-msg { padding: 32px; text-align: center; color: var(--text-muted); font-size: calc(0.85rem * var(--rs-scale, 1)); }
	.rv-empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 10px; }
	.rv-empty-icon { font-size: calc(2.4rem * var(--rs-scale, 1)); }
	.rv-empty-text { font-size: calc(0.9rem * var(--rs-scale, 1)); color: var(--text-muted); max-width: 360px; text-align: center; line-height: 1.4; }
</style>
