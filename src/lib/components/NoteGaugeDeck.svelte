<script lang="ts">
	/**
	 * PJ-068 v2 — the shared instrument deck under every note-graph lens.
	 *
	 * The note's statistics as GAUGES, grouped by the Cognitive Engine's four questions:
	 * DEVELOPMENT · CONTENT · ORIGIN · CONNECTION — in service of CONVICTION. Theme-aware
	 * (app CSS vars); link colours are --rel-*. Reused by the Butterfly and the Ledger so the
	 * data reads identically whichever lens is chosen.
	 */
	import { t } from '$lib/i18n';
	import { deriveStats, STAGES, MATS, clean } from '$lib/cockpitGraphData';

	let { content = '', review = null as any, backlinks = [] as any[], outgoing = [] as any[] }: {
		content?: string; review?: any; backlinks?: any[]; outgoing?: any[];
	} = $props();

	// safe i18n: svelte-i18n returns the KEY for a missing string (truthy), so `$t(k) || fb`
	// never falls back. `$t(k) === k ? fb : …` shows the English fallback instead of a raw key.
	const L = (k: string, fb: string) => { const v = $t(k); return v === k ? fb : v; };

	let s = $derived(deriveStats(content, review, backlinks, outgoing));
	let reviewLabel = $derived.by(() => {
		const m: Record<string, string> = {
			upToDate: L('cockpit.deck.upToDate', 'up to date'), due: L('cockpit.deck.due', 'due'),
			stale: L('cockpit.deck.stale', 'stale'), never: L('cockpit.deck.never', 'never reviewed'),
		};
		const base = m[s.reviewState.key] || '';
		return s.reviewState.key === 'due' && (review?.days_overdue ?? 0) > 0 ? `${base} · ${review.days_overdue}d` : base;
	});
</script>

