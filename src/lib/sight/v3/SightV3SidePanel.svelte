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

    interface Props {
        notePath: string | null;
        noteTitle: string;
        communityName: string;
        centralityRank: number;
        totalNotes: number;
        incomingCount: number;
        outgoingCount: number;
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
        onOpenNote,
        onClose,
    }: Props = $props();
</script>

<div class="sight-v3-side-panel" class:open={notePath !== null} dir="auto">
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
</style>
