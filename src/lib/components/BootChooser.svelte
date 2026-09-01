<script lang="ts">
	// ─── PJ-433 — the Boot Chooser ───
	// The honest answer to a universe that cannot open at boot. The old boot
	// loop silently opened the next registered universe and persisted that
	// substitution as the user's choice; this screen replaces the substitution
	// with a decision: it names the unreachable universe, its location and the
	// reason, lists every registered universe with reachability, and opens
	// NOTHING until the user clicks. A sibling of the UniverseSetup wizard
	// under the same pre-appReady gate — never a mode inside it.
	import { onMount, onDestroy } from 'svelte';
	import { t, dir } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import {
		openExistingUniverse, checkUniverseReachability,
		type UniverseEntry, type UniverseReachability
	} from '$lib/universe/store';

	let {
		failed,
		entries,
		initialError = '',
		onPick,
		onCreateNew,
	}: {
		/** The recorded universe that could not open — null in the pick-one
		 *  state (no active recorded: the A′ remove crash window, or a
		 *  dangling id). */
		failed: { entry: UniverseEntry; error: string } | null;
		entries: UniverseEntry[];
		/** Seed for the inline error when the chooser is REOPENED after a
		 *  post-activation boot-tail failure (2026-08-31 inspection F1) —
		 *  the failing pick's original instance is unmounted by then. */
		initialError?: string;
		/** Runs the FULL boot for the picked entry (RF2 — never a partial
		 *  resume). Resolves to null on success or an error message string;
		 *  on error the chooser stays mounted and shows it inline (RF1). */
		onPick: (entry: UniverseEntry) => Promise<string | null>;
		onCreateNew: () => void;
	} = $props();

	// id → probe result. null until the first probe answers; a probe FAILURE
	// leaves the last known state (degraded mode: rows show no status and
	// every Open stays enabled — a pick of a dead target fails cleanly
	// inline, it never strands the user).
	let reach: Map<string, UniverseReachability> | null = $state(null);
	let busy = $state(false);
	// svelte-ignore state_referenced_locally — the initial capture IS the
	// design: initialError seeds the error of a freshly REMOUNTED chooser
	// (the gate re-creates the component); later pick errors own this state.
	let pickError = $state(initialError);

	// Mount-watch (Boss ruling 2026-08-31): poll while the chooser is open.
	// The poll only refreshes reachability state — it NEVER activates
	// anything; when the missing universe reappears, the "It's back — Open"
	// button lights up and waits for the click. `probing` keeps the probes
	// single-flight (a dead network path can make one probe slow). A stale
	// in-flight result resolving after unmount writes destroyed local $state,
	// which Svelte 5 makes inert — no epoch guard needed.
	let probing = false;
	let pollTimer: ReturnType<typeof setInterval> | undefined;

	async function probe() {
		if (probing) return;
		probing = true;
		try {
			const rows = await checkUniverseReachability();
			reach = new Map(rows.map((r) => [r.id, r]));
		} catch {
			// Degraded mode — keep whatever state we had.
		} finally {
			probing = false;
		}
	}

	onMount(() => {
		void probe();
		pollTimer = setInterval(() => void probe(), 3000);
	});
	onDestroy(() => {
		if (pollTimer) clearInterval(pollTimer);
	});

	// The failed universe came back (drive plugged in / folder restored).
	const failedBack = $derived.by(() => {
		if (!failed || !reach) return false;
		return reach.get(failed.entry.id)?.reachable === true;
	});
	// The list below the banner: every OTHER registered universe (all of
	// them, in the pick-one state).
	const others = $derived.by(() => {
		if (!failed) return entries;
		const fid = failed.entry.id;
		return entries.filter((e) => e.id !== fid);
	});
	const allUnreachable = $derived.by(() => {
		if (!reach || entries.length === 0) return false;
		const r = reach;
		return entries.every((e) => r.get(e.id)?.reachable === false);
	});

	/** The reason line for a universe the probe marked unreachable; falls
	 *  back to the raw activation error for anything unclassified. */
	function reasonText(id: string, rawError: string | null): string {
		const key = reach?.get(id)?.reason;
		if (key === 'not-found') return $t('universe.bootChooser.reasonNotFound');
		if (key === 'not-a-directory') return $t('universe.bootChooser.reasonNotDirectory');
		return rawError ?? '';
	}

	/** The one pick bracket both doors share: busy gate, inline error,
	 *  unmount on success (the post-unmount `busy` write is inert). */
	async function runPick(resolveEntry: () => Promise<UniverseEntry | null>) {
		if (busy) return;
		busy = true;
		pickError = '';
		try {
			const entry = await resolveEntry();
			if (!entry) return; // folder pick cancelled
			const err = await onPick(entry);
			if (err) pickError = $t('universe.bootChooser.pickFailed', { error: err });
		} catch (e: unknown) {
			pickError = $t('universe.bootChooser.pickFailed', { error: e instanceof Error ? e.message : String(e) });
		} finally {
			busy = false;
		}
	}

	const handlePick = (entry: UniverseEntry) => runPick(async () => entry);
	const handleOpenFromFolder = () =>
		runPick(async () => {
			const path = await invoke<string | null>('pick_folder');
			if (!path) return null;
			// The wizard's own Open-Existing door, verbatim: register (never
			// activates — PJ-310), then the full-boot pick path.
			return await openExistingUniverse(path);
		});
