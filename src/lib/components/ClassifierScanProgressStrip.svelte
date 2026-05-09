<!--
  MIG-021v2 §1F' — Status-bar strip for the background classifier scan.

  Shows running counts via `classifier:scan` events emitted by the Rust
  scan_job. Hidden until a `start` event arrives (or until mount finds an
  in-progress scan via classifier_scan_status). Hides 4 seconds after
  `done` / `cancelled` / `error`.

  Mirrors MigrationProgressStrip's pattern intentionally — Eisa is used
  to that visual language; consistency wins.
-->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { t } from '$lib/i18n';

  type Phase = 'start' | 'progress' | 'done' | 'cancelled' | 'error';

  interface ScanProgressEvent {
    phase: Phase;
    total: number;
    completed: number;
    error: string | null;
  }

  interface ScanStatus {
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

  async function cancelScan() {
    cancelling = true;
    try {
      await invoke('classifier_scan_cancel');
    } catch (e) {
      console.error('[ClassifierScan] cancel failed:', e);
    }
  }

  function handleEvent(p: ScanProgressEvent) {
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
      // done | cancelled | error: hide after a beat so the user sees
      // the final count.
      if (hideTimer) clearTimeout(hideTimer);
      hideTimer = setTimeout(() => {
        visible = false;
        phase = null;
        cancelling = false;
      }, 4000);
    }
  }

  onMount(async () => {
    unlisten = await listen<ScanProgressEvent>('classifier:scan', (ev) => {
      handleEvent(ev.payload);
    });
    // Recover existing state on mount — covers the case where the user
    // navigated away and back, or the strip mounted late.
    try {
      const status = await invoke<ScanStatus>('classifier_scan_status');
      if (status.running) {
        visible = true;
        phase = 'progress';
        total = status.total;
        completed = status.completed;
        cancelling = status.cancelling;
      }
    } catch (e) {
      console.error('[ClassifierScan] initial status fetch failed:', e);
    }
  });

  onDestroy(() => {
    unlisten?.();
    if (hideTimer) clearTimeout(hideTimer);
  });
</script>

{#if visible}
  <div class="cls-scan-strip" role="status" aria-live="polite">
    <span class="cls-scan-label">
      {#if phase === 'done'}
        {$t('classifierScan.done') || 'Classification complete'}
      {:else if phase === 'cancelled'}
        {$t('classifierScan.cancelled') || 'Classification cancelled'}
      {:else if phase === 'error'}
        {$t('classifierScan.error') || 'Classification error'}
      {:else if cancelling}
        {$t('classifierScan.cancelling') || 'Cancelling…'}
      {:else}
        {$t('classifierScan.label') || 'Classifying notes…'}
      {/if}
    </span>
    {#if phase !== 'done' && phase !== 'cancelled' && phase !== 'error'}
      <span class="cls-scan-counts">— {fmt(completed)} / {fmt(total)} ({pct}%)</span>
      <button
        class="cls-scan-cancel"
        onclick={cancelScan}
        disabled={cancelling}
        title={$t('classifierScan.cancelTitle') || 'Cancel scan'}
      >
        {$t('classifierScan.cancel') || 'Cancel'}
      </button>
    {:else if phase !== 'cancelled'}
      <span class="cls-scan-counts">— {fmt(completed)} / {fmt(total)}</span>
    {/if}
  </div>
{/if}

<style>
  .cls-scan-strip {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.75rem;
    color: var(--text-muted);
  }
  .cls-scan-counts {
    font-variant-numeric: tabular-nums;
  }
  .cls-scan-cancel {
    border: 1px solid var(--background-modifier-border, rgba(0,0,0,0.18));
    background: transparent;
    color: var(--text-muted);
    border-radius: 4px;
    padding: 1px 6px;
    font-size: 0.7rem;
    cursor: pointer;
    margin-inline-start: 4px;
  }
  .cls-scan-cancel:hover { background: var(--background-modifier-hover); }
  .cls-scan-cancel:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
