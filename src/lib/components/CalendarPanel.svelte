<script lang="ts">
	import { t, dir } from '$lib/i18n';

	let {
		noteDates = {} as Record<string, number>,
		taskDueDates = {} as Record<string, number>,
		onDayClick,
	}: {
		noteDates: Record<string, number>;
		taskDueDates: Record<string, number>;
		onDayClick: (date: string) => void;
	} = $props();

	let viewYear = $state(new Date().getFullYear());
	let viewMonth = $state(new Date().getMonth()); // 0-indexed

	const todayStr = $derived(new Date().toISOString().slice(0, 10));

	const monthName = $derived(
		new Intl.DateTimeFormat(undefined, { month: 'long', year: 'numeric' }).format(
			new Date(viewYear, viewMonth, 1)
		)
	);

	const weekDays = $derived.by(() => {
		const formatter = new Intl.DateTimeFormat(undefined, { weekday: 'narrow' });
		const days: string[] = [];
		// Start from Sunday (0) through Saturday (6)
		for (let i = 0; i < 7; i++) {
			const d = new Date(2024, 0, i); // Jan 2024 starts on Monday, Jan 0 = Sun Dec 31
			// Adjust: Jan 7 = Sun, Jan 1 = Mon, etc
			const date = new Date(2024, 6, 7 + i); // Jul 7 2024 is Sunday
			days.push(formatter.format(date));
		}
		return days;
	});

	interface CalendarDay {
		day: number;
		dateStr: string;
		isCurrentMonth: boolean;
		isToday: boolean;
		noteCount: number;
		taskCount: number;
	}

	const calendarDays = $derived.by(() => {
		const firstDay = new Date(viewYear, viewMonth, 1);
		const lastDay = new Date(viewYear, viewMonth + 1, 0);
		const startDow = firstDay.getDay(); // 0=Sun
		const daysInMonth = lastDay.getDate();

		const days: CalendarDay[] = [];

		// Fill leading days from previous month
		const prevMonthLastDay = new Date(viewYear, viewMonth, 0).getDate();
		for (let i = startDow - 1; i >= 0; i--) {
			const d = prevMonthLastDay - i;
			const m = viewMonth === 0 ? 12 : viewMonth;
			const y = viewMonth === 0 ? viewYear - 1 : viewYear;
			const dateStr = `${y}-${String(m).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
			days.push({
				day: d,
				dateStr,
				isCurrentMonth: false,
				isToday: dateStr === todayStr,
				noteCount: noteDates[dateStr] || 0,
				taskCount: taskDueDates[dateStr] || 0,
			});
		}

		// Current month days
		for (let d = 1; d <= daysInMonth; d++) {
			const dateStr = `${viewYear}-${String(viewMonth + 1).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
			days.push({
				day: d,
				dateStr,
				isCurrentMonth: true,
				isToday: dateStr === todayStr,
				noteCount: noteDates[dateStr] || 0,
				taskCount: taskDueDates[dateStr] || 0,
			});
		}

		// Fill trailing days
		const remaining = 42 - days.length; // 6 rows × 7
		for (let d = 1; d <= remaining; d++) {
			const m = viewMonth === 11 ? 1 : viewMonth + 2;
			const y = viewMonth === 11 ? viewYear + 1 : viewYear;
			const dateStr = `${y}-${String(m).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
			days.push({
				day: d,
				dateStr,
				isCurrentMonth: false,
				isToday: dateStr === todayStr,
				noteCount: noteDates[dateStr] || 0,
				taskCount: taskDueDates[dateStr] || 0,
			});
		}

		return days;
	});

	function prevMonth() {
		if (viewMonth === 0) {
			viewMonth = 11;
			viewYear--;
		} else {
			viewMonth--;
		}
	}

	function nextMonth() {
		if (viewMonth === 11) {
			viewMonth = 0;
			viewYear++;
		} else {
			viewMonth++;
		}
	}

	function goToToday() {
		const now = new Date();
		viewYear = now.getFullYear();
		viewMonth = now.getMonth();
	}
</script>

<div class="calendar-panel" dir={$dir}>
	<!-- Navigation header -->
	<div class="cp-header">
		<button class="cp-nav" onclick={prevMonth} title={$t('calendarPanel.prevMonth')}>‹</button>
		<button class="cp-month" onclick={goToToday} title={$t('calendarPanel.today')}>
			{monthName}
		</button>
		<button class="cp-nav" onclick={nextMonth} title={$t('calendarPanel.nextMonth')}>›</button>
	</div>

	<!-- Weekday headers -->
	<div class="cp-weekdays">
		{#each weekDays as day}
			<div class="cp-weekday">{day}</div>
		{/each}
	</div>

	<!-- Calendar grid -->
	<div class="cp-grid">
		{#each calendarDays as cell}
			<button
				class="cp-day"
				class:other-month={!cell.isCurrentMonth}
				class:today={cell.isToday}
				class:has-notes={cell.noteCount > 0}
				class:has-tasks={cell.taskCount > 0}
				onclick={() => onDayClick(cell.dateStr)}
				title={[
					cell.noteCount > 0 ? $t('calendarPanel.notesCount', { count: cell.noteCount.toLocaleString() }) : '',
					cell.taskCount > 0 ? $t('calendarPanel.tasksCount', { count: cell.taskCount.toLocaleString() }) : ''
				].filter(Boolean).join(' · ')}
			>
				<span class="cp-day-num">{cell.day}</span>
				{#if cell.noteCount > 0 || cell.taskCount > 0}
					<div class="cp-dots">
						{#if cell.noteCount > 0}
							<span class="cp-dot note-dot"></span>
						{/if}
						{#if cell.taskCount > 0}
							<span class="cp-dot task-dot"></span>
						{/if}
					</div>
				{/if}
			</button>
		{/each}
	</div>
</div>

<style>
	.calendar-panel {
		padding: 8px;
	}
	.cp-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 4px 0 8px;
	}
	.cp-nav {
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
	.cp-nav:hover {
		background: var(--bg-hover, rgba(255, 255, 255, 0.05));
		color: var(--text-normal, #ccc);
	}
	.cp-month {
		font-size: 0.82rem;
		font-weight: 600;
		color: var(--text-normal, #ccc);
		background: none;
		border: none;
		cursor: pointer;
		padding: 4px 8px;
		border-radius: 4px;
	}
	.cp-month:hover {
		background: var(--bg-hover, rgba(255, 255, 255, 0.05));
	}
	.cp-weekdays {
		display: grid;
		grid-template-columns: repeat(7, 1fr);
		gap: 1px;
		margin-bottom: 2px;
	}
	.cp-weekday {
		text-align: center;
		font-size: 0.65rem;
		color: var(--text-faint, #666);
		font-weight: 600;
		padding: 2px 0;
		text-transform: uppercase;
	}
	.cp-grid {
		display: grid;
		grid-template-columns: repeat(7, 1fr);
		gap: 1px;
	}
	.cp-day {
		aspect-ratio: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		background: none;
		border: none;
		border-radius: 4px;
		cursor: pointer;
		padding: 2px;
		position: relative;
		min-height: 30px;
	}
	.cp-day:hover {
		background: var(--bg-hover, rgba(255, 255, 255, 0.05));
	}
	.cp-day.other-month {
		opacity: 0.3;
	}
	.cp-day.today .cp-day-num {
		background: var(--accent, #7c3aed);
		color: white;
		border-radius: 50%;
		width: 22px;
		height: 22px;
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.cp-day.has-notes {
		font-weight: 600;
	}
	.cp-day-num {
		font-size: 0.75rem;
		color: var(--text-normal, #ccc);
		line-height: 1;
	}
	.cp-dots {
		display: flex;
		gap: 2px;
		margin-top: 2px;
		position: absolute;
		bottom: 2px;
	}
	.cp-dot {
		width: 4px;
		height: 4px;
		border-radius: 50%;
	}
	.cp-dot.note-dot {
		background: var(--accent, #7c3aed);
	}
	.cp-dot.task-dot {
		background: #ef4444;
	}
</style>
