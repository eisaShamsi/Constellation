<script lang="ts">
	import { t } from '$lib/i18n';
	import { openNoteTab } from '$lib/libraries/store';
	import type { TaskItem } from '$lib/tasks/types';

	let {
		tasks = [] as TaskItem[],
		onToggle,
		libraryColorMap = {} as Record<string, string>,
	}: {
		tasks: TaskItem[];
		onToggle: (filePath: string, lineNumber: number) => void;
		libraryColorMap?: Record<string, string>;
	} = $props();

	let filter = $state<'all' | 'incomplete' | 'completed'>('all');
	let sortBy = $state<'default' | 'due' | 'priority'>('default');

	const priorityOrder: Record<string, number> = { high: 0, medium: 1, low: 2 };

	const filteredTasks = $derived.by(() => {
		let list = tasks;
		if (filter === 'incomplete') list = list.filter(t => !t.completed);
		if (filter === 'completed') list = list.filter(t => t.completed);

		if (sortBy === 'due') {
			list = [...list].sort((a, b) => {
				if (!a.due_date && !b.due_date) return 0;
				if (!a.due_date) return 1;
				if (!b.due_date) return -1;
				return a.due_date.localeCompare(b.due_date);
			});
		} else if (sortBy === 'priority') {
			list = [...list].sort((a, b) => {
				const pa = a.priority ? priorityOrder[a.priority] ?? 3 : 3;
				const pb = b.priority ? priorityOrder[b.priority] ?? 3 : 3;
				return pa - pb;
			});
		}
		return list;
	});

	const incompleteCount = $derived(tasks.filter(t => !t.completed).length);
	const completedCount = $derived(tasks.filter(t => t.completed).length);

	function getDueDateClass(dueDate: string | null): string {
		if (!dueDate) return '';
		const today = new Date().toISOString().slice(0, 10);
		if (dueDate < today) return 'overdue';
		if (dueDate === today) return 'due-today';
		return 'upcoming';
	}

	function formatDueDate(dueDate: string): string {
		const today = new Date().toISOString().slice(0, 10);
		if (dueDate === today) return $t('tasksPanel.dueToday');
		const d = new Date(dueDate + 'T00:00:00');
		const diff = Math.ceil((d.getTime() - new Date(today + 'T00:00:00').getTime()) / 86400000);
		if (diff < 0) return `${Math.abs(diff)}d ${$t('tasksPanel.overdue').toLowerCase()}`;
		if (diff === 1) return $t('tasksPanel.tomorrow');
		if (diff <= 7) return `${diff}d`;
		return dueDate;
	}

	function getPriorityIcon(priority: string | null): string {
		switch (priority) {
			case 'high': return '⏫';
			case 'medium': return '🔼';
			case 'low': return '🔽';
			default: return '';
		}
	}

	function getLibraryColor(libraryName: string): string {
		return libraryColorMap[libraryName] || '#7c3aed';
	}

	async function openTask(task: TaskItem, e?: MouseEvent) {
		const newTab = e ? (e.ctrlKey || e.metaKey || e.button === 1) : false;
		await openNoteTab(task.file_path, task.library_name, getLibraryColor(task.library_name), undefined, newTab);
	}

	// Strip metadata from display text (due dates, priority emojis, completion dates)
	function cleanTaskText(text: string): string {
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
</script>

<div class="tasks-panel">
	<!-- Filter bar -->
	<div class="tp-filters">
		<button class="tp-filter-btn" class:active={filter === 'all'} onclick={() => filter = 'all'}>
			{$t('tasksPanel.all')} <span class="tp-count">{tasks.length}</span>
		</button>
		<button class="tp-filter-btn" class:active={filter === 'incomplete'} onclick={() => filter = 'incomplete'}>
			{$t('tasksPanel.incomplete')} <span class="tp-count">{incompleteCount}</span>
		</button>
		<button class="tp-filter-btn" class:active={filter === 'completed'} onclick={() => filter = 'completed'}>
			{$t('tasksPanel.completed')} <span class="tp-count">{completedCount}</span>
		</button>
	</div>

	<!-- Sort -->
	{#if filteredTasks.length > 1}
		<div class="tp-sort">
			<select bind:value={sortBy}>
				<option value="default">{$t('tasksPanel.sortDefault')}</option>
				<option value="due">{$t('tasksPanel.sortByDue')}</option>
				<option value="priority">{$t('tasksPanel.sortByPriority')}</option>
			</select>
		</div>
	{/if}

	<!-- Task list -->
	{#if filteredTasks.length > 0}
		<div class="tp-list">
			{#each filteredTasks as task (task.file_path + ':' + task.line_number)}
				<div class="tp-item" class:completed={task.completed}>
					<label class="tp-checkbox">
						<input
							type="checkbox"
							checked={task.completed}
							onchange={() => onToggle(task.file_path, task.line_number)}
						/>
					</label>
					<div class="tp-content">
						<span class="tp-text" class:done={task.completed}>{cleanTaskText(task.text)}</span>
						<div class="tp-meta">
							{#if task.priority}
								<span class="tp-priority tp-priority-{task.priority}" title={$t(`tasksPanel.${task.priority}Priority`)}>
									{getPriorityIcon(task.priority)}
								</span>
							{/if}
							{#if task.due_date}
								<span class="tp-due {getDueDateClass(task.due_date)}" title={task.due_date}>
									{formatDueDate(task.due_date)}
								</span>
							{/if}
							{#each task.tags as tag}
								<span class="tp-tag">{tag}</span>
							{/each}
							<button class="tp-file-link" onclick={(e) => openTask(task, e)} title={task.file_name}>
								{task.file_name}
							</button>
						</div>
					</div>
				</div>
			{/each}
		</div>
	{:else}
		<div class="tp-empty">{$t('tasksPanel.noTasks')}</div>
	{/if}
</div>

<style>
	.tasks-panel {
		display: flex;
		flex-direction: column;
		height: 100%;
	}
	.tp-filters {
		display: flex;
		gap: 2px;
		padding: 8px 12px 4px;
	}
	.tp-filter-btn {
		flex: 1;
		padding: 4px 6px;
		font-size: 0.72rem;
		border: 1px solid var(--border-light, #333);
		background: transparent;
		color: var(--text-faint, #888);
		border-radius: 4px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 4px;
	}
	.tp-filter-btn.active {
		background: var(--accent, #7c3aed);
		color: white;
		border-color: var(--accent, #7c3aed);
	}
	.tp-count {
		font-size: 0.65rem;
		opacity: 0.7;
	}
	.tp-sort {
		padding: 4px 12px;
	}
	.tp-sort select {
		width: 100%;
		font-size: 0.72rem;
		padding: 3px 6px;
		border: 1px solid var(--border-light, #333);
		background: var(--bg-secondary, #1e1e1e);
		color: var(--text-normal, #ccc);
		border-radius: 4px;
	}
	.tp-list {
		flex: 1;
		overflow-y: auto;
		padding: 4px 0;
	}
	.tp-item {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		padding: 6px 12px;
		border-bottom: 1px solid var(--border-faint, #222);
		transition: background 0.1s;
	}
	.tp-item:hover {
		background: var(--bg-hover, rgba(255, 255, 255, 0.03));
	}
	.tp-item.completed {
		opacity: 0.6;
	}
	.tp-checkbox {
		flex-shrink: 0;
		margin-top: 2px;
		cursor: pointer;
	}
	.tp-checkbox input {
		width: 14px;
		height: 14px;
		cursor: pointer;
		accent-color: var(--accent, #7c3aed);
	}
	.tp-content {
		flex: 1;
		min-width: 0;
	}
	.tp-text {
		font-size: 0.82rem;
		color: var(--text-normal, #ccc);
		line-height: 1.4;
		word-break: break-word;
	}
	.tp-text.done {
		text-decoration: line-through;
		color: var(--text-faint, #666);
	}
	.tp-meta {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		margin-top: 3px;
		align-items: center;
	}
	.tp-priority {
		font-size: 0.7rem;
	}
	.tp-due {
		font-size: 0.68rem;
		padding: 1px 5px;
		border-radius: 3px;
		font-weight: 500;
	}
	.tp-due.overdue {
		background: rgba(239, 68, 68, 0.2);
		color: #ef4444;
	}
	.tp-due.due-today {
		background: rgba(245, 158, 11, 0.2);
		color: #f59e0b;
	}
	.tp-due.upcoming {
		background: rgba(100, 100, 100, 0.2);
		color: var(--text-faint, #888);
	}
	.tp-tag {
		font-size: 0.65rem;
		padding: 1px 5px;
		border-radius: 3px;
		background: rgba(124, 58, 237, 0.15);
		color: var(--accent, #7c3aed);
	}
	.tp-file-link {
		font-size: 0.65rem;
		color: var(--text-faint, #666);
		background: none;
		border: none;
		cursor: pointer;
		padding: 0;
		text-decoration: underline;
		text-decoration-style: dotted;
		max-width: 120px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.tp-file-link:hover {
		color: var(--accent, #7c3aed);
	}
	.tp-empty {
		padding: 24px 12px;
		text-align: center;
		font-size: 0.8rem;
		color: var(--text-faint, #666);
	}
</style>
