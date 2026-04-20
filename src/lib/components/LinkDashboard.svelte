<script lang="ts">
	import { t } from '$lib/i18n';
	import type { NoteLink } from '$lib/libraries/store';

	let {
		allLinks = [] as NoteLink[],
		allNotes = [] as { name: string; path: string; libraryName: string }[],
		onNoteClick,
		visible = false,
	}: {
		allLinks: NoteLink[];
		allNotes: { name: string; path: string; libraryName: string }[];
		onNoteClick: (path: string, libraryName: string) => void;
		visible?: boolean;
	} = $props();

	// Pre-build lookup maps once — O(n) instead of O(n²)
	const notesByName = $derived.by(() => {
		if (!visible) return new Map<string, { name: string; path: string; libraryName: string }>();
		const map = new Map<string, { name: string; path: string; libraryName: string }>();
		for (const n of allNotes) map.set(n.name.toLowerCase(), n);
		return map;
	});

	const notesByPath = $derived.by(() => {
		if (!visible) return new Map<string, { name: string; path: string; libraryName: string }>();
		const map = new Map<string, { name: string; path: string; libraryName: string }>();
		for (const n of allNotes) map.set(n.path, n);
		return map;
	});

	// Cross-library links — O(m) with map lookups
	const crossLibraryLinks = $derived.by(() => {
		if (!visible) return [];
		return allLinks.filter(l => {
			const sourceLib = notesByPath.get(l.source_path)?.libraryName;
			const targetNote = notesByName.get(l.target.toLowerCase());
			return sourceLib && targetNote && sourceLib !== targetNote.libraryName;
		});
	});

	// Broken links — O(m) with map lookups
	const brokenLinks = $derived.by(() => {
		if (!visible) return [];
		return allLinks.filter(l => !notesByName.has(l.target.toLowerCase()));
	});

	// Orphan notes — O(m + n) with map lookups
	const orphanNotes = $derived.by(() => {
		if (!visible) return [];
		const linked = new Set<string>();
		for (const l of allLinks) {
			linked.add(l.source_path);
			const target = notesByName.get(l.target.toLowerCase());
			if (target) linked.add(target.path);
		}
		return allNotes.filter(n => !linked.has(n.path));
	});

	// Most-traveled paths (P4.3): top 20 links by Living Link traversal_count.
	// Surfaces the user's worn edges — the connections they reach for most.
	// O(m log m) due to sort; m is link count, bounded by vault size.
	const mostTraveled = $derived.by(() => {
		if (!visible) return [];
		return allLinks
			.filter(l => (l.traversal_count ?? 0) > 0)
			.slice()
			.sort((a, b) => (b.traversal_count ?? 0) - (a.traversal_count ?? 0))
			.slice(0, 20)
			.map(l => ({
				source_path: l.source_path,
				source_name: l.source_name,
				target: l.target,
				library_name: l.library_name,
				count: l.traversal_count ?? 0,
			}));
	});

	// Most connected notes (top 10) — O(m + k log k) with map lookups
	const mostConnected = $derived.by(() => {
		if (!visible) return [];
		const counts = new Map<string, number>();
		for (const l of allLinks) {
			counts.set(l.source_path, (counts.get(l.source_path) || 0) + 1);
			const target = notesByName.get(l.target.toLowerCase());
			if (target) counts.set(target.path, (counts.get(target.path) || 0) + 1);
		}
		return [...counts.entries()]
			.sort((a, b) => b[1] - a[1])
			.slice(0, 10)
			.map(([path, count]) => {
				const note = notesByPath.get(path);
				return { name: note?.name || path.split(/[/\\]/).pop() || '', path, libraryName: note?.libraryName || '', count };
			});
	});

	let activeSection = $state<'cross' | 'broken' | 'orphan' | 'top' | 'traveled'>('top');
</script>

