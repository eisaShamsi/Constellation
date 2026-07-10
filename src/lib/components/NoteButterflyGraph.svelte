<script lang="ts">
	/**
	 * PJ-068 v2 — "The Butterfly" note-graph lens.
	 *
	 * The living bloom, finally parted: the open note sits in a central spine; its links bloom
	 * into TWO facing wings that open AWAY from each other across an empty gutter — BACKLINKS on
	 * the left (what points here), OUTGOING on the right (where it points). Each wing is ≤150°
	 * with the top and bottom left open → an hourglass, never a disc. One petal per typed
	 * relationship (width ∝ count, reach ∝ count on a shared scale so the wings are honestly
	 * asymmetric); filaments = individual links (heaviest reach the petal edge). Read-only: hover
	 * names a link, click opens it in the MAIN window. Theme-aware; --rel-* for the Style Setter.
	 */
	import { t } from '$lib/i18n';
	import { groupByType, relColor, tierW, clean, REL_ORDER, deriveStats } from '$lib/cockpitGraphData';
	import NoteGaugeDeck from './NoteGaugeDeck.svelte';

	let { noteName = '', content = '', review = null as any, backlinks = [] as any[], outgoing = [] as any[], resolveTarget, onNavigate }: {
		noteName?: string; content?: string; review?: any; backlinks?: any[]; outgoing?: any[];
		resolveTarget?: (name: string) => { path: string; libraryName: string };
		onNavigate?: (path: string, name: string, libraryName: string) => void;
	} = $props();

	const L = (k: string, fb: string) => { const v = $t(k); return v === k ? fb : v; };

	const CY = 280, GUT = 36;            // gutter half-width — the empty channel between the wings
	const GL = 450 - GUT, GR = 450 + GUT; // left/right wing origins (the inner cliffs)
	const RI = 30, RMAX = 244;
	const P = (ox: number, r: number, d: number): [number, number] => [ox + r * Math.cos((d * Math.PI) / 180), CY + r * Math.sin((d * Math.PI) / 180)];

	function buildFan(items: any[], side: 'left' | 'right', gmax: number) {
		const groups = groupByType(items);
		const types = REL_ORDER.filter((tp) => groups[tp]);
		if (!types.length) return [];
		const ox = side === 'left' ? GL : GR;
		// left wing 108°→252° (opens left), right wing −72°→72° (opens right) — top & bottom open.
		const a0 = side === 'left' ? 108 : -72, a1 = side === 'left' ? 252 : 72;
		const span = a1 - a0, gap = 4;
		const w = types.map((tp) => Math.sqrt(groups[tp].length));
		const tw = w.reduce((a, b) => a + b, 0) || 1;
		const usable = span - gap * (types.length - 1);
		let cur = a0;
		return types.map((tp) => {
			const links = groups[tp].slice().sort((a, b) => tierW(b.tier) - tierW(a.tier));
			const n = links.length;
			const width = Math.max(7, (usable * Math.sqrt(n)) / tw);
			const pa0 = cur, pa1 = cur + width; cur = pa1 + gap;
			const pad = Math.min(2.5, width * 0.14);
			const R = RI + (RMAX - RI) * Math.sqrt(n / gmax);   // reach ∝ count (shared scale)
			const fil = links.map((lk, k) => {
				const ang = n <= 1 ? (pa0 + pa1) / 2 : pa0 + pad + ((k + 0.5) / n) * (width - 2 * pad);
				const len = RI + (R - RI) * (0.45 + 0.55 * tierW(lk.tier));
				const isOut = side === 'right';
				const raw = isOut ? lk.target : lk.name;
				const res = isOut ? (resolveTarget?.(lk.target ?? '') ?? { path: '', libraryName: lk.libraryName }) : { path: lk.path, libraryName: lk.libraryName };
				const [bx, by] = P(ox, RI, ang), [tx, ty] = P(ox, len, ang);
				return { bx, by, tx, ty, name: clean(raw), path: res.path, lib: res.libraryName || lk.libraryName, tier: (lk.tier || 'emerging') };
			});
			const [ix0, iy0] = P(ox, RI, pa0), [ix1, iy1] = P(ox, RI, pa1), [ox1, oy1] = P(ox, R, pa1), [ox0, oy0] = P(ox, R, pa0);
			const large = pa1 - pa0 > 180 ? 1 : 0;
			const wedge = `M${ix0} ${iy0} A${RI} ${RI} 0 0 1 ${ix1} ${iy1} L${ox1} ${oy1} A${R} ${R} 0 ${large} 0 ${ox0} ${oy0} Z`;
			const [lx, ly] = P(ox, R + 14, (pa0 + pa1) / 2);
			return { type: tp, side, count: n, color: relColor(tp), wedge, fil, labelX: lx, labelY: ly, ember: tp === 'contradicts' };
		});
	}

	let petals = $derived.by(() => {
		const gl = groupByType(backlinks), gr = groupByType(outgoing);
		const gmax = Math.max(1, ...Object.values(gl).map((a) => a.length), ...Object.values(gr).map((a) => a.length));
		return [...buildFan(backlinks, 'left', gmax), ...buildFan(outgoing, 'right', gmax)];
	});
	let hasAny = $derived(backlinks.length > 0 || outgoing.length > 0);
	let stats = $derived(deriveStats(content, review, backlinks, outgoing));
	let hovered = $state<{ pi: number; k: number } | null>(null);
	let hoverNode = $derived.by(() => {
		if (!hovered) return null; const p = petals[hovered.pi]; if (!p) return null;
		const f = p.fil[hovered.k]; if (!f) return null; return { ...f, color: p.color, side: p.side };
	});
	function go(f: { path?: string; name: string; lib?: string }) { if (f.path) onNavigate?.(f.path, f.name, f.lib || ''); }
