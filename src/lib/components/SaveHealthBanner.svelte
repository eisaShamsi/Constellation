<script lang="ts">
	// Save-Durability (2026-07-08) — the save-failure surface (Boss: top banner + Retry).
	// Renders one non-blocking row per note whose disk save failed (a .md momentarily
	// locked by a sync tool / antivirus, disk full, offline drive). The user's edit is
	// safe in memory + the write-ahead net; a row auto-dismisses on the next successful
	// save for that path (standardSaveEnv → clearSaveFailure), and Retry re-drives it now.
	// INV-5: no save failure is ever silently swallowed.
	import { saveHealth, retrySaveFailure } from '$lib/libraries/store';
	import { t, dir } from '$lib/i18n';
	import { detectDir } from '$lib/utils';

	const rows = $derived([...$saveHealth.entries()].map(([path, info]) => ({ path, name: info.name })));
</script>

{#if rows.length > 0}
	<div class="save-health" role="alert" aria-live="polite" dir={$dir}>
		{#each rows as row (row.path)}
			<div class="shrow">
				<span class="shicon" aria-hidden="true">⚠</span>
				<span class="shmsg" dir={detectDir(row.name)}>{$t('saveHealth.couldNotSave', { note: row.name })}</span>
				<button class="shretry" type="button" onclick={() => retrySaveFailure(row.path)}>
					{$t('saveHealth.retry')}
				</button>
			</div>
		{/each}
	</div>
{/if}

<style>
	.save-health {
		position: fixed;
		top: 0;
		inset-inline: 0;
		z-index: 99999;
		display: flex;
		flex-direction: column;
		gap: 1px;
		background: #b3261e;
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
	.shicon {
		flex: 0 0 auto;
		font-size: 15px;
	}
	.shmsg {
		flex: 1 1 auto;
		min-width: 0;
	}
	.shretry {
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
	.shretry:hover {
		background: rgba(255, 255, 255, 0.3);
	}
</style>
