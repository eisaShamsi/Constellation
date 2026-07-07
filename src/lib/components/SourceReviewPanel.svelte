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
  // MIG-043 Phase 1 — shared NSC summary store (cache-first, batched, coalesced).
  // Replaces this component's prior direct `invoke('nsc_get_summaries_for_notes')`.
  // Behavior here is unchanged; we just funnel the IPC through the shared
  // store so future surfaces (search results, editor header, the Digest)
  // share the cache + dedup with this panel.
  import { getSummariesFor } from '$lib/nsc/summaryStore';
  import { t, locale, type Locale } from '$lib/i18n';
  import { appSettings } from '$lib/libraries/store';
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
  // MIG-022 §E.2 (PJ-041) — structured i18n template for reasoning prose.
  // Each cataloger emits {key, params}; frontend resolves via $t() with
  // raw `reasoning` as fallback for legacy data + Reasoning cataloger.
  type ReasoningTemplate = {
    key: string;
    params: Record<string, unknown>;
  };
  type PerCatalogerTrail = {
    cataloger: string;
    voiced_opinion: boolean;
    horizontal: { id: string; primary: boolean; weight: number }[];
    vertical: { id: string; primary: boolean; weight: number }[];
    reasoning: string;
    reasoning_template?: ReasoningTemplate | null;
    rules_fired: string[];
    self_reported_confidence: 'high' | 'medium' | 'low' | 'abstain';
  };
  type CompositeAssignment = {
    horizontal: AxisDecision;
    vertical: AxisDecision;
    composite_reasoning: string;
    composite_reasoning_template?: ReasoningTemplate | null;
    catalogers_voiced: string[];
    catalogers_silent: string[];
    synthesis_method: string;
    per_cataloger_trails: PerCatalogerTrail[];
  };

  /** Parse the composite_json blob lazily; returns null on legacy rows. */
  // MIG-039 perf: memoize the JSON.parse of composite_json. filterCounts,
  // splitAwareSkipCount, filteredQueue, AND the per-card render each call
  // parseComposite for every record; on a 4,000+ row queue that re-parsed
  // every blob many times per reactive pass (a big contributor to the lag /
  // memory pressure Eisa hit). Keyed by the immutable composite_json string,
  // so unchanged rows stay cached across scan reloads. Component-local — GC'd
  // with the instance.
  const _compositeCache = new Map<string, CompositeAssignment | null>();
  function parseComposite(record: SuggestionRecord): CompositeAssignment | null {
    if (!record.composite_json) return null;
    const hit = _compositeCache.get(record.composite_json);
    if (hit !== undefined) return hit;
    let parsed: CompositeAssignment | null;
    try {
      parsed = JSON.parse(record.composite_json) as CompositeAssignment;
    } catch {
      parsed = null;
    }
    _compositeCache.set(record.composite_json, parsed);
    return parsed;
  }

  /**
   * V3-§8.r7 Issue #1 fix — does this card actually need the user's call?
   *
   * The Gate 1 Boss-test 2026-05-10 surfaced a discrepancy: the per-card
   * pill correctly detected UA-short-circuited cards as Unanimous, but
   * the queue-level `splitCount` (which used `regime === 'split'`) was
   * including the same card as Split. Confirmed via a Rust test that the
   * synthesis output has `regime: "unanimous"` for both axes when UA
   * short-circuits — so the bug is/was somewhere in the data round-trip
   * that I couldn't reproduce in isolation.
   *
   * Robust fix: filter on the SEMANTIC PROPERTY (was the user asked to
   * disambiguate?) rather than the regime string. The synthesis layer
   * only populates `needs_user_disambiguation_between` when regime ==
   * Split (`synthesis.rs:268-272`). UA short-circuit explicitly sets
   * both axes' `needs_user_disambiguation_between` to None
   * (`synthesis.rs:136, 153`). So if either axis has a populated
   * disambiguation array, the user must pick — full stop. Robust against
   * any serialization weirdness in the regime field.
   */
  function cardNeedsUserCall(record: SuggestionRecord): boolean {
    const c = parseComposite(record);
    if (!c) return false;
    const hNeed = c.horizontal.needs_user_disambiguation_between;
    const vNeed = c.vertical.needs_user_disambiguation_between;
    return (Array.isArray(hNeed) && hNeed.length > 0)
        || (Array.isArray(vNeed) && vNeed.length > 0);
  }

  /**
   * V3-§8.r8 — per-axis "needs your call" predicate. Used by the
   * composition filter to slice the queue into "Source needs your
   * call" / "Content type needs your call" / "Both" / "Agreed"
   * buckets.
   */
  function axisNeedsUserCall(c: CompositeAssignment | null, axis: 'horizontal' | 'vertical'): boolean {
    if (!c) return false;
    const need = c[axis].needs_user_disambiguation_between;
    return Array.isArray(need) && need.length > 0;
  }

  /**
   * V3-§8.r8 — composition filter for the queue. Default 'all' renders
   * the full queue (existing behavior); the four other buckets slice
   * by what kind of decision the card needs from the user. Solves the
   * "find the needle in 268-card haystack" problem the Boss-test
   * surfaced 2026-05-10.
   *
   * Note: legacy v2-era cards (no composite_json) appear ONLY in 'all'
   * — they have no per-axis Split state to filter on.
   */
  type QueueFilter = 'all' | 'both' | 'source' | 'content_type' | 'agreed';
  let queueFilter = $state<QueueFilter>('all');

  /** Per-bucket counts for the filter chip labels. Derived once over
   *  the full queue; updates reactively when the queue changes. */
  let filterCounts = $derived.by(() => {
    let both = 0, source = 0, content = 0, agreed = 0;
    for (const r of queue) {
      const c = parseComposite(r);
      if (!c) continue;
      const h = axisNeedsUserCall(c, 'horizontal');
      const v = axisNeedsUserCall(c, 'vertical');
      if (h && v) both++;
      else if (h) source++;
      else if (v) content++;
      else agreed++;
    }
    return { all: queue.length, both, source, content, agreed };
  });

  /** Filtered render-list. Operates only on the rendered queue; the
   *  full queue is preserved for the count strip + Approve All math. */
  let filteredQueue = $derived.by(() => {
    if (queueFilter === 'all') return queue;
    return queue.filter(r => {
      const c = parseComposite(r);
      if (!c) return false; // legacy rows only show in 'all'
      const h = axisNeedsUserCall(c, 'horizontal');
      const v = axisNeedsUserCall(c, 'vertical');
      switch (queueFilter) {
        case 'both':         return h && v;
        case 'source':       return h && !v;
        case 'content_type': return v && !h;
        case 'agreed':       return !h && !v;
      }
    });
  });

  // MIG-039 perf: cap how many cards actually render. The full queue can be
  // thousands of rows; rendering them all (each a complex card with nested
  // loops) is what froze the app on a large universe. Filter counts + Approve
  // All still operate on the full queue — only the DOM is bounded. "Show more"
  // reveals the next batch on demand. Reset to one batch when the filter changes.
  const RENDER_BATCH = 80;
  let renderCap = $state(RENDER_BATCH);
  let visibleQueue = $derived(filteredQueue.slice(0, renderCap));

  // ── MIG-040 (NSC) / MIG-043 Phase 1: per-note summaries shown under each
  //    card title. Fetched once per visible note via the shared NSC store's
  //    batched cache-first API (zero per-card IPC, same lesson as the render
  //    cap). `summaryRequested` is a PLAIN Set (not reactive) so the effect
  //    reads only `visibleQueue` and writes `summaries` — never reading +
  //    writing the same reactive (Rule 2).
  // (MIG-043 Phase 1: deleted the local `NoteSummaryEntry` type alias here —
  // it predated the shared store and missed the `headline` field; surfaces
  // now use the canonical `NoteSummaryEntry` from `$lib/nsc/summaryStore`.
  // SRP itself doesn't render `headline`, so its local `summaries` map keeps
  // its narrow `{summary, source}` shape — no behavior change.)
  let summaries = $state<Map<string, { summary: string; source: string }>>(new Map());
  const summaryRequested = new Set<string>();
  // MIG-040 fix: NSC must be GENTLE. The first cut eagerly computed summaries
  // for all ~80 visible notes at once and re-fired on every scan reload,
  // spiking embedding work (the regression Eisa caught). Now we fill in small
  // chunks, debounced, and PAUSE while a classifier scan runs — so NSC never
  // piles up or fights the scanner for the embedding engine.
  let summaryScanRunning = $state(false);
  const SUMMARY_CHUNK = 6;
  let summaryFillTimer: ReturnType<typeof setTimeout> | null = null;

  async function fetchSummaries(paths: string[]) {
    try {
      // MIG-043 Phase 1: shared store handles cache + batching + coalescing
      // across surfaces. SRP's own `summaryRequested` set still scopes the
      // gentle-fill scheduling here; the store prevents duplicate IPC when
      // any other surface (post-Phase 1) asks for the same path.
      const map = await getSummariesFor(paths);
      const next = new Map(summaries);
      for (const e of map.values()) next.set(e.path, { summary: e.summary, source: e.source });
      // Mark paths that returned nothing (no body) as empty so we don't refetch.
      for (const p of paths) if (!next.has(p)) next.set(p, { summary: '', source: '' });
      summaries = next;
    } catch (e) {
      console.error('[NSC] fetch summaries failed:', e);
    }
  }

  function scheduleSummaryFill() {
    if (summaryFillTimer) return;
    summaryFillTimer = setTimeout(() => {
      summaryFillTimer = null;
      void fillNextSummaryChunk();
    }, 500);
  }

  async function fillNextSummaryChunk() {
    // Hold off entirely while a scan is running — summaries can wait.
    if (summaryScanRunning) {
      scheduleSummaryFill();
      return;
    }
    const pending = visibleQueue.map(r => r.note_path).filter(p => !summaryRequested.has(p));
    if (pending.length === 0) return;
    const chunk = pending.slice(0, SUMMARY_CHUNK);
    for (const p of chunk) summaryRequested.add(p);
    await fetchSummaries(chunk);
    if (pending.length > chunk.length) scheduleSummaryFill(); // more to do
  }

  // Re-trigger gentle filling when the visible set changes OR a scan ends. The
  // effect does NO heavy work — it only schedules the throttled, chunked,
  // scan-paused filler. Reads visibleQueue + summaryScanRunning; writes
  // neither (no Rule-2 violation).
  $effect(() => {
    visibleQueue;
    summaryScanRunning;
    scheduleSummaryFill();
  });

  // MIG-039 — reload the queue when the Cataloger is reopened so notes
  // classified via the right-sidebar "Classify open note" button appear
  // without the user having to hit the manual refresh icon.
  //
  // Design: `_srp_was_closed` is a plain (non-reactive) JS variable so the
  // effect tracks ONLY `visible` — no Rule-2 violation.  On first mount
  // (visible = true, _srp_was_closed = false) the guard skips the reload;
  // `onMount` already called `loadQueue()`.  When the Cataloger is closed
  // (visible → false) we flip the flag.  On the next open (visible → true,
  // _srp_was_closed = true) we call `loadQueue()` to hydrate with anything
  // classified in the meantime.
  //
  // Right-sidebar instances use the default `visible = true` and the flag
  // never flips, so no spurious reload on them.
  let _srp_was_closed = false;
  $effect(() => {
    if (!visible) {
      _srp_was_closed = true;
      return;
    }
    if (_srp_was_closed) loadQueue();
  });

  /** Filter chip definitions for the queueFilter row. Recomputes when
   *  i18n locale changes (the labels read $t) or when filterCounts
   *  shifts. Declared as $derived so the template can iterate without
   *  an inline {@const} (which Svelte 5 only allows inside specific
   *  block parents). */
  let filterChips = $derived.by(() => {
    return [
      { key: 'all'          as QueueFilter, label: $t('cece.queueFilter.all')         || 'All',                          count: filterCounts.all },
      { key: 'both'         as QueueFilter, label: $t('cece.queueFilter.both')        || 'Both axes need your call',     count: filterCounts.both },
      { key: 'source'       as QueueFilter, label: $t('cece.queueFilter.source')      || 'Source needs your call',       count: filterCounts.source },
      { key: 'content_type' as QueueFilter, label: $t('cece.queueFilter.contentType') || 'Content type needs your call', count: filterCounts.content },
      { key: 'agreed'       as QueueFilter, label: $t('cece.queueFilter.agreed')      || 'Catalogers agreed',            count: filterCounts.agreed },
    ];
  });

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

  /** Per-card reasoning-trail override. A trail can be open-by-default (the
   *  trust-cal banner, or the 'always' visibility pref), so two sets are
   *  needed: `expandedTrails` force-opens, `collapsedTrails` force-closes.
   *  With only an "expanded" set the chevron was a no-op on default-open
   *  cards — "Hide reasoning" did nothing (Eisa, 2026-05-20). MIG-039 fix. */
  let expandedTrails = $state(new Set<string>());
  let collapsedTrails = $state(new Set<string>());
  function toggleTrail(notePath: string, currentlyOpen: boolean) {
    const exp = new Set(expandedTrails);
    const col = new Set(collapsedTrails);
    if (currentlyOpen) { exp.delete(notePath); col.add(notePath); }
    else { col.delete(notePath); exp.add(notePath); }
    expandedTrails = exp;
    collapsedTrails = col;
  }

  /**
   * V3-§8.r5.3 (audit UX agent) — trust-calibration default.
   *
   * Rationale: until the user has reviewed ~50 cards, they haven't
   * developed an intuition for when to trust the catalogers. So we
   * expand the reasoning trail by default for the first 50 reviews,
   * then quiet down to on-demand once they've calibrated.
   *
   * Persisted in localStorage (per-Constellation install, not per-Library
   * — the algorithm behavior is the same across Libraries). Increments
   * on every Accept / Reject / Edit-commit / Disambiguation pick that
   * came from a card carrying a composite trail (legacy v2 cards don't
   * count — they have no trail to learn from).
   */
  const TRUST_CAL_THRESHOLD = 50;
  const TRUST_CAL_KEY = 'cece-trust-cal-reviewed-count';
  let trustCalReviewedCount = $state(0);
  function loadTrustCalCount() {
    try {
      const raw = localStorage.getItem(TRUST_CAL_KEY);
      trustCalReviewedCount = raw ? Math.max(0, parseInt(raw, 10) || 0) : 0;
    } catch { trustCalReviewedCount = 0; }
  }
  function bumpTrustCalCount() {
    trustCalReviewedCount = trustCalReviewedCount + 1;
    try { localStorage.setItem(TRUST_CAL_KEY, String(trustCalReviewedCount)); } catch {}
  }
  let trustCalActive = $derived(trustCalReviewedCount < TRUST_CAL_THRESHOLD);

  /**
   * V3-§10.A — Trail visibility now respects the user's Settings choice.
   * Three-way toggle in Settings → Intelligence → CECE:
   *   - 'always': open on every card regardless of regime
   *   - 'on_disagreement' (default): pre-V3-§10 behavior (trust-cal +
   *     Split/StrongMajority cards open by default)
   *   - 'never': always collapsed unless user clicked the toggle
   *
   * Defaults to 'on_disagreement' when the cece sub-object is absent
   * from saved settings (e.g. existing user upgrading from pre-V3-§10
   * with no cece preferences).
   */
  function isTrailOpen(notePath: string, hasComposite: boolean): boolean {
    // Explicit user clicks win over any default-open rule below, so the
    // chevron can always collapse/expand a trail (MIG-039 fix).
    if (collapsedTrails.has(notePath)) return false;
    if (expandedTrails.has(notePath)) return true;
    const visibilityPref = $appSettings.cece?.reasoningTrailVisibility ?? 'on_disagreement';
    if (visibilityPref === 'always' && hasComposite) return true;
    if (visibilityPref === 'never') return false;
    // 'on_disagreement' — preserve pre-V3-§10 behavior (trust-cal banner
    // auto-opens, plus the per-card pill logic in the render layer
    // handles Split/StrongMajority).
    return trustCalActive && hasComposite;
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
    // MIG-022 §E.1.1 (Boss-Test Gate 1 Stage 2 catch, 2026-05-11):
    // Previously this function was a hardcoded EN/AR switch — every
    // non-en/non-ar locale fell through to the English branch even
    // though `cece.cataloger.*` keys are populated in all 15 locale
    // files (verified per locale via Python batch). Now mirrors the
    // ruleLabel() pattern: $t() lookup with English fallback on a
    // missing key, so every locale renders its own translations.
    const i18nKey = `cece.cataloger.${c}`;
    const translated = $t(i18nKey);
    if (translated && translated !== i18nKey) return translated;
    // English fallback for an unknown cataloger name (defensive — every
    // known cataloger has a matching key in en.json).
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

  /**
   * V3-§8.r5.1 (audit UX agent): Per-cataloger dot color. 6 distinct hues
   * so the user learns "amber dot = wordstems lens" without having to read
   * abbreviations. Status (voiced/silent/dissent) is encoded via fill +
   * border + an icon glyph, so color alone is never the channel.
   * Colors picked to remain legible on the parchment (#fbf8ec) background.
   */
  function catalogerDotColor(c: string): string {
    switch (c) {
      case 'user_authority': return '#2a4a8c';  // blue — your own ground truth
      case 'structural': return '#a83260';      // rose — citations/structure
      case 'linguistic': return '#c9a227';      // amber — wordstems/lexicon
      case 'graph': return '#0f6e56';           // teal — linked notes
      case 'semantic': return '#534ab7';        // violet — similar notes
      case 'reasoning': return '#3a7a3a';       // green — AI judgment
      default: return '#6b6a64';
    }
  }

  /**
   * MIG-022 §E.1 (PJ-042, 2026-05-11) — translate the self-reported
   * Confidence enum (`high` / `medium` / `low` / `abstain`) into the
   * active locale. Falls back to the raw enum string on a missing
   * key so an unexpected enum value never breaks rendering.
   *
   * The enum serializes from Rust's `cataloger.rs::Confidence` with
   * `#[serde(rename_all = "lowercase")]`, so the raw values are always
   * one of the four keys under `cece.confidence.*`.
   */
  function confidenceLabel(c: string): string {
    const i18nKey = `cece.confidence.${c}`;
    const translated = $t(i18nKey);
    if (translated && translated !== i18nKey) return translated;
    return c; // fallback to raw enum string on missing key
  }

  /**
   * MIG-022 §E.2 (PJ-041, 2026-05-11) — render the per-cataloger
   * reasoning prose via the i18n template when present, falling back
   * to the raw English `reasoning` string when the template is absent
   * (Reasoning cataloger; legacy composite_json blobs from before
   * MIG-022) or the i18n catalog is missing the template key in the
   * active locale.
   *
   * The structured template carries {key, params}; key is e.g.
   * "structural.both" and resolves to `cece.reasoning.structural.both`
   * with the params satisfying placeholders like {h_id}, {v_weight}.
   */
  function reasoningLabel(trail: PerCatalogerTrail): string {
    if (!trail.reasoning_template) return trail.reasoning;
    const i18nKey = `cece.reasoning.${trail.reasoning_template.key}`;
    // $t(key, params) — Constellation's i18n passes params as the
    // second argument's top-level fields (verified against
    // ImporterModal/LibraryManager/CCSView call sites).
    // MIG-022 §E.3.f (Boss-Test Gate 2 Stage 2 catch, 2026-05-11):
    // resolve taxonomy ID params to localized labels via cece.taxonomy
    // BEFORE substituting into the reasoning template, so non-en
    // locales render "horizontal → خَبَر الثِّقة (...)" instead of
    // "horizontal → testimony/authoritative (...)".
    const params = resolveTaxonomyParams(trail.reasoning_template.params);
    const translated = $t(i18nKey, params);
    if (translated && translated !== i18nKey) return translated;
    return trail.reasoning; // fallback to English raw on missing key
  }

  /**
   * MIG-022 §E.2 — render the composite_reasoning summary via i18n
   * template when present. Same shape as reasoningLabel above; falls
   * back to raw English on missing template/key.
   */
  function compositeReasoningLabel(c: CompositeAssignment): string {
    if (!c.composite_reasoning_template) return c.composite_reasoning;
    const i18nKey = `cece.reasoning.${c.composite_reasoning_template.key}`;
    const params = resolveTaxonomyParams(c.composite_reasoning_template.params);
    const translated = $t(i18nKey, params);
    if (translated && translated !== i18nKey) return translated;
    return c.composite_reasoning;
  }

  /**
   * MIG-022 §E.3.f — resolve taxonomy ID params in a reasoning
   * template params object to localized labels. The cataloger Rust
   * code emits raw taxonomy IDs (e.g. "testimony/authoritative") as
   * `h_id`/`v_id`/`sources`/`content_type` placeholder values; this
   * helper looks each up via `cece.taxonomy.<id>` and substitutes
   * the localized label. Other params (roots, neighbors) pass
   * through unchanged.
   *
   * The set of "taxonomy-flavored" param keys is hardcoded based on
   * the §E.2 cataloger build_reasoning emissions:
   *   single ID  : h_id, v_id
   *   joined IDs : sources, content_type (UA's comma-joined lists)
   *   other      : roots (CAE roots), neighbors (note paths) — pass through
   */
  function resolveTaxonomyParams(raw: Record<string, unknown>): Record<string, string> {
    const out: Record<string, string> = {};
    for (const [k, v] of Object.entries(raw)) {
      const sv = String(v);
      if (k === 'h_id' || k === 'v_id') {
        out[k] = resolveTaxonomyId(sv);
      } else if (k === 'sources' || k === 'content_type') {
        out[k] = sv
          .split(',')
          .map((s) => resolveTaxonomyId(s.trim()))
          .join(', ');
      } else {
        out[k] = sv;
      }
    }
    return out;
  }

  /**
   * Single-ID lookup helper. Returns the localized label when the
   * `cece.taxonomy.<id>` key resolves; falls back to the raw ID
   * (which is what the Source Review card showed pre-§E.3.f anyway).
   */
  function resolveTaxonomyId(id: string): string {
    if (!id) return id;
    const taxKey = `cece.taxonomy.${id}`;
    const label = $t(taxKey);
    return label && label !== taxKey ? label : id;
  }

  /**
   * V3-§8.r1.f — Sibling Disambiguation pick handler. Calls the new
   * cece_resolve_disambiguation IPC, then drops the resolved card
   * from the queue.
   */
  async function resolveDisambiguation(notePath: string, axis: 'horizontal' | 'vertical', chosenId: string) {
    try {
      // MIG-040 (both-axes-split bugfix): the command returns the updated
      // record when the OTHER axis is still Split — keep the card (it now
      // shows only that axis's chips) — or null when both axes are resolved.
      const updated = await invoke<SuggestionRecord | null>('cece_resolve_disambiguation', {
        notePath,
        axis,
        chosenId,
      });
      if (updated) {
        queue = queue.map(r => (r.note_path === notePath ? updated : r));
      } else {
        queue = queue.filter(r => r.note_path !== notePath);
      }
      bumpTrustCalCount(); // V3-§8.r5.3 — counts as a calibration review
    } catch (e) {
      error = String(e);
    }
  }

  /**
   * V3-§8.r5.2 (audit UX agent) — translate machine-readable rule keys
   * from `rules_fired` into plain-language phrases the user can act on.
   * Falls back to the raw key (de-snake-cased) when no translation
   * exists, so an unmapped rule never breaks rendering — it just looks
   * a touch technical.
   *
   * Both EN and AR phrasings live in i18n keys (`cece.rule.<key>`); the
   * function returns the i18n value if present, else the locale fallback,
   * else a humanized version of the snake_case key.
   */
  function ruleLabel(rule: string): string {
    const i18nKey = `cece.rule.${rule}`;
    const translated = $t(i18nKey);
    if (translated && translated !== i18nKey) return translated;
    // Locale-default fallbacks (English-only — AR speakers see the EN
    // phrase if their i18n cece.rule.* key is missing). Keys below are
    // the EXACT strings the six catalogers emit today (verified against
    // user_authority.rs / structural.rs / linguistic.rs / graph.rs /
    // semantic.rs / reasoning.rs as of 2026-05-10).
    switch (rule) {
      // User Authority (user_authority.rs:103)
      case 'rule_of_authority': return 'Your frontmatter is the authority';
      // Structural (structural.rs:90, 93)
      case 'structural_pattern_match': return 'Structural pattern matched';
      case 'stance_or_form_marker': return 'Stance/form marker present';
      // Linguistic (linguistic.rs:362, 365, 368, 370)
      case 'cae_root_match': return 'Arabic root match (CAE)';
      case 'surface_token_match': return 'Surface keyword match';
      case 'bridge_similarity': return 'Cross-lingual similarity (Bridge)';
      case 'rule_of_side_channel_preference': return 'Side-channel preference rule';
      // Graph (graph.rs:167, 168)
      case 'typed_neighbor_consensus': return 'Typed-link neighbor consensus';
      // Semantic (semantic.rs:186)
      case 'semantic_neighbor_consensus': return 'Similar classified neighbors agree';
      // Shared by Graph + Semantic (graph.rs:168, semantic.rs:187)
      case 'rule_of_authority_control': return 'Authority-control rule';
      // Reasoning (reasoning.rs:174-176)
      case 'schedule_navigation_top_down': return 'Top-down taxonomy navigation';
      case 'gbnf_constrained': return 'AI judgment (grammar-constrained)';
      case 'rule_of_application': return 'Use vs. mention disambiguation';
      default: {
        // De-snake-case fallback.
        return rule.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
      }
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
    visible = true,
  }: {
    onNoteClick?: (path: string, name: string) => void;
    activeNotePath?: string | null;
    /**
     * MIG-039: CatalogerView passes `visible={showCataloger}` so the queue
     * reloads automatically when the full-page Cataloger is reopened — picking
     * up notes that were classified via the right-sidebar "Classify open note"
     * button while the Cataloger was hidden.  Right-sidebar instances omit
     * this prop (defaults to `true`; the `_srp_was_closed` guard prevents
     * spurious reloads on their first render).
     */
    visible?: boolean;
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

  // MIG-080 §D — monotonic load token. The new note-switch $effect can fire
  // loadQueue() faster than the IPC round-trips (rapid A→B→C tab switching);
  // without sequencing, a stale in-flight response could overwrite a newer one
  // and leave the rail showing the wrong note. Each call captures `seq`; only
  // the latest call is allowed to write `queue`/`loading`/`error`.
  let _srpLoadSeq = 0;
  async function loadQueue() {
    const seq = ++_srpLoadSeq;
    loading = true;
    error = null;
    try {
      if (activeNotePath) {
        // Right-rail (note-context): THIS note's pending suggestion record only
        // (sources_get_suggestions → Option<SuggestionRecord>). The universe queue
        // stays in the Cataloger, which mounts this panel WITHOUT activeNotePath
        // → the else-branch below.
        const rec = await invoke<SuggestionRecord | null>('sources_get_suggestions', { notePath: activeNotePath });
        if (seq !== _srpLoadSeq) return; // superseded by a newer load
        queue = rec ? [rec] : [];
      } else {
        const all = await invoke<SuggestionRecord[]>('sources_list_pending_suggestions');
        if (seq !== _srpLoadSeq) return;
        queue = all;
      }
    } catch (e) {
      if (seq !== _srpLoadSeq) return;
      error = String(e);
      queue = [];
    } finally {
      if (seq === _srpLoadSeq) loading = false;
    }
  }

  // MIG-080 §D — right-rail instances reload when the open note changes. The
  // first run is skipped (onMount already loaded); the Cataloger keeps
  // activeNotePath null so this never reloads it.
  let _srpLastNote: string | null | undefined = undefined;
  $effect(() => {
    const p = activeNotePath;
    if (_srpLastNote === undefined) { _srpLastNote = p; return; }
    if (p !== _srpLastNote) { _srpLastNote = p; loadQueue(); }
  });

  async function classifyActiveNote() {
    if (!activeNotePath || classifying) return;
    classifying = true;
    error = null;
    try {
      const record = await invoke<SuggestionRecord>('classifier_suggest_for_note', {
        notePath: activeNotePath,
      });
      queue = [record, ...queue.filter(r => r.note_path !== record.note_path)];
      // MIG-039 cross-instance sync: notify other SRP instances (e.g. the Cataloger)
      // so they also show the freshly-classified note without a page reload.
      // The local instance self-guards via the `if (classifying) return` check
      // inside handleClassifyAndShow (classifying is still true here — the finally
      // block hasn't run yet — so the local listener exits immediately).
      window.dispatchEvent(
        new CustomEvent('constellation:classify-and-show', { detail: { notePath: record.note_path } }),
      );
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
    // V3-§9.C.2 — snapshot composite_json BEFORE the writes so the
    // reliability update can use it. The two writes below clear the
    // suggestion row, so a post-write read would return null.
    const compositeJsonForReliability = record.composite_json;
    try {
      await invoke('sources_set_manual', {
        notePath: record.note_path,
        sources: horizontalIds,
      });
      await invoke('content_type_set_manual', {
        notePath: record.note_path,
        contentType: verticalIds,
      });
      // V3-§9.C.2 — Now that both axes are written, update per-cataloger
      // reliability for BOTH axes from the snapshot. Without this call
      // the per-axis IPCs (which moved their reliability updates out)
      // would leave reliability un-updated for the Accept flow.
      // Best-effort: if it fails, the writes already succeeded; we
      // just don't get a reliability bump.
      if (compositeJsonForReliability) {
        try {
          await invoke('cece_record_correction_for_card', {
            notePath: record.note_path,
            compositeJson: compositeJsonForReliability,
            horizontalPick: horizontalIds,
            verticalPick: verticalIds,
          });
        } catch (relErr) {
          console.warn('[CECE] reliability update failed (non-fatal):', relErr);
        }
      }
      queue = queue.filter(r => r.note_path !== record.note_path);
      cancelEdit();
      // V3-§8.r5.3 — only composite-trail cards count toward calibration.
      if (record.composite_json) bumpTrustCalCount();
    } catch (e) {
      error = String(e);
    }
  }

  async function rejectSuggestion(record: SuggestionRecord) {
    try {
      await invoke('sources_reject_suggestion', { notePath: record.note_path });
      queue = queue.filter(r => r.note_path !== record.note_path);
      // V3-§8.r5.3 — count rejection as a calibration review too
      if (record.composite_json) bumpTrustCalCount();
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
    // MIG-022 §E.3.b (PJ-043, 2026-05-11): prefer i18n catalog over
    // hardcoded en/ar struct fields. Lookup `cece.taxonomy.<id>`;
    // when the key resolves (i.e. translated value != raw key), use
    // it. Otherwise fall back to the Rust struct's en/ar fields
    // (still authoritative for missing-locale defense + as the
    // canonical source the §E.3.a seed extracts from).
    const i18nKey = `cece.taxonomy.${id}`;
    const translated = $t(i18nKey);
    if (translated && translated !== i18nKey) return translated;
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
    // MIG-080 §D — a NOTE-SCOPED instance (right rail, activeNotePath set) honors
    // ONLY events for its own note. Foreign classify-and-show events (e.g. a
    // file-tree right-click on a different note) belong to the universe Cataloger
    // instance (activeNotePath null), which still prepends them. Without this, an
    // X-scoped rail would be contaminated by note Y's freshly-classified card.
    if (activeNotePath && notePath !== activeNotePath) return;
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
    // V3-§8.r5.5: Approve All is Split-aware — backend skips Split-regime
    // cards by default. Counter reflects the *eligible* set, not raw queue
    // length, so progress doesn't appear to stall for skipped cards.
    bulkTotal = queue.length - splitAwareSkipCount;
    try {
      await invoke('sources_accept_all_pending', { skipSplit: true });
    } catch (e) {
      error = String(e);
      bulkRunning = false;
    }
  }

  /**
   * V3-§8.r5.5 — Count of cards that the Split-aware Approve All would skip.
   * Used to show "Approve all (N eligible, M will stay for your call)" in
   * the confirm dialog so the user understands what's about to happen.
   *
   * V3-§8.r7 — uses the robust cardNeedsUserCall helper so this stays
   * consistent with the per-card pill and the queue-level chip.
   */
  let splitAwareSkipCount = $derived.by(() => {
    return queue.filter(cardNeedsUserCall).length;
  });

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

  // Safety Audit G9 (W3-3): unmount-during-async-load guard. onMount awaits the
  // taxonomy load + a dynamic import + the listen() calls; if the panel is
  // destroyed during those awaits, the listeners would register AFTER onDestroy
  // ran (with the unlisten refs still null) and leak forever (Rule 4). This flag
  // makes a post-await registration immediately unlisten instead.
  let destroyed = false;
  onMount(async () => {
    loadTrustCalCount(); // V3-§8.r5.3
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
      // MIG-040: pause NSC summary computation while the scanner is using
      // the embedding engine; resume when it finishes.
      summaryScanRunning = phase === 'start' || phase === 'progress';
      if (phase === 'progress' || phase === 'done' || phase === 'cancelled') {
        scheduleQueueReload();
      }
    });
    if (destroyed) unlisten(); else scanUnlisten = unlisten;

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
    if (destroyed) unlistenBulk(); else bulkUnlisten = unlistenBulk;
  });

  onDestroy(() => {
    destroyed = true;
    window.removeEventListener('constellation:classify-and-show', handleClassifyAndShow);
    if (highlightTimeout) clearTimeout(highlightTimeout);
    if (scanReloadTimer) clearTimeout(scanReloadTimer);
    if (summaryFillTimer) clearTimeout(summaryFillTimer); // MIG-040
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
    {@const splitCount = queue.filter(cardNeedsUserCall).length}
    <!-- V3-§8.r8 — queue composition filter chip row.
         Slices the rendered list by what kind of decision each card
         needs. Always operates on the full queue (so Approve All math
         and the count strip stay accurate). Legacy v2-era cards only
         appear in 'all'. -->
    {#if queue.length > 1 && !activeNotePath}
      <div class="srp-filter-row" role="tablist" aria-label={$t('cece.queueFilter.ariaLabel') || 'Filter the review queue by composition'}>
        {#each filterChips as f}
          <button
            class="srp-filter-chip"
            class:srp-filter-chip-active={queueFilter === f.key}
            class:srp-filter-chip-empty={f.count === 0 && f.key !== 'all'}
            disabled={f.count === 0 && f.key !== 'all'}
            role="tab"
            aria-selected={queueFilter === f.key}
            onclick={() => { queueFilter = f.key; renderCap = RENDER_BATCH; }}
            title={f.label}
          >
            {f.label} <span class="srp-filter-count">({f.count})</span>
          </button>
        {/each}
      </div>
    {/if}
    {#if trustCalActive && !activeNotePath}
      <!-- V3-§8.r5.3 — trust-calibration banner (universe-only; hidden in the
           note-scoped right rail per MIG-080 §D). Quiet but persistent
           reminder that reasoning trails are auto-expanded so the user
           learns when to trust the cataloger ensemble. Disappears once
           the user has reviewed TRUST_CAL_THRESHOLD cards. -->
      <div class="srp-trust-cal-banner" title={$t('cece.trustCal.tooltip') || 'Reasoning trails are shown by default while you build a sense of how the catalogers reach their classifications. This banner disappears after 50 reviews.'}>
        <span class="srp-trust-cal-icon" aria-hidden="true">▸</span>
        {($t('cece.trustCal.banner') || 'Showing reasoning trails until you review {N} more cards — helps you learn when to trust the catalogers.').replace('{N}', String(TRUST_CAL_THRESHOLD - trustCalReviewedCount))}
      </div>
    {/if}
    <div class="srp-count-row">
      <span class="srp-count">
        {queue.length}
        {queue.length === 1
          ? ($t('sources.review.pending') || 'pending')
          : ($t('sources.review.pendingPlural') || 'pending')}
        {#if splitCount > 0}
          <!-- V3-§8.r5.4 (audit UX agent): queue-level Split count chip.
               Differentiates "needs your call" cards from the rest at a
               glance instead of relying on per-card gold borders that
               become wallpaper. -->
          <span class="srp-queue-split-chip" title={$t('cece.queueSplit.tooltip') || 'Cards where catalogers split — your decision is needed'}>
            • {($t('cece.queueSplit.label') || '{N} need your call').replace('{N}', String(splitCount))}
          </span>
        {/if}
      </span>
      <!-- MIG-021v2 §1F'.b — bulk Approve All / Reject All.
           MIG-080 §D — UNIVERSE-ONLY. The note-scoped right rail (activeNotePath set)
           must NOT expose these: sources_accept_all_pending / sources_reject_all_pending
           act on the ENTIRE universe queue (reject = DELETE FROM sources_suggestions with
           NO WHERE clause), so showing them beside a single note's card would let one
           note's panel wipe every pending suggestion across the Universe. The per-note
           rail uses the per-card Accept / Reject only. -->
      {#if !activeNotePath}
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
      {/if}
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
          {#if bulkConfirm === 'accept'}
            <!-- V3-§8.r5.5: Split-aware confirmation. Eligible count = queue
                 minus Split-regime cards (which stay for the user's call). -->
            {@const eligible = queue.length - splitAwareSkipCount}
            {($t('sources.review.confirmAcceptAllSplitAware') || 'Apply suggestions to {N} notes whose catalogers reached agreement.').replace('{N}', eligible.toLocaleString())}
            {#if splitAwareSkipCount > 0}
              <div class="srp-bulk-confirm-aside">
                {($t('sources.review.confirmAcceptAllSkipNote') || '{M} cards where catalogers split will stay in the queue for you to decide.').replace('{M}', splitAwareSkipCount.toLocaleString())}
              </div>
            {/if}
          {:else}
            {$t('sources.review.confirmRejectAll') || 'Clear every suggestion in the queue without writing? You can re-run the scan later to regenerate them.'}
          {/if}
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

    {#if filteredQueue.length === 0 && queueFilter !== 'all'}
      <!-- V3-§8.r8 — bucket is empty; tell the user instead of rendering
           a silent void. They can switch back to 'All' from here. -->
      <div class="srp-empty-bucket">
        <div class="srp-empty-bucket-text">
          {$t('cece.queueFilter.emptyBucket') || 'No cards match this filter.'}
        </div>
        <button class="srp-btn" onclick={() => queueFilter = 'all'}>
          {$t('cece.queueFilter.showAll') || 'Show all cards'}
        </button>
      </div>
    {/if}
    <ul class="srp-list">
      {#each visibleQueue as record (record.note_path)}
        {@const horizontalSuggestions = record.suggestions.filter(s => s.axis === 'horizontal')}
        {@const verticalSuggestions = record.suggestions.filter(s => s.axis === 'vertical')}
        {@const composite = parseComposite(record)}
        {@const isSplit = cardNeedsUserCall(record)}
        {@const isStrongMajority = composite && !isSplit && (composite.horizontal.regime === 'strong_majority' || composite.vertical.regime === 'strong_majority')}
        {@const showTrail = isTrailOpen(record.note_path, !!composite)}
        {@const summaryText = summaries.get(record.note_path)?.summary ?? ''}
        <li class="srp-card"
            class:srp-just-added={highlightedPath === record.note_path}
            class:srp-split-regime={isSplit}>
          <div class="srp-card-header">
            <button
              class="srp-card-title"
              dir="auto"
              onclick={() => onNoteClick(record.note_path, noteName(record.note_path))}
              title={record.note_path}
            >
              {noteName(record.note_path)}
            </button>
            {#if composite}
              <!-- V3-§8.r5.1 (audit UX agent): per-cataloger dot cluster.
                   6 tinted dots (one per cataloger), color-coded by lens.
                   Filled = voiced + agrees with synthesis; ring = voiced
                   but dissents; empty outline = silent (no signal). The
                   cluster is the at-a-glance "ensemble health" indicator;
                   tooltip on each dot names the lens + its status. -->
              <span class="srp-cataloger-cluster" title={$t('cece.badge.clusterTooltip') || 'Cataloger ensemble — hover each dot for details'}>
                {#each CATALOGER_ORDER as catName}
                  {@const status = catalogerBadgeStatus(composite, catName)}
                  {@const color = catalogerDotColor(catName)}
                  <span
                    class="srp-cat-dot"
                    class:srp-cat-dot-voiced={status === '✓'}
                    class:srp-cat-dot-dissent={status === '✗'}
                    class:srp-cat-dot-silent={status === '–'}
                    style:--dot-color={color}
                    title={`${catalogerLabel(catName)} — ${status === '✓' ? ($t('cece.badge.statusAgrees') || 'agrees with synthesis') : status === '✗' ? ($t('cece.badge.statusDissents') || 'dissents') : ($t('cece.badge.statusSilent') || 'silent (no signal in this lens)')}`}
                    aria-label={`${catalogerLabel(catName)}: ${status === '✓' ? 'agrees' : status === '✗' ? 'dissents' : 'silent'}`}
                  ></span>
                {/each}
              </span>
            {:else}
              <!-- V3-§8.r5.6 (audit UX agent): Legacy v2-era row pill.
                   Plain "Legacy" instead of T1/T2 tier abbreviations the
                   user was never told about. -->
              <span class="srp-legacy-pill" title={$t('cece.badge.legacyTooltip') || 'Classified before the cataloger ensemble was added — no per-cataloger trail available'}>
                {$t('cece.badge.legacy') || 'Legacy'}
              </span>
            {/if}
          </div>

          {#if summaryText}
            <!-- MIG-040 (NSC): the note's summary (author's frontmatter summary
                 if present, else an extractive TextRank summary). dir="auto"
                 for any-language / RTL text. -->
            <div class="srp-summary" dir="auto">
              <span class="srp-summary-label">{$t('nsc.summary') || 'Summary'}</span>
              {summaryText}
            </div>
          {/if}

          {#if composite && (isSplit || isStrongMajority || trustCalActive)}
            <!-- Reasoning trail surface.
                 Default: on disagreement only.
                 V3-§8.r5.3: also default-open during the first
                 TRUST_CAL_THRESHOLD reviews (trust-calibration period). -->
            <div class="srp-trail-toggle">
              <button class="srp-trail-btn" onclick={() => toggleTrail(record.note_path, showTrail)}>
                {#if showTrail}{$t('cece.trail.collapse') || '▾ Hide reasoning'}{:else}{$t('cece.trail.expand') || '▸ Why this classification?'}{/if}
              </button>
              {#if isSplit}
                <span class="srp-split-pill">{$t('cece.regime.split') || 'Catalogers split — needs your call'}</span>
              {:else if isStrongMajority}
                <span class="srp-majority-pill">
                  {$t('cece.regime.strongMajority') || 'Strong majority'} {#if composite.horizontal.dissenter || composite.vertical.dissenter}({catalogerLabel(composite.horizontal.dissenter ?? composite.vertical.dissenter ?? '')}){/if}
                </span>
              {:else if trustCalActive}
                <span class="srp-unanimous-pill" title={$t('cece.regime.unanimousTooltip') || 'All voicing catalogers agreed on this classification'}>
                  {$t('cece.regime.unanimous') || 'Unanimous'}
                </span>
              {/if}
            </div>
            {#if showTrail}
              <div class="srp-trail">
                <div class="srp-trail-summary">{compositeReasoningLabel(composite)}</div>
                <ul class="srp-trail-list">
                  {#each composite.per_cataloger_trails.filter(t => t.voiced_opinion) as t}
                    <li class="srp-trail-item">
                      <div class="srp-trail-row">
                        <span
                          class="srp-trail-dot"
                          style:background-color={catalogerDotColor(t.cataloger)}
                          aria-hidden="true"
                        ></span>
                        <strong>{catalogerLabel(t.cataloger)}</strong>
                        <span class="srp-trail-conf">[{confidenceLabel(t.self_reported_confidence)}]</span>
                      </div>
                      <div class="srp-trail-reasoning">{reasoningLabel(t)}</div>
                      <!-- V3-§8.r5.2 — translated rules_fired strip.
                           Each rule the cataloger triggered becomes a
                           one-word friendly chip ("DOI present",
                           "Blockquote with attribution"). Chips help the
                           user audit the lens at a glance instead of
                           parsing the prose reasoning sentence. -->
                      {#if t.rules_fired && t.rules_fired.length > 0}
                        <div class="srp-trail-rules">
                          {#each t.rules_fired as rule}
                            <span class="srp-trail-rule" title={rule}>{ruleLabel(rule)}</span>
                          {/each}
                        </div>
                      {/if}
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
    {#if filteredQueue.length > visibleQueue.length}
      <div class="srp-show-more">
        <button class="srp-btn" onclick={() => renderCap += RENDER_BATCH}>
          {$t('cece.queueShowMore') || 'Show more'}
        </button>
        <span class="srp-show-more-note">
          {($t('cece.queueShowingCount') || 'Showing {shown} of {total}')
            .replace('{shown}', visibleQueue.length.toLocaleString())
            .replace('{total}', filteredQueue.length.toLocaleString())}
        </span>
      </div>
    {/if}
  {/if}
</div>

<style>
  .srp-root {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    font-size: calc(13px * var(--rs-scale, 1));
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
    font-size: calc(11px * var(--rs-scale, 1));
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
    font-size: calc(11px * var(--rs-scale, 1));
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
    font-size: calc(11px * var(--rs-scale, 1));
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
    font-size: calc(12px * var(--rs-scale, 1));
    line-height: 1.5;
  }
  .srp-error {
    color: var(--text-error, #a83232);
  }
  .srp-error-detail {
    font-size: calc(11px * var(--rs-scale, 1));
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
  /* MIG-039 — "Show more" footer for the render-capped queue. */
  .srp-show-more {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 8px 8px 16px;
    flex-shrink: 0;
  }
  .srp-show-more-note {
    font-size: calc(0.72rem * var(--rs-scale, 1));
    color: var(--text-muted);
  }
  /* MIG-040 (NSC): the per-card note summary, under the title. */
  .srp-summary {
    margin: 2px 0 8px;
    padding: 6px 10px;
    border-inline-start: 2px solid var(--interactive-accent, #7c3aed);
    background: var(--background-modifier-hover, rgba(0,0,0,0.03));
    border-radius: 0 4px 4px 0;
    font-size: calc(0.82rem * var(--rs-scale, 1));
    line-height: 1.45;
    color: var(--text-muted, #555);
  }
  .srp-summary-label {
    display: inline-block;
    margin-inline-end: 6px;
    font-size: calc(0.66rem * var(--rs-scale, 1));
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--interactive-accent, #7c3aed);
    opacity: 0.85;
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
    font-size: calc(13px * var(--rs-scale, 1));
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
  /* V3-§8.r5.6 — .srp-tier removed (replaced by .srp-legacy-pill below).
     Kept the selector slot for grep/migration history. */
  .srp-axis-section {
    margin-bottom: 8px;
  }
  .srp-axis-label {
    font-size: calc(10px * var(--rs-scale, 1));
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
    font-size: calc(12px * var(--rs-scale, 1));
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
    font-size: calc(9px * var(--rs-scale, 1));
    font-weight: 600;
    padding: 1px 5px;
    border-radius: 3px;
    flex-shrink: 0;
  }
  .srp-confidence {
    font-size: calc(11px * var(--rs-scale, 1));
    color: var(--text-muted, #6b6a64);
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }
  .srp-evidence {
    margin-top: 3px;
    font-size: calc(11px * var(--rs-scale, 1));
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
    font-size: calc(12px * var(--rs-scale, 1));
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
  /* MIG-021v3 V3-§8.r5.1 — per-cataloger dot cluster.
     Six tinted dots, color-keyed by lens (blue/rose/amber/teal/violet/green).
     Status encoded by fill + ring + glyph (so color alone is never the
     channel — works for users with color-vision difference). */
  .srp-cataloger-cluster {
    display: inline-flex; flex-wrap: nowrap; gap: 3px;
    align-items: center;
    flex-shrink: 0;
  }
  .srp-cat-dot {
    --dot-color: #6b6a64;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    display: inline-block;
    flex-shrink: 0;
    user-select: none;
    line-height: 1;
    position: relative;
    cursor: help;
  }
  .srp-cat-dot-voiced {
    background: var(--dot-color);
    border: 1px solid var(--dot-color);
  }
  .srp-cat-dot-dissent {
    background: var(--background-primary, #fff);
    border: 2px solid var(--dot-color);
    box-shadow: 0 0 0 1px rgba(168, 50, 50, 0.45);
  }
  .srp-cat-dot-silent {
    background: transparent;
    border: 1px dashed rgba(107, 106, 100, 0.45);
  }
  /* V3-§8.r5.6 — Legacy pill (replaces the bare T1/T2 abbreviation
     for v2-era rows that don't carry a per-cataloger trail). */
  .srp-legacy-pill {
    font-size: calc(9px * var(--rs-scale, 1));
    padding: 1px 6px;
    border-radius: 8px;
    background: rgba(0, 0, 0, 0.05);
    color: var(--text-faint, #6b6a64);
    border: 1px dashed rgba(0, 0, 0, 0.18);
    font-style: italic;
    flex-shrink: 0;
    cursor: help;
  }
  .srp-card.srp-split-regime {
    border-inline-start: 3px solid #c9a227;
    padding-inline-start: 8px;
  }
  .srp-trail-toggle {
    display: flex; align-items: center; gap: 8px;
    padding: 4px 8px;
    flex-wrap: wrap;
  }
  .srp-trail-btn {
    background: transparent; border: none;
    color: var(--text-muted, #6b6a64);
    font-size: calc(11px * var(--rs-scale, 1)); cursor: pointer;
    padding: 2px 0;
  }
  .srp-trail-btn:hover { color: var(--text-normal); }
  .srp-split-pill {
    font-size: calc(10px * var(--rs-scale, 1));
    padding: 2px 6px;
    background: rgba(201, 162, 39, 0.18);
    color: #856204;
    border-radius: 3px;
    border: 1px solid rgba(201, 162, 39, 0.4);
  }
  .srp-majority-pill {
    font-size: calc(10px * var(--rs-scale, 1));
    padding: 2px 6px;
    background: rgba(83, 74, 183, 0.12);
    color: #534ab7;
    border-radius: 3px;
    border: 1px solid rgba(83, 74, 183, 0.3);
  }
  /* V3-§8.r5.3 — Unanimous pill (only shown during trust-calibration so
     the user can tell why the trail is auto-open even on agreed cards). */
  .srp-unanimous-pill {
    font-size: calc(10px * var(--rs-scale, 1));
    padding: 2px 6px;
    background: rgba(15, 110, 86, 0.12);
    color: #0f6e56;
    border-radius: 3px;
    border: 1px solid rgba(15, 110, 86, 0.3);
    cursor: help;
  }
  .srp-trust-cal-banner {
    margin: 4px 8px 0;
    padding: 6px 10px;
    border-radius: 4px;
    background: rgba(83, 74, 183, 0.06);
    border: 1px dashed rgba(83, 74, 183, 0.25);
    color: var(--text-muted, #6b6a64);
    font-size: calc(11px * var(--rs-scale, 1));
    line-height: 1.5;
    cursor: help;
  }
  .srp-trust-cal-icon {
    color: #534ab7;
    margin-inline-end: 4px;
    font-weight: 600;
  }
  .srp-trail {
    padding: 6px 12px 8px;
    background: rgba(0, 0, 0, 0.02);
    border-block: 1px solid var(--background-modifier-border, rgba(0,0,0,0.06));
    font-size: calc(11px * var(--rs-scale, 1));
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
    font-size: calc(10px * var(--rs-scale, 1));
    color: var(--text-faint);
    margin-inline-start: 4px;
  }
  /* V3-§8.r5.2 — friendly rules-fired chip strip + lens-color dot inside
     the reasoning trail. Reusing the cluster's color palette so the user
     learns which lens-color = which cataloger across surfaces. */
  .srp-trail-row {
    display: flex; align-items: center; gap: 4px;
    margin-bottom: 2px;
  }
  .srp-trail-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .srp-trail-reasoning {
    color: var(--text-normal);
    margin-inline-start: 11px;
  }
  .srp-trail-rules {
    margin-inline-start: 11px;
    margin-top: 3px;
    display: flex; flex-wrap: wrap; gap: 3px;
  }
  .srp-trail-rule {
    font-size: calc(10px * var(--rs-scale, 1));
    padding: 1px 6px;
    background: rgba(0, 0, 0, 0.04);
    color: var(--text-muted);
    border-radius: 8px;
    border: 1px solid rgba(0, 0, 0, 0.06);
    cursor: help;
  }

  /* MIG-021v3 V3-§8.r1.f — Sibling Disambiguation form */
  .srp-disambig {
    padding: 8px 12px;
    background: rgba(201, 162, 39, 0.06);
    border-block-start: 1px solid rgba(201, 162, 39, 0.25);
    display: flex; flex-direction: column; gap: 6px;
  }
  .srp-disambig-prompt {
    font-size: calc(11px * var(--rs-scale, 1));
    color: var(--text-normal);
    line-height: 1.4;
  }
  .srp-disambig-axis {
    display: flex; flex-direction: column; gap: 4px;
  }
  .srp-disambig-axis-label {
    font-size: calc(10px * var(--rs-scale, 1));
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
    font-size: calc(11px * var(--rs-scale, 1));
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
    padding: 3px 10px; font-size: calc(11px * var(--rs-scale, 1));
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
    font-size: calc(11px * var(--rs-scale, 1)); color: var(--text-normal);
  }
  .srp-bulk-progress-text { font-variant-numeric: tabular-nums; }
  .srp-bulk-cancel {
    border: 1px solid var(--background-modifier-border, rgba(0,0,0,0.18));
    background: transparent;
    color: var(--text-muted);
    border-radius: 4px;
    padding: 1px 8px;
    font-size: calc(11px * var(--rs-scale, 1));
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
    font-size: calc(12px * var(--rs-scale, 1)); line-height: 1.5; color: var(--text-normal);
  }
  /* V3-§8.r5.5 — secondary aside under the main confirm prompt explaining
     the Split-aware skip behavior. */
  .srp-bulk-confirm-aside {
    margin-top: 6px;
    font-size: calc(11px * var(--rs-scale, 1));
    color: var(--text-muted, #6b6a64);
    font-style: italic;
  }
  .srp-bulk-confirm-actions {
    display: flex; gap: 8px; justify-content: flex-end;
  }
  /* V3-§8.r8 — queue composition filter chip row. Sits between the
     trust-cal banner and the count strip. Active chip is highlighted;
     empty buckets are dimmed and disabled. Solves the needle-in-
     haystack problem when the queue has hundreds of cards.

     The whole row hides if the queue has 0 or 1 cards (no point
     filtering one card). */
  .srp-filter-row {
    display: flex; flex-wrap: wrap; gap: 4px;
    padding: 4px 8px 0;
  }
  .srp-filter-chip {
    font-size: calc(10px * var(--rs-scale, 1));
    padding: 3px 8px;
    background: transparent;
    color: var(--text-muted, #6b6a64);
    border: 1px solid var(--background-modifier-border, rgba(0,0,0,0.18));
    border-radius: 12px;
    cursor: pointer;
    user-select: none;
    transition: background 0.12s;
    line-height: 1.3;
  }
  .srp-filter-chip:hover:not(:disabled) {
    background: var(--background-modifier-hover, rgba(0,0,0,0.05));
    color: var(--text-normal, #1a1a1a);
  }
  .srp-filter-chip-active {
    background: rgba(201, 162, 39, 0.18);
    color: #856204;
    border-color: rgba(201, 162, 39, 0.4);
    font-weight: 600;
  }
  .srp-filter-chip-empty {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .srp-filter-count {
    font-variant-numeric: tabular-nums;
    margin-inline-start: 2px;
  }
  /* V3-§8.r8 — empty-bucket hint when the active filter has 0 matches. */
  .srp-empty-bucket {
    padding: 16px;
    margin: 8px;
    text-align: center;
    color: var(--text-muted, #6b6a64);
    background: var(--background-secondary, #fbf8ec);
    border: 1px dashed var(--background-modifier-border, rgba(0,0,0,0.18));
    border-radius: 6px;
    display: flex; flex-direction: column; gap: 8px; align-items: center;
  }
  .srp-empty-bucket-text {
    font-size: calc(12px * var(--rs-scale, 1));
  }

  /* V3-§8.r5.4 — queue-level Split count chip surfaces the "needs your
     call" cards in one number rather than asking the user to scan for
     gold borders. */
  .srp-queue-split-chip {
    margin-inline-start: 6px;
    font-size: calc(10px * var(--rs-scale, 1));
    padding: 1px 6px;
    background: rgba(201, 162, 39, 0.18);
    color: #856204;
    border-radius: 8px;
    border: 1px solid rgba(201, 162, 39, 0.4);
    font-weight: 600;
    cursor: help;
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
