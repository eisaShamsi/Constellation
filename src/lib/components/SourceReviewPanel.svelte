<!--
  MIG-021 §1C — Source Review sidebar panel.

  Surfaces the classifier's `sources_suggested:` queue for user approval.
  Per Plan: Accept (writes `sources:` to frontmatter + clears suggestion),
  Edit (modify before accepting), Reject (clear without writing).

  Anchored against:
    docs/Constellation-Sight-Concept-Paper-v2.0.md §7.3 + §8.2
    lab/reports/MIG-021-EPISTEMIC-CLASSIFIER-PLAN.md §1C
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { t } from '$lib/i18n';

  type Suggestion = {
    source: string;
    confidence: number;
    evidence: string;
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

  let classifying = $state(false);

  let queue = $state<SuggestionRecord[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let editingPath = $state<string | null>(null);
  let editedSources = $state<Set<string>>(new Set());

  // Canonical 11 sources + the 12th `unclassifiable` opt-out token.
  // Order matches SOURCE_IDS in src-tauri/src/sources.rs.
  const ALL_SOURCES = [
    'perception',
    'inference',
    'testimony',
    'mass-transmission',
    'comparison',
    'postulation',
    'non-apprehension',
    'memory',
    'innate-disposition',
    'inspiration',
    'revelation',
    'unclassifiable',
  ] as const;

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
      // Insert/replace record in queue (front of list — newest visible first).
      queue = [record, ...queue.filter(r => r.note_path !== record.note_path)];
    } catch (e) {
      error = String(e);
    } finally {
      classifying = false;
    }
  }

  async function acceptSuggestion(record: SuggestionRecord, editedList?: string[]) {
    const sources = editedList ?? record.suggestions.map(s => s.source);
    try {
      await invoke('sources_set_manual', { notePath: record.note_path, sources });
      // sources_set_manual clears the suggestion server-side; reload locally.
      queue = queue.filter(r => r.note_path !== record.note_path);
      editingPath = null;
      editedSources = new Set();
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
    editedSources = new Set(record.suggestions.map(s => s.source));
  }

  function cancelEdit() {
    editingPath = null;
    editedSources = new Set();
  }

  function toggleSource(source: string) {
    const next = new Set(editedSources);
    if (next.has(source)) next.delete(source);
    else next.add(source);
    editedSources = next;
  }

  function commitEdit(record: SuggestionRecord) {
    acceptSuggestion(record, Array.from(editedSources));
  }

  function noteName(path: string): string {
    // Last path segment without .md extension.
    const seg = path.split(/[\\/]/).pop() ?? path;
    return seg.replace(/\.md$/, '');
  }

  function formatConfidence(c: number): string {
    return `${Math.round(c * 100)}%`;
  }

  onMount(() => {
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
          'Run the classifier from a note’s right-click menu to populate this queue.'}
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
            <!-- Edit mode: multi-select all 11 sources + 'unclassifiable' opt-out -->
            <div class="srp-edit-grid">
              {#each ALL_SOURCES as src}
                <label
                  class="srp-edit-pill"
                  class:active={editedSources.has(src)}
                  title={$t('sources.description.' + src) || src}
                >
                  <input
                    type="checkbox"
                    checked={editedSources.has(src)}
                    onchange={() => toggleSource(src)}
                  />
                  <span>{$t('sources.label.' + src) || src}</span>
                </label>
              {/each}
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
            <!-- Default: show suggestions with confidence + evidence -->
            <ul class="srp-suggestions">
              {#each record.suggestions as s, i}
                <li class="srp-suggestion" class:primary={i === 0}>
                  <div class="srp-source-row">
                    <span
                      class="srp-source-name"
                      title={$t('sources.description.' + s.source) || s.source}
                    >
                      {$t('sources.label.' + s.source) || s.source}
                    </span>
                    <span class="srp-confidence">{formatConfidence(s.confidence)}</span>
                  </div>
                  <!--
                    Evidence string: prefer the locale-aware sources.evidence.{source}
                    lookup over the stored evidence (which is hardcoded English in
                    Tier-1 from brief_signature_for in tier1_embedding.rs).
                    When Tier-2 (Qwen3-1.7B) ships in §1H it will produce dynamic
                    quote-from-note evidence; that takes precedence over the locale
                    lookup because per-classification quotes are richer than the
                    generic per-source signature. For now Tier-1 always returns the
                    generic signature, so the locale lookup wins.
                  -->
                  {#if $t('sources.evidence.' + s.source) || s.evidence}
                    <div class="srp-evidence">
                      {$t('sources.evidence.' + s.source) || s.evidence}
                    </div>
                  {/if}
                </li>
              {/each}
            </ul>

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
  .srp-suggestions {
    list-style: none;
    margin: 0 0 8px;
    padding: 0;
  }
  .srp-suggestion {
    padding: 6px 8px;
    /* Logical property so the border auto-flips for RTL — appears on
       the LEADING edge in any locale (left in LTR, right in RTL). */
    border-inline-start: 2px solid transparent;
    margin-bottom: 4px;
    font-size: 12px;
  }
  .srp-suggestion.primary {
    /* Suwaidi gold — hard-coded to be theme-independent (don't pick up
       the user's --text-accent which is often purple/blue per theme). */
    border-inline-start-color: #c9a227;
    background: rgba(201, 162, 39, 0.06);
  }
  .srp-source-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .srp-source-name {
    font-weight: 500;
  }
  .srp-confidence {
    font-size: 11px;
    color: var(--text-muted, #6b6a64);
    font-variant-numeric: tabular-nums;
  }
  .srp-evidence {
    margin-top: 3px;
    font-size: 11px;
    color: var(--text-muted, #6b6a64);
    font-style: italic;
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
    /* Suwaidi gold — hard-coded so the Accept button stays visually
       consistent with the Sight aesthetic regardless of the user's
       theme (--interactive-accent often resolves to purple). */
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
  .srp-edit-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px;
    margin-bottom: 8px;
  }
  .srp-edit-pill {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 6px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
    background: var(--background-primary, #fff);
    border: 1px solid var(--background-modifier-border, rgba(0,0,0,0.08));
  }
  .srp-edit-pill.active {
    background: rgba(201, 162, 39, 0.12);
    /* Suwaidi gold — hard-coded for consistency with the primary
       suggestion border + Accept button. */
    border-color: #c9a227;
  }
  .srp-edit-pill input {
    margin: 0;
  }
</style>
