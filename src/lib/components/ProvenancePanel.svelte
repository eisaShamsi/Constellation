<script lang="ts">
	import { t } from '$lib/i18n';

	interface AncestorNode {
		name: string;
		path: string;
		depth: number;
		has_external_source: boolean;
	}
	interface ProvenanceChain {
		note_path: string;
		note_name: string;
		origin_type: string;
		trust_depth: number;
		ancestors: AncestorNode[];
	}

	let {
		chain = null as ProvenanceChain | null,
		onNoteClick,
		libraryColorMap = {} as Record<string, string>,
	}: {
		chain?: ProvenanceChain | null;
		onNoteClick?: (path: string, name: string) => void;
		libraryColorMap?: Record<string, string>;
	} = $props();

	function originColor(type: string): string {
		return type === 'received' ? '#4A9EFF' : type === 'discovered' ? '#FFB347' : type === 'mixed' ? '#A78BFA' : '#9ca3af';
	}
	function originLabel(type: string): string {
		return type === 'received' ? ($t('provenancePanel.received') || 'Received')
			: type === 'discovered' ? ($t('provenancePanel.discovered') || 'Discovered')
			: type === 'mixed' ? ($t('provenancePanel.mixed') || 'Mixed')
			: ($t('provenancePanel.noChain') || 'No chain');
	}
</script>

<div class="prov-panel">
	{#if !chain}
		<div class="prov-empty">{$t('provenancePanel.loading') || 'Loading...'}</div>
	{:else if chain.ancestors.length === 0 && chain.origin_type === 'none'}
		<div class="prov-empty-state">
			<div class="prov-empty-icon">🔗</div>
			<div class="prov-empty-text">{$t('provenancePanel.noDerivesFrom') || 'No derives-from chain found.'}</div>
			<div class="prov-empty-hint">{$t('provenancePanel.hint') || 'Add [[note|derives-from]] links to trace source lineage.'}</div>
		</div>
	{:else}
		<!-- Origin badge -->
		<div class="prov-origin">
			<span class="prov-origin-dot" style="background:{originColor(chain.origin_type)}"></span>
			<span class="prov-origin-label">{originLabel(chain.origin_type)}</span>
			{#if chain.trust_depth > 0}
				<span class="prov-depth">{$t('provenancePanel.depth') || 'Depth'}: {chain.trust_depth}</span>
			{/if}
		</div>

		<!-- Current note (root of display) -->
		<div class="prov-chain">
			<div class="prov-node prov-current">
				<span class="prov-node-name">{chain.note_name}</span>
			</div>

			<!-- Ancestor tree -->
			{#each [...chain.ancestors].sort((a, b) => a.depth - b.depth) as ancestor}
				<div class="prov-node" style="padding-inline-start:{ancestor.depth * 16}px">
					<span class="prov-connector">↳</span>
					<button class="prov-ancestor" onclick={() => onNoteClick?.(ancestor.path, ancestor.name)}>
						<span class="prov-source-dot" style="background:{ancestor.has_external_source ? '#4A9EFF' : '#FFB347'}"></span>
						<span class="prov-ancestor-name">{ancestor.name}</span>
						<span class="prov-ancestor-badge">{ancestor.depth}</span>
						{#if ancestor.has_external_source}
							<span class="prov-external-tag">{$t('provenancePanel.external') || 'external'}</span>
						{/if}
					</button>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.prov-panel { padding: 8px 0; flex: 1; min-height: 0; overflow-y: auto; } /* self-scrolling — host column is overflow:hidden */
	.prov-empty { font-size: calc(0.78rem * var(--rs-scale, 1)); color: var(--text-faint); padding: 8px 12px; }
	.prov-empty-state { text-align: center; padding: 24px 16px; }
	.prov-empty-icon { font-size: calc(1.5rem * var(--rs-scale, 1)); margin-bottom: 8px; }
	.prov-empty-text { font-size: calc(0.82rem * var(--rs-scale, 1)); color: var(--text-muted); }
	.prov-empty-hint { font-size: calc(0.72rem * var(--rs-scale, 1)); color: var(--text-faint); margin-top: 6px; }

	.prov-origin {
		display: flex; align-items: center; gap: 6px;
		padding: 8px 12px; margin-bottom: 8px;
		background: var(--background-secondary); border-radius: 6px;
	}
	.prov-origin-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
	.prov-origin-label { font-size: calc(0.82rem * var(--rs-scale, 1)); font-weight: 600; color: var(--text-normal); }
	.prov-depth { margin-inline-start: auto; font-size: calc(0.72rem * var(--rs-scale, 1)); color: var(--text-faint); }

	.prov-chain { padding: 0 8px; }
	.prov-node { display: flex; align-items: center; gap: 4px; padding: 3px 4px; }
	.prov-current { font-weight: 600; font-size: calc(0.82rem * var(--rs-scale, 1)); color: var(--text-normal); padding-bottom: 6px; }
	.prov-node-name { font-size: calc(0.82rem * var(--rs-scale, 1)); }
	.prov-connector { color: var(--text-faint); font-size: calc(0.75rem * var(--rs-scale, 1)); flex-shrink: 0; }

	.prov-ancestor {
		display: flex; align-items: center; gap: 4px;
		border: none; background: none; cursor: pointer; padding: 3px 6px;
		border-radius: 4px; font-family: inherit; font-size: calc(0.78rem * var(--rs-scale, 1));
		color: var(--text-normal); text-align: start;
	}
	.prov-ancestor:hover { background: var(--background-modifier-hover); }
	.prov-source-dot { width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0; }
	.prov-ancestor-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 150px; }
	.prov-ancestor-badge {
		font-size: calc(0.65rem * var(--rs-scale, 1)); font-weight: 600; color: var(--text-faint);
		background: var(--background-modifier-border); border-radius: 3px;
		padding: 0 4px; min-width: 16px; text-align: center;
	}
	.prov-external-tag {
		font-size: calc(0.62rem * var(--rs-scale, 1)); color: #4A9EFF; font-weight: 500;
		border: 1px solid #4A9EFF40; border-radius: 3px; padding: 0 3px;
	}
</style>
