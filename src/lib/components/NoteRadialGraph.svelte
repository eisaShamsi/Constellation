<script lang="ts">
	/**
	 * PJ-068 v2 (P2) — the Note Radial Graph.
	 *
	 * The open note at the centre; its BACKLINKS radiate LEFT and its OUTGOING links
	 * radiate RIGHT — each node a linked note, coloured by its typed relationship
	 * (supports / contradicts / causes / derives-from …), sized by the link's living
	 * weight (via its lifecycle tier). Read-only: clicking a node asks the MAIN window
	 * to open that note (onNavigate → sendNoteToMain). Pure presentational — the host
	 * fetches the persisted note_links rows (get_backlink_rows / get_outgoing_rows) and
	 * passes them in, so this never touches disk or the index.
	 */
	import { detectDir } from '$lib/utils';

	let {
		noteName = '',
		backlinks = [] as any[],
		outgoing = [] as any[],
		resolveTarget,
		onNavigate,
	}: {
		noteName?: string;
		backlinks?: any[];
		outgoing?: any[];
		/** outgoing rows carry a target NAME, not a path — the host resolves it. */
		resolveTarget?: (name: string) => { path: string; libraryName: string };
		onNavigate?: (path: string, name: string, libraryName: string) => void;
	} = $props();

	// The 8 typed links + associative — Constellation's cognitive vocabulary.
	const TYPE_COLOR: Record<string, string> = {
		supports: '#16a34a', contradicts: '#dc2626', causes: '#ea580c',
		exemplifies: '#0d9488', generalizes: '#2563eb', 'derives-from': '#7c3aed',
		'part-of': '#b45309', supersedes: '#db2777', associative: '#94a3b8',
	};
	const typeColor = (t?: string) => TYPE_COLOR[(t || 'associative').toLowerCase()] || TYPE_COLOR.associative;
	const clean = (n: string) => (n || '').replace(/\.md$/, '');
	// Node radius encodes the living-link weight via its lifecycle tier (weight-derived).
	const TIER_R: Record<string, number> = { emerging: 7, established: 10, 'load-bearing': 14, stale: 7 };
	const nodeR = (tier?: string) => TIER_R[(tier || 'emerging').toLowerCase()] ?? 8;

	const CX = 450, CY = 300, R = 205, MAXSIDE = 9;

	function layoutSide(items: any[], side: 'left' | 'right') {
		// Heaviest (load-bearing) first so the strongest relationships take the prime slots.
		const order = ['load-bearing', 'established', 'emerging', 'stale'];
		const sorted = [...items].sort((a, b) => order.indexOf((a.tier || 'emerging')) - order.indexOf((b.tier || 'emerging')));
		const shown = sorted.slice(0, MAXSIDE);
		const n = shown.length;
		const startDeg = side === 'left' ? 108 : -72;
		const endDeg = side === 'left' ? 252 : 72;
		return shown.map((it, i) => {
			const tt = n <= 1 ? 0.5 : i / (n - 1);
			const rad = ((startDeg + tt * (endDeg - startDeg)) * Math.PI) / 180;
			const isOut = side === 'right';
			const rawName = isOut ? it.target : it.name;
			const resolved = isOut
				? (resolveTarget?.(it.target) ?? { path: '', libraryName: it.libraryName })
				: { path: it.path, libraryName: it.libraryName };
			return {
				x: CX + R * Math.cos(rad), y: CY + R * Math.sin(rad),
				r: nodeR(it.tier), color: typeColor(it.linkType),
				name: clean(rawName), path: resolved.path, lib: resolved.libraryName || it.libraryName,
				type: it.linkType || 'associative', side, faded: (it.tier === 'stale'),
			};
		});
	}

	let backNodes = $derived(layoutSide(backlinks, 'left'));
	let outNodes = $derived(layoutSide(outgoing, 'right'));
	let backMore = $derived(Math.max(0, backlinks.length - MAXSIDE));
	let outMore = $derived(Math.max(0, outgoing.length - MAXSIDE));
	let hasAny = $derived(backlinks.length > 0 || outgoing.length > 0);

	function go(node: { path: string; name: string; lib: string }) {
		if (node.path) onNavigate?.(node.path, node.name, node.lib);
	}
