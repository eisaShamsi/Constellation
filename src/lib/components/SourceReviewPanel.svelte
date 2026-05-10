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
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { t, locale, type Locale } from '$lib/i18n';
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
    /** MIG-021v3 V3-§8 — composite reasoning trail JSON. None for v2-era rows. */
    composite_json?: string | null;
  };

  // MIG-021v3 V3-§8 — composite reasoning trail shape (mirrors
  // src-tauri/src/cece/synthesis.rs::CompositeAssignment serde).
  type AxisDecision = {
    primary?: string | null;
    secondary?: string[];
    regime: 'unanimous' | 'strong_majority' | 'split';
    see_also: string[];
    needs_user_disambiguation_between?: string[] | null;
    dissenter?: string | null;
  };
  type PerCatalogerTrail = {
    cataloger: string;
    voiced_opinion: boolean;
    horizontal: { id: string; primary: boolean; weight: number }[];
    vertical: { id: string; primary: boolean; weight: number }[];
    reasoning: string;
    rules_fired: string[];
    self_reported_confidence: 'high' | 'medium' | 'low' | 'abstain';
  };
  type CompositeAssignment = {
    horizontal: AxisDecision;
    vertical: AxisDecision;
    composite_reasoning: string;
    catalogers_voiced: string[];
    catalogers_silent: string[];
    synthesis_method: string;
    per_cataloger_trails: PerCatalogerTrail[];
  };

  /** Parse the composite_json blob lazily; returns null on legacy rows. */
  function parseComposite(record: SuggestionRecord): CompositeAssignment | null {
    if (!record.composite_json) return null;
    try {
      return JSON.parse(record.composite_json) as CompositeAssignment;
    } catch {
      return null;
    }
  }

  /** Per-cataloger badge state for the header cluster. */
  function catalogerBadgeStatus(
    composite: CompositeAssignment | null,
    catalogerName: string,
  ): '✓' | '–' | '✗' {
    if (!composite) return '–';
    const trail = composite.per_cataloger_trails.find(t => t.cataloger === catalogerName);
    if (!trail || !trail.voiced_opinion) return '–';
    // Voiced: did this cataloger's primary match the synthesis primary?
    const primH = composite.horizontal.primary;
    const primV = composite.vertical.primary;
    const trailH = trail.horizontal.find(a => a.primary)?.id;
    const trailV = trail.vertical.find(a => a.primary)?.id;
    const horizontalAgrees = !primH || !trailH || trailH === primH;
    const verticalAgrees = !primV || !trailV || trailV === primV;
    return horizontalAgrees && verticalAgrees ? '✓' : '✗';
  }

  /** Per-card "show reasoning trail" expand state. */
  let expandedTrails = $state(new Set<string>());
  function toggleTrail(notePath: string) {
    const next = new Set(expandedTrails);
    if (next.has(notePath)) next.delete(notePath);
    else next.add(notePath);
    expandedTrails = next;
  }

  // The six catalogers, in render order. Mirrors the orchestrator's
  // cost-ordered registration.
  const CATALOGER_ORDER = [
    'user_authority',
    'structural',
    'linguistic',
    'graph',
    'semantic',
    'reasoning',
  ];

  /**
   * Short label for the per-cataloger badge tooltip.
   *
   * V3-§8.r1.e fix (audit P0.6): switched to plain-English user-facing
   * labels per the UX agent recommendation ("Your frontmatter" / "Linked
   * notes" / "Similar notes" / "Citations & structure" / "Wordstems &
   * lexicon" / "AI judgment"). Returns the AR translation when
   * currentLocale is Arabic. Falls back to the technical name if neither
   * is wired (defensive — should never happen if cece.* i18n keys ship).
   */
  function catalogerLabel(c: string): string {
    const isAr = currentLocale === 'ar';
    if (isAr) {
      switch (c) {
        case 'user_authority': return 'واجهتك الأمامية';
        case 'structural': return 'الاستشهادات والبنية';
        case 'linguistic': return 'الجذور والمعجم';
        case 'graph': return 'الملاحظات المرتبطة';
        case 'semantic': return 'الملاحظات المشابهة';
        case 'reasoning': return 'حُكم الذكاء الاصطناعي';
        default: return c;
      }
    }
    switch (c) {
      case 'user_authority': return 'Your frontmatter';
      case 'structural': return 'Citations & structure';
      case 'linguistic': return 'Wordstems & lexicon';
      case 'graph': return 'Linked notes';
      case 'semantic': return 'Similar notes';
      case 'reasoning': return 'AI judgment';
      default: return c;
    }
  }

  /** 3-letter abbreviation for the badge text. */
  function catalogerAbbr(c: string): string {
    switch (c) {
      case 'user_authority': return 'UA';
      case 'structural': return 'STR';
      case 'linguistic': return 'LIN';
      case 'graph': return 'GRP';
      case 'semantic': return 'SEM';
      case 'reasoning': return 'RSN';
      default: return c.slice(0, 3).toUpperCase();
    }
  }

  /**
   * V3-§8.r1.f — Sibling Disambiguation pick handler. Calls the new
   * cece_resolve_disambiguation IPC, then drops the resolved card
   * from the queue.
   */
  async function resolveDisambiguation(notePath: string, axis: 'horizontal' | 'vertical', chosenId: string) {
    try {
      await invoke('cece_resolve_disambiguation', {
        notePath,
        axis,
        chosenId,
      });
      queue = queue.filter(r => r.note_path !== notePath);
    } catch (e) {
      error = String(e);
    }
  }

  /**
   * For tooltip on each disambiguation chip — find which voicing
   * cataloger preferred this candidate, return its one-line reasoning.
   * Helps the user understand why each candidate is on the list.
   */
  function catalogerReasonForCandidate(
    composite: CompositeAssignment,
    axis: 'horizontal' | 'vertical',
    candId: string,
  ): string {
    for (const trail of composite.per_cataloger_trails) {
      if (!trail.voiced_opinion) continue;
      const assignments = axis === 'horizontal' ? trail.horizontal : trail.vertical;
      if (assignments.some(a => a.id === candId)) {
        return `${catalogerLabel(trail.cataloger)}: ${trail.reasoning}`;
      }
    }
    return candId;
  }

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

  // Locale-aware label lookup. Returns Arabic when interface locale is 'ar';
  // English otherwise. Both EN and AR are present on every taxonomy node
  // (lifted from Eisa's two HTML diagrams). Falls back to ID if node
  // missing (shouldn't happen unless legacy data has a stale slug).
  let currentLocale = $state<Locale>('en');
  $effect(() => {
    return locale.subscribe((l) => {
      currentLocale = l;
    });
  });

  function labelForId(id: string, axis: string): string {
    const isArabic = currentLocale === 'ar';
    if (axis === 'horizontal') {
      const node = horizontalTaxonomy.find(n => n.id === id);
      if (!node) return id;
      return isArabic ? node.ar : node.en;
    }
    const node = verticalTaxonomy.find(n => n.id === id);
    if (!node) return id;
    return isArabic ? node.ar : node.en;
  }

  // MIG-021v2 §1E' — listener for the right-click "Suggest sources & content
  // type" context action (dispatched from +layout.svelte). Classifies the
  // requested note + prepends to the queue + flashes the new entry so the
  // user sees it.
  let highlightedPath = $state<string | null>(null);
  let highlightTimeout: ReturnType<typeof setTimeout> | null = null;

  async function handleClassifyAndShow(e: Event) {
    const detail = (e as CustomEvent).detail as { notePath?: string };
    const notePath = detail?.notePath;
    if (!notePath || classifying) return;
    classifying = true;
    error = null;
    try {
      const record = await invoke<SuggestionRecord>('classifier_suggest_for_note', {
        notePath,
      });
      queue = [record, ...queue.filter(r => r.note_path !== record.note_path)];
      highlightedPath = record.note_path;
      if (highlightTimeout) clearTimeout(highlightTimeout);
      highlightTimeout = setTimeout(() => {
        highlightedPath = null;
        highlightTimeout = null;
      }, 2000);
    } catch (err) {
      error = String(err);
    } finally {
      classifying = false;
    }
  }

  // MIG-021v2 §1F' fix-1 — live-update during background scan. Without
  // this the queue count and visible cards only refresh when the panel
  // re-mounts (i.e. when the user switches away and back). The Rust
  // scan_job emits `classifier:scan` events every 5 notes; we debounce
  // a queue reload to ~1.5 s so we don't thrash on a fast scan.
  let scanReloadTimer: ReturnType<typeof setTimeout> | null = null;
  let scanUnlisten: (() => void) | null = null;
  let bulkUnlisten: (() => void) | null = null;

  // MIG-021v2 §1F'.b — Bulk Approve All / Reject All state.
  let bulkConfirm = $state<null | 'accept' | 'reject'>(null);
  let bulkRunning = $state(false);
  let bulkCompleted = $state(0);
  let bulkTotal = $state(0);
  let bulkCancelling = $state(false);

  async function startBulkAccept() {
    bulkConfirm = null;
    bulkRunning = true;
    bulkCompleted = 0;
    bulkTotal = queue.length;
    try {
      await invoke('sources_accept_all_pending');
    } catch (e) {
      error = String(e);
      bulkRunning = false;
    }
  }

  async function cancelBulkAccept() {
    bulkCancelling = true;
    try {
      await invoke('sources_bulk_accept_cancel');
    } catch (e) {
      console.error('[BulkAccept] cancel failed:', e);
    }
  }

  async function startBulkReject() {
    bulkConfirm = null;
    try {
      const cleared = await invoke<number>('sources_reject_all_pending');
      console.log(`[BulkReject] cleared ${cleared} suggestions`);
      await loadQueue();
    } catch (e) {
      error = String(e);
    }
  }

  function scheduleQueueReload() {
    if (scanReloadTimer) return;
    scanReloadTimer = setTimeout(() => {
      scanReloadTimer = null;
      loadQueue();
    }, 1500);
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
    window.addEventListener('constellation:classify-and-show', handleClassifyAndShow);

    // Listen for backend scan progress so the queue updates live.
    const { listen } = await import('@tauri-apps/api/event');
    const unlisten = await listen<{ phase: string }>('classifier:scan', (ev) => {
      const phase = ev.payload?.phase;
      if (phase === 'progress' || phase === 'done' || phase === 'cancelled') {
        scheduleQueueReload();
      }
    });
    scanUnlisten = unlisten;

    // MIG-021v2 §1F'.b — bulk-accept progress events drive the inline
    // progress bar + auto-reload the queue when it finishes.
    const unlistenBulk = await listen<{ phase: string; total: number; completed: number }>(
      'sources:bulk_accept',
      (ev) => {
        const p = ev.payload;
        if (p.phase === 'start') {
          bulkRunning = true;
          bulkCompleted = 0;
          bulkTotal = p.total;
          bulkCancelling = false;
        } else if (p.phase === 'progress') {
          bulkCompleted = p.completed;
          bulkTotal = p.total;
          scheduleQueueReload();
        } else if (p.phase === 'done' || p.phase === 'cancelled' || p.phase === 'error') {
          bulkCompleted = p.completed;
          bulkTotal = p.total;
          bulkRunning = false;
          bulkCancelling = false;
          loadQueue();
        }
      },
    );
    bulkUnlisten = unlistenBulk;
  });

  onDestroy(() => {
    window.removeEventListener('constellation:classify-and-show', handleClassifyAndShow);
    if (highlightTimeout) clearTimeout(highlightTimeout);
    if (scanReloadTimer) clearTimeout(scanReloadTimer);
    scanUnlisten?.();
    bulkUnlisten?.();
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
    <div class="srp-count-row">
      <span class="srp-count">
        {queue.length}
        {queue.length === 1
          ? ($t('sources.review.pending') || 'pending')
          : ($t('sources.review.pendingPlural') || 'pending')}
      </span>
      <!-- MIG-021v2 §1F'.b — bulk Approve All / Reject All -->
      <span class="srp-bulk-actions">
        <button
          class="srp-bulk-btn srp-bulk-accept"
          onclick={() => bulkConfirm = 'accept'}
          disabled={bulkRunning}
          title={$t('sources.review.acceptAllTitle') || 'Apply every queued suggestion to its note'}
        >
          {$t('sources.review.acceptAll') || 'Approve all'}
        </button>
        <button
          class="srp-bulk-btn srp-bulk-reject"
          onclick={() => bulkConfirm = 'reject'}
          disabled={bulkRunning}
          title={$t('sources.review.rejectAllTitle') || 'Clear every queued suggestion without writing'}
        >
          {$t('sources.review.rejectAll') || 'Reject all'}
        </button>
      </span>
    </div>

    {#if bulkRunning}
      <div class="srp-bulk-progress">
        <div class="srp-bulk-progress-text">
          {bulkCancelling
            ? ($t('sources.review.bulkCancelling') || 'Cancelling…')
            : ($t('sources.review.bulkRunning') || 'Approving…')}
          — {bulkCompleted.toLocaleString()} / {bulkTotal.toLocaleString()}
          {#if bulkTotal > 0}
            ({Math.floor((bulkCompleted / bulkTotal) * 100)}%)
          {/if}
        </div>
        <button
          class="srp-bulk-cancel"
          onclick={cancelBulkAccept}
          disabled={bulkCancelling}
        >
          {$t('sources.review.bulkCancel') || 'Cancel'}
        </button>
      </div>
    {/if}

    {#if bulkConfirm}
      <div class="srp-bulk-confirm" role="dialog" aria-modal="true">
        <div class="srp-bulk-confirm-text">
          {bulkConfirm === 'accept'
            ? ($t('sources.review.confirmAcceptAll') || 'Apply every suggestion in the queue to its note? This writes both axes\' top suggestions to {N} notes\' frontmatter.').replace('{N}', queue.length.toLocaleString())
            : ($t('sources.review.confirmRejectAll') || 'Clear every suggestion in the queue without writing? You can re-run the scan later to regenerate them.')}
        </div>
        <div class="srp-bulk-confirm-actions">
          <button
            class="srp-btn"
            onclick={() => bulkConfirm = null}
          >
            {$t('sources.review.cancel') || 'Cancel'}
          </button>
          <button
            class="srp-btn srp-btn-primary"
            onclick={() => bulkConfirm === 'accept' ? startBulkAccept() : startBulkReject()}
          >
            {bulkConfirm === 'accept'
              ? ($t('sources.review.acceptAll') || 'Approve all')
              : ($t('sources.review.rejectAll') || 'Reject all')}
          </button>
        </div>
      </div>
    {/if}

    <ul class="srp-list">
      {#each queue as record (record.note_path)}
        {@const horizontalSuggestions = record.suggestions.filter(s => s.axis === 'horizontal')}
        {@const verticalSuggestions = record.suggestions.filter(s => s.axis === 'vertical')}
        {@const composite = parseComposite(record)}
        {@const isSplit = composite && (composite.horizontal.regime === 'split' || composite.vertical.regime === 'split')}
        {@const isStrongMajority = composite && (composite.horizontal.regime === 'strong_majority' || composite.vertical.regime === 'strong_majority')}
        {@const showTrail = expandedTrails.has(record.note_path)}
        <li class="srp-card"
            class:srp-just-added={highlightedPath === record.note_path}
            class:srp-split-regime={isSplit}>
          <div class="srp-card-header">
            <button
              class="srp-card-title"
              onclick={() => onNoteClick(record.note_path, noteName(record.note_path))}
              title={record.note_path}
            >
              {noteName(record.note_path)}
            </button>
            {#if composite}
              <!-- MIG-021v3 V3-§8 — per-cataloger badge cluster -->
              <span class="srp-cataloger-cluster" title="Cataloger ensemble: ✓ agrees with synthesis, ✗ dissents, – silent">
                {#each CATALOGER_ORDER as catName}
                  {@const status = catalogerBadgeStatus(composite, catName)}
                  <span
                    class="srp-cataloger-badge"
                    class:srp-cataloger-voiced={status === '✓'}
                    class:srp-cataloger-dissent={status === '✗'}
                    class:srp-cataloger-silent={status === '–'}
                    title={`${catalogerLabel(catName)}: ${status === '✓' ? 'agrees with synthesis' : status === '✗' ? 'dissents' : 'silent (no signal in this lens)'}`}
                  >
                    {catalogerAbbr(catName)} {status}
                  </span>
                {/each}
              </span>
            {:else}
              <!-- Legacy v2-era row: single tier badge -->
              <span class="srp-tier" title={$t('sources.review.tierLabel') || 'Classifier tier'}>
                {record.classifier_tier === 2 ? 'T2' : 'T1'}
              </span>
            {/if}
          </div>

          {#if composite && (isSplit || isStrongMajority)}
            <!-- Reasoning trail surface (on disagreement only by default) -->
            <div class="srp-trail-toggle">
              <button class="srp-trail-btn" onclick={() => toggleTrail(record.note_path)}>
                {#if showTrail}{$t('cece.trail.collapse') || '▾ Hide reasoning'}{:else}{$t('cece.trail.expand') || '▸ Why this classification?'}{/if}
              </button>
              {#if isSplit}
                <span class="srp-split-pill">{$t('cece.regime.split') || 'Catalogers split — needs your call'}</span>
              {:else if isStrongMajority}
                <span class="srp-majority-pill">
                  {$t('cece.regime.strongMajority') || 'Strong majority'} {#if composite.horizontal.dissenter || composite.vertical.dissenter}({composite.horizontal.dissenter ?? composite.vertical.dissenter}){/if}
                </span>
              {/if}
            </div>
            {#if showTrail}
              <div class="srp-trail">
                <div class="srp-trail-summary">{composite.composite_reasoning}</div>
                <ul class="srp-trail-list">
                  {#each composite.per_cataloger_trails.filter(t => t.voiced_opinion) as t}
                    <li class="srp-trail-item">
                      <strong>{catalogerLabel(t.cataloger)}</strong>
                      <span class="srp-trail-conf">[{t.self_reported_confidence}]</span>
                      — {t.reasoning}
                    </li>
                  {/each}
                </ul>
              </div>
            {/if}

            {#if isSplit}
              <!-- V3-§8.r1.f — Sibling Disambiguation form. Renders the
                   actual radio-chip picker the Architect §3.1 specified.
                   For each axis whose regime is split, list the candidate
                   IDs from `needs_user_disambiguation_between` as a
                   one-click chip row. User picks; cece_resolve_disambiguation
                   IPC writes to frontmatter + clears the suggestion. -->
              <div class="srp-disambig">
                <div class="srp-disambig-prompt">{$t('cece.disambiguation.prompt') || 'The catalogers split between these candidates. Pick which one fits the note best:'}</div>
                {#if composite.horizontal.regime === 'split' && composite.horizontal.needs_user_disambiguation_between}
                  <div class="srp-disambig-axis">
                    <div class="srp-disambig-axis-label">{$t('cece.disambiguation.axisHorizontal') || 'Source'}</div>
                    <div class="srp-disambig-chips">
                      {#each composite.horizontal.needs_user_disambiguation_between as candId}
                        <button
                          class="srp-disambig-chip"
                          onclick={() => resolveDisambiguation(record.note_path, 'horizontal', candId)}
                          title={catalogerReasonForCandidate(composite, 'horizontal', candId)}
                        >
                          {labelForId(candId, 'horizontal')}
                        </button>
                      {/each}
                    </div>
                  </div>
                {/if}
                {#if composite.vertical.regime === 'split' && composite.vertical.needs_user_disambiguation_between}
                  <div class="srp-disambig-axis">
                    <div class="srp-disambig-axis-label">{$t('cece.disambiguation.axisVertical') || 'Content type'}</div>
                    <div class="srp-disambig-chips">
                      {#each composite.vertical.needs_user_disambiguation_between as candId}
                        <button
                          class="srp-disambig-chip"
                          onclick={() => resolveDisambiguation(record.note_path, 'vertical', candId)}
                          title={catalogerReasonForCandidate(composite, 'vertical', candId)}
                        >
                          {labelForId(candId, 'vertical')}
                        </button>
                      {/each}
                    </div>
                  </div>
                {/if}
              </div>
            {/if}
          {/if}

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
                        {@const parentId = horizontalTaxonomy.find(n => n.id === s.source)?.parent_id ?? s.source}
                        {@const evidenceKey = 'sources.evidence.' + (s.source.includes('/') ? parentId : s.source)}
                        {@const translated = $t(evidenceKey)}
                        <div class="srp-evidence">
                          {translated && translated !== evidenceKey ? translated : s.evidence}
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
    /* Always stacked per Eisa directive 2026-05-09: side-by-side
       even at ≥1200px makes each picker too narrow on desktop sidebars
       AND splits attention. Stack vertically; user scrolls. */
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 8px;
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
  /* MIG-021v3 V3-§8 — per-cataloger badge cluster + reasoning trail */
  .srp-cataloger-cluster {
    display: inline-flex; flex-wrap: wrap; gap: 3px;
    align-items: center;
  }
  .srp-cataloger-badge {
    font-size: 9px;
    padding: 1px 4px;
    border-radius: 3px;
    font-weight: 600;
    line-height: 1.2;
    user-select: none;
    white-space: nowrap;
  }
  .srp-cataloger-voiced {
    background: rgba(15, 110, 86, 0.15);
    color: #0f6e56;
    border: 1px solid rgba(15, 110, 86, 0.3);
  }
  .srp-cataloger-dissent {
    background: rgba(168, 50, 50, 0.15);
    color: #a83232;
    border: 1px solid rgba(168, 50, 50, 0.3);
  }
  .srp-cataloger-silent {
    background: rgba(0, 0, 0, 0.04);
    color: var(--text-faint, #6b6a64);
    border: 1px solid rgba(0, 0, 0, 0.08);
  }
  .srp-card.srp-split-regime {
    border-left: 3px solid #c9a227;
    padding-left: 8px;
  }
  .srp-trail-toggle {
    display: flex; align-items: center; gap: 8px;
    padding: 4px 8px;
    flex-wrap: wrap;
  }
  .srp-trail-btn {
    background: transparent; border: none;
    color: var(--text-muted, #6b6a64);
    font-size: 11px; cursor: pointer;
    padding: 2px 0;
  }
  .srp-trail-btn:hover { color: var(--text-normal); }
  .srp-split-pill {
    font-size: 10px;
    padding: 2px 6px;
    background: rgba(201, 162, 39, 0.18);
    color: #856204;
    border-radius: 3px;
    border: 1px solid rgba(201, 162, 39, 0.4);
  }
  .srp-majority-pill {
    font-size: 10px;
    padding: 2px 6px;
    background: rgba(83, 74, 183, 0.12);
    color: #534ab7;
    border-radius: 3px;
    border: 1px solid rgba(83, 74, 183, 0.3);
  }
  .srp-trail {
    padding: 6px 12px 8px;
    background: rgba(0, 0, 0, 0.02);
    border-block: 1px solid var(--background-modifier-border, rgba(0,0,0,0.06));
    font-size: 11px;
    color: var(--text-normal);
  }
  .srp-trail-summary {
    margin-bottom: 6px;
    font-style: italic;
    color: var(--text-muted);
  }
  .srp-trail-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .srp-trail-item {
    padding: 3px 0;
    border-top: 1px dashed var(--background-modifier-border, rgba(0,0,0,0.06));
  }
  .srp-trail-item:first-child { border-top: none; }
  .srp-trail-conf {
    font-size: 10px;
    color: var(--text-faint);
    margin-inline-start: 4px;
  }

  /* MIG-021v3 V3-§8.r1.f — Sibling Disambiguation form */
  .srp-disambig {
    padding: 8px 12px;
    background: rgba(201, 162, 39, 0.06);
    border-block-start: 1px solid rgba(201, 162, 39, 0.25);
    display: flex; flex-direction: column; gap: 6px;
  }
  .srp-disambig-prompt {
    font-size: 11px;
    color: var(--text-normal);
    line-height: 1.4;
  }
  .srp-disambig-axis {
    display: flex; flex-direction: column; gap: 4px;
  }
  .srp-disambig-axis-label {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
  }
  .srp-disambig-chips {
    display: flex; flex-wrap: wrap; gap: 6px;
  }
  .srp-disambig-chip {
    border: 1px solid #c9a227;
    background: var(--background-primary, #fff);
    color: var(--text-normal, #1a1a1a);
    padding: 4px 10px;
    border-radius: 12px;
    font-size: 11px;
    cursor: pointer;
    transition: background 0.12s;
  }
  .srp-disambig-chip:hover {
    background: rgba(201, 162, 39, 0.18);
  }

  /* MIG-021v2 §1F'.b — bulk Approve/Reject actions */
  .srp-count-row {
    display: flex; align-items: center; justify-content: space-between;
    gap: 8px; padding: 4px 8px;
    flex-wrap: wrap;
  }
  .srp-bulk-actions {
    display: flex; gap: 6px;
  }
  .srp-bulk-btn {
    padding: 3px 10px; font-size: 11px;
    border-radius: 4px; cursor: pointer;
    border: 1px solid var(--background-modifier-border, rgba(0,0,0,0.18));
    background: transparent;
    color: var(--text-normal, #1a1a1a);
  }
  .srp-bulk-btn:hover { background: var(--background-modifier-hover, rgba(0,0,0,0.05)); }
  .srp-bulk-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .srp-bulk-accept { color: #c9a227; border-color: rgba(201, 162, 39, 0.4); }
  .srp-bulk-reject { color: #a83232; border-color: rgba(168, 50, 50, 0.4); }
  .srp-bulk-progress {
    display: flex; align-items: center; justify-content: space-between;
    gap: 8px; padding: 6px 8px;
    background: rgba(201, 162, 39, 0.10);
    border-block: 1px solid rgba(201, 162, 39, 0.25);
    font-size: 11px; color: var(--text-normal);
  }
  .srp-bulk-progress-text { font-variant-numeric: tabular-nums; }
  .srp-bulk-cancel {
    border: 1px solid var(--background-modifier-border, rgba(0,0,0,0.18));
    background: transparent;
    color: var(--text-muted);
    border-radius: 4px;
    padding: 1px 8px;
    font-size: 11px;
    cursor: pointer;
  }
  .srp-bulk-cancel:disabled { opacity: 0.5; cursor: not-allowed; }
  .srp-bulk-confirm {
    margin: 8px;
    padding: 10px 12px;
    border: 1px solid #c9a227;
    background: rgba(201, 162, 39, 0.08);
    border-radius: 6px;
    display: flex; flex-direction: column; gap: 8px;
  }
  .srp-bulk-confirm-text {
    font-size: 12px; line-height: 1.5; color: var(--text-normal);
  }
  .srp-bulk-confirm-actions {
    display: flex; gap: 8px; justify-content: flex-end;
  }

  /* MIG-021v2 §1E' — flash animation when a record is added via the
     right-click "Suggest sources & content type" action, so the user
     can spot the just-classified note in the queue. */
  .srp-just-added {
    animation: srp-flash 2s ease-out;
  }
  @keyframes srp-flash {
    0%   { background: rgba(201, 162, 39, 0.32); }
    100% { background: transparent; }
  }
</style>
