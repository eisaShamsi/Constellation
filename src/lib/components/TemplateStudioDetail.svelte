<script lang="ts">
	/**
	 * MIG-103 §4 Slice 1 — the detail column: recognition, then the offer.
	 *
	 * Order is the argument. **Five real note titles come FIRST**, before the name
	 * field, because for the 18-of-21 kinds Constellation could not name, those titles
	 * are the whole recognition. A user who reads "Andrea Palladio · Andrea Pisano"
	 * knows to type *Person* without the engine ever guessing — which is the concept
	 * exactly: report what recurs, never infer the meaning.
	 *
	 * Slice 1 is READ-ONLY. There is no Keep button and no Not-a-kind button yet: a
	 * disabled primary action is a worse lie than an absent one.
	 */
	import { t } from '$lib/i18n';
	import { detectDir, formatNumerals } from '$lib/utils';
	import { appSettings } from '$lib/libraries/store';
	import TemplateStudioFields from '$lib/components/TemplateStudioFields.svelte';
	import { isCollisionName, type DiscoveredShape, type ProposedName, type NameEvidence } from '$lib/templates/discoveredKinds';

	let {
		kind,
		draftName = '',
		onNameChange,
		onOpenExample,
	}: {
		kind: DiscoveredShape;
		draftName?: string;
		onNameChange: (v: string) => void;
		onOpenExample: (path: string) => void;
	} = $props();

	let numerals = $derived($appSettings.numeralStyle ?? 'arabic');
	function n(v: number) { return formatNumerals(v, numerals); }

	/** The candidate whose evidence is on screen — rank 1, or the one the user picked. */
	let shownCandidate = $state<ProposedName | null>(null);
	let active = $derived(shownCandidate ?? kind.proposed_name);

	/**
	 * The heading names the token the evidence actually DESCRIBES. On a collision the
	 * proposal is a synthesised compound while its evidence still describes the bare
	 * token, so naming the compound here would attach reasoning to the wrong word.
	 */
	let evidenceToken = $derived(
		isCollisionName(kind) ? (kind.name_candidates[0]?.name ?? '') : (active?.name ?? ''),
	);

	let alternates = $derived(kind.name_candidates.filter((c) => c.name !== active?.name));

	/** One complete sentence per evidence family — never assembled from fragments,
	 *  because Arabic word order does not survive concatenation. */
	function evidenceLine(e: NameEvidence, token: string): string {
		const key = `templateStudio.evidence.${e.family}`;
		return $t(key, {
			token,
			n: n(e.members_with),
			total: n(e.members_total),
			corpusWith: n(e.corpus_with),
			corpusTotal: n(e.corpus_total),
		});
	}

	/** The core fields in the spelling the notes use — the helper text beside the box. */
	let coreDisplay = $derived(
		kind.core.map((k) => kind.fields.find((f) => f.key === k)?.display ?? k).join(', '),
	);
</script>

