<script lang="ts">
	/**
	 * PJ-068 v2 — "The Aster" note-graph lens (flagship).
	 *
	 * The open note is a home-star; its links bloom into a RELATIONSHIP ROSE — one petal per
	 * typed relationship. Backlinks bloom on the LEFT, outgoing on the RIGHT, as two distinct
	 * blooms parted by clear seams + a divider (not one uniform ring). Petal width ∝ count,
	 * filaments = individual links (heaviest reach farthest, so the silhouette is irregular),
	 * the soft glow = the aggregate. Around it, a gauge HUD reflects the note's data by the
	 * four Cognitive-Engine questions. Theme-aware (app light/dark vars); relationship colours
	 * are Style-Setter CSS vars (--rel-*). Read-only: hover names a thread, click opens it in
	 * the MAIN window.
	 */
	import { detectDir } from '$lib/utils';
	import { parseFrontmatter } from '$lib/libraries/store';

	let {
		noteName = '', content = '', review = null as any,
		backlinks = [] as any[], outgoing = [] as any[], resolveTarget, onNavigate,
	}: {
		noteName?: string; content?: string; review?: any | null;
		backlinks?: any[]; outgoing?: any[];
		resolveTarget?: (name: string) => { path: string; libraryName: string };
		onNavigate?: (path: string, name: string, libraryName: string) => void;
	} = $props();

	const REL_ORDER = ['supports', 'exemplifies', 'generalizes', 'causes', 'derives-from', 'part-of', 'supersedes', 'contradicts', 'associative'];
	const REL_DEFAULT: Record<string, string> = {
		supports: '#879A39', exemplifies: '#3AA99F', generalizes: '#4385BE', causes: '#DA702C',
		'derives-from': '#8B7EC8', 'part-of': '#D0A215', supersedes: '#CE5D97', contradicts: '#D14D41', associative: '#B7B5AC',
	};
	const relColor = (t: string) => `var(--rel-${t}, ${REL_DEFAULT[t] || REL_DEFAULT.associative})`;
	const TIERW: Record<string, number> = { 'load-bearing': 1, established: 0.7, emerging: 0.42, stale: 0.2 };
	const tierW = (t?: string) => TIERW[(t || 'emerging').toLowerCase()] ?? 0.42;
	const clean = (n: string) => (n || '').replace(/\.md$/, '');

	const CX = 450, CY = 320, RI = 56, RMAX = 240;
	const P = (r: number, d: number): [number, number] => [CX + r * Math.cos((d * Math.PI) / 180), CY + r * Math.sin((d * Math.PI) / 180)];

	// LEFT bloom = backlinks (110°→250°); RIGHT = outgoing (−70°→70°). Wide seams at top (270°)
	// and bottom (90°) part the two so the form is two facing blooms, not a uniform circle.
	function buildSide(items: any[], side: 'left' | 'right') {
		const groups: Record<string, any[]> = {};
		for (const it of items) { const t = (it.linkType || 'associative').toLowerCase(); (groups[t] = groups[t] || []).push(it); }
		const types = REL_ORDER.filter((t) => groups[t]);
		if (!types.length) return [];
		const a0deg = side === 'left' ? 110 : -70, a1deg = side === 'left' ? 250 : 70;
		const span = a1deg - a0deg, gap = 5;
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
			// petal reach ∝ its heaviest bond → petals reach unequal lengths (broken silhouette).
			const petalW = Math.max(...links.map((l) => tierW(l.tier)), 0.2);
			let maxLen = RI;
			const fil = links.map((lk, k) => {
				const ang = n <= 1 ? (pa0 + pa1) / 2 : pa0 + pad + ((k + 0.5) / n) * (width - 2 * pad);
				const len = RI + (RMAX - RI) * (0.12 + 0.88 * tierW(lk.tier));
				if (len > maxLen) maxLen = len;
				const isOut = side === 'right';
				const rawName = isOut ? lk.target : lk.name;
				const res = isOut ? (resolveTarget?.(lk.target) ?? { path: '', libraryName: lk.libraryName }) : { path: lk.path, libraryName: lk.libraryName };
				const [bx, by] = P(RI, ang); const [tx, ty] = P(len, ang);
				return { ang, bx, by, tx, ty, name: clean(rawName), path: res.path, lib: res.libraryName || lk.libraryName, tier: (lk.tier || 'emerging') };
			});
			const [ix0, iy0] = P(RI, pa0), [ix1, iy1] = P(RI, pa1), [ox1, oy1] = P(maxLen + 6, pa1), [ox0, oy0] = P(maxLen + 6, pa0);
			const wedge = `M${ix0} ${iy0} A${RI} ${RI} 0 0 1 ${ix1} ${iy1} L${ox1} ${oy1} A${maxLen + 6} ${maxLen + 6} 0 0 0 ${ox0} ${oy0} Z`;
			const [lx, ly] = P(maxLen + 15, (pa0 + pa1) / 2);
			return { type: t, side, count: n, color: relColor(t), wedge, fil, labelX: lx, labelY: ly, ember: t === 'contradicts' };
		});
	}

	let petals = $derived([...buildSide(backlinks, 'left'), ...buildSide(outgoing, 'right')]);
	let hasAny = $derived(backlinks.length > 0 || outgoing.length > 0);
	let hovered = $state<{ px: number; k: number } | null>(null);
	let hoverNode = $derived.by(() => {
		if (!hovered) return null; const p = petals[hovered.px]; if (!p) return null;
		const f = p.fil[hovered.k]; if (!f) return null; return { ...f, color: p.color, side: p.side };
	});
	function go(f: { path: string; name: string; lib: string }) { if (f.path) onNavigate?.(f.path, f.name, f.lib); }

	// ── Note data (frontmatter + review + links) ──
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
	let reviewState = $derived.by(() => {
		if (!review) return { label: '', sev: '' };
		if (review.is_stale) return { label: 'stale', sev: 'bad' };
		if (review.never_reviewed) return { label: 'never reviewed', sev: 'mut' };
		if ((review.days_overdue ?? 0) > 0) return { label: 'due · ' + review.days_overdue + 'd', sev: 'warn' };
		return { label: 'up to date', sev: 'ok' };
	});
	let allLinks = $derived([...backlinks, ...outgoing]);
	const STAGES = ['spark', 'birth', 'growth', 'maturity', 'dormancy', 'archival'];
	const MATS = ['seed', 'sapling', 'evergreen', 'canonical'];
	let stageIdx = $derived(STAGES.indexOf(stage.toLowerCase()));
	let matIdx = $derived(MATS.indexOf(maturity.toLowerCase()));
	let typeMix = $derived.by(() => {
		const m: Record<string, number> = {};
		for (const l of allLinks) { const t = (l.linkType || 'associative').toLowerCase(); m[t] = (m[t] || 0) + 1; }
		return REL_ORDER.filter((t) => m[t]).map((t) => ({ type: t, count: m[t], color: relColor(t) }));
	});
	let supportsN = $derived(allLinks.filter((l) => (l.linkType || '').toLowerCase() === 'supports').length);
	let contradictsN = $derived(allLinks.filter((l) => (l.linkType || '').toLowerCase() === 'contradicts').length);
	const CONF = ['hypothesis', 'evidence', 'established', 'contested'];
	const CONF_COLOR = ['var(--text-faint,#9ca3af)', 'var(--rel-generalizes,#4385BE)', 'var(--rel-supports,#879A39)', 'var(--rel-contradicts,#D14D41)'];
	let confMix = $derived.by(() => CONF.map((c, i) => ({ c, n: allLinks.filter((l) => (l.confidence || 'hypothesis') === c).length, color: CONF_COLOR[i] })));
	let loadBearing = $derived(allLinks.filter((l) => l.tier === 'load-bearing').length);