<div class="link-dashboard">
	<div class="ld-tabs">
		<button class="ld-tab" class:active={activeSection === 'top'} onclick={() => activeSection = 'top'}>
			{$t('linkDashboard.mostConnected')} <span class="ld-badge">{mostConnected.length}</span>
		</button>
		<button class="ld-tab" class:active={activeSection === 'traveled'} onclick={() => activeSection = 'traveled'}>
			{$t('linkDashboard.mostTraveled')} <span class="ld-badge">{mostTraveled.length}</span>
		</button>
		<button class="ld-tab" class:active={activeSection === 'cross'} onclick={() => activeSection = 'cross'}>
			{$t('linkDashboard.crossLibrary')} <span class="ld-badge">{crossLibraryLinks.length}</span>
		</button>
		<button class="ld-tab" class:active={activeSection === 'broken'} onclick={() => activeSection = 'broken'}>
			{$t('linkDashboard.broken')} <span class="ld-badge">{brokenLinks.length}</span>
		</button>
		<button class="ld-tab" class:active={activeSection === 'orphan'} onclick={() => activeSection = 'orphan'}>
			{$t('linkDashboard.orphans')} <span class="ld-badge">{orphanNotes.length}</span>
		</button>
	</div>

	<div class="ld-content">
		{#if activeSection === 'top'}
			{#each mostConnected as item}
				<button class="ld-item" onclick={() => onNoteClick(item.path, item.libraryName)}>
					<span class="ld-name">{item.name}</span>
					<span class="ld-detail">{item.count} links</span>
				</button>
			{/each}
		{:else if activeSection === 'traveled'}
			{#each mostTraveled as link}
				<button class="ld-item" onclick={() => onNoteClick(link.source_path, link.library_name)}>
					<span class="ld-name">{link.source_name}</span>
					<span class="ld-detail">→ {link.target}</span>
					<span class="ld-chip">×{link.count}</span>
				</button>
			{/each}
			{#if mostTraveled.length === 0}
				<div class="ld-empty">{$t('linkDashboard.noTraveled')}</div>
			{/if}
		{:else if activeSection === 'cross'}
			{#each crossLibraryLinks.slice(0, 50) as link}
				<button class="ld-item" onclick={() => onNoteClick(link.source_path, link.library_name)}>
					<span class="ld-name">{link.source_name}</span>
					<span class="ld-detail">→ {link.target}</span>
				</button>
			{/each}
			{#if crossLibraryLinks.length === 0}
				<div class="ld-empty">{$t('linkDashboard.noCrossLibrary')}</div>
			{/if}
		{:else if activeSection === 'broken'}
			{#each brokenLinks.slice(0, 50) as link}
				<button class="ld-item" onclick={() => onNoteClick(link.source_path, link.library_name)}>
					<span class="ld-name">{link.source_name}</span>
					<span class="ld-detail ld-broken">→ {link.target}</span>
				</button>
			{/each}
			{#if brokenLinks.length === 0}
				<div class="ld-empty">{$t('linkDashboard.noBroken')}</div>
			{/if}
		{:else if activeSection === 'orphan'}
			{#each orphanNotes.slice(0, 50) as note}
				<button class="ld-item" onclick={() => onNoteClick(note.path, note.libraryName)}>
					<span class="ld-name">{note.name}</span>
					<span class="ld-detail">{note.libraryName}</span>
				</button>
			{/each}
			{#if orphanNotes.length === 0}
				<div class="ld-empty">{$t('linkDashboard.noOrphans')}</div>
			{/if}
		{/if}
	</div>
</div>

<style>
	.link-dashboard { font-size: 0.8rem; }
	.ld-tabs {
		display: flex; flex-wrap: wrap; gap: 2px; margin-bottom: 8px;
	}
	.ld-tab {
		background: none; border: 1px solid var(--background-modifier-border);
		border-radius: 4px; padding: 3px 8px; cursor: pointer;
		font-size: 0.72rem; font-family: inherit; color: var(--text-muted);
	}
	.ld-tab:hover { color: var(--text-normal); }
	.ld-tab.active {
		background: var(--interactive-accent);
		color: white; border-color: var(--interactive-accent);
	}
	.ld-badge {
		background: var(--background-modifier-border-focus);
		border-radius: 8px; padding: 0 4px; font-size: 0.68rem;
		margin-inline-start: 2px;
	}
	.ld-tab.active .ld-badge { background: rgba(255,255,255,0.2); color: white; }
	.ld-content { max-height: 300px; overflow-y: auto; }
	.ld-item {
		display: flex; width: 100%; padding: 4px 8px; gap: 6px;
		background: none; border: none; cursor: pointer; text-align: start;
		border-radius: 3px; font-family: inherit; align-items: center;
	}
	.ld-item:hover { background: var(--background-modifier-hover); }
	.ld-name { color: var(--interactive-accent); font-size: 0.8rem; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.ld-detail { color: var(--text-faint); font-size: 0.72rem; flex-shrink: 0; }
	.ld-broken { color: var(--text-error, #ef4444); }
	.ld-chip {
		flex-shrink: 0;
		display: inline-flex; align-items: center;
		height: 15px; padding: 0 5px; box-sizing: border-box;
		border-radius: 7px;
		background: color-mix(in srgb, var(--interactive-accent) 14%, transparent);
		border: 1px solid color-mix(in srgb, var(--interactive-accent) 30%, transparent);
		color: var(--interactive-accent);
		font-size: 0.6rem; font-weight: 600;
		line-height: 1;
	}
	.ld-empty { color: var(--color-base-40); font-size: 0.78rem; padding: 8px 0; text-align: center; }
</style>
