<script lang="ts">
	import { onMount } from 'svelte';
	import { t, dir } from '$lib/i18n';
	import { libraries, openNoteTab } from '$lib/libraries/store';
	import { get } from 'svelte/store';
	import { scanVaultTasks, toggleTask } from '$lib/tasks/store';
	import type { TaskItem } from '$lib/tasks/types';

	let {
		libraryColorMap = {} as Record<string, string>,
		onClose,
	}: {
		libraryColorMap?: Record<string, string>;
		onClose: () => void;
	} = $props();

	let allTasks = $state<TaskItem[]>([]);
	let loading = $state(true);
	let scanTime = $state(0);

	// Filters
	let statusFilter = $state<'all' | 'incomplete' | 'completed'>('incomplete');
	let libraryFilter = $state<string>('all');
	let dueFilter = $state<'all' | 'overdue' | 'today' | 'week' | 'nodate'>('all');
	let priorityFilter = $state<'all' | 'high' | 'medium' | 'low'>('all');
	let searchQuery = $state('');
	let groupBy = $state<'none' | 'file' | 'vault' | 'priority' | 'due'>('file');
	let sortBy = $state<'due' | 'priority' | 'file'>('due');

	const todayStr = new Date().toISOString().slice(0, 10);
	const weekEnd = (() => {
		const d = new Date();
		d.setDate(d.getDate() + 7);
		return d.toISOString().slice(0, 10);
	})();

	const libraryNames = $derived(
		[...new Set(allTasks.map(t => t.library_name))].sort()
	);

	const priorityOrder: Record<string, number> = { high: 0, medium: 1, low: 2 };

	const filteredTasks = $derived.by(() => {
		let list = allTasks;

		// Status filter
		if (statusFilter === 'incomplete') list = list.filter(t => !t.completed);
		if (statusFilter === 'completed') list = list.filter(t => t.completed);

		// Library filter
		if (libraryFilter !== 'all') list = list.filter(t => t.library_name === libraryFilter);

		// Due date filter
		if (dueFilter === 'overdue') list = list.filter(t => t.due_date && t.due_date < todayStr);
		if (dueFilter === 'today') list = list.filter(t => t.due_date === todayStr);
		if (dueFilter === 'week') list = list.filter(t => t.due_date && t.due_date >= todayStr && t.due_date <= weekEnd);
		if (dueFilter === 'nodate') list = list.filter(t => !t.due_date);

		// Priority filter
		if (priorityFilter !== 'all') list = list.filter(t => t.priority === priorityFilter);

		// Search
		if (searchQuery.trim()) {
			const q = searchQuery.toLowerCase();
			list = list.filter(t =>
				t.text.toLowerCase().includes(q) ||
				t.file_name.toLowerCase().includes(q) ||
				t.tags.some(tag => tag.toLowerCase().includes(q))
			);
		}

		// Sort
		list = [...list].sort((a, b) => {
			if (sortBy === 'due') {
				if (!a.due_date && !b.due_date) return 0;
				if (!a.due_date) return 1;
				if (!b.due_date) return -1;
				return a.due_date.localeCompare(b.due_date);
			}
			if (sortBy === 'priority') {
				const pa = a.priority ? priorityOrder[a.priority] ?? 3 : 3;
				const pb = b.priority ? priorityOrder[b.priority] ?? 3 : 3;
				return pa - pb;
			}
			return a.file_name.localeCompare(b.file_name);
		});

		return list;
	});

	// Group tasks
	const groupedTasks = $derived.by(() => {
		if (groupBy === 'none') return [{ key: '', tasks: filteredTasks }];
		const groups = new Map<string, TaskItem[]>();
		for (const task of filteredTasks) {
			let key = '';
			if (groupBy === 'file') key = task.file_name;
			else if (groupBy === 'vault') key = task.library_name;
			else if (groupBy === 'priority') key = task.priority || $t('tasksPanel.noPriority');
			else if (groupBy === 'due') {
				if (!task.due_date) key = $t('tasksPanel.noDueDate');
				else if (task.due_date < todayStr) key = $t('tasksPanel.overdue');
				else if (task.due_date === todayStr) key = $t('tasksPanel.dueToday');
				else key = task.due_date;
			}
			if (!groups.has(key)) groups.set(key, []);
			groups.get(key)!.push(task);
		}
		return Array.from(groups.entries()).map(([key, tasks]) => ({ key, tasks }));
	});

	async function loadAllTasks() {
		loading = true;
		const start = performance.now();
		const libraryList = get(libraries);
		try {
			const results = await Promise.all(
				libraryList.map(v => scanVaultTasks(v.path, v.name))
			);
			allTasks = results.flatMap(r => r.tasks);
			scanTime = Math.round(performance.now() - start);
		} catch (e) {
			console.error('Failed to scan tasks:', e);
		}
		loading = false;
	}

	async function handleToggle(filePath: string, lineNumber: number) {
		try {
			await toggleTask(filePath, lineNumber);
			// Refresh
			await loadAllTasks();
		} catch (e) {
			console.error('Failed to toggle task:', e);
		}
	}

	async function openTask(task: TaskItem, e?: MouseEvent) {
		const newTab = e ? (e.ctrlKey || e.metaKey || e.button === 1) : false;
		const vc = libraryColorMap[task.library_name] || '#7c3aed';
		await openNoteTab(task.file_path, task.library_name, vc, undefined, newTab);
	}

	function cleanText(text: string): string {
		return text
			.replace(/\u{1F4C5}\s*\d{4}-\d{2}-\d{2}/gu, '')
			.replace(/\u{2705}\s*\d{4}-\d{2}-\d{2}/gu, '')
			.replace(/\u{2795}\s*\d{4}-\d{2}-\d{2}/gu, '')
			.replace(/\[due::\s*\d{4}-\d{2}-\d{2}\]/gi, '')
			.replace(/due::\s*\d{4}-\d{2}-\d{2}/gi, '')
			.replace(/\[priority::\s*\w+\]/gi, '')
			.replace(/\[completion::\s*\d{4}-\d{2}-\d{2}\]/gi, '')
			.replace(/\[created::\s*\d{4}-\d{2}-\d{2}\]/gi, '')
			.replace(/[\u{23EB}\u{1F53C}\u{1F53D}]/gu, '')
			.trim();
	}

	function getDueBadge(dueDate: string | null): { text: string; cls: string } {
		if (!dueDate) return { text: '', cls: '' };
		if (dueDate < todayStr) return { text: $t('tasksPanel.overdue'), cls: 'overdue' };
		if (dueDate === todayStr) return { text: $t('tasksPanel.dueToday'), cls: 'due-today' };
		return { text: dueDate, cls: 'upcoming' };
	}

	function getPriorityIcon(p: string | null): string {
		if (p === 'high') return '⏫';
		if (p === 'medium') return '🔼';
		if (p === 'low') return '🔽';
		return '';
	}

	onMount(() => {
		loadAllTasks();
	});
