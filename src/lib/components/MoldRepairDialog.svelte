<script lang="ts">
	/**
	 * PJ-454 — the Mold Repair Door.
	 *
	 * THE CONCEPT (the horse): a one-time review-and-consent surface that shows the exact template
	 * files wrongly carrying a birth date, PROVES on one sample what the two harmless edits are, and
	 * — only on the user's approval — invokes the already-tested engine, so a template stops falsely
	 * claiming a birth date without any file being touched unseen.
	 *
	 * The Constellation Way: the app OBSERVES (the read-only scan), PROPOSES (this dialog — the list,
	 * the before/after, the delicate files flagged), and the USER decides. Nothing is written without
	 * the Repair click, and per-universe enumerate-then-consent is the non-negotiable invariant — no
	 * automatic rule ever repairs a file (the Boss's ruling: a misidentified real note is stripped of
	 * its earned history silently and permanently).
	 *
	 * SCOPE HONESTY: the scan covers the active universe's OWN libraries only — never a linked
	 * universe's. A template repair is a WRITE, and write sovereignty (MIG-111) keeps a linked
	 * universe's bookkeeping in ITS OWN database, which this window cannot touch. So a mold in
	 * another universe is repaired when THAT universe is the active one — the door shows whatever
	 * THIS universe holds, never a hard-coded 43.
	 *
	 * SAFETY THE DOOR ENFORCES (the engine cannot): (1) refuse to run while any candidate is open in
	 * a tab, re-checked at click-time — a dirty open mold's debounced save fires AFTER the engine
	 * verifies and the write gate lets it through, silently re-stamping; (2) re-scan before each
	 * batch; (3) run the delicate files as their own first batch and HALT the cascade if it reports
	 * any failure, so a systematic problem never reaches the ordinary files.
	 */
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { t, tn } from '$lib/i18n';
	import { openTabs, moldRepairRunning } from '$lib/libraries/store';

	interface MoldCandidate {
		path: string;
		library: string;
		cid_cn: string;
		stamp_date: string;
		syntax: string;
		inbound_cid_links: number;
	}
	interface MoldPreview {
		path: string;
		will_apply: boolean;
		removes: string;
		adds: string;
		reason: string;
		needs_care: boolean;
	}
	// PJ-466 — the engine classifies each file instead of returning `ok: boolean`. `kept` is a file
	// that needed nothing (never an error, never halts); `failed` is one that still carries the
	// false birth date. `reason` is a STABLE code this dialog translates — never render it raw.
	type MoldOutcomeKind = 'repaired' | 'kept' | 'failed';
	interface MoldRepairOutcome {
		path: string;
		kind: MoldOutcomeKind;
		reason: string;
		detail: string;
		backed_up: boolean;
	}
	interface MoldRepairReport {
		attempted: number;
		repaired: number;
		kept: number;
		failed: number;
		/** How many files actually got a verified backup — so the receipt can stop assuming. */
		backed_up: number;
		backup_dir: string;
		outcomes: MoldRepairOutcome[];
		relinked_sources: string[];
	}

	let { onDone, onDismiss }: { onDone?: () => void; onDismiss?: () => void } = $props();

	let visible = $state(false);
	let mode = $state<'review' | 'blocked' | 'running' | 'summary'>('review');
	let candidates = $state<MoldCandidate[]>([]);
	let previews = $state<Map<string, MoldPreview>>(new Map());
	let sample = $state<MoldPreview | null>(null);
	let openBlockers = $state<string[]>([]);
	let phaseLabel = $state('');
	let errorText = $state<string | null>(null);
	// PJ-466 — WHY a run ended early, as a calm lede above the counts. One of 'care' | 'open' |
	// 'error', or null when the run finished. Never combined with `errorText`: the old blocker path
	// set a red "try again" line saying the same thing in alarm colour, pointing at a button that
	// no longer exists.
	let stoppedReason = $state<'care' | 'open' | 'error' | null>(null);
	let stoppedDetail = $state('');
	// Paths the halt meant we never submitted. They produce no engine outcome, so the receipt would
	// have a count with nothing under it — the dialog synthesises their rows.
	let notReached = $state<string[]>([]);
	let outcomes = $state<MoldRepairOutcome[]>([]);
	let report = $state<MoldRepairReport | null>(null);
	// Safety inspection (re-run ×2, LOW concurrency-race): a re-entrancy latch. Without it a
	// double-click on Repair during the preview await launches TWO engine runs; the loser's refusal
	// flips mode to 'summary' while the winner is still writing, which drops `moldRepairRunning`
	// and lifts every running-mode guard for the rest of a live run. Held across the whole of
	// startRepair, cleared in `finally` on every exit path; the buttons are disabled while set.
	let busy = $state(false);

	// The last path segment — what the user sees in the File Explorer.
	function baseName(p: string): string {
		const parts = p.replace(/\\/g, '/').split('/');
		return parts[parts.length - 1] || p;
	}
	// Normalize a path for a separator-/case-insensitive compare (Windows): the engine reports
	// PathBuf.to_string_lossy(), the frontend holds tab strings — they can differ only in these.
	function norm(p: string): string { return p.replace(/\\/g, '/').toLowerCase(); }

	// Group candidates by their library name (the scan's own grouping), preserving scan order.
	const groups = $derived.by(() => {
		const m = new Map<string, MoldCandidate[]>();
		for (const c of candidates) {
			const arr = m.get(c.library) ?? [];
			arr.push(c);
			m.set(c.library, arr);
		}
		return [...m.entries()];
	});

	onMount(async () => {
		try {
			candidates = await invoke<MoldCandidate[]>('scan_stamped_molds');
		} catch (e) {
			// A scan that cannot run is not a repair that should offer itself. Stay closed, tell no lie.
			candidates = [];
		}
		if (candidates.length === 0) { onDone?.(); return; }
		visible = true;
	});

	// Which of the currently-scanned candidates are open in a tab right now.
	function currentOpenBlockers(): string[] {
		const open = new Set(get(openTabs).map((tb) => norm(tb.path)));
		return candidates.filter((c) => open.has(norm(c.path))).map((c) => c.path);
	}

	async function showSample() {
		errorText = null;
		try {
			const pv = await invoke<MoldPreview[]>('preview_mold_repair', { paths: candidates.map((c) => c.path) });
			const map = new Map<string, MoldPreview>();
			for (const p of pv) map.set(p.path, p);
			previews = map;
			// Prefer a delicate file as the sample so the user sees the trickiest case proven.
			sample = pv.find((p) => p.needs_care && p.will_apply) ?? pv.find((p) => p.will_apply) ?? pv[0] ?? null;
		} catch (e) {
			errorText = e instanceof Error ? e.message : String(e);
		}
	}

	async function startRepair() {
		if (busy) return;
		busy = true;
		try {
			await startRepairInner();
		} finally {
			busy = false;
		}
	}

	async function startRepairInner() {
		errorText = null;
		// (1) Open-tab hard block, re-checked at THIS click — not at scan time.
		openBlockers = currentOpenBlockers();
		if (openBlockers.length > 0) { mode = 'blocked'; return; }

		// Ensure we have previews (for the delicate/ordinary split); compute if the user skipped the sample.
		if (previews.size === 0) { await showSample(); if (errorText) return; }
		// Re-inspection ×5: a dismissed dialog must never launch the engine. `busy` now refuses close
		// across this await, so this is defense in depth against any other unmount path.
		if (!visible) return;

		// PJ-460 (panel): the await above is a window. With a clickable app behind the blocked
		// screen, a candidate opened DURING that wait would otherwise be repaired while open —
		// exactly the silent re-stamp the block exists to prevent. Re-check after every await.
		openBlockers = currentOpenBlockers();
		if (openBlockers.length > 0) { mode = 'blocked'; return; }

		// PJ-469 is fixed in the ENGINE, not here: `repair_one` now proves the edit verifies before
		// it writes anything, so a file that cannot be repaired costs no write. An earlier version
		// filtered those files out in this component using the preview — a judgement made minutes
		// earlier, in the layer with the least information, which could label a file "kept ·
		// already fixed" after it had been re-stamped. Everything the user approved is submitted,
		// and the engine's fresh re-proof is the only verdict on the receipt.
		const submit = candidates.map((c) => c.path);
		const delicate = submit.filter((p) => previews.get(p)?.needs_care);
		const ordinary = submit.filter((p) => !previews.get(p)?.needs_care);

		mode = 'running';
		outcomes = [];
		notReached = [];
		stoppedReason = null;
		stoppedDetail = '';
		try {
			// Batch 1 — the delicate files first. Halt the whole cascade if any of them FAILS.
			// `failed` is now class-counted, so a file that simply needed nothing cannot halt it.
			if (delicate.length > 0) {
				phaseLabel = $t('moldRepair.phaseDelicate');
				const r1 = await invoke<MoldRepairReport>('repair_stamped_molds', { paths: delicate });
				if (!visible) return; // re-inspection ×5: never continue on a dismissed dialog
				outcomes = [...outcomes, ...r1.outcomes];
				report = r1;
				if (r1.failed > 0) {
					phaseLabel = '';
					stoppedReason = 'care';
					notReached = ordinary;
					mode = 'summary';
					return; // do not touch the ordinary files if the tricky ones failed
				}
			}
			// PJ-460: a blocker that appeared between the batches halts the cascade the same way a
			// failed delicate batch does — nothing in batch 2 is touched.
			if (ordinary.length > 0 && currentOpenBlockers().length > 0) {
				phaseLabel = '';
				stoppedReason = 'open';
				notReached = ordinary;
				mode = 'summary';
				return;
			}
			// Batch 2 — the ordinary files. Re-scan is implicit: the engine re-proves each file.
			if (ordinary.length > 0) {
				phaseLabel = $t('moldRepair.phaseOrdinary');
				const r2 = await invoke<MoldRepairReport>('repair_stamped_molds', { paths: ordinary });
				if (!visible) return; // re-inspection ×5: never continue on a dismissed dialog
				outcomes = [...outcomes, ...r2.outcomes];
				report = mergeReports(report, r2);
			}
			phaseLabel = '';
			mode = 'summary';
		} catch (e) {
			// Whatever was not submitted when the throw happened was not reached.
			const done = new Set(outcomes.map((o) => o.path));
			notReached = submit.filter((p) => !done.has(p));
			stoppedReason = 'error';
			stoppedDetail = e instanceof Error ? e.message : String(e);
			phaseLabel = '';
			mode = 'summary';
		}
	}

	// One receipt row per file the run never reached. The engine produced no outcome for these —
	// without them the "Did not reach N" line would be a count with nothing beneath it.
	const receiptRows = $derived([
		...outcomes,
		...notReached.map((path) => ({
			path,
			kind: 'failed' as MoldOutcomeKind,
			reason: 'notReached',
			detail: '',
			backed_up: false,
		})),
	]);

	// The counts the summary speaks from. `notReached` is the dialog's own, not the engine's.
	// Counted from `outcomes`, which already holds BOTH the engine's rows and the ones the door
	// synthesised for files it deliberately did not submit. Deriving from `report` instead would
	// undercount by exactly the pre-refused files — and counting each class on its own terms,
	// rather than by subtraction, is the whole point of this job.
	const sums = $derived({
		fixed: outcomes.filter((o) => o.kind === 'repaired').length,
		kept: outcomes.filter((o) => o.kind === 'kept').length,
		failed: outcomes.filter((o) => o.kind === 'failed').length,
		notReached: notReached.length,
	});

	// The raw engine text is shown ONLY where it carries an OS error the user could act on.
	// Elsewhere it is the same sentence in English, and an untranslated echo under a translated
	// line makes the screen read half-finished in Arabic, Hebrew, Persian and Urdu.
	const TAIL_CODES = new Set([
		'unreadable', 'backupFailed', 'writeFailed', 'verifyFailed', 'verifyNotRestored', 'repairedIndexLater',
	]);
	// Wrap an unknown-direction run (a raw OS error) so it cannot reorder the Arabic or Hebrew
	// sentence it sits inside. FIRST STRONG ISOLATE … POP DIRECTIONAL ISOLATE is the Unicode
	// mechanism for exactly this, and it works without splitting the translated string — which is
	// why the path in `backupAt` could be lifted out of its sentence and this one cannot.
	function isolate(s: string): string {
		// Built from char codes deliberately: a literal U+2068/U+2069 in source is invisible and
		// Svelte flags it, for good reason — bidi controls in code can hide what the code does.
		return String.fromCharCode(0x2068) + s + String.fromCharCode(0x2069);
	}

	function glyph(kind: MoldOutcomeKind): string {
		// A Kept file is not a failure and must not wear a cross.
		return kind === 'repaired' ? '\u2713' : kind === 'kept' ? '\u2022' : '\u2715';
	}

	function mergeReports(a: MoldRepairReport | null, b: MoldRepairReport): MoldRepairReport {
		if (!a) return b;
		return {
			attempted: a.attempted + b.attempted,
			repaired: a.repaired + b.repaired,
			kept: a.kept + b.kept,
			failed: a.failed + b.failed,
			backed_up: a.backed_up + b.backed_up,
			backup_dir: a.backup_dir || b.backup_dir,
			outcomes: [...a.outcomes, ...b.outcomes],
			relinked_sources: [...new Set([...a.relinked_sources, ...b.relinked_sources])],
		};
	}

	// Safety inspection (PJ-460 diff, MED): closing while the engine is mid-run unmounts the dialog,
	// hides the receipt, refreshes the banner count against a half-written universe, and — worst —
	// voids the open-tab invariant for the rest of the run (a candidate opened after ✕ is repaired
	// underneath its tab and the tab's next save silently re-stamps it). The Rust loop has no
	// cancellation, so the only honest behaviour is: while running, the dialog cannot be closed.
	function close() {
		// Re-inspection ×5 (MED): the invariant holds while BUSY, not only while running. The busy
		// window before running (the preview read) left ✕/Cancel live; a Cancel there unmounted the
		// dialog and the pending work resumed on the destroyed component and launched the engine
		// anyway — files modified after an explicit Cancel, with every guard torn down.
		if (mode === 'running' || busy) return;
		visible = false; onDismiss?.(); onDone?.();
	}

	// Safety inspection (PJ-460 diff, LOW): the running curtain blocks the MOUSE, not the keyboard —
	// a quick-switcher / command-palette shortcut mid-batch can open a candidate with pre-repair
	// bytes, which is then repaired underneath its tab and un-repaired by its next save. While the
	// engine runs, swallow shortcut keys (any modifier combo, and Escape) at the WINDOW in capture
	// phase — that precedes +layout's document-capture handler in the event path (verified:
	// `document.addEventListener('keydown', handleGlobalKeydown, true)`). Plain typing is untouched;
	// nothing behind a running modal should be receiving it anyway. Released the moment running ends.
	// Safety inspection (re-run, MED cross-window-clobber): the curtain and the keydown swallow reach
	// only THIS webview. A click in the second screen emits `screen:open-in-main`, which the main
	// window honours — so the real gate lives where every tab mounts, `openNoteTab`, keyed on this
	// shared signal. Mirror the running state into it; the cleanup resets it on every exit, including
	// an unmount mid-run, so a stale `true` can never lock note-opening after the dialog is gone.
	$effect(() => {
		moldRepairRunning.set(mode === 'running');
		return () => moldRepairRunning.set(false);
	});

	$effect(() => {
		if (mode !== 'running') return;
		// Re-inspection ×4 (LOW): modifier-only swallowing missed a nav key the user re-mapped to a
		// bare F-key or Shift+F-key, and Tab-focus travelling to a button behind the curtain. Running
		// mode renders NO interactive control (the ✕ is hidden, there are no buttons — only a spinner),
		// so for the seconds the engine writes there is nothing a key could legitimately do here.
		// Swallow every keydown: that closes bare F-keys, Shift-combos and Tab-focus travel in one rule.
		const swallow = (e: KeyboardEvent) => {
			e.stopPropagation();
			e.preventDefault();
		};
		window.addEventListener('keydown', swallow, true);
		return () => window.removeEventListener('keydown', swallow, true);
	});
