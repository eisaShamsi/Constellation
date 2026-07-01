<script lang="ts">
	/**
	 * EmojiIconPicker — unified picker for emoji + vector icons.
	 *
	 * Triggered by Ctrl+. globally or programmatically via the
	 * `constellation:open-icon-picker` event. Inserts the chosen item at
	 * the current cursor position in the active editor (emoji as raw
	 * Unicode, Lucide icons as inline SVG).
	 *
	 * Data sources:
	 *   - Emoji: emojibase-data compact dataset, lazy-loaded per locale
	 *   - Icons: lucide-static (lazy-loaded SVG strings)
	 *
	 * v1 ships emoji + Lucide; future commits add Phosphor/Feather/Heroicons
	 * and an inline `:shortcode:` autocomplete driven by the same dataset.
	 */
	import { t, locale } from '$lib/i18n';
	import { onMount, onDestroy } from 'svelte';

	let {
		onPick,
		onClose,
	}: {
		onPick: (insertion: string) => void;
		onClose: () => void;
	} = $props();

	import { loadAllIcons, wrapForInsertion, type Icon, type IconSet } from '$lib/editor/iconSets';

	type Emoji = {
		unicode: string;
		label: string;
		hexcode: string;
		tags?: string[];
		group?: number;
	};

	const GROUPS: { id: number; name: string; icon: string }[] = [
		{ id: 0, name: 'Smileys', icon: '😀' },
		{ id: 1, name: 'People', icon: '👋' },
		{ id: 3, name: 'Nature', icon: '🌱' },
		{ id: 4, name: 'Food', icon: '🍎' },
		{ id: 5, name: 'Activities', icon: '⚽' },
		{ id: 6, name: 'Travel', icon: '✈️' },
		{ id: 7, name: 'Objects', icon: '💡' },
		{ id: 8, name: 'Symbols', icon: '❤️' },
		{ id: 9, name: 'Flags', icon: '🏳️' },
	];

	let tab = $state<'emoji' | 'icons' | 'recent'>('emoji');
	let group = $state<number>(0);
	let query = $state('');
	let iconSetFilter = $state<IconSet | 'all'>('all');
	let emojis = $state<Emoji[]>([]);
	let icons = $state<Icon[]>([]);
	let loading = $state(true);
	let recent = $state<string[]>([]);
	let searchEl: HTMLInputElement | null = null;

	const RECENT_KEY = 'constellation:picker-recent';
	const RECENT_MAX = 24;

	onMount(async () => {
		try {
			recent = JSON.parse(localStorage.getItem(RECENT_KEY) ?? '[]');
		} catch { recent = []; }
		await loadEmoji();
		loading = false;
		searchEl?.focus();
		window.addEventListener('keydown', handleKey);
	});

	onDestroy(() => {
		window.removeEventListener('keydown', handleKey);
	});

	async function loadEmoji() {
		// Pick the closest supported emojibase locale; English is always the fallback.
		const lang = ($locale ?? 'en').slice(0, 2);
		const SUPPORTED = ['bn','da','de','en','es','et','fi','fr','hi','hu','it','ja','ko','lt','ms','nb','nl','pl','pt','ru','sv','th','uk'];
		const pick = SUPPORTED.includes(lang) ? lang : 'en';
		try {
			const mod = await import(`emojibase-data/${pick}/compact.json`);
			emojis = (mod.default as Emoji[]).filter(e => typeof e.group === 'number');
		} catch {
			const fallback = await import('emojibase-data/en/compact.json');
			emojis = (fallback.default as Emoji[]).filter(e => typeof e.group === 'number');
		}
	}

	async function loadIcons() {
		if (icons.length > 0) return;
		loading = true;
		icons = await loadAllIcons();
		loading = false;
	}

	const filteredEmoji = $derived.by(() => {
		const q = query.trim().toLowerCase();
		if (q) {
			return emojis.filter(e =>
				e.label.toLowerCase().includes(q) ||
				e.tags?.some(t => t.toLowerCase().includes(q))
			).slice(0, 300);
		}
		return emojis.filter(e => e.group === group).slice(0, 300);
	});

	const filteredIcons = $derived.by(() => {
		const q = query.trim().toLowerCase();
		let base = iconSetFilter === 'all' ? icons : icons.filter(i => i.set === iconSetFilter);
		if (q) base = base.filter(i => i.name.includes(q) || i.id.includes(q));
		return base.slice(0, 500);
	});

	async function switchTab(t: typeof tab) {
		tab = t;
		if (t === 'icons' && icons.length === 0) await loadIcons();
	}

	function pickEmoji(e: Emoji) {
		pushRecent(e.unicode);
		onPick(e.unicode);
	}

	function pickIcon(i: Icon) {
		// Insert the shortcode form (`:lucide-heart:`) rather than raw SVG.
		// The editor's live-preview widget resolves the shortcode to an inline
		// <svg> at render time. Keeps the .md file small and editable and
		// matches how emoji work (raw character, decorated).
		const shortcode = `:${i.set}-${i.name}:`;
		pushRecent(shortcode);
		onPick(shortcode);
	}

	function pushRecent(s: string) {
		const next = [s, ...recent.filter(r => r !== s)].slice(0, RECENT_MAX);
		recent = next;
		try { localStorage.setItem(RECENT_KEY, JSON.stringify(next)); } catch {}
	}

	function handleKey(e: KeyboardEvent) {
		if (e.key === 'Escape') { e.preventDefault(); onClose(); }
	}
