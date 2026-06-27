<script lang="ts">
	/**
	 * MIG-067 §G — the Settings → Link Types editor (the "Living Vocabulary").
	 *
	 * Lists the resolved Link-Type Registry (the 8 built-in typed acts + any
	 * user-defined types, nested under their parent) and lets the user:
	 *   • ADD a custom type — label (→ slug id), colour, and parent (top-level OR
	 *     a child of one of the 8);
	 *   • RECOLOUR any type (the 8 are recolourable, just not deletable);
	 *   • DELETE a custom type (the 8 are locked).
	 *
	 * Saving persists the whole list as deltas to `.constellation/link-types.json`
	 * (`saveLinkTypes`) — the Rust merge treats seed-id entries as presentation
	 * overrides and keeps their grammar — and re-seeds the registry, so every
	 * surface (editor colours, autocomplete, 360.3D, Base columns) updates. A
	 * vocabulary change (add/delete) re-materialises the Base aggregates write-time;
	 * a colour-only change is cheap (the registry fingerprint is over ids).
	 */
	import { onMount } from 'svelte';
	import { t, locale } from '$lib/i18n';
	import { detectDir } from '$lib/utils';
	import {
		cognitiveLinkTypes, linkTypesStore, loadLinkTypes, saveLinkTypes, toLinkTypeDeltas, SEED_IDS, SEED_DEFAULTS, type LinkTypeDef,
	} from '$lib/libraries/linkTypeRegistry';
	// MIG-070 §C Phase 5 — the universal saved-colour palette (the SAME store the interface
	// elements use), so a link colour you pick is remembered and reusable for any element (Eisa).
	import { appSettings, addStyleSwatch } from '$lib/libraries/store';
	// §C redesign — in the Setter (embedded) each row shows its LIVE pill in place of the plain
	// name, so the control IS the preview (one integrated surface, no duplicate pill row — Eisa).
	import LinkTypePill from './LinkTypePill.svelte';

	// MIG-070 §C Phase 5 — when embedded in the Style Setter's "Links" category, hide the
	// Settings-scoped heading/desc (the Setter shows its own element name); the list/add/reset
	// (all `.lte-*` self-styled) render unchanged. Default false → the Settings tab is untouched.
	let { embedded = false }: { embedded?: boolean } = $props();

	// The type whose colour box was last touched — so a click on the universal swatch palette
	// knows which type to recolour (mirrors the Setter's `activeColorVar`). Embedded-only.
	let activeTypeId = $state<string | null>(null);

	let types = $state<LinkTypeDef[]>([]);
	let loaded = $state(false);
	let saving = $state(false);
	let error = $state<string | null>(null);

	// Add-type form.
	let newLabel = $state('');
	let newColor = $state('#7FB8FF');
	let newParent = $state(''); // '' = top-level

	const SEED_SET = new Set(SEED_IDS as readonly string[]);
	const isSeed = (id: string) => SEED_SET.has(id);

	// §H — show each type's name localized (linkTypes.<id>), falling back to the
	// registry label for custom types with no i18n key; flip the nesting arrow in RTL.
	const isRtl = $derived(['ar', 'fa', 'he', 'ur'].includes($locale));
	function typeName(tp: LinkTypeDef): string {
		const k = 'linkTypes.' + tp.id;
		const tr = $t(k);
		return tr !== k ? tr : tp.label;
	}

	/** The 8 built-ins (top-level seeds), for the parent dropdown. */
	const seedTypes = $derived(types.filter((tp) => isSeed(tp.id)));
	/** Render order: each top-level type immediately followed by its children. */
	const ordered = $derived.by(() => {
		const top = types.filter((tp) => tp.parent == null);
		const out: { def: LinkTypeDef; depth: number }[] = [];
		for (const p of top) {
			out.push({ def: p, depth: 0 });
			for (const c of types.filter((tp) => tp.parent === p.id)) out.push({ def: c, depth: 1 });
		}
		return out;
	});

	function slug(s: string): string {
		return s.trim().toLowerCase().replace(/[^a-z0-9-]+/g, '-').replace(/^-+|-+$/g, '');
	}
	const newId = $derived(slug(newLabel));
	const newIdTaken = $derived(newId.length > 0 && types.some((tp) => tp.id === newId));

	async function refresh() {
		await loadLinkTypes();
		types = cognitiveLinkTypes().map((tp) => ({ ...tp }));
	}

	async function persist() {
		saving = true;
		error = null;
		try {
			await saveLinkTypes(toLinkTypeDeltas(types));
			types = cognitiveLinkTypes().map((tp) => ({ ...tp })); // reflect the resolved order
		} catch (e) {
			error = typeof e === 'string' ? e : (e as Error)?.message ?? String(e);
		} finally {
			saving = false;
		}
	}

	function addType() {
		const id = newId;
		if (!id || newIdTaken) return;
		const parent = newParent && isSeed(newParent) ? newParent : null;
		const picked = newColor;
		types = [
			...types,
			{
				id, label: newLabel.trim() || id, parent,
				color: picked, order: 999, builtin: false, emoji: null, desc: null,
			},
		];
		newLabel = ''; newColor = '#7FB8FF'; newParent = '';
		if (embedded) addStyleSwatch(picked); // remember the colour in the universal palette
		persist();
	}

	function deleteType(id: string) {
		if (isSeed(id)) return; // the 8 are locked
		// also drop any children of a deleted custom parent (defensive; v1 nests only under the 8)
		types = types.filter((tp) => tp.id !== id && tp.parent !== id);
		persist();
	}

	function recolor(id: string, color: string) {
		// `<input type=color>` onchange fires once on commit (not per drag frame),
		// so persisting here is one save per recolour. A colour-only change doesn't
		// shift the registry fingerprint → no Base re-materialise (cheap).
		activeTypeId = id;
		types = types.map((tp) => (tp.id === id ? { ...tp, color } : tp));
		if (embedded) addStyleSwatch(color); // auto-save to the universal palette
		persist();
	}

	/** §C Phase 5 — apply a saved swatch to the type whose colour box you last touched. */
	function applySwatch(hex: string) {
		if (activeTypeId) recolor(activeTypeId, hex);
	}

	/** Reset every built-in (seed) type's colour to its original default — "go back to
	 *  the original colour scheme". Custom types keep their colours (they have no default). */
	function resetColors() {
		types = types.map((tp) => (SEED_DEFAULTS[tp.id] ? { ...tp, color: SEED_DEFAULTS[tp.id].color } : tp));
		persist();
	}

	// §C — keep the swatches reactive to the registry so applying a Style (or any external
	// recolour) updates them LIVE in Settings, not on reopen. The store mirrors the registry.
	$effect(() => {
		const reg = $linkTypesStore;
		if (reg.length) types = reg.map((tp) => ({ ...tp }));
	});

	onMount(async () => {
		await refresh();
		if (!activeTypeId && types.length) activeTypeId = types[0].id; // a default target for swatch clicks
		loaded = true;
	});
