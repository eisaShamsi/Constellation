<script lang="ts">
	/**
	 * CalloutTypesEditor — the bespoke block mounted in the Style Setter's
	 * "Callouts" element (Editor category), alongside its 10 colour controls.
	 *
	 * MIG-089 Phase A: a per-family ICON picker — change/reset the icon of each
	 * of the 10 built-in callout families, choosing from the app's Emoji & Icon
	 * Library (the same EmojiIconPicker used everywhere). Overrides are stored as
	 * `callout.<family>` entries in the per-Universe iconOverrides map and read by
	 * calloutPlugin; an open editor repaints live via the refreshCallouts hook.
	 *
	 * (Phase B will add the "Add custom callout type" section below.)
	 */
	import { t } from '$lib/i18n';
	import { appSettings } from '$lib/libraries/store';
	import { setOverride } from '$lib/theme/iconOverrides';
	import { CALLOUT_FAMILIES, calloutDefaultIcon } from '$lib/editor/calloutPlugin';
	import EmojiIconPicker from './EmojiIconPicker.svelte';
	import SlotIcon from './SlotIcon.svelte';

	let { embedded = false }: { embedded?: boolean } = $props();

	let pickingFamily = $state<string | null>(null);

	const overrides = $derived($appSettings.iconOverrides ?? {});
	const families = CALLOUT_FAMILIES.map((f) => ({ family: f, defaultIcon: calloutDefaultIcon(f) }));

	// styleSetter.labels.<slug> with an English fallback on a miss (mirrors StyleSetter's L()).
	function lbl(slug: string, fallback: string): string {
		const key = 'styleSetter.labels.' + slug;
		const v = $t(key);
		return !v || v === key ? fallback : v;
	}
	const familyLabel = (family: string) => lbl(family, family.charAt(0).toUpperCase() + family.slice(1));
</script>

<div class="cte" class:cte-embedded={embedded}>
	<div class="cte-head">{lbl('callout_icons', 'Callout icons')}</div>
	<div class="cte-rows">
		{#each families as f (f.family)}
			<div class="cte-row">
				<button class="cte-icon" title={lbl('change_icon', 'Change icon')} onclick={() => (pickingFamily = f.family)}>
					<SlotIcon slot={'callout.' + f.family}><span class="cte-emoji">{f.defaultIcon}</span></SlotIcon>
				</button>
				<span class="cte-label">{familyLabel(f.family)}</span>
				{#if overrides['callout.' + f.family]}
					<button class="cte-reset" title={lbl('reset_icon', 'Reset icon')} onclick={() => setOverride('callout.' + f.family, null)}>↺</button>
				{/if}
			</div>
		{/each}
	</div>
</div>

{#if pickingFamily}
	<EmojiIconPicker
		onClose={() => (pickingFamily = null)}
		onPick={(insertion) => {
			// Normalize the picker's `:lucide-heart:` shortcode (or raw emoji) to the
			// stored ref form ("lucide:heart" / emoji-as-is) — same as IconOverrideSettings.
			let ref: string;
			const sc = insertion.match(/^:(lucide|phosphor|hi|feather)-([a-z0-9-]+):$/);
			if (sc) ref = `${sc[1]}:${sc[2]}`;
			else ref = insertion;
			if (pickingFamily) setOverride('callout.' + pickingFamily, ref);
			pickingFamily = null;
		}}
	/>
{/if}

<style>
	.cte { margin-top: 10px; }
	.cte-head { font-size: 11px; text-transform: uppercase; letter-spacing: 0.07em; color: var(--c-muted, var(--text-muted)); margin: 6px 4px 6px; }
	.cte-rows { display: flex; flex-direction: column; gap: 4px; }
	.cte-row { display: flex; align-items: center; gap: 8px; }
	.cte-icon {
		width: 30px; height: 30px; flex: none;
		display: flex; align-items: center; justify-content: center;
		background: var(--background-primary, #fff); border: 1px solid var(--background-modifier-border, #ddd);
		border-radius: 6px; cursor: pointer; padding: 0;
	}
	.cte-icon:hover { border-color: var(--interactive-accent, #7c3aed); }
	.cte-icon :global(svg) { width: 18px; height: 18px; }
	.cte-emoji { font-size: 17px; line-height: 1; }
	.cte-label { flex: 1; min-width: 0; font-size: 13px; color: var(--c-text, var(--text-normal)); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.cte-reset {
		width: 22px; height: 22px; flex: none;
		background: none; border: 1px solid transparent; border-radius: 6px;
		color: var(--c-muted, var(--text-muted)); cursor: pointer; font-size: 13px;
	}
	.cte-reset:hover { color: var(--c-text, var(--text-normal)); border-color: var(--background-modifier-border, #ddd); }
</style>