</script>

<div class="rg">
	{#if hasAny}
		<svg viewBox="0 0 900 600" preserveAspectRatio="xMidYMid meet" role="img" aria-label="Radial link graph of the open note">
			<text class="rg-side" x="120" y="34">backlinks — what points here</text>
			<text class="rg-side" x="780" y="34" text-anchor="end">outgoing — where it points</text>

			<g class="rg-spokes" fill="none">
				{#each [...backNodes, ...outNodes] as nd}
					<line x1={CX} y1={CY} x2={nd.x} y2={nd.y} stroke={nd.color} stroke-opacity={nd.faded ? 0.25 : 0.5} />
				{/each}
			</g>

			{#each [...backNodes, ...outNodes] as nd}
				<g class="rg-node" class:rg-disabled={!nd.path} opacity={nd.faded ? 0.55 : 1}
					role="button" tabindex={nd.path ? 0 : -1} aria-label={nd.name}
					onclick={() => go(nd)} onkeydown={(e) => { if (e.key === 'Enter') go(nd); }}>
					<circle cx={nd.x} cy={nd.y} r={nd.r} fill={nd.color} />
					<text class="rg-tp" x={nd.side === 'left' ? nd.x - nd.r - 6 : nd.x + nd.r + 6} y={nd.y - 4}
						text-anchor={nd.side === 'left' ? 'end' : 'start'} fill={nd.color}>{nd.type}</text>
					<text class="rg-nm" x={nd.side === 'left' ? nd.x - nd.r - 6 : nd.x + nd.r + 6} y={nd.y + 9}
						text-anchor={nd.side === 'left' ? 'end' : 'start'}>{nd.name}</text>
				</g>
			{/each}

			{#if backMore > 0}<text class="rg-more" x="120" y="566">+{backMore} more backlinks</text>{/if}
			{#if outMore > 0}<text class="rg-more" x="780" y="566" text-anchor="end">+{outMore} more outgoing</text>{/if}

			<rect x={CX - 66} y={CY - 30} width="132" height="60" rx="12" fill="var(--background-primary, #fff)" stroke="var(--background-modifier-border-focus, #b8b8c0)" stroke-width="1.5" />
			<text class="rg-cn" x={CX} y={CY - 2} text-anchor="middle">{clean(noteName)}</text>
			<text class="rg-cs" x={CX} y={CY + 16} text-anchor="middle">{outgoing.length} out · {backlinks.length} in</text>
		</svg>
	{:else}
		<div class="rg-empty">
			<svg viewBox="0 0 24 24" width="34" height="34" fill="none" stroke="currentColor" stroke-width="1.4" opacity="0.35" aria-hidden="true"><circle cx="12" cy="12" r="3" /><circle cx="4" cy="6" r="1.6" /><circle cx="20" cy="18" r="1.6" /></svg>
			<p>this note stands alone — no links yet</p>
		</div>
	{/if}
</div>

<style>
	.rg { width: 100%; height: 100%; min-height: 0; display: flex; }
	.rg svg { width: 100%; height: 100%; }
	.rg-side { font: 500 12px var(--font-sans); fill: var(--text-muted, #6b7280); }
	.rg-more { font: 11px var(--font-sans); fill: var(--text-faint, #9ca3af); }
	.rg-node { cursor: pointer; }
	.rg-node:hover circle { stroke: var(--text-normal, #1a1a1a); stroke-width: 1.5; }
	.rg-node.rg-disabled { cursor: default; }
	.rg-node.rg-disabled:hover circle { stroke: none; }
	.rg-tp { font: 9px var(--font-sans); }
	.rg-nm { font: 12px var(--font-sans); fill: var(--text-normal, #1a1a1a); }
	.rg-cn { font: 500 14px var(--font-sans); fill: var(--text-normal, #1a1a1a); }
	.rg-cs { font: 10px var(--font-sans); fill: var(--text-muted, #6b7280); }
	.rg-empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 10px; color: var(--text-faint, #9ca3af); }
	.rg-empty p { font-size: 13px; }
</style>
