<script lang="ts">
	/**
	 * SlotIcon — renders an icon that can be overridden via the Emoji & Icon
	 * Library plug-in. Displays the user's override if set, else the children
	 * (the original hardcoded SVG) as fallback.
	 *
	 * Usage:
	 *   <SlotIcon slot="dock.knowledgeHealth">
	 *     <svg>...default brain...</svg>
	 *   </SlotIcon>
	 */
	import { onMount } from 'svelte';
	import { appSettings } from '$lib/libraries/store';
	import { resolveOverride } from '$lib/theme/iconOverrides';

	let { slot, children }: { slot: string; children?: any } = $props();

	let rendered = $state<string | null>(null);

	// Re-resolve whenever the overrides map changes
	$effect(() => {
		const overrides = $appSettings.iconOverrides ?? {};
		const ref = overrides[slot];
		if (!ref) { rendered = null; return; }
		resolveOverride(slot).then(r => { rendered = r; });
	});
</script>

{#if rendered}
	{#if rendered.startsWith('<svg')}
		{@html rendered}
	{:else}
		<span class="slot-icon-emoji">{rendered}</span>
	{/if}
{:else if children}
	{@render children()}
{/if}

<style>
	.slot-icon-emoji {
		display: inline-flex; align-items: center; justify-content: center;
		font-size: 1.05em;
	}
</style>
