<script lang="ts">
	/**
	 * ColorField — a colour picker that surfaces the user's SAVED colours
	 * (appSettings.styleSwatches), so custom-callout colours can reuse the same
	 * palette the rest of the Style Setter uses. Click the swatch → a popover with
	 * the saved colours (click to apply) + a native picker (picking a new colour
	 * also saves it to the palette). MIG-089 §B fix.
	 */
	import { appSettings, addStyleSwatch } from '$lib/libraries/store';
	import { t } from '$lib/i18n';

	let { value, onChange, title = '' }: { value: string; onChange: (hex: string) => void; title?: string } = $props();

	let open = $state(false);
	let btnEl = $state<HTMLButtonElement | null>(null);
	let popStyle = $state('');   // fixed top/left, computed from the button rect
	const swatches = $derived($appSettings.styleSwatches ?? []);

	// The popover is position:fixed (the Setter's right rail is overflow:auto, which would
	// clip an absolutely-positioned child). Anchor it to the button, clamped to the viewport.
	function toggle() {
		if (!open && btnEl) {
			const r = btnEl.getBoundingClientRect();
			const W = 184, H = 168;
			const left = Math.max(8, Math.min(r.left, window.innerWidth - W - 8));
			const top = r.bottom + 4 + H > window.innerHeight ? Math.max(8, r.top - H - 4) : r.bottom + 4;
			popStyle = `top:${top}px; left:${left}px;`;
		}
		open = !open;
	}

	function applySaved(hex: string) { onChange(hex); open = false; }
	function nativePick(e: Event) {
		const hex = (e.currentTarget as HTMLInputElement).value;
		onChange(hex);
		addStyleSwatch(hex);   // a freshly-picked colour joins the saved palette
	}
	function lbl(slug: string, fallback: string): string {
		const key = 'styleSetter.labels.' + slug;
		const v = $t(key);
		return !v || v === key ? fallback : v;
	}
</script>

<div class="cf">
	<button class="cf-swatch" bind:this={btnEl} style="background:{value}" title={title || lbl('colour', 'Colour')} aria-label={title || lbl('colour', 'Colour')} onclick={toggle}></button>
	{#if open}
		<!-- transparent backdrop catches an outside click to close (no global listener). -->
		<button class="cf-backdrop" aria-label="close" onclick={() => (open = false)}></button>
		<div class="cf-pop" style={popStyle}>
			{#if swatches.length}
				<div class="cf-saved-label">{lbl('saved_colours', 'Saved colours')}</div>
				<div class="cf-saved">
					{#each swatches as sw (sw.hex)}
						<button class="cf-chip" style="background:{sw.hex}" title={sw.name || sw.hex} aria-label={sw.name || sw.hex} onclick={() => applySaved(sw.hex)}></button>
					{/each}
				</div>
			{/if}
			<label class="cf-native">
				<span>{lbl('custom_colour', 'Custom…')}</span>
				<input type="color" value={value} onchange={nativePick} />
			</label>
		</div>
	{/if}
</div>

<style>
	.cf { position: relative; flex: none; }
	.cf-swatch { width: 30px; height: 30px; padding: 0; border: 1px solid var(--background-modifier-border, #ddd); border-radius: 6px; cursor: pointer; }
	.cf-swatch:hover { border-color: var(--interactive-accent, #7c3aed); }
	.cf-backdrop { position: fixed; inset: 0; z-index: 40; background: none; border: none; cursor: default; }
	.cf-pop {
		position: fixed; z-index: 41;
		background: var(--background-primary, #fff); border: 1px solid var(--background-modifier-border, #ddd);
		border-radius: 8px; box-shadow: var(--popover-shadow, 0 6px 24px rgba(0,0,0,.18)); padding: 8px; width: 184px;
	}
	.cf-saved-label { font-size: 10px; text-transform: uppercase; letter-spacing: .06em; color: var(--text-muted); margin: 0 2px 5px; }
	.cf-saved { display: flex; flex-wrap: wrap; gap: 5px; margin-bottom: 8px; }
	.cf-chip { width: 22px; height: 22px; padding: 0; border: 1px solid var(--background-modifier-border, #ddd); border-radius: 5px; cursor: pointer; }
	.cf-chip:hover { transform: scale(1.08); border-color: var(--interactive-accent, #7c3aed); }
	.cf-native { display: flex; align-items: center; justify-content: space-between; gap: 8px; font-size: 12px; color: var(--text-normal); cursor: pointer; }
	.cf-native input { width: 30px; height: 24px; padding: 0; border: 1px solid var(--background-modifier-border, #ddd); border-radius: 5px; cursor: pointer; }
</style>
