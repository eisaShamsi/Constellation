<script lang="ts">
	/**
	 * PJ-068 v2 (P2) — the Note Radial Graph.
	 *
	 * The open note at the centre; ALL its BACKLINKS radiate LEFT and ALL its OUTGOING
	 * links radiate RIGHT — every link shown (like Sky View), as a small node coloured by
	 * its typed relationship (supports / contradicts / causes / derives-from …) and sized
	 * by the link's living weight (lifecycle tier). Dense-but-clear: hover a node to reveal
	 * its note + a spoke to the centre; click a node → the MAIN window opens it (read-only).
	 * Pure presentational — the host passes the persisted note_links rows (no disk/index).
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
		resolveTarget?: (name: string) => { path: string; libraryName: string };
		onNavigate?: (path: string, name: string, libraryName: string) => void;
	} = $props();

	const TYPE_COLOR: Record<string, string> = {
		supports: '#16a34a', contradicts: '#dc2626', causes: '#ea580c',
		exemplifies: '#0d9488', generalizes: '#2563eb', 'derives-from': '#7c3aed',
		'part-of': '#b45309', supersedes: '#db2777', associative: '#94a3b8',
	};
	const typeColor = (t?: string) => TYPE_COLOR[(t || 'associative').toLowerCase()] || TYPE_COLOR.associative;
	const clean = (n: string) => (n || '').replace(/\.md$/, '');
	// Small radius; encodes the living-link weight via its lifecycle tier (weight-derived).
	const TIER_R: Record<string, number> = { emerging: 4, established: 5.5, 'load-bearing': 8, stale: 4 };
	const nodeR = (tier?: string) => TIER_R[(tier || 'emerging').toLowerCase()] ?? 4.5;

	const CX = 500, CY = 340, R = 268;

	function layoutSide(items: any[], side: 'left' | 'right') {
		const n = items.length;
		// left = backlinks on the left semicircle (95°→265°); right = outgoing (−85°→85°).
		const startDeg = side === 'left' ? 95 : -85;
		const endDeg = side === 'left' ? 265 : 85;
		return items.map((it, i) => {
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
				type: it.linkType || 'associative', side, faded: it.tier === 'stale',
			};
		});
	}

	let nodes = $derived([...layoutSide(backlinks, 'left'), ...layoutSide(outgoing, 'right')]);
	let hasAny = $derived(backlinks.length > 0 || outgoing.length > 0);
	let hovered = $state<number>(-1);

	// The typed relationships actually present → a compact legend.
	let legend = $derived.by(() => {
		const seen = new Set<string>();
		for (const n of nodes) seen.add(n.type.toLowerCase());
		return [...seen].map((t) => ({ type: t, color: typeColor(t) }));
	});

	function go(node: { path: string; name: string; lib: string }) {
		if (node.path) onNavigate?.(node.path, node.name, node.lib);
	}
</script>

<div class="rg">
	{#if hasAny}
		<svg viewBox="0 0 1000 700" preserveAspectRatio="xMidYMid meet" role="img"
			aria-label="Radial link graph — {backlinks.length} backlinks, {outgoing.length} outgoing">
			<text class="rg-side" x="150" y="30">{backlinks.length} backlinks — what points here</text>
			<text class="rg-side" x="850" y="30" text-anchor="end">{outgoing.length} outgoing — where it points</text>

			{#if hovered >= 0 && nodes[hovered]}
				{@const nd = nodes[hovered]}
				<line x1={CX} y1={CY} x2={nd.x} y2={nd.y} stroke={nd.color} stroke-width="1.5" />
			{/if}

			{#each nodes as nd, i}
				<circle class="rg-dot" class:rg-disabled={!nd.path} cx={nd.x} cy={nd.y}
					r={i === hovered ? nd.r + 2.5 : nd.r} fill={nd.color} opacity={nd.faded ? 0.5 : 1}
					role="button" tabindex={nd.path ? 0 : -1} aria-label="{nd.type}: {nd.name}"
					onmouseenter={() => hovered = i} onmouseleave={() => { if (hovered === i) hovered = -1; }}
					onfocus={() => hovered = i} onblur={() => { if (hovered === i) hovered = -1; }}
					onclick={() => go(nd)} onkeydown={(e) => { if (e.key === 'Enter') go(nd); }}>
					<title>{nd.type} · {nd.name}</title>
				</circle>
			{/each}

			{#if hovered >= 0 && nodes[hovered]}
				{@const nd = nodes[hovered]}
				<g pointer-events="none">
					<text class="rg-htype" x={nd.side === 'left' ? nd.x - nd.r - 8 : nd.x + nd.r + 8} y={nd.y - 4}
						text-anchor={nd.side === 'left' ? 'end' : 'start'} fill={nd.color}>{nd.type}</text>
					<text class="rg-hname" x={nd.side === 'left' ? nd.x - nd.r - 8 : nd.x + nd.r + 8} y={nd.y + 10}
						text-anchor={nd.side === 'left' ? 'end' : 'start'}>{nd.name}</text>
				</g>
			{/if}

			<rect x={CX - 70} y={CY - 30} width="140" height="60" rx="12"
				fill="var(--background-primary, #fff)" stroke="var(--background-modifier-border-focus, #b8b8c0)" stroke-width="1.5" />
			<text class="rg-cn" x={CX} y={CY - 2} text-anchor="middle">{clean(noteName)}</text>
			<text class="rg-cs" x={CX} y={CY + 16} text-anchor="middle">{outgoing.length} out · {backlinks.length} in</text>
		</svg>

		<div class="rg-legend">
			{#each legend as l}
				<span class="rg-lg"><span class="rg-lgd" style="background:{l.color}"></span>{l.type}</span>
			{/each}
			<span class="rg-lghint">hover a node to reveal · click to open</span>
		</div>
	{:else}
		<div class="rg-empty">
			<svg viewBox="0 0 24 24" width="34" height="34" fill="none" stroke="currentColor" stroke-width="1.4" opacity="0.35" aria-hidden="true"><circle cx="12" cy="12" r="3" /><circle cx="4" cy="6" r="1.6" /><circle cx="20" cy="18" r="1.6" /></svg>
			<p>this note stands alone — no links yet</p>
		</div>
	{/if}
</div>

<style>
	.rg { width: 100%; height: 100%; min-height: 0; display: flex; flex-direction: column; }
	.rg svg { width: 100%; flex: 1; min-height: 0; }
	.rg-side { font: 500 12px var(--font-sans); fill: var(--text-muted, #6b7280); }
	.rg-dot { cursor: pointer; transition: r 0.08s; }
	.rg-dot:hover { stroke: var(--text-normal, #1a1a1a); stroke-width: 1.5; }
	.rg-dot.rg-disabled { cursor: default; }
	.rg-htype { font: 500 12px var(--font-sans); }
	.rg-hname { font: 13px var(--font-sans); fill: var(--text-normal, #1a1a1a); }
	.rg-cn { font: 500 14px var(--font-sans); fill: var(--text-normal, #1a1a1a); }
	.rg-cs { font: 10px var(--font-sans); fill: var(--text-muted, #6b7280); }
	.rg-legend { display: flex; gap: 12px; flex-wrap: wrap; align-items: center; justify-content: center; padding: 6px 0 2px; font-size: 11px; color: var(--text-secondary, #4b5563); }
	.rg-lg { display: inline-flex; align-items: center; gap: 5px; text-transform: lowercase; }
	.rg-lgd { width: 9px; height: 9px; border-radius: 50%; display: inline-block; }
	.rg-lghint { color: var(--text-faint, #9ca3af); }
	.rg-empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 10px; color: var(--text-faint, #9ca3af); }
	.rg-empty p { font-size: 13px; }
</style>