</script>

<div class="bc-overlay" dir={$dir}>
	<div class="bc-card">
		{#if failed}
			<div class="bc-banner">
				<div class="bc-banner-title" dir="auto">{$t('universe.bootChooser.couldNotOpen', { name: failed.entry.name })}</div>
				<div class="bc-banner-path"><span class="bc-path-label">{$t('universe.bootChooser.pathLabel')}:</span> <span class="bc-path" dir="ltr">{failed.entry.path}</span></div>
				<div class="bc-banner-reason">{reasonText(failed.entry.id, failed.error)}</div>
				<div class="bc-banner-safe">{$t('universe.bootChooser.nothingChanged')}</div>
				<div class="bc-banner-actions">
					<button
						class="bc-btn"
						class:bc-btn-accent={failedBack}
						disabled={busy}
						onclick={() => handlePick(failed!.entry)}
					>
						{failedBack ? $t('universe.bootChooser.itsBack') : $t('universe.bootChooser.retry')}
					</button>
				</div>
			</div>
		{:else}
			<div class="bc-heading">{$t('universe.bootChooser.title')}</div>
			{#if !initialError}
				<!-- Suppressed on the reopened-after-tail-failure chooser: a choice
				     IS recorded there, and the inline error below tells the truth. -->
				<div class="bc-subheading">{$t('universe.bootChooser.noActiveRecorded')}</div>
			{/if}
		{/if}

		{#if others.length > 0}
			<div class="bc-list">
				{#each others as u (u.id)}
					{@const r = reach?.get(u.id)}
					<div class="bc-entry">
						<div class="bc-entry-info">
							<div class="bc-entry-name" dir="auto">
								{u.name}
								{#if r}
									{#if r.reachable}
										<span class="bc-chip bc-chip-ok">{$t('universe.bootChooser.reachable')}</span>
									{:else}
										<span class="bc-chip bc-chip-bad">{$t('universe.bootChooser.unreachable')}</span>
									{/if}
								{/if}
							</div>
							<div class="bc-entry-path" dir="ltr">{u.path}</div>
							{#if r && !r.reachable}
								<div class="bc-entry-reason">{reasonText(u.id, null)}</div>
							{/if}
						</div>
						<div class="bc-entry-actions">
							<button class="bc-btn bc-btn-accent" disabled={busy || r?.reachable === false} onclick={() => handlePick(u)}>
								{$t('universe.bootChooser.open')}
							</button>
						</div>
					</div>
				{/each}
			</div>
		{/if}

		{#if allUnreachable}
			<div class="bc-all-unreachable">{$t('universe.bootChooser.allUnreachable')}</div>
		{/if}

		{#if pickError}
			<div class="bc-error" dir="auto">{pickError}</div>
		{/if}

		<div class="bc-footer">
			<button class="bc-btn" disabled={busy} onclick={handleOpenFromFolder}>{$t('universe.bootChooser.openFromFolder')}</button>
			<button class="bc-btn" disabled={busy} onclick={onCreateNew}>{$t('universe.bootChooser.createNew')}</button>
		</div>
	</div>
</div>

<style>
	/* Metrics deliberately match UniverseManager's row/button kit (.um-*) so
	   the two surfaces that render the same universes cannot drift visually.
	   A shared row component is the noted follow-up (simplify pass). */
	.bc-overlay {
		position: fixed;
		inset: 0;
		z-index: 9999; /* same layer as the UniverseSetup wizard — the only surfaces alive pre-appReady */
		background: var(--background-primary);
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 24px;
		overflow-y: auto;
	}
	.bc-card {
		width: 100%;
		max-width: 620px;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	.bc-banner {
		border: 1px solid var(--background-modifier-border);
		border-inline-start: 3px solid var(--text-error, #f38ba8);
		border-radius: 10px;
		padding: 14px 16px;
		display: flex;
		flex-direction: column;
		gap: 6px;
		background: var(--background-secondary);
	}
	.bc-banner-title {
		font-size: 1rem;
		font-weight: 700;
		color: var(--text-normal);
	}
	.bc-banner-path {
		font-size: 0.8rem;
		color: var(--text-muted);
	}
	.bc-path-label { font-weight: 600; }
	.bc-path {
		unicode-bidi: embed;
		overflow-wrap: anywhere;
	}
	.bc-banner-reason {
		font-size: 0.85rem;
		color: var(--text-normal);
	}
	.bc-banner-safe {
		font-size: 0.8rem;
		color: var(--text-muted);
	}
	.bc-banner-actions {
		margin-top: 6px;
	}
	.bc-heading {
		font-size: 1.1rem;
		font-weight: 700;
		color: var(--text-normal);
	}
	.bc-subheading {
		font-size: 0.85rem;
		color: var(--text-muted);
	}
	.bc-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.bc-entry {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 10px 12px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
	}
	.bc-entry-info { flex: 1; min-width: 0; }
	.bc-entry-name {
		font-size: 0.88rem;
		font-weight: 600;
		color: var(--text-normal);
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.bc-entry-path {
		font-size: 0.75rem;
		color: var(--text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.bc-entry-reason {
		font-size: 0.75rem;
		color: var(--text-muted);
	}
	.bc-chip {
		font-size: 0.65rem;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		padding: 2px 8px;
		border-radius: 4px;
	}
	.bc-chip-ok { color: white; background: #22c55e; } /* the .um-badge green */
	.bc-chip-bad { color: white; background: var(--text-error, #f38ba8); }
	.bc-entry-actions { flex-shrink: 0; }
	.bc-all-unreachable {
		font-size: 0.85rem;
		color: var(--text-normal);
		padding: 8px 12px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		background: var(--background-secondary);
	}
	.bc-error {
		font-size: 0.8rem;
		color: var(--text-error, #f38ba8);
	}
	.bc-footer {
		display: flex;
		gap: 8px;
	}
	.bc-footer .bc-btn { flex: 1; }
	.bc-btn {
		padding: 4px 12px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		background: none;
		color: var(--text-muted);
		font-size: 0.78rem;
		font-family: inherit;
		cursor: pointer;
	}
	.bc-btn:hover:not(:disabled) { color: var(--text-normal); background: var(--background-modifier-hover); }
	.bc-btn:disabled { opacity: 0.5; cursor: not-allowed; }
	.bc-btn.bc-btn-accent {
		background: var(--interactive-accent);
		color: white;
		border-color: var(--interactive-accent);
	}
	.bc-btn.bc-btn-accent:hover:not(:disabled) { opacity: 0.9; }
</style>
