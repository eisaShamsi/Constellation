<!--
    SightV3SidePanel.svelte — slides in from the right when a star is
    clicked. Shows note metadata + linked notes + structural-gap
    suggestions for that note.

    §1E (this commit): minimal version — note title, community name,
    centrality rank, top incoming + outgoing wikilinks (count-only),
    Open in editor button, Close button. The MIG-019 surface adds
    the universe-health card here too.
-->
<script lang="ts">
    import { t } from '$lib/i18n';
    import type { HealthReport, MetricBadge } from '$lib/sight/universe-health';

    interface Props {
        notePath: string | null;
        noteTitle: string;
        communityName: string;
        centralityRank: number;
        totalNotes: number;
        incomingCount: number;
        outgoingCount: number;
        /** MIG-019 §2D: universe-health metrics. Always shown when no
         *  star is selected; tucks below note details when one is. */
        health: HealthReport;
        onOpenNote: () => void;
        onClose: () => void;
    }
    let {
        notePath,
        noteTitle,
        communityName,
        centralityRank,
        totalNotes,
        incomingCount,
        outgoingCount,
        health,
        onOpenNote,
        onClose,
    }: Props = $props();

    /** Open if a star is selected OR no star is selected (always show
     *  universe-health when no star). The panel is hidden only when
     *  the user explicitly closes it via the button. */
    const isOpen = $derived(notePath !== null);

    function statusClass(status: 'healthy' | 'caution' | 'imbalanced'): string {
        return `sv3-sp-badge sv3-sp-badge-${status}`;
    }

    function statusLabel(status: 'healthy' | 'caution' | 'imbalanced'): string {
        if (status === 'healthy') return $t('sightV3.sidePanel.healthy') || 'healthy';
        if (status === 'caution') return $t('sightV3.sidePanel.caution') || 'caution';
        return $t('sightV3.sidePanel.imbalanced') || 'imbalanced';
    }
</script>

