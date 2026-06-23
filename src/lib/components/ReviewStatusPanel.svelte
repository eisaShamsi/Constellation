<script lang="ts">
	// MIG-080 §F — the note-context Review tab. Answers "is THIS open note due or
	// stale?" via get_note_review_status (O(1) row lookup + the per-note Mode-2 probe).
	// Note-scoped: re-loads when the open note changes. The universe-wide queue lives
	// in the left-dock ReviewerView, not here.
	import { t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { computedPriority, effectivePriority } from '$lib/reviewer/priorities';

	interface NoteReviewStatus {
		reason: string | null;          // never_reviewed | interval_due | checkpoint | dismissed | null
		due_days: number | null;        // days-since-2020
		last_reviewed: string | null;
		never_reviewed: boolean;
		is_checkpoint: boolean;
		is_stale: boolean;
		stale_trigger_name: string | null;
		stale_trigger_type: string | null;
		stale_changed_on: string | null;
		// MIG-084 §F.2 — the override (null = use computed) + the signals to mirror the
		// Reviewer's computed priority.
		priority_override: number | null;
		word_count: number;
		incoming_count: number;
		outgoing_count: number;
		maturity: string;
		days_overdue: number;
		alarm_reason: string | null; // resolved lens reason (stale>fragile>orphan>schedule)
	}

	let {
		notePath = null,
		staleGraceDays = 1,
		onRefresh,
	}: {
		notePath?: string | null;
		staleGraceDays?: number;
		onRefresh?: () => void;
	} = $props();

	let status = $state<NoteReviewStatus | null>(null);
	let loading = $state(false);

	// Day-number (days since 2020-01-01) for "due in N days" math. Anchored to the
	// LOCAL calendar date (Date.UTC of the local Y/M/D = the local midnight as UTC-ms)
	// so it shares the Rust LOCAL frame — review.rs writes due_days via date_to_days of
	// today_str() (chrono::Local). 18262 = days from the 1970 epoch to 2020-01-01.
	const todayDay = $derived.by(() => {
		const n = new Date();
		return Math.floor(Date.UTC(n.getFullYear(), n.getMonth(), n.getDate()) / 86_400_000) - 18262;
	});

	// Monotonic request token: fast note-switching can resolve an earlier load() after
	// a later one — only the latest request may write `status`.
	let gen = 0;
	async function load() {
		if (!notePath) { status = null; return; }
		const my = ++gen;
		loading = true;
		try {
			const r = await invoke<NoteReviewStatus>('get_note_review_status', { notePath, staleGraceDays });
			if (my === gen) status = r;
		} catch { if (my === gen) status = null; }
		if (my === gen) loading = false;
	}

	// Re-load when the open note (or the grace setting) changes. Reads props, writes
	// `status` only — no echo loop.
	$effect(() => { notePath; staleGraceDays; load(); });

	async function act(cmd: 'mark_reviewed' | 'snooze_note' | 'dismiss_note') {
		if (!notePath) return;
		try {
			if (cmd === 'snooze_note') await invoke(cmd, { notePath, days: 7 });
			else await invoke(cmd, { notePath });
			await load();
			onRefresh?.();
		} catch {}
	}

	// MIG-084 §F.2 — the priority lever, mirrored from the Reviewer's detail pane. The
	// EFFECTIVE priority = the user override, else the computed score (same engine as the
	// Reviewer). Dragging commits an override; Reset clears it (null = use computed).
	const computed = $derived(status ? computedPriority({
		// Use the resolved lens reason (stale/fragile/orphan/schedule) so the note tab's
		// computed score matches the Reviewer's exactly (review §F.2 P1).
		reason: status.alarm_reason ?? status.reason ?? '', days_overdue: status.days_overdue,
		stale_trigger_type: status.stale_trigger_type, stale_changed_on: status.stale_changed_on,
		incoming_count: status.incoming_count, outgoing_count: status.outgoing_count, maturity: status.maturity,
	}, todayDay).score : 50);
	const effective = $derived(effectivePriority(status?.priority_override, computed));
	const isManual = $derived(status?.priority_override != null);
	let priorityDraft = $state<number | null>(null);
	$effect(() => { notePath; priorityDraft = null; }); // reset on note change
	async function commitPriority(value: number | null) {
		if (!notePath) return;
		try { await invoke('set_review_priority', { notePath, priority: value }); await load(); onRefresh?.(); } catch {}
	}

	// The primary Mode-1/3 status line (separate from the stale lens below).
	const dueLine = $derived.by(() => {
		if (!status || status.reason === null) return { icon: '🆕', text: $t('reviewStatus.notScheduled') || 'Not yet in the review schedule' };
		if (status.reason === 'dismissed') return { icon: '🗄️', text: $t('reviewStatus.dismissed') || 'Dismissed from review' };
		if (status.never_reviewed) return { icon: '📝', text: $t('reviewStatus.neverReviewed') || 'Never reviewed' };
		const dd = status.due_days ?? todayDay;
		const delta = dd - todayDay; // >0 future, 0 today, <0 overdue
		const icon = status.reason === 'checkpoint' ? '🧠' : '🔄';
		let text: string;
		if (delta > 0) text = (status.reason === 'checkpoint' ? ($t('reviewStatus.checkpointIn') || 'Checkpoint in {n}d') : ($t('reviewStatus.dueIn') || 'Due in {n}d')).replace('{n}', String(delta));
		else if (delta === 0) text = $t('reviewStatus.dueToday') || 'Due today';
		else text = ($t('reviewStatus.overdue') || 'Overdue by {n}d').replace('{n}', String(-delta));
		return { icon, text };
	});
</script>

<div class="rsp" dir="auto">
	{#if !notePath}
		<div class="rsp-empty">{$t('panels.noNoteSelected') || 'No note selected'}</div>
	{:else}
		<!-- Mode 1/3: time-based review status -->
		<div class="rsp-row rsp-primary">
			<span class="rsp-icon">{dueLine.icon}</span>
			<span class="rsp-text">{dueLine.text}</span>
			{#if status?.is_checkpoint}<span class="rsp-badge" title={$t('reviewPanel.checkpoints') || 'Mental Model Checkpoint'}>🧠</span>{/if}
		</div>
		{#if status?.last_reviewed}
			<div class="rsp-sub">{($t('reviewStatus.lastReviewed') || 'Last reviewed {date}').replace('{date}', status.last_reviewed)}</div>
		{/if}

		<!-- Mode 2: staleness (a separate lens; shown only when this note is stale) -->
		{#if status?.is_stale}
			<div class="rsp-stale">
				<span class="rsp-icon">🥀</span>
				<span class="rsp-text">{
					($t('reviewStatus.staleBecause') || 'Stale — {name} ({type}) changed on {date}')
						.replace('{name}', status.stale_trigger_name ?? '?')
						.replace('{type}', status.stale_trigger_type ?? '')
						.replace('{date}', status.stale_changed_on ?? '')
				}</span>
			</div>
		{/if}

		<!-- Priority lever (mirrors the Reviewer; computed by default, overridable). -->
		<div class="rsp-priority">
			<label for="rsp-prio">{$t('reviewer.priority') || 'Priority'}</label>
			<input id="rsp-prio" type="range" min="0" max="100" step="5"
				value={priorityDraft ?? effective}
				oninput={(e) => priorityDraft = Number((e.currentTarget as HTMLInputElement).value)}
				onchange={(e) => commitPriority(Number((e.currentTarget as HTMLInputElement).value))} />
			<span class="rsp-prio-val">{priorityDraft ?? effective}</span>
			{#if isManual}<span class="rsp-prio-tag">{$t('reviewer.manual') || 'manual'}</span>{/if}
		</div>
		{#if isManual}
			<div class="rsp-prio-override">
				{($t('reviewer.computedWouldBe') || 'Computed would be {n}').replace('{n}', String(computed))}
				<button class="rsp-reset" onclick={() => commitPriority(null)}>{$t('reviewer.resetComputed') || 'Reset to computed'}</button>
			</div>
		{/if}

		<!-- Actions (the only thing that advances last_reviewed is the explicit ✓). -->
		<div class="rsp-actions">
			<button class="rsp-btn" onclick={() => act('mark_reviewed')} title={$t('reviewPanel.reviewed') || 'Reviewed'}>✓ {$t('reviewPanel.reviewed') || 'Reviewed'}</button>
			<button class="rsp-btn" onclick={() => act('snooze_note')} title={$t('reviewPanel.snooze') || 'Snooze 7d'}>👁</button>
			<button class="rsp-btn" onclick={() => act('dismiss_note')} title={$t('reviewPanel.dismiss') || 'Dismiss'}>🗄️</button>
		</div>
	{/if}
</div>

<style>
	.rsp { padding: 12px; display: flex; flex-direction: column; gap: 8px; }
	.rsp-empty { text-align: center; color: var(--text-muted); padding: 24px 12px; font-size: calc(0.82rem * var(--rs-scale, 1)); }
	.rsp-row { display: flex; align-items: center; gap: 8px; }
	.rsp-primary { font-size: calc(0.9rem * var(--rs-scale, 1)); font-weight: 600; color: var(--text-normal); }
	.rsp-icon { flex-shrink: 0; }
	.rsp-text { min-width: 0; }
	.rsp-badge { margin-inline-start: auto; font-size: calc(0.8rem * var(--rs-scale, 1)); }
	.rsp-sub { font-size: calc(0.74rem * var(--rs-scale, 1)); color: var(--text-faint); }
	.rsp-stale {
		display: flex; align-items: flex-start; gap: 8px; padding: 8px 10px; border-radius: 6px;
		background: var(--background-modifier-error-hover, rgba(220,80,80,0.12));
		font-size: calc(0.8rem * var(--rs-scale, 1)); color: var(--text-normal); line-height: 1.35;
	}
	.rsp-priority { display: flex; align-items: center; gap: 8px; margin-top: 6px; }
	.rsp-priority label { font-size: calc(0.74rem * var(--rs-scale, 1)); color: var(--text-muted); flex-shrink: 0; }
	.rsp-priority input[type="range"] { flex: 1; accent-color: var(--interactive-accent, #7c3aed); min-width: 0; }
	.rsp-prio-val { font-size: calc(0.76rem * var(--rs-scale, 1)); color: var(--text-normal); width: 2.2em; text-align: end; }
	.rsp-prio-tag { font-size: calc(0.62rem * var(--rs-scale, 1)); color: var(--text-on-accent, #fff); background: var(--interactive-accent, #7c3aed); border-radius: 4px; padding: 1px 5px; }
	.rsp-prio-override { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; margin-top: 4px; font-size: calc(0.68rem * var(--rs-scale, 1)); color: var(--text-faint); }
	.rsp-reset { border: none; background: none; cursor: pointer; color: var(--interactive-accent, #7c3aed); font-family: inherit; font-size: inherit; padding: 0; text-decoration: underline; }
	.rsp-actions { display: flex; gap: 6px; margin-top: 4px; }
	.rsp-btn {
		border: 1px solid var(--background-modifier-border); background: var(--background-primary);
		color: var(--text-normal); border-radius: 6px; padding: 5px 10px; cursor: pointer;
		font-family: inherit; font-size: calc(0.78rem * var(--rs-scale, 1));
	}
	.rsp-btn:hover { background: var(--background-modifier-hover); }
</style>
