<script lang="ts">
	/**
	 * Constellation Style Setter (CSS) — MIG-070, standalone, built from scratch.
	 *
	 * A full-page "design studio": your real interface in the centre, click any part to style
	 * it, controls on the right, surfaces + theme cards on the left. Edits go to a DRAFT (CSS
	 * variable overrides scoped to the preview wrapper — the live app is untouched); **Apply**
	 * copies the draft onto the real `:root`. Deliberately independent of the old MIG-069 style
	 * code, and it renders ONE preview (never a gallery of heavy cards — that froze the old panel).
	 *
	 * Iteration 1: the editor surface live-edits the core variables (accent · backgrounds · text ·
	 * link · fonts) with Apply; other surfaces are representative previews; theme presets seed the
	 * draft. Persistence (save / name / export / import) + the full surface set come next.
	 */
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { styleSetterOpen, closeStyleSetter } from '$lib/stores/styleSetter';

	type Ctrl = { label: string; type: 'color' | 'select'; var: string; options?: [string, string][] };
	const FONTS: [string, string][] = [
		['System', 'ui-sans-serif, system-ui, "Segoe UI", sans-serif'],
		['Serif', 'ui-serif, Georgia, "Times New Roman", serif'],
		['Mono', 'ui-monospace, "Courier New", monospace'],
	];
	// Element key → its name + controls (each control writes a REAL app CSS variable).
	const ELEMENTS: Record<string, { name: string; controls: Ctrl[] }> = {
		accent:  { name: 'Accent',          controls: [{ label: 'Accent colour', type: 'color', var: '--interactive-accent' }] },
		noteBg:  { name: 'Note',            controls: [
			{ label: 'Background', type: 'color', var: '--background-primary' },
			{ label: 'Note font', type: 'select', var: '--font-text-theme', options: FONTS } ] },
		sidebar: { name: 'Sidebar',         controls: [
			{ label: 'Sidebar background', type: 'color', var: '--background-secondary' },
			{ label: 'Interface font', type: 'select', var: '--font-interface-theme', options: FONTS } ] },
		text:    { name: 'Text',            controls: [{ label: 'Text colour', type: 'color', var: '--text-normal' }] },
		link:    { name: 'Link',            controls: [{ label: 'Link colour', type: 'color', var: '--link-color' }] },
	};

	const SURFACES: [string, string][] = [
		['editor', 'Editor'], ['sky', 'Sky View'], ['org', 'OrgChart'],
		['index', 'Index'], ['cataloger', 'Cataloger'], ['shell', 'Shell'],
	];

	const THEMES: { name: string; vars: Record<string, string> }[] = [
		{ name: 'Midnight',  vars: { '--background-primary': '#11111b', '--background-secondary': '#181825', '--text-normal': '#cdd6f4', '--interactive-accent': '#cba6f7', '--link-color': '#89b4fa' } },
		{ name: 'Daylight',  vars: { '--background-primary': '#ffffff', '--background-secondary': '#f3f3f1', '--text-normal': '#1f2328', '--interactive-accent': '#0969da', '--link-color': '#0969da' } },
		{ name: 'Chocolate', vars: { '--background-primary': '#fbf3e6', '--background-secondary': '#f2e6d2', '--text-normal': '#4a3b2a', '--interactive-accent': '#b9722f', '--link-color': '#8a5a2b' } },
		{ name: 'Nord',      vars: { '--background-primary': '#2e3440', '--background-secondary': '#3b4252', '--text-normal': '#e5e9f0', '--interactive-accent': '#88c0d0', '--link-color': '#81a1c1' } },
	];

	let activeSurface = $state('editor');
	let selected = $state<string | null>(null);
	let draftName = $state('Untitled style');
	/** The draft: CSS-var → override value. Scoped to the preview wrapper; Apply → :root. */
	let draft = $state<Record<string, string>>({});

	const draftStyle = $derived(Object.entries(draft).map(([k, v]) => `${k}:${v}`).join(';'));
	const sel = $derived(selected ? ELEMENTS[selected] ?? null : null);

	function hexOf(c: string): string {
		c = (c || '').trim();
		if (c.startsWith('#')) return c.length === 4 ? '#' + [...c.slice(1)].map((x) => x + x).join('') : c;
		const m = c.match(/rgba?\((\d+)[,\s]+(\d+)[,\s]+(\d+)/);
		if (m) return '#' + [m[1], m[2], m[3]].map((x) => (+x).toString(16).padStart(2, '0')).join('');
		return '#888888';
	}
	/** Current value of a var: the draft override, else the live value — read from <body>,
	    where the app sets its theme vars (:root would not see them and the swatch would be grey). */
	function curVal(v: string): string {
		if (v in draft) return draft[v];
		try { return getComputedStyle(document.body).getPropertyValue(v).trim(); } catch { return ''; }
	}
	function setVar(v: string, val: string) { draft = { ...draft, [v]: val }; }
	function selectEl(key: string) { selected = key; }
	function pickSurface(s: string) { activeSurface = s; selected = null; }
	function applyTheme(t: { name: string; vars: Record<string, string> }) { draft = { ...draft, ...t.vars }; draftName = t.name; }

	/** hex → HSL (mirrors the app's own hexToHSL; inlined so the Setter stays standalone). */
	function hexToHSL(hex: string): { h: number; s: number; l: number } | null {
		const h6 = hexOf(hex);
		const r = parseInt(h6.slice(1, 3), 16) / 255, g = parseInt(h6.slice(3, 5), 16) / 255, b = parseInt(h6.slice(5, 7), 16) / 255;
		if ([r, g, b].some((x) => Number.isNaN(x))) return null;
		const max = Math.max(r, g, b), min = Math.min(r, g, b);
		let h = 0, s = 0; const l = (max + min) / 2;
		if (max !== min) {
			const d = max - min;
			s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
			if (max === r) h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
			else if (max === g) h = ((b - r) / d + 2) / 6;
			else h = ((r - g) / d + 4) / 6;
		}
		return { h: Math.round(h * 360), s: Math.round(s * 100), l: Math.round(l * 100) };
	}

	function apply() {
		// Iteration 1: copy the draft onto the live app for this session (direct DOM, no reactive
		// path). The app themes <body> (not :root) and shadows :root, so we MUST target body.
		const root = document.body.style;
		for (const [k, v] of Object.entries(draft)) root.setProperty(k, v);
		// The accent is also consumed as --accent-h/s/l components + --text-accent by many
		// controls, so decompose it — otherwise only elements using --interactive-accent change.
		const acc = draft['--interactive-accent'];
		if (acc) {
			const hsl = hexToHSL(acc);
			if (hsl) {
				root.setProperty('--accent-h', String(hsl.h));
				root.setProperty('--accent-s', `${hsl.s}%`);
				root.setProperty('--accent-l', `${hsl.l}%`);
				root.setProperty('--text-accent', `hsl(${hsl.h}, ${hsl.s}%, ${hsl.l}%)`);
				root.setProperty('--interactive-accent-hover', `hsl(${hsl.h}, ${hsl.s}%, ${Math.max(0, hsl.l - 8)}%)`);
			}
		}
	}
	function resetDraft() { draft = {}; selected = null; }

	onMount(() => {
		// Capture phase + stopImmediatePropagation so Escape closes ONLY the Setter, never the
		// Settings modal underneath it. No-op (and doesn't swallow Escape) when the Setter is shut.
		function onKey(e: KeyboardEvent) {
			if (e.key === 'Escape' && get(styleSetterOpen)) {
				e.preventDefault();
				e.stopImmediatePropagation();
				closeStyleSetter();
			}
		}
		window.addEventListener('keydown', onKey, true);
		return () => window.removeEventListener('keydown', onKey, true);
	});
</script>

{#if $styleSetterOpen}
	<div class="ss-overlay" role="dialog" aria-label="Style Setter">
		<div class="ss" style={draftStyle}>
			<!-- Top bar -->
			<header class="ss-top">
				<span class="ss-brand"><span class="ss-star">✦</span> Style Setter</span>
				<span class="ss-draft">draft: <input class="ss-dname" bind:value={draftName} /></span>
				<span class="ss-spacer"></span>
				<button class="ss-btn" onclick={resetDraft}>Reset</button>
				<button class="ss-btn ss-primary" onclick={apply}>Apply to app</button>
				<button class="ss-btn ss-icon" aria-label="Close" onclick={closeStyleSetter}>✕</button>
			</header>

			<!-- Left rail: surfaces + themes -->
			<aside class="ss-left">
				<div class="ss-rlabel">Surfaces</div>
				{#each SURFACES as [key, label] (key)}
					<button class="ss-surface" class:active={activeSurface === key} onclick={() => pickSurface(key)}>
						<span class="ss-sdot"></span> {label}
					</button>
				{/each}
				<div class="ss-divider"></div>
				<div class="ss-rlabel">My themes</div>
				<div class="ss-themes">
					{#each THEMES as t (t.name)}
						<button class="ss-tcard" onclick={() => applyTheme(t)} title={t.name}>
							<span class="ss-tsw">
								<span style="background:{t.vars['--background-primary']}"></span>
								<span style="background:{t.vars['--background-secondary']}"></span>
								<span style="background:{t.vars['--interactive-accent']}"></span>
							</span>
							<span class="ss-tn">{t.name}</span>
						</button>
					{/each}
					<button class="ss-tcard ss-newcard">+ new</button>
				</div>
			</aside>

			<!-- Center: live preview -->
			<main class="ss-center">
				<div class="ss-hint">Hover a part of the interface, click to style it →</div>
				<div class="ss-stage">
					{#if activeSurface === 'editor'}
						<div class="ss-prev">
							<button class="ss-side ss-hot" class:ss-sel={selected === 'sidebar'} onclick={() => selectEl('sidebar')} aria-label="Sidebar">
								<span class="ss-file">Apple (Fruit)</span>
								<span class="ss-file dim">Banana</span>
								<span class="ss-file dim">Carrot</span>
								<span class="ss-file dim">Salad Recipe</span>
							</button>
							<button class="ss-main ss-hot" class:ss-sel={selected === 'noteBg'} onclick={() => selectEl('noteBg')} aria-label="Note background">
								<span class="ss-title ss-hot2" class:ss-sel={selected === 'text'} onclick={(e) => { e.stopPropagation(); selectEl('text'); }}>Apple (Fruit)</span>
								<span class="ss-h ss-hot2" class:ss-sel={selected === 'accent'} onclick={(e) => { e.stopPropagation(); selectEl('accent'); }}>A crisp idea</span>
								<span class="ss-body">
									An apple a day pairs well with a
									<span class="ss-link ss-hot2" class:ss-sel={selected === 'link'} onclick={(e) => { e.stopPropagation(); selectEl('link'); }}>[[Banana]]</span>
									<span class="ss-pill ss-hot2" class:ss-sel={selected === 'accent'} onclick={(e) => { e.stopPropagation(); selectEl('accent'); }}>supports</span>
									in a balanced note.
								</span>
							</button>
						</div>
					{:else}
						<div class="ss-prev-alt">
							<div class="ss-alt-title">{SURFACES.find(([k]) => k === activeSurface)?.[1]}</div>
							{#if activeSurface === 'sky' || activeSurface === 'org'}
								<div class="ss-sky">
									<button class="ss-node ss-hot" class:ss-sel={selected === 'accent'} onclick={() => selectEl('accent')} aria-label="accent"></button>
									<button class="ss-node b ss-hot" class:ss-sel={selected === 'link'} onclick={() => selectEl('link')} aria-label="link"></button>
								</div>
							{:else if activeSurface === 'index'}
								<div class="ss-idx">
									<div class="ss-irow"><button class="ss-ibar ss-hot" style="width:70%" class:ss-sel={selected === 'accent'} onclick={() => selectEl('accent')} aria-label="accent"></button> apple</div>
									<div class="ss-irow"><span class="ss-ibar" style="width:45%"></span> banana</div>
									<div class="ss-irow"><span class="ss-ibar" style="width:30%"></span> carrot</div>
								</div>
							{/if}
							<div class="ss-alt-note">representative snapshot · re-colours with your edits · click a part to style it</div>
						</div>
					{/if}
				</div>
			</main>

			<!-- Right rail: controls for the selected element -->
			<aside class="ss-right">
				{#if sel}
					<div class="ss-rlabel">Selected element</div>
					<div class="ss-selname">{sel.name}</div>
					{#each sel.controls as c (c.var)}
						<div class="ss-ctrl">
							<label for={'ss-' + c.var}>{c.label}</label>
							{#if c.type === 'color'}
								<input id={'ss-' + c.var} type="color" value={hexOf(curVal(c.var))} oninput={(e) => setVar(c.var, (e.currentTarget as HTMLInputElement).value)} />
							{:else if c.type === 'select'}
								<select id={'ss-' + c.var} onchange={(e) => setVar(c.var, (e.currentTarget as HTMLSelectElement).value)}>
									{#each c.options ?? [] as [lbl, val] (val)}<option value={val}>{lbl}</option>{/each}
								</select>
							{/if}
						</div>
					{/each}
				{:else}
					<div class="ss-empty"><span class="ss-big">⊹</span>Click any part of the interface to style it. Its controls appear here, and changes show instantly.</div>
				{/if}
			</aside>
		</div>
	</div>
{/if}

<style>
	.ss-overlay {
		position: fixed; inset: 0; z-index: 9000; display: flex; align-items: center; justify-content: center;
		background: rgba(6, 6, 12, 0.62); backdrop-filter: blur(2px); padding: 16px;
	}
	.ss {
		/* Chrome follows the theme being edited (the .ss element carries the draft + inherits the
		   app theme), with the original dark studio look as fallback (MIG-070 §iter2-#2, Eisa). */
		--c-bg: var(--background-primary, #15151f); --c-surface: var(--background-secondary, #1d1d2a); --c-surface2: var(--background-modifier-hover, #24243440); --c-text: var(--text-normal, #cfd0e0);
		--c-muted: var(--text-muted, #8a8ba0); --c-border: var(--background-modifier-border, #2c2c3e); --c-accent: var(--interactive-accent, #7c6cff);
		width: 100%; max-width: 1180px; height: min(92vh, 760px); background: var(--c-bg);
		border: 1px solid var(--c-border); border-radius: 14px; overflow: hidden; color: var(--c-text);
		display: grid; grid-template-rows: 52px 1fr; grid-template-columns: 210px 1fr 248px;
		grid-template-areas: "top top top" "left center right"; box-shadow: 0 30px 80px rgba(0,0,0,.55);
		font-family: ui-sans-serif, system-ui, "Segoe UI", sans-serif;
	}
	.ss-top { grid-area: top; display: flex; align-items: center; gap: 12px; padding: 0 16px; border-bottom: 1px solid var(--c-border); background: var(--c-surface); }
	.ss-brand { font-weight: 700; } .ss-star { color: var(--c-accent); }
	.ss-draft { color: var(--c-muted); font-size: 13px; }
	.ss-dname { font: inherit; font-size: 13px; color: var(--c-text); background: var(--c-surface2); border: 1px solid var(--c-border); border-radius: 6px; padding: 3px 8px; width: 140px; }
	.ss-spacer { flex: 1; }
	.ss-btn { font: inherit; font-size: 13px; padding: 6px 13px; border-radius: 7px; border: 1px solid var(--c-border); background: var(--c-surface2); color: var(--c-text); cursor: pointer; }
	.ss-btn:hover { border-color: var(--c-accent); }
	.ss-primary { background: var(--c-accent); border-color: var(--c-accent); color: #fff; font-weight: 600; }
	.ss-icon { padding: 6px 9px; }
	.ss-left { grid-area: left; border-right: 1px solid var(--c-border); background: var(--c-surface); display: flex; flex-direction: column; padding: 12px 10px; gap: 5px; overflow-y: auto; }
	.ss-rlabel { font-size: 11px; text-transform: uppercase; letter-spacing: .07em; color: var(--c-muted); margin: 6px 4px 2px; }
	.ss-surface { display: flex; align-items: center; gap: 9px; padding: 7px 9px; border-radius: 8px; cursor: pointer; font: inherit; font-size: 13.5px; color: var(--c-text); background: none; border: none; text-align: left; }
	.ss-surface:hover { background: var(--c-surface2); }
	.ss-surface.active { background: color-mix(in srgb, var(--c-accent) 22%, transparent); color: #fff; }
	.ss-sdot { width: 7px; height: 7px; border-radius: 50%; background: currentColor; opacity: .55; flex: none; }
	.ss-surface.active .ss-sdot { opacity: 1; background: var(--c-accent); }
	.ss-divider { height: 1px; background: var(--c-border); margin: 8px 2px; }
	.ss-themes { display: grid; grid-template-columns: 1fr 1fr; gap: 7px; }
	.ss-tcard { border: 1px solid var(--c-border); border-radius: 8px; overflow: hidden; cursor: pointer; background: var(--c-surface2); padding: 0; }
	.ss-tcard:hover { border-color: var(--c-accent); }
	.ss-tsw { height: 28px; display: flex; } .ss-tsw span { flex: 1; }
	.ss-tn { display: block; font-size: 11px; padding: 4px 6px; color: var(--c-muted); text-align: left; }
	.ss-newcard { display: flex; align-items: center; justify-content: center; min-height: 50px; color: var(--c-muted); font-size: 12px; border-style: dashed; }
	.ss-center { grid-area: center; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 20px; gap: 10px; background: var(--background-secondary, #14141c); }
	.ss-hint { font-size: 12px; color: var(--c-muted); }
	.ss-stage { position: relative; }
	/* The mini interface — uses the REAL app vars (overridden by the draft on .ss). */
	.ss-prev { width: 560px; height: 360px; border-radius: 10px; overflow: hidden; display: grid; grid-template-columns: 124px 1fr; background: var(--background-primary, #fbfbfa); box-shadow: 0 14px 40px rgba(0,0,0,.45); border: 1px solid rgba(0,0,0,.25); }
	.ss-side { background: var(--background-secondary, #f1f1ef); color: var(--text-normal, #2e3338); padding: 12px 10px; display: flex; flex-direction: column; gap: 8px; border: none; text-align: left; font-family: var(--font-interface-theme, inherit); }
	.ss-file { font-size: 11.5px; display: flex; align-items: center; gap: 6px; } .ss-file.dim { opacity: .55; }
	.ss-file::before { content: ""; width: 6px; height: 6px; border-radius: 50%; background: var(--interactive-accent, #7c3aed); flex: none; } .ss-file.dim::before { background: currentColor; opacity: .4; }
	.ss-main { background: var(--background-primary, #fbfbfa); color: var(--text-normal, #2e3338); padding: 18px 20px; text-align: left; border: none; font-family: var(--font-text-theme, inherit); display: block; }
	.ss-title { display: block; font-weight: 800; font-size: 19px; margin-bottom: 12px; color: var(--text-normal, #2e3338); }
	.ss-h { display: block; font-weight: 700; font-size: 22px; margin-bottom: 10px; color: var(--interactive-accent, #c0392b); }
	.ss-body { display: block; font-size: 14px; line-height: 1.75; color: var(--text-normal, #2e3338); }
	.ss-link { color: var(--link-color, var(--interactive-accent, #2f6fed)); text-decoration: underline; }
	.ss-pill { display: inline-flex; align-items: center; background: var(--interactive-accent, #4a9eff); color: #fff; font-size: 11px; font-weight: 700; padding: 1px 8px; border-radius: 9px; text-transform: lowercase; }
	/* Hover/selected rings drawn INSIDE the element (inset box-shadow) so they're never clipped
	   by the preview's overflow:hidden — that was why the edge-touching sidebar/note showed nothing. */
	.ss-hot { cursor: pointer; } .ss-hot:hover { box-shadow: inset 0 0 0 2px #9d8dff; }
	.ss-hot2 { cursor: pointer; border-radius: 3px; } .ss-hot2:hover { outline: 2px dashed #9d8dff; outline-offset: 2px; }
	.ss-sel { box-shadow: inset 0 0 0 2.5px #b9acff !important; }
	.ss-hot2.ss-sel { box-shadow: none !important; outline: 2.5px solid #b9acff !important; outline-offset: 2px; }
	.ss-prev-alt { width: 560px; height: 360px; border-radius: 10px; background: var(--background-primary, #fbfbfa); color: var(--text-normal, #2e3338); box-shadow: 0 14px 40px rgba(0,0,0,.45); border: 1px solid rgba(0,0,0,.25); display: flex; align-items: center; justify-content: center; flex-direction: column; gap: 16px; }
	.ss-alt-title { font-weight: 700; font-size: 15px; color: var(--interactive-accent, #7c3aed); }
	.ss-alt-note { font-size: 11.5px; color: var(--text-normal, #6b7280); opacity: .7; max-width: 70%; text-align: center; }
	.ss-sky { display: flex; gap: 22px; }
	.ss-node { width: 34px; height: 34px; border-radius: 50%; border: none; cursor: pointer; background: var(--interactive-accent, #7c3aed); box-shadow: 0 0 0 4px color-mix(in srgb, var(--interactive-accent, #7c3aed) 25%, transparent); }
	.ss-node.b { background: var(--link-color, #2f6fed); box-shadow: 0 0 0 4px color-mix(in srgb, var(--link-color, #2f6fed) 25%, transparent); }
	.ss-idx { width: 70%; display: flex; flex-direction: column; gap: 8px; }
	.ss-irow { display: flex; align-items: center; gap: 8px; font-size: 12px; }
	.ss-ibar { height: 7px; background: var(--interactive-accent, #7c3aed); border-radius: 3px; border: none; cursor: pointer; }
	.ss-right { grid-area: right; border-left: 1px solid var(--c-border); background: var(--c-surface); padding: 14px; overflow-y: auto; }
	.ss-selname { font-size: 16px; font-weight: 700; margin-bottom: 14px; }
	.ss-ctrl { margin-bottom: 14px; }
	.ss-ctrl label { display: block; font-size: 12px; color: var(--c-muted); margin-bottom: 5px; }
	.ss-ctrl input[type=color] { width: 100%; height: 30px; border: 1px solid var(--c-border); border-radius: 6px; background: none; cursor: pointer; }
	.ss-ctrl select { width: 100%; padding: 6px 8px; border-radius: 6px; border: 1px solid var(--c-border); background: #1d1d2a; color: #e8e9f3; font: inherit; font-size: 13px; }
	.ss-ctrl select option { background: #1d1d2a; color: #e8e9f3; }
	.ss-empty { color: var(--c-muted); font-size: 13px; line-height: 1.6; margin-top: 28px; text-align: center; }
	.ss-big { font-size: 26px; opacity: .5; display: block; margin-bottom: 8px; }
</style>
