<script lang="ts">
	/**
	 * Sight v5 — Layer 1 visual foundation, MIG-024.
	 *
	 * Skeleton component (§1). The dome geometry + Canvas render pipeline
	 * land in §3; mode + scope dispatch in §4; stars + interactivity in §5.
	 * For §1 this component just confirms the gating/mount path works:
	 * when SIGHT_V5_ENABLED flips to true (in §6, the v5 ship moment),
	 * clicking the dock button opens this surface inside `.content-area`
	 * (the SkyView mount pattern v4 inherited from after v3's
	 * position:fixed overlay catastrophe).
	 *
	 * Concept Paper v3.1 §12.1 + Mock B1 (docs/Sight-vNext-MockB1-Toggle.svg)
	 * are the binding contracts. Production code reconciles pixel-for-pixel
	 * within Suwaidi palette tokens once §3 lands.
	 */
	import { t } from '$lib/i18n';
	import { appSettings } from '$lib/libraries/store';
	import type { SightV5Mode, SightV5Scope } from './types';

	// Read persisted mode/scope; fall back to defaults if absent or
	// unrecognized (the latter handles a saved value that's no longer
	// in the SightV5Mode union — e.g. a future deprecation).
	const VALID_MODES: ReadonlySet<SightV5Mode> = new Set(['R', 'L', 'T', 'C', 'S', 'A', 'P']);
	const VALID_SCOPES: ReadonlySet<SightV5Scope> = new Set(['universe', 'library', 'folder']);

	let activeMode: SightV5Mode = $derived.by(() => {
		const saved = $appSettings.sight?.lastMode;
		return saved && VALID_MODES.has(saved) ? saved : 'R';
	});

	let activeScope: SightV5Scope = $derived.by(() => {
		const saved = $appSettings.sight?.lastScope;
		return saved && VALID_SCOPES.has(saved) ? saved : 'universe';
	});
</script>

<div class="sight-v5-root">
	<header class="sight-v5-placeholder-header">
		<h2>{$t('sight.v5.title') || 'Sight v5'}</h2>
		<p class="muted">
			{$t('sight.v5.skeletonNotice') ||
				'Skeleton (MIG-024 §1). Dome, modes, and interactivity land in §3–§5.'}
		</p>
		<dl class="sight-v5-state">
			<dt>Active mode</dt>
			<dd>{activeMode}</dd>
			<dt>Active scope</dt>
			<dd>{activeScope}</dd>
		</dl>
	</header>
</div>

<style>
	.sight-v5-root {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		background: #faf6e8;
		color: #1a1a1a;
		padding: 2rem;
		font-family: Georgia, 'Times New Roman', serif;
	}
	.sight-v5-placeholder-header {
		max-width: 32rem;
		margin: 4rem auto;
		text-align: center;
	}
	.sight-v5-placeholder-header h2 {
		margin: 0 0 0.75rem 0;
		color: #2a4a8c;
		letter-spacing: 0.15em;
		text-transform: uppercase;
		font-size: 1.1rem;
	}
	.muted {
		color: #3a3a3a;
		font-style: italic;
		margin: 0 0 1.5rem 0;
	}
	.sight-v5-state {
		display: grid;
		grid-template-columns: max-content 1fr;
		column-gap: 1rem;
		row-gap: 0.4rem;
		max-width: 24rem;
		margin: 0 auto;
		text-align: start;
		font-size: 0.95rem;
	}
	.sight-v5-state dt {
		color: #2a4a8c;
		font-weight: 600;
		letter-spacing: 0.05em;
	}
	.sight-v5-state dd {
		margin: 0;
	}
</style>
