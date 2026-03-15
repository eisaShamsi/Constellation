<script lang="ts">
	import { renderMarkdown, postProcessRenderedContent } from '$lib/utils';

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
	let contentEl: HTMLDivElement;

	// Detect text direction from raw content
	const RTL_REGEX = /[\u0591-\u07FF\u200F\u202B\u202E\uFB1D-\uFDFD\uFE70-\uFEFC]/;
	const contentDir = $derived(RTL_REGEX.test(content) ? 'rtl' : 'ltr');

	$effect(() => {
		if (visible && content && previewEl) {
			requestAnimationFrame(() => {
				if (previewEl) postProcessRenderedContent(previewEl);
			});
			if (contentEl) contentEl.scrollTop = 0;
		}
	});

	// Capture wheel events to scroll the preview content
	$effect(() => {
		if (!visible || typeof document === 'undefined') return;

		function handleWheel(e: WheelEvent) {
			if (contentEl) {
				contentEl.scrollTop += e.deltaY;
				e.preventDefault();
			}
		}

		document.addEventListener('wheel', handleWheel, { passive: false });
		return () => document.removeEventListener('wheel', handleWheel);
	});

	// Position: appear near cursor, stay within viewport
	const adjustedStyle = $derived.by(() => {
		const maxWidth = 450;
		const maxHeight = 400;
		let left = x + 10;
		let top = y + 10;

		if (typeof window !== 'undefined') {
			if (left + maxWidth > window.innerWidth) left = x - maxWidth - 10;
			if (top + maxHeight > window.innerHeight) top = y - maxHeight - 10;
			if (left < 0) left = 0;
			if (top < 0) top = 0;
		}
		return `left: ${left}px; top: ${top}px`;
	});
</script>

{#if visible && content}
	<div class="page-preview"
		style={adjustedStyle}
		bind:this={previewEl}
		role="tooltip">
		<div class="pp-content" dir={contentDir} bind:this={contentEl}>
			{@html renderMarkdown(content)}
		</div>
		<div class="pp-scroll-hint">&#x21f3; Scroll to explore</div>
	</div>
{/if}

<style>
	.page-preview {
		position: fixed;
		z-index: 1100;
		max-width: 450px;
		max-height: 400px;
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
		max-height: 360px;
	}
	.pp-scroll-hint {
		padding: 3px 12px 5px;
		font-size: 0.68rem;
		color: var(--text-faint);
		text-align: center;
		border-top: 1px solid var(--background-modifier-border);
		background: var(--background-secondary);
	}
	.pp-content :global(h1) { font-size: 1.1rem; margin: 0.5rem 0 0.3rem; }
	.pp-content :global(h2) { font-size: 1rem; margin: 0.4rem 0 0.2rem; }
	.pp-content :global(h3) { font-size: 0.9rem; margin: 0.3rem 0 0.2rem; }
	.pp-content :global(p) { margin: 0.3rem 0; }
	.pp-content :global(code) { font-size: 0.8rem; }
	.pp-content :global(img) { max-width: 100%; }
</style>
