<!--
  MIG-021v2 §1C' — Source Review sidebar panel (dual-axis).

  Shows the classifier's combined queue of suggestions across both axes
  (horizontal sources + vertical content_type). Per record:
    - Two sublists, one per axis, with tier badges on horizontal entries
    - Accept commits BOTH axes (sources_set_manual + content_type_set_manual)
    - Edit mode opens TWO TaxonomyTreePicker instances (one per axis)
    - Reject clears the suggestion without writing either field
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { t } from '$lib/i18n';
  import TaxonomyTreePicker from '$lib/sources/TaxonomyTreePicker.svelte';
  import {
    getHorizontalTaxonomy,
    tierColor,
    type HorizontalNode,
  } from '$lib/sources/horizontalTaxonomy';
  import {
    getVerticalTaxonomy,
    type VerticalNode,
  } from '$lib/sources/verticalTaxonomy';

  type Suggestion = {
    source: string;
    confidence: number;
    evidence: string;
    axis: string;
  };

  type SuggestionRecord = {
    note_path: string;
    suggestions: Suggestion[];
    classifier_tier: number;
    created_at: number;
  };

  let {
    onNoteClick = (_p: string, _n: string) => {},
    activeNotePath = null,
  }: {
    onNoteClick?: (path: string, name: string) => void;
    activeNotePath?: string | null;
  } = $props();

  let queue = $state<SuggestionRecord[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let classifying = $state(false);

  // Edit-mode state per record path
  let editingPath = $state<string | null>(null);
  let editedHorizontal = $state(new Set<string>());
  let editedVertical = $state(new Set<string>());

  // Cached taxonomies (fetched once on mount)
  let horizontalTaxonomy = $state<HorizontalNode[]>([]);
  let verticalTaxonomy = $state<VerticalNode[]>([]);

  async function loadQueue() {
    loading = true;
    error = null;
    try {
      queue = await invoke<SuggestionRecord[]>('sources_list_pending_suggestions');
    } catch (e) {
      error = String(e);
      queue = [];
    } finally {
      loading = false;
    }
  }

  async function classifyActiveNote() {
    if (!activeNotePath || classifying) return;
    classifying = true;
    error = null;
    try {
      const record = await invoke<SuggestionRecord>('classifier_suggest_for_note', {
        notePath: activeNotePath,
      });
      queue = [record, ...queue.filter(r => r.note_path !== record.note_path)];
    } catch (e) {
      error = String(e);
    } finally {
      classifying = false;
    }
  }

  async function acceptSuggestion(
    record: SuggestionRecord,
    horizontalOverride?: string[],
    verticalOverride?: string[],
  ) {
    const horizontalIds =
      horizontalOverride ?? record.suggestions.filter(s => s.axis === 'horizontal').map(s => s.source);
    const verticalIds =
      verticalOverride ?? record.suggestions.filter(s => s.axis === 'vertical').map(s => s.source);
    try {
      await invoke('sources_set_manual', {
        notePath: record.note_path,
        sources: horizontalIds,
      });
      await invoke('content_type_set_manual', {
        notePath: record.note_path,
        contentType: verticalIds,
      });
      queue = queue.filter(r => r.note_path !== record.note_path);
      cancelEdit();
    } catch (e) {
      error = String(e);
    }
  }

  async function rejectSuggestion(record: SuggestionRecord) {
    try {
      await invoke('sources_reject_suggestion', { notePath: record.note_path });
      queue = queue.filter(r => r.note_path !== record.note_path);
    } catch (e) {
      error = String(e);
    }
  }

  function startEdit(record: SuggestionRecord) {
    editingPath = record.note_path;
    editedHorizontal = new Set(
      record.suggestions.filter(s => s.axis === 'horizontal').map(s => s.source),
    );
    editedVertical = new Set(
      record.suggestions.filter(s => s.axis === 'vertical').map(s => s.source),
    );
  }

  function cancelEdit() {
    editingPath = null;
    editedHorizontal = new Set();
    editedVertical = new Set();
  }

  function commitEdit(record: SuggestionRecord) {
    acceptSuggestion(record, [...editedHorizontal], [...editedVertical]);
  }

  function noteName(path: string): string {
    const seg = path.split(/[\\/]/).pop() ?? path;
    return seg.replace(/\.md$/, '');
  }

  function formatConfidence(c: number): string {
    return `${Math.round(c * 100)}%`;
  }

  function effectiveTierForId(id: string): number {
    const node = horizontalTaxonomy.find(n => n.id === id);
    if (!node) return 0;
    if (node.tier > 0) return node.tier;
    if (node.parent_id) {
      const parent = horizontalTaxonomy.find(n => n.id === node.parent_id);
      if (parent) return parent.tier;
    }
    return 0;
  }

  function labelForId(id: string, axis: string): string {
    if (axis === 'horizontal') {
      return horizontalTaxonomy.find(n => n.id === id)?.en ?? id;
    }
    return verticalTaxonomy.find(n => n.id === id)?.en ?? id;
  }

  onMount(async () => {
    try {
      [horizontalTaxonomy, verticalTaxonomy] = await Promise.all([
        getHorizontalTaxonomy(),
        getVerticalTaxonomy(),
      ]);
    } catch (e) {
      error = `Failed to load taxonomies: ${e}`;
    }
    loadQueue();
  });
</script>

<div class="srp-root" dir="auto">
  <div class="srp-header">
    <div class="srp-title">{$t('sources.review.title') || 'Source Review'}</div>
    <div class="srp-header-actions">
      {#if activeNotePath}
        <button
          class="srp-classify"
          onclick={() => classifyActiveNote()}
          disabled={classifying}
          title={$t('sources.review.classifyActive') || 'Classify the open note'}
        >
          {classifying
            ? ($t('sources.review.classifying') || 'Classifying…')
            : ($t('sources.review.classifyActive') || 'Classify open note')}
        </button>
      {/if}
      <button
        class="srp-refresh"
        onclick={() => loadQueue()}
        title={$t('sources.review.refresh') || 'Refresh queue'}
        aria-label={$t('sources.review.refresh') || 'Refresh queue'}
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
             stroke-linecap="round" stroke-linejoin="round">
          <polyline points="23 4 23 10 17 10"/>
          <polyline points="1 20 1 14 7 14"/>
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
        </svg>
      </button>
    </div>
  </div>

  {#if loading}
    <div class="srp-empty">{$t('sources.review.loading') || 'Loading suggestions…'}</div>
  {:else if error}
    <div class="srp-error">
      <strong>{$t('sources.review.errorTitle') || 'Could not load'}</strong>
      <div class="srp-error-detail">{error}</div>
    </div>
  {:else if queue.length === 0}
    <div class="srp-empty">
      <div class="srp-empty-title">{$t('sources.review.emptyTitle') || 'Nothing to review'}</div>
      <div class="srp-empty-sub">
        {$t('sources.review.emptySub') ||
          'Classify a note (right-click → Suggest, or use the Classify open note button) to populate this queue.'}
      </div>
    </div>
  {:else}
    <div class="srp-count">
      {queue.length}
      {queue.length === 1
        ? ($t('sources.review.pending') || 'pending')
        : ($t('sources.review.pendingPlural') || 'pending')}
    </div>

    <ul class="srp-list">
      {#each queue as record (record.note_path)}
        {@const horizontalSuggestions = record.suggestions.filter(s => s.axis === 'horizontal')}
        {@const verticalSuggestions = record.suggestions.filter(s => s.axis === 'vertical')}
        <li class="srp-card">
          <div class="srp-card-header">
            <button
              class="srp-card-title"
              onclick={() => onNoteClick(record.note_path, noteName(record.note_path))}
              title={record.note_path}
            >
              {noteName(record.note_path)}
            </button>
            <span class="srp-tier" title={$t('sources.review.tierLabel') || 'Classifier tier'}>
              {record.classifier_tier === 2 ? 'T2' : 'T1'}
            </span>
          </div>

          {#if editingPath === record.note_path}
            <!-- ─── Edit mode: two tree pickers ─── -->
            <div class="srp-edit-axes">
              <div class="srp-axis-block">
                <div class="srp-axis-label">
                  {$t('sources.review.axis.horizontal') || 'Sources'}
                </div>
                <div class="srp-tree-wrap">
                  <TaxonomyTreePicker
                    taxonomy={horizontalTaxonomy}
                    axis="horizontal"
                    selected={editedHorizontal}
                    onChange={(s) => (editedHorizontal = s)}
                    tierColors={true}
                  />
                </div>
              </div>
              <div class="srp-axis-block">
                <div class="srp-axis-label">
                  {$t('sources.review.axis.vertical') || 'Content type'}
                </div>
                <div class="srp-tree-wrap">
                  <TaxonomyTreePicker
                    taxonomy={verticalTaxonomy}
                    axis="vertical"
                    selected={editedVertical}
                    onChange={(s) => (editedVertical = s)}
                    tierColors={false}
                  />
                </div>
              </div>
            </div>
            <div class="srp-actions">
              <button class="srp-btn srp-btn-primary" onclick={() => commitEdit(record)}>
                {$t('sources.review.save') || 'Save'}
              </button>
              <button class="srp-btn" onclick={() => cancelEdit()}>
                {$t('sources.review.cancel') || 'Cancel'}
              </button>
            </div>
          {:else}
            <!-- ─── Default: dual-axis suggestion display ─── -->
            {#if horizontalSuggestions.length > 0}
              <div class="srp-axis-section">
                <div class="srp-axis-label">
                  {$t('sources.review.axis.horizontal') || 'Sources'}
                </div>
                <ul class="srp-suggestions">
                  {#each horizontalSuggestions as s, i}
                    {@const tier = effectiveTierForId(s.source)}
                    {@const tcolor = tierColor(tier)}
                    <li
                      class="srp-suggestion"
                      class:primary={i === 0}
                      style:--tier-color={tcolor ?? 'transparent'}
                    >
                      <div class="srp-source-row">
                        <span
                          class="srp-source-name"
                          title={$t('sources.description.' + s.source) || s.source}
                        >
                          {labelForId(s.source, 'horizontal')}
                        </span>
                        {#if tier > 0}
                          <span class="srp-tier-badge" style:background-color={tcolor ?? ''}
                                title={`Tier ${tier}`}>T{tier}</span>
                        {/if}
                        <span class="srp-confidence">{formatConfidence(s.confidence)}</span>
                      </div>
                      {#if s.evidence}
                        <div class="srp-evidence">
                          {$t('sources.evidence.' + s.source) || s.evidence}
                        </div>
                      {/if}
                    </li>
                  {/each}
                </ul>
              </div>
            {/if}

            {#if verticalSuggestions.length > 0}
              <div class="srp-axis-section">
                <div class="srp-axis-label">
                  {$t('sources.review.axis.vertical') || 'Content type'}
                </div>
                <ul class="srp-suggestions">
                  {#each verticalSuggestions as s, i}
                    <li class="srp-suggestion vertical" class:primary={i === 0}>
                      <div class="srp-source-row">
                        <span class="srp-source-name">
                          {labelForId(s.source, 'vertical')}
                        </span>
                        <span class="srp-confidence">{formatConfidence(s.confidence)}</span>
                      </div>
                    </li>
                  {/each}
                </ul>
              </div>
            {/if}

            <div class="srp-actions">
              <button class="srp-btn srp-btn-primary" onclick={() => acceptSuggestion(record)}>
                {$t('sources.review.accept') || 'Accept'}
              </button>
              <button class="srp-btn" onclick={() => startEdit(record)}>
                {$t('sources.review.edit') || 'Edit'}
              </button>
              <button class="srp-btn srp-btn-danger" onclick={() => rejectSuggestion(record)}>
                {$t('sources.review.reject') || 'Reject'}
              </button>
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .srp-root {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    font-size: 13px;
    color: var(--text-normal, #1a1a1a);
  }
  .srp-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-bottom: 1px solid var(--background-modifier-border, rgba(0,0,0,0.08));
    flex-shrink: 0;
  }
  .srp-title {
    font-weight: 600;
    letter-spacing: 1px;
    font-size: 11px;
    text-transform: uppercase;
    color: var(--text-muted, #6b6a64);
  }
  .srp-header-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .srp-classify {
    background: transparent;
    border: 1px solid var(--background-modifier-border, rgba(0,0,0,0.18));
    color: var(--text-normal, #1a1a1a);
    cursor: pointer;
    padding: 3px 8px;
    border-radius: 4px;
    font-size: 11px;
  }
  .srp-classify:hover:not(:disabled) {
    background: var(--background-modifier-hover, rgba(0,0,0,0.05));
  }
  .srp-classify:disabled {
    opacity: 0.55;
    cursor: wait;
  }
  .srp-refresh {
    background: transparent;
    border: none;
    color: var(--text-muted, #6b6a64);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: flex;
    align-items: center;
  }
  .srp-refresh:hover {
    background: var(--background-modifier-hover, rgba(0,0,0,0.05));
    color: var(--text-normal, #1a1a1a);
  }
  .srp-count {
    padding: 6px 12px;
    font-size: 11px;
    color: var(--text-muted, #6b6a64);
  }
  .srp-empty,
  .srp-error {
    padding: 24px 16px;
    text-align: center;
    color: var(--text-muted, #6b6a64);
  }
  .srp-empty-title {
    font-weight: 600;
    margin-bottom: 6px;
    color: var(--text-normal, #1a1a1a);
  }
  .srp-empty-sub {
    font-size: 12px;
    line-height: 1.5;
  }
  .srp-error {
    color: var(--text-error, #a83232);
  }
  .srp-error-detail {
    font-size: 11px;
    margin-top: 6px;
    word-break: break-word;
  }
  .srp-list {
    list-style: none;
    margin: 0;
    padding: 0 8px 8px;
    overflow-y: auto;
    flex: 1;
  }
  .srp-card {
    background: var(--background-secondary, #fbf8ec);
    border: 1px solid var(--background-modifier-border, rgba(0,0,0,0.08));
    border-radius: 6px;
    padding: 10px;
    margin-bottom: 8px;
  }
  .srp-card-header {
    display: flex;
    align-items: baseline;
    gap: 8px;
    margin-bottom: 8px;
  }
  .srp-card-title {
    background: transparent;
    border: none;
    padding: 0;
    color: var(--text-accent, #2a4a8c);
    font-weight: 600;
    font-size: 13px;
    cursor: pointer;
    text-align: start;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .srp-card-title:hover {
    text-decoration: underline;
  }
  .srp-tier {
    font-size: 10px;
    color: var(--text-muted, #6b6a64);
    background: var(--background-modifier-hover, rgba(0,0,0,0.05));
    padding: 1px 6px;
    border-radius: 8px;
    flex-shrink: 0;
  }
  .srp-axis-section {
    margin-bottom: 8px;
  }
  .srp-axis-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--text-muted, #6b6a64);
    margin-bottom: 4px;
    font-weight: 600;
  }
  .srp-suggestions {
    list-style: none;
    margin: 0 0 4px;
    padding: 0;
  }
  .srp-suggestion {
    padding: 6px 8px;
    border-inline-start: 2px solid transparent;
    margin-bottom: 4px;
    font-size: 12px;
  }
  .srp-suggestion.primary {
    border-inline-start-color: #c9a227;
    background: rgba(201, 162, 39, 0.06);
  }
  .srp-source-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .srp-source-name {
    font-weight: 500;
    flex: 1;
    min-width: 0;
  }
  .srp-tier-badge {
    color: #faf6e8;
    font-size: 9px;
    font-weight: 600;
    padding: 1px 5px;
    border-radius: 3px;
    flex-shrink: 0;
  }
  .srp-confidence {
    font-size: 11px;
    color: var(--text-muted, #6b6a64);
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }
  .srp-evidence {
    margin-top: 3px;
    font-size: 11px;
    color: var(--text-muted, #6b6a64);
    font-style: italic;
  }
  .srp-edit-axes {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 8px;
  }
  @media (min-width: 1200px) {
    .srp-edit-axes {
      flex-direction: row;
    }
  }
  .srp-axis-block {
    flex: 1;
    min-width: 0;
    background: var(--background-primary, #fff);
    border: 1px solid var(--background-modifier-border, rgba(0,0,0,0.08));
    border-radius: 4px;
    overflow: hidden;
  }
  .srp-axis-block .srp-axis-label {
    padding: 6px 8px;
    background: var(--background-modifier-hover, rgba(0,0,0,0.04));
    border-bottom: 1px solid var(--background-modifier-border, rgba(0,0,0,0.08));
    margin-bottom: 0;
  }
  .srp-tree-wrap {
    height: 280px;
  }
  .srp-actions {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .srp-btn {
    background: transparent;
    border: 1px solid var(--background-modifier-border, rgba(0,0,0,0.18));
    color: var(--text-normal, #1a1a1a);
    padding: 4px 12px;
    font-size: 12px;
    border-radius: 4px;
    cursor: pointer;
  }
  .srp-btn:hover {
    background: var(--background-modifier-hover, rgba(0,0,0,0.05));
  }
  .srp-btn-primary {
    background: #c9a227;
    border-color: #c9a227;
    color: #faf6e8;
  }
  .srp-btn-primary:hover {
    filter: brightness(1.05);
  }
  .srp-btn-danger {
    color: var(--text-error, #a83232);
  }
</style>
