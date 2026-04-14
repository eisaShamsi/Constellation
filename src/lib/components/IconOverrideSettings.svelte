<script lang="ts">
	/**
	 * Settings sub-page: override any app icon with an emoji or vector icon.
	 * Launched from Settings → Core Plug-Ins → Emoji & Icon Library → Customize app icons.
	 */
	import { t } from '$lib/i18n';
	import { appSettings } from '$lib/libraries/store';
	import { ICON_SLOTS, setOverride, clearAllOverrides } from '$lib/theme/iconOverrides';
	import EmojiIconPicker from './EmojiIconPicker.svelte';

	let pickingSlot = $state<string | null>(null);
	let filter = $state('');

	const grouped = $derived.by(() => {
		const map = new Map<string, typeof ICON_SLOTS[number][]>();
		for (const s of ICON_SLOTS) {
			if (filter && !s.label.toLowerCase().includes(filter.toLowerCase()) && !s.slot.toLowerCase().includes(filter.toLowerCase())) continue;
			if (!map.has(s.group)) map.set(s.group, []);
			map.get(s.group)!.push(s);
		}
		return [...map.entries()];
	});

	const overrides = $derived($appSettings.iconOverrides ?? {});
	const count = $derived(Object.keys(overrides).length);

	function renderRef(ref: string | undefined): string {
		if (!ref) return '';
		if (!ref.includes(':')) return ref; // emoji
		return `<span class="ref-token">${ref}</span>`;
	}
</script>

<div class="ios-root">
	<header class="ios-header">
		<div>
			<h3>{$t('picker.customizeAppIcons') ?? 'Customize app icons'}</h3>
			<p class="ios-sub">{$t('picker.customizeDesc') ?? 'Replace any Constellation chrome icon with an emoji or icon from the library.'}</p>
		</div>
		<div class="ios-actions">
			<span class="ios-count">{count} {count === 1 ? 'override' : 'overrides'}</span>
			{#if count > 0}
				<button class="w-btn w-btn-sm" onclick={clearAllOverrides}>{$t('picker.resetAll') ?? 'Reset all'}</button>
			{/if}
		</div>
	</header>

	<input type="text" class="ios-search" bind:value={filter} placeholder={$t('picker.filterSlots') ?? 'Filter slots…'} />

	<div class="ios-groups">
		{#each grouped as [groupName, slots]}
			<section class="ios-group">
				<h4>{groupName}</h4>
				<div class="ios-grid">
					{#each slots as s}
						<div class="slot-row">
							<button class="slot-picker" onclick={() => pickingSlot = s.slot} title={s.slot}>
								{#if overrides[s.slot]}
									{#if !overrides[s.slot].includes(':')}
										<span class="slot-emoji">{overrides[s.slot]}</span>
									{:else}
										<span class="slot-id">{overrides[s.slot]}</span>
									{/if}
								{:else}
									<span class="slot-default">default</span>
								{/if}
							</button>
							<div class="slot-label">{s.label}</div>
							{#if overrides[s.slot]}
								<button class="slot-reset" onclick={() => setOverride(s.slot, null)} title={$t('picker.reset') ?? 'Reset'}>×</button>
							{/if}
						</div>
					{/each}
				</div>
			</section>
		{/each}
	</div>
</div>

{#if pickingSlot}
	<EmojiIconPicker
		onClose={() => pickingSlot = null}
		onPick={(insertion) => {
			// The picker's onPick delivers the RENDERED string (emoji or wrapped SVG).
			// For override storage we want the REF (emoji char or "set:name" id).
			// We extract the ref from the data-icon attribute of the wrapped SVG;
			// raw emoji is stored as-is.
			const dataMatch = insertion.match(/data-icon="([^"]+)"/);
			const ref = dataMatch ? dataMatch[1] : insertion;
			if (pickingSlot) setOverride(pickingSlot, ref);
			pickingSlot = null;
		}}
	/>
{/if}

<style>
	.ios-root { padding: 8px 4px; }
	.ios-header { display: flex; justify-content: space-between; align-items: flex-start; gap: 12px; margin-bottom: 10px; }
	.ios-header h3 { margin: 0; font-size: 14px; font-weight: 600; }
	.ios-sub { margin: 4px 0 0; font-size: 12px; color: var(--text-muted); max-width: 42em; }
	.ios-actions { display: flex; align-items: center; gap: 8px; }
	.ios-count { font-size: 11px; color: var(--text-muted); }
	.ios-search {
		width: 100%; padding: 6px 10px; margin-bottom: 14px;
		border: 1px solid var(--background-modifier-border); border-radius: 6px;
		background: var(--background-secondary); color: var(--text-normal); font-family: inherit; font-size: 12px;
	}
	.ios-group { margin-bottom: 18px; }
	.ios-group h4 {
		font-size: 11px; text-transform: uppercase; letter-spacing: 0.05em;
		color: var(--text-muted); margin: 0 0 6px;
	}
	.ios-grid {
		display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
		gap: 6px;
	}
	.slot-row {
		display: flex; align-items: center; gap: 6px;
		padding: 4px 6px; border-radius: 6px;
		background: var(--background-secondary);
		border: 1px solid var(--background-modifier-border);
	}
	.slot-picker {
		width: 36px; height: 36px; flex-shrink: 0;
		display: flex; align-items: center; justify-content: center;
		background: var(--background-primary); border: 1px solid var(--background-modifier-border);
		border-radius: 6px; cursor: pointer; padding: 0;
	}
	.slot-picker:hover { border-color: var(--interactive-accent); }
	.slot-emoji { font-size: 22px; line-height: 1; }
	.slot-id { font-size: 9px; color: var(--text-muted); font-family: var(--font-monospace-theme, monospace); padding: 0 3px; overflow: hidden; text-overflow: ellipsis; }
	.slot-default { font-size: 10px; color: var(--text-faint); }
	.slot-label {
		flex: 1; min-width: 0;
		font-size: 12px; color: var(--text-normal);
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.slot-reset {
		width: 20px; height: 20px; flex-shrink: 0;
		background: none; border: none; cursor: pointer;
		color: var(--text-muted); border-radius: 4px; font-size: 14px;
	}
	.slot-reset:hover { color: var(--text-error, #e06666); background: var(--background-modifier-hover); }
</style>