</script>

<div class="picker-backdrop" onclick={onClose} role="presentation">
	<div class="picker" onclick={(e) => e.stopPropagation()} role="dialog" aria-label="Emoji and icon picker">
		<div class="picker-tabs">
			<button class="tab" class:active={tab === 'emoji'} onclick={() => switchTab('emoji')}>
				{$t('picker.emoji') ?? 'Emoji'} 😀
			</button>
			<button class="tab" class:active={tab === 'icons'} onclick={() => switchTab('icons')}>
				{$t('picker.icons') ?? 'Icons'}
				<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
			</button>
			<button class="tab" class:active={tab === 'recent'} onclick={() => switchTab('recent')}>
				{$t('picker.recent') ?? 'Recent'} ⏱
			</button>
			<button class="picker-close" onclick={onClose} title={$t('common.close') ?? 'Close'}>×</button>
		</div>

		<input
			bind:this={searchEl}
			type="text"
			class="picker-search"
			placeholder={$t('picker.search') ?? 'Search — type in any language'}
			bind:value={query}
		/>

		{#if tab === 'emoji'}
			{#if !query}
				<div class="group-tabs">
					{#each GROUPS as g}
						<button class="group-tab" class:active={group === g.id} onclick={() => group = g.id} title={g.name}>
							{g.icon}
						</button>
					{/each}
				</div>
			{/if}
			{#if loading}
				<div class="picker-loading">Loading emoji…</div>
			{:else}
				<div class="picker-grid picker-grid-emoji">
					{#each filteredEmoji as e (e.hexcode)}
						<button class="emoji-cell" title={e.label} onclick={() => pickEmoji(e)}>
							{e.unicode}
						</button>
					{/each}
					{#if filteredEmoji.length === 0}
						<div class="picker-empty">{$t('picker.noResults') ?? 'No matches.'}</div>
					{/if}
				</div>
			{/if}
		{:else if tab === 'icons'}
			{#if loading}
				<div class="picker-loading">Loading icons…</div>
			{:else}
				<div class="icon-set-filters">
					<button class="set-btn" class:active={iconSetFilter === 'all'} onclick={() => iconSetFilter = 'all'}>All</button>
					<button class="set-btn" class:active={iconSetFilter === 'lucide'} onclick={() => iconSetFilter = 'lucide'}>Lucide</button>
					<button class="set-btn" class:active={iconSetFilter === 'phosphor'} onclick={() => iconSetFilter = 'phosphor'}>Phosphor</button>
					<button class="set-btn" class:active={iconSetFilter === 'heroicons'} onclick={() => iconSetFilter = 'heroicons'}>Heroicons</button>
					<button class="set-btn" class:active={iconSetFilter === 'feather'} onclick={() => iconSetFilter = 'feather'}>Feather</button>
				</div>
				<div class="picker-grid picker-grid-icons">
					{#each filteredIcons as i (i.id)}
						<button class="icon-cell" title={i.id} onclick={() => pickIcon(i)}>
							{@html i.svg}
						</button>
					{/each}
					{#if filteredIcons.length === 0}
						<div class="picker-empty">{$t('picker.noResults') ?? 'No matches.'}</div>
					{/if}
				</div>
			{/if}
		{:else}
			<div class="picker-grid picker-grid-emoji">
				{#each recent as r, i (i)}
					<button class="emoji-cell" onclick={() => onPick(r)}>
						{#if r.startsWith('<svg')}{@html r}{:else}{r}{/if}
					</button>
				{/each}
				{#if recent.length === 0}
					<div class="picker-empty">{$t('picker.recentEmpty') ?? 'Nothing used yet. Pick something from Emoji or Icons.'}</div>
				{/if}
			</div>
		{/if}
	</div>
</div>

<style>
	.picker-backdrop {
		position: fixed; inset: 0; background: rgba(0,0,0,0.35);
		display: flex; align-items: center; justify-content: center; z-index: 1000;
	}
	.picker {
		width: min(520px, 92vw); max-height: 72vh;
		background: var(--background-primary); color: var(--text-normal);
		border: 1px solid var(--background-modifier-border);
		border-radius: 12px; box-shadow: var(--modal-shadow, 0 24px 48px rgba(0,0,0,0.25));
		display: flex; flex-direction: column;
		font-size: 13px;
	}
	.picker-tabs {
		display: flex; gap: 4px; align-items: center;
		padding: 8px 8px 0 8px;
	}
	.tab {
		background: none; border: none; cursor: pointer;
		padding: 6px 10px; border-radius: 6px 6px 0 0;
		color: var(--text-muted); font-family: inherit; font-size: 12px;
		display: inline-flex; align-items: center; gap: 4px;
	}
	.tab:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.tab.active { background: var(--background-secondary); color: var(--text-normal); font-weight: 600; }
	.picker-close {
		margin-inline-start: auto; background: none; border: none; cursor: pointer;
		font-size: 22px; line-height: 1; color: var(--text-muted); padding: 0 8px;
	}
	.picker-close:hover { color: var(--text-normal); }
	.picker-search {
		margin: 8px; padding: 8px 10px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		background: var(--background-secondary); color: var(--text-normal);
		font-family: inherit;
	}
	.group-tabs {
		display: flex; gap: 2px; padding: 0 8px 6px;
		border-bottom: 1px solid var(--background-modifier-border);
	}
	.group-tab {
		background: none; border: none; cursor: pointer;
		padding: 4px 6px; border-radius: 4px; font-size: 18px;
	}
	.group-tab:hover { background: var(--background-modifier-hover); }
	.group-tab.active { background: var(--background-secondary); }
	.picker-grid {
		flex: 1; overflow-y: auto; padding: 8px;
		display: grid; gap: 2px;
	}
	.picker-grid-emoji { grid-template-columns: repeat(auto-fill, minmax(36px, 1fr)); }
	.picker-grid-icons { grid-template-columns: repeat(auto-fill, minmax(48px, 1fr)); }
	.emoji-cell, .icon-cell {
		background: none; border: none; cursor: pointer;
		display: flex; align-items: center; justify-content: center;
		aspect-ratio: 1; border-radius: 6px; font-size: 22px;
		color: var(--text-normal);
	}
	.emoji-cell:hover, .icon-cell:hover { background: var(--background-modifier-hover); }
	.icon-cell :global(svg) { width: 22px; height: 22px; }
	.picker-empty, .picker-loading {
		padding: 24px; color: var(--text-muted); text-align: center; font-size: 12px;
	}
	.icon-set-filters {
		display: flex; gap: 4px; padding: 0 8px 4px;
	}
	.set-btn {
		background: none; border: 1px solid var(--background-modifier-border);
		color: var(--text-muted); cursor: pointer; font-family: inherit;
		padding: 3px 8px; border-radius: 4px; font-size: 11px;
	}
	.set-btn:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.set-btn.active { background: var(--interactive-accent); color: var(--text-on-accent); border-color: var(--interactive-accent); }
</style>
