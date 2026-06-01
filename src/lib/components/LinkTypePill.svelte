<script lang="ts">
	/**
	 * MIG-067 §H.2 — the ONE typed-link pill.
	 *
	 * The single source for the coloured type badge shown in the Backlinks + Outgoing Links
	 * panels (Eisa: "the pills should come from one source" — so they can never drift apart
	 * again, which is exactly what produced the Backlinks-vs-Outgoing alignment mismatch).
	 *
	 *  • Colour comes from the Link-Type Registry (the §G editor — single source of truth),
	 *    with an auto-contrasted text colour.
	 *  • The label reads in the NOTE's language (§H), not the UI's — pass `loc`.
	 *  • Shape (radius / height / weight) comes from the `--pill-*` vars the host panel sets
	 *    from appSettings, so the user's pill-shape preference still applies.
	 *  • Centred by construction (align-self + justify-content + vertical-align), so no parent
	 *    row or ancestor can push the text off-centre.
	 */
	import { tIn } from '$lib/i18n';
	import { linkTypesStore, linkTypeTextColor } from '$lib/libraries/linkTypeRegistry';

	let { id, loc = 'en' }: { id: string; loc?: string } = $props();

	const fill = $derived($linkTypesStore.find((tp) => tp.id === id)?.color ?? '#888');
	const txt = $derived(linkTypeTextColor(id) ?? '#ffffff');
	// Localized type name in the note's language; raw-id fallback if no translation exists.
	const label = $derived.by(() => {
		const k = `linkTypes.${id.toLowerCase()}`;
		const tr = tIn(loc, k);
		return tr !== k ? tr : id;
	});
</script>

<span class="link-type-pill" style="color:{txt};background:{fill};border-color:{fill}">{label}</span>

<style>
	.link-type-pill {
		display: inline-flex; align-items: center; justify-content: center; align-self: center;
		box-sizing: border-box; flex-shrink: 0; vertical-align: middle;
		font-size: 0.65rem; font-weight: var(--pill-weight, 700); line-height: 1;
		padding: 0 8px; height: var(--pill-height, 20px);
		border-radius: var(--pill-radius, 10px); border: 1px solid;
		white-space: nowrap; text-transform: lowercase; letter-spacing: 0.02em;
	}
</style>
