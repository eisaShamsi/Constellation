<script lang="ts">
	/**
	 * CalloutTypesEditor — the UNIFIED Callouts manager (MIG-089). One box, shown in
	 * the Style Setter's centre zone for the Callouts element: every callout (built-in
	 * AND custom) is a uniform row — [colour] [icon] [name · aliases / [!trigger]] —
	 * with a left border in its own colour (a live mini-preview). Built-ins first, a
	 * divider, then the user's custom types, then an Add row.
	 *
	 * Built-in COLOURS write the per-Universe Style Setter draft vars (--callout-
	 * <family>-color) via getDraftColor/setDraftColor passed by StyleSetter; built-in
	 * ICONS use the iconOverrides callout.<family> slots. Custom callouts store colour +
	 * icon in the per-Universe customCallouts registry. An open editor repaints live via
	 * NotePane's refreshCallouts hook. Reuses ColorField / EmojiIconPicker / SlotIcon /
	 * IconRef — no parallel systems.
	 */
	import { t } from '$lib/i18n';
	import { appSettings } from '$lib/libraries/store';
	import { setOverride } from '$lib/theme/iconOverrides';
	import { CALLOUT_FAMILIES, calloutDefaultIcon, CALLOUT_FAMILY_COLOR, CALLOUT_FAMILY_ALIASES } from '$lib/editor/calloutFamilies';
	import { sanitizeCalloutSlug, slugStatus, addCustomCallout, updateCustomCallout, removeCustomCallout } from '$lib/theme/customCallouts';
	import EmojiIconPicker from './EmojiIconPicker.svelte';
	import SlotIcon from './SlotIcon.svelte';
	import IconRef from './IconRef.svelte';
	import ColorField from './ColorField.svelte';

	let {
		getDraftColor = (_: string) => '',
		setDraftColor = (_: string, __: string) => {},
		resetColours = () => {},
		coloursOverridden = false,
	}: {
		getDraftColor?: (cssVar: string) => string;
		setDraftColor?: (cssVar: string, hex: string) => void;
		resetColours?: () => void;       // clears the 10 --callout-<family>-color draft vars
		coloursOverridden?: boolean;     // any of those colour vars currently set
	} = $props();

	// One picker drives every icon button — target says where the chosen ref goes.
	type PickTarget = { kind: 'family'; family: string } | { kind: 'custom'; slug: string } | { kind: 'new' };
	let picking = $state<PickTarget | null>(null);

	const overrides = $derived($appSettings.iconOverrides ?? {});
	const customList = $derived($appSettings.customCallouts ?? []);

	// "Reset built-ins" reverts the 10 built-in families' COLOURS + ICONS to default. It's
	// active when any built-in colour or icon is overridden. Custom callouts are NOT touched
	// (they have their own ✕ remove — deleting them would be data loss).
	const iconOverridden = $derived(CALLOUT_FAMILIES.some((f) => overrides['callout.' + f]));
	const builtinsOverridden = $derived(coloursOverridden || iconOverridden);
	function resetBuiltins() {
		resetColours();
		for (const f of CALLOUT_FAMILIES) setOverride('callout.' + f, null);
	}
	const families = CALLOUT_FAMILIES.map((f) => ({ family: f, defaultIcon: calloutDefaultIcon(f), aliases: CALLOUT_FAMILY_ALIASES[f] ?? [] }));

	// A built-in family's current colour = its draft override, else its §3a default hex.
	const builtinColor = (family: string) => getDraftColor('--callout-' + family + '-color') || CALLOUT_FAMILY_COLOR[family] || '#448aff';

	// Add-form state.
	let newName = $state('');
	let newTrigger = $state('');
	let newColor = $state('#7c3aed');
	let newIcon = $state('');           // ref: emoji char or "set:name", '' = default
	const newSlug = $derived(sanitizeCalloutSlug(newTrigger || newName));
	const newStatus = $derived(newSlug ? slugStatus(newSlug) : 'empty');
	const canAdd = $derived(newStatus === 'ok' && newName.trim().length > 0);

	// Edit-an-existing-row state.
	let editingSlug = $state<string | null>(null);
	let editName = $state('');
	let editTrigger = $state('');
	const editSlug = $derived(sanitizeCalloutSlug(editTrigger || editName));
	const editStatus = $derived(editingSlug && editSlug ? slugStatus(editSlug, editingSlug) : 'empty');
	const canSaveEdit = $derived(editStatus === 'ok' && editName.trim().length > 0);
	const triggerChanged = $derived(!!editingSlug && editSlug !== editingSlug);

	function startEdit(c: { slug: string; name: string }) { editingSlug = c.slug; editName = c.name; editTrigger = c.slug; }
	function saveEdit() {
		if (!editingSlug || !canSaveEdit) return;
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

<div class="cte">
	<div class="cte-titlerow">
		<div class="cte-title">{lbl('callouts', 'Callouts')}</div>
		<button class="cte-resetall" disabled={!builtinsOverridden} title={lbl('reset_this_element', 'Reset this element')} onclick={resetBuiltins}>↺ {lbl('reset_this_element', 'Reset this element')}</button>
	</div>

	<!-- ── Built-in families ── -->
	<div class="cte-rows">
		{#each families as f (f.family)}
			<div class="cte-row" style="border-inline-start:3px solid {builtinColor(f.family)}">
				<ColorField value={builtinColor(f.family)} title={lbl('colour', 'Colour')} onChange={(hex) => setDraftColor('--callout-' + f.family + '-color', hex)} />
				<button class="cte-icon" title={lbl('change_icon', 'Change icon')} onclick={() => (picking = { kind: 'family', family: f.family })}>
					<SlotIcon slot={'callout.' + f.family}><span class="cte-emoji">{f.defaultIcon}</span></SlotIcon>
				</button>
				<div class="cte-meta">
					<span class="cte-name">{familyLabel(f.family)}</span>
					{#if f.aliases.length}<span class="cte-trig">{f.aliases.join(' · ')}</span>{/if}
				</div>
				{#if overrides['callout.' + f.family]}
					<button class="cte-reset" title={lbl('reset_icon', 'Reset icon')} onclick={() => setOverride('callout.' + f.family, null)}>↺</button>
				{/if}
			</div>
		{/each}
	</div>

	<!-- ── Divider + custom callouts ── -->
	<div class="cte-divider"><span>{lbl('custom_callouts', 'Custom callouts')}</span></div>

	<div class="cte-rows">
		{#each customList as c (c.slug)}
			<div class="cte-row" style="border-inline-start:3px solid {c.color}">
				<ColorField value={c.color} title={lbl('colour', 'Colour')} onChange={(hex) => updateCustomCallout(c.slug, { color: hex })} />
				<button class="cte-icon" title={lbl('change_icon', 'Change icon')} onclick={() => (picking = { kind: 'custom', slug: c.slug })}>
					<IconRef ref={c.icon} fallback="ℹ️" />
				</button>
				{#if editingSlug === c.slug}
					<div class="cte-meta">
						<input class="cte-in" placeholder={lbl('name', 'Name')} bind:value={editName} dir="auto" />
						<input class="cte-in cte-in-trig" placeholder={lbl('trigger', 'Trigger')} bind:value={editTrigger} dir="auto" />
					</div>
					<button class="cte-reset" disabled={!canSaveEdit} title={lbl('save', 'Save')} onclick={saveEdit}>✓</button>
					<button class="cte-reset" title={lbl('cancel', 'Cancel')} onclick={cancelEdit}>✗</button>
				{:else}
					<div class="cte-meta">
						<span class="cte-name" dir="auto">{c.name}</span>
						<span class="cte-trig" dir="auto">[!{c.slug}]</span>
					</div>
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

		<!-- Add row -->
		<div class="cte-row cte-addrow" style="border-inline-start:3px solid {newColor}">
			<ColorField value={newColor} title={lbl('colour', 'Colour')} onChange={(hex) => (newColor = hex)} />
			<button class="cte-icon" title={lbl('change_icon', 'Change icon')} onclick={() => (picking = { kind: 'new' })}>
				<IconRef ref={newIcon} fallback="ℹ️" />
			</button>
			<div class="cte-meta">
				<input class="cte-in" placeholder={lbl('name', 'Name')} bind:value={newName} dir="auto" />
				<input class="cte-in cte-in-trig" placeholder={lbl('trigger', 'Trigger')} bind:value={newTrigger} dir="auto" />
			</div>
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
	.cte { display: flex; flex-direction: column; gap: 4px; width: 100%; }
	.cte-titlerow { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin: 0 2px 10px; }
	.cte-title { font-size: 18px; font-weight: 700; color: var(--c-text, var(--text-normal)); }
	.cte-resetall { flex: none; font: inherit; font-size: 12px; padding: 4px 11px; border-radius: 6px; border: 1px solid var(--c-border, var(--background-modifier-border, #ddd)); background: var(--background-secondary, #f6f6f8); color: var(--c-muted, var(--text-muted)); cursor: pointer; white-space: nowrap; }
	.cte-resetall:hover:not(:disabled) { color: var(--c-text, var(--text-normal)); border-color: var(--interactive-accent, #7c3aed); }
	.cte-resetall:disabled { opacity: .4; cursor: default; }
	.cte-rows { display: flex; flex-direction: column; gap: 5px; }
	.cte-row { display: flex; align-items: center; gap: 9px; padding: 5px 8px; border-radius: 7px; background: var(--background-secondary, #f6f6f8); }
	.cte-addrow { background: none; }
	.cte-icon {
		width: 30px; height: 30px; flex: none;
		display: flex; align-items: center; justify-content: center;
		background: var(--background-primary, #fff); border: 1px solid var(--background-modifier-border, #ddd);
		border-radius: 6px; cursor: pointer; padding: 0;
	}
	.cte-icon:hover { border-color: var(--interactive-accent, #7c3aed); }
	.cte-icon :global(svg) { width: 18px; height: 18px; }
	.cte-emoji { font-size: 17px; line-height: 1; }
	/* name + aliases/trigger stack so neither truncates; in edit/add the inputs stack full-width. */
	.cte-meta { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 3px; }
	.cte-name { font-size: 14px; font-weight: 600; color: var(--c-text, var(--text-normal)); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.cte-trig { color: var(--c-muted, var(--text-muted)); font-size: 11.5px; font-family: var(--font-monospace-theme, monospace); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.cte-meta .cte-in { flex: none; width: 100%; }
	.cte-in { min-width: 0; padding: 5px 8px; font: inherit; font-size: 12px; border: 1px solid var(--background-modifier-border, #ddd); border-radius: 6px; background: var(--background-primary, #fff); color: var(--c-text, var(--text-normal)); unicode-bidi: plaintext; text-align: start; }
	.cte-in-trig { font-family: var(--font-monospace-theme, monospace); }
	.cte-reset {
		width: 24px; height: 24px; flex: none;
		background: none; border: 1px solid transparent; border-radius: 6px;
		color: var(--c-muted, var(--text-muted)); cursor: pointer; font-size: 13px;
	}
	.cte-reset:hover:not(:disabled) { color: var(--c-text, var(--text-normal)); border-color: var(--background-modifier-border, #ddd); }
	.cte-reset:disabled { opacity: .4; cursor: default; }
	.cte-del:hover { color: var(--text-error, #e06666); }
	.cte-addbtn { flex: none; align-self: stretch; font: inherit; font-size: 12px; padding: 0 16px; border-radius: 6px; border: 1px solid var(--interactive-accent, #7c3aed); background: var(--interactive-accent, #7c3aed); color: #fff; cursor: pointer; }
	.cte-addbtn:disabled { opacity: .4; cursor: default; }
	.cte-divider { display: flex; align-items: center; gap: 10px; margin: 14px 2px 8px; font-size: 11px; text-transform: uppercase; letter-spacing: .07em; color: var(--c-muted, var(--text-muted)); }
	.cte-divider::before, .cte-divider::after { content: ''; flex: 1; height: 1px; background: var(--c-border, var(--background-modifier-border, #ddd)); }
	.cte-warn { font-size: 11px; color: var(--text-error, #e06666); padding: 2px 2px; }
	.cte-hint { font-size: 11px; color: var(--c-muted, var(--text-muted)); padding: 2px 2px; }
</style>
