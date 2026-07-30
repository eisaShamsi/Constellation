<script lang="ts">
	/**
	 * MIG-108 — the "One Universe, One Location" proposal dialog.
	 *
	 * The Constellation Way: the app OBSERVES (read-only preflight at boot), PROPOSES (this
	 * dialog — what was found, where each library will land, what is skipped and why, where
	 * the backup goes), and the USER decides. Nothing moves without the Unify click.
	 *
	 * Four states: proposal → running → summary; or resume (an unfinished journal found at
	 * boot — a crash mid-run — is surfaced, never silently continued).
	 *
	 * The run envelope lives here too: flush dirty tabs, close the second screen (it sits
	 * outside every freeze channel — Architect H9), stop every library watcher (H10 — the
	 * only subtree-scale suppression), then hand over to the journaled Rust engine. On
	 * success the app RELOADS: boot re-reads the rewritten registry, rewatches every library
	 * at its new path, and restores the session against the rewritten tab paths — the whole
	 * wake choreography for free, through the one path that is already proven.
	 */
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { onDestroy, onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { t } from '$lib/i18n';
	import { flushAllDirtyTabs, libraries } from '$lib/libraries/store';

	interface PreflightEntry {
		library_id: string;
		library_name: string;
		old_path: string;
		class: { kind: string; reason?: string };
		dest: string | null;
		same_volume: boolean;
	}
	interface PreflightReport {
		universe_root: string;
		entries: PreflightEntry[];
		decollided: string[];
	}
	interface JournalState {
		phase: string;
		entries_total: number;
		entries_moved: number;
		universe_root: string;
	}

	let { onDone }: { onDone?: () => void } = $props();

	let visible = $state(false);
	let mode = $state<'proposal' | 'running' | 'summary' | 'resume'>('proposal');
	let report = $state<PreflightReport | null>(null);
	let resumeState = $state<JournalState | null>(null);
	/** Entries the user flipped to Copy (old_path strings) — Boss D2/D3. */
	let copySet = $state<Set<string>>(new Set());
	let phaseLabel = $state('');
	let errorText = $state<string | null>(null);
	let summary = $state<{ moved: number; copied: number; skipped: number } | null>(null);
	let unlisten: (() => void) | null = null;

	const actionable = (r: PreflightReport | null) =>
		r?.entries.filter((e) => e.class.kind === 'move' || e.class.kind === 'copy') ?? [];
	const skipped = (r: PreflightReport | null) =>
		r?.entries.filter((e) => e.class.kind === 'foreign_universe' || e.class.kind === 'missing') ?? [];

	onMount(async () => {
		unlisten = await listen<string>('mig108:progress', (ev) => {
			phaseLabel = ev.payload;
		});
		try {
			const js = await invoke<JournalState | null>('mig108_journal_state');
			if (js) {
				resumeState = js;
				mode = 'resume';
				visible = true;
				return;
			}
			const r = await invoke<PreflightReport>('mig108_preflight', { copyPaths: [] });
			if (r.entries.some((e) => e.class.kind === 'move' || e.class.kind === 'copy')) {
				report = r;
				mode = 'proposal';
				visible = true;
			}
		} catch (e) {
			// A failed probe must never block boot — this dialog is a proposal, not a gate.
			console.error('[mig108] preflight probe failed:', e);
		}
	});
	onDestroy(() => unlisten?.());

	function toggleCopy(oldPath: string) {
		const next = new Set(copySet);
		if (next.has(oldPath)) next.delete(oldPath);
		else next.add(oldPath);
		copySet = next;
	}

	/** The freeze envelope, then the engine, then the reload. */
	async function runEnvelopeThen(cmd: 'mig108_execute' | 'mig108_resume') {
		errorText = null;
		mode = 'running';
		phaseLabel = 'snapshot';
		try {
			await flushAllDirtyTabs('mig108');
			try {
				if (await invoke<boolean>('is_second_screen_open')) await invoke('close_second_screen');
			} catch { /* no SS support on this setup — fine */ }
			for (const lib of get(libraries)) {
				try { await invoke('unwatch_library', { libraryId: lib.id }); } catch { /* not watching */ }
			}
			if (cmd === 'mig108_execute') {
				await invoke(cmd, { copyPaths: [...copySet] });
			} else {
				await invoke(cmd);
			}
			const acted = actionable(report);
			summary = {
				moved: resumeState ? resumeState.entries_total : acted.filter((e) => !copySet.has(e.old_path)).length,
				copied: copySet.size,
				skipped: skipped(report).length,
			};
			mode = 'summary';
		} catch (e) {
			errorText = String(e);
			// Back to where the user can decide; an unfinished journal will re-surface as a
			// resume proposal on the next boot either way.
			mode = resumeState ? 'resume' : 'proposal';
		}
	}

	function reloadNow() {
		onDone?.();
		window.location.href = '/';
	}

	function dismiss() {
		visible = false; // per-session dismissal — the proposal returns at next activation
	}

	const phaseText = (p: string) =>
		({
			snapshot: $t('mig108.phaseSnapshot') || 'Backing everything up…',
			moving: $t('mig108.phaseMoving') || 'Moving library folders…',
			rewriting: $t('mig108.phaseRewriting') || 'Updating the knowledge index…',
			stores: $t('mig108.phaseStores') || 'Updating saved lists and layouts…',
			trash: $t('mig108.phaseTrash') || 'Consolidating the trash…',
			done: $t('mig108.phaseDone') || 'Finished.',
		})[p] ?? p;
</script>

{#if visible}
	<div class="m108-backdrop" role="dialog" aria-modal="true">
		<div class="m108-card" dir="auto">
			{#if mode === 'proposal' && report}
				<h2>{$t('mig108.title') || 'Bring your universe into one place'}</h2>
				<p class="m108-intro">
					{($t('mig108.intro') || 'A universe is one folder that holds all of its knowledge. {n} of your libraries currently live outside this universe’s folder. Constellation can move them in — your notes, links and history all stay exactly as they are.')
						.replace('{n}', String(actionable(report).length))}
				</p>
				<div class="m108-list">
					{#each actionable(report) as e (e.library_id)}
						<div class="m108-row">
							<span class="m108-name">{e.library_name}</span>
							<span class="m108-paths">
								<span class="m108-old">{e.old_path}</span>
								<span class="m108-arrow">→</span>
								<span class="m108-new">{e.dest}</span>
							</span>
							<button
								class="m108-action"
								class:m108-copy={copySet.has(e.old_path)}
								title={$t('mig108.actionHint') || 'Click to switch between Move and Copy'}
								onclick={() => toggleCopy(e.old_path)}
							>{copySet.has(e.old_path) ? ($t('mig108.actionCopy') || 'Copy in') : ($t('mig108.actionMove') || 'Move in')}</button>
						</div>
					{/each}
					{#each skipped(report) as e (e.library_id)}
						<div class="m108-row m108-skip">
							<span class="m108-name">{e.library_name}</span>
							<span class="m108-skipwhy">
								{e.class.kind === 'missing'
									? ($t('mig108.skipMissing') || 'not found on disk — skipped')
									: ($t('mig108.skipForeign') || 'belongs to another universe — skipped')}
							</span>
						</div>
					{/each}
				</div>
				<p class="m108-backup">{$t('mig108.backupNote') || 'Before anything moves, a verified backup of the knowledge index and every settings file is stored inside the universe folder. Nothing is deleted.'}</p>
				{#if errorText}<p class="m108-error">{errorText}</p>{/if}
				<div class="m108-actions">
					<button class="m108-secondary" onclick={dismiss}>{$t('mig108.notNow') || 'Not now'}</button>
					<button class="m108-primary" onclick={() => runEnvelopeThen('mig108_execute')}>{$t('mig108.unify') || 'Unify'}</button>
				</div>
			{:else if mode === 'running'}
				<h2>{$t('mig108.runningTitle') || 'Unifying your universe…'}</h2>
				<div class="m108-spinner" aria-hidden="true"></div>
				<p class="m108-phase">{phaseText(phaseLabel)}</p>
				<p class="m108-wait">{$t('mig108.dontClose') || 'Please keep Constellation open. This usually takes well under a minute.'}</p>
			{:else if mode === 'summary'}
				<h2>{$t('mig108.summaryTitle') || 'Your universe is in one place'}</h2>
				<p class="m108-intro">
					{($t('mig108.summaryBody') || '{moved} libraries moved in, {copied} copied in. The backup is kept inside the universe folder until you choose to remove it.')
						.replace('{moved}', String(summary?.moved ?? 0))
						.replace('{copied}', String(summary?.copied ?? 0))}
				</p>
				<div class="m108-actions">
					<button class="m108-primary" onclick={reloadNow}>{$t('mig108.reloadNow') || 'Reload Constellation'}</button>
				</div>
			{:else if mode === 'resume' && resumeState}
				<h2>{$t('mig108.resumeTitle') || 'An unfinished unification was found'}</h2>
				<p class="m108-intro">
					{($t('mig108.resumeBody') || 'Constellation was interrupted while bringing your libraries into the universe folder ({done} of {total} moved). Everything is journaled and the backup is intact — it can pick up exactly where it stopped.')
						.replace('{done}', String(resumeState.entries_moved))
						.replace('{total}', String(resumeState.entries_total))}
				</p>
				{#if resumeState.phase === 'verify_failed'}
					<p class="m108-error">{$t('mig108.verifyFailedNote') || 'The last attempt stopped at the safety check and every database change was rolled back. Resuming will try the check again.'}</p>
				{/if}
				{#if errorText}<p class="m108-error">{errorText}</p>{/if}
				<div class="m108-actions">
					<button class="m108-primary" onclick={() => runEnvelopeThen('mig108_resume')}>{$t('mig108.resumeButton') || 'Resume and finish'}</button>
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	.m108-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.45);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 2000;
	}
	.m108-card {
		background: var(--background-primary);
		color: var(--text-normal);
		border-radius: 10px;
		padding: 24px 28px;
		width: min(680px, 92vw);
		max-height: 84vh;
		overflow-y: auto;
		box-shadow: 0 12px 40px rgba(0, 0, 0, 0.35);
	}
	h2 { margin: 0 0 10px; font-size: 1.15rem; }
	.m108-intro { font-size: 0.9rem; line-height: 1.5; color: var(--text-muted); }
	.m108-list { margin: 14px 0; display: flex; flex-direction: column; gap: 6px; }
	.m108-row {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 7px 10px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		font-size: 0.82rem;
	}
	.m108-name { font-weight: 600; white-space: nowrap; }
	.m108-paths { flex: 1; min-width: 0; display: flex; align-items: center; gap: 6px; overflow: hidden; }
	.m108-old { color: var(--text-faint); text-decoration: line-through; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; direction: ltr; }
	.m108-new { color: var(--text-normal); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; direction: ltr; }
	.m108-arrow { color: var(--text-muted); flex: none; }
	.m108-action {
		flex: none;
		font-size: 0.75rem;
		padding: 3px 10px;
		border-radius: 999px;
		border: 1px solid var(--interactive-accent);
		color: var(--interactive-accent);
		background: transparent;
		cursor: pointer;
	}
	.m108-action.m108-copy { background: var(--interactive-accent); color: var(--text-on-accent, #fff); }
	.m108-skip { opacity: 0.75; }
	.m108-skipwhy { font-size: 0.78rem; color: var(--text-faint); }
	.m108-backup { font-size: 0.8rem; color: var(--text-muted); border-inline-start: 3px solid var(--interactive-accent); padding-inline-start: 10px; }
	.m108-error { font-size: 0.82rem; color: var(--text-error); }
	.m108-actions { display: flex; justify-content: flex-end; gap: 10px; margin-top: 16px; }
	.m108-primary {
		background: var(--interactive-accent);
		color: var(--text-on-accent, #fff);
		border: none;
		border-radius: 6px;
		padding: 8px 18px;
		cursor: pointer;
		font-size: 0.88rem;
	}
	.m108-secondary {
		background: transparent;
		color: var(--text-muted);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		padding: 8px 14px;
		cursor: pointer;
		font-size: 0.88rem;
	}
	.m108-phase { font-size: 0.9rem; margin-top: 12px; }
	.m108-wait { font-size: 0.8rem; color: var(--text-faint); }
	.m108-spinner {
		width: 28px;
		height: 28px;
		border: 3px solid var(--background-modifier-border);
		border-top-color: var(--interactive-accent);
		border-radius: 50%;
		animation: m108spin 0.9s linear infinite;
		margin: 14px 0 4px;
	}
	@keyframes m108spin { to { transform: rotate(360deg); } }
</style>
