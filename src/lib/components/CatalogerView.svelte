<!--
  MIG-039 — The Cataloger (CECE left-dock Core Plug-in).

  Full-page dock view that promotes CECE from a right-sidebar tab to a
  universe-wide home. Composes three existing pieces:
    - a "Scan library" control (classifier_scan_start — the manual,
      universe-wide scan; mirrors SettingsModal.startClassifierScan)
    - ClassifierScanProgressStrip (live progress while a scan runs)
    - SourceReviewPanel in library-level mode (no activeNotePath) — the
      suggestion queue the user approves/rejects.

  Accuracy (Concept Paper §5/§6): CECE ships as a 5-cataloger HEURISTIC
  ensemble (the local-LLM "Reasoning" cataloger is designed-but-not-wired)
  and scans are MANUAL-only. No copy here calls this "AI" or implies
  automatic background classification.
-->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { t, dir } from '$lib/i18n';
  import SourceReviewPanel from '$lib/components/SourceReviewPanel.svelte';
  import ClassifierScanProgressStrip from '$lib/components/ClassifierScanProgressStrip.svelte';

  interface Props {
    /** Open a note when one is clicked in the suggestion queue. */
    onNoteClick?: (path: string, name: string) => void;
    /** Close the full-page view (back to the editor). */
    onClose?: () => void;
  }
  let { onNoteClick, onClose }: Props = $props();

  type ScanPhase = 'start' | 'progress' | 'done' | 'cancelled' | 'error';
  let scanRunning = $state(false);
  let unlisten: UnlistenFn | null = null;

  async function startScan() {
    try {
      await invoke('classifier_scan_start');
      scanRunning = true;
    } catch (e) {
      console.error('[Cataloger] classifier_scan_start failed:', e);
    }
  }

  onMount(async () => {
    // Recover an in-progress scan started elsewhere (Settings, on-startup).
    try {
      const status = await invoke<{ running: boolean }>('classifier_scan_status');
      scanRunning = status.running;
    } catch (e) {
      console.error('[Cataloger] classifier_scan_status failed:', e);
    }
    // Keep the button's enabled state in sync with the live scan lifecycle.
    unlisten = await listen<{ phase: ScanPhase }>('classifier:scan', (ev) => {
      const p = ev.payload.phase;
      scanRunning = p === 'start' || p === 'progress';
    });
  });

  onDestroy(() => {
    unlisten?.();
  });
</script>

<div class="cataloger-view" dir={$dir}>
  <div class="cataloger-inner">
    <header class="cataloger-header">
      <div class="cataloger-heading">
        <h1 class="cataloger-title">{$t('cataloger.title') || 'The Cataloger'}</h1>
        <p class="cataloger-tagline">
          {$t('cataloger.tagline') || 'Classify each note by its kind of knowledge and its source.'}
        </p>
      </div>
      <div class="cataloger-actions">
        <button class="cataloger-scan-btn" onclick={startScan} disabled={scanRunning}>
          {scanRunning
            ? ($t('settings.classifier.scanRunning') || 'Running…')
            : ($t('settings.classifier.scanStart') || 'Start scan')}
        </button>
        {#if onClose}
          <button
            class="cataloger-close"
            onclick={() => onClose?.()}
            title={$t('common.close') || 'Close'}
            aria-label={$t('common.close') || 'Close'}
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                 stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        {/if}
      </div>
    </header>

    <div class="cataloger-progress">
      <ClassifierScanProgressStrip />
    </div>

    <div class="cataloger-queue">
      <SourceReviewPanel
        onNoteClick={(path, name) => onNoteClick?.(path, name)}
      />
    </div>
  </div>
</div>

<style>
  .cataloger-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    overflow: hidden;
    background: var(--background-primary, #fff);
    color: var(--text-normal, #222);
  }
  /* Full-width — the dock view uses the whole content area (not a narrow
     centered column, which read as a "window within a window"). */
  .cataloger-inner {
    width: 100%;
    padding: 20px 28px 12px;
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }
  .cataloger-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    flex-shrink: 0;
    border-bottom: 1px solid var(--background-modifier-border, rgba(0,0,0,0.1));
    padding-bottom: 16px;
    margin-bottom: 12px;
  }
  .cataloger-heading { min-width: 0; }
  .cataloger-title {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
    letter-spacing: -0.01em;
  }
  .cataloger-tagline {
    margin: 4px 0 0;
    font-size: 0.85rem;
    color: var(--text-muted, #888);
    line-height: 1.4;
  }
  .cataloger-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }
  .cataloger-scan-btn {
    border: 1px solid var(--interactive-accent, #7c3aed);
    background: var(--interactive-accent, #7c3aed);
    color: var(--text-on-accent, #fff);
    border-radius: 6px;
    padding: 6px 14px;
    font-size: 0.82rem;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
  }
  .cataloger-scan-btn:hover:not(:disabled) { filter: brightness(1.08); }
  .cataloger-scan-btn:disabled { opacity: 0.6; cursor: not-allowed; }
  .cataloger-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    border: 1px solid var(--background-modifier-border, rgba(0,0,0,0.15));
    background: transparent;
    color: var(--text-muted, #888);
    border-radius: 6px;
    cursor: pointer;
  }
  .cataloger-close:hover { background: var(--background-modifier-hover, rgba(0,0,0,0.06)); color: var(--text-normal, #222); }
  .cataloger-progress { flex-shrink: 0; }
  .cataloger-progress:empty { display: none; }
  .cataloger-queue {
    flex: 1;
    min-height: 0;
    margin-top: 4px;
  }
</style>
