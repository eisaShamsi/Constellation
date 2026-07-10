<script lang="ts">
	/**
	 * PJ-068 v2 — "The Butterfly" note-graph lens.
	 *
	 * The living bloom, parted: the open note is a plain title box in a central spine; its links
	 * bloom into TWO facing wings that open AWAY from each other across an empty gutter —
	 * BACKLINKS left (what points here), OUTGOING right (where it points). Each wing spans ≤144°
	 * with the top and bottom left open (an hourglass, never a disc) and is stretched into an
	 * ELLIPSE so it fills the stage's full width instead of squeezing into a circle bounded by
	 * the shorter axis. One petal per typed relationship: angular width ∝ √count, reach ∝ √count
	 * on a scale shared by both wings (so the wings are honestly asymmetric). Filaments = the
	 * individual links, length ∝ their earned weight (tier + traversals), dot size adapted to the
	 * petal's density so nodes stay separated. Type labels live in reserved outer LEDGER COLUMNS
	 * with a collision pass, so they can never overlap the wings or each other.
	 *
	 * Read-only: hover names a link, click opens it in the MAIN window. Theme-aware; --rel-* for
	 * the Style Setter.
	 */
	import { t } from '$lib/i18n';
	import { groupByType, relColor, tierW, clean, REL_ORDER } from '$lib/cockpitGraphData';
	import NoteGaugeDeck from './NoteGaugeDeck.svelte';

	let { noteName = '', content = '', review = null as any, backlinks = [] as any[], outgoing = [] as any[], resolveTarget, onNavigate }: {
		noteName?: string; content?: string; review?: any; backlinks?: any[]; outgoing?: any[];
		resolveTarget?: (name: string) => { path: string; libraryName: string };
		onNavigate?: (path: string, name: string, libraryName: string) => void;
	} = $props();

	const L = (k: string, fb: string) => { const v = $t(k); return v === k ? fb : v; };

	const A_HALF = 72, U_IN = 0.06, LABELW = 152, BOXH = 44, LABEL_GAP = 22;
	const SIN_MAX = Math.sin((A_HALF * Math.PI) / 180);

	let W = $state(0), H = $state(0);
	let title = $derived.by(() => { const n = clean(noteName); return n.length > 30 ? n.slice(0, 29) + '…' : n; });

	let geo = $derived.by(() => {
		const cx = W / 2, cy = H / 2 + 2;
		const boxW = Math.min(280, Math.max(120, title.length * 8.6 + 30));
		const gut = boxW / 2 + 12;
		const RX = Math.max(60, cx - gut - LABELW - 14);
		const RY = Math.max(50, (H / 2 - 36) / SIN_MAX);
		return { cx, cy, boxW, gut, RX, RY, lx: LABELW + 2, rx: W - LABELW - 2 };
	});

	const P = (ox: number, u: number, d: number, g: typeof geo): [number, number] =>
		[ox + u * g.RX * Math.cos((d * Math.PI) / 180), g.cy + u * g.RY * Math.sin((d * Math.PI) / 180)];

	/** Evict labels into their column, then push them apart so none can overlap. */
	function decollide(ys: number[], top: number, bottom: number): number[] {
		const idx = ys.map((y, i) => ({ i, y })).sort((a, b) => a.y - b.y);
		let prev = -Infinity;
		for (const it of idx) { it.y = Math.max(it.y, prev + LABEL_GAP); prev = it.y; }
		let next = Infinity;
		for (let j = idx.length - 1; j >= 0; j--) { idx[j].y = Math.min(idx[j].y, next - LABEL_GAP); next = idx[j].y; }
		const out = new Array(ys.length);
		for (const it of idx) out[it.i] = Math.min(bottom, Math.max(top, it.y));
		return out;
	}

	/** Order a wing's petals CENTRE-OUT by size: the largest relationship points straight out
	 *  along the wing's horizontal axis, the rest fan alternately above and below it. Laying them
	 *  out sequentially from one edge instead (canonical order) drops the biggest petal into a
	 *  corner, pushing all the mass into one quadrant — which reads as a tilted lens. */
	function centreOut(types: string[], count: (t: string) => number, side: 'left' | 'right') {
		const desc = types.slice().sort((a, b) => count(b) - count(a));
		const above: string[] = [], below: string[] = [];
		desc.forEach((tp, i) => { if (i > 0) (i % 2 === 1 ? above : below).push(tp); });
		const topDown = [...above.reverse(), desc[0], ...below];
		// the left fan's angles run bottom→top, so its sequence is reversed to mirror the right
		return side === 'left' ? topDown.reverse() : topDown;
	}

	function buildFan(items: any[], side: 'left' | 'right', gmax: number, maxTrav: number, g: typeof geo) {
		const groups = groupByType(items);
		const present = REL_ORDER.filter((tp) => groups[tp]);
		if (!present.length) return [];
		const types = centreOut(present as unknown as string[], (tp) => groups[tp].length, side);
		const ox = side === 'left' ? g.cx - g.gut : g.cx + g.gut;
		const a0 = side === 'left' ? 180 - A_HALF : -A_HALF;
		const a1 = side === 'left' ? 180 + A_HALF : A_HALF;
		const gap = 4;
		const w = types.map((tp) => Math.sqrt(groups[tp].length));
		const tw = w.reduce((a, b) => a + b, 0) || 1;
		const usable = a1 - a0 - gap * (types.length - 1);
		let cur = a0;
		const petals = types.map((tp, ti) => {
			const links = groups[tp].slice().sort((a, b) => tierW(b.tier) - tierW(a.tier));
			const n = links.length;
			const width = Math.max(6, (usable * w[ti]) / tw);
			const pa0 = cur, pa1 = cur + width; cur = pa1 + gap;
			const pad = Math.min(2.2, width * 0.12);
			const uOut = U_IN + (1 - U_IN) * Math.sqrt(n / gmax);
			// nodes separated: space them by the petal's own arc length, not a fixed dot size
			const arc = ((width * Math.PI) / 180) * uOut * g.RX;
			const dotR = Math.max(0.9, Math.min(3.2, (arc / Math.max(1, n)) * 0.42));
			const strokeW = Math.max(0.5, Math.min(1.9, dotR * 0.7));
			const fil = links.map((lk, k) => {
				const ang = n <= 1 ? (pa0 + pa1) / 2 : pa0 + pad + ((k + 0.5) / n) * (width - 2 * pad);
				const trav = maxTrav > 0 ? Math.min(1, (lk.traversalCount ?? 0) / maxTrav) : 0;
				const wgt = 0.55 * tierW(lk.tier) + 0.45 * trav;             // length = earned weight
				const uLen = U_IN + (uOut - U_IN) * (0.42 + 0.58 * wgt);
				const isOut = side === 'right';
				const res = isOut ? (resolveTarget?.(lk.target ?? '') ?? { path: '', libraryName: lk.libraryName }) : { path: lk.path, libraryName: lk.libraryName };
				const [bx, by] = P(ox, U_IN, ang, g), [tx, ty] = P(ox, uLen, ang, g);
				return { bx, by, tx, ty, name: clean(isOut ? lk.target : lk.name), path: res.path, lib: res.libraryName || lk.libraryName, tier: (lk.tier || 'emerging') };
			});
			const [ix0, iy0] = P(ox, U_IN, pa0, g), [ix1, iy1] = P(ox, U_IN, pa1, g);
			const [ex1, ey1] = P(ox, uOut, pa1, g), [ex0, ey0] = P(ox, uOut, pa0, g);
			const irx = U_IN * g.RX, iry = U_IN * g.RY, orx = uOut * g.RX, ory = uOut * g.RY;
			const wedge = `M${ix0} ${iy0} A${irx} ${iry} 0 0 1 ${ix1} ${iy1} L${ex1} ${ey1} A${orx} ${ory} 0 0 0 ${ex0} ${ey0} Z`;
			const [tipX, tipY] = P(ox, uOut, (pa0 + pa1) / 2, g);
			return { type: tp, side, count: n, color: relColor(tp), wedge, fil, dotR, strokeW, tipX, tipY, labelY: tipY, ember: tp === 'contradicts' };
		});
		const ys = decollide(petals.map((p) => p.tipY), 26, H - 26);
		petals.forEach((p, i) => (p.labelY = ys[i]));
		return petals;
	}

	let petals = $derived.by(() => {
		if (W < 80 || H < 80) return [];
		const gl = groupByType(backlinks), gr = groupByType(outgoing);
		const gmax = Math.max(1, ...Object.values(gl).map((a) => a.length), ...Object.values(gr).map((a) => a.length));
		const maxTrav = Math.max(0, ...[...backlinks, ...outgoing].map((l) => l.traversalCount ?? 0));
		return [...buildFan(backlinks, 'left', gmax, maxTrav, geo), ...buildFan(outgoing, 'right', gmax, maxTrav, geo)];
	});
	let hasAny = $derived(backlinks.length > 0 || outgoing.length > 0);
	let hovered = $state<{ pi: number; k: number } | null>(null);
	let hoverNode = $derived.by(() => {
		if (!hovered) return null; const p = petals[hovered.pi]; if (!p) return null;
		const f = p.fil[hovered.k]; if (!f) return null; return { ...f, color: p.color, side: p.side };
	});
	function go(f: { path?: string; name: string; lib?: string }) { if (f.path) onNavigate?.(f.path, f.name, f.lib || ''); }
