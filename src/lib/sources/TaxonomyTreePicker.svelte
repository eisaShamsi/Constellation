<!--
  MIG-021v2 §1C' fix-2 — Hierarchical taxonomy tree picker (flat-render rewrite).

  Reusable tree picker for both horizontal (sources) and vertical (content_type)
  taxonomies. Renders the visible tree as a single flat list of {node, depth}
  rows — avoids the recursive Svelte 5 snippet pattern entirely (those are
  unreliable when self-referenced from within their own definition).

  Visual language mirrors:
    - sources-of-knowledge-diagram.html (horizontal axis, tier coloring)
    - epistemic-content-taxonomy-chart.html (vertical axis, indented tree)
-->
<script lang="ts">
  import { t, locale, type Locale } from '$lib/i18n';
  import { tierColor as horizontalTierColor } from './horizontalTaxonomy';
  import { branchColor as verticalBranchColor } from './verticalTaxonomy';

  type AnyNode = {
    id: string;
    en: string;
    ar: string;
    parent_id: string | null;
    tr?: string | null;
    tier?: number;
    branch?: number;
  };

  let {
    taxonomy,
    axis,
    selected = new Set<string>(),
    onChange = (_s: Set<string>) => {},
    tierColors = false,
  }: {
    taxonomy: AnyNode[];
    axis: 'horizontal' | 'vertical';
    selected?: Set<string>;
    onChange?: (selected: Set<string>) => void;
    tierColors?: boolean;
  } = $props();

  // ─── Index by parent for O(1) child lookup ──────────────────────────
  let childrenByParent = $derived.by(() => {
    const map = new Map<string | null, AnyNode[]>();
    for (const node of taxonomy) {
      const key = node.parent_id;
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(node);
    }
    return map;
  });

  let topLevel = $derived.by(() => {
    if (axis === 'horizontal') {
      return taxonomy.filter((n) => n.parent_id === null);
    }
    return taxonomy.filter((n) => n.parent_id === 'epistemic-content');
  });

  // ─── State ──────────────────────────────────────────────────────────
  let expanded = $state(new Set<string>());
  let query = $state('');

  function toggleExpanded(id: string) {
    const next = new Set(expanded);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    expanded = next;
  }

  function toggleSelected(id: string) {
    const next = new Set(selected);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    onChange(next);
  }

  function expandAll() {
    const all = new Set<string>();
    for (const n of taxonomy) {
      const has = (childrenByParent.get(n.id) ?? []).length > 0;
      if (has) all.add(n.id);
    }
    expanded = all;
  }

  function collapseAll() {
    expanded = new Set();
  }

  // ─── Search match set + ancestor auto-expand ────────────────────────
  let searchMatches = $derived.by(() => {
    if (!query.trim()) return null;
    const q = query.trim().toLowerCase();
    const matches = new Set<string>();
    for (const n of taxonomy) {
      const blob = `${n.en} ${n.ar} ${n.tr ?? ''}`.toLowerCase();
      if (blob.includes(q)) matches.add(n.id);
    }
    return matches;
  });

  $effect(() => {
    if (!searchMatches || searchMatches.size === 0) return;
    const ancestors = new Set<string>();
    for (const matchId of searchMatches) {
      let cur: AnyNode | undefined = taxonomy.find((n) => n.id === matchId);
      while (cur && cur.parent_id) {
        ancestors.add(cur.parent_id);
        cur = taxonomy.find((n) => n.id === cur!.parent_id);
      }
    }
    expanded = new Set([...expanded, ...ancestors]);
  });

  function isVisible(node: AnyNode): boolean {
    if (!searchMatches) return true;
    if (searchMatches.has(node.id)) return true;
    const stack = [node.id];
    while (stack.length) {
      const cur = stack.pop()!;
      const kids = childrenByParent.get(cur) ?? [];
      for (const k of kids) {
        if (searchMatches.has(k.id)) return true;
        stack.push(k.id);
      }
    }
    return false;
  }

  // ─── Flat render: pre-walk the tree once into a list of rows ────────
  type Row = { node: AnyNode; depth: number };

  let visibleRows = $derived.by(() => {
    const rows: Row[] = [];
    const walk = (nodes: AnyNode[], depth: number) => {
      for (const node of nodes) {
        if (!isVisible(node)) continue;
        rows.push({ node, depth });
        if (expanded.has(node.id)) {
          const kids = childrenByParent.get(node.id) ?? [];
          if (kids.length > 0) walk(kids, depth + 1);
        }
      }
    };
    walk(topLevel, 0);
    return rows;
  });

  // ─── Locale-aware label rendering ───────────────────────────────────
  let currentLocale = $state<Locale>('en');
  $effect(() => {
    return locale.subscribe((l) => {
      currentLocale = l;
    });
  });

  function nodeColor(node: AnyNode): string | null {
    if (tierColors && axis === 'horizontal' && node.parent_id === null && node.tier && node.tier > 0) {
      return horizontalTierColor(node.tier);
    }
    if (axis === 'vertical' && node.branch && node.branch > 0 && node.parent_id === 'epistemic-content') {
      return verticalBranchColor(node.branch);
    }
    return null;
  }

  function hasChildren(node: AnyNode): boolean {
    return (childrenByParent.get(node.id) ?? []).length > 0;
  }
