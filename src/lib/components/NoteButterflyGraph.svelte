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
	 * Every one of the note's links is drawn as its own radial STEM + rim NODE: each stem starts at a
	 * coloured collar just off the box (R_stem) and runs out to its wedge's OUTER RIM (rNodeRing),
	 * ending in a bead — evenly divided across the wedge's angular width, deterministic (links sorted
	 * once by name), with ZERO jitter, sampling, or capping. Earned weight NO LONGER sets stem LENGTH
	 * (every stem reaches the rim); it is re-homed onto two honest continuous channels: PRIMARY = node
	 * radius (heavier link → bigger bead; genuinely unresolvable in the dense 394-stem fan, so NOT
	 * faked there) and SECONDARY = stem opacity (the channel with resolution left in the dense fan, so
	 * heavy links read as slightly darker radial streaks). Tier adds ONLY categorical overlays on top:
	 * load-bearing → ×1.4 stem width + a halo ring; stale → dashed stem + a faded bead. Wedge reach
	 * f(n) carries a legibility floor so a sparse note still reads bold; each wedge names itself as
	 * '<type> · <count>' in a per-wing vertical callout column at the stage edge, forced ≥17px apart
	 * so labels can never overlap for any text width, φ, or locale.
	 *
	 * Read-only (Display-not-Domain): hover names a link, click calls onNavigate() so the MAIN window
	 * travels there — nothing is edited or saved. Theme-aware via app CSS vars; relationship colour
	 * only from relColor() (--rel-*, Style-Setter controlled), so it reads in light AND dark.
	 */
	import { t, tn, locale } from '$lib/i18n';
	import { groupByType, relColor, relLabelIn, tierW, clean } from '$lib/cockpitGraphData';
	import { linkTypesStore } from '$lib/libraries/linkTypeRegistry';
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
	const clamp = (v: number, lo: number, hi: number) => Math.min(Math.max(v, lo), hi);
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
		void $linkTypesStore;   // recolour/re-order when the link-type vocabulary changes
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
		// a sparse note has a small gmax so its top petal → f≈1). Exact count rides the wedge label.
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

		if (!all.length) return { geo, petals: [] as any[], largestPi: 0 };

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
				// per-wedge radial-stem geometry — all angles in DEGREES; P() multiplies by DEG.
				const rO = Math.max(R_in + 8, pet.f * R);            // +8 guarantees rO-5 ≥ R_in+3 so every clamp below is valid
				const phiLo = pet.phi - pet.width / 2;
				const phiHi = pet.phi + pet.width / 2;
				const n = pet.links.length;                          // groupByType only yields present types ⇒ n ≥ 1 always
				const pad_deg = clamp(0.12 * pet.width, 0.6, 3);
				const angSpan = (pet.width - 2 * pad_deg) * DEG;     // radians, > 0 (width ≥ 7, pad ≤ 3 ⇒ span ≥ 1°)
				const arcRim = rO * angSpan;
				const s = arcRim / Math.max(1, n);                   // THE ONE density scalar — px of rim arc per stem
				const HUB_GAP = clamp(0.06 * rO, 7, 16);
				const R_stem = clamp(R_in + HUB_GAP, R_in + 3, rO - 5);           // stems start OFF the box, leaving a coloured collar
				const rCap = clamp(0.60 * s, 0.75, 2.8);
				const RIM_INSET = clamp(1.15 * rCap + 0.6, 1, rO - R_stem - 2);   // hi bound ≥ 3 because R_stem ≤ rO-5 ⇒ never a broken clamp
				const rNodeRing = rO - RIM_INSET;                                 // ONE bead ring; rNodeRing ≥ R_stem + 2 (no inversion)
				const tt = clamp(s / 3, 0, 1);
				const baseSW = 0.45 + 0.95 * tt;                     // per-wedge stem width (dense→thin, sparse→bold)
				const baseOP = 0.30 + 0.50 * tt;                     // per-wedge base stem opacity

				// EVEN ANGULAR DIVISION, deterministic, NO jitter. Sort links ONCE by clean(name).localeCompare
				// (determinism ONLY — the angular slot carries NO weight/order meaning; ordering by weight would
				// fake an angular axis = Form-Aligns-To-Purpose violation).
				const sorted = pet.links.slice().sort((a: any, b: any) =>
					clean(isOut ? a.target : a.name).localeCompare(clean(isOut ? b.target : b.name)));

				const fil: any[] = [];
				sorted.forEach((lk: any, j: number) => {
					const phiJ = phiLo + pad_deg + ((j + 0.5) / n) * (pet.width - 2 * pad_deg); // +0.5 half-step keeps first/last off the side-strokes
					const [bx, by] = P(ox, sign, R_stem / R, phiJ);
					const [tx, ty] = P(ox, sign, rNodeRing / R, phiJ);
					const res = isOut ? (resolveTarget?.(lk.target ?? '') ?? { path: '', libraryName: lk.libraryName }) : { path: lk.path, libraryName: lk.libraryName };
					const travNorm = maxTrav > 0 ? Math.min(1, (lk.traversalCount ?? 0) / maxTrav) : 0;
					const weightNorm = 0.55 * tierW(lk.tier) + 0.45 * travNorm;
					const rN = clamp(rCap * (0.5 + 0.6 * weightNorm), 0.6, rCap * 1.15);       // bead radius (heavier → bigger)
					const stemOP = clamp(baseOP * (0.75 + 0.5 * weightNorm), 0.14, 0.9);       // heavy stem = slightly darker streak
					fil.push({
						bx, by, tx, ty, name: clean(isOut ? lk.target : lk.name), path: res.path,
						lib: res.libraryName || lk.libraryName, tier: String(lk.tier || 'emerging').toLowerCase(),
						weightNorm, rN, stemOP,
					});
				});

				// wedge: true circular arcs, equal radii. Left wing swaps both sweep flags (x mirrored).
				const [ix0, iy0] = P(ox, sign, R_in / R, phiLo);
				const [ix1, iy1] = P(ox, sign, R_in / R, phiHi);
				const [ex1, ey1] = P(ox, sign, rO / R, phiHi);
				const [ex0, ey0] = P(ox, sign, rO / R, phiLo);
				const sIn = sign > 0 ? 1 : 0, sOut = sign > 0 ? 0 : 1;
				const wedge = `M${r2(ix0)} ${r2(iy0)} A${r2(R_in)} ${r2(R_in)} 0 0 ${sIn} ${r2(ix1)} ${r2(iy1)} L${r2(ex1)} ${r2(ey1)} A${r2(rO)} ${r2(rO)} 0 0 ${sOut} ${r2(ex0)} ${r2(ey0)} Z`;

				return {
					type: pet.type, sign, ox, color: relColor(pet.type), ember: pet.type === 'contradicts',
					count: n, wedge, fil, phiMid: pet.phi, rO, baseSW,
					// label fields — seeded so model.petals stays typed; the ladder overwrites them below.
					labelY: 0, estW: 0, rimX: 0, rimY: 0, anchor: 'start', labelX: 0, swatchX: 0, leaderX2: 0,
				};
			});
		}

		const OL = cx - S, OR = cx + S;
		const leftP = geom(aL, -1, OL);
		const rightP = geom(aR, 1, OR);

		// per-wing 1-D VERTICAL CALLOUT COLUMN at the stage edge: all labels on a wing share one
		// horizontal anchor direction, so forcing every pair ≥ DY apart in Y makes their bounding
		// boxes provably disjoint for ANY text width, φ, or locale — overlap is geometrically impossible.
		function ladder(ps: any[], side: number) {
			if (!ps.length) return;
			const items: any[] = ps.map((p: any) => ({ p, yNat: cy - p.rO * Math.sin(p.phiMid * DEG) }));
			items.sort((a, b) => a.yNat - b.yNat);
			const DY = 17;
			const bot = H - FOOT_PAD - 10, top = HEADER_BAND + 10;
			// forward pass: monotonic, each ≥ DY below the previous, first no higher than `top`.
			let y = top - DY;
			for (const it of items) { it.ly = Math.max(it.yNat, y + DY); y = it.ly; }
			// if the column overflows the bottom, compress it up off `bot` — the ≥DY spacing is kept by
			// construction, so labelY is assigned directly (no independent clamp that could collapse it).
			if (items[items.length - 1].ly > bot) {
				let yy = bot;
				for (let i = items.length - 1; i >= 0; i--) { items[i].ly = Math.min(items[i].ly, yy); yy = items[i].ly - DY; } // back-pass compress up
			}
			for (const it of items) {
				const p = it.p;
				p.labelY = it.ly;
				p.estW = (p.type.length + 3 + String(p.count).length) * 6.6;   // matches the file's char metric
				p.rimX = p.ox + side * p.rO * Math.cos(p.phiMid * DEG);
				p.rimY = cy - p.rO * Math.sin(p.phiMid * DEG);
				if (side > 0) { p.anchor = 'end'; p.labelX = W - 12; p.swatchX = (W - 12) - p.estW - 13; p.leaderX2 = p.swatchX - 4; }   // RIGHT: swatch centre-facing
				else { p.anchor = 'start'; p.labelX = 25; p.swatchX = 12; p.leaderX2 = 25 + p.estW + 4; }                                  // LEFT: swatch at edge, text starts x=25
			}
		}
		ladder(leftP, -1); ladder(rightP, 1);
		const petals = [...leftP, ...rightP];

		let largestPi = 0, best = -1;
		petals.forEach((p, i) => { if (p.count > best) { best = p.count; largestPi = i; } });

		return { geo, petals, largestPi };
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
		return { f, color: p.color, side: (p.sign < 0 ? 'left' : 'right') as 'left' | 'right', nodeR: Math.max(4, f.rN + 1) };
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

				<!-- dark dotted seam parting the two wings, above and below the box (Boss #3) -->
				<line class="bf-seam" x1={model.geo.cx} y1={HEADER_BAND} x2={model.geo.cx} y2={model.geo.cy - BOXH / 2 - 8}/>
				<line class="bf-seam" x1={model.geo.cx} y1={model.geo.cy + BOXH / 2 + 8} x2={model.geo.cx} y2={H - FOOT_PAD}/>

				<!-- edge-anchored flanking wing totals (no wedge can reach them) -->
				<text class="bf-flank" x="12" y="20" text-anchor="start">◀ {L('cockpit.incoming', 'incoming')} · {backlinks.length}</text>
				<text class="bf-flank" x={W - 12} y="20" text-anchor="end">{L('cockpit.outgoing', 'outgoing')} · {outgoing.length} ▶</text>

				<!-- wings: solid wedges + individual stem/node marks (delegated, no per-node listeners) -->
				<g class="bf-marks" class:dimmed={!!activeFil}>
					{#each model.petals as p, pi}
						<path class="bf-wedge" d={p.wedge} fill={p.color} fill-opacity={p.ember ? 0.90 : 0.85}
							stroke={p.color} stroke-opacity="0.55" stroke-width="1"/>
						{#each p.fil as f, k}
							<g class="bf-fil" data-pi={pi} data-k={k}>
								<line x1={f.bx} y1={f.by} x2={f.tx} y2={f.ty} stroke="var(--text-normal)"
									stroke-opacity={f.stemOP}
									stroke-width={f.tier === 'load-bearing' ? p.baseSW * 1.4 : p.baseSW}
									stroke-dasharray={f.tier === 'stale' ? '1 2' : undefined}/>
								<circle cx={f.tx} cy={f.ty} r={f.rN}
									fill={p.color} fill-opacity={f.tier === 'stale' ? 0.5 : 1}
									stroke="var(--background-primary)" stroke-width="0.5"/>
								{#if f.tier === 'load-bearing'}
									<circle cx={f.tx} cy={f.ty} r={f.rN + 0.6} fill="none" stroke="var(--text-normal)" stroke-width="0.7"/>
								{/if}
							</g>
						{/each}
					{/each}
				</g>

				<!-- per-wing vertical callout columns (OUTSIDE .bf-marks ⇒ stay full-opacity on hover):
				     leaders → swatches → labels -->
				{#each model.petals as p}
					<line x1={p.rimX} y1={p.rimY} x2={p.leaderX2} y2={p.labelY} stroke={relColor(p.type)} stroke-opacity="0.5" stroke-width="1"/>
				{/each}
				{#each model.petals as p}
					<rect x={p.swatchX} y={p.labelY - 4.5} width="9" height="9" rx="2" fill={relColor(p.type)}/>
				{/each}
				{#each model.petals as p}
					<text class="bf-label" x={p.labelX} y={p.labelY} text-anchor={p.anchor}><tspan font-weight="600" fill="var(--text-normal)">{relLabelIn($locale, p.type)}</tspan><tspan font-weight="400" fill="var(--text-muted)"> · {p.count}</tspan></text>
				{/each}

				<!-- the spine: a plain title box, no arc, no handbag (Boss #5) -->
				<rect class="bf-box" x={model.geo.cx - model.geo.boxW / 2} y={model.geo.cy - BOXH / 2}
					width={model.geo.boxW} height={BOXH} rx="12"/>
				<text class="bf-title" x={model.geo.cx} y={model.geo.cy - 4} text-anchor="middle">{model.geo.title}</text>
				<text class="bf-sub" x={model.geo.cx} y={model.geo.cy + 15} text-anchor="middle">{$tn('plurals.links', total)}</text>
				{#if !hasAny}
					<text class="bf-empty" x={model.geo.cx} y={model.geo.cy + BOXH / 2 + 22} text-anchor="middle">{L('cockpit.noLinks', 'no links yet')}</text>
				{/if}

				<!-- bright O(1) overlay for the active (hovered or keyboard-focused) link: stem + node + plate -->
				{#if activeFil && plate}
					<g pointer-events="none">
						{#if isFocusActive}
							<circle cx={activeFil.f.tx} cy={activeFil.f.ty} r={activeFil.nodeR + 2} fill="none" stroke="var(--interactive-accent)" stroke-width="1.5"/>
						{/if}
						<line x1={activeFil.f.bx} y1={activeFil.f.by} x2={activeFil.f.tx} y2={activeFil.f.ty}
							stroke="var(--text-normal)" stroke-opacity="0.95"
							stroke-width={activeFil.f.tier === 'load-bearing' ? 1.6 : 1.1}/>
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
	/* The cockpit is wrapped in dir={$dir}. SVG text-anchor is resolved against the INLINE BASE
	   DIRECTION, so under RTL "start" means the right edge — every label and header anchored to
	   the wrong side and ran off-canvas. The graph's geometry is LTR by definition (backlinks
	   left, outgoing right), so pin the SVG to ltr and let each text run shape itself via
	   unicode-bidi: plaintext (Arabic still reads right-to-left inside its own box). */
	.bf-svg { width: 100%; height: 100%; display: block; outline: none; direction: ltr; }
	.bf-seam { stroke: var(--text-normal); stroke-opacity: 0.7; stroke-dasharray: 1 4; stroke-width: 1; }
	.bf-flank { font: 12px var(--font-sans); fill: var(--text-muted, #6b7280);  unicode-bidi: plaintext; }
	.bf-label { font: 12px var(--font-sans); dominant-baseline: middle; unicode-bidi: plaintext;  }
	.bf-fil { cursor: pointer; }
	.bf-marks.dimmed { opacity: 0.16; transition: opacity 0.12s; }
	.bf-box { fill: var(--background-primary, #fff); stroke: var(--background-modifier-border, #d4d4d8); }
	.bf-title { font: 600 15px var(--font-text, var(--font-sans)); fill: var(--text-normal, #1a1a1a); unicode-bidi: plaintext; }
	.bf-sub { font: 11px var(--font-sans); fill: var(--text-muted, #6b7280); }
	.bf-empty { font: 13px var(--font-sans); fill: var(--text-muted, #6b7280); }
	.bf-plate { fill: var(--background-primary, #fff); fill-opacity: 0.94; stroke: var(--background-modifier-border, #d4d4d8); }
	.bf-pname { font: 13px var(--font-sans); fill: var(--text-normal, #1a1a1a); unicode-bidi: plaintext; }
</style>
