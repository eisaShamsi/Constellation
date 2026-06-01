<script lang="ts">
	/**
	 * The Typed-Link Pill — the ONE, self-contained source for every typed-link type
	 * badge in the app (Backlinks, Outgoing Links, the Knowledge-Health dashboard, …).
	 *
	 * Redesigned from scratch (Eisa: "one source serves all"): the pill is IMMUNE to its
	 * host. It sets its own font, text direction, size, colour, shape, and centring, so it
	 * renders pixel-identically no matter which panel — with whatever inherited font /
	 * direction / flex alignment — it is dropped into. The drift between the Backlinks and
	 * Outgoing pills came precisely from inheriting those things from two different rows.
	 *
	 *   <LinkTypePill id="supports" loc="ar" />   →  يدعم     (note-language label)
	 *   <LinkTypePill id="derives-from" />         →  derives-from  (UI-language fallback)
	 *
	 * Colour + auto-contrast text come from the Link-Type Registry (the §G editor — the one
	 * colour source), so a recolour reflects here live. Shape is the user's pill preference
	 * in appSettings, read here directly (not via host-set CSS vars). Nothing is inherited.
	 */
	import { tIn, locale } from '$lib/i18n';
	import { appSettings } from '$lib/libraries/store';
	import { linkTypesStore, linkTypeColor, linkTypeTextColor } from '$lib/libraries/linkTypeRegistry';

	let { id, loc = '' }: { id: string; loc?: string } = $props();

	// `void $linkTypesStore` makes colour reactive to a §G recolour while delegating the
	// actual lookup to the registry helpers (the single colour source + auto-contrast).
	const fill = $derived.by(() => { void $linkTypesStore; return linkTypeColor(id); });
	const text = $derived.by(() => { void $linkTypesStore; return linkTypeTextColor(id); });
	const shape = $derived($appSettings.linkPills?.shape ?? { radius: 10, height: 20, fontWeight: 700 });
	// Label in the note's language when given, else the UI language; raw id if untranslated.
	const label = $derived.by(() => {
		const lc = loc || $locale;
		const key = `linkTypes.${id.toLowerCase()}`;
		const tr = tIn(lc, key);
		return tr !== key ? tr : id;
	});
</script>

<span
	class="ltpill"
	dir="auto"
	style="--f:{fill};--t:{text};--r:{shape.radius}px;--h:{shape.height}px;--w:{shape.fontWeight}"
>{label}</span>

<style>
	.ltpill {
		/* Self-centring inline badge — immune to the host row's flex align / baseline. */
		display: inline-flex;
		align-items: center;
		justify-content: center;
		align-self: center;
		vertical-align: middle;
		flex: 0 0 auto;
		box-sizing: border-box;
		height: var(--h, 20px);
		padding: 0 8px;
		/* Type colour from the registry, with auto-contrast text. */
		background: var(--f, #888);
		color: var(--t, #fff);
		border-radius: var(--r, 10px);
		/* Fixed text metrics + the app's interface font, so rendering never depends on
		   whatever font / size / line-height the host panel happens to inherit. */
		font-family: var(--font-interface-theme, system-ui, sans-serif);
		font-size: 0.65rem;
		font-weight: var(--w, 700);
		line-height: 1;
		letter-spacing: 0.02em;
		text-transform: lowercase;
		white-space: nowrap;
	}
</style>
