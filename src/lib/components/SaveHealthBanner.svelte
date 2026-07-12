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
	import { saveHealth, retrySaveFailure, saveConflicts, dismissConflict } from '$lib/libraries/store';
	import { openMergeView } from '$lib/stores/mergeView';
	import { invoke } from '@tauri-apps/api/core';
	import { t, dir } from '$lib/i18n';
	import { detectDir } from '$lib/utils';

	const rows = $derived([...$saveHealth.entries()].map(([path, info]) => ({ path, name: info.name })));
	const conflicts = $derived([...$saveConflicts.entries()].map(([sidecarPath, info]) => ({ sidecarPath, name: info.noteName, notePath: info.notePath })));

	function showCopy(sidecarPath: string) {
		invoke('constellation_show_in_folder', { path: sidecarPath }).catch(() => {});
	}
	function merge(c: { sidecarPath: string; name: string; notePath: string }) {
		openMergeView({ notePath: c.notePath, sidecarPath: c.sidecarPath, noteName: c.name });
	}
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
			</div>
		{/each}
		{#each conflicts as c (c.sidecarPath)}
			<div class="shrow cfrow">
				<span class="shicon" aria-hidden="true">⧉</span>
				<span class="shmsg" dir={detectDir(c.name)}>{$t('conflict.externalKept', { note: c.name })}</span>
				<button class="shbtn" type="button" onclick={() => merge(c)}>
					{$t('conflict.merge')}
				</button>
				<button class="shbtn" type="button" onclick={() => showCopy(c.sidecarPath)}>
					{$t('conflict.showCopy')}
				</button>
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
