<!--
  PJ-207 §10 — THE background-job progress strip. One implementation, three consumers:
  the classifier scan, the NSC summary backfill, and (§11) the index repair.

  This file is deliberately thin: every decision — when to show, when to linger, what a
  mount adopts, what Cancel does to the state — lives in `jobProgressCore.ts`, where the
  tests are. If you are about to add behaviour HERE, it almost certainly belongs there.

  Props name the job:
    eventName      Tauri event carrying { phase, total, completed, error }
    statusCommand  invoke()d once on mount to recover an in-flight job
    cancelCommand  invoke()d when the user presses Cancel
    labelPrefix    i18n namespace with .label/.done/.cancelled/.error/.cancelling/
                   .cancel/.cancelTitle — the keys must exist ×15 locales (parity-gated),
                   which is why there is no `|| 'English'` fallback here: the one the two
                   old copies carried was dead code ($t returns the KEY on a miss, and a
                   key is truthy).

  Replaces ClassifierScanProgressStrip.svelte + NscBackfillProgressStrip.svelte
  (MIG-021v2 §1F' / MIG-040), which were byte-equivalent modulo six identifiers.
  MigrationProgressStrip stays its own component — different event contract.
-->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { t } from '$lib/i18n';
  import {
    createJobStripController,
    HIDDEN,
    type JobProgressEvent,
    type JobStatus,
    type StripState,
  } from './jobProgressCore';

  let {
    eventName,
    statusCommand,
    cancelCommand,
    labelPrefix,
  }: {
    eventName: string;
    statusCommand: string;
    cancelCommand: string;
    labelPrefix: string;
  } = $props();

  // $state.raw: the controller's contract is "one immutable snapshot per change, replaced
  // wholesale" — reassignment triggers reactivity, and no per-event deep proxy is built
  // for granularity nothing uses (property mutation is exactly what the contract forbids).
  let s = $state.raw<StripState>({ ...HIDDEN });
  const ctl = createJobStripController((next) => (s = next));

  let unlisten: UnlistenFn | null = null;
  let destroyed = false;

  const fmt = (n: number) => n.toLocaleString();
  const pct = $derived(s.total > 0 ? Math.floor((s.completed / s.total) * 100) : 0);

  async function cancelJob() {
    ctl.markCancelling();
    try {
      await invoke(cancelCommand);
    } catch (e) {
      console.error(`[${labelPrefix}] cancel failed:`, e);
    }
  }

  onMount(async () => {
    const un = await listen<JobProgressEvent>(eventName, (ev) => ctl.handleEvent(ev.payload));
    // Teardown race, closed (found by the §10 review; both ORIGINAL strips carried it):
    // onMount is async, so a component destroyed before `listen` resolves has already run
    // onDestroy with `unlisten` still null — the listener then registers with nothing left
    // to remove it, for the rest of the session. If we wake up dead, unhook immediately.
    if (destroyed) {
      un();
      return;
    }
    unlisten = un;
    // Recover existing state on mount — covers the case where the user navigated away
    // and back, or the strip mounted late.
    try {
      ctl.adoptStatus(await invoke<JobStatus>(statusCommand));
    } catch (e) {
      console.error(`[${labelPrefix}] initial status fetch failed:`, e);
    }
  });

  onDestroy(() => {
    destroyed = true;
    unlisten?.();
    ctl.destroy();
  });
</script>

{#if s.visible}
  <div class="job-strip" role="status" aria-live="polite">
    <span class="job-label">
      {#if s.phase === 'done'}
        {$t(`${labelPrefix}.done`)}
      {:else if s.phase === 'cancelled'}
        {$t(`${labelPrefix}.cancelled`)}
      {:else if s.phase === 'error'}
        {$t(`${labelPrefix}.error`)}
      {:else if s.cancelling}
        {$t(`${labelPrefix}.cancelling`)}
      {:else}
        {$t(`${labelPrefix}.label`)}
      {/if}
    </span>
    {#if s.phase !== 'done' && s.phase !== 'cancelled' && s.phase !== 'error'}
      <span class="job-counts">— {fmt(s.completed)} / {fmt(s.total)} ({pct}%)</span>
      <button
        class="job-cancel"
        onclick={cancelJob}
        disabled={s.cancelling}
        title={$t(`${labelPrefix}.cancelTitle`)}
      >
        {$t(`${labelPrefix}.cancel`)}
      </button>
    {:else if s.phase !== 'cancelled'}
      <span class="job-counts">— {fmt(s.completed)} / {fmt(s.total)}</span>
    {/if}
  </div>
{/if}

<style>
  .job-strip {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.75rem;
    color: var(--text-muted);
  }
  .job-counts {
    font-variant-numeric: tabular-nums;
  }
  .job-cancel {
    border: 1px solid var(--background-modifier-border, rgba(0, 0, 0, 0.18));
    background: transparent;
    color: var(--text-muted);
    border-radius: 4px;
    padding: 1px 6px;
    font-size: 0.7rem;
    cursor: pointer;
    margin-inline-start: 4px;
  }
  .job-cancel:hover {
    background: var(--background-modifier-hover);
  }
  .job-cancel:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
