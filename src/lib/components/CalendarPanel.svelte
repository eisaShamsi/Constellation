<script lang="ts">
	// MIG-081 §C.2b — RICH calendar, ported from Eisa's hijri-calendar app
	// (eisaShamsi/hijri-calendar). Renders the chosen system via the vendored engine:
	// ornate header + gold "AH" pill (cream when a sacred month), Gregorian-range
	// subtitle, dual-date cells (primary day large + Gregorian sub-number), moon-phase
	// glyphs, Islamic-event dots (Hijri only), note/task dots, optional Wk column.
	// All colours/fonts are CSS variables (--cal-*) so the Style Setter can retheme it
	// (§C.2d). Cells key onDayClick + the note/task dots on Gregorian ISO. RTL via dir.
	import { t, tn, dir, locale } from '$lib/i18n';
	import {
		ensureCalendarEngines, buildRichMonthGrid, todayInSystem, stepMonth, applyCalendarPrefs, setLunarYearStyles, setMonthNameStyle,
		type CalendarSystem, type RichMonthGrid, type CalculationMode,
	} from '$lib/calendar/calendarMath';
	import type { NoteDateEntry, TaskItem } from '$lib/tasks/types';

	let {
		noteEntries = {} as Record<string, NoteDateEntry[]>,
		taskEntries = {} as Record<string, TaskItem[]>,
		onDayClick,
		onOpenNote = (() => {}) as (entry: NoteDateEntry) => void,
		onOpenTask = (() => {}) as (task: TaskItem) => void,
		onToggleTask = (() => {}) as (task: TaskItem) => void,
		primarySystem = 'gregorian' as CalendarSystem,
		secondarySystem = 'none' as CalendarSystem | 'none',
		weekStart = 0 as 0 | 1,
		showWeekNumbers = true,
		corrections = {} as Record<string, number>,
		calculationMode = 'astronomical' as CalculationMode,
		chineseYearStyle = 'sexagenary-gregorian',  // §B — lunisolar year-display preference (Chinese)
		koreanYearStyle = 'dangi',                   // §B — lunisolar year-display preference (Korean)
		monthNameStyle = 'native',                   // §B.2 — lunisolar month names: native | phonetic
	}: {
		noteEntries: Record<string, NoteDateEntry[]>;
		taskEntries: Record<string, TaskItem[]>;
		onDayClick: (date: string) => void;          // empty cell space → create/open the daily note
		onOpenNote?: (entry: NoteDateEntry) => void;  // a note dot → open that note
		onOpenTask?: (task: TaskItem) => void;        // a task dot → open that task (§A.2 adds line-jump)
		onToggleTask?: (task: TaskItem) => void;      // §A.3 — check a task complete from the popover
		primarySystem?: CalendarSystem;
		secondarySystem?: CalendarSystem | 'none'; // the "second date under each day" ('none' = single calendar)
		weekStart?: 0 | 1;
		showWeekNumbers?: boolean;
		corrections?: Record<string, number>;
		calculationMode?: CalculationMode;
		chineseYearStyle?: string;
		koreanYearStyle?: string;
		monthNameStyle?: string;
	} = $props();

	let viewYear = $state(0);
	let viewMonth = $state(0);
	let enginesReady = $state(false);

	$effect(() => {
		const sys = primarySystem;
		const sec = secondarySystem;    // §A.1b — load the secondary engine too (for the second date)
		const corr = corrections;       // §C.2f — re-run when prefs change (engine is a singleton;
		const mode = calculationMode;   // applyCalendarPrefs pushes them in, then we re-anchor + re-derive).
		let cancelled = false;
		enginesReady = false;
		ensureCalendarEngines(sec === 'none' ? [sys] : [sys, sec])
			.then(() => applyCalendarPrefs(corr, mode))
			.then(() => todayInSystem(sys))
			.then((tdy) => { if (cancelled) return; viewYear = tdy.year; viewMonth = tdy.month; enginesReady = true; })
			.catch(() => { if (!cancelled) enginesReady = true; });
		return () => { cancelled = true; };
	});

	const grid = $derived.by<RichMonthGrid | null>(() => {
		void $locale;
		if (!enginesReady || !viewYear) return null;
		// §B — push the user's lunisolar year-display prefs (module state, like applyCalendarPrefs) right
		// before building; reading the props here also makes the grid re-derive when they change.
		setLunarYearStyles({ chinese: chineseYearStyle, korean: koreanYearStyle });
		setMonthNameStyle(monthNameStyle); // §B.2 — native | phonetic
		try { return buildRichMonthGrid(primarySystem, viewYear, viewMonth, $locale, weekStart, secondarySystem); }
		catch { return null; }
	});

	// Split the flat 42-cell list into 6 rows so we can show one Wk number per row.
	const rows = $derived.by(() => {
		const g = grid; if (!g) return [];
		const out: { week: number; cells: typeof g.cells }[] = [];
		for (let i = 0; i < g.cells.length; i += 7) {
			const slice = g.cells.slice(i, i + 7);
			out.push({ week: slice[0]?.weekNumber ?? 0, cells: slice });
		}
		return out;
	});

	function prevMonth() { const n = stepMonth(primarySystem, viewYear, viewMonth, -1); viewYear = n.year; viewMonth = n.month; }
	function nextMonth() { const n = stepMonth(primarySystem, viewYear, viewMonth, 1); viewYear = n.year; viewMonth = n.month; }
	async function goToToday() { const tdy = await todayInSystem(primarySystem); viewYear = tdy.year; viewMonth = tdy.month; }
	function localeCount(n: number): string { try { return n.toLocaleString($locale); } catch { return String(n); } }

	// MIG-082 §A.1 — per-cell dot partition: the daily note (gold) vs other edited notes (purple) vs tasks (red).
	function dailyNotes(iso: string): NoteDateEntry[] { return (noteEntries[iso] ?? []).filter((e) => e.is_daily); }
	function otherNotes(iso: string): NoteDateEntry[] { return (noteEntries[iso] ?? []).filter((e) => !e.is_daily); }
	function tasksFor(iso: string): TaskItem[] { return taskEntries[iso] ?? []; }

	// Single item → open directly; 2+ → a small popover anchored to the dot (FullCalendar pattern).
	let popover = $state<{ x: number; y: number; notes: NoteDateEntry[]; tasks: TaskItem[] } | null>(null);
	function anchorFor(e: MouseEvent): { x: number; y: number } {
		const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
		return { x: r.left, y: r.bottom + 4 };
	}
	function clickNotes(e: MouseEvent, list: NoteDateEntry[]) {
		e.stopPropagation();
		if (list.length === 1) { onOpenNote(list[0]); return; }
		const a = anchorFor(e); popover = { x: a.x, y: a.y, notes: list, tasks: [] };
	}
	function clickTasks(e: MouseEvent, list: TaskItem[]) {
		e.stopPropagation();
		if (list.length === 1) { onOpenTask(list[0]); return; }
		const a = anchorFor(e); popover = { x: a.x, y: a.y, notes: [], tasks: list };
	}
	function pickNote(n: NoteDateEntry) { popover = null; onOpenNote(n); }
	function pickTask(tk: TaskItem) { popover = null; onOpenTask(tk); }
	// §A.3 — check off a task from the popover: complete it, then close (a completed task drops
	// off the calendar, which the parent live-refreshes).
	function toggleTaskFromPopover(tk: TaskItem) { popover = null; onToggleTask(tk); }
