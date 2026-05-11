<!--
  MIG-021v3 V3-§10.A — Per-Library calibration view.

  Read-only table that surfaces the V3-§9.C.2 reliability data (each
  cataloger's per-axis accuracy on the active Library) inside the CECE
  Settings section.

  Empty state when no corrections have been logged yet on this Library.
  "(uniform)" label for catalogers below the 20-correction threshold —
  matches `MIN_SAMPLES_FOR_WEIGHTING` in `reliability.rs`.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { t } from '$lib/i18n';

  type AccuracyHistogram = { correct: number; wrong: number };
  type ReliabilityProfile = {
    stats: Record<string, Record<string, AccuracyHistogram>>;
  };

  // The active note path lets the IPC resolve the right Library when
  // multiple are registered. Optional — falls back to first Library.
  let { activeNotePath = null }: { activeNotePath?: string | null } = $props();

  let loading = $state(true);
  let error = $state<string | null>(null);
  let profile = $state<ReliabilityProfile>({ stats: {} });
  let libraryRoot = $state<string | null>(null);

  const MIN_SAMPLES = 20;

  // Cataloger display names — pulled from the existing cece.cataloger.*
  // i18n keys so the calibration view matches the Source Review panel's
  // labels exactly (UA = "Your frontmatter", etc.).
  function catalogerLabel(name: string): string {
    const k = `cece.cataloger.${name}`;
    const translated = $t(k);
    if (translated && translated !== k) return translated;
    // Fall-through to humanized snake_case.
    return name.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
  }

  function formatCell(h: AccuracyHistogram | undefined): string {
    if (!h) return '—';
    const total = h.correct + h.wrong;
    if (total === 0) return '—';
    if (total < MIN_SAMPLES) {
      return `${h.correct}/${total} (${$t('cece.settings.calibrationUniform') || 'uniform'})`;
    }
    const pct = Math.round((h.correct / total) * 100);
    return `${h.correct}/${total} (${pct}%)`;
  }

  // Cataloger render order — matches the Source Review panel's dot
  // cluster order (cost-ordered, registered in orchestrator).
  const CATALOGER_ORDER = [
    'user_authority',
    'structural',
    'linguistic',
    'graph',
    'semantic',
    'reasoning',
  ];

  async function load() {
    loading = true;
    error = null;
    try {
      [profile, libraryRoot] = await Promise.all([
        invoke<ReliabilityProfile>('cece_get_reliability_for_active_library', { notePath: activeNotePath }),
        invoke<string | null>('cece_get_active_library_root', { notePath: activeNotePath }),
      ]);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  onMount(load);

  // Derived: short Library name from the path (last segment).
  let libraryShortName = $derived.by(() => {
    if (!libraryRoot) return null;
    const parts = libraryRoot.split(/[\\/]/).filter(Boolean);
    return parts[parts.length - 1] ?? libraryRoot;
  });

  // Derived: total correction count to decide between empty-state and
  // the table.
  let totalCorrections = $derived.by(() => {
    let n = 0;
    for (const cat of Object.values(profile.stats || {})) {
      for (const ax of Object.values(cat)) {
        n += ax.correct + ax.wrong;
      }
    }
    return n;
  });
</script>

<div class="cv-root" dir="auto">
  {#if loading}
    <div class="cv-loading">{$t('cece.settings.calibrationLoading') || 'Loading calibration data…'}</div>
  {:else if error}
    <div class="cv-error">
      <strong>{$t('cece.settings.calibrationError') || 'Could not load calibration data'}</strong>
      <div class="cv-error-detail">{error}</div>
    </div>
  {:else if totalCorrections === 0}
    <div class="cv-empty">
      <div class="cv-empty-text">
        {$t('cece.settings.calibrationEmpty') || 'No corrections logged yet on this Library. Calibration data will appear here after you Accept or pick a Sibling Disambiguation chip on at least one classified card.'}
      </div>
    </div>
  {:else}
    {#if libraryShortName}
      <div class="cv-library-line">
        <span class="cv-library-label">{$t('cece.settings.calibrationActive') || 'Active Library:'}</span>
        <span class="cv-library-name">{libraryShortName}</span>
      </div>
    {/if}
    <div class="cv-min-samples-note">
      {($t('cece.settings.calibrationMinSamples') || 'Catalogers need {N} corrections before stable accuracy data; below that they vote with uniform weight.').replace('{N}', String(MIN_SAMPLES))}
    </div>
    <table class="cv-table">
      <thead>
        <tr>
          <th>{$t('cece.settings.calibrationColCataloger') || 'Cataloger'}</th>
          <th>{$t('cece.settings.calibrationColHorizontal') || 'Horizontal (Source)'}</th>
          <th>{$t('cece.settings.calibrationColVertical') || 'Vertical (Content type)'}</th>
        </tr>
      </thead>
      <tbody>
        {#each CATALOGER_ORDER as cat}
          {@const stats = profile.stats?.[cat]}
          <tr>
            <td>{catalogerLabel(cat)}</td>
            <td>{formatCell(stats?.horizontal)}</td>
            <td>{formatCell(stats?.vertical)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .cv-root {
    font-size: 12px;
    color: var(--text-normal, #1a1a1a);
    padding: 4px 0;
  }
  .cv-loading,
  .cv-empty,
  .cv-error {
    padding: 12px;
    text-align: center;
    color: var(--text-muted, #6b6a64);
  }
  .cv-error {
    color: var(--text-error, #a83232);
  }
  .cv-error-detail {
    margin-top: 4px;
    font-size: 11px;
    word-break: break-word;
  }
  .cv-empty-text {
    line-height: 1.5;
    max-width: 50ch;
    margin: 0 auto;
  }
  .cv-library-line {
    margin-bottom: 4px;
    font-size: 12px;
  }
  .cv-library-label {
    color: var(--text-muted, #6b6a64);
    margin-inline-end: 6px;
  }
  .cv-library-name {
    font-weight: 600;
    color: var(--text-normal, #1a1a1a);
  }
  .cv-min-samples-note {
    margin-bottom: 8px;
    font-size: 11px;
    color: var(--text-muted, #6b6a64);
    line-height: 1.5;
  }
  .cv-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 11px;
  }
  .cv-table th,
  .cv-table td {
    padding: 4px 8px;
    text-align: start;
    border-bottom: 1px solid var(--background-modifier-border, rgba(0,0,0,0.08));
  }
  .cv-table th {
    font-weight: 600;
    color: var(--text-muted, #6b6a64);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .cv-table td {
    font-variant-numeric: tabular-nums;
  }
</style>