<div class="kd">
	<!-- RECOGNITION FIRST — the densest signal on the surface for an unnamed kind. -->
	<section class="kd-eg">
		<h3 class="kd-h">{$t('templateStudio.notesLikeThis')}</h3>
		<ul class="kd-eg-list">
			{#each kind.examples as ex (ex.path)}
				<li>
					<button class="kd-eg-item" type="button" dir={detectDir(ex.title)}
						onclick={() => onOpenExample(ex.path)}>{ex.title || ex.path}</button>
				</li>
			{/each}
		</ul>
	</section>

	<section class="kd-name">
		<h3 class="kd-h"><label for="kd-name-input">{$t('templateStudio.whatDoYouCallThese')}</label></h3>
		<!-- Boss ruling 2026-07-22: the box is GENUINELY EMPTY when there is no
		     proposal, and the fields these notes share sit BESIDE it, not inside it.
		     Grey text inside a box reads as an answer already given — which here would
		     manufacture the very name Constellation declined to guess, and get it saved
		     verbatim. Helper text outside also survives focus, so it is still there
		     while the user types. -->
		<input id="kd-name-input" class="kd-input" type="text" value={draftName}
			dir={detectDir(draftName || ' ')} placeholder=""
			oninput={(e) => onNameChange((e.target as HTMLInputElement).value)} />

		<p class="kd-helper">{$t('templateStudio.theseNotesCarry', { fields: coreDisplay })}</p>

		{#if active}
			<p class="kd-ev-h">{$t('templateStudio.whereItComesFrom', { token: evidenceToken })}</p>
			<ul class="kd-ev">
				{#each active.evidence as e (e.family)}
					<li>{evidenceLine(e, evidenceToken)}</li>
				{/each}
			</ul>
			{#if isCollisionName(kind)}
				<p class="kd-note">{$t('templateStudio.collisionNote', { token: evidenceToken })}</p>
			{/if}
			{#if alternates.length > 0}
				<p class="kd-alt-h">{$t('templateStudio.otherWords')}</p>
				<div class="kd-alts">
					{#each alternates as c (c.name)}
						<button class="kd-alt" type="button" dir={detectDir(c.name)}
							onclick={() => { shownCandidate = c; onNameChange(c.name); }}>{c.name}</button>
					{/each}
				</div>
			{/if}
		{:else}
			<!-- Worded to the gates the engine actually applies, NOT "found no word they
			     share" — a word on 45% of the notes, or on 90% in a single family, IS
			     shared and is still rejected. This is the highest-traffic string here and
			     it must not overstate its own finding. -->
			<p class="kd-none">{$t('templateStudio.noNameFound', { count: n(kind.support) })}</p>
		{/if}
	</section>

	<TemplateStudioFields {kind} />
</div>

<style>
	.kd { max-inline-size: 820px; margin-inline: auto; padding: 24px 28px 60px; }
	.kd-h {
		margin: 0 0 8px;
		font-size: calc(0.86rem * var(--rs-scale, 1));
		font-weight: 600;
	}
	.kd-eg-list { list-style: none; margin: 0; padding: 0; }
	.kd-eg-item {
		display: block;
		inline-size: 100%;
		text-align: start;
		padding: 3px 4px;
		border: none;
		background: none;
		color: var(--text-accent);
		cursor: pointer;
		font-size: calc(0.84rem * var(--rs-scale, 1));
		unicode-bidi: isolate;
	}
	.kd-eg-item:hover { text-decoration: underline; }
	.kd-name { margin-block-start: 26px; }
	.kd-input {
		inline-size: 100%;
		max-inline-size: 420px;
		padding: 7px 10px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 4px;
		background: var(--background-primary);
		color: var(--text-normal);
		font-size: calc(0.92rem * var(--rs-scale, 1));
	}
	.kd-helper {
		margin: 6px 0 0;
		color: var(--text-muted);
		font-size: calc(0.78rem * var(--rs-scale, 1));
	}
	.kd-ev-h, .kd-alt-h {
		margin: 16px 0 4px;
		color: var(--text-muted);
		font-size: calc(0.78rem * var(--rs-scale, 1));
	}
	.kd-ev { margin: 0; padding-inline-start: 18px; }
	.kd-ev li {
		color: var(--text-muted);
		font-size: calc(0.8rem * var(--rs-scale, 1));
		line-height: 1.55;
	}
	.kd-note, .kd-none {
		margin: 10px 0 0;
		color: var(--text-muted);
		font-size: calc(0.8rem * var(--rs-scale, 1));
		line-height: 1.55;
	}
	.kd-alts { display: flex; flex-wrap: wrap; gap: 6px; }
	.kd-alt {
		padding: 2px 8px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 3px;
		background: none;
		color: var(--text-normal);
		cursor: pointer;
		font-size: calc(0.8rem * var(--rs-scale, 1));
		unicode-bidi: isolate;
	}
	.kd-alt:hover { background: var(--background-modifier-hover); }
</style>