</script>

<div class="cal-root" dir={$dir} data-style-target="calendar">
	{#if grid}
		<!-- Ornate header -->
		<div class="cal-header">
			<div class="cal-head-left">
				<button class="cal-today" onclick={goToToday}>{$t('calendarPanel.today')}</button>
				<button class="cal-circ" onclick={prevMonth} title={$t('calendarPanel.prevMonth')} aria-label={$t('calendarPanel.prevMonth')}>‹</button>
			</div>
			<div class="cal-head-center">
				<div class="cal-pill" class:sacred={grid.isSacred}>
					{grid.monthLabel}{#if grid.suffix}&nbsp;<span class="cal-suffix">{grid.suffix}</span>{/if}
				</div>
				{#if grid.subtitleRange}<div class="cal-greg-range">{grid.subtitleRange}</div>{/if}
			</div>
			<div class="cal-head-right">
				<button class="cal-circ" onclick={nextMonth} title={$t('calendarPanel.nextMonth')} aria-label={$t('calendarPanel.nextMonth')}>›</button>
			</div>
		</div>

		<!-- Weekday header row -->
		<div class="cal-weekrow" class:with-wk={showWeekNumbers}>
			{#if showWeekNumbers}<div class="cal-wk-head">{$t('calendarPanel.weekAbbrev') || 'Wk'}</div>{/if}
			{#each grid.weekdayLabels as wd}<div class="cal-weekday">{wd}</div>{/each}
		</div>

		<!-- Weeks -->
		<div class="cal-body">
			{#each rows as row}
				<div class="cal-row" class:with-wk={showWeekNumbers}>
					{#if showWeekNumbers}<div class="cal-wk">{localeCount(row.week)}</div>{/if}
					{#each row.cells as cell}
						{@const dailies = dailyNotes(cell.iso)}
						{@const others = otherNotes(cell.iso)}
						{@const tks = tasksFor(cell.iso)}
						{@const nc = dailies.length + others.length}
						<!-- §A.1 — gridcell: empty space + day number → daily note (bg button); each dot is its own button. -->
						<div class="cal-cell" class:other={!cell.inCurrentMonth} class:today={cell.isToday} role="gridcell">
							<button
								class="cal-cell-bg"
								onclick={() => onDayClick(cell.iso)}
								aria-label={cell.iso}
								title={[
									cell.eventName ?? '',
									cell.moonName ?? '',
									nc > 0 ? $tn('plurals.notes', nc) : '',
									tks.length > 0 ? $tn('plurals.tasksDue', tks.length) : ''
								].filter(Boolean).join(' · ')}
							></button>
							{#if cell.moonSymbol}<span class="cal-moon">{cell.moonSymbol}</span>{/if}
							<span class="cal-primary">{cell.dayLabel}</span>
							{#if cell.subLabel}<span class="cal-sub">{cell.subLabel}</span>{/if}
							<span class="cal-dots">
								{#if cell.eventType}<span class="cal-dot cal-event cal-event-{cell.eventType}" title={cell.eventName}></span>{/if}
								{#if dailies.length}<button class="cal-dot cal-note cal-daily" onclick={(e) => clickNotes(e, dailies)} title={$t('calendarPanel.dailyNote') || 'Daily note'} aria-label={$t('calendarPanel.dailyNote') || 'Daily note'}></button>{/if}
								{#if others.length}<button class="cal-dot cal-note" onclick={(e) => clickNotes(e, others)} title={$tn('plurals.notes', others.length)} aria-label={$tn('plurals.notes', others.length)}></button>{/if}
								{#if tks.length}<button class="cal-dot cal-task" onclick={(e) => clickTasks(e, tks)} title={$tn('plurals.tasksDue', tks.length)} aria-label={$tn('plurals.tasksDue', tks.length)}></button>{/if}
							</span>
						</div>
					{/each}
				</div>
			{/each}
		</div>
	{:else}
		<div class="cal-loading">{$t('common.loading') || '…'}</div>
	{/if}

	<!-- §A.1 — popover for a date with 2+ notes/tasks; pick one to open. -->
	{#if popover}
		<button class="cal-pop-backdrop" aria-label={$t('common.close') || 'Close'} onclick={() => popover = null}></button>
		<div class="cal-pop" style="left:{popover.x}px; top:{popover.y}px;" dir={$dir}>
			{#each popover.notes as n (n.file_path)}
				<button class="cal-pop-row" onclick={() => pickNote(n)}>
					<span class="cal-pop-dot cal-note" class:cal-daily={n.is_daily}></span>
					<span class="cal-pop-label" dir="auto">{n.file_name}</span>
					{#if n.is_daily}<span class="cal-pop-badge">{$t('calendarPanel.dailyNote') || 'Daily note'}</span>{/if}
				</button>
			{/each}
			{#each popover.tasks as tk (tk.file_path + ':' + tk.line_number)}
				<div class="cal-pop-row cal-pop-taskrow">
					<input type="checkbox" class="cal-pop-check" aria-label={$t('calendarPanel.completeTask') || 'Complete task'} onclick={() => toggleTaskFromPopover(tk)} />
					<button class="cal-pop-taskmain" onclick={() => pickTask(tk)}>
						<span class="cal-pop-dot cal-task"></span>
						<span class="cal-pop-label" dir="auto">{tk.text}</span>
					</button>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	/* §C.2d — Style-Setter tokens are consumed as `var(--cal-X, default)` inline fallbacks (NOT
	   declared on .cal-root), so a body-level styleOverride (or the Setter preview's draft wrapper)
	   INHERITS down and wins — a local declaration would block it. Defaults = Eisa's app palette.
	   Layout-only --cal-wk-col stays local (it has its own Wk toggle, not a theme colour). */
	.cal-root {
		--cal-wk-col: 44px;
		width: 100%;
		max-width: 1100px;
		display: flex;
		flex-direction: column;
		font-family: var(--cal-font, 'Amiri', 'Cairo', var(--text-font, inherit));
	}

	/* Header */
	.cal-header {
		display: flex; align-items: center; justify-content: space-between;
		gap: 12px; padding: 14px 18px;
		background: linear-gradient(135deg, var(--cal-header-from, #14553f), var(--cal-header-to, #1a6b4f));
		color: #fff; border-radius: 12px 12px 0 0;
	}
	.cal-head-left, .cal-head-right { display: flex; align-items: center; gap: 10px; }
	.cal-head-center { display: flex; flex-direction: column; align-items: center; gap: 4px; flex: 1; }
	.cal-today {
		font: inherit; font-size: var(--cal-today-size, 0.85rem); color: #fff;
		background: rgba(255, 255, 255, 0.12); border: 1px solid rgba(255, 255, 255, 0.25);
		padding: 6px 14px; border-radius: 999px; cursor: pointer;
	}
	.cal-today:hover { background: rgba(255, 255, 255, 0.22); }
	.cal-circ {
		width: 40px; height: 40px; border-radius: 50%;
		display: flex; align-items: center; justify-content: center;
		background: rgba(255, 255, 255, 0.12); border: 1px solid rgba(255, 255, 255, 0.25);
		color: #fff; font-size: var(--cal-nav-size, 1.4rem); cursor: pointer; line-height: 1;
	}
	.cal-circ:hover { background: rgba(255, 255, 255, 0.22); }
	.cal-pill {
		font-size: var(--cal-pill-size, 1.5rem); font-weight: 700; padding: 6px 22px; border-radius: 14px;
		background: var(--cal-pill-bg, #d4a017); color: var(--cal-pill-text, #ffffff);
		border: 2px solid var(--cal-pill-border, #c49440); white-space: nowrap;
	}
	.cal-pill.sacred {
		background: linear-gradient(to bottom, var(--cal-pill-sacred-from, #f5e6c8), var(--cal-pill-sacred-to, #eedbb5));
		color: var(--cal-pill-sacred-text, #6b4400);
	}
	.cal-suffix { font-size: 0.75em; opacity: 0.85; }
	.cal-greg-range { font-size: var(--cal-subtitle-size, 0.82rem); opacity: 0.9; }

	/* Weekday header */
	.cal-weekrow {
		display: grid; grid-template-columns: repeat(7, 1fr);
		background: color-mix(in srgb, var(--cal-header-to, #1a6b4f) 8%, transparent);
	}
	.cal-weekrow.with-wk { grid-template-columns: var(--cal-wk-col) repeat(7, 1fr); }
	.cal-wk-head, .cal-weekday {
		text-align: center; padding: 8px 0; font-size: var(--cal-weekday-size, 0.78rem); font-weight: 600;
		color: var(--cal-weekday-color, #1a6b4f);
	}
	.cal-wk-head { color: var(--cal-wk-color, var(--text-faint, #94a3b8)); font-size: var(--cal-week-size, 0.7rem); }

	/* Grid body */
	.cal-body { display: flex; flex-direction: column; border: 1px solid var(--cal-grid-border, var(--border, #e2e8f0)); border-top: none; border-radius: 0 0 12px 12px; overflow: hidden; }
	.cal-row { display: grid; grid-template-columns: repeat(7, 1fr); }
	.cal-row.with-wk { grid-template-columns: var(--cal-wk-col) repeat(7, 1fr); }
	.cal-wk {
		display: flex; align-items: center; justify-content: center;
		font-size: var(--cal-week-size, 0.72rem); font-weight: 600; color: var(--cal-wk-color, var(--text-faint, #94a3b8));
		border-top: 1px solid var(--cal-grid-border, var(--border, #e2e8f0));
		background: color-mix(in srgb, var(--cal-grid-border, var(--border, #e2e8f0)) 25%, transparent);
	}
	.cal-cell {
		position: relative; min-height: 76px;
		display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 2px;
		background: var(--cal-cell-bg, var(--bg-primary, #fff)); color: var(--cal-primary-color, var(--text, #0d3b2e));
		border-top: 1px solid var(--cal-grid-border, var(--border, #e2e8f0)); border-inline-start: 1px solid var(--cal-grid-border, var(--border, #e2e8f0));
		padding: 6px;
	}
	.cal-cell:hover { background: color-mix(in srgb, var(--cal-header-to, #1a6b4f) 6%, var(--cal-cell-bg, var(--bg-primary, #fff))); }
	.cal-cell.other { opacity: 0.4; }
	.cal-cell.today {
		background: linear-gradient(135deg, var(--cal-today-from, #b8860b), var(--cal-today-to, #d4a017));
		color: var(--cal-today-text, #ffffff);
	}
	.cal-cell.today .cal-sub, .cal-cell.today .cal-moon { color: rgba(255, 255, 255, 0.85); }
	/* §A.1 — full-bleed click target = empty space + day number → daily note; sits BEHIND the dots. */
	.cal-cell-bg { position: absolute; inset: 0; z-index: 0; background: transparent; border: none; cursor: pointer; }
	/* Day number / sub-date / moon are display-only — clicks fall through to the bg button. */
	.cal-primary { position: relative; z-index: 1; pointer-events: none; font-size: var(--cal-day-size, 1.2rem); font-weight: 700; line-height: 1; }
	.cal-sub { position: relative; z-index: 1; pointer-events: none; font-size: var(--cal-subdate-size, 0.68rem); color: var(--cal-sub-color, #0e7490); line-height: 1; }
	.cal-moon { position: absolute; top: 3px; inset-inline-end: 5px; z-index: 1; pointer-events: none; font-size: var(--cal-moon-size, 0.72rem); color: var(--cal-moon-color, var(--text-faint, #6b7280)); }
	/* Dots: the row ignores pointer events; each dot button is its own ≥14px hit target. */
	.cal-dots { position: absolute; bottom: 4px; z-index: 2; display: flex; gap: 3px; pointer-events: none; }
	.cal-dot { width: var(--cal-dot-size, 6px); height: var(--cal-dot-size, 6px); border-radius: 50%; }
	button.cal-dot { box-sizing: content-box; padding: 4px; border: none; background-clip: content-box; cursor: pointer; pointer-events: auto; transition: transform 0.1s; }
	button.cal-dot:hover { transform: scale(1.25); }
	.cal-note { background: var(--cal-note-dot, #7c3aed); }
	.cal-daily { background: var(--cal-daily-dot, #d4a017); }
	.cal-task { background: var(--cal-task-dot, #ef4444); }
	.cal-event-holiday { background: var(--cal-event-holiday, #ef4444); }
	.cal-event-observance { background: var(--cal-event-observance, #d4a017); }
	.cal-event-special { background: var(--cal-event-special, #8b5cf6); }
	.cal-loading { text-align: center; color: var(--text-faint, #888); font-size: 0.85rem; padding: 40px 0; }
	/* §A.1 — multi-item popover (a date with 2+ notes/tasks) */
	.cal-pop-backdrop { position: fixed; inset: 0; z-index: 1000; background: transparent; border: none; padding: 0; margin: 0; cursor: default; }
	.cal-pop {
		position: fixed; z-index: 1001; min-width: 180px; max-width: 340px; max-height: 50vh; overflow-y: auto;
		background: var(--bg-secondary, #fff); border: 1px solid var(--cal-grid-border, var(--border, #e2e8f0));
		border-radius: 8px; box-shadow: 0 6px 24px rgba(0, 0, 0, 0.18); padding: 4px;
		display: flex; flex-direction: column; gap: 2px;
		font-family: var(--cal-font, 'Amiri', 'Cairo', var(--text-font, inherit));
	}
	.cal-pop-row { display: flex; align-items: center; gap: 8px; padding: 7px 9px; border: none; background: transparent; border-radius: 6px; cursor: pointer; text-align: start; font: inherit; color: var(--text, #1e293b); }
	.cal-pop-row:hover { background: color-mix(in srgb, var(--cal-header-to, #1a6b4f) 8%, transparent); }
	/* §A.3 — task rows pair a complete-checkbox with the open-button */
	.cal-pop-taskrow { cursor: default; }
	.cal-pop-check { flex: none; width: 15px; height: 15px; cursor: pointer; accent-color: var(--cal-task-dot, #ef4444); }
	.cal-pop-taskmain { display: flex; align-items: center; gap: 8px; flex: 1; min-width: 0; border: none; background: transparent; padding: 0; cursor: pointer; text-align: start; font: inherit; color: inherit; }
	.cal-pop-dot { flex: none; width: 8px; height: 8px; border-radius: 50%; }
	.cal-pop-label { flex: 1; font-size: 0.85rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.cal-pop-badge { flex: none; font-size: 0.65rem; color: var(--cal-daily-dot, #d4a017); border: 1px solid var(--cal-daily-dot, #d4a017); border-radius: 4px; padding: 1px 5px; }
</style>
