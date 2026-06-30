<script lang="ts">
	/**
	 * CalloutTypesEditor — the bespoke block mounted in the Style Setter's
	 * "Callouts" element (Editor category), alongside its 10 colour controls.
	 *
	 * MIG-089 Phase A: a per-family ICON picker (built-in families).
	 * MIG-089 Phase B: a CUSTOM callout types manager (add / recolour / re-icon /
	 * remove your own `[!trigger]` types). All per-Universe; an open editor repaints
	 * live via NotePane's refreshCallouts hook. Reuses EmojiIconPicker / SlotIcon /
	 * iconOverrides — no parallel icon system.
	 */
	import { t } from '$lib/i18n';
	import { appSettings } from '$lib/libraries/store';
	import { setOverride } from '$lib/theme/iconOverrides';
	import { CALLOUT_FAMILIES, calloutDefaultIcon } from '$lib/editor/calloutPlugin';
	import { sanitizeCalloutSlug, slugStatus, addCustomCallout, updateCustomCallout, removeCustomCallout } from '$lib/theme/customCallouts';
	import EmojiIconPicker from './EmojiIconPicker.svelte';
	import SlotIcon from './SlotIcon.svelte';
	import IconRef from './IconRef.svelte';
	import ColorField from './ColorField.svelte';

	let { embedded = false }: { embedded?: boolean } = $props();

	// One picker drives every icon button — target says where the chosen ref goes.
	type PickTarget = { kind: 'family'; family: string } | { kind: 'custom'; slug: string } | { kind: 'new' };
	let picking = $state<PickTarget | null>(null);

	const overrides = $derived($appSettings.iconOverrides ?? {});
	const customList = $derived($appSettings.customCallouts ?? []);
	const families = CALLOUT_FAMILIES.map((f) => ({ family: f, defaultIcon: calloutDefaultIcon(f) }));

	// Add-form state.
	let newName = $state('');
	let newTrigger = $state('');
	let newColor = $state('#7c3aed');
	let newIcon = $state('');           // ref: emoji char or "set:name", '' = default
	// Slug from the explicit Trigger, or derived from the Name if Trigger is left blank.
	const newSlug = $derived(sanitizeCalloutSlug(newTrigger || newName));
	const newStatus = $derived(newSlug ? slugStatus(newSlug) : 'empty');
	const canAdd = $derived(newStatus === 'ok' && newName.trim().length > 0);

	// Edit-an-existing-row state. editingSlug = the row's CURRENT slug being edited.
	let editingSlug = $state<string | null>(null);
	let editName = $state('');
	let editTrigger = $state('');
	const editSlug = $derived(sanitizeCalloutSlug(editTrigger || editName));
	// slugStatus excludes the row itself (so an unchanged trigger isn't "duplicate").
	const editStatus = $derived(editingSlug && editSlug ? slugStatus(editSlug, editingSlug) : 'empty');
	const canSaveEdit = $derived(editStatus === 'ok' && editName.trim().length > 0);
	const triggerChanged = $derived(!!editingSlug && editSlug !== editingSlug);

	function startEdit(c: { slug: string; name: string }) {
		editingSlug = c.slug;
		editName = c.name;
		editTrigger = c.slug;
	}
	function saveEdit() {
		if (!editingSlug || !canSaveEdit) return;
		// updateCustomCallout matches by the OLD slug and applies the patch (incl. a new slug).
		updateCustomCallout(editingSlug, { slug: editSlug, name: editName.trim() });
		editingSlug = null;
	}
	function cancelEdit() { editingSlug = null; }

	// styleSetter.labels.<slug> with an English fallback on a miss (mirrors StyleSetter's L()).
	function lbl(slug: string, fallback: string): string {
		const key = 'styleSetter.labels.' + slug;
		const v = $t(key);
		return !v || v === key ? fallback : v;
	}
	const familyLabel = (family: string) => lbl(family, family.charAt(0).toUpperCase() + family.slice(1));

	// Normalize the picker's `:lucide-heart:` shortcode (or raw emoji) to the stored ref.
	function toRef(insertion: string): string {
		const sc = insertion.match(/^:(lucide|phosphor|hi|feather)-([a-z0-9-]+):$/);
		return sc ? `${sc[1]}:${sc[2]}` : insertion;
	}
	function onPick(insertion: string) {
		const ref = toRef(insertion);
		if (!picking) return;
		if (picking.kind === 'family') setOverride('callout.' + picking.family, ref);
		else if (picking.kind === 'custom') updateCustomCallout(picking.slug, { icon: ref });
		else newIcon = ref;
		picking = null;
	}

	function add() {
		if (!canAdd) return;
		addCustomCallout({ slug: newSlug, name: newName.trim(), color: newColor, icon: newIcon });
		newName = ''; newTrigger = ''; newColor = '#7c3aed'; newIcon = '';
	}
