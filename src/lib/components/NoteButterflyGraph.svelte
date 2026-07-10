<script lang="ts">
	/**
	 * PJ-068 v3 — "The Butterfly" note-graph lens (Art Director rebuild, final).
	 *
	 * THE HORSE: at a glance, is this note more anchored by what points AT it (BACKLINKS, left wing)
	 * or by where it points OUT (OUTGOING, right wing) — and which typed relationships carry that
	 * weight? Two mirrored wings part across a central spine and are built CENTRE-OUT with GREEDY
	 * BALANCE: the largest relationship lies on each wing's own horizontal axis (φ=0) and the rest
	 * fan to whichever side (above/below) currently holds the smaller cumulative angular width
	 * (tie → above). So each wing's visual mass lands ON the axis, neither wing tilts, and the two
	 * wings mirror across the vertical spine — the figure reads level, never pinwheeled.
	 *
	 * GEOMETRY: ONE isotropic radius R — every arc is a true circle (equal rx=ry, no ellipse, no
	 * stretch). The wide stage is filled by an ADAPTIVE VERTICAL ENVELOPE, never by scaling: because
	 * the biggest petal lies on the axis (small vertical extent), R can grow large while every arc
	 * stays round; narrow stages become horizontal-bound and R shrinks so wings never cross the seam.
	 *
	 * Every one of the note's links is drawn as its own filament + node, packed into 2-D rings by
	 * earned weight — nothing capped, sampled, or jittered. Each link sits on the ring NEAREST its
	 * own weight-radius that still has angular room (heaviest → outer, longest vein); an equal-weight
	 * group (e.g. part-of's 394 structural links) spills into neighbouring rings, so separation falls
	 * out of real weight + honest geometry — never Math.random. Wedge reach f(n) carries a legibility
	 * floor so a sparse note still reads bold; the exact integer count is printed at each tip.
	 *
	 * Read-only (Display-not-Domain): hover names a link, click calls onNavigate() so the MAIN window
	 * travels there — nothing is edited or saved. Theme-aware via app CSS vars; relationship colour
	 * only from relColor() (--rel-*, Style-Setter controlled), so it reads in light AND dark.
	 */
	import { t } from '$lib/i18n';
	import { groupByType, relColor, tierW, clean, REL_ORDER } from '$lib/cockpitGraphData';
	import { detectDir } from '$lib/utils';
	import NoteGaugeDeck from './NoteGaugeDeck.svelte';

	let { noteName = '', content = '', review = null as any, backlinks = [] as any[], outgoing = [] as any[], resolveTarget, onNavigate }: {
		noteName?: string; content?: string; review?: any; backlinks?: any[]; outgoing?: any[];
		resolveTarget?: (name: string) => { path: string; libraryName: string };
		onNavigate?: (path: string, name: string, libraryName: string) => void;
	} = $props();

	// safe i18n: the translation store returns the KEY for a missing string (truthy), so `$t(k) || fb`
	// never falls back. `$t(k) === k ? fb : v` shows the English fallback instead of a raw key.
	const L = (k: string, fb: string) => { const v = $t(k); return v === k ? fb : v; };

	// a selection carries a note-identity key so a stale hover/focus from a previous note cannot
	// dim the graph or light up the wrong node after navigation (runes-clean: no $effect reset).
	type Sel = { pi: number; k: number; key: string };

	const HEADER_BAND = 34, TIP_PAD_V = 14, FOOT_PAD = 16, EDGE_PAD = 48, BOXH = 52, GAP = 2, REACH_FLOOR = 0.34;
	const DEG = Math.PI / 180;
	const TIERR: Record<string, number> = { 'load-bearing': 1.5, established: 1.15, emerging: 1.0, stale: 0.7 };
	const clamp = (v: number, lo: number, hi: number) => Math.min(Math.max(v, lo), hi);
	const nodeRadius = (tier: string, dotR: number) => dotR * (TIERR[tier] ?? 1.0);
	const r2 = (x: number) => Math.round(x * 100) / 100;

	let W = $state(0), H = $state(0);
	let hover = $state(null as Sel | null);
	let focusIdx = $state(null as Sel | null);

	let total = $derived(backlinks.length + outgoing.length);
	let hasAny = $derived(total > 0);
	// identity token: any of these changing means a different note is on screen
	let noteKey = $derived(noteName + '|' + backlinks.length + '|' + outgoing.length);

	// ── ALL geometry lives here — one derivation, no $effect, no per-frame work. ─────────────────
	let model = $derived.by(() => {
		const cx = W / 2, cy = H / 2;
		const raw = clean(noteName);
		const rtl = detectDir(raw || noteName) === 'rtl';
		const title = raw.length > 32 ? (rtl ? '…' + raw.slice(raw.length - 32) : raw.slice(0, 32) + '…') : raw;
		const boxW = clamp(title.length * 8.4 + 34, 140, 300);
		const S = boxW / 2 + 18;
		const horizReach = (cx - S) - EDGE_PAD;
		const vertRoom = H / 2 - HEADER_BAND - TIP_PAD_V;
		const geo = { cx, cy, boxW, S, title, rtl };

		const gL = groupByType(backlinks), gR = groupByType(outgoing);
		const counts = [...Object.values(gL), ...Object.values(gR)].map((a) => a.length);
		const gmax = Math.max(1, ...counts);
		const maxTrav = Math.max(0, ...backlinks.map((l) => l.traversalCount ?? 0), ...outgoing.map((l) => l.traversalCount ?? 0));

		// shared scales across BOTH wings ⇒ the wings are honestly asymmetric.
		// f(n): reach fraction of R with a legibility floor (a count-1 petal still reaches a third of R;
		// a sparse note has a small gmax so its top petal → f≈1). Exact count is printed at the tip.
		const fOf = (n: number) => REACH_FLOOR + (1 - REACH_FLOOR) * Math.sqrt(n / gmax);
		// w(n): angular width; √ on both width and reach ⇒ petal ink ∝ ~count with a legibility floor.
		const wOf = (n: number) => clamp(2.0 * Math.sqrt(n), 7, 40);

		// GREEDY CENTRE-OUT: largest on the wing axis (φ=0); each next petal joins the least-loaded side.
		function assign(groups: Record<string, any[]>) {
			const types = Object.keys(groups).sort((a, b) => groups[b].length - groups[a].length || a.localeCompare(b));
			let above = 0, below = 0;
			const out: any[] = [];
			types.forEach((tp, i) => {
				const n = groups[tp].length;
				const width = wOf(n);
				let phi: number;
				if (i === 0) { phi = 0; above = width / 2; below = width / 2; }
				else if (above <= below) { phi = above + GAP + width / 2; above += GAP + width; } // tie → above
				else { phi = -(below + GAP + width / 2); below += GAP + width; }
				out.push({ type: tp, phi, width, f: fOf(n), links: groups[tp] });
			});
			return out;
		}
		const aL = assign(gL), aR = assign(gR);
		const all = [...aL, ...aR];

		// present-types colour legend (stable REL_ORDER), centred on cx — only when it fits without
		// colliding with the edge-anchored flanking headers.
		const flankL = `◀ ${L('cockpit.incoming', 'incoming')} · ${backlinks.length}`;
		const flankR = `${L('cockpit.outgoing', 'outgoing')} · ${outgoing.length} ▶`;
		const flankLW = 12 + flankL.length * 6.6, flankRW = 12 + flankR.length * 6.6;
		const legend = (REL_ORDER as readonly string[]).filter((tp) => gL[tp] || gR[tp]);
		const estW = (s: string) => s.length * 6.3;
		const totalLegW = legend.reduce((sum, tp) => sum + 9 + 5 + estW(tp), 0) + 14 * Math.max(0, legend.length - 1);
		const legItems: any[] = [];
		if (W >= 820 && legend.length && (cx - totalLegW / 2) > flankLW + 12 && (cx + totalLegW / 2) < W - flankRW - 12) {
			let lgx = cx - totalLegW / 2;
			for (const tp of legend) { legItems.push({ type: tp, color: relColor(tp), sx: lgx, tx: lgx + 14, label: tp }); lgx += 9 + 5 + estW(tp) + 14; }
		}

		if (!all.length) return { geo, petals: [] as any[], legItems, largestPi: 0 };

		// ADAPTIVE ENVELOPE → single isotropic R (never scale the shape to fill the stage).
		let D = 0;
		for (const p of all) { const d = p.f * Math.sin((Math.abs(p.phi) + p.width / 2) * DEG); if (d > D) D = d; }
		let R = clamp(Math.min(horizReach, D > 0 ? vertRoom / D : vertRoom), 60, Math.max(60, horizReach));
		if (horizReach >= 60) R = Math.min(R, horizReach); else R = Math.max(30, horizReach); // never cross the seam on tiny stages
		if (!isFinite(R) || R <= 0) R = 60;
		const R_in = clamp(0.05 * R, 24, 40);

		// point map: single R on both axes, +φ = up in both wings; sign mirrors x across the spine.
		const P = (ox: number, sign: number, u: number, phi: number): [number, number] => {
			const r = u * R, a = phi * DEG;
			return [ox + sign * r * Math.cos(a), cy - r * Math.sin(a)];
		};

		function geom(assignments: any[], sign: number, ox: number) {
			const isOut = sign > 0;
			return assignments.map((pet) => {
				const rO = Math.max(R_in + 2, pet.f * R);
				const phiLo = pet.phi - pet.width / 2, phiHi = pet.phi + pet.width / 2;
				const wRad = pet.width * DEG;
				const n = pet.links.length;
				const petalArea = Math.max(0.0001, 0.5 * (rO * rO - R_in * R_in) * wRad);
				const dotR = clamp(0.42 * Math.sqrt(petalArea / n), 0.9, 3.2);
				const rings = Math.max(1, Math.round((rO - R_in) / (2.6 * dotR)));
				const pad = Math.min(2.2, 0.12 * pet.width);
				const angSpan = Math.max(0, (pet.width - 2 * pad) * DEG);
				const spacing = 2.2 * dotR;

				// per-link earned weight, then sort heaviest-first (name tiebreak → deterministic).
				const withW = pet.links.map((lk: any) => {
					const travNorm = maxTrav > 0 ? Math.min(1, (lk.traversalCount ?? 0) / maxTrav) : 0;
					const weightNorm = 0.55 * tierW(lk.tier) + 0.45 * travNorm;
					return { lk, weightNorm, name: clean(isOut ? lk.target : lk.name) };
				});
				withW.sort((a: any, b: any) => b.weightNorm - a.weightNorm || a.name.localeCompare(b.name));

				// ring capacities: outer rings are longer arcs → hold more dots.
				const cap: number[] = new Array(rings);
				for (let b = 0; b < rings; b++) {
					const rr = R_in + ((b + 0.5) / rings) * (rO - R_in);
					cap[b] = Math.max(1, Math.floor((angSpan * rr) / spacing));
				}
				// Place each link on the ring NEAREST its own weight-radius that still has room:
				// heaviest → outer ring / longest vein (honest); an equal-weight bulk (part-of's 394)
				// spills to neighbouring rings, so 2-D separation falls out of weight + geometry —
				// no bucket-collapse, no jitter. All n links are placed (overflow fallback guarantees it).
				const fillCount = new Array(rings).fill(0);
				const ringBuckets: any[][] = Array.from({ length: rings }, () => []);
				for (const it of withW) {
					const idealB = clamp(Math.round(it.weightNorm * (rings - 1)), 0, rings - 1);
					let chosen = -1;
					for (let d = 0; d < rings; d++) {
						const hi = idealB + d, lo = idealB - d;
						if (hi <= rings - 1 && fillCount[hi] < cap[hi]) { chosen = hi; break; }
						if (lo >= 0 && fillCount[lo] < cap[lo]) { chosen = lo; break; }
					}
					if (chosen < 0) chosen = idealB;
					fillCount[chosen]++;
					ringBuckets[chosen].push(it);
				}

				const fil: any[] = [];
				for (let b = 0; b < rings; b++) {
					const members = ringBuckets[b];
					const m = members.length;
					if (!m) continue;
					const rRing = R_in + ((b + 0.5) / rings) * (rO - R_in);
					members.forEach((it: any, j: number) => {
						const phiJ = phiLo + pad + ((j + 0.5) / m) * (pet.width - 2 * pad); // even division within the ring
						const [bx, by] = P(ox, sign, R_in / R, phiJ);
						const [tx, ty] = P(ox, sign, rRing / R, phiJ);
						const lk = it.lk;
						const res = isOut ? (resolveTarget?.(lk.target ?? '') ?? { path: '', libraryName: lk.libraryName }) : { path: lk.path, libraryName: lk.libraryName };
						fil.push({ bx, by, tx, ty, name: it.name, path: res.path, lib: res.libraryName || lk.libraryName, tier: String(lk.tier || 'emerging').toLowerCase(), weightNorm: it.weightNorm });
					});
				}

				// wedge: true circular arcs, equal radii. Left wing swaps both sweep flags (x mirrored).
				const [ix0, iy0] = P(ox, sign, R_in / R, phiLo);
				const [ix1, iy1] = P(ox, sign, R_in / R, phiHi);
				const [ex1, ey1] = P(ox, sign, rO / R, phiHi);
				const [ex0, ey0] = P(ox, sign, rO / R, phiLo);
				const sIn = sign > 0 ? 1 : 0, sOut = sign > 0 ? 0 : 1;
				const wedge = `M${r2(ix0)} ${r2(iy0)} A${r2(R_in)} ${r2(R_in)} 0 0 ${sIn} ${r2(ix1)} ${r2(iy1)} L${r2(ex1)} ${r2(ey1)} A${r2(rO)} ${r2(rO)} 0 0 ${sOut} ${r2(ex0)} ${r2(ey0)} Z`;

				const sinPhi = Math.sin(pet.phi * DEG);
				const cosTip = sign * Math.cos(pet.phi * DEG);
				const countAnchor = cosTip < -0.34 ? 'end' : cosTip > 0.34 ? 'start' : 'middle';
				const countDy = sinPhi > 0.5 ? -3 : sinPhi < -0.5 ? 11 : 4;
				return {
					type: pet.type, sign, ox, color: relColor(pet.type), ember: pet.type === 'contradicts',
					count: n, wedge, fil, dotR, phiMid: pet.phi, rO, rL: rO + 14, countAnchor, countDy, lx: 0, ly: 0,
				};
			});
		}

		const OL = cx - S, OR = cx + S;
		const leftP = geom(aL, -1, OL);
		const rightP = geom(aR, 1, OR);

		// RADIAL-ONLY declutter of tip counts: push the larger-radius petal further out along ITS OWN
		// φ (a meaningful reach axis, never sideways jitter) by the overlap deficit.
		const labelPt = (p: any): [number, number] => P(p.ox, p.sign, p.rL / R, p.phiMid);
		function declutter(ps: any[]) {
			const order = ps.map((p, i) => ({ i, phi: p.phiMid })).sort((a, b) => a.phi - b.phi);
			for (let pass = 0; pass < 2; pass++) {
				for (let q = 0; q < order.length - 1; q++) {
					const A = ps[order[q].i], B = ps[order[q + 1].i];
					const pa = labelPt(A), pb = labelPt(B);
					const dist = Math.hypot(pa[0] - pb[0], pa[1] - pb[1]);
					if (dist < 22) { const def = 22 - dist; (A.rO >= B.rO ? A : B).rL += def; }
				}
			}
		}
		declutter(leftP); declutter(rightP);
		const petals = [...leftP, ...rightP];
		// resolve + clamp each count label onto the safe band so a declutter push can never drive it
		// into the header band, past the foot, or off the left/right edge.
		for (const p of petals) {
			const [x, y] = labelPt(p);
			p.lx = clamp(x, 10, Math.max(10, W - 10));
			p.ly = clamp(y, HEADER_BAND + 8, Math.max(HEADER_BAND + 8, H - FOOT_PAD - 6));
		}

		let largestPi = 0, best = -1;
		petals.forEach((p, i) => { if (p.count > best) { best = p.count; largestPi = i; } });

		return { geo, petals, legItems, largestPi };
	});

	// active selection routes hover OR keyboard focus to the same O(1) overlay; a stale key (note
	// changed under a held selection) resolves to null so nothing dims or lights up wrongly.
	let active = $derived(hover ?? focusIdx);
	let isFocusActive = $derived(!hover && !!focusIdx);
	let activeFil = $derived.by(() => {
		const a = active as Sel | null;
		if (!a || a.key !== noteKey) return null;
		const p = model.petals[a.pi]; if (!p) return null;
		const f = p.fil[a.k]; if (!f) return null;
		return { f, color: p.color, side: (p.sign < 0 ? 'left' : 'right') as 'left' | 'right', nodeR: Math.max(4, nodeRadius(f.tier, p.dotR) + 1) };
	});
	let plate = $derived.by(() => {
		const af = activeFil; if (!af) return null;
		const name = af.f.name || '';
		const plateW = Math.max(40, name.length * 7 + 16), plateH = 22;
		const nx = af.f.tx, ny = af.f.ty;
		let plateX = af.side === 'left' ? nx - 9 - plateW : nx + 9;
		if (plateX < 6) plateX = nx + 9;                 // flip inboard
		if (plateX + plateW > W - 6) plateX = W - 6 - plateW;
		if (plateX < 6) plateX = 6;
		return { x: plateX, y: ny - plateH / 2, w: plateW, h: plateH, cx: plateX + plateW / 2, ty: ny + 4, name };
	});

	function navigate(pi: number, k: number) {
		const f = model.petals[pi]?.fil[k];
		if (f?.path) onNavigate?.(f.path, f.name, f.lib || '');
	}
	// ── event delegation: 4 listeners on the svg, ZERO per-node listeners ─────────────────────────
	function onMove(e: PointerEvent) {
		const g = (e.target as Element)?.closest?.('[data-k]') as HTMLElement | null;
		hover = g ? { pi: +g.dataset.pi!, k: +g.dataset.k!, key: noteKey } : null;
	}
	function onLeave() { hover = null; }
	function onClick(e: MouseEvent) {
		const g = (e.target as Element)?.closest?.('[data-k]') as HTMLElement | null;
		if (!g) return;
		navigate(+g.dataset.pi!, +g.dataset.k!);
	}
	function onKey(e: KeyboardEvent) {
		const ps = model.petals; if (!ps.length) return;
		const nav = e.key === 'ArrowUp' || e.key === 'ArrowDown' || e.key === 'ArrowLeft' || e.key === 'ArrowRight';
		// first keyboard interaction seeds focus to the largest petal (folds the focus-init in here,
		// so the svg keeps exactly four delegated listeners).
		if (focusIdx == null || focusIdx.key !== noteKey || !ps[focusIdx.pi]) {
			if (nav || e.key === 'Enter' || e.key === ' ') {
				e.preventDefault();
				focusIdx = { pi: model.largestPi, k: 0, key: noteKey };
				if (e.key === 'Enter' || e.key === ' ') navigate(model.largestPi, 0);
			} else if (e.key === 'Escape') { focusIdx = null; }
			return;
		}
		const cur = focusIdx;
		const filLen = ps[cur.pi].fil.length;
		const n = ps.length;
		if (e.key === 'ArrowUp') { e.preventDefault(); focusIdx = { pi: cur.pi, k: clamp(cur.k - 1, 0, filLen - 1), key: noteKey }; }
		else if (e.key === 'ArrowDown') { e.preventDefault(); focusIdx = { pi: cur.pi, k: clamp(cur.k + 1, 0, filLen - 1), key: noteKey }; }
		else if (e.key === 'ArrowLeft') { e.preventDefault(); const npi = (cur.pi - 1 + n) % n; focusIdx = { pi: npi, k: Math.min(cur.k, ps[npi].fil.length - 1), key: noteKey }; }
		else if (e.key === 'ArrowRight') { e.preventDefault(); const npi = (cur.pi + 1) % n; focusIdx = { pi: npi, k: Math.min(cur.k, ps[npi].fil.length - 1), key: noteKey }; }
		else if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); navigate(cur.pi, cur.k); }
		else if (e.key === 'Escape') { focusIdx = null; }
	}
