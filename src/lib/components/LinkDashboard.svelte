<script lang="ts">
	import { t } from '$lib/i18n';
	import { appSettings, linkLifecycle, effectiveLinkWeight, type NoteLink } from '$lib/libraries/store';

	// Share the user-configurable pill shape (radius / height / font-weight)
	// with BacklinksPanel / OutgoingLinksPanel so the ×N chip in the
	// Most-Traveled tab matches the chips the user sees in the sidebar
	// and inside rendered prose. One settings surface, every chip respects it.
	const pillShape = $derived($appSettings.linkPills?.shape ?? { radius: 10, height: 20, fontWeight: 700 });

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

	// Stale links (P5): previously traversed paths that haven't been
	// touched in LINK_STALE_DAYS. Surface them so the user can prune,
	// revisit, or consciously retire the connection. Sort by staleness
	// (oldest-touched first) so the most-forgotten climb to the top.
	const staleLinks = $derived.by(() => {
		if (!visible) return [];
		const now = Date.now();
		return allLinks
			.filter(l => linkLifecycle(l, now) === 'stale')
			.slice()
			.sort((a, b) => {
				// Older last_traversed → higher priority. Empty strings
				// shouldn't reach here (fresh links skip the stale branch)
				// but sort them last defensively.
				const aLt = Date.parse(a.last_traversed ?? '') || 0;
				const bLt = Date.parse(b.last_traversed ?? '') || 0;
				return aLt - bLt;
			})
			.slice(0, 50)
			.map(l => ({
				source_path: l.source_path,
				source_name: l.source_name,
				target: l.target,
				library_name: l.library_name,
				count: l.traversal_count ?? 0,
				last: l.last_traversed ?? '',
			}));
	});

	// Most-traveled paths (P4.3 + P5 slice 2): top 20 links by decayed
	// Living Link weight. Raw `traversal_count` is kept on the row for
	// display (the ×N chip) but the sort key is `effectiveLinkWeight()`
	// — so a link that was hot six months ago sinks beneath one that
	// was merely warm last week. Pure view concern; DB column untouched.
	const mostTraveled = $derived.by(() => {
		if (!visible) return [];
		const now = Date.now();
		const lifecycle = $appSettings.linkLifecycle;
		const halfLife = lifecycle?.halfLifeDays ?? 60;
		const decayOn = lifecycle?.decayEnabled ?? true;
		return allLinks
			.filter(l => (l.traversal_count ?? 0) > 0)
			.slice()
			.sort((a, b) =>
				effectiveLinkWeight(b, now, halfLife, decayOn) -
				effectiveLinkWeight(a, now, halfLife, decayOn)
			)
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

	let activeSection = $state<'cross' | 'broken' | 'orphan' | 'top' | 'traveled' | 'stale'>('top');

	/** Compact "2 weeks ago" style formatter. Same pattern elsewhere in
	 *  the app uses Intl.RelativeTimeFormat via $locale; reuse that here
	 *  so Arabic / RTL users get localized relative strings for free. */
	function relAge(iso: string): string {
		if (!iso) return '';
		const ms = Date.now() - Date.parse(iso);
		if (Number.isNaN(ms)) return '';
		const days = Math.floor(ms / 86_400_000);
		if (days < 30) return $t('linkDashboard.staleDays', { n: String(days) });
		const months = Math.floor(days / 30);
		if (months < 12) return $t('linkDashboard.staleMonths', { n: String(months) });
		const years = Math.floor(days / 365);
		return $t('linkDashboard.staleYears', { n: String(years) });
	}
</script>

<div class="link-dashboard" style="--pill-radius:{pillShape.radius}px;--pill-height:{pillShape.height}px;--pill-weight:{pillShape.fontWeight}">
	<div class="ld-tabs">
		<button class="ld-tab" class:active={activeSection === 'top'} onclick={() => activeSection = 'top'}>
			{$t('linkDashboard.mostConnected')} <span class="ld-badge">{mostConnected.length}</span>
		</button>
		<button class="ld-tab" class:active={activeSection === 'traveled'} onclick={() => activeSection = 'traveled'}>
			{$t('linkDashboard.mostTraveled')} <span class="ld-badge">{mostTraveled.length}</span>
		</button>
		<button class="ld-tab" class:active={activeSection === 'stale'} onclick={() => activeSection = 'stale'}>
			{$t('linkDashboard.stale')} <span class="ld-badge">{staleLinks.length}</span>
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
		{:else if activeSection === 'stale'}
			{#each staleLinks as link}
				<button class="ld-item" onclick={() => onNoteClick(link.source_path, link.library_name)}>
					<span class="ld-name">{link.source_name}</span>
					<span class="ld-detail">→ {link.target}</span>
					<span class="ld-chip ld-chip-stale" title={link.last}>{relAge(link.last)}</span>
				</button>
			{/each}
			{#if staleLinks.length === 0}
				<div class="ld-empty">{$t('linkDashboard.noStale')}</div>
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
		display: inline-flex; align-items: center; gap: 0;
		background: none; border: 1px solid var(--background-modifier-border);
		border-radius: var(--pill-radius, 10px); padding: 2px 6px 2px 10px;
		cursor: pointer;
		font-size: 0.72rem; font-family: inherit; color: var(--text-muted);
	}
	.ld-tab:hover { color: var(--text-normal); }
	.ld-tab.active {
		background: var(--interactive-accent);
		color: white; border-color: var(--interactive-accent);
	}
	.ld-badge {
		display: inline-flex; align-items: center; justify-content: center;
		box-sizing: border-box;
		height: var(--pill-height, 20px);
		padding: 0 8px;
		border-radius: var(--pill-radius, 10px);
		background: var(--background-modifier-border-focus);
		color: #fff;
		font-size: 0.68rem; font-weight: var(--pill-weight, 700);
		line-height: 1;
		margin-inline-start: 4px;
	}
	.ld-tab.active .ld-badge { background: rgba(255,255,255,0.2); color: #fff; }
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
		display: inline-flex; align-items: center; justify-content: center;
		box-sizing: border-box;
		height: var(--pill-height, 20px);
		padding: 0 8px;
		border-radius: var(--pill-radius, 10px);
		background: color-mix(in srgb, var(--interactive-accent) 14%, transparent);
		border: 1px solid color-mix(in srgb, var(--interactive-accent) 30%, transparent);
		color: var(--interactive-accent);
		font-size: 0.72rem; font-weight: var(--pill-weight, 700);
		line-height: 1;
	}
	/* Stale-link chip: muted amber so it reads as "needs attention"
	   without screaming "error" — distinct from the accent chips that
	   signal live traversal activity. */
	.ld-chip-stale {
		background: color-mix(in srgb, #d97706 14%, transparent);
		border-color: color-mix(in srgb, #d97706 30%, transparent);
		color: #d97706;
	}
	.ld-empty { color: var(--color-base-40); font-size: 0.78rem; padding: 8px 0; text-align: center; }
</style>