</script>

<div class="bf">
	<div class="bf-stage" bind:clientWidth={W} bind:clientHeight={H}>
		{#if W > 80 && H > 80}
			<svg class="bf-svg" viewBox="0 0 {W} {H}" role="img"
				aria-label="Butterfly — {backlinks.length} backlinks left, {outgoing.length} outgoing right">
				<defs>
					{#each petals as p, pi}
						<radialGradient id="bf-pg{pi}" gradientUnits="userSpaceOnUse"
							cx={p.side === 'left' ? geo.cx - geo.gut : geo.cx + geo.gut} cy={geo.cy} r={geo.RX}
							gradientTransform="translate(0 {geo.cy}) scale(1 {geo.RY / geo.RX}) translate(0 {-geo.cy})">
							<stop offset={U_IN} stop-color={p.color} stop-opacity={p.ember ? 0.32 : 0.2}/>
							<stop offset="100%" stop-color={p.color} stop-opacity="0"/>
						</radialGradient>
					{/each}
				</defs>

				<!-- the faint dotted seam that parts backlinks (left) from outgoing (right) -->
				<line x1={geo.cx} y1="10" x2={geo.cx} y2={geo.cy - BOXH / 2 - 8} stroke="var(--background-modifier-border, #d4d4d8)" stroke-opacity="0.75" stroke-dasharray="1 5"/>
				<line x1={geo.cx} y1={geo.cy + BOXH / 2 + 8} x2={geo.cx} y2={H - 10} stroke="var(--background-modifier-border, #d4d4d8)" stroke-opacity="0.75" stroke-dasharray="1 5"/>

				<text class="bf-side" x={geo.cx - geo.gut - 10} y="24" text-anchor="end">◀ {L('cockpit.incoming', 'incoming')} · {backlinks.length}</text>
				<text class="bf-side" x={geo.cx + geo.gut + 10} y="24">{L('cockpit.outgoing', 'outgoing')} · {outgoing.length} ▶</text>

				{#each petals as p, pi}
					<path d={p.wedge} fill="url(#bf-pg{pi})"/>
					{#if p.ember}<path d={p.wedge} fill="none" stroke={p.color} stroke-opacity="0.28" stroke-width="1.3"/>{/if}
					{#each p.fil as f, k}
						<g class="bf-fil" class:bf-off={hovered && !(hovered.pi === pi && hovered.k === k)}
							role="button" tabindex={f.path ? 0 : -1} aria-label="{p.type}: {f.name}"
							onmouseenter={() => hovered = { pi, k }} onmouseleave={() => { if (hovered?.pi === pi && hovered?.k === k) hovered = null; }}
							onfocus={() => hovered = { pi, k }} onblur={() => { if (hovered?.pi === pi && hovered?.k === k) hovered = null; }}
							onclick={() => go(f)} onkeydown={(e) => { if (e.key === 'Enter') go(f); }}>
							<line x1={f.bx} y1={f.by} x2={f.tx} y2={f.ty} stroke={p.color} stroke-opacity={f.tier === 'stale' ? 0.26 : 0.55} stroke-width={f.tier === 'load-bearing' ? p.strokeW * 1.8 : p.strokeW}/>
							<circle cx={f.tx} cy={f.ty} r={f.tier === 'load-bearing' ? p.dotR * 1.4 : f.tier === 'stale' ? p.dotR * 0.7 : p.dotR} fill={p.color} opacity={f.tier === 'stale' ? 0.5 : 1}/>
						</g>
					{/each}
					<!-- type label: parked in the outer ledger column, never on the wing -->
					<line x1={p.tipX} y1={p.tipY} x2={p.side === 'left' ? geo.lx + 6 : geo.rx - 6} y2={p.labelY}
						stroke={p.color} stroke-opacity="0.28" stroke-dasharray="1 3"/>
					<text class="bf-plabel" x={p.side === 'left' ? geo.lx : geo.rx} y={p.labelY}
						text-anchor={p.side === 'left' ? 'end' : 'start'} fill={p.color}>{p.type} · {p.count}</text>
				{/each}

				{#if hoverNode}
					<g pointer-events="none">
						<circle cx={hoverNode.tx} cy={hoverNode.ty} r="4.5" fill={hoverNode.color}/>
						<text class="bf-hname" x={hoverNode.side === 'left' ? hoverNode.tx - 9 : hoverNode.tx + 9} y={hoverNode.ty + 4}
							text-anchor={hoverNode.side === 'left' ? 'end' : 'start'}>{hoverNode.name}</text>
					</g>
				{/if}

				<!-- the spine: a plain title box, nothing more -->
				<rect x={geo.cx - geo.boxW / 2} y={geo.cy - BOXH / 2} width={geo.boxW} height={BOXH} rx="10"
					fill="var(--background-primary, #fff)" stroke="var(--background-modifier-border, #d4d4d8)"/>
				{#if !hasAny}<text class="bf-cs" x={geo.cx} y={geo.cy + BOXH / 2 + 18} text-anchor="middle">{L('cockpit.noLinks', 'no links yet')}</text>{/if}
				<text class="bf-cn" x={geo.cx} y={geo.cy + 5} text-anchor="middle">{title}</text>
			</svg>
		{/if}
	</div>

	<NoteGaugeDeck {content} {review} {backlinks} {outgoing} />
</div>

<style>
	.bf { display: flex; flex-direction: column; width: 100%; height: 100%; min-height: 0; }
	.bf-stage { flex: 1; min-height: 0; width: 100%; background: var(--background-primary, #fff); }
	.bf-svg { width: 100%; height: 100%; display: block; }
	.bf-side { font: 500 12px var(--font-sans); fill: var(--text-muted, #6b7280); }
	.bf-plabel { font: 500 12px var(--font-sans); text-transform: lowercase; letter-spacing: 0.02em; dominant-baseline: middle; }
	.bf-fil { cursor: pointer; transition: opacity 0.12s; }
	.bf-fil.bf-off { opacity: 0.24; }
	.bf-fil:hover circle { stroke: var(--text-normal, #1a1a1a); stroke-width: 1; }
	.bf-hname { font: 13px var(--font-sans); fill: var(--text-normal, #1a1a1a); }
	.bf-cn { font: 600 15px var(--font-text, var(--font-sans)); fill: var(--text-normal, #1a1a1a); }
	.bf-cs { font: 12px var(--font-sans); fill: var(--text-muted, #6b7280); }
</style>