</script>

<div class="bf">
	<div class="bf-stage" bind:clientWidth={W} bind:clientHeight={H}>
		{#if W > 80 && H > 80}
			<!-- deliberate: the graph IS the widget (role=application), so it takes focus and
			     handles arrow-key/Enter navigation across links. Delegated listeners, not per-node. -->
			<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
			<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
			<svg class="bf-svg" viewBox="0 0 {W} {H}" role="application" tabindex="0"
				aria-label={hasAny ? `${L('cockpit.incoming', 'incoming')} ${backlinks.length}, ${L('cockpit.outgoing', 'outgoing')} ${outgoing.length}` : L('cockpit.noLinks', 'no links yet')}
				onpointermove={onMove} onpointerleave={onLeave} onclick={onClick} onkeydown={onKey}>

				<!-- faint dotted seam parting the two wings, above and below the box (Boss #8) -->
				<line class="bf-seam" x1={model.geo.cx} y1={HEADER_BAND} x2={model.geo.cx} y2={model.geo.cy - BOXH / 2 - 8}/>
				<line class="bf-seam" x1={model.geo.cx} y1={model.geo.cy + BOXH / 2 + 8} x2={model.geo.cx} y2={H - FOOT_PAD}/>

				<!-- edge-anchored flanking headers (no wedge can reach them) -->
				<text class="bf-flank" x="12" y="20" text-anchor="start">◀ {L('cockpit.incoming', 'incoming')} · {backlinks.length}</text>
				<text class="bf-flank" x={W - 12} y="20" text-anchor="end">{L('cockpit.outgoing', 'outgoing')} · {outgoing.length} ▶</text>

				<!-- present-types colour legend (types stay hoverable when hidden) -->
				{#each model.legItems as it}
					<rect x={it.sx} y="16.5" width="9" height="9" rx="2" fill={it.color}/>
					<text class="bf-legend" x={it.tx} y="21">{it.label}</text>
				{/each}

				<!-- wings: solid wedges + individual filament/node marks (delegated, no per-node listeners) -->
				<g class="bf-marks" class:dimmed={!!activeFil}>
					{#each model.petals as p, pi}
						<path class="bf-wedge" d={p.wedge} fill={p.color} fill-opacity={p.ember ? 0.90 : 0.85}
							stroke={p.color} stroke-opacity="0.55" stroke-width="1"/>
						{#each p.fil as f, k}
							<g class="bf-fil" data-pi={pi} data-k={k}>
								<line x1={f.bx} y1={f.by} x2={f.tx} y2={f.ty}
									stroke="color-mix(in srgb, {p.color} 62%, var(--text-normal) 38%)"
									stroke-opacity="0.42"
									stroke-width={f.tier === 'load-bearing' ? 1.6 : 0.9}
									stroke-dasharray={f.tier === 'stale' ? '1 2' : undefined}/>
								{#if f.tier === 'load-bearing'}
									<circle cx={f.tx} cy={f.ty} r={nodeRadius(f.tier, p.dotR) + 0.6} fill="none" stroke="var(--text-normal)" stroke-width="0.7"/>
								{/if}
								<circle cx={f.tx} cy={f.ty} r={nodeRadius(f.tier, p.dotR)}
									fill={p.color} fill-opacity={f.tier === 'stale' ? 0.5 : 1}
									stroke="var(--background-primary)" stroke-width="0.5"/>
							</g>
						{/each}
					{/each}
				</g>

				<!-- grey tip counts (colour carries type identity; the wing prints only a count) -->
				{#each model.petals as p}
					<text class="bf-count" x={p.lx} y={p.ly} dy={p.countDy} text-anchor={p.countAnchor}>{p.count}</text>
				{/each}

				<!-- the spine: a plain title box, no arc, no handbag (Boss #5) -->
				<rect class="bf-box" x={model.geo.cx - model.geo.boxW / 2} y={model.geo.cy - BOXH / 2}
					width={model.geo.boxW} height={BOXH} rx="12"/>
				<text class="bf-title" x={model.geo.cx} y={model.geo.cy - 4} text-anchor="middle">{model.geo.title}</text>
				<text class="bf-sub" x={model.geo.cx} y={model.geo.cy + 15} text-anchor="middle">{total} {L('cockpit.links', 'links')}</text>
				{#if !hasAny}
					<text class="bf-empty" x={model.geo.cx} y={model.geo.cy + BOXH / 2 + 22} text-anchor="middle">{L('cockpit.noLinks', 'no links yet')}</text>
				{/if}

				<!-- bright O(1) overlay for the active (hovered or keyboard-focused) link -->
				{#if activeFil && plate}
					<g pointer-events="none">
						{#if isFocusActive}
							<circle cx={activeFil.f.tx} cy={activeFil.f.ty} r={activeFil.nodeR + 2} fill="none" stroke="var(--interactive-accent)" stroke-width="1.5"/>
						{/if}
						<circle cx={activeFil.f.tx} cy={activeFil.f.ty} r={activeFil.nodeR} fill={activeFil.color} stroke="var(--background-primary)" stroke-width="0.75"/>
						<rect class="bf-plate" x={plate.x} y={plate.y} width={plate.w} height={plate.h} rx="6"/>
						<text class="bf-pname" x={plate.cx} y={plate.ty} text-anchor="middle">{plate.name}</text>
					</g>
				{/if}
			</svg>
		{/if}
	</div>

	<NoteGaugeDeck {content} {review} {backlinks} {outgoing} />
</div>

<style>
	.bf { display: flex; flex-direction: column; width: 100%; height: 100%; min-height: 0; }
	.bf-stage { flex: 1; min-height: 0; width: 100%; background: var(--background-primary, #fff); }
	.bf-svg { width: 100%; height: 100%; display: block; outline: none; }
	.bf-seam { stroke: var(--background-modifier-border, #d4d4d8); stroke-opacity: 0.75; stroke-dasharray: 1 5; }
	.bf-flank { font: 12px var(--font-sans); fill: var(--text-muted, #6b7280); }
	.bf-legend { font: 11px var(--font-sans); fill: var(--text-muted, #6b7280); dominant-baseline: middle; }
	.bf-count { font: 600 11px var(--font-sans); fill: var(--text-muted, #6b7280); }
	.bf-fil { cursor: pointer; }
	.bf-marks.dimmed :global(.bf-fil) { opacity: 0.16; transition: opacity 0.12s; }
	.bf-box { fill: var(--background-primary, #fff); stroke: var(--background-modifier-border, #d4d4d8); }
	.bf-title { font: 600 15px var(--font-text, var(--font-sans)); fill: var(--text-normal, #1a1a1a); unicode-bidi: plaintext; }
	.bf-sub { font: 11px var(--font-sans); fill: var(--text-muted, #6b7280); }
	.bf-empty { font: 13px var(--font-sans); fill: var(--text-muted, #6b7280); }
	.bf-plate { fill: var(--background-primary, #fff); fill-opacity: 0.94; stroke: var(--background-modifier-border, #d4d4d8); }
	.bf-pname { font: 13px var(--font-sans); fill: var(--text-normal, #1a1a1a); unicode-bidi: plaintext; }
</style>
