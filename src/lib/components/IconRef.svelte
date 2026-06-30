<script lang="ts">
	/**
	 * IconRef — render a raw icon ref (an emoji char or a "set:name" id) the same
	 * way SlotIcon renders a slot override. Used by CalloutTypesEditor to preview a
	 * custom callout's icon (whose ref lives in the customCallouts registry, not a
	 * slot). Emoji render synchronously; an icon ref resolves after the cache warms.
	 */
	import { resolveRefSync, prewarmIcons } from '$lib/theme/iconOverrides';

	let { ref, fallback = '' }: { ref?: string; fallback?: string } = $props();

	let rendered = $state<string | null>(null);

	$effect(() => {
		const r = ref;
		if (!r) { rendered = null; return; }
		if (!r.includes(':')) { rendered = r; return; }   // emoji — the ref IS the glyph
		let cancelled = false;
		prewarmIcons().then(() => { if (!cancelled) rendered = resolveRefSync(r); });
		return () => { cancelled = true; };
	});
</script>

{#if rendered}
	{#if rendered.startsWith('<svg')}{@html rendered}{:else}<span class="ir-emoji">{rendered}</span>{/if}
{:else}
	<span class="ir-emoji">{fallback}</span>
{/if}

<style>
	/* The resolved <svg> is injected via {@html} (unscoped); its size is set by the
	   host button's CSS (e.g. .cte-icon :global(svg)). Here we only size the emoji. */
	.ir-emoji { font-size: 17px; line-height: 1; }
</style>
