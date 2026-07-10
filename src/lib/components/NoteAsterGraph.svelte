<script lang="ts">
	/**
	 * PJ-068 v2 — "The Aster" note-graph lens (flagship).
	 *
	 * The open note is a home-star at the centre; its links bloom into a RELATIONSHIP ROSE —
	 * one petal per typed relationship (backlinks-left / outgoing-right), petal width ∝ count,
	 * filaments = the individual links (heaviest longest), the soft glow = the aggregate. Around
	 * it, a quiet cognitive HUD surfaces EVERY bit available about the note — organized by the
	 * four Cognitive-Engine questions: Development · Content/Altitude · Origin · Connection.
	 * Read-only: hover a thread to name it, click to open it in the MAIN window. Relationship
	 * colours are CSS variables (--rel-*) for the Style Setter. Pure presentational.
	 */
	import { detectDir } from '$lib/utils';
	import { parseFrontmatter } from '$lib/libraries/store';

	let {
		noteName = '',
		content = '',
		review = null as any,
		backlinks = [] as any[],
		outgoing = [] as any[],
		resolveTarget,
		onNavigate,
	}: {
		noteName?: string;
		content?: string;
		review?: any | null;
		backlinks?: any[];
		outgoing?: any[];
		resolveTarget?: (name: string) => { path: string; libraryName: string };
		onNavigate?: (path: string, name: string, libraryName: string) => void;
	} = $props();

	const REL_ORDER = ['supports', 'exemplifies', 'generalizes', 'causes', 'derives-from', 'part-of', 'supersedes', 'contradicts', 'associative'];
	const REL_DEFAULT: Record<string, string> = {
		supports: '#879A39', exemplifies: '#3AA99F', generalizes: '#4385BE', causes: '#DA702C',
		'derives-from': '#8B7EC8', 'part-of': '#D0A215', supersedes: '#CE5D97', contradicts: '#D14D41', associative: '#B7B5AC',
	};
	const relColor = (t: string) => `var(--rel-${t}, ${REL_DEFAULT[t] || REL_DEFAULT.associative})`;
	const TIERW: Record<string, number> = { 'load-bearing': 1, established: 0.72, emerging: 0.46, stale: 0.24 };
	const tierW = (t?: string) => TIERW[(t || 'emerging').toLowerCase()] ?? 0.46;
	const clean = (n: string) => (n || '').replace(/\.md$/, '');

	const CX = 450, CY = 320, RI = 54, RMAX = 238;
	const P = (r: number, d: number): [number, number] => [CX + r * Math.cos((d * Math.PI) / 180), CY + r * Math.sin((d * Math.PI) / 180)];

	function buildSide(items: any[], side: 'left' | 'right') {
		const groups: Record<string, any[]> = {};
		for (const it of items) { const t = (it.linkType || 'associative').toLowerCase(); (groups[t] = groups[t] || []).push(it); }
		const types = REL_ORDER.filter((t) => groups[t]);
		if (!types.length) return [];
		const a0deg = side === 'left' ? 96 : -84, a1deg = side === 'left' ? 264 : 84;
		const span = a1deg - a0deg, gap = 4;
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
			const [ix0, iy0] = P(RI, pa0), [ix1, iy1] = P(RI, pa1), [ox1, oy1] = P(maxLen + 6, pa1), [ox0, oy0] = P(maxLen + 6, pa0);
			const wedge = `M${ix0} ${iy0} A${RI} ${RI} 0 0 1 ${ix1} ${iy1} L${ox1} ${oy1} A${maxLen + 6} ${maxLen + 6} 0 0 0 ${ox0} ${oy0} Z`;
			const [lx, ly] = P(maxLen + 16, (pa0 + pa1) / 2);
			return { type: t, side, count: n, color: relColor(t), wedge, fil, labelX: lx, labelY: ly, ember: t === 'contradicts' };
		});
	}

	let petals = $derived([...buildSide(backlinks, 'left'), ...buildSide(outgoing, 'right')]);
	let hasAny = $derived(backlinks.length > 0 || outgoing.length > 0);
	let hovered = $state<{ px: number; k: number } | null>(null);
	let hoverNode = $derived.by(() => {
		if (!hovered) return null;
		const p = petals[hovered.px]; if (!p) return null;
		const f = p.fil[hovered.k]; if (!f) return null;
		return { ...f, color: p.color, side: p.side };
	});
	function go(f: { path: string; name: string; lib: string }) { if (f.path) onNavigate?.(f.path, f.name, f.lib); }

	// ── Note statistics — every bit available, from frontmatter + review + the links ──
	let fm = $derived(parseFrontmatter(content || ''));
	const propOf = (key: string) => fm.properties.find((p: any) => p.key.toLowerCase() === key.toLowerCase())?.value;
	let stage = $derived(String(propOf('stage') ?? ''));
	let stratum = $derived(String(propOf('stratum') ?? ''));
	let provenance = $derived(String(propOf('provenance') ?? ''));
	let source = $derived.by(() => { const s = propOf('source') ?? propOf('sources'); return Array.isArray(s) ? s.join(', ') : String(s ?? ''); });
	let tags = $derived.by(() => { const t = propOf('tags'); return (Array.isArray(t) ? t : (t ? [String(t)] : [])) as string[]; });
	let created = $derived.by(() => {
		const cid = String(propOf('cid_cn') ?? '');
		if (/^\d{8}T/.test(cid)) return cid.slice(0, 4) + '-' + cid.slice(4, 6) + '-' + cid.slice(6, 8);
		const d = propOf('created') ?? propOf('date'); return d ? String(d).slice(0, 10) : '';
	});
	let maturity = $derived(String(review?.maturity ?? ''));
	let wordCount = $derived((review?.word_count ?? null) as number | null);
	let reviewLine = $derived.by(() => {
		if (!review) return '';
		if (review.is_stale) return 'stale';
		if (review.never_reviewed) return 'never reviewed';
		if ((review.days_overdue ?? 0) > 0) return 'due · ' + review.days_overdue + 'd';
		return 'up to date';
	});
	let allLinks = $derived([...backlinks, ...outgoing]);
	const CONF = ['hypothesis', 'evidence', 'established', 'contested'];
	let dominantConf = $derived.by(() => {
		if (!allLinks.length) return '';
		const avg = allLinks.reduce((s, l) => s + Math.max(0, CONF.indexOf(l.confidence)), 0) / allLinks.length;
		return CONF[Math.round(avg)] ?? 'hypothesis';
	});
	let tensions = $derived(allLinks.filter((l) => String(l.linkType || '').toLowerCase() === 'contradicts').length);
	let loadBearing = $derived(allLinks.filter((l) => l.tier === 'load-bearing').length);
	let dormant = $derived(allLinks.filter((l) => l.tier === 'stale').length);
