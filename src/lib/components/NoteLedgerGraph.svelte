<script lang="ts">
	/**
	 * PJ-068 v2 — "The Ledger" note-graph lens.
	 *
	 * Knowledge on the scales: the note's links as a balance sheet. A hard central rail is the
	 * zero line; each typed relationship is one row in fixed canonical order, IDENTICAL top-to-
	 * bottom on both sides so rows align across the spine for at-a-glance diffing. Backlink bars
	 * grow LEFT, outgoing grow RIGHT, length = count against a shared scale — so the 445-vs-513
	 * asymmetry reads as unequal column mass. No radial geometry: it can never be mistaken for a
	 * disc. Click a bar to open its individual links; click one → the MAIN window navigates.
	 * Theme-aware; --rel-* for the Style Setter.
	 */
	import { t, locale } from '$lib/i18n';
	import { groupByType, relColor, relLabelIn, orderTypes, tierW, clean, deriveStats } from '$lib/cockpitGraphData';
	import { linkTypesStore } from '$lib/libraries/linkTypeRegistry';
	import NoteGaugeDeck from './NoteGaugeDeck.svelte';

	let { noteName = '', content = '', review = null as any, backlinks = [] as any[], outgoing = [] as any[], resolveTarget, onNavigate }: {
		noteName?: string; content?: string; review?: any; backlinks?: any[]; outgoing?: any[];
		resolveTarget?: (name: string) => { path: string; libraryName: string };
		onNavigate?: (path: string, name: string, libraryName: string) => void;
	} = $props();

	const L = (k: string, fb: string) => { const v = $t(k); return v === k ? fb : v; };

	const RAIL = 450, GTOP = 92, BARMAX = 322, BARH = 14, LAB = 26;

	let model = $derived.by(() => {
		void $linkTypesStore;   // recolour/re-order when the link-type vocabulary changes
		const gl = groupByType(backlinks), gr = groupByType(outgoing);
		const types = orderTypes([...new Set([...Object.keys(gl), ...Object.keys(gr)])]);
		const gmax = Math.max(1, ...types.map((tp) => Math.max(gl[tp]?.length || 0, gr[tp]?.length || 0)));
		const rowH = Math.min(52, types.length ? 430 / types.length : 52);
		const rows = types.map((tp, i) => ({
			type: tp, color: relColor(tp), cy: GTOP + i * rowH + rowH / 2,
			back: gl[tp]?.length || 0, out: gr[tp]?.length || 0, backLinks: gl[tp] || [], outLinks: gr[tp] || [],
		}));
		return { rows, gmax, rowH, railBot: GTOP + types.length * rowH };
	});
	let stats = $derived(deriveStats(content, review, backlinks, outgoing));
	const px = (n: number) => (n / model.gmax) * BARMAX;

	let open = $state<{ type: string; side: 'back' | 'out' } | null>(null);
	let openLinks = $derived.by(() => {
		if (!open) return [] as any[];
		const row = model.rows.find((r) => r.type === open!.type); if (!row) return [];
		const src = open.side === 'back' ? row.backLinks : row.outLinks;
		return src.slice().sort((a, b) => tierW(b.tier) - tierW(a.tier)).map((lk) => {
			const isOut = open!.side === 'out';
			const raw = isOut ? lk.target : lk.name;
			const res = isOut ? (resolveTarget?.(lk.target ?? '') ?? { path: '', libraryName: lk.libraryName }) : { path: lk.path, libraryName: lk.libraryName };
			return { name: clean(raw), path: res.path, lib: res.libraryName || lk.libraryName, tier: (lk.tier || 'emerging') };
		});
	});
	function toggle(type: string, side: 'back' | 'out', count: number) {
		if (!count) return;
		open = (open && open.type === type && open.side === side) ? null : { type, side };
	}
	function go(f: { path?: string; name: string; lib?: string }) { if (f.path) { onNavigate?.(f.path, f.name, f.lib || ''); open = null; } }
</script>