<div class="sight-v3-side-panel" class:open={isOpen || health.totalNotes > 0} dir="auto">
    <!-- Note details section — shown only when a star is selected -->
    {#if notePath}
        <div class="sv3-sp-header">
            <h3 class="sv3-sp-title">{noteTitle}</h3>
            <button class="sv3-sp-close-btn" onclick={onClose} aria-label={$t('sightV3.sidePanel.close') || 'Close panel'}>
                ×
            </button>
        </div>

        <div class="sv3-sp-meta">
            <div class="sv3-sp-row">
                <span class="sv3-sp-label">{$t('sightV3.sidePanel.community') || 'Community'}</span>
                <span class="sv3-sp-value">{communityName}</span>
            </div>
            <div class="sv3-sp-row">
                <span class="sv3-sp-label">{$t('sightV3.sidePanel.centralityRank') || 'Centrality rank'}</span>
                <span class="sv3-sp-value">#{centralityRank} of {totalNotes}</span>
            </div>
        </div>

        <div class="sv3-sp-section">
            <div class="sv3-sp-section-title">{$t('sightV3.sidePanel.connections') || 'Connections'}</div>
            <div class="sv3-sp-row">
                <span class="sv3-sp-label">{$t('sightV3.sidePanel.incomingLinks') || 'Incoming links'}</span>
                <span class="sv3-sp-value">{incomingCount}</span>
            </div>
            <div class="sv3-sp-row">
                <span class="sv3-sp-label">{$t('sightV3.sidePanel.outgoingLinks') || 'Outgoing links'}</span>
                <span class="sv3-sp-value">{outgoingCount}</span>
            </div>
        </div>

        <div class="sv3-sp-actions">
            <button class="sv3-sp-action-btn primary" onclick={onOpenNote}>
                {$t('sightV3.sidePanel.openNote') || 'Open in editor'}
            </button>
        </div>

        <div class="sv3-sp-divider"></div>
    {/if}

    <!-- MIG-019 §2D: Universe-health card. Always visible when there's
         data to show (totalNotes > 0). Tucks below note details when a
         star is selected; sits at the top when nothing is selected. -->
    {#if health.totalNotes > 0}
        <div class="sv3-sp-section">
            <div class="sv3-sp-section-title">{$t('sightV3.sidePanel.universeHealth') || 'Universe health'}</div>
            <div class="sv3-sp-score-row">
                <span class="sv3-sp-score">{health.score}</span>
                <span class="sv3-sp-score-label">/ 100</span>
            </div>

            <div class="sv3-sp-metric">
                <div class="sv3-sp-metric-row">
                    <span class="sv3-sp-label">{$t('sightV3.sidePanel.modularity') || 'Modularity'}</span>
                    <span class="sv3-sp-value">{health.modularity.display}</span>
                    <span class={statusClass(health.modularity.status)}>{statusLabel(health.modularity.status)}</span>
                </div>
            </div>
            <div class="sv3-sp-metric">
                <div class="sv3-sp-metric-row">
                    <span class="sv3-sp-label">{$t('sightV3.sidePanel.dominance') || 'Dominance'}</span>
                    <span class="sv3-sp-value">{health.dominance.display}</span>
                    <span class={statusClass(health.dominance.status)}>{statusLabel(health.dominance.status)}</span>
                </div>
            </div>
            <div class="sv3-sp-metric">
                <div class="sv3-sp-metric-row">
                    <span class="sv3-sp-label">{$t('sightV3.sidePanel.entropy') || 'Entropy'}</span>
                    <span class="sv3-sp-value">{health.entropy.display}</span>
                    <span class={statusClass(health.entropy.status)}>{statusLabel(health.entropy.status)}</span>
                </div>
            </div>
            <div class="sv3-sp-metric">
                <div class="sv3-sp-metric-row">
                    <span class="sv3-sp-label">{$t('sightV3.sidePanel.connectivity') || 'Connectivity'}</span>
                    <span class="sv3-sp-value">{health.connectivity.display}</span>
                    <span class={statusClass(health.connectivity.status)}>{statusLabel(health.connectivity.status)}</span>
                </div>
            </div>

            <p class="sv3-sp-hint">
                {health.totalNotes} {$t('sightV3.sidePanel.notes') || 'notes'} · {health.totalEdges} {$t('sightV3.sidePanel.edges') || 'edges'} · {health.communityCount} {$t('sightV3.sidePanel.communities') || 'communities'}
            </p>
        </div>

        {#if !notePath}
            <p class="sv3-sp-hint sv3-sp-hint-secondary">
                {$t('sightV3.sidePanel.clickStarHint') || 'Click a star to see its details.'}
            </p>
        {/if}
    {/if}
</div>

<style>
    .sight-v3-side-panel {
        position: absolute;
        top: 0;
        right: 0;
        width: 320px;
        height: 100vh;
        background: rgba(15, 23, 41, 0.95);
        border-inline-start: 1px solid rgba(245, 230, 200, 0.2);
        color: #f5e6c8;
        padding: 56px 20px 20px;
        box-sizing: border-box;
        transform: translateX(100%);
        transition: transform 200ms ease-out;
        z-index: 5;
        overflow-y: auto;
    }

    .sight-v3-side-panel.open {
        transform: translateX(0);
    }

    .sv3-sp-header {
        display: flex;
        align-items: flex-start;
        justify-content: space-between;
        gap: 12px;
        margin-bottom: 16px;
    }

    .sv3-sp-title {
        font-size: 16px;
        font-weight: 600;
        margin: 0;
        line-height: 1.3;
        flex: 1;
    }

    .sv3-sp-close-btn {
        background: transparent;
        border: none;
        color: #f5e6c8;
        font-size: 22px;
        line-height: 1;
        cursor: pointer;
        padding: 0;
        width: 24px;
        height: 24px;
    }

    .sv3-sp-close-btn:hover {
        color: #d4af37;
    }

    .sv3-sp-meta,
    .sv3-sp-section {
        margin-bottom: 20px;
    }

    .sv3-sp-section-title {
        font-size: 11px;
        text-transform: uppercase;
        letter-spacing: 0.06em;
        color: rgba(245, 230, 200, 0.55);
        margin-bottom: 8px;
    }

    .sv3-sp-row {
        display: flex;
        justify-content: space-between;
        font-size: 13px;
        padding: 4px 0;
    }

    .sv3-sp-label {
        color: rgba(245, 230, 200, 0.65);
    }

    .sv3-sp-value {
        color: #f5e6c8;
        font-weight: 500;
    }

    .sv3-sp-actions {
        margin-top: 24px;
    }

    .sv3-sp-action-btn {
        width: 100%;
        padding: 10px;
        background: rgba(212, 175, 55, 0.15);
        border: 1px solid rgba(212, 175, 55, 0.4);
        border-radius: 6px;
        color: #f5e6c8;
        font-size: 13px;
        cursor: pointer;
    }

    .sv3-sp-action-btn:hover {
        background: rgba(212, 175, 55, 0.3);
        border-color: rgba(212, 175, 55, 0.7);
    }

    .sv3-sp-action-btn.primary {
        font-weight: 500;
    }

    /* MIG-019 §2D: universe-health card styles */
    .sv3-sp-divider {
        height: 1px;
        background: rgba(245, 230, 200, 0.15);
        margin: 16px 0;
    }

    .sv3-sp-score-row {
        display: flex;
        align-items: baseline;
        gap: 6px;
        margin-bottom: 12px;
    }

    .sv3-sp-score {
        font-size: 28px;
        font-weight: 600;
        color: #d4af37;
    }

    .sv3-sp-score-label {
        font-size: 12px;
        color: rgba(245, 230, 200, 0.55);
    }

    .sv3-sp-metric {
        margin: 6px 0;
    }

    .sv3-sp-metric-row {
        display: grid;
        grid-template-columns: 1fr auto auto;
        align-items: center;
        gap: 8px;
        font-size: 12px;
    }

    .sv3-sp-badge {
        font-size: 10px;
        text-transform: uppercase;
        letter-spacing: 0.04em;
        padding: 2px 6px;
        border-radius: 3px;
        font-weight: 500;
    }

    .sv3-sp-badge-healthy {
        background: rgba(74, 222, 128, 0.18);
        color: #4ade80;
    }

    .sv3-sp-badge-caution {
        background: rgba(250, 204, 21, 0.18);
        color: #facc15;
    }

    .sv3-sp-badge-imbalanced {
        background: rgba(248, 113, 113, 0.18);
        color: #f87171;
    }

    .sv3-sp-hint {
        font-size: 11px;
        color: rgba(245, 230, 200, 0.55);
        margin: 12px 0 0;
        line-height: 1.4;
    }

    .sv3-sp-hint-secondary {
        margin-top: 16px;
        font-style: italic;
    }
</style>
