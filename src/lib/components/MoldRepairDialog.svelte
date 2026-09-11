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
	import { t } from '$lib/i18n';
	import { openTabs } from '$lib/libraries/store';

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
	interface MoldRepairOutcome { path: string; ok: boolean; detail: string; }
	interface MoldRepairReport {
		attempted: number;
		repaired: number;
		failed: number;
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
	let outcomes = $state<MoldRepairOutcome[]>([]);
	let report = $state<MoldRepairReport | null>(null);

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
		errorText = null;
		// (1) Open-tab hard block, re-checked at THIS click — not at scan time.
		openBlockers = currentOpenBlockers();
		if (openBlockers.length > 0) { mode = 'blocked'; return; }

		// Ensure we have previews (for the delicate/ordinary split); compute if the user skipped the sample.
		if (previews.size === 0) { await showSample(); if (errorText) return; }

		const delicate = candidates.map((c) => c.path).filter((p) => previews.get(p)?.needs_care);
		const ordinary = candidates.map((c) => c.path).filter((p) => !previews.get(p)?.needs_care);

		mode = 'running';
		outcomes = [];
		try {
			// Batch 1 — the delicate files first. Halt the whole cascade if any of them fails.
			if (delicate.length > 0) {
				phaseLabel = $t('moldRepair.phaseDelicate');
				const r1 = await invoke<MoldRepairReport>('repair_stamped_molds', { paths: delicate });
				outcomes = [...outcomes, ...r1.outcomes];
				report = r1;
				if (r1.failed > 0) {
					phaseLabel = '';
					mode = 'summary';
					return; // do not touch the ordinary files if the tricky ones failed
				}
			}
			// Batch 2 — the ordinary files. Re-scan is implicit: the engine re-proves each file.
			if (ordinary.length > 0) {
				phaseLabel = $t('moldRepair.phaseOrdinary');
				const r2 = await invoke<MoldRepairReport>('repair_stamped_molds', { paths: ordinary });
				outcomes = [...outcomes, ...r2.outcomes];
				report = mergeReports(report, r2);
			}
			phaseLabel = '';
			mode = 'summary';
		} catch (e) {
			errorText = e instanceof Error ? e.message : String(e);
			phaseLabel = '';
			mode = 'summary';
		}
	}

	function mergeReports(a: MoldRepairReport | null, b: MoldRepairReport): MoldRepairReport {
		if (!a) return b;
		return {
			attempted: a.attempted + b.attempted,
			repaired: a.repaired + b.repaired,
			failed: a.failed + b.failed,
			backup_dir: a.backup_dir || b.backup_dir,
			outcomes: [...a.outcomes, ...b.outcomes],
			relinked_sources: [...new Set([...a.relinked_sources, ...b.relinked_sources])],
		};
	}

	function close() { visible = false; onDismiss?.(); onDone?.(); }
</script>

{#if visible}
<div class="mr-overlay" role="dialog" aria-modal="true" aria-label={$t('moldRepair.title')}>
	<div class="mr-card" dir="auto">
		<button class="mr-x" onclick={close} aria-label={$t('common.close')}>✕</button>

		{#if mode === 'review'}
			<h2 class="mr-title">{$t('moldRepair.title')}</h2>
			<p class="mr-lede">{$t('moldRepair.lede')}</p>
			<p class="mr-count">{$t('moldRepair.count', { count: String(candidates.length) })}</p>

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
									· {$t('moldRepair.inbound', { count: String(c.inbound_cid_links) })}
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
				<button class="mr-cancel" onclick={close}>{$t('common.cancel')}</button>
				<button class="mr-go" onclick={startRepair}>{$t('moldRepair.repair', { count: String(candidates.length) })}</button>
			</div>
		{:else if mode === 'blocked'}
			<h2 class="mr-title">{$t('moldRepair.blockedTitle')}</h2>
			<p class="mr-lede">{$t('moldRepair.blockedLede')}</p>
			<ul class="mr-blocked">
				{#each openBlockers as p}<li>{baseName(p)}</li>{/each}
			</ul>
			<div class="mr-actions">
				<button class="mr-cancel" onclick={close}>{$t('common.close')}</button>
				<button class="mr-go" onclick={() => { openBlockers = currentOpenBlockers(); if (openBlockers.length === 0) startRepair(); }}>{$t('moldRepair.recheck')}</button>
			</div>
		{:else if mode === 'running'}
			<h2 class="mr-title">{$t('moldRepair.runningTitle')}</h2>
			<p class="mr-lede">{phaseLabel}</p>
			<div class="mr-spinner" aria-hidden="true"></div>
		{:else if mode === 'summary'}
			<h2 class="mr-title">{$t('moldRepair.summaryTitle')}</h2>
			{#if report}
				<p class="mr-lede">{$t('moldRepair.summaryLine', { repaired: String(report.repaired), failed: String(report.failed) })}</p>
				<p class="mr-count">{$t('moldRepair.backupAt', { dir: report.backup_dir })}</p>
				{#if report.relinked_sources.length > 0}
					<p class="mr-count">{$t('moldRepair.relinked', { count: String(report.relinked_sources.length) })}</p>
				{/if}
			{/if}
			{#if errorText}<p class="mr-err">{errorText}</p>{/if}
			<div class="mr-list mr-receipt">
				{#each outcomes as o}
					<div class="mr-row"><div class="mr-name">{o.ok ? '✓' : '✕'} {baseName(o.path)}</div><div class="mr-harm">{o.detail}</div></div>
				{/each}
			</div>
			<div class="mr-actions">
				<button class="mr-go" onclick={close}>{$t('moldRepair.done')}</button>
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
	.mr-actions { display: flex; justify-content: flex-end; gap: 10px; margin-top: 16px; }
	.mr-cancel, .mr-go { border-radius: 6px; padding: 8px 16px; cursor: pointer; font-size: 0.88rem; border: 1px solid transparent; }
	.mr-cancel { background: none; border-color: var(--background-modifier-border); color: var(--text-normal); }
	.mr-go { background: var(--interactive-accent); color: #fff; }
	.mr-spinner { width: 26px; height: 26px; margin: 16px auto; border: 3px solid var(--background-modifier-border);
		border-top-color: var(--interactive-accent); border-radius: 50%; animation: mr-spin 0.8s linear infinite; }
	@keyframes mr-spin { to { transform: rotate(360deg); } }
</style>
