<script lang="ts">
	// Save-Durability (2026-07-08) — the save-failure surface (Boss: top banner + Retry).
	// Renders one non-blocking row per note whose disk save failed (a .md momentarily
	// locked by a sync tool / antivirus, disk full, offline drive). The user's edit is
	// safe in memory + the write-ahead net; a row auto-dismisses on the next successful
	// save for that path (standardSaveEnv → clearSaveFailure), and Retry re-drives it now.
	// INV-5: no save failure is ever silently swallowed.
	//
	// PJ-070 (2026-07-12) — also renders the EXTERNAL-CONFLICT rows (amber): a note edited
	// outside Constellation while its open model had unsaved edits — the local work is kept
	// and the incoming disk copy is preserved to a `.conflict` sidecar. Distinct from a
	// failure: no auto-clear, no retry — "Show copy" reveals the sidecar, "×" dismisses.
	import { saveHealth, retrySaveFailure, saveConflicts, dismissConflict, saveRecoveredCopy, discardFailedSave, CONFLICT_NO_SIDECAR } from '$lib/libraries/store';
	import { openMergeView } from '$lib/stores/mergeView';
	import { invoke } from '@tauri-apps/api/core';
	import { t, dir } from '$lib/i18n';
	import { detectDir } from '$lib/utils';
	import { onDestroy } from 'svelte';

	const rows = $derived([...$saveHealth.entries()].map(([path, info]) => ({ path, name: info.name })));
	// MIG-111 §0.4 — a row whose sidecar could not be written (the copy lives in a linked universe
	// Constellation may not write to) still appears, because the CONFLICT is real either way. It
	// simply carries no file, so the two actions that need one are not offered.
	const conflicts = $derived([...$saveConflicts.entries()].map(([key, info]) => ({
		sidecarPath: key,
		hasCopy: !key.startsWith(CONFLICT_NO_SIDECAR),
		name: info.noteName,
		notePath: info.notePath,
	})));

	function showCopy(sidecarPath: string) {
		invoke('constellation_show_in_folder', { path: sidecarPath }).catch(() => {});
	}
	function merge(c: { sidecarPath: string; name: string; notePath: string }) {
		openMergeView({ notePath: c.notePath, sidecarPath: c.sidecarPath, noteName: c.name });
	}

	// PJ-102c — the locked-file exits. "Save a copy" writes the unsaved content to a
	// sibling file and opens it (the original keeps retrying). "Discard" is a two-step
	// inline confirm (click once → the button becomes "Really discard?" for 5 s) — an
	// EXPLICIT drop of the unsaved work, the deliberate counterpart of the silent
	// discard the PJ-102 arc eliminated. No native confirm dialogs (MIG-077 ruling).
	let confirmDiscardPath = $state<string | null>(null);
	let confirmTimer: ReturnType<typeof setTimeout> | null = null;
	function askDiscard(path: string) {
		if (confirmDiscardPath === path) {
			if (confirmTimer) { clearTimeout(confirmTimer); confirmTimer = null; }
			confirmDiscardPath = null;
			void discardFailedSave(path);
			return;
		}
		confirmDiscardPath = path;
		if (confirmTimer) clearTimeout(confirmTimer);
		confirmTimer = setTimeout(() => { confirmDiscardPath = null; confirmTimer = null; }, 5000);
	}
	function saveCopy(path: string) {
		void saveRecoveredCopy(path);
	}
	onDestroy(() => { if (confirmTimer) clearTimeout(confirmTimer); });
</script>

{#if rows.length > 0 || conflicts.length > 0}
	<div class="notice-stack" role="alert" aria-live="polite" dir={$dir}>
		{#each rows as row (row.path)}
			<div class="shrow failrow">
				<span class="shicon" aria-hidden="true">⚠</span>
				<span class="shmsg" dir={detectDir(row.name)}>{$t('saveHealth.couldNotSave', { note: row.name })}</span>
				<button class="shbtn" type="button" onclick={() => retrySaveFailure(row.path)}>
					{$t('saveHealth.retry')}
				</button>
				<button class="shbtn" type="button" onclick={() => saveCopy(row.path)}>
					{$t('saveHealth.saveCopy')}
				</button>
				<button class="shbtn" class:sharm={confirmDiscardPath === row.path} type="button" onclick={() => askDiscard(row.path)}>
					{confirmDiscardPath === row.path ? $t('saveHealth.confirmDiscard') : $t('saveHealth.discard')}
				</button>
			</div>
		{/each}
		{#each conflicts as c (c.sidecarPath)}
			<div class="shrow cfrow">
				<span class="shicon" aria-hidden="true">⧉</span>
				<span class="shmsg" dir={detectDir(c.name)}>
					{c.hasCopy
						? $t('conflict.externalKept', { note: c.name })
						: $t('conflict.externalNoCopy', { note: c.name })}
				</span>
				{#if c.hasCopy}
					<button class="shbtn" type="button" onclick={() => merge(c)}>
						{$t('conflict.merge')}
					</button>
					<button class="shbtn" type="button" onclick={() => showCopy(c.sidecarPath)}>
						{$t('conflict.showCopy')}
					</button>
				{/if}
				<button class="shdismiss" type="button" aria-label={$t('conflict.dismiss')} title={$t('conflict.dismiss')} onclick={() => dismissConflict(c.sidecarPath)}>
					×
				</button>
			</div>
		{/each}
	</div>
{/if}

<style>
	.notice-stack {
		position: fixed;
		top: 0;
		inset-inline: 0;
		z-index: 99999;
		display: flex;
		flex-direction: column;
		gap: 1px;
		color: #fff;
		font-size: 13px;
		line-height: 1.4;
		box-shadow: 0 2px 10px rgba(0, 0, 0, 0.3);
	}
	.shrow {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 7px 14px;
	}
	.failrow { background: #b3261e; }  /* save failure — red */
	.cfrow { background: #8a5a00; }    /* external conflict — amber (not an error) */
	.shicon {
		flex: 0 0 auto;
		font-size: 15px;
	}
	.shmsg {
		flex: 1 1 auto;
		min-width: 0;
	}
	.shbtn {
		flex: 0 0 auto;
		background: rgba(255, 255, 255, 0.18);
		color: inherit;
		border: 1px solid rgba(255, 255, 255, 0.45);
		border-radius: 5px;
		padding: 3px 12px;
		cursor: pointer;
		font-size: 12px;
		white-space: nowrap;
	}
	.shbtn:hover {
		background: rgba(255, 255, 255, 0.3);
	}
	/* PJ-102c — the armed discard confirm: unmistakably a destructive second step. */
	.sharm {
		background: #fff;
		color: #b3261e;
		font-weight: 600;
		border-color: #fff;
	}
	.sharm:hover { background: #ffe4e1; }
	.shdismiss {
		flex: 0 0 auto;
		background: transparent;
		color: inherit;
		border: none;
		border-radius: 4px;
		padding: 0 6px;
		cursor: pointer;
		font-size: 18px;
		line-height: 1;
		opacity: 0.85;
	}
	.shdismiss:hover {
		opacity: 1;
		background: rgba(255, 255, 255, 0.2);
	}
</style>
