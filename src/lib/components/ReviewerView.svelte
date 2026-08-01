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
	import { computedPriority, effectivePriority, type ComputedPriority } from '$lib/reviewer/priorities';
	import { activeUniverseRootSync } from '$lib/libraries/store';
	import { getSummariesFor } from '$lib/nsc/summaryStore';
	import VirtualList from './VirtualList.svelte';
	import RelatedCandidates from './RelatedCandidates.svelte';
	import { onNoteMutation } from '$lib/noteMutations';
	import { onDestroy } from 'svelte';

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
		word_count: number;
		priority_override: number | null; // null = use the computed score
		alarm_reason: string | null;      // the note's canonical reason — drives priority
	}

	let {
		libraryPath = null,
		staleGraceDays = 1,
		selectPath = null, // MIG-084 §F.2 — seed the selection (return-to-the-note-you-were-on)
		onNoteClick,
		onClose,
		onOpenWithTab,   // (path, name, rightSidebarTab) — hand-off to 360 / Source Review
		onContext,       // MIG-096 §2 — right-click a queue row → the shared note menu (host-built)
	}: {
		libraryPath?: string | null;
		staleGraceDays?: number;
		selectPath?: string | null;
		onNoteClick?: (path: string, name: string) => void;
		onClose?: () => void;
		onOpenWithTab?: (path: string, name: string, tab: string) => void;
		onContext?: (path: string, name: string, e: MouseEvent) => void;
	} = $props();

	let dueNotes = $state<DueNote[]>([]);
	let loading = $state(true);
	// Selection identity is (reason | path), NOT path alone: a note legitimately appears
	// once per lens (two-lens-never-merged), so the SAME note can be both never_reviewed
	// AND orphan. Keying on path would resolve to the first row and show the wrong lens —
	// hiding an orphan's "Connect" verb behind its never-reviewed twin (review P1).
	let selectedKey = $state<string | null>(null);
	const keyOf = (n: DueNote) => `${n.reason}|${n.note_path}`;
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
			// Return-to-the-note-you-were-on: on a fresh open (no selection yet), seed the
			// selection to the note we came back from, if it's still in the queue.
			if (selectPath && !selectedKey) {
				const r = rows.find(x => x.note_path === selectPath);
				if (r) selectedKey = `${r.reason}|${r.note_path}`;
			}
			// Batched NSC headlines for the row context lines (reused from CeCe/Index — not
			// regenerated here; Rule 3). Fire-and-forget; the UI updates when it resolves.
			getSummariesFor(rows.map(r => r.note_path)).then(m => { if (my === gen) summaries = m; }).catch(() => {});
		} catch { if (my === gen) dueNotes = []; }
		if (my === gen) loading = false;
	}
	// Load on mount AND whenever libraryPath / grace change (the gen token guards races).
	// Reads props, writes dueNotes/loading — no echo loop (Rule 2).
	$effect(() => { libraryPath; staleGraceDays; load(); });

	// The six lenses, in order of consequence. The glyph + key drive header + row icon.
	const LENSES: { reason: string; icon: string; key: string }[] = [
		{ reason: 'stale',         icon: '🥀', key: 'stale' },
		{ reason: 'interval_due',  icon: '🔄', key: 'due' },
		{ reason: 'checkpoint',    icon: '🧠', key: 'checkpoint' },
		{ reason: 'orphan',        icon: '🔗', key: 'orphan' },
		{ reason: 'fragile',       icon: '⚠️', key: 'fragile' },
		{ reason: 'never_reviewed',icon: '📝', key: 'never' },
	];

	// Today as days-since-2020 (local), the engine's decay frame.
	const todayDay = $derived.by(() => {
		const d = new Date();
		return Math.floor(Date.UTC(d.getFullYear(), d.getMonth(), d.getDate()) / 86_400_000) - 18262;
	});
	// Each note carries its COMPUTED priority + EFFECTIVE priority (override ?? computed),
	// computed once here (cheap arithmetic, memoized by the $derived) and reused for
	// grouping, sorting, the detail recipe, and the percentile.
	type RankedNote = DueNote & { _computed: ComputedPriority; _effective: number };
	const ranked = $derived.by<RankedNote[]>(() =>
		dueNotes.map((n) => {
			// Priority is computed from the note's CANONICAL reason (alarm_reason), NOT the
			// per-row lens reason, so a multi-lens note has ONE priority on every row and
			// matches the note tab (review §F.2 P1 fix). The lens grouping still uses n.reason.
			const _computed = computedPriority({ ...n, reason: n.alarm_reason ?? n.reason }, todayDay);
			return { ...n, _computed, _effective: effectivePriority(n.priority_override, _computed.score) };
		})
	);
	const byReason = $derived.by(() => {
		const m = new Map<string, RankedNote[]>();
		for (const l of LENSES) m.set(l.reason, []);
		for (const n of ranked) { if (m.has(n.reason)) m.get(n.reason)!.push(n); }
		// Within each lens, highest EFFECTIVE priority first (the ranking lever).
		for (const arr of m.values()) arr.sort((a, b) => b._effective - a._effective);
		return m;
	});
	// Distinct notes (a note can sit in several lenses) — the truthful "how many need me".
	const distinctCount = $derived(new Set(dueNotes.map(n => n.note_path)).size);
	// Percentile of the selected note's effective priority among all due notes ("top N%").
	const selectedPercentile = $derived.by(() => {
		if (!selected || ranked.length < 2) return null;
		const me = (selected as RankedNote)._effective;
		const above = ranked.filter((r) => r._effective > me).length;
		return Math.max(1, Math.round((100 * (above + 1)) / ranked.length));
	});

	// All six lenses are ALWAYS shown (Eisa): empty ones appear muted with a 0. Each is
	// collapsible (a non-empty lens can be folded away).
	let collapsed = $state<Set<string>>(new Set());
	const isCollapsed = (r: string) => collapsed.has(r);
	function toggleLens(r: string) {
		const s = new Set(collapsed);
		if (s.has(r)) s.delete(r); else s.add(r);
		collapsed = s;
	}
	// The displayed selection: the clicked note, else fall back to the first due note so
	// the detail pane is never empty when there's work. A $derived (not a selectedPath-
	// writing $effect) — no echo loop (Rule 2). A stale selectedPath after reload simply
	// falls through to dueNotes[0].
	const selected = $derived<RankedNote | null>(ranked.find(n => keyOf(n) === selectedKey) ?? ranked[0] ?? null);

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

	// Localize a link-type id (supports/derives-from/…) via the existing linkTypes.*
	// catalog, never leak the raw id into prose (self-explanatory law). Guard on the
	// key-path return (this i18n layer returns the key when a key is missing, so plain
	// `|| fallback` cannot work) — fall back to the raw id for unknown/custom types.
	const typeName = (id: string | null | undefined) => {
		const raw = id ?? '';
		const k = 'linkTypes.' + raw.toLowerCase();
		const tr = $t(k);
		return tr && tr !== k ? tr : raw;
	};

	function sub(s: string, vars: Record<string, string | number>): string {
		return Object.entries(vars).reduce((acc, [k, v]) => acc.replaceAll(`{${k}}`, String(v)), s);
	}

	// The self-explanatory "why now" sentence for a note — the single most load-bearing
	// line. Every row answers "why am I being shown this?" in plain language.
	function whyNow(n: DueNote): string {
		switch (n.reason) {
			case 'stale':
				return sub($t('reviewer.why.stale') || '{type} “{name}” changed on {date}', {
					type: typeName(n.stale_trigger_type), name: n.stale_trigger_name ?? '?', date: n.stale_changed_on ?? '',
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
				// {out} dropped: the lens selects on the derives-from count (≤1), not the
				// total out-links, so showing outgoing_count as "support" would overstate it
				// (review P2). The detail facts row shows the accurate in/out separately.
				return sub($t('reviewer.why.fragile') || '{in} notes lean on this — give it firmer support', {
					in: n.incoming_count,
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
			// 2026-08-01 inspection — universeRoot captured at click time: a review action
			// racing a universe switch must land in THIS universe's pulse file, not the next.
			const universeRoot = activeUniverseRootSync();
			if (cmd === 'snooze_note') await invoke(cmd, { notePath: n.note_path, days: 7, universeRoot });
			else await invoke(cmd, { notePath: n.note_path, universeRoot });
			priorityError = null;
			await load();
			// The acted row usually leaves the queue; if the selection is now stale, move it
			// to the first remaining note so a row stays highlighted (matches the detail
			// pane's fallback). Writing selectedKey here is fine — not inside a $effect.
			if (!ranked.some(x => keyOf(x) === selectedKey)) selectedKey = ranked[0] ? keyOf(ranked[0]) : null;
		} catch (e) {
			priorityError = String(e);
			console.error('[Reviewer] review action failed:', e);
		}
	}

	// MIG-086 §F2 — after a one-click connect, OPTIMISTICALLY drop the in-hand note's
	// current lens row (orphan → now linked; fragile → now shored up) for INSTANT feedback.
	// We do NOT block on get_due_notes here: the connect's reindex runs in the background
	// (a link-dense source reindex can be slow — PJ-066), so the incoming_count isn't
	// updated yet and an immediate reload would still show the row. The DB reconciles on
	// the next Reviewer load (reopen / library change). selected.reason scopes the removal
	// to the lens the user acted on, so the note's OTHER lens rows (e.g. never_reviewed) stay.
	function refreshAfterConnect() {
		const sel = selected;
		if (!sel) return;
		dueNotes = dueNotes.filter(d => !(d.note_path === sel.note_path && d.reason === sel.reason));
		if (!ranked.some(x => keyOf(x) === selectedKey)) selectedKey = ranked[0] ? keyOf(ranked[0]) : null;
	}

	// MIG-096 §2 — refresh-after-mutate: keep the queue current when a note is
	// renamed / moved / deleted from the right-click menu (here or ANY surface,
	// via the broadcast). Rename/move do NOT change review membership, so we
	// re-title / re-path in place — cheap, no IPC, no loading flash; delete
	// removes the note from every lens. `selectedKey` embeds note_path
	// (`reason|path`), so it migrates alongside a rename/move or falls back on a
	// delete — mirroring the act()/refreshAfterConnect() re-point pattern above.
	function migrateSelectedKey(oldPath: string, newPath: string) {
		if (selectedKey && selectedKey.endsWith(`|${oldPath}`)) {
			selectedKey = selectedKey.slice(0, selectedKey.length - oldPath.length) + newPath;
		}
	}
	let unlistenMutations: (() => void) | null = null;
	let mutationsDestroyed = false;
	onNoteMutation({
		onDeleted: ({ path }) => {
			if (!dueNotes.some(d => d.note_path === path)) return;
			dueNotes = dueNotes.filter(d => d.note_path !== path);
			if (!ranked.some(x => keyOf(x) === selectedKey)) selectedKey = ranked[0] ? keyOf(ranked[0]) : null;
		},
		onRenamed: ({ oldPath, newPath, newName }) => {
			if (!dueNotes.some(d => d.note_path === oldPath)) return;
			dueNotes = dueNotes.map(d => d.note_path === oldPath ? { ...d, note_path: newPath, note_name: newName } : d);
			migrateSelectedKey(oldPath, newPath);
		},
		onMoved: ({ oldPath, newPath }) => {
			if (!dueNotes.some(d => d.note_path === oldPath)) return;
			dueNotes = dueNotes.map(d => d.note_path === oldPath ? { ...d, note_path: newPath } : d);
			migrateSelectedKey(oldPath, newPath);
		},
	}).then(u => { if (mutationsDestroyed) u(); else unlistenMutations = u; }).catch(() => {});
	onDestroy(() => { mutationsDestroyed = true; unlistenMutations?.(); });

	// Priority override: dragging commits an explicit override; Reset clears it (NULL =
	// use computed). Either reloads so the queue re-ranks by the new effective priority.
	const isManual = $derived(selected != null && selected.priority_override != null);
	// 2026-08-01 inspection — this was `catch {}`, the exact defect PJ-187 fixed in
	// ReviewStatusPanel (a Whole-Ecosystem miss): the slider stayed where the user dragged it
	// and NOTHING was written. Same treatment: snap the draft back so the control shows the
	// stored value, and say so.
	let priorityError = $state<string | null>(null);
	async function commitPriority(value: number) {
		if (!selected) return;
		try {
			await invoke('set_review_priority', { notePath: selected.note_path, priority: value });
			priorityError = null;
			priorityDraft = null;
			await load();
		} catch (e) {
			priorityDraft = null;
			priorityError = String(e);
			console.error('[Reviewer] set_review_priority failed:', e);
		}
	}
	async function resetPriority() {
		if (!selected) return;
		try {
			await invoke('set_review_priority', { notePath: selected.note_path, priority: null });
			priorityError = null;
			priorityDraft = null;
			await load();
		} catch (e) {
			priorityDraft = null;
			priorityError = String(e);
			console.error('[Reviewer] set_review_priority failed:', e);
		}
	}

	// The healthy PRESCRIPTION — the one remedy that cures this note's condition (the verb
	// itself is a button below). Deterministic per lens; the diagnosis is the why-now line.
	function prescription(n: DueNote): string {
		switch (n.reason) {
			case 'stale': return sub($t('reviewer.rx.stale') || 'Review it against “{name}” — reconcile your stance or update it.', { name: n.stale_trigger_name ?? '?' });
			case 'orphan': return $t('reviewer.rx.orphan') || 'Connect it to a related note — or mark it deliberately standalone.';
			case 'fragile': return $t('reviewer.rx.fragile') || 'Add a supporting (derives-from) link to ground it.';
			case 'interval_due': return $t('reviewer.rx.due') || 'Re-read it and confirm it still holds, then mark it reviewed.';
			case 'never_reviewed': return $t('reviewer.rx.never') || 'Give it its first review — read it through, then confirm or refine.';
			case 'checkpoint': return $t('reviewer.rx.checkpoint') || 'Re-examine this view — confirm it, revise it, or supersede it.';
			default: return '';
		}
	}

	const factorLabel = (key: string): string => (({
		decay: $t('reviewer.factor.decay') || 'Overdue / stale',
		disturbance: $t('reviewer.factor.disturbance') || 'Disruption',
		reach: $t('reviewer.factor.reach') || 'Depended on',
		maturity: $t('reviewer.factor.maturity') || 'Maturity',
		fragility: $t('reviewer.factor.fragility') || 'Fragility',
	}) as Record<string, string>)[key] ?? key;

	const fmtCount = (n: DueNote) =>
		sub($t('reviewer.connections') || '{in} in · {out} out', { in: n.incoming_count, out: n.outgoing_count });
</script>

<div class="rv">
	<header class="rv-head">
		<h1>🕐 {$t('reviewer.title') || 'Reviewer'}</h1>
		<span class="rv-total">{distinctCount}</span>
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
				{#each LENSES as lens (lens.reason)}
					{@const items = byReason.get(lens.reason) ?? []}
					<section class="rv-lens" class:rv-empty={items.length === 0}>
						<button class="rv-lens-head" onclick={() => toggleLens(lens.reason)} disabled={items.length === 0}>
							<span class="rv-chevron" class:collapsed={isCollapsed(lens.reason)}>▾</span>
							<span class="rv-lens-icon">{lens.icon}</span>
							<span class="rv-lens-name">{lensLabel(lens.key)}</span>
							<span class="rv-lens-count">{items.length}</span>
						</button>
						{#if items.length > 0 && !isCollapsed(lens.reason)}
						{#if items.length > 80}
							<div class="rv-vlist">
								<VirtualList items={items} getItemHeight={() => 46} overscan={8}>
									{#snippet row(n, _i)}
										<button class="rv-row" class:sel={keyOf(n) === selectedKey} onclick={() => selectedKey = keyOf(n)} oncontextmenu={(e) => { selectedKey = keyOf(n); onContext?.(n.note_path, n.note_name, e); }}>
											<span class="rv-row-name" dir="auto">{n.note_name}</span>
											<span class="rv-row-why" dir="auto">{whyNow(n)}</span>
										</button>
									{/snippet}
								</VirtualList>
							</div>
						{:else}
							{#each items as n (n.note_path)}
								<button class="rv-row" class:sel={keyOf(n) === selectedKey} onclick={() => selectedKey = keyOf(n)} oncontextmenu={(e) => { selectedKey = keyOf(n); onContext?.(n.note_path, n.note_name, e); }}>
									<span class="rv-row-name" dir="auto">{n.note_name}</span>
									<span class="rv-row-why" dir="auto">{whyNow(n)}</span>
								</button>
							{/each}
						{/if}
						{/if}
					</section>
				{/each}
			</div>

			<!-- DETAIL: the selected note's full review context + the decision verbs -->
			<div class="rv-detail">
				{#if selected}
					{@const n = selected}
					<div class="rv-d-title" dir="auto">{n.note_name}</div>

					<!-- Summary — ALWAYS shown (full body, regardless of any setting; Eisa). -->
					{#if summaries.get(n.note_path)?.summary}
						<div class="rv-d-summary" dir="auto">{summaries.get(n.note_path)?.summary}</div>
					{:else}
						<div class="rv-d-summary rv-d-summary-empty">{$t('reviewer.summaryEmpty') || 'No summary yet.'}</div>
					{/if}

					<!-- DIAGNOSIS: what's wrong (the why-now). -->
					<div class="rv-d-why">
						<span class="rv-d-why-icon">{LENSES.find(l => l.reason === n.reason)?.icon}</span>
						<span dir="auto">{whyNow(n)}</span>
					</div>

					<!-- PRESCRIPTION: the one healthy thing that cures it. -->
					<div class="rv-d-rx">
						<span class="rv-d-rx-label">{$t('reviewer.prescriptionLabel') || 'Prescription'}</span>
						<span class="rv-d-rx-text" dir="auto">{prescription(n)}</span>
					</div>

					<!-- MIG-086: the diagnosis becomes an action — suggested related notes to connect.
					     Orphan ("connect me") + fragile ("shore me up") get the list; fragile pre-sets
					     the derives-from type + a "shore it up" heading (§C). -->
					{#if isOrphan(n) || n.reason === 'fragile'}
						<RelatedCandidates
							notePath={n.note_path}
							noteName={n.note_name}
							{libraryPath}
							defaultType={n.reason === 'fragile' ? 'derives-from' : 'associative'}
							heading={n.reason === 'fragile' ? ($t('reviewer.suggestLabelFragile') || 'Shore it up — connect to:') : null}
							onConnected={refreshAfterConnect}
						/>
					{/if}

					<!-- PRIORITY: the computed score as a readable recipe + the override lever. -->
					<div class="rv-d-prio-box">
						<div class="rv-prio-head">
							<span class="rv-prio-num">{n._effective}</span>
							<span class="rv-prio-label">{$t('reviewer.priority') || 'Priority'}</span>
							{#if isManual}<span class="rv-prio-tag">{$t('reviewer.manual') || 'manual'}</span>{/if}
							{#if selectedPercentile != null}<span class="rv-prio-pct">{sub($t('reviewer.topPct') || 'top {n}%', { n: selectedPercentile })}</span>{/if}
						</div>
							<!-- The recipe explains the COMPUTED score; captioned when overridden so the
							     bar (summing to computed) is not read as the override (review §F.2 P3). -->
						{#if isManual}
							<div class="rv-prio-override">
								{sub($t('reviewer.computedWouldBe') || 'Computed would be {n}', { n: n._computed.score })}
								<button class="rv-link" onclick={resetPriority}>{$t('reviewer.resetComputed') || 'Reset to computed'}</button>
							</div>
						{/if}
						<div class="rv-prio-bar" aria-hidden="true">
							{#each n._computed.contributions.filter(c => c.points > 0) as c}
								<div class="rv-seg rv-seg-{c.axis}" style="flex: {c.points}" title="{factorLabel(c.key)} +{Math.round(c.points)}"></div>
							{/each}
						</div>
						<div class="rv-prio-legend">
							{#each n._computed.contributions.filter(c => c.points > 0) as c}
								<span class="rv-leg"><span class="rv-leg-dot rv-seg-{c.axis}"></span>{factorLabel(c.key)} +{Math.round(c.points)}</span>
							{/each}
						</div>
						<div class="rv-d-priority">
							<input id="rv-prio" type="range" min="0" max="100" step="5"
								value={priorityDraft ?? n._effective}
								oninput={(e) => priorityDraft = Number((e.currentTarget as HTMLInputElement).value)}
								onchange={(e) => commitPriority(Number((e.currentTarget as HTMLInputElement).value))} />
							<span class="rv-prio-val">{priorityDraft ?? n._effective}</span>
						</div>
						{#if priorityError}
							<div class="rv-prio-error">{$t('reviewer.prioritySaveFailed') || 'Could not save the priority — it has been left unchanged.'}</div>
						{/if}
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

					<!-- Decision verbs, each previewing its consequence where it has one. -->
					<div class="rv-d-actions">
						<!-- MIG-086 §C — the orphan's "Connect" primary button (which only opened
						     the editor, duplicating "↗ Open in editor" below) is removed; the real
						     connect is now the inline Link buttons in <RelatedCandidates> above.
						     Non-orphans keep their "Reviewed" verb. -->
						{#if !isOrphan(n)}
							<button class="rv-btn primary" onclick={() => act('mark_reviewed', n)}>✓ {$t('reviewPanel.reviewed') || 'Reviewed'}</button>
						{/if}
						{#if n.reason === 'interval_due' || n.reason === 'checkpoint' || n.reason === 'never_reviewed'}
							<button class="rv-btn" onclick={() => act('snooze_note', n)} title={$t('reviewPanel.snooze') || 'Snooze 7 days'}>👁 {$t('reviewer.snooze7') || 'Snooze 7d'}</button>
						{/if}
						<button class="rv-btn" onclick={() => act('dismiss_note', n)} title={isOrphan(n) ? ($t('reviewer.markStandalone') || 'Mark as standalone') : ($t('reviewPanel.dismiss') || 'Dismiss')}>🗄️ {isOrphan(n) ? ($t('reviewer.markStandalone') || 'Mark standalone') : ($t('reviewer.dismiss') || 'Dismiss')}</button>
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
		width: 100%; border: none; text-align: start; cursor: pointer; font-family: inherit;
		background: var(--background-primary); z-index: 1;
		font-size: calc(0.8rem * var(--rs-scale, 1)); font-weight: 600; color: var(--text-normal);
	}
	.rv-lens-head:hover:not(:disabled) { background: var(--background-modifier-hover); }
	.rv-lens-head:disabled { cursor: default; }
	/* Empty lens: always listed but muted, just the header + a 0 (Eisa). */
	.rv-empty .rv-lens-head { color: var(--text-faint); opacity: 0.6; }
	.rv-chevron { font-size: calc(0.62rem * var(--rs-scale, 1)); transition: transform 0.15s; flex-shrink: 0; }
	.rv-chevron.collapsed { transform: rotate(-90deg); }
	:global([dir="rtl"]) .rv-chevron.collapsed { transform: rotate(90deg); }
	.rv-empty .rv-chevron { visibility: hidden; }
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
	.rv-d-summary { font-size: calc(0.88rem * var(--rs-scale, 1)); color: var(--text-muted); margin-top: 6px; line-height: 1.45; }
	.rv-d-summary-empty { font-style: italic; color: var(--text-faint); }
	.rv-d-why {
		display: flex; align-items: flex-start; gap: 8px; margin-top: 16px; padding: 12px 14px; border-radius: 8px;
		background: var(--background-secondary); font-size: calc(0.92rem * var(--rs-scale, 1)); color: var(--text-normal); line-height: 1.4;
	}
	.rv-d-why-icon { flex-shrink: 0; }

	/* PRESCRIPTION — the remedy, set apart with an accent edge. */
	.rv-d-rx {
		display: flex; flex-direction: column; gap: 3px; margin-top: 12px; padding: 10px 14px; border-radius: 8px;
		border-inline-start: 3px solid var(--interactive-accent, #7c3aed); background: var(--background-secondary);
	}
	.rv-d-rx-label { font-size: calc(0.66rem * var(--rs-scale, 1)); text-transform: uppercase; letter-spacing: 0.05em; color: var(--interactive-accent, #7c3aed); font-weight: 600; }
	.rv-d-rx-text { font-size: calc(0.9rem * var(--rs-scale, 1)); color: var(--text-normal); line-height: 1.4; }

	/* PRIORITY recipe — the score, the stacked contribution bar, the legend, the lever. */
	.rv-d-prio-box { margin-top: 22px; }
	.rv-prio-head { display: flex; align-items: baseline; gap: 8px; }
	.rv-prio-num { font-size: calc(1.4rem * var(--rs-scale, 1)); font-weight: 700; color: var(--text-normal); }
	.rv-prio-label { font-size: calc(0.72rem * var(--rs-scale, 1)); text-transform: uppercase; letter-spacing: 0.05em; color: var(--text-faint); }
	.rv-prio-tag { font-size: calc(0.66rem * var(--rs-scale, 1)); color: var(--text-on-accent, #fff); background: var(--interactive-accent, #7c3aed); border-radius: 4px; padding: 1px 6px; }
	.rv-prio-pct { margin-inline-start: auto; font-size: calc(0.74rem * var(--rs-scale, 1)); color: var(--text-faint); }
	.rv-prio-bar { display: flex; height: 8px; border-radius: 4px; overflow: hidden; margin-top: 8px; background: var(--background-modifier-border); }
	.rv-seg { min-width: 2px; }
	.rv-seg-urgency { background: var(--color-orange, #e08c3b); }
	.rv-seg-importance { background: var(--color-blue, #4b8bd6); }
	.rv-prio-legend { display: flex; flex-wrap: wrap; gap: 4px 12px; margin-top: 6px; }
	.rv-leg { display: inline-flex; align-items: center; gap: 4px; font-size: calc(0.7rem * var(--rs-scale, 1)); color: var(--text-muted); }
	.rv-leg-dot { width: 8px; height: 8px; border-radius: 2px; display: inline-block; }
	.rv-prio-override { font-size: calc(0.74rem * var(--rs-scale, 1)); color: var(--text-faint); margin-top: 4px; display: flex; align-items: center; gap: 8px; }

	.rv-d-facts { display: flex; flex-wrap: wrap; gap: 20px; margin-top: 18px; }
	.rv-fact { display: flex; flex-direction: column; gap: 2px; }
	.rv-fact-k { font-size: calc(0.7rem * var(--rs-scale, 1)); color: var(--text-faint); text-transform: uppercase; letter-spacing: 0.04em; }
	.rv-fact-v { font-size: calc(0.95rem * var(--rs-scale, 1)); color: var(--text-normal); }

	.rv-d-priority { display: flex; align-items: center; gap: 12px; margin-top: 10px; }
	.rv-d-priority input[type="range"] { flex: 1; accent-color: var(--interactive-accent, #7c3aed); }
	.rv-prio-val { font-size: calc(0.85rem * var(--rs-scale, 1)); color: var(--text-normal); width: 2.5em; text-align: end; }
	.rv-prio-error { font-size: calc(0.72rem * var(--rs-scale, 1)); color: var(--text-error, #c0392b); margin-block-start: 2px; }

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