</script>

<div class="bf">
	<svg class="bf-svg" viewBox="0 0 900 560" preserveAspectRatio="xMidYMid meet" role="img"
		aria-label="Butterfly — {backlinks.length} backlinks left, {outgoing.length} outgoing right">
		<defs>
			<radialGradient id="bf-field" cx="50%" cy="50%" r="60%"><stop offset="0%" stop-color="var(--background-secondary, #f6f6f7)"/><stop offset="100%" stop-color="var(--background-primary, #fff)"/></radialGradient>
			{#each petals as p, pi}
				<radialGradient id="bf-pg{pi}" gradientUnits="userSpaceOnUse" cx={p.side === 'left' ? GL : GR} cy={CY} r={RMAX}>
					<stop offset={RI / RMAX} stop-color={p.color} stop-opacity={p.ember ? 0.34 : 0.22}/>
					<stop offset="100%" stop-color={p.color} stop-opacity="0"/>
				</radialGradient>
			{/each}
		</defs>
		<rect x="0" y="0" width="900" height="560" fill="url(#bf-field)"/>

		<text class="bf-side" x={GL - 8} y="34" text-anchor="end">◀ {L('cockpit.incoming', 'incoming')} · {backlinks.length}</text>
		<text class="bf-side" x={GR + 8} y="34">{L('cockpit.outgoing', 'outgoing')} · {outgoing.length} ▶</text>

		{#each petals as p, pi}
			<path d={p.wedge} fill="url(#bf-pg{pi})"/>
			{#if p.ember}<path d={p.wedge} fill="none" stroke={p.color} stroke-opacity="0.3" stroke-width="1.4"/>{/if}
			{#each p.fil as f, k}
				<g class="bf-fil" class:bf-off={hovered && !(hovered.pi === pi && hovered.k === k)}
					role="button" tabindex={f.path ? 0 : -1} aria-label="{p.type}: {f.name}"
					onmouseenter={() => hovered = { pi, k }} onmouseleave={() => { if (hovered?.pi === pi && hovered?.k === k) hovered = null; }}
					onfocus={() => hovered = { pi, k }} onblur={() => { if (hovered?.pi === pi && hovered?.k === k) hovered = null; }}
					onclick={() => go(f)} onkeydown={(e) => { if (e.key === 'Enter') go(f); }}>
					<line x1={f.bx} y1={f.by} x2={f.tx} y2={f.ty} stroke={p.color} stroke-opacity={f.tier === 'stale' ? 0.28 : 0.58} stroke-width={f.tier === 'load-bearing' ? 1.9 : 1}/>
					<circle cx={f.tx} cy={f.ty} r={f.tier === 'load-bearing' ? 3.1 : f.tier === 'stale' ? 1.7 : 2.3} fill={p.color} opacity={f.tier === 'stale' ? 0.5 : 1}/>
				</g>
			{/each}
			<text class="bf-plabel" x={p.labelX} y={p.labelY} text-anchor={p.side === 'left' ? 'end' : 'start'} fill={p.color} opacity="0.82">{p.type} · {p.count}</text>
		{/each}

		{#if hoverNode}
			<g pointer-events="none">
				<circle cx={hoverNode.tx} cy={hoverNode.ty} r="4.5" fill={hoverNode.color}/>
				<text class="bf-hname" x={hoverNode.side === 'left' ? hoverNode.tx - 9 : hoverNode.tx + 9} y={hoverNode.ty + 4}
					text-anchor={hoverNode.side === 'left' ? 'end' : 'start'}>{hoverNode.name}</text>
			</g>
		{/if}

		<!-- the central spine: the note + its conviction ring, sitting in the gutter -->
		<rect x={GL - 4} y={CY - 34} width={GR - GL + 8} height="68" rx="12" fill="var(--background-primary, #fff)" stroke="var(--background-modifier-border, #d4d4d8)"/>
		<path d="M{450 - 30} {CY - 26} A30 30 0 0 1 {450 + 30} {CY - 26}" fill="none"
			stroke={stats.dominantConf ? relColor('supports') : 'var(--background-modifier-border, #d4d4d8)'} stroke-opacity="0.5" stroke-width="2.5"/>
		{#if !hasAny}<circle cx="450" cy={CY} r="3.5" fill="var(--interactive-accent, #7c3aed)" opacity="0.7"/>{/if}
		<text class="bf-cn" x="450" y={CY - 2} text-anchor="middle">{clean(noteName)}</text>
		<text class="bf-cs" x="450" y={CY + 16} text-anchor="middle">{stats.totalLinks}</text>
	</svg>

	<NoteGaugeDeck {content} {review} {backlinks} {outgoing} />
</div>

<style>
	.bf { display: flex; flex-direction: column; width: 100%; height: 100%; min-height: 0; }
	.bf-svg { flex: 1; min-height: 0; width: 100%; display: block; }
	.bf-side { font: 500 12px var(--font-sans); fill: var(--text-muted, #6b7280); }
	.bf-plabel { font: 500 11px var(--font-sans); text-transform: lowercase; letter-spacing: 0.02em; }
	.bf-fil { cursor: pointer; transition: opacity 0.12s; }
	.bf-fil.bf-off { opacity: 0.24; }
	.bf-fil:hover circle { stroke: var(--text-normal, #1a1a1a); stroke-width: 1; }
	.bf-hname { font: 13px var(--font-sans); fill: var(--text-normal, #1a1a1a); }
	.bf-cn { font: 600 18px var(--font-text, var(--font-sans)); fill: var(--text-normal, #1a1a1a); }
	.bf-cs { font: 10px var(--font-sans); fill: var(--text-muted, #6b7280); }
</style>