</script>

<div class="as">
	<svg viewBox="0 0 900 640" preserveAspectRatio="xMidYMid meet" role="img"
		aria-label="Relationship rose — {backlinks.length} backlinks, {outgoing.length} outgoing">
		<defs>
			<radialGradient id="as-field" cx="50%" cy="49%" r="62%"><stop offset="0%" stop-color="var(--background-secondary, #f6f6f7)"/><stop offset="100%" stop-color="var(--background-primary, #fff)"/></radialGradient>
			<radialGradient id="as-home" cx="50%" cy="50%" r="50%"><stop offset="0%" stop-color="var(--interactive-accent, #7c3aed)" stop-opacity="0.22"/><stop offset="60%" stop-color="var(--interactive-accent, #7c3aed)" stop-opacity="0.05"/><stop offset="100%" stop-color="var(--interactive-accent, #7c3aed)" stop-opacity="0"/></radialGradient>
			{#each petals as p, pi}
				<radialGradient id="as-pg{pi}" gradientUnits="userSpaceOnUse" cx={CX} cy={CY} r={RMAX}>
					<stop offset={RI / RMAX} stop-color={p.color} stop-opacity={p.ember ? 0.34 : 0.22}/>
					<stop offset="100%" stop-color={p.color} stop-opacity="0"/>
				</radialGradient>
			{/each}
		</defs>
		<rect x="0" y="0" width="900" height="640" fill="url(#as-field)"/>
		<!-- the seam divider parting backlinks (left) from outgoing (right) -->
		<line x1={CX} y1="70" x2={CX} y2={CY - 96} stroke="var(--background-modifier-border, #d4d4d8)" stroke-opacity="0.6" stroke-dasharray="2 5"/>
		<line x1={CX} y1={CY + 96} x2={CX} y2="570" stroke="var(--background-modifier-border, #d4d4d8)" stroke-opacity="0.6" stroke-dasharray="2 5"/>
		<text class="as-side" x={CX - 26} y="30" text-anchor="end">← backlinks · what points here</text>
		<text class="as-side" x={CX + 26} y="30">outgoing · where it points →</text>

		{#each petals as p, pi}
			<path d={p.wedge} fill="url(#as-pg{pi})"/>
			{#if p.ember}<path d={p.wedge} fill="none" stroke={p.color} stroke-opacity="0.32" stroke-width="1.5"/>{/if}
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
			<text class="as-plabel" x={p.labelX} y={p.labelY} text-anchor="middle" fill={p.color} opacity="0.8">{p.type} · {p.count}</text>
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
		{#if !hasAny}<circle cx={CX} cy={CY} r="3.5" fill="var(--interactive-accent, #7c3aed)" opacity="0.7"/>{/if}
		<text class="as-cn" x={CX} y={CY - 1} text-anchor="middle">{clean(noteName)}</text>
		<text class="as-cs" x={CX} y={CY + 17} text-anchor="middle">{outgoing.length} out · {backlinks.length} in</text>
	</svg>

	<!-- ── gauge HUD — the note's data, by the four questions ── -->
	<div class="as-hud as-tl">
		<div class="as-hh">development</div>
		{#if stageIdx >= 0}<div class="as-g"><span class="as-gl">stage</span><span class="as-lad">{#each STAGES as _, i}<i class:on={i <= stageIdx} class:cur={i === stageIdx}></i>{/each}</span><span class="as-gv">{stage}</span></div>{/if}
		{#if matIdx >= 0}<div class="as-g"><span class="as-gl">maturity</span><span class="as-lad">{#each MATS as _, i}<i class:on={i <= matIdx} class:cur={i === matIdx}></i>{/each}</span><span class="as-gv">{maturity}</span></div>{/if}
		{#if reviewState.label}<div class="as-g"><span class="as-gl">review</span><span class="as-pill as-{reviewState.sev}">{reviewState.label}</span></div>{/if}
	</div>
	<div class="as-hud as-tr">
		<div class="as-hh">content</div>
		{#if wordCount != null}<div class="as-num">{wordCount.toLocaleString()}<span class="as-numl">words</span></div>{/if}
		{#if stratum}<div class="as-g"><span class="as-gl">stratum</span><span class="as-gv">{stratum}</span></div>{/if}
		{#if tags.length}<div class="as-chips">{#each tags.slice(0, 5) as t}<span class="as-chip" dir="auto">{t}</span>{/each}{#if tags.length > 5}<span class="as-chip as-more">+{tags.length - 5}</span>{/if}</div>{/if}
	</div>
	<div class="as-hud as-bl">
		<div class="as-hh">origin</div>
		{#if provenance}<div class="as-g"><span class="as-gl">provenance</span><span class="as-gv">{provenance}</span></div>{/if}
		{#if source}<div class="as-g"><span class="as-gl">source</span><span class="as-gv" dir="auto">{source}</span></div>{/if}
		{#if created}<div class="as-g"><span class="as-gl">created</span><span class="as-gv">{created}</span></div>{/if}
	</div>
	<div class="as-hud as-br">
		<div class="as-hh">connections · {allLinks.length}</div>
		{#if typeMix.length}
			<div class="as-mix" title="relationship mix">{#each typeMix as t}<span style="flex:{t.count};background:{t.color}"></span>{/each}</div>
		{/if}
		{#if supportsN || contradictsN}
			<div class="as-g"><span class="as-gl">balance</span>
				<span class="as-bal"><span class="as-bal-s" style="flex:{supportsN || 0.001}"></span><span class="as-bal-c" style="flex:{contradictsN || 0.001}"></span></span>
				<span class="as-gv">{supportsN}↑ {contradictsN}↓</span>
			</div>
		{/if}
		{#if allLinks.length}
			<div class="as-g"><span class="as-gl">confidence</span><span class="as-mix as-conf">{#each confMix as c}{#if c.n}<span style="flex:{c.n};background:{c.color}" title="{c.c}: {c.n}"></span>{/if}{/each}</span></div>
		{/if}
		{#if loadBearing}<div class="as-g"><span class="as-gl">load-bearing</span><span class="as-gv">{loadBearing}</span></div>{/if}
	</div>
</div>

<style>
	.as { width: 100%; height: 100%; min-height: 0; position: relative; }
	.as svg { width: 100%; height: 100%; display: block; }
	.as-side { font: 500 11px var(--font-sans); fill: var(--text-muted, #6b7280); }
	.as-plabel { font: 500 10px var(--font-sans); text-transform: lowercase; letter-spacing: 0.03em; }
	.as-fil { cursor: pointer; transition: opacity 0.12s; }
	.as-fil.as-off { opacity: 0.26; }
	.as-fil:hover circle { stroke: var(--text-normal, #1a1a1a); stroke-width: 1; }
	.as-hname { font: 13px var(--font-sans); fill: var(--text-normal, #1a1a1a); }
	.as-cn { font: 600 20px var(--font-text, var(--font-sans)); fill: var(--text-normal, #1a1a1a); }
	.as-cs { font: 10px var(--font-sans); fill: var(--text-muted, #6b7280); }

	.as-hud { position: absolute; display: flex; flex-direction: column; gap: 6px; max-width: 32%; color: var(--text-normal, #1a1a1a); }
	.as-tl { top: 16px; left: 18px; }
	.as-tr { top: 16px; right: 18px; align-items: flex-end; }
	.as-bl { bottom: 16px; left: 18px; }
	.as-br { bottom: 16px; right: 18px; align-items: flex-end; }
	.as-hh { font-size: 10px; letter-spacing: 0.08em; text-transform: uppercase; color: var(--text-faint, #9ca3af); }
	.as-g { display: flex; align-items: center; gap: 7px; font-size: 12px; }
	.as-tr .as-g, .as-br .as-g { flex-direction: row-reverse; }
	.as-gl { color: var(--text-muted, #6b7280); }
	.as-gv { color: var(--text-normal, #1a1a1a); text-transform: capitalize; }
	.as-lad { display: inline-flex; gap: 3px; }
	.as-lad i { width: 6px; height: 6px; border-radius: 50%; background: var(--background-modifier-border, #d4d4d8); }
	.as-lad i.on { background: var(--interactive-accent, #7c3aed); }
	.as-lad i.cur { box-shadow: 0 0 0 2px color-mix(in srgb, var(--interactive-accent, #7c3aed) 35%, transparent); }
	.as-pill { font-size: 11px; border-radius: 6px; padding: 1px 7px; }
	.as-ok { color: var(--rel-supports, #16a34a); background: color-mix(in srgb, var(--rel-supports, #16a34a) 14%, transparent); }
	.as-warn { color: #b7791f; background: color-mix(in srgb, #d0a215 18%, transparent); }
	.as-bad { color: var(--rel-contradicts, #dc2626); background: color-mix(in srgb, var(--rel-contradicts, #dc2626) 14%, transparent); }
	.as-mut { color: var(--text-muted, #6b7280); background: color-mix(in srgb, var(--text-muted, #6b7280) 12%, transparent); }
	.as-num { font-size: 22px; font-weight: 600; line-height: 1; display: flex; align-items: baseline; gap: 6px; }
	.as-numl { font-size: 11px; font-weight: 400; color: var(--text-muted, #6b7280); }
	.as-chips { display: flex; gap: 4px; flex-wrap: wrap; justify-content: flex-end; max-width: 220px; }
	.as-chip { font-size: 11px; color: var(--text-muted, #6b7280); background: var(--background-modifier-border, #ececec); border-radius: 5px; padding: 1px 7px; }
	.as-chip.as-more { background: transparent; }
	.as-mix { display: flex; width: 130px; height: 7px; border-radius: 4px; overflow: hidden; gap: 1px; }
	.as-mix.as-conf { width: 110px; height: 6px; }
	.as-mix span { min-width: 2px; }
	.as-bal { display: flex; width: 76px; height: 7px; border-radius: 4px; overflow: hidden; background: var(--background-modifier-border, #ececec); }
	.as-bal-s { background: var(--rel-supports, #879A39); }
	.as-bal-c { background: var(--rel-contradicts, #D14D41); }
</style>