</script>

<div class="global-tasks" dir={$dir}>
	<!-- Header -->
	<div class="gt-header">
		<h2 class="gt-title">{$t('globalTasks.title')}</h2>
		<div class="gt-header-end">
			{#if !loading}
				<span class="gt-stats">{filteredTasks.length} / {allTasks.length} · {scanTime}ms</span>
			{/if}
			<button class="gt-refresh" onclick={loadAllTasks} title={$t('globalTasks.refresh')}>↻</button>
			<button class="gt-close" onclick={onClose}>×</button>
		</div>
	</div>

	<!-- Toolbar -->
	<div class="gt-toolbar">
		<!-- Status filter -->
		<div class="gt-filter-group">
			<button class="gt-fbtn" class:active={statusFilter === 'all'} onclick={() => statusFilter = 'all'}>{$t('tasksPanel.all')}</button>
			<button class="gt-fbtn" class:active={statusFilter === 'incomplete'} onclick={() => statusFilter = 'incomplete'}>{$t('tasksPanel.incomplete')}</button>
			<button class="gt-fbtn" class:active={statusFilter === 'completed'} onclick={() => statusFilter = 'completed'}>{$t('tasksPanel.completed')}</button>
		</div>

		<!-- Due date filter -->
		<div class="gt-filter-group">
			<select bind:value={dueFilter}>
				<option value="all">{$t('globalTasks.allDates')}</option>
				<option value="overdue">{$t('tasksPanel.overdue')}</option>
				<option value="today">{$t('tasksPanel.dueToday')}</option>
				<option value="week">{$t('globalTasks.thisWeek')}</option>
				<option value="nodate">{$t('tasksPanel.noDueDate')}</option>
			</select>
		</div>

		<!-- Library filter -->
		{#if libraryNames.length > 1}
			<div class="gt-filter-group">
				<select bind:value={libraryFilter}>
					<option value="all">{$t('globalTasks.allVaults')}</option>
					{#each libraryNames as vn}
						<option value={vn}>{vn}</option>
					{/each}
				</select>
			</div>
		{/if}

		<!-- Priority -->
		<div class="gt-filter-group">
			<select bind:value={priorityFilter}>
				<option value="all">{$t('globalTasks.allPriorities')}</option>
				<option value="high">⏫ {$t('tasksPanel.highPriority')}</option>
				<option value="medium">🔼 {$t('tasksPanel.mediumPriority')}</option>
				<option value="low">🔽 {$t('tasksPanel.lowPriority')}</option>
			</select>
		</div>

		<!-- Group & Sort -->
		<div class="gt-filter-group">
			<select bind:value={groupBy}>
				<option value="none">{$t('globalTasks.noGroup')}</option>
				<option value="file">{$t('globalTasks.groupByFile')}</option>
				<option value="vault">{$t('globalTasks.groupByVault')}</option>
				<option value="priority">{$t('globalTasks.groupByPriority')}</option>
				<option value="due">{$t('globalTasks.groupByDue')}</option>
			</select>
		</div>

		<div class="gt-filter-group">
			<select bind:value={sortBy}>
				<option value="due">{$t('tasksPanel.sortByDue')}</option>
				<option value="priority">{$t('tasksPanel.sortByPriority')}</option>
				<option value="file">{$t('globalTasks.sortByFile')}</option>
			</select>
		</div>

		<!-- Search -->
		<div class="gt-search">
			<input type="text" bind:value={searchQuery} placeholder={$t('globalTasks.search')} />
		</div>
	</div>

	<!-- Content -->
	<div class="gt-content">
		{#if loading}
			<div class="gt-loading">{$t('globalTasks.scanning')}</div>
		{:else if filteredTasks.length === 0}
			<div class="gt-empty">{$t('globalTasks.noTasksFound')}</div>
		{:else}
			{#each groupedTasks as group}
				{#if group.key}
					<div class="gt-group-header">
						<span class="gt-group-name">{group.key}</span>
						<span class="gt-group-count">{group.tasks.length}</span>
					</div>
				{/if}
				{#each group.tasks as task (task.file_path + ':' + task.line_number)}
					<div class="gt-task" class:completed={task.completed}>
						<label class="gt-checkbox">
							<input
								type="checkbox"
								checked={task.completed}
								onchange={() => handleToggle(task.file_path, task.line_number)}
							/>
						</label>
						<div class="gt-task-body">
							<span class="gt-text" class:done={task.completed}>{cleanText(task.text)}</span>
							<div class="gt-meta">
								{#if task.priority}
									<span class="gt-priority">{getPriorityIcon(task.priority)}</span>
								{/if}
								{#if getDueBadge(task.due_date).text}
									<span class="gt-due {getDueBadge(task.due_date).cls}">{getDueBadge(task.due_date).text}</span>
								{/if}
								{#each task.tags as tag}
									<span class="gt-tag">{tag}</span>
								{/each}
								<button class="gt-file-link" onclick={(e) => openTask(task, e)}>
									<span class="gt-library-dot" style="background:{libraryColorMap[task.library_name] || '#7c3aed'}"></span>
									{task.file_name}
								</button>
							</div>
						</div>
					</div>
				{/each}
			{/each}
		{/if}
	</div>
</div>

<style>
	.global-tasks {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: var(--bg-primary, #1a1a1a);
	}
	.gt-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		border-bottom: 1px solid var(--border-light, #333);
	}
	.gt-title {
		font-size: 1rem;
		font-weight: 600;
		margin: 0;
		color: var(--text-normal, #ccc);
	}
	.gt-header-end {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.gt-stats {
		font-size: 0.72rem;
		color: var(--text-faint, #666);
	}
	.gt-refresh, .gt-close {
		width: 28px;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		background: none;
		border: none;
		color: var(--text-faint, #888);
		font-size: 1.1rem;
		cursor: pointer;
		border-radius: 4px;
	}
	.gt-refresh:hover, .gt-close:hover {
		background: var(--bg-hover, rgba(255, 255, 255, 0.05));
		color: var(--text-normal, #ccc);
	}

	/* Toolbar */
	.gt-toolbar {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		padding: 8px 16px;
		border-bottom: 1px solid var(--border-light, #333);
		align-items: center;
	}
	.gt-filter-group {
		display: flex;
		gap: 2px;
	}
	.gt-fbtn {
		padding: 4px 8px;
		font-size: 0.72rem;
		border: 1px solid var(--border-light, #333);
		background: transparent;
		color: var(--text-faint, #888);
		border-radius: 4px;
		cursor: pointer;
	}
	.gt-fbtn.active {
		background: var(--accent, #7c3aed);
		color: white;
		border-color: var(--accent, #7c3aed);
	}
	.gt-filter-group select {
		font-size: 0.72rem;
		padding: 4px 6px;
		border: 1px solid var(--border-light, #333);
		background: var(--bg-secondary, #1e1e1e);
		color: var(--text-normal, #ccc);
		border-radius: 4px;
	}
	.gt-search {
		flex: 1;
		min-width: 120px;
	}
	.gt-search input {
		width: 100%;
		padding: 4px 8px;
		font-size: 0.75rem;
		border: 1px solid var(--border-light, #333);
		background: var(--bg-secondary, #1e1e1e);
		color: var(--text-normal, #ccc);
		border-radius: 4px;
	}

	/* Content */
	.gt-content {
		flex: 1;
		overflow-y: auto;
		padding: 4px 0;
	}
	.gt-loading, .gt-empty {
		padding: 40px 16px;
		text-align: center;
		font-size: 0.85rem;
		color: var(--text-faint, #666);
	}

	/* Groups */
	.gt-group-header {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 16px 4px;
		font-size: 0.75rem;
		font-weight: 600;
		color: var(--accent, #7c3aed);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}
	.gt-group-count {
		font-size: 0.65rem;
		background: var(--accent, #7c3aed);
		color: white;
		padding: 0 5px;
		border-radius: 8px;
		font-weight: 500;
	}

	/* Task rows */
	.gt-task {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		padding: 8px 16px;
		border-bottom: 1px solid var(--border-faint, #222);
		transition: background 0.1s;
	}
	.gt-task:hover {
		background: var(--bg-hover, rgba(255, 255, 255, 0.03));
	}
	.gt-task.completed {
		opacity: 0.5;
	}
	.gt-checkbox {
		flex-shrink: 0;
		margin-top: 2px;
		cursor: pointer;
	}
	.gt-checkbox input {
		width: 15px;
		height: 15px;
		cursor: pointer;
		accent-color: var(--accent, #7c3aed);
	}
	.gt-task-body {
		flex: 1;
		min-width: 0;
	}
	.gt-text {
		font-size: 0.85rem;
		color: var(--text-normal, #ccc);
		line-height: 1.4;
		word-break: break-word;
	}
	.gt-text.done {
		text-decoration: line-through;
		color: var(--text-faint, #666);
	}
	.gt-meta {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		margin-top: 4px;
		align-items: center;
	}
	.gt-priority {
		font-size: 0.75rem;
	}
	.gt-due {
		font-size: 0.7rem;
		padding: 1px 6px;
		border-radius: 3px;
		font-weight: 500;
	}
	.gt-due.overdue {
		background: rgba(239, 68, 68, 0.2);
		color: #ef4444;
	}
	.gt-due.due-today {
		background: rgba(245, 158, 11, 0.2);
		color: #f59e0b;
	}
	.gt-due.upcoming {
		background: rgba(100, 100, 100, 0.2);
		color: var(--text-faint, #888);
	}
	.gt-tag {
		font-size: 0.68rem;
		padding: 1px 5px;
		border-radius: 3px;
		background: rgba(124, 58, 237, 0.15);
		color: var(--accent, #7c3aed);
	}
	.gt-file-link {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 0.7rem;
		color: var(--text-faint, #666);
		background: none;
		border: none;
		cursor: pointer;
		padding: 0;
		max-width: 180px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.gt-file-link:hover {
		color: var(--accent, #7c3aed);
	}
	.gt-library-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		flex-shrink: 0;
	}
</style>
