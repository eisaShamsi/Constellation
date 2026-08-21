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
		restorable: boolean;
		/** Why the last attempt stopped — journaled by the engine, shown verbatim below. */
		last_error: string | null;
	}

	let { onDone, onDismiss }: { onDone?: () => void; onDismiss?: () => void } = $props();

	let visible = $state(false);
	let mode = $state<'proposal' | 'running' | 'summary' | 'resume' | 'blocked'>('proposal');
	let report = $state<PreflightReport | null>(null);
	let resumeState = $state<JournalState | null>(null);
	/** Entries the user flipped to Copy (old_path strings) — Boss D2/D3. */
	let copySet = $state<Set<string>>(new Set());
	let phaseLabel = $state('');
	let errorText = $state<string | null>(null);
	/**
	 * PJ-322 — WHICH failure the blocked card is describing.
	 *
	 * `transient` = a file was briefly locked (a backup, antivirus or sync tool). Retrying works.
	 * `damaged`   = the file is unreadable or empty. **Retrying is pointless** and offering it is
	 *               a lie, so the Try-again button is not rendered in that state.
	 *
	 * The backend already knew the difference and threw it away at the IPC boundary; the panel
	 * caught the card telling the user "usually temporary" about a permanently damaged file.
	 * The marker is machine-readable on purpose — the frontend must never pattern-match a
	 * translated sentence to decide which buttons are honest.
	 */
	let blockedKind = $state<'transient' | 'damaged'>('transient');

	const REFUSAL_TRANSIENT = 'transient|';
	const REFUSAL_DAMAGED = 'damaged|';

	/** Split the marker off an error and record the kind. The marker is never shown. */
	function readRefusal(e: unknown): string {
		const raw = String(e);
		if (raw.includes(REFUSAL_DAMAGED)) {
			blockedKind = 'damaged';
			return raw.slice(raw.indexOf(REFUSAL_DAMAGED) + REFUSAL_DAMAGED.length);
		}
		if (raw.includes(REFUSAL_TRANSIENT)) {
			blockedKind = 'transient';
			return raw.slice(raw.indexOf(REFUSAL_TRANSIENT) + REFUSAL_TRANSIENT.length);
		}
		// Unmarked: assume transient, which is the recoverable reading — offering a retry that
		// turns out to be useless costs a click; withholding one that would have worked strands
		// the user.
		blockedKind = 'transient';
		return raw;
	}
	let summary = $state<{ moved: number; copied: number; skipped: number } | null>(null);
	let unlisten: (() => void) | null = null;
	let unlistenClose: (() => void) | null = null;
	/** The user tried to close the window mid-run; Rust refused it (Phase-4 audit). */
	let closeBlocked = $state(false);

	const actionable = (r: PreflightReport | null) =>
		r?.entries.filter((e) => e.class.kind === 'move' || e.class.kind === 'copy') ?? [];
	const skipped = (r: PreflightReport | null) =>
		r?.entries.filter((e) => e.class.kind === 'foreign_universe' || e.class.kind === 'missing') ?? [];

	onMount(async () => {
		unlisten = await listen<string>('mig108:progress', (ev) => {
			phaseLabel = ev.payload;
		});
		// Phase-4 audit — the user clicked the window's X mid-run and Rust refused it. Say so
		// on the running screen, so the refusal reads as deliberate rather than as a freeze.
		unlistenClose = await listen('mig108:close-blocked', () => {
			closeBlocked = true;
		});
		try {
			const js = await invoke<JournalState | null>('mig108_journal_state');
			if (js) {
				resumeState = js;
				mode = 'resume';
				visible = true;
				return;
			}
		} catch (e) {
			// Phase-4 audit — a CORRUPT journal is the only record of a possibly half-moved
			// universe; it must reach the USER, not a dev-only console. Surface it in the
			// resume card with no resume/restore offered (the journal cannot be trusted).
			errorText = String(e);
			resumeState = null;
			mode = 'resume';
			visible = true;
			return;
		}
		try {
			const r = await invoke<PreflightReport>('mig108_preflight', { copyPaths: [] });
			if (r.entries.some((e) => e.class.kind === 'move' || e.class.kind === 'copy')) {
				report = r;
				mode = 'proposal';
				visible = true;
			}
		} catch (e) {
			// PJ-322 (Boss decision 1, 2026-08-20) — this used to be a console-only log, and
			// devtools is dev-only in a release build, so the failure was INVISIBLE. That was
			// tolerable only while the probe could not fail for a reason that mattered; now it
			// refuses when it cannot tell which folders belong to other universes, and a
			// proposal that silently fails to appear is the same silent-failure class as a
			// wrong proposal. It must reach the user.
			//
			// Still not a gate: the 'blocked' card offers "Not now" and never wedges boot.
			console.error('[mig108] preflight probe failed:', e);
			errorText = readRefusal(e);
			mode = 'blocked';
			visible = true;
		}
		// Safety inspection 2026-08-01 — EVERY path out of this probe must release the boot
		// gate. The layout parks the whole watcher/session fan-out on a promise only this
		// component resolves; if we finish WITHOUT becoming visible (the layout saw a journal
		// and we did not, a preflight throw, nothing actionable to propose) there is no button
		// left to press and the app would sit with no watchers and no tabs, forever. Releasing
		// here is always safe: not-visible means we are not asking the user for anything.
		if (!visible) onDismiss?.();
	});
	onDestroy(() => { unlisten?.(); unlistenClose?.(); });

	/**
	 * PJ-322 — the reason the probe refuses is usually TRANSIENT (an antivirus or sync tool
	 * holding `universes.json` for a moment), which the backend distinguishes from a
	 * permanently corrupt file. Retrying is the correct affordance for that case and costs
	 * nothing when it is not.
	 */
	async function retryPreflight() {
		errorText = null;
		try {
			const r = await invoke<PreflightReport>('mig108_preflight', { copyPaths: [] });
			if (r.entries.some((e) => e.class.kind === 'move' || e.class.kind === 'copy')) {
				report = r;
				mode = 'proposal';
			} else {
				// Nothing to propose — the check now succeeds and there is nothing to ask.
				dismiss();
			}
		} catch (e) {
			errorText = readRefusal(e);
		}
	}

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
			// Phase-4 audit — THAW on failure: the envelope stopped every watcher; leaving
			// them off ran the live session blind against a possibly-changed disk. Rewatch
			// everything (idempotent), then re-probe the journal so a failed FIRST run
			// switches this dialog into resume mode (Unify would only error
			// "resume it instead" — the button the user needs is Resume/Restore).
			for (const lib of get(libraries)) {
				try { await invoke('watch_library', { libraryId: lib.id, libraryPath: lib.path }); } catch { /* best-effort */ }
			}
			try {
				const js = await invoke<JournalState | null>('mig108_journal_state');
				resumeState = js;
				mode = js ? 'resume' : 'proposal';
			} catch {
				mode = resumeState ? 'resume' : 'proposal';
			}
		}
	}

	async function restoreNow() {
		errorText = null;
		mode = 'running';
		phaseLabel = 'restore';
		try {
			await invoke('mig108_restore');
			// Everything is back at its old paths; reload through the proven boot path.
			window.location.href = '/';
		} catch (e) {
			errorText = String(e);
			mode = 'resume';
		}
	}

	function reloadNow() {
		onDone?.();
		window.location.href = '/';
	}

	function dismiss() {
		visible = false; // per-session dismissal — the proposal returns at next activation
		// Phase-4 audit — releases the boot gate: while an unfinished journal is present the
		// layout holds the watcher/session fan-out for this dialog, so dismissing must hand
		// it back or the session runs on with no watchers and no restored tabs.
		onDismiss?.();
	}

	const phaseText = (p: string) =>
		({
			snapshot: $t('mig108.phaseSnapshot') || 'Backing everything up…',
			moving: $t('mig108.phaseMoving') || 'Moving library folders…',
			rewriting: $t('mig108.phaseRewriting') || 'Updating the knowledge index…',
			stores: $t('mig108.phaseStores') || 'Updating saved lists and layouts…',
			trash: $t('mig108.phaseTrash') || 'Consolidating the trash…',
			restore: $t('mig108.phaseRestore') || 'Putting everything back…',
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
				<p class="m108-wait">{$t('mig108.dontClose') || 'Please keep Constellation open. On a large universe this can take several minutes — the steps above will keep you informed.'}</p>
				{#if closeBlocked}
					<p class="m108-error">{$t('mig108.closeBlocked') || 'Constellation cannot be closed while your libraries are being moved — closing now would leave the move half-finished. It will close normally as soon as this is done.'}</p>
				{/if}
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
			{:else if mode === 'blocked'}
				<!-- PJ-322 — Constellation could not establish which folders belong to OTHER
				     universes, so it proposes nothing. Saying so is the point: the previous
				     behaviour was to fail into a console the user cannot see, which looks
				     exactly like "there was nothing to propose". -->
				<h2>{$t('mig108.blockedTitle') || 'Constellation could not check your other universes'}</h2>
				<p class="m108-intro">
					{#if blockedKind === 'damaged'}
						{$t('mig108.blockedBodyDamaged') || 'To suggest tidying your libraries, Constellation first has to know which folders belong to your other universes — and one of those records is damaged or empty, so it cannot be read at all. Rather than guess, it has stopped and changed nothing. This will not clear up on its own: the file named below needs to be restored or that universe re-linked. Nothing else in Constellation is affected.'}
					{:else}
						{$t('mig108.blockedBody') || 'To suggest tidying your libraries, Constellation first has to know which folders belong to your other universes — and it could not read that just now. Rather than guess, it has stopped and changed nothing. This is usually temporary: a backup, antivirus or sync tool holding a file for a moment. Try again, or carry on and Constellation will ask another time.'}
					{/if}
				</p>
				{#if errorText}<p class="m108-reason">{errorText}</p>{/if}
				<div class="m108-actions">
					<button class="m108-secondary" onclick={dismiss}>{$t('mig108.notNow') || 'Not now'}</button>
					<!-- No retry for `damaged`: the file will not repair itself, and a button that
					     cannot work is worse than no button. -->
					{#if blockedKind !== 'damaged'}
						<button class="m108-primary" onclick={retryPreflight}>{$t('mig108.tryAgain') || 'Try again'}</button>
					{/if}
				</div>
			{:else if mode === 'resume'}
				<h2>{$t('mig108.resumeTitle') || 'An unfinished unification was found'}</h2>
				{#if resumeState}
					<p class="m108-intro">
						{($t('mig108.resumeBody') || 'Constellation was interrupted while bringing your libraries into the universe folder ({done} of {total} moved). Everything is journaled and the backup is intact — it can pick up exactly where it stopped.')
							.replace('{done}', String(resumeState.entries_moved))
							.replace('{total}', String(resumeState.entries_total))}
					</p>
					{#if resumeState.phase === 'verify_failed'}
						<p class="m108-error">{$t('mig108.verifyFailedNote') || 'The last attempt stopped at the safety check and every database change was rolled back. Resuming will try the check again.'}</p>
						{#if resumeState.last_error}
							<!-- Stage-B 2026-08-01 — the reason, verbatim. The live failure showed the
							     sentence above and nothing else, so the cause had to be reconstructed
							     from the user's data afterwards. A rollback that took 45 minutes owes
							     the user an explanation it already has in hand. -->
							<p class="m108-reason">{resumeState.last_error}</p>
						{/if}
					{/if}
				{:else}
					<!-- Phase-4 audit — the journal could not be read: the one state where neither
					     resuming nor restoring can be offered honestly. -->
					<p class="m108-error">{$t('mig108.journalUnreadable') || 'The unification journal could not be read. Nothing will be touched automatically — the backup and journal files are in the universe folder under .constellation; please report this before continuing.'}</p>
				{/if}
				{#if errorText}<p class="m108-error">{errorText}</p>{/if}
				<div class="m108-actions">
					<!-- Phase-4 audit — NEVER wedge the app behind this modal. -->
					<button class="m108-secondary" onclick={dismiss}>{$t('mig108.notNow') || 'Not now'}</button>
					{#if resumeState?.restorable}
						<button class="m108-secondary" onclick={restoreNow}>{$t('mig108.restoreButton') || 'Put everything back'}</button>
					{/if}
					{#if resumeState}
						<button class="m108-primary" onclick={() => runEnvelopeThen('mig108_resume')}>{$t('mig108.resumeButton') || 'Resume and finish'}</button>
					{/if}
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
	/* The journaled failure reason — monospace so a path or a count reads exactly. */
	.m108-reason {
		font-family: var(--font-monospace, ui-monospace, monospace);
		font-size: 0.78rem;
		color: var(--text-muted);
		background: var(--background-secondary, rgba(0,0,0,0.04));
		border-radius: 4px;
		padding: 6px 8px;
		margin-block-start: 4px;
		overflow-wrap: anywhere;
	}
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
