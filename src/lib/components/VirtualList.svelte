<script lang="ts" generics="T">
	/* ═══════════════════════════════════════════════════════════
	   VirtualList — variable-height windowed list for Svelte 5.

	   Renders only visible rows (plus `overscan`) regardless of
	   total item count. Supports per-item heights via `getItemHeight`,
	   so expandable rows work naturally (e.g. Index panel term
	   expansion showing note mentions).

	   Use cases per CLAUDE.md Rule 3 ("virtualize every list that can
	   exceed 50 items"): Index panel (500k+ terms), file tree,
	   search results, tag browser, backlinks.

	   Implementation:
	   - Prefix-sum offsets in a $derived; re-runs when `items` or
	     read dependencies of `getItemHeight` change.
	   - Binary search over offsets → first visible index.
	   - Absolute-positioned rows with translateY so DOM nodes are
	     reused across scrolls (keyed by slot index, not item index).
	   ═══════════════════════════════════════════════════════════ */
	import { onDestroy, onMount } from 'svelte';
	import type { Snippet } from 'svelte';

	let {
		items,
		getItemHeight,
		row,
		overscan = 8,
		scrollResetKey,
	}: {
		items: T[];
		getItemHeight: (item: T, index: number) => number;
		row: Snippet<[T, number]>;
		overscan?: number;
		/** Change this value to scroll the list back to top (e.g. on filter change). */
		scrollResetKey?: unknown;
	} = $props();

	let container = $state<HTMLDivElement | null>(null);
	let viewportHeight = $state(600);
	let innerScrollY = $state(0);
	let ro: ResizeObserver | null = null;

	// Prefix-sum offsets — one entry per item, plus total at offsets[n].
	const heights = $derived.by(() => {
		const n = items.length;
		const offsets = new Float64Array(n + 1);
		let sum = 0;
		for (let i = 0; i < n; i++) {
			offsets[i] = sum;
			sum += getItemHeight(items[i], i);
		}
		offsets[n] = sum;
		return { offsets, total: sum };
	});

	// Binary search: highest index where offsets[i] <= offset.
	function indexForOffset(offset: number): number {
		const offsets = heights.offsets;
		const n = offsets.length - 1;
		if (n <= 0) return 0;
		let lo = 0, hi = n;
		while (lo < hi) {
			const mid = (lo + hi + 1) >>> 1;
			if (offsets[mid] <= offset) lo = mid;
			else hi = mid - 1;
		}
		return lo;
	}

	const startIndex = $derived(Math.max(0, indexForOffset(innerScrollY) - overscan));
	const endIndex = $derived(Math.min(items.length, indexForOffset(innerScrollY + viewportHeight) + overscan + 1));

	const visible = $derived.by(() => {
		const out: Array<{ item: T; index: number; offset: number }> = [];
		for (let i = startIndex; i < endIndex; i++) {
			out.push({ item: items[i], index: i, offset: heights.offsets[i] });
		}
		return out;
	});

	function handleScroll(e: Event) {
		const el = e.target as HTMLDivElement;
		innerScrollY = el.scrollTop;
	}

	// Reset scroll to top whenever scrollResetKey changes (filter/letter/sort switch).
	$effect(() => {
		scrollResetKey; // tracked dependency
		if (container) {
			container.scrollTop = 0;
			innerScrollY = 0;
		}
	});

	onMount(() => {
		if (!container) return;
		viewportHeight = container.clientHeight || 600;
		ro = new ResizeObserver(() => {
			if (container) viewportHeight = container.clientHeight;
		});
		ro.observe(container);
	});

	onDestroy(() => {
		ro?.disconnect();
	});
</script>

<div class="vlist" bind:this={container} onscroll={handleScroll}>
	<div class="vlist-inner" style:height="{heights.total}px">
		{#each visible as entry, i (i)}
			<div class="vlist-row" style:transform="translateY({entry.offset}px)">
				{@render row(entry.item, entry.index)}
			</div>
		{/each}
	</div>
</div>

<style>
	.vlist {
		flex: 1;
		min-height: 0;
		overflow-y: auto;
		overflow-x: hidden;
		position: relative;
	}
	.vlist-inner {
		position: relative;
		width: 100%;
	}
	.vlist-row {
		position: absolute;
		inset-inline-start: 0;
		inset-inline-end: 0;
		width: 100%;
	}
</style>
