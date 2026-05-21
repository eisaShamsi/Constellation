<!--
  MIG-040 — Status-bar strip for the background NSC summary backfill.

  Shows running counts via `nsc:backfill` events emitted by the Rust
  nsc::backfill worker. Hidden until a `start` event arrives (or until mount
  finds an in-progress backfill via nsc_backfill_status). Hides 4 seconds
  after `done` / `cancelled` / `error`.

  Deliberately mirrors ClassifierScanProgressStrip — same visual language,
  same recover-on-mount pattern, same event shape.
-->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { t } from '$lib/i18n';

  type Phase = 'start' | 'progress' | 'done' | 'cancelled' | 'error';

  interface BackfillEvent {
    phase: Phase;
    total: number;
    completed: number;
    error: string | null;
  }

  interface BackfillStatus {
    running: boolean;
    cancelling: boolean;
    completed: number;
    total: number;
    last_error: string | null;
  }

  let visible = $state(false);
  let phase = $state<Phase | null>(null);
  let total = $state(0);
  let completed = $state(0);
  let cancelling = $state(false);

  let hideTimer: ReturnType<typeof setTimeout> | null = null;
  let unlisten: UnlistenFn | null = null;

  const fmt = (n: number) => n.toLocaleString();
  const pct = $derived(total > 0 ? Math.floor((completed / total) * 100) : 0);

  async function cancelBackfill() {
    cancelling = true;
    try {
      await invoke('nsc_backfill_cancel');
    } catch (e) {
      console.error('[NscBackfill] cancel failed:', e);
    }
  }

  function handleEvent(p: BackfillEvent) {
    phase = p.phase;
    total = p.total;
    completed = p.completed;
    if (p.phase === 'start' || p.phase === 'progress') {
      visible = true;
      cancelling = false;
      if (hideTimer) {
        clearTimeout(hideTimer);
        hideTimer = null;
      }
    } else {
      // done | cancelled | error: hide after a beat so the user sees the
      // final count. (A `done` with total 0 means nothing to do — see the
      // trigger guard in +layout; this strip simply never showed.)
      if (hideTimer) clearTimeout(hideTimer);
      hideTimer = setTimeout(() => {
        visible = false;
        phase = null;
        cancelling = false;
      }, 4000);
    }
  }

  onMount(async () => {
    unlisten = await listen<BackfillEvent>('nsc:backfill', (ev) => {
      handleEvent(ev.payload);
    });
    // Recover existing state on mount (navigated away and back / late mount).
    try {
      const status = await invoke<BackfillStatus>('nsc_backfill_status');
      if (status.running) {
        visible = true;
        phase = 'progress';
        total = status.total;
        completed = status.completed;
        cancelling = status.cancelling;
      }
    } catch (e) {
      console.error('[NscBackfill] initial status fetch failed:', e);
    }
  });

  onDestroy(() => {
    unlisten?.();
    if (hideTimer) clearTimeout(hideTimer);
  });
</script>

{#if visible}
  <div class="nsc-bf-strip" role="status" aria-live="polite">
    <span class="nsc-bf-label">
      {#if phase === 'done'}
        {$t('nscBackfill.done') || 'Summaries ready'}
      {:else if phase === 'cancelled'}
        {$t('nscBackfill.cancelled') || 'Summary build cancelled'}
      {:else if phase === 'error'}
        {$t('nscBackfill.error') || 'Summary build error'}
      {:else if cancelling}
        {$t('nscBackfill.cancelling') || 'Cancelling…'}
      {:else}
        {$t('nscBackfill.label') || 'Building note summaries…'}
      {/if}
    </span>
    {#if phase !== 'done' && phase !== 'cancelled' && phase !== 'error'}
      <span class="nsc-bf-counts">— {fmt(completed)} / {fmt(total)} ({pct}%)</span>
      <button
        class="nsc-bf-cancel"
        onclick={cancelBackfill}
        disabled={cancelling}
        title={$t('nscBackfill.cancelTitle') || 'Cancel summary build'}
      >
        {$t('nscBackfill.cancel') || 'Cancel'}
      </button>
    {:else if phase !== 'cancelled'}
      <span class="nsc-bf-counts">— {fmt(completed)} / {fmt(total)}</span>
    {/if}
  </div>
{/if}

<style>
  .nsc-bf-strip {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.75rem;
    color: var(--text-muted);
  }
  .nsc-bf-counts {
    font-variant-numeric: tabular-nums;
  }
  .nsc-bf-cancel {
    border: 1px solid var(--background-modifier-border, rgba(0,0,0,0.18));
    background: transparent;
    color: var(--text-muted);
    border-radius: 4px;
    padding: 1px 6px;
    font-size: 0.7rem;
    cursor: pointer;
    margin-inline-start: 4px;
  }
  .nsc-bf-cancel:hover { background: var(--background-modifier-hover); }
  .nsc-bf-cancel:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
