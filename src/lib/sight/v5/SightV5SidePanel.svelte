<script lang="ts">
	/**
	 * Sight v5 — slide-in right side panel (§5).
	 *
	 * Shows the selected star's note detail. Width ~300 px. Per D-V2
	 * (Eisa, 2026-05-12): slide-in right pattern, matching Backlinks /
	 * Outgoing panels' shape.
	 *
	 * Header: title + close button.
	 * Body: strata badge + maturity + stage + sources + confidence
	 *   summary + top incident link count.
	 * Footer: "Open in editor" — handoff per Concept Paper §10 (Sight
	 *   = whole universe; 360.3D / NotePane = single note). Sight v5
	 *   does NOT deepen into a per-note view.
	 */
	import { t } from '$lib/i18n';
	import type { LayoutCacheRow } from './types';

	interface Props {
		note: LayoutCacheRow;
		linkCount: number;
		onClose: () => void;
		onOpenInEditor: (notePath: string) => void;
	}
	let { note, linkCount, onClose, onOpenInEditor }: Props = $props();

	function stratumLabel(s: number | null): string {
		if (s == null) return $t('sight.v5.field.unstratified') || 'Unstratified';
		const names = ['Datum', 'Fact', 'Opinion', 'Hypothesis', 'Theory', 'Framework', 'Perspective', 'Worldview'];
		const name = names[s - 1] ?? '';
		return `L${s} ${name}`;
	}

	function confidenceLabel(a: number | null, contested: boolean): string {
		if (contested) return $t('sight.v5.field.contested') || 'Contested';
		if (a == null) return $t('sight.v5.field.hypothesis') || 'Hypothesis';
		if (a >= 0.95) return $t('sight.v5.field.established') || 'Established';
		if (a >= 0.6) return $t('sight.v5.field.evidence') || 'Evidence';
		return $t('sight.v5.field.hypothesis') || 'Hypothesis';
	}

	function noteTitle(path: string): string {
		const slash = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
		const name = slash >= 0 ? path.slice(slash + 1) : path;
		return name.replace(/\.md$/i, '');
	}
</script>

<aside class="sight-v5-side-panel" role="complementary" aria-label="Selected note detail">
	<header class="sv5-sp-header">
		<h3 dir="auto" title={note.notePath}>{noteTitle(note.notePath)}</h3>
		<button class="sv5-sp-close" onclick={onClose} aria-label={$t('common.close') || 'Close'}>×</button>
	</header>

	<dl class="sv5-sp-body">
		<dt>{$t('sight.v5.field.strata') || 'Strata'}</dt>
		<dd>{stratumLabel(note.stratum)}</dd>

		<dt>{$t('sight.v5.field.maturity') || 'Maturity'}</dt>
		<dd>{note.maturity ?? '—'}</dd>

		<dt>{$t('sight.v5.field.confidence') || 'Confidence'}</dt>
		<dd>{confidenceLabel(note.confidenceAlpha, note.contested)}</dd>

		{#if note.stage}
			<dt>{$t('sight.v5.field.stage') || 'Stage'}</dt>
			<dd>{note.stage}</dd>
		{/if}

		{#if note.sourcesPrimary}
			<dt>{$t('sight.v5.field.source') || 'Source'}</dt>
			<dd>{note.sourcesPrimary}</dd>
		{/if}

		{#if note.actsPrimary}
			<dt>{$t('sight.v5.field.act') || 'Act'}</dt>
			<dd>{note.actsPrimary}</dd>
		{/if}

		{#if note.libraryName}
			<dt>{$t('sight.v5.field.library') || 'Library'}</dt>
			<dd>{note.libraryName}</dd>
		{/if}

		<dt>{$t('sight.v5.field.incidentLinks') || 'Incident links'}</dt>
		<dd>{linkCount}</dd>
	</dl>

	<footer class="sv5-sp-footer">
		<button class="sv5-sp-open" onclick={() => onOpenInEditor(note.notePath)}>
			{$t('sight.v5.action.openInEditor') || 'Open in editor →'}
		</button>
	</footer>
</aside>

<style>
	.sight-v5-side-panel {
		position: absolute;
		top: 0;
		right: 0;
		bottom: 0;
		width: 300px;
		background: #fbf8ec;
		border-left: 1px solid #b8a98a;
		display: flex;
		flex-direction: column;
		font-family: Georgia, 'Times New Roman', serif;
		color: #1a1a1a;
		animation: sv5-slide-in 220ms ease-out;
		z-index: 5;
		box-shadow: -4px 0 12px rgba(0, 0, 0, 0.06);
	}
	@keyframes sv5-slide-in {
		from { transform: translateX(100%); opacity: 0; }
		to   { transform: translateX(0);    opacity: 1; }
	}
	.sv5-sp-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.75rem 1rem;
		border-bottom: 1px solid #b8a98a;
	}
	.sv5-sp-header h3 {
		margin: 0;
		font-size: 0.95rem;
		font-weight: 600;
		color: #2a4a8c;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		flex: 1;
	}
	.sv5-sp-close {
		background: transparent;
		border: none;
		font-size: 1.5rem;
		color: #1a1a1a;
		cursor: pointer;
		padding: 0 0.4rem;
		line-height: 1;
	}
	.sv5-sp-body {
		flex: 1;
		overflow-y: auto;
		margin: 0;
		padding: 1rem;
		display: grid;
		grid-template-columns: max-content 1fr;
		column-gap: 1rem;
		row-gap: 0.5rem;
		font-size: 0.85rem;
	}
	.sv5-sp-body dt {
		color: #2a4a8c;
		font-weight: 600;
		letter-spacing: 0.05em;
		text-transform: uppercase;
		font-size: 0.7rem;
		align-self: center;
	}
	.sv5-sp-body dd {
		margin: 0;
		align-self: center;
	}
	.sv5-sp-footer {
		padding: 0.75rem 1rem;
		border-top: 1px solid #b8a98a;
	}
	.sv5-sp-open {
		width: 100%;
		padding: 0.5rem;
		background: #2a4a8c;
		color: #faf6e8;
		border: none;
		border-radius: 4px;
		font-family: inherit;
		font-size: 0.9rem;
		cursor: pointer;
	}
	.sv5-sp-open:hover {
		background: #1f3866;
	}
</style>
