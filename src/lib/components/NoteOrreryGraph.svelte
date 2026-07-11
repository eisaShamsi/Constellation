<script module lang="ts">
	// Per-instance unique suffix for gradient ids, so the main window and the second-screen
	// window (two live instances of this lens) never collide on url(#…) references.
	let _uidSeq = 0;
</script>

<script lang="ts">
	/**
	 * PJ-068 — "The Orrery" note-graph lens (Art Director final build + Boss's refinements).
	 *
	 * THE HORSE: at a glance, how RECENTLY has each of this note's links actually been walked, and
	 * which typed relationships are drifting cold? The note is a warm central SUN. Six concentric
	 * recency ORBITS carry the radial axis — inner = today/warm, outer rim = never-walked/cold — drawn
	 * as CLEAR, thin, theme-aware ring lines, each wearing its own recency label so the TIME structure
	 * is legible at a glance. Angular TYPE SECTORS carry the categorical axis (canonical order, one
	 * constant relColour per relationship) and each wing's ANGULAR WIDTH encodes its link COUNT: a
	 * 398-link type gets a wide wing, a 1–2-link type a small floored wing (still visible, hittable,
	 * rim-labelled). Within each (shell × type) CELL the bodies are placed by pure DETERMINISTIC EVEN
	 * ANGULAR DIVISION (name-sorted, zero jitter) — position is a function of the data alone.
	 *
	 * REMARK 1 (Boss, "zoom out" = the HOVER effect, NOT the resting field): at REST the view stays
	 * COMPACT and count-proportional — node dots are drawn at their NORMAL, readable size (a small sun
	 * + small inner dead-zone give the orbits their radial budget) and an over-dense (shell × type)
	 * cell may still tier its lightest excess into ONE counted '+N' cluster (nothing is ever dropped —
	 * the cluster fans on hover/click). The compact base is intentional; "show every node" is the
	 * HOVER-EXPAND effect below, not the resting field.
	 *
	 * REMARK 2 (Boss) — TOP LABEL DE-COLLISION: the six recency ring labels own the 12-o'clock spoke.
	 * Any wing '+N' cluster count or rim type-label that would land on that spoke is pushed clear —
	 * horizontally to the nearer side, and if a narrow pane would clamp it back onto the lane, radially
	 * out of the lane's band — so every recency label and every count stays readable at any width.
	 *
	 * HOVER-EXPAND (Boss remarks — "take advantage of the available space" + "enlarge the nodes"):
	 * hovering a wing (its background OR any of its bodies/clusters) turns it into a real zoom-in.
	 * (a) Its node dots grow to an ENLARGED size, clearly bigger than the resting dots — sized in ONE
	 *     uniform band across the whole wing so relative weight ORDER is preserved wing-wide (a heavy
	 *     node is never smaller than a lighter one, even across cells).
	 * (b) Its ANGULAR width grows AS FAR AS NEEDED — up to ~0.92·2π — so its fullest (shell × type)
	 *     cell seats EVERY node at that enlarged size with even spacing and NO '+N' cluster, while the
	 *     OTHER wings collapse to thin (still hittable) slivers floored > 0. If even at the 0.92 cap the
	 *     fullest cell cannot seat the enlarged dot, the wing's uniform radius steps DOWN together until
	 *     all nodes fit with no overlap — so "every node shown on hover" holds even for the worst case
	 *     (~371 in one cell). A SINGLE-type note (T === 1) already fills the circle, so its width can't
	 *     grow — but hovering it still enlarges its dots and dissolves its '+N' clusters.
	 * On mouse-leave the layout snaps back to the compact resting state. The geometry recomputes only
	 * when the hovered wing actually changes (no per-frame work).
	 *
	 * ENCODINGS (honesty ledger): direction = solid disc (outgoing) vs hollow ring (backlink);
	 * size = earnedWeight; halo = confidence, drawn purely as ring STYLE (hypothesis dotted /
	 * evidence thin / established thick / contested dashed-double) in the relation's OWN hue so no
	 * non-type colour enters; warmth (inner warm → outer cold) is a low-alpha radial-gradient annulus
	 * DECLARED REDUNDANT with the radial orbit axis. ALARM = load-bearing AND walked AND gone-cold → a
	 * STATUS-coloured warning ring (distinct from every type hue) with a slow breathe-pulse on a heavy
	 * body stranded on an outer cold orbit.
	 *
	 * ALL links are represented, read-only (Display-not-Domain): hover names a link, click travels the
	 * MAIN window. Theme-aware; relationship colour only from relColor().
	 */
	import { t, tn, locale, dir } from '$lib/i18n';
	import { relColor, relLabelIn, earnedWeight, recencyShell, normalizeType, orderTypes, clean, RECENCY_SHELLS, NEVER_SHELL } from '$lib/cockpitGraphData';
	import { linkTypesStore } from '$lib/libraries/linkTypeRegistry';
	import { detectDir } from '$lib/utils';
	import NoteGaugeDeck from './NoteGaugeDeck.svelte';

	let { noteName = '', content = '', review = null as any, backlinks = [] as any[], outgoing = [] as any[], resolveTarget, onNavigate }: {
		noteName?: string; content?: string; review?: any; backlinks?: any[]; outgoing?: any[];
		resolveTarget?: (name: string) => { path: string; libraryName: string };
		onNavigate?: (path: string, name: string, libraryName: string) => void;
	} = $props();

	// safe i18n: the store returns the KEY for a missing string (truthy), so `$t(k) || fb` never
	// falls back. `$t(k) === k ? fb : v` shows the English fallback instead of a raw key.
	const L = (k: string, fb: string) => { const v = $t(k); return v === k ? fb : v; };

	const uid = 'orr' + (++_uidSeq);
	const clamp = (v: number, lo: number, hi: number) => Math.min(Math.max(v, lo), hi);
	const r2 = (x: number) => Math.round(x * 100) / 100;

	// script-aware label width estimate (px @ ~11px): Arabic / Hebrew / CJK / Hangul run wider.
	const estW = (str: string) => {
		let w = 8;
		for (const ch of str) {
			const c = ch.charCodeAt(0);
			const wide = (c >= 0x0590 && c <= 0x08FF) || (c >= 0x1100 && c <= 0x30FF) ||
				(c >= 0x3400 && c <= 0x9FFF) || (c >= 0xAC00 && c <= 0xD7A3) ||
				(c >= 0xFB1D && c <= 0xFDFF) || (c >= 0xFE70 && c <= 0xFEFF);
			w += wide ? 9 : 6.2;
		}
		return w;
	};

	// English fallbacks for the six recency shells (no locale keys shipped yet — L() uses these).
	const SHELL_FB: Record<string, string> = {
		today: 'walked today', week: 'walked this week', month: 'walked this month',
		quarter: 'walked this quarter', older: 'walked long ago', never: 'never walked',
	};
	function shellLabel(s: number): string {
		const key = RECENCY_SHELLS[s]?.key ?? 'older';
		return L('cockpit.orrery.recency.' + key + '.long', SHELL_FB[key] ?? key);
	}
	// Concise ring labels — the six time bands drawn ON the chart, tied to their rings.
	const RING_FB: Record<string, string> = {
		today: 'today', week: 'this week', month: 'this month',
		quarter: 'this quarter', older: 'older', never: 'never walked',
	};
	function ringLabelShort(s: number): string {
		const key = RECENCY_SHELLS[s]?.key ?? 'older';
		return L('cockpit.orrery.recency.' + key + '.short', RING_FB[key] ?? key);
	}

	// confidence halo — encoded by RING STYLE only (hue stays the relation's own colour).
	function confHalo(conf: string): { add: number; sw: number; dash: string }[] {
		switch (conf) {
			case 'evidence': return [{ add: 2.4, sw: 0.9, dash: '' }];               // thin solid
			case 'established': return [{ add: 2.8, sw: 1.8, dash: '' }];             // thick solid
			case 'contested': return [{ add: 2.2, sw: 0.9, dash: '1.6 1.4' }, { add: 4.0, sw: 0.9, dash: '1.6 1.4' }]; // dashed-double
			case 'hypothesis':
			default: return [{ add: 2.4, sw: 0.8, dash: '0.4 2' }];                   // dotted
		}
	}

	type Hit = { kind: 'body' | 'fan' | 'cluster'; p: number; i: number; key: string };

	let W = $state(0), H = $state(0);
	let hover = $state(null as Hit | null);
	let focusSel = $state(null as Hit | null);
	// mouse latch: keeps a cluster's fan mounted across the empty gap between the '+N' glyph and its
	// members (they sit 8–16px outward with no data-k in between). Set on cluster/fan hover or a
	// cluster click; cleared on svg pointerleave, on moving onto a normal body, or on keyboard nav.
	let latch = $state(null as { p: number; key: string } | null);
	// Hover-expand — the wing (link TYPE) the pointer is over, and the one to EXPAND. Carries a noteKey
	// so a value left stale by a click-navigate (the note changed under a stationary pointer) can never
	// pre-expand the wrong note's wing. The setter below only reassigns when the wing/key actually
	// changes, so a pointermove that stays inside one wing does NOT invalidate the geometry — the
	// derivation reads this $state but does zero per-frame work.
	let hoverWing = $state(null as { wing: string; key: string } | null);

	let total = $derived(backlinks.length + outgoing.length);
	let hasAny = $derived(total > 0);
	// identity token: any of these changing means a different note is on screen.
	let noteKey = $derived(noteName + '|' + backlinks.length + '|' + outgoing.length);

	function setHoverWing(wing: string | null) {
		const cur = hoverWing;
		if (!wing) { if (cur) hoverWing = null; return; }
		if (cur && cur.wing === wing && cur.key === noteKey) return;   // same wing, same note → no invalidation
		hoverWing = { wing, key: noteKey };
	}

	// ── ALL geometry lives here — one derivation, no $effect, no per-frame work. It reads the hovered
	//    wing: changing WHICH wing is hovered recomputes the layout (expand + re-proportion), nothing
	//    else does. ─────────────────────────────────────────────────────────────────────────────────
	let model = $derived.by(() => {
		void $linkTypesStore;   // recolour / re-order when the link-type vocabulary changes
		const cx = Math.round(W / 2), cy = Math.round(H / 2);
		const now = Date.now();
		const TAU = 2 * Math.PI;
		const nk = noteKey;
		const hw = (hoverWing && hoverWing.key === nk) ? hoverWing.wing : null;

		const rtlTitle = detectDir(clean(noteName) || noteName) === 'rtl';
		let title = clean(noteName);
		if (title.length > 22) title = rtlTitle ? '…' + title.slice(title.length - 22) : title.slice(0, 22) + '…';

		// ── link records ──────────────────────────────────────────────────────────────────────
		const mkRec = (lk: any, dr: 'in' | 'out') => {
			const isOut = dr === 'out';
			const type = normalizeType(lk.linkType);
			const shell = recencyShell(lk, now);
			const w = earnedWeight(lk);
			const trav = lk.traversalCount ?? 0;
			const tier = String(lk.tier || '').toLowerCase();
			const conf = (String(lk.confidence || '').toLowerCase()) || 'hypothesis';
			const res = isOut ? (resolveTarget?.(lk.target ?? '') ?? { path: '', libraryName: lk.libraryName }) : { path: lk.path, libraryName: lk.libraryName };
			const cold = shell >= 4;                                    // "gone cold" = older orbit or the never rim
			const alarm = tier === 'load-bearing' && trav > 0 && cold;  // heavy + walked + stranded outer (incl. NEVER rim)
			const alive = shell <= 1;                                   // recently walked → always individually visible
			return {
				dir: dr, type, shell, w, tier, conf,
				name: clean(isOut ? lk.target : lk.name), path: res.path, lib: res.libraryName || lk.libraryName,
				alarm, mustKeep: tier === 'load-bearing' || alarm || alive,
			};
		};
		const recs: any[] = [];
		for (const lk of backlinks) recs.push(mkRec(lk, 'in'));
		for (const lk of outgoing) recs.push(mkRec(lk, 'out'));

		// present types (canonical order); every present type kept ⇒ no link dropped
		const present = orderTypes([...new Set(recs.map((r) => r.type))]);
		const T = present.length;
		const countByType: Record<string, number> = {};
		const countByTypeShell: Record<string, number[]> = {};
		for (const r of recs) {
			countByType[r.type] = (countByType[r.type] || 0) + 1;
			const arr = countByTypeShell[r.type] || (countByTypeShell[r.type] = [0, 0, 0, 0, 0, 0]);
			arr[r.shell]++;
		}

		// ── radial frame — a SMALL sun with a small inner dead-zone so the orbits get far more radial
		//    budget and 'today' sits close to the centre; a modest outer margin (rim reserve sized to
		//    the LONGEST localized label) so long locales never clip and the rim/count labels have room.
		//    Router reserves the ENLARGED dot ceiling so an enlarged outer-shell dot never clips the rim.
		const maxLabelPx = T ? Math.max(...present.map((tp) => estW(relLabelIn($locale, tp) + ' · ' + countByType[tp]))) : 0;
		const rimReserve = clamp(26 + 0.7 * maxLabelPx, 48, Math.max(48, Math.min(cx, cy) * 0.5));
		const Rmax = Math.max(40, Math.min(cx, cy) - rimReserve);
		const Rsun = clamp(Math.min(W, H) * 0.035, 9, 18);             // small central sun (kept)
		const gap = clamp(Rsun * 0.5, 4, 10);                          // small inner dead-zone (kept)
		const R0 = Math.min(Rsun + gap + 4, Rmax - 8);                 // innermost orbit (today) close to centre
		const BODY_MAX = clamp(Rmax * 0.055, 5, 9);                    // RESTING dot ceiling — normal, readable (v2 sizing)
		const ENL_MAX = clamp(Rmax * 0.11, 9, 16);                     // ENLARGED dot ceiling for the HOVERED wing (always > BODY_MAX)
		const Router = Math.max(R0 + 8, Rmax - (ENL_MAX + 3));         // reserve so an enlarged outer dot clears the rim
		const orbitR: number[] = [];
		for (let s = 0; s < 6; s++) orbitR.push(R0 + (Router - R0) * (s / 5)); // equal radial budget per band, recent inside

		const EMPTY = { cx, cy, Rsun, Rmax, orbitR, title, ringMode: 'none' as string, ringLabels: [] as any[], ringLegend: [] as any[], legendBox: null as any, sectors: [] as any[], cells: [] as any[], navTargets: [] as any[] };
		if (!T) return EMPTY;

		// ── dot sizing — RESTING wings use the NORMAL ceiling (over-dense cells cluster in PASS B); the
		//    HOVERED wing uses ONE uniform ENLARGED radius across all its cells so weight ORDER is
		//    preserved wing-wide (finding: cross-cell inversion). Both are bounded so nothing overlaps. ─
		const gmaxW = Math.max(1, ...recs.map((r) => r.w));
		const restMax = BODY_MAX;
		const restMin = clamp(restMax * 0.5, 2, 3);                    // floor keeps the hollow-ring hole legible
		const enlMax = ENL_MAX;
		const enlMin = clamp(enlMax * 0.5, 3, 6);
		const bandR = (w: number, mn: number, mx: number, maxW: number) =>
			clamp(mn + (mx - mn) * Math.sqrt(w / Math.max(1e-6, maxW)), mn, mx);
		const slotMin = 2 * restMax * 1.08;                           // arc a resting body needs → clustering budget

		// ── Remark 2: the six recency time labels, tied to their ring lines. They ride the top label
		//    spoke, DE-COLLIDED so they never pile into an illegible stack on a small pane; when the pane
		//    is too short to letter six on the spoke, they fall back to a compact corner legend so every
		//    band is still named. A displaced chip draws a hairline tick to its true ring. ──────────────
		const bandGap = (Router - R0) / 5;
		const chipH = clamp(bandGap * 0.9, 10, 14);
		const rFont = clamp(chipH * 0.66, 7.5, 9.5);
		const minSep = chipH + 1.5;
		const botLimit = cy - Rsun - 10;                             // keep the innermost label off the sun pill
		const rawY = orbitR.map((r) => cy - r);                      // s=0 lowest (large y) … s=5 highest (small y)
		const dispY: number[] = new Array(6);
		let prevY = -Infinity;
		for (let s = 5; s >= 0; s--) { const y = Math.max(rawY[s], prevY + minSep); dispY[s] = y; prevY = y; } // push down, keep order
		const ringMode: string = dispY[0] <= botLimit ? 'spoke' : 'legend';
		let ringLabels: any[] = [];
		let ringLegend: any[] = [];
		let legendBox: any = null;
		if (ringMode === 'spoke') {
			ringLabels = orbitR.map((r, s) => {
				const text = ringLabelShort(s);
				const w = estW(text) * 0.9 + 12;
				return {
					s, cold: s >= 4, text, font: r2(rFont), h: r2(chipH),
					chipX: r2(cx - w / 2), chipY: r2(dispY[s] - chipH / 2), chipW: r2(w),
					tx: cx, ty: r2(dispY[s]),
					tick: Math.abs(dispY[s] - rawY[s]) > 1.5, tickY1: r2(dispY[s]), tickY2: r2(rawY[s]),
				};
			});
		} else {
			const lh = clamp(chipH, 10, 12);
			const lfont = r2(clamp(lh * 0.72, 8, 9.5));
			const lpad = 5;
			ringLegend = orbitR.map((r, s) => ({ s, cold: s >= 4, text: ringLabelShort(s), font: lfont, tx: 11, ty: r2(3 + lpad + s * lh + lh / 2) }));
			const lw = Math.max(46, ...ringLegend.map((e: any) => estW(e.text))) + 14;
			legendBox = { x: 3, y: 3, w: r2(lw), h: r2(2 * lpad + 6 * lh) };
		}

		// ── Remark 2: the top-spoke lane the recency chips own, + a helper that pushes any colliding
		//    wing content (a '+N' cluster glyph, a rim label) clear of it. First it shoves horizontally
		//    to the nearer side; if a narrow pane clamps that back onto the lane, it falls back to a
		//    RADIAL offset (lifts the box above the lane's outer edge) so separation is guaranteed on
		//    any width. In legend mode there is no spoke, so the lane is null and the helper is a no-op. ─
		let topLane: { x0: number; x1: number; y0: number; y1: number } | null = null;
		if (ringMode === 'spoke' && ringLabels.length) {
			let lx0 = Infinity, lx1 = -Infinity, ly0 = Infinity, ly1 = -Infinity;
			for (const rl of ringLabels) {
				lx0 = Math.min(lx0, rl.chipX); lx1 = Math.max(lx1, rl.chipX + rl.chipW);
				ly0 = Math.min(ly0, rl.chipY); ly1 = Math.max(ly1, rl.chipY + rl.h);
			}
			topLane = { x0: lx0 - 5, x1: lx1 + 5, y0: ly0, y1: ly1 };
		}
		const clearLane = (x: number, y: number, halfW: number, halfH: number): [number, number] => {
			if (!topLane) return [x, y];
			const vOver = !(y + halfH < topLane.y0 || y - halfH > topLane.y1);
			const hOver = !(x + halfW < topLane.x0 || x - halfW > topLane.x1);
			if (!vOver || !hOver) return [x, y];
			const right = topLane.x1 + halfW + 2;
			const left = topLane.x0 - halfW - 2;
			let nx = (x >= cx) ? right : left;
			nx = clamp(nx, 6 + halfW, Math.max(6 + halfW, W - 6 - halfW));
			// did the clamp land it back on the lane? then lift it radially out of the lane's y-band.
			const stillH = !(nx + halfW < topLane.x0 || nx - halfW > topLane.x1);
			if (stillH) {
				const up = topLane.y0 - halfH - 2;
				const ny = clamp(up, 6 + halfH, Math.max(6 + halfH, H - 6 - halfH));
				return [x, ny];
			}
			return [nx, y];
		};

		// ── angular sectors — each present type's wedge WIDTH ∝ its link count, with a minimum floor so
		//    a 1–2 link type is still visible, hittable, and keeps its rim label. ───────────────────────
		const start = -Math.PI / 2;                                    // 12 o'clock
		const sign = ($dir === 'rtl') ? -1 : 1;
		const totalCount = Math.max(1, recs.length);
		const minFrac = Math.min(0.05, 0.72 / T);                      // floor per wing (≤ 0.72 total, so proportional pool > 0)
		const propFrac = present.map((tp) => minFrac + (1 - T * minFrac) * (countByType[tp] / totalCount));

		// ── HOVER-EXPAND (Boss remarks). The hovered wing ALWAYS becomes a zoom-in — its dots enlarge and
		//    its cells never cluster (findings: every hovered wing must respond; T === 1 must still
		//    enlarge/decluster). STEP B (T > 1 only): grow the wing's ANGULAR width so its fullest cell
		//    seats every node at the enlarged dot with even spacing — per-node arc = enlMax/0.46 + gap
		//    (matched to the arc-fit below) — clamped to [proportional, 0.92]; the OTHER wings share the
		//    remaining angle, floored > 0. A single-type note already fills the circle, so its width
		//    can't grow, but expandType is still set → its dots enlarge and its clusters dissolve. The
		//    actual enlarged RADIUS is derived per-wing after the sectors are laid out (see wingCap). ────
		const NODE_GAP = 2;                                            // px breathing room between two enlarged dots
		const perNode = (dot: number) => dot / 0.46 + NODE_GAP;       // arc a radius-`dot` body needs (matches arc-fit)
		let fracs = propFrac;
		let expandType: string | null = null;
		if (hw && present.includes(hw)) {
			const k = present.indexOf(hw);
			expandType = hw;                                          // hovered wing always zooms + declusters
			if (T > 1) {
				const shells = countByTypeShell[hw] || [];
				let needFrac = propFrac[k];
				for (let s = 0; s < 6; s++) {
					const n = shells[s] || 0;
					if (n > 0) needFrac = Math.max(needFrac, (n * perNode(enlMax)) / (Math.max(1, orbitR[s]) * TAU));
				}
				needFrac = clamp(needFrac * 1.14 + 0.03, propFrac[k], 0.92);   // + wedge pad / inter-sector gap headroom
				if (needFrac > propFrac[k] + 0.004) {
					const rest = 1 - needFrac;
					const otherCount = Math.max(1, totalCount - countByType[hw]);
					const minO = Math.min(0.03, (rest * 0.6) / Math.max(1, T - 1));
					fracs = present.map((tp, i) =>
						i === k ? needFrac : minO + (rest - (T - 1) * minO) * (countByType[tp] / otherCount));
				}
			}
		}

		const gapRad = clamp(Math.min(...fracs) * TAU * 0.14, 0.008, 0.05);
		const pt = (r: number, th: number): [number, number] => [cx + r * Math.cos(th), cy + r * Math.sin(th)];

		let acc = 0;
		const sectors = present.map((type: string, k: number) => {
			const f = fracs[k];
			const e0 = start + sign * acc * TAU;
			const e1 = start + sign * (acc + f) * TAU;
			acc += f;
			let lo = Math.min(e0, e1) + gapRad / 2;
			let hi = Math.max(e0, e1) - gapRad / 2;
			if (hi <= lo) { const m0 = (Math.min(e0, e1) + Math.max(e0, e1)) / 2; lo = m0 - 0.004; hi = m0 + 0.004; }
			const center = (lo + hi) / 2;
			// faint wedge path (true annular sector, hairline) — spans the sun rim to the outer rim
			const rIn = Rsun + 3, rOut = Rmax;
			const [x0, y0] = pt(rIn, lo), [x1, y1] = pt(rIn, hi), [x2, y2] = pt(rOut, hi), [x3, y3] = pt(rOut, lo);
			const la = (hi - lo) > Math.PI ? 1 : 0;
			const wedge = `M${r2(x0)} ${r2(y0)} A${r2(rIn)} ${r2(rIn)} 0 ${la} 1 ${r2(x1)} ${r2(y1)} L${r2(x2)} ${r2(y2)} A${r2(rOut)} ${r2(rOut)} 0 ${la} 0 ${r2(x3)} ${r2(y3)} Z`;
			// boundary hairline at the sector's low edge
			const [bx1, by1] = pt(rIn, lo), [bx2, by2] = pt(rOut, lo);
			// rim label — anchored by quadrant, then clamped so its box always stays on-canvas
			const label = relLabelIn($locale, type);
			const count = countByType[type];
			const lw = estW(label + ' · ' + count);
			const cCos = Math.cos(center);
			const anchor = cCos > 0.25 ? 'start' : cCos < -0.25 ? 'end' : 'middle';
			let lx = cx + (Rmax + 10) * cCos;
			let ly = cy + (Rmax + 10) * Math.sin(center);
			if (anchor === 'start') lx = Math.min(lx, W - 6 - lw);
			else if (anchor === 'end') lx = Math.max(lx, 6 + lw);
			else lx = clamp(lx, 6 + lw / 2, W - 6 - lw / 2);
			lx = clamp(lx, 6, W - 6);
			ly = clamp(ly, 12, H - 8);
			// Remark 2: push a rim label off the top ring-label spoke, then RE-CLAMP to canvas (the
			// nudge shifts an edge-anchored label's x, which could otherwise sit off the rim).
			const lHalf = lw / 2;
			const boxCx = anchor === 'start' ? lx + lHalf : anchor === 'end' ? lx - lHalf : lx;
			const [clearedCx, clearedY] = clearLane(boxCx, ly, lHalf, 8);
			lx = lx + (clearedCx - boxCx);
			ly = clearedY;
			if (anchor === 'start') lx = Math.min(lx, W - 6 - lw);
			else if (anchor === 'end') lx = Math.max(lx, 6 + lw);
			else lx = clamp(lx, 6 + lw / 2, W - 6 - lw / 2);
			lx = clamp(lx, 6, W - 6);
			ly = clamp(ly, 12, H - 8);
			return {
				type, color: relColor(type), label, count, expanded: expandType === type,
				lo, hi, mid: center, wedge,
				bx1: r2(bx1), by1: r2(by1), bx2: r2(bx2), by2: r2(by2),
				lx: r2(lx), ly: r2(ly + 3), anchor,
			};
		});

		// ── PASS A — gather every (type × shell) cell's arc-length + node count for placement. ───────────
		const cellSpecs: any[] = [];
		for (const sec of sectors) {
			for (let s = 0; s < 6; s++) {
				const links = recs.filter((r) => r.type === sec.type && r.shell === s);
				if (!links.length) continue;
				const r_s = orbitR[s];
				const pad = clamp(0.05 * (sec.hi - sec.lo), 0.01, 0.09);
				const usable = Math.max(0.02, (sec.hi - sec.lo) - 2 * pad);
				const arc = Math.max(1, r_s * usable);
				cellSpecs.push({ sec, s, links, r_s, pad, usable, arc });
			}
		}

		// ── the ONE uniform enlarged radius for the hovered wing: the min per-cell arc-fit across all its
		//    (non-empty) cells — every cell can seat its nodes at this radius with even spacing, and it
		//    steps DOWN from enlMax only when the widest cell can't reach it even at the 0.92 width cap.
		//    Applying it uniformly keeps a heavier node from ever rendering smaller than a lighter one
		//    across cells (weight ORDER preserved wing-wide). ──────────────────────────────────────────
		let wingCap = enlMax;
		let wingMaxW = 1;
		if (expandType) {
			wingMaxW = Math.max(1, ...recs.filter((r) => r.type === expandType).map((r) => r.w));
			for (const cs of cellSpecs) {
				if (cs.sec.type !== expandType) continue;
				wingCap = Math.min(wingCap, 0.46 * (cs.arc / Math.max(1, cs.links.length)));
			}
			wingCap = clamp(wingCap, 0.6, enlMax);
		}
		const wingMin = Math.min(wingCap, enlMin);

		const mkBody = (r: any, x: number, y: number, s: number, radius: number) => ({
			name: r.name, path: r.path, lib: r.lib, dir: r.dir, type: r.type, conf: r.conf,
			tier: r.tier, w: r.w, shell: s, alarm: r.alarm, never: s === NEVER_SHELL,
			x: r2(x), y: r2(y), rB: r2(radius), halo: confHalo(r.conf),
		});

		// ── PASS B — place bodies by even angular division. RESTING wing: a cell over its slot budget
		//    keeps a counted '+N' cluster (nothing dropped — fans on hover/click) and its dots are sized
		//    per-cell up to restMax. HOVERED wing: budget = every node → NEVER clusters; all its dots
		//    share the uniform enlarged radius band [wingMin, wingCap], which fits every cell. ───────────
		const cells: any[] = [];
		const navTargets: any[] = [];
		const byName = (a: any, b: any) => a.name.localeCompare(b.name);

		for (const cs of cellSpecs) {
			const { sec, s, links, r_s, pad, usable, arc } = cs;
			const lo = sec.lo, hi = sec.hi;
			const expanded = sec.type === expandType;
			const sorted = links.slice().sort(byName);
			// the hovered wing shows EVERY node (no cluster); a resting wing clusters an over-dense cell.
			const budget = expanded ? sorted.length : Math.max(1, Math.floor(arc / slotMin));

			let individual: any[] = sorted, clustered: any[] = [];
			if (sorted.length > budget) {
				const must = sorted.filter((r: any) => r.mustKeep);
				const opt = sorted.filter((r: any) => !r.mustKeep).sort((a: any, b: any) => b.w - a.w); // collapse the LIGHTEST first
				let keep: any[]; let drop: any[];
				if (must.length <= budget) {
					const keepOpt = budget - must.length;
					keep = [...must, ...opt.slice(0, keepOpt)];
					drop = opt.slice(keepOpt);
				} else {
					// LAST RESORT: even mustKeep alone cannot fit — cluster the lightest excess so nothing
					// overlaps (the synthetic 900-in-one-cell case). Never reached by realistic data.
					const m2 = must.slice().sort((a: any, b: any) => b.w - a.w);       // heaviest kept
					keep = m2.slice(0, budget);
					drop = [...opt, ...m2.slice(budget)];
				}
				if (drop.length >= 2) {                            // never a '+1' cluster — one leftover renders inline
					clustered = drop;
					individual = keep.slice().sort(byName);
				} else {
					individual = sorted;
				}
			}
			const hasCluster = clustered.length > 0;
			const slots = individual.length + (hasCluster ? 1 : 0);
			const slotArc = arc / Math.max(1, slots);
			// resting per-cell arc-fit — floor low enough to shrink to fit rather than overlap on a
			// narrow pane (finding: hard 1.4 floor caused overlap); the hovered wing uses wingCap instead.
			const restCap = Math.min(restMax, Math.max(0.6, 0.46 * slotArc));
			const ci = cells.length;

			const bodies = individual.map((r, k) => {
				const th = lo + pad + ((k + 0.5) / slots) * usable;
				const [x, y] = pt(r_s, th);
				const radius = expanded
					? bandR(r.w, wingMin, wingCap, wingMaxW)             // uniform enlarged band → weight order preserved
					: Math.min(bandR(r.w, restMin, restMax, gmaxW), restCap);
				return mkBody(r, x, y, s, radius);
			});

			let cluster: any = null;
			if (hasCluster) {
				const th = lo + pad + ((slots - 0.5) / slots) * usable;
				const [gx0, gy0] = pt(r_s, th);
				const M = clustered.length;
				const glyphR = clamp(BODY_MAX * 0.95, 6.5, 8);
				const glyphHW = Math.max(glyphR, ('+' + M).length * 3.2);   // the '+N' text runs wider than the disc
				// Remark 2: push the '+N' glyph off the top ring-label spoke; rigid-translate its fan by
				// the same (dx, dy) so the connector lines and members stay consistent.
				const [gx, gy] = clearLane(gx0, gy0, glyphHW, glyphR);
				const dx = gx - gx0, dy = gy - gy0;
				const rFan = r_s + clamp(Rmax * 0.03, 8, 16);       // fan pops just outside the orbit
				let stepA = 14 / rFan;
				if ((M - 1) * stepA > 1.7) stepA = 1.7 / Math.max(1, M - 1);
				const startA = th - ((M - 1) * stepA) / 2;
				const fanCap = Math.min(restMax, Math.max(0.6, 0.46 * rFan * stepA));
				const members = clustered.slice().sort(byName).map((r, m) => {
					const [mx, my] = pt(rFan, startA + m * stepA);
					const cxp = clamp(mx + dx, 6, Math.max(6, W - 6));
					const cyp = clamp(my + dy, 6, Math.max(6, H - 6));
					return mkBody(r, cxp, cyp, s, Math.min(bandR(r.w, restMin, restMax, gmaxW), fanCap));
				});
				cluster = { n: M, gx: r2(gx), gy: r2(gy), r: r2(glyphR), members };
			}

			cells.push({ p: ci, s, type: sec.type, color: sec.color, bodies, cluster });
			bodies.forEach((_, bi) => navTargets.push({ kind: 'body', p: ci, i: bi }));
			if (cluster) cluster.members.forEach((_: any, mi: number) => navTargets.push({ kind: 'fan', p: ci, i: mi }));
		}

		return { cx, cy, Rsun, Rmax, orbitR, title, ringMode, ringLabels, ringLegend, legendBox, sectors, cells, navTargets };
	});

	// centre pill (drawn atop the orbits so the note title stays legible over hairlines)
	let pill = $derived.by(() => {
		const w = clamp(model.title.length * 7.4 + 26, 84, Math.min(230, model.Rmax * 1.1));
		return { w: r2(w), x: r2(model.cx - w / 2) };
	});

	// ── active selection (hover OR keyboard focus) → one O(1) overlay; stale key resolves null. ──
	let act = $derived.by(() => {
		const a = hover ?? focusSel;
		if (!a || a.key !== noteKey) return null;
		const c = model.cells[a.p]; if (!c) return null;
		if (a.kind === 'cluster') {
			const cl = c.cluster; if (!cl) return null;
			return { kind: 'cluster' as const, p: a.p, cell: c, cl, mark: null as any, x: cl.gx, y: cl.gy, r: cl.r, color: c.color };
		}
		if (a.kind === 'fan') {
			const cl = c.cluster; const m = cl?.members[a.i]; if (!cl || !m) return null;
			return { kind: 'fan' as const, p: a.p, cell: c, cl, mark: m, x: m.x, y: m.y, r: m.rB, color: c.color };
		}
		const m = c.bodies[a.i]; if (!m) return null;
		return { kind: 'body' as const, p: a.p, cell: c, cl: null as any, mark: m, x: m.x, y: m.y, r: m.rB, color: c.color };
	});
	let isFocusActive = $derived(!hover && !!focusSel);

	// the fan is shown while the mouse LATCH holds a cell open (survives the glyph→member gap) OR a
	// cluster / fanned member is the active hover / keyboard focus.
	let fanView = $derived.by(() => {
		let p = -1;
		if (latch && latch.key === noteKey) p = latch.p;
		else {
			const a = hover ?? focusSel;
			if (a && a.key === noteKey && (a.kind === 'cluster' || a.kind === 'fan')) p = a.p;
		}
		if (p < 0) return null;
		const c = model.cells[p]; const cl = c?.cluster; if (!cl) return null;
		return { members: cl.members as any[], color: c.color as string, type: c.type as string, p, gx: cl.gx as number, gy: cl.gy as number };
	});

	let plate = $derived.by(() => {
		const a = act; if (!a) return null;
		let l1 = '', l2 = '', l3 = '';
		if (a.kind === 'cluster') {
			l1 = '+' + a.cl.n;
			l2 = relLabelIn($locale, a.cell.type) + ' · ' + L('cockpit.orrery.clustered', 'clustered links');
			l3 = shellLabel(a.cell.s);
		} else {
			const m = a.mark;
			l1 = m.name || '';
			l2 = relLabelIn($locale, m.type) + ' · ' + (m.dir === 'out' ? L('cockpit.outgoing', 'outgoing') : L('cockpit.incoming', 'incoming'));
			l3 = shellLabel(m.shell) + ' · ' + L('cockpit.orrery.weight', 'weight') + ' ' + (Math.round(m.w * 10) / 10);
		}
		const lines = [l1, l2, l3];
		const plateW = Math.max(60, ...lines.map((s) => s.length * 6.4)) + 16;
		const plateH = 46;
		let px = a.x < model.cx ? a.x + a.r + 10 : a.x - a.r - 10 - plateW;
		if (px < 6) px = Math.min(a.x + a.r + 10, W - 6 - plateW);
		if (px < 6) px = 6;
		if (px + plateW > W - 6) px = W - 6 - plateW;
		let py = clamp(a.y - plateH / 2, 6, Math.max(6, H - 6 - plateH));
		return { x: r2(px), y: r2(py), w: r2(plateW), h: plateH, cx: r2(px + plateW / 2), lines };
	});

	// ── navigation + event delegation (4 listeners on the svg, ZERO per-node listeners) ─────────
	function markOf(sel: Hit): any {
		const c = model.cells[sel.p]; if (!c) return null;
		if (sel.kind === 'fan') return c.cluster?.members[sel.i] ?? null;
		return c.bodies[sel.i] ?? null;
	}
	function navigateSel(sel: Hit) {
		if (sel.kind === 'cluster') return;
		const m = markOf(sel);
		if (m?.path) onNavigate?.(m.path, m.name, m.lib || '');
	}
	function onMove(e: PointerEvent) {
		const tgt = e.target as Element;
		// which wing is the pointer over (wedge background OR a mark's data-wing) → drives the expand.
		// Decorative elements are pointer-transparent, so crossing a ring line / gap reads as "no
		// wing" and collapses cleanly; the setter dedupes so staying inside a wing is a no-op.
		const wingEl = tgt?.closest?.('[data-wing]') as any;
		setHoverWing(wingEl ? ((wingEl.dataset.wing as string) || null) : null);
		const el = tgt?.closest?.('[data-k]') as any;
		if (!el) { hover = null; return; }   // wedge background / glyph→member gap: keep latch, drop hover
		const kind = el.dataset.k as Hit['kind'];
		const p = +el.dataset.p;
		hover = { kind, p, i: +(el.dataset.i ?? '0'), key: noteKey };
		if (kind === 'cluster' || kind === 'fan') latch = { p, key: noteKey };
		else latch = null;                    // moved onto a normal body → close any latched fan
	}
	function onLeave() { hover = null; latch = null; setHoverWing(null); }
	function onClick(e: MouseEvent) {
		const el = (e.target as Element)?.closest?.('[data-k]') as any;
		if (!el) return;
		const kind = el.dataset.k as Hit['kind'];
		if (kind === 'cluster') { latch = { p: +el.dataset.p, key: noteKey }; return; }   // open the fan (also for touch)
		setHoverWing(null);                   // navigating away → never carry the wing to the next note
		navigateSel({ kind, p: +el.dataset.p, i: +(el.dataset.i ?? '0'), key: noteKey });
	}
	function onKey(e: KeyboardEvent) {
		if (e.key === 'Escape') { focusSel = null; latch = null; return; }
		const nt = model.navTargets;
		if (!nt.length) return;
		const isNav = e.key === 'ArrowUp' || e.key === 'ArrowDown' || e.key === 'ArrowLeft' || e.key === 'ArrowRight';
		const isAct = e.key === 'Enter' || e.key === ' ';
		if (!isNav && !isAct) return;
		e.preventDefault();
		latch = null;                         // keyboard takes over from any mouse-latched fan
		let idx = (focusSel && focusSel.key === noteKey)
			? nt.findIndex((x: any) => x.kind === focusSel!.kind && x.p === focusSel!.p && x.i === focusSel!.i) : -1;
		if (idx < 0) {
			const sel = { ...nt[0], key: noteKey } as Hit;
			focusSel = sel;
			if (isAct) navigateSel(sel);
			return;
		}
		if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') idx = (idx - 1 + nt.length) % nt.length;
		else if (e.key === 'ArrowDown' || e.key === 'ArrowRight') idx = (idx + 1) % nt.length;
		else if (isAct && focusSel) { navigateSel(focusSel); return; }
		focusSel = { ...nt[idx], key: noteKey };
	}