<div class="lg">
	<div class="lg-stage">
		<svg class="lg-svg" viewBox="0 0 900 560" preserveAspectRatio="xMidYMid meet" role="img"
			aria-label="Ledger — {backlinks.length} backlinks left, {outgoing.length} outgoing right">
			<rect x="0" y="0" width="900" height="560" fill="var(--background-primary, #fff)"/>

			<rect x={RAIL - 42} y="36" width="84" height="24" rx="12" fill="var(--background-secondary, #f4f4f5)" stroke="var(--background-modifier-border, #d4d4d8)"/>
			<text class="lg-cn" x={RAIL} y="49" text-anchor="middle">{clean(noteName)}</text>
			<text class="lg-side" x={RAIL - 52} y="26" text-anchor="end">◀ {L('cockpit.incoming', 'incoming')} · {backlinks.length}</text>
			<text class="lg-side" x={RAIL + 52} y="26">{L('cockpit.outgoing', 'outgoing')} · {outgoing.length} ▶</text>

			{#each [0.5, 1] as f}
				<line x1={RAIL - BARMAX * f} y1={GTOP - 6} x2={RAIL - BARMAX * f} y2={model.railBot} stroke="var(--background-modifier-border, #e2e2e2)" stroke-opacity="0.5" stroke-dasharray="2 4"/>
				<line x1={RAIL + BARMAX * f} y1={GTOP - 6} x2={RAIL + BARMAX * f} y2={model.railBot} stroke="var(--background-modifier-border, #e2e2e2)" stroke-opacity="0.5" stroke-dasharray="2 4"/>
				<text class="lg-tick" x={RAIL - BARMAX * f} y={model.railBot + 14} text-anchor="middle">{Math.round(model.gmax * f)}</text>
				<text class="lg-tick" x={RAIL + BARMAX * f} y={model.railBot + 14} text-anchor="middle">{Math.round(model.gmax * f)}</text>
			{/each}
			<line x1={RAIL} y1={GTOP - 10} x2={RAIL} y2={model.railBot} stroke="var(--background-modifier-border, #b8b8b8)" stroke-width="1.4"/>

			{#each model.rows as r}
				<text class="lg-type" x={RAIL} y={r.cy - BARH / 2 - 6} text-anchor="middle" fill={r.color}>{relLabelIn($locale, r.type)}</text>
				{#if r.back}
					<g class="lg-bar" class:on={open?.type === r.type && open?.side === 'back'} role="button" tabindex="0"
						aria-label="{r.type} · {r.back} backlinks" onclick={() => toggle(r.type, 'back', r.back)}
						onkeydown={(e) => { if (e.key === 'Enter') toggle(r.type, 'back', r.back); }}>
						<rect x={RAIL - LAB - px(r.back)} y={r.cy - BARH / 2} width={px(r.back)} height={BARH} rx="3" fill={r.color}/>
						<text class="lg-ct" x={RAIL - LAB - px(r.back) - 5} y={r.cy + 1} text-anchor="end">{r.back}</text>
					</g>
				{/if}
				{#if r.out}
					<g class="lg-bar" class:on={open?.type === r.type && open?.side === 'out'} role="button" tabindex="0"
						aria-label="{r.type} · {r.out} outgoing" onclick={() => toggle(r.type, 'out', r.out)}
						onkeydown={(e) => { if (e.key === 'Enter') toggle(r.type, 'out', r.out); }}>
						<rect x={RAIL + LAB} y={r.cy - BARH / 2} width={px(r.out)} height={BARH} rx="3" fill={r.color}/>
						<text class="lg-ct" x={RAIL + LAB + px(r.out) + 5} y={r.cy + 1} text-anchor="start">{r.out}</text>
					</g>
				{/if}
				<circle cx={RAIL - 6} cy={r.cy} r="2.5" fill={r.color} opacity={r.back ? 1 : 0.25}/>
				<circle cx={RAIL + 6} cy={r.cy} r="2.5" fill={r.color} opacity={r.out ? 1 : 0.25}/>
			{/each}

			{#if !model.rows.length}
				<text class="lg-empty" x={RAIL} y="280" text-anchor="middle">{L('cockpit.noLinks', 'no links yet')}</text>
			{/if}
		</svg>

		{#if open}
			<div class="lg-drawer" dir="auto">
				<div class="lg-dhead">
					<span class="lg-ddot" style="background:{relColor(open.type)}"></span>
					<span class="lg-dtitle">{relLabelIn($locale, open.type)} · {open.side === 'back' ? L('cockpit.incoming', 'incoming') : L('cockpit.outgoing', 'outgoing')}</span>
					<button class="lg-dclose" onclick={() => open = null} aria-label="close">✕</button>
				</div>
				<div class="lg-dlist">
					{#each openLinks as f}
						<button class="lg-link" class:dead={!f.path} onclick={() => go(f)} disabled={!f.path} dir="auto">
							<span class="lg-ltier lt-{f.tier}"></span><span class="lg-lname">{f.name}</span>
						</button>
					{/each}
				</div>
			</div>
		{/if}
	</div>

	<NoteGaugeDeck {content} {review} {backlinks} {outgoing} />
</div>

<style>
	.lg { display: flex; flex-direction: column; width: 100%; height: 100%; min-height: 0; }
	.lg-stage { flex: 1; min-height: 0; position: relative; }
	/* see NoteButterflyGraph: SVG text-anchor flips under an inherited dir=rtl. */
	.lg-svg { width: 100%; height: 100%; display: block; direction: ltr; }
	.lg-cn { font: 600 13px var(--font-text, var(--font-sans)); fill: var(--text-normal, #1a1a1a);  unicode-bidi: plaintext; }
	.lg-side { font: 500 12px var(--font-sans); fill: var(--text-muted, #6b7280);  unicode-bidi: plaintext; }
	.lg-type { font: 500 12px var(--font-sans); text-transform: lowercase; letter-spacing: 0.02em;  unicode-bidi: plaintext; }
	.lg-ct { font: 11px var(--font-sans); fill: var(--text-muted, #6b7280);  unicode-bidi: plaintext; }
	.lg-tick { font: 10px var(--font-sans); fill: var(--text-faint, #9ca3af);  unicode-bidi: plaintext; }
	.lg-empty { font: 14px var(--font-sans); fill: var(--text-muted, #6b7280);  unicode-bidi: plaintext; }
	.lg-bar { cursor: pointer; }
	.lg-bar rect { transition: opacity 0.12s; }
	.lg-bar:hover rect { opacity: 0.82; }
	.lg-bar.on rect { stroke: var(--text-normal, #1a1a1a); stroke-width: 1.5; }
	.lg-drawer { position: absolute; top: 16px; right: 16px; width: 250px; max-height: calc(100% - 32px);
		display: flex; flex-direction: column; background: var(--background-primary, #fff);
		border: 1px solid var(--background-modifier-border, #d4d4d8); border-radius: 10px; box-shadow: 0 6px 22px rgba(0,0,0,0.14); overflow: hidden; }
	.lg-dhead { display: flex; align-items: center; gap: 8px; padding: 9px 11px; border-bottom: 1px solid var(--background-modifier-border, #e2e2e2); }
	.lg-ddot { width: 10px; height: 10px; border-radius: 3px; flex-shrink: 0; }
	.lg-dtitle { font-size: 12px; font-weight: 600; color: var(--text-normal, #1a1a1a); text-transform: lowercase; flex: 1; }
	.lg-dclose { border: none; background: none; color: var(--text-muted, #6b7280); cursor: pointer; font-size: 13px; padding: 0 2px; }
	.lg-dlist { overflow-y: auto; padding: 5px; display: flex; flex-direction: column; gap: 1px; }
	.lg-link { display: flex; align-items: center; gap: 8px; padding: 6px 8px; border: none; background: none; border-radius: 6px;
		cursor: pointer; text-align: start; color: var(--text-normal, #1a1a1a); font-size: 12.5px; width: 100%; }
	.lg-link:hover { background: var(--background-secondary, #f4f4f5); }
	.lg-link.dead { color: var(--text-faint, #9ca3af); cursor: default; }
	.lg-ltier { width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0; background: var(--text-muted, #6b7280); }
	.lt-load-bearing { background: var(--rel-supports, #879A39); width: 7px; height: 7px; }
	.lt-established { background: var(--interactive-accent, #7c3aed); }
	.lt-stale { background: var(--text-faint, #9ca3af); opacity: 0.6; }
	.lg-lname { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