</script>

<div class="ttp-root" dir="auto">
  <div class="ttp-header">
    <input
      type="text"
      class="ttp-search"
      placeholder={$t('taxonomyTreePicker.search') || 'Search…'}
      bind:value={query}
    />
    <button
      class="ttp-btn"
      onclick={() => expandAll()}
      title={$t('taxonomyTreePicker.expandAll') || 'Expand all'}
    >
      {$t('taxonomyTreePicker.expandAll') || 'Expand all'}
    </button>
    <button
      class="ttp-btn"
      onclick={() => collapseAll()}
      title={$t('taxonomyTreePicker.collapseAll') || 'Collapse all'}
    >
      {$t('taxonomyTreePicker.collapseAll') || 'Collapse all'}
    </button>
  </div>

  <ul class="ttp-tree" role="tree">
    {#each visibleRows as row (row.node.id)}
      {@const node = row.node}
      {@const depth = row.depth}
      {@const kids = hasChildren(node)}
      {@const isExpanded = expanded.has(node.id)}
      {@const isChecked = selected.has(node.id)}
      {@const color = nodeColor(node)}
      {@const isArabic = currentLocale === 'ar'}
      {@const primaryLabel = isArabic ? node.ar : node.en}
      {@const secondaryLabel = isArabic ? node.en : node.ar}
      <li
        class="ttp-node"
        class:has-children={kids}
        style:--node-color={color ?? 'transparent'}
        role="treeitem"
        aria-expanded={kids ? isExpanded : undefined}
      >
        <div class="ttp-row" style:padding-inline-start={`${depth * 18 + 6}px`}>
          {#if kids}
            <button
              class="ttp-chevron"
              class:expanded={isExpanded}
              onclick={() => toggleExpanded(node.id)}
              aria-label={isExpanded ? 'Collapse' : 'Expand'}
            >▸</button>
          {:else}
            <span class="ttp-chevron-spacer"></span>
          {/if}
          <label class="ttp-label">
            <input
              type="checkbox"
              checked={isChecked}
              onchange={() => toggleSelected(node.id)}
            />
            <span class="ttp-primary">{primaryLabel}</span>
            <span class="ttp-secondary">{secondaryLabel}</span>
            {#if node.tr}
              <span class="ttp-tr">{node.tr}</span>
            {/if}
          </label>
        </div>
      </li>
    {/each}
  </ul>
</div>

<style>
  .ttp-root {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    font-size: 12px;
  }
  .ttp-header {
    display: flex;
    gap: 6px;
    padding: 6px 8px;
    border-bottom: 1px solid var(--background-modifier-border, rgba(0,0,0,0.08));
    flex-shrink: 0;
  }
  .ttp-search {
    flex: 1;
    min-width: 0;
    padding: 3px 8px;
    border: 1px solid var(--background-modifier-border, rgba(0,0,0,0.18));
    border-radius: 4px;
    background: var(--background-primary, #fff);
    color: var(--text-normal, #1a1a1a);
    font: inherit;
  }
  .ttp-search:focus {
    outline: none;
    border-color: #c9a227;
  }
  .ttp-btn {
    background: transparent;
    border: 1px solid var(--background-modifier-border, rgba(0,0,0,0.18));
    color: var(--text-normal, #1a1a1a);
    padding: 3px 8px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
    white-space: nowrap;
  }
  .ttp-btn:hover {
    background: var(--background-modifier-hover, rgba(0,0,0,0.05));
  }
  .ttp-tree {
    list-style: none;
    margin: 0;
    padding: 4px 0;
    overflow-y: auto;
    flex: 1;
  }
  .ttp-node {
    list-style: none;
    margin: 0;
    padding: 0;
    border-inline-start: 3px solid var(--node-color);
  }
  .ttp-row {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    cursor: default;
  }
  .ttp-row:hover {
    background: var(--background-modifier-hover, rgba(0,0,0,0.04));
  }
  .ttp-chevron {
    background: transparent;
    border: none;
    cursor: pointer;
    width: 14px;
    height: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 10px;
    color: var(--text-muted, #6b6a64);
    padding: 0;
    transition: transform 0.12s;
    flex-shrink: 0;
  }
  .ttp-chevron.expanded {
    transform: rotate(90deg);
  }
  .ttp-chevron-spacer {
    width: 14px;
    flex-shrink: 0;
  }
  .ttp-label {
    display: flex;
    align-items: baseline;
    gap: 6px;
    cursor: pointer;
    flex: 1;
    min-width: 0;
  }
  .ttp-label input[type="checkbox"] {
    margin: 0;
    flex-shrink: 0;
    cursor: pointer;
  }
  .ttp-primary {
    color: var(--text-normal, #1a1a1a);
    font-weight: 500;
  }
  .ttp-secondary {
    color: var(--text-muted, #6b6a64);
    font-size: 11px;
  }
  .ttp-tr {
    color: var(--text-muted, #6b6a64);
    font-size: 10px;
    font-style: italic;
  }
</style>
