<script lang="ts">
	// 2026-07-25 (Boss-chosen icon D + the Whole-Ecosystem Fix Law): ONE library-mark
	// component every surface uses, so the icon can never drift between the sidebar,
	// the Move picker, the Library picker, the Dashboard, the toolbar and the Style Setter.
	//   kind='library'   → the "library building" mark (pediment + columns).
	//   kind='cuniverse' → the planet/orbit mark (a federated child universe).
	//   kind='root'      → NO icon. The Universe root holds cUniverses + Libraries, not
	//                      content (Boss, 2026-07-25; forward-compatible with MIG-105).
	//   size   → number (px) OR a CSS value ("var(--ft-library-icon-size, 13px)"). OMIT it
	//            to let an AMBIENT css rule size the svg (e.g. the toolbar's
	//            `.tb-btn svg { width: var(--sidebar-icon-size) }`) — an inline size would
	//            override that rule (the "dead icon" bug), so no-size = no inline width.
	//   strokeWidth → matches the host's icon weight (toolbar siblings use 2).
	let {
		kind = 'library',
		size = undefined,
		strokeWidth = 1.7,
		color = 'var(--interactive-accent)',
	}: {
		kind?: 'library' | 'cuniverse' | 'root';
		size?: number | string;
		strokeWidth?: number;
		color?: string;
	} = $props();

	const dim = $derived(size === undefined ? undefined : typeof size === 'number' ? `${size}px` : size);
	const sizeStyle = $derived(dim ? `width:${dim};height:${dim};flex-shrink:0` : 'flex-shrink:0');
</script>

{#if kind === 'cuniverse'}
	<svg viewBox="0 0 24 24" fill="none" stroke={color} stroke-width={strokeWidth} style={sizeStyle}>
		<circle cx="12" cy="12" r="6" />
		<line x1="6" y1="12" x2="18" y2="12" />
		<ellipse cx="12" cy="12" rx="11" ry="3.5" transform="rotate(-25 12 12)" stroke-dasharray="2,2" />
	</svg>
{:else if kind === 'library'}
	<!-- 2026-07-25 (Boss): the building glyph enlarged to fill more of the 24×24 box so
	     the library icon reads ~1.5× larger at any rendered size — WITHOUT touching any
	     caller's size or the Style Setter control. -->
	<svg viewBox="0 0 24 24" fill="none" stroke={color} stroke-width={strokeWidth} stroke-linejoin="round" stroke-linecap="round" style={sizeStyle}>
		<path d="M12 2L2 8.5h20z" />
		<line x1="5.5" y1="11" x2="5.5" y2="18.5" />
		<line x1="10" y1="11" x2="10" y2="18.5" />
		<line x1="14.5" y1="11" x2="14.5" y2="18.5" />
		<line x1="19" y1="11" x2="19" y2="18.5" />
		<line x1="2.5" y1="21" x2="21.5" y2="21" />
	</svg>
{/if}
<!-- kind='root' renders nothing -->
