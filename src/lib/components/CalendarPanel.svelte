<script lang="ts">
	// MIG-081 §C — renders the month grid in the chosen calendar SYSTEM (Gregorian,
	// Hijri [Eisa's astronomical engine], Solar-Hijri/Persian, Hebrew). All cells key
	// back to Gregorian ISO (onDayClick + the note/task dots); display labels are
	// localised (month name + day numbers in the locale's numbering system). RTL follows
	// the UI locale. Engines (hijri.js, Temporal) are lazy-loaded via calendarMath.
	import { t, dir, locale } from '$lib/i18n';
	import {
		ensureCalendarEngines, buildMonthGrid, todayInSystem, stepMonth,
		type CalendarSystem, type MonthGrid,
	} from '$lib/calendar/calendarMath';

	let {
		noteDates = {} as Record<string, number>,
		taskDueDates = {} as Record<string, number>,
		onDayClick,
		primarySystem = 'gregorian' as CalendarSystem,
		weekStart = 0 as 0 | 1,
	}: {
		noteDates: Record<string, number>;
		taskDueDates: Record<string, number>;
		onDayClick: (date: string) => void;
		primarySystem?: CalendarSystem;
		weekStart?: 0 | 1;
	} = $props();

	let viewYear = $state(0); // 0 = not yet initialised
	let viewMonth = $state(0);
	let enginesReady = $state(false);

	// Load the engine(s) the chosen system needs, then anchor the view on today-in-system.
	// Re-runs only when `primarySystem` changes (not on month-nav — it doesn't read the view).
	$effect(() => {
		const sys = primarySystem;
		let cancelled = false;
		enginesReady = false;
		ensureCalendarEngines([sys])
			.then(() => todayInSystem(sys))
			.then((tdy) => {
				if (cancelled) return;
				viewYear = tdy.year;
				viewMonth = tdy.month;
				enginesReady = true;
			})
			.catch(() => { if (!cancelled) enginesReady = true; });
		return () => { cancelled = true; };
	});

	// Synchronous once the engines are loaded; re-derives on view/locale/weekStart change.
	const grid = $derived.by<MonthGrid | null>(() => {
		void $locale; // re-derive when the locale (numerals/month names) changes
		if (!enginesReady || !viewYear) return null;
		try { return buildMonthGrid(primarySystem, viewYear, viewMonth, $locale, weekStart); }
		catch { return null; }
	});

	function prevMonth() { const n = stepMonth(primarySystem, viewYear, viewMonth, -1); viewYear = n.year; viewMonth = n.month; }
	function nextMonth() { const n = stepMonth(primarySystem, viewYear, viewMonth, 1); viewYear = n.year; viewMonth = n.month; }
	async function goToToday() { const tdy = await todayInSystem(primarySystem); viewYear = tdy.year; viewMonth = tdy.month; }

	function localeCount(n: number): string { try { return n.toLocaleString($locale); } catch { return String(n); } }
</script>

<div class="calendar-panel" dir={$dir}>
	<!-- Navigation header -->
	<div class="cp-header">
		<button class="cp-nav" onclick={prevMonth} title={$t('calendarPanel.prevMonth')}>‹</button>
		<button class="cp-month" onclick={goToToday} title={$t('calendarPanel.today')}>{grid?.monthLabel ?? ''}</button>
		<button class="cp-nav" onclick={nextMonth} title={$t('calendarPanel.nextMonth')}>›</button>
	</div>

	{#if grid}
		<!-- Weekday headers -->
		<div class="cp-weekdays">
			{#each grid.weekdayLabels as day}
				<div class="cp-weekday">{day}</div>
			{/each}
		</div>

		<!-- Calendar grid -->
		<div class="cp-grid">
			{#each grid.cells as cell}
				{@const nc = noteDates[cell.iso] || 0}
				{@const tc = taskDueDates[cell.iso] || 0}
				<button
					class="cp-day"
					class:other-month={!cell.inCurrentMonth}
					class:today={cell.isToday}
					class:has-notes={nc > 0}
					class:has-tasks={tc > 0}
					onclick={() => onDayClick(cell.iso)}
					title={[
						nc > 0 ? $t('calendarPanel.notesCount', { count: localeCount(nc) }) : '',
						tc > 0 ? $t('calendarPanel.tasksCount', { count: localeCount(tc) }) : ''
					].filter(Boolean).join(' · ')}
				>
					<span class="cp-day-num">{cell.dayLabel}</span>
					{#if nc > 0 || tc > 0}
						<div class="cp-dots">
							{#if nc > 0}<span class="cp-dot note-dot"></span>{/if}
							{#if tc > 0}<span class="cp-dot task-dot"></span>{/if}
						</div>
					{/if}
				</button>
			{/each}
		</div>
	{:else}
		<div class="cp-loading">{$t('common.loading') || '…'}</div>
	{/if}
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
	.cp-loading {
		text-align: center;
		color: var(--text-faint, #888);
		font-size: 0.8rem;
		padding: 24px 0;
	}
</style>
