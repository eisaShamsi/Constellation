<!--
    SightV3SidePanel.svelte — slides in from the right when a star is
    clicked. Shows note metadata + linked notes for that note.

    §1E: minimal version — note title, community name, centrality
    rank, top incoming + outgoing wikilinks (count-only), Open in
    editor button, Close button.

    §2G (2026-05-07): Universe Health section removed — that surface
    now lives top-center anchored to the dome (per Eisa's directive
    after the §2G.3 Boss test). The side panel is purely a per-note
    detail view; it stays closed unless a star is selected.
-->
<script lang="ts">
    import { t } from '$lib/i18n';

    interface ConnectedNote {
        path: string;
        title: string;
        libraryName: string;
        colorCss: string;
    }
    interface Props {
        notePath: string | null;
        noteTitle: string;
        communityName: string;
        centralityRank: number;
        totalNotes: number;
        incomingCount: number;
        outgoingCount: number;
        /** §2G.3n: 1-hop neighbours of the selected note. The side
         *  panel renders them as a clickable list so the user can see
         *  WHICH notes are linked, with their titles and libraries. */
        connectedNotes?: ConnectedNote[];
        onOpenNote: () => void;
        /** §2G.3n: invoked when the user clicks a connected-note row. */
        onConnectedClick?: (path: string) => void;
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
        connectedNotes = [],
        onOpenNote,
        onConnectedClick,
        onClose,
    }: Props = $props();

    /** Open only when a star is selected — no fallback Universe Health. */
    const isOpen = $derived(notePath !== null);
</script>

<div class="sight-v3-side-panel" class:open={isOpen} dir="auto">
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

        {#if connectedNotes.length > 0}
            <div class="sv3-sp-section">
                <div class="sv3-sp-section-title">
                    {$t('sightV3.sidePanel.connectedNotes') || 'Connected notes'}
                    <span class="sv3-sp-section-count">({connectedNotes.length})</span>
                </div>
                <div class="sv3-sp-connected-list">
                    {#each connectedNotes as cn (cn.path)}
                        <button
                            class="sv3-sp-connected-row"
                            onclick={() => onConnectedClick?.(cn.path)}
                            dir="auto"
                            title={cn.title}
                        >
                            <span class="sv3-sp-connected-dot" style="background: {cn.colorCss};"></span>
                            <span class="sv3-sp-connected-title">{cn.title}</span>
                            <span class="sv3-sp-connected-lib" dir="auto">{cn.libraryName}</span>
                        </button>
                    {/each}
                </div>
            </div>
        {/if}

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
        /* MIG-019 §2G: Suwaidi cream theme — was navy. */
        background: rgba(250, 246, 232, 0.97);
        border-inline-start: 1px solid rgba(26, 26, 26, 0.15);
        color: #1a1a1a;
        padding: 56px 20px 20px;
        box-sizing: border-box;
        transform: translateX(100%);
        transition: transform 200ms ease-out;
        /* §2G.3h: 5 → 50 so the panel sits above legend (z:7),
           rim wrapper (z:6), Universe Health (z:8), Universe-name (z:8)
           but below close button (z:1000). */
        z-index: 50;
        overflow-y: auto;
        box-shadow: -2px 0 6px rgba(26, 26, 26, 0.08);
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
        color: #2a4a8c;
    }

    .sv3-sp-close-btn {
        background: transparent;
        border: none;
        color: #1a1a1a;
        font-size: 22px;
        line-height: 1;
        cursor: pointer;
        padding: 0;
        width: 24px;
        height: 24px;
    }

    .sv3-sp-close-btn:hover {
        color: #c9a227;
    }

    .sv3-sp-meta,
    .sv3-sp-section {
        margin-bottom: 20px;
    }

    .sv3-sp-section-title {
        font-size: 11px;
        text-transform: uppercase;
        letter-spacing: 0.06em;
        color: rgba(26, 26, 26, 0.55);
        margin-bottom: 8px;
    }

    .sv3-sp-row {
        display: flex;
        justify-content: space-between;
        font-size: 13px;
        padding: 4px 0;
    }

    .sv3-sp-label {
        color: rgba(26, 26, 26, 0.65);
    }

    .sv3-sp-value {
        color: #1a1a1a;
        font-weight: 500;
    }

    .sv3-sp-actions {
        margin-top: 24px;
    }

    .sv3-sp-action-btn {
        width: 100%;
        padding: 10px;
        background: rgba(201, 162, 39, 0.12);
        border: 1px solid rgba(201, 162, 39, 0.5);
        border-radius: 6px;
        color: #1a1a1a;
        font-size: 13px;
        cursor: pointer;
    }

    .sv3-sp-action-btn:hover {
        background: rgba(201, 162, 39, 0.25);
        border-color: rgba(201, 162, 39, 0.8);
    }

    .sv3-sp-action-btn.primary {
        font-weight: 500;
    }

    /* §2G.3n: Connected-notes list — 1-hop neighbours of the selected
       star, rendered as a clickable list so the user can see WHICH
       notes are linked, not just how many. Click a row to recentre the
       side panel on that neighbour. */
    .sv3-sp-section-count {
        color: rgba(26, 26, 26, 0.45);
        font-weight: 400;
        margin-inline-start: 4px;
        text-transform: none;
        letter-spacing: 0;
    }

    .sv3-sp-connected-list {
        display: flex;
        flex-direction: column;
        gap: 2px;
        max-height: 320px;
        overflow-y: auto;
        margin-inline: -6px;
        padding-inline: 2px;
    }

    .sv3-sp-connected-row {
        display: flex;
        align-items: center;
        gap: 8px;
        width: 100%;
        padding: 6px 8px;
        background: transparent;
        border: 1px solid transparent;
        border-radius: 4px;
        color: #1a1a1a;
        font-size: 12px;
        cursor: pointer;
        text-align: start;
        font-family: inherit;
    }

    .sv3-sp-connected-row:hover {
        background: rgba(201, 162, 39, 0.10);
        border-color: rgba(201, 162, 39, 0.3);
    }

    .sv3-sp-connected-dot {
        flex: 0 0 auto;
        width: 8px;
        height: 8px;
        border-radius: 50%;
        box-shadow: 0 0 0 1px rgba(26, 26, 26, 0.15);
    }

    .sv3-sp-connected-title {
        flex: 1 1 auto;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        min-width: 0;
    }

    .sv3-sp-connected-lib {
        flex: 0 0 auto;
        color: rgba(26, 26, 26, 0.55);
        font-size: 11px;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        max-width: 110px;
    }
</style>