</script>

{#if visible}
<!-- PJ-460 (the Boss found it): on the BLOCKED screen the curtain must not swallow clicks, or the
     user cannot close the offending tab behind it and "try again" can never do its job. The curtain
     was never the safety device (Ctrl+W already reached the tab through it); the safety is the
     click-time re-check in startRepair, which stays. Precedent: StyleSetter's `--live` overlay. -->
<div class="mr-overlay" class:mr-overlay--porous={mode === 'blocked'} role="dialog" aria-modal={mode !== 'blocked'} aria-label={$t('moldRepair.title')}>
	<div class="mr-card" dir="auto">
		{#if mode !== 'running' && !busy}
			<button class="mr-x" onclick={close} aria-label={$t('common.close')}>✕</button>
		{/if}

		{#if mode === 'review'}
			<h2 class="mr-title">{$t('moldRepair.title')}</h2>
			<p class="mr-lede">{$t('moldRepair.lede')}</p>
			<p class="mr-count">{$t('moldRepair.count', { noun: $tn('plurals.templates', candidates.length) })}</p>

			<div class="mr-list">
				{#each groups as [lib, rows]}
					<div class="mr-group-head">{lib} <span class="mr-badge">{rows.length}</span></div>
					{#each rows as c}
						<div class="mr-row">
							<div class="mr-name">
								{baseName(c.path)}
								{#if previews.get(c.path)?.needs_care}<span class="mr-care" title={$t('moldRepair.needsCareHint')}>{$t('moldRepair.needsCare')}</span>{/if}
							</div>
							<div class="mr-harm">
								{$t('moldRepair.claimsBorn', { date: c.stamp_date })}
								{#if c.inbound_cid_links > 0}
									· {$t('moldRepair.inbound', { noun: $tn('plurals.notes', c.inbound_cid_links) })}
								{/if}
							</div>
						</div>
					{/each}
				{/each}
			</div>

			{#if sample}
				<div class="mr-sample">
					<div class="mr-sample-head">{$t('moldRepair.sampleHead', { name: baseName(sample.path) })}</div>
					<div class="mr-diff"><span class="mr-del">− {sample.removes}</span></div>
					<div class="mr-diff"><span class="mr-add">+ {sample.adds}</span></div>
					<div class="mr-sample-foot">{$t('moldRepair.sampleFoot')}</div>
				</div>
			{:else}
				<button class="mr-secondary" onclick={showSample}>{$t('moldRepair.showSample')}</button>
			{/if}

			{#if errorText}<p class="mr-err">{errorText}</p>{/if}

			<div class="mr-actions">
				<button class="mr-cancel" onclick={close} disabled={busy}>{$t('common.cancel')}</button>
				<button class="mr-go" onclick={startRepair} disabled={busy}>{$t('moldRepair.repair', { noun: $tn('plurals.templates', candidates.length) })}</button>
			</div>
		{:else if mode === 'blocked'}
			<h2 class="mr-title">{$t('moldRepair.blockedTitle')}</h2>
			<p class="mr-lede">{$t('moldRepair.blockedLede')}</p>
			<ul class="mr-blocked">
				{#each openBlockers as p}<li>{baseName(p)}</li>{/each}
			</ul>
			<div class="mr-actions">
				<button class="mr-cancel" onclick={close} disabled={busy}>{$t('common.close')}</button>
				<button class="mr-go" onclick={() => { openBlockers = currentOpenBlockers(); if (openBlockers.length === 0) startRepair(); }} disabled={busy}>{$t('moldRepair.recheck')}</button>
			</div>
		{:else if mode === 'running'}
			<h2 class="mr-title">{$t('moldRepair.runningTitle')}</h2>
			<p class="mr-lede">{phaseLabel}</p>
			<div class="mr-spinner" aria-hidden="true"></div>
		{:else if mode === 'summary'}
			<h2 class="mr-title">{$t('moldRepair.summaryTitle')}</h2>

			<!-- WHY it ended early, first and calmly, before any number. A ladder of titles can be
			     ordered wrongly - which is exactly how "Templates fixed" came to sit over a run that
			     fixed nothing - so the title never changes and this lede does the work. -->
			{#if stoppedReason === 'care'}
				<p class="mr-lede">{$t('moldRepair.stoppedCare')}</p>
			{:else if stoppedReason === 'open'}
				<p class="mr-lede">{$t('moldRepair.stoppedOpen')}</p>
			{:else if stoppedReason === 'error'}
				<p class="mr-lede">{$t('moldRepair.stoppedError', { reason: isolate(stoppedDetail) })}</p>
			{/if}

			<!-- One sentence per NON-ZERO class. A class line must never render at 0: "Kept 0 template
			     files" is noise in English and ungrammatical in Arabic. -->
			{#if sums.fixed > 0}<p class="mr-sum">{$t('moldRepair.sumFixed', { noun: $tn('plurals.templates', sums.fixed) })}</p>{/if}
			{#if sums.kept > 0}<p class="mr-sum">{$t('moldRepair.sumKept', { noun: $tn('plurals.templates', sums.kept) })}</p>{/if}
			{#if sums.failed > 0}<p class="mr-sum mr-sum-bad">{$t('moldRepair.sumFailed', { noun: $tn('plurals.templates', sums.failed) })}</p>{/if}
			{#if sums.notReached > 0}<p class="mr-sum">{$t('moldRepair.sumNotAttempted', { noun: $tn('plurals.templates', sums.notReached) })}</p>{/if}

			<!-- The way back, whenever anything is unfinished. NOT the notice bar: it is dismissible
			     and is not restored until a universe switch, so a user who closed it would be sent
			     somewhere that no longer exists. -->
			{#if stoppedReason !== null || sums.failed > 0 || sums.notReached > 0}
				<p class="mr-count">{$t('moldRepair.resume')}</p>
			{/if}

			{#if report}
				<!-- Gated on the COUNT, never on `backup_dir` being non-empty: the folder is created
				     before any file is touched, so the path is always present even when nothing was
				     backed up. The path leaves the sentence and is isolated, or a Windows path reorders
				     on screen inside an Arabic or Hebrew line. -->
				{#if report.backed_up > 0}
					<p class="mr-count">
						{$t('moldRepair.backupAt')}
						<span class="mr-path" dir="auto">{report.backup_dir}</span>
					</p>
				{/if}
				{#if report.relinked_sources.length > 0}
					<p class="mr-count">{$t('moldRepair.relinked', { noun: $tn('plurals.notes', report.relinked_sources.length) })}</p>
				{/if}
			{/if}
			<div class="mr-list mr-receipt">
				{#each receiptRows as o}
					<div class="mr-row">
						<div class="mr-name">{glyph(o.kind)} <span dir="auto">{baseName(o.path)}</span></div>
						<div class="mr-harm">
							{$t('moldRepair.reason.' + o.reason)}
							{#if o.detail && TAIL_CODES.has(o.reason)}
								<span class="mr-tail"> — <span class="mr-iso" dir="auto">{o.detail}</span></span>
							{/if}
						</div>
					</div>
				{/each}
			</div>
			<div class="mr-actions">
				<!-- "Done" under a receipt that says nothing could be fixed is the same flattery as the
				     old title. The blocked screen in this dialog already says Close. -->
				<button class="mr-go" onclick={close}>{$t('common.close')}</button>
			</div>
		{/if}
	</div>
</div>
{/if}

<style>
	.mr-overlay {
		position: fixed; inset: 0; z-index: 4000;
		background: color-mix(in srgb, var(--background-primary) 55%, transparent);
		display: flex; align-items: center; justify-content: center; padding: 24px;
	}
	/* PJ-460 — blocked screen only: faintly dimmed (Boss-ruled) but CLICKABLE behind the card, so the
	   tab strip's × is reachable. The card itself keeps taking clicks. */
	.mr-overlay--porous { background: color-mix(in srgb, var(--background-primary) 18%, transparent); pointer-events: none; }
	.mr-overlay--porous .mr-card { pointer-events: auto; }
	.mr-card {
		background: var(--background-primary); color: var(--text-normal);
		border: 1px solid var(--background-modifier-border); border-radius: 10px;
		width: min(680px, 96vw); max-height: 88vh; overflow-y: auto;
		padding: 22px 24px; position: relative; box-shadow: 0 12px 40px rgba(0,0,0,0.4);
	}
	.mr-x { position: absolute; top: 12px; inset-inline-end: 12px; background: none; border: none;
		color: var(--text-muted); font-size: 15px; cursor: pointer; }
	.mr-title { font-size: 1.15rem; margin: 0 0 8px; }
	.mr-lede { color: var(--text-muted); margin: 0 0 6px; font-size: 0.9rem; }
	.mr-count { color: var(--text-muted); margin: 4px 0; font-size: 0.85rem; }
	.mr-list { border: 1px solid var(--background-modifier-border); border-radius: 8px;
		margin: 12px 0; max-height: 320px; overflow-y: auto; }
	.mr-receipt { max-height: 240px; }
	.mr-group-head { padding: 8px 12px; background: var(--background-secondary);
		font-weight: 600; font-size: 0.85rem; position: sticky; top: 0; }
	.mr-badge { display: inline-block; background: var(--background-modifier-border);
		border-radius: 10px; padding: 0 8px; font-size: 0.75rem; margin-inline-start: 6px; }
	.mr-row { padding: 7px 12px; border-top: 1px solid var(--background-modifier-border); }
	.mr-name { font-size: 0.88rem; }
	.mr-care { display: inline-block; background: color-mix(in srgb, var(--text-warning) 22%, transparent);
		color: var(--text-normal); border-radius: 4px; padding: 0 6px; font-size: 0.72rem; margin-inline-start: 6px; }
	.mr-harm { font-size: 0.78rem; color: var(--text-muted); margin-top: 2px; }
	.mr-sample { border: 1px solid var(--background-modifier-border); border-radius: 8px; padding: 10px 12px; margin: 10px 0; }
	.mr-sample-head { font-size: 0.82rem; color: var(--text-muted); margin-bottom: 6px; }
	.mr-diff { font-family: var(--font-monospace-theme); font-size: 0.82rem; }
	.mr-del { color: var(--text-error); }
	.mr-add { color: var(--text-success); }
	.mr-sample-foot { font-size: 0.76rem; color: var(--text-muted); margin-top: 6px; }
	.mr-secondary { background: none; border: 1px solid var(--background-modifier-border);
		color: var(--text-normal); border-radius: 6px; padding: 6px 12px; cursor: pointer; font-size: 0.82rem; }
	.mr-blocked { margin: 8px 0; padding-inline-start: 22px; font-size: 0.85rem; }
	.mr-err { color: var(--text-error); font-size: 0.82rem; margin: 8px 0; }
	/* One sentence per outcome class — the shape the phantom-removal receipt already uses. */
	.mr-sum { margin: 4px 0; font-size: 0.88rem; }
	.mr-sum-bad { color: var(--text-error); }
	/* A path or an OS error inside a translated sentence: isolate it, or a Latin run with
	   backslashes reorders visually inside an Arabic or Hebrew line. `auto`, never `ltr` — the
	   same rows carry file names the user wrote in Arabic. */
	.mr-path, .mr-iso { unicode-bidi: isolate; }
	.mr-path { font-family: var(--font-monospace-theme); font-size: 0.8rem; word-break: break-all; }
	.mr-tail { color: var(--text-faint); }
	.mr-actions { display: flex; justify-content: flex-end; gap: 10px; margin-top: 16px; }
	.mr-cancel, .mr-go { border-radius: 6px; padding: 8px 16px; cursor: pointer; font-size: 0.88rem; border: 1px solid transparent; }
	.mr-cancel { background: none; border-color: var(--background-modifier-border); color: var(--text-normal); }
	.mr-go { background: var(--interactive-accent); color: #fff; }
	.mr-spinner { width: 26px; height: 26px; margin: 16px auto; border: 3px solid var(--background-modifier-border);
		border-top-color: var(--interactive-accent); border-radius: 50%; animation: mr-spin 0.8s linear infinite; }
	@keyframes mr-spin { to { transform: rotate(360deg); } }
</style>