</script>

<div class="orr">
	<div class="orr-stage" bind:clientWidth={W} bind:clientHeight={H}>
		{#if W > 80 && H > 80}
			<!-- deliberate: the graph IS the widget (role=application), so it takes focus and handles
			     arrow-key/Enter navigation across links. Delegated listeners, not per-node. -->
			<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
			<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
			<svg class="orr-svg" viewBox="0 0 {W} {H}" role="application" tabindex="0"
				aria-label={hasAny ? `${L('cockpit.incoming', 'incoming')} ${backlinks.length}, ${L('cockpit.outgoing', 'outgoing')} ${outgoing.length}` : L('cockpit.noLinks', 'no links yet')}
				onpointermove={onMove} onpointerleave={onLeave} onclick={onClick} onkeydown={onKey}>

				<defs>
					<radialGradient id="sun-{uid}" cx="50%" cy="50%" r="50%">
						<stop offset="0%" stop-color="#E8944A" stop-opacity="0.30"/>
						<stop offset="55%" stop-color="#E8944A" stop-opacity="0.09"/>
						<stop offset="100%" stop-color="#E8944A" stop-opacity="0"/>
					</radialGradient>
					<!-- warmth = recency (declared REDUNDANT with the radial orbit axis; Boss ORRERY-FEEL
					     requirement): inner warm → outer cold, kept low-alpha so it reinforces not competes -->
					<radialGradient id="warm-{uid}" cx="50%" cy="50%" r="50%">
						<stop offset="0%" stop-color="#E8944A" stop-opacity="0.10"/>
						<stop offset="45%" stop-color="#D98F5A" stop-opacity="0.05"/>
						<stop offset="100%" stop-color="#5B84C4" stop-opacity="0.09"/>
					</radialGradient>
				</defs>

				<!-- warm→cold sky wash + the sun's glow, behind everything (decorative, no pointer target) -->
				<circle cx={model.cx} cy={model.cy} r={r2(model.Rmax)} fill="url(#warm-{uid})" pointer-events="none"/>
				<circle cx={model.cx} cy={model.cy} r={r2(model.orbitR[2] ?? model.Rmax)} fill="url(#sun-{uid})" pointer-events="none"/>

				<!-- count-proportional type wedges (the ONLY pointer targets besides the marks); the
				     hovered/expanded wing brightens -->
				{#each model.sectors as sec}
					<path class="orr-wedge" d={sec.wedge} data-wing={sec.type} fill={sec.color} style:fill-opacity={sec.expanded ? 0.14 : 0.06}/>
				{/each}
				<!-- sector boundary hairlines -->
				{#each model.sectors as sec}
					<line x1={sec.bx1} y1={sec.by1} x2={sec.bx2} y2={sec.by2} stroke="var(--background-modifier-border, #d4d4d8)" stroke-opacity="0.22" stroke-width="1" pointer-events="none"/>
				{/each}
				{#if hasAny}
					<!-- six recency orbits as CLEAR thin ring lines (outer rings fade colder) -->
					{#each model.orbitR as r, s}
						<circle class="orr-orbit" class:cold={s >= 4} cx={model.cx} cy={model.cy} r={r2(r)} fill="none"/>
					{/each}
				{/if}
				<!-- rim labels: type · count -->
				{#each model.sectors as sec}
					<text class="orr-rim" x={sec.lx} y={sec.ly} text-anchor={sec.anchor}><tspan fill={sec.color} font-weight="600">{sec.label}</tspan><tspan fill="var(--text-muted)"> · {sec.count}</tspan></text>
				{/each}

				<!-- bodies + cluster glyphs (delegated; dim when a selection is active) -->
				<g class="orr-marks" class:dimmed={!!act}>
					{#each model.cells as c, ci}
						{#each c.bodies as b, bi}
							<g class="orr-body" data-k="body" data-p={ci} data-i={bi} data-wing={c.type}>
								{#if b.alarm}
									<circle class="orr-alarm" cx={b.x} cy={b.y} r={r2(b.rB + 3.4)} fill="none"/>
								{/if}
								{#each b.halo as h}
									<circle cx={b.x} cy={b.y} r={r2(b.rB + h.add)} fill="none" stroke={c.color} stroke-opacity="0.7" stroke-width={h.sw} stroke-dasharray={h.dash || undefined}/>
								{/each}
								{#if b.dir === 'out'}
									<!-- outgoing → solid disc -->
									<circle cx={b.x} cy={b.y} r={b.rB} fill={c.color} fill-opacity={b.never ? 0.55 : 0.95} stroke="var(--background-primary)" stroke-width="0.5"/>
								{:else}
									<!-- backlink → hollow ring (stroke ≤ 0.34·rB keeps a legible hole at any size) -->
									<circle cx={b.x} cy={b.y} r={b.rB} fill="var(--background-primary)" stroke={c.color} stroke-width={r2(clamp(b.rB * 0.34, 0.7, 2))} opacity={b.never ? 0.82 : 1}/>
								{/if}
							</g>
						{/each}
						{#if c.cluster}
							<g class="orr-cluster" data-k="cluster" data-p={ci} data-wing={c.type}>
								<circle cx={c.cluster.gx} cy={c.cluster.gy} r={c.cluster.r} fill="var(--background-secondary, #f4f4f5)" stroke={c.color} stroke-width="1"/>
								<text class="orr-cn" x={c.cluster.gx} y={r2(c.cluster.gy + 3)} text-anchor="middle">+{c.cluster.n}</text>
							</g>
						{/if}
					{/each}
				</g>

				<!-- the active cluster's members, fanned onto a temporary sub-arc (latched; each navigable) -->
				{#if fanView}
					<g>
						{#each fanView.members as m}
							<line x1={fanView.gx} y1={fanView.gy} x2={m.x} y2={m.y} stroke={fanView.color} stroke-opacity="0.3" stroke-width="0.7" pointer-events="none"/>
						{/each}
						{#each fanView.members as m, mi}
							<g class="orr-body" data-k="fan" data-p={fanView.p} data-i={mi} data-wing={fanView.type}>
								{#if m.dir === 'out'}
									<circle cx={m.x} cy={m.y} r={m.rB} fill={fanView.color} stroke="var(--background-primary)" stroke-width="0.6"/>
								{:else}
									<circle cx={m.x} cy={m.y} r={m.rB} fill="var(--background-primary)" stroke={fanView.color} stroke-width={r2(clamp(m.rB * 0.34, 0.7, 2))}/>
								{/if}
							</g>
						{/each}
					</g>
				{/if}

				<!-- Remark 2: the recency time labels, tied to their rings (spoke ruler, de-collided; or a
				     compact corner legend when the pane is too short). On top, pointer-transparent. -->
				{#if hasAny && model.ringMode === 'spoke'}
					<g class="orr-rings" pointer-events="none">
						<line class="orr-spine" x1={model.cx} y1={r2(model.cy - model.orbitR[5])} x2={model.cx} y2={r2(model.cy - model.orbitR[0])}/>
						{#each model.ringLabels as rl}
							{#if rl.tick}
								<line class="orr-tick" x1={model.cx} y1={rl.tickY1} x2={model.cx} y2={rl.tickY2}/>
							{/if}
							<rect class="orr-ring-chip" x={rl.chipX} y={rl.chipY} width={rl.chipW} height={rl.h} rx={r2(rl.h / 2)}/>
							<text class="orr-ring-lbl" class:cold={rl.cold} x={rl.tx} y={rl.ty} text-anchor="middle" style:font-size={rl.font + 'px'}>{rl.text}</text>
						{/each}
					</g>
				{:else if hasAny && model.ringMode === 'legend'}
					<g class="orr-rings" pointer-events="none">
						<rect class="orr-legend-box" x={model.legendBox.x} y={model.legendBox.y} width={model.legendBox.w} height={model.legendBox.h} rx="5"/>
						{#each model.ringLegend as le}
							<text class="orr-ring-lbl" class:cold={le.cold} x={le.tx} y={le.ty} text-anchor="start" style:font-size={le.font + 'px'}>{le.text}</text>
						{/each}
					</g>
				{/if}

				<!-- centre sun: the note (drawn atop orbits so the title stays legible) -->
				<rect class="orr-pill" x={pill.x} y={r2(model.cy - 13)} width={pill.w} height="26" rx="13"/>
				<text class="orr-title" x={model.cx} y={r2(model.cy + 4)} text-anchor="middle">{model.title}</text>
				<!-- number + UI-language word: force the subtitle's own base direction so Arabic reads "140 رابطًا" -->
				<text class="orr-sub" x={model.cx} y={r2(model.cy + 22)} text-anchor="middle" style:direction={$dir}>{$tn('plurals.links', total)}</text>
				{#if !hasAny}
					<text class="orr-empty" x={model.cx} y={r2(model.cy + 42)} text-anchor="middle">{L('cockpit.noLinks', 'no links yet')}</text>
				{/if}

				<!-- bright O(1) overlay for the active mark: re-draw it at FULL opacity (the field dims to
				     0.2, so the outline alone would leave the selected body desaturated) + info plate -->
				{#if act}
					<g pointer-events="none">
						{#if act.kind === 'cluster'}
							<circle cx={act.x} cy={act.y} r={act.r} fill="var(--background-secondary, #f4f4f5)" stroke={act.color} stroke-width="1"/>
							<text class="orr-cn" x={act.x} y={r2(act.y + 3)} text-anchor="middle">+{act.cl.n}</text>
						{:else if act.mark.dir === 'out'}
							<circle cx={act.x} cy={act.y} r={act.r} fill={act.color} stroke="var(--background-primary)" stroke-width="0.6"/>
						{:else}
							<circle cx={act.x} cy={act.y} r={act.r} fill="var(--background-primary)" stroke={act.color} stroke-width={r2(clamp(act.r * 0.34, 0.9, 2))}/>
						{/if}
						{#if isFocusActive}
							<circle cx={act.x} cy={act.y} r={r2(act.r + 3.6)} fill="none" stroke="var(--interactive-accent)" stroke-width="1.6"/>
						{/if}
						<circle cx={act.x} cy={act.y} r={r2(act.r + 1)} fill="none" stroke="var(--text-normal)" stroke-opacity="0.85" stroke-width="1"/>
						{#if plate}
							<rect class="orr-plate" x={plate.x} y={plate.y} width={plate.w} height={plate.h} rx="6"/>
							<text class="orr-pn" x={plate.cx} y={r2(plate.y + 15)} text-anchor="middle">{plate.lines[0]}</text>
							<text class="orr-pm" x={plate.cx} y={r2(plate.y + 29)} text-anchor="middle">{plate.lines[1]}</text>
							<text class="orr-pm" x={plate.cx} y={r2(plate.y + 42)} text-anchor="middle">{plate.lines[2]}</text>
						{/if}
					</g>
				{/if}
			</svg>
		{/if}
	</div>

	<NoteGaugeDeck {content} {review} {backlinks} {outgoing} />
</div>

<style>
	.orr { display: flex; flex-direction: column; width: 100%; height: 100%; min-height: 0; }
	.orr-stage { flex: 1; min-height: 0; width: 100%; background: var(--background-primary, #fff); }
	/* The cockpit wrapper is dir={$dir}; SVG text-anchor resolves against the inline base direction,
	   so under RTL "start" flips to the right edge. The geometry is LTR by definition, so pin the SVG
	   to ltr and let each label shape itself via unicode-bidi: plaintext. */
	.orr-svg { width: 100%; height: 100%; display: block; outline: none; direction: ltr; }
	/* Clear, thin, theme-aware ring lines (the old 1 4 dash read near-invisible). */
	.orr-orbit { stroke: var(--background-modifier-border, #d4d4d8); stroke-opacity: 0.6; stroke-width: 1; pointer-events: none; }
	.orr-orbit.cold { stroke-opacity: 0.4; }
	.orr-wedge { transition: fill-opacity 0.15s ease; }
	.orr-body { cursor: pointer; }
	.orr-cluster { cursor: pointer; }
	.orr-marks.dimmed { opacity: 0.2; transition: opacity 0.12s; }
	.orr-cn { font: 600 9px var(--font-sans); fill: var(--text-muted, #6b7280); unicode-bidi: plaintext; }
	.orr-rim { font: 11px var(--font-sans); dominant-baseline: middle; unicode-bidi: plaintext; pointer-events: none; }
	/* Remark 2: the recency label spoke + its ring chips / corner legend. */
	.orr-spine { stroke: var(--background-modifier-border, #d4d4d8); stroke-opacity: 0.35; stroke-width: 1; stroke-dasharray: 1 3; }
	.orr-tick { stroke: var(--background-modifier-border, #d4d4d8); stroke-opacity: 0.5; stroke-width: 1; }
	.orr-ring-chip { fill: var(--background-primary, #fff); fill-opacity: 0.72; stroke: var(--background-modifier-border, #d4d4d8); stroke-opacity: 0.6; }
	.orr-legend-box { fill: var(--background-primary, #fff); fill-opacity: 0.82; stroke: var(--background-modifier-border, #d4d4d8); stroke-opacity: 0.6; }
	.orr-ring-lbl { font-family: var(--font-sans); font-weight: 600; fill: var(--text-muted, #6b7280); dominant-baseline: middle; unicode-bidi: plaintext; }
	.orr-ring-lbl.cold { fill: var(--text-faint, #9ca3af); }
	.orr-pill { fill: var(--background-secondary, #f4f4f5); stroke: var(--background-modifier-border, #d4d4d8); }
	.orr-title { font: 600 12px var(--font-text, var(--font-sans)); fill: var(--text-normal, #1a1a1a); unicode-bidi: plaintext; }
	.orr-sub { font: 10px var(--font-sans); fill: var(--text-muted, #6b7280); }
	.orr-empty { font: 13px var(--font-sans); fill: var(--text-muted, #6b7280); }
	.orr-plate { fill: var(--background-primary, #fff); fill-opacity: 0.96; stroke: var(--background-modifier-border, #d4d4d8); }
	.orr-pn { font: 600 12px var(--font-sans); fill: var(--text-normal, #1a1a1a); unicode-bidi: plaintext; }
	.orr-pm { font: 10.5px var(--font-sans); fill: var(--text-muted, #6b7280); unicode-bidi: plaintext; }
	/* ALARM: a STATUS-coloured ring (distinct from every relationship hue) with a slow breathe-pulse. */
	.orr-alarm { stroke: var(--text-error, var(--text-warning, #E5701F)); stroke-width: 1.6; animation: orr-breathe 2.6s ease-in-out infinite; }
	@keyframes orr-breathe { 0%, 100% { stroke-opacity: 0.9; } 50% { stroke-opacity: 0.28; } }
	@media (prefers-reduced-motion: reduce) { .orr-alarm { animation: none; } }
</style>