</script>

<div class="cte" class:cte-embedded={embedded}>
	<!-- ── Built-in family icons (Phase A) ── -->
	<div class="cte-head">{lbl('callout_icons', 'Callout icons')}</div>
	<div class="cte-rows">
		{#each families as f (f.family)}
			<div class="cte-row">
				<button class="cte-icon" title={lbl('change_icon', 'Change icon')} onclick={() => (picking = { kind: 'family', family: f.family })}>
					<SlotIcon slot={'callout.' + f.family}><span class="cte-emoji">{f.defaultIcon}</span></SlotIcon>
				</button>
				<span class="cte-label">{familyLabel(f.family)}</span>
				{#if overrides['callout.' + f.family]}
					<button class="cte-reset" title={lbl('reset_icon', 'Reset icon')} onclick={() => setOverride('callout.' + f.family, null)}>↺</button>
				{/if}
			</div>
		{/each}
	</div>

	<!-- ── Custom callout types (Phase B) ── -->
	<div class="cte-head cte-head-2">{lbl('custom_callouts', 'Custom callouts')}</div>
	{#if customList.length}
		<div class="cte-rows">
			{#each customList as c (c.slug)}
				<div class="cte-row">
					<button class="cte-icon" title={lbl('change_icon', 'Change icon')} onclick={() => (picking = { kind: 'custom', slug: c.slug })}>
						<IconRef ref={c.icon} fallback="ℹ️" />
					</button>
					<!-- ColorField commits ONCE (saved-swatch click or native onchange) — no save/emit storm. -->
					<ColorField value={c.color} title={lbl('colour', 'Colour')} onChange={(hex) => updateCustomCallout(c.slug, { color: hex })} />
					{#if editingSlug === c.slug}
						<input class="cte-in" placeholder={lbl('name', 'Name')} bind:value={editName} dir="auto" />
						<input class="cte-in cte-in-trig" placeholder={lbl('trigger', 'Trigger')} bind:value={editTrigger} dir="auto" />
						<button class="cte-reset" disabled={!canSaveEdit} title={lbl('save', 'Save')} onclick={saveEdit}>✓</button>
						<button class="cte-reset" title={lbl('cancel', 'Cancel')} onclick={cancelEdit}>✗</button>
					{:else}
						<span class="cte-label">{c.name} <span class="cte-trig">[!{c.slug}]</span></span>
						<button class="cte-reset" title={lbl('edit', 'Edit')} onclick={() => startEdit(c)}>✎</button>
						<button class="cte-reset cte-del" title={lbl('remove', 'Remove')} onclick={() => removeCustomCallout(c.slug)}>✕</button>
					{/if}
				</div>
				{#if editingSlug === c.slug}
					{#if editStatus === 'builtin'}
						<div class="cte-warn">{lbl('callout_collision_builtin', "That's a built-in callout — recolour or re-icon it above instead.")}</div>
					{:else if editStatus === 'duplicate'}
						<div class="cte-warn">{lbl('callout_collision_dupe', 'You already have a custom callout with that trigger.')}</div>
					{:else if triggerChanged}
						<div class="cte-hint">{lbl('callout_trigger_edit_hint', "Changing the trigger won't restyle callouts you've already typed.")}</div>
					{/if}
				{/if}
			{/each}
		</div>
	{/if}

	<!-- Add form -->
	<div class="cte-add">
		<div class="cte-add-row">
			<input class="cte-in" placeholder={lbl('name', 'Name')} bind:value={newName} dir="auto" />
			<input class="cte-in cte-in-trig" placeholder={lbl('trigger', 'Trigger')} bind:value={newTrigger} dir="auto" />
		</div>
		<div class="cte-add-row">
			<ColorField value={newColor} title={lbl('colour', 'Colour')} onChange={(hex) => (newColor = hex)} />
			<button class="cte-icon" title={lbl('change_icon', 'Change icon')} onclick={() => (picking = { kind: 'new' })}>
				<IconRef ref={newIcon} fallback="ℹ️" />
			</button>
			<span class="cte-preview">{newSlug ? `[!${newSlug}]` : ''}</span>
			<button class="cte-addbtn" disabled={!canAdd} onclick={add}>{lbl('add', 'Add')}</button>
		</div>
		{#if newSlug && newStatus === 'builtin'}
			<div class="cte-warn">{lbl('callout_collision_builtin', "That's a built-in callout — recolour or re-icon it above instead.")}</div>
		{:else if newStatus === 'duplicate'}
			<div class="cte-warn">{lbl('callout_collision_dupe', 'You already have a custom callout with that trigger.')}</div>
		{/if}
	</div>
</div>

{#if picking}
	<EmojiIconPicker onClose={() => (picking = null)} {onPick} />
{/if}

<style>
	.cte { margin-top: 10px; }
	.cte-head { font-size: 11px; text-transform: uppercase; letter-spacing: 0.07em; color: var(--c-muted, var(--text-muted)); margin: 6px 4px 6px; }
	.cte-head-2 { margin-top: 16px; }
	.cte-rows { display: flex; flex-direction: column; gap: 4px; }
	.cte-row { display: flex; align-items: center; gap: 8px; }
	.cte-icon {
		width: 30px; height: 30px; flex: none;
		display: flex; align-items: center; justify-content: center;
		background: var(--background-primary, #fff); border: 1px solid var(--background-modifier-border, #ddd);
		border-radius: 6px; cursor: pointer; padding: 0;
	}
	.cte-icon:hover { border-color: var(--interactive-accent, #7c3aed); }
	.cte-icon :global(svg) { width: 18px; height: 18px; }
	.cte-emoji { font-size: 17px; line-height: 1; }
	.cte-label { flex: 1; min-width: 0; font-size: 13px; color: var(--c-text, var(--text-normal)); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.cte-trig { color: var(--c-muted, var(--text-muted)); font-size: 11px; font-family: var(--font-monospace-theme, monospace); }
	.cte-reset {
		width: 22px; height: 22px; flex: none;
		background: none; border: 1px solid transparent; border-radius: 6px;
		color: var(--c-muted, var(--text-muted)); cursor: pointer; font-size: 13px;
	}
	.cte-reset:hover { color: var(--c-text, var(--text-normal)); border-color: var(--background-modifier-border, #ddd); }
	.cte-del:hover { color: var(--text-error, #e06666); }
	.cte-add { margin-top: 10px; display: flex; flex-direction: column; gap: 6px; }
	.cte-add-row { display: flex; align-items: center; gap: 8px; }
	/* MIG-089 — bidi-correct caret/Home-End/double-click for Arabic (etc.): the same
	   `unicode-bidi: plaintext` the editor uses on .cm-line, paired with dir="auto" on
	   the inputs. text-align follows the resolved direction (not hard-coded left). */
	.cte-in { flex: 1; min-width: 0; padding: 5px 8px; font: inherit; font-size: 12px; border: 1px solid var(--background-modifier-border, #ddd); border-radius: 6px; background: var(--background-secondary, #f6f6f8); color: var(--c-text, var(--text-normal)); unicode-bidi: plaintext; text-align: start; }
	.cte-in-trig { flex: 0 0 110px; font-family: var(--font-monospace-theme, monospace); }
	.cte-preview { flex: 1; min-width: 0; font-size: 11px; font-family: var(--font-monospace-theme, monospace); color: var(--c-muted, var(--text-muted)); overflow: hidden; text-overflow: ellipsis; }
	.cte-addbtn { flex: none; font: inherit; font-size: 12px; padding: 5px 14px; border-radius: 6px; border: 1px solid var(--interactive-accent, #7c3aed); background: var(--interactive-accent, #7c3aed); color: #fff; cursor: pointer; }
	.cte-addbtn:disabled { opacity: .4; cursor: default; }
	.cte-warn { font-size: 11px; color: var(--text-error, #e06666); padding: 2px 2px; }
	.cte-hint { font-size: 11px; color: var(--c-muted, var(--text-muted)); padding: 2px 2px; }
</style>