</script>

<div class="lte">
	{#if !embedded}
		<div class="setting-section-heading">{$t('settings.linkTypes.title') || 'Link Types'}</div>
		<div class="setting-desc" style="margin-bottom: 10px;">
			{$t('settings.linkTypes.desc') || 'The vocabulary of typed connections. Add your own types — top-level, or nested under one of the eight built-ins — to match how you think. They flow into the editor, autocomplete, 360.3D, and Base columns. The eight built-ins can be recoloured but not deleted.'}
		</div>
	{/if}

	{#if !loaded}
		<div class="lte-state">{$t('lensBlock.loading') || 'Loading…'}</div>
	{:else}
		<div class="lte-list">
			{#each ordered as { def, depth } (def.id)}
				<div class="lte-row" class:lte-child={depth === 1} class:lte-active={embedded && activeTypeId === def.id}>
					{#if depth === 1}<span class="lte-nest">{isRtl ? '↲' : '↳'}</span>{/if}
					{#if embedded}
						<!-- §C redesign — the pill IS the colour control (Eisa: drop the empty colour box; the
						     exact pill already shows shape + colour + text). Click it → the native picker. -->
						<label class="lte-pillpick" title={`Recolour ${def.label}`}>
							<LinkTypePill id={def.id} />
							<input
								type="color"
								value={def.color}
								aria-label={`Colour for ${def.label}`}
								onfocus={() => (activeTypeId = def.id)}
								onchange={(e) => recolor(def.id, (e.target as HTMLInputElement).value)}
							/>
						</label>
					{:else}
						<input
							type="color"
							class="color-input"
							value={def.color}
							aria-label={`Colour for ${def.label}`}
							onchange={(e) => recolor(def.id, (e.target as HTMLInputElement).value)}
						/>
						<span class="lte-name" dir={detectDir(typeName(def))}>{typeName(def)}</span>
					{/if}
					<span class="lte-id">· {def.id}</span>
					{#if isSeed(def.id)}
						<span class="lte-locked">{$t('settings.linkTypes.builtin') || 'built-in'}</span>
					{:else}
						<button class="lte-del" title={$t('common.delete') || 'Delete'} onclick={() => deleteType(def.id)}>✕</button>
					{/if}
				</div>
			{/each}
		</div>

		<button class="lte-reset" onclick={resetColors}>{$t('settings.linkTypes.resetColours') || 'Reset colours to default'}</button>

		{#if embedded && ($appSettings.styleSwatches ?? []).length}
			<!-- §C Phase 5 — the universal saved-colour palette (shared with the interface elements):
			     a colour you picked is reusable; click a swatch to recolour the highlighted type. -->
			<div class="lte-sw-label">Saved colours — click to apply to the highlighted type (rename / remove via Manage on any colour element)</div>
			<div class="lte-swatches">
				{#each $appSettings.styleSwatches as sw (sw.hex)}
					<button class="lte-sw" style="background:{sw.hex}" title={(sw.name ? sw.name + ' · ' : '') + sw.hex + ' — click to apply'} aria-label={sw.name || sw.hex} onclick={() => applySwatch(sw.hex)}></button>
				{/each}
			</div>
		{/if}

		<!-- Add a type -->
		<div class="lte-add">
			<input
				type="color"
				class="color-input"
				bind:value={newColor}
				aria-label={$t('settings.linkTypes.newColor') || 'New type colour'}
			/>
			<input
				class="lte-add-label"
				type="text"
				bind:value={newLabel}
				placeholder={$t('settings.linkTypes.newLabel') || 'New type name…'}
				dir={detectDir(newLabel)}
				onkeydown={(e) => { if (e.key === 'Enter') addType(); }}
			/>
			<select class="lte-add-parent" bind:value={newParent} aria-label={$t('settings.linkTypes.parent') || 'Parent'}>
				<option value="">{$t('settings.linkTypes.topLevel') || 'Top-level'}</option>
				{#each seedTypes as st (st.id)}
					<option value={st.id}>{isRtl ? '↲' : '↳'} {typeName(st)}</option>
				{/each}
			</select>
			<button class="lte-add-btn" disabled={!newId || newIdTaken} onclick={addType}>
				{$t('settings.linkTypes.add') || 'Add'}
			</button>
		</div>
		{#if newIdTaken}
			<div class="lte-hint lte-warn">{$t('settings.linkTypes.idTaken') || 'A type with that id already exists.'}</div>
		{:else if newId}
			<div class="lte-hint">{$t('settings.linkTypes.willCreate') || 'Will create'}: <code>{newId}</code></div>
		{/if}

		{#if error}<div class="lte-state lte-warn" dir="auto">{error}</div>{/if}
		{#if saving}<div class="lte-hint">{$t('settings.linkTypes.saving') || 'Saving…'}</div>{/if}
	{/if}
</div>

<style>
	.lte { margin-block: 6px; }
	.lte-list { display: flex; flex-direction: column; gap: 2px; margin-bottom: 12px; }
	.lte-row {
		display: flex; align-items: center; gap: 8px;
		padding: 4px 6px; border-radius: 6px;
	}
	.lte-row:hover { background: var(--background-modifier-hover); }
	.lte-child { margin-inline-start: 20px; }
	.lte-nest { color: var(--text-faint); margin-inline-end: -2px; }
	.lte-name { font-size: 0.9rem; color: var(--text-normal); }
	.lte-id { font-size: 0.74rem; color: var(--text-faint); font-family: var(--font-monospace, monospace); }
	.lte-locked {
		margin-inline-start: auto; font-size: 0.66rem; color: var(--text-faint);
		background: var(--background-modifier-hover); padding: 1px 7px; border-radius: 999px;
		text-transform: lowercase;
	}
	.lte-del {
		margin-inline-start: auto; background: none; border: none; cursor: pointer;
		color: var(--text-faint); font-size: 0.9rem; padding: 2px 6px; border-radius: 5px;
	}
	.lte-del:hover { color: var(--text-error, #e53e3e); background: var(--background-modifier-hover); }
	.lte-add {
		display: flex; align-items: center; gap: 8px; margin-top: 4px;
		padding: 8px; border: 1px dashed var(--background-modifier-border); border-radius: 8px;
	}
	.lte-add-label {
		flex: 1; min-width: 0; padding: 5px 8px; font: inherit; font-size: 0.86rem;
		border: 1px solid var(--background-modifier-border); border-radius: 6px;
		background: var(--background-primary); color: var(--text-normal); outline: none;
	}
	.lte-add-label:focus { border-color: var(--interactive-accent); }
	.lte-add-parent {
		padding: 5px 6px; font: inherit; font-size: 0.82rem;
		border: 1px solid var(--background-modifier-border); border-radius: 6px;
		background: var(--background-primary); color: var(--text-normal);
	}
	.lte-add-btn {
		padding: 5px 14px; font: inherit; font-size: 0.84rem; font-weight: 600;
		border: none; border-radius: 6px; cursor: pointer;
		background: var(--interactive-accent); color: var(--text-on-accent, #fff);
	}
	.lte-add-btn:disabled { opacity: 0.45; cursor: default; }
	.lte-hint { font-size: 0.76rem; color: var(--text-muted); padding: 6px 2px 0; }
	.lte-hint code { font-family: var(--font-monospace, monospace); color: var(--text-normal); }
	.lte-warn { color: var(--text-error, #e53e3e); }
	.lte-state { color: var(--text-muted); font-size: 0.82rem; padding: 8px 2px; }
	.lte-reset {
		background: none; border: none; cursor: pointer; color: var(--text-muted);
		font-size: 0.76rem; padding: 5px 2px; text-decoration: underline; text-underline-offset: 2px;
	}
	.lte-reset:hover { color: var(--text-normal); }
	/* MIG-070 §C Phase 5 — self-contained colour swatch: a fixed PILL, identical in Settings AND the
	   Style Setter. SettingsModal's `.color-input` is scoped and never reached the embedded editor,
	   so the native input took varying widths in the flex row (Eisa remark 1). `flex: none` + a fixed
	   size pins it; the -webkit pseudo-elements fill the pill edge-to-edge (Tauri = Chromium WebView). */
	.color-input {
		flex: none; width: 46px; height: 22px; padding: 0;
		border: 1px solid var(--background-modifier-border); border-radius: 999px;
		background: none; cursor: pointer;
	}
	.color-input::-webkit-color-swatch-wrapper { padding: 0; }
	.color-input::-webkit-color-swatch { border: none; border-radius: 999px; }
	/* §C redesign — the pill doubles as the colour control: a transparent native picker overlays it,
	   so a click anywhere on the pill opens the OS colour dialog (Eisa: no separate empty colour box). */
	.lte-pillpick { position: relative; display: inline-flex; cursor: pointer; border-radius: 999px; flex: none; }
	.lte-pillpick input[type="color"] { position: absolute; inset: 0; width: 100%; height: 100%; opacity: 0; margin: 0; padding: 0; border: none; cursor: pointer; }
	.lte-pillpick:hover { outline: 2px solid var(--background-modifier-border); outline-offset: 2px; }
	/* The row whose colour a swatch-click will recolour (embedded Setter only). */
	.lte-active { background: color-mix(in srgb, var(--interactive-accent) 12%, transparent); box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--interactive-accent) 35%, transparent); }
	.lte-sw-label { font-size: 0.7rem; color: var(--text-muted); padding: 6px 2px 4px; line-height: 1.4; }
	.lte-swatches { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 10px; }
	.lte-sw { width: 22px; height: 22px; border-radius: 5px; border: 1px solid var(--background-modifier-border); cursor: pointer; padding: 0; }
	.lte-sw:hover { outline: 2px solid var(--interactive-accent); outline-offset: 1px; }
</style>