</script>

<div class="as">
	<svg viewBox="0 0 900 640" preserveAspectRatio="xMidYMid meet" role="img"
		aria-label="Relationship rose — {backlinks.length} backlinks, {outgoing.length} outgoing">
		<defs>
			<radialGradient id="as-field" cx="50%" cy="49%" r="62%"><stop offset="0%" stop-color="var(--as-field-in, #16181f)"/><stop offset="100%" stop-color="var(--as-field-out, #0e0f14)"/></radialGradient>
			<radialGradient id="as-home" cx="50%" cy="50%" r="50%"><stop offset="0%" stop-color="var(--as-home, #F4EEE2)" stop-opacity="0.42"/><stop offset="55%" stop-color="var(--as-home, #F4EEE2)" stop-opacity="0.09"/><stop offset="100%" stop-color="var(--as-home, #F4EEE2)" stop-opacity="0"/></radialGradient>
			{#each petals as p, pi}
				<radialGradient id="as-pg{pi}" gradientUnits="userSpaceOnUse" cx={CX} cy={CY} r={RMAX}>
					<stop offset={RI / RMAX} stop-color={p.color} stop-opacity={p.ember ? 0.32 : 0.2}/>
					<stop offset="100%" stop-color={p.color} stop-opacity="0"/>
				</radialGradient>
			{/each}
		</defs>
		<rect x="0" y="0" width="900" height="640" fill="url(#as-field)"/>
		<circle cx={CX} cy={CY} r={RMAX} fill="none" stroke="var(--as-ring, #B7B5AC)" stroke-opacity="0.07"/>
		{#if hasAny}
			<text class="as-side" x="150" y="30">{backlinks.length} backlinks — what points here</text>
			<text class="as-side" x="750" y="30" text-anchor="end">{outgoing.length} outgoing — where it points</text>
		{/if}

		{#each petals as p, pi}
			<path d={p.wedge} fill="url(#as-pg{pi})"/>
			{#if p.ember}<path d={p.wedge} fill="none" stroke={p.color} stroke-opacity="0.3" stroke-width="1.5"/>{/if}
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

		<circle cx={CX} cy={CY} r="92" fill="url(#as-home)"/>
		{#if !hasAny}<circle cx={CX} cy={CY} r="3.5" fill="var(--as-home, #F4EEE2)" opacity="0.8"/>{/if}
		<text class="as-cn" x={CX} y={CY - 1} text-anchor="middle">{clean(noteName)}</text>
		<text class="as-cs" x={CX} y={CY + 17} text-anchor="middle">{outgoing.length} out · {backlinks.length} in</text>
	</svg>

	<!-- cognitive HUD — every bit about the note, by the four questions -->
	<div class="as-hud as-tl">
		<div class="as-hh">development</div>
		{#if stage}<div class="as-row"><span class="as-k">stage</span><span class="as-v">{stage}</span></div>{/if}
		{#if maturity}<div class="as-row"><span class="as-k">maturity</span><span class="as-v">{maturity}</span></div>{/if}
		{#if reviewLine}<div class="as-row"><span class="as-k">review</span><span class="as-v" class:as-warn={review?.is_stale || (review?.days_overdue ?? 0) > 0}>{reviewLine}</span></div>{/if}
	</div>
	<div class="as-hud as-tr">
		<div class="as-hh">content</div>
		{#if wordCount != null}<div class="as-row"><span class="as-k">words</span><span class="as-v">{wordCount.toLocaleString()}</span></div>{/if}
		{#if stratum}<div class="as-row"><span class="as-k">stratum</span><span class="as-v">{stratum}</span></div>{/if}
		{#if tags.length}<div class="as-row"><span class="as-k">tags</span><span class="as-v" dir="auto">{tags.slice(0, 4).join(', ')}{tags.length > 4 ? ' +' + (tags.length - 4) : ''}</span></div>{/if}
	</div>
	<div class="as-hud as-bl">
		<div class="as-hh">origin</div>
		{#if provenance}<div class="as-row"><span class="as-k">provenance</span><span class="as-v">{provenance}</span></div>{/if}
		{#if source}<div class="as-row"><span class="as-k">source</span><span class="as-v" dir="auto">{source}</span></div>{/if}
		{#if created}<div class="as-row"><span class="as-k">created</span><span class="as-v">{created}</span></div>{/if}
	</div>
	<div class="as-hud as-br">
		<div class="as-hh">connections</div>
		<div class="as-row"><span class="as-k">links</span><span class="as-v">{outgoing.length} out · {backlinks.length} in</span></div>
		{#if dominantConf}<div class="as-row"><span class="as-k">confidence</span><span class="as-v">{dominantConf}</span></div>{/if}
		{#if tensions}<div class="as-row"><span class="as-k as-warn">tensions</span><span class="as-v as-warn">{tensions}</span></div>{/if}
		{#if loadBearing}<div class="as-row"><span class="as-k">load-bearing</span><span class="as-v">{loadBearing}</span></div>{/if}
		{#if dormant}<div class="as-row"><span class="as-k">dormant</span><span class="as-v">{dormant}</span></div>{/if}
	</div>
</div>

<style>
	.as { width: 100%; height: 100%; min-height: 0; position: relative; }
	.as svg { width: 100%; height: 100%; display: block; }
	.as-side { font: 500 12px var(--font-sans); fill: var(--as-cap, #8a8880); }
	.as-plabel { font: 500 10px var(--font-sans); text-transform: lowercase; letter-spacing: 0.03em; }
	.as-fil { cursor: pointer; transition: opacity 0.12s; }
	.as-fil.as-off { opacity: 0.28; }
	.as-fil:hover circle { stroke: var(--as-home, #F4EEE2); stroke-width: 1; }
	.as-hname { font: 13px var(--font-sans); fill: var(--as-label, #e8e4da); }
	.as-cn { font: 600 20px var(--font-text, var(--font-sans)); fill: var(--as-title, #f2eee4); }
	.as-cs { font: 10px var(--font-sans); fill: var(--as-cap, #a7a49b); }
	.as-hud { position: absolute; display: flex; flex-direction: column; gap: 3px; max-width: 30%; pointer-events: none; }
	.as-tl { top: 16px; left: 18px; }
	.as-tr { top: 16px; right: 18px; text-align: right; }
	.as-bl { bottom: 16px; left: 18px; }
	.as-br { bottom: 16px; right: 18px; text-align: right; }
	.as-hh { font-size: 10px; letter-spacing: 0.08em; text-transform: uppercase; color: var(--as-hud-h, #6f6d66); margin-bottom: 3px; }
	.as-tr .as-row, .as-br .as-row { justify-content: flex-end; }
	.as-row { display: flex; gap: 8px; font-size: 12px; align-items: baseline; }
	.as-k { color: var(--as-cap, #8a8880); }
	.as-v { color: var(--as-label, #d8d4ca); text-transform: capitalize; max-width: 190px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.as-warn { color: var(--rel-contradicts, #D14D41); }
</style>
