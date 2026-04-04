<script lang="ts">
	import { t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import {
		libraries, libraryStats, loadAllStats,
		openNoteTab,
		type LibraryStats
	} from '$lib/libraries/store';
	import { getChildUniverses, type ChildUniverseInfo } from '$lib/universe/store';
	import { getRecentLists, type RecentOpenedNote, type RecentEditedNote } from '$lib/libraries/recentNotes';
	import { scanAllLibraryTags, type DashboardTag } from '$lib/libraries/tagUtils';
	import { get } from 'svelte/store';

	let {
		universeName = '',
		libraryColorMap = {} as Record<string, string>,
		onNoteClick,
		onNoteToMain,
		onNoteToScreen,
		onTagSelect,
	}: {
		universeName?: string;
		libraryColorMap?: Record<string, string>;
		onNoteClick?: (path: string, name: string, libraryName: string) => void;
		onNoteToMain?: (note: { path: string; name: string; libraryName: string; libraryPath: string; libraryColor: string }) => void;
		onNoteToScreen?: (note: { path: string; name: string; libraryName: string; libraryPath: string; libraryColor: string }) => void;
		onTagSelect?: (tag: string, notes: { name: string; path: string; libraryName: string }[]) => void;
	} = $props();

	// Dashboard state
	let childUniverses = $state<ChildUniverseInfo[]>([]);
	let childUniverseLibs = $state<Record<string, { name: string; path: string }[]>>({});
	let dashboardTags = $state<DashboardTag[]>([]);
	let selectedTag = $state<string | null>(null);
	let selectedTagNotes = $state<{ name: string; path: string; libraryName: string }[]>([]);
	let loadingTagNotes = $state(false);
	let loaded = $state(false);

	async function selectTag(tag: string) {
		if (selectedTag === tag) {
			selectedTag = null;
			selectedTagNotes = [];
			return;
		}
		selectedTag = tag;
		loadingTagNotes = true;
		const notes: { name: string; path: string; libraryName: string }[] = [];
		try {
			for (const lib of get(libraries)) {
				const results = await invoke<any[]>('notes_by_tag', { libraryPath: lib.path, tag });
				notes.push(...results.map((n: any) => ({ name: n.name, path: n.path, libraryName: n.library_name || lib.name })));
			}
			selectedTagNotes = notes.sort((a, b) => a.name.localeCompare(b.name));
		} catch {
			selectedTagNotes = [];
		}
		loadingTagNotes = false;
		// Emit to second screen if callback provided
		if (onTagSelect && selectedTagNotes.length > 0) {
			onTagSelect(tag, selectedTagNotes);
		}
	}

	let totalNotes = $derived($libraryStats.reduce((sum: number, s: any) => sum + s.star_count, 0));
	let totalFolders = $derived($libraryStats.reduce((sum: number, s: any) => sum + s.folder_count, 0));
	let cuLibNames = $derived.by(() => {
		const names = new Set<string>();
		for (const libs of Object.values(childUniverseLibs)) {
			for (const lib of libs) names.add(lib.name);
		}
		return names;
	});
	let topLevelStats = $derived($libraryStats.filter((s: any) => !cuLibNames.has(s.name) && !s.is_universe_notes));
	let universeNotesStats = $derived($libraryStats.find((s: any) => s.is_universe_notes) ?? null);

	// Recently opened/edited (shared utility)
	let recentlyEdited = $state<RecentEditedNote[]>([]);
	let recentlyOpened = $state<RecentOpenedNote[]>([]);

	function refreshRecentLists() {
		const lists = getRecentLists();
		recentlyEdited = lists.recentlyEdited;
		recentlyOpened = lists.recentlyOpened;
	}

	async function loadDashboardData() {
		try {
			await loadAllStats();
			try {
				childUniverses = await getChildUniverses();
				const libMap: Record<string, { name: string; path: string }[]> = {};
				for (const cu of childUniverses) {
					try {
						const libs = await invoke<{ name: string; path: string }[]>('read_child_universe_libraries', { childPath: cu.path });
						libMap[cu.path] = libs.map(l => ({ name: l.name, path: l.path }));
					} catch { libMap[cu.path] = []; }
				}
				childUniverseLibs = libMap;
			} catch { childUniverses = []; childUniverseLibs = {}; }
			try {
				dashboardTags = await scanAllLibraryTags();
			} catch { dashboardTags = []; }
		} catch {}
		loaded = true;
	}

	function handleNoteClick(path: string, name: string, libraryName: string) {
		if (onNoteToScreen) {
			const lib = get(libraries).find(l => l.name === libraryName);
			onNoteToScreen({ path, name, libraryName, libraryPath: lib?.path ?? '', libraryColor: libraryColorMap[libraryName] || '#7c3aed' });
		} else if (onNoteClick) {
			onNoteClick(path, name, libraryName);
		} else {
			openNoteTab(path, libraryName, libraryColorMap[libraryName] || '#7c3aed');
		}
	}

	onMount(() => {
		refreshRecentLists();
		loadDashboardData();
		// Poll localStorage for recent lists
		const interval = setInterval(refreshRecentLists, 5000);
		const handleStorage = (e: StorageEvent) => {
			if (e.key?.startsWith('constellation-recent-')) refreshRecentLists();
		};
		window.addEventListener('storage', handleStorage);
		return () => {
			clearInterval(interval);
			window.removeEventListener('storage', handleStorage);
		};
	});
</script>

<div class="dashboard-companion">
	<div class="dashboard-scroll">
		<div class="dashboard-header">
			<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" opacity="0.5">
				<circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
			</svg>
			<h2>{universeName || $t('secondScreen.dashboard.universe')}</h2>
		</div>

		<div class="dashboard-stats">
			{#if childUniverses.length > 0}
				<div class="stat-card">
					<span class="stat-value">{childUniverses.length}</span>
					<span class="stat-label">{$t('secondScreen.dashboard.childUniverses')}</span>
				</div>
			{/if}
			<div class="stat-card">
				<span class="stat-value">{$libraries.length}</span>
				<span class="stat-label">{$t('secondScreen.dashboard.libraries')}</span>
			</div>
			<div class="stat-card">
				<span class="stat-value">{totalFolders}</span>
				<span class="stat-label">{$t('secondScreen.dashboard.folders')}</span>
			</div>
			<div class="stat-card">
				<span class="stat-value">{totalNotes}</span>
				<span class="stat-label">{$t('secondScreen.dashboard.notes')}</span>
			</div>
		</div>

		{#if universeNotesStats}
			<div class="dashboard-section">
				<h3 class="dashboard-section-title">
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--interactive-accent)" stroke-width="1.5" style="flex-shrink: 0;">
						<circle cx="12" cy="12" r="6"/><line x1="6" y1="12" x2="18" y2="12"/>
						<path d="M9.5 6.5a8.5 8.5 0 010 11"/><path d="M14.5 6.5a8.5 8.5 0 000 11"/>
						<ellipse cx="12" cy="12" rx="11" ry="3.5" transform="rotate(-25 12 12)" stroke-dasharray="2,2"/>
					</svg>
					{universeNotesStats.name}
				</h3>
				<div class="library-card-stats" style="--lib-color: var(--interactive-accent)">
					<div class="lib-stat-box">
						<span class="lib-stat-value">{universeNotesStats.folder_count}</span>
						<span class="lib-stat-label">{$t('secondScreen.dashboard.folders')}</span>
					</div>
					<div class="lib-stat-box">
						<span class="lib-stat-value">{universeNotesStats.star_count}</span>
						<span class="lib-stat-label">{$t('secondScreen.dashboard.notes')}</span>
					</div>
				</div>
			</div>
		{/if}

		{#if childUniverses.length > 0}
			<div class="dashboard-section">
				<h3 class="dashboard-section-title">{$t('secondScreen.dashboard.childUniverses')}</h3>
				<div class="cu-list">
					{#each childUniverses as cu}
						{@const cuLibs = childUniverseLibs[cu.path] || []}
						{@const cuStats = cuLibs.map(l => $libraryStats.find(s => s.name === l.name)).filter(Boolean)}
						{@const cuFolders = cuStats.reduce((sum, s) => sum + (s?.folder_count ?? 0), 0)}
						{@const cuNotes = cuStats.reduce((sum, s) => sum + (s?.star_count ?? 0), 0)}
						<div class="cu-group">
							<div class="cu-header">
								<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#6366f1" stroke-width="1.5" style="flex-shrink: 0;">
									<circle cx="12" cy="12" r="6"/><line x1="6" y1="12" x2="18" y2="12"/>
									<path d="M9.5 6.5a8.5 8.5 0 010 11"/><path d="M14.5 6.5a8.5 8.5 0 000 11"/>
									<ellipse cx="12" cy="12" rx="11" ry="3.5" transform="rotate(-25 12 12)" stroke-dasharray="2,2"/>
								</svg>
								<span class="cu-name">{cu.name}</span>
							</div>
							<div class="cu-stat-boxes">
								<div class="cu-stat-box">
									<span class="cu-stat-value">{cu.library_count}</span>
									<span class="cu-stat-label">{$t('secondScreen.dashboard.libraries')}</span>
								</div>
								<div class="cu-stat-box">
									<span class="cu-stat-value">{cuFolders}</span>
									<span class="cu-stat-label">{$t('secondScreen.dashboard.folders')}</span>
								</div>
								<div class="cu-stat-box">
									<span class="cu-stat-value">{cuNotes}</span>
									<span class="cu-stat-label">{$t('secondScreen.dashboard.notes')}</span>
								</div>
							</div>
							{#if childUniverseLibs[cu.path]?.length}
								<div class="cu-libs">
									{#each childUniverseLibs[cu.path] as lib}
										{@const stats = $libraryStats.find(s => s.name === lib.name)}
										{@const color = libraryColorMap[lib.name] || '#7c3aed'}
										<div class="library-card">
											<div class="library-card-header">
												<span class="lib-dot" style="background:{color}"></span>
												<span class="lib-name">{lib.name}</span>
											</div>
											<div class="library-card-stats">
												<div class="lib-stat-box" style="--lib-color:{color}">
													<span class="lib-stat-value">{stats?.folder_count ?? 0}</span>
													<span class="lib-stat-label">{$t('secondScreen.dashboard.folders')}</span>
												</div>
												<div class="lib-stat-box" style="--lib-color:{color}">
													<span class="lib-stat-value">{stats?.star_count ?? 0}</span>
													<span class="lib-stat-label">{$t('secondScreen.dashboard.notes')}</span>
												</div>
											</div>
										</div>
									{/each}
								</div>
							{/if}
						</div>
					{/each}
				</div>
			</div>
		{/if}

		<div class="dashboard-section">
			<h3 class="dashboard-section-title">{$t('secondScreen.dashboard.libraryBreakdown')}</h3>
			<div class="library-list">
				{#each topLevelStats as lib}
					{@const color = libraryColorMap[lib.name] || '#7c3aed'}
					<div class="library-card">
						<div class="library-card-header">
							<span class="lib-dot" style="background:{color}"></span>
							<span class="lib-name">{lib.name}</span>
						</div>
						<div class="library-card-stats">
							<div class="lib-stat-box" style="--lib-color:{color}">
								<span class="lib-stat-value">{lib.folder_count}</span>
								<span class="lib-stat-label">{$t('secondScreen.dashboard.folders')}</span>
							</div>
							<div class="lib-stat-box" style="--lib-color:{color}">
								<span class="lib-stat-value">{lib.star_count}</span>
								<span class="lib-stat-label">{$t('secondScreen.dashboard.notes')}</span>
							</div>
						</div>
					</div>
				{/each}
			</div>
		</div>

		<div class="recent-columns">
			<div class="recent-column">
				<h3 class="dashboard-section-title">{$t('secondScreen.dashboard.recentlyEdited')}</h3>
				{#if recentlyEdited.length > 0}
					<div class="recent-list">
						{#each recentlyEdited as note}
							<button class="recent-note" onclick={() => handleNoteClick(note.path, note.name, note.libraryName)}>
								<span class="lib-dot" style="background:{libraryColorMap[note.libraryName] || '#7c3aed'}"></span>
								<span class="recent-name">{note.name.replace(/\.md$/, '')}</span>
								<span class="recent-time">{new Date(note.editedAt).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
							</button>
						{/each}
					</div>
				{:else}
					<p class="recent-empty">&mdash;</p>
				{/if}
			</div>
			<div class="recent-column">
				<h3 class="dashboard-section-title">{$t('secondScreen.dashboard.recentlyOpened')}</h3>
				{#if recentlyOpened.length > 0}
					<div class="recent-list">
						{#each recentlyOpened as note}
							<button class="recent-note" onclick={() => handleNoteClick(note.path, note.name, note.libraryName)}>
								<span class="lib-dot" style="background:{libraryColorMap[note.libraryName] || '#7c3aed'}"></span>
								<span class="recent-name">{note.name.replace(/\.md$/, '')}</span>
								<span class="recent-time">{new Date(note.openedAt).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
							</button>
						{/each}
					</div>
				{:else}
					<p class="recent-empty">&mdash;</p>
				{/if}
			</div>
		</div>

		{#if dashboardTags.length > 0}
			<div class="dashboard-section">
				<h3 class="dashboard-section-title">{$t('secondScreen.dashboard.tags')}</h3>
				<div class="tags-layout" class:tags-split={selectedTag}>
					<div class="tags-list-col">
						<div class="dashboard-tags">
							{#each dashboardTags as { tag, count }}
								<button
									class="dashboard-tag"
									class:tag-selected={selectedTag === tag}
									onclick={() => selectTag(tag)}
								>
									<span class="tag-name">#{tag}</span>
									<span class="tag-count">{count}</span>
								</button>
							{/each}
						</div>
					</div>
					{#if selectedTag}
						<div class="tags-notes-col">
							<h4 class="tags-notes-title">
								<span class="tag-badge">#{selectedTag}</span>
								<span class="tags-notes-count">{selectedTagNotes.length}</span>
								<button class="tags-notes-close" onclick={() => { selectedTag = null; selectedTagNotes = []; }}>
									<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
								</button>
							</h4>
							{#if loadingTagNotes}
								<p class="recent-empty">{$t('secondScreen.loading')}</p>
							{:else if selectedTagNotes.length > 0}
								<div class="recent-list">
									{#each selectedTagNotes as note}
										<button class="recent-note" onclick={() => {
											if (onNoteToMain) {
												const lib = $libraries.find(v => v.name === note.libraryName);
												onNoteToMain({ path: note.path, name: note.name, libraryName: note.libraryName, libraryPath: lib?.path ?? '', libraryColor: libraryColorMap[note.libraryName] || '#7c3aed' });
											} else {
												handleNoteClick(note.path, note.name, note.libraryName);
											}
										}}>
											<span class="lib-dot" style="background:{libraryColorMap[note.libraryName] || '#7c3aed'}"></span>
											<span class="recent-name">{note.name.replace(/\.md$/, '')}</span>
										</button>
									{/each}
								</div>
							{:else}
								<p class="recent-empty">&mdash;</p>
							{/if}
						</div>
					{/if}
				</div>
			</div>
		{/if}
	</div>
</div>

<style>
	.dashboard-companion { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
	.dashboard-scroll { flex: 1; overflow-y: auto; padding: 24px 28px; }

	.dashboard-header { display: flex; align-items: center; gap: 10px; margin-bottom: 24px; }
	.dashboard-header h2 { font-size: 20px; font-weight: 700; color: var(--text-normal); margin: 0; }

	.dashboard-stats {
		display: grid; grid-template-columns: repeat(auto-fit, minmax(110px, 1fr));
		gap: 12px; margin-bottom: 28px;
	}
	.stat-card {
		display: flex; flex-direction: column; align-items: center; gap: 4px;
		padding: 16px 12px; border-radius: 10px;
		background: var(--background-secondary);
		border: 1px solid var(--background-modifier-border);
	}
	.stat-value { font-size: 28px; font-weight: 700; color: var(--interactive-accent); line-height: 1; }
	.stat-label { font-size: 11px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; text-align: center; }

	.dashboard-section { margin-bottom: 24px; }
	.dashboard-section-title {
		font-size: 12px; font-weight: 600; color: var(--text-muted);
		text-transform: uppercase; letter-spacing: 0.5px;
		margin: 0 0 12px 0; padding-bottom: 8px;
		border-bottom: 1px solid var(--background-modifier-border);
	}

	.library-list { display: flex; flex-direction: column; gap: 10px; }
	.library-card {
		background: var(--background-secondary);
		border-radius: 10px; padding: 10px 14px;
		border: 1px solid var(--background-modifier-border);
	}
	.library-card-header { display: flex; align-items: center; gap: 8px; margin-bottom: 8px; }
	.lib-dot { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }
	.lib-name { flex: 1; font-size: 14px; font-weight: 600; color: var(--text-normal); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.library-card-stats { display: flex; gap: 8px; }
	.lib-stat-box {
		flex: 1; display: flex; flex-direction: column; align-items: center; gap: 2px;
		padding: 8px 6px; border-radius: 8px;
		background: color-mix(in srgb, var(--lib-color) 8%, var(--background-primary));
		border: 1px solid color-mix(in srgb, var(--lib-color) 18%, transparent);
	}
	.lib-stat-value { font-size: 18px; font-weight: 700; color: var(--lib-color); line-height: 1; }
	.lib-stat-label { font-size: 10px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.4px; }

	.recent-columns { display: grid; grid-template-columns: 1fr 1fr; gap: 24px; }
	.recent-column { min-width: 0; }
	.recent-empty { color: var(--text-faint); font-size: 13px; padding: 8px 12px; margin: 0; }
	.recent-list { display: flex; flex-direction: column; gap: 2px; }
	.recent-note {
		display: flex; align-items: center; gap: 10px;
		padding: 8px 12px; border-radius: 8px; border: none; width: 100%;
		background: transparent; color: var(--text-normal); cursor: pointer;
		text-align: start; font-family: inherit; transition: background 0.15s;
	}
	.recent-note:hover { background: var(--background-modifier-hover); }
	.recent-name { flex: 1; font-size: 14px; font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; min-width: 0; }
	.recent-time { font-size: 11px; color: var(--text-faint); white-space: nowrap; margin-inline-start: auto; }

	.cu-list { display: flex; flex-direction: column; gap: 8px; }
	.cu-group {
		background: var(--background-secondary);
		border-radius: 10px; overflow: hidden;
		border: 1px solid var(--background-modifier-border);
	}
	.cu-header { display: flex; align-items: center; gap: 10px; padding: 10px 14px 0; }
	.cu-name { flex: 1; font-size: 14px; font-weight: 600; color: var(--text-normal); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.cu-stat-boxes { display: flex; gap: 8px; padding: 10px 14px; }
	.cu-stat-box {
		flex: 1; display: flex; flex-direction: column; align-items: center; gap: 2px;
		padding: 8px 6px; border-radius: 8px;
		background: color-mix(in srgb, #6366f1 8%, var(--background-primary));
		border: 1px solid color-mix(in srgb, #6366f1 18%, transparent);
	}
	.cu-stat-value { font-size: 18px; font-weight: 700; color: #6366f1; line-height: 1; }
	.cu-stat-label { font-size: 10px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.4px; }
	.cu-libs { padding: 0 14px 10px; margin-top: 0; display: flex; flex-direction: column; gap: 8px; }

	.tags-layout { display: block; }
	.tags-layout.tags-split { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }
	.tags-list-col { min-width: 0; }
	.tags-notes-col {
		min-width: 0; border-left: 1px solid var(--background-modifier-border);
		padding-left: 20px; font-family: var(--font-interface-theme);
	}
	.tags-notes-title {
		display: flex; align-items: center; gap: 8px;
		margin: 0 0 10px 0; font-size: 13px; font-weight: 600; color: var(--text-normal);
	}
	.tags-notes-close {
		margin-left: auto; background: none; border: none; cursor: pointer;
		color: var(--text-muted); padding: 2px; border-radius: 4px;
		display: flex; align-items: center;
	}
	.tags-notes-close:hover { color: var(--text-normal); background: var(--background-modifier-hover); }
	.tag-badge {
		padding: 2px 10px; border-radius: 12px;
		background: var(--interactive-accent); color: white; font-size: 12px;
	}
	.tags-notes-count {
		font-size: 11px; color: var(--text-faint);
		background: var(--background-modifier-border);
		padding: 1px 6px; border-radius: 8px;
	}

	.dashboard-tags { display: flex; flex-wrap: wrap; gap: 6px; }
	.dashboard-tag {
		display: inline-flex; align-items: center; gap: 4px;
		padding: 4px 10px; border-radius: 12px;
		background: var(--background-secondary);
		border: 1px solid var(--background-modifier-border);
		font-family: var(--font-interface-theme);
		font-size: 12px; color: var(--text-muted);
		cursor: pointer; transition: all 0.15s;
	}
	.dashboard-tag:hover { border-color: var(--interactive-accent); }
	.dashboard-tag.tag-selected {
		background: var(--interactive-accent); border-color: var(--interactive-accent);
	}
	.dashboard-tag.tag-selected .tag-name { color: white; }
	.dashboard-tag.tag-selected .tag-count { background: rgba(255,255,255,0.25); color: white; }
	.tag-name { color: var(--text-normal); }
	.tag-count {
		font-size: 10px; color: var(--text-faint);
		background: var(--background-modifier-border);
		padding: 1px 5px; border-radius: 8px; min-width: 16px; text-align: center;
	}
</style>
