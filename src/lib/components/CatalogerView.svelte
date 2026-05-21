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

  MIG-039 note-picker: "Classify a note…" button opens an inline search
  popover so the user can classify any note from the full-page view (no
  open-note context is required). Uses the existing constellation_search
  IPC (lexical mode, limit 10) for the search, then dispatches the
  constellation:classify-and-show window event so the embedded
  SourceReviewPanel and the right-sidebar instance both update.
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
    /**
     * MIG-039: passed from +layout.svelte as `visible={showCataloger}`.
     * Forwarded to SourceReviewPanel so it reloads its queue whenever the
     * Cataloger is reopened (picks up notes classified via the right-sidebar
     * "Classify open note" button while the Cataloger was hidden).
     */
    visible?: boolean;
  }
  let { onNoteClick, onClose, visible = true }: Props = $props();

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

  // ── Summary backfill (MANUAL) ─────────────────────────────────────────────
  // "Build all summaries" pre-computes the summary for every note that lacks a
  // current one (progress shows in the status-bar strip). Manual only (Boss
  // decision 2026-05-21) — it is never started on boot, so it can't touch
  // startup time. Summaries also still fill lazily as cards scroll into view.
  type BackfillPhase = 'start' | 'progress' | 'done' | 'cancelled' | 'error';
  let backfillRunning = $state(false);
  let unlistenBackfill: UnlistenFn | null = null;

  async function startBackfill() {
    try {
      await invoke('nsc_backfill_start');
      backfillRunning = true;
    } catch (e) {
      // The idempotent guard errors if a run is already going — not fatal.
      console.error('[Cataloger] nsc_backfill_start failed:', e);
    }
  }

  // ── Note-picker ─────────────────────────────────────────────────────────
  // "Classify a note…" button: opens a compact inline search-and-pick
  // popover so the Cataloger can classify any note without needing an
  // open-note context.  Uses constellation_search (lexical, limit 10) to
  // find candidates; dispatches constellation:classify-and-show so both
  // SRP instances (this one + the right-sidebar) update their queues.

  type PickerResult = { name: string; path: string };

  let showPicker = $state(false);
  let pickerQuery = $state('');
  let pickerResults = $state<PickerResult[]>([]);
  let pickerLoading = $state(false);
  let pickerClassifying = $state(false);
  let pickerError = $state<string | null>(null);
  let pickerDebounceTimer: ReturnType<typeof setTimeout> | null = null;

  async function searchPickerNotes(query: string) {
    if (!query.trim()) { pickerResults = []; return; }
    pickerLoading = true;
    try {
      const results = await invoke<Array<{ name: string; path: string }>>('constellation_search', {
        request: { query, mode: 'lexical', limit: 10, include_snippet: false },
      });
      pickerResults = results.map(r => ({ name: r.name, path: r.path }));
    } catch (e) {
      console.error('[Cataloger picker] search failed:', e);
      pickerResults = [];
    } finally {
      pickerLoading = false;
    }
  }

  function onPickerInput(e: Event) {
    pickerQuery = (e.target as HTMLInputElement).value;
    if (pickerDebounceTimer) clearTimeout(pickerDebounceTimer);
    pickerDebounceTimer = setTimeout(() => {
      void searchPickerNotes(pickerQuery);
    }, 300);
  }

  async function classifyPickedNote(path: string) {
    if (pickerClassifying) return;
    pickerClassifying = true;
    pickerError = null;
    try {
      // Run the classifier.  We don't need the return value — the window
      // event below tells all SRP instances to fetch the fresh result.
      await invoke('classifier_suggest_for_note', { notePath: path });
      // Notify the embedded SRP + the right-sidebar SRP.
      window.dispatchEvent(
        new CustomEvent('constellation:classify-and-show', { detail: { notePath: path } }),
      );
      closePicker();
    } catch (e) {
      pickerError = String(e);
    } finally {
      pickerClassifying = false;
    }
  }

  function closePicker() {
    showPicker = false;
    pickerQuery = '';
    pickerResults = [];
    pickerError = null;
    if (pickerDebounceTimer) { clearTimeout(pickerDebounceTimer); pickerDebounceTimer = null; }
  }

  // Escape handling. +layout's global keydown handler is registered on
  // `document` in the CAPTURE phase and closes the full-page Cataloger on
  // Escape (`+layout.svelte` ~line 2963). Capture runs outermost→innermost, so
  // that handler fires BEFORE any bubble-phase handler here — a plain
  // stopPropagation in the picker can't preempt it (the Cataloger would close
  // instead of just the picker — Stage-3 #7 bug, 2026-05-20). We listen on
  // `window`, which is OUTSIDE `document` in the capture path, so this runs
  // first: while the picker is open we consume Escape and close only the
  // picker. When the picker is closed this is a no-op and Escape flows to the
  // global handler as before (closing the Cataloger).
  function onWindowKeydownCapture(e: KeyboardEvent) {
    if (e.key === 'Escape' && showPicker) {
      e.stopPropagation();
      closePicker();
    }
  }

  // ── Scan status ─────────────────────────────────────────────────────────

  onMount(async () => {
    // Window-capture Escape handler (see onWindowKeydownCapture). Capture phase
    // so it runs before +layout's document-capture handler.
    window.addEventListener('keydown', onWindowKeydownCapture, true);
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
    // Recover an in-progress backfill + track its lifecycle for the button.
    try {
      const bf = await invoke<{ running: boolean }>('nsc_backfill_status');
      backfillRunning = bf.running;
    } catch (e) {
      console.error('[Cataloger] nsc_backfill_status failed:', e);
    }
    unlistenBackfill = await listen<{ phase: BackfillPhase }>('nsc:backfill', (ev) => {
      const p = ev.payload.phase;
      backfillRunning = p === 'start' || p === 'progress';
    });
  });

  onDestroy(() => {
    unlisten?.();
    unlistenBackfill?.();
    window.removeEventListener('keydown', onWindowKeydownCapture, true);
    if (pickerDebounceTimer) clearTimeout(pickerDebounceTimer);
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
        <!-- Note-picker: search and classify any note from the full-page view -->
        <div class="cataloger-picker-wrapper">
          <button
            class="cataloger-pick-btn"
            onclick={() => { showPicker = !showPicker; if (!showPicker) closePicker(); }}
            title={$t('cataloger.classifyNote') || 'Classify a note…'}
          >
            {$t('cataloger.classifyNote') || 'Classify a note…'}
          </button>

          {#if showPicker}
            <div
              class="cataloger-picker"
              role="dialog"
              aria-label={$t('cataloger.classifyNote') || 'Classify a note'}
            >
              <input
                class="cataloger-picker-input"
                type="text"
                placeholder={$t('cataloger.searchNotes') || 'Search notes…'}
                value={pickerQuery}
                oninput={onPickerInput}
                autofocus
                aria-label={$t('cataloger.searchNotes') || 'Search notes'}
              />
              {#if pickerLoading}
                <div class="cataloger-picker-state">…</div>
              {:else if pickerResults.length > 0}
                <ul class="cataloger-picker-results" role="listbox">
                  {#each pickerResults as result}
                    <li role="option" aria-selected="false">
                      <button
                        class="cataloger-picker-result"
                        onclick={() => classifyPickedNote(result.path)}
                        disabled={pickerClassifying}
                        title={result.path}
                      >
                        {result.name}
                      </button>
                    </li>
                  {/each}
                </ul>
              {:else if pickerQuery.trim()}
                <div class="cataloger-picker-state">{$t('cataloger.noNotesFound') || 'No notes found'}</div>
              {/if}
              {#if pickerError}
                <div class="cataloger-picker-error">{pickerError}</div>
              {/if}
            </div>
          {/if}
        </div>

        <button
          class="cataloger-pick-btn"
          onclick={startBackfill}
          disabled={backfillRunning}
          title={$t('nscBackfill.buildNowTitle') || 'Pre-compute a summary for every note that lacks one (runs in the background; progress shows in the status bar)'}
        >
          {backfillRunning
            ? ($t('nscBackfill.label') || 'Building note summaries…')
            : ($t('nscBackfill.buildNow') || 'Build all summaries')}
        </button>

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
        {visible}
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

  /* ── Note-picker ─────────────────────────────────────────── */
  .cataloger-picker-wrapper {
    position: relative;
  }
  .cataloger-pick-btn {
    border: 1px solid var(--background-modifier-border, rgba(0,0,0,0.15));
    background: transparent;
    color: var(--text-normal, #222);
    border-radius: 6px;
    padding: 6px 12px;
    font-size: 0.82rem;
    font-weight: 500;
    cursor: pointer;
    white-space: nowrap;
  }
  .cataloger-pick-btn:hover {
    background: var(--background-modifier-hover, rgba(0,0,0,0.06));
  }
  /* Popover: floats below the button, RTL-aware via inset-inline-end. */
  .cataloger-picker {
    position: absolute;
    top: calc(100% + 6px);
    inset-inline-end: 0;
    width: 300px;
    background: var(--background-primary, #fff);
    border: 1px solid var(--background-modifier-border, rgba(0,0,0,0.15));
    border-radius: 8px;
    box-shadow: 0 6px 20px rgba(0,0,0,0.13);
    z-index: 200;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .cataloger-picker-input {
    width: 100%;
    padding: 6px 10px;
    border: 1px solid var(--background-modifier-border, rgba(0,0,0,0.15));
    border-radius: 5px;
    font-size: 0.85rem;
    background: var(--background-secondary, #f5f5f5);
    color: var(--text-normal, #222);
    outline: none;
    box-sizing: border-box;
  }
  .cataloger-picker-input:focus {
    border-color: var(--interactive-accent, #7c3aed);
  }
  .cataloger-picker-results {
    list-style: none;
    margin: 0;
    padding: 0;
    max-height: 220px;
    overflow-y: auto;
  }
  .cataloger-picker-result {
    display: block;
    width: 100%;
    text-align: start;
    padding: 6px 10px;
    border: none;
    background: transparent;
    color: var(--text-normal, #222);
    font-size: 0.83rem;
    border-radius: 4px;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .cataloger-picker-result:hover:not(:disabled) {
    background: var(--background-modifier-hover, rgba(0,0,0,0.06));
  }
  .cataloger-picker-result:disabled { opacity: 0.55; cursor: not-allowed; }
  .cataloger-picker-state {
    font-size: 0.8rem;
    color: var(--text-muted, #888);
    padding: 4px 10px;
    text-align: center;
  }
  .cataloger-picker-error {
    font-size: 0.78rem;
    color: var(--text-error, #c00);
    padding: 4px 10px;
  }

  /* ── Scan button ─────────────────────────────────────────── */
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
