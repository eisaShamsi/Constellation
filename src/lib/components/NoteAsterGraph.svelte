<script lang="ts">
	/**
	 * PJ-068 v2 — "The Aster" note-graph lens (the flagship).
	 *
	 * The open note at the centre; its links bloom into a RELATIONSHIP ROSE — one petal
	 * per typed relationship present, split backlinks-left / outgoing-right. A petal's
	 * angular WIDTH ∝ how many links of that type; its filaments are the individual links
	 * (radial threads, heaviest = longest, sorted toward the spine); the soft petal glow
	 * is the aggregate. Density becomes texture: 3 links = a few slender petals, 200 = a
	 * full balanced bloom, never a hairball. Read-only: hover a thread to name it, click to
	 * open it in the MAIN window. Relationship colours are CSS variables (--rel-*) so the
	 * Style Setter can retune the palette. Pure presentational (host passes note_links rows).
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

	// Seniority order → stable petal order across notes; associative last, contradicts is the ember.
	const REL_ORDER = ['supports', 'exemplifies', 'generalizes', 'causes', 'derives-from', 'part-of', 'supersedes', 'contradicts', 'associative'];
	// Flexoki-derived defaults; Style-Setter-overridable via --rel-*.
	const REL_DEFAULT: Record<string, string> = {
		supports: '#879A39', exemplifies: '#3AA99F', generalizes: '#4385BE', causes: '#DA702C',
		'derives-from': '#8B7EC8', 'part-of': '#D0A215', supersedes: '#CE5D97', contradicts: '#D14D41', associative: '#B7B5AC',
	};
	const relColor = (t: string) => `var(--rel-${t}, ${REL_DEFAULT[t] || REL_DEFAULT.associative})`;
	const TIERW: Record<string, number> = { 'load-bearing': 1, established: 0.72, emerging: 0.46, stale: 0.24 };
	const tierW = (t?: string) => TIERW[(t || 'emerging').toLowerCase()] ?? 0.46;
	const clean = (n: string) => (n || '').replace(/\.md$/, '');

	const CX = 450, CY = 315, RI = 56, RMAX = 250;
	const P = (r: number, d: number): [number, number] => [CX + r * Math.cos((d * Math.PI) / 180), CY + r * Math.sin((d * Math.PI) / 180)];

	function buildSide(items: any[], side: 'left' | 'right') {
		const groups: Record<string, any[]> = {};
		for (const it of items) { const t = (it.linkType || 'associative').toLowerCase(); (groups[t] = groups[t] || []).push(it); }
		const types = REL_ORDER.filter((t) => groups[t]);
		if (!types.length) return [];
		const a0deg = side === 'left' ? 96 : -84;
		const a1deg = side === 'left' ? 264 : 84;
		const span = a1deg - a0deg;
		const gap = 4;
		const w = types.map((t) => Math.sqrt(groups[t].length));
		const tw = w.reduce((a, b) => a + b, 0) || 1;
		const usable = span - gap * (types.length - 1);
		let cur = a0deg;
		return types.map((t, ti) => {
			const width = Math.max(9, (usable * w[ti]) / tw);
			const pa0 = cur, pa1 = cur + width; cur = pa1 + gap;
			const pad = Math.min(3, width * 0.15);
			const links = groups[t].slice().sort((a, b) => tierW(b.tier) - tierW(a.tier));
			const n = links.length;
			let maxLen = RI;
			const fil = links.map((lk, k) => {
				const ang = n <= 1 ? (pa0 + pa1) / 2 : pa0 + pad + ((k + 0.5) / n) * (width - 2 * pad);
				const len = RI + (RMAX - RI) * (0.28 + 0.72 * tierW(lk.tier));
				if (len > maxLen) maxLen = len;
				const isOut = side === 'right';
				const rawName = isOut ? lk.target : lk.name;
				const res = isOut ? (resolveTarget?.(lk.target) ?? { path: '', libraryName: lk.libraryName }) : { path: lk.path, libraryName: lk.libraryName };
				const [bx, by] = P(RI, ang); const [tx, ty] = P(len, ang);
				return { ang, bx, by, tx, ty, name: clean(rawName), path: res.path, lib: res.libraryName || lk.libraryName, tier: (lk.tier || 'emerging') };
			});
			// petal bloom wedge from RI to maxLen across [pa0,pa1]
			const [ix0, iy0] = P(RI, pa0), [ix1, iy1] = P(RI, pa1), [ox1, oy1] = P(maxLen + 6, pa1), [ox0, oy0] = P(maxLen + 6, pa0);
			const wedge = `M${ix0} ${iy0} A${RI} ${RI} 0 0 1 ${ix1} ${iy1} L${ox1} ${oy1} A${maxLen + 6} ${maxLen + 6} 0 0 0 ${ox0} ${oy0} Z`;
			const [lx, ly] = P(maxLen + 16, (pa0 + pa1) / 2);
			return { type: t, side, count: n, color: relColor(t), wedge, fil, labelX: lx, labelY: ly, ember: t === 'contradicts' };
		});
	}

	let leftP = $derived(buildSide(backlinks, 'left'));
	let rightP = $derived(buildSide(outgoing, 'right'));
	let petals = $derived([...leftP, ...rightP]);
	let hasAny = $derived(backlinks.length > 0 || outgoing.length > 0);
	let hovered = $state<{ px: number; k: number } | null>(null);
	let hoverNode = $derived.by(() => {
		if (!hovered) return null;
		const p = petals[hovered.px]; if (!p) return null;
		const f = p.fil[hovered.k]; if (!f) return null;
		return { ...f, color: p.color, side: p.side };
	});

	function go(f: { path: string; name: string; lib: string }) { if (f.path) onNavigate?.(f.path, f.name, f.lib); }
</script>

<div class="as">
	{#if hasAny}
		<svg viewBox="0 0 900 640" preserveAspectRatio="xMidYMid meet" role="img"
			aria-label="Relationship rose — {backlinks.length} backlinks, {outgoing.length} outgoing">
			<defs>
				<radialGradient id="as-field" cx="50%" cy="49%" r="60%"><stop offset="0%" stop-color="var(--as-field-in, #15171e)"/><stop offset="100%" stop-color="var(--as-field-out, #100F0F)"/></radialGradient>
				<radialGradient id="as-home" cx="50%" cy="50%" r="50%"><stop offset="0%" stop-color="var(--as-home, #F4EEE2)" stop-opacity="0.4"/><stop offset="55%" stop-color="var(--as-home, #F4EEE2)" stop-opacity="0.09"/><stop offset="100%" stop-color="var(--as-home, #F4EEE2)" stop-opacity="0"/></radialGradient>
				{#each petals as p, pi}
					<radialGradient id="as-pg{pi}" gradientUnits="userSpaceOnUse" cx={CX} cy={CY} r={RMAX}>
						<stop offset={RI / RMAX} stop-color={p.color} stop-opacity={p.ember ? 0.3 : 0.2}/>
						<stop offset="100%" stop-color={p.color} stop-opacity="0"/>
					</radialGradient>
				{/each}
			</defs>
			<rect x="0" y="0" width="900" height="640" fill="url(#as-field)"/>
			<circle cx={CX} cy={CY} r={RMAX} fill="none" stroke="var(--as-ring, #B7B5AC)" stroke-opacity="0.07"/>
			<text class="as-side" x="140" y="34">{backlinks.length} backlinks — what points here</text>
			<text class="as-side" x="760" y="34" text-anchor="end">{outgoing.length} outgoing — where it points</text>

			{#each petals as p, pi}
				<path d={p.wedge} fill="url(#as-pg{pi})"/>
				{#if p.ember}<path d={p.wedge} fill="none" stroke={p.color} stroke-opacity="0.28" stroke-width="1.5"/>{/if}
				{#each p.fil as f, k}
					<g class="as-fil" class:as-off={hovered && !(hovered.px === pi && hovered.k === k)}
						role="button" tabindex={f.path ? 0 : -1} aria-label="{p.type}: {f.name}"
						onmouseenter={() => hovered = { px: pi, k }} onmouseleave={() => { if (hovered?.px === pi && hovered?.k === k) hovered = null; }}
						onfocus={() => hovered = { px: pi, k }} onblur={() => { if (hovered?.px === pi && hovered?.k === k) hovered = null; }}
						onclick={() => go(f)} onkeydown={(e) => { if (e.key === 'Enter') go(f); }}>
						<line x1={f.bx} y1={f.by} x2={f.tx} y2={f.ty} stroke={p.color} stroke-opacity={f.tier === 'stale' ? 0.3 : 0.6} stroke-width={f.tier === 'load-bearing' ? 2 : 1.1}/>
						<circle cx={f.tx} cy={f.ty} r={f.tier === 'load-bearing' ? 3.2 : f.tier === 'stale' ? 1.8 : 2.4} fill={p.color} opacity={f.tier === 'stale' ? 0.5 : 1}/>
					</g>
				{/each}
				<text class="as-plabel" x={p.labelX} y={p.labelY} text-anchor="middle" fill={p.color} opacity="0.72">{p.type} · {p.count}</text>
			{/each}

			{#if hoverNode}
				<g pointer-events="none">
					<line x1={CX} y1={CY} x2={hoverNode.tx} y2={hoverNode.ty} stroke={hoverNode.color} stroke-opacity="0.55" stroke-width="1.5"/>
					<circle cx={hoverNode.tx} cy={hoverNode.ty} r="4.5" fill={hoverNode.color}/>
					<text class="as-hname" x={hoverNode.side === 'left' ? hoverNode.tx - 9 : hoverNode.tx + 9} y={hoverNode.ty + 4}
						text-anchor={hoverNode.side === 'left' ? 'end' : 'start'}>{hoverNode.name}</text>
				</g>
			{/if}

			<circle cx={CX} cy={CY} r="94" fill="url(#as-home)"/>
			<text class="as-cn" x={CX} y={CY - 1} text-anchor="middle">{clean(noteName)}</text>
			<text class="as-cs" x={CX} y={CY + 17} text-anchor="middle">{outgoing.length} out · {backlinks.length} in</text>
		</svg>
	{:else}
		<div class="as-empty">
			<svg viewBox="0 0 24 24" width="34" height="34" fill="none" stroke="currentColor" stroke-width="1.4" opacity="0.35" aria-hidden="true"><circle cx="12" cy="12" r="3"/><path d="M12 2v4M12 18v4M2 12h4M18 12h4"/></svg>
			<p>this idea stands alone — a single quiet star</p>
		</div>
	{/if}
</div>

<style>
	.as { width: 100%; height: 100%; min-height: 0; display: flex; }
	.as svg { width: 100%; height: 100%; }
	.as-side { font: 500 12px var(--font-sans); fill: var(--as-cap, #8a8880); }
	.as-plabel { font: 500 10px var(--font-sans); text-transform: lowercase; letter-spacing: 0.03em; }
	.as-fil { cursor: pointer; transition: opacity 0.12s; }
	.as-fil.as-off { opacity: 0.28; }
	.as-fil:hover circle { stroke: var(--as-home, #F4EEE2); stroke-width: 1; }
	.as-hname { font: 13px var(--font-sans); fill: var(--as-label, #e8e4da); }
	.as-cn { font: 600 20px var(--font-text, var(--font-sans)); fill: var(--as-title, #f2eee4); }
	.as-cs { font: 10px var(--font-sans); fill: var(--as-cap, #a7a49b); }
	.as-empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 10px; color: var(--text-faint, #9ca3af); }
	.as-empty p { font-size: 13px; }
</style>