<div class="deck">
	<div class="cell">
		<div class="hh">{L('cockpit.deck.development', 'development')}</div>
		{#if s.stageIdx >= 0}
			<div class="g"><span class="gl">{L('cockpit.deck.stage', 'stage')}</span>
				<span class="lad">{#each STAGES as _, i}<i class:on={i <= s.stageIdx} class:cur={i === s.stageIdx}></i>{/each}</span>
				<span class="gv">{s.stage}</span></div>
		{/if}
		{#if s.matIdx >= 0}
			<div class="g"><span class="gl">{L('cockpit.deck.maturity', 'maturity')}</span>
				<span class="lad">{#each MATS as _, i}<i class:on={i <= s.matIdx} class:cur={i === s.matIdx}></i>{/each}</span>
				<span class="gv">{s.maturity}</span></div>
		{/if}
		{#if reviewLabel}
			<div class="g"><span class="gl">{L('cockpit.deck.review', 'review')}</span>
				<span class="pill sev-{s.reviewState.sev}">{reviewLabel}</span></div>
		{/if}
	</div>

	<div class="cell">
		<div class="hh">{L('cockpit.deck.content', 'content')}</div>
		{#if s.wordCount != null}
			<div class="num">{s.wordCount.toLocaleString()}<span class="numl">{L('cockpit.deck.words', 'words')}</span></div>
		{/if}
		{#if s.stratum}<div class="g"><span class="gl">{L('cockpit.deck.stratum', 'stratum')}</span><span class="gv">{s.stratum}</span></div>{/if}
		{#if s.tags.length}
			<div class="chips">{#each s.tags.slice(0, 4) as tg}<span class="chip" dir="auto">{tg}</span>{/each}{#if s.tags.length > 4}<span class="chip more">+{s.tags.length - 4}</span>{/if}</div>
		{/if}
	</div>

	<div class="cell">
		<div class="hh">{L('cockpit.deck.origin', 'origin')}</div>
		{#if s.provenance}<div class="g"><span class="gl">{L('cockpit.deck.provenance', 'provenance')}</span><span class="gv">{s.provenance}</span></div>{/if}
		{#if s.source}<div class="g"><span class="gl">{L('cockpit.deck.source', 'source')}</span><span class="gv" dir="auto">{s.source}</span></div>{/if}
		{#if s.created}<div class="g"><span class="gl">{L('cockpit.deck.created', 'created')}</span><span class="gv">{s.created}</span></div>{/if}
	</div>

	<div class="cell">
		<div class="hh">{L('cockpit.deck.connection', 'connection')} · {s.totalLinks}</div>
		{#if s.typeMix.length}
			<div class="mix" title="{L('cockpit.deck.mix', 'relationship mix')}">{#each s.typeMix as tm}<span style="flex:{tm.count};background:{tm.color}"></span>{/each}</div>
		{/if}
		{#if s.supportsN || s.contradictsN}
			<div class="g"><span class="gl">{L('cockpit.deck.balance', 'balance')}</span>
				<span class="bal"><span class="bal-s" style="flex:{s.supportsN || 0.001}"></span><span class="bal-c" style="flex:{s.contradictsN || 0.001}"></span></span>
				<span class="gv">{s.supportsN}↑ {s.contradictsN}↓</span></div>
		{/if}
		{#if s.totalLinks}
			<div class="g"><span class="gl">{L('cockpit.deck.confidence', 'confidence')}</span>
				<span class="mix conf">{#each s.confMix as c}{#if c.n}<span style="flex:{c.n};background:{c.color}" title="{c.c}: {c.n}"></span>{/if}{/each}</span></div>
		{/if}
		{#if s.loadBearing}<div class="g"><span class="gl">{L('cockpit.deck.loadBearing', 'load-bearing')}</span><span class="gv">{s.loadBearing}</span></div>{/if}
	</div>
</div>

<style>
	.deck { display: grid; grid-template-columns: repeat(4, 1fr); gap: 14px; padding: 12px 18px 14px;
		border-top: 1px solid var(--background-modifier-border, #e2e2e2); color: var(--text-normal, #1a1a1a); }
	.cell { display: flex; flex-direction: column; gap: 6px; min-width: 0; }
	.hh { font-size: 10px; letter-spacing: 0.08em; text-transform: uppercase; color: var(--text-faint, #9ca3af); }
	.g { display: flex; align-items: center; gap: 7px; font-size: 12px; min-width: 0; }
	.gl { color: var(--text-muted, #6b7280); flex-shrink: 0; }
	.gv { color: var(--text-normal, #1a1a1a); text-transform: capitalize; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.lad { display: inline-flex; gap: 3px; }
	.lad i { width: 6px; height: 6px; border-radius: 50%; background: var(--background-modifier-border, #d4d4d8); }
	.lad i.on { background: var(--interactive-accent, #7c3aed); }
	.lad i.cur { box-shadow: 0 0 0 2px color-mix(in srgb, var(--interactive-accent, #7c3aed) 35%, transparent); }
	.pill { font-size: 11px; border-radius: 6px; padding: 1px 7px; white-space: nowrap; }
	.sev-ok { color: var(--rel-supports, #16a34a); background: color-mix(in srgb, var(--rel-supports, #16a34a) 14%, transparent); }
	.sev-warn { color: #b7791f; background: color-mix(in srgb, #d0a215 18%, transparent); }
	.sev-bad { color: var(--rel-contradicts, #dc2626); background: color-mix(in srgb, var(--rel-contradicts, #dc2626) 14%, transparent); }
	.sev-mut { color: var(--text-muted, #6b7280); background: color-mix(in srgb, var(--text-muted, #6b7280) 12%, transparent); }
	.num { font-size: 21px; font-weight: 600; line-height: 1; display: flex; align-items: baseline; gap: 6px; }
	.numl { font-size: 11px; font-weight: 400; color: var(--text-muted, #6b7280); }
	.chips { display: flex; gap: 4px; flex-wrap: wrap; }
	.chip { font-size: 11px; color: var(--text-muted, #6b7280); background: var(--background-modifier-border, #ececec); border-radius: 5px; padding: 1px 7px; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.chip.more { background: transparent; }
	.mix { display: flex; width: 100%; max-width: 150px; height: 8px; border-radius: 4px; overflow: hidden; gap: 1px; }
	.mix.conf { height: 6px; max-width: 120px; }
	.mix span { min-width: 2px; }
	.bal { display: flex; width: 72px; height: 8px; border-radius: 4px; overflow: hidden; background: var(--background-modifier-border, #ececec); flex-shrink: 0; }
	.bal-s { background: var(--rel-supports, #879A39); }
	.bal-c { background: var(--rel-contradicts, #D14D41); }
	@media (max-width: 720px) { .deck { grid-template-columns: repeat(2, 1fr); } }
</style>
