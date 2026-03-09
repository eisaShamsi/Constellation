<script lang="ts">
	import { renderMarkdown, postProcessRenderedContent } from '$lib/utils';
	import { onMount } from 'svelte';

	let {
		content = '',
		x = 0,
		y = 0,
		visible = false,
	}: {
		content: string;
		x: number;
		y: number;
		visible: boolean;
	} = $props();

	let previewEl: HTMLDivElement;

	$effect(() => {
		if (visible && content && previewEl) {
			requestAnimationFrame(() => {
				if (previewEl) postProcessRenderedContent(previewEl);
			});
		}
	});

	// Adjust position to stay within viewport
	const adjustedStyle = $derived.by(() => {
		let left = x + 10;
		let top = y + 10;
		const maxWidth = 400;
		const maxHeight = 300;
		if (typeof window !== 'undefined') {
			if (left + maxWidth > window.innerWidth) left = x - maxWidth - 10;
			if (top + maxHeight > window.innerHeight) top = y - maxHeight - 10;
		}
		return `left: ${Math.max(0, left)}px; top: ${Math.max(0, top)}px`;
	});
</script>

{#if visible && content}
	<div class="page-preview" style={adjustedStyle} bind:this={previewEl}>
		<div class="pp-content">
			{@html renderMarkdown(content)}
		</div>
	</div>
{/if}

<style>
	.page-preview {
		position: fixed;
		z-index: 1100;
		max-width: 400px;
		max-height: 300px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		box-shadow: var(--shadow-l);
		overflow: hidden;
		pointer-events: none;
	}
	.pp-content {
		padding: 12px 16px;
		font-size: 0.82rem;
		line-height: 1.6;
		color: var(--text-normal);
		overflow-y: auto;
		max-height: 280px;
	}
	.pp-content :global(h1) { font-size: 1.1rem; margin: 0.5rem 0 0.3rem; }
	.pp-content :global(h2) { font-size: 1rem; margin: 0.4rem 0 0.2rem; }
	.pp-content :global(h3) { font-size: 0.9rem; margin: 0.3rem 0 0.2rem; }
	.pp-content :global(p) { margin: 0.3rem 0; }
	.pp-content :global(code) { font-size: 0.8rem; }
	.pp-content :global(img) { max-width: 100%; }
</style>
