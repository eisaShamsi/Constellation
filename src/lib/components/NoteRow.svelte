<script lang="ts" module>
	/**
	 * MIG-090 §4 — the SHARED note-row primitive (the concept paper's §3
	 * complementarity contract: one row component, built for later adoption by
	 * the surfaces that hand-roll note lists today — PJ-069). v1 consumer: the
	 * Workbench. The row owns ONLY layout / selection / dir / a11y; hosts own
	 * data + actions. Fixed heights per density (exported here — the single
	 * source virtualizers pre-measure from, defusing the keep-in-sync trap
	 * BacklinksPanel documents).
	 */
	export const NOTE_ROW_HEIGHT = 52;
	export function heightFor(): number {
		return NOTE_ROW_HEIGHT;
	}
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';
	import { detectDir } from '$lib/utils';

	let {
		name,
		meta = '',
		chips = [],
		selected = false,
		missing = false,
		onActivate,
		trailing,
	}: {
		name: string;
		/** One muted line under the name (library · date …). */
		meta?: string;
		/** Small state pills (already-localized labels). */
		chips?: string[];
		selected?: boolean;
		missing?: boolean;
		onActivate: (e: MouseEvent) => void;
		/** Host-owned trailing actions (remove/other buttons). */
		trailing?: Snippet;
	} = $props();
</script>

<!-- MIG-092 — per-title row direction (Language-First): an RTL title mirrors the
     whole row (name at the reading start, trailing actions at the reading end). -->
<div class="nr" class:nr-selected={selected} class:nr-missing={missing} dir={detectDir(name)} style="height: {NOTE_ROW_HEIGHT}px">
	<button class="nr-main" onclick={onActivate} onauxclick={onActivate} dir="auto">
		<span class="nr-name" dir="auto">{name}</span>
		<span class="nr-meta" dir="auto">
			{#if chips.length > 0}
				{#each chips as c}<span class="nr-chip">{c}</span>{/each}
			{/if}
			{meta}
		</span>
	</button>
	{#if trailing}
		<div class="nr-trailing">{@render trailing()}</div>
	{/if}
</div>

<style>
	.nr {
		display: flex;
		align-items: stretch;
		border-bottom: 1px solid var(--background-modifier-border);
		background: transparent;
	}
	.nr:hover { background: var(--background-modifier-hover); }
	.nr-selected { background: var(--background-modifier-hover); }
	.nr-missing .nr-name { color: var(--text-muted); font-style: italic; }
	.nr-main {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		justify-content: center;
		gap: 2px;
		padding: 0 12px;
		background: none;
		border: none;
		cursor: pointer;
		text-align: start;
		font-family: var(--font-interface-theme);
	}
	.nr-name {
		font-size: 13px;
		font-weight: 500;
		color: var(--text-normal);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.nr-meta {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 11px;
		color: var(--text-muted);
		white-space: nowrap;
		overflow: hidden;
	}
	.nr-chip {
		padding: 0 6px;
		border-radius: 8px;
		font-size: 10px;
		line-height: 16px;
		background: color-mix(in srgb, var(--interactive-accent) 14%, transparent);
		color: var(--interactive-accent);
		flex-shrink: 0;
	}
	.nr-trailing {
		display: flex;
		align-items: center;
		gap: 4px;
		padding-inline-end: 10px;
		flex-shrink: 0;
	}
</style>
