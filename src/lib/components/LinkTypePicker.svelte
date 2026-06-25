<script lang="ts">
	// MIG-086 §C — the 8-type Link picker. No reusable type picker existed (only the
	// CM6 `[[`-autocomplete, which isn't openable as UI), so this builds one on the
	// link-type registry + the self-contained LinkTypePill. It is the deliberate
	// "what kind of relationship?" step every suggestion-born connection passes
	// through (concept invariant C-1: one Link button per candidate, ALWAYS typed —
	// never a bulk accept). Interaction shape copied from the confidence popover
	// (OutgoingLinksPanel.svelte): a fixed overlay + a button-per-option menu.
	import { t, dir as uiDir } from '$lib/i18n';
	import { linkTypesStore, getLinkTypes, getLinkType } from '$lib/libraries/linkTypeRegistry';
	import LinkTypePill from './LinkTypePill.svelte';

	let {
		x,
		y,
		defaultType = 'associative',
		onChoose,
		onCancel,
	}: {
		x: number;
		y: number;
		defaultType?: string;
		onChoose: (type: string) => void;
		onCancel: () => void;
	} = $props();

	// `associative` is the null/default type — excluded from the registry's SEED_IDS,
	// so prepend it manually (it leads the list as the sensible "these relate" default).
	// Use getLinkTypes() (the FULL resolved list: the 8 seeds + custom top-level types,
	// each followed by its custom children in canonical order) — NOT topLevelLinkTypes() —
	// so custom CHILD types are selectable here too, matching the editor's `[[` autocomplete
	// (completions.ts uses getLinkTypes()). `void $linkTypesStore` makes the list reactive
	// to a §G vocabulary edit / recolour.
	const types = $derived.by<string[]>(() => {
		void $linkTypesStore;
		const all = getLinkTypes().map((d) => d.id).filter((id) => id !== 'associative');
		return ['associative', ...all];
	});

	// Keyboard: Esc cancels (the overlay handles click-away).
	function onKey(e: KeyboardEvent) {
		if (e.key === 'Escape') { e.preventDefault(); onCancel(); }
	}

	// Viewport-clamp the menu so it never runs off-screen (the Link buttons sit at the
	// pane's right edge, so an un-clamped menu opened at the click point overflowed right
	// and truncated the type labels — Boss finding). Measure once after mount, then shift
	// the menu fully into view (it opens leftward/upward from the click as needed). Hidden
	// until placed so there's no flash at the un-clamped position.
	let menuEl = $state<HTMLDivElement>();
	// `pos` is null until the clamp runs; the template falls back to the raw x/y props
	// (read in markup — no reactive-capture warning) while hidden, then snaps to `pos`.
	let pos = $state<{ left: number; top: number; maxHeight: number } | null>(null);
	$effect(() => {
		if (!menuEl || pos) return;
		const pad = 8;
		const vw = window.innerWidth;
		const vh = window.innerHeight;
		// Measure the menu's NATURAL height (no CSS cap while hidden — see the removed
		// `max-height: 60vh`). The 8 seeds + associative + the user's CUSTOM types can make
		// the list taller than the window; cap it to the viewport in JS px (window.innerHeight,
		// NOT a `vh` CSS unit the webview wasn't honoring) and clamp the top so it NEVER runs
		// off the bottom edge — it scrolls in place instead (Boss finding: picker truncated).
		const r = menuEl.getBoundingClientRect();
		const desiredH = r.height || vh; // natural (uncapped) height; fall back to full viewport
		let nx = x;
		let ny = y;
		if (nx + r.width + pad > vw) nx = Math.max(pad, vw - r.width - pad);
		if (ny + desiredH + pad > vh) ny = Math.max(pad, vh - desiredH - pad);
		// Cap to the space from the CLAMPED top down to the viewport bottom: bottom = ny +
		// maxHeight = vh − pad, ALWAYS on-screen — even if `desiredH` was measured stale (e.g.
		// the custom-type list populated after the first measure). The menu scrolls in place.
		const maxHeight = vh - ny - pad;
		pos = { left: nx, top: ny, maxHeight };
	});
</script>

<svelte:window onkeydown={onKey} />

<!-- A real <button> for the click-away scrim → no a11y warning, keyboard-dismissable. -->
<button
	class="ltp-overlay"
	type="button"
	aria-label={$t('common.close') || 'Close'}
	onclick={() => onCancel()}
	oncontextmenu={(e) => { e.preventDefault(); onCancel(); }}
></button>
<div class="ltp-menu" class:placed={pos} bind:this={menuEl} style="left:{pos?.left ?? x}px;top:{pos?.top ?? y}px;max-height:{pos ? pos.maxHeight + 'px' : 'none'}" dir={$uiDir}>
	<div class="ltp-header">{$t('reviewer.pickTypeTitle') || 'How do they relate?'}</div>
	{#each types as id (id)}
		<button
			class="ltp-item"
			class:active={id === defaultType}
			onclick={() => onChoose(id)}
			title={getLinkType(id)?.desc ?? ''}
		>
			<LinkTypePill {id} />
		</button>
	{/each}
</div>

<style>
	.ltp-overlay {
		position: fixed;
		inset: 0;
		z-index: 1000;
		/* a transparent full-screen scrim button — strip all default button chrome */
		border: none;
		background: transparent;
		padding: 0;
		margin: 0;
		cursor: default;
	}
	.ltp-menu {
		position: fixed;
		z-index: 1001;
		min-width: 180px;
		max-width: min(320px, 90vw);
		/* max-height is set INLINE in px by the clamp $effect (window.innerHeight − padding).
		   No CSS `max-height: 60vh` here: it would cap the natural-height measurement and the
		   webview wasn't honoring `vh` reliably, so a long type list ran off the bottom. */
		overflow-y: auto;
		visibility: hidden; /* shown only after the clamp $effect places it (no flash) */
	}
	.ltp-menu.placed {
		visibility: visible;
		padding: 6px;
		border-radius: 10px;
		background: var(--background-primary, #fff);
		border: 1px solid var(--background-modifier-border, #ccc);
		box-shadow: 0 8px 28px rgba(0, 0, 0, 0.22);
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.ltp-header {
		font-size: calc(0.66rem * var(--rs-scale, 1));
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-faint, #999);
		font-weight: 600;
		padding: 4px 8px 6px;
		text-align: start;
	}
	.ltp-item {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		text-align: start;
		padding: 6px 8px;
		border: 1px solid transparent;
		border-radius: 7px;
		background: none;
		cursor: pointer;
		font-family: inherit;
	}
	.ltp-item:hover {
		background: var(--background-modifier-hover, rgba(0, 0, 0, 0.06));
	}
	.ltp-item.active {
		border-color: var(--interactive-accent, #7c3aed);
		background: color-mix(in srgb, var(--interactive-accent, #7c3aed) 12%, transparent);
	}
</style>
